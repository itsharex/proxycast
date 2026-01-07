//! Agent 工具系统模块
//!
//! 提供工具定义、注册、执行的核心框架
//!
//! ## 模块结构
//! - `types`: 工具类型定义（ToolDefinition, ToolCall, ToolResult 等）
//! - `registry`: 工具注册表和 Tool trait
//! - `security`: 安全管理器（路径验证、符号链接检查等）
//! - `bash`: Bash 命令执行工具
//! - `read_file`: 文件读取工具
//! - `write_file`: 文件写入工具
//! - `edit_file`: 文件编辑工具
//! - `prompt`: 工具 Prompt 生成器（System Prompt 工具注入）

pub mod bash;
pub mod edit_file;
pub mod prompt;
pub mod read_file;
pub mod registry;
pub mod security;
pub mod types;
pub mod write_file;

pub use bash::{BashExecutionResult, BashTool, ShellType};
pub use edit_file::{EditFileResult, EditFileTool, UndoResult};
pub use prompt::{generate_tools_prompt, PromptFormat, ToolPromptGenerator};
pub use read_file::{ReadFileResult, ReadFileTool};
pub use registry::{Tool, ToolRegistry};
pub use security::{SecurityError, SecurityManager};
pub use types::*;
pub use write_file::{WriteFileResult, WriteFileTool};

use std::path::Path;
use std::sync::Arc;
use tracing::info;

/// 创建包含所有默认工具的注册表
///
/// # Arguments
/// * `base_dir` - 基础目录，所有文件操作必须在此目录内
///
/// # Returns
/// 包含 bash, read_file, write_file, edit_file 工具的注册表
pub fn create_default_registry(base_dir: impl AsRef<Path>) -> ToolRegistry {
    let security = Arc::new(SecurityManager::new(base_dir.as_ref()));
    let registry = ToolRegistry::new();

    // 注册核心工具
    if let Err(e) = registry.register(BashTool::new(Arc::clone(&security))) {
        tracing::error!("注册 BashTool 失败: {}", e);
    }

    if let Err(e) = registry.register(ReadFileTool::new(Arc::clone(&security))) {
        tracing::error!("注册 ReadFileTool 失败: {}", e);
    }

    if let Err(e) = registry.register(WriteFileTool::new(Arc::clone(&security))) {
        tracing::error!("注册 WriteFileTool 失败: {}", e);
    }

    if let Err(e) = registry.register(EditFileTool::new(Arc::clone(&security))) {
        tracing::error!("注册 EditFileTool 失败: {}", e);
    }

    info!(
        "[Tools] 已创建默认工具注册表，共 {} 个工具: {:?}",
        registry.len(),
        registry.list_names()
    );

    registry
}
