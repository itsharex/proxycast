//! ShellController 实现
//!
//! 实现 BlockController trait，管理本地和远程 Shell 进程。
//!
//! ## 功能
//! - 实现 BlockController trait 接口
//! - 管理 Shell 进程生命周期（init、running、done）
//! - 支持 "shell" 和 "cmd" 两种控制器类型
//! - 状态更新事件广播
//!
//! ## Requirements
//! - 1.2: 创建本地终端时实例化 Shell_Controller 并设置 controller_type 为 "shell"
//! - 1.3: 创建命令执行终端时实例化 Shell_Controller 并设置 controller_type 为 "cmd"
//! - 2.7: 会话状态变更时通过事件广播状态更新到所有订阅者

use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use tauri::Emitter;
use tokio::sync::{mpsc, RwLock};

use super::traits::{
    BlockController, BlockControllerRuntimeStatus, BlockInputUnion, BlockMeta, RuntimeOpts,
};
use crate::terminal::connections::ShellProc;
use crate::terminal::error::TerminalError;
use crate::terminal::persistence::BlockFile;

/// 控制器状态事件名称
pub const CONTROLLER_STATUS_EVENT: &str = "controller:status";

/// 控制器状态事件
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ControllerStatusEvent {
    /// 块 ID
    pub block_id: String,
    /// 状态版本号
    pub version: i32,
    /// Shell 进程状态
    pub shell_proc_status: String,
    /// 连接名称
    pub shell_proc_conn_name: Option<String>,
    /// 退出码
    pub shell_proc_exit_code: i32,
}

impl From<BlockControllerRuntimeStatus> for ControllerStatusEvent {
    fn from(status: BlockControllerRuntimeStatus) -> Self {
        Self {
            block_id: status.block_id,
            version: status.version,
            shell_proc_status: status.shell_proc_status,
            shell_proc_conn_name: status.shell_proc_conn_name,
            shell_proc_exit_code: status.shell_proc_exit_code,
        }
    }
}

/// Shell 控制器
///
/// 实现 BlockController trait，管理本地和远程 Shell 进程。
/// 支持 "shell"（交互式 Shell）和 "cmd"（命令执行）两种模式。
///
/// ## Cmd 模式配置选项
/// - `cmd_run_on_start`: 启动时自动运行命令
/// - `cmd_run_once`: 仅运行一次（不自动重启）
/// - `cmd_clear_on_start`: 启动前清空输出
/// - `cmd_close_on_exit`: 退出后自动关闭
pub struct ShellController {
    /// 控制器类型: "shell" | "cmd"
    controller_type: String,
    /// Tab ID
    tab_id: String,
    /// Block ID
    block_id: String,
    /// 运行锁，防止并发启动
    run_lock: AtomicBool,
    /// 进程状态: "init" | "running" | "done"
    proc_status: RwLock<String>,
    /// 进程退出码
    proc_exit_code: AtomicI32,
    /// 状态版本号，每次状态变更递增
    status_version: AtomicI32,
    /// 连接名称（用于 SSH/WSL）
    conn_name: RwLock<Option<String>>,
    /// Shell 进程
    shell_proc: RwLock<Option<ShellProc>>,
    /// Shell 输入发送器
    shell_input_tx: RwLock<Option<mpsc::Sender<BlockInputUnion>>>,
    /// Tauri 应用句柄
    app_handle: tauri::AppHandle,
    /// 块文件存储
    block_file: Option<Arc<BlockFile>>,
    /// 是否已运行过（用于 cmd:runonce）
    has_run: AtomicBool,
    /// 当前块元数据（用于重启）
    current_meta: RwLock<Option<BlockMeta>>,
}

impl ShellController {
    /// 创建新的 ShellController
    ///
    /// # 参数
    /// - `tab_id`: Tab ID
    /// - `block_id`: Block ID
    /// - `controller_type`: 控制器类型 ("shell" | "cmd")
    /// - `app_handle`: Tauri 应用句柄
    ///
    /// # 返回
    /// 新的 ShellController 实例
    ///
    /// _Requirements: 1.2, 1.3_
    pub fn new(
        tab_id: String,
        block_id: String,
        controller_type: String,
        app_handle: tauri::AppHandle,
    ) -> Self {
        tracing::info!(
            "[ShellController] 创建控制器: block_id={}, type={}",
            block_id,
            controller_type
        );

        Self {
            controller_type,
            tab_id,
            block_id,
            run_lock: AtomicBool::new(false),
            proc_status: RwLock::new("init".to_string()),
            proc_exit_code: AtomicI32::new(0),
            status_version: AtomicI32::new(0),
            conn_name: RwLock::new(None),
            shell_proc: RwLock::new(None),
            shell_input_tx: RwLock::new(None),
            app_handle,
            block_file: None,
            has_run: AtomicBool::new(false),
            current_meta: RwLock::new(None),
        }
    }

    /// 创建带块文件的 ShellController
    ///
    /// # 参数
    /// - `tab_id`: Tab ID
    /// - `block_id`: Block ID
    /// - `controller_type`: 控制器类型
    /// - `app_handle`: Tauri 应用句柄
    /// - `block_file`: 块文件存储
    pub fn with_block_file(
        tab_id: String,
        block_id: String,
        controller_type: String,
        app_handle: tauri::AppHandle,
        block_file: Arc<BlockFile>,
    ) -> Self {
        let mut controller = Self::new(tab_id, block_id, controller_type, app_handle);
        controller.block_file = Some(block_file);
        controller
    }

    /// 获取 Tab ID
    pub fn tab_id(&self) -> &str {
        &self.tab_id
    }

    /// 获取 Block ID
    pub fn block_id(&self) -> &str {
        &self.block_id
    }

    /// 设置块文件
    pub fn set_block_file(&mut self, block_file: Arc<BlockFile>) {
        self.block_file = Some(block_file);
    }

    /// 获取块文件引用
    pub fn block_file(&self) -> Option<&Arc<BlockFile>> {
        self.block_file.as_ref()
    }

    /// 更新进程状态
    ///
    /// 更新状态并递增版本号，然后广播状态更新事件。
    ///
    /// # 参数
    /// - `new_status`: 新状态 ("init" | "running" | "done")
    ///
    /// _Requirements: 2.7_
    async fn set_status(&self, new_status: &str) {
        {
            let mut status = self.proc_status.write().await;
            *status = new_status.to_string();
        }
        self.status_version.fetch_add(1, Ordering::SeqCst);
        self.send_status_update().await;
    }

    /// 设置退出码
    ///
    /// # 参数
    /// - `exit_code`: 进程退出码
    fn set_exit_code(&self, exit_code: i32) {
        self.proc_exit_code.store(exit_code, Ordering::SeqCst);
    }

    /// 设置连接名称
    ///
    /// # 参数
    /// - `conn_name`: 连接名称
    async fn set_conn_name(&self, conn_name: Option<String>) {
        let mut name = self.conn_name.write().await;
        *name = conn_name;
    }

    /// 发送状态更新事件
    ///
    /// 通过 Tauri 事件系统广播控制器状态更新。
    ///
    /// _Requirements: 2.7_
    async fn send_status_update(&self) {
        let status = self.get_runtime_status();
        let event = ControllerStatusEvent::from(status);

        if let Err(e) = self.app_handle.emit(CONTROLLER_STATUS_EVENT, &event) {
            tracing::error!(
                "[ShellController] 发送状态更新事件失败: block_id={}, error={}",
                self.block_id,
                e
            );
        } else {
            tracing::debug!(
                "[ShellController] 状态更新: block_id={}, status={}, version={}",
                self.block_id,
                event.shell_proc_status,
                event.version
            );
        }
    }

    /// 尝试获取运行锁
    ///
    /// # 返回
    /// 成功获取返回 true，已被占用返回 false
    fn try_acquire_run_lock(&self) -> bool {
        self.run_lock
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    /// 释放运行锁
    fn release_run_lock(&self) {
        self.run_lock.store(false, Ordering::SeqCst);
    }

    /// 检查是否正在运行
    pub async fn is_running(&self) -> bool {
        let status = self.proc_status.read().await;
        *status == "running"
    }

    /// 检查是否已完成
    pub async fn is_done(&self) -> bool {
        let status = self.proc_status.read().await;
        *status == "done"
    }

    /// 检查是否为初始状态
    pub async fn is_init(&self) -> bool {
        let status = self.proc_status.read().await;
        *status == "init"
    }

    /// 检查是否应该运行（考虑 cmd:runonce）
    ///
    /// _Requirements: 16.6_
    fn should_run(&self, block_meta: &BlockMeta) -> bool {
        // 如果是 cmd 模式且设置了 runonce，检查是否已运行过
        if self.controller_type == "cmd"
            && block_meta.cmd_run_once.unwrap_or(false)
            && self.has_run.load(Ordering::SeqCst)
        {
            tracing::debug!(
                "[ShellController] cmd:runonce 已运行过，跳过: block_id={}",
                self.block_id
            );
            return false;
        }
        true
    }

    /// 处理 cmd:clearonstart 配置
    ///
    /// _Requirements: 16.7_
    async fn handle_clear_on_start(&self, block_meta: &BlockMeta) {
        if self.controller_type == "cmd" && block_meta.cmd_clear_on_start.unwrap_or(false) {
            if let Some(ref bf) = self.block_file {
                if let Err(e) = bf.truncate() {
                    tracing::warn!(
                        "[ShellController] cmd:clearonstart 清空块文件失败: block_id={}, error={}",
                        self.block_id,
                        e
                    );
                } else {
                    tracing::debug!(
                        "[ShellController] cmd:clearonstart 已清空块文件: block_id={}",
                        self.block_id
                    );
                }
            }
        }
    }

    /// 检查是否应该自动运行（cmd:runonstart）
    ///
    /// _Requirements: 16.5_
    fn should_auto_run(&self, block_meta: &BlockMeta) -> bool {
        if self.controller_type == "cmd" {
            // 默认 cmd 模式自动运行，除非明确设置为 false
            block_meta.cmd_run_on_start.unwrap_or(true)
        } else {
            // shell 模式总是自动运行
            true
        }
    }

    /// 获取当前块元数据
    pub async fn get_current_meta(&self) -> Option<BlockMeta> {
        self.current_meta.read().await.clone()
    }

    /// 重启控制器（用于 cmd 模式重新运行）
    ///
    /// _Requirements: 16.9_
    pub async fn restart(&mut self) -> Result<(), TerminalError> {
        let meta = self.current_meta.read().await.clone();
        if let Some(block_meta) = meta {
            // 重置 has_run 标志以允许重新运行
            self.has_run.store(false, Ordering::SeqCst);
            self.start(block_meta, None, true).await
        } else {
            Err(TerminalError::Internal("没有保存的块元数据".to_string()))
        }
    }
}

#[async_trait]
impl BlockController for ShellController {
    /// 启动控制器
    ///
    /// 根据 block_meta 配置启动 Shell 或命令执行进程。
    ///
    /// # 参数
    /// - `block_meta`: 块元数据配置
    /// - `rt_opts`: 运行时选项（终端大小等）
    /// - `force`: 是否强制重启
    ///
    /// # 返回
    /// 成功返回 Ok(()), 失败返回错误
    ///
    /// _Requirements: 1.2, 1.3, 16.5, 16.6, 16.7_
    async fn start(
        &mut self,
        block_meta: BlockMeta,
        rt_opts: Option<RuntimeOpts>,
        force: bool,
    ) -> Result<(), TerminalError> {
        // 尝试获取运行锁
        if !self.try_acquire_run_lock() {
            tracing::warn!(
                "[ShellController] 控制器已在运行中: block_id={}",
                self.block_id
            );
            return Ok(());
        }

        // 确保在函数退出时释放锁
        let _lock_guard = scopeguard::guard((), |_| {
            self.release_run_lock();
        });

        // 检查当前状态
        let current_status = self.proc_status.read().await.clone();
        if current_status == "running" && !force {
            tracing::debug!(
                "[ShellController] 控制器已在运行: block_id={}",
                self.block_id
            );
            return Ok(());
        }

        // 检查是否应该运行（cmd:runonce）
        if !self.should_run(&block_meta) {
            return Ok(());
        }

        // 检查是否应该自动运行（cmd:runonstart）
        if !self.should_auto_run(&block_meta) && !force {
            tracing::debug!(
                "[ShellController] cmd:runonstart=false，跳过自动启动: block_id={}",
                self.block_id
            );
            return Ok(());
        }

        // 如果强制重启，先停止现有进程
        if force && current_status == "running" {
            tracing::info!(
                "[ShellController] 强制重启控制器: block_id={}",
                self.block_id
            );
            // 停止现有进程
            let mut shell_proc = self.shell_proc.write().await;
            if let Some(proc) = shell_proc.take() {
                proc.kill().await;
            }
        }

        // 处理 cmd:clearonstart
        self.handle_clear_on_start(&block_meta).await;

        // 保存块元数据
        {
            let mut meta = self.current_meta.write().await;
            *meta = Some(block_meta.clone());
        }

        // 更新连接名称
        let conn_name = block_meta.connection.clone();
        self.set_conn_name(conn_name.clone()).await;

        // 获取终端大小
        let term_size = rt_opts
            .as_ref()
            .map(|opts| opts.term_size)
            .unwrap_or_default();

        tracing::info!(
            "[ShellController] 启动控制器: block_id={}, type={}, conn={:?}, size={}x{}",
            self.block_id,
            self.controller_type,
            conn_name,
            term_size.cols,
            term_size.rows
        );

        // 创建输入通道
        let (input_tx, input_rx) = mpsc::channel::<BlockInputUnion>(256);
        {
            let mut tx = self.shell_input_tx.write().await;
            *tx = Some(input_tx);
        }

        // 创建 Shell 进程
        let shell_proc = ShellProc::new(
            self.block_id.clone(),
            self.controller_type.clone(),
            term_size.rows,
            term_size.cols,
            self.app_handle.clone(),
            block_meta.clone(),
            input_rx,
            self.block_file.clone(),
        )
        .await?;

        // 保存进程引用
        {
            let mut proc = self.shell_proc.write().await;
            *proc = Some(shell_proc);
        }

        // 标记已运行（用于 cmd:runonce）
        self.has_run.store(true, Ordering::SeqCst);

        // 更新状态为运行中
        self.set_status("running").await;
        self.set_exit_code(0);

        tracing::info!("[ShellController] 控制器已启动: block_id={}", self.block_id);

        Ok(())
    }

    /// 停止控制器
    ///
    /// 停止 Shell 进程并更新状态。
    ///
    /// # 参数
    /// - `graceful`: 是否优雅停止（发送 SIGTERM 而非 SIGKILL）
    /// - `new_status`: 停止后的新状态
    ///
    /// # 返回
    /// 成功返回 Ok(()), 失败返回错误
    async fn stop(&mut self, graceful: bool, new_status: String) -> Result<(), TerminalError> {
        tracing::info!(
            "[ShellController] 停止控制器: block_id={}, graceful={}, new_status={}",
            self.block_id,
            graceful,
            new_status
        );

        // 关闭输入通道
        {
            let mut tx = self.shell_input_tx.write().await;
            *tx = None;
        }

        // 停止 Shell 进程
        let exit_code = {
            let mut shell_proc = self.shell_proc.write().await;
            if let Some(proc) = shell_proc.take() {
                if graceful {
                    proc.terminate().await
                } else {
                    proc.kill().await;
                    -1
                }
            } else {
                0
            }
        };

        // 更新退出码和状态
        self.set_exit_code(exit_code);
        self.set_status(&new_status).await;

        tracing::info!(
            "[ShellController] 控制器已停止: block_id={}, exit_code={}",
            self.block_id,
            exit_code
        );

        Ok(())
    }

    /// 获取运行时状态
    ///
    /// # 返回
    /// 当前控制器的运行时状态
    fn get_runtime_status(&self) -> BlockControllerRuntimeStatus {
        // 使用 try_read 避免死锁，如果无法获取锁则返回默认值
        let status = self
            .proc_status
            .try_read()
            .map(|s| s.clone())
            .unwrap_or_else(|_| "init".to_string());

        let conn_name = self.conn_name.try_read().map(|n| n.clone()).unwrap_or(None);

        BlockControllerRuntimeStatus {
            block_id: self.block_id.clone(),
            version: self.status_version.load(Ordering::SeqCst),
            shell_proc_status: status,
            shell_proc_conn_name: conn_name,
            shell_proc_exit_code: self.proc_exit_code.load(Ordering::SeqCst),
        }
    }

    /// 发送输入到控制器
    ///
    /// 将输入数据发送到 Shell 进程。
    ///
    /// # 参数
    /// - `input`: 输入数据（键盘输入、信号、终端大小调整）
    ///
    /// # 返回
    /// 成功返回 Ok(()), 失败返回错误
    async fn send_input(&self, input: &BlockInputUnion) -> Result<(), TerminalError> {
        let tx = self.shell_input_tx.read().await;
        if let Some(sender) = tx.as_ref() {
            sender
                .send(input.clone())
                .await
                .map_err(|e| TerminalError::WriteFailed(format!("发送输入失败: {e}")))?;
            Ok(())
        } else {
            Err(TerminalError::SessionClosed)
        }
    }

    /// 获取控制器类型
    ///
    /// # 返回
    /// 控制器类型字符串: "shell" | "cmd"
    fn controller_type(&self) -> &str {
        &self.controller_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 注意：完整的测试需要 Tauri 运行时环境，这里只测试基本逻辑

    #[test]
    fn test_controller_status_event_from() {
        let status = BlockControllerRuntimeStatus {
            block_id: "test-block".to_string(),
            version: 5,
            shell_proc_status: "running".to_string(),
            shell_proc_conn_name: Some("ssh://user@host".to_string()),
            shell_proc_exit_code: 0,
        };

        let event = ControllerStatusEvent::from(status);
        assert_eq!(event.block_id, "test-block");
        assert_eq!(event.version, 5);
        assert_eq!(event.shell_proc_status, "running");
        assert_eq!(
            event.shell_proc_conn_name,
            Some("ssh://user@host".to_string())
        );
        assert_eq!(event.shell_proc_exit_code, 0);
    }
}
