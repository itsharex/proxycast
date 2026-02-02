//! 模型注册表数据结构

use serde::{Deserialize, Serialize};

/// 模型能力
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelCapabilities {
    pub vision: bool,
    pub tools: bool,
    pub streaming: bool,
    pub json_mode: bool,
    pub function_calling: bool,
    pub reasoning: bool,
}

/// 模型定价
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub input_per_million: Option<f64>,
    pub output_per_million: Option<f64>,
    pub cache_read_per_million: Option<f64>,
    pub cache_write_per_million: Option<f64>,
    pub currency: String,
}

impl Default for ModelPricing {
    fn default() -> Self {
        Self {
            input_per_million: None,
            output_per_million: None,
            cache_read_per_million: None,
            cache_write_per_million: None,
            currency: "USD".to_string(),
        }
    }
}

/// 模型限制
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelLimits {
    pub context_length: Option<u32>,
    pub max_output_tokens: Option<u32>,
    pub requests_per_minute: Option<u32>,
    pub tokens_per_minute: Option<u32>,
}

/// 模型状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ModelStatus {
    Active,
    Preview,
    Alpha,
    Beta,
    Deprecated,
    Legacy,
}

impl Default for ModelStatus {
    fn default() -> Self {
        Self::Active
    }
}

impl std::fmt::Display for ModelStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Preview => write!(f, "preview"),
            Self::Alpha => write!(f, "alpha"),
            Self::Beta => write!(f, "beta"),
            Self::Deprecated => write!(f, "deprecated"),
            Self::Legacy => write!(f, "legacy"),
        }
    }
}

impl std::str::FromStr for ModelStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(Self::Active),
            "preview" => Ok(Self::Preview),
            "alpha" => Ok(Self::Alpha),
            "beta" => Ok(Self::Beta),
            "deprecated" => Ok(Self::Deprecated),
            "legacy" => Ok(Self::Legacy),
            _ => Err(format!("Unknown model status: {s}")),
        }
    }
}

/// 模型服务等级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ModelTier {
    Mini,
    Pro,
    Max,
}

impl Default for ModelTier {
    fn default() -> Self {
        Self::Pro
    }
}

impl std::fmt::Display for ModelTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mini => write!(f, "mini"),
            Self::Pro => write!(f, "pro"),
            Self::Max => write!(f, "max"),
        }
    }
}

impl std::str::FromStr for ModelTier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mini" => Ok(Self::Mini),
            "pro" => Ok(Self::Pro),
            "max" => Ok(Self::Max),
            _ => Err(format!("Unknown model tier: {s}")),
        }
    }
}

/// 模型数据来源
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ModelSource {
    Embedded,
    ModelsDev,
    Local,
    Custom,
}

impl Default for ModelSource {
    fn default() -> Self {
        Self::Local
    }
}

impl std::fmt::Display for ModelSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Embedded => write!(f, "embedded"),
            Self::ModelsDev => write!(f, "models.dev"),
            Self::Local => write!(f, "local"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

impl std::str::FromStr for ModelSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "embedded" => Ok(Self::Embedded),
            "models.dev" | "modelsdev" => Ok(Self::ModelsDev),
            "local" => Ok(Self::Local),
            "custom" => Ok(Self::Custom),
            _ => Err(format!("Unknown model source: {s}")),
        }
    }
}

/// 增强的模型元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedModelMetadata {
    pub id: String,
    pub display_name: String,
    pub provider_id: String,
    pub provider_name: String,
    pub family: Option<String>,
    pub tier: ModelTier,
    pub capabilities: ModelCapabilities,
    pub pricing: Option<ModelPricing>,
    pub limits: ModelLimits,
    pub status: ModelStatus,
    pub release_date: Option<String>,
    pub is_latest: bool,
    pub description: Option<String>,
    pub source: ModelSource,
    pub created_at: i64,
    pub updated_at: i64,
}

impl EnhancedModelMetadata {
    pub fn new(
        id: String,
        display_name: String,
        provider_id: String,
        provider_name: String,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id,
            display_name,
            provider_id,
            provider_name,
            family: None,
            tier: ModelTier::Pro,
            capabilities: ModelCapabilities::default(),
            pricing: None,
            limits: ModelLimits::default(),
            status: ModelStatus::Active,
            release_date: None,
            is_latest: false,
            description: None,
            source: ModelSource::Local,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_family(mut self, family: impl Into<String>) -> Self {
        self.family = Some(family.into());
        self
    }

    pub fn with_tier(mut self, tier: ModelTier) -> Self {
        self.tier = tier;
        self
    }

    pub fn with_capabilities(mut self, capabilities: ModelCapabilities) -> Self {
        self.capabilities = capabilities;
        self
    }

    pub fn with_pricing(mut self, pricing: ModelPricing) -> Self {
        self.pricing = Some(pricing);
        self
    }

    pub fn with_limits(mut self, limits: ModelLimits) -> Self {
        self.limits = limits;
        self
    }

    pub fn with_status(mut self, status: ModelStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_release_date(mut self, date: impl Into<String>) -> Self {
        self.release_date = Some(date.into());
        self
    }

    pub fn with_is_latest(mut self, is_latest: bool) -> Self {
        self.is_latest = is_latest;
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_source(mut self, source: ModelSource) -> Self {
        self.source = source;
        self
    }
}

/// 用户模型偏好
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserModelPreference {
    pub model_id: String,
    pub is_favorite: bool,
    pub is_hidden: bool,
    pub custom_alias: Option<String>,
    pub usage_count: u32,
    pub last_used_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl UserModelPreference {
    pub fn new(model_id: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            model_id,
            is_favorite: false,
            is_hidden: false,
            custom_alias: None,
            usage_count: 0,
            last_used_at: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// 模型同步状态
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelSyncState {
    pub last_sync_at: Option<i64>,
    pub model_count: u32,
    pub is_syncing: bool,
    pub last_error: Option<String>,
}

// Provider Alias 相关类型

/// 单个模型别名映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelAlias {
    pub actual: String,
    pub internal_name: Option<String>,
    pub provider: Option<String>,
    pub description: Option<String>,
}

/// Provider 的别名配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAliasConfig {
    pub provider: String,
    pub description: Option<String>,
    #[serde(default)]
    pub models: Vec<String>,
    pub aliases: std::collections::HashMap<String, ModelAlias>,
    pub updated_at: Option<String>,
}

impl ProviderAliasConfig {
    pub fn supports_model(&self, model: &str) -> bool {
        self.models.contains(&model.to_string()) || self.aliases.contains_key(model)
    }

    pub fn get_internal_name(&self, model: &str) -> Option<&str> {
        self.aliases
            .get(model)
            .and_then(|a| a.internal_name.as_deref())
    }

    pub fn get_actual_model(&self, model: &str) -> Option<&str> {
        self.aliases.get(model).map(|a| a.actual.as_str())
    }
}

/// models.dev API 响应中的 Provider 结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ModelsDevProvider {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub api: Option<String>,
    #[serde(default)]
    pub npm: Option<String>,
    #[serde(default)]
    pub models: std::collections::HashMap<String, ModelsDevModel>,
}

/// models.dev API 响应中的 Model 结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsDevModel {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub family: Option<String>,
    #[serde(default)]
    pub release_date: Option<String>,
    #[serde(default)]
    pub attachment: bool,
    #[serde(default)]
    pub reasoning: bool,
    #[serde(default)]
    pub temperature: bool,
    #[serde(default)]
    pub tool_call: bool,
    #[serde(default)]
    pub cost: Option<ModelsDevCost>,
    #[serde(default)]
    pub limit: Option<ModelsDevLimit>,
    #[serde(default)]
    pub modalities: Option<ModelsDevModalities>,
    #[serde(default)]
    pub experimental: Option<bool>,
    #[serde(default)]
    pub status: Option<String>,
}

/// models.dev API 响应中的 Cost 结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsDevCost {
    #[serde(default)]
    pub input: Option<f64>,
    #[serde(default)]
    pub output: Option<f64>,
    #[serde(default)]
    pub cache_read: Option<f64>,
    #[serde(default)]
    pub cache_write: Option<f64>,
}

/// models.dev API 响应中的 Limit 结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsDevLimit {
    #[serde(default)]
    pub context: Option<u32>,
    #[serde(default)]
    pub output: Option<u32>,
}

/// models.dev API 响应中的 Modalities 结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsDevModalities {
    #[serde(default)]
    pub input: Vec<String>,
    #[serde(default)]
    pub output: Vec<String>,
}

impl ModelsDevModel {
    #[allow(dead_code)]
    pub fn to_enhanced_metadata(
        &self,
        provider_id: &str,
        provider_name: &str,
    ) -> EnhancedModelMetadata {
        let now = chrono::Utc::now().timestamp();

        let supports_vision = self
            .modalities
            .as_ref()
            .map(|m| m.input.iter().any(|i| i == "image" || i == "video"))
            .unwrap_or(false)
            || self.attachment;

        let tier = infer_model_tier(&self.id, &self.name);

        let status = self
            .status
            .as_ref()
            .and_then(|s| s.parse().ok())
            .unwrap_or(ModelStatus::Active);

        let is_latest = self.id.contains("latest");

        EnhancedModelMetadata {
            id: self.id.clone(),
            display_name: self.name.clone(),
            provider_id: provider_id.to_string(),
            provider_name: provider_name.to_string(),
            family: self.family.clone(),
            tier,
            capabilities: ModelCapabilities {
                vision: supports_vision,
                tools: self.tool_call,
                streaming: true,
                json_mode: true,
                function_calling: self.tool_call,
                reasoning: self.reasoning,
            },
            pricing: self.cost.as_ref().map(|c| ModelPricing {
                input_per_million: c.input,
                output_per_million: c.output,
                cache_read_per_million: c.cache_read,
                cache_write_per_million: c.cache_write,
                currency: "USD".to_string(),
            }),
            limits: ModelLimits {
                context_length: self.limit.as_ref().and_then(|l| l.context),
                max_output_tokens: self.limit.as_ref().and_then(|l| l.output),
                requests_per_minute: None,
                tokens_per_minute: None,
            },
            status,
            release_date: self.release_date.clone(),
            is_latest,
            description: None,
            source: ModelSource::ModelsDev,
            created_at: now,
            updated_at: now,
        }
    }
}

/// 根据模型 ID 和名称推断服务等级
#[allow(dead_code)]
fn infer_model_tier(model_id: &str, model_name: &str) -> ModelTier {
    let id_lower = model_id.to_lowercase();
    let name_lower = model_name.to_lowercase();

    let max_patterns = [
        "opus",
        "gpt-4o",
        "gpt-4-turbo",
        "gemini-2.5-pro",
        "gemini-ultra",
        "claude-3-opus",
        "qwen-max",
        "glm-4-plus",
        "deepseek-v3",
    ];
    for pattern in max_patterns {
        if id_lower.contains(pattern) || name_lower.contains(pattern) {
            return ModelTier::Max;
        }
    }

    let mini_patterns = [
        "mini",
        "nano",
        "lite",
        "flash",
        "haiku",
        "gpt-4o-mini",
        "gemini-flash",
        "qwen-turbo",
        "glm-4-flash",
    ];
    for pattern in mini_patterns {
        if id_lower.contains(pattern) || name_lower.contains(pattern) {
            return ModelTier::Mini;
        }
    }

    ModelTier::Pro
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_tier_inference() {
        assert_eq!(
            infer_model_tier("claude-opus-4-5-20250514", "Claude Opus 4.5"),
            ModelTier::Max
        );
        assert_eq!(
            infer_model_tier("gpt-4o-mini", "GPT-4o Mini"),
            ModelTier::Mini
        );
        assert_eq!(
            infer_model_tier("claude-sonnet-4-5", "Claude Sonnet 4.5"),
            ModelTier::Pro
        );
        assert_eq!(
            infer_model_tier("gemini-2.5-flash", "Gemini 2.5 Flash"),
            ModelTier::Mini
        );
    }

    #[test]
    fn test_model_status_parsing() {
        assert_eq!(
            "active".parse::<ModelStatus>().unwrap(),
            ModelStatus::Active
        );
        assert_eq!(
            "deprecated".parse::<ModelStatus>().unwrap(),
            ModelStatus::Deprecated
        );
        assert_eq!("beta".parse::<ModelStatus>().unwrap(), ModelStatus::Beta);
    }
}
