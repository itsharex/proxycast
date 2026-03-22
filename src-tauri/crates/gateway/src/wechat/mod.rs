//! WeChat Gateway 运行时
//!
//! 目录隔离版微信渠道实现：
//! - 登录与账号管理
//! - 长轮询消息接收
//! - 媒体下载/上传与解密
//! - 统一接入 Gateway RPC / Agent Runtime

mod api;
mod auth;
mod media;
mod runtime;
mod types;

pub use auth::{
    start_login, wait_login, WechatLoginStartResult, WechatLoginState, WechatLoginWaitResult,
};
pub use media::{purge_account_data, resolve_channel_data_dir};
pub use runtime::{
    probe_gateway_account, start_gateway, status_gateway, stop_gateway, WechatGatewayAccountStatus,
    WechatGatewayState, WechatGatewayStatus, WechatProbeResult,
};
pub use types::{DEFAULT_BASE_URL, DEFAULT_CDN_BASE_URL, DEFAULT_ILINK_BOT_TYPE};
