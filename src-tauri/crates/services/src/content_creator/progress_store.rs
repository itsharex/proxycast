//! 进度持久化存储
//!
//! 将工作流进度保存到 SQLite 数据库

use super::types::*;
use anyhow::Result;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

/// 进度存储服务
pub struct ProgressStore {
    conn: Arc<Mutex<Connection>>,
}

impl ProgressStore {
    /// 创建新的进度存储
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // 创建表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS workflow_progress (
                workflow_id TEXT PRIMARY KEY,
                content_id TEXT NOT NULL,
                theme TEXT NOT NULL,
                mode TEXT NOT NULL,
                steps_json TEXT NOT NULL,
                current_step_index INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        // 创建索引
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_workflow_updated_at ON workflow_progress(updated_at DESC)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_workflow_content_id ON workflow_progress(content_id)",
            [],
        )?;

        info!("进度存储初始化完成");

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// 保存工作流进度
    pub async fn save_progress(&self, workflow: &WorkflowState) -> Result<()> {
        let conn = self.conn.lock().await;

        let steps_json = serde_json::to_string(&workflow.steps)?;
        let theme_str = serde_json::to_string(&workflow.theme)?;
        let mode_str = serde_json::to_string(&workflow.mode)?;

        conn.execute(
            "INSERT OR REPLACE INTO workflow_progress
             (workflow_id, content_id, theme, mode, steps_json, current_step_index, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                workflow.id,
                workflow.content_id,
                theme_str,
                mode_str,
                steps_json,
                workflow.current_step_index as i32,
                workflow.created_at,
                workflow.updated_at,
            ],
        )?;

        debug!("保存工作流进度: {}", workflow.id);
        Ok(())
    }

    /// 加载工作流进度
    pub async fn load_progress(&self, workflow_id: &str) -> Result<Option<WorkflowState>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn.prepare(
            "SELECT workflow_id, content_id, theme, mode, steps_json, current_step_index, created_at, updated_at
             FROM workflow_progress WHERE workflow_id = ?1",
        )?;

        let result = stmt.query_row(params![workflow_id], |row| {
            let workflow_id: String = row.get(0)?;
            let content_id: String = row.get(1)?;
            let theme_str: String = row.get(2)?;
            let mode_str: String = row.get(3)?;
            let steps_json: String = row.get(4)?;
            let current_step_index: i32 = row.get(5)?;
            let created_at: i64 = row.get(6)?;
            let updated_at: i64 = row.get(7)?;

            Ok(WorkflowProgress {
                workflow_id,
                content_id,
                theme: serde_json::from_str(&theme_str).unwrap_or_default(),
                mode: serde_json::from_str(&mode_str).unwrap_or_default(),
                steps_json,
                current_step_index,
                created_at,
                updated_at,
            })
        });

        match result {
            Ok(progress) => {
                let steps: Vec<WorkflowStep> = serde_json::from_str(&progress.steps_json)?;
                Ok(Some(WorkflowState {
                    id: progress.workflow_id,
                    content_id: progress.content_id,
                    theme: progress.theme,
                    mode: progress.mode,
                    steps,
                    current_step_index: progress.current_step_index as usize,
                    created_at: progress.created_at,
                    updated_at: progress.updated_at,
                }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// 根据 content_id 加载工作流进度
    pub async fn load_by_content_id(&self, content_id: &str) -> Result<Option<WorkflowState>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn.prepare(
            "SELECT workflow_id, content_id, theme, mode, steps_json, current_step_index, created_at, updated_at
             FROM workflow_progress WHERE content_id = ?1 ORDER BY updated_at DESC LIMIT 1",
        )?;

        let result = stmt.query_row(params![content_id], |row| {
            let workflow_id: String = row.get(0)?;
            let content_id: String = row.get(1)?;
            let theme_str: String = row.get(2)?;
            let mode_str: String = row.get(3)?;
            let steps_json: String = row.get(4)?;
            let current_step_index: i32 = row.get(5)?;
            let created_at: i64 = row.get(6)?;
            let updated_at: i64 = row.get(7)?;

            Ok(WorkflowProgress {
                workflow_id,
                content_id,
                theme: serde_json::from_str(&theme_str).unwrap_or_default(),
                mode: serde_json::from_str(&mode_str).unwrap_or_default(),
                steps_json,
                current_step_index,
                created_at,
                updated_at,
            })
        });

        match result {
            Ok(progress) => {
                let steps: Vec<WorkflowStep> = serde_json::from_str(&progress.steps_json)?;
                Ok(Some(WorkflowState {
                    id: progress.workflow_id,
                    content_id: progress.content_id,
                    theme: progress.theme,
                    mode: progress.mode,
                    steps,
                    current_step_index: progress.current_step_index as usize,
                    created_at: progress.created_at,
                    updated_at: progress.updated_at,
                }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// 删除工作流进度
    pub async fn delete_progress(&self, workflow_id: &str) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "DELETE FROM workflow_progress WHERE workflow_id = ?1",
            params![workflow_id],
        )?;
        debug!("删除工作流进度: {}", workflow_id);
        Ok(())
    }

    /// 获取最近的工作流列表
    pub async fn list_recent(&self, limit: usize) -> Result<Vec<WorkflowProgress>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn.prepare(
            "SELECT workflow_id, content_id, theme, mode, steps_json, current_step_index, created_at, updated_at
             FROM workflow_progress ORDER BY updated_at DESC LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit as i32], |row| {
            let workflow_id: String = row.get(0)?;
            let content_id: String = row.get(1)?;
            let theme_str: String = row.get(2)?;
            let mode_str: String = row.get(3)?;
            let steps_json: String = row.get(4)?;
            let current_step_index: i32 = row.get(5)?;
            let created_at: i64 = row.get(6)?;
            let updated_at: i64 = row.get(7)?;

            Ok(WorkflowProgress {
                workflow_id,
                content_id,
                theme: serde_json::from_str(&theme_str).unwrap_or_default(),
                mode: serde_json::from_str(&mode_str).unwrap_or_default(),
                steps_json,
                current_step_index,
                created_at,
                updated_at,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }

    /// 清理过期的工作流（超过指定天数）
    pub async fn cleanup_expired(&self, days: i64) -> Result<usize> {
        let conn = self.conn.lock().await;

        let cutoff = chrono::Utc::now().timestamp_millis() - (days * 24 * 60 * 60 * 1000);

        let count = conn.execute(
            "DELETE FROM workflow_progress WHERE updated_at < ?1",
            params![cutoff],
        )?;

        if count > 0 {
            info!("清理了 {} 个过期工作流", count);
        }

        Ok(count)
    }
}
