# Antigravity Provider 插件文档

> 版本: 1.0.0
> 仓库: `aiclientproxy/antigravity-provider`
> 类型: OAuth Provider Plugin

---

## 一、概述

### 1.1 插件简介

Antigravity Provider 是 ProxyCast 的 OAuth Provider 插件，用于对接 **Google 内部 Gemini CLI** 服务（Antigravity）。它支持 **动态协议选择**：根据模型类型自动选择输出协议（Claude 模型 → Anthropic，Gemini 模型 → Gemini）。

### 1.2 核心能力

| 能力 | 说明 |
|------|------|
| 动态协议 | claude-* → Anthropic SSE，gemini-* → Gemini 协议 |
| 多模型支持 | Gemini 3 Pro、Gemini 2.5 Flash、Claude Sonnet/Opus |
| Google OAuth | 使用 Google 账户认证 |
| 安全设置 | 自动附加 Safety Settings（关闭内容过滤）|
| 思维链 | 支持 reasoning_effort 配置 |
| 流式响应 | 真实端到端流式传输 |

### 1.3 支持的模型

| 用户模型名 | 内部 API 名称 | 底层模型 | 输出协议 |
|-----------|-------------|---------|---------|
| `gemini-3-pro-preview` | `gemini-3-pro-high` | Gemini | Gemini |
| `gemini-3-pro-image-preview` | `gemini-3-pro-image` | Gemini | Gemini |
| `gemini-3-flash-preview` | `gemini-3-flash` | Gemini | Gemini |
| `gemini-2.5-flash` | `gemini-2.5-flash` | Gemini | Gemini |
| `gemini-2.5-computer-use-preview-10-2025` | `rev19-uic3-1p` | Gemini | Gemini |
| `gemini-claude-sonnet-4-5` | `claude-sonnet-4-5` | Claude | **Anthropic** |
| `gemini-claude-sonnet-4-5-thinking` | `claude-sonnet-4-5-thinking` | Claude | **Anthropic** |
| `gemini-claude-opus-4-5-thinking` | `claude-opus-4-5-thinking` | Claude | **Anthropic** |

---

## 二、插件架构

### 2.1 项目结构

```
antigravity-provider/
├── plugin/
│   ├── plugin.json              # 插件元数据
│   └── config.json              # 默认配置
│
├── src-tauri/src/               # 后端 Rust 代码
│   ├── lib.rs                   # 插件入口
│   ├── commands.rs              # Tauri 命令
│   ├── provider.rs              # AntigravityProvider 核心实现
│   ├── credentials.rs           # 凭证管理
│   ├── oauth.rs                 # Google OAuth 流程
│   ├── token_refresh.rs         # Token 刷新
│   ├── models.rs                # 模型定义和别名映射
│   ├── safety_settings.rs       # 安全设置
│   └── converter/               # 协议转换
│       ├── mod.rs
│       ├── openai_to_antigravity.rs   # OpenAI → Antigravity
│       ├── anthropic_to_antigravity.rs # Anthropic → Antigravity
│       ├── antigravity_to_anthropic.rs # Antigravity → Anthropic SSE
│       └── antigravity_to_gemini.rs    # Antigravity → Gemini
│
├── src/                         # 前端 React UI
│   ├── index.tsx                # 插件 UI 入口
│   ├── components/
│   │   ├── CredentialList.tsx   # 凭证列表
│   │   ├── CredentialCard.tsx   # 凭证卡片
│   │   ├── AntigravityForm.tsx  # 凭证添加表单
│   │   ├── OAuthLogin.tsx       # Google OAuth 登录
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
  "name": "antigravity-provider",
  "version": "1.0.0",
  "description": "Antigravity (Google Gemini CLI) OAuth Provider - 支持 Gemini 和 Claude 模型",
  "author": "ProxyCast Team",
  "homepage": "https://github.com/aiclientproxy/antigravity-provider",
  "license": "MIT",

  "plugin_type": "oauth_provider",
  "entry": "antigravity-provider-cli",
  "min_proxycast_version": "1.0.0",

  "provider": {
    "id": "antigravity",
    "display_name": "Antigravity (Gemini CLI)",
    "target_protocol": "dynamic",
    "protocol_rules": {
      "claude-*": "anthropic",
      "gemini-*": "gemini"
    },
    "supported_models": [
      "gemini-3-pro-*",
      "gemini-2.5-*",
      "gemini-claude-*"
    ],
    "auth_types": ["oauth"],
    "credential_schema": {
      "type": "object",
      "properties": {
        "access_token": { "type": "string", "title": "Access Token" },
        "refresh_token": { "type": "string", "title": "Refresh Token" },
        "expiry_date": { "type": "integer", "title": "过期时间戳" },
        "project_id": { "type": "string", "title": "Project ID" },
        "email": { "type": "string", "title": "Google 邮箱" }
      },
      "required": ["access_token", "refresh_token"]
    }
  },

  "binary": {
    "binary_name": "antigravity-provider-cli",
    "github_owner": "aiclientproxy",
    "github_repo": "antigravity-provider",
    "platform_binaries": {
      "macos-arm64": "antigravity-provider-aarch64-apple-darwin",
      "macos-x64": "antigravity-provider-x86_64-apple-darwin",
      "linux-x64": "antigravity-provider-x86_64-unknown-linux-gnu",
      "windows-x64": "antigravity-provider-x86_64-pc-windows-msvc.exe"
    },
    "checksum_file": "checksums.txt"
  },

  "ui": {
    "surfaces": ["oauth_providers"],
    "icon": "Sparkles",
    "title": "Antigravity Provider",
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
  "timeout_ms": 120000,
  "settings": {
    "api": {
      "environment": "daily",
      "base_url_daily": "https://daily-cloudcode-pa.sandbox.googleapis.com",
      "base_url_autopush": "https://autopush-cloudcode-pa.sandbox.googleapis.com",
      "api_version": "v1internal"
    },
    "safety_settings": {
      "harassment": "OFF",
      "hate_speech": "OFF",
      "sexually_explicit": "OFF",
      "dangerous_content": "OFF",
      "civic_integrity": "BLOCK_NONE"
    },
    "token_refresh": {
      "auto_refresh": true,
      "refresh_skew_seconds": 3000,
      "max_retry": 3
    },
    "reasoning": {
      "default_effort": "medium",
      "enable_thinking_models": true
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
pub struct AntigravityCredentials {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_type: Option<String>,
    pub expiry_date: Option<i64>,         // 毫秒时间戳
    pub expire: Option<String>,           // RFC3339 格式
    pub scope: Option<String>,
    pub last_refresh: Option<String>,     // RFC3339 格式
    pub cred_type: String,                // 默认: "antigravity"
    pub expires_in: Option<i64>,          // 有效期（秒）
    pub timestamp: Option<i64>,           // 获取时间（毫秒）
    pub enable: Option<bool>,
    pub project_id: Option<String>,
    pub email: Option<String>,
}
```

#### 模型别名映射

```rust
/// 用户友好模型名 → 内部 API 模型名
pub fn map_model_name(model: &str) -> String {
    let mappings = [
        ("gemini-2.5-computer-use-preview-10-2025", "rev19-uic3-1p"),
        ("gemini-3-pro-image-preview", "gemini-3-pro-image"),
        ("gemini-3-pro-preview", "gemini-3-pro-high"),
        ("gemini-3-flash-preview", "gemini-3-flash"),
        ("gemini-2.5-flash", "gemini-2.5-flash"),
        ("gemini-claude-sonnet-4-5", "claude-sonnet-4-5"),
        ("gemini-claude-sonnet-4-5-thinking", "claude-sonnet-4-5-thinking"),
        ("gemini-claude-opus-4-5-thinking", "claude-opus-4-5-thinking"),
    ];

    for (from, to) in mappings {
        if model.contains(from) {
            return to.to_string();
        }
    }

    model.to_string()
}
```

### 3.2 动态协议选择

```rust
/// 根据模型确定输出协议
pub fn determine_output_protocol(model: &str) -> OutputProtocol {
    // Claude 模型 → Anthropic 协议
    if model.contains("claude") {
        return OutputProtocol::Anthropic;
    }

    // Gemini 模型 → Gemini 协议
    OutputProtocol::Gemini
}

#[derive(Debug, Clone)]
pub enum OutputProtocol {
    Anthropic,  // Claude Code 使用
    Gemini,     // Gemini 客户端使用
}
```

### 3.3 OAuth 配置

```rust
/// OAuth 2.0 配置（Google）
pub const OAUTH_CLIENT_ID: &str =
    "1071006060591-tmhssin2h21lcre235vtolojh4g403ep.apps.googleusercontent.com";
pub const OAUTH_CLIENT_SECRET: &str =
    "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";

pub const OAUTH_SCOPES: &[&str] = &[
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/userinfo.email",
    "https://www.googleapis.com/auth/userinfo.profile",
    "https://www.googleapis.com/auth/cclog",
    "https://www.googleapis.com/auth/experimentsandconfigs",
];

pub const OAUTH_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/auth";
pub const OAUTH_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
```

### 3.4 Token 刷新

```rust
impl AntigravityProvider {
    /// Token 刷新时间偏移（提前 50 分钟刷新）
    const REFRESH_SKEW: i64 = 3000; // 秒

    /// 检查 Token 是否需要刷新
    pub fn needs_refresh(&self, credential: &AntigravityCredentials) -> bool {
        let now = Utc::now().timestamp();

        // 优先使用 expiry_date（毫秒时间戳）
        if let Some(expiry_date) = credential.expiry_date {
            let expiry_secs = expiry_date / 1000;
            return now >= expiry_secs - Self::REFRESH_SKEW;
        }

        // 使用 expire（RFC3339 字符串）
        if let Some(expire) = &credential.expire {
            if let Ok(expiry) = DateTime::parse_from_rfc3339(expire) {
                return now >= expiry.timestamp() - Self::REFRESH_SKEW;
            }
        }

        // 使用 expires_in + timestamp
        if let (Some(expires_in), Some(timestamp)) = (credential.expires_in, credential.timestamp) {
            let expiry_secs = timestamp / 1000 + expires_in;
            return now >= expiry_secs - Self::REFRESH_SKEW;
        }

        // 默认需要刷新
        true
    }

    /// 刷新 Token
    pub async fn refresh_token(&self, credential: &mut AntigravityCredentials) -> Result<()> {
        let refresh_token = credential.refresh_token.as_ref()
            .ok_or(Error::MissingRefreshToken)?;

        let response = self.http_client
            .post(OAUTH_TOKEN_URL)
            .form(&[
                ("client_id", OAUTH_CLIENT_ID),
                ("client_secret", OAUTH_CLIENT_SECRET),
                ("refresh_token", refresh_token.as_str()),
                ("grant_type", "refresh_token"),
            ])
            .send()
            .await?;

        let token_response: TokenResponse = response.json().await?;

        // 更新凭证
        credential.access_token = Some(token_response.access_token);
        credential.expiry_date = Some(Utc::now().timestamp_millis() + token_response.expires_in * 1000);
        credential.last_refresh = Some(Utc::now().to_rfc3339());

        Ok(())
    }
}
```

### 3.5 安全设置

```rust
/// 默认安全设置（关闭内容过滤）
pub fn default_safety_settings() -> Vec<SafetySetting> {
    vec![
        SafetySetting {
            category: "HARM_CATEGORY_HARASSMENT".to_string(),
            threshold: "OFF".to_string(),
        },
        SafetySetting {
            category: "HARM_CATEGORY_HATE_SPEECH".to_string(),
            threshold: "OFF".to_string(),
        },
        SafetySetting {
            category: "HARM_CATEGORY_SEXUALLY_EXPLICIT".to_string(),
            threshold: "OFF".to_string(),
        },
        SafetySetting {
            category: "HARM_CATEGORY_DANGEROUS_CONTENT".to_string(),
            threshold: "OFF".to_string(),
        },
        SafetySetting {
            category: "HARM_CATEGORY_CIVIC_INTEGRITY".to_string(),
            threshold: "BLOCK_NONE".to_string(),
        },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetySetting {
    pub category: String,
    pub threshold: String,
}
```

### 3.6 协议转换

#### OpenAI → Antigravity

```rust
/// 将 OpenAI ChatCompletion 请求转换为 Antigravity 格式
pub fn convert_openai_to_antigravity(
    request: &OpenAiRequest,
    project_id: Option<&str>,
) -> AntigravityRequest {
    // 1. 模型映射
    let model = map_model_name(&request.model);

    // 2. 消息转换
    let contents = convert_messages(&request.messages);

    // 3. 系统指令提取
    let system_instruction = extract_system_instruction(&request.messages);

    // 4. 工具转换
    let tools = request.tools.as_ref().map(|t| convert_tools(t));

    // 5. 生成配置
    let generation_config = GenerationConfig {
        max_output_tokens: request.max_tokens,
        temperature: request.temperature,
        top_p: request.top_p,
        candidate_count: Some(1),
        stop_sequences: request.stop.clone(),
        // 思维链配置
        thinking_config: request.reasoning_effort.as_ref().map(|effort| {
            ThinkingConfig {
                thinking_budget: match effort.as_str() {
                    "low" => 1024,
                    "medium" => 4096,
                    "high" => 16384,
                    _ => 4096,
                },
            }
        }),
    };

    AntigravityRequest {
        model,
        contents,
        system_instruction,
        tools,
        generation_config: Some(generation_config),
        safety_settings: Some(default_safety_settings()),
    }
}

/// 消息格式转换
fn convert_messages(messages: &[OpenAiMessage]) -> Vec<Content> {
    messages
        .iter()
        .filter(|m| m.role != "system")  // 系统消息单独处理
        .map(|msg| {
            let role = match msg.role.as_str() {
                "user" => "user",
                "assistant" => "model",
                "tool" => "function",
                _ => "user",
            };

            Content {
                role: role.to_string(),
                parts: convert_content_parts(&msg.content, &msg.tool_calls),
            }
        })
        .collect()
}
```

#### Antigravity → Anthropic SSE（Claude 模型）

```rust
/// 将 Antigravity 响应转换为 Anthropic SSE 格式
pub struct AntigravityToAnthropicTranslator {
    message_id: String,
    model: String,
    current_index: u32,
    input_tokens: u32,
    output_tokens: u32,
}

impl AntigravityToAnthropicTranslator {
    pub fn translate_chunk(&mut self, chunk: &GeminiStreamChunk) -> Vec<AnthropicSseEvent> {
        let mut events = Vec::new();

        // 候选内容处理
        if let Some(candidates) = &chunk.candidates {
            for candidate in candidates {
                if let Some(content) = &candidate.content {
                    for part in &content.parts {
                        // 文本内容
                        if let Some(text) = &part.text {
                            events.push(AnthropicSseEvent::ContentBlockDelta {
                                index: self.current_index,
                                delta: Delta::TextDelta { text: text.clone() },
                            });
                        }

                        // 思维内容（thinking models）
                        if let Some(thought) = &part.thought {
                            events.push(AnthropicSseEvent::ContentBlockDelta {
                                index: self.current_index,
                                delta: Delta::ThinkingDelta { thinking: thought.clone() },
                            });
                        }

                        // 工具调用
                        if let Some(function_call) = &part.function_call {
                            events.push(AnthropicSseEvent::ContentBlockStart {
                                index: self.current_index,
                                content_block: ContentBlock::ToolUse {
                                    id: Uuid::new_v4().to_string(),
                                    name: function_call.name.clone(),
                                },
                            });
                        }
                    }
                }

                // 完成原因
                if let Some(finish_reason) = &candidate.finish_reason {
                    let stop_reason = match finish_reason.as_str() {
                        "STOP" => "end_turn",
                        "MAX_TOKENS" => "max_tokens",
                        "SAFETY" => "content_filter",
                        "TOOL_CALL" => "tool_use",
                        _ => "end_turn",
                    };

                    events.push(AnthropicSseEvent::MessageDelta {
                        delta: MessageDelta {
                            stop_reason: Some(stop_reason.to_string()),
                        },
                        usage: Usage {
                            input_tokens: self.input_tokens,
                            output_tokens: self.output_tokens,
                        },
                    });
                }
            }
        }

        // 使用统计
        if let Some(usage) = &chunk.usage_metadata {
            self.input_tokens = usage.prompt_token_count.unwrap_or(0);
            self.output_tokens = usage.candidates_token_count.unwrap_or(0);
        }

        events
    }
}
```

### 3.7 API 调用

```rust
impl AntigravityProvider {
    /// API 基础 URL
    fn get_base_url(&self) -> &str {
        match self.config.environment.as_str() {
            "autopush" => "https://autopush-cloudcode-pa.sandbox.googleapis.com",
            _ => "https://daily-cloudcode-pa.sandbox.googleapis.com",
        }
    }

    /// 流式 API 调用
    pub async fn call_api_stream(
        &self,
        request: AntigravityRequest,
        credential: &AntigravityCredentials,
    ) -> Result<impl Stream<Item = Result<GeminiStreamChunk>>> {
        let url = format!(
            "{}/v1internal/models/{}:streamGenerateContent?alt=sse",
            self.get_base_url(),
            request.model
        );

        let response = self.http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", credential.access_token.as_ref().unwrap()))
            .header("Content-Type", "application/json")
            .header("X-Goog-Api-Client", "genai-js/0.21.0")
            .json(&request)
            .send()
            .await?;

        // 解析 SSE 流
        Ok(parse_sse_stream(response.bytes_stream()))
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
import { AntigravityForm } from './components/AntigravityForm';
import { SettingsPanel } from './components/SettingsPanel';

interface PluginProps {
  sdk: ProxyCastPluginSDK;
  pluginId: string;
}

export default function AntigravityProviderUI({ sdk, pluginId }: PluginProps) {
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
    <div className="antigravity-provider-ui">
      <Header>
        <Title>Antigravity Provider</Title>
        <Subtitle>支持 Gemini 3 Pro 和 Claude 模型</Subtitle>
        <Actions>
          <Button onClick={() => setView('add')}>添加凭证</Button>
          <Button onClick={() => setView('settings')}>设置</Button>
        </Actions>
      </Header>

      {/* 动态协议说明 */}
      <InfoBanner>
        <p>此 Provider 支持动态协议选择：</p>
        <ul>
          <li><strong>gemini-*</strong> 模型 → Gemini 协议输出</li>
          <li><strong>claude-*</strong> 模型 → Anthropic 协议输出</li>
        </ul>
      </InfoBanner>

      {view === 'list' && (
        <CredentialList
          credentials={credentials}
          onRefresh={loadCredentials}
          sdk={sdk}
        />
      )}

      {view === 'add' && (
        <AntigravityForm
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
// src/components/AntigravityForm.tsx

type AddMode = 'oauth' | 'file';

interface AntigravityFormProps {
  sdk: ProxyCastPluginSDK;
  onSuccess: () => void;
  onCancel: () => void;
}

export function AntigravityForm({ sdk, onSuccess, onCancel }: AntigravityFormProps) {
  const [mode, setMode] = useState<AddMode>('oauth');
  const [loading, setLoading] = useState(false);
  const [projectId, setProjectId] = useState('');
  const [name, setName] = useState('');

  const handleOAuthLogin = async () => {
    setLoading(true);

    try {
      // 启动 OAuth 流程
      const result = await sdk.http.request('/api/antigravity/oauth/start', {
        method: 'POST',
        body: JSON.stringify({
          name: name || undefined,
          skipProjectIdFetch: !projectId,
        }),
      });

      // 打开授权 URL
      await sdk.shell.open(result.authUrl);

      // 等待回调
      const credential = await sdk.http.request('/api/antigravity/oauth/callback/wait', {
        method: 'POST',
        timeout: 120000,  // 2 分钟超时
      });

      sdk.notification.success('OAuth 登录成功');
      onSuccess();
    } catch (error) {
      sdk.notification.error(`登录失败: ${error.message}`);
    } finally {
      setLoading(false);
    }
  };

  const handleFileImport = async (filePath: string) => {
    setLoading(true);

    try {
      await sdk.http.request('/api/antigravity/credentials/import', {
        method: 'POST',
        body: JSON.stringify({
          credsFilePath: filePath,
          projectId: projectId || undefined,
          name: name || undefined,
        }),
      });

      sdk.notification.success('凭证导入成功');
      onSuccess();
    } catch (error) {
      sdk.notification.error(`导入失败: ${error.message}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="antigravity-form">
      {/* 模式选择 */}
      <Tabs value={mode} onChange={setMode}>
        <Tab value="oauth">Google OAuth 登录</Tab>
        <Tab value="file">导入凭证文件</Tab>
      </Tabs>

      {/* 通用配置 */}
      <FormField>
        <Label>凭证名称（可选）</Label>
        <Input
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="我的 Antigravity 凭证"
        />
      </FormField>

      <FormField>
        <Label>Project ID（可选）</Label>
        <Input
          value={projectId}
          onChange={(e) => setProjectId(e.target.value)}
          placeholder="自动获取或手动输入"
        />
        <HelpText>留空将自动从 Google 账户获取</HelpText>
      </FormField>

      {mode === 'oauth' && (
        <OAuthMode
          loading={loading}
          onLogin={handleOAuthLogin}
        />
      )}

      {mode === 'file' && (
        <FileMode
          loading={loading}
          onImport={handleFileImport}
        />
      )}

      <FormActions>
        <Button variant="secondary" onClick={onCancel}>取消</Button>
      </FormActions>
    </div>
  );
}
```

### 4.3 OAuth 登录模式

```tsx
// src/components/OAuthLogin.tsx

interface OAuthModeProps {
  loading: boolean;
  onLogin: () => void;
}

export function OAuthMode({ loading, onLogin }: OAuthModeProps) {
  return (
    <div className="oauth-mode">
      <Alert type="info">
        <p>点击下方按钮将打开浏览器进行 Google 账户授权。</p>
        <p>授权完成后，凭证将自动添加。</p>
      </Alert>

      <div className="oauth-scopes">
        <h4>请求的权限：</h4>
        <ul>
          <li>Cloud Platform 访问</li>
          <li>用户邮箱和个人资料</li>
          <li>Cloud Code 日志</li>
        </ul>
      </div>

      <Button
        onClick={onLogin}
        loading={loading}
        disabled={loading}
        icon={<GoogleIcon />}
      >
        {loading ? '等待授权...' : '使用 Google 账户登录'}
      </Button>
    </div>
  );
}
```

### 4.4 凭证卡片

```tsx
// src/components/CredentialCard.tsx

interface CredentialCardProps {
  credential: Credential;
  onRefresh: () => void;
  onDelete: () => void;
}

export function CredentialCard({ credential, onRefresh, onDelete }: CredentialCardProps) {
  const data = JSON.parse(credential.credential_data) as AntigravityCredentials;
  const isHealthy = credential.status === 'active';

  return (
    <Card className={`credential-card ${isHealthy ? 'healthy' : 'unhealthy'}`}>
      <CardHeader>
        <div className="status-indicator">
          <StatusDot status={isHealthy ? 'green' : 'red'} />
          <span>{isHealthy ? '健康' : '异常'}</span>
        </div>
        <Badge variant="primary">Google OAuth</Badge>
      </CardHeader>

      <CardBody>
        <div className="info-row">
          <label>名称</label>
          <span>{credential.name || '未命名'}</span>
        </div>
        <div className="info-row">
          <label>邮箱</label>
          <span>{data.email || '-'}</span>
        </div>
        <div className="info-row">
          <label>Project ID</label>
          <span>{data.project_id || '自动'}</span>
        </div>
        <div className="info-row">
          <label>过期时间</label>
          <span>
            {data.expiry_date
              ? formatDate(new Date(data.expiry_date))
              : '-'
            }
          </span>
        </div>
        <div className="info-row">
          <label>上次刷新</label>
          <span>{data.last_refresh || '-'}</span>
        </div>

        {/* 支持的模型 */}
        <div className="supported-models">
          <label>支持的模型</label>
          <div className="model-tags">
            <Tag>Gemini 3 Pro</Tag>
            <Tag>Gemini 2.5 Flash</Tag>
            <Tag>Claude Sonnet 4.5</Tag>
            <Tag>Claude Opus 4.5</Tag>
          </div>
        </div>
      </CardBody>

      <CardFooter>
        <Button size="small" onClick={onRefresh}>刷新 Token</Button>
        <Button size="small" variant="danger" onClick={onDelete}>删除</Button>
      </CardFooter>
    </Card>
  );
}
```

### 4.5 设置面板

```tsx
// src/components/SettingsPanel.tsx

interface SettingsPanelProps {
  sdk: ProxyCastPluginSDK;
  pluginId: string;
  onClose: () => void;
}

export function SettingsPanel({ sdk, pluginId, onClose }: SettingsPanelProps) {
  const [config, setConfig] = useState<PluginConfig | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    const result = await sdk.database.query<{ config: string }>(
      'SELECT config FROM plugin_configs WHERE plugin_id = ?',
      [pluginId]
    );
    if (result.length > 0) {
      setConfig(JSON.parse(result[0].config));
    }
    setLoading(false);
  };

  const saveConfig = async () => {
    await sdk.database.execute(
      'UPDATE plugin_configs SET config = ? WHERE plugin_id = ?',
      [JSON.stringify(config), pluginId]
    );
    sdk.notification.success('设置已保存');
  };

  if (loading) return <Loading />;

  return (
    <div className="settings-panel">
      <h3>插件设置</h3>

      {/* API 环境 */}
      <FormField>
        <Label>API 环境</Label>
        <Select
          value={config?.settings?.api?.environment || 'daily'}
          onChange={(value) => setConfig({
            ...config,
            settings: {
              ...config?.settings,
              api: { ...config?.settings?.api, environment: value },
            },
          })}
        >
          <Option value="daily">Daily（推荐）</Option>
          <Option value="autopush">Autopush</Option>
        </Select>
      </FormField>

      {/* 思维链配置 */}
      <FormField>
        <Label>默认思维深度</Label>
        <Select
          value={config?.settings?.reasoning?.default_effort || 'medium'}
          onChange={(value) => setConfig({
            ...config,
            settings: {
              ...config?.settings,
              reasoning: { ...config?.settings?.reasoning, default_effort: value },
            },
          })}
        >
          <Option value="low">低（1024 tokens）</Option>
          <Option value="medium">中（4096 tokens）</Option>
          <Option value="high">高（16384 tokens）</Option>
        </Select>
        <HelpText>影响 thinking 模型的思考预算</HelpText>
      </FormField>

      {/* 安全设置 */}
      <FormField>
        <Label>安全设置</Label>
        <Checkbox
          checked={config?.settings?.safety_settings?.harassment === 'OFF'}
          onChange={(checked) => {
            const value = checked ? 'OFF' : 'BLOCK_MEDIUM_AND_ABOVE';
            setConfig({
              ...config,
              settings: {
                ...config?.settings,
                safety_settings: {
                  harassment: value,
                  hate_speech: value,
                  sexually_explicit: value,
                  dangerous_content: value,
                  civic_integrity: 'BLOCK_NONE',
                },
              },
            });
          }}
        >
          关闭所有内容过滤
        </Checkbox>
      </FormField>

      <FormActions>
        <Button variant="secondary" onClick={onClose}>取消</Button>
        <Button onClick={saveConfig}>保存设置</Button>
      </FormActions>
    </div>
  );
}
```

---

## 五、凭证文件格式

### 5.1 单凭证格式

**路径**: `~/.antigravity/oauth_creds.json`

```json
{
  "access_token": "ya29.a0AfH6SMC...",
  "refresh_token": "1//0gXXXXXXXXXXXX...",
  "token_type": "Bearer",
  "expiry_date": 1704369600000,
  "scope": "https://www.googleapis.com/auth/cloud-platform ...",
  "cred_type": "antigravity",
  "project_id": "my-project-123",
  "email": "user@example.com"
}
```

### 5.2 多凭证数组格式

**路径**: `accounts.json`（兼容 antigravity2api-nodejs）

```json
[
  {
    "access_token": "ya29.a0AfH6SMC...",
    "refresh_token": "1//0gXXXXXXXXXXXX...",
    "expiry_date": 1704369600000,
    "email": "user1@example.com",
    "enable": true
  },
  {
    "access_token": "ya29.b1BgH7TNE...",
    "refresh_token": "1//0hYYYYYYYYYYYY...",
    "expiry_date": 1704456000000,
    "email": "user2@example.com",
    "enable": true
  }
]
```

---

## 六、API 调用流程

### 6.1 完整调用链路

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    Antigravity Provider 调用流程                         │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  1. 客户端请求 (Anthropic/OpenAI 格式)                                    │
│     │                                                                    │
│     ▼                                                                    │
│  2. 模型识别 & 协议选择                                                   │
│     ├── gemini-* → 输出 Gemini 协议                                      │
│     └── claude-* → 输出 Anthropic 协议                                   │
│     │                                                                    │
│     ▼                                                                    │
│  3. 凭证获取                                                              │
│     ├── 从凭证池选择健康凭证                                              │
│     ├── 检查 Token 是否过期                                               │
│     └── 必要时刷新 Token                                                  │
│     │                                                                    │
│     ▼                                                                    │
│  4. 请求转换                                                              │
│     ├── 模型名映射（用户名 → 内部 API 名）                                │
│     ├── 消息格式转换                                                      │
│     ├── 附加 Safety Settings                                             │
│     └── 配置思维链（reasoning_effort）                                    │
│     │                                                                    │
│     ▼                                                                    │
│  5. Antigravity API 调用                                                  │
│     └── POST {base_url}/v1internal/models/{model}:streamGenerateContent  │
│     │                                                                    │
│     ▼                                                                    │
│  6. 响应转换（根据模型类型）                                               │
│     ├── Gemini 模型 → Gemini SSE 格式                                    │
│     └── Claude 模型 → Anthropic SSE 格式                                 │
│     │                                                                    │
│     ▼                                                                    │
│  7. 返回客户端                                                            │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 6.2 Antigravity API

**端点**: `POST {base_url}/v1internal/models/{model}:streamGenerateContent?alt=sse`

**请求格式**:

```json
{
  "model": "gemini-3-pro-high",
  "contents": [
    {
      "role": "user",
      "parts": [
        { "text": "Hello, how are you?" }
      ]
    }
  ],
  "systemInstruction": {
    "parts": [
      { "text": "You are a helpful assistant." }
    ]
  },
  "generationConfig": {
    "maxOutputTokens": 4096,
    "temperature": 0.7,
    "topP": 0.9,
    "candidateCount": 1,
    "thinkingConfig": {
      "thinkingBudget": 4096
    }
  },
  "safetySettings": [
    { "category": "HARM_CATEGORY_HARASSMENT", "threshold": "OFF" },
    { "category": "HARM_CATEGORY_HATE_SPEECH", "threshold": "OFF" },
    { "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT", "threshold": "OFF" },
    { "category": "HARM_CATEGORY_DANGEROUS_CONTENT", "threshold": "OFF" },
    { "category": "HARM_CATEGORY_CIVIC_INTEGRITY", "threshold": "BLOCK_NONE" }
  ]
}
```

**响应格式**: Gemini SSE Stream

---

## 七、错误处理

### 7.1 错误类型

| 错误码 | 说明 | 处理方式 |
|--------|------|---------|
| 401 | Token 过期 | 自动刷新 Token 重试 |
| 403 | 权限不足 / 账户无访问权限 | 标记凭证异常 |
| 429 | 请求限流 | 冷却期后重试 |
| 500 | 服务器错误 | 切换凭证重试 |

### 7.2 Token 刷新错误

```rust
pub enum TokenRefreshError {
    MissingRefreshToken,
    InvalidRefreshToken,
    NetworkError(String),
    AuthorizationRevoked,
}
```

---

## 八、开发指南

### 8.1 本地开发

```bash
# 克隆仓库
git clone https://github.com/aiclientproxy/antigravity-provider.git
cd antigravity-provider

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

# 协议转换测试
cargo test --test converter

# 前端测试
pnpm test
```

---

## 附录

### A. 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `ANTIGRAVITY_ENVIRONMENT` | API 环境 | `daily` |
| `ANTIGRAVITY_DEBUG` | 调试模式 | `false` |
| `ANTIGRAVITY_TIMEOUT_MS` | 请求超时 | `120000` |

### B. 参考链接

- [Google Cloud AI Platform](https://cloud.google.com/ai-platform)
- [Gemini API 文档](https://ai.google.dev/docs)
- [ProxyCast 插件开发指南](../prd/credential-provider-plugin-architecture.md)
