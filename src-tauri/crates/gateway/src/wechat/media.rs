use super::api;
use super::types::{
    CdnMedia, FileItem, GetUploadUrlReq, ImageItem, MessageItem, MessageItemType, MessageState,
    MessageType, SendMessageReq, UploadMediaType, VideoItem, WechatMessage,
};
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyInit};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[allow(dead_code)]
type Aes128EcbEnc = ecb::Encryptor<aes::Aes128>;
type Aes128EcbDec = ecb::Decryptor<aes::Aes128>;

const MAX_MEDIA_BYTES: usize = 100 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WechatInboundMedia {
    pub media_type: String,
    pub file_path: String,
    pub mime_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UploadedFileInfo {
    pub download_encrypted_query_param: String,
    pub aeskey_hex: String,
    pub file_size: u64,
    pub file_size_ciphertext: u64,
    pub file_name: String,
}

pub fn resolve_channel_data_dir() -> Result<PathBuf, String> {
    let root = dirs::data_dir()
        .or_else(dirs::home_dir)
        .ok_or_else(|| "无法解析微信渠道数据目录".to_string())?;
    let path = root.join("lime").join("channels").join("wechat");
    fs::create_dir_all(&path).map_err(|e| format!("创建微信渠道目录失败: {e}"))?;
    Ok(path)
}

pub fn resolve_account_data_dir(account_id: &str) -> Result<PathBuf, String> {
    let path = resolve_channel_data_dir()?
        .join("accounts")
        .join(account_id);
    fs::create_dir_all(&path).map_err(|e| format!("创建微信账号目录失败: {e}"))?;
    Ok(path)
}

pub fn purge_account_data(account_id: &str) -> Result<(), String> {
    let path = resolve_channel_data_dir()?
        .join("accounts")
        .join(account_id);
    if path.exists() {
        fs::remove_dir_all(path).map_err(|e| format!("删除微信账号目录失败: {e}"))?;
    }
    Ok(())
}

fn build_cdn_download_url(cdn_base_url: &str, encrypted_query_param: &str) -> String {
    format!(
        "{}/download?encrypted_query_param={}",
        cdn_base_url.trim_end_matches('/'),
        urlencoding::encode(encrypted_query_param)
    )
}

#[allow(dead_code)]
fn build_cdn_upload_url(cdn_base_url: &str, upload_param: &str, filekey: &str) -> String {
    format!(
        "{}/upload?encrypted_query_param={}&filekey={}",
        cdn_base_url.trim_end_matches('/'),
        urlencoding::encode(upload_param),
        urlencoding::encode(filekey)
    )
}

fn parse_aes_key(aes_key_base64: &str) -> Result<Vec<u8>, String> {
    let decoded = BASE64_STANDARD
        .decode(aes_key_base64)
        .map_err(|e| format!("解码 aes_key 失败: {e}"))?;
    if decoded.len() == 16 {
        return Ok(decoded);
    }
    if decoded.len() == 32 && decoded.iter().all(|byte| byte.is_ascii_hexdigit()) {
        return hex::decode(decoded).map_err(|e| format!("解析 hex aes_key 失败: {e}"));
    }
    Err(format!("不支持的 aes_key 长度: {}", decoded.len()))
}

fn decrypt_aes_ecb(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = Aes128EcbDec::new_from_slice(key).map_err(|e| e.to_string())?;
    let mut buf = ciphertext.to_vec();
    cipher
        .decrypt_padded_mut::<Pkcs7>(&mut buf)
        .map(|value| value.to_vec())
        .map_err(|e| format!("AES-ECB 解密失败: {e}"))
}

#[allow(dead_code)]
fn encrypt_aes_ecb(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = Aes128EcbEnc::new_from_slice(key).map_err(|e| e.to_string())?;
    let msg_len = plaintext.len();
    let mut buf = plaintext.to_vec();
    let pad_len = 16 - (msg_len % 16);
    buf.resize(msg_len + pad_len, 0);
    cipher
        .encrypt_padded_mut::<Pkcs7>(&mut buf, msg_len)
        .map(|value| value.to_vec())
        .map_err(|e| format!("AES-ECB 加密失败: {e}"))
}

fn guess_mime_from_name(file_name: &str) -> String {
    let ext = Path::new(file_name)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "mp4" => "video/mp4",
        "mov" => "video/quicktime",
        "pdf" => "application/pdf",
        "txt" | "md" => "text/plain",
        "zip" => "application/zip",
        "json" => "application/json",
        _ => "application/octet-stream",
    }
    .to_string()
}

fn default_extension_for_mime(mime: &str) -> &'static str {
    match mime {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "video/mp4" => "mp4",
        "video/quicktime" => "mov",
        "audio/wav" => "wav",
        "audio/silk" => "silk",
        "application/pdf" => "pdf",
        "text/plain" => "txt",
        "application/json" => "json",
        "application/zip" => "zip",
        _ => "bin",
    }
}

fn build_media_file_path(
    account_id: &str,
    direction: &str,
    mime: &str,
    file_name: Option<&str>,
) -> Result<PathBuf, String> {
    let dir = resolve_account_data_dir(account_id)?
        .join("media")
        .join(direction);
    fs::create_dir_all(&dir).map_err(|e| format!("创建媒体目录失败: {e}"))?;
    let ext = file_name
        .and_then(|value| Path::new(value).extension().and_then(|ext| ext.to_str()))
        .unwrap_or_else(|| default_extension_for_mime(mime));
    Ok(dir.join(format!(
        "{}-{}.{}",
        chrono::Utc::now().timestamp_millis(),
        Uuid::new_v4(),
        ext
    )))
}

async fn download_cdn_bytes(
    client: &reqwest::Client,
    cdn_base_url: &str,
    encrypted_query_param: &str,
) -> Result<Vec<u8>, String> {
    let response = client
        .get(build_cdn_download_url(cdn_base_url, encrypted_query_param))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = response.status();
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!("CDN 下载失败: HTTP {}", status.as_u16()));
    }
    Ok(bytes.to_vec())
}

#[allow(dead_code)]
async fn upload_cdn_ciphertext(
    client: &reqwest::Client,
    cdn_base_url: &str,
    upload_param: &str,
    filekey: &str,
    ciphertext: Vec<u8>,
) -> Result<String, String> {
    let response = client
        .post(build_cdn_upload_url(cdn_base_url, upload_param, filekey))
        .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
        .body(ciphertext)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("CDN 上传失败: HTTP {}", status.as_u16()));
    }
    response
        .headers()
        .get("x-encrypted-param")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
        .ok_or_else(|| "CDN 上传响应缺少 x-encrypted-param".to_string())
}

fn media_item_type(item: &MessageItem) -> Option<MessageItemType> {
    match item.r#type {
        Some(value) if value == MessageItemType::Text as i32 => Some(MessageItemType::Text),
        Some(value) if value == MessageItemType::Image as i32 => Some(MessageItemType::Image),
        Some(value) if value == MessageItemType::Voice as i32 => Some(MessageItemType::Voice),
        Some(value) if value == MessageItemType::File as i32 => Some(MessageItemType::File),
        Some(value) if value == MessageItemType::Video as i32 => Some(MessageItemType::Video),
        _ => None,
    }
}

pub fn body_from_item_list(item_list: Option<&[MessageItem]>) -> String {
    let Some(item_list) = item_list else {
        return String::new();
    };

    for item in item_list {
        match media_item_type(item) {
            Some(MessageItemType::Text) => {
                let text = item
                    .text_item
                    .as_ref()
                    .and_then(|value| value.text.as_deref())
                    .unwrap_or("")
                    .to_string();
                if text.is_empty() {
                    continue;
                }
                if let Some(ref_msg) = item.ref_msg.as_ref() {
                    let mut parts = Vec::new();
                    if let Some(title) = ref_msg.title.as_deref() {
                        if !title.trim().is_empty() {
                            parts.push(title.trim().to_string());
                        }
                    }
                    if let Some(message_item) = ref_msg.message_item.as_deref() {
                        let body = body_from_item_list(Some(std::slice::from_ref(message_item)));
                        if !body.trim().is_empty() {
                            parts.push(body);
                        }
                    }
                    if !parts.is_empty() {
                        return format!("[引用: {}]\n{text}", parts.join(" | "));
                    }
                }
                return text;
            }
            Some(MessageItemType::Voice) => {
                if let Some(text) = item
                    .voice_item
                    .as_ref()
                    .and_then(|value| value.text.as_deref())
                    .filter(|value| !value.trim().is_empty())
                {
                    return text.to_string();
                }
            }
            _ => {}
        }
    }

    String::new()
}

pub fn find_media_item(item_list: Option<&[MessageItem]>) -> Option<MessageItem> {
    let item_list = item_list?;
    for target in [
        MessageItemType::Image,
        MessageItemType::Video,
        MessageItemType::File,
        MessageItemType::Voice,
    ] {
        if let Some(item) = item_list.iter().find(|item| {
            media_item_type(item) == Some(target)
                && match target {
                    MessageItemType::Image => item
                        .image_item
                        .as_ref()
                        .and_then(|value| value.media.as_ref())
                        .and_then(|value| value.encrypt_query_param.as_deref())
                        .is_some(),
                    MessageItemType::Video => item
                        .video_item
                        .as_ref()
                        .and_then(|value| value.media.as_ref())
                        .and_then(|value| value.encrypt_query_param.as_deref())
                        .is_some(),
                    MessageItemType::File => item
                        .file_item
                        .as_ref()
                        .and_then(|value| value.media.as_ref())
                        .and_then(|value| value.encrypt_query_param.as_deref())
                        .is_some(),
                    MessageItemType::Voice => item
                        .voice_item
                        .as_ref()
                        .and_then(|value| value.media.as_ref())
                        .and_then(|value| value.encrypt_query_param.as_deref())
                        .is_some(),
                    MessageItemType::Text => false,
                }
        }) {
            return Some(item.clone());
        }
    }
    None
}

async fn download_and_decrypt(
    client: &reqwest::Client,
    cdn_base_url: &str,
    media: &CdnMedia,
    aes_key_override_hex: Option<&str>,
) -> Result<Vec<u8>, String> {
    let encrypted_query_param = media
        .encrypt_query_param
        .as_deref()
        .ok_or_else(|| "缺少 encrypt_query_param".to_string())?;
    let ciphertext = download_cdn_bytes(client, cdn_base_url, encrypted_query_param).await?;
    let key = if let Some(hex_key) = aes_key_override_hex.filter(|value| !value.trim().is_empty()) {
        hex::decode(hex_key).map_err(|e| format!("解析 hex aeskey 失败: {e}"))?
    } else {
        let aes_key = media
            .aes_key
            .as_deref()
            .ok_or_else(|| "缺少 aes_key".to_string())?;
        parse_aes_key(aes_key)?
    };
    decrypt_aes_ecb(&ciphertext, &key)
}

pub async fn download_media_from_item(
    client: &reqwest::Client,
    account_id: &str,
    cdn_base_url: &str,
    item: &MessageItem,
) -> Result<Option<WechatInboundMedia>, String> {
    let media_type = media_item_type(item);
    let Some(media_type) = media_type else {
        return Ok(None);
    };

    match media_type {
        MessageItemType::Image => {
            let image = item
                .image_item
                .as_ref()
                .ok_or_else(|| "缺少 image_item".to_string())?;
            let media = image
                .media
                .as_ref()
                .ok_or_else(|| "缺少 image media".to_string())?;
            let plaintext = if image
                .aeskey
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .is_some()
            {
                download_and_decrypt(client, cdn_base_url, media, image.aeskey.as_deref()).await?
            } else {
                let encrypted_query_param = media
                    .encrypt_query_param
                    .as_deref()
                    .ok_or_else(|| "缺少 encrypt_query_param".to_string())?;
                download_cdn_bytes(client, cdn_base_url, encrypted_query_param).await?
            };
            if plaintext.len() > MAX_MEDIA_BYTES {
                return Err("图片体积超过限制".to_string());
            }
            let path = build_media_file_path(account_id, "inbound", "image/jpeg", None)?;
            fs::write(&path, plaintext).map_err(|e| format!("写入图片失败: {e}"))?;
            Ok(Some(WechatInboundMedia {
                media_type: "image".to_string(),
                file_path: path.display().to_string(),
                mime_type: "image/jpeg".to_string(),
                file_name: None,
            }))
        }
        MessageItemType::Video => {
            let video = item
                .video_item
                .as_ref()
                .ok_or_else(|| "缺少 video_item".to_string())?;
            let media = video
                .media
                .as_ref()
                .ok_or_else(|| "缺少 video media".to_string())?;
            let plaintext = download_and_decrypt(client, cdn_base_url, media, None).await?;
            if plaintext.len() > MAX_MEDIA_BYTES {
                return Err("视频体积超过限制".to_string());
            }
            let path = build_media_file_path(account_id, "inbound", "video/mp4", None)?;
            fs::write(&path, plaintext).map_err(|e| format!("写入视频失败: {e}"))?;
            Ok(Some(WechatInboundMedia {
                media_type: "video".to_string(),
                file_path: path.display().to_string(),
                mime_type: "video/mp4".to_string(),
                file_name: None,
            }))
        }
        MessageItemType::File => {
            let file = item
                .file_item
                .as_ref()
                .ok_or_else(|| "缺少 file_item".to_string())?;
            let media = file
                .media
                .as_ref()
                .ok_or_else(|| "缺少 file media".to_string())?;
            let plaintext = download_and_decrypt(client, cdn_base_url, media, None).await?;
            if plaintext.len() > MAX_MEDIA_BYTES {
                return Err("文件体积超过限制".to_string());
            }
            let file_name = file
                .file_name
                .clone()
                .unwrap_or_else(|| "attachment.bin".to_string());
            let mime = guess_mime_from_name(&file_name);
            let path = build_media_file_path(account_id, "inbound", &mime, Some(&file_name))?;
            fs::write(&path, plaintext).map_err(|e| format!("写入文件失败: {e}"))?;
            Ok(Some(WechatInboundMedia {
                media_type: "file".to_string(),
                file_path: path.display().to_string(),
                mime_type: mime,
                file_name: Some(file_name),
            }))
        }
        MessageItemType::Voice => {
            let voice = item
                .voice_item
                .as_ref()
                .ok_or_else(|| "缺少 voice_item".to_string())?;
            let media = voice
                .media
                .as_ref()
                .ok_or_else(|| "缺少 voice media".to_string())?;
            let plaintext = download_and_decrypt(client, cdn_base_url, media, None).await?;
            if plaintext.len() > MAX_MEDIA_BYTES {
                return Err("语音体积超过限制".to_string());
            }
            let path =
                build_media_file_path(account_id, "inbound", "audio/silk", Some("voice.silk"))?;
            fs::write(&path, plaintext).map_err(|e| format!("写入语音失败: {e}"))?;
            Ok(Some(WechatInboundMedia {
                media_type: "audio".to_string(),
                file_path: path.display().to_string(),
                mime_type: "audio/silk".to_string(),
                file_name: Some("voice.silk".to_string()),
            }))
        }
        MessageItemType::Text => Ok(None),
    }
}

fn build_text_message(to_user_id: &str, text: &str, context_token: Option<&str>) -> SendMessageReq {
    SendMessageReq {
        msg: WechatMessage {
            from_user_id: Some(String::new()),
            to_user_id: Some(to_user_id.to_string()),
            client_id: Some(Uuid::new_v4().to_string()),
            message_type: Some(MessageType::Bot as i32),
            message_state: Some(MessageState::Finish as i32),
            item_list: Some(vec![MessageItem {
                r#type: Some(MessageItemType::Text as i32),
                text_item: Some(super::types::TextItem {
                    text: Some(text.to_string()),
                }),
                ..MessageItem::default()
            }]),
            context_token: context_token.map(|value| value.to_string()),
            ..WechatMessage::default()
        },
    }
}

pub async fn send_text_message(
    client: &reqwest::Client,
    base_url: &str,
    token: &str,
    to_user_id: &str,
    text: &str,
    context_token: Option<&str>,
) -> Result<(), String> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(());
    }
    api::send_message(
        client,
        base_url,
        token,
        build_text_message(to_user_id, text, context_token),
    )
    .await
}

#[allow(dead_code)]
pub async fn upload_media_file(
    client: &reqwest::Client,
    base_url: &str,
    token: &str,
    cdn_base_url: &str,
    to_user_id: &str,
    file_path: &Path,
    media_type: UploadMediaType,
) -> Result<UploadedFileInfo, String> {
    let plaintext = fs::read(file_path).map_err(|e| format!("读取待上传文件失败: {e}"))?;
    let rawsize = plaintext.len() as u64;
    let rawfilemd5 = format!("{:x}", md5::compute(&plaintext));
    let file_size_ciphertext = (((rawsize + 1) + 15) / 16) * 16;
    let filekey = hex::encode(rand::random::<[u8; 16]>());
    let aeskey = rand::random::<[u8; 16]>();
    let upload_url_resp = api::get_upload_url(
        client,
        base_url,
        token,
        GetUploadUrlReq {
            filekey: filekey.clone(),
            media_type: media_type as i32,
            to_user_id: to_user_id.to_string(),
            rawsize,
            rawfilemd5,
            filesize: file_size_ciphertext,
            no_need_thumb: true,
            aeskey: hex::encode(aeskey),
        },
    )
    .await?;
    let upload_param = upload_url_resp
        .upload_param
        .ok_or_else(|| "get_upload_url 未返回 upload_param".to_string())?;
    let ciphertext = encrypt_aes_ecb(&plaintext, &aeskey)?;
    let download_encrypted_query_param =
        upload_cdn_ciphertext(client, cdn_base_url, &upload_param, &filekey, ciphertext).await?;
    let file_name = file_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("attachment.bin")
        .to_string();
    Ok(UploadedFileInfo {
        download_encrypted_query_param,
        aeskey_hex: hex::encode(aeskey),
        file_size: rawsize,
        file_size_ciphertext,
        file_name,
    })
}

#[allow(dead_code)]
pub async fn send_media_message(
    client: &reqwest::Client,
    base_url: &str,
    token: &str,
    cdn_base_url: &str,
    to_user_id: &str,
    file_path: &Path,
    text: Option<&str>,
    context_token: Option<&str>,
) -> Result<(), String> {
    let mime = guess_mime_from_name(file_path.to_string_lossy().as_ref());
    let media_type = if mime.starts_with("image/") {
        UploadMediaType::Image
    } else if mime.starts_with("video/") {
        UploadMediaType::Video
    } else {
        UploadMediaType::File
    };
    let uploaded = upload_media_file(
        client,
        base_url,
        token,
        cdn_base_url,
        to_user_id,
        file_path,
        media_type,
    )
    .await?;

    let media_item = match media_type {
        UploadMediaType::Image => MessageItem {
            r#type: Some(MessageItemType::Image as i32),
            image_item: Some(ImageItem {
                media: Some(CdnMedia {
                    encrypt_query_param: Some(uploaded.download_encrypted_query_param),
                    aes_key: Some(
                        BASE64_STANDARD
                            .encode(hex::decode(uploaded.aeskey_hex).map_err(|e| e.to_string())?),
                    ),
                    encrypt_type: Some(1),
                }),
                mid_size: Some(uploaded.file_size_ciphertext),
                ..ImageItem::default()
            }),
            ..MessageItem::default()
        },
        UploadMediaType::Video => MessageItem {
            r#type: Some(MessageItemType::Video as i32),
            video_item: Some(VideoItem {
                media: Some(CdnMedia {
                    encrypt_query_param: Some(uploaded.download_encrypted_query_param),
                    aes_key: Some(
                        BASE64_STANDARD
                            .encode(hex::decode(uploaded.aeskey_hex).map_err(|e| e.to_string())?),
                    ),
                    encrypt_type: Some(1),
                }),
                video_size: Some(uploaded.file_size_ciphertext),
            }),
            ..MessageItem::default()
        },
        UploadMediaType::File | UploadMediaType::Voice => MessageItem {
            r#type: Some(MessageItemType::File as i32),
            file_item: Some(FileItem {
                media: Some(CdnMedia {
                    encrypt_query_param: Some(uploaded.download_encrypted_query_param),
                    aes_key: Some(
                        BASE64_STANDARD
                            .encode(hex::decode(uploaded.aeskey_hex).map_err(|e| e.to_string())?),
                    ),
                    encrypt_type: Some(1),
                }),
                file_name: Some(uploaded.file_name.clone()),
            }),
            ..MessageItem::default()
        },
    };

    if let Some(text) = text.filter(|value| !value.trim().is_empty()) {
        send_text_message(client, base_url, token, to_user_id, text, context_token).await?;
    }
    api::send_message(
        client,
        base_url,
        token,
        SendMessageReq {
            msg: WechatMessage {
                from_user_id: Some(String::new()),
                to_user_id: Some(to_user_id.to_string()),
                client_id: Some(Uuid::new_v4().to_string()),
                message_type: Some(MessageType::Bot as i32),
                message_state: Some(MessageState::Finish as i32),
                item_list: Some(vec![media_item]),
                context_token: context_token.map(|value| value.to_string()),
                ..WechatMessage::default()
            },
        },
    )
    .await
}
