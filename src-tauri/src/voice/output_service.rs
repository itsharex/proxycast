//! 文字输出服务
//!
//! 提供模拟键盘输入和剪贴板输出功能

use crate::config::VoiceOutputMode;
use arboard::Clipboard;

/// 输出文字到系统
///
/// 根据配置的输出模式，将文字输出到当前焦点应用
pub fn output_text(text: &str, mode: VoiceOutputMode) -> Result<(), String> {
    match mode {
        VoiceOutputMode::Type => type_text(text),
        VoiceOutputMode::Clipboard => copy_to_clipboard(text),
        VoiceOutputMode::Both => {
            copy_to_clipboard(text)?;
            type_text(text)
        }
    }
}

/// 模拟键盘输入文字
fn type_text(text: &str) -> Result<(), String> {
    use enigo::{Enigo, Keyboard, Settings};

    let mut enigo =
        Enigo::new(&Settings::default()).map_err(|e| format!("初始化键盘模拟器失败: {e}"))?;

    enigo.text(text).map_err(|e| format!("键盘输入失败: {e}"))?;

    tracing::info!("[语音输出] 键盘输入完成: {} 字符", text.chars().count());
    Ok(())
}

/// 复制到剪贴板
fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| format!("初始化剪贴板失败: {e}"))?;

    clipboard
        .set_text(text)
        .map_err(|e| format!("复制到剪贴板失败: {e}"))?;

    tracing::info!("[语音输出] 已复制到剪贴板: {} 字符", text.chars().count());
    Ok(())
}
