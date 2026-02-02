//! Kiro 凭证指纹绑定模型
//!
//! 为每个 Kiro 凭证存储独立的 Machine ID，实现多账号指纹隔离。

#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Kiro 凭证指纹绑定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KiroFingerprintBinding {
    /// 凭证 UUID
    pub credential_uuid: String,
    /// 绑定的 Machine ID
    pub machine_id: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后切换时间
    pub last_switched_at: Option<DateTime<Utc>>,
}

/// 指纹绑定存储
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KiroFingerprintStore {
    /// 凭证 UUID -> 指纹绑定
    pub bindings: HashMap<String, KiroFingerprintBinding>,
}

impl KiroFingerprintStore {
    /// 获取存储文件路径
    pub fn get_storage_path() -> Result<PathBuf, String> {
        let app_data_dir = dirs::data_dir()
            .ok_or_else(|| "无法获取应用数据目录".to_string())?
            .join("proxycast");

        if !app_data_dir.exists() {
            fs::create_dir_all(&app_data_dir).map_err(|e| format!("创建应用数据目录失败: {e}"))?;
        }

        Ok(app_data_dir.join("kiro_fingerprints.json"))
    }

    /// 从文件加载
    pub fn load() -> Result<Self, String> {
        let path = Self::get_storage_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let content =
            fs::read_to_string(&path).map_err(|e| format!("读取指纹存储文件失败: {e}"))?;

        serde_json::from_str(&content).map_err(|e| format!("解析指纹存储文件失败: {e}"))
    }

    /// 保存到文件
    pub fn save(&self) -> Result<(), String> {
        let path = Self::get_storage_path()?;
        let content =
            serde_json::to_string_pretty(self).map_err(|e| format!("序列化指纹存储失败: {e}"))?;

        fs::write(&path, content).map_err(|e| format!("写入指纹存储文件失败: {e}"))
    }

    /// 获取凭证的指纹绑定
    pub fn get_binding(&self, credential_uuid: &str) -> Option<&KiroFingerprintBinding> {
        self.bindings.get(credential_uuid)
    }

    /// 获取或创建凭证的指纹绑定
    pub fn get_or_create_binding(
        &mut self,
        credential_uuid: &str,
        profile_arn: Option<&str>,
        client_id: Option<&str>,
    ) -> Result<&KiroFingerprintBinding, String> {
        if !self.bindings.contains_key(credential_uuid) {
            let machine_id = generate_stable_machine_id(credential_uuid, profile_arn, client_id);

            let binding = KiroFingerprintBinding {
                credential_uuid: credential_uuid.to_string(),
                machine_id,
                created_at: Utc::now(),
                last_switched_at: None,
            };

            self.bindings.insert(credential_uuid.to_string(), binding);
            self.save()?;
        }

        Ok(self.bindings.get(credential_uuid).unwrap())
    }

    /// 更新最后切换时间
    pub fn update_last_switched(&mut self, credential_uuid: &str) -> Result<(), String> {
        if let Some(binding) = self.bindings.get_mut(credential_uuid) {
            binding.last_switched_at = Some(Utc::now());
            self.save()?;
        }
        Ok(())
    }

    /// 删除凭证的指纹绑定
    pub fn remove_binding(&mut self, credential_uuid: &str) -> Result<(), String> {
        self.bindings.remove(credential_uuid);
        self.save()
    }
}

/// 生成稳定的 Machine ID
fn generate_stable_machine_id(
    credential_uuid: &str,
    profile_arn: Option<&str>,
    client_id: Option<&str>,
) -> String {
    use sha2::{Digest, Sha256};

    let seed = format!(
        "kiro_fingerprint:{}:{}:{}",
        credential_uuid,
        profile_arn.unwrap_or(""),
        client_id.unwrap_or("")
    );

    let mut hasher = Sha256::new();
    hasher.update(seed.as_bytes());
    let result = hasher.finalize();

    let hex = format!("{result:x}");
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

/// 切换到本地的结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchToLocalResult {
    pub success: bool,
    pub message: String,
    pub requires_action: bool,
    pub machine_id: Option<String>,
    pub requires_kiro_restart: bool,
}

impl SwitchToLocalResult {
    pub fn success(message: impl Into<String>, machine_id: String) -> Self {
        Self {
            success: true,
            message: message.into(),
            requires_action: false,
            machine_id: Some(machine_id),
            requires_kiro_restart: true,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            requires_action: false,
            machine_id: None,
            requires_kiro_restart: false,
        }
    }

    pub fn requires_admin(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            requires_action: true,
            machine_id: None,
            requires_kiro_restart: false,
        }
    }
}
