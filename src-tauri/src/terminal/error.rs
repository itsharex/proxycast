//! 终端模块错误类型
//!
//! 定义终端核心能力相关的错误类型。
//!
//! ## 功能
//! - 会话管理错误
//! - PTY 操作错误
//! - 序列化支持

use thiserror::Error;

/// 终端错误类型
#[derive(Debug, Error)]
pub enum TerminalError {
    /// 会话不存在
    #[error("会话不存在: {0}")]
    SessionNotFound(String),

    /// PTY 创建失败
    #[error("PTY 创建失败: {0}")]
    PtyCreationFailed(String),

    /// 写入失败
    #[error("写入失败: {0}")]
    WriteFailed(String),

    /// 调整大小失败
    #[error("调整大小失败: {0}")]
    ResizeFailed(String),

    /// 会话已关闭
    #[error("会话已关闭")]
    SessionClosed,

    /// Base64 解码失败
    #[error("Base64 解码失败: {0}")]
    Base64DecodeFailed(String),

    /// 内部错误
    #[error("内部错误: {0}")]
    Internal(String),
}

impl From<TerminalError> for String {
    fn from(err: TerminalError) -> Self {
        err.to_string()
    }
}

impl serde::Serialize for TerminalError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
