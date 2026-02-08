//! MCP（Model Context Protocol）模块
//!
//! 业务逻辑已迁移到 proxycast-mcp crate，
//! 本模块仅作为桥接层 re-export。

// 从 proxycast-mcp crate re-export 所有公开类型
pub use proxycast_mcp::client;
pub use proxycast_mcp::manager;
pub use proxycast_mcp::tool_converter;
pub use proxycast_mcp::types;

pub use proxycast_mcp::{McpClientManager, ProxyCastMcpClient};
pub use proxycast_mcp::{
    McpClientWrapper, McpContent, McpError, McpManagerState, McpPromptArgument,
    McpPromptDefinition, McpPromptMessage, McpPromptResult, McpResourceContent,
    McpResourceDefinition, McpServerCapabilities, McpServerConfig, McpServerErrorPayload,
    McpServerInfo, McpServerStartedPayload, McpServerStoppedPayload, McpToolCall,
    McpToolDefinition, McpToolResult, McpToolsUpdatedPayload, ToolConverter,
};
