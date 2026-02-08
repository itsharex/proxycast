//! System Prompt 模块
//!
//! 为 Aster Agent 提供 System Prompt 配置
//! 参考 claude-code-open 的设计，提供模块化的提示词组件
//!
//! ## 模块结构
//! - templates - 提示词模板定义
//! - builder - 提示词构建器

pub mod builder;
pub mod templates;

pub use builder::SystemPromptBuilder;
pub use templates::*;
