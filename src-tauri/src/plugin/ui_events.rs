//! 插件 UI 事件系统
//!
//! 提供从 Rust 向前端推送 UI 更新的能力

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use super::ui_types::UIMessage;

/// 插件 UI 事件载荷
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginUIEventPayload {
    /// 插件 ID
    pub plugin_id: String,
    /// UI 消息
    pub message: UIMessage,
}

/// 插件 UI 事件发射器
pub struct PluginUIEmitter {
    app_handle: AppHandle,
}

impl PluginUIEmitter {
    /// 创建新的事件发射器
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    /// 发送单个 UI 消息
    pub fn emit(&self, plugin_id: &str, message: UIMessage) -> Result<(), String> {
        let payload = PluginUIEventPayload {
            plugin_id: plugin_id.to_string(),
            message,
        };

        self.app_handle
            .emit("plugin-ui-message", payload)
            .map_err(|e| e.to_string())
    }

    /// 发送多个 UI 消息
    pub fn emit_all(&self, plugin_id: &str, messages: Vec<UIMessage>) -> Result<(), String> {
        for message in messages {
            self.emit(plugin_id, message)?;
        }
        Ok(())
    }

    /// 发送 Surface 更新
    pub fn emit_surface_update(
        &self,
        plugin_id: &str,
        update: super::ui_types::SurfaceUpdate,
    ) -> Result<(), String> {
        self.emit(plugin_id, UIMessage::SurfaceUpdate(update))
    }

    /// 发送数据模型更新
    pub fn emit_data_update(
        &self,
        plugin_id: &str,
        update: super::ui_types::DataModelUpdate,
    ) -> Result<(), String> {
        self.emit(plugin_id, UIMessage::DataModelUpdate(update))
    }

    /// 发送删除 Surface
    pub fn emit_delete_surface(&self, plugin_id: &str, surface_id: &str) -> Result<(), String> {
        self.emit(
            plugin_id,
            UIMessage::DeleteSurface(super::ui_types::DeleteSurface {
                surface_id: surface_id.to_string(),
            }),
        )
    }
}

/// 全局事件发射器状态
pub struct PluginUIEmitterState(pub Option<PluginUIEmitter>);

impl PluginUIEmitterState {
    /// 创建空状态
    pub fn new() -> Self {
        Self(None)
    }

    /// 初始化发射器
    pub fn init(&mut self, app_handle: AppHandle) {
        self.0 = Some(PluginUIEmitter::new(app_handle));
    }

    /// 获取发射器
    pub fn get(&self) -> Option<&PluginUIEmitter> {
        self.0.as_ref()
    }
}

impl Default for PluginUIEmitterState {
    fn default() -> Self {
        Self::new()
    }
}
