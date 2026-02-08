//! 请求处理器模块（重导出层）
//!
//! 核心逻辑已迁移到 `proxycast-processor` crate。
//! 本模块保留向后兼容路径和本地测试入口。

pub use proxycast_processor::*;

#[cfg(test)]
mod tests;
