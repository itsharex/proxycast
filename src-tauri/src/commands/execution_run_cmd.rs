//! 统一执行轨迹查询命令
//!
//! 提供对 `agent_runs` 的只读查询能力，供前端查看 chat / skill / heartbeat 执行摘要。

use crate::database::dao::agent_run::{AgentRun, AgentRunDao, AgentRunStatus};
use crate::database::DbConnection;
use crate::services::execution_tracker_service::ExecutionTracker;
use chrono::Utc;
use serde::Serialize;
use serde_json::Value;
use tauri::State;

const STALE_RUN_TIMEOUT_SECONDS: i64 = 180;

fn parse_run_time(raw: &str) -> Option<chrono::DateTime<Utc>> {
    chrono::DateTime::parse_from_rfc3339(raw)
        .ok()
        .map(|parsed| parsed.with_timezone(&Utc))
}

fn collect_stale_run_ids(runs: &[AgentRun], now: chrono::DateTime<Utc>) -> Vec<String> {
    runs.iter()
        .filter(|run| matches!(run.status, AgentRunStatus::Running | AgentRunStatus::Queued))
        .filter_map(|run| {
            let started_at = parse_run_time(run.started_at.as_str())?;
            let elapsed_seconds = now.signed_duration_since(started_at).num_seconds();
            if elapsed_seconds > STALE_RUN_TIMEOUT_SECONDS {
                Some(run.id.clone())
            } else {
                None
            }
        })
        .collect()
}

fn mark_stale_runs_as_timeout(
    db: &DbConnection,
    stale_run_ids: &[String],
    finished_at: &str,
) -> Result<(), String> {
    if stale_run_ids.is_empty() {
        return Ok(());
    }

    let conn = db.lock().map_err(|e| format!("数据库锁定失败: {e}"))?;
    for run_id in stale_run_ids {
        AgentRunDao::finish_run(
            &conn,
            run_id,
            AgentRunStatus::Timeout,
            finished_at,
            None,
            Some("run_stale_timeout"),
            Some("运行状态已超时，自动回收"),
            None,
        )
        .map_err(|e| format!("回收超时运行记录失败: {e}"))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn execution_run_list(
    db: State<'_, DbConnection>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<AgentRun>, String> {
    let safe_limit = limit.unwrap_or(50).clamp(1, 200);
    let safe_offset = offset.unwrap_or(0);
    let tracker = ExecutionTracker::new(db.inner().clone());
    tracker.list_runs(safe_limit, safe_offset)
}

#[tauri::command]
pub async fn execution_run_get(
    db: State<'_, DbConnection>,
    run_id: String,
) -> Result<Option<AgentRun>, String> {
    let id = run_id.trim();
    if id.is_empty() {
        return Err("run_id 不能为空".to_string());
    }
    let tracker = ExecutionTracker::new(db.inner().clone());
    tracker.get_run(id)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ThemeWorkbenchRunTodoItem {
    pub run_id: String,
    pub execution_id: Option<String>,
    pub session_id: Option<String>,
    pub artifact_paths: Vec<String>,
    pub title: String,
    pub gate_key: String,
    pub status: AgentRunStatus,
    pub source: String,
    pub source_ref: Option<String>,
    pub started_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ThemeWorkbenchRunTerminalItem {
    pub run_id: String,
    pub execution_id: Option<String>,
    pub session_id: Option<String>,
    pub artifact_paths: Vec<String>,
    pub title: String,
    pub gate_key: String,
    pub status: AgentRunStatus,
    pub source: String,
    pub source_ref: Option<String>,
    pub started_at: String,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ThemeWorkbenchRunState {
    pub run_state: String,
    pub current_gate_key: String,
    pub queue_items: Vec<ThemeWorkbenchRunTodoItem>,
    pub latest_terminal: Option<ThemeWorkbenchRunTerminalItem>,
    pub updated_at: String,
}

fn normalize_gate_key(raw: &str) -> Option<String> {
    let normalized = raw.trim().to_lowercase();
    match normalized.as_str() {
        "topic_select" | "write_mode" | "publish_confirm" => Some(normalized),
        _ => None,
    }
}

fn infer_gate_key_from_probe(probe: &str) -> String {
    let normalized = probe.to_lowercase();
    if normalized.contains("publish")
        || normalized.contains("adapt")
        || normalized.contains("distribution")
        || normalized.contains("release")
        || normalized.contains("发布")
        || normalized.contains("分发")
        || normalized.contains("平台适配")
    {
        return "publish_confirm".to_string();
    }
    if normalized.contains("topic")
        || normalized.contains("research")
        || normalized.contains("trend")
        || normalized.contains("idea")
        || normalized.contains("选题")
        || normalized.contains("方向")
        || normalized.contains("调研")
        || normalized.contains("洞察")
    {
        return "topic_select".to_string();
    }
    "write_mode".to_string()
}

fn derive_run_title(run: &AgentRun) -> String {
    let parsed_metadata = run
        .metadata
        .as_ref()
        .and_then(|raw| serde_json::from_str::<Value>(raw).ok());

    let skill_title = parsed_metadata
        .as_ref()
        .and_then(|value| value.get("skill_name"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("执行技能 {value}"));
    if let Some(title) = skill_title {
        return title;
    }

    let task_title = parsed_metadata
        .as_ref()
        .and_then(|value| value.get("task_name"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("执行任务 {value}"));
    if let Some(title) = task_title {
        return title;
    }

    let source_ref_title = run
        .source_ref
        .as_ref()
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("运行节点 {value}"));
    if let Some(title) = source_ref_title {
        return title;
    }

    match run.source.as_str() {
        "skill" => "执行主题工作台技能".to_string(),
        "heartbeat" => "执行定时任务".to_string(),
        _ => "执行主题工作台编排".to_string(),
    }
}

fn derive_run_gate_key(run: &AgentRun, title: &str) -> String {
    let parsed_metadata = run
        .metadata
        .as_ref()
        .and_then(|raw| serde_json::from_str::<Value>(raw).ok());

    if let Some(value) = parsed_metadata
        .as_ref()
        .and_then(|value| value.get("gate_key"))
        .and_then(Value::as_str)
        .and_then(normalize_gate_key)
    {
        return value;
    }

    let metadata_probe = parsed_metadata
        .as_ref()
        .map(|value| value.to_string())
        .unwrap_or_default();
    let source_ref_probe = run.source_ref.clone().unwrap_or_default();
    let probe = format!(
        "{} {} {} {}",
        title, source_ref_probe, run.source, metadata_probe
    );
    infer_gate_key_from_probe(probe.as_str())
}

fn derive_current_gate_key(queue_items: &[ThemeWorkbenchRunTodoItem]) -> String {
    queue_items
        .iter()
        .find(|item| item.status == AgentRunStatus::Running)
        .map(|item| item.gate_key.clone())
        .or_else(|| queue_items.first().map(|item| item.gate_key.clone()))
        .unwrap_or_else(|| "idle".to_string())
}

fn derive_run_execution_id(run: &AgentRun) -> Option<String> {
    let parsed_metadata = run
        .metadata
        .as_ref()
        .and_then(|raw| serde_json::from_str::<Value>(raw).ok());

    parsed_metadata
        .as_ref()
        .and_then(|value| {
            value
                .get("execution_id")
                .or_else(|| value.get("version_id"))
                .and_then(Value::as_str)
        })
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn derive_run_artifact_paths(run: &AgentRun) -> Vec<String> {
    let parsed_metadata = run
        .metadata
        .as_ref()
        .and_then(|raw| serde_json::from_str::<Value>(raw).ok());

    parsed_metadata
        .as_ref()
        .and_then(|value| value.get("artifact_paths"))
        .and_then(Value::as_array)
        .map(|paths| {
            paths
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|path| !path.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

#[tauri::command]
pub async fn execution_run_get_theme_workbench_state(
    db: State<'_, DbConnection>,
    session_id: String,
    limit: Option<usize>,
) -> Result<ThemeWorkbenchRunState, String> {
    let trimmed_session_id = session_id.trim();
    if trimmed_session_id.is_empty() {
        return Err("session_id 不能为空".to_string());
    }

    let safe_limit = limit.unwrap_or(3).clamp(1, 10);
    let tracker = ExecutionTracker::new(db.inner().clone());
    let mut runs = tracker.list_runs_by_session(trimmed_session_id, safe_limit * 5)?;
    let now = Utc::now();
    let stale_run_ids = collect_stale_run_ids(runs.as_slice(), now);
    if !stale_run_ids.is_empty() {
        mark_stale_runs_as_timeout(db.inner(), stale_run_ids.as_slice(), &now.to_rfc3339())?;
        runs = tracker.list_runs_by_session(trimmed_session_id, safe_limit * 5)?;
    }

    let queue_items: Vec<ThemeWorkbenchRunTodoItem> = runs
        .iter()
        .filter(|run| matches!(run.status, AgentRunStatus::Running | AgentRunStatus::Queued))
        .take(safe_limit)
        .map(|run| {
            let title = derive_run_title(run);
            let gate_key = derive_run_gate_key(run, title.as_str());
            ThemeWorkbenchRunTodoItem {
                run_id: run.id.clone(),
                execution_id: derive_run_execution_id(run),
                session_id: run.session_id.clone(),
                artifact_paths: derive_run_artifact_paths(run),
                title,
                gate_key,
                status: run.status.clone(),
                source: run.source.clone(),
                source_ref: run.source_ref.clone(),
                started_at: run.started_at.clone(),
            }
        })
        .collect();

    let run_state = if queue_items.is_empty() {
        "idle".to_string()
    } else {
        "auto_running".to_string()
    };
    let current_gate_key = derive_current_gate_key(queue_items.as_slice());

    let latest_terminal = runs
        .iter()
        .find(|run| {
            matches!(
                run.status,
                AgentRunStatus::Success
                    | AgentRunStatus::Error
                    | AgentRunStatus::Canceled
                    | AgentRunStatus::Timeout
            )
        })
        .map(|run| {
            let title = derive_run_title(run);
            let gate_key = derive_run_gate_key(run, title.as_str());
            ThemeWorkbenchRunTerminalItem {
                run_id: run.id.clone(),
                execution_id: derive_run_execution_id(run),
                session_id: run.session_id.clone(),
                artifact_paths: derive_run_artifact_paths(run),
                title,
                gate_key,
                status: run.status.clone(),
                source: run.source.clone(),
                source_ref: run.source_ref.clone(),
                started_at: run.started_at.clone(),
                finished_at: run.finished_at.clone(),
            }
        });

    Ok(ThemeWorkbenchRunState {
        run_state,
        current_gate_key,
        queue_items,
        latest_terminal,
        updated_at: Utc::now().to_rfc3339(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_run_with_metadata(metadata: Option<Value>) -> AgentRun {
        AgentRun {
            id: "run-test-1".to_string(),
            source: "skill".to_string(),
            source_ref: Some("topic_research".to_string()),
            session_id: Some("session-test".to_string()),
            status: AgentRunStatus::Running,
            started_at: "2026-03-06T00:00:00Z".to_string(),
            finished_at: None,
            duration_ms: None,
            error_code: None,
            error_message: None,
            metadata: metadata.map(|raw| raw.to_string()),
            created_at: "2026-03-06T00:00:00Z".to_string(),
            updated_at: "2026-03-06T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn derive_run_gate_key_should_prefer_metadata_gate_key() {
        let run = sample_run_with_metadata(Some(serde_json::json!({
            "gate_key": "publish_confirm"
        })));
        let gate_key = derive_run_gate_key(&run, "任意标题");
        assert_eq!(gate_key, "publish_confirm");
    }

    #[test]
    fn derive_run_gate_key_should_fallback_to_probe_inference() {
        let run = sample_run_with_metadata(None);
        let gate_key = derive_run_gate_key(&run, "执行选题调研");
        assert_eq!(gate_key, "topic_select");
    }

    #[test]
    fn normalize_gate_key_should_reject_unknown_values() {
        assert_eq!(
            normalize_gate_key("write_mode"),
            Some("write_mode".to_string())
        );
        assert!(normalize_gate_key("unknown_gate").is_none());
    }

    #[test]
    fn derive_current_gate_key_should_prefer_running_item() {
        let queue_items = vec![
            ThemeWorkbenchRunTodoItem {
                run_id: "run-1".to_string(),
                execution_id: None,
                session_id: None,
                artifact_paths: vec![],
                title: "选题调研".to_string(),
                gate_key: "topic_select".to_string(),
                status: AgentRunStatus::Queued,
                source: "skill".to_string(),
                source_ref: None,
                started_at: "2026-03-06T00:00:00Z".to_string(),
            },
            ThemeWorkbenchRunTodoItem {
                run_id: "run-2".to_string(),
                execution_id: None,
                session_id: None,
                artifact_paths: vec![],
                title: "写作中".to_string(),
                gate_key: "write_mode".to_string(),
                status: AgentRunStatus::Running,
                source: "skill".to_string(),
                source_ref: None,
                started_at: "2026-03-06T00:00:01Z".to_string(),
            },
        ];

        assert_eq!(
            derive_current_gate_key(queue_items.as_slice()),
            "write_mode".to_string()
        );
    }

    #[test]
    fn derive_current_gate_key_should_fallback_to_first_item() {
        let queue_items = vec![ThemeWorkbenchRunTodoItem {
            run_id: "run-1".to_string(),
            execution_id: None,
            session_id: None,
            artifact_paths: vec![],
            title: "选题调研".to_string(),
            gate_key: "topic_select".to_string(),
            status: AgentRunStatus::Queued,
            source: "skill".to_string(),
            source_ref: None,
            started_at: "2026-03-06T00:00:00Z".to_string(),
        }];

        assert_eq!(
            derive_current_gate_key(queue_items.as_slice()),
            "topic_select".to_string()
        );
    }

    #[test]
    fn derive_current_gate_key_should_return_idle_when_empty() {
        assert_eq!(derive_current_gate_key(&[]), "idle".to_string());
    }

    #[test]
    fn derive_run_execution_id_should_prefer_metadata_execution_id() {
        let run = sample_run_with_metadata(Some(serde_json::json!({
            "execution_id": "exec-12345",
            "version_id": "version-legacy",
        })));
        assert_eq!(
            derive_run_execution_id(&run),
            Some("exec-12345".to_string())
        );
    }

    #[test]
    fn derive_run_artifact_paths_should_parse_non_empty_paths() {
        let run = sample_run_with_metadata(Some(serde_json::json!({
            "artifact_paths": [
                "social-posts/demo.md",
                " ",
                "social-posts/demo.cover.json"
            ],
        })));

        assert_eq!(
            derive_run_artifact_paths(&run),
            vec![
                "social-posts/demo.md".to_string(),
                "social-posts/demo.cover.json".to_string(),
            ]
        );
    }

    #[test]
    fn collect_stale_run_ids_should_only_pick_expired_non_terminal_runs() {
        let now = Utc::now();
        let stale_started_at =
            (now - chrono::Duration::seconds(STALE_RUN_TIMEOUT_SECONDS + 20)).to_rfc3339();
        let fresh_started_at = (now - chrono::Duration::seconds(10)).to_rfc3339();

        let stale_run = AgentRun {
            id: "run-stale".to_string(),
            source: "chat".to_string(),
            source_ref: Some("aster_agent_chat_stream".to_string()),
            session_id: Some("session-1".to_string()),
            status: AgentRunStatus::Running,
            started_at: stale_started_at.clone(),
            finished_at: None,
            duration_ms: None,
            error_code: None,
            error_message: None,
            metadata: None,
            created_at: stale_started_at.clone(),
            updated_at: stale_started_at,
        };
        let fresh_run = AgentRun {
            id: "run-fresh".to_string(),
            source: "chat".to_string(),
            source_ref: Some("aster_agent_chat_stream".to_string()),
            session_id: Some("session-1".to_string()),
            status: AgentRunStatus::Queued,
            started_at: fresh_started_at.clone(),
            finished_at: None,
            duration_ms: None,
            error_code: None,
            error_message: None,
            metadata: None,
            created_at: fresh_started_at.clone(),
            updated_at: fresh_started_at,
        };
        let terminal_run = AgentRun {
            id: "run-terminal".to_string(),
            source: "chat".to_string(),
            source_ref: Some("aster_agent_chat_stream".to_string()),
            session_id: Some("session-1".to_string()),
            status: AgentRunStatus::Success,
            started_at: now.to_rfc3339(),
            finished_at: Some(now.to_rfc3339()),
            duration_ms: Some(1200),
            error_code: None,
            error_message: None,
            metadata: None,
            created_at: now.to_rfc3339(),
            updated_at: now.to_rfc3339(),
        };

        let stale_ids = collect_stale_run_ids(&[stale_run, fresh_run, terminal_run], now);
        assert_eq!(stale_ids, vec!["run-stale".to_string()]);
    }
}
