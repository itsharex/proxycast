//! 模型选择器
//!
//! 提供统一的模型选择接口，整合策略和模型池。

use super::strategy::{SelectionContext, StrategyError, StrategyRegistry, StrategyResult};
use super::tier::{AvailableModel, ServiceTier, TierConfig, TierPool};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// 选择结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionResult {
    /// 选中的模型
    pub model: AvailableModel,
    /// 使用的策略 ID
    pub strategy_id: String,
    /// 选择原因
    pub reason: String,
    /// 置信度 (0-100)
    pub confidence: u8,
    /// 服务等级
    pub tier: ServiceTier,
    /// 是否是降级选择
    pub is_fallback: bool,
    /// 降级原因（如果是降级）
    pub fallback_reason: Option<String>,
}

/// 模型选择器
pub struct ModelSelector {
    /// 策略注册表
    registry: Arc<RwLock<StrategyRegistry>>,
    /// 等级配置
    tier_configs: HashMap<ServiceTier, TierConfig>,
    /// 模型池
    pool: Arc<RwLock<TierPool>>,
}

impl ModelSelector {
    /// 创建新的模型选择器
    pub fn new(registry: StrategyRegistry) -> Self {
        Self {
            registry: Arc::new(RwLock::new(registry)),
            tier_configs: TierConfig::defaults(),
            pool: Arc::new(RwLock::new(TierPool::new())),
        }
    }

    /// 使用自定义配置创建
    pub fn with_configs(
        registry: StrategyRegistry,
        configs: HashMap<ServiceTier, TierConfig>,
    ) -> Self {
        Self {
            registry: Arc::new(RwLock::new(registry)),
            tier_configs: configs,
            pool: Arc::new(RwLock::new(TierPool::new())),
        }
    }

    /// 更新模型池
    pub async fn update_pool(&self, pool: TierPool) {
        let mut current = self.pool.write().await;
        *current = pool;
        info!(
            "模型池已更新: Mini={}, Pro={}, Max={}",
            current.mini.len(),
            current.pro.len(),
            current.max.len()
        );
    }

    /// 获取模型池
    pub async fn get_pool(&self) -> TierPool {
        self.pool.read().await.clone()
    }

    /// 选择模型
    pub async fn select(&self, ctx: &SelectionContext) -> StrategyResult<SelectionResult> {
        let pool = self.pool.read().await;
        let models = pool.get(ctx.tier);

        if models.is_empty() {
            warn!("等级 {} 没有可用模型，尝试降级", ctx.tier);
            return self.select_with_fallback(ctx).await;
        }

        // 获取等级配置
        let config = self
            .tier_configs
            .get(&ctx.tier)
            .cloned()
            .unwrap_or_else(TierConfig::pro);

        // 获取策略
        let registry = self.registry.read().await;
        let strategy = registry
            .get(&config.default_strategy)
            .or_else(|| registry.get_default())
            .ok_or_else(|| StrategyError::StrategyNotFound(config.default_strategy.clone()))?;

        debug!("使用策略 {} 选择模型 (等级: {})", strategy.id(), ctx.tier);

        // 执行选择
        let selection = strategy.select(models, ctx).await?;

        Ok(SelectionResult {
            model: selection.model,
            strategy_id: strategy.id().to_string(),
            reason: selection.reason,
            confidence: selection.confidence,
            tier: ctx.tier,
            is_fallback: false,
            fallback_reason: None,
        })
    }

    /// 使用指定策略选择模型
    pub async fn select_with_strategy(
        &self,
        strategy_id: &str,
        ctx: &SelectionContext,
    ) -> StrategyResult<SelectionResult> {
        let pool = self.pool.read().await;
        let models = pool.get(ctx.tier);

        if models.is_empty() {
            return Err(StrategyError::NoAvailableModels);
        }

        let registry = self.registry.read().await;
        let strategy = registry
            .get(strategy_id)
            .ok_or_else(|| StrategyError::StrategyNotFound(strategy_id.to_string()))?;

        let selection = strategy.select(models, ctx).await?;

        Ok(SelectionResult {
            model: selection.model,
            strategy_id: strategy.id().to_string(),
            reason: selection.reason,
            confidence: selection.confidence,
            tier: ctx.tier,
            is_fallback: false,
            fallback_reason: None,
        })
    }

    /// 带降级的选择
    async fn select_with_fallback(
        &self,
        ctx: &SelectionContext,
    ) -> StrategyResult<SelectionResult> {
        let pool = self.pool.read().await;

        // 尝试降级到更低等级
        let fallback_tiers = match ctx.tier {
            ServiceTier::Max => vec![ServiceTier::Pro, ServiceTier::Mini],
            ServiceTier::Pro => vec![ServiceTier::Mini],
            ServiceTier::Mini => vec![],
        };

        for fallback_tier in fallback_tiers {
            let models = pool.get(fallback_tier);
            if !models.is_empty() {
                let mut fallback_ctx = ctx.clone();
                fallback_ctx.tier = fallback_tier;

                let config = self
                    .tier_configs
                    .get(&fallback_tier)
                    .cloned()
                    .unwrap_or_else(TierConfig::pro);

                let registry = self.registry.read().await;
                let strategy = registry
                    .get(&config.default_strategy)
                    .or_else(|| registry.get_default())
                    .ok_or_else(|| {
                        StrategyError::StrategyNotFound(config.default_strategy.clone())
                    })?;

                let selection = strategy.select(models, &fallback_ctx).await?;

                info!(
                    "降级选择: {} -> {} (模型: {})",
                    ctx.tier, fallback_tier, selection.model.id
                );

                return Ok(SelectionResult {
                    model: selection.model,
                    strategy_id: strategy.id().to_string(),
                    reason: selection.reason,
                    confidence: selection.confidence.saturating_sub(20), // 降级降低置信度
                    tier: fallback_tier,
                    is_fallback: true,
                    fallback_reason: Some(format!(
                        "等级 {} 无可用模型，降级到 {}",
                        ctx.tier, fallback_tier
                    )),
                });
            }
        }

        Err(StrategyError::NoAvailableModels)
    }

    /// 获取策略注册表
    pub async fn get_registry(&self) -> Arc<RwLock<StrategyRegistry>> {
        self.registry.clone()
    }

    /// 列出所有可用策略
    pub async fn list_strategies(&self) -> Vec<super::strategy::StrategyInfo> {
        let registry = self.registry.read().await;
        registry.list_all()
    }

    /// 设置等级的默认策略
    pub fn set_tier_strategy(&mut self, tier: ServiceTier, strategy_id: &str) {
        if let Some(config) = self.tier_configs.get_mut(&tier) {
            config.default_strategy = strategy_id.to_string();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestrator::strategies::create_default_registry;

    fn create_test_pool() -> TierPool {
        let mut pool = TierPool::new();

        pool.add(
            ServiceTier::Mini,
            AvailableModel {
                id: "haiku".to_string(),
                display_name: "Claude Haiku".to_string(),
                provider_type: "anthropic".to_string(),
                family: Some("haiku".to_string()),
                credential_id: "cred-1".to_string(),
                context_length: Some(200000),
                supports_vision: true,
                supports_tools: true,
                input_cost_per_million: None,
                output_cost_per_million: None,
                is_healthy: true,
                current_load: Some(20),
            },
        );

        pool.add(
            ServiceTier::Pro,
            AvailableModel {
                id: "sonnet".to_string(),
                display_name: "Claude Sonnet".to_string(),
                provider_type: "anthropic".to_string(),
                family: Some("sonnet".to_string()),
                credential_id: "cred-2".to_string(),
                context_length: Some(200000),
                supports_vision: true,
                supports_tools: true,
                input_cost_per_million: None,
                output_cost_per_million: None,
                is_healthy: true,
                current_load: Some(30),
            },
        );

        pool
    }

    #[tokio::test]
    async fn test_model_selector() {
        let registry = create_default_registry();
        let selector = ModelSelector::new(registry);

        selector.update_pool(create_test_pool()).await;

        let ctx = SelectionContext::new(ServiceTier::Pro);
        let result = selector.select(&ctx).await.unwrap();

        assert_eq!(result.tier, ServiceTier::Pro);
        assert!(!result.is_fallback);
    }

    #[tokio::test]
    async fn test_fallback_selection() {
        let registry = create_default_registry();
        let selector = ModelSelector::new(registry);

        // 只有 Mini 等级有模型
        let mut pool = TierPool::new();
        pool.add(
            ServiceTier::Mini,
            AvailableModel {
                id: "haiku".to_string(),
                display_name: "Claude Haiku".to_string(),
                provider_type: "anthropic".to_string(),
                family: Some("haiku".to_string()),
                credential_id: "cred-1".to_string(),
                context_length: None,
                supports_vision: false,
                supports_tools: false,
                input_cost_per_million: None,
                output_cost_per_million: None,
                is_healthy: true,
                current_load: None,
            },
        );
        selector.update_pool(pool).await;

        // 请求 Max 等级，应该降级到 Mini
        let ctx = SelectionContext::new(ServiceTier::Max);
        let result = selector.select(&ctx).await.unwrap();

        assert_eq!(result.tier, ServiceTier::Mini);
        assert!(result.is_fallback);
        assert!(result.fallback_reason.is_some());
    }
}
