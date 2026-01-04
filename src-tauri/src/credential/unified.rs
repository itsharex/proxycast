//! 统一凭证管理器
//!
//! 整合 orchestrator 和 credential 模块，提供统一的凭证管理接口。
//!
//! ## 功能
//!
//! - 统一的凭证获取接口
//! - 自动风控和冷却管理
//! - 与 orchestrator 的模型选择集成

use super::balancer::{CredentialSelection, LoadBalancer};
use super::pool::{CredentialPool, PoolError};
use super::risk::{CooldownConfig, RateLimitEvent, RiskController, RiskLevel};
use super::types::{Credential, CredentialData};
use crate::orchestrator::get_global_orchestrator;
use crate::ProviderType;
use chrono::Duration;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// 统一凭证管理器
///
/// 整合 orchestrator 的模型选择和 credential 的凭证管理
pub struct UnifiedCredentialManager {
    /// 负载均衡器
    load_balancer: LoadBalancer,
    /// 风控控制器
    risk_controller: RiskController,
    /// 是否启用风控
    risk_control_enabled: RwLock<bool>,
}

impl UnifiedCredentialManager {
    /// 创建新的统一凭证管理器
    pub fn new() -> Self {
        Self {
            load_balancer: LoadBalancer::round_robin(),
            risk_controller: RiskController::with_defaults(),
            risk_control_enabled: RwLock::new(true),
        }
    }

    /// 使用自定义配置创建
    pub fn with_config(cooldown_config: CooldownConfig) -> Self {
        Self {
            load_balancer: LoadBalancer::round_robin(),
            risk_controller: RiskController::new(cooldown_config),
            risk_control_enabled: RwLock::new(true),
        }
    }

    /// 获取负载均衡器
    pub fn load_balancer(&self) -> &LoadBalancer {
        &self.load_balancer
    }

    /// 获取风控控制器
    pub fn risk_controller(&self) -> &RiskController {
        &self.risk_controller
    }

    /// 设置是否启用风控
    pub async fn set_risk_control_enabled(&self, enabled: bool) {
        let mut flag = self.risk_control_enabled.write().await;
        *flag = enabled;
    }

    /// 检查风控是否启用
    pub async fn is_risk_control_enabled(&self) -> bool {
        *self.risk_control_enabled.read().await
    }

    /// 注册凭证池
    pub fn register_pool(&self, pool: Arc<CredentialPool>) {
        self.load_balancer.register_pool(pool);
    }

    /// 选择凭证（带风控检查）
    ///
    /// # 参数
    /// - `provider`: Provider 类型
    ///
    /// # 返回
    /// - `Ok(CredentialSelection)`: 选中的凭证和 HTTP 客户端
    /// - `Err(PoolError)`: 选择失败
    pub async fn select_credential(
        &self,
        provider: ProviderType,
    ) -> Result<CredentialSelection, PoolError> {
        let risk_enabled = self.is_risk_control_enabled().await;

        // 如果启用风控，先检查是否有凭证在冷却中
        if risk_enabled {
            let cooling = self.risk_controller.get_cooling_credentials();
            if !cooling.is_empty() {
                debug!("有 {} 个凭证在冷却中", cooling.len());
            }
        }

        // 使用负载均衡器选择凭证
        let selection = self.load_balancer.select_with_client(provider)?;

        // 检查选中的凭证是否在冷却中
        if risk_enabled
            && self
                .risk_controller
                .is_in_cooldown(&selection.credential.id)
        {
            warn!(
                "凭证 {} 在冷却中，尝试选择其他凭证",
                selection.credential.id
            );
            // 尝试故障转移
            return self.load_balancer.select_with_failover(provider, None);
        }

        Ok(selection)
    }

    /// 报告请求成功
    pub fn report_success(&self, provider: ProviderType, credential_id: &str, latency_ms: u64) {
        // 更新负载均衡器统计
        let _ = self
            .load_balancer
            .report(provider, credential_id, true, latency_ms);

        // 更新风控状态
        self.risk_controller.record_success(credential_id);
    }

    /// 报告请求失败
    ///
    /// # 参数
    /// - `provider`: Provider 类型
    /// - `credential_id`: 凭证 ID
    /// - `status_code`: HTTP 状态码
    /// - `error_body`: 错误响应体
    /// - `retry_after`: Retry-After 头的值
    ///
    /// # 返回
    /// 如果是限流错误，返回建议的冷却时间（秒）
    pub async fn report_failure(
        &self,
        provider: ProviderType,
        credential_id: &str,
        status_code: Option<u16>,
        error_body: Option<&str>,
        retry_after: Option<&str>,
    ) -> Option<u64> {
        // 更新负载均衡器统计
        let _ = self.load_balancer.report(provider, credential_id, false, 0);

        // 检查是否为限流错误
        let is_rate_limit = status_code
            .map(|code| RiskController::is_rate_limit_error(code, error_body))
            .unwrap_or(false);

        if !is_rate_limit {
            return None;
        }

        // 解析 Retry-After
        let retry_after_secs = retry_after.and_then(RiskController::parse_retry_after);

        // 记录限流事件
        let mut event = RateLimitEvent::new(credential_id.to_string());
        if let Some(code) = status_code {
            event = event.with_status_code(code);
        }
        if let Some(body) = error_body {
            event = event.with_error_message(body.to_string());
        }
        if let Some(secs) = retry_after_secs {
            event = event.with_retry_after(secs);
        }

        let cooldown_secs = self.risk_controller.record_rate_limit(event);

        // 在负载均衡器中标记冷却
        let _ = self.load_balancer.mark_cooldown(
            provider,
            credential_id,
            Duration::seconds(cooldown_secs as i64),
        );

        info!("凭证 {} 触发限流，冷却 {} 秒", credential_id, cooldown_secs);

        Some(cooldown_secs)
    }

    /// 获取凭证的风险等级
    pub fn get_risk_level(&self, credential_id: &str) -> RiskLevel {
        self.risk_controller.get_risk_level(credential_id)
    }

    /// 手动清除凭证的冷却状态
    pub fn clear_cooldown(&self, provider: ProviderType, credential_id: &str) {
        self.risk_controller.clear_cooldown(credential_id);
        let _ = self.load_balancer.mark_active(provider, credential_id);
    }

    /// 从 orchestrator 同步凭证到凭证池
    ///
    /// 将 orchestrator 的 CredentialInfo 转换为 credential 模块的 Credential
    pub async fn sync_from_orchestrator(&self) -> Result<usize, String> {
        let orchestrator = get_global_orchestrator().ok_or("编排器未初始化")?;

        // 获取所有可用模型
        let models = orchestrator.get_all_models().await;

        let mut synced_count = 0;

        // 按 provider 分组
        for model in models {
            let provider_type = self.map_orchestrator_provider(&model.provider_type);

            // 获取或创建凭证池
            let pool = self
                .load_balancer
                .get_pool(provider_type)
                .unwrap_or_else(|| {
                    let new_pool = Arc::new(CredentialPool::new(provider_type));
                    self.load_balancer.register_pool(new_pool.clone());
                    new_pool
                });

            // 检查凭证是否已存在
            if pool.get(&model.credential_id).is_none() {
                // 创建新凭证
                let credential = Credential::new(
                    model.credential_id.clone(),
                    provider_type,
                    CredentialData::ApiKey {
                        key: format!("synced-{}", model.credential_id),
                        base_url: None,
                    },
                );

                if pool.add(credential).is_ok() {
                    synced_count += 1;
                }
            }
        }

        info!("从 orchestrator 同步了 {} 个凭证", synced_count);
        Ok(synced_count)
    }

    /// 映射 orchestrator 的 ProviderType 到 credential 的 ProviderType
    fn map_orchestrator_provider(&self, provider: &str) -> ProviderType {
        match provider.to_lowercase().as_str() {
            "anthropic" => ProviderType::ClaudeOAuth,
            "openai" => ProviderType::Codex,
            "google" | "gemini" => ProviderType::Gemini,
            "kiro" => ProviderType::Kiro,
            _ => ProviderType::Kiro, // 默认
        }
    }
}

impl Default for UnifiedCredentialManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局统一凭证管理器
static GLOBAL_UNIFIED_MANAGER: once_cell::sync::OnceCell<Arc<UnifiedCredentialManager>> =
    once_cell::sync::OnceCell::new();

/// 初始化全局统一凭证管理器
pub fn init_global_unified_manager() -> Arc<UnifiedCredentialManager> {
    GLOBAL_UNIFIED_MANAGER
        .get_or_init(|| Arc::new(UnifiedCredentialManager::new()))
        .clone()
}

/// 获取全局统一凭证管理器
pub fn get_global_unified_manager() -> Option<Arc<UnifiedCredentialManager>> {
    GLOBAL_UNIFIED_MANAGER.get().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_manager_new() {
        let manager = UnifiedCredentialManager::new();
        assert!(manager.load_balancer().providers().is_empty());
    }

    #[tokio::test]
    async fn test_risk_control_toggle() {
        let manager = UnifiedCredentialManager::new();

        assert!(manager.is_risk_control_enabled().await);

        manager.set_risk_control_enabled(false).await;
        assert!(!manager.is_risk_control_enabled().await);

        manager.set_risk_control_enabled(true).await;
        assert!(manager.is_risk_control_enabled().await);
    }

    #[test]
    fn test_map_orchestrator_provider() {
        let manager = UnifiedCredentialManager::new();

        assert_eq!(
            manager.map_orchestrator_provider("anthropic"),
            ProviderType::ClaudeOAuth
        );
        assert_eq!(
            manager.map_orchestrator_provider("openai"),
            ProviderType::Codex
        );
        assert_eq!(
            manager.map_orchestrator_provider("google"),
            ProviderType::Gemini
        );
        assert_eq!(
            manager.map_orchestrator_provider("kiro"),
            ProviderType::Kiro
        );
    }
}
