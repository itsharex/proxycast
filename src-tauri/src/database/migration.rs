use rusqlite::{params, Connection};

/// 从旧的 JSON 配置迁移数据到 SQLite
#[allow(dead_code)]
pub fn migrate_from_json(conn: &Connection) -> Result<(), String> {
    // 检查是否已经迁移过
    let migrated: bool = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'migrated_from_json'",
            [],
            |row| row.get::<_, String>(0),
        )
        .map(|v| v == "true")
        .unwrap_or(false);

    if migrated {
        return Ok(());
    }

    // 读取旧配置文件（历史路径）
    let home = dirs::home_dir().ok_or_else(|| "无法获取主目录".to_string())?;
    let config_path = home.join(".proxycast").join("config.json");

    if config_path.exists() {
        // 备份旧配置，避免误覆盖
        let backup_path = config_path.with_file_name("config.json.backup");
        if !backup_path.exists() {
            std::fs::copy(&config_path, &backup_path)
                .map_err(|e| format!("备份旧配置失败: {}", e))?;
        }

        return Err(
            "检测到旧版 config.json（~/.proxycast/config.json），当前版本尚未支持自动迁移。请手动导出/重建配置后再启动。"
                .to_string(),
        );
    }

    // 标记迁移完成
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('migrated_from_json', 'true')",
        [],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// 将 api_keys 表中的数据迁移到 provider_pool_credentials 表
///
/// 迁移逻辑：
/// 1. 读取 api_keys 表中的所有 API Key
/// 2. 根据 provider_id 查找对应的 api_key_providers 配置
/// 3. 将 API Key 转换为 CredentialData::OpenAIKey 或 CredentialData::ClaudeKey
/// 4. 插入到 provider_pool_credentials 表
pub fn migrate_api_keys_to_pool(conn: &Connection) -> Result<usize, String> {
    // 检查是否已经迁移过
    let migrated: bool = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'migrated_api_keys_to_pool'",
            [],
            |row| row.get::<_, String>(0),
        )
        .map(|v| v == "true")
        .unwrap_or(false);

    if migrated {
        tracing::debug!("[迁移] API Keys 已迁移过，跳过");
        return Ok(0);
    }

    tracing::info!("[迁移] 开始将 api_keys 迁移到 provider_pool_credentials");

    // 查询所有 API Keys 及其对应的 Provider 信息
    let mut stmt = conn
        .prepare(
            "SELECT k.id, k.provider_id, k.api_key_encrypted, k.alias, k.enabled,
                    k.usage_count, k.error_count, k.last_used_at, k.created_at,
                    p.type, p.api_host, p.name as provider_name
             FROM api_keys k
             JOIN api_key_providers p ON k.provider_id = p.id
             ORDER BY k.created_at ASC",
        )
        .map_err(|e| format!("准备查询语句失败: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ApiKeyMigrationRow {
                id: row.get(0)?,
                provider_id: row.get(1)?,
                api_key_encrypted: row.get(2)?,
                alias: row.get(3)?,
                enabled: row.get(4)?,
                usage_count: row.get::<_, i64>(5)? as u64,
                error_count: row.get::<_, i64>(6)? as u32,
                last_used_at: row.get(7)?,
                created_at: row.get(8)?,
                provider_type: row.get(9)?,
                api_host: row.get(10)?,
                provider_name: row.get(11)?,
            })
        })
        .map_err(|e| format!("查询 API Keys 失败: {}", e))?;

    let mut migrated_count = 0;
    let now = chrono::Utc::now().timestamp();

    for row_result in rows {
        let row = row_result.map_err(|e| format!("读取行数据失败: {}", e))?;

        // 检查是否已存在相同的凭证（通过 api_key_encrypted 判断）
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM provider_pool_credentials 
                 WHERE credential_data LIKE ?1",
                params![format!("%{}%", row.api_key_encrypted)],
                |r| r.get(0),
            )
            .unwrap_or(false);

        if exists {
            tracing::debug!(
                "[迁移] 跳过已存在的 API Key: {} (provider: {})",
                row.alias.as_deref().unwrap_or(&row.id),
                row.provider_id
            );
            continue;
        }

        // 根据 provider_type 确定 pool_provider_type 和 credential_data
        let (pool_provider_type, credential_data) = match row.provider_type.to_lowercase().as_str()
        {
            "anthropic" => {
                let cred = serde_json::json!({
                    "type": "claude_key",
                    "api_key": row.api_key_encrypted,
                    "base_url": if row.api_host.is_empty() { None } else { Some(&row.api_host) }
                });
                ("claude", cred)
            }
            "openai" | "openai-response" => {
                let cred = serde_json::json!({
                    "type": "openai_key",
                    "api_key": row.api_key_encrypted,
                    "base_url": if row.api_host.is_empty() { None } else { Some(&row.api_host) }
                });
                ("openai", cred)
            }
            "gemini" => {
                let cred = serde_json::json!({
                    "type": "gemini_api_key",
                    "api_key": row.api_key_encrypted,
                    "base_url": if row.api_host.is_empty() { None } else { Some(&row.api_host) },
                    "excluded_models": []
                });
                ("gemini_api_key", cred)
            }
            "vertex" | "vertexai" => {
                let cred = serde_json::json!({
                    "type": "vertex_key",
                    "api_key": row.api_key_encrypted,
                    "base_url": if row.api_host.is_empty() { None } else { Some(&row.api_host) },
                    "model_aliases": {}
                });
                ("vertex", cred)
            }
            // 其他类型默认作为 OpenAI 兼容处理
            _ => {
                let cred = serde_json::json!({
                    "type": "openai_key",
                    "api_key": row.api_key_encrypted,
                    "base_url": if row.api_host.is_empty() { None } else { Some(&row.api_host) }
                });
                ("openai", cred)
            }
        };

        // 生成名称：优先使用 alias，否则使用 provider_name
        let name = row
            .alias
            .clone()
            .or_else(|| Some(format!("{} (迁移)", row.provider_name)));

        // 解析时间
        let created_at_ts = row
            .created_at
            .as_ref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.timestamp())
            .unwrap_or(now);

        let last_used_ts = row
            .last_used_at
            .as_ref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.timestamp());

        // 插入到 provider_pool_credentials
        let uuid = uuid::Uuid::new_v4().to_string();
        let credential_json = credential_data.to_string();

        conn.execute(
            "INSERT INTO provider_pool_credentials
             (uuid, provider_type, credential_data, name, is_healthy, is_disabled,
              check_health, check_model_name, not_supported_models, usage_count, error_count,
              last_used, last_error_time, last_error_message, last_health_check_time,
              last_health_check_model, created_at, updated_at, source, proxy_url)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)",
            params![
                uuid,
                pool_provider_type,
                credential_json,
                name,
                true,                    // is_healthy
                !row.enabled,            // is_disabled (反转 enabled)
                true,                    // check_health
                Option::<String>::None,  // check_model_name
                "[]",                    // not_supported_models
                row.usage_count as i64,
                row.error_count as i32,
                last_used_ts,
                Option::<i64>::None,     // last_error_time
                Option::<String>::None,  // last_error_message
                Option::<i64>::None,     // last_health_check_time
                Option::<String>::None,  // last_health_check_model
                created_at_ts,
                now,
                "imported",              // source: 标记为导入
                Option::<String>::None,  // proxy_url
            ],
        )
        .map_err(|e| format!("插入凭证失败: {}", e))?;

        tracing::info!(
            "[迁移] 已迁移 API Key: {} -> {} (provider_type: {})",
            row.alias.as_deref().unwrap_or(&row.id),
            uuid,
            pool_provider_type
        );

        migrated_count += 1;
    }

    // 标记迁移完成
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('migrated_api_keys_to_pool', 'true')",
        [],
    )
    .map_err(|e| format!("标记迁移完成失败: {}", e))?;

    tracing::info!("[迁移] API Keys 迁移完成，共迁移 {} 条记录", migrated_count);

    Ok(migrated_count)
}

/// API Key 迁移行数据
struct ApiKeyMigrationRow {
    id: String,
    provider_id: String,
    api_key_encrypted: String,
    alias: Option<String>,
    enabled: bool,
    usage_count: u64,
    error_count: u32,
    last_used_at: Option<String>,
    created_at: Option<String>,
    provider_type: String,
    api_host: String,
    provider_name: String,
}

/// 清理旧的 API Key 凭证（OpenAIKey 和 ClaudeKey 类型）
///
/// 这些凭证是通过旧的 UI 添加的，现在已经被新的 API Key Provider 系统取代。
/// 此函数会删除 provider_pool_credentials 表中的 openai_key 和 claude_key 类型凭证。
pub fn cleanup_legacy_api_key_credentials(conn: &Connection) -> Result<usize, String> {
    // 检查是否已经清理过
    let cleaned: bool = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'cleaned_legacy_api_key_credentials'",
            [],
            |row| row.get::<_, String>(0),
        )
        .map(|v| v == "true")
        .unwrap_or(false);

    if cleaned {
        tracing::debug!("[清理] 旧 API Key 凭证已清理过，跳过");
        return Ok(0);
    }

    tracing::info!("[清理] 开始清理旧的 API Key 凭证（openai_key, claude_key 类型）");

    // 查询需要清理的凭证数量
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM provider_pool_credentials
             WHERE credential_data LIKE '%\"type\":\"openai_key\"%'
                OR credential_data LIKE '%\"type\":\"claude_key\"%'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if count == 0 {
        tracing::info!("[清理] 没有需要清理的旧 API Key 凭证");
        // 标记清理完成
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('cleaned_legacy_api_key_credentials', 'true')",
            [],
        )
        .map_err(|e| format!("标记清理完成失败: {}", e))?;
        return Ok(0);
    }

    // 记录将要删除的凭证信息
    let mut stmt = conn
        .prepare(
            "SELECT uuid, name, provider_type, credential_data
             FROM provider_pool_credentials
             WHERE credential_data LIKE '%\"type\":\"openai_key\"%'
                OR credential_data LIKE '%\"type\":\"claude_key\"%'",
        )
        .map_err(|e| format!("准备查询语句失败: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| format!("查询旧凭证失败: {}", e))?;

    for row_result in rows {
        if let Ok((uuid, name, provider_type)) = row_result {
            tracing::info!(
                "[清理] 将删除旧凭证: {} (name: {}, type: {})",
                uuid,
                name.as_deref().unwrap_or("未命名"),
                provider_type
            );
        }
    }

    // 删除旧的 API Key 凭证
    let deleted = conn
        .execute(
            "DELETE FROM provider_pool_credentials
             WHERE credential_data LIKE '%\"type\":\"openai_key\"%'
                OR credential_data LIKE '%\"type\":\"claude_key\"%'",
            [],
        )
        .map_err(|e| format!("删除旧凭证失败: {}", e))?;

    // 标记清理完成
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('cleaned_legacy_api_key_credentials', 'true')",
        [],
    )
    .map_err(|e| format!("标记清理完成失败: {}", e))?;

    tracing::info!("[清理] 旧 API Key 凭证清理完成，共删除 {} 条记录", deleted);

    Ok(deleted)
}
