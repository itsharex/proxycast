//! Terminal Scrollback 工具模块
//!
//! 提供只读访问终端输出历史的功能
//! 参考 Waveterm 的 term_get_scrollback 工具设计

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
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// 获取滚动缓冲区的请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetScrollbackRequest {
    /// 请求 ID
    pub request_id: String,
    /// 终端会话 ID
    pub session_id: String,
    /// 起始行（可选，默认从最后 200 行开始）
    pub line_start: Option<usize>,
    /// 行数（可选，默认 200 行）
    pub count: Option<usize>,
}

/// 获取滚动缓冲区的响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetScrollbackResponse {
    /// 请求 ID
    pub request_id: String,
    /// 是否成功
    pub success: bool,
    /// 总行数
    pub total_lines: usize,
    /// 起始行
    pub line_start: usize,
    /// 结束行
    pub line_end: usize,
    /// 内容
    pub content: String,
    /// 是否有更多数据
    pub has_more: bool,
    /// 错误信息
    pub error: Option<String>,
}

/// 待处理的请求（内部使用）
pub(crate) struct PendingRequest {
    /// 响应发送器
    response_tx: oneshot::Sender<GetScrollbackResponse>,
}

/// Terminal Scrollback 工具
///
/// 允许 AI 读取终端的输出历史，而不是直接执行命令
pub struct TermScrollbackTool {
    /// 待处理的请求
    pending_requests: Arc<RwLock<HashMap<String, PendingRequest>>>,
    /// 默认超时时间（秒）
    timeout_secs: u64,
    /// Tauri AppHandle（用于发送事件）
    app_handle: Arc<RwLock<Option<tauri::AppHandle>>>,
}

impl TermScrollbackTool {
    /// 创建新的 TermScrollbackTool
    pub fn new() -> Self {
        Self {
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
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
        eprintln!("[TermScrollbackTool] AppHandle 已设置");
        tracing::info!("[TermScrollbackTool] AppHandle 已设置");
    }

    /// 检查 AppHandle 是否已设置
    pub fn is_app_handle_set(&self) -> bool {
        let app_handle = self.app_handle.read();
        app_handle.is_some()
    }

    /// 处理响应（由前端调用）
    pub fn handle_response(&self, response: GetScrollbackResponse) {
        let request_id = response.request_id.clone();

        let pending = {
            let mut requests = self.pending_requests.write();
            requests.remove(&request_id)
        };

        if let Some(pending) = pending {
            if pending.response_tx.send(response).is_err() {
                warn!(
                    "[TermScrollbackTool] 发送响应失败，接收端已关闭: {}",
                    request_id
                );
            }
        } else {
            warn!("[TermScrollbackTool] 未找到待处理的请求: {}", request_id);
        }
    }

    /// 获取终端滚动缓冲区
    async fn get_scrollback(
        &self,
        session_id: &str,
        line_start: Option<usize>,
        count: Option<usize>,
    ) -> Result<GetScrollbackResponse, ToolError> {
        let request_id = uuid::Uuid::new_v4().to_string();

        info!(
            "[TermScrollbackTool] 请求获取滚动缓冲区: session_id={}, request_id={}",
            session_id, request_id
        );

        // 创建响应通道
        let (response_tx, response_rx) = oneshot::channel();

        // 添加到待处理列表
        {
            let mut requests = self.pending_requests.write();
            requests.insert(request_id.clone(), PendingRequest { response_tx });
        }

        // 构建请求
        let request = GetScrollbackRequest {
            request_id: request_id.clone(),
            session_id: session_id.to_string(),
            line_start,
            count,
        };

        // 发送事件到前端
        {
            let app_handle = self.app_handle.read();
            eprintln!(
                "[TermScrollbackTool] 检查 AppHandle: is_some={}",
                app_handle.is_some()
            );

            if let Some(handle) = app_handle.as_ref() {
                use tauri::Emitter;
                eprintln!("[TermScrollbackTool] 尝试发送事件到前端: {}", request_id);
                if let Err(e) = handle.emit("term_get_scrollback_request", &request) {
                    // 清理待处理请求
                    let mut requests = self.pending_requests.write();
                    requests.remove(&request_id);

                    warn!("[TermScrollbackTool] 发送事件到前端失败: {}", e);
                    return Err(ToolError::ExecutionFailed(format!(
                        "无法发送请求到前端：{}",
                        e
                    )));
                }
                debug!("[TermScrollbackTool] 已发送请求到前端: {}", request_id);
            } else {
                // 清理待处理请求
                let mut requests = self.pending_requests.write();
                requests.remove(&request_id);
                eprintln!("[TermScrollbackTool] AppHandle 为 None，无法发送事件");

                warn!("[TermScrollbackTool] AppHandle 未设置");
                return Err(ToolError::ExecutionFailed(
                    "TermScrollbackTool 未正确初始化".to_string(),
                ));
            }
        }

        // 等待响应（带超时）
        let timeout_duration = Duration::from_secs(self.timeout_secs);
        match timeout(timeout_duration, response_rx).await {
            Ok(Ok(response)) => {
                debug!(
                    "[TermScrollbackTool] 收到响应: request_id={}, success={}",
                    response.request_id, response.success
                );
                Ok(response)
            }
            Ok(Err(_)) => {
                // 通道关闭
                let mut requests = self.pending_requests.write();
                requests.remove(&request_id);
                Err(ToolError::ExecutionFailed("响应通道已关闭".to_string()))
            }
            Err(_) => {
                // 超时
                warn!("[TermScrollbackTool] 请求超时: {}", request_id);
                let mut requests = self.pending_requests.write();
                requests.remove(&request_id);
                Err(ToolError::Timeout)
            }
        }
    }
}

impl Default for TermScrollbackTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for TermScrollbackTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "term_get_scrollback",
            "Get the terminal output history (scrollback buffer). This is a READ-ONLY tool \
             that allows you to view what has been output in the terminal.\n\n\
             IMPORTANT: This tool does NOT execute commands. It only reads the terminal output. \
             Use this to:\n\
             - Check the results of commands that the user has run\n\
             - View error messages and logs\n\
             - Understand the current state of the terminal\n\n\
             The user must manually execute commands in their terminal. You can suggest commands \
             for the user to run, but you cannot execute them directly with this tool.",
        )
        .with_parameters(
            JsonSchema::new()
                .add_property(
                    "session_id",
                    PropertySchema::string(
                        "The terminal session ID to read from. This is provided by the system.",
                    ),
                    true,
                )
                .add_property(
                    "line_start",
                    PropertySchema::integer(
                        "Optional starting line number. If not specified, returns the last 200 lines.",
                    ),
                    false,
                )
                .add_property(
                    "count",
                    PropertySchema::integer(
                        "Optional number of lines to return. Defaults to 200 lines.",
                    )
                    .with_default(serde_json::json!(200)),
                    false,
                ),
        )
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, ToolError> {
        // 解析参数
        let session_id = args
            .get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("缺少 session_id 参数".to_string()))?;

        let line_start = args
            .get("line_start")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);
        let count = args
            .get("count")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);

        // 获取滚动缓冲区
        let response = self.get_scrollback(session_id, line_start, count).await?;

        // 构建输出
        if response.success {
            let mut output = format!(
                "Terminal output (lines {}-{} of {}):\n\n",
                response.line_start, response.line_end, response.total_lines
            );
            output.push_str(&response.content);

            if response.has_more {
                output.push_str(
                    "\n\n[More output available. Use line_start parameter to fetch earlier lines.]",
                );
            }

            Ok(ToolResult::success(output))
        } else {
            let error_msg = response
                .error
                .unwrap_or_else(|| "Unknown error".to_string());
            Err(ToolError::ExecutionFailed(error_msg))
        }
    }
}

/// 全局 TermScrollbackTool 实例
static TERM_SCROLLBACK_TOOL: once_cell::sync::Lazy<Arc<TermScrollbackTool>> =
    once_cell::sync::Lazy::new(|| {
        eprintln!("[TermScrollbackTool] 创建全局实例");
        Arc::new(TermScrollbackTool::new())
    });

/// 获取全局 TermScrollbackTool 实例
pub fn get_term_scrollback_tool() -> Arc<TermScrollbackTool> {
    eprintln!("[TermScrollbackTool] 获取全局实例");
    Arc::clone(&TERM_SCROLLBACK_TOOL)
}

/// 设置全局 TermScrollbackTool 的 AppHandle
pub fn set_term_scrollback_tool_app_handle(handle: tauri::AppHandle) {
    eprintln!("[TermScrollbackTool] 设置全局 AppHandle");
    tracing::info!("[TermScrollbackTool] 设置全局 AppHandle");

    // 强制初始化全局实例（如果还未初始化）
    let _ = &*TERM_SCROLLBACK_TOOL;

    TERM_SCROLLBACK_TOOL.set_app_handle(handle);

    // 验证设置是否成功
    let app_handle = TERM_SCROLLBACK_TOOL.app_handle.read();
    if app_handle.is_some() {
        eprintln!("[TermScrollbackTool] AppHandle 设置成功，已验证");
        tracing::info!("[TermScrollbackTool] AppHandle 设置成功，已验证");
    } else {
        eprintln!("[TermScrollbackTool] 警告：AppHandle 设置后仍为 None");
        tracing::error!("[TermScrollbackTool] 警告：AppHandle 设置后仍为 None");
    }
}

/// 处理滚动缓冲区响应（由 Tauri 命令调用）
pub fn handle_term_scrollback_response(response: GetScrollbackResponse) {
    TERM_SCROLLBACK_TOOL.handle_response(response);
}
