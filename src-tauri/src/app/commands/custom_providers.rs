//! 自定义 Provider 命令
//!
//! 包含 OpenAI Custom 和 Claude Custom Provider 的配置命令。

use crate::app::types::{AppState, LogState};

/// OpenAI Custom 状态
#[derive(serde::Serialize, serde::Deserialize)]
pub struct OpenAICustomStatus {
    pub enabled: bool,
    pub has_api_key: bool,
    pub base_url: String,
}

/// Claude Custom 状态
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ClaudeCustomStatus {
    pub enabled: bool,
    pub has_api_key: bool,
    pub base_url: String,
}

/// 获取 OpenAI Custom 状态
#[tauri::command]
pub async fn get_openai_custom_status(
    state: tauri::State<'_, AppState>,
) -> Result<OpenAICustomStatus, String> {
    let s = state.read().await;
    let config = &s.openai_custom_provider.config;
    Ok(OpenAICustomStatus {
        enabled: config.enabled,
        has_api_key: config.api_key.is_some(),
        base_url: s.openai_custom_provider.get_base_url(),
    })
}

/// 设置 OpenAI Custom 配置
#[tauri::command]
pub async fn set_openai_custom_config(
    state: tauri::State<'_, AppState>,
    logs: tauri::State<'_, LogState>,
    api_key: Option<String>,
    base_url: Option<String>,
    enabled: bool,
) -> Result<String, String> {
    let mut s = state.write().await;
    s.openai_custom_provider.config.api_key = api_key;
    s.openai_custom_provider.config.base_url = base_url;
    s.openai_custom_provider.config.enabled = enabled;
    logs.write().await.add(
        "info",
        &format!("[OpenAI Custom] 配置已更新, enabled={enabled}"),
    );
    Ok("OpenAI Custom config updated".to_string())
}

/// 获取 Claude Custom 状态
#[tauri::command]
pub async fn get_claude_custom_status(
    state: tauri::State<'_, AppState>,
) -> Result<ClaudeCustomStatus, String> {
    let s = state.read().await;
    let config = &s.claude_custom_provider.config;
    Ok(ClaudeCustomStatus {
        enabled: config.enabled,
        has_api_key: config.api_key.is_some(),
        base_url: s.claude_custom_provider.get_base_url(),
    })
}

/// 设置 Claude Custom 配置
#[tauri::command]
pub async fn set_claude_custom_config(
    state: tauri::State<'_, AppState>,
    logs: tauri::State<'_, LogState>,
    api_key: Option<String>,
    base_url: Option<String>,
    enabled: bool,
) -> Result<String, String> {
    let mut s = state.write().await;
    s.claude_custom_provider.config.api_key = api_key;
    s.claude_custom_provider.config.base_url = base_url;
    s.claude_custom_provider.config.enabled = enabled;
    logs.write().await.add(
        "info",
        &format!("[Claude Custom] 配置已更新, enabled={enabled}"),
    );
    Ok("Claude Custom config updated".to_string())
}
