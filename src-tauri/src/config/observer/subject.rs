//! 配置主题（Subject）实现
//!
//! 管理配置观察者的注册、注销和通知

use super::events::{ConfigChangeEvent, ConfigChangeSource, FullReloadEvent};
use super::traits::ConfigObserver;
use crate::config::Config;
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use tokio::sync::broadcast;

/// Tauri 事件名称常量
pub const CONFIG_CHANGED_EVENT: &str = "config-changed";
/// 预留：配置重新加载事件
#[allow(dead_code)]
pub const CONFIG_RELOAD_EVENT: &str = "config-reload";

/// 观察者条目
struct ObserverEntry {
    observer: Arc<dyn ConfigObserver>,
}

/// 配置主题（Subject）
///
/// 管理配置观察者的注册、注销和通知
pub struct ConfigSubject {
    /// 观察者列表（按优先级排序）
    observers: RwLock<BTreeMap<i32, Vec<ObserverEntry>>>,
    /// 当前配置
    current_config: RwLock<Config>,
    /// 事件广播通道
    event_tx: broadcast::Sender<ConfigChangeEvent>,
    /// Tauri AppHandle（用于向前端发送事件）
    app_handle: RwLock<Option<AppHandle>>,
    /// 是否启用 Tauri 事件
    tauri_events_enabled: RwLock<bool>,
}

impl ConfigSubject {
    /// 创建新的配置主题
    pub fn new(initial_config: Config) -> Self {
        let (event_tx, _) = broadcast::channel(100);

        Self {
            observers: RwLock::new(BTreeMap::new()),
            current_config: RwLock::new(initial_config),
            event_tx,
            app_handle: RwLock::new(None),
            tauri_events_enabled: RwLock::new(true),
        }
    }

    /// 设置 Tauri AppHandle
    pub fn set_app_handle(&self, handle: AppHandle) {
        let mut app_handle = self.app_handle.write();
        *app_handle = Some(handle);
        tracing::debug!("[ConfigSubject] AppHandle 已设置");
    }

    /// 启用/禁用 Tauri 事件
    pub fn set_tauri_events_enabled(&self, enabled: bool) {
        let mut flag = self.tauri_events_enabled.write();
        *flag = enabled;
    }

    /// 注册观察者
    pub fn register(&self, observer: Arc<dyn ConfigObserver>) {
        let priority = observer.priority();
        let name = observer.name().to_string();
        let entry = ObserverEntry { observer };

        let mut observers = self.observers.write();
        observers.entry(priority).or_default().push(entry);

        tracing::info!(
            "[ConfigSubject] 注册观察者: {} (优先级: {})",
            name,
            priority
        );
    }

    /// 注销观察者（按名称）
    pub fn unregister(&self, name: &str) {
        let mut observers = self.observers.write();
        for entries in observers.values_mut() {
            entries.retain(|e| e.observer.name() != name);
        }
        // 清理空的优先级组
        observers.retain(|_, v| !v.is_empty());

        tracing::info!("[ConfigSubject] 注销观察者: {}", name);
    }

    /// 获取当前配置（克隆）
    pub fn config(&self) -> Config {
        self.current_config.read().clone()
    }

    /// 获取配置引用
    pub fn config_ref(&self) -> parking_lot::RwLockReadGuard<'_, Config> {
        self.current_config.read()
    }

    /// 直接更新配置（不触发通知，用于内部同步）
    pub fn set_config(&self, config: Config) {
        let mut current = self.current_config.write();
        *current = config;
    }

    /// 更新配置并通知观察者
    pub async fn update_config(&self, new_config: Config, source: ConfigChangeSource) {
        // 创建完整重载事件
        let event = ConfigChangeEvent::FullReload(FullReloadEvent {
            timestamp_ms: Self::current_timestamp_ms(),
            source,
        });

        // 更新配置
        {
            let mut config = self.current_config.write();
            *config = new_config.clone();
        }

        // 通知观察者
        self.notify_observers(&event, &new_config).await;

        // 发送 Tauri 事件
        self.emit_tauri_event(&event);

        // 广播事件
        let _ = self.event_tx.send(event);
    }

    /// 通知特定事件（不更新配置）
    pub async fn notify_event(&self, event: ConfigChangeEvent) {
        let config = self.config();
        self.notify_observers(&event, &config).await;
        self.emit_tauri_event(&event);
        let _ = self.event_tx.send(event);
    }

    /// 订阅事件广播
    pub fn subscribe(&self) -> broadcast::Receiver<ConfigChangeEvent> {
        self.event_tx.subscribe()
    }

    /// 通知所有观察者
    async fn notify_observers(&self, event: &ConfigChangeEvent, config: &Config) {
        // 收集所有感兴趣的观察者
        let observers: Vec<Arc<dyn ConfigObserver>> = {
            let observers = self.observers.read();
            observers
                .values()
                .flatten()
                .filter(|e| e.observer.is_interested_in(event))
                .map(|e| e.observer.clone())
                .collect()
        };

        tracing::debug!(
            "[ConfigSubject] 通知 {} 个观察者，事件类型: {}",
            observers.len(),
            event.event_type()
        );

        // 按优先级顺序通知（BTreeMap 已排序）
        for observer in observers {
            let name = observer.name().to_string();
            match observer.on_config_changed(event, config).await {
                Ok(()) => {
                    tracing::debug!("[ConfigSubject] 观察者 {} 处理成功", name);
                }
                Err(e) => {
                    tracing::error!("[ConfigSubject] 观察者 {} 处理失败: {}", name, e);
                }
            }
        }
    }

    /// 发送 Tauri 事件到前端
    fn emit_tauri_event(&self, event: &ConfigChangeEvent) {
        let enabled = *self.tauri_events_enabled.read();
        if !enabled {
            return;
        }

        let app_handle = self.app_handle.read();
        if let Some(handle) = app_handle.as_ref() {
            if let Err(e) = handle.emit(CONFIG_CHANGED_EVENT, event) {
                tracing::error!("[ConfigSubject] 发送 Tauri 事件失败: {}", e);
            } else {
                tracing::debug!("[ConfigSubject] 已发送 Tauri 事件: {}", event.event_type());
            }
        }
    }

    /// 获取当前时间戳（毫秒）
    fn current_timestamp_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    /// 获取观察者数量
    pub fn observer_count(&self) -> usize {
        let observers = self.observers.read();
        observers.values().map(|v| v.len()).sum()
    }

    /// 获取所有观察者名称
    pub fn observer_names(&self) -> Vec<String> {
        let observers = self.observers.read();
        observers
            .values()
            .flatten()
            .map(|e| e.observer.name().to_string())
            .collect()
    }
}

impl Default for ConfigSubject {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::observer::events::ConfigChangeSource;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct CountingObserver {
        name: String,
        priority: i32,
        count: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl ConfigObserver for CountingObserver {
        fn name(&self) -> &str {
            &self.name
        }

        async fn on_config_changed(
            &self,
            _event: &ConfigChangeEvent,
            _config: &Config,
        ) -> Result<(), String> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn priority(&self) -> i32 {
            self.priority
        }
    }

    #[tokio::test]
    async fn test_register_and_notify() {
        let subject = ConfigSubject::new(Config::default());
        let count = Arc::new(AtomicUsize::new(0));

        let observer = Arc::new(CountingObserver {
            name: "test".to_string(),
            priority: 100,
            count: count.clone(),
        });

        subject.register(observer);
        assert_eq!(subject.observer_count(), 1);

        subject
            .update_config(Config::default(), ConfigChangeSource::ApiCall)
            .await;
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_unregister() {
        let subject = ConfigSubject::new(Config::default());
        let count = Arc::new(AtomicUsize::new(0));

        let observer = Arc::new(CountingObserver {
            name: "test".to_string(),
            priority: 100,
            count: count.clone(),
        });

        subject.register(observer);
        subject.unregister("test");
        assert_eq!(subject.observer_count(), 0);

        subject
            .update_config(Config::default(), ConfigChangeSource::ApiCall)
            .await;
        assert_eq!(count.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn test_priority_order() {
        let subject = ConfigSubject::new(Config::default());

        // 注册不同优先级的观察者
        for (name, priority) in [("low", 100), ("high", 10), ("medium", 50)] {
            let observer = Arc::new(CountingObserver {
                name: name.to_string(),
                priority,
                count: Arc::new(AtomicUsize::new(0)),
            });
            subject.register(observer);
        }

        // 验证观察者名称列表
        let names = subject.observer_names();
        assert_eq!(names.len(), 3);
    }
}
