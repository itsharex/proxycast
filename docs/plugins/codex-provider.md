# Codex Provider 插件文档

> 版本: 1.0.0
> 仓库: `aiclientproxy/codex-provider`
> 类型: OAuth Provider Plugin

---

## 一、概述

### 1.1 插件简介

Codex Provider 是 ProxyCast 的 OpenAI Codex 凭证提供者插件，支持 **OAuth 2.0 + PKCE** 和 **API Key** 两种认证方式访问 OpenAI GPT 系列模型。插件兼容 Codex CLI 的凭证格式，可以直接导入 `~/.codex/auth.json` 使用。

### 1.2 支持的认证方式

| 认证方式 | 说明 | 适用场景 |
|---------|------|---------|
| **OAuth** | OAuth 2.0 + PKCE | ChatGPT Plus/Pro 账户 |
| **API Key** | OpenAI API Key | 标准 API 访问 |

### 1.3 核心能力

| 能力 | 说明 |
|------|------|
| 双重认证支持 | OAuth 和 API Key 两种模式 |
| 自动 Token 刷新 | OAuth 模式自动刷新，提前 5 分钟 |
| PKCE 安全 | OAuth 使用 S256 PKCE 流程 |
| Codex CLI 兼容 | 支持 snake_case 和 camelCase 字段名 |
| 自定义 Base URL | 支持第三方 OpenAI 兼容 API |
| 请求格式转换 | OpenAI Chat → Codex Responses API |

### 1.4 支持的模型

| 模型系列 | 示例模型 | 说明 |
|----------|----------|------|
| GPT-4 | `gpt-4`, `gpt-4o`, `gpt-4-turbo` | 最新 GPT-4 系列 |
| GPT-3.5 | `gpt-3.5-turbo` | 快速响应模型 |
| O 系列 | `o1`, `o1-preview`, `o3`, `o4-mini` | 推理增强模型 |
| Codex | `codex-*` | 代码专用模型 |

---

## 二、插件架构

### 2.1 项目结构

```
codex-provider/
├── plugin/
│   ├── plugin.json              # 插件元数据
│   └── config.json              # 默认配置
│
├── src-tauri/src/               # 后端 Rust 代码
│   ├── lib.rs                   # 插件入口
│   ├── commands.rs              # Tauri 命令
│   ├── provider.rs              # CodexProvider 核心实现
│   ├── auth/                    # 认证模块
│   │   ├── mod.rs
│   │   ├── oauth.rs             # OAuth 2.0 + PKCE
│   │   └── api_key.rs           # API Key 模式
│   ├── credentials.rs           # 凭证管理
│   ├── token_refresh.rs         # Token 刷新
│   ├── transform.rs             # 请求格式转换
│   └── api/                     # API 调用
│       ├── mod.rs
│       └── codex.rs             # Codex API
│
├── src/                         # 前端 React UI
│   ├── index.tsx                # 插件 UI 入口
│   ├── components/
│   │   ├── CredentialList.tsx   # 凭证列表
│   │   ├── CredentialCard.tsx   # 凭证卡片
│   │   ├── AuthMethodTabs.tsx   # 认证方式选择
│   │   ├── OAuthForm.tsx        # OAuth 表单
│   │   ├── ApiKeyForm.tsx       # API Key 表单
│   │   ├── ImportForm.tsx       # 导入表单
│   │   └── SettingsPanel.tsx    # 插件设置
│   └── types/
│       └── index.ts             # 类型定义
│
└── .github/
    └── workflows/
        └── release.yml          # 自动构建发布
```

### 2.2 plugin.json

```json
{
  "name": "codex-provider",
  "version": "1.0.0",
  "description": "Codex Provider - 支持 OAuth 和 API Key 两种认证方式访问 OpenAI GPT 模型",
  "author": "ProxyCast Team",
  "homepage": "https://github.com/aiclientproxy/codex-provider",
  "license": "MIT",

  "plugin_type": "oauth_provider",
  "entry": "codex-provider-cli",
  "min_proxycast_version": "1.0.0",

  "provider": {
    "id": "codex",
    "display_name": "Codex (OpenAI)",
    "target_protocol": "openai",
    "supported_models": ["gpt-*", "o1*", "o3*", "o4*", "codex-*"],
    "auth_types": ["oauth", "api_key"],
    "credential_schemas": {
      "oauth": {
        "type": "object",
        "properties": {
          "id_token": { "type": "string" },
          "access_token": { "type": "string" },
          "refresh_token": { "type": "string" },
          "account_id": { "type": "string" },
          "email": { "type": "string" },
          "expires_at": { "type": "string" }
        },
        "required": ["access_token"]
      },
      "api_key": {
        "type": "object",
        "properties": {
          "api_key": { "type": "string" },
          "api_base_url": { "type": "string" }
        },
        "required": ["api_key"]
      }
    }
  },

  "binary": {
    "binary_name": "codex-provider-cli",
    "github_owner": "aiclientproxy",
    "github_repo": "codex-provider",
    "platform_binaries": {
      "macos-arm64": "codex-provider-aarch64-apple-darwin",
      "macos-x64": "codex-provider-x86_64-apple-darwin",
      "linux-x64": "codex-provider-x86_64-unknown-linux-gnu",
      "windows-x64": "codex-provider-x86_64-pc-windows-msvc.exe"
    },
    "checksum_file": "checksums.txt"
  },

  "ui": {
    "surfaces": ["oauth_providers"],
    "icon": "Sparkles",
    "title": "Codex Provider",
    "entry": "dist/index.js",
    "styles": "dist/styles.css",
    "default_width": 900,
    "default_height": 700,
    "permissions": [
      "database:read",
      "database:write",
      "http:request",
      "crypto:encrypt",
      "shell:open"
    ]
  }
}
```

### 2.3 config.json

```json
{
  "enabled": true,
  "timeout_ms": 60000,
  "settings": {
    "oauth": {
      "client_id": "app_EMoamEEZ73f0CkXaXp7hrann",
      "auth_url": "https://auth.openai.com/oauth/authorize",
      "token_url": "https://auth.openai.com/oauth/token",
      "callback_port": 1455,
      "callback_path": "/auth/callback",
      "scopes": "openid email profile offline_access"
    },
    "api": {
      "oauth_base_url": "https://chatgpt.com/backend-api/codex",
      "api_key_base_url": "https://api.openai.com"
    },
    "token_refresh": {
      "auto_refresh": true,
      "refresh_threshold_minutes": 5,
      "max_retry": 3,
      "retry_delay_ms": 1000
    },
    "codex_cli": {
      "default_creds_path": "~/.codex/auth.json",
      "user_agent": "codex_cli_rs/0.50.0 (Mac OS 26.0.1; arm64) Apple_Terminal/464",
      "version_header": "0.21.0"
    }
  }
}
```

---

## 三、认证方式详解

### 3.1 OAuth 认证（OAuth 2.0 + PKCE）

#### OAuth 配置

```rust
// OAuth 端点
const OPENAI_AUTH_URL: &str = "https://auth.openai.com/oauth/authorize";
const OPENAI_TOKEN_URL: &str = "https://auth.openai.com/oauth/token";

// Client ID（Codex CLI 注册的应用 ID）
const OPENAI_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";

// 固定回调配置（OpenAI OAuth 要求）
const OAUTH_CALLBACK_PORT: u16 = 1455;
const OAUTH_CALLBACK_PATH: &str = "/auth/callback";

// OAuth Scope
const OAUTH_SCOPE: &str = "openid email profile offline_access";
```

#### PKCE 流程实现

```rust
use sha2::{Sha256, Digest};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;

/// PKCE 代码对
#[derive(Debug, Clone)]
pub struct PKCECodes {
    /// 128 字符的随机字符串（96 bytes base64url 编码）
    pub code_verifier: String,
    /// code_verifier 的 SHA256 哈希，base64url 编码
    pub code_challenge: String,
}

impl PKCECodes {
    /// 生成 PKCE 代码对
    pub fn generate() -> Result<Self, Error> {
        // 生成 96 字节随机数
        let mut bytes = [0u8; 96];
        rand::thread_rng().fill_bytes(&mut bytes);
        let code_verifier = URL_SAFE_NO_PAD.encode(bytes);

        // S256 方法：SHA256 哈希
        let mut hasher = Sha256::new();
        hasher.update(code_verifier.as_bytes());
        let hash = hasher.finalize();
        let code_challenge = URL_SAFE_NO_PAD.encode(hash);

        Ok(Self { code_verifier, code_challenge })
    }
}

/// 生成 OAuth 授权 URL
pub fn generate_auth_url(state: &str, pkce_codes: &PKCECodes) -> String {
    let redirect_uri = format!(
        "http://localhost:{}{}",
        OAUTH_CALLBACK_PORT, OAUTH_CALLBACK_PATH
    );

    let params = [
        ("client_id", OPENAI_CLIENT_ID),
        ("response_type", "code"),
        ("redirect_uri", redirect_uri.as_str()),
        ("scope", OAUTH_SCOPE),
        ("state", state),
        ("code_challenge", &pkce_codes.code_challenge),
        ("code_challenge_method", "S256"),
        ("prompt", "login"),
        ("id_token_add_organizations", "true"),
        ("codex_cli_simplified_flow", "true"),
    ];

    let query = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    format!("{}?{}", OPENAI_AUTH_URL, query)
}

/// 交换授权码获取 Token
pub async fn exchange_code_for_tokens(
    code: &str,
    code_verifier: &str,
    redirect_uri: &str,
) -> Result<TokenResponse> {
    let params = [
        ("grant_type", "authorization_code"),
        ("client_id", OPENAI_CLIENT_ID),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("code_verifier", code_verifier),
    ];

    let response = reqwest::Client::new()
        .post(OPENAI_TOKEN_URL)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Accept", "application/json")
        .form(&params)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(Error::TokenExchangeFailed(error));
    }

    let token_response: TokenResponse = response.json().await?;
    Ok(token_response)
}
```

#### JWT Token 解析

```rust
/// 从 ID Token 解析用户信息
fn parse_jwt_claims(token: &str) -> (Option<String>, Option<String>) {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return (None, None);
    }

    // 解码 payload（第二部分）
    let payload = match URL_SAFE_NO_PAD.decode(parts[1]) {
        Ok(bytes) => bytes,
        Err(_) => {
            // 尝试带 padding 的解码
            let padded = format!("{}{}", parts[1], "=".repeat((4 - parts[1].len() % 4) % 4));
            match URL_SAFE.decode(&padded) {
                Ok(bytes) => bytes,
                Err(_) => return (None, None),
            }
        }
    };

    let claims: serde_json::Value = match serde_json::from_slice(&payload) {
        Ok(v) => v,
        Err(_) => return (None, None),
    };

    // 提取 email
    let email = claims["email"].as_str().map(String::from);

    // 提取 account_id（优先级：chatgpt_account_id > user_id > sub）
    let auth_info = &claims["https://api.openai.com/auth"];
    let account_id = auth_info["chatgpt_account_id"].as_str()
        .or_else(|| auth_info["user_id"].as_str())
        .or_else(|| claims["sub"].as_str())
        .map(String::from);

    (account_id, email)
}
```

### 3.2 API Key 认证

```rust
/// API Key 模式验证
impl CodexProvider {
    /// 获取有效的 API Key（trim 后的非空值）
    fn get_api_key(&self) -> Option<&str> {
        self.credentials
            .api_key
            .as_deref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
    }

    /// API Key 模式不需要刷新
    pub fn is_token_expired(&self) -> bool {
        // API Key 模式：不涉及过期概念
        if self.get_api_key().is_some() {
            return false;
        }

        // OAuth 模式：检查过期时间
        if let Some(expires_str) = &self.credentials.expires_at {
            if let Ok(expires) = DateTime::parse_from_rfc3339(expires_str) {
                let now = Utc::now();
                // 提前 5 分钟视为过期
                return expires < now + Duration::minutes(5);
            }
        }
        true
    }
}

/// Base URL 构建逻辑
fn build_responses_url(base_url: &str) -> String {
    let base = base_url.trim_end_matches('/');

    // 规则 1: 以 /v1 结尾 → 直接拼 /responses
    if base.ends_with("/v1") {
        return format!("{}/responses", base);
    }

    // 规则 2: 只有域名 → 拼 /v1/responses
    if let Ok(parsed) = url::Url::parse(base) {
        let path = parsed.path().trim_end_matches('/');
        if path.is_empty() || path == "/" {
            return format!("{}/v1/responses", base);
        }
        // 规则 3: 有路径前缀 → 拼 /responses
        return format!("{}/responses", base);
    }

    // 兜底
    format!("{}/v1/responses", base)
}
```

**Base URL 示例**：

| 输入 Base URL | 输出 API URL |
|---------------|--------------|
| `https://api.openai.com` | `https://api.openai.com/v1/responses` |
| `https://api.openai.com/v1` | `https://api.openai.com/v1/responses` |
| `https://yunyi.cfd/codex` | `https://yunyi.cfd/codex/responses` |

---

## 四、Token 刷新机制

### 4.1 自动刷新逻辑

```rust
impl CodexProvider {
    /// Token 刷新阈值（提前 5 分钟）
    const REFRESH_THRESHOLD: Duration = Duration::minutes(5);

    /// 检查 Token 是否需要刷新
    pub fn needs_refresh(&self, lead_time: Duration) -> bool {
        // API Key 模式无需刷新
        if self.get_api_key().is_some() {
            return false;
        }

        // 无 access_token 需要刷新
        if self.credentials.access_token.is_none() {
            return true;
        }

        // 检查过期时间
        if let Some(expires_str) = &self.credentials.expires_at {
            if let Ok(expires) = DateTime::parse_from_rfc3339(expires_str) {
                return expires < Utc::now() + lead_time;
            }
        }

        // 无过期信息，假设需要刷新
        true
    }

    /// 刷新 Token（带重试）
    pub async fn refresh_token_with_retry(
        &mut self,
        max_retries: u32,
    ) -> Result<String> {
        let mut last_error = None;

        for attempt in 0..max_retries {
            if attempt > 0 {
                // 线性退避：1s, 2s, 3s...
                let delay = Duration::from_secs(attempt as u64);
                tracing::info!(
                    "[CODEX] 重试 {}/{} 等待 {:?}",
                    attempt + 1, max_retries, delay
                );
                tokio::time::sleep(delay).await;
            }

            match self.refresh_token().await {
                Ok(token) => {
                    if attempt > 0 {
                        tracing::info!(
                            "[CODEX] Token 刷新成功（第 {} 次尝试）",
                            attempt + 1
                        );
                    }
                    return Ok(token);
                }
                Err(e) => {
                    tracing::warn!(
                        "[CODEX] Token 刷新失败 {}/{}: {}",
                        attempt + 1, max_retries, e
                    );
                    last_error = Some(e);
                }
            }
        }

        // 所有重试失败，标记凭证无效
        self.mark_invalid();
        Err(last_error.unwrap())
    }

    /// 刷新 Token
    async fn refresh_token(&mut self) -> Result<String> {
        // 1. API Key 模式无需刷新
        if let Some(api_key) = self.get_api_key() {
            return Ok(api_key.to_string());
        }

        // 2. 无 refresh_token 的降级处理
        if self.credentials.refresh_token.is_none() {
            if let Some(ref access_token) = self.credentials.access_token {
                tracing::warn!("[CODEX] 没有 refresh_token，返回现有 access_token");
                return Ok(access_token.clone());
            }
            return Err(Error::MissingCredentials);
        }

        // 3. OAuth 刷新流程
        let refresh_token = self.credentials.refresh_token.as_ref().unwrap();

        let params = [
            ("client_id", OPENAI_CLIENT_ID),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token.as_str()),
            ("scope", "openid profile email"),
        ];

        let response = self.client
            .post(OPENAI_TOKEN_URL)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            self.mark_invalid();
            return Err(Error::TokenRefreshFailed(status, body));
        }

        let data: serde_json::Value = response.json().await?;

        // 更新凭证
        let new_access_token = data["access_token"]
            .as_str()
            .ok_or(Error::MissingAccessToken)?
            .to_string();

        self.credentials.access_token = Some(new_access_token.clone());

        if let Some(rt) = data["refresh_token"].as_str() {
            self.credentials.refresh_token = Some(rt.to_string());
        }

        if let Some(id_token) = data["id_token"].as_str() {
            self.credentials.id_token = Some(id_token.to_string());
            let (account_id, email) = parse_jwt_claims(id_token);
            if account_id.is_some() {
                self.credentials.account_id = account_id;
            }
            if email.is_some() {
                self.credentials.email = email;
            }
        }

        let expires_in = data["expires_in"].as_i64().unwrap_or(3600);
        let expires_at = Utc::now() + Duration::seconds(expires_in);
        self.credentials.expires_at = Some(expires_at.to_rfc3339());
        self.credentials.last_refresh = Some(Utc::now().to_rfc3339());

        // 保存更新后的凭证
        self.save_credentials().await?;

        Ok(new_access_token)
    }
}
```

---

## 五、请求格式转换

### 5.1 OpenAI → Codex 转换

```rust
/// 转换 OpenAI Chat Completion 请求为 Codex 格式
fn transform_to_codex_format(request: &Value) -> Result<Value> {
    let model = request["model"].as_str().unwrap_or("gpt-4o");
    let messages = request["messages"].as_array();
    let stream = request["stream"].as_bool().unwrap_or(true);

    let mut input = Vec::new();
    let mut instructions = None;

    if let Some(msgs) = messages {
        for msg in msgs {
            let role = msg["role"].as_str().unwrap_or("user");
            let content = &msg["content"];

            match role {
                "system" => {
                    // system → instructions
                    if let Some(text) = content.as_str() {
                        instructions = Some(text.to_string());
                    }
                }
                "user" | "assistant" => {
                    // user/assistant → input message
                    let content_parts = if let Some(text) = content.as_str() {
                        vec![json!({"type": "input_text", "text": text})]
                    } else if let Some(arr) = content.as_array() {
                        arr.iter()
                            .filter_map(|part| {
                                part["text"].as_str().map(|text| {
                                    json!({"type": "input_text", "text": text})
                                })
                            })
                            .collect()
                    } else {
                        vec![]
                    };

                    input.push(json!({
                        "type": "message",
                        "role": role,
                        "content": content_parts
                    }));
                }
                "tool" => {
                    // tool → function_call_output
                    let tool_call_id = msg["tool_call_id"].as_str().unwrap_or("");
                    let output = content.as_str().unwrap_or("");
                    input.push(json!({
                        "type": "function_call_output",
                        "call_id": tool_call_id,
                        "output": output
                    }));
                }
                _ => {}
            }
        }
    }

    // 构建 Codex 请求
    let mut codex_request = json!({
        "model": model,
        "input": input,
        "stream": stream
    });

    if let Some(inst) = instructions {
        codex_request["instructions"] = json!(inst);
    }

    // 转换工具定义
    if let Some(tools) = request["tools"].as_array() {
        let codex_tools: Vec<Value> = tools
            .iter()
            .map(|tool| {
                let func = &tool["function"];
                json!({
                    "type": "function",
                    "name": func["name"],
                    "description": func["description"],
                    "parameters": func["parameters"]
                })
            })
            .collect();
        codex_request["tools"] = json!(codex_tools);
    }

    // 转换其他参数
    if let Some(temp) = request["temperature"].as_f64() {
        codex_request["temperature"] = json!(temp);
    }
    if let Some(max_tokens) = request["max_tokens"].as_i64() {
        codex_request["max_output_tokens"] = json!(max_tokens);
    }
    if let Some(top_p) = request["top_p"].as_f64() {
        codex_request["top_p"] = json!(top_p);
    }
    if let Some(reasoning) = request.get("reasoning") {
        codex_request["reasoning"] = reasoning.clone();
    }

    Ok(codex_request)
}
```

### 5.2 转换示例

**输入（OpenAI Chat Completion）**：
```json
{
  "model": "gpt-4o",
  "messages": [
    {"role": "system", "content": "You are a helpful assistant."},
    {"role": "user", "content": "Hello!"}
  ],
  "temperature": 0.7,
  "max_tokens": 1000,
  "stream": true
}
```

**输出（Codex Responses API）**：
```json
{
  "model": "gpt-4o",
  "instructions": "You are a helpful assistant.",
  "input": [
    {
      "type": "message",
      "role": "user",
      "content": [{"type": "input_text", "text": "Hello!"}]
    }
  ],
  "temperature": 0.7,
  "max_output_tokens": 1000,
  "stream": true
}
```

---

## 六、前端 UI 实现

### 6.1 插件入口

```tsx
// src/index.tsx
import { ProxyCastPluginSDK } from '@proxycast/plugin-sdk';
import { CredentialList } from './components/CredentialList';
import { AuthMethodTabs } from './components/AuthMethodTabs';
import { SettingsPanel } from './components/SettingsPanel';

interface PluginProps {
  sdk: ProxyCastPluginSDK;
  pluginId: string;
}

export default function CodexProviderUI({ sdk, pluginId }: PluginProps) {
  const [view, setView] = useState<'list' | 'add' | 'settings'>('list');
  const [credentials, setCredentials] = useState<Credential[]>([]);

  useEffect(() => {
    loadCredentials();
  }, []);

  const loadCredentials = async () => {
    const result = await sdk.database.query<Credential>(
      'SELECT * FROM plugin_credentials WHERE plugin_id = ? ORDER BY created_at DESC',
      [pluginId]
    );
    setCredentials(result);
  };

  return (
    <div className="codex-provider-ui">
      <Header>
        <Title>Codex Provider</Title>
        <Subtitle>支持 OAuth 和 API Key 两种认证方式</Subtitle>
        <Actions>
          <Button onClick={() => setView('add')}>添加凭证</Button>
          <Button onClick={() => setView('settings')}>设置</Button>
        </Actions>
      </Header>

      {view === 'list' && (
        <CredentialList
          credentials={credentials}
          onRefresh={loadCredentials}
          sdk={sdk}
        />
      )}

      {view === 'add' && (
        <AuthMethodTabs
          sdk={sdk}
          onSuccess={() => {
            loadCredentials();
            setView('list');
          }}
          onCancel={() => setView('list')}
        />
      )}

      {view === 'settings' && (
        <SettingsPanel
          sdk={sdk}
          pluginId={pluginId}
          onClose={() => setView('list')}
        />
      )}
    </div>
  );
}
```

### 6.2 认证方式选择

```tsx
// src/components/AuthMethodTabs.tsx

type AuthMethod = 'oauth' | 'api_key' | 'import';

interface AuthMethodTabsProps {
  sdk: ProxyCastPluginSDK;
  onSuccess: () => void;
  onCancel: () => void;
}

export function AuthMethodTabs({ sdk, onSuccess, onCancel }: AuthMethodTabsProps) {
  const [method, setMethod] = useState<AuthMethod>('oauth');

  return (
    <div className="auth-method-tabs">
      <Tabs value={method} onChange={setMethod}>
        <Tab value="oauth">
          <Icon name="Key" />
          OAuth 登录
        </Tab>
        <Tab value="api_key">
          <Icon name="Lock" />
          API Key
        </Tab>
        <Tab value="import">
          <Icon name="Upload" />
          导入凭证
        </Tab>
      </Tabs>

      <div className="tab-content">
        {method === 'oauth' && <OAuthForm sdk={sdk} onSuccess={onSuccess} />}
        {method === 'api_key' && <ApiKeyForm sdk={sdk} onSuccess={onSuccess} />}
        {method === 'import' && <ImportForm sdk={sdk} onSuccess={onSuccess} />}
      </div>

      <FormActions>
        <Button variant="secondary" onClick={onCancel}>取消</Button>
      </FormActions>
    </div>
  );
}
```

### 6.3 OAuth 表单

```tsx
// src/components/OAuthForm.tsx

export function OAuthForm({ sdk, onSuccess }: FormProps) {
  const [authUrl, setAuthUrl] = useState('');
  const [loading, setLoading] = useState(false);
  const [status, setStatus] = useState<'idle' | 'waiting' | 'success' | 'error'>('idle');

  const handleOAuthLogin = async () => {
    setLoading(true);
    setStatus('waiting');
    try {
      // 1. 获取授权 URL
      const result = await sdk.http.request('/api/codex/oauth/start');
      setAuthUrl(result.authUrl);

      // 2. 打开浏览器
      await sdk.shell.open(result.authUrl);

      // 3. 等待回调（5 分钟超时）
      const credential = await sdk.http.request('/api/codex/oauth/callback/wait', {
        timeout: 300000,
      });

      setStatus('success');
      sdk.notification.success(`OAuth 认证成功: ${credential.email || '未知邮箱'}`);
      onSuccess();
    } catch (error) {
      setStatus('error');
      sdk.notification.error(`认证失败: ${error.message}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="oauth-form">
      <Alert type="info">
        <p>使用 ChatGPT Plus/Pro 账户进行 OAuth 授权</p>
        <p className="text-sm text-gray-500">
          注意：OpenAI OAuth 要求使用固定端口 1455，请确保该端口未被占用
        </p>
      </Alert>

      {status === 'waiting' && (
        <div className="waiting-status">
          <Spinner />
          <p>正在等待浏览器授权...</p>
          <p className="text-sm">请在浏览器中完成 OpenAI 账号登录</p>
        </div>
      )}

      {authUrl && (
        <div className="auth-url">
          <Label>授权 URL（如果浏览器未自动打开）</Label>
          <div className="flex gap-2">
            <Input value={authUrl} readOnly className="flex-1" />
            <Button
              size="small"
              onClick={() => {
                navigator.clipboard.writeText(authUrl);
                sdk.notification.success('已复制到剪贴板');
              }}
            >
              复制
            </Button>
          </div>
        </div>
      )}

      <Button
        onClick={handleOAuthLogin}
        loading={loading}
        disabled={status === 'waiting'}
      >
        {status === 'waiting' ? '等待授权中...' : '打开浏览器授权'}
      </Button>
    </div>
  );
}
```

### 6.4 API Key 表单

```tsx
// src/components/ApiKeyForm.tsx

export function ApiKeyForm({ sdk, onSuccess }: FormProps) {
  const [form, setForm] = useState({
    apiKey: '',
    apiBaseUrl: '',
    name: '',
  });
  const [loading, setLoading] = useState(false);
  const [testResult, setTestResult] = useState<'idle' | 'testing' | 'success' | 'error'>('idle');

  const handleTest = async () => {
    setTestResult('testing');
    try {
      await sdk.http.request('/api/codex/apikey/test', {
        method: 'POST',
        body: JSON.stringify({
          apiKey: form.apiKey,
          apiBaseUrl: form.apiBaseUrl || undefined,
        }),
      });
      setTestResult('success');
      sdk.notification.success('API Key 验证成功');
    } catch (error) {
      setTestResult('error');
      sdk.notification.error(`验证失败: ${error.message}`);
    }
  };

  const handleSubmit = async () => {
    setLoading(true);
    try {
      await sdk.http.request('/api/codex/apikey/add', {
        method: 'POST',
        body: JSON.stringify(form),
      });
      sdk.notification.success('API Key 添加成功');
      onSuccess();
    } catch (error) {
      sdk.notification.error(`添加失败: ${error.message}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="api-key-form">
      <FormField>
        <Label>凭证名称（可选）</Label>
        <Input
          value={form.name}
          onChange={(e) => setForm({ ...form, name: e.target.value })}
          placeholder="我的 OpenAI API Key"
        />
      </FormField>

      <FormField>
        <Label>API Key *</Label>
        <Input
          type="password"
          value={form.apiKey}
          onChange={(e) => setForm({ ...form, apiKey: e.target.value })}
          placeholder="sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
        />
        <HelpText>以 sk- 开头的 OpenAI API Key</HelpText>
      </FormField>

      <FormField>
        <Label>Base URL（可选）</Label>
        <Input
          value={form.apiBaseUrl}
          onChange={(e) => setForm({ ...form, apiBaseUrl: e.target.value })}
          placeholder="https://api.openai.com"
        />
        <HelpText>
          留空使用官方 API，或填写第三方兼容 API 地址
        </HelpText>
      </FormField>

      {!form.apiKey.startsWith('sk-') && form.apiKey && !form.apiBaseUrl && (
        <Alert type="warning">
          API Key 不是以 sk- 开头，但未配置自定义 Base URL。
          如果使用第三方服务，请填写 Base URL。
        </Alert>
      )}

      <div className="flex gap-2">
        <Button
          variant="secondary"
          onClick={handleTest}
          loading={testResult === 'testing'}
          disabled={!form.apiKey}
        >
          {testResult === 'success' ? '✓ 验证成功' : '验证 API Key'}
        </Button>
        <Button
          onClick={handleSubmit}
          loading={loading}
          disabled={!form.apiKey}
        >
          添加凭证
        </Button>
      </div>
    </div>
  );
}
```

### 6.5 导入表单

```tsx
// src/components/ImportForm.tsx

type ImportMode = 'codex_cli' | 'file' | 'paste';

export function ImportForm({ sdk, onSuccess }: FormProps) {
  const [mode, setMode] = useState<ImportMode>('codex_cli');
  const [jsonContent, setJsonContent] = useState('');
  const [loading, setLoading] = useState(false);

  const handleImportCodexCli = async () => {
    setLoading(true);
    try {
      await sdk.http.request('/api/codex/import/codex-cli', {
        method: 'POST',
      });
      sdk.notification.success('从 ~/.codex/auth.json 导入成功');
      onSuccess();
    } catch (error) {
      sdk.notification.error(`导入失败: ${error.message}`);
    } finally {
      setLoading(false);
    }
  };

  const handleImportJson = async () => {
    setLoading(true);
    try {
      const parsed = JSON.parse(jsonContent);
      await sdk.http.request('/api/codex/import/json', {
        method: 'POST',
        body: JSON.stringify(parsed),
      });
      sdk.notification.success('JSON 导入成功');
      onSuccess();
    } catch (error) {
      sdk.notification.error(`导入失败: ${error.message}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="import-form">
      <Tabs value={mode} onChange={setMode}>
        <Tab value="codex_cli">从 Codex CLI 导入</Tab>
        <Tab value="paste">粘贴 JSON</Tab>
        <Tab value="file">选择文件</Tab>
      </Tabs>

      {mode === 'codex_cli' && (
        <div className="codex-cli-import">
          <Alert type="info">
            <p>从默认路径导入 Codex CLI 凭证</p>
            <code>~/.codex/auth.json</code>
          </Alert>
          <Button onClick={handleImportCodexCli} loading={loading}>
            导入 Codex CLI 凭证
          </Button>
        </div>
      )}

      {mode === 'paste' && (
        <div className="json-paste">
          <FormField>
            <Label>凭证 JSON</Label>
            <TextArea
              rows={8}
              value={jsonContent}
              onChange={(e) => setJsonContent(e.target.value)}
              placeholder={`{
  "refresh_token": "...",
  "access_token": "...",
  "email": "user@example.com"
}`}
            />
            <HelpText>
              支持 snake_case 和 camelCase 两种格式
            </HelpText>
          </FormField>
          <Button
            onClick={handleImportJson}
            loading={loading}
            disabled={!jsonContent}
          >
            导入
          </Button>
        </div>
      )}

      {mode === 'file' && (
        <FileUpload
          accept=".json"
          onFile={async (file) => {
            const content = await file.text();
            setJsonContent(content);
            setMode('paste');
          }}
        />
      )}
    </div>
  );
}
```

### 6.6 凭证卡片

```tsx
// src/components/CredentialCard.tsx

const AUTH_TYPE_LABELS: Record<string, string> = {
  oauth: 'OAuth',
  api_key: 'API Key',
};

const AUTH_TYPE_COLORS: Record<string, string> = {
  oauth: 'green',
  api_key: 'blue',
};

export function CredentialCard({ credential, onRefresh, onDelete }: CredentialCardProps) {
  const data = JSON.parse(credential.credential_data);
  const authType = data.api_key ? 'api_key' : 'oauth';
  const isHealthy = credential.status === 'active';

  return (
    <Card className={`credential-card ${isHealthy ? 'healthy' : 'unhealthy'}`}>
      <CardHeader>
        <div className="status-indicator">
          <StatusDot status={isHealthy ? 'green' : 'red'} />
          <span>{isHealthy ? '健康' : '异常'}</span>
        </div>
        <Badge color={AUTH_TYPE_COLORS[authType]}>
          {AUTH_TYPE_LABELS[authType]}
        </Badge>
      </CardHeader>

      <CardBody>
        <div className="info-row">
          <label>名称</label>
          <span>{credential.name || '未命名'}</span>
        </div>

        {data.email && (
          <div className="info-row">
            <label>邮箱</label>
            <span>{data.email}</span>
          </div>
        )}

        {data.api_key && (
          <div className="info-row">
            <label>API Key</label>
            <span className="truncate">
              {data.api_key.substring(0, 8)}...{data.api_key.slice(-4)}
            </span>
          </div>
        )}

        {data.api_base_url && (
          <div className="info-row">
            <label>Base URL</label>
            <span className="truncate">{data.api_base_url}</span>
          </div>
        )}

        {data.expires_at && (
          <div className="info-row">
            <label>过期时间</label>
            <span>{formatDate(data.expires_at)}</span>
          </div>
        )}

        {data.last_refresh && (
          <div className="info-row">
            <label>最后刷新</label>
            <span>{formatRelativeTime(data.last_refresh)}</span>
          </div>
        )}
      </CardBody>

      <CardFooter>
        {authType === 'oauth' && (
          <Button size="small" onClick={onRefresh}>刷新 Token</Button>
        )}
        {authType === 'api_key' && (
          <Button size="small" onClick={() => {}}>测试</Button>
        )}
        <Button size="small" variant="danger" onClick={onDelete}>删除</Button>
      </CardFooter>
    </Card>
  );
}
```

---

## 七、凭证文件格式

### 7.1 OAuth 凭证

```json
{
  "id_token": "eyJhbGciOiJSUzI1NiIs...",
  "access_token": "eyJhbGciOiJSUzI1NiIs...",
  "refresh_token": "v1.MjAyNS0wMS0xNVQx...",
  "account_id": "acc_xxxxxxxxxxxxxxxx",
  "last_refresh": "2025-01-15T10:30:00Z",
  "email": "user@example.com",
  "type": "codex",
  "expires_at": "2025-01-15T14:30:00Z"
}
```

### 7.2 API Key 凭证

```json
{
  "api_key": "sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
  "api_base_url": "https://api.openai.com",
  "type": "codex"
}
```

### 7.3 字段别名支持

| 标准字段名 | 支持的别名 | 说明 |
|------------|-----------|------|
| `id_token` | `idToken` | JWT ID Token |
| `access_token` | `accessToken` | OAuth Access Token |
| `refresh_token` | `refreshToken` | OAuth Refresh Token |
| `api_key` | `apiKey`, `OPENAI_API_KEY` | API Key |
| `api_base_url` | `apiBaseUrl` | 自定义 Base URL |
| `account_id` | `accountId` | 账户 ID |
| `last_refresh` | `lastRefresh` | 最后刷新时间 |
| `expires_at` | `expired`, `expiresAt` | 过期时间 |

---

## 八、错误处理

### 8.1 错误类型

| 错误类型 | 说明 | 处理方式 |
|---------|------|---------|
| `MissingCredentials` | 无可用凭证 | 添加凭证 |
| `TokenRefreshFailed` | Token 刷新失败 | 重试或重新授权 |
| `InvalidApiKey` | API Key 无效 | 检查 Key 配置 |
| `PortInUse` | 端口 1455 被占用 | 关闭占用端口的应用 |
| `OAuthTimeout` | OAuth 超时（5分钟） | 重新尝试 |
| `StateValidationFailed` | CSRF 验证失败 | 重新尝试 |

### 8.2 错误消息

```rust
// 无凭证错误
"没有可用的认证凭证。请配置以下任一方式：
 1. API Key 模式：在凭证文件中添加 api_key/apiKey 字段
 2. OAuth 模式：使用 OAuth 登录获取 refresh_token
 3. Access Token 模式：在凭证文件中添加 access_token/accessToken 字段"

// 端口占用错误
"端口 1455 已被占用。OpenAI OAuth 要求使用固定端口 1455，
 请关闭占用该端口的应用后重试。"

// API Key 格式警告
"[CODEX] API key does not appear to be an OpenAI key (doesn't start with 'sk-'),
 but no api_base_url is configured. Requests will be sent to https://api.openai.com.
 If you're using a third-party API provider, please add 'api_base_url' to the config."
```

---

## 九、开发指南

### 9.1 本地开发

```bash
# 克隆仓库
git clone https://github.com/aiclientproxy/codex-provider.git
cd codex-provider

# 安装依赖
pnpm install
cd src-tauri && cargo build

# 前端开发
pnpm dev

# 后端开发
cargo watch -x run
```

### 9.2 测试

```bash
# 单元测试
cargo test

# OAuth 流程测试
cargo test --test oauth

# 请求转换测试
cargo test transform_to_codex_format

# 前端测试
pnpm test
```

---

## 十、SSE 响应解析

### 10.1 SSE 事件格式

OpenAI Responses API 返回的 SSE 事件格式：

```
data: {"type":"response.created","response":{...}}

data: {"type":"response.output_item.added",...}

data: {"type":"response.content_part.added",...}

data: {"type":"response.output_text.delta","delta":"Hello"}

data: {"type":"response.completed","response":{"id":"...","model":"gpt-4o","usage":{...}}}

data: [DONE]
```

### 10.2 解析 Usage 数据

```rust
/// 从 SSE 事件中提取 usage 数据
fn parse_sse_for_usage(data: &str) -> Option<UsageData> {
    for line in data.lines() {
        if !line.starts_with("data:") {
            continue;
        }

        let json_str = line[5..].trim();
        if json_str == "[DONE]" {
            continue;
        }

        if let Ok(event) = serde_json::from_str::<Value>(json_str) {
            // 检查 response.completed 事件
            if event["type"] == "response.completed" {
                if let Some(response) = event.get("response") {
                    // 获取真实 model
                    let model = response["model"].as_str();

                    // 获取 usage 数据
                    if let Some(usage) = response.get("usage") {
                        return Some(UsageData {
                            input_tokens: usage["input_tokens"].as_i64().unwrap_or(0),
                            output_tokens: usage["output_tokens"].as_i64().unwrap_or(0),
                            total_tokens: usage["total_tokens"].as_i64().unwrap_or(0),
                            cached_tokens: usage["input_tokens_details"]["cached_tokens"]
                                .as_i64().unwrap_or(0),
                            cache_creation_tokens: extract_cache_creation_tokens(usage),
                            model: model.map(String::from),
                        });
                    }
                }
            }

            // 检查流中的限流错误
            if let Some(error) = event.get("error") {
                let error_type = error["type"].as_str().unwrap_or("");
                if error_type == "rate_limit_error"
                    || error_type == "usage_limit_reached"
                    || error_type == "rate_limit_exceeded"
                {
                    // 限流错误，提取 resets_in_seconds
                    let resets_in = error["resets_in_seconds"].as_i64();
                    // 触发限流处理...
                }
            }
        }
    }

    None
}

/// 提取缓存写入 tokens（兼容多种字段命名）
fn extract_cache_creation_tokens(usage: &Value) -> i64 {
    let details = usage.get("input_tokens_details")
        .or_else(|| usage.get("prompt_tokens_details"));

    let candidates = [
        details.and_then(|d| d["cache_creation_input_tokens"].as_i64()),
        details.and_then(|d| d["cache_creation_tokens"].as_i64()),
        usage["cache_creation_input_tokens"].as_i64(),
        usage["cache_creation_tokens"].as_i64(),
    ];

    for value in candidates {
        if let Some(v) = value {
            return v;
        }
    }

    0
}
```

### 10.3 Usage 数据结构

```rust
#[derive(Debug, Clone)]
pub struct UsageData {
    /// 总输入 tokens（包含缓存）
    pub input_tokens: i64,
    /// 输出 tokens
    pub output_tokens: i64,
    /// 总 tokens
    pub total_tokens: i64,
    /// 缓存读取 tokens
    pub cached_tokens: i64,
    /// 缓存写入 tokens
    pub cache_creation_tokens: i64,
    /// 实际使用的模型
    pub model: Option<String>,
}

impl UsageData {
    /// 计算实际输入 tokens（不含缓存）
    pub fn actual_input_tokens(&self) -> i64 {
        (self.input_tokens - self.cached_tokens).max(0)
    }
}
```

---

## 十一、限流处理

### 11.1 429 错误处理

```rust
/// 处理 429 限流错误
async fn handle_429_error(
    &self,
    account: &Account,
    response: &Response,
    is_stream: bool,
    session_hash: Option<&str>,
) -> Result<(Option<i64>, Value)> {
    let mut resets_in_seconds: Option<i64> = None;
    let mut error_data: Option<Value> = None;

    // 解析错误响应
    if is_stream {
        // 流式响应需要先收集数据
        let body = collect_stream_body(response).await?;

        // 尝试解析 SSE 格式
        if body.contains("data: ") {
            for line in body.lines() {
                if line.starts_with("data: ") {
                    let json_str = line[6..].trim();
                    if let Ok(data) = serde_json::from_str::<Value>(json_str) {
                        error_data = Some(data);
                        break;
                    }
                }
            }
        }

        // 尝试直接解析 JSON
        if error_data.is_none() {
            error_data = serde_json::from_str(&body).ok();
        }
    } else {
        error_data = response.json().await.ok();
    }

    // 从响应中提取重置时间
    if let Some(ref data) = error_data {
        if let Some(error) = data.get("error") {
            // OpenAI 标准格式
            if let Some(secs) = error["resets_in_seconds"].as_i64() {
                resets_in_seconds = Some(secs);
                tracing::info!(
                    "🕐 Rate limit will reset in {} seconds ({} minutes)",
                    secs, secs / 60
                );
            }
            // 备用字段名
            else if let Some(secs) = error["resets_in"].as_i64() {
                resets_in_seconds = Some(secs);
            }
        }
    }

    // 标记账户为限流状态
    self.mark_account_rate_limited(
        &account.id,
        session_hash,
        resets_in_seconds,
    ).await?;

    Ok((resets_in_seconds, error_data.unwrap_or(json!({
        "error": {
            "message": "Rate limit exceeded",
            "type": "rate_limit_error",
            "code": "rate_limit_exceeded"
        }
    }))))
}
```

### 11.2 限流状态管理

```rust
/// 标记账户为限流状态
async fn mark_account_rate_limited(
    &self,
    account_id: &str,
    session_hash: Option<&str>,
    resets_in_seconds: Option<i64>,
) -> Result<()> {
    let duration_minutes = resets_in_seconds
        .map(|s| (s as f64 / 60.0).ceil() as i64)
        .unwrap_or(60); // 默认 60 分钟

    let now = Utc::now();
    let reset_at = now + Duration::minutes(duration_minutes);

    // 更新账户状态
    self.update_credential(account_id, |cred| {
        cred.rate_limited_at = Some(now.to_rfc3339());
        cred.rate_limit_reset_at = Some(reset_at.to_rfc3339());
        cred.status = CredentialStatus::RateLimited;
        cred.schedulable = false;
        cred.error_message = Some(format!(
            "Rate limited until {}",
            reset_at.format("%Y-%m-%d %H:%M:%S")
        ));
    }).await?;

    tracing::warn!(
        "⏳ Account {} rate limited for {} minutes (until {})",
        account_id, duration_minutes, reset_at
    );

    Ok(())
}

/// 检查并清除过期的限流状态
async fn check_and_clear_rate_limit(&self, account_id: &str) -> Result<bool> {
    let account = self.get_credential(account_id).await?;

    if account.status != CredentialStatus::RateLimited {
        return Ok(false);
    }

    let now = Utc::now();
    let should_clear = if let Some(reset_at) = &account.rate_limit_reset_at {
        DateTime::parse_from_rfc3339(reset_at)
            .map(|t| now >= t)
            .unwrap_or(true)
    } else {
        true
    };

    if should_clear {
        self.update_credential(account_id, |cred| {
            cred.rate_limited_at = None;
            cred.rate_limit_reset_at = None;
            cred.status = CredentialStatus::Active;
            cred.schedulable = true;
            cred.error_message = None;
        }).await?;

        tracing::info!("✅ Rate limit cleared for account {}", account_id);
        return Ok(true);
    }

    Ok(false)
}
```

---

## 十二、凭证加密

### 12.1 加密实现

```rust
use aes::cipher::{BlockEncrypt, BlockDecrypt, KeyInit};
use aes::Aes256;
use crypto::scrypt::{scrypt, ScryptParams};

const ENCRYPTION_ALGORITHM: &str = "aes-256-cbc";
const ENCRYPTION_SALT: &[u8] = b"codex-provider-salt";

/// 加密敏感数据
fn encrypt_sensitive_data(text: &str, master_key: &str) -> Result<String> {
    if text.is_empty() {
        return Ok(String::new());
    }

    // 派生加密密钥
    let key = derive_encryption_key(master_key)?;

    // 生成随机 IV
    let mut iv = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut iv);

    // AES-256-CBC 加密
    let cipher = Aes256Cbc::new_from_slices(&key, &iv)?;
    let encrypted = cipher.encrypt_vec(text.as_bytes());

    // 返回格式：IV:EncryptedData（十六进制）
    Ok(format!("{}:{}", hex::encode(iv), hex::encode(encrypted)))
}

/// 解密敏感数据
fn decrypt_sensitive_data(text: &str, master_key: &str) -> Result<String> {
    if text.is_empty() {
        return Ok(String::new());
    }

    let parts: Vec<&str> = text.split(':').collect();
    if parts.len() != 2 {
        return Err(Error::InvalidEncryptedFormat);
    }

    let iv = hex::decode(parts[0])?;
    let encrypted = hex::decode(parts[1])?;

    // 派生加密密钥
    let key = derive_encryption_key(master_key)?;

    // AES-256-CBC 解密
    let cipher = Aes256Cbc::new_from_slices(&key, &iv)?;
    let decrypted = cipher.decrypt_vec(&encrypted)?;

    Ok(String::from_utf8(decrypted)?)
}

/// 使用 scrypt 派生加密密钥
fn derive_encryption_key(master_key: &str) -> Result<[u8; 32]> {
    let mut key = [0u8; 32];
    let params = ScryptParams::new(14, 8, 1)?;
    scrypt(master_key.as_bytes(), ENCRYPTION_SALT, &params, &mut key)?;
    Ok(key)
}
```

### 12.2 凭证加密字段

| 字段 | 是否加密 | 说明 |
|------|---------|------|
| `api_key` | ✅ 是 | API Key 敏感数据 |
| `access_token` | ✅ 是 | OAuth Access Token |
| `refresh_token` | ✅ 是 | OAuth Refresh Token |
| `id_token` | ✅ 是 | JWT ID Token |
| `email` | ❌ 否 | 用户标识，非敏感 |
| `account_id` | ❌ 否 | 账户 ID，非敏感 |

---

## 十三、额度管理

### 13.1 每日额度配置

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaConfig {
    /// 每日额度限制（美元），0 表示不限制
    pub daily_quota: f64,
    /// 当前已使用额度
    pub daily_usage: f64,
    /// 最后重置日期
    pub last_reset_date: String,
    /// 额度重置时间（HH:mm 格式）
    pub quota_reset_time: String,
    /// 额度超限停止时间
    pub quota_stopped_at: Option<String>,
}

impl QuotaConfig {
    /// 检查并重置每日额度
    pub fn check_and_reset(&mut self) -> bool {
        let today = get_date_string_in_timezone();
        if self.last_reset_date != today {
            self.daily_usage = 0.0;
            self.last_reset_date = today;
            self.quota_stopped_at = None;
            return true;
        }
        false
    }

    /// 更新使用额度
    pub fn update_usage(&mut self, amount: f64) -> bool {
        self.daily_usage += amount;

        // 检查是否超出额度
        if self.daily_quota > 0.0 && self.daily_usage >= self.daily_quota {
            self.quota_stopped_at = Some(Utc::now().to_rfc3339());
            return true; // 返回 true 表示超限
        }

        false
    }
}
```

### 13.2 费用计算

```rust
/// 计算 API 调用费用（考虑缓存 token 的不同价格）
pub fn calculate_cost(usage: &UsageData, model: &str) -> CostInfo {
    let pricing = get_model_pricing(model);

    // 实际输入（不含缓存）
    let actual_input = usage.actual_input_tokens();

    let input_cost = (actual_input as f64 / 1_000_000.0) * pricing.input_per_million;
    let output_cost = (usage.output_tokens as f64 / 1_000_000.0) * pricing.output_per_million;

    // 缓存读取通常有折扣（如 50%）
    let cache_read_cost = (usage.cached_tokens as f64 / 1_000_000.0)
        * pricing.input_per_million * 0.5;

    // 缓存写入通常有额外费用（如 25%）
    let cache_write_cost = (usage.cache_creation_tokens as f64 / 1_000_000.0)
        * pricing.input_per_million * 1.25;

    CostInfo {
        input_cost,
        output_cost,
        cache_read_cost,
        cache_write_cost,
        total: input_cost + output_cost + cache_read_cost + cache_write_cost,
    }
}
```

---

## 附录

### A. 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `CODEX_CLIENT_ID` | OAuth Client ID | 内置 |
| `CODEX_DEBUG` | 调试模式 | `false` |
| `CODEX_TIMEOUT_MS` | 请求超时 | `60000` |

### B. 参考链接

- [OpenAI API 文档](https://platform.openai.com/docs/api-reference)
- [Codex CLI 项目](https://github.com/openai/codex)
- [OAuth 2.0 PKCE 规范](https://datatracker.ietf.org/doc/html/rfc7636)
- [ProxyCast 插件开发指南](../prd/credential-provider-plugin-architecture.md)
