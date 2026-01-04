# 数据库模块

本模块负责 SQLite 数据库的初始化、表结构定义和数据迁移。

## 文件索引

| 文件 | 说明 |
|------|------|
| `mod.rs` | 模块入口，数据库初始化 |
| `schema.rs` | 表结构定义和创建 |
| `migration.rs` | 数据迁移逻辑 |
| `system_providers.rs` | 系统预设 Provider 配置 |
| `dao/` | 数据访问对象层 |

## 数据库表

### 核心表

- `api_key_providers` - API Key Provider 配置
- `api_keys` - API Key 条目（已迁移到 provider_pool_credentials）
- `provider_pool_credentials` - 凭证池（统一管理所有凭证）
- `providers` - Provider 配置
- `settings` - 应用设置

### 功能表

- `mcp_servers` - MCP 服务器配置
- `prompts` - 提示词模板
- `skills` - 技能配置
- `skill_repos` - 技能仓库
- `installed_plugins` - 已安装插件

## 数据迁移

### API Keys 迁移

`migrate_api_keys_to_pool()` 函数将 `api_keys` 表中的数据迁移到 `provider_pool_credentials` 表：

- 根据 provider_type 自动转换为对应的 CredentialData 类型
- 保留使用统计和错误计数
- 标记来源为 `imported`
- 迁移完成后设置 `migrated_api_keys_to_pool` 标记，避免重复迁移

## 使用示例

```rust
use crate::database::{init_database, DbConnection};

// 初始化数据库
let db: DbConnection = init_database()?;

// 使用 DAO 操作数据
let conn = db.lock().unwrap();
let providers = ApiKeyProviderDao::get_all_providers(&conn)?;
```
