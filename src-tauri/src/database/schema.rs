use rusqlite::Connection;

pub fn create_tables(conn: &Connection) -> Result<(), rusqlite::Error> {
    // Providers 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS providers (
            id TEXT NOT NULL,
            app_type TEXT NOT NULL,
            name TEXT NOT NULL,
            settings_config TEXT NOT NULL,
            category TEXT,
            icon TEXT,
            icon_color TEXT,
            notes TEXT,
            created_at INTEGER,
            sort_index INTEGER,
            is_current INTEGER DEFAULT 0,
            PRIMARY KEY (id, app_type)
        )",
        [],
    )?;

    // MCP 服务器表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS mcp_servers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            server_config TEXT NOT NULL,
            description TEXT,
            enabled_proxycast INTEGER DEFAULT 0,
            enabled_claude INTEGER DEFAULT 0,
            enabled_codex INTEGER DEFAULT 0,
            enabled_gemini INTEGER DEFAULT 0,
            created_at INTEGER
        )",
        [],
    )?;

    // Prompts 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS prompts (
            id TEXT NOT NULL,
            app_type TEXT NOT NULL,
            name TEXT NOT NULL,
            content TEXT NOT NULL,
            description TEXT,
            enabled INTEGER DEFAULT 0,
            created_at INTEGER,
            updated_at INTEGER,
            PRIMARY KEY (id, app_type)
        )",
        [],
    )?;

    // Migration: rename is_current to enabled if old column exists
    let _ = conn.execute(
        "ALTER TABLE prompts RENAME COLUMN is_current TO enabled",
        [],
    );

    // Migration: add updated_at column if it doesn't exist
    let _ = conn.execute("ALTER TABLE prompts ADD COLUMN updated_at INTEGER", []);

    // 设置表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;

    // Skills 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS skills (
            directory TEXT NOT NULL,
            app_type TEXT NOT NULL,
            installed INTEGER NOT NULL DEFAULT 0,
            installed_at INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (directory, app_type)
        )",
        [],
    )?;

    // Skill Repos 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS skill_repos (
            owner TEXT NOT NULL,
            name TEXT NOT NULL,
            branch TEXT NOT NULL DEFAULT 'main',
            enabled INTEGER NOT NULL DEFAULT 1,
            PRIMARY KEY (owner, name)
        )",
        [],
    )?;

    // Provider Pool 凭证表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS provider_pool_credentials (
            uuid TEXT PRIMARY KEY,
            provider_type TEXT NOT NULL,
            credential_data TEXT NOT NULL,
            name TEXT,
            is_healthy INTEGER DEFAULT 1,
            is_disabled INTEGER DEFAULT 0,
            check_health INTEGER DEFAULT 1,
            check_model_name TEXT,
            not_supported_models TEXT,
            usage_count INTEGER DEFAULT 0,
            error_count INTEGER DEFAULT 0,
            last_used INTEGER,
            last_error_time INTEGER,
            last_error_message TEXT,
            last_health_check_time INTEGER,
            last_health_check_model TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    )?;

    // 创建 provider_type 索引
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_provider_pool_type ON provider_pool_credentials(provider_type)",
        [],
    )?;

    // Migration: 添加 Token 缓存字段
    let _ = conn.execute(
        "ALTER TABLE provider_pool_credentials ADD COLUMN cached_access_token TEXT",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE provider_pool_credentials ADD COLUMN cached_refresh_token TEXT",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE provider_pool_credentials ADD COLUMN token_expiry_time TEXT",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE provider_pool_credentials ADD COLUMN last_refresh_time TEXT",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE provider_pool_credentials ADD COLUMN refresh_error_count INTEGER DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE provider_pool_credentials ADD COLUMN last_refresh_error TEXT",
        [],
    );

    // Migration: 添加凭证来源字段
    let _ = conn.execute(
        "ALTER TABLE provider_pool_credentials ADD COLUMN source TEXT DEFAULT 'manual'",
        [],
    );

    // Migration: 添加代理URL字段 - 使用重建表结构的方式
    migrate_add_proxy_url_column(conn)?;

    // 已安装插件表
    // _需求: 1.2, 1.3_
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
    )?;

    // 创建 installed_plugins 索引
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_installed_plugins_name ON installed_plugins(name)",
        [],
    )?;

    Ok(())
}

/// 迁移：添加proxy_url列到provider_pool_credentials表
/// 使用重建表结构的方式确保数据完整性
fn migrate_add_proxy_url_column(conn: &Connection) -> Result<(), rusqlite::Error> {
    // 检查是否已经存在proxy_url列
    let mut stmt = conn.prepare("PRAGMA table_info(provider_pool_credentials)")?;
    let column_info: Vec<String> = stmt
        .query_map([], |row| {
            let column_name: String = row.get(1)?;
            Ok(column_name)
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // 如果proxy_url列已存在，跳过迁移
    if column_info.contains(&"proxy_url".to_string()) {
        return Ok(());
    }

    tracing::info!("开始迁移：添加proxy_url列到provider_pool_credentials表");

    // 开始事务
    conn.execute("BEGIN TRANSACTION", [])?;

    let migration_result = (|| -> Result<(), rusqlite::Error> {
        // 1. 备份现有数据
        conn.execute(
            "CREATE TABLE provider_pool_credentials_backup AS
             SELECT * FROM provider_pool_credentials",
            [],
        )?;

        // 2. 删除原表
        conn.execute("DROP TABLE provider_pool_credentials", [])?;

        // 3. 重建表结构（包含proxy_url列）
        conn.execute(
            "CREATE TABLE provider_pool_credentials (
                uuid TEXT PRIMARY KEY,
                provider_type TEXT NOT NULL,
                credential_data TEXT NOT NULL,
                name TEXT,
                is_healthy INTEGER DEFAULT 1,
                is_disabled INTEGER DEFAULT 0,
                check_health INTEGER DEFAULT 1,
                check_model_name TEXT,
                not_supported_models TEXT,
                usage_count INTEGER DEFAULT 0,
                error_count INTEGER DEFAULT 0,
                last_used INTEGER,
                last_error_time INTEGER,
                last_error_message TEXT,
                last_health_check_time INTEGER,
                last_health_check_model TEXT,
                proxy_url TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                cached_access_token TEXT,
                cached_refresh_token TEXT,
                token_expiry_time TEXT,
                last_refresh_time TEXT,
                refresh_error_count INTEGER DEFAULT 0,
                last_refresh_error TEXT,
                source TEXT DEFAULT 'manual'
            )",
            [],
        )?;

        // 4. 恢复数据（proxy_url默认为NULL）
        conn.execute(
            "INSERT INTO provider_pool_credentials (
                uuid, provider_type, credential_data, name, is_healthy, is_disabled,
                check_health, check_model_name, not_supported_models, usage_count,
                error_count, last_used, last_error_time, last_error_message,
                last_health_check_time, last_health_check_model, proxy_url,
                created_at, updated_at, cached_access_token, cached_refresh_token,
                token_expiry_time, last_refresh_time, refresh_error_count,
                last_refresh_error, source
            ) SELECT
                uuid, provider_type, credential_data, name, is_healthy, is_disabled,
                check_health, check_model_name, not_supported_models, usage_count,
                error_count, last_used, last_error_time, last_error_message,
                last_health_check_time, last_health_check_model, NULL as proxy_url,
                created_at, updated_at, cached_access_token, cached_refresh_token,
                token_expiry_time, last_refresh_time, refresh_error_count,
                last_refresh_error, source
            FROM provider_pool_credentials_backup",
            [],
        )?;

        // 5. 重建索引
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_provider_pool_type ON provider_pool_credentials(provider_type)",
            [],
        )?;

        // 6. 删除备份表
        conn.execute("DROP TABLE provider_pool_credentials_backup", [])?;

        Ok(())
    })();

    match migration_result {
        Ok(()) => {
            conn.execute("COMMIT", [])?;
            tracing::info!("proxy_url列迁移成功完成");
            Ok(())
        }
        Err(e) => {
            conn.execute("ROLLBACK", [])?;
            tracing::error!("proxy_url列迁移失败，已回滚: {}", e);
            Err(e)
        }
    }
}
