//! LLM Provider trait 和错误类型
//!
//! 定义 Skill 执行引擎调用 LLM 的接口。
//! 具体实现（ProxyCastLlmProvider）留在主 crate。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Skill 执行错误类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillError {
    ProviderError(String),
    ExecutionError(String),
    ConfigError(String),
}

impl std::fmt::Display for SkillError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillError::ProviderError(msg) => write!(f, "Provider error: {}", msg),
            SkillError::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
            SkillError::ConfigError(msg) => write!(f, "Config error: {}", msg),
        }
    }
}

impl std::error::Error for SkillError {}

/// LLM Provider Trait
///
/// 定义 Skill 执行引擎调用 LLM 的接口。
/// 应用层需要实现此 trait 以提供 LLM 调用能力。
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn chat(
        &self,
        system_prompt: &str,
        user_message: &str,
        model: Option<&str>,
    ) -> Result<String, SkillError>;
}
