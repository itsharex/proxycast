use super::api::{fetch_qr_code, poll_qr_status};
use super::types::{DEFAULT_BASE_URL, DEFAULT_ILINK_BOT_TYPE};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

const ACTIVE_LOGIN_TTL_SECS: i64 = 5 * 60;
const MAX_QR_REFRESH_COUNT: u8 = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WechatLoginStartResult {
    pub session_key: String,
    pub qrcode_url: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WechatLoginWaitResult {
    pub connected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone)]
struct ActiveLogin {
    session_key: String,
    qrcode: String,
    started_at: chrono::DateTime<Utc>,
    refresh_count: u8,
}

pub struct WechatLoginState {
    inner: Arc<RwLock<HashMap<String, ActiveLogin>>>,
}

impl Default for WechatLoginState {
    fn default() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

fn normalize_base_url(base_url: Option<&str>) -> String {
    base_url
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_BASE_URL)
        .to_string()
}

fn is_login_fresh(login: &ActiveLogin) -> bool {
    Utc::now() - login.started_at < Duration::seconds(ACTIVE_LOGIN_TTL_SECS)
}

async fn purge_expired(logins: &Arc<RwLock<HashMap<String, ActiveLogin>>>) {
    let mut guard = logins.write().await;
    guard.retain(|_, login| is_login_fresh(login));
}

pub async fn start_login(
    state: &WechatLoginState,
    client: &reqwest::Client,
    base_url: Option<&str>,
    bot_type: Option<&str>,
    session_key: Option<&str>,
) -> Result<WechatLoginStartResult, String> {
    purge_expired(&state.inner).await;
    let base_url = normalize_base_url(base_url);
    let bot_type = bot_type.unwrap_or(DEFAULT_ILINK_BOT_TYPE);
    let session_key = session_key
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let qr = fetch_qr_code(client, &base_url, bot_type).await?;
    let login = ActiveLogin {
        session_key: session_key.clone(),
        qrcode: qr.qrcode.clone(),
        started_at: Utc::now(),
        refresh_count: 1,
    };
    state.inner.write().await.insert(session_key.clone(), login);
    Ok(WechatLoginStartResult {
        session_key,
        qrcode_url: qr.qrcode_img_content,
        message: "使用微信扫描二维码完成连接。".to_string(),
    })
}

pub async fn wait_login(
    state: &WechatLoginState,
    client: &reqwest::Client,
    session_key: &str,
    base_url: Option<&str>,
    bot_type: Option<&str>,
    timeout_ms: Option<u64>,
) -> Result<WechatLoginWaitResult, String> {
    let base_url = normalize_base_url(base_url);
    let bot_type = bot_type.unwrap_or(DEFAULT_ILINK_BOT_TYPE);
    let timeout_ms = timeout_ms.unwrap_or(480_000).max(1_000);
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);

    loop {
        let snapshot = {
            let guard = state.inner.read().await;
            guard
                .get(session_key)
                .cloned()
                .ok_or_else(|| "当前没有进行中的微信登录".to_string())?
        };
        if !is_login_fresh(&snapshot) {
            state.inner.write().await.remove(session_key);
            return Ok(WechatLoginWaitResult {
                connected: false,
                bot_token: None,
                account_id: None,
                base_url: None,
                user_id: None,
                message: "二维码已过期，请重新生成。".to_string(),
            });
        }
        if std::time::Instant::now() >= deadline {
            return Ok(WechatLoginWaitResult {
                connected: false,
                bot_token: None,
                account_id: None,
                base_url: None,
                user_id: None,
                message: "登录超时，请稍后重试。".to_string(),
            });
        }

        let status = poll_qr_status(client, &base_url, &snapshot.qrcode).await?;
        match status.status.as_str() {
            "wait" | "scaned" => {
                tokio::time::sleep(std::time::Duration::from_millis(1_200)).await;
            }
            "confirmed" => {
                state.inner.write().await.remove(session_key);
                return Ok(WechatLoginWaitResult {
                    connected: true,
                    bot_token: status.bot_token,
                    account_id: status.ilink_bot_id,
                    base_url: status.baseurl.or(Some(base_url)),
                    user_id: status.ilink_user_id,
                    message: "微信连接成功。".to_string(),
                });
            }
            "expired" => {
                if snapshot.refresh_count >= MAX_QR_REFRESH_COUNT {
                    state.inner.write().await.remove(session_key);
                    return Ok(WechatLoginWaitResult {
                        connected: false,
                        bot_token: None,
                        account_id: None,
                        base_url: None,
                        user_id: None,
                        message: "二维码多次过期，请重新开始登录。".to_string(),
                    });
                }
                let new_qr = fetch_qr_code(client, &base_url, bot_type).await?;
                state.inner.write().await.insert(
                    session_key.to_string(),
                    ActiveLogin {
                        session_key: snapshot.session_key,
                        qrcode: new_qr.qrcode,
                        started_at: Utc::now(),
                        refresh_count: snapshot.refresh_count + 1,
                    },
                );
            }
            other => {
                return Err(format!("未知的二维码状态: {other}"));
            }
        }
    }
}
