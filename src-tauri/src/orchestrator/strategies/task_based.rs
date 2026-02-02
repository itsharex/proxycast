//! 任务匹配策略
//!
//! 根据任务类型选择最适合的模型。

use crate::orchestrator::strategy::{
    ModelSelection, SelectionContext, SelectionStrategy, StrategyError, StrategyResult, TaskHint,
};
use crate::orchestrator::tier::AvailableModel;
use async_trait::async_trait;

/// 任务匹配策略
pub struct TaskBasedStrategy;

impl TaskBasedStrategy {
    /// 创建新的任务匹配策略
    pub fn new() -> Self {
        Self
    }

    /// 根据任务类型计算模型得分
    fn score_for_task(model: &AvailableModel, task: Option<TaskHint>) -> f64 {
        let mut score = 0.0;

        // 基础分：健康状态
        if !model.is_healthy {
            return 0.0;
        }

        let family = model.family.as_deref().unwrap_or("").to_lowercase();

        match task {
            Some(TaskHint::Coding) => {
                // 代码任务偏好 Sonnet/GPT-4 级别
                if family.contains("sonnet") || family.contains("gpt-4") {
                    score += 100.0;
                } else if family.contains("opus") || family.contains("o1") {
                    score += 90.0;
                } else if family.contains("haiku") || family.contains("flash") {
                    score += 60.0;
                }
                // 工具调用对代码任务很重要
                if model.supports_tools {
                    score += 20.0;
                }
            }
            Some(TaskHint::Writing) | Some(TaskHint::Analysis) => {
                // 写作/分析任务偏好 Opus/O1 级别
                if family.contains("opus") || family.contains("o1") {
                    score += 100.0;
                } else if family.contains("sonnet") || family.contains("gpt-4") {
                    score += 80.0;
                } else {
                    score += 50.0;
                }
            }
            Some(TaskHint::Chat) => {
                // 对话任务偏好快速响应
                if family.contains("haiku") || family.contains("flash") {
                    score += 100.0;
                } else if family.contains("sonnet") {
                    score += 80.0;
                } else {
                    score += 60.0;
                }
            }
            Some(TaskHint::Math) => {
                // 数学任务偏好推理能力强的模型
                if family.contains("o1") {
                    score += 100.0;
                } else if family.contains("opus") {
                    score += 90.0;
                } else if family.contains("sonnet") || family.contains("gpt-4") {
                    score += 70.0;
                } else {
                    score += 50.0;
                }
            }
            Some(TaskHint::Translation) | Some(TaskHint::Summarization) => {
                // 翻译/摘要任务偏好均衡模型
                if family.contains("sonnet") || family.contains("gpt-4") {
                    score += 100.0;
                } else if family.contains("opus") {
                    score += 80.0;
                } else {
                    score += 60.0;
                }
            }
            _ => {
                // 默认：按家族等级评分
                if family.contains("opus") || family.contains("o1") {
                    score += 90.0;
                } else if family.contains("sonnet") || family.contains("gpt-4") {
                    score += 80.0;
                } else {
                    score += 70.0;
                }
            }
        }

        // 上下文长度加分
        if let Some(ctx_len) = model.context_length {
            score += (ctx_len as f64 / 50000.0).min(10.0);
        }

        // 负载惩罚
        if let Some(load) = model.current_load {
            score -= load as f64 * 0.3;
        }

        score
    }
}

impl Default for TaskBasedStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SelectionStrategy for TaskBasedStrategy {
    fn id(&self) -> &str {
        "task_based"
    }

    fn display_name(&self) -> &str {
        "任务匹配"
    }

    fn description(&self) -> &str {
        "根据任务类型选择最适合的模型"
    }

    fn supports_task(&self, _task: TaskHint) -> bool {
        true
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

        // 按任务类型评分排序
        available.sort_by(|a, b| {
            let score_a = Self::score_for_task(a, ctx.task_hint);
            let score_b = Self::score_for_task(b, ctx.task_hint);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let selected = available.remove(0);
        let task_name = ctx.task_hint.map(|t| t.display_name()).unwrap_or("通用");

        Ok(ModelSelection {
            model: selected,
            reason: format!("任务匹配选择 (任务类型: {task_name})"),
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
                id: "claude-sonnet".to_string(),
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
            AvailableModel {
                id: "claude-haiku".to_string(),
                display_name: "Claude Haiku".to_string(),
                provider_type: "anthropic".to_string(),
                family: Some("haiku".to_string()),
                credential_id: "cred-3".to_string(),
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
    async fn test_task_based_coding() {
        let strategy = TaskBasedStrategy::new();
        let models = create_test_models();
        let ctx = SelectionContext::new(ServiceTier::Pro).with_task_hint(TaskHint::Coding);

        let result = strategy.select(&models, &ctx).await.unwrap();
        // 代码任务应该选择 Sonnet
        assert_eq!(result.model.family, Some("sonnet".to_string()));
    }

    #[tokio::test]
    async fn test_task_based_chat() {
        let strategy = TaskBasedStrategy::new();
        let models = create_test_models();
        let ctx = SelectionContext::new(ServiceTier::Mini).with_task_hint(TaskHint::Chat);

        let result = strategy.select(&models, &ctx).await.unwrap();
        // 对话任务应该选择 Haiku
        assert_eq!(result.model.family, Some("haiku".to_string()));
    }

    #[tokio::test]
    async fn test_task_based_analysis() {
        let strategy = TaskBasedStrategy::new();
        let models = create_test_models();
        let ctx = SelectionContext::new(ServiceTier::Max).with_task_hint(TaskHint::Analysis);

        let result = strategy.select(&models, &ctx).await.unwrap();
        // 分析任务应该选择 Opus
        assert_eq!(result.model.family, Some("opus".to_string()));
    }
}
