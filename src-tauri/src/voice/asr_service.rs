//! ASR 服务
//!
//! 统一管理语音识别服务，支持本地 Whisper 和云端 ASR。
//!
//! ## 功能
//! - 本地 Whisper 识别（离线、隐私）
//! - OpenAI Whisper API
//! - 百度语音识别
//! - 讯飞语音识别（WebSocket 流式）
//!
//! ## 模型文件路径
//! Whisper 模型文件存储在：`~/Library/Application Support/proxycast/models/whisper/`
//!
//! 支持的模型：
//! - `ggml-tiny.bin` (~75MB)
//! - `ggml-base.bin` (~142MB)
//! - `ggml-small.bin` (~466MB)
//! - `ggml-medium.bin` (~1.5GB)
//!
//! ## 使用示例
//! ```rust,ignore
//! let credential = AsrService::get_default_credential()?.unwrap();
//! let text = AsrService::transcribe(&credential, &audio_data, 16000).await?;
//! ```

use std::path::PathBuf;

use crate::config::{load_config, AsrCredentialEntry, AsrProviderType, WhisperModelSize};

/// ASR 服务
pub struct AsrService;

impl AsrService {
    /// 获取默认 ASR 凭证
    pub fn get_default_credential() -> Result<Option<AsrCredentialEntry>, String> {
        let config = load_config().map_err(|e| e.to_string())?;
        Ok(config
            .credential_pool
            .asr
            .into_iter()
            .find(|c| c.is_default && !c.disabled))
    }

    /// 获取指定 ID 的 ASR 凭证
    pub fn get_credential(id: &str) -> Result<Option<AsrCredentialEntry>, String> {
        let config = load_config().map_err(|e| e.to_string())?;
        Ok(config.credential_pool.asr.into_iter().find(|c| c.id == id))
    }

    /// 使用指定凭证进行语音识别
    ///
    /// 当云端服务失败时，自动回退到本地 Whisper（需求 3.4）
    pub async fn transcribe(
        credential: &AsrCredentialEntry,
        audio_data: &[u8],
        sample_rate: u32,
    ) -> Result<String, String> {
        // 如果是本地 Whisper，直接调用
        if matches!(credential.provider, AsrProviderType::WhisperLocal) {
            return Self::transcribe_whisper_local(credential, audio_data, sample_rate).await;
        }

        // 云端服务：先尝试云端，失败则回退到本地 Whisper
        let cloud_result = match credential.provider {
            AsrProviderType::OpenAI => {
                Self::transcribe_openai(credential, audio_data, sample_rate).await
            }
            AsrProviderType::Baidu => {
                Self::transcribe_baidu(credential, audio_data, sample_rate).await
            }
            AsrProviderType::Xunfei => {
                Self::transcribe_xunfei(credential, audio_data, sample_rate).await
            }
            AsrProviderType::WhisperLocal => unreachable!(), // 已在上面处理
        };

        // 云端成功，直接返回
        if cloud_result.is_ok() {
            return cloud_result;
        }

        // 云端失败，尝试回退到本地 Whisper
        let cloud_error = cloud_result.unwrap_err();
        tracing::warn!(
            "云端 ASR 服务 ({:?}) 失败: {}，尝试回退到本地 Whisper",
            credential.provider,
            cloud_error
        );

        // 尝试获取本地 Whisper 凭证
        match Self::get_whisper_local_credential() {
            Ok(Some(whisper_credential)) => {
                tracing::info!("正在使用本地 Whisper 进行回退识别...");
                match Self::transcribe_whisper_local(&whisper_credential, audio_data, sample_rate)
                    .await
                {
                    Ok(text) => {
                        tracing::info!("本地 Whisper 回退识别成功");
                        Ok(text)
                    }
                    Err(whisper_error) => {
                        tracing::error!("本地 Whisper 回退也失败: {}", whisper_error);
                        // 返回原始云端错误，因为那是用户选择的服务
                        Err(format!(
                            "云端服务失败: {cloud_error}；本地 Whisper 回退也失败: {whisper_error}"
                        ))
                    }
                }
            }
            Ok(None) => {
                tracing::warn!("未找到本地 Whisper 凭证，无法回退");
                Err(format!(
                    "云端服务失败: {cloud_error}；未配置本地 Whisper，无法回退"
                ))
            }
            Err(e) => {
                tracing::error!("获取本地 Whisper 凭证失败: {}", e);
                Err(format!(
                    "云端服务失败: {cloud_error}；获取本地 Whisper 凭证失败: {e}"
                ))
            }
        }
    }

    /// 获取本地 Whisper 凭证（用于回退）
    fn get_whisper_local_credential() -> Result<Option<AsrCredentialEntry>, String> {
        let config = load_config().map_err(|e| e.to_string())?;
        Ok(config
            .credential_pool
            .asr
            .into_iter()
            .find(|c| matches!(c.provider, AsrProviderType::WhisperLocal) && !c.disabled))
    }

    /// 本地 Whisper 识别
    async fn transcribe_whisper_local(
        credential: &AsrCredentialEntry,
        audio_data: &[u8],
        sample_rate: u32,
    ) -> Result<String, String> {
        // 获取 Whisper 配置
        let whisper_config = credential
            .whisper_config
            .as_ref()
            .ok_or("Whisper 本地配置缺失")?;

        // 获取模型文件路径
        let model_path = Self::get_whisper_model_path(&whisper_config.model)?;

        // 将 PCM 字节转换为 i16 采样
        let samples: Vec<i16> = audio_data
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        // 检查音频数据是否有效
        if samples.is_empty() {
            return Err("音频数据为空".to_string());
        }

        // 创建 AudioData
        let audio = voice_core::types::AudioData::new(samples, sample_rate, 1);

        // 检查录音时长
        if !audio.is_valid() {
            return Err("录音时间过短（需要至少 0.5 秒）".to_string());
        }

        // 转换模型大小枚举
        let model = Self::convert_model_size(&whisper_config.model);

        // 创建 Whisper 识别器
        let transcriber =
            voice_core::WhisperTranscriber::new(model_path, model, &credential.language)
                .map_err(|e| format!("Whisper 模型加载失败: {e}"))?;

        // 执行识别
        let result = transcriber
            .transcribe(&audio)
            .map_err(|e| format!("Whisper 识别失败: {e}"))?;

        Ok(result.text)
    }

    /// 获取 Whisper 模型文件路径
    fn get_whisper_model_path(model_size: &WhisperModelSize) -> Result<PathBuf, String> {
        // 模型文件名
        let filename = match model_size {
            WhisperModelSize::Tiny => "ggml-tiny.bin",
            WhisperModelSize::Base => "ggml-base.bin",
            WhisperModelSize::Small => "ggml-small.bin",
            WhisperModelSize::Medium => "ggml-medium.bin",
        };

        // 模型存储目录：~/Library/Application Support/proxycast/models/whisper/
        let models_dir = dirs::data_dir()
            .ok_or("无法获取数据目录")?
            .join("proxycast")
            .join("models")
            .join("whisper");

        let model_path = models_dir.join(filename);

        // 检查模型文件是否存在
        if !model_path.exists() {
            return Err(format!(
                "Whisper 模型文件不存在: {}\n请下载模型文件到: {}",
                filename,
                models_dir.display()
            ));
        }

        Ok(model_path)
    }

    /// 转换模型大小枚举
    fn convert_model_size(size: &WhisperModelSize) -> voice_core::types::WhisperModel {
        match size {
            WhisperModelSize::Tiny => voice_core::types::WhisperModel::Tiny,
            WhisperModelSize::Base => voice_core::types::WhisperModel::Base,
            WhisperModelSize::Small => voice_core::types::WhisperModel::Small,
            WhisperModelSize::Medium => voice_core::types::WhisperModel::Medium,
        }
    }

    /// OpenAI Whisper API 识别
    ///
    /// 使用手动构建 multipart/form-data 请求
    async fn transcribe_openai(
        credential: &AsrCredentialEntry,
        audio_data: &[u8],
        sample_rate: u32,
    ) -> Result<String, String> {
        let config = credential.openai_config.as_ref().ok_or("OpenAI 配置缺失")?;

        // 构建 WAV 文件
        let wav_data = Self::build_wav(audio_data, sample_rate, 1)?;

        // 构建 multipart/form-data 请求体
        let boundary = format!("----WebKitFormBoundary{}", uuid::Uuid::new_v4().simple());
        let mut body = Vec::new();

        // 添加 file 字段
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            b"Content-Disposition: form-data; name=\"file\"; filename=\"audio.wav\"\r\n",
        );
        body.extend_from_slice(b"Content-Type: audio/wav\r\n\r\n");
        body.extend_from_slice(&wav_data);
        body.extend_from_slice(b"\r\n");

        // 添加 model 字段
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(b"Content-Disposition: form-data; name=\"model\"\r\n\r\n");
        body.extend_from_slice(b"whisper-1\r\n");

        // 添加 language 字段
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(b"Content-Disposition: form-data; name=\"language\"\r\n\r\n");
        body.extend_from_slice(credential.language.as_bytes());
        body.extend_from_slice(b"\r\n");

        // 结束边界
        body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());

        // 构建请求
        let base_url = config
            .base_url
            .as_deref()
            .unwrap_or("https://api.openai.com");
        let url = format!("{base_url}/v1/audio/transcriptions");

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header(
                "Content-Type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(body)
            .send()
            .await
            .map_err(|e| format!("请求失败: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("OpenAI API 错误: {status} - {body}"));
        }

        #[derive(serde::Deserialize)]
        struct WhisperResponse {
            text: String,
        }

        let result: WhisperResponse = response
            .json()
            .await
            .map_err(|e| format!("解析响应失败: {e}"))?;

        Ok(result.text)
    }

    /// 百度语音识别
    async fn transcribe_baidu(
        credential: &AsrCredentialEntry,
        audio_data: &[u8],
        sample_rate: u32,
    ) -> Result<String, String> {
        let config = credential.baidu_config.as_ref().ok_or("百度配置缺失")?;

        // 获取 Access Token
        let token_url = format!(
            "https://aip.baidubce.com/oauth/2.0/token?grant_type=client_credentials&client_id={}&client_secret={}",
            config.api_key, config.secret_key
        );

        let client = reqwest::Client::new();
        let token_resp = client
            .post(&token_url)
            .send()
            .await
            .map_err(|e| format!("获取 Token 失败: {e}"))?;

        #[derive(serde::Deserialize)]
        struct TokenResponse {
            access_token: String,
        }

        let token: TokenResponse = token_resp
            .json()
            .await
            .map_err(|e| format!("解析 Token 失败: {e}"))?;

        // 构建 WAV 并 Base64 编码
        let wav_data = Self::build_wav(audio_data, sample_rate, 1)?;
        let speech = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &wav_data);

        #[derive(serde::Serialize)]
        struct AsrRequest {
            format: String,
            rate: u32,
            channel: u16,
            cuid: String,
            token: String,
            speech: String,
            len: usize,
        }

        let request = AsrRequest {
            format: "wav".to_string(),
            rate: sample_rate,
            channel: 1,
            cuid: "proxycast".to_string(),
            token: token.access_token,
            speech,
            len: wav_data.len(),
        };

        let response = client
            .post("https://vop.baidu.com/server_api")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("请求失败: {e}"))?;

        #[derive(serde::Deserialize)]
        struct AsrResponse {
            err_no: i32,
            err_msg: String,
            #[serde(default)]
            result: Vec<String>,
        }

        let result: AsrResponse = response
            .json()
            .await
            .map_err(|e| format!("解析响应失败: {e}"))?;

        if result.err_no != 0 {
            return Err(format!(
                "百度 ASR 错误: {} - {}",
                result.err_no, result.err_msg
            ));
        }

        Ok(result.result.join(""))
    }

    /// 讯飞语音识别
    ///
    /// 使用 WebSocket 流式识别，支持实时语音转文字
    async fn transcribe_xunfei(
        credential: &AsrCredentialEntry,
        audio_data: &[u8],
        sample_rate: u32,
    ) -> Result<String, String> {
        let config = credential.xunfei_config.as_ref().ok_or("讯飞配置缺失")?;

        // 将 PCM 字节转换为 i16 采样
        let samples: Vec<i16> = audio_data
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        // 创建 AudioData
        let audio = voice_core::types::AudioData::new(samples, sample_rate, 1);

        // 创建讯飞客户端
        // 讯飞语言代码转换：zh -> zh_cn, en -> en_us
        let xunfei_language = match credential.language.as_str() {
            "zh" => "zh_cn".to_string(),
            "en" => "en_us".to_string(),
            other => other.to_string(),
        };

        let client = voice_core::asr_client::XunfeiClient::new(
            config.app_id.clone(),
            config.api_key.clone(),
            config.api_secret.clone(),
        )
        .with_language(xunfei_language);

        // 调用识别
        use voice_core::asr_client::AsrClient;
        let result = client
            .transcribe(&audio)
            .await
            .map_err(|e| format!("讯飞识别失败: {e}"))?;

        Ok(result.text)
    }

    /// 构建 WAV 文件
    fn build_wav(pcm_data: &[u8], sample_rate: u32, channels: u16) -> Result<Vec<u8>, String> {
        let bits_per_sample: u16 = 16;
        let byte_rate = sample_rate * u32::from(channels) * u32::from(bits_per_sample) / 8;
        let block_align = channels * bits_per_sample / 8;
        let data_size = pcm_data.len() as u32;
        let file_size = 36 + data_size;

        let mut wav = Vec::with_capacity(44 + pcm_data.len());

        // RIFF header
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&file_size.to_le_bytes());
        wav.extend_from_slice(b"WAVE");

        // fmt chunk
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes()); // chunk size
        wav.extend_from_slice(&1u16.to_le_bytes()); // PCM format
        wav.extend_from_slice(&channels.to_le_bytes());
        wav.extend_from_slice(&sample_rate.to_le_bytes());
        wav.extend_from_slice(&byte_rate.to_le_bytes());
        wav.extend_from_slice(&block_align.to_le_bytes());
        wav.extend_from_slice(&bits_per_sample.to_le_bytes());

        // data chunk
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data_size.to_le_bytes());
        wav.extend_from_slice(pcm_data);

        Ok(wav)
    }
}
