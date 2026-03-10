use crate::app::AppState;
use crate::database::dao::api_key_provider::{ApiKeyProvider, ApiProviderType};
use dirs::{data_dir, home_dir};
use rand::{distributions::Alphanumeric, Rng};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::{HashSet, VecDeque};
use std::ffi::OsString;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::OnceLock;
use std::time::SystemTime;
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tokio::time::{sleep, timeout, Duration};

const DEFAULT_GATEWAY_PORT: u16 = 18790;
const OPENCLAW_INSTALL_EVENT: &str = "openclaw:install-progress";
const OPENCLAW_CONFIG_ENV: &str = "OPENCLAW_CONFIG_PATH";
const OPENCLAW_CN_PACKAGE: &str = "@qingchencloud/openclaw-zh@latest";
const OPENCLAW_DEFAULT_PACKAGE: &str = "openclaw@latest";
const NPM_MIRROR_CN: &str = "https://registry.npmmirror.com";
const NODE_MIN_VERSION: (u64, u64, u64) = (22, 0, 0);
const OPENCLAW_PROGRESS_LOG_LIMIT: usize = 400;
const OPENCLAW_INSTALLER_USER_AGENT: &str = "ProxyCast-OpenClaw";
const OPENCLAW_TEMP_CARGO_CHECK_DIR: &str = "/tmp/proxycast-cargo-check";
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinaryInstallStatus {
    pub installed: bool,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinaryAvailabilityStatus {
    pub available: bool,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeCheckResult {
    pub status: String,
    pub version: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionResult {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyStatus {
    pub status: String,
    pub version: Option<String>,
    pub path: Option<String>,
    pub message: String,
    pub auto_install_supported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentStatus {
    pub node: DependencyStatus,
    pub git: DependencyStatus,
    pub openclaw: DependencyStatus,
    pub recommended_action: String,
    pub summary: String,
    #[serde(default)]
    pub temp_artifacts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandPreview {
    pub title: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewayStatusInfo {
    pub status: GatewayStatus,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GatewayStatus {
    Stopped,
    Starting,
    Running,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthInfo {
    pub status: String,
    pub gateway_port: u16,
    pub uptime: Option<u64>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelInfo {
    pub id: String,
    pub name: String,
    pub channel_type: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallProgressEvent {
    pub message: String,
    pub level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncModelEntry {
    pub id: String,
    pub name: String,
    pub context_window: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DependencyKind {
    Node,
    Git,
}

impl DependencyKind {
    fn label(self) -> &'static str {
        match self {
            Self::Node => "Node.js",
            Self::Git => "Git",
        }
    }
}

#[derive(Debug, Clone)]
struct InstallerAsset {
    filename: String,
    download_url: String,
}

#[derive(Debug)]
pub struct OpenClawService {
    gateway_process: Option<Child>,
    gateway_status: GatewayStatus,
    gateway_port: u16,
    gateway_auth_token: String,
    gateway_started_at: Option<SystemTime>,
    progress_logs: VecDeque<InstallProgressEvent>,
}

impl Default for OpenClawService {
    fn default() -> Self {
        Self {
            gateway_process: None,
            gateway_status: GatewayStatus::Stopped,
            gateway_port: DEFAULT_GATEWAY_PORT,
            gateway_auth_token: String::new(),
            gateway_started_at: None,
            progress_logs: VecDeque::new(),
        }
    }
}

pub struct OpenClawServiceState(pub std::sync::Arc<Mutex<OpenClawService>>);

impl Default for OpenClawServiceState {
    fn default() -> Self {
        Self(std::sync::Arc::new(Mutex::new(OpenClawService::default())))
    }
}

impl OpenClawService {
    pub fn clear_progress_logs(&mut self) {
        self.progress_logs.clear();
    }

    pub fn get_progress_logs(&self) -> Vec<InstallProgressEvent> {
        self.progress_logs.iter().cloned().collect()
    }

    fn push_progress_log(&mut self, message: String, level: String) {
        if self.progress_logs.len() >= OPENCLAW_PROGRESS_LOG_LIMIT {
            self.progress_logs.pop_front();
        }
        self.progress_logs
            .push_back(InstallProgressEvent { message, level });
    }

    pub async fn get_command_preview(
        &mut self,
        app: &AppHandle,
        operation: &str,
        port: Option<u16>,
    ) -> Result<CommandPreview, String> {
        match operation {
            "install" => self.build_install_command_preview(app).await,
            "uninstall" => self.build_uninstall_command_preview().await,
            "restart" => self.build_restart_command_preview(port).await,
            "start" => self.build_start_command_preview(port).await,
            "stop" => self.build_stop_command_preview(port).await,
            _ => Err(format!("不支持的 OpenClaw 操作预览: {operation}")),
        }
    }

    pub async fn get_environment_status(&self) -> Result<EnvironmentStatus, String> {
        let node = inspect_node_dependency_status().await?;
        let git = inspect_git_dependency_status().await?;
        let openclaw = inspect_openclaw_dependency_status().await?;

        Ok(build_environment_status(node, git, openclaw))
    }

    pub async fn check_installed(&self) -> Result<BinaryInstallStatus, String> {
        let openclaw = inspect_openclaw_dependency_status().await?;
        Ok(BinaryInstallStatus {
            installed: openclaw.status == "ok",
            path: openclaw.path,
        })
    }

    pub async fn check_git_available(&self) -> Result<BinaryAvailabilityStatus, String> {
        let git = inspect_git_dependency_status().await?;
        Ok(BinaryAvailabilityStatus {
            available: git.status == "ok",
            path: git.path,
        })
    }

    pub async fn check_node_version(&self) -> Result<NodeCheckResult, String> {
        let node = inspect_node_dependency_status().await?;
        Ok(NodeCheckResult {
            status: match node.status.as_str() {
                "missing" => "not_found".to_string(),
                other => other.to_string(),
            },
            version: node.version,
            path: node.path,
        })
    }

    pub fn get_node_download_url(&self) -> String {
        if cfg!(target_os = "windows") {
            "https://nodejs.org/en/download".to_string()
        } else if cfg!(target_os = "macos") {
            "https://nodejs.org/en/download".to_string()
        } else if cfg!(target_os = "linux") {
            "https://nodejs.org/en/download".to_string()
        } else {
            "https://nodejs.org/en/download".to_string()
        }
    }

    pub fn get_git_download_url(&self) -> String {
        if cfg!(target_os = "windows") {
            "https://git-scm.com/download/win".to_string()
        } else if cfg!(target_os = "macos") {
            "https://git-scm.com/download/mac".to_string()
        } else if cfg!(target_os = "linux") {
            "https://git-scm.com/download/linux".to_string()
        } else {
            "https://git-scm.com/downloads".to_string()
        }
    }

    pub async fn install(&mut self, app: &AppHandle) -> Result<ActionResult, String> {
        emit_install_progress(app, "开始准备 OpenClaw 环境。", "info");

        let node_result = self
            .ensure_dependency_ready(app, DependencyKind::Node)
            .await?;
        if !node_result.success {
            return Ok(node_result);
        }

        let git_result = self
            .ensure_dependency_ready(app, DependencyKind::Git)
            .await?;
        if !git_result.success {
            return Ok(git_result);
        }

        let (_, npm_path, npm_prefix, cleanup_command, install_command) =
            self.resolve_install_commands(app).await?;
        let command = format!("{cleanup_command}\n{install_command}");

        emit_install_progress(app, &format!("使用 npm: {npm_path}"), "info");
        if let Some(prefix) = npm_prefix {
            emit_install_progress(app, &format!("npm 全局前缀: {prefix}"), "info");
        }
        emit_install_progress(app, "安装前先清理已有 OpenClaw 全局包。", "info");

        emit_install_progress(app, &format!("执行安装命令: {install_command}"), "info");
        let result = run_shell_command_with_progress(app, &command).await?;
        if !result.success {
            return Ok(result);
        }

        let installed = self.check_installed().await?;
        if installed.installed {
            emit_install_progress(app, "已检测到 OpenClaw 可执行文件。", "info");
            return Ok(ActionResult {
                success: true,
                message: installed
                    .path
                    .map(|path| format!("OpenClaw 安装完成：{path}"))
                    .unwrap_or_else(|| "OpenClaw 安装完成。".to_string()),
            });
        }

        Ok(ActionResult {
            success: false,
            message:
                "安装命令执行完成，但仍未检测到 OpenClaw 可执行文件，请检查 npm 全局目录或权限设置。"
                    .to_string(),
        })
    }

    pub async fn install_dependency(
        &mut self,
        app: &AppHandle,
        kind: &str,
    ) -> Result<ActionResult, String> {
        let dependency = match kind {
            "node" => DependencyKind::Node,
            "git" => DependencyKind::Git,
            _ => return Err(format!("不支持的依赖类型: {kind}")),
        };

        self.ensure_dependency_ready(app, dependency).await
    }

    pub async fn cleanup_temp_artifacts(
        &mut self,
        app: Option<&AppHandle>,
    ) -> Result<ActionResult, String> {
        let mut removed = Vec::new();
        let mut failed = Vec::new();

        for target in collect_temp_artifact_paths(app) {
            if !target.exists() {
                continue;
            }

            let result = if target.is_dir() {
                std::fs::remove_dir_all(&target)
            } else {
                std::fs::remove_file(&target)
            };

            match result {
                Ok(_) => {
                    if let Some(app) = app {
                        emit_install_progress(
                            app,
                            &format!("已清理临时文件：{}", target.display()),
                            "info",
                        );
                    }
                    removed.push(target.display().to_string());
                }
                Err(error) => {
                    if let Some(app) = app {
                        emit_install_progress(
                            app,
                            &format!("清理临时文件失败({}): {error}", target.display()),
                            "warn",
                        );
                    }
                    failed.push(format!("{}: {error}", target.display()));
                }
            }
        }

        if failed.is_empty() {
            Ok(ActionResult {
                success: true,
                message: if removed.is_empty() {
                    "未发现需要清理的 OpenClaw 临时文件。".to_string()
                } else {
                    format!("已清理 {} 项临时文件。", removed.len())
                },
            })
        } else {
            Ok(ActionResult {
                success: false,
                message: format!("部分临时文件清理失败：{}", failed.join("；")),
            })
        }
    }

    pub async fn uninstall(&mut self, app: &AppHandle) -> Result<ActionResult, String> {
        if self.gateway_status == GatewayStatus::Running || self.gateway_process.is_some() {
            let _ = self.stop_gateway(None).await;
        }

        let (npm_path, npm_prefix, command) = self.resolve_uninstall_command().await?;

        emit_install_progress(app, &format!("使用 npm: {npm_path}"), "info");
        if let Some(prefix) = npm_prefix {
            emit_install_progress(app, &format!("npm 全局前缀: {prefix}"), "info");
        }
        emit_install_progress(app, &format!("执行卸载命令: {command}"), "info");
        run_shell_command_with_progress(app, &command).await
    }

    async fn ensure_dependency_ready(
        &mut self,
        app: &AppHandle,
        dependency: DependencyKind,
    ) -> Result<ActionResult, String> {
        let status = self.inspect_dependency_status(dependency).await?;
        if status.status == "ok" {
            emit_install_progress(
                app,
                &format!(
                    "{} 已就绪{}。",
                    dependency.label(),
                    status
                        .version
                        .as_deref()
                        .map(|version| format!(" · {version}"))
                        .unwrap_or_default()
                ),
                "info",
            );
            return Ok(ActionResult {
                success: true,
                message: format!("{} 已满足要求。", dependency.label()),
            });
        }

        emit_install_progress(
            app,
            &format!("{}，开始修复 {} 环境。", status.message, dependency.label()),
            "warn",
        );

        match dependency {
            DependencyKind::Node => self.install_node_runtime(app).await,
            DependencyKind::Git => self.install_git_runtime(app).await,
        }
    }

    async fn inspect_dependency_status(
        &self,
        dependency: DependencyKind,
    ) -> Result<DependencyStatus, String> {
        match dependency {
            DependencyKind::Node => inspect_node_dependency_status().await,
            DependencyKind::Git => inspect_git_dependency_status().await,
        }
    }

    async fn install_node_runtime(&mut self, app: &AppHandle) -> Result<ActionResult, String> {
        #[cfg(target_os = "windows")]
        {
            if let Some(winget_path) = find_command_in_shell("winget").await? {
                emit_install_progress(app, "检测到 winget，准备通过 winget 安装 Node.js。", "info");
                let command = format!(
                    "{}{} install --id OpenJS.NodeJS.LTS -e --accept-source-agreements --accept-package-agreements",
                    shell_path_assignment(&winget_path),
                    shell_command_escape(&winget_path)
                );
                let result = run_shell_command_with_progress(app, &command).await?;
                if !result.success {
                    return Ok(result);
                }
                return self
                    .verify_dependency_after_install(app, DependencyKind::Node)
                    .await;
            }

            emit_install_progress(
                app,
                "未检测到 winget，准备下载官方 Node.js 安装器。",
                "warn",
            );
            let asset = resolve_node_installer_asset().await?;
            let installer_path = download_installer_asset(app, &asset).await?;
            launch_installer(&installer_path)?;
            return self
                .wait_for_dependency_ready(app, DependencyKind::Node, 900)
                .await;
        }

        #[cfg(target_os = "macos")]
        {
            if let Some(brew_path) = find_command_in_shell("brew").await? {
                emit_install_progress(
                    app,
                    "检测到 Homebrew，准备通过 Homebrew 安装 Node.js。",
                    "info",
                );
                let brew_cmd = shell_command_escape(&brew_path);
                let path_env = shell_path_assignment(&brew_path);
                let command = format!(
                    "{path_env}{brew_cmd} install node || {path_env}{brew_cmd} upgrade node"
                );
                let result = run_shell_command_with_progress(app, &command).await?;
                if !result.success {
                    return Ok(result);
                }
                return self
                    .verify_dependency_after_install(app, DependencyKind::Node)
                    .await;
            }

            emit_install_progress(
                app,
                "未检测到 Homebrew，准备下载官方 Node.js 安装器。",
                "warn",
            );
            let asset = resolve_node_installer_asset().await?;
            let installer_path = download_installer_asset(app, &asset).await?;
            launch_installer(&installer_path)?;
            return self
                .wait_for_dependency_ready(app, DependencyKind::Node, 900)
                .await;
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            let message = "当前平台暂不支持应用内自动安装 Node.js，请手动安装 Node.js 22+ 后重试。"
                .to_string();
            emit_install_progress(app, &message, "warn");
            Ok(ActionResult {
                success: false,
                message,
            })
        }
    }

    async fn install_git_runtime(&mut self, app: &AppHandle) -> Result<ActionResult, String> {
        #[cfg(target_os = "windows")]
        {
            if let Some(winget_path) = find_command_in_shell("winget").await? {
                emit_install_progress(app, "检测到 winget，准备通过 winget 安装 Git。", "info");
                let command = format!(
                    "{}{} install --id Git.Git -e --accept-source-agreements --accept-package-agreements",
                    shell_path_assignment(&winget_path),
                    shell_command_escape(&winget_path)
                );
                let result = run_shell_command_with_progress(app, &command).await?;
                if !result.success {
                    return Ok(result);
                }
                return self
                    .verify_dependency_after_install(app, DependencyKind::Git)
                    .await;
            }

            let message =
                "当前系统缺少 winget，暂时无法一键安装 Git，请点击“手动下载 Git”完成安装后重试。"
                    .to_string();
            emit_install_progress(app, &message, "warn");
            return Ok(ActionResult {
                success: false,
                message,
            });
        }

        #[cfg(target_os = "macos")]
        {
            if let Some(brew_path) = find_command_in_shell("brew").await? {
                emit_install_progress(app, "检测到 Homebrew，准备通过 Homebrew 安装 Git。", "info");
                let brew_cmd = shell_command_escape(&brew_path);
                let path_env = shell_path_assignment(&brew_path);
                let command =
                    format!("{path_env}{brew_cmd} install git || {path_env}{brew_cmd} upgrade git");
                let result = run_shell_command_with_progress(app, &command).await?;
                if !result.success {
                    return Ok(result);
                }
                return self
                    .verify_dependency_after_install(app, DependencyKind::Git)
                    .await;
            }

            emit_install_progress(
                app,
                "未检测到 Homebrew，准备拉起 macOS Command Line Tools 安装器。",
                "warn",
            );
            let trigger_result = trigger_macos_command_line_tools_install().await?;
            emit_install_progress(app, &trigger_result, "info");
            return self
                .wait_for_dependency_ready(app, DependencyKind::Git, 1200)
                .await;
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            let message = "当前平台暂不支持应用内自动安装 Git，请使用系统包管理器手动安装后重试。"
                .to_string();
            emit_install_progress(app, &message, "warn");
            Ok(ActionResult {
                success: false,
                message,
            })
        }
    }

    async fn verify_dependency_after_install(
        &self,
        app: &AppHandle,
        dependency: DependencyKind,
    ) -> Result<ActionResult, String> {
        let status = self.inspect_dependency_status(dependency).await?;
        if status.status == "ok" {
            emit_install_progress(
                app,
                &format!(
                    "{} 已准备完成{}。",
                    dependency.label(),
                    status
                        .version
                        .as_deref()
                        .map(|version| format!(" · {version}"))
                        .unwrap_or_default()
                ),
                "info",
            );
            return Ok(ActionResult {
                success: true,
                message: format!("{} 已安装完成。", dependency.label()),
            });
        }

        Ok(ActionResult {
            success: false,
            message: format!(
                "{} 安装完成后仍未通过校验：{}",
                dependency.label(),
                status.message
            ),
        })
    }

    async fn wait_for_dependency_ready(
        &self,
        app: &AppHandle,
        dependency: DependencyKind,
        timeout_secs: u64,
    ) -> Result<ActionResult, String> {
        emit_install_progress(
            app,
            &format!(
                "已拉起 {} 安装器，正在等待安装完成（最长 {} 秒）。",
                dependency.label(),
                timeout_secs
            ),
            "info",
        );

        let start = tokio::time::Instant::now();
        let mut last_notice_at = 0_u64;
        while start.elapsed() < Duration::from_secs(timeout_secs) {
            let elapsed = start.elapsed().as_secs();
            if elapsed >= last_notice_at + 15 {
                last_notice_at = elapsed;
                emit_install_progress(
                    app,
                    &format!("正在等待 {} 安装完成…", dependency.label()),
                    "info",
                );
            }

            sleep(Duration::from_secs(2)).await;
            let status = self.inspect_dependency_status(dependency).await?;
            if status.status == "ok" {
                emit_install_progress(
                    app,
                    &format!(
                        "{} 已检测通过{}。",
                        dependency.label(),
                        status
                            .version
                            .as_deref()
                            .map(|version| format!(" · {version}"))
                            .unwrap_or_default()
                    ),
                    "info",
                );
                return Ok(ActionResult {
                    success: true,
                    message: format!("{} 已安装完成。", dependency.label()),
                });
            }
        }

        Ok(ActionResult {
            success: false,
            message: format!(
                "等待 {} 安装完成超时，请完成安装后重新点击重试。",
                dependency.label()
            ),
        })
    }

    pub async fn start_gateway(
        &mut self,
        app: Option<&AppHandle>,
        port: Option<u16>,
    ) -> Result<ActionResult, String> {
        if let Some(next_port) = port {
            self.gateway_port = next_port.max(1);
        }

        if let Some(app) = app {
            emit_install_progress(
                app,
                &format!("准备启动 Gateway，目标端口 {}。", self.gateway_port),
                "info",
            );
        }

        self.ensure_runtime_config(None, None)?;
        self.refresh_process_state().await?;

        if self.gateway_status == GatewayStatus::Running {
            if let Some(app) = app {
                emit_install_progress(
                    app,
                    &format!("检测到 Gateway 已在端口 {} 运行。", self.gateway_port),
                    "info",
                );
            }
            return Ok(ActionResult {
                success: true,
                message: format!("Gateway 已在端口 {} 运行", self.gateway_port),
            });
        }

        let Some(binary) = find_command_in_shell("openclaw").await? else {
            self.gateway_status = GatewayStatus::Error;
            if let Some(app) = app {
                emit_install_progress(app, "未检测到 OpenClaw 可执行文件，请先安装。", "error");
            }
            return Ok(ActionResult {
                success: false,
                message: "未检测到 OpenClaw 可执行文件，请先安装。".to_string(),
            });
        };

        self.gateway_status = GatewayStatus::Starting;

        let config_path = openclaw_proxycast_config_path();
        if let Some(app) = app {
            emit_install_progress(
                app,
                &format!("使用配置文件启动 Gateway: {}", config_path.display()),
                "info",
            );
        }
        let mut command = Command::new(&binary);
        apply_binary_runtime_path(&mut command, &binary);
        command
            .arg("gateway")
            .arg("--port")
            .arg(self.gateway_port.to_string())
            .env(OPENCLAW_CONFIG_ENV, &config_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = command
            .spawn()
            .map_err(|e| format!("启动 Gateway 失败: {e}"))?;

        if let Some(stdout) = child.stdout.take() {
            tokio::spawn(async move {
                let mut lines = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    tracing::info!(target: "openclaw", "Gateway stdout: {}", line);
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(async move {
                let mut lines = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    tracing::warn!(target: "openclaw", "Gateway stderr: {}", line);
                }
            });
        }

        self.gateway_process = Some(child);
        self.gateway_started_at = Some(SystemTime::now());

        if let Some(app) = app {
            emit_install_progress(app, "Gateway 进程已拉起，等待服务就绪。", "info");
        }

        let start_at = tokio::time::Instant::now();
        while start_at.elapsed() < Duration::from_secs(30) {
            sleep(Duration::from_millis(300)).await;
            self.refresh_process_state().await?;
            if self.gateway_status == GatewayStatus::Running {
                if let Some(app) = app {
                    emit_install_progress(
                        app,
                        &format!("Gateway 启动成功，监听端口 {}。", self.gateway_port),
                        "info",
                    );
                }
                return Ok(ActionResult {
                    success: true,
                    message: format!("Gateway 已启动，端口 {}", self.gateway_port),
                });
            }

            if self.check_port_open().await {
                self.gateway_status = GatewayStatus::Running;
                if let Some(app) = app {
                    emit_install_progress(
                        app,
                        &format!("Gateway 探测成功，监听端口 {}。", self.gateway_port),
                        "info",
                    );
                }
                return Ok(ActionResult {
                    success: true,
                    message: format!("Gateway 已启动，端口 {}", self.gateway_port),
                });
            }
        }

        self.gateway_status = GatewayStatus::Error;
        if let Some(app) = app {
            emit_install_progress(app, "Gateway 启动超时，请检查配置或端口占用。", "error");
        }
        Ok(ActionResult {
            success: false,
            message: "Gateway 启动超时，请检查配置或端口占用。".to_string(),
        })
    }

    pub async fn stop_gateway(&mut self, app: Option<&AppHandle>) -> Result<ActionResult, String> {
        if let Some(app) = app {
            emit_install_progress(app, "准备停止 Gateway。", "info");
        }

        if let Some(mut child) = self.gateway_process.take() {
            if let Some(app) = app {
                emit_install_progress(app, "正在终止当前托管的 Gateway 子进程。", "info");
            }
            let _ = child.kill().await;
            let _ = timeout(Duration::from_secs(3), child.wait()).await;
        } else {
            let binary = find_command_in_shell("openclaw").await?;
            if let Some(openclaw_path) = binary.as_deref() {
                let mut cmd = Command::new(openclaw_path);
                apply_binary_runtime_path(&mut cmd, openclaw_path);
                cmd.arg("gateway")
                    .arg("stop")
                    .arg("--url")
                    .arg(self.gateway_ws_url())
                    .arg("--token")
                    .arg(&self.gateway_auth_token)
                    .env(OPENCLAW_CONFIG_ENV, openclaw_proxycast_config_path())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null());
                match timeout(Duration::from_secs(5), cmd.status()).await {
                    Ok(Ok(status)) if status.success() => {
                        if let Some(app) = app {
                            emit_install_progress(app, "已发送 Gateway 停止命令。", "info");
                        }
                    }
                    Ok(Ok(status)) => {
                        if let Some(app) = app {
                            emit_install_progress(
                                app,
                                &format!("Gateway 停止命令返回异常状态: {:?}", status.code()),
                                "warn",
                            );
                        }
                    }
                    Ok(Err(error)) => {
                        if let Some(app) = app {
                            emit_install_progress(
                                app,
                                &format!("执行 Gateway 停止命令失败: {error}"),
                                "warn",
                            );
                        }
                    }
                    Err(_) => {
                        if let Some(app) = app {
                            emit_install_progress(
                                app,
                                "Gateway 停止命令超时，继续本地状态收敛。",
                                "warn",
                            );
                        }
                    }
                }
            }
        }

        self.gateway_status = GatewayStatus::Stopped;
        self.gateway_started_at = None;

        if let Some(app) = app {
            emit_install_progress(app, "Gateway 已停止。", "info");
        }

        Ok(ActionResult {
            success: true,
            message: "Gateway 已停止。".to_string(),
        })
    }

    pub async fn restart_gateway(&mut self, app: &AppHandle) -> Result<ActionResult, String> {
        emit_install_progress(app, "开始重启 Gateway。", "info");
        let _ = self.stop_gateway(Some(app)).await;
        emit_install_progress(app, "Gateway 停止阶段结束，开始重新启动。", "info");
        self.start_gateway(Some(app), Some(self.gateway_port)).await
    }

    pub async fn get_status(&mut self) -> Result<GatewayStatusInfo, String> {
        self.refresh_process_state().await?;
        Ok(GatewayStatusInfo {
            status: self.gateway_status.clone(),
            port: self.gateway_port,
        })
    }

    pub async fn check_health(&mut self) -> Result<HealthInfo, String> {
        self.refresh_process_state().await?;

        self.restore_auth_token_from_config();

        let health_snapshot = self.fetch_authenticated_gateway_health_json().await;
        let healthy = self.gateway_status == GatewayStatus::Running
            && self.check_port_open().await
            && health_snapshot
                .as_ref()
                .and_then(|value| value.get("ok").and_then(Value::as_bool))
                .unwrap_or(false);
        let version = self.read_openclaw_version().await.ok().flatten();
        let uptime = self.gateway_started_at.and_then(|start| {
            SystemTime::now()
                .duration_since(start)
                .ok()
                .map(|elapsed| elapsed.as_secs())
        });

        Ok(HealthInfo {
            status: if healthy { "healthy" } else { "unhealthy" }.to_string(),
            gateway_port: self.gateway_port,
            uptime,
            version,
        })
    }

    pub fn get_dashboard_url(&mut self) -> String {
        self.restore_auth_token_from_config();
        let mut url = format!("http://127.0.0.1:{}", self.gateway_port);
        if !self.gateway_auth_token.is_empty() {
            url.push_str(&format!(
                "/#token={}",
                urlencoding::encode(&self.gateway_auth_token)
            ));
        }
        url
    }

    pub async fn get_channels(&mut self) -> Result<Vec<ChannelInfo>, String> {
        self.refresh_process_state().await?;
        if self.gateway_status != GatewayStatus::Running {
            return Ok(Vec::new());
        }

        self.restore_auth_token_from_config();

        let Some(body) = self.fetch_authenticated_gateway_health_json().await else {
            return Ok(Vec::new());
        };

        let channels_map = body
            .get("channels")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let labels = body
            .get("channelLabels")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let ordered_ids = body
            .get("channelOrder")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        let mut ordered = Vec::new();
        for channel_id in ordered_ids.iter().filter_map(Value::as_str) {
            if let Some(entry) = channels_map.get(channel_id) {
                ordered.push(build_channel_info(
                    channel_id,
                    entry,
                    labels.get(channel_id),
                ));
            }
        }

        if ordered.is_empty() {
            ordered = channels_map
                .iter()
                .map(|(channel_id, entry)| {
                    build_channel_info(channel_id, entry, labels.get(channel_id))
                })
                .collect();
        }

        Ok(ordered)
    }

    pub fn sync_provider_config(
        &mut self,
        provider: &ApiKeyProvider,
        api_key: &str,
        primary_model_id: &str,
        models: &[SyncModelEntry],
    ) -> Result<ActionResult, String> {
        if api_key.trim().is_empty() && provider.provider_type != ApiProviderType::Ollama {
            return Ok(ActionResult {
                success: false,
                message: "该 Provider 没有可用的 API Key。".to_string(),
            });
        }

        let api_type = determine_api_type(provider.provider_type)?;
        let base_url = format_provider_base_url(provider)?;
        let provider_key = format!("proxycast-{}", provider.id);

        let normalized_models = if models.is_empty() {
            vec![SyncModelEntry {
                id: primary_model_id.to_string(),
                name: primary_model_id.to_string(),
                context_window: None,
            }]
        } else {
            let mut items = models.to_vec();
            if !items.iter().any(|item| item.id == primary_model_id) {
                items.insert(
                    0,
                    SyncModelEntry {
                        id: primary_model_id.to_string(),
                        name: primary_model_id.to_string(),
                        context_window: None,
                    },
                );
            }
            items
        };

        self.ensure_runtime_config(
            Some((
                &provider_key,
                json!({
                    "baseUrl": base_url,
                    "apiKey": api_key,
                    "api": api_type,
                    "models": normalized_models.iter().map(|model| {
                        json!({
                            "id": model.id,
                            "name": model.name,
                            "contextWindow": model.context_window,
                        })
                    }).collect::<Vec<_>>()
                }),
            )),
            Some(format!("{provider_key}/{primary_model_id}")),
        )?;

        Ok(ActionResult {
            success: true,
            message: format!("已同步 Provider“{}”到 OpenClaw。", provider.name),
        })
    }

    async fn refresh_process_state(&mut self) -> Result<(), String> {
        let mut process_exited = false;

        if let Some(child) = self.gateway_process.as_mut() {
            match child.try_wait() {
                Ok(Some(status)) => {
                    tracing::info!(target: "openclaw", "Gateway 进程已退出: {}", status);
                    process_exited = true;
                }
                Ok(None) => {}
                Err(error) => {
                    tracing::warn!(target: "openclaw", "检查 Gateway 进程状态失败: {}", error);
                    process_exited = true;
                }
            }
        }

        if process_exited {
            self.gateway_process = None;
            self.gateway_started_at = None;
        }

        let binary = find_command_in_shell("openclaw").await?;
        let running =
            self.check_port_open().await || self.check_gateway_status(binary.as_deref()).await?;

        self.gateway_status = if running {
            GatewayStatus::Running
        } else if self.gateway_status == GatewayStatus::Starting {
            GatewayStatus::Error
        } else {
            GatewayStatus::Stopped
        };

        if !running {
            self.gateway_process = None;
            self.gateway_started_at = None;
        }

        Ok(())
    }

    async fn check_port_open(&self) -> bool {
        timeout(
            Duration::from_secs(2),
            TcpStream::connect(("127.0.0.1", self.gateway_port)),
        )
        .await
        .map(|result| result.is_ok())
        .unwrap_or(false)
    }

    async fn check_gateway_status(&self, binary: Option<&str>) -> Result<bool, String> {
        let Some(openclaw_path) = binary else {
            return Ok(false);
        };

        let mut command = Command::new(openclaw_path);
        apply_binary_runtime_path(&mut command, &openclaw_path);
        let output = command
            .arg("gateway")
            .arg("status")
            .arg("--url")
            .arg(self.gateway_ws_url())
            .arg("--token")
            .arg(&self.gateway_auth_token)
            .env(OPENCLAW_CONFIG_ENV, openclaw_proxycast_config_path())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout).to_lowercase();
                let stderr = String::from_utf8_lossy(&result.stderr).to_lowercase();
                Ok(result.status.success()
                    && (stdout.contains("listening")
                        || stdout.contains("running")
                        || stderr.contains("listening")))
            }
            Err(_) => Ok(false),
        }
    }

    async fn read_openclaw_version(&self) -> Result<Option<String>, String> {
        let Some(binary) = find_command_in_shell("openclaw").await? else {
            return Ok(None);
        };

        let mut command = Command::new(&binary);
        apply_binary_runtime_path(&mut command, &binary);
        let output = command
            .arg("--version")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("读取 OpenClaw 版本失败: {e}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if stdout.is_empty() {
            Ok(None)
        } else {
            Ok(Some(stdout))
        }
    }

    fn gateway_ws_url(&self) -> String {
        format!("ws://127.0.0.1:{}", self.gateway_port)
    }

    fn restore_auth_token_from_config(&mut self) {
        if !self.gateway_auth_token.is_empty() {
            return;
        }

        match read_base_openclaw_config()
            .ok()
            .and_then(|config| extract_gateway_auth_token(&config))
        {
            Some(token) => {
                self.gateway_auth_token = token;
            }
            None => {
                tracing::warn!(
                    target: "openclaw",
                    "未能从 OpenClaw 配置恢复 gateway token，Dashboard 访问可能鉴权失败"
                );
            }
        }
    }

    async fn fetch_authenticated_gateway_health_json(&self) -> Option<Value> {
        if self.gateway_auth_token.is_empty() {
            return None;
        }

        let Some(openclaw_path) = find_command_in_shell("openclaw").await.ok().flatten() else {
            return None;
        };

        let mut command = Command::new(&openclaw_path);
        apply_binary_runtime_path(&mut command, &openclaw_path);
        let output = command
            .arg("gateway")
            .arg("health")
            .arg("--url")
            .arg(self.gateway_ws_url())
            .arg("--token")
            .arg(&self.gateway_auth_token)
            .arg("--json")
            .env(OPENCLAW_CONFIG_ENV, openclaw_proxycast_config_path())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match output {
            Ok(output) if output.status.success() => {
                serde_json::from_slice::<Value>(&output.stdout)
                    .map_err(|error| {
                        tracing::warn!(
                            target: "openclaw",
                            "解析 Gateway 官方健康检查结果失败: {}",
                            error
                        );
                        error
                    })
                    .ok()
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::warn!(
                    target: "openclaw",
                    "Gateway 官方健康检查失败: {}",
                    stderr.trim()
                );
                None
            }
            Err(error) => {
                tracing::warn!(target: "openclaw", "执行 Gateway 官方健康检查失败: {}", error);
                None
            }
        }
    }

    fn ensure_runtime_config(
        &mut self,
        provider_entry: Option<(&str, Value)>,
        primary_model: Option<String>,
    ) -> Result<(), String> {
        let config_dir = openclaw_config_dir();
        std::fs::create_dir_all(&config_dir).map_err(|e| format!("创建配置目录失败: {e}"))?;

        let proxycast_config_path = openclaw_proxycast_config_path();
        let mut config = read_base_openclaw_config()?;

        if self.gateway_auth_token.is_empty() {
            self.gateway_auth_token = generate_auth_token();
        }

        ensure_path_object(&mut config, &["gateway"]);
        set_json_path(
            &mut config,
            &["gateway", "mode"],
            Value::String("local".to_string()),
        );
        set_json_path(
            &mut config,
            &["gateway", "port"],
            Value::Number(self.gateway_port.into()),
        );
        set_json_path(
            &mut config,
            &["gateway", "auth", "token"],
            Value::String(self.gateway_auth_token.clone()),
        );
        set_json_path(
            &mut config,
            &["gateway", "remote", "token"],
            Value::String(self.gateway_auth_token.clone()),
        );

        if let Some((provider_key, provider_value)) = provider_entry {
            set_json_path(
                &mut config,
                &["models", "mode"],
                Value::String("merge".to_string()),
            );
            set_json_path(
                &mut config,
                &["models", "providers", provider_key],
                provider_value,
            );
        }

        if let Some(primary) = primary_model {
            set_json_path(
                &mut config,
                &["agents", "defaults", "model", "primary"],
                Value::String(primary),
            );
        }

        let content =
            serde_json::to_string_pretty(&config).map_err(|e| format!("序列化配置失败: {e}"))?;
        std::fs::write(proxycast_config_path, content).map_err(|e| format!("写入配置失败: {e}"))?;
        Ok(())
    }

    async fn resolve_install_commands(
        &self,
        app: &AppHandle,
    ) -> Result<(String, String, Option<String>, String, String), String> {
        let npm_path = find_command_in_shell("npm")
            .await?
            .ok_or_else(|| "未检测到 npm，可先安装或修复 Node.js 环境。".to_string())?;
        let npm_prefix = detect_npm_global_prefix(&npm_path).await;
        let package = if should_use_china_package(app).await {
            OPENCLAW_CN_PACKAGE
        } else {
            OPENCLAW_DEFAULT_PACKAGE
        };
        let path_env = shell_path_assignment(&npm_path);
        let prefix_env = npm_prefix
            .as_deref()
            .map(shell_npm_prefix_assignment)
            .unwrap_or_default();
        let npm_cmd = shell_command_escape(&npm_path);
        let cleanup_command = format!(
            "{path_env}{prefix_env}{npm_cmd} uninstall -g openclaw @qingchencloud/openclaw-zh || true"
        );
        let install_command = if should_use_china_package(app).await {
            format!(
                "{path_env}{prefix_env}{npm_cmd} install -g {package} --registry={NPM_MIRROR_CN}"
            )
        } else {
            format!("{path_env}{prefix_env}{npm_cmd} install -g {package}")
        };
        Ok((
            package.to_string(),
            npm_path,
            npm_prefix,
            cleanup_command,
            install_command,
        ))
    }

    async fn resolve_uninstall_command(&self) -> Result<(String, Option<String>, String), String> {
        let npm_path = find_command_in_shell("npm")
            .await?
            .ok_or_else(|| "未检测到 npm，可先安装或修复 Node.js 环境。".to_string())?;
        let npm_prefix = detect_npm_global_prefix(&npm_path).await;
        let path_env = shell_path_assignment(&npm_path);
        let prefix_env = npm_prefix
            .as_deref()
            .map(shell_npm_prefix_assignment)
            .unwrap_or_default();
        let command = format!(
            "{}{}{} uninstall -g openclaw @qingchencloud/openclaw-zh",
            path_env,
            prefix_env,
            shell_command_escape(&npm_path)
        );
        Ok((npm_path, npm_prefix, command))
    }

    async fn build_install_command_preview(
        &self,
        app: &AppHandle,
    ) -> Result<CommandPreview, String> {
        let (package, npm_path, npm_prefix, cleanup_command, install_command) =
            self.resolve_install_commands(app).await?;
        let prefix_note = npm_prefix
            .map(|prefix| format!("npm: {npm_path}\nprefix: {prefix}\n"))
            .unwrap_or_else(|| format!("npm: {npm_path}\n"));
        Ok(CommandPreview {
            title: format!("安装 {package}"),
            command: format!("{prefix_note}{cleanup_command}\n{install_command}"),
        })
    }

    async fn build_uninstall_command_preview(&self) -> Result<CommandPreview, String> {
        let (npm_path, npm_prefix, command) = self.resolve_uninstall_command().await?;
        let prefix_note = npm_prefix
            .map(|prefix| format!("npm: {npm_path}\nprefix: {prefix}\n"))
            .unwrap_or_else(|| format!("npm: {npm_path}\n"));
        Ok(CommandPreview {
            title: "卸载 OpenClaw".to_string(),
            command: format!("{prefix_note}{command}"),
        })
    }

    async fn build_start_command_preview(
        &mut self,
        port: Option<u16>,
    ) -> Result<CommandPreview, String> {
        if let Some(next_port) = port {
            self.gateway_port = next_port.max(1);
        }
        self.restore_auth_token_from_config();
        let binary = find_command_in_shell("openclaw")
            .await?
            .ok_or_else(|| "未检测到 OpenClaw 可执行文件，请先安装。".to_string())?;
        let config_path = openclaw_proxycast_config_path();
        Ok(CommandPreview {
            title: "启动 Gateway".to_string(),
            command: format!(
                "{}OPENCLAW_CONFIG_PATH={} {} gateway --port {}",
                if cfg!(target_os = "windows") {
                    "set "
                } else {
                    ""
                },
                shell_escape(config_path.to_string_lossy().as_ref()),
                shell_escape(&binary),
                self.gateway_port
            ),
        })
    }

    async fn build_stop_command_preview(
        &mut self,
        port: Option<u16>,
    ) -> Result<CommandPreview, String> {
        if let Some(next_port) = port {
            self.gateway_port = next_port.max(1);
        }
        self.restore_auth_token_from_config();
        let binary = find_command_in_shell("openclaw")
            .await?
            .ok_or_else(|| "未检测到 OpenClaw 可执行文件，请先安装。".to_string())?;
        let config_path = openclaw_proxycast_config_path();
        Ok(CommandPreview {
            title: "停止 Gateway".to_string(),
            command: format!(
                "OPENCLAW_CONFIG_PATH={} {} gateway stop --url {} --token {}",
                shell_escape(config_path.to_string_lossy().as_ref()),
                shell_escape(&binary),
                self.gateway_ws_url(),
                shell_escape(&self.gateway_auth_token)
            ),
        })
    }

    async fn build_restart_command_preview(
        &mut self,
        port: Option<u16>,
    ) -> Result<CommandPreview, String> {
        let stop = self.build_stop_command_preview(port).await?;
        let start = self.build_start_command_preview(port).await?;
        Ok(CommandPreview {
            title: "重启 Gateway".to_string(),
            command: format!("{}\n{}", stop.command, start.command),
        })
    }
}

pub fn openclaw_install_event_name() -> &'static str {
    OPENCLAW_INSTALL_EVENT
}

fn openclaw_config_dir() -> PathBuf {
    home_dir()
        .or_else(data_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".openclaw")
}

fn openclaw_original_config_path() -> PathBuf {
    openclaw_config_dir().join("openclaw.json")
}

fn openclaw_proxycast_config_path() -> PathBuf {
    openclaw_config_dir().join("openclaw.proxycast.json")
}

fn openclaw_installer_download_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let _ = app;
    let app_data_dir = proxycast_core::app_paths::preferred_data_dir()
        .map_err(|e| format!("无法获取应用数据目录: {e}"))?;
    let dir = app_data_dir.join("downloads").join("openclaw-installers");
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建 OpenClaw 下载目录失败: {e}"))?;
    Ok(dir)
}

fn collect_temp_artifact_paths(app: Option<&AppHandle>) -> Vec<PathBuf> {
    let mut targets = Vec::new();

    #[cfg(not(target_os = "windows"))]
    {
        targets.push(PathBuf::from(OPENCLAW_TEMP_CARGO_CHECK_DIR));
    }

    if let Some(app) = app {
        if let Ok(dir) = openclaw_installer_download_dir(app) {
            targets.push(dir);
        }
    }

    targets
}

fn build_environment_status(
    node: DependencyStatus,
    git: DependencyStatus,
    mut openclaw: DependencyStatus,
) -> EnvironmentStatus {
    let node_ready = node.status == "ok";
    let git_ready = git.status == "ok";
    openclaw.auto_install_supported = node_ready && git_ready;

    let (recommended_action, summary) = if !node_ready {
        (
            "install_node".to_string(),
            "当前缺少可用的 Node.js 22+ 运行时，建议先一键安装或修复 Node.js。".to_string(),
        )
    } else if !git_ready {
        (
            "install_git".to_string(),
            "当前缺少可用的 Git，建议先一键安装或修复 Git。".to_string(),
        )
    } else if openclaw.status != "ok" {
        (
            "install_openclaw".to_string(),
            "运行环境已就绪，可以继续一键安装 OpenClaw。".to_string(),
        )
    } else {
        (
            "ready".to_string(),
            "Node.js、Git 和 OpenClaw 均已就绪，可以继续配置与启动。".to_string(),
        )
    };

    EnvironmentStatus {
        node,
        git,
        openclaw,
        recommended_action,
        summary,
        temp_artifacts: collect_temp_artifact_paths(None)
            .into_iter()
            .filter(|path| path.exists())
            .map(|path| path.display().to_string())
            .collect(),
    }
}

async fn inspect_node_dependency_status() -> Result<DependencyStatus, String> {
    let Some(path) = find_command_in_shell("node").await? else {
        return Ok(DependencyStatus {
            status: "missing".to_string(),
            version: None,
            path: None,
            message: format!(
                "未检测到 Node.js，需要安装 {}+。",
                format_semver(NODE_MIN_VERSION)
            ),
            auto_install_supported: cfg!(target_os = "windows") || cfg!(target_os = "macos"),
        });
    };

    let version_text = read_command_version_text(&path, &["--version"]).await?;
    let Some(version) = parse_semver_from_text(&version_text) else {
        return Ok(DependencyStatus {
            status: "version_low".to_string(),
            version: Some(version_text.clone()),
            path: Some(path),
            message: format!(
                "检测到 Node.js，但无法识别版本：{version_text}。请安装 {}+。",
                format_semver(NODE_MIN_VERSION)
            ),
            auto_install_supported: cfg!(target_os = "windows") || cfg!(target_os = "macos"),
        });
    };

    let normalized = format_semver(version);
    if version >= NODE_MIN_VERSION {
        Ok(DependencyStatus {
            status: "ok".to_string(),
            version: Some(normalized.clone()),
            path: Some(path),
            message: format!("Node.js 已就绪：{normalized}"),
            auto_install_supported: cfg!(target_os = "windows") || cfg!(target_os = "macos"),
        })
    } else {
        Ok(DependencyStatus {
            status: "version_low".to_string(),
            version: Some(normalized.clone()),
            path: Some(path),
            message: format!(
                "Node.js 版本过低：{normalized}，需要 {}+。",
                format_semver(NODE_MIN_VERSION)
            ),
            auto_install_supported: cfg!(target_os = "windows") || cfg!(target_os = "macos"),
        })
    }
}

async fn inspect_git_dependency_status() -> Result<DependencyStatus, String> {
    let Some(path) = find_command_in_shell("git").await? else {
        return Ok(DependencyStatus {
            status: "missing".to_string(),
            version: None,
            path: None,
            message: "未检测到 Git。".to_string(),
            auto_install_supported: git_auto_install_supported().await?,
        });
    };

    let version_text = read_command_version_text(&path, &["--version"]).await?;
    let version = parse_semver_from_text(&version_text).map(format_semver);
    let detail = version.clone().unwrap_or(version_text);

    Ok(DependencyStatus {
        status: "ok".to_string(),
        version,
        path: Some(path),
        message: format!("Git 已就绪：{detail}"),
        auto_install_supported: git_auto_install_supported().await?,
    })
}

async fn inspect_openclaw_dependency_status() -> Result<DependencyStatus, String> {
    let Some(path) = find_command_in_shell("openclaw").await? else {
        return Ok(DependencyStatus {
            status: "missing".to_string(),
            version: None,
            path: None,
            message: "未检测到 OpenClaw，可在环境就绪后一键安装。".to_string(),
            auto_install_supported: false,
        });
    };

    let version_text = read_command_version_text(&path, &["--version"]).await?;
    Ok(DependencyStatus {
        status: "ok".to_string(),
        version: if version_text.is_empty() {
            None
        } else {
            Some(version_text.clone())
        },
        path: Some(path),
        message: if version_text.is_empty() {
            "已检测到 OpenClaw。".to_string()
        } else {
            format!("已检测到 OpenClaw：{version_text}")
        },
        auto_install_supported: false,
    })
}

async fn git_auto_install_supported() -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        Ok(find_command_in_shell("winget").await?.is_some())
    }

    #[cfg(target_os = "macos")]
    {
        Ok(true)
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Ok(false)
    }
}

async fn read_command_version_text(command_path: &str, args: &[&str]) -> Result<String, String> {
    let mut command = Command::new(command_path);
    apply_binary_runtime_path(&mut command, command_path);
    for arg in args {
        command.arg(arg);
    }
    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("执行命令失败({command_path}): {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !stdout.is_empty() {
        Ok(stdout)
    } else {
        Ok(stderr)
    }
}

async fn resolve_node_installer_asset() -> Result<InstallerAsset, String> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://nodejs.org/dist/index.json")
        .header("User-Agent", OPENCLAW_INSTALLER_USER_AGENT)
        .send()
        .await
        .map_err(|e| format!("请求 Node.js 版本列表失败: {e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "获取 Node.js 版本列表失败: HTTP {}",
            response.status()
        ));
    }

    let releases: Vec<Value> = response
        .json()
        .await
        .map_err(|e| format!("解析 Node.js 版本列表失败: {e}"))?;

    let select_version = |only_lts: bool| -> Option<String> {
        releases.iter().find_map(|release| {
            let version = release.get("version")?.as_str()?;
            let parsed = parse_semver(version)?;
            let is_lts = release
                .get("lts")
                .map(|value| match value {
                    Value::Bool(flag) => *flag,
                    Value::String(text) => !text.trim().is_empty() && text != "false",
                    _ => false,
                })
                .unwrap_or(false);
            if parsed >= NODE_MIN_VERSION && (!only_lts || is_lts) {
                Some(version.to_string())
            } else {
                None
            }
        })
    };

    let version = select_version(true)
        .or_else(|| select_version(false))
        .ok_or_else(|| "未找到满足要求的 Node.js 官方安装包版本。".to_string())?;

    #[cfg(target_os = "windows")]
    let filename = {
        #[cfg(target_arch = "aarch64")]
        {
            format!("node-{version}-arm64.msi")
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            format!("node-{version}-x64.msi")
        }
    };

    #[cfg(target_os = "macos")]
    let filename = format!("node-{version}.pkg");

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let filename = String::new();

    if filename.is_empty() {
        return Err("当前平台暂不支持自动下载官方 Node.js 安装器。".to_string());
    }

    Ok(InstallerAsset {
        download_url: format!("https://nodejs.org/dist/{version}/{filename}"),
        filename,
    })
}

async fn download_installer_asset(
    app: &AppHandle,
    asset: &InstallerAsset,
) -> Result<PathBuf, String> {
    let download_dir = openclaw_installer_download_dir(app)?;
    let installer_path = download_dir.join(&asset.filename);
    if installer_path.exists() {
        let _ = std::fs::remove_file(&installer_path);
    }

    emit_install_progress(
        app,
        &format!("开始下载安装器：{}", asset.download_url),
        "info",
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&asset.download_url)
        .header("User-Agent", OPENCLAW_INSTALLER_USER_AGENT)
        .send()
        .await
        .map_err(|e| format!("下载官方安装器失败: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("下载安装器失败: HTTP {}", response.status()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("读取安装器文件失败: {e}"))?;
    std::fs::write(&installer_path, bytes)
        .map_err(|e| format!("保存安装器失败({}): {e}", installer_path.display()))?;

    emit_install_progress(
        app,
        &format!("安装器已保存到：{}", installer_path.display()),
        "info",
    );

    Ok(installer_path)
}

fn launch_installer(file_path: &Path) -> Result<(), String> {
    let extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    match extension.as_str() {
        "exe" => {
            #[cfg(target_os = "windows")]
            {
                std::process::Command::new(file_path)
                    .spawn()
                    .map_err(|e| format!("启动安装程序失败: {e}"))?;
            }

            #[cfg(not(target_os = "windows"))]
            {
                return Err("EXE 安装器只能在 Windows 上运行。".to_string());
            }
        }
        "msi" => {
            #[cfg(target_os = "windows")]
            {
                std::process::Command::new("msiexec")
                    .arg("/i")
                    .arg(file_path)
                    .spawn()
                    .map_err(|e| format!("启动 MSI 安装程序失败: {e}"))?;
            }

            #[cfg(not(target_os = "windows"))]
            {
                return Err("MSI 安装器只能在 Windows 上运行。".to_string());
            }
        }
        "pkg" | "dmg" => {
            #[cfg(target_os = "macos")]
            {
                std::process::Command::new("open")
                    .arg(file_path)
                    .spawn()
                    .map_err(|e| format!("打开 macOS 安装器失败: {e}"))?;
            }

            #[cfg(not(target_os = "macos"))]
            {
                return Err("该安装器只能在 macOS 上运行。".to_string());
            }
        }
        _ => return Err(format!("不支持的安装器文件类型: {extension}")),
    }

    Ok(())
}

#[cfg(target_os = "macos")]
async fn trigger_macos_command_line_tools_install() -> Result<String, String> {
    let output = Command::new("/usr/bin/xcode-select")
        .arg("--install")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("拉起 macOS 开发者工具安装器失败: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let combined = if !stderr.is_empty() { stderr } else { stdout };
    let lower = combined.to_ascii_lowercase();

    if output.status.success()
        || lower.contains("install requested")
        || lower.contains("already been requested")
    {
        return Ok("已拉起 macOS 开发者工具安装器。".to_string());
    }

    if lower.contains("already installed") {
        return Err(
            "系统提示 Command Line Tools 已安装，但当前仍未检测到 Git，请先执行系统更新或安装 Homebrew 后重试。"
                .to_string(),
        );
    }

    Err(format!("拉起 macOS 开发者工具安装器失败: {combined}"))
}

#[cfg(not(target_os = "macos"))]
async fn trigger_macos_command_line_tools_install() -> Result<String, String> {
    Err("当前平台不支持拉起 macOS 开发者工具安装器。".to_string())
}

fn read_base_openclaw_config() -> Result<Value, String> {
    let proxycast_path = openclaw_proxycast_config_path();
    if proxycast_path.exists() {
        return read_json_file(&proxycast_path);
    }

    let original_path = openclaw_original_config_path();
    if original_path.exists() {
        return read_json_file(&original_path);
    }

    Ok(json!({}))
}

fn read_json_file(path: &Path) -> Result<Value, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("读取配置文件失败({}): {e}", path.display()))?;
    serde_json::from_str(&content).map_err(|e| format!("解析配置文件失败({}): {e}", path.display()))
}

fn ensure_path_object<'a>(root: &'a mut Value, path: &[&str]) -> &'a mut Map<String, Value> {
    let mut current = root;
    for segment in path {
        let object = ensure_value_object(current);
        current = object
            .entry((*segment).to_string())
            .or_insert_with(|| Value::Object(Map::new()));
    }
    ensure_value_object(current)
}

fn set_json_path(root: &mut Value, path: &[&str], value: Value) {
    if path.is_empty() {
        *root = value;
        return;
    }

    let parent = ensure_path_object(root, &path[..path.len() - 1]);
    parent.insert(path[path.len() - 1].to_string(), value);
}

fn ensure_value_object(value: &mut Value) -> &mut Map<String, Value> {
    if !value.is_object() {
        *value = Value::Object(Map::new());
    }
    value.as_object_mut().expect("value should be object")
}

fn build_channel_info(channel_id: &str, entry: &Value, label: Option<&Value>) -> ChannelInfo {
    ChannelInfo {
        id: channel_id.to_string(),
        name: entry
            .get("name")
            .and_then(Value::as_str)
            .or_else(|| label.and_then(Value::as_str))
            .unwrap_or("未命名通道")
            .to_string(),
        channel_type: entry
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        status: entry
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
    }
}

fn extract_gateway_auth_token(config: &Value) -> Option<String> {
    config
        .get("gateway")
        .and_then(|gateway| {
            gateway
                .get("auth")
                .and_then(|auth| auth.get("token"))
                .or_else(|| gateway.get("remote").and_then(|remote| remote.get("token")))
        })
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(ToString::to_string)
}

fn determine_api_type(provider_type: ApiProviderType) -> Result<&'static str, String> {
    match provider_type {
        ApiProviderType::Anthropic | ApiProviderType::AnthropicCompatible => {
            Ok("anthropic-messages")
        }
        ApiProviderType::OpenaiResponse => Ok("openai-responses"),
        ApiProviderType::Openai
        | ApiProviderType::Codex
        | ApiProviderType::Gemini
        | ApiProviderType::Ollama
        | ApiProviderType::Fal
        | ApiProviderType::NewApi
        | ApiProviderType::Gateway => Ok("openai-completions"),
        ApiProviderType::AzureOpenai | ApiProviderType::Vertexai | ApiProviderType::AwsBedrock => {
            Err("当前暂不支持将该 Provider 同步到 OpenClaw。".to_string())
        }
    }
}

fn format_provider_base_url(provider: &ApiKeyProvider) -> Result<String, String> {
    let api_host = trim_trailing_slash(&provider.api_host);

    match provider.provider_type {
        ApiProviderType::Anthropic | ApiProviderType::AnthropicCompatible => Ok(api_host),
        ApiProviderType::Gemini => {
            if api_host.contains("generativelanguage.googleapis.com") {
                if api_host.ends_with("/v1beta/openai") {
                    Ok(api_host)
                } else {
                    Ok(format!("{api_host}/v1beta/openai"))
                }
            } else if has_api_version(&api_host) {
                Ok(api_host)
            } else {
                Ok(format!("{api_host}/v1"))
            }
        }
        ApiProviderType::Gateway => {
            if api_host.ends_with("/v1/ai") {
                Ok(api_host.trim_end_matches("/ai").to_string())
            } else if has_api_version(&api_host) {
                Ok(api_host)
            } else {
                Ok(format!("{api_host}/v1"))
            }
        }
        ApiProviderType::Openai
        | ApiProviderType::OpenaiResponse
        | ApiProviderType::Codex
        | ApiProviderType::Ollama
        | ApiProviderType::Fal
        | ApiProviderType::NewApi => {
            if has_api_version(&api_host) {
                Ok(api_host)
            } else {
                Ok(format!("{api_host}/v1"))
            }
        }
        ApiProviderType::AzureOpenai | ApiProviderType::Vertexai | ApiProviderType::AwsBedrock => {
            Err("当前暂不支持将该 Provider 同步到 OpenClaw。".to_string())
        }
    }
}

fn trim_trailing_slash(value: &str) -> String {
    value.trim().trim_end_matches('/').to_string()
}

fn has_api_version(url: &str) -> bool {
    static VERSION_RE: OnceLock<Regex> = OnceLock::new();
    VERSION_RE
        .get_or_init(|| Regex::new(r"/v\d+(?:[./]|$)").expect("regex should compile"))
        .is_match(url)
}

fn generate_auth_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(48)
        .map(char::from)
        .collect()
}

async fn should_use_china_package(app: &AppHandle) -> bool {
    if let Some(app_state) = app.try_state::<AppState>() {
        let language = {
            let state = app_state.read().await;
            state.config.language.clone()
        };

        if language.starts_with("zh") {
            return true;
        }
    }

    let locale = std::env::var("LC_ALL")
        .ok()
        .or_else(|| std::env::var("LANG").ok())
        .unwrap_or_default()
        .to_lowercase();
    let timezone = std::env::var("TZ").unwrap_or_default().to_lowercase();
    locale.contains("zh_cn") || locale.contains("zh-hans") || timezone.contains("shanghai")
}

async fn detect_npm_global_prefix(npm_path: &str) -> Option<String> {
    let mut command = Command::new(npm_path);
    apply_binary_runtime_path(&mut command, npm_path);
    let output = command
        .arg("config")
        .arg("get")
        .arg("prefix")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let prefix = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if prefix.is_empty() || prefix.eq_ignore_ascii_case("undefined") {
        None
    } else {
        Some(prefix)
    }
}

fn shell_escape(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn shell_command_escape(value: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        shell_escape(value)
    }
}

fn shell_npm_prefix_assignment(value: &str) -> String {
    if cfg!(target_os = "windows") {
        format!(
            "set \"NPM_CONFIG_PREFIX={}\" && ",
            value.replace('"', "\"\"")
        )
    } else {
        format!("NPM_CONFIG_PREFIX={} ", shell_escape(value))
    }
}

fn shell_path_assignment(binary_path: &str) -> String {
    let Some(bin_dir) = Path::new(binary_path).parent() else {
        return String::new();
    };
    let bin_dir = bin_dir.to_string_lossy();
    if cfg!(target_os = "windows") {
        format!("set \"PATH={};%PATH%\" && ", bin_dir.replace('"', "\"\""))
    } else {
        format!("PATH={}:$PATH ", shell_escape(bin_dir.as_ref()))
    }
}

fn prepend_path(dir: &Path) -> Option<OsString> {
    let mut paths = vec![dir.to_path_buf()];
    if let Some(current) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&current));
    }
    std::env::join_paths(paths).ok()
}

fn apply_binary_runtime_path(command: &mut Command, binary_path: &str) {
    apply_windows_no_window(command);

    let Some(bin_dir) = Path::new(binary_path).parent() else {
        return;
    };
    if let Some(path) = prepend_path(bin_dir) {
        command.env("PATH", path);
    }
}

fn apply_windows_no_window(_command: &mut Command) {
    #[cfg(target_os = "windows")]
    {
        _command.creation_flags(CREATE_NO_WINDOW);
    }
}

async fn find_command_in_shell(command_name: &str) -> Result<Option<String>, String> {
    if cfg!(target_os = "windows") {
        let mut command = Command::new("cmd");
        apply_windows_no_window(&mut command);
        let output = command
            .arg("/C")
            .arg("where")
            .arg(command_name)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .await
            .map_err(|e| format!("查找命令失败: {e}"))?;

        if output.status.success() {
            let result = String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(str::trim)
                .find(|line| !line.is_empty())
                .map(str::to_string);
            if result.is_some() {
                return Ok(result);
            }
        }

        return Ok(find_command_in_known_locations(command_name)
            .await?
            .map(|path| path.to_string_lossy().to_string()));
    }

    Ok(find_command_in_known_locations(command_name)
        .await?
        .map(|path| path.to_string_lossy().to_string()))
}

async fn find_command_in_known_locations(command_name: &str) -> Result<Option<PathBuf>, String> {
    let candidates = find_all_commands_in_known_locations(command_name);
    if candidates.is_empty() {
        return Ok(None);
    }

    if command_name == "node" {
        return select_best_node_candidate(candidates).await;
    }

    if matches!(command_name, "npm" | "npx" | "openclaw") {
        return select_node_runtime_candidate(candidates).await;
    }

    Ok(candidates.into_iter().next())
}

fn find_all_commands_in_known_locations(command_name: &str) -> Vec<PathBuf> {
    let mut search_dirs = Vec::new();
    let mut seen = HashSet::new();

    let mut push_dir = |dir: PathBuf| {
        if dir.as_os_str().is_empty() || !dir.exists() {
            return;
        }
        if seen.insert(dir.clone()) {
            search_dirs.push(dir);
        }
    };

    if let Some(path_var) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_var) {
            push_dir(dir);
        }
    }

    if let Some(home) = home_dir() {
        push_dir(home.join(".npm-global/bin"));
        push_dir(home.join(".local/bin"));
        push_dir(home.join(".bun/bin"));
        push_dir(home.join(".volta/bin"));
        push_dir(home.join(".asdf/shims"));
        push_dir(home.join(".local/share/mise/shims"));
        push_dir(home.join("Library/PhpWebStudy/env/node/bin"));

        let nvm_versions = home.join(".nvm/versions/node");
        if let Ok(entries) = std::fs::read_dir(nvm_versions) {
            for entry in entries.flatten() {
                push_dir(entry.path().join("bin"));
            }
        }

        let fnm_versions = home.join(".fnm/node-versions");
        if let Ok(entries) = std::fs::read_dir(fnm_versions) {
            for entry in entries.flatten() {
                push_dir(entry.path().join("installation/bin"));
            }
        }
    }

    if cfg!(target_os = "macos") {
        push_dir(PathBuf::from("/opt/homebrew/bin"));
        push_dir(PathBuf::from("/usr/local/bin"));
        push_dir(PathBuf::from("/usr/bin"));
        push_dir(PathBuf::from("/bin"));
    }

    find_all_commands_in_paths(command_name, &search_dirs)
}

fn find_all_commands_in_paths(command_name: &str, search_dirs: &[PathBuf]) -> Vec<PathBuf> {
    #[cfg(target_os = "windows")]
    let candidates = [
        format!("{command_name}.exe"),
        format!("{command_name}.cmd"),
        format!("{command_name}.bat"),
        command_name.to_string(),
    ];

    #[cfg(not(target_os = "windows"))]
    let candidates = [command_name.to_string()];

    let mut matches = Vec::new();
    let mut seen = HashSet::new();
    for dir in search_dirs {
        for candidate in &candidates {
            let path = dir.join(candidate);
            if path.is_file() && seen.insert(path.clone()) {
                matches.push(path);
            }
        }
    }

    matches
}

async fn select_best_node_candidate(candidates: Vec<PathBuf>) -> Result<Option<PathBuf>, String> {
    let mut versioned = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let version = read_binary_semver(&candidate).await;
        versioned.push((candidate, version));
    }
    Ok(select_best_semver_candidate(versioned))
}

async fn select_node_runtime_candidate(
    candidates: Vec<PathBuf>,
) -> Result<Option<PathBuf>, String> {
    let preferred_node =
        select_best_node_candidate(find_all_commands_in_known_locations("node")).await?;
    if let Some(preferred_bin_dir) = preferred_node.as_deref().and_then(Path::parent) {
        if let Some(candidate) = candidates
            .iter()
            .find(|candidate| candidate.parent() == Some(preferred_bin_dir))
        {
            return Ok(Some(candidate.clone()));
        }
    }

    let mut versioned = Vec::with_capacity(candidates.len());
    for candidate in &candidates {
        let version = match sibling_node_path(candidate) {
            Some(node_path) => read_binary_semver(&node_path).await,
            None => None,
        };
        versioned.push((candidate.clone(), version));
    }

    Ok(select_best_semver_candidate(versioned).or_else(|| candidates.into_iter().next()))
}

fn sibling_node_path(command_path: &Path) -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    let node_name = "node.exe";

    #[cfg(not(target_os = "windows"))]
    let node_name = "node";

    let node_path = command_path.parent()?.join(node_name);
    node_path.is_file().then_some(node_path)
}

async fn read_binary_semver(path: &Path) -> Option<(u64, u64, u64)> {
    let mut command = Command::new(path);
    apply_windows_no_window(&mut command);
    let output = command
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    parse_semver(stdout.trim()).or_else(|| parse_semver(stderr.trim()))
}

fn select_best_semver_candidate(
    candidates: Vec<(PathBuf, Option<(u64, u64, u64)>)>,
) -> Option<PathBuf> {
    candidates
        .iter()
        .filter_map(|(path, version)| {
            version
                .filter(|version| *version >= NODE_MIN_VERSION)
                .map(|version| (path.clone(), version))
        })
        .max_by_key(|(_, version)| *version)
        .map(|(path, _)| path)
        .or_else(|| {
            candidates
                .iter()
                .filter_map(|(path, version)| version.map(|version| (path.clone(), version)))
                .max_by_key(|(_, version)| *version)
                .map(|(path, _)| path)
        })
        .or_else(|| candidates.into_iter().next().map(|(path, _)| path))
}

async fn run_shell_command_with_progress(
    app: &AppHandle,
    command_line: &str,
) -> Result<ActionResult, String> {
    let mut child = spawn_shell_command(command_line)?;

    let stdout_task = child.stdout.take().map(|stdout| {
        let app = app.clone();
        tokio::spawn(async move {
            stream_reader_to_progress(app, stdout, "info").await;
        })
    });

    let stderr_task = child.stderr.take().map(|stderr| {
        let app = app.clone();
        tokio::spawn(async move {
            stream_reader_to_progress(app, stderr, "error").await;
        })
    });

    let status = child
        .wait()
        .await
        .map_err(|e| format!("执行命令失败: {e}"))?;

    if let Some(task) = stdout_task {
        let _ = task.await;
    }
    if let Some(task) = stderr_task {
        let _ = task.await;
    }

    if status.success() {
        emit_install_progress(app, "命令执行成功。", "info");
        Ok(ActionResult {
            success: true,
            message: "操作成功完成。".to_string(),
        })
    } else {
        emit_install_progress(
            app,
            &format!("命令执行失败，退出码: {:?}", status.code()),
            "error",
        );
        Ok(ActionResult {
            success: false,
            message: format!("命令执行失败，退出码: {:?}", status.code()),
        })
    }
}

fn spawn_shell_command(command_line: &str) -> Result<Child, String> {
    let mut command = if cfg!(target_os = "windows") {
        let mut cmd = Command::new("cmd");
        cmd.arg("/C").arg(command_line);
        cmd
    } else if cfg!(target_os = "macos") {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
        let mut cmd = Command::new("script");
        cmd.arg("-q")
            .arg("/dev/null")
            .arg(shell)
            .arg("-lc")
            .arg(command_line);
        cmd
    } else {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let mut cmd = Command::new(shell);
        cmd.arg("-lc").arg(command_line);
        cmd
    };

    apply_windows_no_window(&mut command);

    command
        .env("NO_COLOR", "1")
        .env("CLICOLOR", "0")
        .env("FORCE_COLOR", "0")
        .env("npm_config_color", "false")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command.spawn().map_err(|e| format!("启动命令失败: {e}"))
}

async fn stream_reader_to_progress<R>(app: AppHandle, mut reader: R, default_level: &'static str)
where
    R: AsyncRead + Unpin,
{
    let mut buffer = [0_u8; 2048];
    let mut pending = String::new();

    loop {
        match reader.read(&mut buffer).await {
            Ok(0) => break,
            Ok(size) => {
                pending.push_str(&String::from_utf8_lossy(&buffer[..size]));
                flush_progress_chunks(&app, &mut pending, default_level);
            }
            Err(error) => {
                emit_install_progress(&app, &format!("读取命令输出失败: {error}"), "warn");
                break;
            }
        }
    }

    let tail = pending.trim();
    if !tail.is_empty() {
        emit_install_progress(&app, tail, classify_progress_level(tail, default_level));
    }
}

fn flush_progress_chunks(app: &AppHandle, pending: &mut String, default_level: &'static str) {
    loop {
        let next_break = pending.find(['\n', '\r']);
        let Some(index) = next_break else {
            break;
        };

        let mut line = pending[..index].trim().to_string();
        let mut consume_len = index + 1;
        while pending
            .get(consume_len..consume_len + 1)
            .is_some_and(|ch| ch == "\n" || ch == "\r")
        {
            consume_len += 1;
        }

        pending.drain(..consume_len);

        if line.is_empty() {
            continue;
        }

        line = sanitize_progress_line(&line);
        if line.is_empty() {
            continue;
        }

        emit_install_progress(app, &line, classify_progress_level(&line, default_level));
    }

    if pending.len() > 4096 {
        let line = sanitize_progress_line(pending.trim());
        if !line.is_empty() {
            emit_install_progress(app, &line, classify_progress_level(&line, default_level));
        }
        pending.clear();
    }
}

fn sanitize_progress_line(value: &str) -> String {
    value
        .replace('\u{1b}', "")
        .replace("[?25h", "")
        .replace("[?25l", "")
        .trim()
        .to_string()
}

fn classify_progress_level(message: &str, default_level: &'static str) -> &'static str {
    let lower = message.to_ascii_lowercase();
    if lower.contains("error") || lower.contains("fatal") {
        "error"
    } else if lower.contains("warn") || lower.contains("warning") {
        "warn"
    } else {
        default_level
    }
}

fn emit_install_progress(app: &AppHandle, message: &str, level: &str) {
    if let Some(service_state) = app.try_state::<OpenClawServiceState>() {
        if let Ok(mut service) = service_state.0.try_lock() {
            service.push_progress_log(message.to_string(), level.to_string());
        }
    }

    let payload = InstallProgressEvent {
        message: message.to_string(),
        level: level.to_string(),
    };
    let _ = app.emit(OPENCLAW_INSTALL_EVENT, payload);
}

fn parse_semver(value: &str) -> Option<(u64, u64, u64)> {
    let sanitized = value.trim().trim_start_matches('v');
    let core = sanitized.split(['-', '+']).next()?;
    let mut parts = core.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next().unwrap_or("0").parse().ok()?;
    let patch = parts.next().unwrap_or("0").parse().ok()?;
    Some((major, minor, patch))
}

fn parse_semver_from_text(value: &str) -> Option<(u64, u64, u64)> {
    parse_semver(value).or_else(|| {
        value
            .split(|ch: char| ch.is_whitespace() || ch == ',' || ch == '(' || ch == ')')
            .find_map(parse_semver)
    })
}

fn format_semver(version: (u64, u64, u64)) -> String {
    format!("{}.{}.{}", version.0, version.1, version.2)
}

#[cfg(test)]
mod tests {
    use super::{
        build_environment_status, determine_api_type, extract_gateway_auth_token,
        format_provider_base_url, has_api_version, parse_semver_from_text, trim_trailing_slash,
        DependencyStatus,
    };
    use crate::database::dao::api_key_provider::{ApiKeyProvider, ApiProviderType, ProviderGroup};
    use chrono::Utc;
    use serde_json::json;

    fn build_provider(provider_type: ApiProviderType, api_host: &str) -> ApiKeyProvider {
        ApiKeyProvider {
            id: "provider-1".to_string(),
            name: "Provider 1".to_string(),
            provider_type,
            api_host: api_host.to_string(),
            is_system: false,
            group: ProviderGroup::Custom,
            enabled: true,
            sort_order: 0,
            api_version: None,
            project: None,
            location: None,
            region: None,
            custom_models: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn trims_trailing_slash() {
        assert_eq!(
            trim_trailing_slash("https://api.openai.com/"),
            "https://api.openai.com"
        );
    }

    #[test]
    fn detects_version_segment() {
        assert!(has_api_version("https://api.openai.com/v1"));
        assert!(!has_api_version("https://api.openai.com"));
    }

    #[test]
    fn maps_api_type_correctly() {
        assert_eq!(
            determine_api_type(ApiProviderType::Openai).unwrap(),
            "openai-completions"
        );
        assert_eq!(
            determine_api_type(ApiProviderType::OpenaiResponse).unwrap(),
            "openai-responses"
        );
        assert_eq!(
            determine_api_type(ApiProviderType::Anthropic).unwrap(),
            "anthropic-messages"
        );
    }

    #[test]
    fn formats_openai_url() {
        let provider = build_provider(ApiProviderType::Openai, "https://api.openai.com");
        assert_eq!(
            format_provider_base_url(&provider).unwrap(),
            "https://api.openai.com/v1"
        );
    }

    #[test]
    fn keeps_existing_version_url() {
        let provider = build_provider(ApiProviderType::Openai, "https://example.com/v2");
        assert_eq!(
            format_provider_base_url(&provider).unwrap(),
            "https://example.com/v2"
        );
    }

    #[test]
    fn formats_gemini_url() {
        let provider = build_provider(
            ApiProviderType::Gemini,
            "https://generativelanguage.googleapis.com",
        );
        assert_eq!(
            format_provider_base_url(&provider).unwrap(),
            "https://generativelanguage.googleapis.com/v1beta/openai"
        );
    }

    #[test]
    fn formats_gateway_url() {
        let provider = build_provider(
            ApiProviderType::Gateway,
            "https://gateway.example.com/v1/ai",
        );
        assert_eq!(
            format_provider_base_url(&provider).unwrap(),
            "https://gateway.example.com/v1"
        );
    }

    #[test]
    fn rejects_unsupported_provider_types() {
        let provider = build_provider(ApiProviderType::AzureOpenai, "https://example.com");
        assert!(format_provider_base_url(&provider).is_err());
    }

    #[test]
    fn extracts_gateway_auth_token_from_config() {
        let config = json!({
            "gateway": {
                "auth": {
                    "token": "proxycast-token"
                }
            }
        });

        assert_eq!(
            extract_gateway_auth_token(&config).as_deref(),
            Some("proxycast-token")
        );
    }

    #[test]
    fn ignores_empty_gateway_auth_token() {
        let config = json!({
            "gateway": {
                "auth": {
                    "token": "   "
                }
            }
        });

        assert_eq!(extract_gateway_auth_token(&config), None);
    }

    #[test]
    fn parses_semver_from_git_version_text() {
        assert_eq!(
            parse_semver_from_text("git version 2.39.5 (Apple Git-154)"),
            Some((2, 39, 5))
        );
    }

    #[test]
    fn environment_status_prioritizes_missing_node() {
        let env = build_environment_status(
            DependencyStatus {
                status: "missing".to_string(),
                version: None,
                path: None,
                message: "missing node".to_string(),
                auto_install_supported: true,
            },
            DependencyStatus {
                status: "ok".to_string(),
                version: Some("2.43.0".to_string()),
                path: Some("/usr/bin/git".to_string()),
                message: "git ok".to_string(),
                auto_install_supported: true,
            },
            DependencyStatus {
                status: "missing".to_string(),
                version: None,
                path: None,
                message: "openclaw missing".to_string(),
                auto_install_supported: false,
            },
        );

        assert_eq!(env.recommended_action, "install_node");
        assert_eq!(env.openclaw.auto_install_supported, false);
    }
}
