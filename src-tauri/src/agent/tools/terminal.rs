//! Terminal 工具模块
//!
//! 提供终端命令执行功能，通过 Tauri 事件与前端通信
//! 支持命令审批流程，命令在用户的实际终端中执行
//!
//! ## 工作流程
//! 1. AI 调用 terminal 工具
//! 2. 工具发送事件到前端请求执行命令
//! 3. 前端显示审批 UI
//! 4. 用户批准后，命令发送到实际终端
//! 5. 终端执行结果返回给 AI

use super::registry::Tool;
use super::types::{JsonSchema, PropertySchema, ToolDefinition, ToolError, ToolResult};
use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// 默认超时时间（秒）
const DEFAULT_TIMEOUT_SECS: u64 = 120;

/// 命令执行请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalCommandRequest {
    /// 请求 ID
    pub request_id: String,
    /// 要执行的命令
    pub command: String,
    /// 工作目录（可选）
    pub working_dir: Option<String>,
    /// 超时时间（秒）
    pub timeout_secs: u64,
}

/// 命令执行响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalCommandResponse {
    /// 请求 ID
    pub request_id: String,
    /// 是否成功
    pub success: bool,
    /// 输出内容
    pub output: String,
    /// 错误信息
    pub error: Option<String>,
    /// 退出码
    pub exit_code: Option<i32>,
    /// 是否被用户拒绝
    pub rejected: bool,
}

/// 命令执行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandStatus {
    /// 等待审批
    Pending,
    /// 已批准，执行中
    Executing,
    /// 已完成
    Completed,
    /// 已拒绝
    Rejected,
    /// 超时
    Timeout,
}

/// 待处理的命令（内部使用）
pub(crate) struct PendingCommand {
    /// 响应发送器
    response_tx: oneshot::Sender<TerminalCommandResponse>,
}

/// 已执行命令的记录
#[derive(Debug, Clone)]
struct ExecutedCommand {
    /// 命令内容
    command: String,
    /// 执行时间
    executed_at: std::time::Instant,
    /// 是否成功
    success: bool,
}

/// 重复命令检测的时间窗口（秒）
const DUPLICATE_DETECTION_WINDOW_SECS: u64 = 30;

/// Terminal 工具
///
/// 通过 Tauri 事件与前端通信，在用户终端中执行命令
pub struct TerminalTool {
    /// 待处理的命令
    pending_commands: Arc<RwLock<HashMap<String, PendingCommand>>>,
    /// 已执行的命令历史（用于检测重复）
    executed_commands: Arc<RwLock<Vec<ExecutedCommand>>>,
    /// 默认超时时间（秒）
    timeout_secs: u64,
    /// Tauri AppHandle（用于发送事件）
    app_handle: Arc<RwLock<Option<tauri::AppHandle>>>,
}

impl TerminalTool {
    /// 创建新的 Terminal 工具
    pub fn new() -> Self {
        Self {
            pending_commands: Arc::new(RwLock::new(HashMap::new())),
            executed_commands: Arc::new(RwLock::new(Vec::new())),
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            app_handle: Arc::new(RwLock::new(None)),
        }
    }

    /// 设置超时时间
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// 设置 Tauri AppHandle
    pub fn set_app_handle(&self, handle: tauri::AppHandle) {
        let mut app_handle = self.app_handle.write();
        *app_handle = Some(handle);
        eprintln!("[TerminalTool] AppHandle 已设置");
        tracing::info!("[TerminalTool] AppHandle 已设置");
    }

    /// 检查 AppHandle 是否已设置
    pub fn is_app_handle_set(&self) -> bool {
        let app_handle = self.app_handle.read();
        app_handle.is_some()
    }

    /// 处理命令响应（由前端调用）
    pub fn handle_response(&self, response: TerminalCommandResponse) {
        let request_id = response.request_id.clone();

        let pending = {
            let mut commands = self.pending_commands.write();
            commands.remove(&request_id)
        };

        if let Some(pending) = pending {
            if pending.response_tx.send(response).is_err() {
                warn!("[TerminalTool] 发送响应失败，接收端已关闭: {}", request_id);
            }
        } else {
            warn!("[TerminalTool] 未找到待处理的命令: {}", request_id);
        }
    }

    /// 检查命令是否在最近执行过（用于防止重复执行）
    fn check_duplicate_command(&self, command: &str) -> Option<&'static str> {
        let now = std::time::Instant::now();
        let window = Duration::from_secs(DUPLICATE_DETECTION_WINDOW_SECS);

        // 清理过期的记录
        {
            let mut history = self.executed_commands.write();
            history.retain(|cmd| now.duration_since(cmd.executed_at) < window);
        }

        // 检查是否有重复
        let history = self.executed_commands.read();
        for cmd in history.iter() {
            if cmd.command == command && cmd.success {
                return Some(
                    "This exact command was already executed successfully within the last 30 seconds. \
                     Do NOT re-execute it. If you need to verify the result, use the term_get_scrollback \
                     tool to read the terminal output."
                );
            }
        }

        None
    }

    /// 记录已执行的命令
    fn record_executed_command(&self, command: &str, success: bool) {
        let mut history = self.executed_commands.write();
        history.push(ExecutedCommand {
            command: command.to_string(),
            executed_at: std::time::Instant::now(),
            success,
        });

        // 限制历史记录大小
        if history.len() > 100 {
            history.remove(0);
        }
    }

    /// 执行命令
    async fn execute_command(
        &self,
        command: &str,
        working_dir: Option<&str>,
        timeout_secs: u64,
    ) -> Result<TerminalCommandResponse, ToolError> {
        let request_id = uuid::Uuid::new_v4().to_string();

        info!(
            "[TerminalTool] 请求执行命令: {} (request_id: {}, timeout: {}s)",
            command, request_id, timeout_secs
        );

        // 创建响应通道
        let (response_tx, response_rx) = oneshot::channel();

        // 添加到待处理列表
        {
            let mut commands = self.pending_commands.write();
            commands.insert(request_id.clone(), PendingCommand { response_tx });
        }

        // 构建请求
        let request = TerminalCommandRequest {
            request_id: request_id.clone(),
            command: command.to_string(),
            working_dir: working_dir.map(|s| s.to_string()),
            timeout_secs,
        };

        // 发送事件到前端
        {
            let app_handle = self.app_handle.read();
            eprintln!(
                "[TerminalTool] 检查 AppHandle: is_some={}",
                app_handle.is_some()
            );

            if let Some(handle) = app_handle.as_ref() {
                use tauri::Emitter;
                eprintln!("[TerminalTool] 尝试发送事件到前端: {}", request_id);
                if let Err(e) = handle.emit("terminal_command_request", &request) {
                    // 清理待处理命令
                    let mut commands = self.pending_commands.write();
                    commands.remove(&request_id);

                    // 返回失败响应而不是错误，避免 Agent 重试
                    warn!("[TerminalTool] 发送事件到前端失败: {}", e);
                    return Ok(TerminalCommandResponse {
                        request_id: request_id.clone(),
                        success: false,
                        output: String::new(),
                        error: Some(format!("无法发送命令到终端：{}。请检查应用配置。", e)),
                        exit_code: Some(-1),
                        rejected: false,
                    });
                }
                debug!("[TerminalTool] 已发送命令请求到前端: {}", request_id);
            } else {
                // 清理待处理命令
                let mut commands = self.pending_commands.write();
                commands.remove(&request_id);
                eprintln!("[TerminalTool] AppHandle 为 None，无法发送事件");

                // 返回失败响应而不是错误，避免 Agent 重试
                warn!("[TerminalTool] AppHandle 未设置");
                return Ok(TerminalCommandResponse {
                    request_id: request_id.clone(),
                    success: false,
                    output: String::new(),
                    error: Some(
                        "终端工具未正确初始化。这是一个应用程序配置问题，请联系开发者。\n\
                        作为替代方案，我可以为您提供命令建议，但无法直接执行。"
                            .to_string(),
                    ),
                    exit_code: Some(-1),
                    rejected: false,
                });
            }
        }

        // 等待响应（带超时）
        let timeout_duration = Duration::from_secs(timeout_secs);
        match timeout(timeout_duration, response_rx).await {
            Ok(Ok(response)) => {
                debug!(
                    "[TerminalTool] 收到响应: request_id={}, success={}",
                    response.request_id, response.success
                );
                Ok(response)
            }
            Ok(Err(_)) => {
                // 通道关闭
                let mut commands = self.pending_commands.write();
                commands.remove(&request_id);
                Err(ToolError::ExecutionFailed("响应通道已关闭".to_string()))
            }
            Err(_) => {
                // 超时
                warn!("[TerminalTool] 命令执行超时: {}", request_id);
                let mut commands = self.pending_commands.write();
                commands.remove(&request_id);
                Err(ToolError::Timeout)
            }
        }
    }
}

impl Default for TerminalTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for TerminalTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "terminal",
            "Execute a command in the user's terminal. The command will be sent to the \
             active terminal session and requires user approval before execution.\n\n\
             CRITICAL RULES:\n\
             1. NEVER re-execute a command that has already succeeded. When you receive \
                '[COMMAND EXECUTED SUCCESSFULLY]' in the response, the command is DONE.\n\
             2. Each command runs exactly ONCE. Do not retry successful commands.\n\
             3. If you need to verify the result, use the term_get_scrollback tool to \
                read the terminal output instead of re-running the command.\n\n\
             Use this for running system commands, scripts, or any command-line operations \
             that should be visible to the user.",
        )
        .with_parameters(
            JsonSchema::new()
                .add_property(
                    "command",
                    PropertySchema::string(
                        "The command to execute in the terminal. Can be any valid shell command.",
                    ),
                    true,
                )
                .add_property(
                    "working_dir",
                    PropertySchema::string(
                        "Optional working directory for the command. If not specified, \
                         uses the terminal's current directory.",
                    ),
                    false,
                )
                .add_property(
                    "timeout",
                    PropertySchema::integer(
                        "Optional timeout in seconds. Defaults to 120 seconds.",
                    )
                    .with_default(serde_json::json!(120)),
                    false,
                ),
        )
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, ToolError> {
        // 解析参数
        let command = args
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("缺少 command 参数".to_string()))?;

        let working_dir = args.get("working_dir").and_then(|v| v.as_str());

        let timeout_secs = args
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(self.timeout_secs);

        // 检查是否是重复命令
        if let Some(duplicate_msg) = self.check_duplicate_command(command) {
            warn!("[TerminalTool] 检测到重复命令: {}", command);
            return Ok(ToolResult::success(format!(
                "[DUPLICATE COMMAND BLOCKED]\n{}\n\nOriginal command: {}",
                duplicate_msg, command
            )));
        }

        // 执行命令
        let response = self
            .execute_command(command, working_dir, timeout_secs)
            .await?;

        // 记录已执行的命令
        self.record_executed_command(command, response.success);

        // 构建输出
        if response.rejected {
            return Ok(ToolResult::failure_with_output(
                "用户拒绝执行此命令".to_string(),
                "命令被用户拒绝".to_string(),
            ));
        }

        if response.success {
            Ok(ToolResult::success(response.output))
        } else {
            let error_msg = response.error.unwrap_or_else(|| {
                format!(
                    "命令执行失败 (退出码: {})",
                    response.exit_code.unwrap_or(-1)
                )
            });
            Ok(ToolResult::failure_with_output(response.output, error_msg))
        }
    }
}

/// 全局 TerminalTool 实例
static TERMINAL_TOOL: once_cell::sync::Lazy<Arc<TerminalTool>> = once_cell::sync::Lazy::new(|| {
    eprintln!("[TerminalTool] 创建全局实例");
    Arc::new(TerminalTool::new())
});

/// 获取全局 TerminalTool 实例
pub fn get_terminal_tool() -> Arc<TerminalTool> {
    eprintln!("[TerminalTool] 获取全局实例");
    Arc::clone(&TERMINAL_TOOL)
}

/// 设置全局 TerminalTool 的 AppHandle
pub fn set_terminal_tool_app_handle(handle: tauri::AppHandle) {
    eprintln!("[TerminalTool] 设置全局 AppHandle");
    tracing::info!("[TerminalTool] 设置全局 AppHandle");

    // 强制初始化全局实例（如果还未初始化）
    let _ = &*TERMINAL_TOOL;

    TERMINAL_TOOL.set_app_handle(handle);

    // 验证设置是否成功
    let app_handle = TERMINAL_TOOL.app_handle.read();
    if app_handle.is_some() {
        eprintln!("[TerminalTool] AppHandle 设置成功，已验证");
        tracing::info!("[TerminalTool] AppHandle 设置成功，已验证");
    } else {
        eprintln!("[TerminalTool] 警告：AppHandle 设置后仍为 None");
        tracing::error!("[TerminalTool] 警告：AppHandle 设置后仍为 None");
    }
}

/// 处理终端命令响应（由 Tauri 命令调用）
pub fn handle_terminal_command_response(response: TerminalCommandResponse) {
    TERMINAL_TOOL.handle_response(response);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_definition() {
        let tool = TerminalTool::new();
        let def = tool.definition();

        assert_eq!(def.name, "terminal");
        assert!(!def.description.is_empty());
        assert!(def.parameters.required.contains(&"command".to_string()));
    }

    #[test]
    fn test_terminal_command_request_serialization() {
        let request = TerminalCommandRequest {
            request_id: "test-123".to_string(),
            command: "echo hello".to_string(),
            working_dir: Some("/home/user".to_string()),
            timeout_secs: 60,
        };

        let json = serde_json::to_string(&request).unwrap();
        let parsed: TerminalCommandRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.request_id, "test-123");
        assert_eq!(parsed.command, "echo hello");
        assert_eq!(parsed.working_dir, Some("/home/user".to_string()));
        assert_eq!(parsed.timeout_secs, 60);
    }

    #[test]
    fn test_terminal_command_response_serialization() {
        let response = TerminalCommandResponse {
            request_id: "test-123".to_string(),
            success: true,
            output: "hello\n".to_string(),
            error: None,
            exit_code: Some(0),
            rejected: false,
        };

        let json = serde_json::to_string(&response).unwrap();
        let parsed: TerminalCommandResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.request_id, "test-123");
        assert!(parsed.success);
        assert_eq!(parsed.output, "hello\n");
        assert_eq!(parsed.exit_code, Some(0));
        assert!(!parsed.rejected);
    }
}
