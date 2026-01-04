# ProxyCast 凭证提供商插件化架构 PRD

> 版本: 1.0.0
> 日期: 2026-01-03
> 状态: Draft

---

## 一、背景与目标

### 1.1 背景

当前 ProxyCast 支持 11 种 Provider 类型，但存在以下问题：

1. **硬编码耦合**：Provider 类型通过 `ProviderType` 枚举硬编码，新增 Provider 需要修改核心代码
2. **协议转换分散**：`converter/` 和 `translator/` 两套转换逻辑并存
3. **风控逻辑耦合**：Kiro、Antigravity 等的特殊风控逻辑散落在各处
4. **难以独立更新**：某个 Provider（如 Kiro）因风控变化需要更新时，影响整体

### 1.2 目标

将 **OAuth 凭证系统**重构为插件化架构，**API Key 系统保持现有设计不变**：

1. **OAuth 凭证插件化**：Kiro、Codex、Gemini OAuth、Qwen、Antigravity、iFlow 等各自独立
2. **API Key 系统不变**：现有 `api_key_providers` + `api_keys` 表结构保持不变，60+ 系统预设继续使用
3. **独立更新**：某个 OAuth Provider 风控变化时，只需更新对应插件
4. **复用现有插件系统**：基于现有的 `Plugin` trait 扩展

### 1.3 插件化范围

| 系统 | 是否插件化 | 原因 |
|------|-----------|------|
| **OAuth 凭证** | ✅ 是 | Kiro 风控复杂、Token 刷新、不同凭证格式 |
| **API Key** | ❌ 否 | 统一结构、配置简单、用户可自定义、已有 60+ 预设 |

---

## 二、现有架构分析

### 2.1 凭证管理模块

```
src-tauri/src/credential/
├── types.rs      # Credential, CredentialData, CredentialStatus
├── pool.rs       # CredentialPool (DashMap 实现)
├── balancer.rs   # LoadBalancer (轮询/最少使用/随机)
├── health.rs     # HealthChecker (3次失败标记不健康)
├── quota.rs      # QuotaManager (配额超限检测)
└── sync.rs       # 数据库同步
```

**现有凭证数据结构**：
```rust
pub enum CredentialData {
    OAuth { access_token, refresh_token, expires_at },
    ApiKey { key, base_url },
}
```

### 2.2 Provider 类型（现有问题：硬编码枚举）

**问题**：现有设计使用硬编码枚举，新增 Provider 必须修改核心代码

```rust
// ❌ 现有设计：硬编码枚举
pub enum ProviderType {
    Kiro, Gemini, Qwen, OpenAI, Claude, Antigravity,
    Vertex, GeminiApiKey, Codex, ClaudeOAuth, IFlow,
    // 新增 Provider？必须修改这个枚举！
}

pub enum CredentialData {
    KiroOAuth { ... }, GeminiOAuth { ... }, ...
    // 新增凭证类型？必须修改这个枚举！
}
```

**目标**：删除硬编码枚举，改为动态注册

```rust
// ✅ 目标设计：动态注册
// 不再有 ProviderType 枚举
// 不再有 CredentialData 枚举
// 新增 Provider 只需实现 trait 并注册
```

### 2.3 现有凭证类型（需要迁移到插件）

| Provider | 凭证类型 | 配置字段 |
|----------|---------|---------|
| Kiro | OAuth | creds_file_path |
| Gemini | OAuth | creds_file_path, project_id |
| Qwen | OAuth | creds_file_path |
| Antigravity | OAuth | creds_file_path, project_id |
| OpenAI | API Key | api_key, base_url |
| Claude | API Key | api_key, base_url |
| Vertex | API Key | api_key, base_url, model_aliases |
| GeminiApiKey | API Key | api_key, base_url, excluded_models |
| Codex | OAuth | creds_file_path, api_base_url |
| ClaudeOAuth | OAuth | creds_file_path |
| IFlow | OAuth/Cookie | creds_file_path |

**迁移后**：每个 Provider 插件自己定义凭证配置 Schema

### 2.4 协议转换

**现有协议类型**：

| 协议 | 用途 |
|------|------|
| OpenAI | 标准 Chat Completions API |
| Anthropic | Claude Messages API（Claude Code 使用此协议）|
| CodeWhisperer | AWS Kiro IDE（底层是 Claude）|
| Gemini | Google Gemini API |
| Antigravity | Google 内部（支持 Claude 和 Gemini 模型）|

#### 2.4.1 双向转换架构

ProxyCast 作为 **API 代理**，核心职责是协议转换：

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        协议转换流程                                      │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  客户端请求 (Anthropic 协议 /v1/messages)                                │
│       │                                                                  │
│       ▼                                                                  │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                    输入转换层 (Request Translator)                │    │
│  │                                                                   │    │
│  │  Anthropic → CodeWhisperer   (translator/kiro/anthropic/request) │    │
│  │  Anthropic → Antigravity     (converter/anthropic_to_antigravity)│    │
│  │  Anthropic → Anthropic       (Claude OAuth, 直通)                │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│       │                                                                  │
│       ▼                                                                  │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                    Provider 后端调用                              │    │
│  │                                                                   │    │
│  │  Kiro: AWS CodeWhisperer API (返回 AWS Event Stream)             │    │
│  │  Antigravity: Gemini CLI API (返回 Antigravity 响应)             │    │
│  │  Claude OAuth: Anthropic API (返回 Anthropic SSE, 直通)          │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│       │                                                                  │
│       ▼                                                                  │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                    输出转换层 (Response Translator)               │    │
│  │                                                                   │    │
│  │  AWS Event Stream → Anthropic SSE                                │    │
│  │       (translator/kiro/anthropic/response.rs)                    │    │
│  │                                                                   │    │
│  │  Antigravity → Anthropic SSE (claude-* 模型)                     │    │
│  │  Antigravity → Gemini 协议   (gemini-* 模型)                     │    │
│  │                                                                   │    │
│  │  Anthropic SSE → Anthropic SSE (Claude OAuth, 直通)              │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│       │                                                                  │
│       ▼                                                                  │
│  Anthropic 协议响应 (给 Claude Code 等客户端)                            │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

#### 2.4.2 转换规则（按底层模型决定）

**核心原则**：输出协议由 **Provider 后端的底层模型** 决定，而非客户端输入格式

| Provider | 底层模型 | 后端协议 | 输出协议 |
|----------|---------|---------|---------|
| Kiro | Claude | CodeWhisperer (AWS Stream) | **Anthropic** (Claude API) |
| Antigravity (claude-*) | Claude | Antigravity | **Anthropic** (Claude API) |
| Antigravity (gemini-*) | Gemini | Antigravity | **Gemini** |
| Claude OAuth | Claude | Anthropic | **Anthropic** (直通) |
| Codex | GPT | OpenAI | **OpenAI** (直通) |
| Qwen | 通义千问 | OpenAI 兼容 | **OpenAI** |
| iFlow | - | OpenAI 兼容 | **OpenAI** |
| Gemini OAuth | Gemini | Gemini | **Gemini** |

**转换链路示例**：

```
┌─────────────────────────────────────────────────────────────────────┐
│  Kiro Provider（底层 Claude）                                        │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Anthropic 协议请求 (/v1/messages)                                   │
│       │                                                              │
│       ▼                                                              │
│  输入转换: Anthropic → CodeWhisperer 请求                            │
│       │         (translator/kiro/anthropic/request.rs)               │
│       ▼                                                              │
│  Kiro 后端调用 (返回 AWS Event Stream)                               │
│       │                                                              │
│       ▼                                                              │
│  输出转换: AWS Stream → Anthropic SSE                                │
│       │         (translator/kiro/anthropic/response.rs)              │
│       ▼                                                              │
│  Anthropic 协议响应 (给 Claude Code 等客户端)                         │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│  Antigravity Provider（动态协议）                                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Anthropic 协议请求 (model: claude-opus-4.5)                         │
│       │                                                              │
│       ▼                                                              │
│  输入转换: Anthropic → Antigravity 请求                              │
│       │                                                              │
│       ▼                                                              │
│  匹配规则: claude-* → 底层 Claude                                    │
│       │                                                              │
│       ▼                                                              │
│  输出转换: Antigravity → Anthropic SSE                               │
│                                                                      │
│  ─────────────────────────────────────────────────────────────────  │
│                                                                      │
│  Anthropic 协议请求 (model: gemini-2.0-flash)                        │
│       │                                                              │
│       ▼                                                              │
│  输入转换: Anthropic → Antigravity 请求                              │
│       │                                                              │
│       ▼                                                              │
│  匹配规则: gemini-* → 底层 Gemini                                    │
│       │                                                              │
│       ▼                                                              │
│  输出转换: Antigravity → Gemini 协议响应                             │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

**常见使用场景**：

| 客户端 | Provider | 底层模型 | 输出协议 |
|--------|----------|---------|---------|
| **ProxyCast 内置 Agent** | Kiro | Claude | Anthropic (Claude API) |
| **ProxyCast 内置 Agent** | Antigravity (claude-*) | Claude | Anthropic (Claude API) |
| **ProxyCast 内置 Agent** | Claude OAuth | Claude | Anthropic (Claude API) |
| Claude Code | Kiro | Claude | Anthropic (Claude API) |
| Claude Code | Antigravity (claude-*) | Claude | Anthropic (Claude API) |
| Claude Code | Claude OAuth | Claude | Anthropic (Claude API) |
| Gemini 客户端 | Antigravity (gemini-*) | Gemini | Gemini |
| OpenAI 兼容客户端 | Codex | GPT | OpenAI |
| OpenAI 兼容客户端 | Qwen/iFlow | 通义千问等 | OpenAI |

#### 2.4.3 ProxyCast API Server 架构

ProxyCast 提供 **API Server**，统一服务内部和外部客户端：

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    ProxyCast API Server                                  │
│                                                                          │
│  端口: 8999 | 支持多种协议格式 | API Key 认证                            │
│  地址: 127.0.0.1:8999 (本地) | 198.18.0.1:8999 (局域网)                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                       客户端接入                                  │    │
│  │                                                                   │    │
│  │  内部客户端:                                                      │    │
│  │  ├── ProxyCast 内置 AI Agent（技能、话题、工具调用）              │    │
│  │                                                                   │    │
│  │  外部客户端:                                                      │    │
│  │  ├── Claude Code (Anthropic 协议)                                │    │
│  │  ├── Cursor (OpenAI 协议)                                        │    │
│  │  ├── Continue (OpenAI 协议)                                      │    │
│  │  └── 任意支持 OpenAI/Anthropic API 的应用                        │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│       │                                                                  │
│       │ OpenAI / Anthropic / Gemini 等协议请求                            │
│       ▼                                                                  │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                    服务器控制                                     │    │
│  │                                                                   │    │
│  │  默认 Provider: [Kiro] [Gemini] [Qwen] [Antigravity] [OpenAI] [Claude]│
│  │  当前可用凭证: P2 (绿色表示健康)                                  │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│       │                                                                  │
│       ▼                                                                  │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                    代理层处理                                     │    │
│  │                                                                   │    │
│  │  1. API Key 验证                                                 │    │
│  │  2. Provider 路由（根据默认设置或请求指定）                       │    │
│  │  3. OAuth 凭证获取                                               │    │
│  │  4. 协议转换（Anthropic → CodeWhisperer 等）                     │    │
│  │  5. 后端调用                                                     │    │
│  │  6. 响应转换                                                     │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│       │                                                                  │
│       ▼                                                                  │
│  Kiro / Antigravity / Claude OAuth / Codex 等后端                        │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

**API Server 配置**：

| 配置项 | 说明 |
|--------|------|
| 端口 | 默认 8999，可配置 |
| API Key | 访问密钥（如 Proxycast-key11）|
| 默认 Provider | 用户可选择默认路由的 Provider |

**支持的输入/输出协议**：

| 协议 | 端点 | 说明 |
|------|------|------|
| OpenAI | `/v1/chat/completions` | 标准 OpenAI Chat Completions API |
| Anthropic | `/v1/messages` | Claude Messages API |
| Gemini | `/v1/gemini/*` | Google Gemini API |
| OpenAI Responses | `/v1/responses` | OpenAI Responses API (流式) |
| 其他兼容协议 | - | 支持扩展更多协议 |

**内置 AI Agent 特点**：

| 功能 | 说明 |
|------|------|
| 模型选择 | 用户可选择任意已配置的模型（如 claude-opus-4-5-20251101）|
| 技能系统 | 支持自定义 Skills，扩展 Agent 能力 |
| 工具调用 | 支持 MCP、联网搜索、文件附件等 |
| 对话管理 | 话题列表、历史记录 |

**统一调用流程**（内部/外部客户端一致）：

```
客户端发送请求 (内置 Agent 或外部 Claude Code/Cursor)
    │
    ▼
API Server 接收 (端口 8999)
    │
    ├── 验证 API Key
    ├── 识别协议格式 (OpenAI / Anthropic / Gemini / OpenAI Responses 等)
    │
    ▼
根据默认 Provider 或请求参数路由
    │
    ▼
OAuth Provider 处理
    ├── 获取凭证
    ├── 协议转换 (如 Anthropic → CodeWhisperer)
    └── 调用后端
    │
    ▼
响应转换 (如 AWS Stream → Anthropic SSE)
    │
    ▼
返回给客户端
```

#### 2.4.4 StreamEvent 统一事件

中间表示层，解耦输入输出：

```rust
pub enum StreamEvent {
    MessageStart { id: String, model: String },
    ContentBlockStart { index: u32, block_type: ContentBlockType },
    TextDelta { text: String },
    ToolUseStart { id: String, name: String },
    ToolUseInputDelta { id: String, partial_json: String },
    ToolUseStop { id: String },
    ContentBlockStop { index: u32 },
    MessageStop { stop_reason: StopReason },
    Usage { input_tokens: u32, output_tokens: u32 },
    Error { error_type: String, message: String },
    Ping,
}
```

#### 2.4.5 现有代码位置

| 功能 | 文件路径 |
|------|---------|
| Anthropic → Kiro 请求 | `translator/kiro/anthropic/request.rs` |
| Kiro → Anthropic 响应 | `translator/kiro/anthropic/response.rs` |
| OpenAI → Kiro 请求 | `translator/kiro/openai/request.rs` |
| Kiro → OpenAI 响应 | `translator/kiro/openai/response.rs` |
| OpenAI → Antigravity | `converter/openai_to_antigravity.rs` |
| AWS 流解析 | `stream/parsers/aws_event_stream.rs` |
| Anthropic SSE 生成 | `stream/generators/anthropic_sse.rs` |
| OpenAI SSE 生成 | `stream/generators/openai_sse.rs` |

#### 2.4.6 插件化后的协议转换

**目标**：每个 Provider 插件负责自己的双向转换

```rust
#[async_trait]
pub trait OAuthProviderPlugin: Send + Sync {
    // ... 其他方法 ...

    /// 输入转换：将客户端请求转换为 Provider 特有格式
    async fn transform_request(&self, req: &mut ChatRequest) -> Result<ProviderRequest>;

    /// 输出转换：将 Provider 响应转换为 StreamEvent
    fn parse_response_chunk(&self, chunk: &[u8]) -> Result<Vec<StreamEvent>>;

    /// 目标输出协议（决定使用哪个 SSE Generator）
    fn output_protocol(&self) -> OutputProtocol;  // Anthropic | OpenAI
}

pub enum OutputProtocol {
    Anthropic,  // Claude Code 使用
    OpenAI,     // OpenAI 兼容客户端使用
}
```

### 2.5 现有插件系统

```rust
#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn manifest(&self) -> &PluginManifest;

    async fn init(&mut self, config: &PluginConfig) -> Result<(), PluginError>;
    async fn on_request(&self, ctx: &mut PluginContext, request: &mut Value) -> Result<HookResult, PluginError>;
    async fn on_response(&self, ctx: &mut PluginContext, response: &mut Value) -> Result<HookResult, PluginError>;
    async fn on_error(&self, ctx: &mut PluginContext, error: &str) -> Result<HookResult, PluginError>;
    async fn shutdown(&mut self) -> Result<(), PluginError>;
}
```

### 2.6 处理管道

```
Auth → Injection → Routing → PluginPre → Provider → PluginPost → Telemetry
```

---

## 三、目标架构设计

### 3.1 核心设计原则

1. **插件职责明确**：每个 Provider 插件负责 凭证管理 + 协议转换 + 风控适配
2. **转换成标准协议**：所有插件输出标准协议（Anthropic/OpenAI/Gemini）
3. **认证类型固定**：AuthType 作为枚举，不插件化
4. **复用现有基础设施**：复用 credential/pool.rs, balancer.rs, health.rs

### 3.2 完全动态化架构

**核心原则**：没有任何硬编码枚举，所有类型都通过注册表动态管理

```
┌─────────────────────────────────────────────────────────────────────┐
│                CredentialProviderRegistry（动态）                    │
│                                                                      │
│  新增 Provider？只需实现 trait 并注册，无需修改核心代码              │
│                                                                      │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │   kiro   │ │anthropic │ │  openai  │ │   qwen   │ │  iflow   │  │
│  │ Provider │ │ Provider │ │ Provider │ │ Provider │ │ Provider │  │
│  │          │ │          │ │          │ │          │ │          │  │
│  │ OAuth    │ │ ApiKey   │ │ ApiKey   │ │ OAuth    │ │ OAuth    │  │
│  │ 风控适配 │ │ 原生     │ │ 原生     │ │ 兼容     │ │ Cookie   │  │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘  │
│       │            │            │            │            │         │
│  目标协议:     目标协议:    目标协议:    目标协议:    目标协议:      │
│  "anthropic"  "anthropic"   "openai"     "qwen"      "openai"      │
│       │            │            │            │            │         │
│       └────────────┴────────────┴────────────┴────────────┘         │
│                                    │                                │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐               │
│  │  gemini  │ │  vertex  │ │  codex   │ │antigravity│  ...更多     │
│  │ Provider │ │ Provider │ │ Provider │ │ Provider │               │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘               │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│               StandardProtocolRegistry（动态）                       │
│                                                                      │
│  新增协议？只需实现 ProtocolHandler trait 并注册                     │
│                                                                      │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │anthropic │ │  openai  │ │  gemini  │ │   qwen   │ │  doubao  │  │
│  │ Handler  │ │ Handler  │ │ Handler  │ │ Handler  │ │ Handler  │  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘  │
│                                                                      │
│  ┌──────────┐ ┌──────────┐                                          │
│  │  wenxin  │ │   ...    │  ← 可继续扩展中国厂商协议                 │
│  │ Handler  │ │          │                                          │
│  └──────────┘ └──────────┘                                          │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        输出适配层                                    │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐        │
│  │ Anthropic API  │  │   OpenAI API   │  │ Claude Code    │        │
│  │    Output      │  │    Output      │  │   Protocol     │        │
│  └────────────────┘  └────────────────┘  └────────────────┘        │
└─────────────────────────────────────────────────────────────────────┘
```

### 转换流程示例

```
用户请求 (model: claude-opus-4.5)
    │
    ▼
CredentialProviderRegistry.find_by_model("claude-opus-4.5")
    │
    ▼
KiroProvider (匹配 claude-* 模型)
    ├─ acquire_credential() → 获取 OAuth 凭证
    ├─ transform_request() → CodeWhisperer 格式
    ├─ apply_risk_control() → Machine ID、特殊头部
    └─ target_protocol() → "anthropic"
    │
    ▼
StandardProtocolRegistry.get("anthropic")
    │
    ▼
AnthropicProtocolHandler
    ├─ format_request() → Anthropic Messages API 格式
    └─ parse_response() → 统一响应格式
    │
    ▼
输出适配层 (根据客户端选择)
    ├─ Anthropic API Output
    ├─ OpenAI API Output
    └─ Claude Code Protocol Output
```

### 3.3 CredentialProviderPlugin Trait

```rust
/// 凭证提供商插件 - 核心 Trait
///
/// 设计原则：
/// - 不依赖任何硬编码枚举
/// - 新增 Provider 只需实现此 trait 并注册
/// - 凭证配置由插件自己定义 Schema
/// - 一个插件可支持多种认证方式（OAuth、API Key、第三方中转）
#[async_trait]
pub trait CredentialProviderPlugin: Send + Sync {
    // ========== 基础信息 ==========

    /// 插件唯一标识（代替 ProviderType 枚举）
    fn id(&self) -> &str;

    /// 显示名称
    fn display_name(&self) -> &str;

    /// 插件版本
    fn version(&self) -> &str;

    /// 插件描述
    fn description(&self) -> &str { "" }

    /// 默认目标标准协议
    fn target_protocol(&self) -> &str;

    /// 根据模型动态返回目标协议（用于 Antigravity 等多协议 Provider）
    fn target_protocol_for_model(&self, model: &str) -> &str {
        self.target_protocol()  // 默认返回固定协议
    }

    // ========== 多认证方式支持 ==========

    /// 支持的认证方式（一个插件可支持多种）
    /// 例如 Anthropic 同时支持 OAuth、API Key、第三方中转
    fn supported_auth_types(&self) -> Vec<AuthTypeInfo>;

    /// 根据认证方式返回对应的凭证配置 Schema
    fn credential_schema_for_auth(&self, auth_type: &str) -> serde_json::Value;

    /// 解析凭证配置（从 JSON 解析成插件内部结构）
    fn parse_credential_config(&self, auth_type: &str, config: serde_json::Value) -> Result<Box<dyn CredentialConfig>>;

    /// 创建凭证（从用户输入创建）
    async fn create_credential(&self, auth_type: &str, config: serde_json::Value) -> Result<String>;

    // ========== 模型能力 ==========

    /// 模型家族定义（用于 Mini/Pro/Max 分层）
    fn model_families(&self) -> Vec<ModelFamily>;

    /// 获取支持的模型列表
    async fn list_models(&self) -> Result<Vec<ModelInfo>>;

    /// 检查是否支持某个模型
    fn supports_model(&self, model: &str) -> bool;

    // ========== 凭证管理 ==========

    /// 获取可用凭证
    async fn acquire_credential(&self, model: &str) -> Result<AcquiredCredential>;

    /// 释放凭证
    async fn release_credential(&self, credential_id: &str, result: UsageResult);

    /// 验证凭证有效性
    async fn validate_credential(&self, credential_id: &str) -> Result<ValidationResult>;

    /// 刷新 Token（OAuth 类型）
    async fn refresh_token(&self, credential_id: &str) -> Result<TokenRefreshResult>;

    // ========== 协议转换 ==========

    /// 将输入请求转换成标准协议
    async fn transform_request(&self, req: &mut ChatRequest) -> Result<()>;

    /// 将响应转换回来（如果需要）
    async fn transform_response(&self, resp: &mut ChatResponse) -> Result<()>;

    // ========== 风控适配 ==========

    /// 应用特有的风控逻辑
    async fn apply_risk_control(&self, req: &mut ChatRequest, credential_id: &str) -> Result<()>;

    /// 解析特有的错误码
    fn parse_error(&self, status: u16, body: &str) -> Option<ProviderError>;

    // ========== 插件配置（非凭证配置）==========

    /// 插件配置 Schema（用于 UI 动态生成表单）
    fn plugin_config_schema(&self) -> serde_json::Value { serde_json::json!({}) }

    /// 更新插件配置
    async fn update_plugin_config(&mut self, config: serde_json::Value) -> Result<()> { Ok(()) }

    // ========== 生命周期 ==========

    /// 初始化插件
    async fn init(&mut self) -> Result<()>;

    /// 关闭插件
    async fn shutdown(&mut self) -> Result<()>;
}

/// 认证方式信息
pub struct AuthTypeInfo {
    /// 认证方式 ID
    pub id: String,           // "oauth", "api_key", "third_party"
    /// 显示名称
    pub display_name: String, // "OAuth 登录", "官方 API Key", "第三方中转"
    /// 描述
    pub description: String,  // "使用官方 OAuth 授权"
    /// UI 分组（显示在哪个 Tab）
    pub category: CredentialCategory,
}

/// 凭证配置 trait（代替 CredentialData 枚举）
/// 每个插件自己定义凭证配置结构
pub trait CredentialConfig: Send + Sync + Any {
    fn as_any(&self) -> &dyn Any;
    fn credential_type(&self) -> &str;  // "oauth", "api_key", "third_party"
}
```

### 3.4 标准协议（动态注册）

```rust
// ❌ 不再硬编码枚举
// pub enum StandardProtocol { Anthropic, OpenAI, Gemini }

// ✅ 改为字符串标识 + 注册表
pub struct StandardProtocol(pub String);

impl StandardProtocol {
    // 内置常量（方便使用，但不限制扩展）
    pub const ANTHROPIC: &'static str = "anthropic";
    pub const OPENAI: &'static str = "openai";
    pub const GEMINI: &'static str = "gemini";
    pub const QWEN: &'static str = "qwen";        // 通义千问
    pub const DOUBAO: &'static str = "doubao";    // 字节豆包
    pub const WENXIN: &'static str = "wenxin";    // 百度文心
    // ... 可继续扩展
}

/// 标准协议注册表
pub struct StandardProtocolRegistry {
    protocols: HashMap<String, Arc<dyn ProtocolHandler>>,
}

impl StandardProtocolRegistry {
    pub fn new() -> Self {
        let mut registry = Self::default();

        // 内置协议处理器
        registry.register("anthropic", Arc::new(AnthropicProtocolHandler::new()));
        registry.register("openai", Arc::new(OpenAIProtocolHandler::new()));
        registry.register("gemini", Arc::new(GeminiProtocolHandler::new()));

        // 中国厂商（目前大多兼容 OpenAI）
        registry.register("qwen", Arc::new(OpenAICompatHandler::new("qwen")));
        registry.register("doubao", Arc::new(OpenAICompatHandler::new("doubao")));

        registry
    }

    /// 注册新协议（支持运行时扩展）
    pub fn register(&mut self, id: &str, handler: Arc<dyn ProtocolHandler>);

    /// 获取协议处理器
    pub fn get(&self, id: &str) -> Option<Arc<dyn ProtocolHandler>>;

    /// 列出所有已注册协议
    pub fn list(&self) -> Vec<String>;
}

/// 协议处理器 trait
#[async_trait]
pub trait ProtocolHandler: Send + Sync {
    /// 协议 ID
    fn id(&self) -> &str;

    /// 显示名称
    fn display_name(&self) -> &str;

    /// 请求格式化（转换成该协议的请求格式）
    fn format_request(&self, req: &ChatRequest) -> Result<serde_json::Value>;

    /// 响应解析
    fn parse_response(&self, resp: &serde_json::Value) -> Result<ChatResponse>;

    /// 流式响应解析
    fn parse_stream_chunk(&self, chunk: &[u8]) -> Result<Option<ChatChunk>>;

    /// 错误解析
    fn parse_error(&self, status: u16, body: &str) -> ProviderError;
}
```

### 3.5 现有系统代码分析

#### 3.5.1 现有硬编码枚举（需要消除）

**ProviderType 枚举（11种）** - `src-tauri/src/lib.rs:71-91`：
```rust
pub enum ProviderType {
    Kiro,        // AWS 凭证同步
    Gemini,      // Google Gemini OAuth
    Qwen,        // 阿里通义千问 OAuth
    OpenAI,      // OpenAI API Key
    Claude,      // Anthropic API Key
    Antigravity, // Gemini 3 Pro
    Vertex,      // Google Vertex AI
    GeminiApiKey,// Gemini API Key
    Codex,       // OpenAI OAuth
    ClaudeOAuth, // Anthropic OAuth
    IFlow,       // IFlow
}
```

**ApiProviderType 枚举（10种）** - `src-tauri/src/database/dao/api_key_provider.rs`：
```rust
pub enum ApiProviderType {
    Openai, OpenaiResponse, Anthropic, Gemini,
    AzureOpenai, Vertexai, AwsBedrock, Ollama,
    NewApi, Gateway,
}
```

**问题**：两套枚举并存，新增 Provider 需要修改多处代码。

#### 3.5.2 现有凭证数据结构（过于简单）

**CredentialData** - `src-tauri/src/credential/types.rs:73-88`：
```rust
pub enum CredentialData {
    OAuth {
        access_token: String,
        refresh_token: Option<String>,
        expires_at: Option<DateTime<Utc>>,
    },
    ApiKey {
        key: String,
        base_url: Option<String>,
    },
}
```

**问题**：只有 2 种类型，无法支持：
- AWS 凭证（Access Key + Secret Key + Region）
- Azure 凭证（Endpoint + API Version + Deployment）
- 复杂配置（订阅类型、限流、额度等）

#### 3.5.3 现有数据库表结构

**表 1: provider_pool_credentials**（新凭证池）：
```sql
uuid, provider_type, credential_data (JSON),
name, is_healthy, is_disabled,
check_health, check_model_name, not_supported_models,
usage_count, error_count, last_used, last_error_time,
cached_access_token, cached_refresh_token, token_expiry_time,
source, proxy_url, created_at, updated_at
```

**表 2: api_key_providers**（旧 API Key 系统）：
```sql
id, name, type, api_host, is_system, group_name,
enabled, sort_order, api_version, project, location, region
```

#### 3.5.4 现有 UI 结构

```
ProviderPoolPage
├── Tab: OAuth（卡片布局）
│   └── CredentialCard × N
├── Tab: API Key（左右分栏）
│   ├── 左栏: ProviderList（60+ 系统预设）
│   └── 右栏: ProviderSetting + ApiKeyList
└── Tab: Config
    └── VertexAI / Amp 配置
```

#### 3.5.5 OAuth Provider 插件架构（参考 MachineIdTool）

**核心设计理念**：

- **每个插件实现自己的 UI** - 包括凭证管理、配置界面、状态展示等
- **ProxyCast 作为入口和注册管理中心** - 提供插件容器、协议路由、注册表管理
- **插件完全自治** - UI、业务逻辑、风控策略都在插件内部实现

**插件架构设计**：OAuth Provider 采用与 MachineIdTool 一致的独立项目形式

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    OAuth Provider 插件架构                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  每个 OAuth Provider 是独立的项目/仓库：                                  │
│                                                                          │
│  github.com/aiclientproxy/kiro-provider/                                │
│  ├── plugin/                                                            │
│  │   ├── plugin.json          # 插件元数据                              │
│  │   └── config.json          # 插件配置                                │
│  ├── src-tauri/src/                                                     │
│  │   ├── lib.rs               # 插件入口                                │
│  │   ├── commands.rs          # Tauri 命令                              │
│  │   ├── service.rs           # 核心服务逻辑                            │
│  │   └── models.rs            # 数据模型                                │
│  └── src/                     # 【必须】插件前端 UI                      │
│      ├── components/          # UI 组件                                 │
│      │   ├── CredentialList.tsx    # 凭证列表                          │
│      │   ├── CredentialForm.tsx    # 凭证添加/编辑表单                  │
│      │   ├── SettingsPanel.tsx     # 插件设置面板                       │
│      │   └── StatusCard.tsx        # 状态卡片                           │
│      ├── hooks/               # 插件专用 hooks                          │
│      └── index.tsx            # 插件 UI 入口                            │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

**ProxyCast 与插件的职责划分**：

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          职责划分                                        │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ProxyCast 主应用（入口 + 注册中心）:                                     │
│  ├── 插件容器 (Plugin Host)                                              │
│  │   ├── 加载插件 UI 组件                                                │
│  │   ├── 提供插件挂载点 (mounting points)                                │
│  │   └── 插件生命周期管理                                                 │
│  ├── 注册管理中心                                                        │
│  │   ├── OAuthProviderRegistry (插件注册表)                              │
│  │   ├── 插件发现与安装                                                  │
│  │   └── 插件启用/禁用控制                                               │
│  ├── 协议路由层                                                          │
│  │   ├── 请求路由到对应插件                                              │
│  │   └── 响应协议转换                                                    │
│  └── 公共基础设施                                                        │
│      ├── 数据库连接                                                      │
│      ├── HTTP 客户端                                                     │
│      └── 加密存储                                                        │
│                                                                          │
│  ─────────────────────────────────────────────────────────────────────  │
│                                                                          │
│  插件（自治单元）:                                                        │
│  ├── 【必须】自己的 UI                                                    │
│  │   ├── 凭证管理界面                                                    │
│  │   ├── 配置界面                                                        │
│  │   └── 状态展示                                                        │
│  ├── 业务逻辑                                                            │
│  │   ├── Token 刷新                                                      │
│  │   ├── 凭证验证                                                        │
│  │   └── 协议转换                                                        │
│  └── 风控策略                                                            │
│      ├── Machine ID 管理                                                 │
│      ├── 限流检测                                                        │
│      └── 冷却期控制                                                      │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

**plugin.json 结构（参考 MachineIdTool）**：

```json
{
  "name": "kiro-provider",
  "version": "1.0.0",
  "description": "Kiro (AWS CodeWhisperer) OAuth Provider - 支持 Claude 模型",
  "author": "ProxyCast Team",
  "homepage": "https://github.com/aiclientproxy/kiro-provider",
  "license": "MIT",

  "plugin_type": "oauth_provider",
  "entry": "kiro-provider-cli",
  "min_proxycast_version": "1.0.0",

  "provider": {
    "id": "kiro",
    "display_name": "Kiro (CodeWhisperer)",
    "target_protocol": "anthropic",
    "supported_models": ["claude-*"],
    "auth_type": "oauth",
    "credential_schema": {
      "type": "object",
      "required": ["creds_file_path"],
      "properties": {
        "creds_file_path": {
          "type": "string",
          "title": "凭证文件路径",
          "description": "AWS SSO 凭证文件"
        }
      }
    }
  },

  "binary": {
    "binary_name": "kiro-provider-cli",
    "github_owner": "aiclientproxy",
    "github_repo": "kiro-provider",
    "platform_binaries": {
      "macos-arm64": "kiro-provider-aarch64-apple-darwin",
      "macos-x64": "kiro-provider-x86_64-apple-darwin",
      "linux-x64": "kiro-provider-x86_64-unknown-linux-gnu",
      "windows-x64": "kiro-provider-x86_64-pc-windows-msvc.exe"
    },
    "checksum_file": "checksums.txt"
  },

  "ui": {
    "surfaces": ["oauth_providers"],
    "icon": "Cloud",
    "title": "Kiro Provider"
  }
}
```

**config.json 结构**：

```json
{
  "enabled": true,
  "timeout_ms": 30000,
  "settings": {
    "risk_control": {
      "machine_id_rotation": true,
      "version_spoofing": true
    }
  }
}
```

**OAuth Provider 插件列表（每个独立仓库）**：

| 插件名 | 仓库 | plugin_type | 复杂度 |
|--------|------|-------------|--------|
| kiro-provider | aiclientproxy/kiro-provider | oauth_provider | 🔴 高 |
| antigravity-provider | aiclientproxy/antigravity-provider | oauth_provider | 🔴 高 |
| claude-oauth-provider | aiclientproxy/claude-oauth-provider | oauth_provider | 🟡 中 |
| codex-provider | aiclientproxy/codex-provider | oauth_provider | 🟡 中 |
| gemini-oauth-provider | aiclientproxy/gemini-oauth-provider | oauth_provider | 🟡 中 |
| qwen-provider | aiclientproxy/qwen-provider | oauth_provider | 🟢 低 |
| iflow-provider | aiclientproxy/iflow-provider | oauth_provider | 🟢 低 |

#### 3.5.6 ProxyCast 插件宿主架构

**核心理念**：ProxyCast 只提供入口和容器，具体 UI 由各插件自己实现

**ProxyCast 主应用 UI 结构**：

```
ProviderPoolPage
├── Tab: OAuth 插件
│   ├── 插件导航栏（顶部）
│   │   ├── [+ 安装插件] 按钮
│   │   └── 已安装插件列表
│   │       ├── 🔌 Kiro Provider      ● 已启用  [打开]
│   │       ├── 🔌 Antigravity        ● 已启用  [打开]
│   │       ├── 🔌 Claude OAuth       ○ 已禁用  [打开]
│   │       └── ...
│   │
│   └── 插件 UI 挂载区域（主体）
│       └── <PluginContainer pluginId="kiro-provider">
│           │
│           └── 【由插件自己渲染的 UI】
│               ├── 凭证管理界面
│               ├── 配置面板
│               └── 状态展示
│
├── Tab: API Key（保持不变）
│
└── Tab: Config（保持不变）
```

**插件 UI 加载机制**：

```
┌─────────────────────────────────────────────────────────────────────────┐
│                       插件 UI 加载流程                                   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  1. ProxyCast 启动                                                       │
│     │                                                                    │
│     ▼                                                                    │
│  2. 扫描 ~/.proxycast/plugins/ 目录                                      │
│     │                                                                    │
│     ▼                                                                    │
│  3. 读取每个插件的 plugin.json                                            │
│     ├── 获取 ui.entry 字段（如 "dist/index.js"）                          │
│     └── 获取 ui.surfaces 字段（如 ["oauth_providers"]）                   │
│     │                                                                    │
│     ▼                                                                    │
│  4. 注册到 PluginUIRegistry                                               │
│     │                                                                    │
│     ▼                                                                    │
│  5. 用户点击某个插件时                                                     │
│     │                                                                    │
│     ▼                                                                    │
│  6. PluginContainer 动态加载插件 UI                                        │
│     ├── 加载插件的 JavaScript/CSS                                         │
│     ├── 创建 iframe 或 Web Component 容器                                 │
│     └── 传入 ProxyCast SDK (数据库访问、HTTP 客户端等)                     │
│     │                                                                    │
│     ▼                                                                    │
│  7. 插件 UI 渲染在容器中                                                   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

**plugin.json UI 配置扩展**：

```json
{
  "ui": {
    "surfaces": ["oauth_providers"],
    "icon": "Cloud",
    "title": "Kiro Provider",
    "entry": "dist/index.js",           // 插件 UI 入口文件
    "styles": "dist/styles.css",        // 插件样式文件
    "default_width": 800,
    "default_height": 600,
    "permissions": [                    // 插件需要的权限
      "database:read",
      "database:write",
      "http:request",
      "crypto:encrypt"
    ]
  }
}
```

**ProxyCast SDK（供插件使用）**：

```typescript
// ProxyCast 提供给插件的 SDK
interface ProxyCastPluginSDK {
  // 数据库操作
  database: {
    query<T>(sql: string, params?: any[]): Promise<T[]>;
    execute(sql: string, params?: any[]): Promise<void>;
  };

  // HTTP 客户端
  http: {
    request(url: string, options?: RequestOptions): Promise<Response>;
  };

  // 加密存储
  crypto: {
    encrypt(data: string): Promise<string>;
    decrypt(data: string): Promise<string>;
  };

  // 通知
  notification: {
    success(message: string): void;
    error(message: string): void;
    info(message: string): void;
  };

  // 插件间通信
  events: {
    emit(event: string, data: any): void;
    on(event: string, callback: (data: any) => void): void;
  };
}
```

**插件 UI 示例（Kiro Provider）**：

```tsx
// kiro-provider/src/index.tsx
import { ProxyCastPluginSDK } from '@proxycast/plugin-sdk';

interface PluginProps {
  sdk: ProxyCastPluginSDK;
  pluginId: string;
}

export default function KiroProviderUI({ sdk, pluginId }: PluginProps) {
  const [credentials, setCredentials] = useState<Credential[]>([]);

  useEffect(() => {
    // 使用 ProxyCast SDK 查询数据
    sdk.database.query<Credential>(
      'SELECT * FROM plugin_credentials WHERE plugin_id = ?',
      [pluginId]
    ).then(setCredentials);
  }, []);

  return (
    <div className="kiro-provider-ui">
      <CredentialList credentials={credentials} />
      <CredentialForm onSubmit={handleAddCredential} />
      <SettingsPanel />
    </div>
  );
}
```

**ProxyCast 提供的公共功能**（非 UI）：

| 功能 | ProxyCast 提供 | 插件实现 |
|------|--------------|---------|
| 插件安装/卸载 | ✅ | - |
| 插件启用/禁用 | ✅ | - |
| 插件更新检查 | ✅ | - |
| UI 容器/挂载点 | ✅ | - |
| 凭证管理 UI | - | ✅ 各插件自己实现 |
| 配置界面 | - | ✅ 各插件自己实现 |
| 状态展示 | - | ✅ 各插件自己实现 |
| Token 刷新逻辑 | - | ✅ 各插件自己实现 |
| 风控策略 | - | ✅ 各插件自己实现 |

**插件管理功能（ProxyCast 实现）**：

| 功能 | 说明 |
|------|------|
| 安装插件 | 从 GitHub Release 下载或本地文件安装 |
| 检查更新 | 比较 GitHub Release 版本，提示更新 |
| 启用/禁用 | 修改 config.json 的 enabled 字段 |
| 卸载插件 | 删除插件文件和凭证数据（需确认）|
| 权限管理 | 控制插件可访问的 SDK 能力 |

**插件安装流程**：

```
1. 用户点击 [+ 安装插件]
       │
       ▼
2. 选择安装方式
   ├── 从 GitHub: 输入仓库地址 (aiclientproxy/kiro-provider)
   ├── 从文件: 选择 plugin.json
   └── 内置列表: 选择预置插件
       │
       ▼
3. 下载插件
   ├── 解析 plugin.json
   ├── 根据平台下载对应二进制
   ├── 下载 UI 资源 (dist/index.js, dist/styles.css)
   ├── 校验 checksum
   └── 安装到 ~/.proxycast/plugins/{plugin-name}/
       │
       ▼
4. 注册插件
   ├── 加载到 OAuthProviderRegistry (后端)
   ├── 加载到 PluginUIRegistry (前端)
   ├── 创建 config.json
   └── 显示在插件列表
       │
       ▼
5. 用户点击插件
   └── 加载插件自己的 UI
```

#### 3.5.7 插件化改造目标

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    现有硬编码架构 → 插件化架构                            │
├─────────────────────────────────────────────────────────────────────────┤
│  ProviderType 枚举 (11种)  →  CredentialProviderRegistry (动态注册)      │
│  ApiProviderType 枚举 (10种) →  合并到 CredentialProviderPlugin          │
│  CredentialData 枚举 (2种) →  插件自定义 CredentialConfig trait          │
│  硬编码配置字段           →  插件返回 JSON Schema                        │
├─────────────────────────────────────────────────────────────────────────┤
│  新增 Provider:                                                          │
│  - 现有: 修改 2 个枚举 + 多处 match                                       │
│  - 插件化: 实现 trait + 注册，核心代码不变                                │
└─────────────────────────────────────────────────────────────────────────┘
```

#### 3.5.8 插件化凭证配置设计

**核心理念**：凭证配置由插件自己定义，通过 JSON Schema 动态生成前端表单

```rust
/// 插件返回 JSON Schema，前端动态渲染表单
pub trait CredentialProviderPlugin {
    /// 返回凭证配置的 JSON Schema
    fn credential_schema(&self) -> serde_json::Value;

    /// 验证并解析凭证配置
    fn parse_credential(&self, config: serde_json::Value) -> Result<Box<dyn CredentialConfig>>;
}
```

**示例：现有 ProviderType 迁移为插件**

| 现有类型 | 插件 ID | 凭证类型 | JSON Schema 定义的字段 |
|---------|--------|---------|----------------------|
| Kiro | `kiro` | OAuth | `creds_file_path` |
| ClaudeOAuth | `claude_oauth` | OAuth | `creds_file_path` |
| Claude | `claude_api` | ApiKey | `key`, `base_url` |
| OpenAI | `openai_api` | ApiKey | `key`, `base_url` |
| Codex | `codex` | OAuth | `creds_file_path`, `api_base_url` |
| Gemini | `gemini_oauth` | OAuth | `creds_file_path`, `project_id` |
| GeminiApiKey | `gemini_api` | ApiKey | `key`, `base_url`, `excluded_models` |
| Vertex | `vertex` | ApiKey | `key`, `base_url`, `project`, `location` |
| Antigravity | `antigravity` | OAuth | `creds_file_path`, `project_id` |
| Qwen | `qwen` | OAuth | `creds_file_path` |
| IFlow | `iflow` | OAuth/Cookie | `creds_file_path` |

**示例：现有 ApiProviderType 迁移为插件**

| 现有类型 | 插件 ID | JSON Schema 定义的字段 |
|---------|--------|----------------------|
| AzureOpenai | `azure_openai` | `api_host`, `api_key`, `api_version` |
| AwsBedrock | `aws_bedrock` | `api_key`, `region` |
| Vertexai | `vertexai` | `api_key`, `project`, `location` |

#### 3.5.9 插件实现示例

**Kiro Provider（OAuth + 风控）**：

```rust
pub struct KiroProvider {
    credential_pool: CredentialPool,
    machine_id_cache: HashMap<String, String>,
}

impl CredentialProviderPlugin for KiroProvider {
    fn id(&self) -> &str { "kiro" }
    fn display_name(&self) -> &str { "Kiro (AWS CodeWhisperer)" }
    fn target_protocol(&self) -> &str { "anthropic" }
    fn ui_category(&self) -> CredentialCategory { CredentialCategory::OAuth }

    fn credential_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "required": ["creds_file_path"],
            "properties": {
                "creds_file_path": {
                    "type": "string",
                    "title": "凭证文件路径",
                    "description": "AWS SSO 凭证文件"
                }
            }
        })
    }

    fn model_families(&self) -> Vec<ModelFamily> {
        vec![
            ModelFamily { name: "opus", pattern: "claude-opus-*" },
            ModelFamily { name: "sonnet", pattern: "claude-sonnet-*" },
            ModelFamily { name: "haiku", pattern: "claude-*-haiku" },
        ]
    }

    async fn apply_risk_control(&self, req: &mut ChatRequest, cred_id: &str) -> Result<()> {
        // Kiro 特有风控：Machine ID、系统信息、版本号
        let machine_id = self.get_or_generate_machine_id(cred_id);
        req.headers.insert("X-Amz-Machine-Id", machine_id);
        req.headers.insert("X-Amz-Os", get_os_info());
        req.headers.insert("X-Kiro-Version", KIRO_VERSION);
        Ok(())
    }
}
```

**Azure OpenAI Provider（API Key + 特殊配置）**：

```rust
impl CredentialProviderPlugin for AzureOpenAIProvider {
    fn id(&self) -> &str { "azure_openai" }
    fn display_name(&self) -> &str { "Azure OpenAI" }
    fn target_protocol(&self) -> &str { "openai" }
    fn ui_category(&self) -> CredentialCategory { CredentialCategory::ApiKey }

    fn credential_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "required": ["api_host", "api_key", "api_version"],
            "properties": {
                "api_host": {
                    "type": "string",
                    "title": "Azure Endpoint",
                    "description": "https://your-resource.openai.azure.com"
                },
                "api_key": { "type": "string", "title": "API Key" },
                "api_version": {
                    "type": "string",
                    "title": "API 版本",
                    "default": "2024-02-01"
                }
            }
        })
    }

    async fn acquire_credential(&self, model: &str) -> Result<AcquiredCredential> {
        let cred = self.credential_pool.acquire().await?;
        let config: AzureConfig = serde_json::from_value(cred.config)?;

        // Azure 特殊的 URL 构建
        let url = format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            config.api_host, model, config.api_version
        );

        Ok(AcquiredCredential {
            id: cred.id,
            base_url: url,
            headers: vec![("api-key", config.api_key)],
        })
    }
}
```

### 3.6 凭证分组（用于 UI）

```rust
/// 凭证分组（用于 UI Tab 展示）
/// 保留枚举，因为 UI Tab 结构相对稳定
#[derive(Clone, Copy, Debug)]
pub enum CredentialCategory {
    OAuth,      // OAuth 凭证 Tab
    ApiKey,     // API Key Tab
    Other,      // 其他配置 Tab（第三方中转、Cookie 等）
}
```

### 3.7 认证类型

```rust
/// 认证类型（内置常见类型，但不限制扩展）
#[derive(Clone, Debug)]
pub enum AuthType {
    /// API Key 认证
    ApiKey {
        header_name: String,      // "Authorization", "x-api-key"
        prefix: Option<String>,   // "Bearer ", "sk-"
    },
    /// OAuth 2.0
    OAuth {
        token_url: String,
        client_id: Option<String>,
        client_secret: Option<String>,
    },
    /// Cookie 认证
    Cookie {
        cookie_name: String,
    },
    /// 自定义头部
    CustomHeaders(HashMap<String, String>),
    /// 无认证
    None,
}
```

### 3.8 插件注册表

```rust
pub struct CredentialProviderRegistry {
    providers: HashMap<String, Arc<dyn CredentialProviderPlugin>>,
    model_index: HashMap<String, String>,  // model -> provider_id
    plugins_dir: PathBuf,                  // ~/.proxycast/plugins/
}

impl CredentialProviderRegistry {
    pub fn new(plugins_dir: PathBuf) -> Self {
        Self {
            providers: HashMap::new(),
            model_index: HashMap::new(),
            plugins_dir,
        }
    }

    /// 从外部目录加载所有插件
    pub async fn load_plugins(&mut self) -> Result<()> {
        // 扫描 ~/.proxycast/plugins/ 目录
        for entry in fs::read_dir(&self.plugins_dir)? {
            let plugin_dir = entry?.path();
            if plugin_dir.is_dir() {
                self.load_plugin(&plugin_dir).await?;
            }
        }
        Ok(())
    }

    /// 加载单个插件
    async fn load_plugin(&mut self, plugin_dir: &Path) -> Result<()> {
        // 1. 读取 plugin.json
        let manifest_path = plugin_dir.join("plugin.json");
        let manifest: PluginManifest = serde_json::from_str(
            &fs::read_to_string(&manifest_path)?
        )?;

        // 2. 检查 plugin_type 是否为 oauth_provider
        if manifest.plugin_type != "oauth_provider" {
            return Ok(());  // 跳过非 OAuth Provider 插件
        }

        // 3. 加载插件二进制
        let binary_path = plugin_dir.join("bin").join(&manifest.binary.binary_name);
        let plugin = ExternalOAuthPlugin::new(binary_path, manifest)?;

        // 4. 注册到注册表
        self.register(Arc::new(plugin));
        Ok(())
    }

    /// 注册插件
    pub fn register(&mut self, provider: Arc<dyn CredentialProviderPlugin>);

    /// 根据模型查找 Provider
    pub fn find_by_model(&self, model: &str) -> Option<Arc<dyn CredentialProviderPlugin>>;

    /// 获取所有已启用的 Provider
    pub fn get_enabled(&self) -> Vec<Arc<dyn CredentialProviderPlugin>>;

    /// 获取某个 Provider
    pub fn get(&self, id: &str) -> Option<Arc<dyn CredentialProviderPlugin>>;

    /// 安装新插件（从 GitHub 或本地文件）
    pub async fn install_plugin(&mut self, source: PluginSource) -> Result<()>;

    /// 卸载插件
    pub async fn uninstall_plugin(&mut self, plugin_id: &str) -> Result<()>;

    /// 检查插件更新
    pub async fn check_updates(&self) -> Result<Vec<PluginUpdate>>;
}

/// 外部 OAuth 插件（通过二进制调用）
pub struct ExternalOAuthPlugin {
    manifest: PluginManifest,
    binary_path: PathBuf,
    config: PluginConfig,
}

impl CredentialProviderPlugin for ExternalOAuthPlugin {
    // 通过调用外部二进制实现 trait 方法
    // 使用 JSON-RPC 或 stdin/stdout 通信
}
```

---

## 四、OAuth 凭证插件迁移清单

### 4.1 需要插件化的 ProviderType（仅 OAuth）

根据现有代码 `src-tauri/src/lib.rs`，需要迁移 **8 个 OAuth 类型**：

| 现有枚举值 | 插件 ID | 认证方式 | 目标协议 | 复杂度 |
|-----------|--------|---------|---------|--------|
| `Kiro` | `kiro` | OAuth (creds_file) | anthropic | 🔴 高 |
| `Antigravity` | `antigravity` | OAuth (creds_file, project_id) | 动态* | 🔴 高 |
| `ClaudeOAuth` | `claude_oauth` | OAuth (creds_file) | anthropic | 🟡 中 |
| `Codex` | `codex` | OAuth (creds_file) | openai | 🟡 中 |
| `Gemini` | `gemini_oauth` | OAuth (creds_file, project_id) | gemini | 🟡 中 |
| `Qwen` | `qwen` | OAuth (creds_file) | openai | 🟢 低 |
| `IFlow` | `iflow` | OAuth/Cookie | openai | 🟢 低 |

> *Antigravity 根据模型动态选择协议：`claude-*` → anthropic, 其他 → gemini

### 4.2 不需要插件化的类型（保持现有）

**API Key 类型 - 继续使用现有 `api_key_providers` 系统**：

| 现有枚举值 | 处理方式 | 原因 |
|-----------|---------|------|
| `Claude` | 保持现有 | 简单 API Key，已有系统支持 |
| `OpenAI` | 保持现有 | 简单 API Key，已有系统支持 |
| `GeminiApiKey` | 保持现有 | 简单 API Key，已有系统支持 |
| `Vertex` | 保持现有 | 已有配置字段 (project, location) |

**ApiProviderType - 完全不变**：
- 60+ 系统预设 Provider 继续使用
- 用户自定义 Provider 功能继续使用
- `api_key_providers` + `api_keys` 表结构不变

### 4.3 复杂度分析

**🔴 高复杂度**：
- `kiro`: Machine ID 生成、系统指纹、版本号伪装、特殊头部
- `antigravity`: 动态协议选择、模型别名映射、Safety Settings

**🟡 中复杂度**：
- `claude_oauth`, `codex`, `gemini_oauth`: Token 刷新、凭证文件解析

**🟢 低复杂度**：
- `qwen`, `iflow`: 标准 OAuth 流程

---

## 五、迁移方案

### 5.1 OAuth 凭证代码映射

**迁移策略**：将现有代码迁移到独立的 GitHub 仓库

| 现有模块 | 迁移目标（独立仓库） | 说明 |
|---------|---------------------|------|
| `providers/kiro.rs` | `aiclientproxy/kiro-provider` | 提取风控逻辑 + UI |
| `providers/claude_oauth.rs` | `aiclientproxy/claude-oauth-provider` | Token 刷新 + UI |
| `providers/codex.rs` | `aiclientproxy/codex-provider` | OpenAI OAuth + UI |
| `providers/gemini.rs` | `aiclientproxy/gemini-oauth-provider` | Google OAuth + UI |
| `providers/antigravity.rs` | `aiclientproxy/antigravity-provider` | 动态协议 + UI |
| `providers/qwen.rs` | `aiclientproxy/qwen-provider` | + UI |
| `providers/iflow.rs` | `aiclientproxy/iflow-provider` | + UI |
| `converter/openai_to_cw.rs` | `kiro-provider` 内部 | 协议转换逻辑 |
| `translator/kiro/*` | `kiro-provider` 内部 | 请求/响应转换 |

**每个独立仓库包含**：
- 后端 Rust 代码（凭证管理、协议转换、风控）
- 前端 React UI（凭证列表、配置表单、状态展示）
- plugin.json 元数据
- GitHub Actions 自动构建发布

### 5.2 保持不变的模块

| 现有模块 | 处理方式 | 原因 |
|---------|---------|------|
| `database/dao/api_key_provider.rs` | **不变** | API Key 系统继续使用 |
| `database/system_providers.rs` | **不变** | 60+ 系统预设 |
| `providers/openai_custom.rs` | **不变** | API Key 方式 |
| `credential/pool.rs` | **复用** | 作为插件内部实现 |
| `credential/balancer.rs` | **复用** | |
| `credential/health.rs` | **复用** | |

### 5.3 目录结构

**ProxyCast 主项目结构**（入口 + 注册中心）：

```
src-tauri/src/
├── credential/
│   ├── mod.rs
│   ├── types.rs              # 保留，通用类型
│   ├── pool.rs               # 保留，复用（供插件使用）
│   ├── balancer.rs           # 保留，复用
│   ├── health.rs             # 保留，复用
│   ├── quota.rs              # 保留，复用
│   │
│   ├── plugin.rs             # OAuthProviderPlugin trait (新增)
│   └── registry.rs           # OAuthProviderRegistry (新增，管理外部插件)
│
├── plugin/
│   ├── mod.rs
│   ├── loader.rs             # 插件加载器（从外部目录加载）
│   ├── host.rs               # 插件宿主（生命周期管理）
│   └── sdk.rs                # ProxyCast SDK（供插件调用）
│
├── database/
│   └── dao/
│       └── api_key_provider.rs  # 保持不变！60+ 系统预设
│
├── converter/                # 保留
│   └── ...
│
└── providers/                # 渐进式清理
    └── ...                   # OAuth 相关代码迁移到独立插件项目
```

**前端目录结构**（插件容器）：

```
src/
├── components/
│   └── plugin/
│       ├── PluginContainer.tsx    # 插件 UI 容器
│       ├── PluginLoader.tsx       # 插件 JS/CSS 加载
│       └── PluginRegistry.tsx     # 前端插件注册表
│
└── pages/
    └── ProviderPool/
        └── OAuthPluginTab.tsx     # OAuth 插件列表 + 挂载点
```

**外部插件安装目录**：

```
~/.proxycast/plugins/
├── kiro-provider/                 # 独立插件项目
│   ├── plugin.json                # 插件元数据
│   ├── config.json                # 插件配置
│   ├── bin/
│   │   └── kiro-provider-cli      # 后端二进制
│   └── dist/
│       ├── index.js               # 插件 UI 入口
│       └── styles.css             # 插件样式
│
├── antigravity-provider/
│   └── ...
│
├── claude-oauth-provider/
│   └── ...
│
└── ... (其他 OAuth Provider 插件)
```

**插件项目结构**（独立仓库，参考 MachineIdTool）：

```
github.com/aiclientproxy/kiro-provider/
├── plugin/
│   ├── plugin.json                # 插件元数据
│   └── config.json                # 默认配置
├── src-tauri/src/                 # 后端 Rust 代码
│   ├── lib.rs
│   ├── commands.rs
│   ├── service.rs
│   └── models.rs
├── src/                           # 前端 UI (React)
│   ├── components/
│   │   ├── CredentialList.tsx
│   │   ├── CredentialForm.tsx
│   │   └── SettingsPanel.tsx
│   └── index.tsx
└── .github/
    └── workflows/
        └── release.yml            # 自动构建发布
```

### 5.4 数据库迁移

**新增表**：
```sql
-- 插件配置表
CREATE TABLE credential_provider_plugins (
    id TEXT PRIMARY KEY,              -- "kiro", "anthropic", etc.
    display_name TEXT NOT NULL,
    version TEXT NOT NULL,
    enabled INTEGER DEFAULT 1,
    config TEXT,                      -- JSON 配置
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- 插件凭证关联表（替代现有的 provider_pool_credentials）
CREATE TABLE plugin_credentials (
    id TEXT PRIMARY KEY,
    plugin_id TEXT NOT NULL,
    credential_data TEXT NOT NULL,    -- 加密的凭证数据
    status TEXT DEFAULT 'active',     -- active/cooldown/unhealthy/disabled
    stats TEXT,                       -- JSON 统计数据
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (plugin_id) REFERENCES credential_provider_plugins(id)
);
```

---

## 六、请求处理流程

### 6.1 新流程

```
客户端请求
    │
    ▼
┌─────────────────────────────────────┐
│  1. 路由解析                         │
│     - 识别目标 Provider              │
│     - 识别输入协议                   │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│  2. 插件注册表查找                   │
│     CredentialProviderRegistry       │
│     .find_by_model(model)            │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│  3. Provider 插件处理                │
│     ┌─────────────────────────────┐ │
│     │ acquire_credential()        │ │
│     │ transform_request()         │ │
│     │ apply_risk_control()        │ │
│     └─────────────────────────────┘ │
└─────────────────────────────────────┘
    │
    ▼ (标准协议: Anthropic/OpenAI/Gemini)
    │
┌─────────────────────────────────────┐
│  4. 输出适配（如需要）               │
│     - Anthropic API Output          │
│     - Claude Code Protocol Output   │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│  5. HTTP 请求发送                    │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│  6. 响应处理                         │
│     - transform_response()          │
│     - release_credential()          │
│     - 错误处理和重试                 │
└─────────────────────────────────────┘
```

### 6.2 处理管道集成

```rust
// 修改 ProviderStep，使用插件注册表
impl ProviderStep {
    async fn execute(&self, ctx: &mut RequestContext, payload: &mut Value) -> Result<()> {
        // 1. 获取目标 Provider 插件
        let provider = self.registry
            .find_by_model(&ctx.model)
            .ok_or(Error::NoProviderFound)?;

        // 2. 获取凭证
        let credential = provider.acquire_credential(&ctx.model).await?;

        // 3. 转换请求
        provider.transform_request(payload).await?;

        // 4. 应用风控
        provider.apply_risk_control(payload, &credential).await?;

        // 5. 发送请求
        let result = self.http_client.send(payload, &credential).await;

        // 6. 处理结果
        match &result {
            Ok(resp) => {
                provider.transform_response(resp).await?;
                provider.release_credential(&credential.id, UsageResult::Success).await;
            }
            Err(e) => {
                let error = provider.parse_error(e.status, &e.body);
                provider.release_credential(&credential.id, UsageResult::Error(error)).await;
            }
        }

        result
    }
}
```

---

## 七、与现有插件系统集成

### 7.1 关系设计

```
┌─────────────────────────────────────────────────────────────┐
│                     PluginManager                           │
│  (管理通用插件: 请求修改、响应修改、监控等)                   │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ 并行
                              │
┌─────────────────────────────────────────────────────────────┐
│              CredentialProviderRegistry                      │
│  (管理凭证 Provider 插件: Kiro, Anthropic, OpenAI 等)        │
└─────────────────────────────────────────────────────────────┘
```

### 7.2 共享能力

- **PluginContext**：共享请求上下文
- **超时隔离**：复用 PluginManager 的超时机制
- **UI 系统**：CredentialProviderPlugin 可选实现 PluginUI trait

```rust
// 可选：实现 PluginUI 以提供配置界面
impl PluginUI for KiroProvider {
    fn get_surfaces(&self) -> Vec<SurfaceDefinition> {
        vec![
            SurfaceDefinition {
                id: "kiro_credentials",
                title: "Kiro 凭证管理",
                components: self.build_credential_list_ui(),
            }
        ]
    }
}
```

---

## 八、实施计划

### Phase 1: ProxyCast 插件宿主架构

**目标**：在 ProxyCast 中实现插件宿主和 SDK

**后端任务**：
1. 定义 `CredentialProviderPlugin` trait
2. 实现 `CredentialProviderRegistry`（从外部目录加载）
3. 实现 `PluginLoader`（加载插件二进制）
4. 实现 `ProxyCast SDK`（供插件调用的接口）
5. 创建数据库迁移脚本

**前端任务**：
1. 实现 `PluginContainer` 组件（加载插件 UI）
2. 实现 `PluginLoader`（动态加载 JS/CSS）
3. 实现 `OAuthPluginTab`（插件列表 + 挂载点）
4. 定义 `@proxycast/plugin-sdk` TypeScript 接口

**产出**：
- `src-tauri/src/credential/plugin.rs`
- `src-tauri/src/credential/registry.rs`
- `src-tauri/src/plugin/loader.rs`
- `src-tauri/src/plugin/sdk.rs`
- `src/components/plugin/PluginContainer.tsx`
- `src/components/plugin/PluginLoader.tsx`

### Phase 2: 核心 OAuth Provider 独立仓库

**目标**：创建最复杂的 OAuth Provider 独立项目

**优先级**：
1. **kiro-provider** - 最复杂，有风控逻辑、Machine ID、版本伪装
2. **antigravity-provider** - 动态协议选择（claude-* → Anthropic，gemini-* → Gemini）

**每个仓库任务**：
1. 创建 GitHub 仓库 `aiclientproxy/kiro-provider`
2. 搭建项目结构（plugin.json、src-tauri/、src/）
3. 迁移后端代码（凭证管理、协议转换、风控）
4. 实现前端 UI（凭证列表、配置表单、状态展示）
5. 配置 GitHub Actions 自动构建发布
6. 单元测试 + 集成测试

### Phase 3: 其他 OAuth Provider 独立仓库

**目标**：创建剩余 5 个 OAuth Provider 独立项目

**仓库列表**：
1. `aiclientproxy/claude-oauth-provider` - Anthropic OAuth + UI
2. `aiclientproxy/codex-provider` - OpenAI OAuth + UI
3. `aiclientproxy/gemini-oauth-provider` - Google OAuth + UI
4. `aiclientproxy/qwen-provider` - 阿里云 OAuth + UI
5. `aiclientproxy/iflow-provider` - iFlow OAuth/Cookie + UI

### Phase 4: 协议转换层完善

**目标**：确保所有输入/输出协议组合正常工作

**任务**：
1. 完善 `StreamEvent` 统一事件层
2. 实现 `OpenAI SSE Generator` - 输出 OpenAI 格式
3. 实现 `Anthropic SSE Generator` - 输出 Anthropic 格式
4. 支持按端点自动选择输出协议
5. 集成到处理管道

### Phase 5: 插件管理功能

**目标**：实现插件安装、更新、卸载功能

**任务**：
1. 实现从 GitHub Release 安装插件
2. 实现从本地文件安装插件
3. 实现插件更新检查
4. 实现插件卸载（清理文件和数据）
5. 实现插件启用/禁用
6. 实现插件权限管理

### Phase 6: 清理和测试

**任务**：
1. 删除 ProxyCast 中旧的 `providers/` OAuth 代码
2. 简化 `converter/` 模块
3. 端到端测试（安装插件 → 配置凭证 → 调用 API）
4. 文档更新

---

## 九、风险和缓解

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| 迁移过程中服务中断 | 高 | 保留旧代码，通过 feature flag 切换 |
| 某个 Provider 转换出错 | 中 | 每个 Provider 独立测试，灰度发布 |
| 性能下降 | 中 | 基准测试，优化热路径 |
| 数据库迁移失败 | 高 | 备份现有数据，提供回滚脚本 |

---

## 十、成功标准

### 10.1 OAuth 插件化

1. [ ] 所有 7 个 OAuth Provider 迁移为独立 GitHub 仓库
2. [ ] 每个插件包含完整的后端代码和前端 UI
3. [ ] 新增 OAuth Provider 只需创建新仓库，无需修改 ProxyCast 核心代码
4. [ ] 单个 OAuth Provider 可独立更新（通过 GitHub Release）
5. [ ] 插件 UI 在 ProxyCast 中正确加载和渲染

### 10.2 API Key 系统

1. [ ] API Key 系统保持现有设计不变
2. [ ] 60+ 系统预设 Provider 继续正常工作
3. [ ] 用户自定义 Provider 功能正常

### 10.3 协议转换

1. [ ] Kiro: Anthropic → CodeWhisperer → Anthropic SSE 转换正常
2. [ ] Antigravity (claude-*): Anthropic → Antigravity → Anthropic SSE 转换正常
3. [ ] Antigravity (gemini-*): Anthropic → Antigravity → Gemini 协议转换正常
4. [ ] Claude OAuth: Anthropic → Anthropic 直通正常
5. [ ] 所有 OAuth Provider 的协议转换测试通过

### 10.4 质量保证

1. [ ] 现有功能 100% 兼容
2. [ ] 处理性能无明显下降（<10%）
3. [ ] 完整的单元测试和集成测试

---

## 附录

### A. ProxyCast 关键文件路径

**后端（插件宿主）**：

| 文件 | 用途 |
|------|------|
| `/src-tauri/src/credential/plugin.rs` | CredentialProviderPlugin trait |
| `/src-tauri/src/credential/registry.rs` | CredentialProviderRegistry（加载外部插件）|
| `/src-tauri/src/plugin/loader.rs` | 插件加载器 |
| `/src-tauri/src/plugin/host.rs` | 插件宿主（生命周期管理）|
| `/src-tauri/src/plugin/sdk.rs` | ProxyCast SDK（供插件调用）|

**前端（插件容器）**：

| 文件 | 用途 |
|------|------|
| `/src/components/plugin/PluginContainer.tsx` | 插件 UI 容器 |
| `/src/components/plugin/PluginLoader.tsx` | 动态加载插件 JS/CSS |
| `/src/components/plugin/PluginRegistry.tsx` | 前端插件注册表 |
| `/src/pages/ProviderPool/OAuthPluginTab.tsx` | OAuth 插件列表 + 挂载点 |

### B. 外部插件仓库

| 仓库 | 用途 |
|------|------|
| `aiclientproxy/kiro-provider` | Kiro OAuth Provider（后端 + UI）|
| `aiclientproxy/antigravity-provider` | Antigravity OAuth Provider（后端 + UI）|
| `aiclientproxy/claude-oauth-provider` | Claude OAuth Provider（后端 + UI）|
| `aiclientproxy/codex-provider` | Codex OAuth Provider（后端 + UI）|
| `aiclientproxy/gemini-oauth-provider` | Gemini OAuth Provider（后端 + UI）|
| `aiclientproxy/qwen-provider` | Qwen OAuth Provider（后端 + UI）|
| `aiclientproxy/iflow-provider` | iFlow OAuth Provider（后端 + UI）|

### C. 参考现有代码

| 功能 | 现有文件（迁移到独立仓库）|
|------|---------|
| Kiro 风控逻辑 | `/src-tauri/src/providers/kiro.rs` → `kiro-provider` |
| 协议转换 | `/src-tauri/src/converter/openai_to_cw.rs` → `kiro-provider` |
| 凭证池 | `/src-tauri/src/credential/pool.rs`（保留，供插件复用）|
| 负载均衡 | `/src-tauri/src/credential/balancer.rs`（保留，供插件复用）|
| 插件 trait | `/src-tauri/src/plugin/types.rs`（参考）|

### D. 插件安装目录

```
~/.proxycast/plugins/
├── kiro-provider/
│   ├── plugin.json
│   ├── config.json
│   ├── bin/kiro-provider-cli
│   └── dist/
│       ├── index.js
│       └── styles.css
├── antigravity-provider/
│   └── ...
└── ...
```
