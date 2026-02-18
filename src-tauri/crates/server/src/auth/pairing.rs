//! 配对认证系统
//!
//! 提供一次性配对码认证流程：
//! 1. 启动时生成配对码
//! 2. 客户端通过配对码获取 bearer token
//! 3. 后续请求使用 bearer token 认证
//! 4. 暴力破解保护

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::time::{Duration, Instant};

const MAX_FAILED_ATTEMPTS: u32 = 5;
const LOCKOUT_DURATION_SECS: u64 = 300; // 5 分钟

/// 配对认证配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingConfig {
    /// 是否启用配对认证
    #[serde(default)]
    pub enabled: bool,
}

impl Default for PairingConfig {
    fn default() -> Self {
        Self { enabled: false }
    }
}

/// 失败尝试状态
#[derive(Debug)]
struct FailureState {
    count: u32,
    window_start: Option<Instant>,
    blocked_until: Option<Instant>,
}

impl Default for FailureState {
    fn default() -> Self {
        Self {
            count: 0,
            window_start: None,
            blocked_until: None,
        }
    }
}

/// 配对结果
#[derive(Debug)]
pub enum PairingResult {
    /// 配对成功，返回 token
    Success { token: String },
    /// 配对码错误
    InvalidCode,
    /// 被锁定
    Locked { retry_after_secs: u64 },
    /// 配对未启用
    Disabled,
}

/// 认证结果
#[derive(Debug, PartialEq)]
pub enum AuthResult {
    /// 认证成功
    Authenticated,
    /// 未认证
    Unauthenticated,
    /// 配对未启用（允许通过）
    Disabled,
}

/// 配对认证守卫
pub struct PairingGuard {
    config: PairingConfig,
    /// 当前配对码
    pairing_code: Mutex<Option<String>>,
    /// 已配对的 token（存储 SHA-256 哈希）
    paired_tokens: Mutex<HashSet<String>>,
    /// 失败尝试追踪
    failed_attempts: Mutex<FailureState>,
}

impl PairingGuard {
    pub fn new(config: PairingConfig) -> Self {
        let code = if config.enabled {
            Some(Self::generate_pairing_code())
        } else {
            None
        };

        if let Some(ref code) = code {
            tracing::info!("========================================");
            tracing::info!("配对码: {}", code);
            tracing::info!("========================================");
        }

        Self {
            config,
            pairing_code: Mutex::new(code),
            paired_tokens: Mutex::new(HashSet::new()),
            failed_attempts: Mutex::new(FailureState::default()),
        }
    }

    /// 创建带指定配对码的守卫（用于测试）
    #[cfg(test)]
    fn with_code(config: PairingConfig, code: String) -> Self {
        Self {
            config,
            pairing_code: Mutex::new(Some(code)),
            paired_tokens: Mutex::new(HashSet::new()),
            failed_attempts: Mutex::new(FailureState::default()),
        }
    }

    /// 生成 6 位配对码
    fn generate_pairing_code() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        format!("{:06}", rng.gen_range(0..1_000_000))
    }

    /// 生成 bearer token
    fn generate_token() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        hex::encode(bytes)
    }

    /// 计算 token 的 SHA-256 哈希
    fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// 尝试配对
    pub fn pair(&self, code: &str) -> PairingResult {
        if !self.config.enabled {
            return PairingResult::Disabled;
        }

        // 检查是否被锁定
        {
            let state = self.failed_attempts.lock();
            if let Some(blocked_until) = state.blocked_until {
                if Instant::now() < blocked_until {
                    let remaining = blocked_until.duration_since(Instant::now()).as_secs();
                    return PairingResult::Locked {
                        retry_after_secs: remaining + 1,
                    };
                }
            }
        }

        // 验证配对码
        let valid = {
            let pairing_code = self.pairing_code.lock();
            pairing_code.as_deref() == Some(code)
        };

        if valid {
            // 重置失败计数
            {
                let mut state = self.failed_attempts.lock();
                *state = FailureState::default();
            }

            // 生成 token
            let token = Self::generate_token();
            let hash = Self::hash_token(&token);
            self.paired_tokens.lock().insert(hash);

            PairingResult::Success { token }
        } else {
            // 记录失败
            let mut state = self.failed_attempts.lock();
            let now = Instant::now();

            match state.window_start {
                Some(start)
                    if now.duration_since(start) < Duration::from_secs(LOCKOUT_DURATION_SECS) =>
                {
                    state.count += 1;
                }
                _ => {
                    state.count = 1;
                    state.window_start = Some(now);
                }
            }

            if state.count >= MAX_FAILED_ATTEMPTS {
                state.blocked_until = Some(now + Duration::from_secs(LOCKOUT_DURATION_SECS));
                tracing::warn!(
                    "配对认证：暴力破解保护触发，锁定 {} 秒",
                    LOCKOUT_DURATION_SECS
                );
            }

            PairingResult::InvalidCode
        }
    }

    /// 验证 bearer token
    pub fn authenticate(&self, token: &str) -> AuthResult {
        if !self.config.enabled {
            return AuthResult::Disabled;
        }

        let hash = Self::hash_token(token);
        let tokens = self.paired_tokens.lock();

        if tokens.contains(&hash) {
            AuthResult::Authenticated
        } else {
            AuthResult::Unauthenticated
        }
    }

    /// 是否启用
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enabled_config() -> PairingConfig {
        PairingConfig { enabled: true }
    }

    #[test]
    fn test_disabled_pairing() {
        let guard = PairingGuard::new(PairingConfig::default());
        assert!(!guard.is_enabled());
        assert!(matches!(guard.pair("anything"), PairingResult::Disabled));
    }

    #[test]
    fn test_successful_pairing() {
        let guard = PairingGuard::with_code(enabled_config(), "123456".to_string());

        match guard.pair("123456") {
            PairingResult::Success { token } => {
                assert_eq!(token.len(), 64); // 32 bytes hex
                assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
            }
            other => panic!("期望 Success，得到 {:?}", other),
        }
    }

    #[test]
    fn test_invalid_code() {
        let guard = PairingGuard::with_code(enabled_config(), "123456".to_string());
        assert!(matches!(guard.pair("000000"), PairingResult::InvalidCode));
    }

    #[test]
    fn test_authentication() {
        let guard = PairingGuard::with_code(enabled_config(), "123456".to_string());

        let token = match guard.pair("123456") {
            PairingResult::Success { token } => token,
            _ => panic!("配对应成功"),
        };

        assert_eq!(guard.authenticate(&token), AuthResult::Authenticated);
        assert_eq!(guard.authenticate("bad_token"), AuthResult::Unauthenticated);
    }

    #[test]
    fn test_brute_force_protection() {
        let guard = PairingGuard::with_code(enabled_config(), "123456".to_string());

        // 5 次失败触发锁定
        for _ in 0..MAX_FAILED_ATTEMPTS {
            assert!(matches!(guard.pair("000000"), PairingResult::InvalidCode));
        }

        // 第 6 次应被锁定
        match guard.pair("000000") {
            PairingResult::Locked { retry_after_secs } => {
                assert!(retry_after_secs > 0);
                assert!(retry_after_secs <= LOCKOUT_DURATION_SECS + 1);
            }
            other => panic!("期望 Locked，得到 {:?}", other),
        }

        // 即使用正确码也应被锁定
        assert!(matches!(guard.pair("123456"), PairingResult::Locked { .. }));
    }

    #[test]
    fn test_disabled_auth_allows_all() {
        let guard = PairingGuard::new(PairingConfig::default());
        assert_eq!(guard.authenticate("any_token"), AuthResult::Disabled);
    }

    #[test]
    fn test_default_config() {
        let config = PairingConfig::default();
        assert!(!config.enabled);
    }
}
