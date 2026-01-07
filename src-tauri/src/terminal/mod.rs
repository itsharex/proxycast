//! 终端核心模块
//!
//! 提供 PTY 管理和会话管理能力，通过 Tauri Commands 和 Events 暴露给前端。
//!
//! ## 模块结构
//! - `error` - 错误类型定义
//! - `events` - Tauri 事件定义
//! - `pty_session` - PTY 会话封装
//! - `session_manager` - 会话管理器
//!
//! ## 使用示例
//! ```ignore
//! use proxycast_lib::terminal::{TerminalSessionManager, SessionStatus};
//!
//! let manager = TerminalSessionManager::new(app_handle);
//! let session_id = manager.create_session(24, 80).await?;
//! manager.write_to_session(&session_id, b"ls -la\n").await?;
//! ```

pub mod error;
pub mod events;
pub mod pty_session;
pub mod session_manager;

#[cfg(test)]
mod tests;

// 重新导出常用类型
pub use error::TerminalError;
pub use events::{SessionStatus, TerminalOutputEvent, TerminalStatusEvent};
pub use pty_session::{PtySession, DEFAULT_COLS, DEFAULT_ROWS};
pub use session_manager::{SessionMetadata, TerminalSessionManager};
