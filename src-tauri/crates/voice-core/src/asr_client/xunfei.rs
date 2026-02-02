//! 讯飞语音识别客户端
//!
//! 使用讯飞开放平台的语音识别 WebSocket API (v2)。
//!
//! ## 协议说明
//!
//! 讯飞语音识别使用 WebSocket 流式传输，协议流程：
//! 1. 建立 WebSocket 连接（带鉴权参数）
//! 2. 分帧发送音频数据（每帧约 1280 字节）
//! 3. 接收识别结果（流式返回）
//! 4. 发送结束帧，等待最终结果
//!
//! ## 参考文档
//! https://www.xfyun.cn/doc/asr/voicedictation/API.html

use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use super::AsrClient;
use crate::error::{Result, VoiceError};
use crate::types::{AudioData, Segment, TranscribeResult};

/// 讯飞 WebSocket 帧大小（字节）
/// 讯飞建议每帧发送 1280 字节（约 40ms 的 16kHz 16bit 单声道音频）
const FRAME_SIZE: usize = 1280;

/// 简单的线性插值重采样
///
/// 将音频从源采样率转换到目标采样率
fn resample(samples: &[i16], from_rate: u32, to_rate: u32) -> Vec<i16> {
    if from_rate == to_rate {
        return samples.to_vec();
    }

    let ratio = from_rate as f64 / to_rate as f64;
    let new_len = (samples.len() as f64 / ratio) as usize;
    let mut result = Vec::with_capacity(new_len);

    for i in 0..new_len {
        let src_idx = i as f64 * ratio;
        let idx_floor = src_idx.floor() as usize;
        let idx_ceil = (idx_floor + 1).min(samples.len() - 1);
        let frac = src_idx - idx_floor as f64;

        // 线性插值
        let sample = if idx_floor < samples.len() {
            let s1 = samples[idx_floor] as f64;
            let s2 = samples[idx_ceil] as f64;
            (s1 + (s2 - s1) * frac) as i16
        } else {
            0
        };

        result.push(sample);
    }

    result
}

/// 讯飞客户端
pub struct XunfeiClient {
    app_id: String,
    api_key: String,
    api_secret: String,
    language: String,
}

impl XunfeiClient {
    /// 创建新的客户端
    pub fn new(app_id: String, api_key: String, api_secret: String) -> Self {
        Self {
            app_id,
            api_key,
            api_secret,
            language: "zh_cn".to_string(),
        }
    }

    /// 设置语言
    pub fn with_language(mut self, language: String) -> Self {
        self.language = language;
        self
    }

    /// 生成鉴权 URL
    ///
    /// 讯飞 WebSocket 鉴权使用 URL 参数传递，包含：
    /// - authorization: Base64 编码的鉴权信息
    /// - date: RFC1123 格式的时间戳
    /// - host: 主机名
    fn generate_auth_url(&self) -> Result<String> {
        let host = "iat-api.xfyun.cn";
        let path = "/v2/iat";
        let date = Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();

        tracing::debug!("讯飞鉴权 - date: {}", date);
        tracing::debug!("讯飞鉴权 - api_key 长度: {}", self.api_key.len());
        tracing::debug!("讯飞鉴权 - api_secret 长度: {}", self.api_secret.len());

        // 构建签名原文
        let signature_origin = format!("host: {host}\ndate: {date}\nGET {path} HTTP/1.1");
        tracing::debug!("讯飞鉴权 - signature_origin:\n{}", signature_origin);

        // HMAC-SHA256 签名
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(self.api_secret.as_bytes())
            .map_err(|e| VoiceError::AsrAuthError(e.to_string()))?;
        mac.update(signature_origin.as_bytes());
        let signature = BASE64.encode(mac.finalize().into_bytes());
        tracing::debug!("讯飞鉴权 - signature: {}", signature);

        // 构建 authorization
        let authorization_origin = format!(
            "api_key=\"{}\", algorithm=\"hmac-sha256\", headers=\"host date request-line\", signature=\"{}\"",
            self.api_key, signature
        );
        let authorization = BASE64.encode(authorization_origin.as_bytes());
        tracing::debug!("讯飞鉴权 - authorization 长度: {}", authorization.len());

        // 构建 URL
        let url = format!(
            "wss://{}{}?authorization={}&date={}&host={}",
            host,
            path,
            urlencoding::encode(&authorization),
            urlencoding::encode(&date),
            urlencoding::encode(host)
        );

        Ok(url)
    }

    /// 构建首帧请求（包含业务参数）
    fn build_first_frame(&self, audio_chunk: &[u8]) -> XunfeiRequest {
        XunfeiRequest {
            common: XunfeiCommon {
                app_id: self.app_id.clone(),
            },
            business: Some(XunfeiBusiness {
                language: self.language.clone(),
                domain: "iat".to_string(),
                accent: "mandarin".to_string(),
                vad_eos: 3000,                 // 静音检测时间（毫秒）
                dwa: Some("wpgs".to_string()), // 动态修正
                ptt: Some(1),                  // 添加标点
            }),
            data: XunfeiData {
                status: 0, // 首帧
                format: "audio/L16;rate=16000".to_string(),
                encoding: "raw".to_string(),
                audio: BASE64.encode(audio_chunk),
            },
        }
    }

    /// 构建中间帧请求
    fn build_continue_frame(&self, audio_chunk: &[u8]) -> XunfeiRequest {
        XunfeiRequest {
            common: XunfeiCommon {
                app_id: self.app_id.clone(),
            },
            business: None,
            data: XunfeiData {
                status: 1, // 中间帧
                format: "audio/L16;rate=16000".to_string(),
                encoding: "raw".to_string(),
                audio: BASE64.encode(audio_chunk),
            },
        }
    }

    /// 构建尾帧请求
    fn build_last_frame(&self, audio_chunk: &[u8]) -> XunfeiRequest {
        XunfeiRequest {
            common: XunfeiCommon {
                app_id: self.app_id.clone(),
            },
            business: None,
            data: XunfeiData {
                status: 2, // 尾帧
                format: "audio/L16;rate=16000".to_string(),
                encoding: "raw".to_string(),
                audio: BASE64.encode(audio_chunk),
            },
        }
    }

    /// 解析识别结果（支持动态修正）
    ///
    /// 动态修正说明：
    /// - pgs="apd": 追加到之前的结果
    /// - pgs="rpl": 替换之前的部分结果，替换范围由 rg 字段指定
    fn parse_result(responses: &[XunfeiResponse]) -> TranscribeResult {
        // 使用 HashMap 存储每个 sn 对应的文本，支持动态修正替换
        let mut sn_texts: std::collections::HashMap<i32, String> = std::collections::HashMap::new();
        let mut max_sn: i32 = 0;

        for resp in responses {
            if let Some(ref data) = resp.data {
                if let Some(ref result) = data.result {
                    let sn = result.sn.unwrap_or(0);
                    max_sn = max_sn.max(sn);

                    // 提取当前结果的文本
                    let mut current_text = String::new();
                    for ws in &result.ws {
                        for cw in &ws.cw {
                            current_text.push_str(&cw.w);
                        }
                    }

                    // 根据 pgs 字段处理
                    match result.pgs.as_deref() {
                        Some("rpl") => {
                            // 替换模式：删除 rg 范围内的结果，然后添加当前结果
                            if let Some(ref rg) = result.rg {
                                if rg.len() >= 2 {
                                    let start = rg[0];
                                    let end = rg[1];
                                    // 删除 [start, end] 范围内的所有 sn
                                    for i in start..=end {
                                        sn_texts.remove(&i);
                                    }
                                    tracing::debug!(
                                        "动态修正替换: sn={}, rg=[{}, {}], text={}",
                                        sn,
                                        start,
                                        end,
                                        current_text
                                    );
                                }
                            }
                            sn_texts.insert(sn, current_text);
                        }
                        Some("apd") | None => {
                            // 追加模式或无标识：直接添加
                            sn_texts.insert(sn, current_text);
                        }
                        _ => {
                            // 其他情况也直接添加
                            sn_texts.insert(sn, current_text);
                        }
                    }
                }
            }
        }

        // 按 sn 顺序拼接最终文本
        let mut full_text = String::new();
        let mut sorted_sns: Vec<i32> = sn_texts.keys().cloned().collect();
        sorted_sns.sort();

        for sn in sorted_sns {
            if let Some(text) = sn_texts.get(&sn) {
                full_text.push_str(text);
            }
        }

        let mut segments = Vec::new();
        // 如果有文本，创建一个整体的 segment
        if !full_text.is_empty() {
            segments.push(Segment {
                start: 0.0,
                end: 0.0, // 讯飞不返回时间戳
                text: full_text.clone(),
            });
        }

        TranscribeResult {
            text: full_text,
            language: Some("zh".to_string()),
            confidence: None,
            segments,
        }
    }
}

#[async_trait]
impl AsrClient for XunfeiClient {
    async fn transcribe(&self, audio: &AudioData) -> Result<TranscribeResult> {
        // 生成鉴权 URL
        let url = self.generate_auth_url()?;
        tracing::debug!("讯飞 WebSocket URL 长度: {}", url.len());

        // 建立 WebSocket 连接
        tracing::info!("正在连接讯飞 WebSocket...");
        let (ws_stream, response) = connect_async(&url).await.map_err(|e| {
            tracing::error!("讯飞 WebSocket 连接失败: {:?}", e);
            VoiceError::NetworkError(format!("WebSocket 连接失败: {e}"))
        })?;

        tracing::info!(
            "讯飞 WebSocket 连接成功，HTTP 状态: {:?}",
            response.status()
        );

        let (mut write, mut read) = ws_stream.split();

        // 如果采样率不是 16000，需要重采样
        let samples_16k = if audio.sample_rate != 16000 {
            tracing::info!(
                "重采样: {}Hz -> 16000Hz (原始样本数: {})",
                audio.sample_rate,
                audio.samples.len()
            );
            resample(&audio.samples, audio.sample_rate, 16000)
        } else {
            audio.samples.clone()
        };

        tracing::info!("重采样后样本数: {}", samples_16k.len());

        // 检查重采样后的音频数据是否有效
        let non_zero_count = samples_16k.iter().filter(|&&s| s != 0).count();
        let max_sample = samples_16k.iter().map(|&s| s.abs()).max().unwrap_or(0);
        tracing::info!(
            "重采样后音频检查: 非零样本={}/{} ({:.1}%), 最大振幅={}",
            non_zero_count,
            samples_16k.len(),
            non_zero_count as f64 / samples_16k.len().max(1) as f64 * 100.0,
            max_sample
        );

        // 将音频数据转换为字节（16-bit PCM）
        let audio_bytes: Vec<u8> = samples_16k.iter().flat_map(|s| s.to_le_bytes()).collect();

        // 分帧发送音频数据
        let chunks: Vec<&[u8]> = audio_bytes.chunks(FRAME_SIZE).collect();
        let total_chunks = chunks.len();

        tracing::info!(
            "开始发送音频数据，共 {} 帧，总大小 {} 字节",
            total_chunks,
            audio_bytes.len()
        );

        // 启动接收任务
        let receive_task = tokio::spawn(async move {
            let mut responses: Vec<XunfeiResponse> = Vec::new();

            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        tracing::debug!("收到讯飞响应: {}", text);

                        match serde_json::from_str::<XunfeiResponse>(&text) {
                            Ok(response) => {
                                // 检查是否是最后一帧
                                let is_last = response
                                    .data
                                    .as_ref()
                                    .map(|d| d.status == 2)
                                    .unwrap_or(false);

                                responses.push(response);

                                if is_last {
                                    tracing::info!("收到最终识别结果");
                                    break;
                                }
                            }
                            Err(e) => {
                                tracing::error!("解析响应失败: {}", e);
                            }
                        }
                    }
                    Ok(Message::Close(frame)) => {
                        tracing::info!("WebSocket 连接关闭: {:?}", frame);
                        break;
                    }
                    Ok(Message::Ping(_)) => {
                        tracing::debug!("收到 Ping");
                    }
                    Err(e) => {
                        tracing::error!("接收数据失败: {}", e);
                        break;
                    }
                    _ => {}
                }
            }

            responses
        });

        // 发送音频数据
        let mut send_error: Option<VoiceError> = None;

        for (i, chunk) in chunks.iter().enumerate() {
            let request = if i == 0 {
                // 首帧
                self.build_first_frame(chunk)
            } else if i == total_chunks - 1 {
                // 尾帧
                self.build_last_frame(chunk)
            } else {
                // 中间帧
                self.build_continue_frame(chunk)
            };

            let json = match serde_json::to_string(&request) {
                Ok(j) => j,
                Err(e) => {
                    send_error = Some(VoiceError::AsrError(format!("序列化请求失败: {e}")));
                    break;
                }
            };

            // 发送数据，如果失败则记录错误但继续尝试
            match write.send(Message::Text(json)).await {
                Ok(_) => {
                    if i == 0 {
                        tracing::debug!("首帧发送成功");
                    } else if i == total_chunks - 1 {
                        tracing::debug!("尾帧发送成功");
                    }
                }
                Err(e) => {
                    tracing::error!("发送第 {} 帧失败: {}", i, e);
                    send_error = Some(VoiceError::NetworkError(format!("发送数据失败: {e}")));
                    break;
                }
            }

            // 控制发送速率，避免发送过快
            // 讯飞建议发送间隔与音频时长一致，每帧 1280 字节 = 40ms 音频
            // 增加一点缓冲时间以提高稳定性
            if i < total_chunks - 1 {
                tokio::time::sleep(tokio::time::Duration::from_millis(45)).await;
            }
        }

        // 等待接收任务完成（设置超时）
        let responses =
            match tokio::time::timeout(tokio::time::Duration::from_secs(30), receive_task).await {
                Ok(Ok(responses)) => responses,
                Ok(Err(e)) => {
                    return Err(VoiceError::AsrError(format!("接收任务失败: {e}")));
                }
                Err(_) => {
                    return Err(VoiceError::AsrError("等待识别结果超时".to_string()));
                }
            };

        // 如果发送过程中有错误，但仍然收到了响应，则检查响应
        if let Some(err) = send_error {
            if responses.is_empty() {
                return Err(err);
            }
            tracing::warn!("发送过程中出现错误，但仍收到 {} 个响应", responses.len());
        }

        // 检查响应中是否有错误
        for response in &responses {
            if response.code != 0 {
                return Err(VoiceError::AsrError(format!(
                    "讯飞 ASR 错误 [{}]: {}",
                    response.code,
                    response.message.clone().unwrap_or_default()
                )));
            }
        }

        // 解析最终结果
        let result = Self::parse_result(&responses);
        tracing::info!("讯飞识别完成: {}", result.text);

        Ok(result)
    }

    fn name(&self) -> &'static str {
        "讯飞语音"
    }
}

// ============================================================================
// 讯飞 WebSocket 协议数据结构
// ============================================================================

/// 讯飞请求
#[derive(Debug, Serialize)]
struct XunfeiRequest {
    /// 公共参数
    common: XunfeiCommon,
    /// 业务参数（仅首帧需要）
    #[serde(skip_serializing_if = "Option::is_none")]
    business: Option<XunfeiBusiness>,
    /// 数据
    data: XunfeiData,
}

/// 公共参数
#[derive(Debug, Serialize)]
struct XunfeiCommon {
    /// 应用 ID
    app_id: String,
}

/// 业务参数
#[derive(Debug, Serialize)]
struct XunfeiBusiness {
    /// 语言（zh_cn: 中文，en_us: 英文）
    language: String,
    /// 领域（iat: 日常用语）
    domain: String,
    /// 方言（mandarin: 普通话）
    accent: String,
    /// 静音检测时间（毫秒）
    vad_eos: u32,
    /// 动态修正（wpgs: 开启）
    #[serde(skip_serializing_if = "Option::is_none")]
    dwa: Option<String>,
    /// 是否添加标点（1: 添加）
    #[serde(skip_serializing_if = "Option::is_none")]
    ptt: Option<u8>,
}

/// 数据参数
#[derive(Debug, Serialize)]
struct XunfeiData {
    /// 状态（0: 首帧，1: 中间帧，2: 尾帧）
    status: u8,
    /// 音频格式
    format: String,
    /// 编码方式
    encoding: String,
    /// Base64 编码的音频数据
    audio: String,
}

/// 讯飞响应
#[derive(Debug, Deserialize)]
struct XunfeiResponse {
    /// 错误码（0 表示成功）
    code: i32,
    /// 错误信息
    message: Option<String>,
    /// 会话 ID
    #[allow(dead_code)]
    sid: Option<String>,
    /// 数据
    data: Option<XunfeiResponseData>,
}

/// 响应数据
#[derive(Debug, Deserialize)]
struct XunfeiResponseData {
    /// 状态（0: 首帧，1: 中间帧，2: 尾帧）
    status: u8,
    /// 识别结果
    result: Option<XunfeiResult>,
}

/// 识别结果
#[derive(Debug, Deserialize)]
struct XunfeiResult {
    /// 返回结果的序号
    sn: Option<i32>,
    /// 词列表
    ws: Vec<XunfeiWord>,
    /// 是否是最终结果
    #[allow(dead_code)]
    ls: Option<bool>,
    /// 动态修正标识：apd=追加，rpl=替换
    pgs: Option<String>,
    /// 替换范围 [start, end]，当 pgs=rpl 时有效
    rg: Option<Vec<i32>>,
}

/// 词
#[derive(Debug, Deserialize)]
struct XunfeiWord {
    /// 候选词列表
    cw: Vec<XunfeiCandidate>,
}

/// 候选词
#[derive(Debug, Deserialize)]
struct XunfeiCandidate {
    /// 词内容
    w: String,
}
