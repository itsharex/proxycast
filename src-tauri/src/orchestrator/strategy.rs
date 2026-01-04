//! 选择策略 trait 和注册表
//!
//! 定义模型选择策略的接口和策略注册表。

use super::tier::{AvailableModel, ServiceTier};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

/// 策略错误
#[derive(Error, Debug)]
pub enum StrategyError {
    #[error("没有可用的模型")]
    NoAvailableModels,

    #[error("策略不存在: {0}")]
    StrategyNotFound(String),

    #[error("选择失败: {0}")]
    SelectionFailed(String),

    #[error("配置错误: {0}")]
    ConfigError(String),
}

pub type StrategyResult<T> = Result<T, StrategyError>;

/// 选择上下文
///
/// 包含选择模型时需要的所有上下文信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionContext {
    /// 服务等级
    pub tier: ServiceTier,
    /// 请求的模型名称（如果有）
    pub requested_model: Option<String>,
    /// 任务类型提示
    pub task_hint: Option<TaskHint>,
    /// 是否需要视觉能力
    pub requires_vision: bool,
    /// 是否需要工具调用
    pub requires_tools: bool,
    /// 预估输入 tokens
    pub estimated_input_tokens: Option<u32>,
    /// 预估输出 tokens
    pub estimated_output_tokens: Option<u32>,
    /// 用户偏好的 Provider
    pub preferred_provider: Option<String>,
    /// 排除的模型 ID 列表
    pub excluded_models: Vec<String>,
    /// 额外元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for SelectionContext {
    fn default() -> Self {
        Self {
            tier: ServiceTier::Pro,
            requested_model: None,
            task_hint: None,
            requires_vision: false,
            requires_tools: false,
            estimated_input_tokens: None,
            estimated_output_tokens: None,
            preferred_provider: None,
            excluded_models: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

impl SelectionContext {
    /// 创建新的选择上下文
    pub fn new(tier: ServiceTier) -> Self {
        Self {
            tier,
            ..Default::default()
        }
    }

    /// 设置任务提示
    pub fn with_task_hint(mut self, hint: TaskHint) -> Self {
        self.task_hint = Some(hint);
        self
    }

    /// 设置视觉需求
    pub fn with_vision(mut self, requires: bool) -> Self {
        self.requires_vision = requires;
        self
    }

    /// 设置工具调用需求
    pub fn with_tools(mut self, requires: bool) -> Self {
        self.requires_tools = requires;
        self
    }

    /// 设置偏好的 Provider
    pub fn with_preferred_provider(mut self, provider: &str) -> Self {
        self.preferred_provider = Some(provider.to_string());
        self
    }

    /// 添加排除的模型
    pub fn exclude_model(mut self, model_id: &str) -> Self {
        self.excluded_models.push(model_id.to_string());
        self
    }
}

/// 任务类型提示
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskHint {
    /// 代码生成/编辑
    Coding,
    /// 写作/创意
    Writing,
    /// 分析/推理
    Analysis,
    /// 对话/聊天
    Chat,
    /// 翻译
    Translation,
    /// 摘要
    Summarization,
    /// 数学/计算
    Math,
    /// 其他
    Other,
}

impl TaskHint {
    /// 获取任务提示的显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            TaskHint::Coding => "代码",
            TaskHint::Writing => "写作",
            TaskHint::Analysis => "分析",
            TaskHint::Chat => "对话",
            TaskHint::Translation => "翻译",
            TaskHint::Summarization => "摘要",
            TaskHint::Math => "数学",
            TaskHint::Other => "其他",
        }
    }
}

/// 模型选择结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSelection {
    /// 选中的模型
    pub model: AvailableModel,
    /// 选择原因
    pub reason: String,
    /// 置信度 (0-100)
    pub confidence: u8,
    /// 备选模型列表
    pub alternatives: Vec<AvailableModel>,
}

/// 选择策略 trait
///
/// 所有模型选择策略必须实现此 trait
#[async_trait]
pub trait SelectionStrategy: Send + Sync {
    /// 策略 ID
    fn id(&self) -> &str;

    /// 策略显示名称
    fn display_name(&self) -> &str;

    /// 策略描述
    fn description(&self) -> &str {
        ""
    }

    /// 选择模型
    async fn select(
        &self,
        pool: &[AvailableModel],
        ctx: &SelectionContext,
    ) -> StrategyResult<ModelSelection>;

    /// 是否支持指定的任务类型
    fn supports_task(&self, _task: TaskHint) -> bool {
        true
    }

    /// 获取策略配置 Schema
    fn config_schema(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    /// 更新策略配置
    fn update_config(&mut self, _config: serde_json::Value) -> StrategyResult<()> {
        Ok(())
    }
}

/// 策略注册表
///
/// 管理所有可用的选择策略
pub struct StrategyRegistry {
    /// 已注册的策略
    strategies: HashMap<String, Arc<dyn SelectionStrategy>>,
    /// 默认策略 ID
    default_strategy: String,
}

impl StrategyRegistry {
    /// 创建新的策略注册表
    pub fn new() -> Self {
        Self {
            strategies: HashMap::new(),
            default_strategy: "round_robin".to_string(),
        }
    }

    /// 注册策略
    pub fn register(&mut self, strategy: Arc<dyn SelectionStrategy>) {
        let id = strategy.id().to_string();
        tracing::info!("注册选择策略: {} ({})", id, strategy.display_name());
        self.strategies.insert(id, strategy);
    }

    /// 获取策略
    pub fn get(&self, id: &str) -> Option<Arc<dyn SelectionStrategy>> {
        self.strategies.get(id).cloned()
    }

    /// 获取默认策略
    pub fn get_default(&self) -> Option<Arc<dyn SelectionStrategy>> {
        self.get(&self.default_strategy)
    }

    /// 设置默认策略
    pub fn set_default(&mut self, id: &str) -> StrategyResult<()> {
        if self.strategies.contains_key(id) {
            self.default_strategy = id.to_string();
            Ok(())
        } else {
            Err(StrategyError::StrategyNotFound(id.to_string()))
        }
    }

    /// 获取所有策略 ID
    pub fn list_ids(&self) -> Vec<&str> {
        self.strategies.keys().map(|s| s.as_str()).collect()
    }

    /// 获取所有策略信息
    pub fn list_all(&self) -> Vec<StrategyInfo> {
        self.strategies
            .values()
            .map(|s| StrategyInfo {
                id: s.id().to_string(),
                display_name: s.display_name().to_string(),
                description: s.description().to_string(),
                is_default: s.id() == self.default_strategy,
            })
            .collect()
    }

    /// 使用指定策略选择模型
    pub async fn select_with(
        &self,
        strategy_id: &str,
        pool: &[AvailableModel],
        ctx: &SelectionContext,
    ) -> StrategyResult<ModelSelection> {
        let strategy = self
            .get(strategy_id)
            .ok_or_else(|| StrategyError::StrategyNotFound(strategy_id.to_string()))?;

        strategy.select(pool, ctx).await
    }

    /// 使用默认策略选择模型
    pub async fn select(
        &self,
        pool: &[AvailableModel],
        ctx: &SelectionContext,
    ) -> StrategyResult<ModelSelection> {
        let strategy = self
            .get_default()
            .ok_or_else(|| StrategyError::StrategyNotFound(self.default_strategy.clone()))?;

        strategy.select(pool, ctx).await
    }
}

impl Default for StrategyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 策略信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyInfo {
    /// 策略 ID
    pub id: String,
    /// 显示名称
    pub display_name: String,
    /// 描述
    pub description: String,
    /// 是否是默认策略
    pub is_default: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockStrategy {
        id: String,
    }

    #[async_trait]
    impl SelectionStrategy for MockStrategy {
        fn id(&self) -> &str {
            &self.id
        }

        fn display_name(&self) -> &str {
            "Mock Strategy"
        }

        async fn select(
            &self,
            pool: &[AvailableModel],
            _ctx: &SelectionContext,
        ) -> StrategyResult<ModelSelection> {
            if pool.is_empty() {
                return Err(StrategyError::NoAvailableModels);
            }

            Ok(ModelSelection {
                model: pool[0].clone(),
                reason: "Mock selection".to_string(),
                confidence: 100,
                alternatives: pool[1..].to_vec(),
            })
        }
    }

    #[test]
    fn test_selection_context() {
        let ctx = SelectionContext::new(ServiceTier::Pro)
            .with_task_hint(TaskHint::Coding)
            .with_vision(true)
            .with_tools(true)
            .with_preferred_provider("anthropic")
            .exclude_model("model-1");

        assert_eq!(ctx.tier, ServiceTier::Pro);
        assert_eq!(ctx.task_hint, Some(TaskHint::Coding));
        assert!(ctx.requires_vision);
        assert!(ctx.requires_tools);
        assert_eq!(ctx.preferred_provider, Some("anthropic".to_string()));
        assert!(ctx.excluded_models.contains(&"model-1".to_string()));
    }

    #[tokio::test]
    async fn test_strategy_registry() {
        let mut registry = StrategyRegistry::new();

        let strategy = Arc::new(MockStrategy {
            id: "mock".to_string(),
        });
        registry.register(strategy);

        assert!(registry.get("mock").is_some());
        assert!(registry.get("nonexistent").is_none());

        registry.set_default("mock").unwrap();
        assert!(registry.get_default().is_some());
    }
}
