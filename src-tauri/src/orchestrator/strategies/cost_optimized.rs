//! 成本优化策略
//!
//! 选择成本最低的模型。

use crate::orchestrator::strategy::{
    ModelSelection, SelectionContext, SelectionStrategy, StrategyError, StrategyResult,
};
use crate::orchestrator::tier::AvailableModel;
use async_trait::async_trait;

/// 成本优化策略
pub struct CostOptimizedStrategy;

impl CostOptimizedStrategy {
    /// 创建新的成本优化策略
    pub fn new() -> Self {
        Self
    }

    /// 计算模型的成本得分（越低越好）
    fn cost_score(model: &AvailableModel) -> f64 {
        // 如果有价格信息，使用价格
        if let (Some(input), Some(output)) =
            (model.input_cost_per_million, model.output_cost_per_million)
        {
            // 假设输入输出比例为 1:1
            return input + output;
        }

        // 否则根据家族估算成本
        let family = model.family.as_deref().unwrap_or("").to_lowercase();

        if family.contains("haiku") || family.contains("flash") || family.contains("gpt-3.5") {
            1.0 // 最便宜
        } else if family.contains("sonnet") || family.contains("pro") {
            5.0 // 中等
        } else if family.contains("opus") || family.contains("ultra") || family.contains("o1") {
            15.0 // 最贵
        } else if family.contains("gpt-4") {
            10.0 // 较贵
        } else {
            5.0 // 默认中等
        }
    }
}

impl Default for CostOptimizedStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SelectionStrategy for CostOptimizedStrategy {
    fn id(&self) -> &str {
        "cost_optimized"
    }

    fn display_name(&self) -> &str {
        "成本优先"
    }

    fn description(&self) -> &str {
        "选择成本最低的模型"
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

        // 按成本排序（从低到高）
        available.sort_by(|a, b| {
            let cost_a = Self::cost_score(a);
            let cost_b = Self::cost_score(b);
            cost_a
                .partial_cmp(&cost_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let selected = available.remove(0);
        let cost = Self::cost_score(&selected);

        Ok(ModelSelection {
            model: selected,
            reason: format!("成本优先选择 (估算成本: {cost:.2})"),
            confidence: 90,
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
                id: "claude-opus".to_string(),
                display_name: "Claude Opus".to_string(),
                provider_type: "anthropic".to_string(),
                family: Some("opus".to_string()),
                credential_id: "cred-1".to_string(),
                context_length: Some(200000),
                supports_vision: true,
                supports_tools: true,
                input_cost_per_million: Some(15.0),
                output_cost_per_million: Some(75.0),
                is_healthy: true,
                current_load: None,
            },
            AvailableModel {
                id: "claude-haiku".to_string(),
                display_name: "Claude Haiku".to_string(),
                provider_type: "anthropic".to_string(),
                family: Some("haiku".to_string()),
                credential_id: "cred-2".to_string(),
                context_length: Some(200000),
                supports_vision: true,
                supports_tools: true,
                input_cost_per_million: Some(0.25),
                output_cost_per_million: Some(1.25),
                is_healthy: true,
                current_load: None,
            },
        ]
    }

    #[tokio::test]
    async fn test_cost_optimized_selection() {
        let strategy = CostOptimizedStrategy::new();
        let models = create_test_models();
        let ctx = SelectionContext::new(ServiceTier::Pro);

        let result = strategy.select(&models, &ctx).await.unwrap();
        // 应该选择最便宜的 Haiku
        assert_eq!(result.model.id, "claude-haiku");
    }
}
