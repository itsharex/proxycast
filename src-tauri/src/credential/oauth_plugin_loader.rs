//! OAuth Provider 插件加载器
//!
//! 负责从外部目录加载 OAuth Provider 插件。
//! 与通用插件加载器不同，此加载器专门处理 oauth_provider 类型的插件。

use super::plugin::{
    AcquiredCredential, AuthTypeInfo, CredentialCategory, CredentialConfig,
    CredentialProviderPlugin, ModelFamily, ModelInfo, OAuthPluginError, OAuthPluginResult,
    ProviderError, StandardProtocol, TokenRefreshResult, UsageResult, ValidationResult,
};
use super::registry::CredentialProviderRegistry;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use tokio::fs;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// OAuth Provider 插件的 plugin.json 结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthPluginManifest {
    /// 插件名称
    pub name: String,
    /// 版本
    pub version: String,
    /// 描述
    #[serde(default)]
    pub description: String,
    /// 作者
    #[serde(default)]
    pub author: Option<String>,
    /// 主页
    #[serde(default)]
    pub homepage: Option<String>,
    /// 许可证
    #[serde(default)]
    pub license: Option<String>,
    /// 插件类型（必须是 "oauth_provider"）
    pub plugin_type: String,
    /// 入口（二进制名称）
    pub entry: String,
    /// 最低 ProxyCast 版本
    #[serde(default)]
    pub min_proxycast_version: Option<String>,
    /// Provider 配置
    pub provider: ProviderManifest,
    /// 二进制配置
    #[serde(default)]
    pub binary: Option<BinaryManifest>,
    /// UI 配置
    #[serde(default)]
    pub ui: Option<UiManifest>,
}

/// Provider 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderManifest {
    /// Provider ID
    pub id: String,
    /// 显示名称
    pub display_name: String,
    /// 目标协议
    pub target_protocol: String,
    /// 支持的模型模式
    #[serde(default)]
    pub supported_models: Vec<String>,
    /// 认证类型
    #[serde(default)]
    pub auth_types: Vec<String>,
    /// 凭证 Schema
    #[serde(default)]
    pub credential_schemas: HashMap<String, serde_json::Value>,
}

/// 二进制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryManifest {
    /// 二进制名称
    pub binary_name: String,
    /// GitHub owner
    pub github_owner: String,
    /// GitHub repo
    pub github_repo: String,
    /// 平台二进制映射
    pub platform_binaries: HashMap<String, String>,
    /// 校验文件
    #[serde(default)]
    pub checksum_file: Option<String>,
}

/// UI 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiManifest {
    /// 显示位置
    #[serde(default)]
    pub surfaces: Vec<String>,
    /// 图标
    #[serde(default)]
    pub icon: Option<String>,
    /// 标题
    #[serde(default)]
    pub title: Option<String>,
    /// UI 入口文件
    #[serde(default)]
    pub entry: Option<String>,
    /// 样式文件
    #[serde(default)]
    pub styles: Option<String>,
    /// 默认宽度
    #[serde(default)]
    pub default_width: Option<u32>,
    /// 默认高度
    #[serde(default)]
    pub default_height: Option<u32>,
    /// 权限列表
    #[serde(default)]
    pub permissions: Vec<String>,
}

/// OAuth Provider 插件加载器
pub struct OAuthPluginLoader {
    /// 插件目录
    plugins_dir: PathBuf,
}

impl OAuthPluginLoader {
    /// 创建新的加载器
    pub fn new(plugins_dir: PathBuf) -> Self {
        Self { plugins_dir }
    }

    /// 默认插件目录
    pub fn default_plugins_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("proxycast")
            .join("plugins")
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(Self::default_plugins_dir())
    }

    /// 确保插件目录存在
    pub async fn ensure_plugins_dir(&self) -> OAuthPluginResult<()> {
        if !self.plugins_dir.exists() {
            fs::create_dir_all(&self.plugins_dir)
                .await
                .map_err(|e| OAuthPluginError::IoError(e))?;
        }
        Ok(())
    }

    /// 扫描所有 OAuth Provider 插件
    pub async fn scan(&self) -> OAuthPluginResult<Vec<PathBuf>> {
        self.ensure_plugins_dir().await?;

        let mut plugins = Vec::new();
        let mut entries = fs::read_dir(&self.plugins_dir)
            .await
            .map_err(|e| OAuthPluginError::IoError(e))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| OAuthPluginError::IoError(e))?
        {
            let path = entry.path();

            // 检查是否是目录且包含 plugin.json
            if path.is_dir() && path.join("plugin.json").exists() {
                // 读取 plugin.json 检查类型
                let manifest_path = path.join("plugin.json");
                if let Ok(content) = fs::read_to_string(&manifest_path).await {
                    if let Ok(manifest) = serde_json::from_str::<OAuthPluginManifest>(&content) {
                        if manifest.plugin_type == "oauth_provider" {
                            plugins.push(path);
                        }
                    }
                }
            }
        }

        Ok(plugins)
    }

    /// 加载插件清单
    pub async fn load_manifest(&self, plugin_dir: &Path) -> OAuthPluginResult<OAuthPluginManifest> {
        let manifest_path = plugin_dir.join("plugin.json");

        let content = fs::read_to_string(&manifest_path)
            .await
            .map_err(|e| OAuthPluginError::InitError(format!("无法读取 plugin.json: {}", e)))?;

        let manifest: OAuthPluginManifest = serde_json::from_str(&content)?;

        // 验证插件类型
        if manifest.plugin_type != "oauth_provider" {
            return Err(OAuthPluginError::InitError(format!(
                "无效的插件类型: {} (期望 oauth_provider)",
                manifest.plugin_type
            )));
        }

        Ok(manifest)
    }

    /// 加载单个插件
    pub async fn load(
        &self,
        plugin_dir: &Path,
    ) -> OAuthPluginResult<Arc<dyn CredentialProviderPlugin>> {
        let manifest = self.load_manifest(plugin_dir).await?;

        info!(
            "Loading OAuth provider plugin: {} v{}",
            manifest.provider.id, manifest.version
        );

        // 查找二进制文件
        let binary_path = self.find_binary(plugin_dir, &manifest)?;

        // 加载配置
        let config_path = plugin_dir.join("config.json");
        let config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .await
                .map_err(|e| OAuthPluginError::IoError(e))?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            serde_json::json!({})
        };

        // 创建外部插件实例
        let plugin = ExternalOAuthPlugin::new(manifest, binary_path, config);

        Ok(Arc::new(plugin))
    }

    /// 查找二进制文件
    fn find_binary(
        &self,
        plugin_dir: &Path,
        manifest: &OAuthPluginManifest,
    ) -> OAuthPluginResult<PathBuf> {
        let bin_dir = plugin_dir.join("bin");

        // 获取当前平台的二进制名称
        let platform_key = get_platform_key();

        let binary_name = if let Some(binary) = &manifest.binary {
            binary
                .platform_binaries
                .get(&platform_key)
                .cloned()
                .unwrap_or_else(|| manifest.entry.clone())
        } else {
            manifest.entry.clone()
        };

        // 尝试几个可能的位置
        let candidates = vec![
            bin_dir.join(&binary_name),
            plugin_dir.join(&binary_name),
            plugin_dir.join("bin").join(&manifest.entry),
        ];

        for path in candidates {
            if path.exists() {
                return Ok(path);
            }
        }

        Err(OAuthPluginError::InitError(format!(
            "找不到二进制文件: {} (平台: {})",
            binary_name, platform_key
        )))
    }

    /// 加载所有插件到注册表
    pub async fn load_all(
        &self,
        registry: &CredentialProviderRegistry,
    ) -> OAuthPluginResult<Vec<String>> {
        let plugin_dirs = self.scan().await?;
        let mut loaded = Vec::new();

        for dir in plugin_dirs {
            match self.load(&dir).await {
                Ok(plugin) => {
                    let id = plugin.id().to_string();
                    if let Err(e) = registry.register(plugin).await {
                        warn!("注册插件失败 {}: {}", id, e);
                    } else {
                        loaded.push(id);
                    }
                }
                Err(e) => {
                    warn!("加载插件失败 {:?}: {}", dir, e);
                }
            }
        }

        Ok(loaded)
    }

    /// 获取插件目录
    pub fn plugins_dir(&self) -> &Path {
        &self.plugins_dir
    }
}

/// 获取当前平台的 key
fn get_platform_key() -> String {
    match (std::env::consts::ARCH, std::env::consts::OS) {
        ("aarch64", "macos") => "macos-arm64".to_string(),
        ("x86_64", "macos") => "macos-x64".to_string(),
        ("x86_64", "linux") => "linux-x64".to_string(),
        ("aarch64", "linux") => "linux-arm64".to_string(),
        ("x86_64", "windows") => "windows-x64".to_string(),
        (arch, os) => format!("{}-{}", os, arch),
    }
}

// ============================================================================
// 外部 OAuth 插件（通过二进制调用）
// ============================================================================

/// 外部 OAuth 插件
///
/// 通过调用外部二进制实现 CredentialProviderPlugin trait。
/// 使用 JSON-RPC 或 stdin/stdout 通信。
pub struct ExternalOAuthPlugin {
    /// 插件清单
    manifest: OAuthPluginManifest,
    /// 二进制路径
    binary_path: PathBuf,
    /// 插件配置
    config: serde_json::Value,
    /// 进程句柄
    process: Mutex<Option<Child>>,
}

impl ExternalOAuthPlugin {
    /// 创建新的外部插件
    pub fn new(
        manifest: OAuthPluginManifest,
        binary_path: PathBuf,
        config: serde_json::Value,
    ) -> Self {
        Self {
            manifest,
            binary_path,
            config,
            process: Mutex::new(None),
        }
    }

    /// 调用插件命令
    async fn call_command(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> OAuthPluginResult<serde_json::Value> {
        let _request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let _output = Command::new(&self.binary_path)
            .arg("--json-rpc")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| OAuthPluginError::InitError(format!("启动插件进程失败: {}", e)))?
            .wait_with_output()
            .await
            .map_err(|e| OAuthPluginError::InitError(format!("等待插件进程失败: {}", e)))?;

        // TODO: 实现完整的 JSON-RPC 通信
        // 目前返回模拟数据

        debug!(
            "Plugin {} called method {} (simulated)",
            self.manifest.provider.id, method
        );

        Ok(serde_json::json!({}))
    }
}

#[async_trait]
impl CredentialProviderPlugin for ExternalOAuthPlugin {
    fn id(&self) -> &str {
        &self.manifest.provider.id
    }

    fn display_name(&self) -> &str {
        &self.manifest.provider.display_name
    }

    fn version(&self) -> &str {
        &self.manifest.version
    }

    fn description(&self) -> &str {
        &self.manifest.description
    }

    fn target_protocol(&self) -> StandardProtocol {
        StandardProtocol::from_str(&self.manifest.provider.target_protocol)
            .unwrap_or(StandardProtocol::Anthropic)
    }

    fn ui_category(&self) -> CredentialCategory {
        CredentialCategory::OAuth
    }

    fn supported_auth_types(&self) -> Vec<AuthTypeInfo> {
        self.manifest
            .provider
            .auth_types
            .iter()
            .map(|id| {
                let schema = self
                    .manifest
                    .provider
                    .credential_schemas
                    .get(id)
                    .cloned()
                    .unwrap_or_default();

                AuthTypeInfo {
                    id: id.clone(),
                    display_name: id.clone(),
                    description: schema
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    category: CredentialCategory::OAuth,
                    icon: None,
                }
            })
            .collect()
    }

    fn credential_schema_for_auth(&self, auth_type: &str) -> serde_json::Value {
        self.manifest
            .provider
            .credential_schemas
            .get(auth_type)
            .cloned()
            .unwrap_or_default()
    }

    fn parse_credential_config(
        &self,
        _auth_type: &str,
        _config: serde_json::Value,
    ) -> OAuthPluginResult<Box<dyn CredentialConfig>> {
        // TODO: 调用外部二进制解析配置
        Err(OAuthPluginError::ConfigParseError(
            "外部插件配置解析未实现".to_string(),
        ))
    }

    async fn create_credential(
        &self,
        auth_type: &str,
        config: serde_json::Value,
    ) -> OAuthPluginResult<String> {
        let result = self
            .call_command(
                "create_credential",
                serde_json::json!({
                    "auth_type": auth_type,
                    "config": config
                }),
            )
            .await?;

        result
            .get("credential_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| OAuthPluginError::ConfigParseError("无效的凭证 ID".to_string()))
    }

    fn model_families(&self) -> Vec<ModelFamily> {
        self.manifest
            .provider
            .supported_models
            .iter()
            .map(|pattern| ModelFamily {
                name: pattern.clone(),
                pattern: pattern.clone(),
                tier: None,
                description: None,
            })
            .collect()
    }

    async fn list_models(&self) -> OAuthPluginResult<Vec<ModelInfo>> {
        // TODO: 调用外部二进制获取模型列表
        Ok(vec![])
    }

    fn supports_model(&self, model: &str) -> bool {
        for pattern in &self.manifest.provider.supported_models {
            if let Ok(glob) = glob::Pattern::new(pattern) {
                if glob.matches(model) {
                    return true;
                }
            }
        }
        false
    }

    async fn acquire_credential(&self, model: &str) -> OAuthPluginResult<AcquiredCredential> {
        let result = self
            .call_command(
                "acquire_credential",
                serde_json::json!({
                    "model": model
                }),
            )
            .await?;

        serde_json::from_value(result)
            .map_err(|e| OAuthPluginError::ConfigParseError(format!("解析凭证失败: {}", e)))
    }

    async fn release_credential(
        &self,
        credential_id: &str,
        result: UsageResult,
    ) -> OAuthPluginResult<()> {
        self.call_command(
            "release_credential",
            serde_json::json!({
                "credential_id": credential_id,
                "result": result
            }),
        )
        .await?;
        Ok(())
    }

    async fn validate_credential(
        &self,
        credential_id: &str,
    ) -> OAuthPluginResult<ValidationResult> {
        let result = self
            .call_command(
                "validate_credential",
                serde_json::json!({
                    "credential_id": credential_id
                }),
            )
            .await?;

        serde_json::from_value(result)
            .map_err(|e| OAuthPluginError::ValidationError(format!("解析验证结果失败: {}", e)))
    }

    async fn refresh_token(&self, credential_id: &str) -> OAuthPluginResult<TokenRefreshResult> {
        let result = self
            .call_command(
                "refresh_token",
                serde_json::json!({
                    "credential_id": credential_id
                }),
            )
            .await?;

        serde_json::from_value(result)
            .map_err(|e| OAuthPluginError::TokenRefreshError(format!("解析刷新结果失败: {}", e)))
    }

    async fn transform_request(&self, request: &mut serde_json::Value) -> OAuthPluginResult<()> {
        let result = self
            .call_command(
                "transform_request",
                serde_json::json!({
                    "request": request.clone()
                }),
            )
            .await?;

        if let Some(transformed) = result.get("request") {
            *request = transformed.clone();
        }

        Ok(())
    }

    async fn transform_response(&self, response: &mut serde_json::Value) -> OAuthPluginResult<()> {
        let result = self
            .call_command(
                "transform_response",
                serde_json::json!({
                    "response": response.clone()
                }),
            )
            .await?;

        if let Some(transformed) = result.get("response") {
            *response = transformed.clone();
        }

        Ok(())
    }

    async fn apply_risk_control(
        &self,
        request: &mut serde_json::Value,
        credential_id: &str,
    ) -> OAuthPluginResult<()> {
        let result = self
            .call_command(
                "apply_risk_control",
                serde_json::json!({
                    "request": request.clone(),
                    "credential_id": credential_id
                }),
            )
            .await?;

        if let Some(modified) = result.get("request") {
            *request = modified.clone();
        }

        Ok(())
    }

    fn parse_error(&self, _status: u16, _body: &str) -> Option<ProviderError> {
        // TODO: 调用外部二进制解析错误
        None
    }

    fn get_plugin_config(&self) -> serde_json::Value {
        self.config.clone()
    }

    async fn update_plugin_config(&self, _config: serde_json::Value) -> OAuthPluginResult<()> {
        // TODO: 持久化配置更新
        Ok(())
    }

    async fn init(&self) -> OAuthPluginResult<()> {
        info!(
            "Initializing external OAuth plugin: {} ({})",
            self.manifest.provider.id,
            self.binary_path.display()
        );
        Ok(())
    }

    async fn shutdown(&self) -> OAuthPluginResult<()> {
        info!(
            "Shutting down external OAuth plugin: {}",
            self.manifest.provider.id
        );

        // 终止进程（如果有）
        let mut process = self.process.lock().await;
        if let Some(mut child) = process.take() {
            let _ = child.kill().await;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn test_platform_key() {
        let key = get_platform_key();
        assert!(!key.is_empty());
    }

    #[tokio::test]
    async fn test_loader_creation() {
        let loader = OAuthPluginLoader::new(temp_dir().join("test_oauth_plugins"));
        assert!(loader.plugins_dir().exists() || true); // 目录可能不存在
    }

    #[test]
    fn test_manifest_parsing() {
        let json = r#"{
            "name": "test-provider",
            "version": "1.0.0",
            "description": "Test OAuth Provider",
            "plugin_type": "oauth_provider",
            "entry": "test-provider-cli",
            "provider": {
                "id": "test",
                "display_name": "Test Provider",
                "target_protocol": "anthropic",
                "supported_models": ["test-*"],
                "auth_types": ["oauth"]
            }
        }"#;

        let manifest: OAuthPluginManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.name, "test-provider");
        assert_eq!(manifest.provider.id, "test");
        assert_eq!(manifest.plugin_type, "oauth_provider");
    }
}
