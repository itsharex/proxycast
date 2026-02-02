//! 推理内容处理器
//!
//! 处理不同模型的推理/思考内容（reasoning_content）在多轮对话中的传递规则。
//!
//! # 支持的模型
//!
//! | 模型 | 字段名 | 多轮对话处理 |
//! |------|--------|--------------|
//! | DeepSeek R1/Reasoner | `reasoning_content` | 丢弃，只保留 `content` |
//! | OpenAI o1/o3/o4 | `reasoning` | 通过 `previous_response_id` 引用 |
//!
//! # 设计原则
//!
//! 根据 DeepSeek API 文档：
//! - 多轮对话时，`reasoning_content` 不应传递给下一轮
//! - 只有 `content` 字段需要保留在对话历史中
//! - Tool Calls 场景下，需要正确处理 reasoning_content 的传递
//!
//! # 使用状态
//!
//! 此模块为预留功能，将在 Proxy 层集成推理模型时启用。
//! 目前代码已完成，等待在 `proxy_handler.rs` 中调用 `ReasoningHandler::preprocess_messages`。

// 预留功能模块，暂未在主流程中调用
#![allow(dead_code)]

use crate::models::openai::ChatMessage;

/// 模型类型，用于确定推理内容处理策略
#[derive(Debug, Clone, PartialEq)]
pub enum ReasoningModelType {
    /// DeepSeek R1/Reasoner 系列
    DeepSeek,
    /// OpenAI o1/o3/o4 系列
    OpenAI,
    /// 其他模型（不处理）
    Other,
}

impl ReasoningModelType {
    /// 从模型名称检测模型类型
    pub fn from_model_name(model: &str) -> Self {
        let model_lower = model.to_lowercase();

        if model_lower.contains("deepseek")
            && (model_lower.contains("reasoner") || model_lower.contains("r1"))
        {
            Self::DeepSeek
        } else if model_lower.starts_with("o1")
            || model_lower.starts_with("o3")
            || model_lower.starts_with("o4")
        {
            Self::OpenAI
        } else {
            Self::Other
        }
    }
}

/// 推理内容处理器
pub struct ReasoningHandler;

impl ReasoningHandler {
    /// 预处理消息列表，根据模型类型清理 reasoning_content
    ///
    /// # DeepSeek 处理规则
    ///
    /// 根据 DeepSeek API 文档：
    /// - 多轮对话时，历史消息中的 `reasoning_content` 应该被丢弃
    /// - 只保留 `content` 字段用于上下文
    /// - 这样可以节省网络带宽，避免 400 错误
    ///
    /// # 参数
    ///
    /// - `messages`: 消息列表
    /// - `model`: 模型名称
    ///
    /// # 返回
    ///
    /// 处理后的消息列表
    pub fn preprocess_messages(messages: Vec<ChatMessage>, model: &str) -> Vec<ChatMessage> {
        let model_type = ReasoningModelType::from_model_name(model);

        match model_type {
            ReasoningModelType::DeepSeek => Self::process_deepseek_messages(messages),
            ReasoningModelType::OpenAI => Self::process_openai_messages(messages),
            ReasoningModelType::Other => messages,
        }
    }

    /// 处理 DeepSeek 消息
    ///
    /// 清除历史消息中的 reasoning_content，只保留最后一条 assistant 消息的 reasoning_content
    fn process_deepseek_messages(mut messages: Vec<ChatMessage>) -> Vec<ChatMessage> {
        // 先找出最后一条 assistant 消息的索引
        let last_assistant_idx = messages
            .iter()
            .enumerate()
            .rev()
            .find(|(_, m)| m.role == "assistant")
            .map(|(i, _)| i);

        for (i, msg) in messages.iter_mut().enumerate() {
            // 只处理 assistant 消息
            if msg.role != "assistant" {
                continue;
            }

            // 保留最后一条 assistant 消息的 reasoning_content（如果有 tool_calls）
            // 因为 DeepSeek 在 tool calls 场景下需要这个字段
            let is_last_assistant = Some(i) == last_assistant_idx;

            if !is_last_assistant {
                // 清除非最后一条 assistant 消息的 reasoning_content
                msg.reasoning_content = None;
            }
        }

        messages
    }

    /// 处理 OpenAI o1/o3 消息
    ///
    /// OpenAI 的推理模型使用 previous_response_id 机制，
    /// 目前不需要特殊处理消息内容
    fn process_openai_messages(messages: Vec<ChatMessage>) -> Vec<ChatMessage> {
        // OpenAI 推理模型目前不需要特殊处理
        // 未来可能需要处理 reasoning 字段
        messages
    }

    /// 检查模型是否支持推理模式
    pub fn supports_reasoning(model: &str) -> bool {
        let model_type = ReasoningModelType::from_model_name(model);
        !matches!(model_type, ReasoningModelType::Other)
    }

    /// 检查模型是否需要清理历史 reasoning_content
    pub fn needs_reasoning_cleanup(model: &str) -> bool {
        matches!(
            ReasoningModelType::from_model_name(model),
            ReasoningModelType::DeepSeek
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::openai::MessageContent;

    #[test]
    fn test_model_type_detection() {
        assert_eq!(
            ReasoningModelType::from_model_name("deepseek-reasoner"),
            ReasoningModelType::DeepSeek
        );
        assert_eq!(
            ReasoningModelType::from_model_name("deepseek-r1"),
            ReasoningModelType::DeepSeek
        );
        assert_eq!(
            ReasoningModelType::from_model_name("DeepSeek-R1-0528"),
            ReasoningModelType::DeepSeek
        );
        assert_eq!(
            ReasoningModelType::from_model_name("o1-preview"),
            ReasoningModelType::OpenAI
        );
        assert_eq!(
            ReasoningModelType::from_model_name("o3-mini"),
            ReasoningModelType::OpenAI
        );
        assert_eq!(
            ReasoningModelType::from_model_name("gpt-4o"),
            ReasoningModelType::Other
        );
        assert_eq!(
            ReasoningModelType::from_model_name("deepseek-chat"),
            ReasoningModelType::Other
        );
    }

    #[test]
    fn test_deepseek_reasoning_cleanup() {
        let messages = vec![
            ChatMessage {
                role: "user".to_string(),
                content: Some(MessageContent::Text("Hello".to_string())),
                tool_calls: None,
                tool_call_id: None,
                reasoning_content: None,
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: Some(MessageContent::Text("Hi there!".to_string())),
                tool_calls: None,
                tool_call_id: None,
                reasoning_content: Some("Thinking about greeting...".to_string()),
            },
            ChatMessage {
                role: "user".to_string(),
                content: Some(MessageContent::Text("How are you?".to_string())),
                tool_calls: None,
                tool_call_id: None,
                reasoning_content: None,
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: Some(MessageContent::Text("I'm doing well!".to_string())),
                tool_calls: None,
                tool_call_id: None,
                reasoning_content: Some("Thinking about response...".to_string()),
            },
        ];

        let processed = ReasoningHandler::preprocess_messages(messages, "deepseek-reasoner");

        // 第一条 assistant 消息的 reasoning_content 应该被清除
        assert!(processed[1].reasoning_content.is_none());
        // 最后一条 assistant 消息的 reasoning_content 应该保留
        assert!(processed[3].reasoning_content.is_some());
    }
}
