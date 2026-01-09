//! 悬浮窗口管理
//!
//! 提供截图对话悬浮窗口的创建、显示和关闭功能

use mouse_position::mouse_position::Mouse;
use std::path::Path;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tracing::{debug, info, warn};

#[cfg(target_os = "macos")]
use cocoa::appkit::{NSColor, NSWindow};
#[cfg(target_os = "macos")]
use cocoa::base::{id, nil};

/// 窗口错误类型
#[derive(Debug, thiserror::Error)]
pub enum WindowError {
    #[error("窗口创建失败: {0}")]
    CreateFailed(String),
    #[error("窗口未找到: {0}")]
    NotFound(String),
    #[error("窗口操作失败: {0}")]
    OperationFailed(String),
}

/// 悬浮窗口标签
const FLOATING_WINDOW_LABEL: &str = "screenshot-chat";

/// 窗口尺寸（包含 padding 用于阴影）
const WINDOW_WIDTH: f64 = 645.0;
const WINDOW_HEIGHT: f64 = 70.0;
/// 距离屏幕底部的距离
const BOTTOM_MARGIN: f64 = 200.0;

/// 获取鼠标所在的显示器
///
/// 使用 mouse_position crate 获取鼠标位置，然后遍历所有显示器找到鼠标所在的显示器
fn get_monitor_at_cursor(app: &AppHandle) -> Option<tauri::Monitor> {
    // 使用 mouse_position crate 获取鼠标位置
    let (cursor_x, cursor_y) = match Mouse::get_mouse_position() {
        Mouse::Position { x, y } => {
            debug!("鼠标位置 (mouse_position crate): ({}, {})", x, y);
            (x as f64, y as f64)
        }
        Mouse::Error => {
            debug!("无法获取鼠标位置");
            return None;
        }
    };

    // 获取所有显示器
    let monitors = match app.available_monitors() {
        Ok(monitors) => monitors,
        Err(e) => {
            debug!("无法获取显示器列表: {}", e);
            return None;
        }
    };

    // 查找鼠标所在的显示器
    for monitor in monitors {
        let pos = monitor.position();
        let size = monitor.size();

        let left = pos.x as f64;
        let top = pos.y as f64;
        let right = left + size.width as f64;
        let bottom = top + size.height as f64;

        if cursor_x >= left && cursor_x < right && cursor_y >= top && cursor_y < bottom {
            debug!(
                "鼠标在显示器: {:?}, 位置: ({}, {}), 尺寸: {}x{}",
                monitor.name(),
                pos.x,
                pos.y,
                size.width,
                size.height
            );
            return Some(monitor);
        }
    }

    debug!(
        "未找到鼠标所在的显示器，鼠标位置: ({}, {})",
        cursor_x, cursor_y
    );
    None
}

/// 计算窗口位置（屏幕底部居中）
///
/// 优先使用鼠标所在的显示器，否则使用主显示器
/// 返回逻辑坐标（考虑 DPI 缩放）
fn calculate_window_position(app: &AppHandle) -> (f64, f64) {
    // 优先获取鼠标所在的显示器
    let monitor = get_monitor_at_cursor(app).or_else(|| app.primary_monitor().ok().flatten());

    if let Some(monitor) = monitor {
        let screen_pos = monitor.position();
        let screen_size = monitor.size();
        let scale_factor = monitor.scale_factor();

        // 物理像素转换为逻辑像素
        let screen_width = screen_size.width as f64 / scale_factor;
        let screen_height = screen_size.height as f64 / scale_factor;
        let screen_x = screen_pos.x as f64 / scale_factor;
        let screen_y = screen_pos.y as f64 / scale_factor;

        // 在该显示器上水平居中（使用逻辑坐标）
        let x = screen_x + (screen_width - WINDOW_WIDTH) / 2.0;
        // 距离该显示器底部 BOTTOM_MARGIN（使用逻辑坐标）
        let y = screen_y + screen_height - WINDOW_HEIGHT - BOTTOM_MARGIN;

        debug!(
            "显示器: scale_factor={}, 物理位置: ({}, {}), 物理尺寸: {}x{}",
            scale_factor, screen_pos.x, screen_pos.y, screen_size.width, screen_size.height
        );
        debug!(
            "逻辑坐标: 屏幕({}, {}), 尺寸: {}x{}, 窗口位置: ({}, {})",
            screen_x, screen_y, screen_width, screen_height, x, y
        );
        return (x, y);
    }

    // 默认位置（如果无法获取屏幕尺寸）
    debug!("无法获取显示器信息，使用默认位置");
    (400.0, 600.0)
}

/// 打开悬浮对话窗口
///
/// 如果窗口已在 tauri.conf.json 中预定义，则显示并导航到新 URL
/// 否则动态创建一个全屏透明的悬浮窗口
///
/// # 参数
/// - `app`: Tauri 应用句柄
/// - `image_path`: 截图文件路径
///
/// # 返回
/// 成功返回 Ok(()), 失败返回错误
pub fn open_floating_window(app: &AppHandle, image_path: &Path) -> Result<(), WindowError> {
    info!("打开悬浮对话窗口");

    // 构建窗口 URL，包含图片路径参数
    let image_path_str = image_path.to_str().unwrap_or("");
    let encoded_path = urlencoding::encode(image_path_str);
    let url = format!("/screenshot-chat?image={}", encoded_path);

    debug!("悬浮窗口 URL: {}", url);

    // 检查窗口是否已存在（可能是预定义的或之前创建的）
    if let Some(window) = app.get_webview_window(FLOATING_WINDOW_LABEL) {
        info!("悬浮窗口已存在，导航到新 URL 并显示");

        // 计算窗口位置（返回逻辑坐标）
        let (x, y) = calculate_window_position(app);

        // 设置窗口位置（使用逻辑坐标）
        use tauri::LogicalPosition;
        let _ = window.set_position(LogicalPosition::new(x, y));

        // macOS: 设置窗口和 webview 背景透明
        #[cfg(target_os = "macos")]
        {
            use objc::{msg_send, sel, sel_impl};
            if let Ok(ns_win) = window.ns_window() {
                unsafe {
                    let ns_window = ns_win as id;
                    // 设置窗口背景透明
                    let clear_color = NSColor::clearColor(nil);
                    ns_window.setBackgroundColor_(clear_color);
                    let _: () = msg_send![ns_window, setOpaque: false];
                    // 禁用窗口阴影
                    let _: () = msg_send![ns_window, setHasShadow: false];
                }
            }
        }

        // 使用 JavaScript 导航到新的 URL（更新图片路径）
        let js = format!("window.location.href = '{}';", url);
        window
            .eval(&js)
            .map_err(|e| WindowError::OperationFailed(format!("导航失败: {}", e)))?;

        // 显示窗口
        window
            .show()
            .map_err(|e| WindowError::OperationFailed(format!("显示窗口失败: {}", e)))?;

        // 聚焦窗口
        window
            .set_focus()
            .map_err(|e| WindowError::OperationFailed(format!("聚焦窗口失败: {}", e)))?;

        return Ok(());
    }

    // 窗口不存在，动态创建
    info!("动态创建悬浮窗口");

    // 计算窗口位置
    let (x, y) = calculate_window_position(app);

    // 创建悬浮窗口（启用透明）
    let window = WebviewWindowBuilder::new(app, FLOATING_WINDOW_LABEL, WebviewUrl::App(url.into()))
        .inner_size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .position(x, y)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(true)
        .focused(true)
        .transparent(true)
        .build()
        .map_err(|e| WindowError::CreateFailed(format!("{}", e)))?;

    // macOS: 设置窗口和 webview 背景透明
    #[cfg(target_os = "macos")]
    {
        use objc::{msg_send, sel, sel_impl};
        if let Ok(ns_win) = window.ns_window() {
            unsafe {
                let ns_window = ns_win as id;
                // 设置窗口背景透明
                let clear_color = NSColor::clearColor(nil);
                ns_window.setBackgroundColor_(clear_color);
                let _: () = msg_send![ns_window, setOpaque: false];
                // 禁用窗口阴影
                let _: () = msg_send![ns_window, setHasShadow: false];
            }
        }
    }

    info!("悬浮窗口创建成功: {}", FLOATING_WINDOW_LABEL);

    Ok(())
}

/// 关闭悬浮对话窗口
///
/// # 参数
/// - `app`: Tauri 应用句柄
///
/// # 返回
/// 成功返回 Ok(()), 失败返回错误
pub fn close_floating_window(app: &AppHandle) -> Result<(), WindowError> {
    info!("关闭悬浮对话窗口");

    if let Some(window) = app.get_webview_window(FLOATING_WINDOW_LABEL) {
        window
            .close()
            .map_err(|e| WindowError::OperationFailed(format!("关闭窗口失败: {}", e)))?;
        info!("悬浮窗口已关闭");
    } else {
        debug!("悬浮窗口不存在，无需关闭");
    }

    Ok(())
}

/// 检查悬浮窗口是否打开
///
/// # 参数
/// - `app`: Tauri 应用句柄
///
/// # 返回
/// 如果窗口存在且可见返回 true，否则返回 false
pub fn is_floating_window_open(app: &AppHandle) -> bool {
    app.get_webview_window(FLOATING_WINDOW_LABEL)
        .map(|w| w.is_visible().unwrap_or(false))
        .unwrap_or(false)
}

/// 聚焦悬浮窗口
///
/// # 参数
/// - `app`: Tauri 应用句柄
///
/// # 返回
/// 成功返回 Ok(()), 失败返回错误
pub fn focus_floating_window(app: &AppHandle) -> Result<(), WindowError> {
    if let Some(window) = app.get_webview_window(FLOATING_WINDOW_LABEL) {
        window
            .set_focus()
            .map_err(|e| WindowError::OperationFailed(format!("聚焦窗口失败: {}", e)))?;
        Ok(())
    } else {
        Err(WindowError::NotFound(FLOATING_WINDOW_LABEL.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_label() {
        assert_eq!(FLOATING_WINDOW_LABEL, "screenshot-chat");
    }
}
