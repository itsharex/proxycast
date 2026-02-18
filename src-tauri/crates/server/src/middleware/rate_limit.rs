//! 滑动窗口速率限制中间件
//!
//! 基于客户端 IP 的请求速率限制，防止 API 滥用

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

/// 速率限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// 是否启用
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// 窗口内最大请求数
    #[serde(default = "default_requests_per_minute")]
    pub requests_per_minute: u32,
    /// 窗口大小（秒）
    #[serde(default = "default_window_secs")]
    pub window_secs: u64,
}

fn default_enabled() -> bool {
    false
}
fn default_requests_per_minute() -> u32 {
    60
}
fn default_window_secs() -> u64 {
    60
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            requests_per_minute: 60,
            window_secs: 60,
        }
    }
}

/// 滑动窗口速率限制器
pub struct SlidingWindowRateLimiter {
    config: RateLimitConfig,
    /// 客户端 IP -> 请求时间戳列表
    requests: Mutex<HashMap<String, Vec<Instant>>>,
}

impl SlidingWindowRateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            requests: Mutex::new(HashMap::new()),
        }
    }

    /// 检查是否允许请求
    pub fn check_rate_limit(&self, client_id: &str) -> RateLimitResult {
        if !self.config.enabled {
            return RateLimitResult::Allowed;
        }

        let now = Instant::now();
        let window = Duration::from_secs(self.config.window_secs);
        let mut requests = self.requests.lock();

        let timestamps = requests.entry(client_id.to_string()).or_default();

        // 清理窗口外的请求
        timestamps.retain(|t| now.duration_since(*t) < window);

        if timestamps.len() >= self.config.requests_per_minute as usize {
            // 计算最早请求到窗口结束的剩余时间
            let oldest = timestamps.first().copied();
            let retry_after = oldest
                .map(|t| window.saturating_sub(now.duration_since(t)))
                .unwrap_or(window);
            RateLimitResult::Limited { retry_after }
        } else {
            timestamps.push(now);
            RateLimitResult::Allowed
        }
    }

    /// 清理过期条目（应定期调用）
    pub fn cleanup(&self) {
        let now = Instant::now();
        let window = Duration::from_secs(self.config.window_secs);
        let mut requests = self.requests.lock();

        requests.retain(|_, timestamps| {
            timestamps.retain(|t| now.duration_since(*t) < window);
            !timestamps.is_empty()
        });
    }
}

/// 速率限制检查结果
#[derive(Debug)]
pub enum RateLimitResult {
    /// 允许
    Allowed,
    /// 被限制
    Limited {
        /// 建议重试等待时间
        retry_after: Duration,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_disabled_allows_all() {
        let limiter = SlidingWindowRateLimiter::new(RateLimitConfig {
            enabled: false,
            requests_per_minute: 1,
            window_secs: 60,
        });

        // 即使超过限制，禁用时也应全部允许
        for _ in 0..100 {
            assert!(matches!(
                limiter.check_rate_limit("client1"),
                RateLimitResult::Allowed
            ));
        }
    }

    #[test]
    fn test_within_limit() {
        let limiter = SlidingWindowRateLimiter::new(RateLimitConfig {
            enabled: true,
            requests_per_minute: 5,
            window_secs: 60,
        });

        for _ in 0..5 {
            assert!(matches!(
                limiter.check_rate_limit("client1"),
                RateLimitResult::Allowed
            ));
        }
    }

    #[test]
    fn test_exceeds_limit() {
        let limiter = SlidingWindowRateLimiter::new(RateLimitConfig {
            enabled: true,
            requests_per_minute: 3,
            window_secs: 60,
        });

        // 前 3 个请求应允许
        for _ in 0..3 {
            assert!(matches!(
                limiter.check_rate_limit("client1"),
                RateLimitResult::Allowed
            ));
        }

        // 第 4 个应被限制
        match limiter.check_rate_limit("client1") {
            RateLimitResult::Limited { retry_after } => {
                assert!(retry_after.as_secs() <= 60);
            }
            RateLimitResult::Allowed => panic!("应该被限制"),
        }
    }

    #[test]
    fn test_window_expiry() {
        let limiter = SlidingWindowRateLimiter::new(RateLimitConfig {
            enabled: true,
            requests_per_minute: 2,
            window_secs: 1, // 1 秒窗口，方便测试过期
        });

        // 用完配额
        assert!(matches!(
            limiter.check_rate_limit("client1"),
            RateLimitResult::Allowed
        ));
        assert!(matches!(
            limiter.check_rate_limit("client1"),
            RateLimitResult::Allowed
        ));
        assert!(matches!(
            limiter.check_rate_limit("client1"),
            RateLimitResult::Limited { .. }
        ));

        // 等待窗口过期
        thread::sleep(Duration::from_millis(1100));

        // 窗口过期后应重新允许
        assert!(matches!(
            limiter.check_rate_limit("client1"),
            RateLimitResult::Allowed
        ));
    }

    #[test]
    fn test_cleanup() {
        let limiter = SlidingWindowRateLimiter::new(RateLimitConfig {
            enabled: true,
            requests_per_minute: 10,
            window_secs: 1,
        });

        // 添加一些请求
        limiter.check_rate_limit("client1");
        limiter.check_rate_limit("client2");

        // 等待窗口过期
        thread::sleep(Duration::from_millis(1100));

        // 清理应移除过期条目
        limiter.cleanup();

        let requests = limiter.requests.lock();
        assert!(requests.is_empty(), "清理后应无过期条目");
    }

    #[test]
    fn test_default_config() {
        let config = RateLimitConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.requests_per_minute, 60);
        assert_eq!(config.window_secs, 60);
    }
}
