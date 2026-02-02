//! SSH 远程 Shell 进程实现
//!
//! 封装 SSH 远程 PTY 进程，提供 SSHShellProc 结构体供 ShellController 使用。
//!
//! ## 功能
//! - 创建和管理远程 PTY 会话
//! - 异步读取远程 PTY 输出并通过 Tauri Event 推送
//! - 处理远程 PTY 输入写入
//! - 监控远程进程退出状态
//! - 终端大小同步
//!
//! ## Requirements
//! - 4.2: SSH 连接建立成功时创建远程 PTY 会话
//! - 4.7: 支持 ProxyJump 配置
//! - 4.11: 用户调整终端大小时同步调整远程 PTY 大小

use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Arc;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use parking_lot::Mutex;
use ssh2::{Channel, Session};
use tauri::Emitter;
use tokio::sync::mpsc;

use crate::terminal::block_controller::{BlockInputUnion, BlockMeta, TermSize};
use crate::terminal::error::TerminalError;
use crate::terminal::events::{
    event_names, SessionStatus, TerminalOutputEvent, TerminalStatusEvent,
};
use crate::terminal::persistence::BlockFile;

use super::ssh_connection::SSHConn;

/// SSH Shell 进程封装
///
/// 封装 SSH 远程 PTY 进程，提供输入输出和生命周期管理。
///
/// _Requirements: 4.2, 4.11_
pub struct SSHShellProc {
    /// Block ID
    block_id: String,
    /// 控制器类型
    controller_type: String,
    /// SSH Channel（用于数据传输）
    channel: Arc<Mutex<Channel>>,
    /// 关闭标志
    shutdown_flag: Arc<AtomicBool>,
    /// 进程退出码
    exit_code: Arc<AtomicI32>,
    /// 是否已退出
    exited: Arc<AtomicBool>,
    /// 当前终端大小
    term_size: Arc<Mutex<TermSize>>,
}

impl SSHShellProc {
    /// 创建新的 SSH Shell 进程
    ///
    /// # 参数
    /// - `block_id`: Block ID
    /// - `controller_type`: 控制器类型 ("shell" | "cmd")
    /// - `session`: SSH 会话
    /// - `rows`: 终端行数
    /// - `cols`: 终端列数
    /// - `app_handle`: Tauri 应用句柄
    /// - `block_meta`: 块元数据配置
    /// - `input_rx`: 输入接收器
    /// - `block_file`: 块文件存储（可选）
    ///
    /// # 返回
    /// - `Ok(SSHShellProc)`: 创建成功
    /// - `Err(TerminalError)`: 创建失败
    ///
    /// _Requirements: 4.2_
    pub async fn new(
        block_id: String,
        controller_type: String,
        session: &Session,
        rows: u16,
        cols: u16,
        app_handle: tauri::AppHandle,
        block_meta: BlockMeta,
        input_rx: mpsc::Receiver<BlockInputUnion>,
        block_file: Option<Arc<BlockFile>>,
    ) -> Result<Self, TerminalError> {
        tracing::info!(
            "[SSHShellProc] 创建远程进程: block_id={}, type={}, size={}x{}",
            block_id,
            controller_type,
            cols,
            rows
        );

        // 创建 SSH Channel
        let mut channel = session.channel_session().map_err(|e| {
            TerminalError::SSHConnectionFailed(format!("创建 SSH Channel 失败: {e}"))
        })?;

        // 请求 PTY
        // 使用 xterm-256color 终端类型
        channel
            .request_pty(
                "xterm-256color",
                None,
                Some((cols as u32, rows as u32, 0, 0)),
            )
            .map_err(|e| TerminalError::SSHConnectionFailed(format!("请求远程 PTY 失败: {e}")))?;

        // 根据控制器类型启动 Shell 或执行命令
        if controller_type == "cmd" {
            // 命令执行模式
            let cmd = Self::build_remote_command(&block_meta)?;
            tracing::info!("[SSHShellProc] 执行远程命令: {}", cmd);
            channel.exec(&cmd).map_err(|e| {
                TerminalError::SSHConnectionFailed(format!("执行远程命令失败: {e}"))
            })?;
        } else {
            // Shell 模式 - 启动交互式 Shell
            channel.shell().map_err(|e| {
                TerminalError::SSHConnectionFailed(format!("启动远程 Shell 失败: {e}"))
            })?;
        }

        // 设置非阻塞模式
        session.set_blocking(false);

        // 创建共享状态
        let shutdown_flag = Arc::new(AtomicBool::new(false));
        let exit_code = Arc::new(AtomicI32::new(0));
        let exited = Arc::new(AtomicBool::new(false));
        let channel = Arc::new(Mutex::new(channel));
        let term_size = Arc::new(Mutex::new(TermSize { rows, cols }));

        // 启动输出读取任务
        Self::spawn_output_reader(
            block_id.clone(),
            channel.clone(),
            app_handle.clone(),
            shutdown_flag.clone(),
            exit_code.clone(),
            exited.clone(),
            block_file,
        );

        // 启动输入处理任务
        Self::spawn_input_handler(
            block_id.clone(),
            channel.clone(),
            term_size.clone(),
            input_rx,
            shutdown_flag.clone(),
        );

        tracing::info!("[SSHShellProc] 远程进程已创建: block_id={}", block_id);

        Ok(Self {
            block_id,
            controller_type,
            channel,
            shutdown_flag,
            exit_code,
            exited,
            term_size,
        })
    }

    /// 从 SSHConn 创建 SSH Shell 进程
    ///
    /// 便捷方法，从已连接的 SSHConn 创建远程 Shell 进程。
    ///
    /// # 参数
    /// - `block_id`: Block ID
    /// - `controller_type`: 控制器类型
    /// - `ssh_conn`: SSH 连接管理器
    /// - `rows`: 终端行数
    /// - `cols`: 终端列数
    /// - `app_handle`: Tauri 应用句柄
    /// - `block_meta`: 块元数据配置
    /// - `input_rx`: 输入接收器
    /// - `block_file`: 块文件存储（可选）
    ///
    /// # 返回
    /// - `Ok(SSHShellProc)`: 创建成功
    /// - `Err(TerminalError)`: 创建失败
    pub async fn from_ssh_conn(
        block_id: String,
        controller_type: String,
        ssh_conn: &SSHConn,
        rows: u16,
        cols: u16,
        app_handle: tauri::AppHandle,
        block_meta: BlockMeta,
        input_rx: mpsc::Receiver<BlockInputUnion>,
        block_file: Option<Arc<BlockFile>>,
    ) -> Result<Self, TerminalError> {
        let session = ssh_conn
            .get_session()
            .ok_or_else(|| TerminalError::SSHConnectionFailed("SSH 会话未建立".to_string()))?;

        Self::new(
            block_id,
            controller_type,
            &session,
            rows,
            cols,
            app_handle,
            block_meta,
            input_rx,
            block_file,
        )
        .await
    }

    /// 构建远程命令
    ///
    /// 根据块元数据构建要在远程执行的命令。
    ///
    /// _Requirements: 16.1, 16.2, 16.3_
    fn build_remote_command(block_meta: &BlockMeta) -> Result<String, TerminalError> {
        let cmd_str = block_meta.cmd.as_ref().ok_or_else(|| {
            TerminalError::SSHConnectionFailed("cmd 模式需要指定命令".to_string())
        })?;

        // 构建完整命令字符串
        let mut full_cmd = String::new();

        // 如果指定了工作目录，先 cd 到该目录
        if let Some(cwd) = &block_meta.cmd_cwd {
            full_cmd.push_str(&format!("cd {} && ", shell_escape(cwd)));
        }

        // 设置环境变量
        if let Some(env_vars) = &block_meta.cmd_env {
            for (key, value) in env_vars {
                full_cmd.push_str(&format!("export {}={} && ", key, shell_escape(value)));
            }
        }

        // 添加命令和参数
        full_cmd.push_str(cmd_str);
        if let Some(args) = &block_meta.cmd_args {
            for arg in args {
                full_cmd.push(' ');
                full_cmd.push_str(&shell_escape(arg));
            }
        }

        Ok(full_cmd)
    }

    /// 启动输出读取任务
    ///
    /// 在独立线程中读取远程 PTY 输出，并通过 Tauri 事件发送到前端。
    ///
    /// _Requirements: 4.2_
    fn spawn_output_reader(
        block_id: String,
        channel: Arc<Mutex<Channel>>,
        app_handle: tauri::AppHandle,
        shutdown_flag: Arc<AtomicBool>,
        exit_code: Arc<AtomicI32>,
        exited: Arc<AtomicBool>,
        block_file: Option<Arc<BlockFile>>,
    ) {
        std::thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            let mut consecutive_empty_reads = 0;
            const MAX_EMPTY_READS: u32 = 100; // 防止空循环

            loop {
                // 检查关闭标志
                if shutdown_flag.load(Ordering::Relaxed) {
                    tracing::debug!("[SSHShellProc] 收到关闭信号: block_id={}", block_id);
                    break;
                }

                // 读取输出
                let read_result = {
                    let mut ch = channel.lock();

                    // 检查 Channel 是否已关闭
                    if ch.eof() {
                        // 获取退出状态
                        let code = ch.exit_status().unwrap_or(0);
                        exit_code.store(code, Ordering::SeqCst);
                        exited.store(true, Ordering::SeqCst);

                        tracing::info!(
                            "[SSHShellProc] 远程进程已退出: block_id={}, exit_code={}",
                            block_id,
                            code
                        );

                        // 发送状态事件
                        let _ = app_handle.emit(
                            event_names::TERMINAL_STATUS,
                            TerminalStatusEvent {
                                session_id: block_id.clone(),
                                status: SessionStatus::Done,
                                exit_code: Some(code),
                                error: None,
                            },
                        );
                        break;
                    }

                    ch.read(&mut buffer)
                };

                match read_result {
                    Ok(0) => {
                        // 没有数据可读，短暂休眠后重试
                        consecutive_empty_reads += 1;
                        if consecutive_empty_reads > MAX_EMPTY_READS {
                            // 检查 Channel 状态
                            let ch = channel.lock();
                            if ch.eof() {
                                let code = ch.exit_status().unwrap_or(0);
                                exit_code.store(code, Ordering::SeqCst);
                                exited.store(true, Ordering::SeqCst);

                                let _ = app_handle.emit(
                                    event_names::TERMINAL_STATUS,
                                    TerminalStatusEvent {
                                        session_id: block_id.clone(),
                                        status: SessionStatus::Done,
                                        exit_code: Some(code),
                                        error: None,
                                    },
                                );
                                break;
                            }
                            consecutive_empty_reads = 0;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                    Ok(n) => {
                        consecutive_empty_reads = 0;
                        let output_data = &buffer[..n];

                        // 保存到块文件
                        if let Some(ref bf) = block_file {
                            if let Err(e) = bf.append_data(output_data) {
                                tracing::warn!(
                                    "[SSHShellProc] 写入块文件失败: block_id={}, error={}",
                                    block_id,
                                    e
                                );
                            }
                        }

                        // 发送输出事件
                        let data = BASE64.encode(output_data);
                        let _ = app_handle.emit(
                            event_names::TERMINAL_OUTPUT,
                            TerminalOutputEvent {
                                session_id: block_id.clone(),
                                data,
                            },
                        );
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // 非阻塞模式下没有数据，短暂休眠
                        consecutive_empty_reads += 1;
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                    Err(e) => {
                        // 检查是否是因为关闭导致的错误
                        if shutdown_flag.load(Ordering::Relaxed) {
                            break;
                        }

                        tracing::error!(
                            "[SSHShellProc] 读取错误: block_id={}, error={}",
                            block_id,
                            e
                        );
                        exited.store(true, Ordering::SeqCst);

                        let _ = app_handle.emit(
                            event_names::TERMINAL_STATUS,
                            TerminalStatusEvent {
                                session_id: block_id.clone(),
                                status: SessionStatus::Error,
                                exit_code: None,
                                error: Some(e.to_string()),
                            },
                        );
                        break;
                    }
                }
            }
        });
    }

    /// 启动输入处理任务
    ///
    /// 在独立任务中处理输入数据，包括键盘输入、信号和终端大小调整。
    ///
    /// _Requirements: 4.11_
    fn spawn_input_handler(
        block_id: String,
        channel: Arc<Mutex<Channel>>,
        term_size: Arc<Mutex<TermSize>>,
        mut input_rx: mpsc::Receiver<BlockInputUnion>,
        shutdown_flag: Arc<AtomicBool>,
    ) {
        tokio::spawn(async move {
            while let Some(input) = input_rx.recv().await {
                // 检查关闭标志
                if shutdown_flag.load(Ordering::Relaxed) {
                    break;
                }

                // 处理输入数据
                if let Some(data) = &input.input_data {
                    let mut ch = channel.lock();
                    if let Err(e) = ch.write_all(data) {
                        tracing::error!(
                            "[SSHShellProc] 写入失败: block_id={}, error={}",
                            block_id,
                            e
                        );
                        continue;
                    }
                    if let Err(e) = ch.flush() {
                        tracing::error!(
                            "[SSHShellProc] Flush 失败: block_id={}, error={}",
                            block_id,
                            e
                        );
                    }
                }

                // 处理终端大小调整
                if let Some(size) = &input.term_size {
                    // 更新本地记录的终端大小
                    {
                        let mut ts = term_size.lock();
                        *ts = *size;
                    }

                    // 发送 PTY 大小调整请求到远程
                    let mut ch = channel.lock();
                    if let Err(e) =
                        ch.request_pty_size(size.cols as u32, size.rows as u32, Some(0), Some(0))
                    {
                        tracing::error!(
                            "[SSHShellProc] 调整远程 PTY 大小失败: block_id={}, error={}",
                            block_id,
                            e
                        );
                    } else {
                        tracing::debug!(
                            "[SSHShellProc] 调整远程 PTY 大小: block_id={}, size={}x{}",
                            block_id,
                            size.cols,
                            size.rows
                        );
                    }
                }

                // 处理信号
                if let Some(sig_name) = &input.sig_name {
                    tracing::debug!(
                        "[SSHShellProc] 收到信号: block_id={}, signal={}",
                        block_id,
                        sig_name
                    );

                    // SSH 协议支持发送信号，但 ssh2 crate 没有直接暴露此功能
                    // 对于 SIGINT，我们可以发送 Ctrl+C (0x03)
                    // 对于其他信号，记录日志但不执行操作
                    match sig_name.as_str() {
                        "SIGINT" => {
                            // 发送 Ctrl+C
                            let mut ch = channel.lock();
                            if let Err(e) = ch.write_all(&[0x03]) {
                                tracing::warn!(
                                    "[SSHShellProc] 发送 Ctrl+C 失败: block_id={}, error={}",
                                    block_id,
                                    e
                                );
                            }
                        }
                        "SIGQUIT" => {
                            // 发送 Ctrl+\ (0x1C)
                            let mut ch = channel.lock();
                            if let Err(e) = ch.write_all(&[0x1C]) {
                                tracing::warn!(
                                    "[SSHShellProc] 发送 Ctrl+\\ 失败: block_id={}, error={}",
                                    block_id,
                                    e
                                );
                            }
                        }
                        _ => {
                            tracing::warn!(
                                "[SSHShellProc] 不支持的信号: block_id={}, signal={}",
                                block_id,
                                sig_name
                            );
                        }
                    }
                }
            }

            tracing::debug!("[SSHShellProc] 输入处理任务结束: block_id={}", block_id);
        });
    }

    /// 获取 Block ID
    pub fn block_id(&self) -> &str {
        &self.block_id
    }

    /// 获取控制器类型
    pub fn controller_type(&self) -> &str {
        &self.controller_type
    }

    /// 检查进程是否已退出
    pub fn is_exited(&self) -> bool {
        self.exited.load(Ordering::SeqCst)
    }

    /// 获取退出码
    pub fn get_exit_code(&self) -> i32 {
        self.exit_code.load(Ordering::SeqCst)
    }

    /// 获取当前终端大小
    pub fn get_term_size(&self) -> TermSize {
        *self.term_size.lock()
    }

    /// 写入数据到远程 PTY
    ///
    /// _Requirements: 4.2_
    pub fn write(&self, data: &[u8]) -> Result<(), TerminalError> {
        let mut channel = self.channel.lock();
        channel
            .write_all(data)
            .map_err(|e| TerminalError::WriteFailed(e.to_string()))?;
        channel
            .flush()
            .map_err(|e| TerminalError::WriteFailed(e.to_string()))?;
        Ok(())
    }

    /// 调整远程 PTY 大小
    ///
    /// _Requirements: 4.11_
    pub fn resize(&self, rows: u16, cols: u16) -> Result<(), TerminalError> {
        // 更新本地记录
        {
            let mut ts = self.term_size.lock();
            *ts = TermSize { rows, cols };
        }

        // 发送到远程
        let mut channel = self.channel.lock();
        channel
            .request_pty_size(cols as u32, rows as u32, Some(0), Some(0))
            .map_err(|e| TerminalError::ResizeFailed(e.to_string()))?;

        tracing::debug!(
            "[SSHShellProc] 调整远程 PTY 大小: block_id={}, size={}x{}",
            self.block_id,
            cols,
            rows
        );
        Ok(())
    }

    /// 优雅终止进程
    ///
    /// 发送 EOF 并等待进程退出。
    ///
    /// # 返回
    /// 进程退出码
    pub async fn terminate(&self) -> i32 {
        tracing::info!(
            "[SSHShellProc] 优雅终止远程进程: block_id={}",
            self.block_id
        );

        // 设置关闭标志
        self.shutdown_flag.store(true, Ordering::SeqCst);

        // 发送 Ctrl+C 尝试中断进程
        {
            let mut channel = self.channel.lock();
            let _ = channel.write_all(&[0x03]); // Ctrl+C
        }

        // 等待一小段时间让进程响应
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // 关闭 Channel
        {
            let mut channel = self.channel.lock();
            let _ = channel.send_eof();
            let _ = channel.wait_close();
        }

        self.exit_code.load(Ordering::SeqCst)
    }

    /// 强制终止进程
    ///
    /// 关闭 Channel 立即终止连接。
    pub async fn kill(&self) {
        tracing::info!(
            "[SSHShellProc] 强制终止远程进程: block_id={}",
            self.block_id
        );

        // 设置关闭标志
        self.shutdown_flag.store(true, Ordering::SeqCst);

        // 直接关闭 Channel
        {
            let mut channel = self.channel.lock();
            let _ = channel.send_eof();
            let _ = channel.close();
        }
    }

    /// 发送 EOF 到远程
    ///
    /// 用于通知远程进程输入已结束。
    pub fn send_eof(&self) -> Result<(), TerminalError> {
        let mut channel = self.channel.lock();
        channel
            .send_eof()
            .map_err(|e| TerminalError::WriteFailed(format!("发送 EOF 失败: {e}")))?;
        Ok(())
    }
}

impl Drop for SSHShellProc {
    fn drop(&mut self) {
        // 确保关闭标志被设置
        self.shutdown_flag.store(true, Ordering::SeqCst);

        // 尝试关闭 Channel
        if let Some(mut channel) = self.channel.try_lock() {
            let _ = channel.send_eof();
            let _ = channel.close();
        }

        tracing::debug!("[SSHShellProc] 远程进程已销毁: block_id={}", self.block_id);
    }
}

/// Shell 转义辅助函数
///
/// 对字符串进行 Shell 转义，防止命令注入。
fn shell_escape(s: &str) -> String {
    // 如果字符串只包含安全字符，直接返回
    if s.chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '/')
    {
        return s.to_string();
    }

    // 否则用单引号包裹，并转义内部的单引号
    format!("'{}'", s.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_escape_simple() {
        assert_eq!(shell_escape("hello"), "hello");
        assert_eq!(shell_escape("hello_world"), "hello_world");
        assert_eq!(shell_escape("/path/to/file"), "/path/to/file");
        assert_eq!(shell_escape("file.txt"), "file.txt");
    }

    #[test]
    fn test_shell_escape_special_chars() {
        assert_eq!(shell_escape("hello world"), "'hello world'");
        assert_eq!(shell_escape("hello'world"), "'hello'\\''world'");
        assert_eq!(shell_escape("$HOME"), "'$HOME'");
        assert_eq!(shell_escape("a;b"), "'a;b'");
    }

    #[test]
    fn test_term_size_default() {
        let size = TermSize::default();
        assert_eq!(size.rows, 24);
        assert_eq!(size.cols, 80);
    }
}
