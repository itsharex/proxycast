//! 终端模块单元测试
//!
//! 测试终端核心能力的各个组件。
//!
//! ## 测试覆盖
//! - 错误类型序列化
//! - 会话状态转换
//! - 事件结构序列化

#[cfg(test)]
mod tests {
    use super::super::error::TerminalError;
    use super::super::events::{SessionStatus, TerminalOutputEvent, TerminalStatusEvent};

    // ========================================================================
    // 错误类型测试
    // ========================================================================

    #[test]
    fn test_terminal_error_session_not_found() {
        let err = TerminalError::SessionNotFound("test-session-123".to_string());
        assert_eq!(err.to_string(), "会话不存在: test-session-123");
    }

    #[test]
    fn test_terminal_error_pty_creation_failed() {
        let err = TerminalError::PtyCreationFailed("spawn failed".to_string());
        assert_eq!(err.to_string(), "PTY 创建失败: spawn failed");
    }

    #[test]
    fn test_terminal_error_write_failed() {
        let err = TerminalError::WriteFailed("broken pipe".to_string());
        assert_eq!(err.to_string(), "写入失败: broken pipe");
    }

    #[test]
    fn test_terminal_error_resize_failed() {
        let err = TerminalError::ResizeFailed("invalid size".to_string());
        assert_eq!(err.to_string(), "调整大小失败: invalid size");
    }

    #[test]
    fn test_terminal_error_session_closed() {
        let err = TerminalError::SessionClosed;
        assert_eq!(err.to_string(), "会话已关闭");
    }

    #[test]
    fn test_terminal_error_base64_decode_failed() {
        let err = TerminalError::Base64DecodeFailed("invalid base64".to_string());
        assert_eq!(err.to_string(), "Base64 解码失败: invalid base64");
    }

    #[test]
    fn test_terminal_error_internal() {
        let err = TerminalError::Internal("unexpected error".to_string());
        assert_eq!(err.to_string(), "内部错误: unexpected error");
    }

    #[test]
    fn test_terminal_error_to_string_conversion() {
        let err = TerminalError::SessionNotFound("abc".to_string());
        let s: String = err.into();
        assert_eq!(s, "会话不存在: abc");
    }

    #[test]
    fn test_terminal_error_serialize() {
        let err = TerminalError::SessionNotFound("test".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, "\"会话不存在: test\"");
    }

    // ========================================================================
    // 会话状态测试
    // ========================================================================

    #[test]
    fn test_session_status_default() {
        let status = SessionStatus::default();
        assert_eq!(status, SessionStatus::Connecting);
    }

    #[test]
    fn test_session_status_serialize() {
        assert_eq!(
            serde_json::to_string(&SessionStatus::Connecting).unwrap(),
            "\"connecting\""
        );
        assert_eq!(
            serde_json::to_string(&SessionStatus::Running).unwrap(),
            "\"running\""
        );
        assert_eq!(
            serde_json::to_string(&SessionStatus::Done).unwrap(),
            "\"done\""
        );
        assert_eq!(
            serde_json::to_string(&SessionStatus::Error).unwrap(),
            "\"error\""
        );
    }

    #[test]
    fn test_session_status_deserialize() {
        assert_eq!(
            serde_json::from_str::<SessionStatus>("\"connecting\"").unwrap(),
            SessionStatus::Connecting
        );
        assert_eq!(
            serde_json::from_str::<SessionStatus>("\"running\"").unwrap(),
            SessionStatus::Running
        );
        assert_eq!(
            serde_json::from_str::<SessionStatus>("\"done\"").unwrap(),
            SessionStatus::Done
        );
        assert_eq!(
            serde_json::from_str::<SessionStatus>("\"error\"").unwrap(),
            SessionStatus::Error
        );
    }

    #[test]
    fn test_session_status_equality() {
        assert_eq!(SessionStatus::Running, SessionStatus::Running);
        assert_ne!(SessionStatus::Running, SessionStatus::Done);
    }

    #[test]
    fn test_session_status_clone() {
        let status = SessionStatus::Running;
        let cloned = status;
        assert_eq!(status, cloned);
    }

    // ========================================================================
    // 事件结构测试
    // ========================================================================

    #[test]
    fn test_terminal_output_event_serialize() {
        let event = TerminalOutputEvent {
            session_id: "session-123".to_string(),
            data: "SGVsbG8gV29ybGQ=".to_string(), // "Hello World" in Base64
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"session_id\":\"session-123\""));
        assert!(json.contains("\"data\":\"SGVsbG8gV29ybGQ=\""));
    }

    #[test]
    fn test_terminal_output_event_deserialize() {
        let json = r#"{"session_id":"abc","data":"dGVzdA=="}"#;
        let event: TerminalOutputEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.session_id, "abc");
        assert_eq!(event.data, "dGVzdA==");
    }

    #[test]
    fn test_terminal_status_event_serialize_done() {
        let event = TerminalStatusEvent {
            session_id: "session-456".to_string(),
            status: SessionStatus::Done,
            exit_code: Some(0),
            error: None,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"status\":\"done\""));
        assert!(json.contains("\"exit_code\":0"));
    }

    #[test]
    fn test_terminal_status_event_serialize_error() {
        let event = TerminalStatusEvent {
            session_id: "session-789".to_string(),
            status: SessionStatus::Error,
            exit_code: None,
            error: Some("Connection refused".to_string()),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"status\":\"error\""));
        assert!(json.contains("\"error\":\"Connection refused\""));
    }

    #[test]
    fn test_terminal_status_event_deserialize() {
        let json = r#"{"session_id":"test","status":"running","exit_code":null,"error":null}"#;
        let event: TerminalStatusEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.session_id, "test");
        assert_eq!(event.status, SessionStatus::Running);
        assert!(event.exit_code.is_none());
        assert!(event.error.is_none());
    }

    // ========================================================================
    // 事件名称常量测试
    // ========================================================================

    #[test]
    fn test_event_names() {
        use super::super::events::event_names;
        assert_eq!(event_names::TERMINAL_OUTPUT, "terminal:output");
        assert_eq!(event_names::TERMINAL_STATUS, "terminal:status");
    }
}
