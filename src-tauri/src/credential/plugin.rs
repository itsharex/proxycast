//! OAuth 凭证提供商插件 Trait
//!
//! 定义 OAuth Provider 插件必须实现的接口，支持动态注册和独立更新。
//! 设计原则：
//! - 不依赖任何硬编码枚举
//! - 新增 Provider 只需实现此 trait 并注册
//! - 凭证配置由插件自己定义 Schema
//! - 一个插件可支持多种认证方式（OAuth、API Key、第三方中转）

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

/// OAuth Provider 插件错误类型
#[derive(Error, Debug)]
pub enum OAuthPluginError {
    #[error("凭证获取失败: {0}")]
    AcquireError(String),

    #[error("凭证释放失败: {0}")]
    ReleaseError(String),

    #[error("Token 刷新失败: {0}")]
    TokenRefreshError(String),

    #[error("凭证验证失败: {0}")]
    ValidationError(String),

    #[error("配置解析失败: {0}")]
    ConfigParseError(String),

    #[error("协议转换失败: {0}")]
    TransformError(String),

    #[error("风控检查失败: {0}")]
    RiskControlError(String),

    #[error("模型不支持: {0}")]
    UnsupportedModel(String),

    #[error("插件初始化失败: {0}")]
    InitError(String),

    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON 解析错误: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type OAuthPluginResult<T> = Result<T, OAuthPluginError>;

// ============================================================================
// 认证类型信息
// ============================================================================

/// 凭证分组（用于 UI Tab 展示）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CredentialCategory {
    /// OAuth 凭证 Tab
    #[default]
    OAuth,
    /// API Key Tab
    ApiKey,
    /// 其他配置 Tab（第三方中转、Cookie 等）
    Other,
}

/// 认证方式信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTypeInfo {
    /// 认证方式 ID（如 "oauth", "api_key", "third_party"）
    pub id: String,
    /// 显示名称（如 "OAuth 登录", "官方 API Key", "第三方中转"）
    pub display_name: String,
    /// 描述（如 "使用官方 OAuth 授权"）
    pub description: String,
    /// UI 分组（显示在哪个 Tab）
    pub category: CredentialCategory,
    /// 图标名称 (Lucide icon)
    #[serde(default)]
    pub icon: Option<String>,
}

// ============================================================================
// 模型家族定义
// ============================================================================

/// 模型家族（用于 Mini/Pro/Max 分层）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFamily {
    /// 家族名称（如 "opus", "sonnet", "haiku"）
    pub name: String,
    /// 匹配模式（如 "claude-opus-*", "claude-*-sonnet"）
    pub pattern: String,
    /// 服务等级（1=Mini, 2=Pro, 3=Max）
    #[serde(default)]
    pub tier: Option<u8>,
    /// 描述
    #[serde(default)]
    pub description: Option<String>,
}

/// 模型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// 模型 ID
    pub id: String,
    /// 显示名称
    pub display_name: String,
    /// 模型家族
    #[serde(default)]
    pub family: Option<String>,
    /// 上下文长度
    #[serde(default)]
    pub context_length: Option<u32>,
    /// 是否支持视觉
    #[serde(default)]
    pub supports_vision: bool,
    /// 是否支持工具调用
    #[serde(default)]
    pub supports_tools: bool,
    /// 输入价格（每 1M tokens）
    #[serde(default)]
    pub input_cost_per_million: Option<f64>,
    /// 输出价格（每 1M tokens）
    #[serde(default)]
    pub output_cost_per_million: Option<f64>,
}

// ============================================================================
// 凭证配置 Trait
// ============================================================================

/// 凭证配置 trait（代替 CredentialData 枚举）
///
/// 每个插件自己定义凭证配置结构
pub trait CredentialConfig: Send + Sync + Any {
    /// 转换为 Any，用于向下转型
    fn as_any(&self) -> &dyn Any;

    /// 凭证类型（如 "oauth", "api_key", "third_party"）
    fn credential_type(&self) -> &str;

    /// 序列化为 JSON
    fn to_json(&self) -> serde_json::Value;

    /// 克隆
    fn clone_box(&self) -> Box<dyn CredentialConfig>;
}

// ============================================================================
// 凭证获取结果
// ============================================================================

/// 获取的凭证
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcquiredCredential {
    /// 凭证 ID
    pub id: String,
    /// 凭证名称
    #[serde(default)]
    pub name: Option<String>,
    /// 认证方式
    pub auth_type: String,
    /// Base URL（如果有）
    #[serde(default)]
    pub base_url: Option<String>,
    /// 请求头（Key-Value 对）
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// 额外元数据
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 凭证使用结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsageResult {
    /// 成功使用
    Success {
        /// 延迟（毫秒）
        latency_ms: u64,
        /// 输入 tokens
        input_tokens: Option<u32>,
        /// 输出 tokens
        output_tokens: Option<u32>,
    },
    /// 使用失败
    Error {
        /// 错误类型
        error_type: String,
        /// 错误消息
        message: String,
        /// 是否应标记为不健康
        mark_unhealthy: bool,
        /// 冷却时间（秒）
        cooldown_seconds: Option<u64>,
    },
}

/// Token 刷新结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRefreshResult {
    /// 新的 access_token
    pub access_token: String,
    /// 新的 refresh_token（如果更新了）
    #[serde(default)]
    pub refresh_token: Option<String>,
    /// 过期时间
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
}

/// 凭证验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// 是否有效
    pub valid: bool,
    /// 消息
    #[serde(default)]
    pub message: Option<String>,
    /// 额外信息
    #[serde(default)]
    pub details: HashMap<String, serde_json::Value>,
}

// ============================================================================
// Provider 错误解析
// ============================================================================

/// Provider 错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderError {
    /// 错误类型
    pub error_type: ProviderErrorType,
    /// 错误消息
    pub message: String,
    /// HTTP 状态码
    #[serde(default)]
    pub status_code: Option<u16>,
    /// 是否可重试
    #[serde(default)]
    pub retryable: bool,
    /// 建议的冷却时间（秒）
    #[serde(default)]
    pub cooldown_seconds: Option<u64>,
}

/// Provider 错误类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderErrorType {
    /// 认证错误（Token 无效、过期等）
    Authentication,
    /// 授权错误（无权限）
    Authorization,
    /// 限流
    RateLimit,
    /// 配额超限
    QuotaExceeded,
    /// 模型不可用
    ModelUnavailable,
    /// 内容安全过滤
    ContentFiltered,
    /// 服务器错误
    ServerError,
    /// 网络错误
    NetworkError,
    /// 未知错误
    Unknown,
}

// ============================================================================
// 输出协议
// ============================================================================

/// 目标标准协议
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StandardProtocol {
    /// Anthropic Claude API
    Anthropic,
    /// OpenAI Chat Completions API
    OpenAI,
    /// Google Gemini API
    Gemini,
    /// 通义千问
    Qwen,
    /// 其他 OpenAI 兼容
    OpenAICompat,
}

impl StandardProtocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            StandardProtocol::Anthropic => "anthropic",
            StandardProtocol::OpenAI => "openai",
            StandardProtocol::Gemini => "gemini",
            StandardProtocol::Qwen => "qwen",
            StandardProtocol::OpenAICompat => "openai_compat",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "anthropic" => Some(StandardProtocol::Anthropic),
            "openai" => Some(StandardProtocol::OpenAI),
            "gemini" => Some(StandardProtocol::Gemini),
            "qwen" => Some(StandardProtocol::Qwen),
            "openai_compat" => Some(StandardProtocol::OpenAICompat),
            _ => None,
        }
    }
}

// ============================================================================
// 主 Trait：CredentialProviderPlugin
// ============================================================================

/// 凭证提供商插件 - 核心 Trait
///
/// 设计原则：
/// - 不依赖任何硬编码枚举
/// - 新增 Provider 只需实现此 trait 并注册
/// - 凭证配置由插件自己定义 Schema
/// - 一个插件可支持多种认证方式（OAuth、API Key、第三方中转）
#[async_trait]
pub trait CredentialProviderPlugin: Send + Sync {
    // ========== 基础信息 ==========

    /// 插件唯一标识（代替 ProviderType 枚举）
    fn id(&self) -> &str;

    /// 显示名称
    fn display_name(&self) -> &str;

    /// 插件版本
    fn version(&self) -> &str;

    /// 插件描述
    fn description(&self) -> &str {
        ""
    }

    /// 默认目标标准协议
    fn target_protocol(&self) -> StandardProtocol;

    /// 根据模型动态返回目标协议（用于 Antigravity 等多协议 Provider）
    fn target_protocol_for_model(&self, _model: &str) -> StandardProtocol {
        self.target_protocol() // 默认返回固定协议
    }

    /// UI 分组
    fn ui_category(&self) -> CredentialCategory {
        CredentialCategory::OAuth
    }

    // ========== 多认证方式支持 ==========

    /// 支持的认证方式（一个插件可支持多种）
    /// 例如 Anthropic 同时支持 OAuth、API Key、第三方中转
    fn supported_auth_types(&self) -> Vec<AuthTypeInfo>;

    /// 根据认证方式返回对应的凭证配置 Schema
    fn credential_schema_for_auth(&self, auth_type: &str) -> serde_json::Value;

    /// 解析凭证配置（从 JSON 解析成插件内部结构）
    fn parse_credential_config(
        &self,
        auth_type: &str,
        config: serde_json::Value,
    ) -> OAuthPluginResult<Box<dyn CredentialConfig>>;

    /// 创建凭证（从用户输入创建）
    async fn create_credential(
        &self,
        auth_type: &str,
        config: serde_json::Value,
    ) -> OAuthPluginResult<String>;

    // ========== 模型能力 ==========

    /// 模型家族定义（用于 Mini/Pro/Max 分层）
    fn model_families(&self) -> Vec<ModelFamily>;

    /// 获取支持的模型列表
    async fn list_models(&self) -> OAuthPluginResult<Vec<ModelInfo>>;

    /// 检查是否支持某个模型
    fn supports_model(&self, model: &str) -> bool;

    // ========== 凭证管理 ==========

    /// 获取可用凭证
    async fn acquire_credential(&self, model: &str) -> OAuthPluginResult<AcquiredCredential>;

    /// 释放凭证（报告使用结果）
    async fn release_credential(
        &self,
        credential_id: &str,
        result: UsageResult,
    ) -> OAuthPluginResult<()>;

    /// 验证凭证有效性
    async fn validate_credential(&self, credential_id: &str)
        -> OAuthPluginResult<ValidationResult>;

    /// 刷新 Token（OAuth 类型）
    async fn refresh_token(&self, credential_id: &str) -> OAuthPluginResult<TokenRefreshResult>;

    // ========== 协议转换 ==========

    /// 将输入请求转换成标准协议
    async fn transform_request(&self, request: &mut serde_json::Value) -> OAuthPluginResult<()>;

    /// 将响应转换回来（如果需要）
    async fn transform_response(&self, response: &mut serde_json::Value) -> OAuthPluginResult<()>;

    // ========== 风控适配 ==========

    /// 应用特有的风控逻辑
    async fn apply_risk_control(
        &self,
        request: &mut serde_json::Value,
        credential_id: &str,
    ) -> OAuthPluginResult<()>;

    /// 解析特有的错误码
    fn parse_error(&self, status: u16, body: &str) -> Option<ProviderError>;

    // ========== 插件配置（非凭证配置）==========

    /// 插件配置 Schema（用于 UI 动态生成表单）
    fn plugin_config_schema(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    /// 获取插件配置
    fn get_plugin_config(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    /// 更新插件配置
    async fn update_plugin_config(&self, _config: serde_json::Value) -> OAuthPluginResult<()> {
        Ok(())
    }

    // ========== 生命周期 ==========

    /// 初始化插件
    async fn init(&self) -> OAuthPluginResult<()>;

    /// 关闭插件
    async fn shutdown(&self) -> OAuthPluginResult<()>;
}

/// 插件实例类型别名
pub type PluginInstance = Arc<dyn CredentialProviderPlugin>;

// ============================================================================
// 插件信息（用于 UI 显示）
// ============================================================================

/// 插件信息（用于 UI 显示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthPluginInfo {
    /// 插件 ID
    pub id: String,
    /// 显示名称
    pub display_name: String,
    /// 版本
    pub version: String,
    /// 描述
    pub description: String,
    /// 目标协议
    pub target_protocol: String,
    /// UI 分组
    pub category: CredentialCategory,
    /// 支持的认证方式
    pub auth_types: Vec<AuthTypeInfo>,
    /// 是否启用
    pub enabled: bool,
    /// 凭证数量
    pub credential_count: u32,
    /// 健康凭证数量
    pub healthy_credential_count: u32,
}

impl OAuthPluginInfo {
    /// 从插件实例创建信息
    pub fn from_plugin(plugin: &dyn CredentialProviderPlugin) -> Self {
        Self {
            id: plugin.id().to_string(),
            display_name: plugin.display_name().to_string(),
            version: plugin.version().to_string(),
            description: plugin.description().to_string(),
            target_protocol: plugin.target_protocol().as_str().to_string(),
            category: plugin.ui_category(),
            auth_types: plugin.supported_auth_types(),
            enabled: true,
            credential_count: 0,
            healthy_credential_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_protocol_conversion() {
        assert_eq!(
            StandardProtocol::from_str("anthropic"),
            Some(StandardProtocol::Anthropic)
        );
        assert_eq!(
            StandardProtocol::from_str("OPENAI"),
            Some(StandardProtocol::OpenAI)
        );
        assert_eq!(StandardProtocol::from_str("unknown"), None);

        assert_eq!(StandardProtocol::Anthropic.as_str(), "anthropic");
        assert_eq!(StandardProtocol::OpenAI.as_str(), "openai");
    }

    #[test]
    fn test_auth_type_info_serialization() {
        let info = AuthTypeInfo {
            id: "oauth".to_string(),
            display_name: "OAuth 登录".to_string(),
            description: "使用官方 OAuth 授权".to_string(),
            category: CredentialCategory::OAuth,
            icon: Some("Key".to_string()),
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("oauth"));
        assert!(json.contains("OAuth 登录"));

        let parsed: AuthTypeInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "oauth");
    }

    #[test]
    fn test_provider_error_serialization() {
        let error = ProviderError {
            error_type: ProviderErrorType::RateLimit,
            message: "Too many requests".to_string(),
            status_code: Some(429),
            retryable: true,
            cooldown_seconds: Some(60),
        };

        let json = serde_json::to_string(&error).unwrap();
        let parsed: ProviderError = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.error_type, ProviderErrorType::RateLimit);
        assert_eq!(parsed.status_code, Some(429));
        assert!(parsed.retryable);
    }
}
