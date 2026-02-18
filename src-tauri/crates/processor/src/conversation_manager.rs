//! 对话历史管理器
//!
//! 提供对话历史修剪策略，防止上下文溢出

use serde::{Deserialize, Serialize};

/// 修剪策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrimStrategy {
    /// 丢弃最旧的消息
    DropOldest,
    /// 滑动窗口（保留最近 N 条）
    SlidingWindow,
}

impl Default for TrimStrategy {
    fn default() -> Self {
        Self::SlidingWindow
    }
}

/// 修剪配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrimConfig {
    /// 是否启用
    #[serde(default)]
    pub enabled: bool,
    /// 最大消息数
    #[serde(default = "default_max_messages")]
    pub max_messages: usize,
    /// 是否保留 system 提示
    #[serde(default = "default_preserve_system")]
    pub preserve_system_prompt: bool,
    /// 修剪策略
    #[serde(default)]
    pub strategy: TrimStrategy,
}

fn default_max_messages() -> usize {
    100
}
fn default_preserve_system() -> bool {
    true
}

impl Default for TrimConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_messages: default_max_messages(),
            preserve_system_prompt: default_preserve_system(),
            strategy: TrimStrategy::default(),
        }
    }
}

/// 修剪结果
#[derive(Debug)]
pub struct TrimResult {
    /// 修剪后的消息
    pub messages: Vec<serde_json::Value>,
    /// 是否进行了修剪
    pub trimmed: bool,
    /// 被移除的消息数
    pub removed_count: usize,
}

/// 对话修剪器
pub struct ConversationTrimmer {
    config: TrimConfig,
}

impl ConversationTrimmer {
    pub fn new(config: TrimConfig) -> Self {
        Self { config }
    }

    /// 修剪消息列表
    ///
    /// 兼容 Anthropic 和 OpenAI 消息格式（都使用 "role" 字段）
    pub fn trim_messages(&self, messages: Vec<serde_json::Value>) -> TrimResult {
        if !self.config.enabled || messages.len() <= self.config.max_messages {
            return TrimResult {
                messages,
                trimmed: false,
                removed_count: 0,
            };
        }

        let original_count = messages.len();

        // 分离 system 消息和非 system 消息
        let (system_msgs, non_system_msgs): (Vec<_>, Vec<_>) =
            if self.config.preserve_system_prompt {
                messages.into_iter().partition(|msg| {
                    msg.get("role")
                        .and_then(|r| r.as_str())
                        .map(|r| r == "system")
                        .unwrap_or(false)
                })
            } else {
                (Vec::new(), messages)
            };

        // 计算非 system 消息的最大数量
        let max_non_system = self.config.max_messages.saturating_sub(system_msgs.len());

        // 按策略修剪
        let trimmed_non_system = match self.config.strategy {
            TrimStrategy::DropOldest | TrimStrategy::SlidingWindow => {
                let len = non_system_msgs.len();
                if len > max_non_system {
                    non_system_msgs
                        .into_iter()
                        .skip(len - max_non_system)
                        .collect()
                } else {
                    non_system_msgs
                }
            }
        };

        // 合并：system 消息在前，非 system 消息在后
        let mut result = system_msgs;
        result.extend(trimmed_non_system);

        let removed_count = original_count - result.len();

        TrimResult {
            messages: result,
            trimmed: removed_count > 0,
            removed_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_msg(role: &str, content: &str) -> serde_json::Value {
        json!({ "role": role, "content": content })
    }

    #[test]
    fn test_disabled_no_trim() {
        let trimmer = ConversationTrimmer::new(TrimConfig {
            enabled: false,
            max_messages: 2,
            ..Default::default()
        });
        let msgs = vec![
            make_msg("user", "1"),
            make_msg("assistant", "2"),
            make_msg("user", "3"),
        ];
        let result = trimmer.trim_messages(msgs);
        assert!(!result.trimmed);
        assert_eq!(result.removed_count, 0);
        assert_eq!(result.messages.len(), 3);
    }

    #[test]
    fn test_within_limit_no_trim() {
        let trimmer = ConversationTrimmer::new(TrimConfig {
            enabled: true,
            max_messages: 5,
            ..Default::default()
        });
        let msgs = vec![make_msg("user", "1"), make_msg("assistant", "2")];
        let result = trimmer.trim_messages(msgs);
        assert!(!result.trimmed);
        assert_eq!(result.messages.len(), 2);
    }

    #[test]
    fn test_trim_preserves_system() {
        let trimmer = ConversationTrimmer::new(TrimConfig {
            enabled: true,
            max_messages: 3,
            preserve_system_prompt: true,
            strategy: TrimStrategy::SlidingWindow,
        });
        let msgs = vec![
            make_msg("system", "You are helpful"),
            make_msg("user", "1"),
            make_msg("assistant", "2"),
            make_msg("user", "3"),
            make_msg("assistant", "4"),
        ];
        let result = trimmer.trim_messages(msgs);
        assert!(result.trimmed);
        assert_eq!(result.removed_count, 2);
        assert_eq!(result.messages.len(), 3);
        assert_eq!(result.messages[0]["role"], "system");
        assert_eq!(result.messages[1]["content"], "3");
        assert_eq!(result.messages[2]["content"], "4");
    }

    #[test]
    fn test_trim_drops_oldest() {
        let trimmer = ConversationTrimmer::new(TrimConfig {
            enabled: true,
            max_messages: 2,
            preserve_system_prompt: false,
            strategy: TrimStrategy::DropOldest,
        });
        let msgs = vec![
            make_msg("user", "old"),
            make_msg("assistant", "old-reply"),
            make_msg("user", "new"),
            make_msg("assistant", "new-reply"),
        ];
        let result = trimmer.trim_messages(msgs);
        assert!(result.trimmed);
        assert_eq!(result.removed_count, 2);
        assert_eq!(result.messages.len(), 2);
        assert_eq!(result.messages[0]["content"], "new");
        assert_eq!(result.messages[1]["content"], "new-reply");
    }

    #[test]
    fn test_trim_with_no_system() {
        let trimmer = ConversationTrimmer::new(TrimConfig {
            enabled: true,
            max_messages: 2,
            preserve_system_prompt: true,
            strategy: TrimStrategy::SlidingWindow,
        });
        let msgs = vec![
            make_msg("user", "1"),
            make_msg("assistant", "2"),
            make_msg("user", "3"),
        ];
        let result = trimmer.trim_messages(msgs);
        assert!(result.trimmed);
        assert_eq!(result.removed_count, 1);
        assert_eq!(result.messages.len(), 2);
        assert_eq!(result.messages[0]["content"], "2");
        assert_eq!(result.messages[1]["content"], "3");
    }

    #[test]
    fn test_trim_all_system_messages_preserved() {
        let trimmer = ConversationTrimmer::new(TrimConfig {
            enabled: true,
            max_messages: 3,
            preserve_system_prompt: true,
            strategy: TrimStrategy::SlidingWindow,
        });
        let msgs = vec![
            make_msg("system", "sys1"),
            make_msg("system", "sys2"),
            make_msg("user", "1"),
            make_msg("assistant", "2"),
            make_msg("user", "3"),
        ];
        let result = trimmer.trim_messages(msgs);
        assert!(result.trimmed);
        assert_eq!(result.messages.len(), 3);
        assert_eq!(result.messages[0]["role"], "system");
        assert_eq!(result.messages[1]["role"], "system");
        assert_eq!(result.messages[2]["content"], "3");
    }

    #[test]
    fn test_openai_format_compatibility() {
        let trimmer = ConversationTrimmer::new(TrimConfig {
            enabled: true,
            max_messages: 2,
            preserve_system_prompt: true,
            strategy: TrimStrategy::SlidingWindow,
        });
        // OpenAI 格式：system/user/assistant + content 字符串
        let msgs = vec![
            json!({ "role": "system", "content": "You are a helpful assistant." }),
            json!({ "role": "user", "content": "Hello" }),
            json!({ "role": "assistant", "content": "Hi there!" }),
            json!({ "role": "user", "content": "How are you?" }),
        ];
        let result = trimmer.trim_messages(msgs);
        assert!(result.trimmed);
        assert_eq!(result.messages.len(), 2);
        assert_eq!(result.messages[0]["role"], "system");
        assert_eq!(result.messages[1]["content"], "How are you?");
    }

    #[test]
    fn test_anthropic_format_compatibility() {
        let trimmer = ConversationTrimmer::new(TrimConfig {
            enabled: true,
            max_messages: 3,
            preserve_system_prompt: true,
            strategy: TrimStrategy::SlidingWindow,
        });
        // Anthropic 格式：content 可以是数组
        let msgs = vec![
            json!({ "role": "system", "content": "You are Claude." }),
            json!({ "role": "user", "content": [{"type": "text", "text": "msg1"}] }),
            json!({ "role": "assistant", "content": [{"type": "text", "text": "reply1"}] }),
            json!({ "role": "user", "content": [{"type": "text", "text": "msg2"}] }),
            json!({ "role": "assistant", "content": [{"type": "text", "text": "reply2"}] }),
        ];
        let result = trimmer.trim_messages(msgs);
        assert!(result.trimmed);
        assert_eq!(result.messages.len(), 3);
        assert_eq!(result.messages[0]["role"], "system");
        assert_eq!(result.messages[1]["role"], "user");
        assert_eq!(result.messages[2]["role"], "assistant");
        assert_eq!(result.messages[2]["content"][0]["text"], "reply2");
    }
}