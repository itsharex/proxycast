//! 心跳引擎 Tauri 命令

use crate::app::LogState;
use crate::config::save_config;
use crate::database::DbConnection;
use crate::services::heartbeat_service::schedule::{
    preview_next_run, validate_schedule as validate_schedule_fn,
};
use crate::services::heartbeat_service::templates::{TaskTemplate, TaskTemplateRegistry};
use crate::services::heartbeat_service::{delivery::deliver_result, delivery::TaskResult};
use crate::services::heartbeat_service::{
    CycleResult, HeartbeatServiceState, HeartbeatStatus, HeartbeatTaskPreview,
};
use crate::AppState;
use proxycast_core::app_paths;
use proxycast_core::config::{DeliveryConfig, HeartbeatSecurityConfig, TaskSchedule};
use proxycast_core::database::dao::heartbeat::HeartbeatExecution;
use proxycast_websocket::handlers::{RpcHandler, RpcHandlerState};
use proxycast_websocket::protocol::{CronHealthResult, GatewayRpcRequest, RpcMethod};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ========== 配置响应类型 ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatConfigResponse {
    pub enabled: bool,
    pub interval_secs: u64,
    pub schedule: Option<TaskSchedule>,
    pub task_file: String,
    pub execution_mode: String,
    pub enable_history: bool,
    pub max_retries: u32,
    pub delivery: DeliveryConfig,
    pub security: HeartbeatSecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HeartbeatTaskHealthQuery {
    pub running_timeout_minutes: Option<u64>,
    pub top_limit: Option<usize>,
    pub cooldown_alert_threshold: Option<usize>,
    pub stale_running_alert_threshold: Option<usize>,
    pub failed_24h_alert_threshold: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatTaskHealthAlertDeliveryResult {
    pub delivered: bool,
    pub alert_count: usize,
    pub channel: Option<String>,
    pub message: String,
}

// ========== 配置命令 ==========

#[tauri::command]
pub async fn get_heartbeat_config(
    state: tauri::State<'_, AppState>,
) -> Result<HeartbeatConfigResponse, String> {
    let s = state.read().await;
    let c = &s.config.heartbeat;
    Ok(HeartbeatConfigResponse {
        enabled: c.enabled,
        interval_secs: c.interval_secs,
        schedule: c.schedule.clone(),
        task_file: c.task_file.clone(),
        execution_mode: match c.execution_mode {
            proxycast_core::config::HeartbeatExecutionMode::Intelligent => "intelligent".into(),
            proxycast_core::config::HeartbeatExecutionMode::Skill => "skill".into(),
            proxycast_core::config::HeartbeatExecutionMode::LogOnly => "log_only".into(),
        },
        enable_history: c.enable_history,
        max_retries: c.max_retries,
        delivery: c.delivery.clone(),
        security: c.security.clone(),
    })
}

#[tauri::command]
pub async fn update_heartbeat_config(
    state: tauri::State<'_, AppState>,
    hb_state: tauri::State<'_, HeartbeatServiceState>,
    app: tauri::AppHandle,
    config: HeartbeatConfigResponse,
) -> Result<(), String> {
    use proxycast_core::config::HeartbeatExecutionMode;

    let execution_mode = match config.execution_mode.as_str() {
        "intelligent" => HeartbeatExecutionMode::Intelligent,
        "skill" => HeartbeatExecutionMode::Skill,
        "log_only" => HeartbeatExecutionMode::LogOnly,
        _ => HeartbeatExecutionMode::Intelligent,
    };

    // 验证调度配置
    if let Some(ref schedule) = config.schedule {
        if let Err(e) = validate_schedule_fn(schedule, chrono::Utc::now()) {
            return Err(format!("调度配置无效: {}", e));
        }
    }

    let was_enabled;
    // 更新 AppState 中的配置
    {
        let mut s = state.write().await;
        was_enabled = s.config.heartbeat.enabled;
        s.config.heartbeat.enabled = config.enabled;
        s.config.heartbeat.interval_secs = config.interval_secs;
        s.config.heartbeat.schedule = config.schedule.clone();
        s.config.heartbeat.task_file = config.task_file.clone();
        s.config.heartbeat.execution_mode = execution_mode;
        s.config.heartbeat.enable_history = config.enable_history;
        s.config.heartbeat.max_retries = config.max_retries;
        s.config.heartbeat.delivery = config.delivery.clone();
        s.config.heartbeat.security = config.security.clone();
        save_config(&s.config).map_err(|e| e.to_string())?;
    }

    // 同步更新 HeartbeatService 的配置，并处理启停
    {
        let mut service = hb_state.0.write().await;
        service.update_config(proxycast_core::config::HeartbeatSettings {
            enabled: config.enabled,
            interval_secs: config.interval_secs,
            schedule: config.schedule,
            task_file: config.task_file,
            execution_mode,
            enable_history: config.enable_history,
            max_retries: config.max_retries,
            delivery: config.delivery,
            security: config.security,
        });

        // 处理启停逻辑
        if config.enabled && !was_enabled {
            service.set_app_handle(app.clone());
            let app_data_dir = app_paths::preferred_data_dir()
                .map_err(|e| format!("无法获取应用数据目录: {e}"))?;
            let self_ref = hb_state.0.clone();
            service.start(app_data_dir, self_ref).await?;
        } else if !config.enabled && was_enabled {
            service.stop().await?;
        }
    }

    Ok(())
}

// ========== 状态和任务命令 ==========

#[tauri::command]
pub async fn get_heartbeat_status(
    hb_state: tauri::State<'_, HeartbeatServiceState>,
) -> Result<HeartbeatStatus, String> {
    let service = hb_state.0.read().await;
    Ok(service.get_status())
}

#[tauri::command]
pub async fn get_heartbeat_tasks(
    hb_state: tauri::State<'_, HeartbeatServiceState>,
    _app: tauri::AppHandle,
) -> Result<Vec<HeartbeatTaskPreview>, String> {
    let app_data_dir =
        app_paths::preferred_data_dir().map_err(|e| format!("无法获取应用数据目录: {e}"))?;
    let service = hb_state.0.read().await;
    service.preview_tasks(&app_data_dir)
}

// ========== 执行历史命令 ==========

#[tauri::command]
pub async fn get_heartbeat_history(
    hb_state: tauri::State<'_, HeartbeatServiceState>,
    limit: Option<usize>,
) -> Result<Vec<HeartbeatExecution>, String> {
    let service = hb_state.0.read().await;
    service.get_execution_history(limit.unwrap_or(50))
}

#[tauri::command]
pub async fn get_heartbeat_execution_detail(
    hb_state: tauri::State<'_, HeartbeatServiceState>,
    execution_id: i64,
) -> Result<Option<HeartbeatExecution>, String> {
    let service = hb_state.0.read().await;
    service.get_execution_detail(execution_id)
}

#[tauri::command]
pub async fn get_heartbeat_task_health(
    db: tauri::State<'_, DbConnection>,
    logs: tauri::State<'_, LogState>,
    query: Option<HeartbeatTaskHealthQuery>,
) -> Result<CronHealthResult, String> {
    query_heartbeat_task_health_via_rpc(db.inner().clone(), logs.inner().clone(), query).await
}

#[tauri::command]
pub async fn deliver_heartbeat_task_health_alerts(
    state: tauri::State<'_, AppState>,
    db: tauri::State<'_, DbConnection>,
    logs: tauri::State<'_, LogState>,
    query: Option<HeartbeatTaskHealthQuery>,
) -> Result<HeartbeatTaskHealthAlertDeliveryResult, String> {
    let health =
        query_heartbeat_task_health_via_rpc(db.inner().clone(), logs.inner().clone(), query)
            .await?;
    if health.alerts.is_empty() {
        return Ok(HeartbeatTaskHealthAlertDeliveryResult {
            delivered: false,
            alert_count: 0,
            channel: None,
            message: "当前无告警，未触发投递".to_string(),
        });
    }

    let delivery_config = {
        let app_state = state.read().await;
        app_state.config.heartbeat.delivery.clone()
    };
    let channel = delivery_config.channel.clone();

    let output = health
        .alerts
        .iter()
        .take(5)
        .map(|alert| {
            format!(
                "- [{}] {} ({}/{})",
                alert.severity, alert.message, alert.current_value, alert.threshold
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let has_critical = health
        .alerts
        .iter()
        .any(|item| item.severity.eq_ignore_ascii_case("critical"));
    let result = TaskResult {
        task: format!("Heartbeat 治理告警（{} 条）", health.alerts.len()),
        status: if has_critical {
            "failed".to_string()
        } else {
            "partial".to_string()
        },
        output,
        duration_ms: 0,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    let delivery_result = deliver_result(&delivery_config, &result).await;
    if !delivery_result.success && !delivery_config.best_effort {
        return Err(format!("告警投递失败: {}", delivery_result.message));
    }

    Ok(HeartbeatTaskHealthAlertDeliveryResult {
        delivered: delivery_result.success,
        alert_count: health.alerts.len(),
        channel,
        message: delivery_result.message,
    })
}

async fn query_heartbeat_task_health_via_rpc(
    db: DbConnection,
    logs: LogState,
    query: Option<HeartbeatTaskHealthQuery>,
) -> Result<CronHealthResult, String> {
    let rpc_state = RpcHandlerState::new(Some(db), None, logs);
    let rpc_handler = RpcHandler::new(rpc_state);
    let params = query
        .map(|q| serde_json::to_value(q).map_err(|e| format!("序列化健康查询参数失败: {e}")))
        .transpose()?;
    let request = GatewayRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Uuid::new_v4().to_string(),
        method: RpcMethod::CronHealth,
        params,
    };
    let response = rpc_handler.handle_request(request).await;
    if let Some(error) = response.error {
        return Err(format!(
            "获取任务健康失败: {} (code={})",
            error.message, error.code
        ));
    }
    let result = response
        .result
        .ok_or_else(|| "获取任务健康失败: RPC 返回缺少 result".to_string())?;
    serde_json::from_value(result).map_err(|e| format!("解析任务健康结果失败: {e}"))
}

// ========== 任务模板命令 ==========

#[tauri::command]
pub async fn get_task_templates() -> Result<Vec<TaskTemplate>, String> {
    Ok(TaskTemplateRegistry::get_all_templates())
}

#[tauri::command]
pub async fn apply_task_template(
    template_id: String,
    _app: tauri::AppHandle,
) -> Result<(), String> {
    let app_data_dir =
        app_paths::preferred_data_dir().map_err(|e| format!("无法获取应用数据目录: {e}"))?;

    let template = TaskTemplateRegistry::get_template_by_id(&template_id)
        .ok_or_else(|| format!("模板不存在: {}", template_id))?;

    TaskTemplateRegistry::apply_template(&template, &app_data_dir)
}

// ========== 任务增删改命令 ==========

#[tauri::command]
pub async fn add_heartbeat_task(
    hb_state: tauri::State<'_, HeartbeatServiceState>,
    _app: tauri::AppHandle,
    description: String,
    priority: Option<u8>,
    timeout_secs: Option<u64>,
    once: Option<bool>,
    model: Option<String>,
) -> Result<(), String> {
    let app_data_dir =
        app_paths::preferred_data_dir().map_err(|e| format!("无法获取应用数据目录: {e}"))?;
    let service = hb_state.0.read().await;
    service.add_task(
        &app_data_dir,
        description,
        priority,
        timeout_secs,
        once,
        model,
    )
}

#[tauri::command]
pub async fn delete_heartbeat_task(
    hb_state: tauri::State<'_, HeartbeatServiceState>,
    _app: tauri::AppHandle,
    index: usize,
) -> Result<(), String> {
    let app_data_dir =
        app_paths::preferred_data_dir().map_err(|e| format!("无法获取应用数据目录: {e}"))?;
    let service = hb_state.0.read().await;
    service.delete_task(&app_data_dir, index)
}

#[tauri::command]
pub async fn update_heartbeat_task(
    hb_state: tauri::State<'_, HeartbeatServiceState>,
    _app: tauri::AppHandle,
    index: usize,
    description: String,
    priority: Option<u8>,
    timeout_secs: Option<u64>,
    once: Option<bool>,
    model: Option<String>,
) -> Result<(), String> {
    let app_data_dir =
        app_paths::preferred_data_dir().map_err(|e| format!("无法获取应用数据目录: {e}"))?;
    let service = hb_state.0.read().await;
    service.update_task(
        &app_data_dir,
        index,
        description,
        priority,
        timeout_secs,
        once,
        model,
    )
}

// ========== 内容创作集成命令 ==========

#[tauri::command]
pub async fn generate_content_creator_tasks(
    state: tauri::State<'_, AppState>,
    _app: tauri::AppHandle,
) -> Result<usize, String> {
    use crate::services::heartbeat_service::templates::ContentCreatorTaskGenerator;

    let enabled_themes = {
        let s = state.read().await;
        s.config.content_creator.enabled_themes.clone()
    };

    let tasks = ContentCreatorTaskGenerator::generate_tasks(&enabled_themes);
    let count = tasks.len();

    if count > 0 {
        let app_data_dir =
            app_paths::preferred_data_dir().map_err(|e| format!("无法获取应用数据目录: {e}"))?;
        ContentCreatorTaskGenerator::append_to_heartbeat(tasks, &app_data_dir)?;
    }

    Ok(count)
}

// ========== 手动触发命令 ==========

#[tauri::command]
pub async fn trigger_heartbeat_now(
    hb_state: tauri::State<'_, HeartbeatServiceState>,
    app: tauri::AppHandle,
) -> Result<CycleResult, String> {
    let app_data_dir =
        app_paths::preferred_data_dir().map_err(|e| format!("无法获取应用数据目录: {e}"))?;

    let result = {
        let service = hb_state.0.read().await;
        service.trigger_now(app_data_dir, Some(app.clone())).await
    };
    {
        let mut service = hb_state.0.write().await;
        service.update_status_after_cycle(&result);
    }
    Ok(result)
}

// ========== 调度预览和验证命令 ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleValidationResult {
    pub valid: bool,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn preview_heartbeat_schedule(schedule: TaskSchedule) -> Result<Option<String>, String> {
    preview_next_run(&schedule).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn validate_heartbeat_schedule(
    schedule: TaskSchedule,
) -> Result<ScheduleValidationResult, String> {
    match validate_schedule_fn(&schedule, chrono::Utc::now()) {
        Ok(()) => Ok(ScheduleValidationResult {
            valid: true,
            error: None,
        }),
        Err(e) => Ok(ScheduleValidationResult {
            valid: false,
            error: Some(e.to_string()),
        }),
    }
}
