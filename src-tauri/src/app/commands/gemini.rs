//! Gemini Provider 命令 (Legacy)
//!
//! 包含 Gemini 凭证管理相关命令。
//! 这些命令保留用于向后兼容，新代码应使用统一的 OAuth 命令。

use crate::app::commands::kiro::{CheckResult, EnvVariable};
use crate::app::types::{AppState, LogState};
use crate::app::utils::mask_token;
use crate::providers;

/// Gemini 凭证状态
#[derive(serde::Serialize)]
pub struct GeminiCredentialStatus {
    pub loaded: bool,
    pub has_access_token: bool,
    pub has_refresh_token: bool,
    pub expiry_date: Option<i64>,
    pub is_valid: bool,
    pub creds_path: String,
}

/// 获取 Gemini 凭证状态
#[tauri::command]
pub async fn get_gemini_credentials(
    state: tauri::State<'_, AppState>,
) -> Result<GeminiCredentialStatus, String> {
    let s = state.read().await;
    let creds = &s.gemini_provider.credentials;
    let path = providers::gemini::GeminiProvider::default_creds_path();

    Ok(GeminiCredentialStatus {
        loaded: creds.access_token.is_some() || creds.refresh_token.is_some(),
        has_access_token: creds.access_token.is_some(),
        has_refresh_token: creds.refresh_token.is_some(),
        expiry_date: creds.expiry_date,
        is_valid: s.gemini_provider.is_token_valid(),
        creds_path: path.to_string_lossy().to_string(),
    })
}

/// 重新加载 Gemini 凭证
#[tauri::command]
pub async fn reload_gemini_credentials(
    state: tauri::State<'_, AppState>,
    logs: tauri::State<'_, LogState>,
) -> Result<String, String> {
    let mut s = state.write().await;
    logs.write().await.add("info", "[Gemini] 正在加载凭证...");
    s.gemini_provider
        .load_credentials()
        .await
        .map_err(|e| e.to_string())?;
    logs.write().await.add("info", "[Gemini] 凭证加载成功");
    Ok("Gemini credentials reloaded".to_string())
}

/// 刷新 Gemini Token
#[tauri::command]
pub async fn refresh_gemini_token(
    state: tauri::State<'_, AppState>,
    logs: tauri::State<'_, LogState>,
) -> Result<String, String> {
    let mut s = state.write().await;
    logs.write().await.add("info", "[Gemini] 正在刷新 Token...");
    let result = s
        .gemini_provider
        .refresh_token()
        .await
        .map_err(|e| e.to_string());
    match &result {
        Ok(_) => logs.write().await.add("info", "[Gemini] Token 刷新成功"),
        Err(e) => logs
            .write()
            .await
            .add("error", &format!("[Gemini] Token 刷新失败: {e}")),
    }
    result
}

/// 获取 Gemini 环境变量
#[tauri::command]
pub async fn get_gemini_env_variables(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<EnvVariable>, String> {
    let s = state.read().await;
    let creds = &s.gemini_provider.credentials;
    let mut vars = Vec::new();

    if let Some(token) = &creds.access_token {
        vars.push(EnvVariable {
            key: "GEMINI_ACCESS_TOKEN".to_string(),
            value: token.clone(),
            masked: mask_token(token),
        });
    }
    if let Some(token) = &creds.refresh_token {
        vars.push(EnvVariable {
            key: "GEMINI_REFRESH_TOKEN".to_string(),
            value: token.clone(),
            masked: mask_token(token),
        });
    }
    if let Some(expiry) = creds.expiry_date {
        let expiry_str = expiry.to_string();
        vars.push(EnvVariable {
            key: "GEMINI_EXPIRY_DATE".to_string(),
            value: expiry_str.clone(),
            masked: expiry_str,
        });
    }

    Ok(vars)
}

/// 获取 Gemini Token 文件哈希
#[tauri::command]
pub async fn get_gemini_token_file_hash() -> Result<String, String> {
    let path = providers::gemini::GeminiProvider::default_creds_path();
    if !tokio::fs::try_exists(&path).await.unwrap_or(false) {
        return Ok("".to_string());
    }

    let content = tokio::fs::read(&path).await.map_err(|e| e.to_string())?;
    let hash = format!("{:x}", md5::compute(&content));
    Ok(hash)
}

/// 检查并重新加载 Gemini 凭证
#[tauri::command]
pub async fn check_and_reload_gemini_credentials(
    state: tauri::State<'_, AppState>,
    logs: tauri::State<'_, LogState>,
    last_hash: String,
) -> Result<CheckResult, String> {
    let path = providers::gemini::GeminiProvider::default_creds_path();

    if !tokio::fs::try_exists(&path).await.unwrap_or(false) {
        return Ok(CheckResult {
            changed: false,
            new_hash: "".to_string(),
            reloaded: false,
        });
    }

    let content = tokio::fs::read(&path).await.map_err(|e| e.to_string())?;
    let new_hash = format!("{:x}", md5::compute(&content));

    if !last_hash.is_empty() && new_hash != last_hash {
        logs.write()
            .await
            .add("info", "[Gemini][自动检测] 凭证文件已变化，正在重新加载...");

        let mut s = state.write().await;
        match s.gemini_provider.load_credentials().await {
            Ok(_) => {
                logs.write()
                    .await
                    .add("info", "[Gemini][自动检测] 凭证重新加载成功");
                Ok(CheckResult {
                    changed: true,
                    new_hash,
                    reloaded: true,
                })
            }
            Err(e) => {
                logs.write().await.add(
                    "error",
                    &format!("[Gemini][自动检测] 凭证重新加载失败: {e}"),
                );
                Ok(CheckResult {
                    changed: true,
                    new_hash,
                    reloaded: false,
                })
            }
        }
    } else {
        Ok(CheckResult {
            changed: false,
            new_hash,
            reloaded: false,
        })
    }
}
