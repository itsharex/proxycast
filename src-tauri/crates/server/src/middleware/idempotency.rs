//! 幂等性中间件
//!
//! 通过 Idempotency-Key header 防止重复请求

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 幂等性配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdempotencyConfig {
    /// 是否启用
    #[serde(default)]
    pub enabled: bool,
    /// 缓存 TTL（秒）
    #[serde(default = "default_ttl_secs")]
    pub ttl_secs: u64,
    /// Header 名称
    #[serde(default = "default_header_name")]
    pub header_name: String,
}

fn default_ttl_secs() -> u64 {
    86400 // 24 小时
}
fn default_header_name() -> String {
    "Idempotency-Key".to_string()
}

impl Default for IdempotencyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ttl_secs: default_ttl_secs(),
            header_name: default_header_name(),
        }
    }
}

/// 幂等性检查结果
#[derive(Debug, PartialEq)]
pub enum IdempotencyCheck {
    /// 新请求，可以处理
    New,
    /// 正在处理中（返回 409 Conflict）
    InProgress,
    /// 已完成，有缓存响应
    Completed { status: u16, body: String },
}

/// 请求状态
#[derive(Debug, Clone)]
enum RequestState {
    /// 正在处理
    InProgress { started_at: Instant },
    /// 已完成
    Completed {
        status: u16,
        body: String,
        completed_at: Instant,
    },
}

/// 幂等性存储
pub struct IdempotencyStore {
    config: IdempotencyConfig,
    entries: Mutex<HashMap<String, RequestState>>,
}

impl IdempotencyStore {
    pub fn new(config: IdempotencyConfig) -> Self {
        Self {
            config,
            entries: Mutex::new(HashMap::new()),
        }
    }

    /// 检查幂等性键
    pub fn check(&self, key: &str) -> IdempotencyCheck {
        if !self.config.enabled {
            return IdempotencyCheck::New;
        }

        let mut entries = self.entries.lock();
        let ttl = Duration::from_secs(self.config.ttl_secs);
        let now = Instant::now();

        match entries.get(key) {
            Some(RequestState::InProgress { started_at }) => {
                // 如果处理超过 TTL，视为过期
                if now.duration_since(*started_at) > ttl {
                    entries.insert(
                        key.to_string(),
                        RequestState::InProgress { started_at: now },
                    );
                    IdempotencyCheck::New
                } else {
                    IdempotencyCheck::InProgress
                }
            }
            Some(RequestState::Completed {
                status,
                body,
                completed_at,
            }) => {
                if now.duration_since(*completed_at) > ttl {
                    entries.insert(
                        key.to_string(),
                        RequestState::InProgress { started_at: now },
                    );
                    IdempotencyCheck::New
                } else {
                    IdempotencyCheck::Completed {
                        status: *status,
                        body: body.clone(),
                    }
                }
            }
            None => {
                entries.insert(
                    key.to_string(),
                    RequestState::InProgress { started_at: now },
                );
                IdempotencyCheck::New
            }
        }
    }

    /// 标记请求完成
    pub fn complete(&self, key: &str, status: u16, body: String) {
        if !self.config.enabled {
            return;
        }
        let mut entries = self.entries.lock();
        entries.insert(
            key.to_string(),
            RequestState::Completed {
                status,
                body,
                completed_at: Instant::now(),
            },
        );
    }

    /// 移除键（请求失败时调用，允许重试）
    pub fn remove(&self, key: &str) {
        let mut entries = self.entries.lock();
        entries.remove(key);
    }

    /// 清理过期条目
    pub fn cleanup(&self) {
        let ttl = Duration::from_secs(self.config.ttl_secs);
        let now = Instant::now();
        let mut entries = self.entries.lock();
        entries.retain(|_, state| match state {
            RequestState::InProgress { started_at } => now.duration_since(*started_at) < ttl,
            RequestState::Completed { completed_at, .. } => now.duration_since(*completed_at) < ttl,
        });
    }

    /// 获取当前条目数
    pub fn len(&self) -> usize {
        self.entries.lock().len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.lock().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    fn enabled_config(ttl_secs: u64) -> IdempotencyConfig {
        IdempotencyConfig {
            enabled: true,
            ttl_secs,
            header_name: "Idempotency-Key".to_string(),
        }
    }

    #[test]
    fn test_disabled_always_new() {
        let store = IdempotencyStore::new(IdempotencyConfig::default());
        assert_eq!(store.check("key1"), IdempotencyCheck::New);
        assert_eq!(store.check("key1"), IdempotencyCheck::New);
        assert!(store.is_empty());
    }

    #[test]
    fn test_new_request() {
        let store = IdempotencyStore::new(enabled_config(60));
        assert_eq!(store.check("key1"), IdempotencyCheck::New);
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_in_progress_request() {
        let store = IdempotencyStore::new(enabled_config(60));
        assert_eq!(store.check("key1"), IdempotencyCheck::New);
        // 同一 key 再次检查应返回 InProgress
        assert_eq!(store.check("key1"), IdempotencyCheck::InProgress);
    }

    #[test]
    fn test_completed_request() {
        let store = IdempotencyStore::new(enabled_config(60));
        assert_eq!(store.check("key1"), IdempotencyCheck::New);

        store.complete("key1", 200, "ok".to_string());

        assert_eq!(
            store.check("key1"),
            IdempotencyCheck::Completed {
                status: 200,
                body: "ok".to_string(),
            }
        );
    }

    #[test]
    fn test_expired_entry() {
        let store = IdempotencyStore::new(enabled_config(1)); // 1 秒 TTL

        assert_eq!(store.check("key1"), IdempotencyCheck::New);
        store.complete("key1", 200, "ok".to_string());

        // 等待过期
        thread::sleep(Duration::from_millis(1100));

        // 过期后应视为新请求
        assert_eq!(store.check("key1"), IdempotencyCheck::New);
    }

    #[test]
    fn test_cleanup() {
        let store = IdempotencyStore::new(enabled_config(1));
        assert_eq!(store.check("key1"), IdempotencyCheck::New);
        assert_eq!(store.check("key2"), IdempotencyCheck::New);
        store.complete("key1", 200, "ok".to_string());

        thread::sleep(Duration::from_millis(1100));

        store.cleanup();
        assert!(store.is_empty(), "清理后应无过期条目");
    }

    #[test]
    fn test_remove_allows_retry() {
        let store = IdempotencyStore::new(enabled_config(60));
        assert_eq!(store.check("key1"), IdempotencyCheck::New);
        assert_eq!(store.check("key1"), IdempotencyCheck::InProgress);

        // 移除后应可重试
        store.remove("key1");
        assert_eq!(store.check("key1"), IdempotencyCheck::New);
    }

    #[test]
    fn test_default_config() {
        let config = IdempotencyConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.ttl_secs, 86400);
        assert_eq!(config.header_name, "Idempotency-Key");
    }
}
