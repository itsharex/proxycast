//! 终端会话管理器
//!
//! 管理所有终端会话的生命周期，提供会话的创建、查询、销毁功能。
//!
//! ## 功能
//! - 维护活跃会话的 HashMap
//! - 生成唯一的会话 ID
//! - 提供线程安全的会话访问

use std::collections::HashMap;
use std::sync::Arc;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::error::TerminalError;
use super::events::SessionStatus;
use super::pty_session::{PtySession, DEFAULT_COLS, DEFAULT_ROWS};

/// 会话元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// 会话 ID
    pub id: String,
    /// 会话状态
    pub status: SessionStatus,
    /// 创建时间（Unix 时间戳，毫秒）
    pub created_at: i64,
    /// 终端行数
    pub rows: u16,
    /// 终端列数
    pub cols: u16,
}

/// 内部会话数据
struct SessionData {
    session: PtySession,
    metadata: SessionMetadata,
}

/// 终端会话管理器
pub struct TerminalSessionManager {
    /// 会话映射表
    sessions: Arc<RwLock<HashMap<String, SessionData>>>,
    /// Tauri 应用句柄
    app_handle: tauri::AppHandle,
}

impl TerminalSessionManager {
    /// 创建新的会话管理器
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        tracing::info!("[终端] 会话管理器已初始化");
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            app_handle,
        }
    }

    /// 创建新的终端会话（使用默认大小）
    ///
    /// PTY 使用默认大小 (24x80) 预创建，前端连接后通过 resize 同步实际大小。
    ///
    /// # 返回
    /// - `Ok(String)`: 会话 ID
    /// - `Err(TerminalError)`: 创建失败
    pub async fn create_session(&self) -> Result<String, TerminalError> {
        let session_id = Uuid::new_v4().to_string();

        let session = PtySession::new(session_id.clone(), self.app_handle.clone())?;

        let metadata = SessionMetadata {
            id: session_id.clone(),
            status: SessionStatus::Running,
            created_at: Utc::now().timestamp_millis(),
            rows: DEFAULT_ROWS,
            cols: DEFAULT_COLS,
        };

        let data = SessionData { session, metadata };

        self.sessions.write().await.insert(session_id.clone(), data);

        tracing::info!(
            "[终端] 创建会话: {} (默认大小 {}x{})",
            session_id,
            DEFAULT_COLS,
            DEFAULT_ROWS
        );
        Ok(session_id)
    }

    /// 向会话发送输入
    ///
    /// # 参数
    /// - `session_id`: 会话 ID
    /// - `data`: 输入数据
    pub async fn write_to_session(
        &self,
        session_id: &str,
        data: &[u8],
    ) -> Result<(), TerminalError> {
        let sessions = self.sessions.read().await;
        let session_data = sessions
            .get(session_id)
            .ok_or_else(|| TerminalError::SessionNotFound(session_id.to_string()))?;

        session_data.session.write(data)
    }

    /// 向会话发送 Base64 编码的输入
    ///
    /// # 参数
    /// - `session_id`: 会话 ID
    /// - `data_base64`: Base64 编码的输入数据
    pub async fn write_to_session_base64(
        &self,
        session_id: &str,
        data_base64: &str,
    ) -> Result<(), TerminalError> {
        let data = BASE64
            .decode(data_base64)
            .map_err(|e| TerminalError::Base64DecodeFailed(e.to_string()))?;
        self.write_to_session(session_id, &data).await
    }

    /// 调整会话大小
    ///
    /// # 参数
    /// - `session_id`: 会话 ID
    /// - `rows`: 新的行数
    /// - `cols`: 新的列数
    pub async fn resize_session(
        &self,
        session_id: &str,
        rows: u16,
        cols: u16,
    ) -> Result<(), TerminalError> {
        let mut sessions = self.sessions.write().await;
        let session_data = sessions
            .get_mut(session_id)
            .ok_or_else(|| TerminalError::SessionNotFound(session_id.to_string()))?;

        session_data.session.resize(rows, cols)?;
        session_data.metadata.rows = rows;
        session_data.metadata.cols = cols;

        Ok(())
    }

    /// 关闭会话
    ///
    /// # 参数
    /// - `session_id`: 会话 ID
    pub async fn close_session(&self, session_id: &str) -> Result<(), TerminalError> {
        let mut sessions = self.sessions.write().await;
        let session_data = sessions
            .remove(session_id)
            .ok_or_else(|| TerminalError::SessionNotFound(session_id.to_string()))?;

        session_data.session.close().await?;

        tracing::info!("[终端] 关闭会话: {}", session_id);
        Ok(())
    }

    /// 获取会话的输出历史数据（Base64 编码）
    ///
    /// # 参数
    /// - `session_id`: 会话 ID
    ///
    /// # 返回
    /// - `Ok(String)`: Base64 编码的输出历史
    /// - `Err(TerminalError)`: 会话不存在
    pub async fn get_session_history(&self, session_id: &str) -> Result<String, TerminalError> {
        let sessions = self.sessions.read().await;
        let session_data = sessions
            .get(session_id)
            .ok_or_else(|| TerminalError::SessionNotFound(session_id.to_string()))?;

        Ok(session_data.session.get_output_history())
    }

    /// 获取所有会话列表
    pub async fn list_sessions(&self) -> Vec<SessionMetadata> {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .map(|data| data.metadata.clone())
            .collect()
    }

    /// 获取单个会话信息
    ///
    /// # 参数
    /// - `session_id`: 会话 ID
    pub async fn get_session(&self, session_id: &str) -> Option<SessionMetadata> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).map(|data| data.metadata.clone())
    }

    /// 获取活跃会话数量
    pub async fn session_count(&self) -> usize {
        self.sessions.read().await.len()
    }
}
