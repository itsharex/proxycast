//! 本地 PTY 连接实现
//!
//! 封装本地 PTY 进程，提供 ShellProc 结构体供 ShellController 使用。
//!
//! ## 功能
//! - 创建和管理本地 PTY 子进程
//! - 异步读取 PTY 输出并通过 Tauri Event 推送
//! - 处理 PTY 输入写入
//! - 监控进程退出状态
//! - 支持命令执行模式（cmd）
//! - 支持 Shell 集成脚本加载
//!
//! ## Requirements
//! - 17.1: 管理 Shell 进程的完整生命周期
//! - 17.2: 设置正确的环境变量
//! - 17.3: 支持优雅终止和强制终止
//! - 17.4: 记录退出码并更新状态
//! - 17.5: 支持自定义 Shell 路径和参数
//! - 17.8: zsh 使用 ZDOTDIR 指向集成目录
//! - 17.9: bash 使用 --rcfile 加载集成脚本
//! - 17.10: fish 使用 -C 参数 source 集成脚本

use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Arc;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use parking_lot::Mutex;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use tauri::Emitter;
use tauri::Manager;
use tokio::sync::mpsc;

use crate::terminal::block_controller::{BlockInputUnion, BlockMeta};
use crate::terminal::error::TerminalError;
use crate::terminal::events::{
    event_names, SessionStatus, TerminalOutputEvent, TerminalStatusEvent,
};
use crate::terminal::integration::{ShellLaunchBuilder, ShellType};
use crate::terminal::persistence::BlockFile;

/// Shell 进程封装
///
/// 封装 PTY 进程，提供输入输出和生命周期管理。
pub struct ShellProc {
    /// Block ID
    block_id: String,
    /// 控制器类型
    controller_type: String,
    /// PTY 写入器
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    /// PTY Master（用于调整大小）
    master: Arc<Mutex<Box<dyn portable_pty::MasterPty + Send>>>,
    /// 关闭标志
    shutdown_flag: Arc<AtomicBool>,
    /// 进程退出码
    exit_code: Arc<AtomicI32>,
    /// 是否已退出
    exited: Arc<AtomicBool>,
}

impl ShellProc {
    /// 创建新的 Shell 进程
    ///
    /// # 参数
    /// - `block_id`: Block ID
    /// - `controller_type`: 控制器类型 ("shell" | "cmd")
    /// - `rows`: 终端行数
    /// - `cols`: 终端列数
    /// - `app_handle`: Tauri 应用句柄
    /// - `block_meta`: 块元数据配置
    /// - `input_rx`: 输入接收器
    /// - `block_file`: 块文件存储（可选）
    ///
    /// # 返回
    /// - `Ok(ShellProc)`: 创建成功
    /// - `Err(TerminalError)`: 创建失败
    ///
    /// _Requirements: 17.1, 17.2, 17.8, 17.9, 17.10_
    pub async fn new(
        block_id: String,
        controller_type: String,
        rows: u16,
        cols: u16,
        app_handle: tauri::AppHandle,
        block_meta: BlockMeta,
        input_rx: mpsc::Receiver<BlockInputUnion>,
        block_file: Option<Arc<BlockFile>>,
    ) -> Result<Self, TerminalError> {
        tracing::info!(
            "[ShellProc] 创建进程: block_id={}, type={}, size={}x{}",
            block_id,
            controller_type,
            cols,
            rows
        );

        let pty_system = native_pty_system();

        // 创建 PTY
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| TerminalError::PtyCreationFailed(e.to_string()))?;

        // 构建命令（传递 app_handle 和 block_id 用于 Shell 集成）
        let cmd = Self::build_command(&controller_type, &block_meta, &app_handle, &block_id)?;

        // 启动子进程
        let _child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| TerminalError::PtyCreationFailed(e.to_string()))?;

        // 获取写入器
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| TerminalError::PtyCreationFailed(e.to_string()))?;

        // 获取读取器
        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| TerminalError::PtyCreationFailed(e.to_string()))?;

        // 创建共享状态
        let shutdown_flag = Arc::new(AtomicBool::new(false));
        let exit_code = Arc::new(AtomicI32::new(0));
        let exited = Arc::new(AtomicBool::new(false));
        let writer = Arc::new(Mutex::new(writer));
        let master = Arc::new(Mutex::new(pair.master));

        // 启动输出读取任务
        Self::spawn_output_reader(
            block_id.clone(),
            reader,
            app_handle.clone(),
            shutdown_flag.clone(),
            exit_code.clone(),
            exited.clone(),
            block_file,
        );

        // 启动输入处理任务
        Self::spawn_input_handler(
            block_id.clone(),
            writer.clone(),
            master.clone(),
            input_rx,
            shutdown_flag.clone(),
        );

        tracing::info!("[ShellProc] 进程已创建: block_id={}", block_id);

        Ok(Self {
            block_id,
            controller_type,
            writer,
            master,
            shutdown_flag,
            exit_code,
            exited,
        })
    }

    /// 构建命令
    ///
    /// 根据控制器类型和块元数据构建要执行的命令。
    ///
    /// # 参数
    /// - `controller_type`: 控制器类型
    /// - `block_meta`: 块元数据
    /// - `app_handle`: Tauri 应用句柄
    /// - `block_id`: Block ID
    ///
    /// # 返回
    /// 构建好的命令
    ///
    /// _Requirements: 17.2, 17.5, 17.8, 17.9, 17.10_
    fn build_command(
        controller_type: &str,
        block_meta: &BlockMeta,
        app_handle: &tauri::AppHandle,
        block_id: &str,
    ) -> Result<CommandBuilder, TerminalError> {
        let mut cmd = if controller_type == "cmd" {
            // 命令执行模式
            Self::build_cmd_command(block_meta)?
        } else {
            // Shell 模式 - 使用集成脚本
            Self::build_shell_command(block_meta, app_handle, block_id)?
        };

        // 设置工作目录
        if let Some(cwd) = &block_meta.cmd_cwd {
            cmd.cwd(cwd);
        } else if let Some(home) = dirs::home_dir() {
            cmd.cwd(home);
        }

        Ok(cmd)
    }

    /// 构建 Shell 命令
    ///
    /// 使用 Shell 集成脚本构建启动命令。
    ///
    /// _Requirements: 17.5, 17.7, 17.8, 17.9, 17.10_
    fn build_shell_command(
        block_meta: &BlockMeta,
        app_handle: &tauri::AppHandle,
        block_id: &str,
    ) -> Result<CommandBuilder, TerminalError> {
        // 获取用户默认 shell
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        tracing::info!("[ShellProc] 使用 shell: {}", shell);

        // 获取应用数据目录
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| TerminalError::Internal(format!("获取应用数据目录失败: {e}")))?;

        // 使用 ShellLaunchBuilder 构建启动配置
        let builder = ShellLaunchBuilder::new(&app_data_dir, block_id.to_string());
        let launch_config = builder.build(&shell, block_meta.cmd_env.as_ref())?;

        // 构建命令
        let mut cmd = CommandBuilder::new(&launch_config.shell_path);

        // 添加参数
        for arg in &launch_config.args {
            cmd.arg(arg);
        }

        // 设置环境变量
        for (key, value) in &launch_config.env {
            cmd.env(key, value);
        }

        // 检测 Shell 类型并记录
        let shell_type = ShellType::from_path(&shell);
        tracing::info!(
            "[ShellProc] Shell 类型: {:?}, 参数: {:?}",
            shell_type,
            launch_config.args
        );

        Ok(cmd)
    }

    /// 构建命令执行命令
    ///
    /// _Requirements: 16.1, 16.2, 16.3, 17.2_
    fn build_cmd_command(block_meta: &BlockMeta) -> Result<CommandBuilder, TerminalError> {
        let cmd_str = block_meta
            .cmd
            .as_ref()
            .ok_or_else(|| TerminalError::PtyCreationFailed("cmd 模式需要指定命令".to_string()))?;

        tracing::info!("[ShellProc] 执行命令: {}", cmd_str);

        // 使用 shell 执行命令
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        let mut cmd = CommandBuilder::new(&shell);
        cmd.arg("-c");

        // 构建完整命令字符串
        let full_cmd = if let Some(args) = &block_meta.cmd_args {
            format!("{} {}", cmd_str, args.join(" "))
        } else {
            cmd_str.clone()
        };

        cmd.arg(&full_cmd);

        // 设置通用环境变量
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");

        // 设置自定义环境变量
        if let Some(env_vars) = &block_meta.cmd_env {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        Ok(cmd)
    }

    /// 启动输出读取任务
    ///
    /// 在独立线程中读取 PTY 输出，并通过 Tauri 事件发送到前端。
    fn spawn_output_reader(
        block_id: String,
        mut reader: Box<dyn Read + Send>,
        app_handle: tauri::AppHandle,
        shutdown_flag: Arc<AtomicBool>,
        exit_code: Arc<AtomicI32>,
        exited: Arc<AtomicBool>,
        block_file: Option<Arc<BlockFile>>,
    ) {
        std::thread::spawn(move || {
            let mut buffer = [0u8; 4096];

            loop {
                // 检查关闭标志
                if shutdown_flag.load(Ordering::Relaxed) {
                    tracing::debug!("[ShellProc] 收到关闭信号: block_id={}", block_id);
                    break;
                }

                // 读取输出
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        // EOF，进程已退出
                        tracing::info!("[ShellProc] 进程已退出: block_id={}", block_id);
                        exited.store(true, Ordering::SeqCst);

                        // 发送状态事件
                        let _ = app_handle.emit(
                            event_names::TERMINAL_STATUS,
                            TerminalStatusEvent {
                                session_id: block_id.clone(),
                                status: SessionStatus::Done,
                                exit_code: Some(exit_code.load(Ordering::SeqCst)),
                                error: None,
                            },
                        );
                        break;
                    }
                    Ok(n) => {
                        let output_data = &buffer[..n];

                        // 保存到块文件
                        if let Some(ref bf) = block_file {
                            if let Err(e) = bf.append_data(output_data) {
                                tracing::warn!(
                                    "[ShellProc] 写入块文件失败: block_id={}, error={}",
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
                    Err(e) => {
                        // 检查是否是因为关闭导致的错误
                        if shutdown_flag.load(Ordering::Relaxed) {
                            break;
                        }

                        tracing::error!("[ShellProc] 读取错误: block_id={}, error={}", block_id, e);
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
    fn spawn_input_handler(
        block_id: String,
        writer: Arc<Mutex<Box<dyn Write + Send>>>,
        master: Arc<Mutex<Box<dyn portable_pty::MasterPty + Send>>>,
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
                    let mut w = writer.lock();
                    if let Err(e) = w.write_all(data) {
                        tracing::error!("[ShellProc] 写入失败: block_id={}, error={}", block_id, e);
                        continue;
                    }
                    if let Err(e) = w.flush() {
                        tracing::error!(
                            "[ShellProc] Flush 失败: block_id={}, error={}",
                            block_id,
                            e
                        );
                    }
                }

                // 处理终端大小调整
                if let Some(size) = &input.term_size {
                    let m = master.lock();
                    if let Err(e) = m.resize(PtySize {
                        rows: size.rows,
                        cols: size.cols,
                        pixel_width: 0,
                        pixel_height: 0,
                    }) {
                        tracing::error!(
                            "[ShellProc] 调整大小失败: block_id={}, error={}",
                            block_id,
                            e
                        );
                    } else {
                        tracing::debug!(
                            "[ShellProc] 调整大小: block_id={}, size={}x{}",
                            block_id,
                            size.cols,
                            size.rows
                        );
                    }
                }

                // 处理信号
                if let Some(sig_name) = &input.sig_name {
                    tracing::debug!(
                        "[ShellProc] 收到信号: block_id={}, signal={}",
                        block_id,
                        sig_name
                    );
                    // TODO: 实现信号发送
                }
            }

            tracing::debug!("[ShellProc] 输入处理任务结束: block_id={}", block_id);
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

    /// 写入数据到 PTY
    pub fn write(&self, data: &[u8]) -> Result<(), TerminalError> {
        let mut writer = self.writer.lock();
        writer
            .write_all(data)
            .map_err(|e| TerminalError::WriteFailed(e.to_string()))?;
        writer
            .flush()
            .map_err(|e| TerminalError::WriteFailed(e.to_string()))?;
        Ok(())
    }

    /// 调整 PTY 大小
    pub fn resize(&self, rows: u16, cols: u16) -> Result<(), TerminalError> {
        let master = self.master.lock();
        master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| TerminalError::ResizeFailed(e.to_string()))?;
        tracing::debug!(
            "[ShellProc] 调整大小: block_id={}, size={}x{}",
            self.block_id,
            cols,
            rows
        );
        Ok(())
    }

    /// 优雅终止进程
    ///
    /// 发送 SIGTERM 信号并等待进程退出。
    ///
    /// # 返回
    /// 进程退出码
    ///
    /// _Requirements: 17.3_
    pub async fn terminate(&self) -> i32 {
        tracing::info!("[ShellProc] 优雅终止进程: block_id={}", self.block_id);

        // 设置关闭标志
        self.shutdown_flag.store(true, Ordering::SeqCst);

        // TODO: 发送 SIGTERM 信号
        // 目前 portable_pty 不直接支持发送信号，需要通过其他方式实现

        // 等待一小段时间让进程退出
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        self.exit_code.load(Ordering::SeqCst)
    }

    /// 强制终止进程
    ///
    /// 发送 SIGKILL 信号立即终止进程。
    ///
    /// _Requirements: 17.3_
    pub async fn kill(&self) {
        tracing::info!("[ShellProc] 强制终止进程: block_id={}", self.block_id);

        // 设置关闭标志
        self.shutdown_flag.store(true, Ordering::SeqCst);

        // TODO: 发送 SIGKILL 信号
    }
}

impl Drop for ShellProc {
    fn drop(&mut self) {
        // 确保关闭标志被设置
        self.shutdown_flag.store(true, Ordering::SeqCst);
        tracing::debug!("[ShellProc] 进程已销毁: block_id={}", self.block_id);
    }
}
