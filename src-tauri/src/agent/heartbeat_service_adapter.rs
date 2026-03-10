//! Heartbeat Service Adapter
//!
//! 将 Tauri 的 HeartbeatServiceState 适配为 Aster Agent 工具的 HeartbeatService trait

use crate::services::heartbeat_service::{
    CycleResult, HeartbeatService as ProxycastHeartbeatService, HeartbeatServiceState,
    HeartbeatStatus as ProxycastHeartbeatStatus,
    HeartbeatTaskPreview as ProxycastHeartbeatTaskPreview,
};
use proxycast_agent::tools::heartbeat_tool::{
    HeartbeatCycleResult, HeartbeatExecutionRecord, HeartbeatService, HeartbeatStatus,
    HeartbeatTaskPreview, HeartbeatToolError,
};
use proxycast_core::app_paths;
use proxycast_core::database::dao::heartbeat::HeartbeatExecution;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::AppHandle;

/// Heartbeat Service 适配器
///
/// 将 Tauri 的 HeartbeatServiceState 转换为 Agent 工具可使用的 trait
pub struct HeartbeatServiceAdapter {
    hb_service: Arc<tokio::sync::RwLock<ProxycastHeartbeatService>>,
    app_handle: AppHandle,
}

impl HeartbeatServiceAdapter {
    /// 创建新的适配器
    pub fn new(hb_state: HeartbeatServiceState, app_handle: AppHandle) -> Self {
        Self {
            hb_service: Arc::clone(&hb_state.0),
            app_handle,
        }
    }

    /// 获取应用数据目录
    fn app_data_dir(&self) -> Result<PathBuf, HeartbeatToolError> {
        app_paths::preferred_data_dir()
            .map_err(|e| HeartbeatToolError::IoError(format!("获取应用数据目录失败: {}", e)))
    }

    /// 转换执行记录
    fn convert_execution(exec: &HeartbeatExecution) -> HeartbeatExecutionRecord {
        HeartbeatExecutionRecord {
            id: exec.id,
            task_description: exec.task_description.clone(),
            status: exec.status.clone(),
            started_at: exec.started_at.clone(),
            completed_at: exec.completed_at.clone(),
            duration_ms: exec.duration_ms,
            output: exec.output.clone(),
            retry_count: exec.retry_count,
        }
    }

    /// 转换任务预览
    fn convert_task_preview(task: &ProxycastHeartbeatTaskPreview) -> HeartbeatTaskPreview {
        HeartbeatTaskPreview {
            description: task.description.clone(),
            priority: task.priority,
            timeout_secs: task.timeout_secs,
            once: task.once,
            model: task.model.clone(),
        }
    }

    /// 转换状态
    fn convert_status(status: ProxycastHeartbeatStatus) -> HeartbeatStatus {
        HeartbeatStatus {
            running: status.running,
            last_run: status.last_run,
            next_run: status.next_run,
            last_task_count: status.last_task_count,
            total_executions: status.total_executions,
            schedule_description: status.schedule_description,
        }
    }
}

#[async_trait::async_trait]
impl HeartbeatService for HeartbeatServiceAdapter {
    fn get_status(&self) -> Result<HeartbeatStatus, HeartbeatToolError> {
        let service = self.hb_service.blocking_read();
        Ok(Self::convert_status(service.get_status()))
    }

    fn get_app_data_dir(&self) -> Result<PathBuf, HeartbeatToolError> {
        self.app_data_dir()
    }

    fn preview_tasks(&self) -> Result<Vec<HeartbeatTaskPreview>, HeartbeatToolError> {
        let app_data_dir = self.app_data_dir()?;
        let service = self.hb_service.blocking_read();
        let tasks = service
            .preview_tasks(&app_data_dir)
            .map_err(|e| HeartbeatToolError::ExecutionFailed(format!("获取任务列表失败: {}", e)))?;
        Ok(tasks.iter().map(Self::convert_task_preview).collect())
    }

    fn add_task(
        &self,
        description: String,
        priority: Option<u8>,
        timeout_secs: Option<u64>,
        once: Option<bool>,
        model: Option<String>,
    ) -> Result<(), HeartbeatToolError> {
        let app_data_dir = self.app_data_dir()?;
        let service = self.hb_service.blocking_read();
        service
            .add_task(
                &app_data_dir,
                description,
                priority,
                timeout_secs,
                once,
                model,
            )
            .map_err(|e| HeartbeatToolError::ExecutionFailed(format!("添加任务失败: {}", e)))
    }

    fn delete_task(&self, index: usize) -> Result<(), HeartbeatToolError> {
        let app_data_dir = self.app_data_dir()?;
        let service = self.hb_service.blocking_read();
        service
            .delete_task(&app_data_dir, index)
            .map_err(|e| HeartbeatToolError::ExecutionFailed(format!("删除任务失败: {}", e)))
    }

    fn update_task(
        &self,
        index: usize,
        description: String,
        priority: Option<u8>,
        timeout_secs: Option<u64>,
        once: Option<bool>,
        model: Option<String>,
    ) -> Result<(), HeartbeatToolError> {
        let app_data_dir = self.app_data_dir()?;
        let service = self.hb_service.blocking_read();
        service
            .update_task(
                &app_data_dir,
                index,
                description,
                priority,
                timeout_secs,
                once,
                model,
            )
            .map_err(|e| HeartbeatToolError::ExecutionFailed(format!("更新任务失败: {}", e)))
    }

    fn get_history(
        &self,
        limit: usize,
    ) -> Result<Vec<HeartbeatExecutionRecord>, HeartbeatToolError> {
        let service = self.hb_service.blocking_read();
        let records = service
            .get_execution_history(limit)
            .map_err(|e| HeartbeatToolError::ExecutionFailed(format!("获取历史失败: {}", e)))?;
        Ok(records.iter().map(Self::convert_execution).collect())
    }

    fn get_execution_detail(
        &self,
        id: i64,
    ) -> Result<Option<HeartbeatExecutionRecord>, HeartbeatToolError> {
        let service = self.hb_service.blocking_read();
        let record = service
            .get_execution_detail(id)
            .map_err(|e| HeartbeatToolError::ExecutionFailed(format!("获取详情失败: {}", e)))?;
        Ok(record.map(|r| Self::convert_execution(&r)))
    }

    async fn trigger_now(&self) -> Result<HeartbeatCycleResult, HeartbeatToolError> {
        let app_data_dir = self.app_data_dir()?;
        let result = {
            let service = self.hb_service.read().await;
            service
                .trigger_now(app_data_dir, Some(self.app_handle.clone()))
                .await
        };

        // 更新状态
        {
            let mut service = self.hb_service.write().await;
            service.update_status_after_cycle(&CycleResult {
                task_count: result.task_count,
                success_count: result.success_count,
                failed_count: result.failed_count,
                timeout_count: result.timeout_count,
            });
        }

        Ok(HeartbeatCycleResult {
            task_count: result.task_count,
            success_count: result.success_count,
            failed_count: result.failed_count,
            timeout_count: result.timeout_count,
        })
    }
}
