# Claude Provider 插件文档

> 版本: 1.0.0
> 仓库: `aiclientproxy/claude-provider`
> 类型: OAuth Provider Plugin

---

## 一、概述

### 1.1 插件简介

Claude Provider 是 ProxyCast 的综合性 Anthropic/Claude 插件，支持 **多种认证方式** 访问 Claude 模型。无论是官方 OAuth、Claude Code、Console、AWS Bedrock 还是第三方中转服务，都可以通过此插件统一管理。

### 1.2 支持的认证方式

| 认证方式 | 说明 | 适用场景 |
|---------|------|---------|
| **OAuth** | 标准 OAuth 2.0 + PKCE | Claude.ai 个人账户 |
| **Claude Code** | Claude Code CLI 认证 | 开发者工具 |
| **Console** | Anthropic Console OAuth | 企业/团队账户 |
| **Setup Token** | 只读推理 Token | 最小权限场景 |
| **Bedrock** | AWS Bedrock Claude | AWS 云服务 |
| **CCR** | 第三方中转服务 | 自定义 API 端点 |

### 1.3 核心能力

| 能力 | 说明 |
|------|------|
| 多认证统一管理 | 一个插件管理所有 Claude 访问方式 |
| 自动 Token 刷新 | OAuth 类型自动刷新，提前 5 分钟 |
| PKCE 安全 | OAuth 使用 PKCE 流程确保安全 |
| Cookie 快速授权 | 使用 sessionKey 自动完成 OAuth |
| 凭证加密存储 | AES-256 加密敏感信息 |
| 健康检查 | 凭证池级别健康监控 |

### 1.4 支持的模型

| 模型 | 说明 |
|------|------|
| `claude-opus-4-20250514` | Claude Opus 4 最新版 |
| `claude-opus-4-5-20251101` | Claude Opus 4.5 |
| `claude-sonnet-4-5-20250929` | Claude Sonnet 4.5 |
| `claude-sonnet-4-20250514` | Claude Sonnet 4 |
| `claude-haiku-3-5-20241022` | Claude Haiku 3.5 |

---

## 二、插件架构

### 2.1 项目结构

```
claude-provider/
├── plugin/
│   ├── plugin.json              # 插件元数据
│   └── config.json              # 默认配置
│
├── src-tauri/src/               # 后端 Rust 代码
│   ├── lib.rs                   # 插件入口
│   ├── commands.rs              # Tauri 命令
│   ├── provider.rs              # ClaudeProvider 核心实现
│   ├── auth/                    # 认证模块
│   │   ├── mod.rs
│   │   ├── oauth.rs             # OAuth 2.0 + PKCE
│   │   ├── claude_code.rs       # Claude Code 认证
│   │   ├── console.rs           # Console OAuth
│   │   ├── setup_token.rs       # Setup Token
│   │   ├── bedrock.rs           # AWS Bedrock
│   │   └── ccr.rs               # 第三方中转
│   ├── credentials.rs           # 凭证管理
│   ├── token_refresh.rs         # Token 刷新
│   └── api/                     # API 调用
│       ├── mod.rs
│       ├── anthropic.rs         # Anthropic API
│       └── bedrock.rs           # Bedrock API
│
├── src/                         # 前端 React UI
│   ├── index.tsx                # 插件 UI 入口
│   ├── components/
│   │   ├── CredentialList.tsx   # 凭证列表
│   │   ├── CredentialCard.tsx   # 凭证卡片
│   │   ├── AuthMethodTabs.tsx   # 认证方式选择
│   │   ├── OAuthForm.tsx        # OAuth 表单
│   │   ├── ClaudeCodeForm.tsx   # Claude Code 表单
│   │   ├── ConsoleForm.tsx      # Console 表单
│   │   ├── BedrockForm.tsx      # Bedrock 表单
│   │   ├── CCRForm.tsx          # CCR 表单
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
  "name": "claude-provider",
  "version": "1.0.0",
  "description": "Claude Provider - 支持 OAuth、Claude Code、Console、Bedrock、CCR 多种认证方式",
  "author": "ProxyCast Team",
  "homepage": "https://github.com/aiclientproxy/claude-provider",
  "license": "MIT",

  "plugin_type": "oauth_provider",
  "entry": "claude-provider-cli",
  "min_proxycast_version": "1.0.0",

  "provider": {
    "id": "claude",
    "display_name": "Claude (Anthropic)",
    "target_protocol": "anthropic",
    "supported_models": ["claude-*"],
    "auth_types": ["oauth", "claude_code", "console", "setup_token", "bedrock", "ccr"],
    "credential_schemas": {
      "oauth": {
        "type": "object",
        "properties": {
          "access_token": { "type": "string" },
          "refresh_token": { "type": "string" },
          "email": { "type": "string" },
          "expire": { "type": "string" }
        },
        "required": ["access_token", "refresh_token"]
      },
      "claude_code": {
        "type": "object",
        "properties": {
          "access_token": { "type": "string" },
          "refresh_token": { "type": "string" },
          "session_key": { "type": "string" }
        }
      },
      "console": {
        "type": "object",
        "properties": {
          "access_token": { "type": "string" },
          "refresh_token": { "type": "string" },
          "organization_id": { "type": "string" }
        }
      },
      "setup_token": {
        "type": "object",
        "properties": {
          "access_token": { "type": "string" }
        },
        "required": ["access_token"]
      },
      "bedrock": {
        "type": "object",
        "properties": {
          "access_key_id": { "type": "string" },
          "secret_access_key": { "type": "string" },
          "session_token": { "type": "string" },
          "region": { "type": "string", "default": "us-east-1" }
        },
        "required": ["access_key_id", "secret_access_key", "region"]
      },
      "ccr": {
        "type": "object",
        "properties": {
          "api_key": { "type": "string" },
          "base_url": { "type": "string" }
        },
        "required": ["api_key", "base_url"]
      }
    }
  },

  "binary": {
    "binary_name": "claude-provider-cli",
    "github_owner": "aiclientproxy",
    "github_repo": "claude-provider",
    "platform_binaries": {
      "macos-arm64": "claude-provider-aarch64-apple-darwin",
      "macos-x64": "claude-provider-x86_64-apple-darwin",
      "linux-x64": "claude-provider-x86_64-unknown-linux-gnu",
      "windows-x64": "claude-provider-x86_64-pc-windows-msvc.exe"
    },
    "checksum_file": "checksums.txt"
  },

  "ui": {
    "surfaces": ["oauth_providers"],
    "icon": "MessageSquare",
    "title": "Claude Provider",
    "entry": "dist/index.js",
    "styles": "dist/styles.css",
    "default_width": 950,
    "default_height": 750,
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
      "client_id": "9d1c250a-e61b-44d9-88ed-5944d1962f5e",
      "auth_url": "https://claude.ai/oauth/authorize",
      "token_url": "https://console.anthropic.com/v1/oauth/token",
      "redirect_uri": "https://console.anthropic.com/oauth/code/callback",
      "scopes": "org:create_api_key user:profile user:inference",
      "scopes_setup": "user:inference"
    },
    "api": {
      "base_url": "https://api.anthropic.com",
      "version": "2023-06-01"
    },
    "bedrock": {
      "default_region": "us-east-1",
      "model_prefix": "us.anthropic."
    },
    "token_refresh": {
      "auto_refresh": true,
      "refresh_threshold_minutes": 5,
      "max_retry": 3,
      "retry_delay_ms": 1000
    },
    "encryption": {
      "algorithm": "aes-256-cbc",
      "key_derivation": "pbkdf2"
    }
  }
}
```

---

## 三、认证方式详解

### 3.1 OAuth 认证（标准 Claude.ai）

#### OAuth 配置

```rust
const CLAUDE_AUTH_URL: &str = "https://claude.ai/oauth/authorize";
const CLAUDE_TOKEN_URL: &str = "https://console.anthropic.com/v1/oauth/token";
const CLAUDE_CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
const CLAUDE_REDIRECT_URI: &str = "https://console.anthropic.com/oauth/code/callback";
const CLAUDE_SCOPES: &str = "org:create_api_key user:profile user:inference";
```

#### PKCE 流程实现

```rust
use sha2::{Sha256, Digest};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::Rng;

/// 生成 OAuth 参数（PKCE）
pub fn generate_oauth_params() -> OAuthParams {
    // 1. 生成随机 state
    let state: [u8; 32] = rand::thread_rng().gen();
    let state = URL_SAFE_NO_PAD.encode(&state);

    // 2. 生成 code_verifier
    let code_verifier: [u8; 32] = rand::thread_rng().gen();
    let code_verifier = URL_SAFE_NO_PAD.encode(&code_verifier);

    // 3. 计算 code_challenge = SHA256(code_verifier)
    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let code_challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());

    // 4. 构建授权 URL
    let auth_url = format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
        CLAUDE_AUTH_URL,
        CLAUDE_CLIENT_ID,
        urlencoding::encode(CLAUDE_REDIRECT_URI),
        urlencoding::encode(CLAUDE_SCOPES),
        state,
        code_challenge
    );

    OAuthParams {
        auth_url,
        code_verifier,
        state,
        code_challenge,
    }
}

/// 交换授权码获取 Token
pub async fn exchange_authorization_code(
    authorization_code: &str,
    code_verifier: &str,
    state: &str,
) -> Result<OAuthTokens> {
    let response = reqwest::Client::new()
        .post(CLAUDE_TOKEN_URL)
        .json(&json!({
            "client_id": CLAUDE_CLIENT_ID,
            "grant_type": "authorization_code",
            "code": authorization_code,
            "redirect_uri": CLAUDE_REDIRECT_URI,
            "code_verifier": code_verifier,
            "state": state
        }))
        .send()
        .await?;

    let token_response: TokenResponse = response.json().await?;

    Ok(OAuthTokens {
        access_token: token_response.access_token,
        refresh_token: token_response.refresh_token,
        expires_at: Utc::now() + Duration::seconds(token_response.expires_in),
        email: token_response.account.map(|a| a.email_address),
    })
}
```

#### Cookie 快速授权

```rust
/// 使用 sessionKey 自动完成 OAuth 流程
pub async fn oauth_with_cookie(
    session_key: &str,
    is_setup_token: bool,
) -> Result<OAuthTokens> {
    let client = reqwest::Client::new();

    // 1. 获取组织信息
    let orgs_response = client
        .get("https://claude.ai/api/organizations")
        .header("Cookie", format!("sessionKey={}", session_key))
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)")
        .header("Origin", "https://claude.ai")
        .header("Referer", "https://claude.ai/new")
        .send()
        .await?;

    let organizations: Vec<Organization> = orgs_response.json().await?;

    // 2. 选择具有 chat 能力的组织
    let org = organizations
        .iter()
        .find(|o| o.capabilities.contains(&"chat".to_string()))
        .ok_or(Error::NoValidOrganization)?;

    // 3. 生成 OAuth 参数
    let params = generate_oauth_params();
    let scopes = if is_setup_token { CLAUDE_SCOPES_SETUP } else { CLAUDE_SCOPES };

    // 4. 使用 Cookie 请求授权码
    let auth_url = format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
        CLAUDE_AUTH_URL, CLAUDE_CLIENT_ID,
        urlencoding::encode(CLAUDE_REDIRECT_URI),
        urlencoding::encode(scopes),
        params.state, params.code_challenge
    );

    let auth_response = client
        .get(&auth_url)
        .header("Cookie", format!("sessionKey={}", session_key))
        .send()
        .await?;

    // 5. 解析回调中的授权码
    let callback_url = auth_response.url().to_string();
    let code = extract_code_from_url(&callback_url)?;

    // 6. 交换 Token
    exchange_authorization_code(&code, &params.code_verifier, &params.state).await
}
```

### 3.2 Claude Code 认证

```rust
/// Claude Code 凭证结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeCodeCredentials {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub session_key: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// 默认凭证路径
const CLAUDE_CODE_CREDS_PATH: &str = "~/.claude/oauth_creds.json";

/// 从 Claude Code 配置加载凭证
pub fn load_claude_code_credentials() -> Result<ClaudeCodeCredentials> {
    let path = expand_tilde(CLAUDE_CODE_CREDS_PATH);
    let content = fs::read_to_string(&path)?;
    let creds: ClaudeCodeCredentials = serde_json::from_str(&content)?;
    Ok(creds)
}
```

### 3.3 Console OAuth（企业/团队）

```rust
/// Console OAuth 配置
const CONSOLE_AUTH_URL: &str = "https://console.anthropic.com/oauth/authorize";
const CONSOLE_TOKEN_URL: &str = "https://console.anthropic.com/v1/oauth/token";

/// Console 凭证结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleCredentials {
    pub access_token: String,
    pub refresh_token: String,
    pub organization_id: Option<String>,
    pub organization_name: Option<String>,
    pub expires_at: DateTime<Utc>,
}

/// Console OAuth 流程（与标准 OAuth 类似，但针对企业账户）
pub async fn console_oauth_flow(
    authorization_code: &str,
    code_verifier: &str,
) -> Result<ConsoleCredentials> {
    // 与标准 OAuth 类似，但返回组织信息
    let response = exchange_authorization_code(authorization_code, code_verifier).await?;

    Ok(ConsoleCredentials {
        access_token: response.access_token,
        refresh_token: response.refresh_token,
        organization_id: response.organization.map(|o| o.id),
        organization_name: response.organization.map(|o| o.name),
        expires_at: response.expires_at,
    })
}
```

### 3.4 Setup Token（最小权限）

```rust
/// Setup Token - 只有推理权限，无 refresh_token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupTokenCredentials {
    pub access_token: String,
    pub creds_type: String,  // "claude_setup_token"
}

/// 使用 Cookie 获取 Setup Token
pub async fn get_setup_token(session_key: &str) -> Result<SetupTokenCredentials> {
    let tokens = oauth_with_cookie(session_key, true /* is_setup_token */).await?;

    Ok(SetupTokenCredentials {
        access_token: tokens.access_token,
        creds_type: "claude_setup_token".to_string(),
    })
}
```

### 3.5 AWS Bedrock

```rust
/// AWS Bedrock 凭证结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BedrockCredentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: Option<String>,
    pub region: String,
    pub default_model: Option<String>,
}

/// Bedrock 模型映射
const BEDROCK_MODEL_MAP: &[(&str, &str)] = &[
    ("claude-opus-4-20250514", "us.anthropic.claude-opus-4-20250514-v1:0"),
    ("claude-sonnet-4-20250514", "us.anthropic.claude-sonnet-4-20250514-v1:0"),
    ("claude-sonnet-4-5-20250929", "us.anthropic.claude-sonnet-4-5-20250929-v1:0"),
    ("claude-haiku-3-5-20241022", "us.anthropic.claude-haiku-3-5-20241022-v1:0"),
];

/// Bedrock API 调用
pub async fn call_bedrock_api(
    credentials: &BedrockCredentials,
    request: &AnthropicRequest,
) -> Result<impl Stream<Item = Result<AnthropicSseEvent>>> {
    // 1. 模型名映射
    let model_id = map_to_bedrock_model(&request.model);

    // 2. 构建 AWS 签名
    let aws_credentials = AwsCredentials::new(
        &credentials.access_key_id,
        &credentials.secret_access_key,
        credentials.session_token.as_deref(),
    );

    // 3. 调用 Bedrock API
    let url = format!(
        "https://bedrock-runtime.{}.amazonaws.com/model/{}/invoke-with-response-stream",
        credentials.region,
        model_id
    );

    let signed_request = sign_aws_request(
        "POST",
        &url,
        &aws_credentials,
        &credentials.region,
        "bedrock",
        &serde_json::to_vec(request)?,
    )?;

    // 4. 发送请求并返回流
    let response = reqwest::Client::new()
        .post(&url)
        .headers(signed_request.headers)
        .body(signed_request.body)
        .send()
        .await?;

    Ok(parse_bedrock_stream(response.bytes_stream()))
}
```

### 3.6 CCR（第三方中转）

```rust
/// CCR（Custom Claude Relay）凭证结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CCRCredentials {
    pub api_key: String,
    pub base_url: String,
    pub name: Option<String>,
}

/// CCR API 调用（直接转发）
pub async fn call_ccr_api(
    credentials: &CCRCredentials,
    request: &AnthropicRequest,
) -> Result<impl Stream<Item = Result<AnthropicSseEvent>>> {
    let url = format!("{}/v1/messages", credentials.base_url);

    let response = reqwest::Client::new()
        .post(&url)
        .header("x-api-key", &credentials.api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(request)
        .send()
        .await?;

    Ok(parse_anthropic_sse_stream(response.bytes_stream()))
}
```

---

## 四、Token 刷新机制

### 4.1 自动刷新逻辑

```rust
impl ClaudeProvider {
    /// Token 刷新阈值（提前 5 分钟）
    const REFRESH_THRESHOLD: Duration = Duration::minutes(5);

    /// 检查 Token 是否需要刷新
    pub fn needs_refresh(&self, credentials: &OAuthCredentials) -> bool {
        if let Some(expires_at) = credentials.expires_at {
            let now = Utc::now();
            return now >= expires_at - Self::REFRESH_THRESHOLD;
        }
        true  // 无过期时间则默认需要刷新
    }

    /// 刷新 Token（带重试）
    pub async fn refresh_token_with_retry(
        &self,
        credentials: &mut OAuthCredentials,
        max_retries: u32,
    ) -> Result<()> {
        let mut last_error = None;

        for attempt in 0..max_retries {
            match self.refresh_token(credentials).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    last_error = Some(e);
                    // 指数退避
                    let delay = Duration::milliseconds(1000 * 2_i64.pow(attempt));
                    tokio::time::sleep(delay.to_std().unwrap()).await;
                }
            }
        }

        Err(last_error.unwrap())
    }

    /// 刷新 Token
    async fn refresh_token(&self, credentials: &mut OAuthCredentials) -> Result<()> {
        let refresh_token = credentials.refresh_token.as_ref()
            .ok_or(Error::MissingRefreshToken)?;

        let response = self.http_client
            .post(CLAUDE_TOKEN_URL)
            .json(&json!({
                "client_id": CLAUDE_CLIENT_ID,
                "grant_type": "refresh_token",
                "refresh_token": refresh_token
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(Error::TokenRefreshFailed(error_text));
        }

        let token_response: TokenResponse = response.json().await?;

        // 更新凭证
        credentials.access_token = token_response.access_token;
        if let Some(new_refresh) = token_response.refresh_token {
            credentials.refresh_token = Some(new_refresh);
        }
        credentials.expires_at = Some(Utc::now() + Duration::seconds(token_response.expires_in));
        credentials.last_refresh = Some(Utc::now());

        // 更新邮箱（如果有）
        if let Some(account) = token_response.account {
            credentials.email = Some(account.email_address);
        }

        Ok(())
    }
}
```

---

## 五、前端 UI 实现

### 5.1 插件入口

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

export default function ClaudeProviderUI({ sdk, pluginId }: PluginProps) {
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
    <div className="claude-provider-ui">
      <Header>
        <Title>Claude Provider</Title>
        <Subtitle>支持 OAuth、Claude Code、Console、Bedrock、CCR</Subtitle>
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

### 5.2 认证方式选择

```tsx
// src/components/AuthMethodTabs.tsx

type AuthMethod = 'oauth' | 'claude_code' | 'console' | 'setup_token' | 'bedrock' | 'ccr';

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
          OAuth
        </Tab>
        <Tab value="claude_code">
          <Icon name="Terminal" />
          Claude Code
        </Tab>
        <Tab value="console">
          <Icon name="Building" />
          Console
        </Tab>
        <Tab value="setup_token">
          <Icon name="Lock" />
          Setup Token
        </Tab>
        <Tab value="bedrock">
          <Icon name="Cloud" />
          Bedrock
        </Tab>
        <Tab value="ccr">
          <Icon name="Server" />
          CCR
        </Tab>
      </Tabs>

      <div className="tab-content">
        {method === 'oauth' && <OAuthForm sdk={sdk} onSuccess={onSuccess} />}
        {method === 'claude_code' && <ClaudeCodeForm sdk={sdk} onSuccess={onSuccess} />}
        {method === 'console' && <ConsoleForm sdk={sdk} onSuccess={onSuccess} />}
        {method === 'setup_token' && <SetupTokenForm sdk={sdk} onSuccess={onSuccess} />}
        {method === 'bedrock' && <BedrockForm sdk={sdk} onSuccess={onSuccess} />}
        {method === 'ccr' && <CCRForm sdk={sdk} onSuccess={onSuccess} />}
      </div>

      <FormActions>
        <Button variant="secondary" onClick={onCancel}>取消</Button>
      </FormActions>
    </div>
  );
}
```

### 5.3 OAuth 表单

```tsx
// src/components/OAuthForm.tsx

type OAuthMode = 'browser' | 'cookie' | 'file';

export function OAuthForm({ sdk, onSuccess }: FormProps) {
  const [mode, setMode] = useState<OAuthMode>('cookie');
  const [sessionKey, setSessionKey] = useState('');
  const [authUrl, setAuthUrl] = useState('');
  const [loading, setLoading] = useState(false);

  const handleCookieAuth = async () => {
    setLoading(true);
    try {
      await sdk.http.request('/api/claude/oauth/cookie', {
        method: 'POST',
        body: JSON.stringify({ sessionKey, isSetupToken: false }),
      });
      sdk.notification.success('OAuth 认证成功');
      onSuccess();
    } catch (error) {
      sdk.notification.error(`认证失败: ${error.message}`);
    } finally {
      setLoading(false);
    }
  };

  const handleBrowserAuth = async () => {
    setLoading(true);
    try {
      const result = await sdk.http.request('/api/claude/oauth/start');
      setAuthUrl(result.authUrl);
      await sdk.shell.open(result.authUrl);

      // 等待回调
      const credential = await sdk.http.request('/api/claude/oauth/callback/wait', {
        timeout: 120000,
      });
      sdk.notification.success('OAuth 认证成功');
      onSuccess();
    } catch (error) {
      sdk.notification.error(`认证失败: ${error.message}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="oauth-form">
      <Tabs value={mode} onChange={setMode}>
        <Tab value="cookie">Cookie 快速授权</Tab>
        <Tab value="browser">浏览器授权</Tab>
        <Tab value="file">导入文件</Tab>
      </Tabs>

      {mode === 'cookie' && (
        <div className="cookie-mode">
          <FormField>
            <Label>Session Key</Label>
            <TextArea
              rows={3}
              value={sessionKey}
              onChange={(e) => setSessionKey(e.target.value)}
              placeholder="从 claude.ai Cookie 中获取 sessionKey 值"
            />
            <HelpText>
              打开 claude.ai → F12 开发者工具 → Application → Cookies → 复制 sessionKey
            </HelpText>
          </FormField>

          <Button onClick={handleCookieAuth} loading={loading}>
            使用 Cookie 授权
          </Button>
        </div>
      )}

      {mode === 'browser' && (
        <div className="browser-mode">
          <Alert type="info">
            点击下方按钮将打开浏览器进行 OAuth 授权
          </Alert>

          {authUrl && (
            <div className="auth-url">
              <Label>授权 URL</Label>
              <CodeBlock>{authUrl}</CodeBlock>
              <Button size="small" onClick={() => navigator.clipboard.writeText(authUrl)}>
                复制
              </Button>
            </div>
          )}

          <Button onClick={handleBrowserAuth} loading={loading}>
            {loading ? '等待授权...' : '打开浏览器授权'}
          </Button>
        </div>
      )}

      {mode === 'file' && (
        <FileImportForm
          sdk={sdk}
          onSuccess={onSuccess}
          defaultPath="~/.claude/oauth_creds.json"
        />
      )}
    </div>
  );
}
```

### 5.4 Bedrock 表单

```tsx
// src/components/BedrockForm.tsx

export function BedrockForm({ sdk, onSuccess }: FormProps) {
  const [form, setForm] = useState({
    accessKeyId: '',
    secretAccessKey: '',
    sessionToken: '',
    region: 'us-east-1',
    name: '',
  });
  const [loading, setLoading] = useState(false);

  const handleSubmit = async () => {
    setLoading(true);
    try {
      await sdk.http.request('/api/claude/bedrock/add', {
        method: 'POST',
        body: JSON.stringify(form),
      });
      sdk.notification.success('Bedrock 凭证添加成功');
      onSuccess();
    } catch (error) {
      sdk.notification.error(`添加失败: ${error.message}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="bedrock-form">
      <Alert type="info">
        使用 AWS IAM 凭证访问 Bedrock 上的 Claude 模型
      </Alert>

      <FormField>
        <Label>凭证名称（可选）</Label>
        <Input
          value={form.name}
          onChange={(e) => setForm({ ...form, name: e.target.value })}
          placeholder="我的 Bedrock 凭证"
        />
      </FormField>

      <FormField>
        <Label>Access Key ID *</Label>
        <Input
          value={form.accessKeyId}
          onChange={(e) => setForm({ ...form, accessKeyId: e.target.value })}
          placeholder="AKIAIOSFODNN7EXAMPLE"
        />
      </FormField>

      <FormField>
        <Label>Secret Access Key *</Label>
        <Input
          type="password"
          value={form.secretAccessKey}
          onChange={(e) => setForm({ ...form, secretAccessKey: e.target.value })}
          placeholder="wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
        />
      </FormField>

      <FormField>
        <Label>Session Token（可选）</Label>
        <TextArea
          rows={2}
          value={form.sessionToken}
          onChange={(e) => setForm({ ...form, sessionToken: e.target.value })}
          placeholder="临时凭证的 Session Token"
        />
        <HelpText>如果使用 STS 临时凭证，需要填写此项</HelpText>
      </FormField>

      <FormField>
        <Label>Region *</Label>
        <Select
          value={form.region}
          onChange={(value) => setForm({ ...form, region: value })}
        >
          <Option value="us-east-1">US East (N. Virginia)</Option>
          <Option value="us-west-2">US West (Oregon)</Option>
          <Option value="eu-west-1">EU (Ireland)</Option>
          <Option value="ap-northeast-1">Asia Pacific (Tokyo)</Option>
        </Select>
      </FormField>

      <Button onClick={handleSubmit} loading={loading}>
        添加 Bedrock 凭证
      </Button>
    </div>
  );
}
```

### 5.5 CCR 表单

```tsx
// src/components/CCRForm.tsx

export function CCRForm({ sdk, onSuccess }: FormProps) {
  const [form, setForm] = useState({
    apiKey: '',
    baseUrl: '',
    name: '',
  });
  const [loading, setLoading] = useState(false);

  const handleSubmit = async () => {
    setLoading(true);
    try {
      await sdk.http.request('/api/claude/ccr/add', {
        method: 'POST',
        body: JSON.stringify(form),
      });
      sdk.notification.success('CCR 凭证添加成功');
      onSuccess();
    } catch (error) {
      sdk.notification.error(`添加失败: ${error.message}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="ccr-form">
      <Alert type="info">
        配置第三方 Claude 中转服务（Custom Claude Relay）
      </Alert>

      <FormField>
        <Label>凭证名称（可选）</Label>
        <Input
          value={form.name}
          onChange={(e) => setForm({ ...form, name: e.target.value })}
          placeholder="我的 CCR 服务"
        />
      </FormField>

      <FormField>
        <Label>API Key *</Label>
        <Input
          type="password"
          value={form.apiKey}
          onChange={(e) => setForm({ ...form, apiKey: e.target.value })}
          placeholder="cr_xxxxxxxxxxxxxxxx"
        />
      </FormField>

      <FormField>
        <Label>Base URL *</Label>
        <Input
          value={form.baseUrl}
          onChange={(e) => setForm({ ...form, baseUrl: e.target.value })}
          placeholder="https://your-relay-service.com"
        />
        <HelpText>不需要包含 /v1/messages 路径</HelpText>
      </FormField>

      <Button onClick={handleSubmit} loading={loading}>
        添加 CCR 凭证
      </Button>
    </div>
  );
}
```

### 5.6 凭证卡片

```tsx
// src/components/CredentialCard.tsx

const AUTH_TYPE_LABELS: Record<string, string> = {
  oauth: 'OAuth',
  claude_code: 'Claude Code',
  console: 'Console',
  setup_token: 'Setup Token',
  bedrock: 'Bedrock',
  ccr: 'CCR',
};

const AUTH_TYPE_COLORS: Record<string, string> = {
  oauth: 'blue',
  claude_code: 'purple',
  console: 'green',
  setup_token: 'yellow',
  bedrock: 'orange',
  ccr: 'gray',
};

export function CredentialCard({ credential, onRefresh, onDelete }: CredentialCardProps) {
  const data = JSON.parse(credential.credential_data);
  const authType = data.auth_type || 'oauth';
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

        {data.region && (
          <div className="info-row">
            <label>区域</label>
            <span>{data.region}</span>
          </div>
        )}

        {data.base_url && (
          <div className="info-row">
            <label>Base URL</label>
            <span className="truncate">{data.base_url}</span>
          </div>
        )}

        {data.expires_at && (
          <div className="info-row">
            <label>过期时间</label>
            <span>{formatDate(data.expires_at)}</span>
          </div>
        )}
      </CardBody>

      <CardFooter>
        {['oauth', 'claude_code', 'console'].includes(authType) && (
          <Button size="small" onClick={onRefresh}>刷新 Token</Button>
        )}
        <Button size="small" variant="danger" onClick={onDelete}>删除</Button>
      </CardFooter>
    </Card>
  );
}
```

---

## 六、凭证文件格式

### 6.1 OAuth 凭证

```json
{
  "access_token": "sk-ant-...",
  "refresh_token": "re_...",
  "email": "user@example.com",
  "expire": "2025-01-05T12:34:56+00:00",
  "last_refresh": "2025-01-05T11:34:56+00:00",
  "type": "claude_oauth"
}
```

### 6.2 Bedrock 凭证

```json
{
  "access_key_id": "AKIAIOSFODNN7EXAMPLE",
  "secret_access_key": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
  "session_token": "...",
  "region": "us-east-1",
  "type": "bedrock"
}
```

### 6.3 CCR 凭证

```json
{
  "api_key": "cr_xxxxxxxxxxxxxxxx",
  "base_url": "https://your-relay-service.com",
  "type": "ccr"
}
```

---

## 七、错误处理

### 7.1 错误类型

| 错误类型 | 说明 | 处理方式 |
|---------|------|---------|
| `MissingRefreshToken` | 无刷新 Token（Setup Token） | 重新授权 |
| `TokenRefreshFailed` | Token 刷新失败 | 重试或重新授权 |
| `InvalidCredentials` | 凭证无效 | 检查凭证配置 |
| `AuthorizationRevoked` | 授权已撤销 | 重新授权 |
| `BedrockAccessDenied` | Bedrock 访问被拒 | 检查 IAM 权限 |
| `CCRConnectionError` | CCR 连接失败 | 检查 Base URL |

---

## 八、开发指南

### 8.1 本地开发

```bash
# 克隆仓库
git clone https://github.com/aiclientproxy/claude-provider.git
cd claude-provider

# 安装依赖
pnpm install
cd src-tauri && cargo build

# 前端开发
pnpm dev

# 后端开发
cargo watch -x run
```

### 8.2 测试

```bash
# 单元测试
cargo test

# OAuth 流程测试
cargo test --test oauth

# 前端测试
pnpm test
```

---

## 附录

### A. 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `CLAUDE_CLIENT_ID` | OAuth Client ID | 内置 |
| `CLAUDE_DEBUG` | 调试模式 | `false` |
| `CLAUDE_TIMEOUT_MS` | 请求超时 | `60000` |

### B. 参考链接

- [Anthropic API 文档](https://docs.anthropic.com/)
- [AWS Bedrock Claude](https://docs.aws.amazon.com/bedrock/latest/userguide/model-parameters-claude.html)
- [ProxyCast 插件开发指南](../prd/credential-provider-plugin-architecture.md)
- [claude-relay-service](https://github.com/aiclientproxy/claude-relay-service)
