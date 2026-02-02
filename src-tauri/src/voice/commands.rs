//! 语音输入 Tauri 命令
//!
//! 提供前端调用的语音输入相关命令

use crate::config::{VoiceInputConfig, VoiceInstruction};
use tauri::{command, AppHandle};

use super::config;
use super::recording_service::AudioDeviceInfo;

/// 获取所有可用的麦克风设备
#[command]
pub async fn list_audio_devices() -> Result<Vec<AudioDeviceInfo>, String> {
    super::recording_service::list_audio_devices()
}

/// 获取语音输入配置
#[command]
pub async fn get_voice_input_config() -> Result<VoiceInputConfig, String> {
    config::load_voice_config()
}

/// 保存语音输入配置
#[command]
pub async fn save_voice_input_config(
    app: AppHandle,
    voice_config: VoiceInputConfig,
) -> Result<(), String> {
    let old_config = config::load_voice_config()?;

    // 如果快捷键变化，更新注册
    if old_config.shortcut != voice_config.shortcut {
        super::shortcut::update(&app, &voice_config.shortcut)?;
    }

    // 如果启用状态变化
    if old_config.enabled != voice_config.enabled {
        if voice_config.enabled {
            super::shortcut::register(&app, &voice_config.shortcut)?;
        } else {
            super::shortcut::unregister(&app)?;
        }
    }

    config::save_voice_config(voice_config)
}

/// 获取指令列表
#[command]
pub async fn get_voice_instructions() -> Result<Vec<VoiceInstruction>, String> {
    config::get_instructions()
}

/// 保存指令
#[command]
pub async fn save_voice_instruction(instruction: VoiceInstruction) -> Result<(), String> {
    let mut voice_config = config::load_voice_config()?;

    // 查找是否已存在
    if let Some(idx) = voice_config
        .instructions
        .iter()
        .position(|i| i.id == instruction.id)
    {
        voice_config.instructions[idx] = instruction;
    } else {
        voice_config.instructions.push(instruction);
    }

    config::save_voice_config(voice_config)
}

/// 删除指令
#[command]
pub async fn delete_voice_instruction(id: String) -> Result<(), String> {
    let mut voice_config = config::load_voice_config()?;

    // 检查是否为预设指令
    if let Some(instruction) = voice_config.instructions.iter().find(|i| i.id == id) {
        if instruction.is_preset {
            return Err("无法删除预设指令".to_string());
        }
    }

    voice_config.instructions.retain(|i| i.id != id);
    config::save_voice_config(voice_config)
}

/// 打开语音输入窗口
#[command]
pub async fn open_voice_window(app: AppHandle) -> Result<(), String> {
    super::window::open_voice_window(&app)
}

/// 关闭语音输入窗口
#[command]
pub async fn close_voice_window(app: AppHandle) -> Result<(), String> {
    super::window::close_voice_window(&app)
}

/// 语音识别结果
#[derive(serde::Serialize)]
pub struct TranscribeResult {
    /// 识别文本
    pub text: String,
    /// 使用的 ASR 服务
    pub provider: String,
}

/// 执行语音识别
#[command]
pub async fn transcribe_audio(
    audio_data: Vec<u8>,
    sample_rate: u32,
    credential_id: Option<String>,
) -> Result<TranscribeResult, String> {
    use super::asr_service::AsrService;

    tracing::info!(
        "[语音识别] 开始识别，音频大小: {} 字节，采样率: {}",
        audio_data.len(),
        sample_rate
    );

    // 检查音频数据是否有效
    if audio_data.is_empty() {
        tracing::error!("[语音识别] 音频数据为空！");
        return Err("音频数据为空，请检查麦克风权限".to_string());
    }

    // 检查音频数据是否全为静音（全零）
    let non_zero_count = audio_data.iter().filter(|&&b| b != 0).count();
    let non_zero_ratio = non_zero_count as f32 / audio_data.len() as f32;
    tracing::info!(
        "[语音识别] 非零字节比例: {:.2}% ({}/{})",
        non_zero_ratio * 100.0,
        non_zero_count,
        audio_data.len()
    );

    if non_zero_ratio < 0.01 {
        tracing::warn!("[语音识别] 音频数据几乎全为静音，可能是麦克风权限问题或未正确录音");
    }

    // 获取凭证
    let credential = if let Some(id) = credential_id {
        tracing::info!("[语音识别] 使用指定凭证: {}", id);
        AsrService::get_credential(&id)?.ok_or_else(|| format!("凭证不存在: {id}"))?
    } else {
        tracing::info!("[语音识别] 获取默认凭证...");
        match AsrService::get_default_credential() {
            Ok(Some(cred)) => {
                tracing::info!(
                    "[语音识别] 找到默认凭证: id={}, provider={:?}",
                    cred.id,
                    cred.provider
                );
                cred
            }
            Ok(None) => {
                // 打印所有 ASR 凭证用于调试
                if let Ok(config) = crate::config::load_config() {
                    tracing::error!(
                        "[语音识别] 未找到默认凭证，当前 ASR 凭证数量: {}",
                        config.credential_pool.asr.len()
                    );
                    for (i, c) in config.credential_pool.asr.iter().enumerate() {
                        tracing::error!(
                            "[语音识别] 凭证 {}: id={}, is_default={}, disabled={}",
                            i,
                            c.id,
                            c.is_default,
                            c.disabled
                        );
                    }
                }
                return Err("未配置语音识别服务。请在设置 → 凭证池 → ASR 中添加讯飞、百度或 OpenAI Whisper 凭证。".to_string());
            }
            Err(e) => {
                tracing::error!("[语音识别] 获取默认凭证失败: {}", e);
                return Err(format!("获取凭证失败: {e}"));
            }
        }
    };

    let provider_name = match credential.provider {
        crate::config::AsrProviderType::WhisperLocal => "本地 Whisper",
        crate::config::AsrProviderType::OpenAI => "OpenAI Whisper",
        crate::config::AsrProviderType::Baidu => "百度语音",
        crate::config::AsrProviderType::Xunfei => "讯飞语音",
    };
    tracing::info!("[语音识别] 使用服务: {}", provider_name);

    // 执行识别
    let text = AsrService::transcribe(&credential, &audio_data, sample_rate).await?;
    tracing::info!("[语音识别] 识别完成，文本长度: {} 字符", text.len());

    Ok(TranscribeResult {
        text,
        provider: provider_name.to_string(),
    })
}

/// 润色文本结果
#[derive(serde::Serialize)]
pub struct PolishResult {
    /// 润色后的文本
    pub text: String,
    /// 使用的指令
    pub instruction_name: String,
}

/// 润色文本
#[command]
pub async fn polish_voice_text(
    text: String,
    instruction_id: Option<String>,
) -> Result<PolishResult, String> {
    let voice_config = config::load_voice_config()?;

    // 获取指令
    let instruction_id =
        instruction_id.unwrap_or_else(|| voice_config.processor.default_instruction_id.clone());

    let instruction = voice_config
        .instructions
        .iter()
        .find(|i| i.id == instruction_id)
        .ok_or_else(|| format!("指令不存在: {instruction_id}"))?;

    // 如果是原始输出，直接返回
    if instruction_id == "raw" {
        return Ok(PolishResult {
            text,
            instruction_name: instruction.name.clone(),
        });
    }

    // 调用 LLM 润色
    let polished = super::processor::polish_text(
        &text,
        instruction,
        voice_config.processor.polish_provider.as_deref(),
        voice_config.processor.polish_model.as_deref(),
    )
    .await?;

    Ok(PolishResult {
        text: polished,
        instruction_name: instruction.name.clone(),
    })
}

/// 输出文本到系统
///
/// 根据配置的输出模式，将文字输出到当前焦点应用
#[command]
pub async fn output_voice_text(text: String, mode: Option<String>) -> Result<(), String> {
    use crate::config::VoiceOutputMode;

    // 解析输出模式
    let output_mode = match mode.as_deref() {
        Some("type") => VoiceOutputMode::Type,
        Some("clipboard") => VoiceOutputMode::Clipboard,
        Some("both") => VoiceOutputMode::Both,
        None => {
            // 使用配置的默认模式
            let config = config::load_voice_config()?;
            config.output.mode
        }
        Some(other) => return Err(format!("未知的输出模式: {other}")),
    };

    // 执行输出
    super::output_service::output_text(&text, output_mode)?;

    tracing::info!("[语音输出] 文本已输出: {} 字符", text.chars().count());
    Ok(())
}

// ============ 录音控制命令 ============
// 使用独立线程 + channel 通信解决 cpal::Stream 不是 Send 的问题

use super::recording_service::RecordingServiceState;
use tauri::State;

/// 开始录音
#[command]
pub async fn start_recording(
    recording_service: State<'_, RecordingServiceState>,
    device_id: Option<String>,
) -> Result<(), String> {
    tracing::info!("[录音命令] 收到开始录音请求，设备ID: {:?}", device_id);
    let mut service = recording_service.0.lock();
    let result = service.start(device_id);
    tracing::info!("[录音命令] 开始录音结果: {:?}", result.is_ok());
    result
}

/// 停止录音并返回音频数据
///
/// 返回的数据结构：
/// - audio_data: i16 样本的字节数组（小端序）
/// - sample_rate: 采样率
/// - duration: 录音时长（秒）
#[command]
pub async fn stop_recording(
    recording_service: State<'_, RecordingServiceState>,
) -> Result<StopRecordingResult, String> {
    let mut service = recording_service.0.lock();
    let audio = service.stop()?;

    tracing::info!(
        "[录音命令] 停止录音，样本数: {}, 采样率: {}, 时长: {:.2}s",
        audio.samples.len(),
        audio.sample_rate,
        audio.duration_secs
    );

    // 检查音频数据是否有效
    let non_zero_samples = audio.samples.iter().filter(|&&s| s != 0).count();
    let non_zero_ratio = non_zero_samples as f32 / audio.samples.len().max(1) as f32;
    tracing::info!(
        "[录音命令] 非零样本比例: {:.2}% ({}/{})",
        non_zero_ratio * 100.0,
        non_zero_samples,
        audio.samples.len()
    );

    // 将 i16 样本转换为字节（小端序）
    let bytes: Vec<u8> = audio
        .samples
        .iter()
        .flat_map(|&s| s.to_le_bytes())
        .collect();

    Ok(StopRecordingResult {
        audio_data: bytes,
        sample_rate: audio.sample_rate,
        duration: audio.duration_secs,
    })
}

/// 停止录音的返回结果
#[derive(serde::Serialize)]
pub struct StopRecordingResult {
    /// 音频数据（i16 样本的字节数组，小端序）
    pub audio_data: Vec<u8>,
    /// 采样率
    pub sample_rate: u32,
    /// 录音时长（秒）
    pub duration: f32,
}

/// 取消录音
#[command]
pub async fn cancel_recording(
    recording_service: State<'_, RecordingServiceState>,
) -> Result<(), String> {
    // 使用 try_lock 避免阻塞，如果锁被占用则跳过
    match recording_service.0.try_lock() {
        Some(mut service) => {
            service.cancel();
            tracing::info!("[录音命令] 取消录音成功");
        }
        None => {
            tracing::warn!("[录音命令] 取消录音时锁被占用，跳过");
            // 即使锁被占用，也尝试直接重置状态标志
        }
    }
    Ok(())
}

/// 录音状态
#[derive(serde::Serialize)]
pub struct RecordingStatus {
    /// 是否正在录音
    pub is_recording: bool,
    /// 当前音量级别（0-100）
    pub volume: u32,
    /// 录音时长（秒）
    pub duration: f32,
}

/// 获取录音状态
#[command]
pub async fn get_recording_status(
    recording_service: State<'_, RecordingServiceState>,
) -> Result<RecordingStatus, String> {
    let service = recording_service.0.lock();
    let status = RecordingStatus {
        is_recording: service.is_recording(),
        volume: service.get_volume(),
        duration: service.get_duration(),
    };
    tracing::debug!(
        "[录音命令] 获取状态: is_recording={}, volume={}, duration={:.2}",
        status.is_recording,
        status.volume,
        status.duration
    );
    Ok(status)
}
