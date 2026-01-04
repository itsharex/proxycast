# Kiro Provider 插件文档

> 版本: 1.0.0
> 仓库: `aiclientproxy/kiro-provider`
> 类型: OAuth Provider Plugin

---

## 一、概述

### 1.1 插件简介

Kiro Provider 是 ProxyCast 的 OAuth Provider 插件，用于对接 **AWS CodeWhisperer** (Kiro IDE) 服务。它将 Anthropic/OpenAI 协议请求转换为 CodeWhisperer 协议，并处理复杂的风控逻辑。

### 1.2 核心能力

| 能力 | 说明 |
|------|------|
| 协议转换 | Anthropic/OpenAI → CodeWhisperer → Anthropic SSE |
| 多认证方式 | Social Auth (Google/GitHub) + AWS Builder ID (IdC) |
| 风控适配 | Machine ID 生成、版本伪装、指纹隔离 |
| Token 管理 | 自动刷新、健康检查、凭证池负载均衡 |
| 前端 UI | 凭证管理、在线登录、状态展示 |

### 1.3 支持的模型

| 请求模型名 | 实际模型 |
|-----------|---------|
| `claude-opus-4-5`, `claude-opus-4-5-20251101` | `claude-opus-4.5` |
| `claude-haiku-4-5`, `claude-haiku-4-5-20251001` | `claude-haiku-4.5` |
| `claude-sonnet-4-5`, `claude-sonnet-4-5-20250929` | `CLAUDE_SONNET_4_5_20250929_V1_0` |
| `claude-sonnet-4-20250514` | `CLAUDE_SONNET_4_20250514_V1_0` |
| `claude-3-7-sonnet-20250219` | `CLAUDE_3_7_SONNET_20250219_V1_0` |

---

## 二、插件架构

### 2.1 项目结构

```
kiro-provider/
├── plugin/
│   ├── plugin.json              # 插件元数据
│   └── config.json              # 默认配置
│
├── src-tauri/src/               # 后端 Rust 代码
│   ├── lib.rs                   # 插件入口
│   ├── commands.rs              # Tauri 命令
│   ├── provider.rs              # KiroProvider 核心实现
│   ├── credentials.rs           # 凭证管理
│   ├── fingerprint.rs           # 指纹绑定
│   ├── risk_control.rs          # 风控逻辑
│   ├── token_refresh.rs         # Token 刷新
│   └── translator/              # 协议转换
│       ├── mod.rs
│       ├── openai_to_cw.rs      # OpenAI → CodeWhisperer
│       ├── anthropic_to_cw.rs   # Anthropic → CodeWhisperer
│       └── cw_to_anthropic.rs   # CodeWhisperer → Anthropic SSE
│
├── src/                         # 前端 React UI
│   ├── index.tsx                # 插件 UI 入口
│   ├── components/
│   │   ├── CredentialList.tsx   # 凭证列表
│   │   ├── CredentialCard.tsx   # 凭证卡片
│   │   ├── KiroForm.tsx         # 凭证添加表单
│   │   ├── LoginModes.tsx       # 登录模式选择
│   │   ├── BrowserModeSelector.tsx  # 浏览器模式
│   │   └── SettingsPanel.tsx    # 插件设置
│   ├── hooks/
│   │   ├── useCredentials.ts    # 凭证 hooks
│   │   └── usePlaywright.ts     # Playwright hooks
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
    "auth_types": ["oauth"],
    "credential_schema": {
      "type": "object",
      "properties": {
        "accessToken": { "type": "string", "title": "Access Token" },
        "refreshToken": { "type": "string", "title": "Refresh Token" },
        "profileArn": { "type": "string", "title": "Profile ARN" },
        "clientId": { "type": "string", "title": "Client ID" },
        "clientSecret": { "type": "string", "title": "Client Secret" },
        "region": { "type": "string", "default": "us-east-1" },
        "authMethod": { "type": "string", "enum": ["social", "idc"] }
      },
      "required": ["accessToken", "refreshToken"]
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
    "title": "Kiro Provider",
    "entry": "dist/index.js",
    "styles": "dist/styles.css",
    "default_width": 900,
    "default_height": 700,
    "permissions": [
      "database:read",
      "database:write",
      "http:request",
      "crypto:encrypt",
      "shell:open",
      "playwright:browser"
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
    "risk_control": {
      "machine_id_rotation": true,
      "version_spoofing": true,
      "fingerprint_isolation": true,
      "hour_slot_variation": true
    },
    "token_refresh": {
      "auto_refresh": true,
      "refresh_threshold_minutes": 5,
      "max_retry": 3
    },
    "health_check": {
      "enabled": true,
      "interval_seconds": 300,
      "unhealthy_threshold": 3
    }
  }
}
```

---

## 三、后端实现

### 3.1 核心数据结构

#### 凭证结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KiroCredentials {
    pub access_token: String,
    pub refresh_token: String,
    pub profile_arn: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub region: String,                    // 默认 "us-east-1"
    pub auth_method: AuthMethod,           // Social | IdC
    pub client_id_hash: Option<String>,
    pub expire: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    Social,  // Google/GitHub OAuth
    IdC,     // AWS Builder ID (Device Code Flow)
}
```

#### 指纹绑定

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KiroFingerprintBinding {
    pub credential_uuid: String,
    pub machine_id: String,
    pub created_at: DateTime<Utc>,
    pub last_switched_at: Option<DateTime<Utc>>,
}
```

### 3.2 风控实现

#### Machine ID 生成

```rust
/// 为每个凭证生成独立的 Machine ID
///
/// 策略：
/// 1. 基于 profile_arn 或 client_id 生成唯一标识
/// 2. 添加时间变化因子（每小时变化）避免指纹固化
/// 3. 每个凭证独立 Machine ID，防止多账号共用指纹被检测
pub fn generate_machine_id_from_credentials(
    profile_arn: Option<&str>,
    client_id: Option<&str>,
) -> String {
    // 1. 确定唯一标识
    let unique_key = profile_arn
        .or(client_id)
        .unwrap_or("default-kiro-key");

    // 2. 添加时间变化因子（每小时变化）
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let hour_slot = now / 3600;

    // 3. SHA256 哈希
    let mut hasher = Sha256::new();
    hasher.update(unique_key.as_bytes());
    hasher.update(&hour_slot.to_le_bytes());
    let hash = hasher.finalize();

    // 4. 格式化为 UUID 格式
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        u32::from_be_bytes(hash[0..4].try_into().unwrap()),
        u16::from_be_bytes(hash[4..6].try_into().unwrap()),
        u16::from_be_bytes(hash[6..8].try_into().unwrap()),
        u16::from_be_bytes(hash[8..10].try_into().unwrap()),
        u64::from_be_bytes([0, 0, hash[10], hash[11], hash[12], hash[13], hash[14], hash[15]])
    )
}
```

#### 系统信息获取

```rust
/// 获取系统运行时信息
pub fn get_system_runtime_info() -> RuntimeInfo {
    RuntimeInfo {
        os_name: get_os_name(),           // "macos", "linux", "windows"
        os_version: get_os_version(),     // "14.0.0"
        node_version: get_node_version(), // "20.10.0" (模拟)
    }
}

/// 获取 Kiro IDE 版本（macOS 从 Info.plist 读取）
pub fn get_kiro_version() -> String {
    #[cfg(target_os = "macos")]
    {
        let plist_path = "/Applications/Kiro.app/Contents/Info.plist";
        if let Ok(plist) = plist::from_file::<_, plist::Value>(plist_path) {
            if let Some(version) = plist.as_dictionary()
                .and_then(|d| d.get("CFBundleShortVersionString"))
                .and_then(|v| v.as_string())
            {
                return version.to_string();
            }
        }
    }
    "1.0.0".to_string()  // 默认版本
}
```

#### User-Agent 构造

```rust
/// Social Auth Token 刷新 User-Agent
fn build_social_auth_user_agent(kiro_version: &str, machine_id: &str) -> String {
    format!("KiroIDE-{}-{}", kiro_version, machine_id)
}

/// IdC Auth Token 刷新 User-Agent
fn build_idc_auth_user_agent(kiro_version: &str, machine_id: &str) -> String {
    format!(
        "aws-sdk-js/3.738.0 ua/2.1 os/other lang/js api/sso-oidc#3.738.0 m/E KiroIDE-{}-{}",
        kiro_version, machine_id
    )
}

/// API 调用 User-Agent
fn build_api_user_agent(
    os_name: &str,
    node_version: &str,
    kiro_version: &str,
    machine_id: &str,
) -> String {
    format!(
        "aws-sdk-js/1.0.0 ua/2.1 os/{} lang/js md/nodejs#{} api/codewhispererruntime#1.0.0 m/E KiroIDE-{}-{}",
        os_name, node_version, kiro_version, machine_id
    )
}
```

#### 请求头构造

```rust
/// 构造 API 请求头
fn build_api_headers(credential: &KiroCredentials, machine_id: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();

    // 认证
    headers.insert(
        "Authorization",
        format!("Bearer {}", credential.access_token).parse().unwrap()
    );

    // 风控关键头
    let runtime_info = get_system_runtime_info();
    let kiro_version = get_kiro_version();

    headers.insert(
        "User-Agent",
        build_api_user_agent(
            &runtime_info.os_name,
            &runtime_info.node_version,
            &kiro_version,
            machine_id
        ).parse().unwrap()
    );

    headers.insert(
        "x-amz-user-agent",
        format!("aws-sdk-js/1.0.0 KiroIDE-{}-{}", kiro_version, machine_id)
            .parse().unwrap()
    );

    headers.insert("x-amzn-kiro-agent-mode", "vibe".parse().unwrap());
    headers.insert("amz-sdk-invocation-id", Uuid::new_v4().to_string().parse().unwrap());
    headers.insert("amz-sdk-request", "attempt=1; max=1".parse().unwrap());
    headers.insert("Connection", "close".parse().unwrap());  // 避免连接复用被检测

    headers
}
```

### 3.3 Token 刷新

```rust
impl KiroProvider {
    /// 刷新 Token
    pub async fn refresh_token(&self, credential: &mut KiroCredentials) -> Result<()> {
        // 验证 refreshToken 完整性
        if credential.refresh_token.len() < 100 {
            return Err(Error::InvalidRefreshToken("Token appears truncated"));
        }

        let machine_id = self.get_or_create_machine_id(&credential.credential_uuid);
        let kiro_version = get_kiro_version();

        match credential.auth_method {
            AuthMethod::Social => {
                self.refresh_social_token(credential, &machine_id, &kiro_version).await
            }
            AuthMethod::IdC => {
                self.refresh_idc_token(credential, &machine_id, &kiro_version).await
            }
        }
    }

    /// Social Auth Token 刷新
    async fn refresh_social_token(
        &self,
        credential: &mut KiroCredentials,
        machine_id: &str,
        kiro_version: &str,
    ) -> Result<()> {
        let url = format!(
            "https://prod.{}.auth.desktop.kiro.dev/refreshToken",
            credential.region
        );

        let response = self.http_client
            .post(&url)
            .header("User-Agent", build_social_auth_user_agent(kiro_version, machine_id))
            .header("Content-Type", "application/json")
            .json(&json!({
                "refreshToken": credential.refresh_token
            }))
            .send()
            .await?;

        let result: TokenResponse = response.json().await?;
        credential.access_token = result.access_token;
        credential.expire = Some(Utc::now() + Duration::hours(1));

        Ok(())
    }

    /// IdC Auth Token 刷新
    async fn refresh_idc_token(
        &self,
        credential: &mut KiroCredentials,
        machine_id: &str,
        kiro_version: &str,
    ) -> Result<()> {
        let url = format!(
            "https://oidc.{}.amazonaws.com/token",
            credential.region
        );

        let response = self.http_client
            .post(&url)
            .header("User-Agent", build_idc_auth_user_agent(kiro_version, machine_id))
            .header("x-amz-user-agent", format!(
                "aws-sdk-js/3.738.0 KiroIDE-{}-{}", kiro_version, machine_id
            ))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("grant_type", "refresh_token"),
                ("client_id", credential.client_id.as_deref().unwrap_or("")),
                ("client_secret", credential.client_secret.as_deref().unwrap_or("")),
                ("refresh_token", &credential.refresh_token),
            ])
            .send()
            .await?;

        let result: TokenResponse = response.json().await?;
        credential.access_token = result.access_token;
        credential.expire = Some(Utc::now() + Duration::hours(1));

        Ok(())
    }
}
```

### 3.4 协议转换

#### Anthropic → CodeWhisperer

```rust
/// 将 Anthropic 请求转换为 CodeWhisperer 格式
pub fn convert_anthropic_to_codewhisperer(
    request: &AnthropicRequest,
    profile_arn: Option<&str>,
) -> CodeWhispererRequest {
    // 模型映射
    let model = map_model_name(&request.model);

    // 消息转换
    let conversation_state = ConversationState {
        current_message: convert_messages(&request.messages),
        chat_trigger_type: "MANUAL".to_string(),
        user_intent: "CHAT".to_string(),
        customization_arn: None,
    };

    // 系统提示
    let system_prompt = request.system.clone();

    // 工具转换
    let tools = request.tools.as_ref().map(|t| convert_tools(t));

    CodeWhispererRequest {
        conversation_state,
        profile_arn: profile_arn.map(String::from),
        source: "CHAT".to_string(),
        assistant_response_config: AssistantResponseConfig {
            max_output_tokens: request.max_tokens,
            temperature: request.temperature,
            response_style: Some(ResponseStyle {
                system_prompt_user_customization: system_prompt,
            }),
        },
        tools,
    }
}

/// 模型名称映射
fn map_model_name(model: &str) -> String {
    let mappings = [
        ("claude-opus-4-5", "claude-opus-4.5"),
        ("claude-opus-4-5-20251101", "claude-opus-4.5"),
        ("claude-haiku-4-5", "claude-haiku-4.5"),
        ("claude-haiku-4-5-20251001", "claude-haiku-4.5"),
        ("claude-sonnet-4-5", "CLAUDE_SONNET_4_5_20250929_V1_0"),
        ("claude-sonnet-4-5-20250929", "CLAUDE_SONNET_4_5_20250929_V1_0"),
        ("claude-sonnet-4-20250514", "CLAUDE_SONNET_4_20250514_V1_0"),
        ("claude-3-7-sonnet-20250219", "CLAUDE_3_7_SONNET_20250219_V1_0"),
        ("claude-3-5-sonnet-20241022", "CLAUDE_3_7_SONNET_20250219_V1_0"),
    ];

    for (from, to) in mappings {
        if model.contains(from) {
            return to.to_string();
        }
    }

    model.to_string()
}
```

#### CodeWhisperer → Anthropic SSE

```rust
/// 将 AWS Event Stream 转换为 Anthropic SSE
pub struct CwToAnthropicTranslator {
    message_id: String,
    model: String,
    current_index: u32,
    input_tokens: u32,
    output_tokens: u32,
}

impl CwToAnthropicTranslator {
    pub fn translate_chunk(&mut self, chunk: &AwsEventChunk) -> Vec<AnthropicSseEvent> {
        let mut events = Vec::new();

        match &chunk.event_type {
            EventType::MessageStart => {
                events.push(AnthropicSseEvent::MessageStart {
                    message: MessageStartData {
                        id: self.message_id.clone(),
                        type_: "message".to_string(),
                        role: "assistant".to_string(),
                        model: self.model.clone(),
                    },
                });
            }
            EventType::ContentBlockStart { content_type } => {
                events.push(AnthropicSseEvent::ContentBlockStart {
                    index: self.current_index,
                    content_block: ContentBlock {
                        type_: content_type.clone(),
                    },
                });
            }
            EventType::ContentBlockDelta { delta } => {
                events.push(AnthropicSseEvent::ContentBlockDelta {
                    index: self.current_index,
                    delta: Delta::TextDelta { text: delta.clone() },
                });
            }
            EventType::ContentBlockStop => {
                events.push(AnthropicSseEvent::ContentBlockStop {
                    index: self.current_index,
                });
                self.current_index += 1;
            }
            EventType::MessageDelta { stop_reason, usage } => {
                self.input_tokens = usage.input_tokens;
                self.output_tokens = usage.output_tokens;
                events.push(AnthropicSseEvent::MessageDelta {
                    delta: MessageDelta {
                        stop_reason: stop_reason.clone(),
                    },
                    usage: Usage {
                        input_tokens: self.input_tokens,
                        output_tokens: self.output_tokens,
                    },
                });
            }
            EventType::MessageStop => {
                events.push(AnthropicSseEvent::MessageStop);
            }
        }

        events
    }
}
```

---

## 四、前端 UI 实现

### 4.1 插件入口

```tsx
// src/index.tsx
import { ProxyCastPluginSDK } from '@proxycast/plugin-sdk';
import { CredentialList } from './components/CredentialList';
import { KiroForm } from './components/KiroForm';
import { SettingsPanel } from './components/SettingsPanel';

interface PluginProps {
  sdk: ProxyCastPluginSDK;
  pluginId: string;
}

export default function KiroProviderUI({ sdk, pluginId }: PluginProps) {
  const [view, setView] = useState<'list' | 'add' | 'settings'>('list');
  const [credentials, setCredentials] = useState<Credential[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadCredentials();
  }, []);

  const loadCredentials = async () => {
    setLoading(true);
    const result = await sdk.database.query<Credential>(
      'SELECT * FROM plugin_credentials WHERE plugin_id = ? ORDER BY created_at DESC',
      [pluginId]
    );
    setCredentials(result);
    setLoading(false);
  };

  return (
    <div className="kiro-provider-ui">
      <Header>
        <Title>Kiro Provider</Title>
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
        <KiroForm
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

### 4.2 凭证添加表单

```tsx
// src/components/KiroForm.tsx

type AddMode = 'login' | 'json' | 'file';
type BrowserMode = 'system' | 'playwright';
type OAuthProvider = 'google' | 'github' | 'builder_id';

interface KiroFormProps {
  sdk: ProxyCastPluginSDK;
  onSuccess: () => void;
  onCancel: () => void;
}

export function KiroForm({ sdk, onSuccess, onCancel }: KiroFormProps) {
  const [mode, setMode] = useState<AddMode>('login');
  const [browserMode, setBrowserMode] = useState<BrowserMode>('system');
  const [oauthProvider, setOAuthProvider] = useState<OAuthProvider>('google');
  const [loading, setLoading] = useState(false);
  const [playwrightStatus, setPlaywrightStatus] = useState<PlaywrightStatus>();

  // 检查 Playwright 可用性
  useEffect(() => {
    checkPlaywrightAvailable().then(setPlaywrightStatus);
  }, []);

  return (
    <div className="kiro-form">
      {/* 模式选择 */}
      <Tabs value={mode} onChange={setMode}>
        <Tab value="login">在线登录</Tab>
        <Tab value="json">粘贴 JSON</Tab>
        <Tab value="file">导入文件</Tab>
      </Tabs>

      {mode === 'login' && (
        <LoginMode
          browserMode={browserMode}
          setBrowserMode={setBrowserMode}
          oauthProvider={oauthProvider}
          setOAuthProvider={setOAuthProvider}
          playwrightStatus={playwrightStatus}
          sdk={sdk}
          onSuccess={onSuccess}
        />
      )}

      {mode === 'json' && (
        <JsonMode sdk={sdk} onSuccess={onSuccess} />
      )}

      {mode === 'file' && (
        <FileMode sdk={sdk} onSuccess={onSuccess} />
      )}
    </div>
  );
}
```

### 4.3 在线登录模式

```tsx
// src/components/LoginModes.tsx

interface LoginModeProps {
  browserMode: BrowserMode;
  setBrowserMode: (mode: BrowserMode) => void;
  oauthProvider: OAuthProvider;
  setOAuthProvider: (provider: OAuthProvider) => void;
  playwrightStatus?: PlaywrightStatus;
  sdk: ProxyCastPluginSDK;
  onSuccess: () => void;
}

export function LoginMode(props: LoginModeProps) {
  const { browserMode, setBrowserMode, oauthProvider, setOAuthProvider, playwrightStatus, sdk, onSuccess } = props;
  const [loading, setLoading] = useState(false);
  const [builderIdData, setBuilderIdData] = useState<BuilderIdLoginData | null>(null);

  const handleLogin = async () => {
    setLoading(true);

    try {
      if (oauthProvider === 'builder_id') {
        // AWS Builder ID - Device Code Flow
        await startBuilderIdLogin();
      } else {
        // Google/GitHub OAuth
        if (browserMode === 'system') {
          await startSystemBrowserLogin(oauthProvider);
        } else {
          await startPlaywrightLogin(oauthProvider);
        }
      }
    } catch (error) {
      sdk.notification.error(`登录失败: ${error.message}`);
    } finally {
      setLoading(false);
    }
  };

  const startBuilderIdLogin = async () => {
    // 1. 获取设备码
    const { userCode, verificationUri, interval } = await sdk.http.request(
      '/api/kiro/builder-id/start',
      { method: 'POST' }
    );

    setBuilderIdData({ userCode, verificationUri });

    // 2. 打开浏览器
    await sdk.shell.open(verificationUri);

    // 3. 轮询检查授权状态
    const pollInterval = setInterval(async () => {
      const result = await sdk.http.request('/api/kiro/builder-id/poll');
      if (result.status === 'complete') {
        clearInterval(pollInterval);
        setBuilderIdData(null);
        onSuccess();
      } else if (result.status === 'expired') {
        clearInterval(pollInterval);
        setBuilderIdData(null);
        sdk.notification.error('授权已过期，请重试');
      }
    }, interval * 1000);
  };

  return (
    <div className="login-mode">
      {/* OAuth Provider 选择 */}
      <div className="oauth-providers">
        <Button
          variant={oauthProvider === 'google' ? 'primary' : 'secondary'}
          onClick={() => setOAuthProvider('google')}
        >
          <GoogleIcon /> Google
        </Button>
        <Button
          variant={oauthProvider === 'github' ? 'primary' : 'secondary'}
          onClick={() => setOAuthProvider('github')}
        >
          <GitHubIcon /> GitHub
        </Button>
        <Button
          variant={oauthProvider === 'builder_id' ? 'primary' : 'secondary'}
          onClick={() => setOAuthProvider('builder_id')}
        >
          <AwsIcon /> Builder ID
        </Button>
      </div>

      {/* 浏览器模式选择（非 Builder ID） */}
      {oauthProvider !== 'builder_id' && (
        <BrowserModeSelector
          value={browserMode}
          onChange={setBrowserMode}
          playwrightStatus={playwrightStatus}
        />
      )}

      {/* Builder ID 设备码显示 */}
      {builderIdData && (
        <div className="builder-id-code">
          <p>请在浏览器中输入以下代码：</p>
          <code className="user-code">{builderIdData.userCode}</code>
          <p>等待授权中...</p>
        </div>
      )}

      {/* 登录按钮 */}
      <Button
        onClick={handleLogin}
        loading={loading}
        disabled={loading}
      >
        {loading ? '登录中...' : '开始登录'}
      </Button>
    </div>
  );
}
```

### 4.4 浏览器模式选择器

```tsx
// src/components/BrowserModeSelector.tsx

interface BrowserModeSelectorProps {
  value: BrowserMode;
  onChange: (mode: BrowserMode) => void;
  playwrightStatus?: PlaywrightStatus;
}

export function BrowserModeSelector({ value, onChange, playwrightStatus }: BrowserModeSelectorProps) {
  const playwrightAvailable = playwrightStatus?.available ?? false;

  return (
    <div className="browser-mode-selector">
      <label>浏览器模式</label>

      <div className="mode-options">
        <RadioOption
          value="system"
          checked={value === 'system'}
          onChange={() => onChange('system')}
          label="系统浏览器"
          description="使用默认浏览器进行 OAuth 登录"
        />

        <RadioOption
          value="playwright"
          checked={value === 'playwright'}
          onChange={() => onChange('playwright')}
          disabled={!playwrightAvailable}
          label="指纹浏览器"
          description={
            playwrightAvailable
              ? "使用 Playwright 反爬虫浏览器绕过机器人检测"
              : "Playwright 未安装"
          }
        />
      </div>

      {!playwrightAvailable && (
        <Alert type="info">
          <p>Playwright 未安装。</p>
          <a href="#" onClick={() => installPlaywright()}>点击安装</a>
        </Alert>
      )}
    </div>
  );
}
```

### 4.5 凭证列表

```tsx
// src/components/CredentialList.tsx

interface CredentialListProps {
  credentials: Credential[];
  onRefresh: () => void;
  sdk: ProxyCastPluginSDK;
}

export function CredentialList({ credentials, onRefresh, sdk }: CredentialListProps) {
  const handleRefreshToken = async (credentialId: string) => {
    try {
      await sdk.http.request(`/api/kiro/credentials/${credentialId}/refresh`, {
        method: 'POST',
      });
      sdk.notification.success('Token 刷新成功');
      onRefresh();
    } catch (error) {
      sdk.notification.error(`刷新失败: ${error.message}`);
    }
  };

  const handleDelete = async (credentialId: string) => {
    if (!confirm('确定删除此凭证？')) return;

    await sdk.database.execute(
      'DELETE FROM plugin_credentials WHERE id = ?',
      [credentialId]
    );
    sdk.notification.success('凭证已删除');
    onRefresh();
  };

  const handleSwitchToLocal = async (credentialId: string) => {
    try {
      await sdk.http.request(`/api/kiro/credentials/${credentialId}/switch-to-local`, {
        method: 'POST',
      });
      sdk.notification.success('已切换到本地 Kiro IDE');
    } catch (error) {
      sdk.notification.error(`切换失败: ${error.message}`);
    }
  };

  if (credentials.length === 0) {
    return (
      <EmptyState>
        <p>暂无凭证</p>
        <p>点击"添加凭证"开始使用</p>
      </EmptyState>
    );
  }

  return (
    <div className="credential-list">
      {credentials.map((credential) => (
        <CredentialCard
          key={credential.id}
          credential={credential}
          onRefresh={() => handleRefreshToken(credential.id)}
          onDelete={() => handleDelete(credential.id)}
          onSwitchToLocal={() => handleSwitchToLocal(credential.id)}
        />
      ))}
    </div>
  );
}
```

### 4.6 凭证卡片

```tsx
// src/components/CredentialCard.tsx

interface CredentialCardProps {
  credential: Credential;
  onRefresh: () => void;
  onDelete: () => void;
  onSwitchToLocal: () => void;
}

export function CredentialCard({ credential, onRefresh, onDelete, onSwitchToLocal }: CredentialCardProps) {
  const data = JSON.parse(credential.credential_data) as KiroCredentials;
  const isHealthy = credential.status === 'active';
  const isExpiringSoon = data.expire && new Date(data.expire) < new Date(Date.now() + 5 * 60 * 1000);

  return (
    <Card className={`credential-card ${isHealthy ? 'healthy' : 'unhealthy'}`}>
      <CardHeader>
        <div className="status-indicator">
          <StatusDot status={isHealthy ? 'green' : 'red'} />
          <span>{isHealthy ? '健康' : '异常'}</span>
        </div>
        <div className="auth-method">
          <Badge>{data.authMethod === 'social' ? 'Social' : 'Builder ID'}</Badge>
        </div>
      </CardHeader>

      <CardBody>
        <div className="info-row">
          <label>名称</label>
          <span>{credential.name || '未命名'}</span>
        </div>
        <div className="info-row">
          <label>区域</label>
          <span>{data.region}</span>
        </div>
        <div className="info-row">
          <label>Profile ARN</label>
          <span className="truncate">{data.profileArn || '-'}</span>
        </div>
        <div className="info-row">
          <label>过期时间</label>
          <span className={isExpiringSoon ? 'warning' : ''}>
            {data.expire ? formatDate(data.expire) : '-'}
            {isExpiringSoon && <Badge variant="warning">即将过期</Badge>}
          </span>
        </div>
        <div className="info-row">
          <label>使用次数</label>
          <span>{credential.usage_count}</span>
        </div>
        <div className="info-row">
          <label>错误次数</label>
          <span>{credential.error_count}</span>
        </div>
      </CardBody>

      <CardFooter>
        <Button size="small" onClick={onRefresh}>刷新 Token</Button>
        <Button size="small" onClick={onSwitchToLocal}>切换到本地</Button>
        <Button size="small" variant="danger" onClick={onDelete}>删除</Button>
      </CardFooter>
    </Card>
  );
}
```

---

## 五、凭证文件格式

### 5.1 主凭证文件

**路径**: `~/.aws/sso/cache/kiro-auth-token.json`

```json
{
  "accessToken": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refreshToken": "rt-xxxxxxxxxxxxxxxxxxxxxxxxxx...",
  "profileArn": "arn:aws:iam::123456789012:user/username",
  "region": "us-east-1",
  "authMethod": "social",
  "clientIdHash": "a1b2c3d4e5f6...",
  "expire": "2025-01-04T12:00:00.000Z"
}
```

### 5.2 Client 注册文件

**路径**: `~/.aws/sso/cache/{clientIdHash}.json`

```json
{
  "clientId": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "clientSecret": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "expiresAt": "2025-01-04T12:00:00.000Z",
  "scopes": [
    "codewhisperer:conversations",
    "codewhisperer:transformations"
  ]
}
```

---

## 六、API 调用流程

### 6.1 完整调用链路

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Kiro Provider 调用流程                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  1. 客户端请求 (Anthropic/OpenAI 格式)                                    │
│     │                                                                    │
│     ▼                                                                    │
│  2. 协议识别 & 路由                                                       │
│     ├── /v1/messages → Anthropic 入口                                    │
│     └── /v1/chat/completions → OpenAI 入口                               │
│     │                                                                    │
│     ▼                                                                    │
│  3. 凭证获取                                                              │
│     ├── 从凭证池选择健康凭证                                              │
│     ├── 检查 Token 是否过期                                               │
│     └── 必要时刷新 Token                                                  │
│     │                                                                    │
│     ▼                                                                    │
│  4. 风控处理                                                              │
│     ├── 获取/生成 Machine ID                                              │
│     ├── 获取 Kiro 版本号                                                  │
│     ├── 获取系统信息                                                      │
│     └── 构造请求头                                                        │
│     │                                                                    │
│     ▼                                                                    │
│  5. 协议转换 (Request)                                                    │
│     ├── Anthropic → CodeWhisperer                                        │
│     └── OpenAI → CodeWhisperer                                           │
│     │                                                                    │
│     ▼                                                                    │
│  6. AWS CodeWhisperer API 调用                                            │
│     └── POST https://codewhisperer.{region}.amazonaws.com/generateChat   │
│     │                                                                    │
│     ▼                                                                    │
│  7. 响应转换 (Response)                                                   │
│     └── AWS Event Stream → Anthropic SSE                                 │
│     │                                                                    │
│     ▼                                                                    │
│  8. 返回客户端                                                            │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 6.2 AWS CodeWhisperer API

**端点**: `POST https://codewhisperer.{region}.amazonaws.com/generateChat`

**请求格式**:

```json
{
  "conversationState": {
    "currentMessage": {
      "userInputMessage": {
        "content": "Hello, how are you?",
        "userInputMessageContext": {}
      }
    },
    "chatTriggerType": "MANUAL",
    "userIntent": "CHAT"
  },
  "profileArn": "arn:aws:iam::...",
  "source": "CHAT",
  "assistantResponseConfig": {
    "maxOutputTokens": 4096,
    "temperature": 0.7,
    "responseStyle": {
      "systemPromptUserCustomization": "You are a helpful assistant."
    }
  }
}
```

**响应格式**: AWS Event Stream

---

## 七、错误处理

### 7.1 错误类型

| 错误码 | 说明 | 处理方式 |
|--------|------|---------|
| 401 | Token 过期 | 自动刷新 Token 重试 |
| 403 | 权限不足 | 标记凭证异常 |
| 429 | 请求限流 | 冷却期后重试 |
| 500 | 服务器错误 | 切换凭证重试 |

### 7.2 Playwright 错误

```tsx
type PlaywrightErrorType =
  | 'USER_CANCELLED'      // 用户取消登录
  | 'BROWSER_CLOSED'      // 浏览器被关闭
  | 'TIMEOUT'             // 登录超时
  | 'NETWORK_ERROR'       // 网络错误
  | 'CAPTCHA_FAILED'      // 验证码失败
  | 'UNKNOWN';            // 未知错误
```

---

## 八、开发指南

### 8.1 本地开发

```bash
# 克隆仓库
git clone https://github.com/aiclientproxy/kiro-provider.git
cd kiro-provider

# 安装依赖
pnpm install
cd src-tauri && cargo build

# 前端开发
pnpm dev

# 后端开发
cargo watch -x run
```

### 8.2 构建发布

```bash
# 构建前端
pnpm build

# 构建后端（各平台）
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-pc-windows-msvc

# 生成 checksum
sha256sum target/*/release/kiro-provider-cli* > checksums.txt
```

### 8.3 测试

```bash
# 单元测试
cargo test

# 集成测试
cargo test --test integration

# 前端测试
pnpm test
```

---

## 附录

### A. 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `KIRO_REGION` | AWS 区域 | `us-east-1` |
| `KIRO_DEBUG` | 调试模式 | `false` |
| `KIRO_TIMEOUT_MS` | 请求超时 | `60000` |

### B. 参考链接

- [AWS CodeWhisperer 文档](https://docs.aws.amazon.com/codewhisperer/)
- [Kiro IDE](https://kiro.dev/)
- [ProxyCast 插件开发指南](../prd/credential-provider-plugin-architecture.md)
