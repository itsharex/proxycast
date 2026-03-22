use super::types::{
    BaseInfo, GetConfigResp, GetUpdatesResp, GetUploadUrlReq, GetUploadUrlResp, QrCodeResponse,
    QrStatusResponse, SendMessageReq, SendTypingReq, SendTypingResp,
};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use rand::Rng;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::de::DeserializeOwned;
use serde_json::json;
use std::time::Duration;

const CHANNEL_VERSION: &str = "lime-wechat-rust";
const DEFAULT_API_TIMEOUT_MS: u64 = 15_000;
const DEFAULT_LONG_POLL_TIMEOUT_MS: u64 = 35_000;
const DEFAULT_CONFIG_TIMEOUT_MS: u64 = 10_000;

pub(crate) fn build_base_info() -> BaseInfo {
    BaseInfo {
        channel_version: Some(CHANNEL_VERSION.to_string()),
    }
}

fn ensure_trailing_slash(url: &str) -> String {
    if url.ends_with('/') {
        url.to_string()
    } else {
        format!("{url}/")
    }
}

fn random_wechat_uin() -> String {
    let mut rng = rand::thread_rng();
    let value: u32 = rng.gen();
    BASE64_STANDARD.encode(value.to_string().as_bytes())
}

fn build_api_headers(token: Option<&str>) -> Result<HeaderMap, String> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        "AuthorizationType",
        HeaderValue::from_static("ilink_bot_token"),
    );
    headers.insert(
        "X-WECHAT-UIN",
        HeaderValue::from_str(&random_wechat_uin()).map_err(|e| e.to_string())?,
    );
    if let Some(token) = token.map(str::trim).filter(|value| !value.is_empty()) {
        headers.insert(
            reqwest::header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}")).map_err(|e| e.to_string())?,
        );
    }
    Ok(headers)
}

async fn parse_json_response<T: DeserializeOwned>(
    response: reqwest::Response,
) -> Result<T, String> {
    let status = response.status();
    let text = response.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!("HTTP {}: {}", status.as_u16(), text));
    }
    serde_json::from_str(&text).map_err(|e| format!("解析响应失败: {e}; body={text}"))
}

async fn post_json<T: DeserializeOwned>(
    client: &reqwest::Client,
    base_url: &str,
    endpoint: &str,
    body: serde_json::Value,
    token: Option<&str>,
    timeout_ms: u64,
) -> Result<T, String> {
    let url = format!("{}{}", ensure_trailing_slash(base_url), endpoint);
    let headers = build_api_headers(token)?;
    let response = client
        .post(url)
        .headers(headers)
        .timeout(Duration::from_millis(timeout_ms))
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    parse_json_response(response).await
}

pub async fn fetch_qr_code(
    client: &reqwest::Client,
    base_url: &str,
    bot_type: &str,
) -> Result<QrCodeResponse, String> {
    let url = format!(
        "{}ilink/bot/get_bot_qrcode?bot_type={}",
        ensure_trailing_slash(base_url),
        urlencoding::encode(bot_type)
    );
    let response = client.get(url).send().await.map_err(|e| e.to_string())?;
    parse_json_response(response).await
}

pub async fn poll_qr_status(
    client: &reqwest::Client,
    base_url: &str,
    qrcode: &str,
) -> Result<QrStatusResponse, String> {
    let url = format!(
        "{}ilink/bot/get_qrcode_status?qrcode={}",
        ensure_trailing_slash(base_url),
        urlencoding::encode(qrcode)
    );
    let mut headers = HeaderMap::new();
    headers.insert("iLink-App-ClientVersion", HeaderValue::from_static("1"));
    let response = client
        .get(url)
        .headers(headers)
        .timeout(Duration::from_millis(DEFAULT_LONG_POLL_TIMEOUT_MS))
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                "timeout".to_string()
            } else {
                e.to_string()
            }
        })?;
    parse_json_response(response).await
}

pub async fn get_updates(
    client: &reqwest::Client,
    base_url: &str,
    token: &str,
    get_updates_buf: Option<&str>,
    timeout_ms: u64,
) -> Result<GetUpdatesResp, String> {
    match post_json(
        client,
        base_url,
        "ilink/bot/getupdates",
        json!({
            "get_updates_buf": get_updates_buf.unwrap_or(""),
            "base_info": build_base_info(),
        }),
        Some(token),
        timeout_ms.max(DEFAULT_LONG_POLL_TIMEOUT_MS),
    )
    .await
    {
        Ok(resp) => Ok(resp),
        Err(error) if error.contains("operation timed out") || error == "timeout" => {
            Ok(GetUpdatesResp {
                ret: Some(0),
                msgs: Some(Vec::new()),
                get_updates_buf: get_updates_buf.map(|value| value.to_string()),
                ..GetUpdatesResp::default()
            })
        }
        Err(error) => Err(error),
    }
}

pub async fn send_message(
    client: &reqwest::Client,
    base_url: &str,
    token: &str,
    body: SendMessageReq,
) -> Result<(), String> {
    let _: serde_json::Value = post_json(
        client,
        base_url,
        "ilink/bot/sendmessage",
        serde_json::to_value(body).map_err(|e| e.to_string())?,
        Some(token),
        DEFAULT_API_TIMEOUT_MS,
    )
    .await?;
    Ok(())
}

#[allow(dead_code)]
pub async fn get_upload_url(
    client: &reqwest::Client,
    base_url: &str,
    token: &str,
    req: GetUploadUrlReq,
) -> Result<GetUploadUrlResp, String> {
    post_json(
        client,
        base_url,
        "ilink/bot/getuploadurl",
        json!({
            "filekey": req.filekey,
            "media_type": req.media_type,
            "to_user_id": req.to_user_id,
            "rawsize": req.rawsize,
            "rawfilemd5": req.rawfilemd5,
            "filesize": req.filesize,
            "no_need_thumb": req.no_need_thumb,
            "aeskey": req.aeskey,
            "base_info": build_base_info(),
        }),
        Some(token),
        DEFAULT_API_TIMEOUT_MS,
    )
    .await
}

pub async fn get_config(
    client: &reqwest::Client,
    base_url: &str,
    token: &str,
    ilink_user_id: &str,
    context_token: Option<&str>,
) -> Result<GetConfigResp, String> {
    post_json(
        client,
        base_url,
        "ilink/bot/getconfig",
        json!({
            "ilink_user_id": ilink_user_id,
            "context_token": context_token,
            "base_info": build_base_info(),
        }),
        Some(token),
        DEFAULT_CONFIG_TIMEOUT_MS,
    )
    .await
}

pub async fn send_typing(
    client: &reqwest::Client,
    base_url: &str,
    token: &str,
    req: SendTypingReq,
) -> Result<SendTypingResp, String> {
    post_json(
        client,
        base_url,
        "ilink/bot/sendtyping",
        json!({
            "ilink_user_id": req.ilink_user_id,
            "typing_ticket": req.typing_ticket,
            "status": req.status,
            "base_info": build_base_info(),
        }),
        Some(token),
        DEFAULT_CONFIG_TIMEOUT_MS,
    )
    .await
}
