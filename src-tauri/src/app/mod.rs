//! 应用核心模块
//!
//! 包含 Tauri 应用的核心类型、状态管理和启动逻辑。
//!
//! ## 模块结构
//! - `types` - 核心类型定义（ProviderType 等）
//! - `state` - 状态类型和初始化
//! - `setup` - Tauri setup hook
//! - `commands` - 内置 Tauri 命令
//! - `utils` - 辅助函数
//! - `bootstrap` - 应用启动引导（配置验证、状态初始化）
//! - `runner` - 应用运行器（Tauri Builder 配置和命令注册）

pub mod bootstrap;
pub mod commands;
pub mod runner;
mod setup;
mod state;
mod types;
mod utils;

pub use runner::run;
pub use setup::setup_app;
pub use state::*;
pub use types::*;
pub use utils::*;
