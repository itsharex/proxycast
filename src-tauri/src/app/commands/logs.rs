//! 日志命令
//!
//! 包含日志查询和清理命令。

use crate::app::types::LogState;
use crate::logger;

/// 获取日志
#[tauri::command]
pub async fn get_logs(logs: tauri::State<'_, LogState>) -> Result<Vec<logger::LogEntry>, String> {
    Ok(logs.read().await.get_logs())
}

/// 清除日志
#[tauri::command]
pub async fn clear_logs(logs: tauri::State<'_, LogState>) -> Result<(), String> {
    logs.write().await.clear();
    Ok(())
}
