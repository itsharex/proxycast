//! 音频输入设备枚举
//!
//! 提供跨平台的麦克风设备发现能力。

use cpal::traits::{DeviceTrait, HostTrait};
use serde::{Deserialize, Serialize};

use crate::error::{Result, VoiceError};

/// 麦克风设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDeviceInfo {
    /// 设备 ID（用于选择设备）
    pub id: String,
    /// 设备名称
    pub name: String,
    /// 是否为默认设备
    pub is_default: bool,
}

/// 获取所有可用的麦克风设备
pub fn list_audio_devices() -> Result<Vec<AudioDeviceInfo>> {
    let host = cpal::default_host();
    let default_device = host.default_input_device();
    let default_name = default_device.as_ref().and_then(|d| d.name().ok());

    let devices = host
        .input_devices()
        .map_err(|e| VoiceError::RecorderError(format!("无法枚举音频设备: {e}")))?
        .filter_map(|device| {
            let name = device.name().ok()?;
            let is_default = default_name.as_ref().map(|n| n == &name).unwrap_or(false);

            Some(AudioDeviceInfo {
                id: name.clone(),
                name,
                is_default,
            })
        })
        .collect();

    Ok(devices)
}
