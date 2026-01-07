pub mod dao;
pub mod migration;
pub mod schema;
pub mod system_providers;

use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub type DbConnection = Arc<Mutex<Connection>>;

/// 获取数据库文件路径
pub fn get_db_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "无法获取主目录".to_string())?;
    let db_dir = home.join(".proxycast");
    std::fs::create_dir_all(&db_dir)
        .map_err(|e| format!("无法创建数据库目录 {:?}: {}", db_dir, e))?;
    Ok(db_dir.join("proxycast.db"))
}

/// 初始化数据库连接
pub fn init_database() -> Result<DbConnection, String> {
    let db_path = get_db_path()?;
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

    // 设置 busy_timeout 为 5 秒，避免 "database is locked" 错误
    conn.busy_timeout(std::time::Duration::from_secs(5))
        .map_err(|e| format!("设置 busy_timeout 失败: {}", e))?;

    // 创建表结构
    schema::create_tables(&conn).map_err(|e| e.to_string())?;
    migration::migrate_from_json(&conn)?;

    // 执行 API Keys 到 Provider Pool 的迁移
    match migration::migrate_api_keys_to_pool(&conn) {
        Ok(count) => {
            if count > 0 {
                tracing::info!("[数据库] 已将 {} 条 API Key 迁移到凭证池", count);
            }
        }
        Err(e) => {
            tracing::warn!("[数据库] API Key 迁移失败（非致命）: {}", e);
        }
    }

    // 清理旧的 API Key 凭证（openai_key, claude_key 类型）
    match migration::cleanup_legacy_api_key_credentials(&conn) {
        Ok(count) => {
            if count > 0 {
                tracing::info!("[数据库] 已清理 {} 条旧 API Key 凭证", count);
            }
        }
        Err(e) => {
            tracing::warn!("[数据库] 旧 API Key 凭证清理失败（非致命）: {}", e);
        }
    }

    Ok(Arc::new(Mutex::new(conn)))
}
