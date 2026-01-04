//! Kiro Provider 命令 (Legacy)
//!
//! 包含 Kiro 凭证管理相关命令。
//! 这些命令保留用于向后兼容，新代码应使用统一的 OAuth 命令。

use crate::app::types::{AppState, LogState};
use crate::app::utils::mask_token;
use crate::providers;

/// Kiro 凭证状态
#[derive(serde::Serialize)]
pub struct KiroCredentialStatus {
    pub loaded: bool,
    pub has_access_token: bool,
    pub has_refresh_token: bool,
    pub region: Option<String>,
    pub auth_method: Option<String>,
    pub expires_at: Option<String>,
    pub creds_path: String,
}

/// 环境变量
#[derive(serde::Serialize)]
pub struct EnvVariable {
    pub key: String,
    pub value: String,
    pub masked: String,
}

/// 检查结果
#[derive(serde::Serialize)]
pub struct CheckResult {
    pub changed: bool,
    pub new_hash: String,
    pub reloaded: bool,
}

/// 刷新 Kiro Token
#[tauri::command]
pub async fn refresh_kiro_token(
    state: tauri::State<'_, AppState>,
    logs: tauri::State<'_, LogState>,
) -> Result<String, String> {
    let mut s = state.write().await;
    logs.write().await.add("info", "Refreshing Kiro token...");
    let result = s
        .kiro_provider
        .refresh_token()
        .await
        .map_err(|e| e.to_string());
    match &result {
        Ok(_) => logs
            .write()
            .await
            .add("info", "Token refreshed successfully"),
        Err(e) => logs
            .write()
            .await
            .add("error", &format!("Token refresh failed: {e}")),
    }
    result
}

/// 重新加载凭证
#[tauri::command]
pub async fn reload_credentials(
    state: tauri::State<'_, AppState>,
    logs: tauri::State<'_, LogState>,
) -> Result<String, String> {
    let mut s = state.write().await;
    logs.write().await.add("info", "Reloading credentials...");
    s.kiro_provider
        .load_credentials()
        .await
        .map_err(|e| e.to_string())?;
    logs.write().await.add("info", "Credentials reloaded");
    Ok("Credentials reloaded".to_string())
}

/// 获取 Kiro 凭证状态
#[tauri::command]
pub async fn get_kiro_credentials(
    state: tauri::State<'_, AppState>,
) -> Result<KiroCredentialStatus, String> {
    let s = state.read().await;
    let creds = &s.kiro_provider.credentials;
    let path = providers::kiro::KiroProvider::default_creds_path();

    Ok(KiroCredentialStatus {
        loaded: creds.access_token.is_some() || creds.refresh_token.is_some(),
        has_access_token: creds.access_token.is_some(),
        has_refresh_token: creds.refresh_token.is_some(),
        region: creds.region.clone(),
        auth_method: creds.auth_method.clone(),
        expires_at: creds.expires_at.clone(),
        creds_path: path.to_string_lossy().to_string(),
    })
}

/// 获取环境变量
#[tauri::command]
pub async fn get_env_variables(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<EnvVariable>, String> {
    let s = state.read().await;
    let creds = &s.kiro_provider.credentials;
    let mut vars = Vec::new();

    // P0 安全修复：不再返回明文敏感凭证，仅返回 masked 版本
    if let Some(token) = &creds.access_token {
        vars.push(EnvVariable {
            key: "KIRO_ACCESS_TOKEN".to_string(),
            value: String::new(), // 不返回明文
            masked: mask_token(token),
        });
    }
    if let Some(token) = &creds.refresh_token {
        vars.push(EnvVariable {
            key: "KIRO_REFRESH_TOKEN".to_string(),
            value: String::new(), // 不返回明文
            masked: mask_token(token),
        });
    }
    if let Some(id) = &creds.client_id {
        vars.push(EnvVariable {
            key: "KIRO_CLIENT_ID".to_string(),
            value: String::new(), // 不返回明文
            masked: mask_token(id),
        });
    }
    if let Some(secret) = &creds.client_secret {
        vars.push(EnvVariable {
            key: "KIRO_CLIENT_SECRET".to_string(),
            value: String::new(), // 不返回明文
            masked: mask_token(secret),
        });
    }
    if let Some(arn) = &creds.profile_arn {
        vars.push(EnvVariable {
            key: "KIRO_PROFILE_ARN".to_string(),
            value: arn.clone(),
            masked: arn.clone(),
        });
    }
    if let Some(region) = &creds.region {
        vars.push(EnvVariable {
            key: "KIRO_REGION".to_string(),
            value: region.clone(),
            masked: region.clone(),
        });
    }
    if let Some(method) = &creds.auth_method {
        vars.push(EnvVariable {
            key: "KIRO_AUTH_METHOD".to_string(),
            value: method.clone(),
            masked: method.clone(),
        });
    }

    Ok(vars)
}

/// 获取 Token 文件哈希
#[tauri::command]
pub async fn get_token_file_hash() -> Result<String, String> {
    let path = providers::kiro::KiroProvider::default_creds_path();
    if !tokio::fs::try_exists(&path).await.unwrap_or(false) {
        return Ok("".to_string());
    }

    let content = tokio::fs::read(&path).await.map_err(|e| e.to_string())?;
    let hash = format!("{:x}", md5::compute(&content));
    Ok(hash)
}

/// 检查凭证文件变化并自动重新加载
#[tauri::command]
pub async fn check_and_reload_credentials(
    state: tauri::State<'_, AppState>,
    logs: tauri::State<'_, LogState>,
    last_hash: String,
) -> Result<CheckResult, String> {
    let path = providers::kiro::KiroProvider::default_creds_path();

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
            .add("info", "[自动检测] 凭证文件已变化，正在重新加载...");

        let mut s = state.write().await;
        match s.kiro_provider.load_credentials().await {
            Ok(_) => {
                logs.write()
                    .await
                    .add("info", "[自动检测] 凭证重新加载成功");
                Ok(CheckResult {
                    changed: true,
                    new_hash,
                    reloaded: true,
                })
            }
            Err(e) => {
                logs.write()
                    .await
                    .add("error", &format!("[自动检测] 凭证重新加载失败: {e}"));
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
