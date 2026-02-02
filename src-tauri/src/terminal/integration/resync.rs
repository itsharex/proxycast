//! 状态重同步控制器
//!
//! 负责在连接恢复时重建终端状态，包括检查控制器状态、发送终端重置序列、恢复历史数据。
//!
//! ## 功能
//! - 检查当前控制器状态并决定是否需要重启
//! - 连接名称变更时停止旧控制器并创建新控制器
//! - 发送终端重置序列（重置属性、显示光标、禁用鼠标跟踪等）
//! - 从 BlockFile 读取历史输出并推送到前端
//!
//! ## Requirements
//! - 2.1: 检查当前控制器状态并决定是否需要重启
//! - 2.2: 连接名称变更时停止当前控制器并使用新连接重新创建
//! - 2.3: 发送终端重置序列
//! - 2.4: 从 BlockFile 读取历史输出并推送到前端
//! - 2.5: 控制器状态为 "init" 或 "done" 时启动新的控制器实例
//! - 2.6: force 参数为 true 时强制重启控制器

use std::sync::Arc;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use tauri::Emitter;

use crate::terminal::block_controller::{
    BlockController, BlockControllerRuntimeStatus, BlockMeta, ControllerRegistry, RuntimeOpts,
    ShellController,
};
use crate::terminal::error::TerminalError;
use crate::terminal::events::{event_names, TerminalOutputEvent};
use crate::terminal::persistence::BlockFile;

/// 终端重置序列
///
/// 用于在重同步时重置终端状态，包括：
/// - ESC c: 完全重置终端
/// - ESC [?25h: 显示光标
/// - ESC [?1000l: 禁用鼠标跟踪
/// - ESC [?1002l: 禁用按钮事件鼠标跟踪
/// - ESC [?1003l: 禁用任意事件鼠标跟踪
/// - ESC [?1006l: 禁用 SGR 鼠标模式
/// - ESC [0m: 重置所有属性
pub const TERMINAL_RESET_SEQUENCE: &[u8] =
    b"\x1bc\x1b[?25h\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1006l\x1b[0m";

/// 软重置序列（不完全重置终端）
///
/// 用于较轻量的重置，保留部分状态：
/// - ESC [!p: 软重置
/// - ESC [?25h: 显示光标
/// - ESC [0m: 重置所有属性
pub const TERMINAL_SOFT_RESET_SEQUENCE: &[u8] = b"\x1b[!p\x1b[?25h\x1b[0m";

/// 重同步选项
#[derive(Debug, Clone, Default)]
pub struct ResyncOptions {
    /// 是否强制重启控制器
    pub force: bool,
    /// 是否发送完全重置序列（否则使用软重置）
    pub full_reset: bool,
    /// 是否恢复历史数据
    pub restore_history: bool,
}

impl ResyncOptions {
    /// 创建默认选项
    pub fn new() -> Self {
        Self {
            force: false,
            full_reset: false,
            restore_history: true,
        }
    }

    /// 设置强制重启
    pub fn with_force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    /// 设置完全重置
    pub fn with_full_reset(mut self, full_reset: bool) -> Self {
        self.full_reset = full_reset;
        self
    }

    /// 设置是否恢复历史
    pub fn with_restore_history(mut self, restore: bool) -> Self {
        self.restore_history = restore;
        self
    }
}

/// 重同步结果
#[derive(Debug, Clone, Default)]
pub struct ResyncResult {
    /// 是否创建了新控制器
    pub controller_created: bool,
    /// 是否重启了控制器
    pub controller_restarted: bool,
    /// 是否恢复了历史数据
    pub history_restored: bool,
    /// 恢复的历史数据大小（字节）
    pub history_size: usize,
}

/// 状态重同步控制器
///
/// 提供终端状态重同步功能，用于在连接恢复或控制器状态变更时重建终端状态。
pub struct ResyncController;

impl ResyncController {
    /// 重同步控制器
    ///
    /// 检查当前控制器状态并决定是否需要重启、创建新控制器或恢复历史数据。
    ///
    /// # 参数
    /// - `registry`: 控制器注册表
    /// - `tab_id`: Tab ID
    /// - `block_id`: Block ID
    /// - `block_meta`: 块元数据
    /// - `rt_opts`: 运行时选项
    /// - `app_handle`: Tauri 应用句柄
    /// - `block_file`: 块文件（可选）
    /// - `options`: 重同步选项
    ///
    /// # 返回
    /// - `Ok(ResyncResult)`: 重同步结果
    /// - `Err(TerminalError)`: 重同步失败
    ///
    /// _Requirements: 2.1, 2.2, 2.5, 2.6_
    pub async fn resync_controller(
        registry: &ControllerRegistry,
        tab_id: &str,
        block_id: &str,
        block_meta: BlockMeta,
        rt_opts: Option<RuntimeOpts>,
        app_handle: tauri::AppHandle,
        block_file: Option<Arc<BlockFile>>,
        options: ResyncOptions,
    ) -> Result<ResyncResult, TerminalError> {
        let mut result = ResyncResult::default();

        // 1. 获取控制器名称
        let controller_name = block_meta.get_string("controller");
        let conn_name = block_meta.connection.clone();

        tracing::info!(
            "[ResyncController] 开始重同步: block_id={}, controller={}, conn={:?}, force={}",
            block_id,
            controller_name,
            conn_name,
            options.force
        );

        // 2. 如果不需要控制器，停止现有的
        if controller_name.is_empty() {
            if let Some(ctrl) = registry.get(block_id).await {
                tracing::info!(
                    "[ResyncController] 控制器名称为空，停止现有控制器: block_id={}",
                    block_id
                );
                let mut ctrl_guard = ctrl.write().await;
                ctrl_guard.stop(true, "done".to_string()).await?;
                drop(ctrl_guard);
                registry.remove(block_id).await;
            }
            return Ok(result);
        }

        // 3. 获取现有控制器
        let existing = registry.get(block_id).await;

        // 4. 检查是否需要替换控制器类型
        if let Some(ref ctrl) = existing {
            let ctrl_guard = ctrl.read().await;
            let status = ctrl_guard.get_runtime_status();
            let current_type = ctrl_guard.controller_type();

            // 检查控制器类型是否变更
            let needs_replace = Self::check_needs_replace(current_type, &controller_name);

            if needs_replace {
                tracing::info!(
                    "[ResyncController] 控制器类型变更，需要替换: block_id={}, old={}, new={}",
                    block_id,
                    current_type,
                    controller_name
                );
                drop(ctrl_guard);

                // 停止并删除旧控制器
                let mut ctrl_guard = ctrl.write().await;
                ctrl_guard.stop(true, "done".to_string()).await?;
                drop(ctrl_guard);
                registry.remove(block_id).await;
            } else if controller_name == "shell" || controller_name == "cmd" {
                // 检查连接是否变更
                if status.shell_proc_status == "running" {
                    if let Some(ref new_conn) = conn_name {
                        if status.shell_proc_conn_name.as_ref() != Some(new_conn) {
                            tracing::info!(
                                "[ResyncController] 连接变更，需要重启: block_id={}, old={:?}, new={}",
                                block_id,
                                status.shell_proc_conn_name,
                                new_conn
                            );
                            drop(ctrl_guard);

                            // 停止控制器但不删除，设置状态为 init
                            let mut ctrl_guard = ctrl.write().await;
                            ctrl_guard.stop(true, "init".to_string()).await?;
                            result.controller_restarted = true;
                        }
                    }
                }
            }
        }

        // 5. 强制重启
        if options.force {
            if let Some(ctrl) = registry.get(block_id).await {
                tracing::info!("[ResyncController] 强制重启控制器: block_id={}", block_id);
                let mut ctrl_guard = ctrl.write().await;
                ctrl_guard.stop(true, "init".to_string()).await?;
                result.controller_restarted = true;
            }
        }

        // 6. 创建或重用控制器
        let controller = match registry.get(block_id).await {
            Some(ctrl) => ctrl,
            None => {
                tracing::info!(
                    "[ResyncController] 创建新控制器: block_id={}, type={}",
                    block_id,
                    controller_name
                );

                let ctrl = Self::create_controller(
                    &controller_name,
                    tab_id,
                    block_id,
                    app_handle.clone(),
                    block_file.clone(),
                )?;

                registry.register(block_id.to_string(), ctrl).await;

                result.controller_created = true;

                registry
                    .get(block_id)
                    .await
                    .ok_or_else(|| TerminalError::Internal("控制器注册失败".to_string()))?
            }
        };

        // 7. 检查是否需要启动
        let status = {
            let ctrl_guard = controller.read().await;
            ctrl_guard.get_runtime_status()
        };

        if status.shell_proc_status == "init" || status.shell_proc_status == "done" {
            tracing::info!(
                "[ResyncController] 启动控制器: block_id={}, status={}",
                block_id,
                status.shell_proc_status
            );

            // 发送重置序列
            Self::send_reset_sequence(&app_handle, block_id, options.full_reset)?;

            // 恢复历史数据
            if options.restore_history {
                if let Some(ref bf) = block_file {
                    let history_size = Self::restore_history(&app_handle, block_id, bf)?;
                    if history_size > 0 {
                        result.history_restored = true;
                        result.history_size = history_size;
                    }
                }
            }

            // 启动控制器
            let mut ctrl_guard = controller.write().await;
            ctrl_guard.start(block_meta, rt_opts, options.force).await?;
        }

        tracing::info!(
            "[ResyncController] 重同步完成: block_id={}, created={}, restarted={}, history_restored={}",
            block_id,
            result.controller_created,
            result.controller_restarted,
            result.history_restored
        );

        Ok(result)
    }

    /// 检查是否需要替换控制器
    ///
    /// # 参数
    /// - `current_type`: 当前控制器类型
    /// - `new_type`: 新控制器类型
    ///
    /// # 返回
    /// 需要替换返回 true
    fn check_needs_replace(current_type: &str, new_type: &str) -> bool {
        // 如果类型不同，需要替换
        // shell 和 cmd 可以互相转换（都是 ShellController）
        if current_type == new_type {
            return false;
        }

        // shell 和 cmd 之间不需要替换控制器实例，只需要重启
        if (current_type == "shell" || current_type == "cmd")
            && (new_type == "shell" || new_type == "cmd")
        {
            return false;
        }

        true
    }

    /// 创建控制器
    ///
    /// # 参数
    /// - `controller_name`: 控制器类型名称
    /// - `tab_id`: Tab ID
    /// - `block_id`: Block ID
    /// - `app_handle`: Tauri 应用句柄
    /// - `block_file`: 块文件（可选）
    ///
    /// # 返回
    /// 创建的控制器实例
    fn create_controller(
        controller_name: &str,
        tab_id: &str,
        block_id: &str,
        app_handle: tauri::AppHandle,
        block_file: Option<Arc<BlockFile>>,
    ) -> Result<Box<dyn BlockController>, TerminalError> {
        match controller_name {
            "shell" | "cmd" => {
                let controller = if let Some(bf) = block_file {
                    ShellController::with_block_file(
                        tab_id.to_string(),
                        block_id.to_string(),
                        controller_name.to_string(),
                        app_handle,
                        bf,
                    )
                } else {
                    ShellController::new(
                        tab_id.to_string(),
                        block_id.to_string(),
                        controller_name.to_string(),
                        app_handle,
                    )
                };
                Ok(Box::new(controller))
            }
            _ => Err(TerminalError::Internal(format!(
                "未知的控制器类型: {controller_name}"
            ))),
        }
    }

    /// 发送终端重置序列
    ///
    /// 通过 Tauri 事件发送终端重置序列到前端。
    ///
    /// # 参数
    /// - `app_handle`: Tauri 应用句柄
    /// - `block_id`: Block ID
    /// - `full_reset`: 是否使用完全重置序列
    ///
    /// _Requirements: 2.3_
    fn send_reset_sequence(
        app_handle: &tauri::AppHandle,
        block_id: &str,
        full_reset: bool,
    ) -> Result<(), TerminalError> {
        let reset_data = if full_reset {
            TERMINAL_RESET_SEQUENCE
        } else {
            TERMINAL_SOFT_RESET_SEQUENCE
        };

        let data = BASE64.encode(reset_data);

        app_handle
            .emit(
                event_names::TERMINAL_OUTPUT,
                TerminalOutputEvent {
                    session_id: block_id.to_string(),
                    data,
                },
            )
            .map_err(|e| TerminalError::Internal(format!("发送重置序列失败: {e}")))?;

        tracing::debug!(
            "[ResyncController] 发送重置序列: block_id={}, full={}",
            block_id,
            full_reset
        );

        Ok(())
    }

    /// 恢复历史数据
    ///
    /// 从 BlockFile 读取历史数据并通过 Tauri 事件发送到前端。
    ///
    /// # 参数
    /// - `app_handle`: Tauri 应用句柄
    /// - `block_id`: Block ID
    /// - `block_file`: 块文件
    ///
    /// # 返回
    /// 恢复的数据大小（字节）
    ///
    /// _Requirements: 2.4_
    fn restore_history(
        app_handle: &tauri::AppHandle,
        block_id: &str,
        block_file: &BlockFile,
    ) -> Result<usize, TerminalError> {
        let history_data = block_file.read_all()?;

        if history_data.is_empty() {
            tracing::debug!(
                "[ResyncController] 无历史数据需要恢复: block_id={}",
                block_id
            );
            return Ok(0);
        }

        let data_size = history_data.len();
        let data = BASE64.encode(&history_data);

        app_handle
            .emit(
                event_names::TERMINAL_OUTPUT,
                TerminalOutputEvent {
                    session_id: block_id.to_string(),
                    data,
                },
            )
            .map_err(|e| TerminalError::Internal(format!("发送历史数据失败: {e}")))?;

        tracing::info!(
            "[ResyncController] 恢复历史数据: block_id={}, size={} bytes",
            block_id,
            data_size
        );

        Ok(data_size)
    }

    /// 停止并删除控制器
    ///
    /// # 参数
    /// - `registry`: 控制器注册表
    /// - `block_id`: Block ID
    /// - `graceful`: 是否优雅停止
    pub async fn stop_and_remove_controller(
        registry: &ControllerRegistry,
        block_id: &str,
        graceful: bool,
    ) -> Result<(), TerminalError> {
        if let Some(ctrl) = registry.get(block_id).await {
            let mut ctrl_guard = ctrl.write().await;
            ctrl_guard.stop(graceful, "done".to_string()).await?;
            drop(ctrl_guard);
            registry.remove(block_id).await;

            tracing::info!(
                "[ResyncController] 停止并删除控制器: block_id={}, graceful={}",
                block_id,
                graceful
            );
        }
        Ok(())
    }

    /// 获取控制器状态
    ///
    /// # 参数
    /// - `registry`: 控制器注册表
    /// - `block_id`: Block ID
    ///
    /// # 返回
    /// 控制器运行时状态，如果控制器不存在返回 None
    pub async fn get_controller_status(
        registry: &ControllerRegistry,
        block_id: &str,
    ) -> Option<BlockControllerRuntimeStatus> {
        if let Some(ctrl) = registry.get(block_id).await {
            let ctrl_guard = ctrl.read().await;
            Some(ctrl_guard.get_runtime_status())
        } else {
            None
        }
    }

    /// 检查是否需要重同步
    ///
    /// 根据当前控制器状态和块元数据判断是否需要重同步。
    ///
    /// # 参数
    /// - `status`: 当前控制器状态（如果存在）
    /// - `block_meta`: 块元数据
    ///
    /// # 返回
    /// 需要重同步返回 true
    pub fn needs_resync(
        status: Option<&BlockControllerRuntimeStatus>,
        block_meta: &BlockMeta,
    ) -> bool {
        let controller_name = block_meta.get_string("controller");

        // 如果没有控制器状态，需要重同步
        let Some(status) = status else {
            return !controller_name.is_empty();
        };

        // 如果控制器名称为空但有控制器，需要重同步（停止控制器）
        if controller_name.is_empty() {
            return true;
        }

        // 如果状态为 init 或 done，需要重同步
        if status.shell_proc_status == "init" || status.shell_proc_status == "done" {
            return true;
        }

        // 如果连接变更，需要重同步
        if let Some(ref new_conn) = block_meta.connection {
            if status.shell_proc_conn_name.as_ref() != Some(new_conn) {
                return true;
            }
        }

        false
    }
}

/// 便捷函数：执行简单的重同步
///
/// 使用默认选项执行重同步。
pub async fn resync_controller(
    registry: &ControllerRegistry,
    tab_id: &str,
    block_id: &str,
    block_meta: BlockMeta,
    rt_opts: Option<RuntimeOpts>,
    app_handle: tauri::AppHandle,
    block_file: Option<Arc<BlockFile>>,
    force: bool,
) -> Result<ResyncResult, TerminalError> {
    let options = ResyncOptions::new().with_force(force);
    ResyncController::resync_controller(
        registry, tab_id, block_id, block_meta, rt_opts, app_handle, block_file, options,
    )
    .await
}
