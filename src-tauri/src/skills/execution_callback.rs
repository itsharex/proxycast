//! Tauri 执行回调实现
//!
//! 通过 Tauri 事件系统向前端发送 Skill 执行进度更新。

use std::sync::atomic::{AtomicUsize, Ordering};
use tauri::{AppHandle, Emitter};

use proxycast_skills::{
    events, ExecutionCallback, ExecutionCompletePayload, StepCompletePayload, StepErrorPayload,
    StepStartPayload,
};

/// Tauri 执行回调
///
/// 通过 Tauri 事件系统向前端发送 Skill 执行进度更新。
pub struct TauriExecutionCallback {
    app_handle: AppHandle,
    execution_id: String,
    current_step: AtomicUsize,
}

impl TauriExecutionCallback {
    pub fn new(app_handle: AppHandle, execution_id: String) -> Self {
        Self {
            app_handle,
            execution_id,
            current_step: AtomicUsize::new(0),
        }
    }

    pub fn execution_id(&self) -> &str {
        &self.execution_id
    }

    pub fn current_step(&self) -> usize {
        self.current_step.load(Ordering::SeqCst)
    }
}

impl ExecutionCallback for TauriExecutionCallback {
    fn on_step_start(
        &self,
        step_id: &str,
        step_name: &str,
        current_step: usize,
        total_steps: usize,
    ) {
        self.current_step.store(current_step, Ordering::SeqCst);

        let payload = StepStartPayload {
            execution_id: self.execution_id.clone(),
            step_id: step_id.to_string(),
            step_name: step_name.to_string(),
            current_step,
            total_steps,
        };

        tracing::info!(
            "[TauriExecutionCallback] 步骤开始: execution_id={}, step_id={}, step_name={}, {}/{}",
            self.execution_id,
            step_id,
            step_name,
            current_step,
            total_steps
        );

        if let Err(e) = self.app_handle.emit(events::STEP_START, &payload) {
            tracing::error!(
                "[TauriExecutionCallback] 发送 {} 事件失败: {}",
                events::STEP_START,
                e
            );
        }
    }

    fn on_step_complete(&self, step_id: &str, output: &str) {
        let payload = StepCompletePayload {
            execution_id: self.execution_id.clone(),
            step_id: step_id.to_string(),
            output: output.to_string(),
        };

        tracing::info!(
            "[TauriExecutionCallback] 步骤完成: execution_id={}, step_id={}, output_len={}",
            self.execution_id,
            step_id,
            output.len()
        );

        if let Err(e) = self.app_handle.emit(events::STEP_COMPLETE, &payload) {
            tracing::error!(
                "[TauriExecutionCallback] 发送 {} 事件失败: {}",
                events::STEP_COMPLETE,
                e
            );
        }
    }

    fn on_step_error(&self, step_id: &str, error: &str, will_retry: bool) {
        let payload = StepErrorPayload {
            execution_id: self.execution_id.clone(),
            step_id: step_id.to_string(),
            error: error.to_string(),
            will_retry,
        };

        tracing::warn!(
            "[TauriExecutionCallback] 步骤错误: execution_id={}, step_id={}, error={}, will_retry={}",
            self.execution_id, step_id, error, will_retry
        );

        if let Err(e) = self.app_handle.emit(events::STEP_ERROR, &payload) {
            tracing::error!(
                "[TauriExecutionCallback] 发送 {} 事件失败: {}",
                events::STEP_ERROR,
                e
            );
        }
    }

    fn on_complete(&self, success: bool, final_output: Option<&str>, error: Option<&str>) {
        let payload = ExecutionCompletePayload {
            execution_id: self.execution_id.clone(),
            success,
            output: final_output.map(|s| s.to_string()),
            error: error.map(|s| s.to_string()),
        };

        if success {
            tracing::info!(
                "[TauriExecutionCallback] 执行完成: execution_id={}, success=true, output_len={}",
                self.execution_id,
                final_output.map(|s| s.len()).unwrap_or(0)
            );
        } else {
            tracing::warn!(
                "[TauriExecutionCallback] 执行失败: execution_id={}, error={:?}",
                self.execution_id,
                error
            );
        }

        if let Err(e) = self.app_handle.emit(events::COMPLETE, &payload) {
            tracing::error!(
                "[TauriExecutionCallback] 发送 {} 事件失败: {}",
                events::COMPLETE,
                e
            );
        }
    }
}
