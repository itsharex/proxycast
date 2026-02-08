//! ProxyCast Agent Crate
//!
//! 包含 Agent 模块中不依赖主 crate 内部模块的纯逻辑部分。
//! 深耦合部分（aster_state、aster_agent、credential_bridge、subagent_scheduler）
//! 留在主 crate。

pub mod event_converter;
pub mod mcp_bridge;
pub mod prompt;

pub use event_converter::{convert_agent_event, convert_to_tauri_message, TauriAgentEvent};
pub use prompt::SystemPromptBuilder;
