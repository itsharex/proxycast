//! Aster 事件转换器
//!
//! 将 Aster AgentEvent 转换为 Tauri 可用的事件格式
//! 用于前端实时显示流式响应

use aster::agents::AgentEvent;
use aster::conversation::message::{ActionRequiredData, Message, MessageContent};
use serde::{Deserialize, Serialize};

/// 从工具结果中提取文本内容
///
/// 使用 serde_json 来处理，避免直接依赖 rmcp 类型
fn extract_tool_result_text<T: serde::Serialize>(result: &T) -> String {
    if let Ok(json) = serde_json::to_value(result) {
        if let Some(content) = json.get("content").and_then(|c| c.as_array()) {
            return content
                .iter()
                .filter_map(|item| {
                    if item.get("type").and_then(|t| t.as_str()) == Some("text") {
                        item.get("text").and_then(|t| t.as_str()).map(String::from)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
        }
    }
    String::new()
}

/// Tauri Agent 事件
///
/// 用于前端消费的事件格式，与现有的 StreamEvent 兼容
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TauriAgentEvent {
    /// 文本增量
    #[serde(rename = "text_delta")]
    TextDelta { text: String },

    /// 思考内容增量
    #[serde(rename = "thinking_delta")]
    ThinkingDelta { text: String },

    /// 工具调用开始
    #[serde(rename = "tool_start")]
    ToolStart {
        tool_name: String,
        tool_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        arguments: Option<String>,
    },

    /// 工具调用结束
    #[serde(rename = "tool_end")]
    ToolEnd {
        tool_id: String,
        result: TauriToolResult,
    },

    /// 需要用户操作（权限确认、用户输入等）
    #[serde(rename = "action_required")]
    ActionRequired {
        request_id: String,
        action_type: String,
        data: serde_json::Value,
    },

    /// 模型变更
    #[serde(rename = "model_change")]
    ModelChange { model: String, mode: String },

    /// 完成（单次响应完成）
    #[serde(rename = "done")]
    Done {
        #[serde(skip_serializing_if = "Option::is_none")]
        usage: Option<TauriTokenUsage>,
    },

    /// 最终完成（整个对话完成）
    #[serde(rename = "final_done")]
    FinalDone {
        #[serde(skip_serializing_if = "Option::is_none")]
        usage: Option<TauriTokenUsage>,
    },

    /// 错误
    #[serde(rename = "error")]
    Error { message: String },

    /// 完整消息（用于历史记录）
    #[serde(rename = "message")]
    Message { message: TauriMessage },
}

/// 工具执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TauriToolResult {
    pub success: bool,
    pub output: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Token 使用量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TauriTokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// 简化的消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TauriMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub role: String,
    pub content: Vec<TauriMessageContent>,
    pub timestamp: i64,
}

/// 简化的消息内容
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TauriMessageContent {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "thinking")]
    Thinking { text: String },

    #[serde(rename = "tool_request")]
    ToolRequest {
        id: String,
        tool_name: String,
        arguments: serde_json::Value,
    },

    #[serde(rename = "tool_response")]
    ToolResponse {
        id: String,
        success: bool,
        output: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },

    #[serde(rename = "action_required")]
    ActionRequired {
        id: String,
        action_type: String,
        data: serde_json::Value,
    },

    #[serde(rename = "image")]
    Image { mime_type: String, data: String },
}

/// 将 Aster AgentEvent 转换为 TauriAgentEvent 列表
///
/// 一个 AgentEvent 可能产生多个 TauriAgentEvent
pub fn convert_agent_event(event: AgentEvent) -> Vec<TauriAgentEvent> {
    match event {
        AgentEvent::Message(message) => convert_message(message),
        AgentEvent::McpNotification((server_name, notification)) => {
            // MCP 通知暂时忽略或转换为日志
            tracing::debug!("MCP notification from {}: {:?}", server_name, notification);
            vec![]
        }
        AgentEvent::ModelChange { model, mode } => {
            vec![TauriAgentEvent::ModelChange { model, mode }]
        }
        AgentEvent::HistoryReplaced(_conversation) => {
            // 历史替换事件，可能需要特殊处理
            tracing::debug!("History replaced");
            vec![]
        }
    }
}

/// 将 Aster Message 转换为 TauriAgentEvent 列表
fn convert_message(message: Message) -> Vec<TauriAgentEvent> {
    let mut events = Vec::new();

    for content in &message.content {
        match content {
            MessageContent::Text(text_content) => {
                events.push(TauriAgentEvent::TextDelta {
                    text: text_content.text.clone(),
                });
            }
            MessageContent::Thinking(thinking) => {
                events.push(TauriAgentEvent::ThinkingDelta {
                    text: thinking.thinking.clone(),
                });
            }
            MessageContent::ToolRequest(tool_request) => match &tool_request.tool_call {
                Ok(call) => {
                    events.push(TauriAgentEvent::ToolStart {
                        tool_name: call.name.to_string(),
                        tool_id: tool_request.id.clone(),
                        arguments: serde_json::to_string(&call.arguments).ok(),
                    });
                }
                Err(e) => {
                    events.push(TauriAgentEvent::Error {
                        message: format!("Invalid tool call: {e}"),
                    });
                }
            },
            MessageContent::ToolResponse(tool_response) => {
                let (success, output, error) = match &tool_response.tool_result {
                    Ok(result) => {
                        // 从 CallToolResult 中提取文本内容
                        let output = extract_tool_result_text(result);
                        (true, output, None)
                    }
                    Err(e) => (false, String::new(), Some(e.to_string())),
                };

                events.push(TauriAgentEvent::ToolEnd {
                    tool_id: tool_response.id.clone(),
                    result: TauriToolResult {
                        success,
                        output,
                        error,
                    },
                });
            }
            MessageContent::ActionRequired(action_required) => {
                let (request_id, action_type, data) = match &action_required.data {
                    ActionRequiredData::ToolConfirmation {
                        id,
                        tool_name,
                        arguments,
                        prompt,
                    } => (
                        id.clone(),
                        "tool_confirmation".to_string(),
                        serde_json::json!({
                            "tool_name": tool_name,
                            "arguments": arguments,
                            "prompt": prompt,
                        }),
                    ),
                    ActionRequiredData::Elicitation {
                        id,
                        message,
                        requested_schema,
                    } => (
                        id.clone(),
                        "elicitation".to_string(),
                        serde_json::json!({
                            "message": message,
                            "requested_schema": requested_schema,
                        }),
                    ),
                    ActionRequiredData::ElicitationResponse { id, user_data } => (
                        id.clone(),
                        "elicitation_response".to_string(),
                        serde_json::json!({
                            "user_data": user_data,
                        }),
                    ),
                };

                events.push(TauriAgentEvent::ActionRequired {
                    request_id,
                    action_type,
                    data,
                });
            }
            MessageContent::SystemNotification(notification) => {
                // 系统通知转换为文本
                events.push(TauriAgentEvent::TextDelta {
                    text: notification.msg.clone(),
                });
            }
            MessageContent::Image(image) => {
                // 图片内容暂时忽略
                tracing::debug!("Image content: {}", image.mime_type);
            }
            MessageContent::ToolConfirmationRequest(req) => {
                events.push(TauriAgentEvent::ActionRequired {
                    request_id: req.id.clone(),
                    action_type: "tool_confirmation".to_string(),
                    data: serde_json::json!({
                        "tool_name": req.tool_name,
                        "arguments": req.arguments,
                        "prompt": req.prompt,
                    }),
                });
            }
            MessageContent::FrontendToolRequest(req) => match &req.tool_call {
                Ok(call) => {
                    events.push(TauriAgentEvent::ToolStart {
                        tool_name: call.name.to_string(),
                        tool_id: req.id.clone(),
                        arguments: serde_json::to_string(&call.arguments).ok(),
                    });
                }
                Err(e) => {
                    events.push(TauriAgentEvent::Error {
                        message: format!("Invalid frontend tool call: {e}"),
                    });
                }
            },
            MessageContent::RedactedThinking(_) => {
                // 隐藏的思考内容，忽略
            }
        }
    }

    events
}

/// 将 Aster Message 转换为 TauriMessage
pub fn convert_to_tauri_message(message: &Message) -> TauriMessage {
    let content = message
        .content
        .iter()
        .filter_map(convert_message_content)
        .collect();

    TauriMessage {
        id: message.id.clone(),
        role: format!("{:?}", message.role).to_lowercase(),
        content,
        timestamp: message.created,
    }
}

/// 将 MessageContent 转换为 TauriMessageContent
fn convert_message_content(content: &MessageContent) -> Option<TauriMessageContent> {
    match content {
        MessageContent::Text(text) => Some(TauriMessageContent::Text {
            text: text.text.clone(),
        }),
        MessageContent::Thinking(thinking) => Some(TauriMessageContent::Thinking {
            text: thinking.thinking.clone(),
        }),
        MessageContent::ToolRequest(req) => {
            req.tool_call
                .as_ref()
                .ok()
                .map(|call| TauriMessageContent::ToolRequest {
                    id: req.id.clone(),
                    tool_name: call.name.to_string(),
                    arguments: serde_json::to_value(&call.arguments).unwrap_or_default(),
                })
        }
        MessageContent::ToolResponse(resp) => {
            let (success, output, error) = match &resp.tool_result {
                Ok(result) => {
                    let output = extract_tool_result_text(result);
                    (true, output, None)
                }
                Err(e) => (false, String::new(), Some(e.to_string())),
            };
            Some(TauriMessageContent::ToolResponse {
                id: resp.id.clone(),
                success,
                output,
                error,
            })
        }
        MessageContent::ActionRequired(action) => {
            let (id, action_type, data) = match &action.data {
                ActionRequiredData::ToolConfirmation {
                    id,
                    tool_name,
                    arguments,
                    prompt,
                } => (
                    id.clone(),
                    "tool_confirmation".to_string(),
                    serde_json::json!({
                        "tool_name": tool_name,
                        "arguments": arguments,
                        "prompt": prompt,
                    }),
                ),
                ActionRequiredData::Elicitation {
                    id,
                    message,
                    requested_schema,
                } => (
                    id.clone(),
                    "elicitation".to_string(),
                    serde_json::json!({
                        "message": message,
                        "requested_schema": requested_schema,
                    }),
                ),
                ActionRequiredData::ElicitationResponse { id, user_data } => (
                    id.clone(),
                    "elicitation_response".to_string(),
                    user_data.clone(),
                ),
            };
            Some(TauriMessageContent::ActionRequired {
                id,
                action_type,
                data,
            })
        }
        MessageContent::Image(image) => Some(TauriMessageContent::Image {
            mime_type: image.mime_type.clone(),
            data: image.data.clone(),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_text_delta() {
        let message = Message::assistant().with_text("Hello, world!");
        let events = convert_message(message);

        assert_eq!(events.len(), 1);
        match &events[0] {
            TauriAgentEvent::TextDelta { text } => {
                assert_eq!(text, "Hello, world!");
            }
            _ => panic!("Expected TextDelta event"),
        }
    }

    #[test]
    fn test_convert_model_change() {
        let event = AgentEvent::ModelChange {
            model: "claude-3".to_string(),
            mode: "chat".to_string(),
        };
        let events = convert_agent_event(event);

        assert_eq!(events.len(), 1);
        match &events[0] {
            TauriAgentEvent::ModelChange { model, mode } => {
                assert_eq!(model, "claude-3");
                assert_eq!(mode, "chat");
            }
            _ => panic!("Expected ModelChange event"),
        }
    }
}
