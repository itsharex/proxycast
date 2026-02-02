//! 降级处理器
//!
//! 处理模型选择失败时的降级逻辑。

use super::tier::{AvailableModel, ServiceTier};
use serde::{Deserialize, Serialize};

/// 降级策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum FallbackPolicy {
    /// 不降级，直接失败
    None,
    /// 降级到下一个等级
    #[default]
    NextTier,
    /// 降级到任意可用模型
    AnyAvailable,
    /// 使用指定的备用模型
    Specific,
}

/// 降级结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackResult {
    /// 是否成功降级
    pub success: bool,
    /// 降级后的模型
    pub model: Option<AvailableModel>,
    /// 原始等级
    pub original_tier: ServiceTier,
    /// 降级后的等级
    pub fallback_tier: Option<ServiceTier>,
    /// 降级原因
    pub reason: String,
    /// 尝试次数
    pub attempts: u32,
}

/// 降级处理器
pub struct FallbackHandler {
    /// 降级策略
    policy: FallbackPolicy,
    /// 最大尝试次数
    max_attempts: u32,
    /// 备用模型 ID（用于 Specific 策略）
    fallback_model_id: Option<String>,
}

impl FallbackHandler {
    /// 创建新的降级处理器
    pub fn new(policy: FallbackPolicy) -> Self {
        Self {
            policy,
            max_attempts: 3,
            fallback_model_id: None,
        }
    }

    /// 设置最大尝试次数
    pub fn with_max_attempts(mut self, max: u32) -> Self {
        self.max_attempts = max;
        self
    }

    /// 设置备用模型 ID
    pub fn with_fallback_model(mut self, model_id: &str) -> Self {
        self.fallback_model_id = Some(model_id.to_string());
        self
    }

    /// 获取降级策略
    pub fn policy(&self) -> FallbackPolicy {
        self.policy
    }

    /// 获取下一个降级等级
    pub fn next_tier(tier: ServiceTier) -> Option<ServiceTier> {
        match tier {
            ServiceTier::Max => Some(ServiceTier::Pro),
            ServiceTier::Pro => Some(ServiceTier::Mini),
            ServiceTier::Mini => None,
        }
    }

    /// 获取所有降级等级（按优先级排序）
    pub fn fallback_tiers(tier: ServiceTier) -> Vec<ServiceTier> {
        match tier {
            ServiceTier::Max => vec![ServiceTier::Pro, ServiceTier::Mini],
            ServiceTier::Pro => vec![ServiceTier::Mini],
            ServiceTier::Mini => vec![],
        }
    }

    /// 处理降级
    pub fn handle(
        &self,
        original_tier: ServiceTier,
        available_models: &[(ServiceTier, Vec<AvailableModel>)],
        reason: &str,
    ) -> FallbackResult {
        match self.policy {
            FallbackPolicy::None => FallbackResult {
                success: false,
                model: None,
                original_tier,
                fallback_tier: None,
                reason: format!("降级策略为 None，不进行降级: {reason}"),
                attempts: 0,
            },

            FallbackPolicy::NextTier => {
                let fallback_tiers = Self::fallback_tiers(original_tier);
                let mut attempts = 0;

                for tier in fallback_tiers {
                    attempts += 1;
                    if attempts > self.max_attempts {
                        break;
                    }

                    if let Some((_, models)) = available_models.iter().find(|(t, _)| *t == tier) {
                        if let Some(model) = models.iter().find(|m| m.is_healthy).cloned() {
                            return FallbackResult {
                                success: true,
                                model: Some(model),
                                original_tier,
                                fallback_tier: Some(tier),
                                reason: format!("从 {original_tier} 降级到 {tier}: {reason}"),
                                attempts,
                            };
                        }
                    }
                }

                FallbackResult {
                    success: false,
                    model: None,
                    original_tier,
                    fallback_tier: None,
                    reason: format!("所有降级等级都没有可用模型: {reason}"),
                    attempts,
                }
            }

            FallbackPolicy::AnyAvailable => {
                let mut attempts = 0;

                // 按等级优先级遍历所有模型
                for tier in [ServiceTier::Max, ServiceTier::Pro, ServiceTier::Mini] {
                    attempts += 1;
                    if attempts > self.max_attempts {
                        break;
                    }

                    if let Some((_, models)) = available_models.iter().find(|(t, _)| *t == tier) {
                        if let Some(model) = models.iter().find(|m| m.is_healthy).cloned() {
                            return FallbackResult {
                                success: true,
                                model: Some(model),
                                original_tier,
                                fallback_tier: Some(tier),
                                reason: format!("选择任意可用模型 (等级 {tier}): {reason}"),
                                attempts,
                            };
                        }
                    }
                }

                FallbackResult {
                    success: false,
                    model: None,
                    original_tier,
                    fallback_tier: None,
                    reason: format!("没有任何可用模型: {reason}"),
                    attempts,
                }
            }

            FallbackPolicy::Specific => {
                if let Some(fallback_id) = &self.fallback_model_id {
                    for (tier, models) in available_models {
                        if let Some(model) = models
                            .iter()
                            .find(|m| m.id == *fallback_id && m.is_healthy)
                            .cloned()
                        {
                            return FallbackResult {
                                success: true,
                                model: Some(model),
                                original_tier,
                                fallback_tier: Some(*tier),
                                reason: format!("使用指定备用模型 {fallback_id}: {reason}"),
                                attempts: 1,
                            };
                        }
                    }

                    FallbackResult {
                        success: false,
                        model: None,
                        original_tier,
                        fallback_tier: None,
                        reason: format!("指定的备用模型 {fallback_id} 不可用: {reason}"),
                        attempts: 1,
                    }
                } else {
                    FallbackResult {
                        success: false,
                        model: None,
                        original_tier,
                        fallback_tier: None,
                        reason: format!("未配置备用模型: {reason}"),
                        attempts: 0,
                    }
                }
            }
        }
    }
}

impl Default for FallbackHandler {
    fn default() -> Self {
        Self::new(FallbackPolicy::NextTier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_models() -> Vec<(ServiceTier, Vec<AvailableModel>)> {
        vec![
            (
                ServiceTier::Mini,
                vec![AvailableModel {
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
                }],
            ),
            (
                ServiceTier::Pro,
                vec![AvailableModel {
                    id: "sonnet".to_string(),
                    display_name: "Claude Sonnet".to_string(),
                    provider_type: "anthropic".to_string(),
                    family: Some("sonnet".to_string()),
                    credential_id: "cred-2".to_string(),
                    context_length: None,
                    supports_vision: false,
                    supports_tools: false,
                    input_cost_per_million: None,
                    output_cost_per_million: None,
                    is_healthy: true,
                    current_load: None,
                }],
            ),
        ]
    }

    #[test]
    fn test_fallback_next_tier() {
        let handler = FallbackHandler::new(FallbackPolicy::NextTier);
        let models = create_test_models();

        let result = handler.handle(ServiceTier::Max, &models, "测试降级");

        assert!(result.success);
        assert_eq!(result.fallback_tier, Some(ServiceTier::Pro));
        assert_eq!(result.model.unwrap().id, "sonnet");
    }

    #[test]
    fn test_fallback_none() {
        let handler = FallbackHandler::new(FallbackPolicy::None);
        let models = create_test_models();

        let result = handler.handle(ServiceTier::Max, &models, "测试降级");

        assert!(!result.success);
        assert!(result.model.is_none());
    }

    #[test]
    fn test_fallback_specific() {
        let handler = FallbackHandler::new(FallbackPolicy::Specific).with_fallback_model("haiku");
        let models = create_test_models();

        let result = handler.handle(ServiceTier::Max, &models, "测试降级");

        assert!(result.success);
        assert_eq!(result.model.unwrap().id, "haiku");
    }

    #[test]
    fn test_next_tier() {
        assert_eq!(
            FallbackHandler::next_tier(ServiceTier::Max),
            Some(ServiceTier::Pro)
        );
        assert_eq!(
            FallbackHandler::next_tier(ServiceTier::Pro),
            Some(ServiceTier::Mini)
        );
        assert_eq!(FallbackHandler::next_tier(ServiceTier::Mini), None);
    }
}
