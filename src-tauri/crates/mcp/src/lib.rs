//! ProxyCast MCP Crate
//!
//! MCP（Model Context Protocol）集成模块，提供 MCP 协议的客户端实现。
//! 使用 DynEmitter 替代 Tauri AppHandle 进行事件发射，实现与 Tauri 的解耦。

pub mod client;
pub mod manager;
pub mod tool_converter;
pub mod types;

pub use client::{McpClientWrapper, ProxyCastMcpClient};
pub use manager::McpClientManager;
pub use tool_converter::ToolConverter;
pub use types::{
    McpContent, McpError, McpManagerState, McpPromptArgument, McpPromptDefinition,
    McpPromptMessage, McpPromptResult, McpResourceContent, McpResourceDefinition,
    McpServerCapabilities, McpServerConfig, McpServerErrorPayload, McpServerInfo,
    McpServerStartedPayload, McpServerStoppedPayload, McpToolCall, McpToolDefinition,
    McpToolResult, McpToolsUpdatedPayload,
};
