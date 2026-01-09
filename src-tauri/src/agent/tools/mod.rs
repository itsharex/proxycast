//! Agent 工具系统模块
//!
//! 提供工具定义、注册、执行的核心框架
//!
//! ## 模块结构
//! - `types`: 工具类型定义（ToolDefinition, ToolCall, ToolResult 等）
//! - `registry`: 工具注册表和 Tool trait
//! - `security`: 安全管理器（路径验证、符号链接检查等）
//! - `bash`: Bash 命令执行工具
//! - `terminal`: 终端命令执行工具（通过前端审批）
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
pub mod term_scrollback;
pub mod terminal;
pub mod types;
pub mod write_file;

pub use bash::{BashExecutionResult, BashTool, ShellType};
pub use edit_file::{EditFileResult, EditFileTool, UndoResult};
pub use prompt::{generate_tools_prompt, PromptFormat, ToolPromptGenerator};
pub use read_file::{ReadFileResult, ReadFileTool};
pub use registry::{Tool, ToolRegistry};
pub use security::{SecurityError, SecurityManager};
pub use term_scrollback::{
    get_term_scrollback_tool, handle_term_scrollback_response, set_term_scrollback_tool_app_handle,
    GetScrollbackRequest, GetScrollbackResponse, TermScrollbackTool,
};
pub use terminal::{
    get_terminal_tool, handle_terminal_command_response, set_terminal_tool_app_handle,
    TerminalCommandRequest, TerminalCommandResponse, TerminalTool,
};
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

/// 创建 Terminal AI 模式的工具注册表
///
/// 使用 TerminalTool 替代 BashTool，命令通过前端审批后在用户终端执行
/// 同时添加 TermScrollbackTool 用于只读访问终端输出
///
/// # Arguments
/// * `base_dir` - 基础目录，所有文件操作必须在此目录内
///
/// # Returns
/// 包含 terminal, term_get_scrollback, read_file, write_file, edit_file 工具的注册表
pub fn create_terminal_registry(base_dir: impl AsRef<Path>) -> ToolRegistry {
    let security = Arc::new(SecurityManager::new(base_dir.as_ref()));
    let registry = ToolRegistry::new();

    // 注册 TerminalTool（替代 BashTool）
    // 使用全局实例以便前端可以响应
    let terminal_tool = get_terminal_tool();

    // 验证 AppHandle 是否已设置
    if terminal_tool.is_app_handle_set() {
        tracing::info!("[Tools] TerminalTool AppHandle 已设置，可以正常使用");
    } else {
        tracing::error!("[Tools] 警告：TerminalTool AppHandle 未设置！工具将无法正常工作");
        eprintln!("[Tools] 警告：TerminalTool AppHandle 未设置！工具将无法正常工作");
    }

    if let Err(e) = registry.register_arc(terminal_tool) {
        tracing::error!("注册 TerminalTool 失败: {}", e);
    }

    // 注册 TermScrollbackTool（只读访问终端输出）
    // 使用全局实例以便前端可以响应
    let scrollback_tool = get_term_scrollback_tool();

    // 验证 AppHandle 是否已设置
    if scrollback_tool.is_app_handle_set() {
        tracing::info!("[Tools] TermScrollbackTool AppHandle 已设置，可以正常使用");
    } else {
        tracing::error!("[Tools] 警告：TermScrollbackTool AppHandle 未设置！工具将无法正常工作");
        eprintln!("[Tools] 警告：TermScrollbackTool AppHandle 未设置！工具将无法正常工作");
    }

    if let Err(e) = registry.register_arc(scrollback_tool) {
        tracing::error!("注册 TermScrollbackTool 失败: {}", e);
    }

    // 注册文件操作工具
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
        "[Tools] 已创建 Terminal AI 工具注册表，共 {} 个工具: {:?}",
        registry.len(),
        registry.list_names()
    );

    registry
}
