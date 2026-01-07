//! 插件注册表
//!
//! 管理已安装插件的元数据

use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::{Arc, Mutex};

use super::types::{InstallError, InstallSource, InstalledPlugin};

/// 插件注册表
///
/// 管理已安装插件的元数据
pub struct PluginRegistry {
    conn: Arc<Mutex<Connection>>,
}

impl PluginRegistry {
    /// 创建新的注册表实例
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// 从数据库路径创建注册表
    pub fn from_path(db_path: &Path) -> Result<Self, InstallError> {
        let conn =
            Connection::open(db_path).map_err(|e| InstallError::DatabaseError(e.to_string()))?;

        // 设置 busy_timeout 为 5 秒，避免 "database is locked" 错误
        conn.busy_timeout(std::time::Duration::from_secs(5))
            .map_err(|e| InstallError::DatabaseError(format!("设置 busy_timeout 失败: {}", e)))?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// 初始化数据库表
    pub fn init_tables(&self) -> Result<(), InstallError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| InstallError::DatabaseError(e.to_string()))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS installed_plugins (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                version TEXT NOT NULL,
                description TEXT,
                author TEXT,
                install_path TEXT NOT NULL,
                installed_at TEXT NOT NULL,
                source_type TEXT NOT NULL,
                source_data TEXT,
                enabled INTEGER DEFAULT 1
            )",
            [],
        )
        .map_err(|e| InstallError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// 注册插件
    ///
    /// _需求: 1.2_
    pub fn register(&self, plugin: &InstalledPlugin) -> Result<(), InstallError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| InstallError::DatabaseError(e.to_string()))?;

        let (source_type, source_data) = serialize_source(&plugin.source);

        conn.execute(
            "INSERT OR REPLACE INTO installed_plugins 
             (id, name, version, description, author, install_path, installed_at, source_type, source_data, enabled)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                plugin.id,
                plugin.name,
                plugin.version,
                plugin.description,
                plugin.author,
                plugin.install_path.to_string_lossy().to_string(),
                plugin.installed_at.to_rfc3339(),
                source_type,
                source_data,
                plugin.enabled as i32,
            ],
        )
        .map_err(|e| InstallError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// 注销插件
    ///
    /// _需求: 4.2_
    pub fn unregister(&self, plugin_id: &str) -> Result<(), InstallError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| InstallError::DatabaseError(e.to_string()))?;

        let rows_affected = conn
            .execute(
                "DELETE FROM installed_plugins WHERE id = ?1",
                params![plugin_id],
            )
            .map_err(|e| InstallError::DatabaseError(e.to_string()))?;

        if rows_affected == 0 {
            return Err(InstallError::NotFound(plugin_id.to_string()));
        }

        Ok(())
    }

    /// 获取插件信息
    pub fn get(&self, plugin_id: &str) -> Result<Option<InstalledPlugin>, InstallError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| InstallError::DatabaseError(e.to_string()))?;

        let result = conn
            .query_row(
                "SELECT id, name, version, description, author, install_path, installed_at, source_type, source_data, enabled
                 FROM installed_plugins WHERE id = ?1",
                params![plugin_id],
                |row| {
                    Ok(PluginRow {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        version: row.get(2)?,
                        description: row.get(3)?,
                        author: row.get(4)?,
                        install_path: row.get(5)?,
                        installed_at: row.get(6)?,
                        source_type: row.get(7)?,
                        source_data: row.get(8)?,
                        enabled: row.get(9)?,
                    })
                },
            )
            .optional()
            .map_err(|e| InstallError::DatabaseError(e.to_string()))?;

        match result {
            Some(row) => Ok(Some(row.into_installed_plugin()?)),
            None => Ok(None),
        }
    }

    /// 列出所有插件
    pub fn list(&self) -> Result<Vec<InstalledPlugin>, InstallError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| InstallError::DatabaseError(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, name, version, description, author, install_path, installed_at, source_type, source_data, enabled
                 FROM installed_plugins ORDER BY installed_at DESC",
            )
            .map_err(|e| InstallError::DatabaseError(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(PluginRow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    version: row.get(2)?,
                    description: row.get(3)?,
                    author: row.get(4)?,
                    install_path: row.get(5)?,
                    installed_at: row.get(6)?,
                    source_type: row.get(7)?,
                    source_data: row.get(8)?,
                    enabled: row.get(9)?,
                })
            })
            .map_err(|e| InstallError::DatabaseError(e.to_string()))?;

        let mut plugins = Vec::new();
        for row in rows {
            let row = row.map_err(|e| InstallError::DatabaseError(e.to_string()))?;
            plugins.push(row.into_installed_plugin()?);
        }

        Ok(plugins)
    }

    /// 检查插件是否存在
    pub fn exists(&self, plugin_id: &str) -> Result<bool, InstallError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| InstallError::DatabaseError(e.to_string()))?;

        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM installed_plugins WHERE id = ?1",
                params![plugin_id],
                |row| row.get(0),
            )
            .map_err(|e| InstallError::DatabaseError(e.to_string()))?;

        Ok(count > 0)
    }

    /// 更新插件启用状态
    pub fn set_enabled(&self, plugin_id: &str, enabled: bool) -> Result<(), InstallError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| InstallError::DatabaseError(e.to_string()))?;

        let rows_affected = conn
            .execute(
                "UPDATE installed_plugins SET enabled = ?1 WHERE id = ?2",
                params![enabled as i32, plugin_id],
            )
            .map_err(|e| InstallError::DatabaseError(e.to_string()))?;

        if rows_affected == 0 {
            return Err(InstallError::NotFound(plugin_id.to_string()));
        }

        Ok(())
    }
}

/// 数据库行结构
struct PluginRow {
    id: String,
    name: String,
    version: String,
    description: Option<String>,
    author: Option<String>,
    install_path: String,
    installed_at: String,
    source_type: String,
    source_data: Option<String>,
    enabled: i32,
}

impl PluginRow {
    fn into_installed_plugin(self) -> Result<InstalledPlugin, InstallError> {
        let source = deserialize_source(&self.source_type, self.source_data.as_deref())?;
        let installed_at = chrono::DateTime::parse_from_rfc3339(&self.installed_at)
            .map_err(|e| InstallError::DatabaseError(format!("无效的时间格式: {}", e)))?
            .with_timezone(&chrono::Utc);

        Ok(InstalledPlugin {
            id: self.id,
            name: self.name,
            version: self.version,
            description: self.description.unwrap_or_default(),
            author: self.author,
            install_path: std::path::PathBuf::from(self.install_path),
            installed_at,
            source,
            enabled: self.enabled != 0,
        })
    }
}

/// 序列化安装来源
fn serialize_source(source: &InstallSource) -> (String, Option<String>) {
    match source {
        InstallSource::Local { path } => ("local".to_string(), Some(path.clone())),
        InstallSource::Url { url } => ("url".to_string(), Some(url.clone())),
        InstallSource::GitHub { owner, repo, tag } => {
            let data = serde_json::json!({
                "owner": owner,
                "repo": repo,
                "tag": tag
            });
            ("github".to_string(), Some(data.to_string()))
        }
    }
}

/// 反序列化安装来源
fn deserialize_source(
    source_type: &str,
    source_data: Option<&str>,
) -> Result<InstallSource, InstallError> {
    match source_type {
        "local" => Ok(InstallSource::Local {
            path: source_data.unwrap_or_default().to_string(),
        }),
        "url" => Ok(InstallSource::Url {
            url: source_data.unwrap_or_default().to_string(),
        }),
        "github" => {
            let data: serde_json::Value = serde_json::from_str(source_data.unwrap_or("{}"))?;
            Ok(InstallSource::GitHub {
                owner: data["owner"].as_str().unwrap_or_default().to_string(),
                repo: data["repo"].as_str().unwrap_or_default().to_string(),
                tag: data["tag"].as_str().unwrap_or_default().to_string(),
            })
        }
        _ => Err(InstallError::DatabaseError(format!(
            "未知的来源类型: {}",
            source_type
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_registry() -> PluginRegistry {
        let conn = Connection::open_in_memory().unwrap();
        let registry = PluginRegistry {
            conn: Arc::new(Mutex::new(conn)),
        };
        registry.init_tables().unwrap();
        registry
    }

    fn create_test_plugin(id: &str) -> InstalledPlugin {
        InstalledPlugin {
            id: id.to_string(),
            name: format!("Test Plugin {}", id),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: Some("Test Author".to_string()),
            install_path: PathBuf::from(format!("/plugins/{}", id)),
            installed_at: chrono::Utc::now(),
            source: InstallSource::Local {
                path: "/tmp/plugin.zip".to_string(),
            },
            enabled: true,
        }
    }

    #[test]
    fn test_register_and_get() {
        let registry = create_test_registry();
        let plugin = create_test_plugin("test-1");

        registry.register(&plugin).unwrap();

        let retrieved = registry.get("test-1").unwrap().unwrap();
        assert_eq!(retrieved.id, "test-1");
        assert_eq!(retrieved.name, "Test Plugin test-1");
        assert_eq!(retrieved.version, "1.0.0");
    }

    #[test]
    fn test_unregister() {
        let registry = create_test_registry();
        let plugin = create_test_plugin("test-2");

        registry.register(&plugin).unwrap();
        assert!(registry.exists("test-2").unwrap());

        registry.unregister("test-2").unwrap();
        assert!(!registry.exists("test-2").unwrap());
    }

    #[test]
    fn test_unregister_not_found() {
        let registry = create_test_registry();
        let result = registry.unregister("non-existent");
        assert!(result.is_err());
    }

    #[test]
    fn test_list() {
        let registry = create_test_registry();

        registry.register(&create_test_plugin("test-a")).unwrap();
        registry.register(&create_test_plugin("test-b")).unwrap();

        let plugins = registry.list().unwrap();
        assert_eq!(plugins.len(), 2);
    }

    #[test]
    fn test_set_enabled() {
        let registry = create_test_registry();
        let plugin = create_test_plugin("test-3");

        registry.register(&plugin).unwrap();

        registry.set_enabled("test-3", false).unwrap();
        let retrieved = registry.get("test-3").unwrap().unwrap();
        assert!(!retrieved.enabled);

        registry.set_enabled("test-3", true).unwrap();
        let retrieved = registry.get("test-3").unwrap().unwrap();
        assert!(retrieved.enabled);
    }

    #[test]
    fn test_github_source_serialization() {
        let registry = create_test_registry();
        let mut plugin = create_test_plugin("test-github");
        plugin.source = InstallSource::GitHub {
            owner: "user".to_string(),
            repo: "repo".to_string(),
            tag: "v1.0.0".to_string(),
        };

        registry.register(&plugin).unwrap();

        let retrieved = registry.get("test-github").unwrap().unwrap();
        match retrieved.source {
            InstallSource::GitHub { owner, repo, tag } => {
                assert_eq!(owner, "user");
                assert_eq!(repo, "repo");
                assert_eq!(tag, "v1.0.0");
            }
            _ => panic!("Expected GitHub source"),
        }
    }
}
