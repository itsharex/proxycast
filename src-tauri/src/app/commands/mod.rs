//! 内置 Tauri 命令模块
//!
//! 包含 lib.rs 中定义的所有 Tauri 命令，按功能分类。
//!
//! ## 模块结构
//! - `server` - 服务器控制命令
//! - `config` - 配置管理命令
//! - `kiro` - Kiro Provider 命令 (legacy)
//! - `gemini` - Gemini Provider 命令 (legacy)
//! - `qwen` - Qwen Provider 命令 (legacy)
//! - `custom_providers` - 自定义 Provider 命令 (OpenAI/Claude Custom)
//! - `logs` - 日志命令
//! - `api_test` - API 测试和兼容性检查命令

mod api_test;
mod config;
mod custom_providers;
mod gemini;
mod kiro;
mod logs;
mod qwen;
mod server;

// 重新导出所有命令
pub use api_test::*;
pub use config::*;
pub use custom_providers::*;
pub use gemini::*;
pub use kiro::*;
pub use logs::*;
pub use qwen::*;
pub use server::*;
