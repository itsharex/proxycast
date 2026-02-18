//! 对话摘要器
//!
//! 当对话历史过长时，使用 LLM 生成简洁摘要替代旧消息，
//! 保留关键上下文同时减少 token 消耗。

use serde::{Deserialize, Serialize};

/// 摘要配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryConfig {
    /// 是否启用
    #[serde(default)]
    pub enabled: bool,
    /// 触发摘要的消息数阈值
    #[serde(default = "default_threshold")]
    pub threshold_messages: usize,
    /// 摘要后保留的最近消息数
    #[serde(default = "default_keep_recent")]
    pub keep_recent_messages: usize,
    /// 摘要最大要点数
    #[serde(default = "default_max_points")]
    pub max_summary_points: usize,
}

fn default_threshold() -> usize {
    50
}
fn default_keep_recent() -> usize {
    20
}
fn default_max_points() -> usize {
    12
}

impl Default for SummaryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold_messages: default_threshold(),
            keep_recent_messages: default_keep_recent(),
            max_summary_points: default_max_points(),
        }
    }
}

/// 摘要请求
///
/// 包含需要发送给 LLM 的摘要请求信息
#[derive(Debug, Clone)]
pub struct SummaryRequest {
    /// 摘要 prompt（system 消息）
    pub system_prompt: String,
    /// 需要摘要的消息（作为 user 消息发送）
    pub messages_to_summarize: String,
}

/// 摘要结果
#[derive(Debug, Clone)]
pub struct SummaryResult {
    /// 摘要后的消息列表（摘要 system 消息 + 保留的最近消息）
    pub messages: Vec<serde_json::Value>,
    /// 是否进行了摘要
    pub summarized: bool,
    /// 被摘要的消息数
    pub summarized_count: usize,
}

/// 对话摘要器
pub struct ConversationSummarizer {
    config: SummaryConfig,
}

impl ConversationSummarizer {
    pub fn new(config: SummaryConfig) -> Self {
        Self { config }
    }

    /// 判断是否需要摘要
    pub fn should_summarize(&self, message_count: usize) -> bool {
        self.config.enabled && message_count > self.config.threshold_messages
    }

    /// 构建摘要请求
    ///
    /// 将需要摘要的旧消息格式化为 LLM 请求
    pub fn build_summary_request(
        &self,
        messages: &[serde_json::Value],
    ) -> Option<SummaryRequest> {
        if !self.should_summarize(messages.len()) {
            return None;
        }

        let non_system_msgs: Vec<_> = messages
            .iter()
            .filter(|msg| {
                msg.get("role")
                    .and_then(|r| r.as_str())
                    .map(|r| r != "system")
                    .unwrap_or(true)
            })
            .collect();

        let keep = self.config.keep_recent_messages.min(non_system_msgs.len());
        let to_summarize = non_system_msgs.len().saturating_sub(keep);

        if to_summarize == 0 {
            return None;
        }

        let msgs_text: Vec<String> = non_system_msgs[..to_summarize]
            .iter()
            .map(|msg| {
                let role = msg
                    .get("role")
                    .and_then(|r| r.as_str())
                    .unwrap_or("unknown");
                let content = extract_content_text(msg);
                format!("[{role}]: {content}")
            })
            .collect();

        let messages_text = msgs_text.join("\n\n");

        let system_prompt = format!(
            "你是一个对话摘要助手。请将以下对话历史总结为最多 {} 个关键要点。\n\
             要求：\n\
             - 保留重要的决策、结论和上下文\n\
             - 保留关键的技术细节和代码引用\n\
             - 使用简洁的要点格式\n\
             - 按时间顺序组织\n\
             - 不要遗漏用户的关键需求",
            self.config.max_summary_points
        );

        Some(SummaryRequest {
            system_prompt,
            messages_to_summarize: messages_text,
        })
    }

    /// 将摘要文本组装为最终消息列表
    ///
    /// 结构：原始 system 消息 + 摘要 system 消息 + 保留的最近消息
    pub fn assemble_with_summary(
        &self,
        original_messages: &[serde_json::Value],
        summary_text: &str,
    ) -> SummaryResult {
        let (system_msgs, non_system_msgs): (Vec<_>, Vec<_>) =
            original_messages.iter().partition(|msg| {
                msg.get("role")
                    .and_then(|r| r.as_str())
                    .map(|r| r == "system")
                    .unwrap_or(false)
            });

        let keep = self.config.keep_recent_messages.min(non_system_msgs.len());
        let summarized_count = non_system_msgs.len().saturating_sub(keep);

        let mut result = Vec::new();

        // 1. 原始 system 消息
        for msg in &system_msgs {
            result.push((*msg).clone());
        }

        // 2. 摘要 system 消息
        if summarized_count > 0 && !summary_text.is_empty() {
            result.push(serde_json::json!({
                "role": "system",
                "content": format!(
                    "[对话摘要 - 以下是之前 {} 条消息的摘要]\n\n{}",
                    summarized_count, summary_text
                )
            }));
        }

        // 3. 保留的最近消息
        let start = non_system_msgs.len().saturating_sub(keep);
        for msg in &non_system_msgs[start..] {
            result.push((*msg).clone());
        }

        SummaryResult {
            messages: result,
            summarized: summarized_count > 0,
            summarized_count,
        }
    }
}

/// 从消息中提取文本内容
///
/// 兼容 OpenAI 格式（content 为字符串）和 Anthropic 格式（content 为数组）
fn extract_content_text(msg: &serde_json::Value) -> String {
    match msg.get("content") {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|item| {
                if item.get("type").and_then(|t| t.as_str()) == Some("text") {
                    item.get("text").and_then(|t| t.as_str()).map(String::from)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_default_config() {
        let config = SummaryConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.threshold_messages, 50);
        assert_eq!(config.keep_recent_messages, 20);
        assert_eq!(config.max_summary_points, 12);
    }

    #[test]
    fn test_should_summarize_disabled() {
        let s = ConversationSummarizer::new(SummaryConfig {
            enabled: false,
            threshold_messages: 5,
            ..Default::default()
        });
        assert!(!s.should_summarize(100));
    }

    #[test]
    fn test_should_summarize_below_threshold() {
        let s = ConversationSummarizer::new(SummaryConfig {
            enabled: true,
            threshold_messages: 50,
            ..Default::default()
        });
        assert!(!s.should_summarize(30));
        assert!(!s.should_summarize(50)); // 等于阈值不触发
    }

    #[test]
    fn test_should_summarize_above_threshold() {
        let s = ConversationSummarizer::new(SummaryConfig {
            enabled: true,
            threshold_messages: 5,
            ..Default::default()
        });
        assert!(s.should_summarize(6));
        assert!(s.should_summarize(100));
    }

    #[test]
    fn test_build_summary_request_none_when_disabled() {
        let s = ConversationSummarizer::new(SummaryConfig {
            enabled: false,
            threshold_messages: 2,
            keep_recent_messages: 1,
            ..Default::default()
        });
        let msgs = vec![
            json!({"role": "user", "content": "a"}),
            json!({"role": "assistant", "content": "b"}),
            json!({"role": "user", "content": "c"}),
        ];
        assert!(s.build_summary_request(&msgs).is_none());
    }

    #[test]
    fn test_build_summary_request_with_messages() {
        let s = ConversationSummarizer::new(SummaryConfig {
            enabled: true,
            threshold_messages: 2,
            keep_recent_messages: 1,
            max_summary_points: 5,
        });
        let msgs = vec![
            json!({"role": "system", "content": "You are helpful."}),
            json!({"role": "user", "content": "Hello"}),
            json!({"role": "assistant", "content": "Hi!"}),
            json!({"role": "user", "content": "Latest"}),
        ];
        // 总消息数 4 > threshold 2，非 system 消息 3 条，保留 1 条，摘要 2 条
        let req = s.build_summary_request(&msgs).unwrap();
        assert!(req.system_prompt.contains("5"));
        assert!(req.messages_to_summarize.contains("[user]: Hello"));
        assert!(req.messages_to_summarize.contains("[assistant]: Hi!"));
        assert!(!req.messages_to_summarize.contains("Latest"));
    }

    #[test]
    fn test_assemble_with_summary() {
        let s = ConversationSummarizer::new(SummaryConfig {
            enabled: true,
            threshold_messages: 2,
            keep_recent_messages: 1,
            ..Default::default()
        });
        let msgs = vec![
            json!({"role": "user", "content": "old1"}),
            json!({"role": "assistant", "content": "old2"}),
            json!({"role": "user", "content": "recent"}),
        ];
        let result = s.assemble_with_summary(&msgs, "摘要内容");
        assert!(result.summarized);
        assert_eq!(result.summarized_count, 2);
        // 摘要 system + 保留的 1 条 = 2
        assert_eq!(result.messages.len(), 2);
        assert_eq!(result.messages[0]["role"], "system");
        assert!(result.messages[0]["content"]
            .as_str()
            .unwrap()
            .contains("摘要内容"));
        assert_eq!(result.messages[1]["content"], "recent");
    }

    #[test]
    fn test_assemble_preserves_system_messages() {
        let s = ConversationSummarizer::new(SummaryConfig {
            enabled: true,
            threshold_messages: 2,
            keep_recent_messages: 1,
            ..Default::default()
        });
        let msgs = vec![
            json!({"role": "system", "content": "You are helpful."}),
            json!({"role": "user", "content": "old"}),
            json!({"role": "assistant", "content": "old reply"}),
            json!({"role": "user", "content": "recent"}),
        ];
        let result = s.assemble_with_summary(&msgs, "summary");
        // system 原始 + 摘要 system + 保留 1 条 = 3
        assert_eq!(result.messages.len(), 3);
        assert_eq!(result.messages[0]["content"], "You are helpful.");
        assert_eq!(result.messages[1]["role"], "system");
        assert!(result.messages[1]["content"]
            .as_str()
            .unwrap()
            .contains("summary"));
        assert_eq!(result.messages[2]["content"], "recent");
    }

    #[test]
    fn test_extract_content_text_string() {
        let msg = json!({"role": "user", "content": "hello world"});
        assert_eq!(extract_content_text(&msg), "hello world");
    }

    #[test]
    fn test_extract_content_text_array() {
        // Anthropic 格式
        let msg = json!({
            "role": "user",
            "content": [
                {"type": "text", "text": "part1"},
                {"type": "image", "source": {}},
                {"type": "text", "text": "part2"}
            ]
        });
        assert_eq!(extract_content_text(&msg), "part1\npart2");
    }

    #[test]
    fn test_extract_content_text_empty() {
        let msg = json!({"role": "user"});
        assert_eq!(extract_content_text(&msg), "");

        let msg2 = json!({"role": "user", "content": null});
        assert_eq!(extract_content_text(&msg2), "");

        let msg3 = json!({"role": "user", "content": 42});
        assert_eq!(extract_content_text(&msg3), "");
    }
}
