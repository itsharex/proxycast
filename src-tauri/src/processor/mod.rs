//! 请求处理器模块
//!
//! 提供统一的请求处理管道，集成路由、容错、监控、插件等功能模块。
//!
//! # 架构
//!
//! 请求处理流程：
//! 1. 认证 (AuthStep)
//! 2. 参数注入 (InjectionStep)
//! 3. 路由解析 (RoutingStep)
//! 4. 插件前置钩子 (PluginPreStep)
//! 5. Provider 调用 (ProviderStep) - 包含重试和故障转移
//! 6. 插件后置钩子 (PluginPostStep)
//! 7. 统计记录 (TelemetryStep)

mod context;
mod error;
mod steps;

pub use context::RequestContext;
pub use error::ProcessError;
pub use steps::{
    AuthStep, InjectionStep, PipelineStep, PluginPostStep, PluginPreStep, ProviderStep,
    RoutingStep, TelemetryStep,
};

use crate::injection::Injector;
use crate::plugin::PluginManager;
use crate::resilience::{Failover, Retrier, TimeoutController};
use crate::router::{ModelMapper, Router};
use crate::services::provider_pool_service::ProviderPoolService;
use crate::telemetry::{StatsAggregator, TokenTracker};
use parking_lot::RwLock as ParkingLotRwLock;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 统一的请求处理器
///
/// 集成所有功能模块，提供完整的请求处理管道
pub struct RequestProcessor {
    /// 路由器
    pub router: Arc<RwLock<Router>>,
    /// 模型映射器
    pub mapper: Arc<RwLock<ModelMapper>>,
    /// 参数注入器
    pub injector: Arc<RwLock<Injector>>,
    /// 重试器
    pub retrier: Arc<Retrier>,
    /// 故障转移器
    pub failover: Arc<Failover>,
    /// 超时控制器
    pub timeout: Arc<TimeoutController>,
    /// 插件管理器
    pub plugins: Arc<PluginManager>,
    /// 统计聚合器（使用 parking_lot::RwLock 以支持与 TelemetryState 共享）
    pub stats: Arc<ParkingLotRwLock<StatsAggregator>>,
    /// Token 追踪器（使用 parking_lot::RwLock 以支持与 TelemetryState 共享）
    pub tokens: Arc<ParkingLotRwLock<TokenTracker>>,
    /// 凭证池服务
    pub pool_service: Arc<ProviderPoolService>,
    /// 热重载协调锁（避免配置更新期间请求读取不一致的配置）
    pub reload_lock: Arc<RwLock<()>>,
}

impl RequestProcessor {
    /// 创建新的请求处理器
    pub fn new(
        router: Arc<RwLock<Router>>,
        mapper: Arc<RwLock<ModelMapper>>,
        injector: Arc<RwLock<Injector>>,
        retrier: Arc<Retrier>,
        failover: Arc<Failover>,
        timeout: Arc<TimeoutController>,
        plugins: Arc<PluginManager>,
        stats: Arc<ParkingLotRwLock<StatsAggregator>>,
        tokens: Arc<ParkingLotRwLock<TokenTracker>>,
        pool_service: Arc<ProviderPoolService>,
    ) -> Self {
        Self {
            router,
            mapper,
            injector,
            retrier,
            failover,
            timeout,
            plugins,
            stats,
            tokens,
            pool_service,
            reload_lock: Arc::new(RwLock::new(())),
        }
    }

    /// 使用默认配置创建请求处理器
    pub fn with_defaults(pool_service: Arc<ProviderPoolService>) -> Self {
        Self {
            router: Arc::new(RwLock::new(Self::create_router_with_defaults())),
            mapper: Arc::new(RwLock::new(ModelMapper::new())),
            injector: Arc::new(RwLock::new(Injector::new())),
            retrier: Arc::new(Retrier::with_defaults()),
            failover: Arc::new(Failover::with_defaults()),
            timeout: Arc::new(TimeoutController::with_defaults()),
            plugins: Arc::new(PluginManager::with_defaults()),
            stats: Arc::new(ParkingLotRwLock::new(StatsAggregator::with_defaults())),
            tokens: Arc::new(ParkingLotRwLock::new(TokenTracker::with_defaults())),
            pool_service,
            reload_lock: Arc::new(RwLock::new(())),
        }
    }

    /// 创建带默认路由规则的路由器
    ///
    /// 注意：不再添加硬编码的路由规则，让用户设置的默认 Provider 生效
    /// 用户可以通过 UI 或配置文件自定义路由规则
    fn create_router_with_defaults() -> Router {
        // 创建空的路由器，默认 Provider 会在启动时从配置中设置
        // 不要硬编码任何 Provider，避免与用户配置冲突
        let router = Router::new_empty();

        tracing::info!("[ROUTER] 初始化空路由器，等待从配置加载默认 Provider");

        router
    }

    /// 使用共享的统计和 Token 追踪器创建请求处理器
    ///
    /// 这允许 RequestProcessor 与 TelemetryState 共享同一个 StatsAggregator 和 TokenTracker，
    /// 使得请求处理过程中记录的统计数据能够在前端监控页面中显示。
    pub fn with_shared_telemetry(
        pool_service: Arc<ProviderPoolService>,
        stats: Arc<ParkingLotRwLock<StatsAggregator>>,
        tokens: Arc<ParkingLotRwLock<TokenTracker>>,
    ) -> Self {
        Self {
            router: Arc::new(RwLock::new(Self::create_router_with_defaults())),
            mapper: Arc::new(RwLock::new(ModelMapper::new())),
            injector: Arc::new(RwLock::new(Injector::new())),
            retrier: Arc::new(Retrier::with_defaults()),
            failover: Arc::new(Failover::with_defaults()),
            timeout: Arc::new(TimeoutController::with_defaults()),
            plugins: Arc::new(PluginManager::with_defaults()),
            stats,
            tokens,
            pool_service,
            reload_lock: Arc::new(RwLock::new(())),
        }
    }

    /// 解析模型别名
    ///
    /// 使用 ModelMapper 将模型别名解析为实际模型名称
    ///
    /// # Arguments
    /// * `model` - 原始模型名称（可能是别名）
    ///
    /// # Returns
    /// 解析后的实际模型名称
    pub async fn resolve_model(&self, model: &str) -> String {
        let mapper = self.mapper.read().await;
        mapper.resolve(model)
    }

    /// 解析模型别名并更新请求上下文
    ///
    /// # Arguments
    /// * `ctx` - 请求上下文
    ///
    /// # Returns
    /// 解析后的模型名称
    pub async fn resolve_model_for_context(&self, ctx: &mut RequestContext) -> String {
        let resolved = self.resolve_model(&ctx.original_model).await;
        ctx.set_resolved_model(resolved.clone());

        tracing::debug!(
            "[MAPPER] request_id={} original_model={} resolved_model={}",
            ctx.request_id,
            ctx.original_model,
            resolved
        );

        resolved
    }

    /// 根据模型选择 Provider
    ///
    /// 使用 Router 根据路由规则选择合适的 Provider
    ///
    /// # Arguments
    /// * `model` - 模型名称（应该是解析后的实际模型名）
    ///
    /// # Returns
    /// 选择的 Provider 类型（如果设置了）和是否使用默认 Provider
    pub async fn route_model(&self, model: &str) -> (Option<crate::ProviderType>, bool) {
        let router = self.router.read().await;
        let result = router.route(model);
        (result.provider, result.is_default)
    }

    /// 根据模型选择 Provider 并更新请求上下文
    ///
    /// # Arguments
    /// * `ctx` - 请求上下文
    ///
    /// # Returns
    /// 选择的 Provider 类型，如果未设置默认 Provider 则返回 None
    pub async fn route_for_context(&self, ctx: &mut RequestContext) -> Option<crate::ProviderType> {
        let (provider, is_default) = self.route_model(&ctx.resolved_model).await;

        if let Some(p) = provider {
            ctx.set_provider(p);
            tracing::info!(
                "[ROUTE] request_id={} model={} provider={} is_default={}",
                ctx.request_id,
                ctx.resolved_model,
                p,
                is_default
            );
        } else {
            tracing::warn!(
                "[ROUTE] request_id={} model={} 未设置默认 Provider",
                ctx.request_id,
                ctx.resolved_model
            );
        }

        provider
    }

    /// 执行完整的路由解析流程
    ///
    /// 包括模型别名解析和 Provider 选择
    ///
    /// # Arguments
    /// * `ctx` - 请求上下文
    ///
    /// # Returns
    /// 选择的 Provider 类型，如果未设置默认 Provider 则返回 None
    pub async fn resolve_and_route(&self, ctx: &mut RequestContext) -> Option<crate::ProviderType> {
        // 1. 解析模型别名
        self.resolve_model_for_context(ctx).await;

        // 2. 根据解析后的模型选择 Provider
        self.route_for_context(ctx).await
    }
}

#[cfg(test)]
mod tests;
