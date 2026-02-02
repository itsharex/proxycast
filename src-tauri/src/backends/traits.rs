//! 后端调用层 Trait 定义
//!
//! 定义后端 HTTP 调用的核心接口。
//! 后端层只负责 HTTP 请求/响应，不包含任何协议转换逻辑。

use async_trait::async_trait;
use bytes::Bytes;
use futures::Stream;
use std::error::Error;
use std::pin::Pin;

/// 字节流类型
pub type ByteStream =
    Pin<Box<dyn Stream<Item = Result<Bytes, Box<dyn Error + Send + Sync>>> + Send>>;

/// 后端调用结果
pub type BackendResult<T> = Result<T, BackendError>;

/// 后端错误类型
#[derive(Debug, Clone)]
pub struct BackendError {
    /// 错误类型
    pub kind: BackendErrorKind,
    /// 错误消息
    pub message: String,
    /// HTTP 状态码（如果有）
    pub status_code: Option<u16>,
}

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(code) = self.status_code {
            write!(f, "{} ({}): {}", self.kind, code, self.message)
        } else {
            write!(f, "{}: {}", self.kind, self.message)
        }
    }
}

impl std::error::Error for BackendError {}

/// 后端错误类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendErrorKind {
    /// 认证错误
    AuthenticationError,
    /// 网络错误
    NetworkError,
    /// 请求超时
    Timeout,
    /// 服务端错误
    ServerError,
    /// 请求格式错误
    BadRequest,
    /// 速率限制
    RateLimited,
    /// 其他错误
    Other,
}

impl std::fmt::Display for BackendErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AuthenticationError => write!(f, "AuthenticationError"),
            Self::NetworkError => write!(f, "NetworkError"),
            Self::Timeout => write!(f, "Timeout"),
            Self::ServerError => write!(f, "ServerError"),
            Self::BadRequest => write!(f, "BadRequest"),
            Self::RateLimited => write!(f, "RateLimited"),
            Self::Other => write!(f, "Other"),
        }
    }
}

impl BackendError {
    /// 创建新的后端错误
    pub fn new(kind: BackendErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            status_code: None,
        }
    }

    /// 带 HTTP 状态码创建错误
    pub fn with_status(kind: BackendErrorKind, message: impl Into<String>, status: u16) -> Self {
        Self {
            kind,
            message: message.into(),
            status_code: Some(status),
        }
    }

    /// 从 HTTP 状态码推断错误类型
    pub fn from_status(status: u16, message: impl Into<String>) -> Self {
        let kind = match status {
            401 | 403 => BackendErrorKind::AuthenticationError,
            400 => BackendErrorKind::BadRequest,
            429 => BackendErrorKind::RateLimited,
            500..=599 => BackendErrorKind::ServerError,
            _ => BackendErrorKind::Other,
        };
        Self::with_status(kind, message, status)
    }

    /// 是否可重试
    pub fn is_retryable(&self) -> bool {
        matches!(
            self.kind,
            BackendErrorKind::NetworkError
                | BackendErrorKind::Timeout
                | BackendErrorKind::ServerError
                | BackendErrorKind::RateLimited
        )
    }
}

/// 后端 Trait
///
/// 定义后端 HTTP 调用的接口。
#[async_trait]
pub trait Backend: Send + Sync {
    /// 后端请求类型
    type Request: Send;

    /// 非流式调用
    ///
    /// # 返回
    ///
    /// 响应的原始字节
    async fn call(&self, request: &Self::Request) -> BackendResult<Bytes>;

    /// 流式调用
    ///
    /// # 返回
    ///
    /// 字节流
    async fn call_stream(&self, request: &Self::Request) -> BackendResult<ByteStream>;

    /// 获取后端名称
    fn name(&self) -> &str;

    /// 检查后端是否可用
    async fn is_available(&self) -> bool {
        true
    }
}

/// 带认证的后端 Trait
#[async_trait]
pub trait AuthenticatedBackend: Backend {
    /// 刷新认证凭证
    async fn refresh_credentials(&mut self) -> BackendResult<()>;

    /// 检查凭证是否有效
    fn credentials_valid(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_error_display() {
        let err = BackendError::new(BackendErrorKind::NetworkError, "connection refused");
        assert_eq!(format!("{err}"), "NetworkError: connection refused");

        let err = BackendError::with_status(BackendErrorKind::ServerError, "internal error", 500);
        assert_eq!(format!("{err}"), "ServerError (500): internal error");
    }

    #[test]
    fn test_backend_error_from_status() {
        let err = BackendError::from_status(401, "unauthorized");
        assert_eq!(err.kind, BackendErrorKind::AuthenticationError);
        assert_eq!(err.status_code, Some(401));

        let err = BackendError::from_status(429, "too many requests");
        assert_eq!(err.kind, BackendErrorKind::RateLimited);

        let err = BackendError::from_status(503, "service unavailable");
        assert_eq!(err.kind, BackendErrorKind::ServerError);
    }

    #[test]
    fn test_backend_error_retryable() {
        assert!(BackendError::new(BackendErrorKind::NetworkError, "").is_retryable());
        assert!(BackendError::new(BackendErrorKind::Timeout, "").is_retryable());
        assert!(BackendError::new(BackendErrorKind::ServerError, "").is_retryable());
        assert!(BackendError::new(BackendErrorKind::RateLimited, "").is_retryable());
        assert!(!BackendError::new(BackendErrorKind::AuthenticationError, "").is_retryable());
        assert!(!BackendError::new(BackendErrorKind::BadRequest, "").is_retryable());
    }
}
