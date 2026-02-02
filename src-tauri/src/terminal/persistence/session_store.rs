//! 会话元数据存储
//!
//! 使用 SQLite 存储终端会话的元数据信息。
//!
//! ## 功能
//! - 会话元数据的 CRUD 操作
//! - 会话状态查询
//! - 会话恢复支持
//!
//! _Requirements: 3.5, 3.9_

use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::database::DbConnection;
use crate::terminal::error::TerminalError;

/// 会话记录（存储在 SQLite）
///
/// _Requirements: 3.5_
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    /// 会话 ID
    pub id: String,
    /// 块 ID（用于关联 BlockFile）
    pub block_id: String,
    /// 标签页 ID
    pub tab_id: String,
    /// 控制器类型（shell/cmd）
    pub controller_type: String,
    /// 连接名称（本地/SSH/WSL）
    pub connection: Option<String>,
    /// 会话状态（running/done/error）
    pub status: String,
    /// 创建时间（Unix 时间戳，毫秒）
    pub created_at: i64,
    /// 更新时间（Unix 时间戳，毫秒）
    pub updated_at: i64,
    /// 退出码
    pub exit_code: Option<i32>,
}

impl SessionRecord {
    /// 创建新的会话记录
    pub fn new(
        id: String,
        block_id: String,
        tab_id: String,
        controller_type: String,
        connection: Option<String>,
    ) -> Self {
        let now = Utc::now().timestamp_millis();
        Self {
            id,
            block_id,
            tab_id,
            controller_type,
            connection,
            status: "running".to_string(),
            created_at: now,
            updated_at: now,
            exit_code: None,
        }
    }
}

/// 会话元数据存储服务
///
/// 提供会话元数据的 SQLite 存储和查询功能。
///
/// _Requirements: 3.5, 3.9_
pub struct SessionMetadataStore {
    db: DbConnection,
}

impl SessionMetadataStore {
    /// 创建新的会话存储服务
    pub fn new(db: DbConnection) -> Self {
        Self { db }
    }

    /// 初始化数据库表
    ///
    /// 创建 terminal_sessions 表（如果不存在）。
    pub fn init_tables(&self) -> Result<(), TerminalError> {
        let conn = self
            .db
            .lock()
            .map_err(|e| TerminalError::DatabaseError(format!("无法获取数据库锁: {e}")))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS terminal_sessions (
                id TEXT PRIMARY KEY,
                block_id TEXT NOT NULL,
                tab_id TEXT NOT NULL,
                controller_type TEXT NOT NULL,
                connection TEXT,
                status TEXT NOT NULL DEFAULT 'running',
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                exit_code INTEGER
            )",
            [],
        )
        .map_err(|e| TerminalError::DatabaseError(format!("创建表失败: {e}")))?;

        // 创建索引
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_terminal_sessions_block_id ON terminal_sessions(block_id)",
            [],
        )
        .map_err(|e| TerminalError::DatabaseError(format!("创建索引失败: {e}")))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_terminal_sessions_tab_id ON terminal_sessions(tab_id)",
            [],
        )
        .map_err(|e| TerminalError::DatabaseError(format!("创建索引失败: {e}")))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_terminal_sessions_status ON terminal_sessions(status)",
            [],
        )
        .map_err(|e| TerminalError::DatabaseError(format!("创建索引失败: {e}")))?;

        tracing::debug!("[SessionStore] 数据库表初始化完成");
        Ok(())
    }

    /// 保存会话记录
    ///
    /// 如果记录已存在则更新，否则插入新记录。
    ///
    /// _Requirements: 3.5_
    pub fn save(&self, record: &SessionRecord) -> Result<(), TerminalError> {
        let conn = self
            .db
            .lock()
            .map_err(|e| TerminalError::DatabaseError(format!("无法获取数据库锁: {e}")))?;

        conn.execute(
            "INSERT OR REPLACE INTO terminal_sessions 
             (id, block_id, tab_id, controller_type, connection, status, created_at, updated_at, exit_code)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                record.id,
                record.block_id,
                record.tab_id,
                record.controller_type,
                record.connection,
                record.status,
                record.created_at,
                record.updated_at,
                record.exit_code,
            ],
        )
        .map_err(|e| TerminalError::DatabaseError(format!("保存会话失败: {e}")))?;

        tracing::debug!("[SessionStore] 保存会话: {}", record.id);
        Ok(())
    }

    /// 根据 ID 获取会话记录
    pub fn get_by_id(&self, id: &str) -> Result<Option<SessionRecord>, TerminalError> {
        let conn = self
            .db
            .lock()
            .map_err(|e| TerminalError::DatabaseError(format!("无法获取数据库锁: {e}")))?;

        let result = conn
            .query_row(
                "SELECT id, block_id, tab_id, controller_type, connection, status, created_at, updated_at, exit_code
                 FROM terminal_sessions WHERE id = ?1",
                params![id],
                |row| {
                    Ok(SessionRecord {
                        id: row.get(0)?,
                        block_id: row.get(1)?,
                        tab_id: row.get(2)?,
                        controller_type: row.get(3)?,
                        connection: row.get(4)?,
                        status: row.get(5)?,
                        created_at: row.get(6)?,
                        updated_at: row.get(7)?,
                        exit_code: row.get(8)?,
                    })
                },
            )
            .optional()
            .map_err(|e| TerminalError::DatabaseError(format!("查询会话失败: {e}")))?;

        Ok(result)
    }

    /// 根据块 ID 获取会话记录
    pub fn get_by_block_id(&self, block_id: &str) -> Result<Option<SessionRecord>, TerminalError> {
        let conn = self
            .db
            .lock()
            .map_err(|e| TerminalError::DatabaseError(format!("无法获取数据库锁: {e}")))?;

        let result = conn
            .query_row(
                "SELECT id, block_id, tab_id, controller_type, connection, status, created_at, updated_at, exit_code
                 FROM terminal_sessions WHERE block_id = ?1",
                params![block_id],
                |row| {
                    Ok(SessionRecord {
                        id: row.get(0)?,
                        block_id: row.get(1)?,
                        tab_id: row.get(2)?,
                        controller_type: row.get(3)?,
                        connection: row.get(4)?,
                        status: row.get(5)?,
                        created_at: row.get(6)?,
                        updated_at: row.get(7)?,
                        exit_code: row.get(8)?,
                    })
                },
            )
            .optional()
            .map_err(|e| TerminalError::DatabaseError(format!("查询会话失败: {e}")))?;

        Ok(result)
    }

    /// 获取所有会话记录
    pub fn get_all(&self) -> Result<Vec<SessionRecord>, TerminalError> {
        let conn = self
            .db
            .lock()
            .map_err(|e| TerminalError::DatabaseError(format!("无法获取数据库锁: {e}")))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, block_id, tab_id, controller_type, connection, status, created_at, updated_at, exit_code
                 FROM terminal_sessions ORDER BY created_at DESC",
            )
            .map_err(|e| TerminalError::DatabaseError(format!("准备查询失败: {e}")))?;

        let records = stmt
            .query_map([], |row| {
                Ok(SessionRecord {
                    id: row.get(0)?,
                    block_id: row.get(1)?,
                    tab_id: row.get(2)?,
                    controller_type: row.get(3)?,
                    connection: row.get(4)?,
                    status: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                    exit_code: row.get(8)?,
                })
            })
            .map_err(|e| TerminalError::DatabaseError(format!("查询会话失败: {e}")))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| TerminalError::DatabaseError(format!("读取会话失败: {e}")))?;

        Ok(records)
    }

    /// 获取指定状态的会话记录
    pub fn get_by_status(&self, status: &str) -> Result<Vec<SessionRecord>, TerminalError> {
        let conn = self
            .db
            .lock()
            .map_err(|e| TerminalError::DatabaseError(format!("无法获取数据库锁: {e}")))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, block_id, tab_id, controller_type, connection, status, created_at, updated_at, exit_code
                 FROM terminal_sessions WHERE status = ?1 ORDER BY created_at DESC",
            )
            .map_err(|e| TerminalError::DatabaseError(format!("准备查询失败: {e}")))?;

        let records = stmt
            .query_map(params![status], |row| {
                Ok(SessionRecord {
                    id: row.get(0)?,
                    block_id: row.get(1)?,
                    tab_id: row.get(2)?,
                    controller_type: row.get(3)?,
                    connection: row.get(4)?,
                    status: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                    exit_code: row.get(8)?,
                })
            })
            .map_err(|e| TerminalError::DatabaseError(format!("查询会话失败: {e}")))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| TerminalError::DatabaseError(format!("读取会话失败: {e}")))?;

        Ok(records)
    }

    /// 获取指定标签页的会话记录
    pub fn get_by_tab_id(&self, tab_id: &str) -> Result<Vec<SessionRecord>, TerminalError> {
        let conn = self
            .db
            .lock()
            .map_err(|e| TerminalError::DatabaseError(format!("无法获取数据库锁: {e}")))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, block_id, tab_id, controller_type, connection, status, created_at, updated_at, exit_code
                 FROM terminal_sessions WHERE tab_id = ?1 ORDER BY created_at DESC",
            )
            .map_err(|e| TerminalError::DatabaseError(format!("准备查询失败: {e}")))?;

        let records = stmt
            .query_map(params![tab_id], |row| {
                Ok(SessionRecord {
                    id: row.get(0)?,
                    block_id: row.get(1)?,
                    tab_id: row.get(2)?,
                    controller_type: row.get(3)?,
                    connection: row.get(4)?,
                    status: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                    exit_code: row.get(8)?,
                })
            })
            .map_err(|e| TerminalError::DatabaseError(format!("查询会话失败: {e}")))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| TerminalError::DatabaseError(format!("读取会话失败: {e}")))?;

        Ok(records)
    }

    /// 更新会话状态
    ///
    /// _Requirements: 3.9_
    pub fn update_status(
        &self,
        id: &str,
        status: &str,
        exit_code: Option<i32>,
    ) -> Result<(), TerminalError> {
        let conn = self
            .db
            .lock()
            .map_err(|e| TerminalError::DatabaseError(format!("无法获取数据库锁: {e}")))?;

        let now = Utc::now().timestamp_millis();

        conn.execute(
            "UPDATE terminal_sessions SET status = ?1, exit_code = ?2, updated_at = ?3 WHERE id = ?4",
            params![status, exit_code, now, id],
        )
        .map_err(|e| TerminalError::DatabaseError(format!("更新会话状态失败: {e}")))?;

        tracing::debug!("[SessionStore] 更新会话状态: {} -> {}", id, status);
        Ok(())
    }

    /// 删除会话记录
    pub fn delete(&self, id: &str) -> Result<(), TerminalError> {
        let conn = self
            .db
            .lock()
            .map_err(|e| TerminalError::DatabaseError(format!("无法获取数据库锁: {e}")))?;

        conn.execute("DELETE FROM terminal_sessions WHERE id = ?1", params![id])
            .map_err(|e| TerminalError::DatabaseError(format!("删除会话失败: {e}")))?;

        tracing::debug!("[SessionStore] 删除会话: {}", id);
        Ok(())
    }

    /// 删除指定标签页的所有会话记录
    pub fn delete_by_tab_id(&self, tab_id: &str) -> Result<usize, TerminalError> {
        let conn = self
            .db
            .lock()
            .map_err(|e| TerminalError::DatabaseError(format!("无法获取数据库锁: {e}")))?;

        let count = conn
            .execute(
                "DELETE FROM terminal_sessions WHERE tab_id = ?1",
                params![tab_id],
            )
            .map_err(|e| TerminalError::DatabaseError(format!("删除会话失败: {e}")))?;

        tracing::debug!("[SessionStore] 删除标签页 {} 的 {} 个会话", tab_id, count);
        Ok(count)
    }

    /// 清理已完成的旧会话
    ///
    /// 删除状态为 "done" 且创建时间早于指定时间的会话。
    pub fn cleanup_old_sessions(&self, before_timestamp: i64) -> Result<usize, TerminalError> {
        let conn = self
            .db
            .lock()
            .map_err(|e| TerminalError::DatabaseError(format!("无法获取数据库锁: {e}")))?;

        let count = conn
            .execute(
                "DELETE FROM terminal_sessions WHERE status = 'done' AND created_at < ?1",
                params![before_timestamp],
            )
            .map_err(|e| TerminalError::DatabaseError(format!("清理会话失败: {e}")))?;

        if count > 0 {
            tracing::info!("[SessionStore] 清理了 {} 个旧会话", count);
        }
        Ok(count)
    }

    /// 获取会话数量
    pub fn count(&self) -> Result<usize, TerminalError> {
        let conn = self
            .db
            .lock()
            .map_err(|e| TerminalError::DatabaseError(format!("无法获取数据库锁: {e}")))?;

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM terminal_sessions", [], |row| {
                row.get(0)
            })
            .map_err(|e| TerminalError::DatabaseError(format!("查询会话数量失败: {e}")))?;

        Ok(count as usize)
    }
}
