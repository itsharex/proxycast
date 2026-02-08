//! WebSocket 核心类型定义
//!
//! 包含 WebSocket 连接、消息、配置和统计类型。
//! 完整的 WebSocket 处理逻辑（handler、lifecycle、stream）保留在主 crate 中。

pub mod types;

pub use types::*;
