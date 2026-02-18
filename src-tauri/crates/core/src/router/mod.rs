//! 路由系统模块
//!
//! 支持动态路由注册、命名空间路由解析、模型映射。
//!
//! 路由格式：
//! - `/{provider-name}/v1/messages` - Provider 命名空间路由
//! - `/{selector}/v1/messages` - 凭证选择器路由（向后兼容）
//! - `/v1/messages` - 默认路由
//! - `/api/provider/{provider}/v1/*` - Amp CLI 路由
//!
//! 模型映射：
//! - 支持模型别名映射（如 `gpt-4` -> `claude-sonnet-4-5-20250514`）
//!
//! 提示路由：
//! - 支持消息前缀提示路由（如 `[reasoning] 请分析...`）

mod amp_router;
mod hint_router;
mod mapper;
mod provider_router;
mod route_registry;
mod rules;

pub use amp_router::AmpRouter;
pub use hint_router::{HintMatch, HintRoute, HintRouter, HintRouterConfig, HintRouteEntry};
pub use mapper::ModelMapper;
pub use rules::Router;
