//! 插件 UI Trait 定义
//!
//! 定义插件需要实现的 UI 接口

use async_trait::async_trait;

use super::ui_types::{SurfaceDefinition, UIMessage, UserAction};
use super::PluginError;

/// 插件 UI Trait
/// 插件实现此 trait 以提供声明式 UI
#[async_trait]
pub trait PluginUI: Send + Sync {
    /// 获取插件的 Surface 定义列表
    /// 返回插件想要渲染的所有 UI Surface
    fn get_surfaces(&self) -> Vec<SurfaceDefinition>;

    /// 处理用户操作
    /// 返回需要发送给前端的 UI 消息列表
    async fn handle_action(&mut self, action: UserAction) -> Result<Vec<UIMessage>, PluginError>;

    /// 是否支持 UI
    fn has_ui(&self) -> bool {
        !self.get_surfaces().is_empty()
    }
}

/// 空 UI 实现 - 用于不需要 UI 的插件
pub struct NoUI;

#[async_trait]
impl PluginUI for NoUI {
    fn get_surfaces(&self) -> Vec<SurfaceDefinition> {
        Vec::new()
    }

    async fn handle_action(&mut self, _action: UserAction) -> Result<Vec<UIMessage>, PluginError> {
        Ok(Vec::new())
    }

    fn has_ui(&self) -> bool {
        false
    }
}
