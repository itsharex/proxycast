//! 截图对话模块
//!
//! 提供截图对话功能的核心实现，包括：
//! - 屏幕截图服务
//! - 全局快捷键管理
//! - 悬浮窗口管理
//! - 实验室功能配置

pub mod capture;
pub mod config;
pub mod shortcut;
pub mod window;

use tauri::AppHandle;
use tracing::{error, info};

/// 截图模块错误类型
#[derive(Debug, thiserror::Error)]
pub enum ScreenshotError {
    #[error("配置错误: {0}")]
    Config(String),
    #[error("快捷键错误: {0}")]
    Shortcut(#[from] shortcut::ShortcutError),
    #[error("截图错误: {0}")]
    Capture(#[from] capture::CaptureError),
    #[error("窗口错误: {0}")]
    Window(#[from] window::WindowError),
}

/// 初始化截图对话模块
///
/// 根据配置决定是否注册全局快捷键
///
/// # 参数
/// - `app`: Tauri 应用句柄
///
/// # 返回
/// 成功返回 Ok(()), 失败返回错误
pub fn init(app: &AppHandle) -> Result<(), ScreenshotError> {
    info!("初始化截图对话模块");

    // 加载实验室功能配置
    let experimental_config = config::load_experimental_config(app)
        .map_err(|e| ScreenshotError::Config(e.to_string()))?;

    // 检查截图对话功能是否启用
    if config::is_screenshot_chat_enabled(&experimental_config) {
        info!("截图对话功能已启用，注册快捷键");
        shortcut::register(app, &experimental_config.screenshot_chat.shortcut)?;
    } else {
        info!("截图对话功能未启用，跳过快捷键注册");
    }

    Ok(())
}

/// 清理截图对话模块资源
///
/// 注销快捷键并清理临时文件
pub fn cleanup(app: &AppHandle) -> Result<(), ScreenshotError> {
    info!("清理截图对话模块资源");

    // 尝试注销快捷键（忽略错误，因为可能未注册）
    if let Err(e) = shortcut::unregister(app) {
        error!("注销快捷键失败: {}", e);
    }

    Ok(())
}
