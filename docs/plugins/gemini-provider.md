# Gemini Provider 插件文档

> 版本: 1.0.0
> 仓库: `aiclientproxy/gemini-provider`
> 类型: OAuth Provider Plugin

---

## 一、概述

### 1.1 插件简介

Gemini Provider 是 ProxyCast 的 Google Gemini 凭证提供者插件，支持 **OAuth 2.0 + PKCE**（Gemini CLI 兼容）和 **API Key**（Google AI Studio）两种认证方式。插件通过 Google Code Assist API 访问 Gemini 模型，支持个人 Google 账户和 Google Cloud/Workspace 企业账户。

### 1.2 支持的认证方式

| 认证方式 | 说明 | 适用场景 |
|---------|------|---------|
| **OAuth** | Google OAuth 2.0 + PKCE | 个人/企业 Google 账户 |
| **API Key** | Google AI Studio API Key | 标准 API 访问 |

### 1.3 核心能力

| 能力 | 说明 |
|------|------|
| 双重认证支持 | OAuth 和 API Key 两种模式 |
| 自动 Token 刷新 | OAuth 模式自动刷新，提前 5 分钟 |
| PKCE 安全 | OAuth 使用 S256 PKCE 流程 |
| Project ID 自动发现 | 通过 loadCodeAssist 获取临时 projectId |
| 请求格式转换 | OpenAI Chat Completion → Gemini Content |
| 代理支持 | HTTP/HTTPS/SOCKS5 代理 + TCP Keep-Alive |
| 凭证加密 | AES-256-CBC + scrypt + LRU 缓存 |

### 1.4 支持的模型

| 模型系列 | 示例模型 | 说明 |
|----------|----------|------|
| Gemini 2.5 | `gemini-2.5-pro`, `gemini-2.5-flash` | 最新旗舰/快速模型 |
| Gemini 2.0 | `gemini-2.0-flash-exp` | 实验性 Flash 模型 |
| Gemini 3 | `gemini-3-pro-preview` | 预览版 |

---

## 二、插件架构

### 2.1 项目结构

```
gemini-provider/
├── plugin/
│   ├── plugin.json              # 插件元数据
│   └── config.json              # 默认配置
│
├── src-tauri/src/               # 后端 Rust 代码
│   ├── lib.rs                   # 插件入口
│   ├── commands.rs              # Tauri 命令
│   ├── provider.rs              # GeminiProvider 核心实现
│   ├── auth/                    # 认证模块
│   │   ├── mod.rs
│   │   ├── oauth.rs             # Google OAuth 2.0 + PKCE
│   │   └── api_key.rs           # API Key 模式
│   ├── credentials.rs           # 凭证管理
│   ├── token_refresh.rs         # Token 刷新
│   ├── code_assist.rs           # Code Assist API 封装
│   ├── transform.rs             # 请求格式转换
│   └── api/                     # API 调用
│       ├── mod.rs
│       ├── code_assist.rs       # Code Assist API
│       └── generative.rs        # Generative Language API
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
  "name": "gemini-provider",
  "version": "1.0.0",
  "description": "Gemini Provider - 支持 OAuth 和 API Key 两种认证方式访问 Google Gemini 模型",
  "author": "ProxyCast Team",
  "homepage": "https://github.com/aiclientproxy/gemini-provider",
  "license": "MIT",

  "plugin_type": "oauth_provider",
  "entry": "gemini-provider-cli",
  "min_proxycast_version": "1.0.0",

  "provider": {
    "id": "gemini",
    "display_name": "Gemini (Google)",
    "target_protocol": "gemini",
    "supported_models": ["gemini-*"],
    "auth_types": ["oauth", "api_key"],
    "credential_schemas": {
      "oauth": {
        "type": "object",
        "properties": {
          "access_token": { "type": "string" },
          "refresh_token": { "type": "string" },
          "expiry_date": { "type": "integer" },
          "expire": { "type": "string" },
          "scope": { "type": "string" },
          "email": { "type": "string" },
          "project_id": { "type": "string" },
          "temp_project_id": { "type": "string" }
        },
        "required": ["access_token"]
      },
      "api_key": {
        "type": "object",
        "properties": {
          "api_key": { "type": "string" },
          "base_url": { "type": "string" }
        },
        "required": ["api_key"]
      }
    }
  },

  "binary": {
    "binary_name": "gemini-provider-cli",
    "github_owner": "aiclientproxy",
    "github_repo": "gemini-provider",
    "platform_binaries": {
      "macos-arm64": "gemini-provider-aarch64-apple-darwin",
      "macos-x64": "gemini-provider-x86_64-apple-darwin",
      "linux-x64": "gemini-provider-x86_64-unknown-linux-gnu",
      "windows-x64": "gemini-provider-x86_64-pc-windows-msvc.exe"
    },
    "checksum_file": "checksums.txt"
  },

  "ui": {
    "surfaces": ["oauth_providers"],
    "icon": "Sparkles",
    "title": "Gemini Provider",
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
  "timeout_ms": 600000,
  "settings": {
    "oauth": {
      "client_id": "681255809395-oo8ft2oprdrnp9e3aqf6av3hmdib135j.apps.googleusercontent.com",
      "client_secret": "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",
      "scopes": ["https://www.googleapis.com/auth/cloud-platform"],
      "redirect_uri": "https://codeassist.google.com/authcode",
      "token_url": "https://oauth2.googleapis.com/token",
      "auth_url": "https://accounts.google.com/o/oauth2/v2/auth"
    },
    "api": {
      "code_assist_endpoint": "https://cloudcode-pa.googleapis.com",
      "code_assist_version": "v1internal",
      "generative_endpoint": "https://generativelanguage.googleapis.com/v1beta"
    },
    "token_refresh": {
      "auto_refresh": true,
      "refresh_threshold_minutes": 5,
      "max_retry": 3,
      "retry_delay_ms": 1000
    },
    "network": {
      "keep_alive": true,
      "keep_alive_msecs": 30000,
      "timeout": 120000,
      "max_sockets": 100,
      "max_free_sockets": 10
    }
  }
}
```

---

## 三、认证方式详解

### 3.1 OAuth 认证（Google OAuth 2.0 + PKCE）

#### OAuth 配置

```rust
// Gemini CLI OAuth 配置 - 公开的 Gemini CLI 凭据
const OAUTH_CLIENT_ID: &str =
    "681255809395-oo8ft2oprdrnp9e3aqf6av3hmdib135j.apps.googleusercontent.com";
const OAUTH_CLIENT_SECRET: &str = "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";
const OAUTH_SCOPES: &[&str] = &["https://www.googleapis.com/auth/cloud-platform"];
const OAUTH_REDIRECT_URI: &str = "https://codeassist.google.com/authcode";
const OAUTH_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const OAUTH_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
```

#### PKCE 流程实现

```rust
use sha2::{Sha256, Digest};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

/// 生成 PKCE code_verifier 和 code_challenge
fn generate_pkce() -> (String, String) {
    // 生成 43-128 字符的随机字符串作为 code_verifier
    let code_verifier: String = (0..64)
        .map(|_| {
            let idx = rand::random::<u8>() % 66;
            let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
            chars[idx as usize] as char
        })
        .collect();

    // 计算 code_challenge = BASE64URL(SHA256(code_verifier))
    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let hash = hasher.finalize();
    let code_challenge = URL_SAFE_NO_PAD.encode(hash);

    (code_verifier, code_challenge)
}

/// 生成 OAuth 授权 URL（使用 PKCE）
pub fn generate_auth_url(state: &str, code_challenge: &str) -> String {
    let scopes = OAUTH_SCOPES.join(" ");

    let params = [
        ("access_type", "offline"),
        ("client_id", OAUTH_CLIENT_ID),
        ("code_challenge", code_challenge),
        ("code_challenge_method", "S256"),
        ("prompt", "select_account"),
        ("redirect_uri", OAUTH_REDIRECT_URI),
        ("response_type", "code"),
        ("scope", &scopes),
        ("state", state),
    ];

    let query = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    format!("{}?{}", OAUTH_AUTH_URL, query)
}
```

#### Token 交换

```rust
/// 交换授权码获取 tokens (支持 PKCE)
pub async fn exchange_code_for_tokens(
    code: &str,
    redirect_uri: &str,
    code_verifier: &str,
) -> Result<TokenResponse> {
    let params = [
        ("code", code),
        ("client_id", OAUTH_CLIENT_ID),
        ("client_secret", OAUTH_CLIENT_SECRET),
        ("code_verifier", code_verifier),
        ("redirect_uri", redirect_uri),
        ("grant_type", "authorization_code"),
    ];

    let client = reqwest::Client::new();
    let resp = client
        .post(OAUTH_TOKEN_URL)
        .form(&params)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(Error::TokenExchangeFailed(status.as_u16(), body));
    }

    let data: serde_json::Value = resp.json().await?;

    Ok(TokenResponse {
        access_token: data["access_token"].as_str().unwrap().to_string(),
        refresh_token: data["refresh_token"].as_str().map(String::from),
        scope: data["scope"].as_str().map(String::from),
        token_type: data["token_type"].as_str().unwrap_or("Bearer").to_string(),
        expiry_date: data["expires_in"]
            .as_i64()
            .map(|secs| Utc::now().timestamp_millis() + secs * 1000),
    })
}
```

#### 获取用户信息

```rust
/// 获取用户邮箱
pub async fn fetch_user_email(access_token: &str) -> Result<Option<String>> {
    let resp = reqwest::Client::new()
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?;

    if resp.status().is_success() {
        let data: serde_json::Value = resp.json().await?;
        Ok(data["email"].as_str().map(String::from))
    } else {
        Ok(None)
    }
}
```

### 3.2 API Key 认证

```rust
/// Gemini API Key Provider
pub struct GeminiApiKeyCredential {
    /// API Key
    pub api_key: String,
    /// Custom base URL (optional)
    pub base_url: Option<String>,
    /// Excluded models (supports wildcards)
    pub excluded_models: Vec<String>,
    /// Whether this credential is disabled
    pub disabled: bool,
}

impl GeminiApiKeyCredential {
    /// Get the effective base URL
    pub fn get_base_url(&self) -> &str {
        self.base_url.as_deref()
            .unwrap_or("https://generativelanguage.googleapis.com")
    }

    /// Build the API URL for a given model and action
    pub fn build_api_url(&self, model: &str, action: &str) -> String {
        format!(
            "{}/v1beta/models/{}:{}",
            self.get_base_url(),
            model,
            action
        )
    }

    /// Check if this credential supports the given model
    pub fn supports_model(&self, model: &str) -> bool {
        !self.excluded_models.iter().any(|pattern| {
            if pattern.contains('*') {
                let regex_pattern = pattern.replace('*', ".*");
                regex::Regex::new(&format!("^{}$", regex_pattern))
                    .map(|re| re.is_match(model))
                    .unwrap_or(false)
            } else {
                pattern == model
            }
        })
    }
}
```

---

## 四、Code Assist API

### 4.1 API 端点

```rust
const CODE_ASSIST_ENDPOINT: &str = "https://cloudcode-pa.googleapis.com";
const CODE_ASSIST_API_VERSION: &str = "v1internal";
```

### 4.2 loadCodeAssist - 获取 Project ID

```rust
/// 调用 loadCodeAssist 获取用户配置和 Project ID
pub async fn load_code_assist(
    access_token: &str,
    project_id: Option<&str>,
) -> Result<LoadCodeAssistResponse> {
    // 对于个人账户（无 projectId），先调用 tokeninfo/userinfo
    // 帮助 Google 获取临时 projectId
    if project_id.is_none() {
        // 验证 token
        let _ = reqwest::Client::new()
            .post("https://oauth2.googleapis.com/tokeninfo")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[("access_token", access_token)])
            .send()
            .await;

        // 获取用户信息
        let _ = reqwest::Client::new()
            .get("https://www.googleapis.com/oauth2/v2/userinfo")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await;
    }

    // 构建请求
    let mut request = json!({
        "metadata": {
            "ideType": "IDE_UNSPECIFIED",
            "platform": "PLATFORM_UNSPECIFIED",
            "pluginType": "GEMINI"
        }
    });

    if let Some(pid) = project_id {
        request["cloudaicompanionProject"] = json!(pid);
        request["metadata"]["duetProject"] = json!(pid);
    }

    let resp = reqwest::Client::new()
        .post(format!(
            "{}/{}:loadCodeAssist",
            CODE_ASSIST_ENDPOINT, CODE_ASSIST_API_VERSION
        ))
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    let data: serde_json::Value = resp.json().await?;

    Ok(LoadCodeAssistResponse {
        cloud_ai_companion_project: data["cloudaicompanionProject"]
            .as_str()
            .map(String::from),
        current_tier: data.get("currentTier").cloned(),
        allowed_tiers: data.get("allowedTiers").cloned(),
    })
}
```

### 4.3 onboardUser - 用户注册

```rust
/// 用户层级枚举
#[derive(Debug, Clone)]
pub enum UserTier {
    Legacy,
    Free,
    Pro,
}

/// 获取 onboard 层级
fn get_onboard_tier(load_res: &LoadCodeAssistResponse) -> UserTier {
    if let Some(current) = &load_res.current_tier {
        return match current["id"].as_str() {
            Some("PRO") => UserTier::Pro,
            Some("FREE") => UserTier::Free,
            _ => UserTier::Legacy,
        };
    }

    if let Some(tiers) = &load_res.allowed_tiers {
        if let Some(arr) = tiers.as_array() {
            for tier in arr {
                if tier["isDefault"].as_bool().unwrap_or(false) {
                    return match tier["id"].as_str() {
                        Some("PRO") => UserTier::Pro,
                        Some("FREE") => UserTier::Free,
                        _ => UserTier::Legacy,
                    };
                }
            }
        }
    }

    UserTier::Legacy
}

/// 调用 onboardUser（包含轮询逻辑）
pub async fn onboard_user(
    access_token: &str,
    tier_id: &str,
    project_id: Option<&str>,
) -> Result<OnboardResponse> {
    let mut request = json!({
        "tierId": tier_id,
        "metadata": {
            "ideType": "IDE_UNSPECIFIED",
            "platform": "PLATFORM_UNSPECIFIED",
            "pluginType": "GEMINI"
        }
    });

    if let Some(pid) = project_id {
        request["cloudaicompanionProject"] = json!(pid);
    }

    let client = reqwest::Client::new();
    let url = format!(
        "{}/{}:onboardUser",
        CODE_ASSIST_ENDPOINT, CODE_ASSIST_API_VERSION
    );

    // 轮询直到长运行操作完成
    let mut lro_resp: serde_json::Value;
    let mut attempts = 0;
    const MAX_ATTEMPTS: u32 = 12; // 最多等待 1 分钟

    loop {
        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        lro_resp = resp.json().await?;

        if lro_resp["done"].as_bool().unwrap_or(false) {
            break;
        }

        attempts += 1;
        if attempts >= MAX_ATTEMPTS {
            return Err(Error::OnboardTimeout);
        }

        tracing::info!("Waiting for onboardUser... ({}/{})", attempts, MAX_ATTEMPTS);
        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    Ok(OnboardResponse {
        project_id: lro_resp["response"]["cloudaicompanionProject"]["id"]
            .as_str()
            .map(String::from),
    })
}
```

### 4.4 generateContent - 生成内容

```rust
/// 调用 generateContent（流式）
pub async fn stream_generate_content(
    access_token: &str,
    request: &GeminiRequest,
    project_id: Option<&str>,
    session_id: Option<&str>,
) -> Result<impl Stream<Item = Result<Bytes>>> {
    let mut body = json!({
        "model": request.model,
        "request": request.request
    });

    if let Some(sid) = session_id {
        body["request"]["session_id"] = json!(sid);
    }

    if let Some(pid) = project_id {
        body["project"] = json!(pid);
    }

    let url = format!(
        "{}/{}:streamGenerateContent?alt=sse",
        CODE_ASSIST_ENDPOINT, CODE_ASSIST_API_VERSION
    );

    let resp = reqwest::Client::new()
        .post(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(Error::ApiError(status.as_u16(), body));
    }

    Ok(resp.bytes_stream())
}
```

---

## 五、Token 刷新机制

### 5.1 Token 有效性检查

```rust
impl GeminiProvider {
    /// 检查 Token 是否有效
    pub fn is_token_valid(&self) -> bool {
        if self.credentials.access_token.is_none() {
            return false;
        }

        // 优先检查 RFC3339 格式的过期时间
        if let Some(expire_str) = &self.credentials.expire {
            if let Ok(expires) = DateTime::parse_from_rfc3339(expire_str) {
                let now = Utc::now();
                // Token 有效期需要超过 5 分钟
                return expires > now + Duration::minutes(5);
            }
        }

        // 兼容旧的毫秒时间戳格式
        if let Some(expiry) = self.credentials.expiry_date {
            let now = Utc::now().timestamp_millis();
            return expiry > now + 300_000; // 5 分钟
        }

        true
    }
}
```

### 5.2 Token 刷新

```rust
/// 刷新访问令牌
pub async fn refresh_access_token(
    refresh_token: &str,
) -> Result<TokenResponse> {
    let params = [
        ("client_id", OAUTH_CLIENT_ID),
        ("client_secret", OAUTH_CLIENT_SECRET),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
    ];

    let resp = reqwest::Client::new()
        .post(OAUTH_TOKEN_URL)
        .form(&params)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(Error::TokenRefreshFailed(status.as_u16(), body));
    }

    let data: serde_json::Value = resp.json().await?;

    Ok(TokenResponse {
        access_token: data["access_token"]
            .as_str()
            .ok_or(Error::MissingAccessToken)?
            .to_string(),
        refresh_token: data["refresh_token"].as_str().map(String::from),
        expiry_date: data["expires_in"]
            .as_i64()
            .map(|secs| Utc::now().timestamp_millis() + secs * 1000),
        ..Default::default()
    })
}
```

### 5.3 分布式锁防止并发刷新

```rust
/// 获取刷新锁
pub async fn acquire_refresh_lock(
    account_id: &str,
    platform: &str,
) -> Result<bool> {
    let lock_key = format!("token_refresh_lock:{}:{}", platform, account_id);

    // 使用 Redis SETNX 实现分布式锁
    let result: bool = redis_client
        .set_nx(&lock_key, "1", Duration::from_secs(30))
        .await?;

    Ok(result)
}

/// 释放刷新锁
pub async fn release_refresh_lock(
    account_id: &str,
    platform: &str,
) -> Result<()> {
    let lock_key = format!("token_refresh_lock:{}:{}", platform, account_id);
    redis_client.del(&lock_key).await?;
    Ok(())
}

/// 带分布式锁的 Token 刷新
pub async fn refresh_account_token(account_id: &str) -> Result<TokenResponse> {
    let mut lock_acquired = false;

    // 尝试获取锁
    lock_acquired = acquire_refresh_lock(account_id, "gemini").await?;

    if !lock_acquired {
        tracing::info!(
            "Token refresh already in progress for account: {}",
            account_id
        );
        // 等待其他进程完成
        tokio::time::sleep(Duration::from_secs(2)).await;
        // 返回更新后的凭证
        return get_account_credentials(account_id).await;
    }

    let result = async {
        let account = get_account(account_id).await?;
        let refresh_token = account.refresh_token
            .ok_or(Error::NoRefreshToken)?;

        let new_tokens = refresh_access_token(&refresh_token).await?;

        // 更新账户
        update_account(account_id, &new_tokens).await?;

        Ok(new_tokens)
    }.await;

    // 释放锁
    if lock_acquired {
        let _ = release_refresh_lock(account_id, "gemini").await;
    }

    result
}
```

---

## 六、请求格式转换

### 6.1 OpenAI → Gemini 转换

```rust
/// 转换 OpenAI 消息格式到 Gemini 格式
pub fn convert_messages_to_gemini(
    messages: &[Message],
) -> (Vec<GeminiContent>, Option<String>) {
    let mut contents = Vec::new();
    let mut system_instruction = String::new();

    for message in messages {
        match message.role.as_str() {
            "system" => {
                if !system_instruction.is_empty() {
                    system_instruction.push_str("\n\n");
                }
                system_instruction.push_str(&message.content);
            }
            "user" => {
                contents.push(GeminiContent {
                    role: "user".to_string(),
                    parts: vec![GeminiPart {
                        text: Some(message.content.clone()),
                    }],
                });
            }
            "assistant" => {
                contents.push(GeminiContent {
                    role: "model".to_string(),
                    parts: vec![GeminiPart {
                        text: Some(message.content.clone()),
                    }],
                });
            }
            _ => {}
        }
    }

    let system = if system_instruction.is_empty() {
        None
    } else {
        Some(system_instruction)
    };

    (contents, system)
}
```

### 6.2 Gemini → OpenAI 转换

```rust
/// 转换 Gemini 响应到 OpenAI 格式
pub fn convert_gemini_response(
    gemini_response: &GeminiResponse,
    model: &str,
    stream: bool,
) -> Option<ChatCompletionResponse> {
    let candidate = gemini_response.candidates.as_ref()?.first()?;

    if stream {
        // 流式响应
        let content = candidate.content.as_ref()?
            .parts.first()?
            .text.clone()
            .unwrap_or_default();

        let finish_reason = candidate.finish_reason.as_ref()
            .map(|r| r.to_lowercase())
            .filter(|r| r == "stop")
            .map(|_| "stop".to_string());

        Some(ChatCompletionResponse {
            id: format!("chatcmpl-{}", Utc::now().timestamp_millis()),
            object: "chat.completion.chunk".to_string(),
            created: Utc::now().timestamp(),
            model: model.to_string(),
            choices: vec![Choice {
                index: 0,
                delta: Some(Delta { content: Some(content) }),
                message: None,
                finish_reason,
            }],
            usage: None,
        })
    } else {
        // 非流式响应
        let content = candidate.content.as_ref()?
            .parts.first()?
            .text.clone()
            .unwrap_or_default();

        let usage = gemini_response.usage_metadata.as_ref().map(|u| Usage {
            prompt_tokens: u.prompt_token_count.unwrap_or(0),
            completion_tokens: u.candidates_token_count.unwrap_or(0),
            total_tokens: u.total_token_count.unwrap_or(0),
        });

        Some(ChatCompletionResponse {
            id: format!("chatcmpl-{}", Utc::now().timestamp_millis()),
            object: "chat.completion".to_string(),
            created: Utc::now().timestamp(),
            model: model.to_string(),
            choices: vec![Choice {
                index: 0,
                message: Some(Message {
                    role: "assistant".to_string(),
                    content,
                }),
                delta: None,
                finish_reason: Some("stop".to_string()),
            }],
            usage,
        })
    }
}
```

---

## 七、前端 UI 实现

### 7.1 插件入口

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

export default function GeminiProviderUI({ sdk, pluginId }: PluginProps) {
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
    <div className="gemini-provider-ui">
      <Header>
        <Title>Gemini Provider</Title>
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

### 7.2 OAuth 表单

```tsx
// src/components/OAuthForm.tsx

export function OAuthForm({ sdk, onSuccess }: FormProps) {
  const [loading, setLoading] = useState(false);
  const [status, setStatus] = useState<'idle' | 'waiting' | 'success' | 'error'>('idle');
  const [authUrl, setAuthUrl] = useState('');

  const handleOAuthLogin = async () => {
    setLoading(true);
    setStatus('waiting');
    try {
      // 1. 获取授权 URL
      const result = await sdk.http.request('/api/gemini/oauth/start');
      setAuthUrl(result.authUrl);

      // 2. 打开浏览器
      await sdk.shell.open(result.authUrl);

      // 3. 等待回调
      const credential = await sdk.http.request('/api/gemini/oauth/callback/wait', {
        timeout: 300000, // 5 分钟超时
      });

      setStatus('success');
      sdk.notification.success(
        `OAuth 认证成功: ${credential.email || '未知邮箱'}`
      );
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
        <p>使用 Google 账户进行 OAuth 授权</p>
        <p className="text-sm text-gray-500">
          支持个人 Google 账户和 Google Cloud/Workspace 企业账户
        </p>
      </Alert>

      {status === 'waiting' && (
        <div className="waiting-status">
          <Spinner />
          <p>正在等待浏览器授权...</p>
          <p className="text-sm">请在浏览器中完成 Google 账号登录</p>
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
        className="google-signin-btn"
      >
        <GoogleIcon />
        {status === 'waiting' ? '等待授权中...' : '使用 Google 账号登录'}
      </Button>
    </div>
  );
}
```

### 7.3 API Key 表单

```tsx
// src/components/ApiKeyForm.tsx

export function ApiKeyForm({ sdk, onSuccess }: FormProps) {
  const [form, setForm] = useState({
    apiKey: '',
    baseUrl: '',
    name: '',
  });
  const [loading, setLoading] = useState(false);

  const handleSubmit = async () => {
    setLoading(true);
    try {
      // 测试 API Key
      const testResult = await sdk.http.request('/api/gemini/apikey/test', {
        method: 'POST',
        body: JSON.stringify({
          apiKey: form.apiKey,
          baseUrl: form.baseUrl || undefined,
        }),
      });

      if (!testResult.success) {
        throw new Error(testResult.error || 'API Key 验证失败');
      }

      // 保存凭证
      await sdk.http.request('/api/gemini/apikey/add', {
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
          placeholder="我的 Gemini API Key"
        />
      </FormField>

      <FormField>
        <Label>API Key *</Label>
        <Input
          type="password"
          value={form.apiKey}
          onChange={(e) => setForm({ ...form, apiKey: e.target.value })}
          placeholder="AIzaSy..."
        />
        <HelpText>
          从 <a href="https://aistudio.google.com/app/apikey" target="_blank">
            Google AI Studio
          </a> 获取 API Key
        </HelpText>
      </FormField>

      <FormField>
        <Label>Base URL（可选）</Label>
        <Input
          value={form.baseUrl}
          onChange={(e) => setForm({ ...form, baseUrl: e.target.value })}
          placeholder="https://generativelanguage.googleapis.com"
        />
        <HelpText>留空使用官方 API</HelpText>
      </FormField>

      <Button onClick={handleSubmit} loading={loading} disabled={!form.apiKey}>
        添加凭证
      </Button>
    </div>
  );
}
```

---

## 八、凭证文件格式

### 8.1 OAuth 凭证

```json
{
  "access_token": "ya29.a0AfH6...",
  "refresh_token": "1//0g...",
  "token_type": "Bearer",
  "expiry_date": 1704067200000,
  "expire": "2025-01-01T12:00:00Z",
  "scope": "https://www.googleapis.com/auth/cloud-platform",
  "email": "user@gmail.com",
  "last_refresh": "2025-01-01T10:00:00Z",
  "type": "gemini",
  "project_id": null,
  "temp_project_id": "cloudai-companion-xxxx"
}
```

### 8.2 API Key 凭证

```json
{
  "api_key": "AIzaSyXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
  "base_url": "https://generativelanguage.googleapis.com",
  "type": "gemini_api_key"
}
```

### 8.3 嵌套 Token 格式（兼容）

```json
{
  "token": {
    "access_token": "ya29.a0AfH6...",
    "refresh_token": "1//0g...",
    "token_uri": "https://oauth2.googleapis.com/token",
    "client_id": "681255809395-oo8ft2oprdrnp9e3aqf6av3hmdib135j.apps.googleusercontent.com",
    "client_secret": "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",
    "scopes": ["https://www.googleapis.com/auth/cloud-platform"]
  },
  "type": "gemini"
}
```

---

## 九、凭证加密

### 9.1 加密实现

```rust
use aes::cipher::{BlockEncrypt, BlockDecrypt, KeyInit};
use aes::Aes256;
use crypto::scrypt::{scrypt, ScryptParams};

const ALGORITHM: &str = "aes-256-cbc";
const ENCRYPTION_SALT: &[u8] = b"gemini-account-salt";
const IV_LENGTH: usize = 16;

// 缓存派生的加密密钥，避免每次重复计算
static ENCRYPTION_KEY_CACHE: OnceCell<[u8; 32]> = OnceCell::new();

/// 生成加密密钥
fn generate_encryption_key(master_key: &str) -> &'static [u8; 32] {
    ENCRYPTION_KEY_CACHE.get_or_init(|| {
        let mut key = [0u8; 32];
        let params = ScryptParams::new(14, 8, 1).unwrap();
        scrypt(master_key.as_bytes(), ENCRYPTION_SALT, &params, &mut key).unwrap();
        tracing::info!("Gemini encryption key derived and cached");
        key
    })
}

/// 加密敏感数据
fn encrypt(text: &str, master_key: &str) -> String {
    if text.is_empty() {
        return String::new();
    }

    let key = generate_encryption_key(master_key);
    let mut iv = [0u8; IV_LENGTH];
    rand::thread_rng().fill_bytes(&mut iv);

    let cipher = Aes256Cbc::new_from_slices(key, &iv).unwrap();
    let encrypted = cipher.encrypt_vec(text.as_bytes());

    format!("{}:{}", hex::encode(iv), hex::encode(encrypted))
}

/// 解密敏感数据（带 LRU 缓存）
fn decrypt(text: &str, master_key: &str, cache: &LruCache<String, String>) -> String {
    if text.is_empty() {
        return String::new();
    }

    // 检查缓存
    let cache_key = sha256_hash(text);
    if let Some(cached) = cache.get(&cache_key) {
        return cached.clone();
    }

    let key = generate_encryption_key(master_key);
    let parts: Vec<&str> = text.split(':').collect();
    if parts.len() != 2 {
        return String::new();
    }

    let iv = hex::decode(parts[0]).unwrap_or_default();
    let encrypted = hex::decode(parts[1]).unwrap_or_default();

    let cipher = Aes256Cbc::new_from_slices(key, &iv).unwrap();
    let decrypted = cipher.decrypt_vec(&encrypted).unwrap_or_default();
    let result = String::from_utf8(decrypted).unwrap_or_default();

    // 存入缓存（5分钟过期）
    cache.insert(cache_key, result.clone(), Duration::from_secs(300));

    result
}
```

### 9.2 加密字段

| 字段 | 是否加密 | 说明 |
|------|---------|------|
| `access_token` | ✅ 是 | OAuth Access Token |
| `refresh_token` | ✅ 是 | OAuth Refresh Token |
| `api_key` | ✅ 是 | API Key |
| `email` | ❌ 否 | 用户标识 |
| `project_id` | ❌ 否 | 项目 ID |

---

## 十、限流处理

### 10.1 限流状态管理

```rust
/// 设置账户限流状态
pub async fn set_account_rate_limited(
    account_id: &str,
    is_limited: bool,
) -> Result<()> {
    let updates = if is_limited {
        json!({
            "rateLimitStatus": "limited",
            "rateLimitedAt": Utc::now().to_rfc3339()
        })
    } else {
        json!({
            "rateLimitStatus": "",
            "rateLimitedAt": ""
        })
    };

    update_account(account_id, updates).await
}

/// 获取账户限流信息
pub async fn get_account_rate_limit_info(
    account_id: &str,
) -> Result<RateLimitInfo> {
    let account = get_account(account_id).await?;

    if account.rate_limit_status == "limited" {
        if let Some(limited_at) = account.rate_limited_at {
            let limited_at = DateTime::parse_from_rfc3339(&limited_at)?;
            let now = Utc::now();
            let minutes_since = (now - limited_at).num_minutes();

            // Gemini 限流持续时间为 1 小时
            let minutes_remaining = (60 - minutes_since).max(0);
            let is_rate_limited = minutes_remaining > 0;

            return Ok(RateLimitInfo {
                is_rate_limited,
                rate_limited_at: Some(limited_at.to_rfc3339()),
                minutes_remaining,
                rate_limit_end_at: Some(
                    (limited_at + Duration::hours(1)).to_rfc3339()
                ),
            });
        }
    }

    Ok(RateLimitInfo::default())
}
```

### 10.2 检查账户是否被限流

```rust
/// 检查账户是否被限流
fn is_rate_limited(account: &Account) -> bool {
    if account.rate_limit_status != "limited" {
        return false;
    }

    if let Some(limited_at) = &account.rate_limited_at {
        if let Ok(limited_at) = DateTime::parse_from_rfc3339(limited_at) {
            let now = Utc::now();
            let limit_duration = Duration::hours(1);
            return now < limited_at + limit_duration;
        }
    }

    false
}
```

---

## 十一、订阅过期管理

### 11.1 订阅过期检查

```rust
/// 检查账户订阅是否过期
fn is_subscription_expired(account: &Account) -> bool {
    if let Some(expires_at) = &account.subscription_expires_at {
        if let Ok(expiry) = DateTime::parse_from_rfc3339(expires_at) {
            return expiry <= Utc::now();
        }
    }
    // 未设置视为永不过期
    false
}
```

### 11.2 Token 过期 vs 订阅过期

```rust
// Token 过期时间（技术字段，自动刷新）
account.expires_at

// 订阅过期时间（业务字段，手动管理）
account.subscription_expires_at

// 前端显示订阅过期时间
{
  "tokenExpiresAt": account.expires_at,
  "subscriptionExpiresAt": account.subscription_expires_at,
  "expiresAt": account.subscription_expires_at  // 前端显示用
}
```

---

## 十二、代理支持

### 12.1 TCP Keep-Alive Agent

```rust
/// TCP Keep-Alive Agent 配置
/// 解决长时间流式请求中 NAT/防火墙空闲超时导致的连接中断问题
pub fn create_keep_alive_agent() -> reqwest::Client {
    reqwest::Client::builder()
        .tcp_keepalive(Duration::from_secs(30))
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Duration::from_secs(120))
        .timeout(Duration::from_secs(600))
        .build()
        .unwrap()
}
```

### 12.2 代理配置

```rust
/// 创建代理 Agent
pub fn create_proxy_agent(proxy_config: &ProxyConfig) -> Option<reqwest::Proxy> {
    match proxy_config.protocol.as_str() {
        "http" | "https" => {
            let url = format!(
                "{}://{}:{}",
                proxy_config.protocol,
                proxy_config.host,
                proxy_config.port
            );
            let mut proxy = reqwest::Proxy::all(&url).ok()?;

            if let (Some(user), Some(pass)) = (&proxy_config.username, &proxy_config.password) {
                proxy = proxy.basic_auth(user, pass);
            }

            Some(proxy)
        }
        "socks5" => {
            let url = format!(
                "socks5://{}:{}",
                proxy_config.host,
                proxy_config.port
            );
            reqwest::Proxy::all(&url).ok()
        }
        _ => None,
    }
}
```

---

## 十三、错误处理

### 13.1 错误类型

| 错误类型 | 说明 | 处理方式 |
|---------|------|---------|
| `MissingCredentials` | 无可用凭证 | 添加凭证 |
| `TokenRefreshFailed` | Token 刷新失败 | 重试或重新授权 |
| `InvalidApiKey` | API Key 无效 | 检查 Key 配置 |
| `ProjectIdRequired` | 需要 Project ID | 设置 GOOGLE_CLOUD_PROJECT |
| `OnboardTimeout` | 用户注册超时 | 重试 |
| `RateLimited` | 被限流 | 等待 1 小时 |
| `SubscriptionExpired` | 订阅已过期 | 续费或更换账户 |

### 13.2 Webhook 通知

```rust
/// 发送账户异常 Webhook 通知
async fn send_account_anomaly_notification(
    account_id: &str,
    account_name: &str,
    status: &str,
    error_code: &str,
    reason: &str,
) {
    let payload = json!({
        "accountId": account_id,
        "accountName": account_name,
        "platform": "gemini",
        "status": status,
        "errorCode": error_code,
        "reason": reason,
        "timestamp": Utc::now().to_rfc3339()
    });

    // 发送到配置的 Webhook URL
    if let Some(webhook_url) = config.webhook_url {
        let _ = reqwest::Client::new()
            .post(&webhook_url)
            .json(&payload)
            .send()
            .await;
    }
}
```

---

## 十四、开发指南

### 14.1 本地开发

```bash
# 克隆仓库
git clone https://github.com/aiclientproxy/gemini-provider.git
cd gemini-provider

# 安装依赖
pnpm install
cd src-tauri && cargo build

# 前端开发
pnpm dev

# 后端开发
cargo watch -x run
```

### 14.2 测试

```bash
# 单元测试
cargo test

# OAuth 流程测试
cargo test --test oauth

# API 调用测试
cargo test --test api

# 前端测试
pnpm test
```

---

## 附录

### A. 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `GOOGLE_CLOUD_PROJECT` | Google Cloud 项目 ID | - |
| `GEMINI_DEBUG` | 调试模式 | `false` |
| `GEMINI_TIMEOUT_MS` | 请求超时 | `600000` |

### B. 参考链接

- [Gemini API 文档](https://ai.google.dev/docs)
- [Google Cloud OAuth 2.0](https://developers.google.com/identity/protocols/oauth2)
- [Gemini CLI 项目](https://github.com/google/gemini-cli)
- [ProxyCast 插件开发指南](../prd/credential-provider-plugin-architecture.md)
