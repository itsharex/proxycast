//! OpenAI Whisper API 客户端
//!
//! 使用 OpenAI 的 Whisper API 进行语音识别。

use async_trait::async_trait;
use reqwest::multipart::{Form, Part};
use serde::Deserialize;

use super::AsrClient;
use crate::error::{Result, VoiceError};
use crate::types::{AudioData, TranscribeResult};

/// OpenAI Whisper 响应
#[derive(Debug, Deserialize)]
struct WhisperResponse {
    text: String,
    #[serde(default)]
    language: Option<String>,
}

/// OpenAI Whisper 客户端
pub struct OpenAIWhisperClient {
    api_key: String,
    api_host: String,
    model: String,
    language: Option<String>,
}

impl OpenAIWhisperClient {
    /// 创建新的客户端
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            api_host: "https://api.openai.com".to_string(),
            model: "whisper-1".to_string(),
            language: None,
        }
    }

    /// 设置 API Host（用于代理）
    pub fn with_host(mut self, host: String) -> Self {
        self.api_host = host;
        self
    }

    /// 设置语言
    pub fn with_language(mut self, language: String) -> Self {
        self.language = Some(language);
        self
    }
}

#[async_trait]
impl AsrClient for OpenAIWhisperClient {
    async fn transcribe(&self, audio: &AudioData) -> Result<TranscribeResult> {
        let url = format!("{}/v1/audio/transcriptions", self.api_host);
        let wav_bytes = audio.to_wav_bytes();

        // 构建 multipart form
        let file_part = Part::bytes(wav_bytes)
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|e| VoiceError::AsrError(e.to_string()))?;

        let mut form = Form::new()
            .part("file", file_part)
            .text("model", self.model.clone());

        if let Some(ref lang) = self.language {
            form = form.text("language", lang.clone());
        }

        // 发送请求
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await
            .map_err(|e| VoiceError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(VoiceError::AsrError(format!(
                "OpenAI API 错误: {status} - {body}"
            )));
        }

        let result: WhisperResponse = response
            .json()
            .await
            .map_err(|e| VoiceError::AsrError(e.to_string()))?;

        Ok(TranscribeResult {
            text: result.text,
            language: result.language,
            confidence: None,
            segments: vec![],
        })
    }

    fn name(&self) -> &'static str {
        "OpenAI Whisper"
    }
}
