# AI Agent 指南

本文件为 AI Agent 在此代码库中工作时提供指导。

## 基本规则

1. **始终使用中文输出** - 所有回复、注释、文档都使用中文
2. **文件超过 20 行，分批输出** - 避免一次性输出过长内容
3. **先读后写** - 修改文件前必须先读取现有内容

## AGENTS.md 维护原则

1. **根 AGENTS.md 只放仓库级规则** - 保留全局约束、高频命令、文档索引，避免塞入过长操作手册
2. **长流程拆到独立文档** - 像 Playwright E2E、内容创作工作流这类步骤型说明，放到 `docs/aiprompts/`，根 AGENTS 只保留入口
3. **谨慎新增子目录 AGENTS.md** - 仅当某个目录树存在长期稳定、只对该子树生效的规则时才新增；临时排障说明不要新增 AGENTS
4. **优先索引化而不是堆叠说明** - 根 AGENTS 更适合作为目录与约定入口，详细上下文交给专门文档

## 详细文档

模块级详细文档位于 `docs/aiprompts/`：

| 文档 | 说明 |
|------|------|
| [overview.md](docs/aiprompts/overview.md) | 项目架构概览 |
| [providers.md](docs/aiprompts/providers.md) | Provider 系统 |
| [credential-pool.md](docs/aiprompts/credential-pool.md) | 凭证池管理 |
| [converter.md](docs/aiprompts/converter.md) | 协议转换 |
| [server.md](docs/aiprompts/server.md) | HTTP 服务器 |
| [components.md](docs/aiprompts/components.md) | 组件系统 |
| [hooks.md](docs/aiprompts/hooks.md) | React Hooks |
| [services.md](docs/aiprompts/services.md) | 业务服务 |
| [commands.md](docs/aiprompts/commands.md) | Tauri 命令 |
| [mcp.md](docs/aiprompts/mcp.md) | MCP 服务器 |
| [database.md](docs/aiprompts/database.md) | 数据库层 |
| [terminal.md](docs/aiprompts/terminal.md) | 内置终端 |
| [plugins.md](docs/aiprompts/plugins.md) | 插件系统 |
| [lib.md](docs/aiprompts/lib.md) | 工具库 |
| [workspace.md](docs/aiprompts/workspace.md) | Workspace 设计文档 |
| [content-creator.md](docs/aiprompts/content-creator.md) | 内容创作系统 |
| [aster-integration.md](docs/aiprompts/aster-integration.md) | Aster 集成方案 |
| [playwright-e2e.md](docs/aiprompts/playwright-e2e.md) | Playwright MCP 续测与 E2E 指南 |

## 构建命令

```bash
# 构建 Tauri 应用
cd src-tauri && cargo build

# 构建前端
npm run build

# 开发模式
npm run tauri:dev

# 浏览器 + DevBridge + Playwright MCP 调试模式
npm run tauri:dev:headless
```

## 测试命令

```bash
# 运行 Rust 测试
cd src-tauri && cargo test

# 运行前端测试
npm test

# 运行针对浏览器桥接的前端测试
npm test -- src/lib/dev-bridge/safeInvoke.test.ts src/lib/tauri-mock/core.test.ts
```

## 代码检查

```bash
# Rust 代码检查
cd src-tauri && cargo clippy

# 前端代码检查
npm run lint
```

## Playwright E2E 入口

- 需要继续浏览器 E2E、复用现有 Playwright MCP 会话、排查 DevBridge/console 错误时，先读 `docs/aiprompts/playwright-e2e.md`
- 如果只是仓库级规则，不要继续往本文件堆叠步骤说明

## 项目架构

### 技术栈
- 前端：React + TypeScript + Vite + TailwindCSS
- 后端：Rust + Tauri
- 数据库：SQLite (rusqlite)

### 核心模块

1. **Provider 系统** (`src-tauri/src/providers/`)
   - Kiro/CodeWhisperer OAuth 认证
   - Gemini OAuth 认证
   - Qwen OAuth 认证
   - Antigravity OAuth 认证
   - OpenAI/Claude API Key 认证

2. **凭证池管理** (`src-tauri/src/services/provider_pool_service.rs`)
   - 多凭证轮询负载均衡
   - 健康检查机制
   - Token 自动刷新

3. **API 服务器** (`src-tauri/src/server.rs`)
   - OpenAI 兼容 API 端点
   - Claude 兼容 API 端点
   - 流式响应支持

4. **协议转换** (`src-tauri/src/converter/`)
   - OpenAI ↔ CodeWhisperer 转换
   - OpenAI ↔ Claude 转换

### 凭证管理策略（方案 B）

Kiro 凭证采用完全独立的副本策略：
- 上传凭证时，自动合并 `clientIdHash` 文件中的 `client_id`/`client_secret` 到副本
- 每个副本文件完全独立，支持多账号场景
- 刷新 Token 时只使用副本文件中的凭证，不依赖原始文件

## 开发指南

### 添加新 Provider

1. 在 `src-tauri/src/providers/` 创建新的 provider 模块
2. 实现凭证加载、Token 刷新、API 调用方法
3. 在 `CredentialData` 枚举中添加新类型
4. 在 `ProviderPoolService` 中添加健康检查逻辑

### 修改凭证管理

- 凭证文件存储在 `~/Library/Application Support/proxycast/credentials/`
- 数据库存储凭证元数据和状态
- Token 缓存在数据库中，避免频繁读取文件

### 调试技巧

- 日志输出使用 `tracing` 宏
- API 请求调试文件保存在 `~/.proxycast/logs/`
- 使用 `debug_kiro_credentials` 命令调试凭证加载

## 文档维护

文档维护规范详见 `.kiro/steering/doc-maintenance.md`（Kiro 自动加载）。
