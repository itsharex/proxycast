//! 插件凭证数据访问对象
//!
//! 提供 OAuth Provider 插件凭证的 CRUD 操作。

#![allow(dead_code)]

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

/// 凭证状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CredentialStatus {
    /// 活跃可用
    Active,
    /// 已禁用
    Disabled,
    /// 已过期
    Expired,
    /// 错误状态
    Error,
}

impl CredentialStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            CredentialStatus::Active => "active",
            CredentialStatus::Disabled => "disabled",
            CredentialStatus::Expired => "expired",
            CredentialStatus::Error => "error",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "active" => CredentialStatus::Active,
            "disabled" => CredentialStatus::Disabled,
            "expired" => CredentialStatus::Expired,
            "error" => CredentialStatus::Error,
            _ => CredentialStatus::Active,
        }
    }
}

/// 插件凭证记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCredentialRecord {
    /// 凭证 ID
    pub id: String,
    /// 插件 ID
    pub plugin_id: String,
    /// 认证类型 (oauth, api_key, cookie, etc.)
    pub auth_type: String,
    /// 显示名称
    pub display_name: Option<String>,
    /// 状态
    pub status: CredentialStatus,
    /// 加密配置 (JSON)
    pub config_encrypted: String,
    /// 使用次数
    pub usage_count: u32,
    /// 错误次数
    pub error_count: u32,
    /// 最后使用时间
    pub last_used_at: Option<DateTime<Utc>>,
    /// 最后错误时间
    pub last_error_at: Option<DateTime<Utc>>,
    /// 最后错误消息
    pub last_error_message: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// 新建凭证参数
#[derive(Debug, Clone)]
pub struct NewPluginCredential {
    pub id: String,
    pub plugin_id: String,
    pub auth_type: String,
    pub display_name: Option<String>,
    pub config_encrypted: String,
}

/// 数据库行结构
struct CredentialRow {
    id: String,
    plugin_id: String,
    auth_type: String,
    display_name: Option<String>,
    status: String,
    config_encrypted: String,
    usage_count: i32,
    error_count: i32,
    last_used_at: Option<String>,
    last_error_at: Option<String>,
    last_error_message: Option<String>,
    created_at: String,
    updated_at: String,
}

impl CredentialRow {
    fn into_record(self) -> Result<PluginCredentialRecord, String> {
        let created_at = DateTime::parse_from_rfc3339(&self.created_at)
            .map_err(|e| format!("无效的创建时间格式: {}", e))?
            .with_timezone(&Utc);

        let updated_at = DateTime::parse_from_rfc3339(&self.updated_at)
            .map_err(|e| format!("无效的更新时间格式: {}", e))?
            .with_timezone(&Utc);

        let last_used_at = self
            .last_used_at
            .map(|s| DateTime::parse_from_rfc3339(&s).ok())
            .flatten()
            .map(|dt| dt.with_timezone(&Utc));

        let last_error_at = self
            .last_error_at
            .map(|s| DateTime::parse_from_rfc3339(&s).ok())
            .flatten()
            .map(|dt| dt.with_timezone(&Utc));

        Ok(PluginCredentialRecord {
            id: self.id,
            plugin_id: self.plugin_id,
            auth_type: self.auth_type,
            display_name: self.display_name,
            status: CredentialStatus::from_str(&self.status),
            config_encrypted: self.config_encrypted,
            usage_count: self.usage_count as u32,
            error_count: self.error_count as u32,
            last_used_at,
            last_error_at,
            last_error_message: self.last_error_message,
            created_at,
            updated_at,
        })
    }
}

pub struct PluginCredentialDao;

impl PluginCredentialDao {
    /// 创建凭证
    pub fn create(
        conn: &Connection,
        credential: &NewPluginCredential,
    ) -> Result<(), rusqlite::Error> {
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO plugin_credentials
             (id, plugin_id, auth_type, display_name, status, config_encrypted,
              usage_count, error_count, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'active', ?5, 0, 0, ?6, ?7)",
            params![
                credential.id,
                credential.plugin_id,
                credential.auth_type,
                credential.display_name,
                credential.config_encrypted,
                now,
                now,
            ],
        )?;

        Ok(())
    }

    /// 获取单个凭证
    pub fn get(
        conn: &Connection,
        credential_id: &str,
    ) -> Result<Option<PluginCredentialRecord>, String> {
        let result = conn
            .query_row(
                "SELECT id, plugin_id, auth_type, display_name, status, config_encrypted,
                        usage_count, error_count, last_used_at, last_error_at,
                        last_error_message, created_at, updated_at
                 FROM plugin_credentials WHERE id = ?1",
                params![credential_id],
                |row| {
                    Ok(CredentialRow {
                        id: row.get(0)?,
                        plugin_id: row.get(1)?,
                        auth_type: row.get(2)?,
                        display_name: row.get(3)?,
                        status: row.get(4)?,
                        config_encrypted: row.get(5)?,
                        usage_count: row.get(6)?,
                        error_count: row.get(7)?,
                        last_used_at: row.get(8)?,
                        last_error_at: row.get(9)?,
                        last_error_message: row.get(10)?,
                        created_at: row.get(11)?,
                        updated_at: row.get(12)?,
                    })
                },
            )
            .optional()
            .map_err(|e| format!("数据库错误: {}", e))?;

        match result {
            Some(row) => Ok(Some(row.into_record()?)),
            None => Ok(None),
        }
    }

    /// 列出插件的所有凭证
    pub fn list_by_plugin(
        conn: &Connection,
        plugin_id: &str,
    ) -> Result<Vec<PluginCredentialRecord>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT id, plugin_id, auth_type, display_name, status, config_encrypted,
                        usage_count, error_count, last_used_at, last_error_at,
                        last_error_message, created_at, updated_at
                 FROM plugin_credentials WHERE plugin_id = ?1 ORDER BY created_at DESC",
            )
            .map_err(|e| format!("数据库错误: {}", e))?;

        let rows = stmt
            .query_map(params![plugin_id], |row| {
                Ok(CredentialRow {
                    id: row.get(0)?,
                    plugin_id: row.get(1)?,
                    auth_type: row.get(2)?,
                    display_name: row.get(3)?,
                    status: row.get(4)?,
                    config_encrypted: row.get(5)?,
                    usage_count: row.get(6)?,
                    error_count: row.get(7)?,
                    last_used_at: row.get(8)?,
                    last_error_at: row.get(9)?,
                    last_error_message: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                })
            })
            .map_err(|e| format!("数据库错误: {}", e))?;

        let mut credentials = Vec::new();
        for row in rows {
            let row = row.map_err(|e| format!("数据库错误: {}", e))?;
            credentials.push(row.into_record()?);
        }

        Ok(credentials)
    }

    /// 列出所有活跃凭证
    pub fn list_active(conn: &Connection) -> Result<Vec<PluginCredentialRecord>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT id, plugin_id, auth_type, display_name, status, config_encrypted,
                        usage_count, error_count, last_used_at, last_error_at,
                        last_error_message, created_at, updated_at
                 FROM plugin_credentials WHERE status = 'active' ORDER BY usage_count DESC",
            )
            .map_err(|e| format!("数据库错误: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(CredentialRow {
                    id: row.get(0)?,
                    plugin_id: row.get(1)?,
                    auth_type: row.get(2)?,
                    display_name: row.get(3)?,
                    status: row.get(4)?,
                    config_encrypted: row.get(5)?,
                    usage_count: row.get(6)?,
                    error_count: row.get(7)?,
                    last_used_at: row.get(8)?,
                    last_error_at: row.get(9)?,
                    last_error_message: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                })
            })
            .map_err(|e| format!("数据库错误: {}", e))?;

        let mut credentials = Vec::new();
        for row in rows {
            let row = row.map_err(|e| format!("数据库错误: {}", e))?;
            credentials.push(row.into_record()?);
        }

        Ok(credentials)
    }

    /// 更新凭证配置
    pub fn update_config(
        conn: &Connection,
        credential_id: &str,
        config_encrypted: &str,
    ) -> Result<bool, rusqlite::Error> {
        let now = Utc::now().to_rfc3339();
        let rows_affected = conn.execute(
            "UPDATE plugin_credentials SET config_encrypted = ?1, updated_at = ?2 WHERE id = ?3",
            params![config_encrypted, now, credential_id],
        )?;

        Ok(rows_affected > 0)
    }

    /// 更新凭证状态
    pub fn update_status(
        conn: &Connection,
        credential_id: &str,
        status: CredentialStatus,
    ) -> Result<bool, rusqlite::Error> {
        let now = Utc::now().to_rfc3339();
        let rows_affected = conn.execute(
            "UPDATE plugin_credentials SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status.as_str(), now, credential_id],
        )?;

        Ok(rows_affected > 0)
    }

    /// 记录使用
    pub fn record_usage(conn: &Connection, credential_id: &str) -> Result<bool, rusqlite::Error> {
        let now = Utc::now().to_rfc3339();
        let rows_affected = conn.execute(
            "UPDATE plugin_credentials
             SET usage_count = usage_count + 1, last_used_at = ?1, updated_at = ?2
             WHERE id = ?3",
            params![now, now, credential_id],
        )?;

        Ok(rows_affected > 0)
    }

    /// 记录错误
    pub fn record_error(
        conn: &Connection,
        credential_id: &str,
        error_message: &str,
    ) -> Result<bool, rusqlite::Error> {
        let now = Utc::now().to_rfc3339();
        let rows_affected = conn.execute(
            "UPDATE plugin_credentials
             SET error_count = error_count + 1, last_error_at = ?1,
                 last_error_message = ?2, updated_at = ?3
             WHERE id = ?4",
            params![now, error_message, now, credential_id],
        )?;

        Ok(rows_affected > 0)
    }

    /// 重置错误计数
    pub fn reset_errors(conn: &Connection, credential_id: &str) -> Result<bool, rusqlite::Error> {
        let now = Utc::now().to_rfc3339();
        let rows_affected = conn.execute(
            "UPDATE plugin_credentials
             SET error_count = 0, last_error_at = NULL, last_error_message = NULL,
                 status = 'active', updated_at = ?1
             WHERE id = ?2",
            params![now, credential_id],
        )?;

        Ok(rows_affected > 0)
    }

    /// 删除凭证
    pub fn delete(conn: &Connection, credential_id: &str) -> Result<bool, rusqlite::Error> {
        let rows_affected = conn.execute(
            "DELETE FROM plugin_credentials WHERE id = ?1",
            params![credential_id],
        )?;

        Ok(rows_affected > 0)
    }

    /// 删除插件的所有凭证
    pub fn delete_by_plugin(conn: &Connection, plugin_id: &str) -> Result<u32, rusqlite::Error> {
        let rows_affected = conn.execute(
            "DELETE FROM plugin_credentials WHERE plugin_id = ?1",
            params![plugin_id],
        )?;

        Ok(rows_affected as u32)
    }

    /// 统计插件凭证数量
    pub fn count_by_plugin(conn: &Connection, plugin_id: &str) -> Result<u32, rusqlite::Error> {
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM plugin_credentials WHERE plugin_id = ?1",
            params![plugin_id],
            |row| row.get(0),
        )?;

        Ok(count as u32)
    }

    /// 统计活跃凭证数量
    pub fn count_active_by_plugin(
        conn: &Connection,
        plugin_id: &str,
    ) -> Result<u32, rusqlite::Error> {
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM plugin_credentials WHERE plugin_id = ?1 AND status = 'active'",
            params![plugin_id],
            |row| row.get(0),
        )?;

        Ok(count as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_connection() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS plugin_credentials (
                id TEXT PRIMARY KEY,
                plugin_id TEXT NOT NULL,
                auth_type TEXT NOT NULL,
                display_name TEXT,
                status TEXT NOT NULL DEFAULT 'active',
                config_encrypted TEXT NOT NULL,
                usage_count INTEGER DEFAULT 0,
                error_count INTEGER DEFAULT 0,
                last_used_at TEXT,
                last_error_at TEXT,
                last_error_message TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )
        .unwrap();
        conn
    }

    fn create_test_credential(id: &str, plugin_id: &str) -> NewPluginCredential {
        NewPluginCredential {
            id: id.to_string(),
            plugin_id: plugin_id.to_string(),
            auth_type: "oauth".to_string(),
            display_name: Some("Test Credential".to_string()),
            config_encrypted: r#"{"token":"test"}"#.to_string(),
        }
    }

    #[test]
    fn test_create_and_get() {
        let conn = create_test_connection();
        let credential = create_test_credential("cred-1", "plugin-1");

        PluginCredentialDao::create(&conn, &credential).unwrap();

        let retrieved = PluginCredentialDao::get(&conn, "cred-1").unwrap().unwrap();
        assert_eq!(retrieved.id, "cred-1");
        assert_eq!(retrieved.plugin_id, "plugin-1");
        assert_eq!(retrieved.auth_type, "oauth");
        assert_eq!(retrieved.status, CredentialStatus::Active);
    }

    #[test]
    fn test_list_by_plugin() {
        let conn = create_test_connection();

        PluginCredentialDao::create(&conn, &create_test_credential("cred-1", "plugin-1")).unwrap();
        PluginCredentialDao::create(&conn, &create_test_credential("cred-2", "plugin-1")).unwrap();
        PluginCredentialDao::create(&conn, &create_test_credential("cred-3", "plugin-2")).unwrap();

        let credentials = PluginCredentialDao::list_by_plugin(&conn, "plugin-1").unwrap();
        assert_eq!(credentials.len(), 2);
    }

    #[test]
    fn test_update_status() {
        let conn = create_test_connection();
        PluginCredentialDao::create(&conn, &create_test_credential("cred-1", "plugin-1")).unwrap();

        PluginCredentialDao::update_status(&conn, "cred-1", CredentialStatus::Disabled).unwrap();

        let retrieved = PluginCredentialDao::get(&conn, "cred-1").unwrap().unwrap();
        assert_eq!(retrieved.status, CredentialStatus::Disabled);
    }

    #[test]
    fn test_record_usage() {
        let conn = create_test_connection();
        PluginCredentialDao::create(&conn, &create_test_credential("cred-1", "plugin-1")).unwrap();

        PluginCredentialDao::record_usage(&conn, "cred-1").unwrap();
        PluginCredentialDao::record_usage(&conn, "cred-1").unwrap();

        let retrieved = PluginCredentialDao::get(&conn, "cred-1").unwrap().unwrap();
        assert_eq!(retrieved.usage_count, 2);
        assert!(retrieved.last_used_at.is_some());
    }

    #[test]
    fn test_record_error() {
        let conn = create_test_connection();
        PluginCredentialDao::create(&conn, &create_test_credential("cred-1", "plugin-1")).unwrap();

        PluginCredentialDao::record_error(&conn, "cred-1", "Token expired").unwrap();

        let retrieved = PluginCredentialDao::get(&conn, "cred-1").unwrap().unwrap();
        assert_eq!(retrieved.error_count, 1);
        assert_eq!(
            retrieved.last_error_message,
            Some("Token expired".to_string())
        );
    }

    #[test]
    fn test_delete() {
        let conn = create_test_connection();
        PluginCredentialDao::create(&conn, &create_test_credential("cred-1", "plugin-1")).unwrap();

        let deleted = PluginCredentialDao::delete(&conn, "cred-1").unwrap();
        assert!(deleted);

        let retrieved = PluginCredentialDao::get(&conn, "cred-1").unwrap();
        assert!(retrieved.is_none());
    }
}
