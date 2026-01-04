//! 凭证池管理模块
//!
//! 提供多凭证管理、负载均衡和健康检查功能
//!
//! ## 模块结构
//!
//! - `types` - 凭证相关类型定义
//! - `pool` - 凭证池管理
//! - `balancer` - 负载均衡策略
//! - `health` - 健康检查
//! - `quota` - 配额管理
//! - `sync` - 数据库同步
//! - `plugin` - OAuth Provider 插件 Trait
//! - `registry` - 插件注册表
//! - `oauth_plugin_loader` - OAuth Provider 插件加载器
//! - `sdk` - ProxyCast Plugin SDK
//! - `risk` - 风控模块（限流检测、冷却期管理）
//! - `unified` - 统一凭证管理器

mod balancer;
mod health;
pub mod oauth_plugin_loader;
pub mod plugin;
mod pool;
mod quota;
pub mod registry;
pub mod risk;
pub mod sdk;
mod sync;
mod types;
mod unified;

pub use balancer::{BalanceStrategy, CooldownInfo, CredentialSelection, LoadBalancer};
pub use health::{HealthCheckConfig, HealthCheckResult, HealthChecker, HealthStatus};
pub use oauth_plugin_loader::{
    BinaryManifest, ExternalOAuthPlugin, OAuthPluginLoader, OAuthPluginManifest, ProviderManifest,
    UiManifest,
};
pub use plugin::{
    AcquiredCredential, AuthTypeInfo, CredentialCategory, CredentialConfig,
    CredentialProviderPlugin, ModelFamily, ModelInfo, OAuthPluginError, OAuthPluginInfo,
    OAuthPluginResult, PluginInstance, ProviderError, ProviderErrorType, StandardProtocol,
    TokenRefreshResult, UsageResult, ValidationResult,
};
pub use pool::{CredentialPool, PoolError, PoolStatus};
pub use quota::{
    create_shared_quota_manager, start_quota_cleanup_task, AllCredentialsExhaustedError,
    QuotaAutoSwitchResult, QuotaExceededRecord, QuotaManager,
};
pub use registry::{
    get_global_registry, init_global_registry, CredentialProviderRegistry, PluginSource,
    PluginState, PluginUpdate,
};
pub use risk::{CooldownConfig, RateLimitEvent, RateLimitStats, RiskController, RiskLevel};
pub use sdk::{
    DatabaseCallback, HttpRequestOptions, HttpResponse, JsonRpcError, JsonRpcRequest,
    JsonRpcResponse, PluginPermission, PluginSdkContext, QueryResult, SdkError, SdkMethodHandler,
    SdkResult,
};
pub use sync::{CredentialSyncService, SyncError};
pub use types::{Credential, CredentialData, CredentialStats, CredentialStatus};
pub use unified::{
    get_global_unified_manager, init_global_unified_manager, UnifiedCredentialManager,
};

#[cfg(test)]
mod tests;
