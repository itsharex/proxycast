use crate::aster_runtime_support::{list_aster_runtime_queued_turns, load_aster_runtime_snapshot};
use crate::team_runtime_governor::snapshot_team_runtime_session;
use aster::session::extension_data::{ExtensionData, ExtensionState};
use aster::session::{
    list_subagent_sessions_with_metadata, require_shared_session_runtime_queue_service,
    resolve_subagent_session_metadata, QueuedTurnRuntime, Session, SessionManager, SessionType,
    TurnStatus,
};
use chrono::Utc;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default, PartialEq)]
pub struct SubagentControlState {
    #[serde(default)]
    pub closed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stashed_queued_turns: Vec<QueuedTurnRuntime>,
}

impl ExtensionState for SubagentControlState {
    const EXTENSION_NAME: &'static str = "subagent_control";
    const VERSION: &'static str = "v0";
}

impl SubagentControlState {
    pub fn from_extension_data(extension_data: &ExtensionData) -> Option<Self> {
        <Self as ExtensionState>::from_extension_data(extension_data)
    }

    pub fn from_session(session: &Session) -> Option<Self> {
        Self::from_extension_data(&session.extension_data)
    }

    pub fn to_extension_data(&self, extension_data: &mut ExtensionData) -> Result<(), String> {
        <Self as ExtensionState>::to_extension_data(self, extension_data)
            .map_err(|error| error.to_string())
    }

    pub fn into_updated_extension_data(self, session: &Session) -> Result<ExtensionData, String> {
        let mut extension_data = session.extension_data.clone();
        self.to_extension_data(&mut extension_data)?;
        Ok(extension_data)
    }

    pub fn closed(reason: Option<String>, stashed_queued_turns: Vec<QueuedTurnRuntime>) -> Self {
        Self {
            closed: true,
            closed_at: Some(Utc::now().to_rfc3339()),
            closed_reason: normalize_optional_text(reason),
            stashed_queued_turns,
        }
    }

    pub fn opened(mut self) -> Self {
        self.closed = false;
        self.closed_at = None;
        self.closed_reason = None;
        self
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubagentRuntimeStatusKind {
    Idle,
    Queued,
    Running,
    Completed,
    Failed,
    Aborted,
    Closed,
    NotFound,
}

impl SubagentRuntimeStatusKind {
    pub fn is_final(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Aborted | Self::Closed | Self::NotFound
        )
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct SubagentRuntimeStatus {
    pub session_id: String,
    pub kind: SubagentRuntimeStatusKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_turn_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_turn_status: Option<SubagentRuntimeStatusKind>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub queued_turn_count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_phase: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_parallel_budget: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_active_count: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_queued_count: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_concurrency_group: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_parallel_budget: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queue_reason: Option<String>,
    #[serde(default)]
    pub retryable_overload: bool,
    #[serde(default)]
    pub closed: bool,
}

impl SubagentRuntimeStatus {
    fn not_found(session_id: &str) -> Self {
        Self {
            session_id: session_id.to_string(),
            kind: SubagentRuntimeStatusKind::NotFound,
            latest_turn_id: None,
            latest_turn_status: None,
            queued_turn_count: 0,
            team_phase: None,
            team_parallel_budget: None,
            team_active_count: None,
            team_queued_count: None,
            provider_concurrency_group: None,
            provider_parallel_budget: None,
            queue_reason: None,
            retryable_overload: false,
            closed: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LatestTurnProjection {
    turn_id: String,
    status: TurnStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubagentRuntimeStatusInput {
    pub closed: bool,
    pub has_active_turn: bool,
    pub queued_turn_count: usize,
    pub latest_turn_status: Option<TurnStatus>,
}

fn is_zero(value: &usize) -> bool {
    *value == 0
}

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    let trimmed = value?.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn looks_like_session_not_found(error: &str) -> bool {
    let normalized = error.to_ascii_lowercase();
    normalized.contains("not found") || error.contains("不存在")
}

fn ensure_subagent_session(session: &Session) -> Result<(), String> {
    if session.session_type != SessionType::SubAgent {
        return Err(format!(
            "会话不是 subagent session: session_id={}, session_type={}",
            session.id, session.session_type
        ));
    }
    Ok(())
}

fn map_turn_status(status: TurnStatus) -> SubagentRuntimeStatusKind {
    match status {
        TurnStatus::Queued => SubagentRuntimeStatusKind::Queued,
        TurnStatus::Running => SubagentRuntimeStatusKind::Running,
        TurnStatus::Completed => SubagentRuntimeStatusKind::Completed,
        TurnStatus::Failed => SubagentRuntimeStatusKind::Failed,
        TurnStatus::Aborted => SubagentRuntimeStatusKind::Aborted,
    }
}

fn latest_turn_projection(
    snapshot: &aster::session::SessionRuntimeSnapshot,
) -> Option<LatestTurnProjection> {
    snapshot
        .threads
        .iter()
        .flat_map(|thread| thread.turns.iter())
        .max_by(|left, right| {
            left.updated_at
                .cmp(&right.updated_at)
                .then_with(|| left.created_at.cmp(&right.created_at))
                .then_with(|| left.id.cmp(&right.id))
        })
        .map(|turn| LatestTurnProjection {
            turn_id: turn.id.clone(),
            status: turn.status,
        })
}

pub fn derive_subagent_runtime_status_kind(
    input: SubagentRuntimeStatusInput,
) -> SubagentRuntimeStatusKind {
    if input.closed {
        return SubagentRuntimeStatusKind::Closed;
    }

    if input.has_active_turn {
        return SubagentRuntimeStatusKind::Running;
    }

    if input.queued_turn_count > 0 {
        return SubagentRuntimeStatusKind::Queued;
    }

    input
        .latest_turn_status
        .map(map_turn_status)
        .unwrap_or(SubagentRuntimeStatusKind::Idle)
}

pub async fn read_subagent_control_state(
    session_id: &str,
) -> Result<(Session, SubagentControlState), String> {
    let session = SessionManager::get_session(session_id, false)
        .await
        .map_err(|error| format!("读取 subagent session 失败: {error}"))?;
    ensure_subagent_session(&session)?;
    Ok((
        session.clone(),
        SubagentControlState::from_session(&session).unwrap_or_default(),
    ))
}

pub async fn write_subagent_control_state(
    session: &Session,
    control_state: &SubagentControlState,
) -> Result<(), String> {
    ensure_subagent_session(session)?;
    let extension_data = control_state
        .clone()
        .into_updated_extension_data(session)
        .map_err(|error| format!("写入 subagent control state 失败: {error}"))?;
    SessionManager::update_session(&session.id)
        .extension_data(extension_data)
        .apply()
        .await
        .map_err(|error| format!("持久化 subagent control state 失败: {error}"))
}

pub async fn load_subagent_runtime_status(
    session_id: &str,
) -> Result<SubagentRuntimeStatus, String> {
    let session = match SessionManager::get_session(session_id, false).await {
        Ok(session) => session,
        Err(error) => {
            let message = error.to_string();
            if looks_like_session_not_found(&message) {
                return Ok(SubagentRuntimeStatus::not_found(session_id));
            }
            return Err(format!("读取 subagent session 失败: {message}"));
        }
    };
    ensure_subagent_session(&session)?;

    let control_state = SubagentControlState::from_session(&session).unwrap_or_default();
    let latest_turn = match load_aster_runtime_snapshot(session_id).await {
        Ok(snapshot) => latest_turn_projection(&snapshot),
        Err(error) => {
            tracing::debug!(
                "[SubagentControl] 读取 runtime snapshot 失败，按无运行态继续: session_id={}, error={}",
                session_id,
                error
            );
            None
        }
    };

    let queued_turn_count = list_aster_runtime_queued_turns(session_id).await?.len();
    let governor_snapshot = snapshot_team_runtime_session(session_id).await;
    let effective_queued_turn_count = queued_turn_count.max(
        governor_snapshot
            .as_ref()
            .and_then(|snapshot| (snapshot.team_phase == "queued").then_some(1))
            .unwrap_or(0),
    );
    let has_active_turn = require_shared_session_runtime_queue_service()
        .map_err(|error| format!("读取 runtime queue service 失败: {error}"))?
        .has_active_turn(session_id);
    let kind = if governor_snapshot
        .as_ref()
        .map(|snapshot| snapshot.team_phase == "queued")
        .unwrap_or(false)
    {
        SubagentRuntimeStatusKind::Queued
    } else {
        derive_subagent_runtime_status_kind(SubagentRuntimeStatusInput {
            closed: control_state.closed,
            has_active_turn,
            queued_turn_count: effective_queued_turn_count,
            latest_turn_status: latest_turn.as_ref().map(|turn| turn.status),
        })
    };

    Ok(SubagentRuntimeStatus {
        session_id: session_id.to_string(),
        kind,
        latest_turn_id: latest_turn.as_ref().map(|turn| turn.turn_id.clone()),
        latest_turn_status: latest_turn.map(|turn| map_turn_status(turn.status)),
        queued_turn_count: effective_queued_turn_count,
        team_phase: governor_snapshot
            .as_ref()
            .map(|snapshot| snapshot.team_phase.clone()),
        team_parallel_budget: governor_snapshot
            .as_ref()
            .map(|snapshot| snapshot.team_parallel_budget),
        team_active_count: governor_snapshot
            .as_ref()
            .map(|snapshot| snapshot.team_active_count),
        team_queued_count: governor_snapshot
            .as_ref()
            .map(|snapshot| snapshot.team_queued_count),
        provider_concurrency_group: governor_snapshot
            .as_ref()
            .map(|snapshot| snapshot.provider_concurrency_group.clone()),
        provider_parallel_budget: governor_snapshot
            .as_ref()
            .map(|snapshot| snapshot.provider_parallel_budget),
        queue_reason: governor_snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.queue_reason.clone()),
        retryable_overload: governor_snapshot
            .as_ref()
            .map(|snapshot| snapshot.retryable_overload)
            .unwrap_or(false),
        closed: control_state.closed,
    })
}

pub async fn list_subagent_cascade_session_ids(session_id: &str) -> Result<Vec<String>, String> {
    let root_session = SessionManager::get_session(session_id, false)
        .await
        .map_err(|error| format!("读取 subagent session 失败: {error}"))?;
    ensure_subagent_session(&root_session)?;

    let sessions = list_subagent_sessions_with_metadata()
        .await
        .map_err(|error| format!("读取 subagent session 列表失败: {error}"))?;
    Ok(collect_subagent_cascade_session_ids(session_id, &sessions))
}

pub fn collect_subagent_cascade_session_ids(session_id: &str, sessions: &[Session]) -> Vec<String> {
    let mut children_by_parent: HashMap<String, Vec<String>> = HashMap::new();
    for session in sessions {
        let Some(metadata) = resolve_subagent_session_metadata(&session.extension_data) else {
            continue;
        };
        children_by_parent
            .entry(metadata.parent_session_id)
            .or_default()
            .push(session.id.clone());
    }

    let mut ordered = vec![session_id.to_string()];
    let mut queue = VecDeque::from([session_id.to_string()]);
    while let Some(parent_id) = queue.pop_front() {
        let Some(children) = children_by_parent.get(&parent_id) else {
            continue;
        };
        for child_id in children {
            ordered.push(child_id.clone());
            queue.push_back(child_id.clone());
        }
    }
    ordered
}

#[cfg(test)]
mod tests {
    use super::*;
    use aster::session::Session;
    use chrono::{Duration, Utc};

    #[test]
    fn subagent_control_state_roundtrip() {
        let state = SubagentControlState::closed(
            Some("manual_close".to_string()),
            vec![QueuedTurnRuntime {
                queued_turn_id: "queued-1".to_string(),
                session_id: "child-1".to_string(),
                message_preview: "preview".to_string(),
                message_text: "message".to_string(),
                created_at: 1,
                image_count: 0,
                payload: serde_json::json!({ "message": "test" }),
                metadata: HashMap::new(),
            }],
        );

        let mut extension_data = ExtensionData::default();
        state.to_extension_data(&mut extension_data).unwrap();
        let restored = SubagentControlState::from_extension_data(&extension_data).unwrap();

        assert_eq!(restored, state);
    }

    #[test]
    fn collect_subagent_cascade_session_ids_returns_breadth_first_tree() {
        let now = Utc::now();
        let child_a = Session {
            id: "child-a".to_string(),
            session_type: SessionType::SubAgent,
            updated_at: now,
            extension_data: aster::session::SubagentSessionMetadata::new("root")
                .into_updated_extension_data(&Session::default())
                .unwrap(),
            ..Session::default()
        };
        let child_b = Session {
            id: "child-b".to_string(),
            session_type: SessionType::SubAgent,
            updated_at: now - Duration::minutes(1),
            extension_data: aster::session::SubagentSessionMetadata::new("root")
                .into_updated_extension_data(&Session::default())
                .unwrap(),
            ..Session::default()
        };
        let grandchild = Session {
            id: "grandchild".to_string(),
            session_type: SessionType::SubAgent,
            updated_at: now - Duration::minutes(2),
            extension_data: aster::session::SubagentSessionMetadata::new("child-a")
                .into_updated_extension_data(&Session::default())
                .unwrap(),
            ..Session::default()
        };

        let ids = collect_subagent_cascade_session_ids("root", &[child_a, child_b, grandchild]);

        assert_eq!(ids, vec!["root", "child-a", "child-b", "grandchild"]);
    }

    #[test]
    fn derive_subagent_runtime_status_kind_prioritizes_closed_and_final_states() {
        assert_eq!(
            derive_subagent_runtime_status_kind(SubagentRuntimeStatusInput {
                closed: true,
                has_active_turn: true,
                queued_turn_count: 2,
                latest_turn_status: Some(TurnStatus::Running),
            }),
            SubagentRuntimeStatusKind::Closed
        );
        assert_eq!(
            derive_subagent_runtime_status_kind(SubagentRuntimeStatusInput {
                closed: false,
                has_active_turn: false,
                queued_turn_count: 0,
                latest_turn_status: Some(TurnStatus::Completed),
            }),
            SubagentRuntimeStatusKind::Completed
        );
        assert_eq!(
            derive_subagent_runtime_status_kind(SubagentRuntimeStatusInput {
                closed: false,
                has_active_turn: false,
                queued_turn_count: 1,
                latest_turn_status: Some(TurnStatus::Completed),
            }),
            SubagentRuntimeStatusKind::Queued
        );
    }
}
