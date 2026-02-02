//! Shell 集成脚本管理
//!
//! 提供各种 Shell 的集成脚本安装和管理功能。
//!
//! ## 功能
//! - 生成 Shell 集成脚本
//! - 安装脚本到用户目录
//! - 构建带集成的 Shell 启动命令
//!
//! ## 支持的 Shell
//! - Bash (--rcfile)
//! - Zsh (ZDOTDIR)
//! - Fish (-C source)
//! - PowerShell (pwsh)
//!
//! ## Requirements
//! - 17.5: 支持自定义 Shell 路径和参数
//! - 17.6: 支持自定义初始化脚本
//! - 17.7: 支持环境变量配置
//! - 17.8: zsh 使用 ZDOTDIR 指向集成目录
//! - 17.9: bash 使用 --rcfile 加载集成脚本
//! - 17.10: fish 使用 -C 参数 source 集成脚本

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::terminal::error::TerminalError;
use crate::terminal::integration::shell_integration::ShellType;

/// Shell 集成脚本目录名
const SHELL_INTEGRATION_DIR: &str = "shell-integration";

/// Bash 集成脚本内容
const BASH_INTEGRATION_SCRIPT: &str = r#"# ProxyCast Shell Integration for Bash
# This script provides shell integration features

# 保存原始 PS1
if [ -z "$_PROXYCAST_ORIG_PS1" ]; then
    _PROXYCAST_ORIG_PS1="$PS1"
fi

# OSC 7 - 报告当前工作目录
__proxycast_osc7() {
    printf '\033]7;file://%s%s\033\\' "${HOSTNAME:-localhost}" "$PWD"
}

# OSC 133 - 命令提示符标记
__proxycast_prompt_start() {
    printf '\033]133;A\033\\'
}

__proxycast_command_start() {
    printf '\033]133;B\033\\'
}

__proxycast_command_executed() {
    printf '\033]133;C\033\\'
}

__proxycast_command_finished() {
    printf '\033]133;D;%s\033\\' "$?"
}

# 设置 PROMPT_COMMAND
__proxycast_precmd() {
    local exit_code=$?
    __proxycast_command_finished
    __proxycast_osc7
    __proxycast_prompt_start
    return $exit_code
}

__proxycast_preexec() {
    __proxycast_command_executed
}

# 安装 preexec 钩子（如果可用）
if [ -n "$BASH_VERSION" ]; then
    # 使用 DEBUG trap 模拟 preexec
    __proxycast_debug_trap() {
        if [ -n "$COMP_LINE" ]; then
            return
        fi
        if [ "$BASH_COMMAND" = "$PROMPT_COMMAND" ]; then
            return
        fi
        __proxycast_preexec
    }
    
    trap '__proxycast_debug_trap' DEBUG
fi

# 设置 PROMPT_COMMAND
if [ -z "$PROMPT_COMMAND" ]; then
    PROMPT_COMMAND="__proxycast_precmd"
else
    PROMPT_COMMAND="__proxycast_precmd;$PROMPT_COMMAND"
fi

# 加载用户的 .bashrc（如果存在且我们是通过 --rcfile 启动的）
if [ -n "$_PROXYCAST_LOAD_BASHRC" ] && [ -f "$HOME/.bashrc" ]; then
    source "$HOME/.bashrc"
fi

# 标记集成已加载
export PROXYCAST_SHELL_INTEGRATION=1
"#;

/// Zsh 集成脚本内容 (.zshrc)
const ZSH_INTEGRATION_SCRIPT: &str = r#"# ProxyCast Shell Integration for Zsh
# This script provides shell integration features

# OSC 7 - 报告当前工作目录
__proxycast_osc7() {
    printf '\033]7;file://%s%s\033\\' "${HOST:-localhost}" "$PWD"
}

# OSC 133 - 命令提示符标记
__proxycast_prompt_start() {
    printf '\033]133;A\033\\'
}

__proxycast_command_start() {
    printf '\033]133;B\033\\'
}

__proxycast_command_executed() {
    printf '\033]133;C\033\\'
}

__proxycast_command_finished() {
    printf '\033]133;D;%s\033\\' "$?"
}

# precmd 钩子 - 命令执行后
__proxycast_precmd() {
    local exit_code=$?
    __proxycast_command_finished
    __proxycast_osc7
    __proxycast_prompt_start
    return $exit_code
}

# preexec 钩子 - 命令执行前
__proxycast_preexec() {
    __proxycast_command_executed
}

# 注册钩子
autoload -Uz add-zsh-hook
add-zsh-hook precmd __proxycast_precmd
add-zsh-hook preexec __proxycast_preexec

# 加载用户的原始配置
if [ -n "$_PROXYCAST_ORIG_ZDOTDIR" ]; then
    if [ -f "$_PROXYCAST_ORIG_ZDOTDIR/.zshrc" ]; then
        source "$_PROXYCAST_ORIG_ZDOTDIR/.zshrc"
    fi
elif [ -f "$HOME/.zshrc" ]; then
    source "$HOME/.zshrc"
fi

# 标记集成已加载
export PROXYCAST_SHELL_INTEGRATION=1
"#;

/// Zsh .zshenv 内容（用于设置 ZDOTDIR）
const ZSH_ZSHENV_SCRIPT: &str = r#"# ProxyCast Zsh Environment
# 保存原始 ZDOTDIR
if [ -z "$_PROXYCAST_ORIG_ZDOTDIR" ]; then
    export _PROXYCAST_ORIG_ZDOTDIR="${ZDOTDIR:-$HOME}"
fi

# 加载原始 .zshenv
if [ -f "$_PROXYCAST_ORIG_ZDOTDIR/.zshenv" ]; then
    source "$_PROXYCAST_ORIG_ZDOTDIR/.zshenv"
fi
"#;

/// Fish 集成脚本内容
const FISH_INTEGRATION_SCRIPT: &str = r#"# ProxyCast Shell Integration for Fish
# This script provides shell integration features

# OSC 7 - 报告当前工作目录
function __proxycast_osc7 --on-variable PWD
    printf '\033]7;file://%s%s\033\\' (hostname) $PWD
end

# OSC 133 - 命令提示符标记
function __proxycast_prompt_start
    printf '\033]133;A\033\\'
end

function __proxycast_command_executed
    printf '\033]133;C\033\\'
end

function __proxycast_command_finished
    printf '\033]133;D;%s\033\\' $status
end

# Fish 事件钩子
function __proxycast_fish_prompt --on-event fish_prompt
    __proxycast_command_finished
    __proxycast_osc7
    __proxycast_prompt_start
end

function __proxycast_fish_preexec --on-event fish_preexec
    __proxycast_command_executed
end

# 初始化
__proxycast_osc7

# 标记集成已加载
set -gx PROXYCAST_SHELL_INTEGRATION 1
"#;

/// PowerShell 集成脚本内容
const PWSH_INTEGRATION_SCRIPT: &str = r#"# ProxyCast Shell Integration for PowerShell
# This script provides shell integration features

# OSC 7 - 报告当前工作目录
function Send-ProxyCastOsc7 {
    $hostname = [System.Net.Dns]::GetHostName()
    $pwd = $PWD.Path -replace '\\', '/'
    Write-Host -NoNewline "`e]7;file://$hostname$pwd`e\"
}

# OSC 133 - 命令提示符标记
function Send-ProxyCastPromptStart {
    Write-Host -NoNewline "`e]133;A`e\"
}

function Send-ProxyCastCommandExecuted {
    Write-Host -NoNewline "`e]133;C`e\"
}

function Send-ProxyCastCommandFinished {
    param([int]$ExitCode = 0)
    Write-Host -NoNewline "`e]133;D;$ExitCode`e\"
}

# 保存原始 prompt 函数
if (-not (Test-Path Function:\__ProxyCastOriginalPrompt)) {
    if (Test-Path Function:\prompt) {
        Copy-Item Function:\prompt Function:\__ProxyCastOriginalPrompt
    } else {
        function __ProxyCastOriginalPrompt { "PS $($PWD.Path)> " }
    }
}

# 自定义 prompt 函数
function prompt {
    $exitCode = $LASTEXITCODE
    Send-ProxyCastCommandFinished -ExitCode $exitCode
    Send-ProxyCastOsc7
    Send-ProxyCastPromptStart
    $LASTEXITCODE = $exitCode
    __ProxyCastOriginalPrompt
}

# PSReadLine 钩子（如果可用）
if (Get-Module -ListAvailable -Name PSReadLine) {
    $existingHandler = (Get-PSReadLineOption).AddToHistoryHandler
    Set-PSReadLineOption -AddToHistoryHandler {
        param([string]$line)
        Send-ProxyCastCommandExecuted
        if ($existingHandler) {
            return & $existingHandler $line
        }
        return $true
    }
}

# 标记集成已加载
$env:PROXYCAST_SHELL_INTEGRATION = "1"
"#;

/// Shell 集成脚本管理器
pub struct ShellScripts {
    /// 集成脚本目录
    integration_dir: PathBuf,
}

impl ShellScripts {
    /// 创建新的 Shell 脚本管理器
    ///
    /// # 参数
    /// - `app_data_dir`: 应用数据目录
    ///
    /// # 返回
    /// 新的 ShellScripts 实例
    pub fn new(app_data_dir: &Path) -> Self {
        Self {
            integration_dir: app_data_dir.join(SHELL_INTEGRATION_DIR),
        }
    }

    /// 获取集成脚本目录
    pub fn integration_dir(&self) -> &Path {
        &self.integration_dir
    }

    /// 确保集成脚本目录存在
    fn ensure_integration_dir(&self) -> Result<(), TerminalError> {
        if !self.integration_dir.exists() {
            fs::create_dir_all(&self.integration_dir)
                .map_err(|e| TerminalError::Internal(format!("创建集成脚本目录失败: {e}")))?;
        }
        Ok(())
    }

    /// 安装所有 Shell 集成脚本
    ///
    /// 将集成脚本写入应用数据目录。
    ///
    /// # 返回
    /// 成功返回 Ok(()), 失败返回错误
    pub fn install_all(&self) -> Result<(), TerminalError> {
        self.ensure_integration_dir()?;

        // 安装 Bash 脚本
        self.install_bash_scripts()?;

        // 安装 Zsh 脚本
        self.install_zsh_scripts()?;

        // 安装 Fish 脚本
        self.install_fish_scripts()?;

        // 安装 PowerShell 脚本
        self.install_pwsh_scripts()?;

        tracing::info!(
            "[ShellScripts] 所有集成脚本已安装: dir={}",
            self.integration_dir.display()
        );

        Ok(())
    }

    /// 安装 Bash 集成脚本
    ///
    /// _Requirements: 17.9_
    fn install_bash_scripts(&self) -> Result<(), TerminalError> {
        let bash_dir = self.integration_dir.join("bash");
        fs::create_dir_all(&bash_dir)
            .map_err(|e| TerminalError::Internal(format!("创建 bash 目录失败: {e}")))?;

        let script_path = bash_dir.join("proxycast.bash");
        fs::write(&script_path, BASH_INTEGRATION_SCRIPT)
            .map_err(|e| TerminalError::Internal(format!("写入 bash 脚本失败: {e}")))?;

        tracing::debug!("[ShellScripts] Bash 脚本已安装: {}", script_path.display());

        Ok(())
    }

    /// 安装 Zsh 集成脚本
    ///
    /// _Requirements: 17.8_
    fn install_zsh_scripts(&self) -> Result<(), TerminalError> {
        let zsh_dir = self.integration_dir.join("zsh");
        fs::create_dir_all(&zsh_dir)
            .map_err(|e| TerminalError::Internal(format!("创建 zsh 目录失败: {e}")))?;

        // 写入 .zshenv
        let zshenv_path = zsh_dir.join(".zshenv");
        fs::write(&zshenv_path, ZSH_ZSHENV_SCRIPT)
            .map_err(|e| TerminalError::Internal(format!("写入 .zshenv 失败: {e}")))?;

        // 写入 .zshrc
        let zshrc_path = zsh_dir.join(".zshrc");
        fs::write(&zshrc_path, ZSH_INTEGRATION_SCRIPT)
            .map_err(|e| TerminalError::Internal(format!("写入 .zshrc 失败: {e}")))?;

        tracing::debug!("[ShellScripts] Zsh 脚本已安装: {}", zsh_dir.display());

        Ok(())
    }

    /// 安装 Fish 集成脚本
    ///
    /// _Requirements: 17.10_
    fn install_fish_scripts(&self) -> Result<(), TerminalError> {
        let fish_dir = self.integration_dir.join("fish");
        fs::create_dir_all(&fish_dir)
            .map_err(|e| TerminalError::Internal(format!("创建 fish 目录失败: {e}")))?;

        let script_path = fish_dir.join("proxycast.fish");
        fs::write(&script_path, FISH_INTEGRATION_SCRIPT)
            .map_err(|e| TerminalError::Internal(format!("写入 fish 脚本失败: {e}")))?;

        tracing::debug!("[ShellScripts] Fish 脚本已安装: {}", script_path.display());

        Ok(())
    }

    /// 安装 PowerShell 集成脚本
    fn install_pwsh_scripts(&self) -> Result<(), TerminalError> {
        let pwsh_dir = self.integration_dir.join("pwsh");
        fs::create_dir_all(&pwsh_dir)
            .map_err(|e| TerminalError::Internal(format!("创建 pwsh 目录失败: {e}")))?;

        let script_path = pwsh_dir.join("proxycast.ps1");
        fs::write(&script_path, PWSH_INTEGRATION_SCRIPT)
            .map_err(|e| TerminalError::Internal(format!("写入 pwsh 脚本失败: {e}")))?;

        tracing::debug!(
            "[ShellScripts] PowerShell 脚本已安装: {}",
            script_path.display()
        );

        Ok(())
    }

    /// 获取 Bash 集成脚本路径
    pub fn bash_script_path(&self) -> PathBuf {
        self.integration_dir.join("bash").join("proxycast.bash")
    }

    /// 获取 Zsh 集成目录路径（用于 ZDOTDIR）
    pub fn zsh_integration_dir(&self) -> PathBuf {
        self.integration_dir.join("zsh")
    }

    /// 获取 Fish 集成脚本路径
    pub fn fish_script_path(&self) -> PathBuf {
        self.integration_dir.join("fish").join("proxycast.fish")
    }

    /// 获取 PowerShell 集成脚本路径
    pub fn pwsh_script_path(&self) -> PathBuf {
        self.integration_dir.join("pwsh").join("proxycast.ps1")
    }

    /// 检查集成脚本是否已安装
    pub fn is_installed(&self) -> bool {
        self.bash_script_path().exists()
            && self.zsh_integration_dir().join(".zshrc").exists()
            && self.fish_script_path().exists()
            && self.pwsh_script_path().exists()
    }
}

/// Shell 启动配置
///
/// 包含启动 Shell 所需的命令和环境变量配置。
#[derive(Debug, Clone)]
pub struct ShellLaunchConfig {
    /// Shell 可执行文件路径
    pub shell_path: String,
    /// Shell 参数
    pub args: Vec<String>,
    /// 环境变量
    pub env: HashMap<String, String>,
}

impl ShellLaunchConfig {
    /// 创建新的 Shell 启动配置
    pub fn new(shell_path: String) -> Self {
        Self {
            shell_path,
            args: Vec::new(),
            env: HashMap::new(),
        }
    }

    /// 添加参数
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// 添加环境变量
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }
}

/// Shell 启动配置构建器
///
/// 根据 Shell 类型和集成脚本路径构建启动配置。
pub struct ShellLaunchBuilder {
    /// Shell 脚本管理器
    scripts: ShellScripts,
    /// Block ID（用于环境变量）
    block_id: String,
}

impl ShellLaunchBuilder {
    /// 创建新的启动配置构建器
    ///
    /// # 参数
    /// - `app_data_dir`: 应用数据目录
    /// - `block_id`: Block ID
    pub fn new(app_data_dir: &Path, block_id: String) -> Self {
        Self {
            scripts: ShellScripts::new(app_data_dir),
            block_id,
        }
    }

    /// 确保集成脚本已安装
    pub fn ensure_scripts_installed(&self) -> Result<(), TerminalError> {
        if !self.scripts.is_installed() {
            self.scripts.install_all()?;
        }
        Ok(())
    }

    /// 构建 Shell 启动配置
    ///
    /// 根据 Shell 类型构建带有集成脚本的启动配置。
    ///
    /// # 参数
    /// - `shell_path`: Shell 可执行文件路径
    /// - `custom_env`: 自定义环境变量
    ///
    /// # 返回
    /// Shell 启动配置
    ///
    /// _Requirements: 17.5, 17.6, 17.7, 17.8, 17.9, 17.10_
    pub fn build(
        &self,
        shell_path: &str,
        custom_env: Option<&HashMap<String, String>>,
    ) -> Result<ShellLaunchConfig, TerminalError> {
        // 确保脚本已安装
        self.ensure_scripts_installed()?;

        let shell_type = ShellType::from_path(shell_path);
        let mut config = ShellLaunchConfig::new(shell_path.to_string());

        // 设置通用环境变量
        config = self.set_common_env(config);

        // 根据 Shell 类型设置特定配置
        config = match shell_type {
            ShellType::Bash => self.configure_bash(config)?,
            ShellType::Zsh => self.configure_zsh(config)?,
            ShellType::Fish => self.configure_fish(config)?,
            ShellType::Pwsh => self.configure_pwsh(config)?,
            ShellType::Unknown => {
                tracing::warn!(
                    "[ShellLaunchBuilder] 未知 Shell 类型，不加载集成脚本: {}",
                    shell_path
                );
                config
            }
        };

        // 添加自定义环境变量
        if let Some(env) = custom_env {
            for (key, value) in env {
                config.env.insert(key.clone(), value.clone());
            }
        }

        Ok(config)
    }

    /// 设置通用环境变量
    ///
    /// 设置所有 Shell 类型共用的环境变量，包括：
    /// - TERM: 终端类型
    /// - COLORTERM: 颜色支持
    /// - PROXYCAST_BLOCKID: 块 ID（与 WAVETERM_BLOCKID 兼容）
    /// - WAVETERM_BLOCKID: Waveterm 兼容的块 ID
    /// - PROXYCAST_VERSION: 应用版本
    /// - LANG: 语言设置（如果未设置）
    ///
    /// _Requirements: 17.2, 17.7_
    fn set_common_env(&self, config: ShellLaunchConfig) -> ShellLaunchConfig {
        let mut config = config
            .env("TERM", "xterm-256color")
            .env("COLORTERM", "truecolor")
            .env("PROXYCAST_BLOCKID", &self.block_id)
            // Waveterm 兼容性
            .env("WAVETERM_BLOCKID", &self.block_id)
            .env("PROXYCAST_VERSION", env!("CARGO_PKG_VERSION"));

        // 设置 LANG（如果未设置）
        if std::env::var("LANG").is_err() {
            config = config.env("LANG", "en_US.UTF-8");
        }

        // 设置 LC_ALL（如果未设置）
        if std::env::var("LC_ALL").is_err() {
            config = config.env("LC_ALL", "en_US.UTF-8");
        }

        config
    }

    /// 配置 Bash 启动
    ///
    /// 使用 --rcfile 参数加载集成脚本。
    ///
    /// _Requirements: 17.9_
    fn configure_bash(
        &self,
        config: ShellLaunchConfig,
    ) -> Result<ShellLaunchConfig, TerminalError> {
        let script_path = self.scripts.bash_script_path();

        if !script_path.exists() {
            tracing::warn!(
                "[ShellLaunchBuilder] Bash 集成脚本不存在: {}",
                script_path.display()
            );
            return Ok(config);
        }

        let script_path_str = script_path.to_string_lossy().to_string();

        Ok(config
            .arg("--rcfile")
            .arg(&script_path_str)
            .env("_PROXYCAST_LOAD_BASHRC", "1"))
    }

    /// 配置 Zsh 启动
    ///
    /// 使用 ZDOTDIR 环境变量指向集成目录。
    ///
    /// _Requirements: 17.8_
    fn configure_zsh(&self, config: ShellLaunchConfig) -> Result<ShellLaunchConfig, TerminalError> {
        let zsh_dir = self.scripts.zsh_integration_dir();

        if !zsh_dir.join(".zshrc").exists() {
            tracing::warn!(
                "[ShellLaunchBuilder] Zsh 集成脚本不存在: {}",
                zsh_dir.display()
            );
            return Ok(config);
        }

        let zsh_dir_str = zsh_dir.to_string_lossy().to_string();

        Ok(config.env("ZDOTDIR", &zsh_dir_str))
    }

    /// 配置 Fish 启动
    ///
    /// 使用 -C 参数 source 集成脚本。
    ///
    /// _Requirements: 17.10_
    fn configure_fish(
        &self,
        config: ShellLaunchConfig,
    ) -> Result<ShellLaunchConfig, TerminalError> {
        let script_path = self.scripts.fish_script_path();

        if !script_path.exists() {
            tracing::warn!(
                "[ShellLaunchBuilder] Fish 集成脚本不存在: {}",
                script_path.display()
            );
            return Ok(config);
        }

        let source_cmd = format!("source {}", script_path.to_string_lossy());

        Ok(config.arg("-C").arg(&source_cmd))
    }

    /// 配置 PowerShell 启动
    fn configure_pwsh(
        &self,
        config: ShellLaunchConfig,
    ) -> Result<ShellLaunchConfig, TerminalError> {
        let script_path = self.scripts.pwsh_script_path();

        if !script_path.exists() {
            tracing::warn!(
                "[ShellLaunchBuilder] PowerShell 集成脚本不存在: {}",
                script_path.display()
            );
            return Ok(config);
        }

        let script_path_str = script_path.to_string_lossy().to_string();

        // PowerShell 使用 -NoExit 保持会话，-Command 执行脚本
        Ok(config
            .arg("-NoExit")
            .arg("-Command")
            .arg(format!(". '{script_path_str}'")))
    }
}

/// 终端环境变量配置
///
/// 提供终端环境变量的配置和管理功能。
///
/// ## 标准环境变量
/// - `TERM`: 终端类型（默认 xterm-256color）
/// - `COLORTERM`: 颜色支持（默认 truecolor）
/// - `PROXYCAST_BLOCKID`: 块 ID
/// - `WAVETERM_BLOCKID`: Waveterm 兼容的块 ID
/// - `PROXYCAST_VERSION`: 应用版本
/// - `LANG`: 语言设置
/// - `LC_ALL`: 区域设置
///
/// _Requirements: 17.2, 17.7_
#[derive(Debug, Clone, Default)]
pub struct TerminalEnvConfig {
    /// 环境变量映射
    env: HashMap<String, String>,
}

impl TerminalEnvConfig {
    /// 创建新的环境变量配置
    pub fn new() -> Self {
        Self {
            env: HashMap::new(),
        }
    }

    /// 创建带有默认环境变量的配置
    ///
    /// # 参数
    /// - `block_id`: 块 ID
    ///
    /// # 返回
    /// 包含默认环境变量的配置
    ///
    /// _Requirements: 17.2_
    pub fn with_defaults(block_id: &str) -> Self {
        let mut config = Self::new();

        // 终端类型
        config.set("TERM", "xterm-256color");
        config.set("COLORTERM", "truecolor");

        // 块标识
        config.set("PROXYCAST_BLOCKID", block_id);
        config.set("WAVETERM_BLOCKID", block_id); // Waveterm 兼容

        // 版本信息
        config.set("PROXYCAST_VERSION", env!("CARGO_PKG_VERSION"));

        // 语言设置（如果未设置）
        if std::env::var("LANG").is_err() {
            config.set("LANG", "en_US.UTF-8");
        }
        if std::env::var("LC_ALL").is_err() {
            config.set("LC_ALL", "en_US.UTF-8");
        }

        config
    }

    /// 设置环境变量
    ///
    /// # 参数
    /// - `key`: 环境变量名
    /// - `value`: 环境变量值
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.env.insert(key.into(), value.into());
    }

    /// 获取环境变量
    ///
    /// # 参数
    /// - `key`: 环境变量名
    ///
    /// # 返回
    /// 环境变量值（如果存在）
    pub fn get(&self, key: &str) -> Option<&String> {
        self.env.get(key)
    }

    /// 移除环境变量
    ///
    /// # 参数
    /// - `key`: 环境变量名
    ///
    /// # 返回
    /// 被移除的值（如果存在）
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.env.remove(key)
    }

    /// 合并自定义环境变量
    ///
    /// 自定义环境变量会覆盖已有的同名变量。
    ///
    /// # 参数
    /// - `custom_env`: 自定义环境变量
    ///
    /// _Requirements: 17.7_
    pub fn merge(&mut self, custom_env: &HashMap<String, String>) {
        for (key, value) in custom_env {
            self.env.insert(key.clone(), value.clone());
        }
    }

    /// 获取所有环境变量
    ///
    /// # 返回
    /// 环境变量映射的引用
    pub fn all(&self) -> &HashMap<String, String> {
        &self.env
    }

    /// 转换为 HashMap
    ///
    /// # 返回
    /// 环境变量的 HashMap
    pub fn into_map(self) -> HashMap<String, String> {
        self.env
    }

    /// 检查是否包含指定的环境变量
    ///
    /// # 参数
    /// - `key`: 环境变量名
    ///
    /// # 返回
    /// 是否包含该环境变量
    pub fn contains(&self, key: &str) -> bool {
        self.env.contains_key(key)
    }

    /// 获取环境变量数量
    pub fn len(&self) -> usize {
        self.env.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.env.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_shell_scripts_install() {
        let temp_dir = TempDir::new().unwrap();
        let scripts = ShellScripts::new(temp_dir.path());

        scripts.install_all().unwrap();

        assert!(scripts.bash_script_path().exists());
        assert!(scripts.zsh_integration_dir().join(".zshrc").exists());
        assert!(scripts.zsh_integration_dir().join(".zshenv").exists());
        assert!(scripts.fish_script_path().exists());
        assert!(scripts.pwsh_script_path().exists());
        assert!(scripts.is_installed());
    }

    #[test]
    fn test_shell_launch_builder_bash() {
        let temp_dir = TempDir::new().unwrap();
        let builder = ShellLaunchBuilder::new(temp_dir.path(), "test-block".to_string());

        let config = builder.build("/bin/bash", None).unwrap();

        assert_eq!(config.shell_path, "/bin/bash");
        assert!(config.args.contains(&"--rcfile".to_string()));
        assert!(config.env.contains_key("TERM"));
        assert!(config.env.contains_key("PROXYCAST_BLOCKID"));
        assert!(config.env.contains_key("WAVETERM_BLOCKID"));
        assert_eq!(
            config.env.get("_PROXYCAST_LOAD_BASHRC"),
            Some(&"1".to_string())
        );
    }

    #[test]
    fn test_shell_launch_builder_zsh() {
        let temp_dir = TempDir::new().unwrap();
        let builder = ShellLaunchBuilder::new(temp_dir.path(), "test-block".to_string());

        let config = builder.build("/bin/zsh", None).unwrap();

        assert_eq!(config.shell_path, "/bin/zsh");
        assert!(config.env.contains_key("ZDOTDIR"));
        assert!(config.env.contains_key("TERM"));
        assert!(config.env.contains_key("WAVETERM_BLOCKID"));
    }

    #[test]
    fn test_shell_launch_builder_fish() {
        let temp_dir = TempDir::new().unwrap();
        let builder = ShellLaunchBuilder::new(temp_dir.path(), "test-block".to_string());

        let config = builder.build("/usr/bin/fish", None).unwrap();

        assert_eq!(config.shell_path, "/usr/bin/fish");
        assert!(config.args.contains(&"-C".to_string()));
        assert!(config.env.contains_key("TERM"));
        assert!(config.env.contains_key("WAVETERM_BLOCKID"));
    }

    #[test]
    fn test_shell_launch_builder_pwsh() {
        let temp_dir = TempDir::new().unwrap();
        let builder = ShellLaunchBuilder::new(temp_dir.path(), "test-block".to_string());

        let config = builder.build("/usr/bin/pwsh", None).unwrap();

        assert_eq!(config.shell_path, "/usr/bin/pwsh");
        assert!(config.args.contains(&"-NoExit".to_string()));
        assert!(config.args.contains(&"-Command".to_string()));
        assert!(config.env.contains_key("TERM"));
        assert!(config.env.contains_key("WAVETERM_BLOCKID"));
    }

    #[test]
    fn test_shell_launch_builder_custom_env() {
        let temp_dir = TempDir::new().unwrap();
        let builder = ShellLaunchBuilder::new(temp_dir.path(), "test-block".to_string());

        let mut custom_env = HashMap::new();
        custom_env.insert("MY_VAR".to_string(), "my_value".to_string());

        let config = builder.build("/bin/bash", Some(&custom_env)).unwrap();

        assert_eq!(config.env.get("MY_VAR"), Some(&"my_value".to_string()));
    }

    #[test]
    fn test_shell_launch_config_builder_pattern() {
        let config = ShellLaunchConfig::new("/bin/bash".to_string())
            .arg("--login")
            .arg("-i")
            .env("FOO", "bar")
            .env("BAZ", "qux");

        assert_eq!(config.shell_path, "/bin/bash");
        assert_eq!(config.args, vec!["--login", "-i"]);
        assert_eq!(config.env.get("FOO"), Some(&"bar".to_string()));
        assert_eq!(config.env.get("BAZ"), Some(&"qux".to_string()));
    }

    #[test]
    fn test_terminal_env_config_defaults() {
        let config = TerminalEnvConfig::with_defaults("test-block");

        assert_eq!(config.get("TERM"), Some(&"xterm-256color".to_string()));
        assert_eq!(config.get("COLORTERM"), Some(&"truecolor".to_string()));
        assert_eq!(
            config.get("PROXYCAST_BLOCKID"),
            Some(&"test-block".to_string())
        );
        assert_eq!(
            config.get("WAVETERM_BLOCKID"),
            Some(&"test-block".to_string())
        );
        assert!(config.contains("PROXYCAST_VERSION"));
    }

    #[test]
    fn test_terminal_env_config_merge() {
        let mut config = TerminalEnvConfig::with_defaults("test-block");

        let mut custom = HashMap::new();
        custom.insert("MY_VAR".to_string(), "my_value".to_string());
        custom.insert("TERM".to_string(), "xterm".to_string()); // 覆盖默认值

        config.merge(&custom);

        assert_eq!(config.get("MY_VAR"), Some(&"my_value".to_string()));
        assert_eq!(config.get("TERM"), Some(&"xterm".to_string())); // 被覆盖
    }

    #[test]
    fn test_terminal_env_config_operations() {
        let mut config = TerminalEnvConfig::new();

        config.set("KEY1", "value1");
        config.set("KEY2", "value2");

        assert_eq!(config.len(), 2);
        assert!(!config.is_empty());
        assert!(config.contains("KEY1"));
        assert!(!config.contains("KEY3"));

        let removed = config.remove("KEY1");
        assert_eq!(removed, Some("value1".to_string()));
        assert_eq!(config.len(), 1);
    }
}
