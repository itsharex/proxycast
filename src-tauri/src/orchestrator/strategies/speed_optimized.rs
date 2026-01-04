//! 速度优化策略
//!
//! 选择响应速度最快的模型。

use crate::orchestrator::strategy::{
    ModelSelection, SelectionContext, SelectionStrategy, StrategyError, StrategyResult,
};
use crate::orchestrator::tier::AvailableModel;
use async_trait::async_trait;

/// 速度优化策略
pub struct SpeedOptimizedStrategy;

impl SpeedOptimizedStrategy {
    /// 创建新的速度优化策略
    pub fn new() -> Self {
        Self
    }

    /// 计算模型的速度得分（越高越好）
    fn speed_score(model: &AvailableModel) -> f64 {
        let mut score = 100.0;

        // 根据家族估算速度
        let family = model.family.as_deref().unwrap_or("").to_lowercase();

        if family.contains("haiku") || family.contains("flash") {
            score += 50.0; // 最快
        } else if family.contains("gpt-3.5") {
            score += 40.0;
        } else if family.contains("sonnet") || family.contains("pro") {
            score += 20.0; // 中等
        } else if family.contains("gpt-4") {
            score += 10.0;
        } else if family.contains("opus") || family.contains("ultra") || family.contains("o1") {
            score += 0.0; // 最慢
        }

        // 负载惩罚（负载越高，速度越慢）
        if let Some(load) = model.current_load {
            score -= load as f64 * 0.5;
        }

        score
    }
}

impl Default for SpeedOptimizedStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SelectionStrategy for SpeedOptimizedStrategy {
    fn id(&self) -> &str {
        "speed_optimized"
    }

    fn display_name(&self) -> &str {
        "速度优先"
    }

    fn description(&self) -> &str {
        "选择响应速度最快的模型"
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

        // 按速度排序（从高到低）
        available.sort_by(|a, b| {
            let speed_a = Self::speed_score(a);
            let speed_b = Self::speed_score(b);
            speed_b
                .partial_cmp(&speed_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let selected = available.remove(0);

        Ok(ModelSelection {
            model: selected,
            reason: "速度优先选择".to_string(),
            confidence: 85,
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
                input_cost_per_million: None,
                output_cost_per_million: None,
                is_healthy: true,
                current_load: Some(20),
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
                input_cost_per_million: None,
                output_cost_per_million: None,
                is_healthy: true,
                current_load: Some(10),
            },
        ]
    }

    #[tokio::test]
    async fn test_speed_optimized_selection() {
        let strategy = SpeedOptimizedStrategy::new();
        let models = create_test_models();
        let ctx = SelectionContext::new(ServiceTier::Mini);

        let result = strategy.select(&models, &ctx).await.unwrap();
        // 应该选择最快的 Haiku
        assert_eq!(result.model.id, "claude-haiku");
    }
}
