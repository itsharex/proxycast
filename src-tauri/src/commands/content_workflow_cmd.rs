//! 内容创作工作流命令
//!
//! 暴露工作流服务给前端

use crate::app::bootstrap::AppStates;
use anyhow::Result;
use proxycast_services::content_creator::{CreationMode, StepResult, ThemeType, WorkflowState};
use tauri::State;
use tracing::{error, info};

/// 创建工作流
#[tauri::command]
pub async fn content_workflow_create(
    content_id: String,
    theme: String,
    mode: String,
    state: State<'_, AppStates>,
) -> Result<WorkflowState, String> {
    info!(
        "创建工作流: content_id={}, theme={}, mode={}",
        content_id, theme, mode
    );

    // 解析主题和模式
    let theme_type: ThemeType = serde_json::from_value(serde_json::json!(theme))
        .map_err(|e| format!("无效的主题类型: {}", e))?;
    let creation_mode: CreationMode = serde_json::from_value(serde_json::json!(mode))
        .map_err(|e| format!("无效的创作模式: {}", e))?;

    // 获取服务
    let workflow_service = state.workflow_service.read().await;
    let progress_store = state.progress_store.read().await;

    // 创建工作流
    let workflow = workflow_service
        .create_workflow(content_id, theme_type, creation_mode)
        .await
        .map_err(|e| {
            error!("创建工作流失败: {}", e);
            format!("创建工作流失败: {}", e)
        })?;

    // 持久化
    progress_store.save_progress(&workflow).await.map_err(|e| {
        error!("保存工作流进度失败: {}", e);
        format!("保存工作流进度失败: {}", e)
    })?;

    Ok(workflow)
}

/// 获取工作流
#[tauri::command]
pub async fn content_workflow_get(
    workflow_id: String,
    state: State<'_, AppStates>,
) -> Result<Option<WorkflowState>, String> {
    info!("获取工作流: workflow_id={}", workflow_id);

    let workflow_service = state.workflow_service.read().await;
    let progress_store = state.progress_store.read().await;

    // 先从内存缓存获取
    if let Some(workflow) = workflow_service.get_workflow(&workflow_id).await {
        return Ok(Some(workflow));
    }

    // 从数据库加载
    let workflow = progress_store
        .load_progress(&workflow_id)
        .await
        .map_err(|e| {
            error!("加载工作流进度失败: {}", e);
            format!("加载工作流进度失败: {}", e)
        })?;

    // 如果从数据库加载成功，更新内存缓存
    if let Some(ref wf) = workflow {
        workflow_service.update_workflow(wf.clone()).await.ok();
    }

    Ok(workflow)
}

/// 根据 content_id 获取工作流
#[tauri::command]
pub async fn content_workflow_get_by_content(
    content_id: String,
    state: State<'_, AppStates>,
) -> Result<Option<WorkflowState>, String> {
    info!("根据 content_id 获取工作流: content_id={}", content_id);

    let workflow_service = state.workflow_service.read().await;
    let progress_store = state.progress_store.read().await;

    // 先从内存缓存获取
    if let Some(workflow) = workflow_service.get_workflow_by_content(&content_id).await {
        return Ok(Some(workflow));
    }

    // 从数据库加载
    let workflow = progress_store
        .load_by_content_id(&content_id)
        .await
        .map_err(|e| {
            error!("根据 content_id 加载工作流进度失败: {}", e);
            format!("根据 content_id 加载工作流进度失败: {}", e)
        })?;

    // 如果从数据库加载成功，更新内存缓存
    if let Some(ref wf) = workflow {
        workflow_service.update_workflow(wf.clone()).await.ok();
    }

    Ok(workflow)
}

/// 推进工作流（完成当前步骤）
#[tauri::command]
pub async fn content_workflow_advance(
    workflow_id: String,
    step_result: StepResult,
    state: State<'_, AppStates>,
) -> Result<WorkflowState, String> {
    info!("推进工作流: workflow_id={}", workflow_id);

    let workflow_service = state.workflow_service.read().await;
    let progress_store = state.progress_store.read().await;

    // 完成当前步骤
    let workflow = workflow_service
        .complete_step(&workflow_id, step_result)
        .await
        .map_err(|e| {
            error!("完成步骤失败: {}", e);
            format!("完成步骤失败: {}", e)
        })?;

    // 持久化
    progress_store.save_progress(&workflow).await.map_err(|e| {
        error!("保存工作流进度失败: {}", e);
        format!("保存工作流进度失败: {}", e)
    })?;

    Ok(workflow)
}

/// 重试失败的步骤
#[tauri::command]
pub async fn content_workflow_retry(
    workflow_id: String,
    state: State<'_, AppStates>,
) -> Result<WorkflowState, String> {
    info!("重试工作流步骤: workflow_id={}", workflow_id);

    let workflow_service = state.workflow_service.read().await;
    let progress_store = state.progress_store.read().await;

    // 重做当前步骤
    let mut workflow = workflow_service
        .get_workflow(&workflow_id)
        .await
        .ok_or_else(|| format!("工作流不存在: {}", workflow_id))?;

    let current_index = workflow.current_step_index;
    if current_index < workflow.steps.len() {
        workflow.steps[current_index].status =
            proxycast_services::content_creator::StepStatus::Pending;
        workflow.steps[current_index].result = None;
        workflow.updated_at = chrono::Utc::now().timestamp_millis();

        // 更新工作流
        workflow_service
            .update_workflow(workflow.clone())
            .await
            .map_err(|e| {
                error!("更新工作流失败: {}", e);
                format!("更新工作流失败: {}", e)
            })?;

        // 持久化
        progress_store.save_progress(&workflow).await.map_err(|e| {
            error!("保存工作流进度失败: {}", e);
            format!("保存工作流进度失败: {}", e)
        })?;
    }

    Ok(workflow)
}

/// 取消工作流
#[tauri::command]
pub async fn content_workflow_cancel(
    workflow_id: String,
    state: State<'_, AppStates>,
) -> Result<(), String> {
    info!("取消工作流: workflow_id={}", workflow_id);

    let progress_store = state.progress_store.read().await;

    // 从数据库删除
    progress_store
        .delete_progress(&workflow_id)
        .await
        .map_err(|e| {
            error!("删除工作流进度失败: {}", e);
            format!("删除工作流进度失败: {}", e)
        })?;

    Ok(())
}
