use crate::browser_interceptor::{BrowserInterceptorError, InterceptorState, Result};
use chrono::Utc;
use std::sync::{Arc, RwLock};
use tokio::time::{Duration, Instant};

/// 状态管理器，负责管理拦截器的状态和恢复机制
pub struct StateManager {
    state: Arc<RwLock<InterceptorState>>,
    original_system_state: Arc<RwLock<Option<SystemState>>>,
    temporary_disable_timer: Arc<RwLock<Option<Instant>>>,
}

/// 系统原始状态备份
#[derive(Debug, Clone)]
pub struct SystemState {
    pub default_browser: Option<String>,
    pub registry_backup: std::collections::HashMap<String, String>,
    pub environment_backup: std::collections::HashMap<String, String>,
    pub timestamp: chrono::DateTime<Utc>,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(InterceptorState::default())),
            original_system_state: Arc::new(RwLock::new(None)),
            temporary_disable_timer: Arc::new(RwLock::new(None)),
        }
    }

    /// 获取当前状态
    pub fn get_state(&self) -> Result<InterceptorState> {
        self.state
            .read()
            .map_err(|e| BrowserInterceptorError::StateError(format!("读取状态失败: {e}")))
            .map(|state| state.clone())
    }

    /// 启用拦截器
    pub async fn enable_interceptor(&self) -> Result<()> {
        // 备份系统状态
        self.backup_system_state().await?;

        // 更新状态
        {
            let mut state = self
                .state
                .write()
                .map_err(|e| BrowserInterceptorError::StateError(format!("写入状态失败: {e}")))?;

            state.enabled = true;
            state.can_restore = true;
            state.last_activity = Some(Utc::now());
        }

        tracing::info!("浏览器拦截器已启用");
        Ok(())
    }

    /// 禁用拦截器
    pub async fn disable_interceptor(&self) -> Result<()> {
        {
            let mut state = self
                .state
                .write()
                .map_err(|e| BrowserInterceptorError::StateError(format!("写入状态失败: {e}")))?;

            state.enabled = false;
            state.active_hooks.clear();
            state.last_activity = Some(Utc::now());
        }

        tracing::info!("浏览器拦截器已禁用");
        Ok(())
    }

    /// 临时禁用拦截器
    pub async fn temporary_disable(&self, duration_seconds: u64) -> Result<()> {
        self.disable_interceptor().await?;

        // 设置定时器
        {
            let mut timer = self
                .temporary_disable_timer
                .write()
                .map_err(|e| BrowserInterceptorError::StateError(format!("设置定时器失败: {e}")))?;
            *timer = Some(Instant::now() + Duration::from_secs(duration_seconds));
        }

        // 启动后台任务来重新启用
        let state_manager = self.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(duration_seconds)).await;
            if let Err(e) = state_manager.enable_interceptor().await {
                tracing::error!("自动重新启用拦截器失败: {}", e);
            } else {
                tracing::info!("拦截器已自动重新启用");
            }
        });

        tracing::info!("拦截器已临时禁用 {} 秒", duration_seconds);
        Ok(())
    }

    /// 恢复正常浏览器行为
    pub async fn restore_normal_behavior(&self) -> Result<()> {
        // 先禁用拦截器
        self.disable_interceptor().await?;

        // 恢复系统状态
        self.restore_system_state().await?;

        {
            let mut state = self
                .state
                .write()
                .map_err(|e| BrowserInterceptorError::StateError(format!("写入状态失败: {e}")))?;
            state.can_restore = false;
        }

        tracing::info!("已恢复正常浏览器行为");
        Ok(())
    }

    /// 增加拦截计数
    pub fn increment_intercept_count(&self) -> Result<()> {
        let mut state = self
            .state
            .write()
            .map_err(|e| BrowserInterceptorError::StateError(format!("写入状态失败: {e}")))?;

        state.intercepted_count += 1;
        state.last_activity = Some(Utc::now());

        Ok(())
    }

    /// 添加活跃钩子
    pub fn add_active_hook(&self, hook_name: String) -> Result<()> {
        let mut state = self
            .state
            .write()
            .map_err(|e| BrowserInterceptorError::StateError(format!("写入状态失败: {e}")))?;

        if !state.active_hooks.contains(&hook_name) {
            state.active_hooks.push(hook_name);
        }

        Ok(())
    }

    /// 移除活跃钩子
    pub fn remove_active_hook(&self, hook_name: &str) -> Result<()> {
        let mut state = self
            .state
            .write()
            .map_err(|e| BrowserInterceptorError::StateError(format!("写入状态失败: {e}")))?;

        state.active_hooks.retain(|h| h != hook_name);

        Ok(())
    }

    /// 备份系统状态
    async fn backup_system_state(&self) -> Result<()> {
        let system_state = SystemState {
            default_browser: self.get_default_browser().await?,
            registry_backup: self.backup_registry_keys().await?,
            environment_backup: self.backup_environment_variables().await?,
            timestamp: Utc::now(),
        };

        {
            let mut backup = self.original_system_state.write().map_err(|e| {
                BrowserInterceptorError::StateError(format!("备份系统状态失败: {e}"))
            })?;
            *backup = Some(system_state);
        }

        tracing::info!("系统状态已备份");
        Ok(())
    }

    /// 恢复系统状态
    async fn restore_system_state(&self) -> Result<()> {
        let backup = {
            let backup_guard = self.original_system_state.read().map_err(|e| {
                BrowserInterceptorError::StateError(format!("读取备份状态失败: {e}"))
            })?;
            backup_guard.clone()
        };

        if let Some(system_state) = backup {
            // 恢复默认浏览器
            if let Some(default_browser) = &system_state.default_browser {
                self.restore_default_browser(default_browser).await?;
            }

            // 恢复注册表项
            self.restore_registry_keys(&system_state.registry_backup)
                .await?;

            // 恢复环境变量
            self.restore_environment_variables(&system_state.environment_backup)
                .await?;

            tracing::info!("系统状态已恢复到 {} 的备份", system_state.timestamp);
        } else {
            tracing::warn!("没有找到系统状态备份");
        }

        Ok(())
    }

    /// 获取默认浏览器（平台特定实现）
    async fn get_default_browser(&self) -> Result<Option<String>> {
        #[cfg(target_os = "macos")]
        {
            // macOS 实现
            Ok(None)
        }

        #[cfg(target_os = "linux")]
        {
            // Linux 实现
            Ok(None)
        }

        #[cfg(target_os = "windows")]
        {
            // Windows 实现 - 简化版本
            Ok(None)
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Ok(None)
        }
    }

    /// 备份注册表项
    async fn backup_registry_keys(&self) -> Result<std::collections::HashMap<String, String>> {
        let backup = std::collections::HashMap::new();
        // 平台特定实现
        Ok(backup)
    }

    /// 备份环境变量
    async fn backup_environment_variables(
        &self,
    ) -> Result<std::collections::HashMap<String, String>> {
        let mut backup = std::collections::HashMap::new();

        // 备份可能影响浏览器启动的环境变量
        if let Ok(browser) = std::env::var("BROWSER") {
            backup.insert("BROWSER".to_string(), browser);
        }

        Ok(backup)
    }

    /// 恢复默认浏览器
    async fn restore_default_browser(&self, _browser: &str) -> Result<()> {
        // 平台特定实现
        Ok(())
    }

    /// 恢复注册表项
    async fn restore_registry_keys(
        &self,
        _backup: &std::collections::HashMap<String, String>,
    ) -> Result<()> {
        // 平台特定实现
        Ok(())
    }

    /// 恢复环境变量
    async fn restore_environment_variables(
        &self,
        backup: &std::collections::HashMap<String, String>,
    ) -> Result<()> {
        for (key, value) in backup {
            std::env::set_var(key, value);
        }
        Ok(())
    }
}

impl Clone for StateManager {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            original_system_state: Arc::clone(&self.original_system_state),
            temporary_disable_timer: Arc::clone(&self.temporary_disable_timer),
        }
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}
