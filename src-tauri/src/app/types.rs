//! 核心类型定义
//!
//! 包含 Provider 类型枚举和相关实现。

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::Runtime;
use tokio::sync::RwLock;

use crate::logger;
use crate::server;
use crate::services::token_cache_service::TokenCacheService;
use crate::tray::TrayManager;

/// Provider 类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    Kiro,
    Gemini,
    Qwen,
    #[serde(rename = "openai")]
    OpenAI,
    Claude,
    Antigravity,
    Vertex,
    #[serde(rename = "gemini_api_key")]
    GeminiApiKey,
    Codex,
    #[serde(rename = "claude_oauth")]
    ClaudeOAuth,
    #[serde(rename = "iflow")]
    IFlow,
    // API Key Provider 类型
    Anthropic,
    #[serde(rename = "azure_openai")]
    AzureOpenai,
    #[serde(rename = "aws_bedrock")]
    AwsBedrock,
    Ollama,
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderType::Kiro => write!(f, "kiro"),
            ProviderType::Gemini => write!(f, "gemini"),
            ProviderType::Qwen => write!(f, "qwen"),
            ProviderType::OpenAI => write!(f, "openai"),
            ProviderType::Claude => write!(f, "claude"),
            ProviderType::Antigravity => write!(f, "antigravity"),
            ProviderType::Vertex => write!(f, "vertex"),
            ProviderType::GeminiApiKey => write!(f, "gemini_api_key"),
            ProviderType::Codex => write!(f, "codex"),
            ProviderType::ClaudeOAuth => write!(f, "claude_oauth"),
            ProviderType::IFlow => write!(f, "iflow"),
            ProviderType::Anthropic => write!(f, "anthropic"),
            ProviderType::AzureOpenai => write!(f, "azure_openai"),
            ProviderType::AwsBedrock => write!(f, "aws_bedrock"),
            ProviderType::Ollama => write!(f, "ollama"),
        }
    }
}

impl std::str::FromStr for ProviderType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "kiro" => Ok(ProviderType::Kiro),
            "gemini" => Ok(ProviderType::Gemini),
            "qwen" => Ok(ProviderType::Qwen),
            "openai" => Ok(ProviderType::OpenAI),
            "claude" => Ok(ProviderType::Claude),
            "antigravity" => Ok(ProviderType::Antigravity),
            "vertex" => Ok(ProviderType::Vertex),
            "gemini_api_key" => Ok(ProviderType::GeminiApiKey),
            "codex" => Ok(ProviderType::Codex),
            "claude_oauth" => Ok(ProviderType::ClaudeOAuth),
            "iflow" => Ok(ProviderType::IFlow),
            "anthropic" => Ok(ProviderType::Anthropic),
            "azure_openai" | "azure-openai" => Ok(ProviderType::AzureOpenai),
            "aws_bedrock" | "aws-bedrock" => Ok(ProviderType::AwsBedrock),
            "ollama" => Ok(ProviderType::Ollama),
            _ => Err(format!("Invalid provider: {s}")),
        }
    }
}

/// 应用状态类型别名
pub type AppState = Arc<RwLock<server::ServerState>>;

/// 日志状态类型别名
pub type LogState = Arc<RwLock<logger::LogStore>>;

/// TokenCacheService 状态封装
pub struct TokenCacheServiceState(pub Arc<TokenCacheService>);

/// TrayManager 状态封装
pub struct TrayManagerState<R: Runtime>(pub Arc<tokio::sync::RwLock<Option<TrayManager<R>>>>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type_from_str() {
        assert_eq!("kiro".parse::<ProviderType>().unwrap(), ProviderType::Kiro);
        assert_eq!(
            "gemini".parse::<ProviderType>().unwrap(),
            ProviderType::Gemini
        );
        assert_eq!("qwen".parse::<ProviderType>().unwrap(), ProviderType::Qwen);
        assert_eq!(
            "openai".parse::<ProviderType>().unwrap(),
            ProviderType::OpenAI
        );
        assert_eq!(
            "claude".parse::<ProviderType>().unwrap(),
            ProviderType::Claude
        );
        assert_eq!(
            "vertex".parse::<ProviderType>().unwrap(),
            ProviderType::Vertex
        );
        assert_eq!(
            "gemini_api_key".parse::<ProviderType>().unwrap(),
            ProviderType::GeminiApiKey
        );
        assert_eq!("KIRO".parse::<ProviderType>().unwrap(), ProviderType::Kiro);
        assert_eq!(
            "Gemini".parse::<ProviderType>().unwrap(),
            ProviderType::Gemini
        );
        assert_eq!(
            "VERTEX".parse::<ProviderType>().unwrap(),
            ProviderType::Vertex
        );
        assert!("invalid".parse::<ProviderType>().is_err());
    }

    #[test]
    fn test_provider_type_display() {
        assert_eq!(ProviderType::Kiro.to_string(), "kiro");
        assert_eq!(ProviderType::Gemini.to_string(), "gemini");
        assert_eq!(ProviderType::Qwen.to_string(), "qwen");
        assert_eq!(ProviderType::OpenAI.to_string(), "openai");
        assert_eq!(ProviderType::Claude.to_string(), "claude");
        assert_eq!(ProviderType::Vertex.to_string(), "vertex");
        assert_eq!(ProviderType::GeminiApiKey.to_string(), "gemini_api_key");
    }

    #[test]
    fn test_provider_type_serde() {
        assert_eq!(
            serde_json::to_string(&ProviderType::Kiro).unwrap(),
            "\"kiro\""
        );
        assert_eq!(
            serde_json::to_string(&ProviderType::OpenAI).unwrap(),
            "\"openai\""
        );
        assert_eq!(
            serde_json::from_str::<ProviderType>("\"kiro\"").unwrap(),
            ProviderType::Kiro
        );
        assert_eq!(
            serde_json::from_str::<ProviderType>("\"openai\"").unwrap(),
            ProviderType::OpenAI
        );
    }
}
