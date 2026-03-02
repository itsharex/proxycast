//! 日志命令
//!
//! 包含日志查询和清理命令。

use crate::app::types::LogState;
use crate::logger;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// 前端异常上报参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendCrashReport {
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_step: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creation_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}

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

fn parse_persisted_log_line(line: &str) -> Option<logger::LogEntry> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some((timestamp, rest)) = trimmed.split_once(" [") {
        if let Some((level, message)) = rest.split_once("] ") {
            return Some(logger::LogEntry {
                timestamp: timestamp.trim().to_string(),
                level: level.trim().to_lowercase(),
                message: message.trim().to_string(),
            });
        }
    }

    Some(logger::LogEntry {
        timestamp: Utc::now().to_rfc3339(),
        level: "info".to_string(),
        message: trimmed.to_string(),
    })
}

/// 获取持久化日志文件尾部（用于崩溃后恢复诊断）
#[tauri::command]
pub async fn get_persisted_logs_tail(
    logs: tauri::State<'_, LogState>,
    lines: Option<usize>,
) -> Result<Vec<logger::LogEntry>, String> {
    let safe_limit = lines.unwrap_or(200).clamp(20, 1000);
    let log_file_path = logs.read().await.get_log_file_path();

    let Some(path) = log_file_path else {
        return Ok(Vec::new());
    };

    let content = std::fs::read_to_string(&path).map_err(|e| format!("读取持久化日志失败: {e}"))?;

    if content.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut parsed: Vec<logger::LogEntry> = content
        .lines()
        .rev()
        .take(safe_limit)
        .filter_map(parse_persisted_log_line)
        .collect();

    parsed.reverse();
    Ok(parsed)
}

/// 写入前端异常到本地日志并同步到崩溃上报后端
#[tauri::command]
pub async fn report_frontend_crash(
    logs: tauri::State<'_, LogState>,
    report: FrontendCrashReport,
) -> Result<(), String> {
    let sanitized_message = logger::sanitize_log_message(&report.message);
    let sanitized_component = report
        .component
        .as_deref()
        .map(logger::sanitize_log_message)
        .unwrap_or_else(|| "unknown".to_string());
    let sanitized_step = report
        .workflow_step
        .as_deref()
        .map(logger::sanitize_log_message)
        .unwrap_or_else(|| "unknown".to_string());
    let sanitized_mode = report
        .creation_mode
        .as_deref()
        .map(logger::sanitize_log_message)
        .unwrap_or_else(|| "unknown".to_string());

    let stack_preview = report
        .stack
        .as_deref()
        .map(logger::sanitize_log_message)
        .map(|stack| stack.lines().take(3).collect::<Vec<_>>().join(" | "))
        .unwrap_or_default();

    logs.write().await.add(
        "error",
        &format!(
            "[FrontendCrash] component={sanitized_component} step={sanitized_step} mode={sanitized_mode} message={sanitized_message} stack={stack_preview}"
        ),
    );

    let mut merged_context = match report.context {
        Some(Value::Object(context)) => context,
        Some(other) => {
            let mut context = Map::new();
            context.insert("raw_context".to_string(), other);
            context
        }
        None => Map::new(),
    };

    if let Some(name) = report.name.as_deref() {
        merged_context.insert(
            "error_name".to_string(),
            Value::String(logger::sanitize_log_message(name)),
        );
    }
    if let Some(stack) = report.stack.as_deref() {
        merged_context.insert(
            "error_stack".to_string(),
            Value::String(logger::sanitize_log_message(stack)),
        );
    }

    crate::crash_reporting::capture_frontend_report(
        &sanitized_message,
        report.component.as_deref(),
        report.workflow_step.as_deref(),
        report.creation_mode.as_deref(),
        Some(Value::Object(merged_context)),
    );

    Ok(())
}
