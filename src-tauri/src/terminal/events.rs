//! 终端事件定义
//!
//! 定义终端模块使用的 Tauri 事件类型。
//!
//! ## 事件列表
//! - `terminal:output` - 终端输出数据
//! - `terminal:status` - 终端状态变化

use serde::{Deserialize, Serialize};

/// 会话状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    /// 正在连接
    Connecting,
    /// 运行中
    Running,
    /// 已结束
    Done,
    /// 错误
    Error,
}

impl Default for SessionStatus {
    fn default() -> Self {
        Self::Connecting
    }
}

/// 终端输出事件
///
/// Event name: `terminal:output`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalOutputEvent {
    /// 会话 ID
    pub session_id: String,
    /// 输出数据（Base64 编码）
    pub data: String,
}

/// 终端状态事件
///
/// Event name: `terminal:status`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalStatusEvent {
    /// 会话 ID
    pub session_id: String,
    /// 会话状态
    pub status: SessionStatus,
    /// 退出码（仅当状态为 Done 时有效）
    pub exit_code: Option<i32>,
    /// 错误信息（仅当状态为 Error 时有效）
    pub error: Option<String>,
}

/// 事件名称常量
pub mod event_names {
    /// 终端输出事件名
    pub const TERMINAL_OUTPUT: &str = "terminal:output";
    /// 终端状态事件名
    pub const TERMINAL_STATUS: &str = "terminal:status";
}
