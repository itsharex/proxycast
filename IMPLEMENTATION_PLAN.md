# OAuth 插件系统删除实施计划

## 目标说明

**删除内容**：OAuth 插件管理系统（"OAuth 插件" 标签页及其相关功能）
**保留内容**：5 个内置 OAuth 提供者的凭证管理功能（Kiro, Gemini, Antigravity, Codex, Claude）

## 架构确认

### ✅ 保留 - 内置 OAuth 提供者
这些提供者有完整的内置实现，不依赖插件系统：

1. **Kiro** - `src-tauri/src/providers/kiro.rs`
2. **Gemini** - `src-tauri/src/providers/gemini.rs`
3. **Antigravity** - `src-tauri/src/providers/antigravity.rs`
4. **Codex** - `src-tauri/src/providers/codex.rs`
5. **Claude** - `src-tauri/src/providers/claude_oauth.rs`

### ❌ 删除 - OAuth 插件系统
这些是可扩展的插件管理系统，允许安装/卸载第三方 OAuth 提供者插件：

- 插件加载器：`oauth_plugin_loader.rs`
- 插件注册表：`credential/registry.rs` 中的插件部分
- 插件管理命令：`oauth_plugin_cmd.rs`
- 插件 UI 组件：`OAuthPluginTab.tsx`, `OAuthPluginContainer.tsx`
- 插件 API：`oauthPlugin.ts`, `useOAuthPlugins.ts`

---

## Stage 1: 删除前端插件 UI 组件
**Goal**: 删除 "OAuth 插件" 标签页及相关 UI 组件
**Success Criteria**: 前端编译无错误，UI 中不再显示 "OAuth 插件" 标签
**Tests**: 应用启动正常，OAuth 凭证管理功能正常
**Status**: ✅ Complete

### 需要删除的文件：
- `src/components/provider-pool/OAuthPluginTab.tsx` - OAuth 插件标签页
- `src/components/plugins/OAuthPluginContainer.tsx` - 插件容器组件
- `src/components/provider-pool/credential-forms/ClaudeOAuthForm.tsx` - 如果仅用于插件
- `src/components/provider-pool/credential-forms/OAuthUrlDisplay.tsx` - 如果仅用于插件

### 需要修改的文件：
- `src/components/provider-pool/ProviderPoolPage.tsx` 或类似的父组件
  - 移除 "OAuth 插件" 标签页的引用
  - 保留 "OAuth 凭证" 标签页

---

## Stage 2: 删除前端插件 API 和 Hooks
**Goal**: 删除插件管理相关的前端 API 调用和 Hooks
**Success Criteria**: 前端编译无错误，无未使用的导入
**Tests**: 其他 API 调用正常工作
**Status**: Not Started

### 需要删除的文件：
- `src/lib/api/oauthPlugin.ts` - 插件管理 API
- `src/hooks/useOAuthPlugins.ts` - 插件管理 Hook
- `src/hooks/useOAuthCredentials.ts` - 如果仅用于插件凭证

### 需要保留的文件：
- `src/lib/api/credentials.ts` - 保留内置 OAuth 凭证 API
- 其他与 Kiro/Gemini/Antigravity/Codex/Claude 凭证管理相关的 API

---

## Stage 3: 删除后端插件命令
**Goal**: 删除插件管理相关的 Tauri 命令
**Success Criteria**: 后端编译无错误
**Tests**: 内置 OAuth 命令正常工作
**Status**: Not Started

### 需要删除的文件：
- `src-tauri/src/commands/oauth_plugin_cmd.rs` - 完整删除

### 需要保留的文件：
- `src-tauri/src/commands/oauth_cmd.rs` - 保留（内置 OAuth 凭证命令）

### 需要修改的文件：
- `src-tauri/src/commands/mod.rs`
  - 移除 `oauth_plugin_cmd` 模块引用
  - 移除所有插件相关命令的注册：
    - `init_oauth_plugin_system`
    - `list_oauth_plugins`
    - `get_oauth_plugin`
    - `enable_oauth_plugin`
    - `disable_oauth_plugin`
    - `install_oauth_plugin`
    - `uninstall_oauth_plugin`
    - `check_oauth_plugin_updates`
    - `update_oauth_plugin`
    - `reload_oauth_plugins`
    - `get_oauth_plugin_config`
    - `update_oauth_plugin_config`
    - `scan_oauth_plugin_directory`
    - `plugin_credential_*` 系列命令
    - `plugin_database_*` 系列命令
    - `plugin_http_request`
    - `plugin_crypto_*` 系列命令
    - `plugin_notification`
    - `plugin_storage_*` 系列命令
    - `plugin_config_*` 系列命令
    - `read_plugin_ui_file`

- `src-tauri/src/main.rs` 或 `src-tauri/src/lib.rs`
  - 移除 `OAuthPluginManagerState` 的初始化和注册

---

## Stage 4: 删除插件加载器和注册表
**Goal**: 删除插件系统的核心组件
**Success Criteria**: 后端编译无错误
**Tests**: 内置 OAuth 提供者正常工作
**Status**: Not Started

### 需要删除的文件：
- `src-tauri/src/credential/oauth_plugin_loader.rs` - 插件加载器
- `src-tauri/src/credential/plugin.rs` - 插件接口定义
- `src-tauri/src/credential/unified.rs` - 统一凭证接口（如果仅用于插件）
- `src-tauri/src/credential/sdk.rs` - 插件 SDK

### 需要修改的文件：
- `src-tauri/src/credential/mod.rs`
  - 移除插件相关模块的导出
  - 移除 `get_global_registry`, `init_global_registry` 等插件注册表函数

- `src-tauri/src/credential/registry.rs`
  - 移除插件注册表相关代码
  - 保留基础凭证注册功能（如果有）

---

## Stage 5: 清理数据库层
**Goal**: 删除插件相关的数据库表和 DAO
**Success Criteria**: 数据库迁移成功，后端编译无错误
**Tests**: 内置 OAuth 凭证的数据库操作正常
**Status**: Not Started

### 需要删除的表：
从 `src-tauri/src/database/schema.rs` 中删除：
- `credential_provider_plugins` - 插件元数据表
- `plugin_credentials` - 插件凭证表
- `plugin_storage` - 插件存储表
- `plugin_event_logs` - 插件事件日志表

### 需要保留的表：
- `provider_pool_credentials` - 内置 OAuth 凭证表（保留）
- 其他与内置提供者相关的表

### 需要删除的文件：
- `src-tauri/src/database/dao/plugin_credential.rs` - 插件凭证 DAO

### 需要修改的文件：
- `src-tauri/src/database/dao/mod.rs`
  - 移除 `plugin_credential` 模块引用

- `src-tauri/src/database/mod.rs`
  - 移除插件相关的数据库初始化代码

### 数据库迁移：
创建迁移脚本删除插件相关表：
```sql
DROP TABLE IF EXISTS plugin_event_logs;
DROP TABLE IF EXISTS plugin_storage;
DROP TABLE IF EXISTS plugin_credentials;
DROP TABLE IF EXISTS credential_provider_plugins;
```

---

## Stage 6: 清理类型定义
**Goal**: 删除插件相关的类型定义
**Success Criteria**: 代码编译无错误
**Tests**: 应用正常运行
**Status**: Not Started

### 需要修改的文件：
- `src-tauri/crates/core/src/models/provider_type.rs`
  - 检查是否有插件特定的 provider 类型，如有则移除
  - 保留 Kiro, Gemini, Antigravity, Codex, Claude 的类型定义

- `src-tauri/crates/core/src/models/provider_pool_model.rs`
  - 移除插件凭证相关的类型定义
  - 保留内置 OAuth 凭证类型

- `src/lib/plugin-sdk/types.ts`
  - 如果整个目录仅用于插件 SDK，则删除整个目录
  - 否则移除插件相关的类型定义

---

## Stage 7: 清理依赖项
**Goal**: 移除插件系统相关的依赖包
**Success Criteria**: 依赖安装成功，无冗余依赖
**Tests**: 应用正常启动和运行
**Status**: Not Started

### 前端 (package.json):
- 检查是否有插件系统专用的依赖，如有则移除
- 保留 OAuth 凭证管理所需的依赖

### 后端 (Cargo.toml):
- 检查 `src-tauri/Cargo.toml` 中是否有插件系统专用的 crate
- 可能需要移除的依赖：
  - 动态加载相关的 crate（如 `libloading`, `dlopen` 等）
  - 插件沙箱相关的 crate

---

## Stage 8: 清理文档和脚本
**Goal**: 删除插件相关的文档和脚本
**Success Criteria**: 文档目录清理完成
**Tests**: 无
**Status**: Not Started

### 需要检查和修改的文档：
- `docs/plugins/` 目录 - 如果整个目录仅用于插件文档，则删除
- `docs/content/03.providers/1.overview.md` - 移除插件系统相关的说明
- `docs/content/02.user-guide/4.configuration-example.md` - 移除插件配置示例
- `README.md` - 移除插件系统相关的说明

### 需要保留的文档：
- 关于 Kiro, Gemini, Antigravity, Codex, Claude 的 OAuth 配置文档

### 需要删除的脚本：
- 检查 `scripts/` 目录中是否有插件相关的脚本

---

## Stage 9: 最终验证和清理
**Goal**: 确保所有插件系统代码已删除，内置 OAuth 功能正常
**Success Criteria**: 所有验证项通过
**Tests**: 完整的功能测试
**Status**: Not Started

### 验证清单：
- [ ] 前端应用正常启动
- [ ] 后端应用正常启动
- [ ] 无编译错误或警告
- [ ] UI 中不再显示 "OAuth 插件" 标签页
- [ ] "OAuth 凭证" 标签页正常显示
- [ ] Kiro OAuth 凭证管理正常
- [ ] Gemini OAuth 凭证管理正常
- [ ] Antigravity OAuth 凭证管理正常
- [ ] Codex OAuth 凭证管理正常
- [ ] Claude OAuth 凭证管理正常
- [ ] 数据库迁移成功
- [ ] 代码库中不再有插件系统相关的引用

### 代码搜索验证：
使用以下关键词搜索，确保没有遗漏：
- `oauth_plugin`
- `OAuthPlugin`
- `plugin_credential`
- `PluginCredential`
- `credential_provider_plugins`
- `plugin_storage`
- `plugin_event_logs`
- `OAuthPluginLoader`
- `PluginRegistry`

---

## 注意事项

1. **备份**: 在开始之前，建议创建数据库备份和代码分支
2. **依赖检查**: 仔细检查是否有非插件代码依赖插件模块
3. **测试**: 每个阶段完成后进行编译测试
4. **提交**: 每个阶段完成后创建一个 commit
5. **渐进式**: 按阶段顺序执行，不要跳跃

## 关键区别

### 插件系统（删除）
- 可扩展架构，支持第三方插件
- 插件加载器和注册表
- 插件安装/卸载功能
- 插件 SDK 和权限系统
- 动态加载外部代码

### 内置 OAuth（保留）
- 硬编码的 5 个提供者
- 直接在代码中实现
- 不支持动态加载
- 凭证管理功能
- OAuth 流程实现
