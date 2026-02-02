//! 轮询策略
//!
//! 按顺序轮询选择模型，实现简单的负载分散。

use crate::orchestrator::strategy::{
    ModelSelection, SelectionContext, SelectionStrategy, StrategyError, StrategyResult,
};
use crate::orchestrator::tier::AvailableModel;
use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};

/// 轮询策略
pub struct RoundRobinStrategy {
    /// 当前索引
    index: AtomicUsize,
}

impl RoundRobinStrategy {
    /// 创建新的轮询策略
    pub fn new() -> Self {
        Self {
            index: AtomicUsize::new(0),
        }
    }
}

impl Default for RoundRobinStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SelectionStrategy for RoundRobinStrategy {
    fn id(&self) -> &str {
        "round_robin"
    }

    fn display_name(&self) -> &str {
        "轮询"
    }

    fn description(&self) -> &str {
        "按顺序轮询选择模型，实现简单的负载分散"
    }

    async fn select(
        &self,
        pool: &[AvailableModel],
        ctx: &SelectionContext,
    ) -> StrategyResult<ModelSelection> {
        // 过滤可用模型
        let available: Vec<_> = pool
            .iter()
            .filter(|m| {
                m.is_healthy
                    && !ctx.excluded_models.contains(&m.id)
                    && (!ctx.requires_vision || m.supports_vision)
                    && (!ctx.requires_tools || m.supports_tools)
            })
            .collect();

        if available.is_empty() {
            return Err(StrategyError::NoAvailableModels);
        }

        // 获取下一个索引
        let idx = self.index.fetch_add(1, Ordering::Relaxed) % available.len();
        let selected = available[idx].clone();

        // 构建备选列表
        let alternatives: Vec<_> = available
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != idx)
            .map(|(_, m)| (*m).clone())
            .collect();

        Ok(ModelSelection {
            model: selected,
            reason: format!("轮询选择 (索引 {idx})"),
            confidence: 80,
            alternatives,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestrator::tier::ServiceTier;

    fn create_test_models() -> Vec<AvailableModel> {
        vec![
            AvailableModel {
                id: "model-1".to_string(),
                display_name: "Model 1".to_string(),
                provider_type: "test".to_string(),
                family: None,
                credential_id: "cred-1".to_string(),
                context_length: None,
                supports_vision: false,
                supports_tools: false,
                input_cost_per_million: None,
                output_cost_per_million: None,
                is_healthy: true,
                current_load: None,
            },
            AvailableModel {
                id: "model-2".to_string(),
                display_name: "Model 2".to_string(),
                provider_type: "test".to_string(),
                family: None,
                credential_id: "cred-2".to_string(),
                context_length: None,
                supports_vision: false,
                supports_tools: false,
                input_cost_per_million: None,
                output_cost_per_million: None,
                is_healthy: true,
                current_load: None,
            },
            AvailableModel {
                id: "model-3".to_string(),
                display_name: "Model 3".to_string(),
                provider_type: "test".to_string(),
                family: None,
                credential_id: "cred-3".to_string(),
                context_length: None,
                supports_vision: false,
                supports_tools: false,
                input_cost_per_million: None,
                output_cost_per_million: None,
                is_healthy: true,
                current_load: None,
            },
        ]
    }

    #[tokio::test]
    async fn test_round_robin_selection() {
        let strategy = RoundRobinStrategy::new();
        let models = create_test_models();
        let ctx = SelectionContext::new(ServiceTier::Pro);

        // 第一次选择
        let result1 = strategy.select(&models, &ctx).await.unwrap();
        assert_eq!(result1.model.id, "model-1");

        // 第二次选择
        let result2 = strategy.select(&models, &ctx).await.unwrap();
        assert_eq!(result2.model.id, "model-2");

        // 第三次选择
        let result3 = strategy.select(&models, &ctx).await.unwrap();
        assert_eq!(result3.model.id, "model-3");

        // 第四次选择（回到第一个）
        let result4 = strategy.select(&models, &ctx).await.unwrap();
        assert_eq!(result4.model.id, "model-1");
    }

    #[tokio::test]
    async fn test_round_robin_empty_pool() {
        let strategy = RoundRobinStrategy::new();
        let models: Vec<AvailableModel> = vec![];
        let ctx = SelectionContext::new(ServiceTier::Pro);

        let result = strategy.select(&models, &ctx).await;
        assert!(result.is_err());
    }
}
