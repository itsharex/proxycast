//! 安全中间件
//!
//! 提供请求体大小限制和请求超时控制

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 安全中间件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMiddlewareConfig {
    /// 最大请求体大小（字节），默认 10MB
    #[serde(default = "default_max_body_size")]
    pub max_body_size: usize,
    /// 请求超时（秒），默认 300 秒（LLM 请求可能很长）
    #[serde(default = "default_request_timeout_secs")]
    pub request_timeout_secs: u64,
}

fn default_max_body_size() -> usize {
    10 * 1024 * 1024 // 10MB
}

fn default_request_timeout_secs() -> u64 {
    300 // 5 分钟
}

impl Default for SecurityMiddlewareConfig {
    fn default() -> Self {
        Self {
            max_body_size: default_max_body_size(),
            request_timeout_secs: default_request_timeout_secs(),
        }
    }
}

impl SecurityMiddlewareConfig {
    /// 获取请求超时 Duration
    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.request_timeout_secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SecurityMiddlewareConfig::default();
        assert_eq!(config.max_body_size, 10 * 1024 * 1024);
        assert_eq!(config.request_timeout_secs, 300);
    }

    #[test]
    fn test_request_timeout() {
        let config = SecurityMiddlewareConfig {
            max_body_size: 1024,
            request_timeout_secs: 60,
        };
        assert_eq!(config.request_timeout(), Duration::from_secs(60));
    }
}
