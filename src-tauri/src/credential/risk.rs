//! 风控模块
//!
//! 提供限流检测、冷却期管理和风险评估功能。
//!
//! ## 功能
//!
//! - **限流检测**: 检测 API 返回的限流错误（429、rate limit）
//! - **冷却期管理**: 自动计算和管理凭证冷却时间
//! - **风险评估**: 根据历史数据评估凭证风险等级

use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};

/// 风险等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    /// 低风险 - 正常使用
    Low,
    /// 中风险 - 接近限流阈值
    Medium,
    /// 高风险 - 频繁触发限流
    High,
    /// 危险 - 需要立即冷却
    Critical,
}

impl RiskLevel {
    /// 获取风险等级对应的冷却时间倍数
    pub fn cooldown_multiplier(&self) -> f64 {
        match self {
            RiskLevel::Low => 1.0,
            RiskLevel::Medium => 1.5,
            RiskLevel::High => 2.0,
            RiskLevel::Critical => 3.0,
        }
    }

    /// 获取风险等级描述
    pub fn description(&self) -> &'static str {
        match self {
            RiskLevel::Low => "正常",
            RiskLevel::Medium => "接近限流",
            RiskLevel::High => "频繁限流",
            RiskLevel::Critical => "需要冷却",
        }
    }
}

/// 限流事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitEvent {
    /// 凭证 ID
    pub credential_id: String,
    /// 事件时间
    pub timestamp: DateTime<Utc>,
    /// HTTP 状态码
    pub status_code: Option<u16>,
    /// 错误消息
    pub error_message: Option<String>,
    /// 建议的重试时间（秒）
    pub retry_after_secs: Option<u64>,
}

impl RateLimitEvent {
    /// 创建新的限流事件
    pub fn new(credential_id: String) -> Self {
        Self {
            credential_id,
            timestamp: Utc::now(),
            status_code: None,
            error_message: None,
            retry_after_secs: None,
        }
    }

    /// 设置状态码
    pub fn with_status_code(mut self, code: u16) -> Self {
        self.status_code = Some(code);
        self
    }

    /// 设置错误消息
    pub fn with_error_message(mut self, message: String) -> Self {
        self.error_message = Some(message);
        self
    }

    /// 设置重试时间
    pub fn with_retry_after(mut self, secs: u64) -> Self {
        self.retry_after_secs = Some(secs);
        self
    }
}

/// 冷却配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CooldownConfig {
    /// 基础冷却时间（秒）
    pub base_cooldown_secs: u64,
    /// 最大冷却时间（秒）
    pub max_cooldown_secs: u64,
    /// 冷却时间增长因子（指数退避）
    pub backoff_factor: f64,
    /// 限流事件窗口大小（保留最近 N 个事件）
    pub event_window_size: usize,
    /// 限流事件时间窗口（秒）- 只统计此时间内的事件
    pub event_time_window_secs: u64,
    /// 触发中风险的限流次数阈值
    pub medium_risk_threshold: u32,
    /// 触发高风险的限流次数阈值
    pub high_risk_threshold: u32,
    /// 触发危险的限流次数阈值
    pub critical_risk_threshold: u32,
}

impl Default for CooldownConfig {
    fn default() -> Self {
        Self {
            base_cooldown_secs: 60,       // 1 分钟
            max_cooldown_secs: 3600,      // 1 小时
            backoff_factor: 2.0,          // 指数退避因子
            event_window_size: 100,       // 保留最近 100 个事件
            event_time_window_secs: 3600, // 1 小时内的事件
            medium_risk_threshold: 3,     // 3 次限流 -> 中风险
            high_risk_threshold: 5,       // 5 次限流 -> 高风险
            critical_risk_threshold: 10,  // 10 次限流 -> 危险
        }
    }
}

/// 凭证风控状态
#[derive(Debug)]
struct CredentialRiskState {
    /// 限流事件历史
    events: VecDeque<RateLimitEvent>,
    /// 连续限流次数
    consecutive_rate_limits: AtomicU64,
    /// 当前冷却结束时间
    cooldown_until: Option<DateTime<Utc>>,
    /// 上次限流时间
    last_rate_limit: Option<DateTime<Utc>>,
}

impl CredentialRiskState {
    fn new() -> Self {
        Self {
            events: VecDeque::new(),
            consecutive_rate_limits: AtomicU64::new(0),
            cooldown_until: None,
            last_rate_limit: None,
        }
    }
}

/// 风控控制器
///
/// 管理凭证的限流检测和冷却期
pub struct RiskController {
    /// 配置
    config: CooldownConfig,
    /// 各凭证的风控状态
    states: DashMap<String, CredentialRiskState>,
}

impl RiskController {
    /// 创建新的风控控制器
    pub fn new(config: CooldownConfig) -> Self {
        Self {
            config,
            states: DashMap::new(),
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(CooldownConfig::default())
    }

    /// 获取配置
    pub fn config(&self) -> &CooldownConfig {
        &self.config
    }

    /// 记录限流事件
    ///
    /// # 返回
    /// 建议的冷却时间（秒）
    pub fn record_rate_limit(&self, event: RateLimitEvent) -> u64 {
        let credential_id = event.credential_id.clone();
        let retry_after = event.retry_after_secs;

        let mut state = self
            .states
            .entry(credential_id.clone())
            .or_insert_with(CredentialRiskState::new);

        // 更新连续限流次数
        state.consecutive_rate_limits.fetch_add(1, Ordering::SeqCst);
        state.last_rate_limit = Some(Utc::now());

        // 添加事件到历史
        state.events.push_back(event);

        // 清理过期事件
        self.cleanup_old_events(&mut state);

        // 计算冷却时间
        let cooldown_secs = self.calculate_cooldown(&state, retry_after);

        // 设置冷却结束时间
        state.cooldown_until = Some(Utc::now() + Duration::seconds(cooldown_secs as i64));

        cooldown_secs
    }

    /// 记录成功请求（重置连续限流计数）
    pub fn record_success(&self, credential_id: &str) {
        if let Some(state) = self.states.get_mut(credential_id) {
            state.consecutive_rate_limits.store(0, Ordering::SeqCst);
        }
    }

    /// 获取凭证的风险等级
    pub fn get_risk_level(&self, credential_id: &str) -> RiskLevel {
        let state = match self.states.get(credential_id) {
            Some(s) => s,
            None => return RiskLevel::Low,
        };

        let recent_count = self.count_recent_events(&state);

        if recent_count >= self.config.critical_risk_threshold {
            RiskLevel::Critical
        } else if recent_count >= self.config.high_risk_threshold {
            RiskLevel::High
        } else if recent_count >= self.config.medium_risk_threshold {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }

    /// 检查凭证是否在冷却中
    pub fn is_in_cooldown(&self, credential_id: &str) -> bool {
        self.states
            .get(credential_id)
            .and_then(|state| state.cooldown_until)
            .map(|until| Utc::now() < until)
            .unwrap_or(false)
    }

    /// 获取凭证的冷却结束时间
    pub fn get_cooldown_until(&self, credential_id: &str) -> Option<DateTime<Utc>> {
        self.states
            .get(credential_id)
            .and_then(|state| state.cooldown_until)
            .filter(|until| Utc::now() < *until)
    }

    /// 获取凭证的剩余冷却时间（秒）
    pub fn get_remaining_cooldown_secs(&self, credential_id: &str) -> Option<u64> {
        self.get_cooldown_until(credential_id).map(|until| {
            let remaining = until - Utc::now();
            remaining.num_seconds().max(0) as u64
        })
    }

    /// 手动清除凭证的冷却状态
    pub fn clear_cooldown(&self, credential_id: &str) {
        if let Some(mut state) = self.states.get_mut(credential_id) {
            state.cooldown_until = None;
            state.consecutive_rate_limits.store(0, Ordering::SeqCst);
        }
    }

    /// 获取所有处于冷却中的凭证 ID
    pub fn get_cooling_credentials(&self) -> Vec<String> {
        let now = Utc::now();
        self.states
            .iter()
            .filter(|entry| {
                entry
                    .value()
                    .cooldown_until
                    .map(|until| now < until)
                    .unwrap_or(false)
            })
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// 获取凭证的限流事件统计
    pub fn get_event_stats(&self, credential_id: &str) -> Option<RateLimitStats> {
        self.states.get(credential_id).map(|state| {
            let recent_count = self.count_recent_events(&state);
            let consecutive = state.consecutive_rate_limits.load(Ordering::SeqCst);

            RateLimitStats {
                total_events: state.events.len(),
                recent_events: recent_count as usize,
                consecutive_rate_limits: consecutive,
                last_rate_limit: state.last_rate_limit,
                cooldown_until: state.cooldown_until,
                risk_level: self.get_risk_level(credential_id),
            }
        })
    }

    /// 检测响应是否为限流错误
    pub fn is_rate_limit_error(status_code: u16, body: Option<&str>) -> bool {
        // HTTP 429 Too Many Requests
        if status_code == 429 {
            return true;
        }

        // 检查响应体中的限流关键词
        if let Some(body) = body {
            let body_lower = body.to_lowercase();
            if body_lower.contains("rate limit")
                || body_lower.contains("rate_limit")
                || body_lower.contains("ratelimit")
                || body_lower.contains("too many requests")
                || body_lower.contains("quota exceeded")
                || body_lower.contains("resource_exhausted")
            {
                return true;
            }
        }

        false
    }

    /// 从响应头解析 Retry-After
    pub fn parse_retry_after(header_value: &str) -> Option<u64> {
        // 尝试解析为秒数
        if let Ok(secs) = header_value.parse::<u64>() {
            return Some(secs);
        }

        // 尝试解析为 HTTP 日期格式
        if let Ok(date) = DateTime::parse_from_rfc2822(header_value) {
            let until = date.with_timezone(&Utc);
            let now = Utc::now();
            if until > now {
                return Some((until - now).num_seconds() as u64);
            }
        }

        None
    }

    /// 清理过期事件
    fn cleanup_old_events(&self, state: &mut CredentialRiskState) {
        let cutoff = Utc::now() - Duration::seconds(self.config.event_time_window_secs as i64);

        // 移除过期事件
        while let Some(front) = state.events.front() {
            if front.timestamp < cutoff {
                state.events.pop_front();
            } else {
                break;
            }
        }

        // 限制事件数量
        while state.events.len() > self.config.event_window_size {
            state.events.pop_front();
        }
    }

    /// 统计最近的限流事件数
    fn count_recent_events(&self, state: &CredentialRiskState) -> u32 {
        let cutoff = Utc::now() - Duration::seconds(self.config.event_time_window_secs as i64);
        state
            .events
            .iter()
            .filter(|e| e.timestamp >= cutoff)
            .count() as u32
    }

    /// 计算冷却时间
    fn calculate_cooldown(&self, state: &CredentialRiskState, retry_after: Option<u64>) -> u64 {
        // 如果有 Retry-After，优先使用
        if let Some(retry) = retry_after {
            return retry.min(self.config.max_cooldown_secs);
        }

        // 使用指数退避计算冷却时间
        let consecutive = state.consecutive_rate_limits.load(Ordering::SeqCst);
        let base = self.config.base_cooldown_secs as f64;
        let factor = self.config.backoff_factor;

        // cooldown = base * factor^(consecutive - 1)
        let cooldown = if consecutive > 0 {
            base * factor.powi((consecutive - 1) as i32)
        } else {
            base
        };

        // 根据风险等级调整
        let risk_level = self.get_risk_level_from_state(state);
        let adjusted = cooldown * risk_level.cooldown_multiplier();

        // 限制在最大值内
        (adjusted as u64).min(self.config.max_cooldown_secs)
    }

    /// 从状态计算风险等级
    fn get_risk_level_from_state(&self, state: &CredentialRiskState) -> RiskLevel {
        let recent_count = self.count_recent_events(state);

        if recent_count >= self.config.critical_risk_threshold {
            RiskLevel::Critical
        } else if recent_count >= self.config.high_risk_threshold {
            RiskLevel::High
        } else if recent_count >= self.config.medium_risk_threshold {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }
}

impl Default for RiskController {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// 限流事件统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitStats {
    /// 总事件数
    pub total_events: usize,
    /// 最近事件数（时间窗口内）
    pub recent_events: usize,
    /// 连续限流次数
    pub consecutive_rate_limits: u64,
    /// 上次限流时间
    pub last_rate_limit: Option<DateTime<Utc>>,
    /// 冷却结束时间
    pub cooldown_until: Option<DateTime<Utc>>,
    /// 风险等级
    pub risk_level: RiskLevel,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_controller_new() {
        let controller = RiskController::with_defaults();
        assert_eq!(controller.config().base_cooldown_secs, 60);
    }

    #[test]
    fn test_record_rate_limit() {
        let controller = RiskController::with_defaults();
        let event = RateLimitEvent::new("cred-1".to_string()).with_status_code(429);

        let cooldown = controller.record_rate_limit(event);
        assert!(cooldown >= 60); // 至少是基础冷却时间

        assert!(controller.is_in_cooldown("cred-1"));
        assert_eq!(controller.get_risk_level("cred-1"), RiskLevel::Low);
    }

    #[test]
    fn test_risk_level_escalation() {
        let controller = RiskController::with_defaults();

        // 记录多次限流事件
        for i in 0..5 {
            let event = RateLimitEvent::new("cred-1".to_string())
                .with_status_code(429)
                .with_error_message(format!("Rate limit {}", i));
            controller.record_rate_limit(event);
        }

        // 应该达到高风险
        assert_eq!(controller.get_risk_level("cred-1"), RiskLevel::High);
    }

    #[test]
    fn test_record_success_resets_consecutive() {
        let controller = RiskController::with_defaults();

        // 记录限流
        let event = RateLimitEvent::new("cred-1".to_string());
        controller.record_rate_limit(event);

        // 记录成功
        controller.record_success("cred-1");

        // 连续计数应该重置
        let stats = controller.get_event_stats("cred-1").unwrap();
        assert_eq!(stats.consecutive_rate_limits, 0);
    }

    #[test]
    fn test_clear_cooldown() {
        let controller = RiskController::with_defaults();

        let event = RateLimitEvent::new("cred-1".to_string());
        controller.record_rate_limit(event);

        assert!(controller.is_in_cooldown("cred-1"));

        controller.clear_cooldown("cred-1");

        assert!(!controller.is_in_cooldown("cred-1"));
    }

    #[test]
    fn test_is_rate_limit_error() {
        assert!(RiskController::is_rate_limit_error(429, None));
        assert!(RiskController::is_rate_limit_error(
            200,
            Some("rate limit exceeded")
        ));
        assert!(RiskController::is_rate_limit_error(
            500,
            Some("RESOURCE_EXHAUSTED")
        ));
        assert!(!RiskController::is_rate_limit_error(200, Some("success")));
    }

    #[test]
    fn test_parse_retry_after() {
        assert_eq!(RiskController::parse_retry_after("60"), Some(60));
        assert_eq!(RiskController::parse_retry_after("3600"), Some(3600));
        assert!(RiskController::parse_retry_after("invalid").is_none());
    }

    #[test]
    fn test_retry_after_priority() {
        let controller = RiskController::with_defaults();

        // 使用 retry_after 的事件
        let event = RateLimitEvent::new("cred-1".to_string()).with_retry_after(120);

        let cooldown = controller.record_rate_limit(event);
        assert_eq!(cooldown, 120); // 应该使用 retry_after 的值
    }

    #[test]
    fn test_exponential_backoff() {
        let controller = RiskController::with_defaults();

        // 第一次限流
        let event1 = RateLimitEvent::new("cred-1".to_string());
        let cooldown1 = controller.record_rate_limit(event1);

        // 第二次限流（应该更长）
        let event2 = RateLimitEvent::new("cred-1".to_string());
        let cooldown2 = controller.record_rate_limit(event2);

        assert!(cooldown2 > cooldown1);
    }

    #[test]
    fn test_get_cooling_credentials() {
        let controller = RiskController::with_defaults();

        controller.record_rate_limit(RateLimitEvent::new("cred-1".to_string()));
        controller.record_rate_limit(RateLimitEvent::new("cred-2".to_string()));

        let cooling = controller.get_cooling_credentials();
        assert_eq!(cooling.len(), 2);
        assert!(cooling.contains(&"cred-1".to_string()));
        assert!(cooling.contains(&"cred-2".to_string()));
    }

    #[test]
    fn test_risk_level_cooldown_multiplier() {
        assert_eq!(RiskLevel::Low.cooldown_multiplier(), 1.0);
        assert_eq!(RiskLevel::Medium.cooldown_multiplier(), 1.5);
        assert_eq!(RiskLevel::High.cooldown_multiplier(), 2.0);
        assert_eq!(RiskLevel::Critical.cooldown_multiplier(), 3.0);
    }
}
