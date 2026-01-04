//! 负载均衡策略
//!
//! 根据当前负载选择模型，实现负载均衡。

use crate::orchestrator::strategy::{
    ModelSelection, SelectionContext, SelectionStrategy, StrategyError, StrategyResult,
};
use crate::orchestrator::tier::AvailableModel;
use async_trait::async_trait;

/// 负载均衡策略
pub struct LoadBalancedStrategy;

impl LoadBalancedStrategy {
    /// 创建新的负载均衡策略
    pub fn new() -> Self {
        Self
    }

    /// 计算模型的负载得分（越低越好）
    fn load_score(model: &AvailableModel) -> f64 {
        // 基础负载
        let load = model.current_load.unwrap_or(50) as f64;

        // 如果不健康，给予最高负载
        if !model.is_healthy {
            return 1000.0;
        }

        load
    }
}

impl Default for LoadBalancedStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SelectionStrategy for LoadBalancedStrategy {
    fn id(&self) -> &str {
        "load_balanced"
    }

    fn display_name(&self) -> &str {
        "负载均衡"
    }

    fn description(&self) -> &str {
        "根据当前负载选择模型，实现负载均衡"
    }

    async fn select(
        &self,
        pool: &[AvailableModel],
        ctx: &SelectionContext,
    ) -> StrategyResult<ModelSelection> {
        // 过滤可用模型
        let mut available: Vec<_> = pool
            .iter()
            .filter(|m| {
                m.is_healthy
                    && !ctx.excluded_models.contains(&m.id)
                    && (!ctx.requires_vision || m.supports_vision)
                    && (!ctx.requires_tools || m.supports_tools)
            })
            .cloned()
            .collect();

        if available.is_empty() {
            return Err(StrategyError::NoAvailableModels);
        }

        // 按负载排序（从低到高）
        available.sort_by(|a, b| {
            let load_a = Self::load_score(a);
            let load_b = Self::load_score(b);
            load_a
                .partial_cmp(&load_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let selected = available.remove(0);
        let load = selected.current_load.unwrap_or(50);

        Ok(ModelSelection {
            model: selected,
            reason: format!("负载均衡选择 (当前负载: {}%)", load),
            confidence: 80,
            alternatives: available,
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
                id: "model-high-load".to_string(),
                display_name: "High Load Model".to_string(),
                provider_type: "test".to_string(),
                family: None,
                credential_id: "cred-1".to_string(),
                context_length: None,
                supports_vision: false,
                supports_tools: false,
                input_cost_per_million: None,
                output_cost_per_million: None,
                is_healthy: true,
                current_load: Some(80),
            },
            AvailableModel {
                id: "model-low-load".to_string(),
                display_name: "Low Load Model".to_string(),
                provider_type: "test".to_string(),
                family: None,
                credential_id: "cred-2".to_string(),
                context_length: None,
                supports_vision: false,
                supports_tools: false,
                input_cost_per_million: None,
                output_cost_per_million: None,
                is_healthy: true,
                current_load: Some(20),
            },
            AvailableModel {
                id: "model-medium-load".to_string(),
                display_name: "Medium Load Model".to_string(),
                provider_type: "test".to_string(),
                family: None,
                credential_id: "cred-3".to_string(),
                context_length: None,
                supports_vision: false,
                supports_tools: false,
                input_cost_per_million: None,
                output_cost_per_million: None,
                is_healthy: true,
                current_load: Some(50),
            },
        ]
    }

    #[tokio::test]
    async fn test_load_balanced_selection() {
        let strategy = LoadBalancedStrategy::new();
        let models = create_test_models();
        let ctx = SelectionContext::new(ServiceTier::Pro);

        let result = strategy.select(&models, &ctx).await.unwrap();
        // 应该选择负载最低的模型
        assert_eq!(result.model.id, "model-low-load");
    }
}
