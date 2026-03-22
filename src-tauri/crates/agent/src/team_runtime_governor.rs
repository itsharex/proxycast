use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, OnceLock};
use tokio::sync::{Mutex, Notify};

const DEFAULT_TEAM_GLOBAL_MAX_PARALLEL: usize = 2;
const HIGH_RISK_PROVIDER_MAX_PARALLEL: usize = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TeamRuntimeGovernorSnapshot {
    pub team_phase: String,
    pub team_parallel_budget: usize,
    pub team_active_count: usize,
    pub team_queued_count: usize,
    pub provider_concurrency_group: String,
    pub provider_parallel_budget: usize,
    pub queue_reason: Option<String>,
    pub retryable_overload: bool,
}

#[derive(Debug, Clone)]
struct TeamRuntimeActiveLease {
    parent_session_id: String,
    provider_group: String,
}

#[derive(Debug, Clone)]
struct TeamRuntimeWaiter {
    waiter_id: u64,
    session_id: String,
    parent_session_id: String,
    provider_group: String,
    notify: Arc<Notify>,
}

#[derive(Debug, Default)]
struct TeamRuntimeGovernorState {
    next_waiter_id: u64,
    active_by_session_id: HashMap<String, TeamRuntimeActiveLease>,
    waiters: VecDeque<TeamRuntimeWaiter>,
}

#[derive(Debug)]
struct TeamRuntimeGovernorInner {
    state: Mutex<TeamRuntimeGovernorState>,
}

#[derive(Debug)]
pub struct TeamRuntimePermit {
    session_id: String,
}

#[derive(Debug)]
struct WaitingRegistration {
    governor: Arc<TeamRuntimeGovernorInner>,
    waiter_id: u64,
    disarmed: bool,
}

impl WaitingRegistration {
    fn new(governor: Arc<TeamRuntimeGovernorInner>, waiter_id: u64) -> Self {
        Self {
            governor,
            waiter_id,
            disarmed: false,
        }
    }

    fn disarm(&mut self) {
        self.disarmed = true;
    }
}

impl Drop for WaitingRegistration {
    fn drop(&mut self) {
        if self.disarmed {
            return;
        }

        let governor = self.governor.clone();
        let waiter_id = self.waiter_id;
        tokio::spawn(async move {
            let mut state = governor.state.lock().await;
            let Some(index) = state
                .waiters
                .iter()
                .position(|waiter| waiter.waiter_id == waiter_id)
            else {
                return;
            };
            state.waiters.remove(index);
            notify_waiters(&state);
        });
    }
}

fn shared_team_runtime_governor() -> Arc<TeamRuntimeGovernorInner> {
    static SHARED: OnceLock<Arc<TeamRuntimeGovernorInner>> = OnceLock::new();
    SHARED
        .get_or_init(|| {
            Arc::new(TeamRuntimeGovernorInner {
                state: Mutex::new(TeamRuntimeGovernorState::default()),
            })
        })
        .clone()
}

fn notify_waiters(state: &TeamRuntimeGovernorState) {
    for waiter in &state.waiters {
        waiter.notify.notify_one();
    }
}

fn normalize_provider_group(provider_group: &str) -> String {
    let normalized = provider_group.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return "default".to_string();
    }

    if ["glm", "zhipu", "zhipuai", "zai", "bigmodel"]
        .iter()
        .any(|keyword| normalized.contains(keyword))
    {
        return "zhipuai".to_string();
    }

    if normalized.contains("openai") || normalized.starts_with("gpt") {
        return "openai".to_string();
    }

    if normalized.contains("anthropic") || normalized.contains("claude") {
        return "anthropic".to_string();
    }

    if normalized.contains("gemini") || normalized.contains("google") {
        return "google".to_string();
    }

    if normalized.contains("deepseek") {
        return "deepseek".to_string();
    }

    if normalized.contains("qwen") || normalized.contains("tongyi") {
        return "qwen".to_string();
    }

    normalized
}

fn resolve_provider_parallel_budget(provider_group: &str) -> usize {
    if provider_group == "zhipuai" {
        HIGH_RISK_PROVIDER_MAX_PARALLEL
    } else {
        DEFAULT_TEAM_GLOBAL_MAX_PARALLEL
    }
}

fn active_count_for_parent(state: &TeamRuntimeGovernorState, parent_session_id: &str) -> usize {
    state
        .active_by_session_id
        .values()
        .filter(|lease| lease.parent_session_id == parent_session_id)
        .count()
}

fn active_count_for_provider(
    state: &TeamRuntimeGovernorState,
    parent_session_id: &str,
    provider_group: &str,
) -> usize {
    state
        .active_by_session_id
        .values()
        .filter(|lease| {
            lease.parent_session_id == parent_session_id && lease.provider_group == provider_group
        })
        .count()
}

fn queued_count_for_parent(state: &TeamRuntimeGovernorState, parent_session_id: &str) -> usize {
    state
        .waiters
        .iter()
        .filter(|waiter| waiter.parent_session_id == parent_session_id)
        .count()
}

fn build_queue_reason(
    state: &TeamRuntimeGovernorState,
    parent_session_id: &str,
    provider_group: &str,
) -> String {
    let active_count = active_count_for_parent(state, parent_session_id);
    let provider_active_count = active_count_for_provider(state, parent_session_id, provider_group);
    let provider_parallel_budget = resolve_provider_parallel_budget(provider_group);

    if provider_parallel_budget == HIGH_RISK_PROVIDER_MAX_PARALLEL && provider_active_count > 0 {
        return "当前服务在同时处理过多请求时容易直接失败，系统已切换为更稳妥的顺序处理。"
            .to_string();
    }

    if active_count >= DEFAULT_TEAM_GLOBAL_MAX_PARALLEL {
        return format!(
            "当前已有 {} 位协作成员在处理，系统会按顺序继续后续任务。",
            active_count
        );
    }

    "系统正在安排可用的处理窗口，这位协作成员会在前一项完成后继续。".to_string()
}

fn can_activate(
    state: &TeamRuntimeGovernorState,
    parent_session_id: &str,
    provider_group: &str,
) -> bool {
    active_count_for_parent(state, parent_session_id) < DEFAULT_TEAM_GLOBAL_MAX_PARALLEL
        && active_count_for_provider(state, parent_session_id, provider_group)
            < resolve_provider_parallel_budget(provider_group)
}

fn is_waiter_turn(
    state: &TeamRuntimeGovernorState,
    waiter_id: u64,
    parent_session_id: &str,
    provider_group: &str,
) -> bool {
    let Some(first_ready_waiter) = state.waiters.iter().find(|candidate| {
        can_activate(
            state,
            &candidate.parent_session_id,
            &candidate.provider_group,
        )
    }) else {
        return false;
    };

    first_ready_waiter.waiter_id == waiter_id
        && first_ready_waiter.parent_session_id == parent_session_id
        && first_ready_waiter.provider_group == provider_group
}

fn activate_session(
    state: &mut TeamRuntimeGovernorState,
    session_id: String,
    parent_session_id: String,
    provider_group: String,
) {
    state.active_by_session_id.insert(
        session_id,
        TeamRuntimeActiveLease {
            parent_session_id,
            provider_group,
        },
    );
}

pub fn normalize_team_runtime_provider_group(provider_group: &str) -> String {
    normalize_provider_group(provider_group)
}

pub fn resolve_team_runtime_provider_parallel_budget(provider_group: &str) -> usize {
    resolve_provider_parallel_budget(&normalize_provider_group(provider_group))
}

pub fn default_team_runtime_parallel_budget() -> usize {
    DEFAULT_TEAM_GLOBAL_MAX_PARALLEL
}

pub async fn preview_team_runtime_wait_snapshot(
    parent_session_id: &str,
    provider_group: &str,
) -> Option<TeamRuntimeGovernorSnapshot> {
    let governor = shared_team_runtime_governor();
    let state = governor.state.lock().await;
    let provider_group = normalize_provider_group(provider_group);
    if can_activate(&state, parent_session_id, &provider_group) {
        return None;
    }

    let provider_parallel_budget = resolve_provider_parallel_budget(&provider_group);
    Some(TeamRuntimeGovernorSnapshot {
        team_phase: "queued".to_string(),
        team_parallel_budget: DEFAULT_TEAM_GLOBAL_MAX_PARALLEL,
        team_active_count: active_count_for_parent(&state, parent_session_id),
        team_queued_count: queued_count_for_parent(&state, parent_session_id) + 1,
        provider_concurrency_group: provider_group.clone(),
        provider_parallel_budget,
        queue_reason: Some(build_queue_reason(
            &state,
            parent_session_id,
            &provider_group,
        )),
        retryable_overload: provider_parallel_budget == HIGH_RISK_PROVIDER_MAX_PARALLEL,
    })
}

pub async fn acquire_team_runtime_permit(
    session_id: impl Into<String>,
    parent_session_id: impl Into<String>,
    provider_group: impl Into<String>,
) -> TeamRuntimePermit {
    let governor = shared_team_runtime_governor();
    let session_id = session_id.into();
    let parent_session_id = parent_session_id.into();
    let provider_group = normalize_provider_group(&provider_group.into());

    let mut state = governor.state.lock().await;
    if can_activate(&state, &parent_session_id, &provider_group) {
        activate_session(
            &mut state,
            session_id.clone(),
            parent_session_id.clone(),
            provider_group.clone(),
        );
        notify_waiters(&state);
        return TeamRuntimePermit { session_id };
    }

    let waiter_id = state.next_waiter_id;
    state.next_waiter_id += 1;
    let notify = Arc::new(Notify::new());
    state.waiters.push_back(TeamRuntimeWaiter {
        waiter_id,
        session_id: session_id.clone(),
        parent_session_id: parent_session_id.clone(),
        provider_group: provider_group.clone(),
        notify: notify.clone(),
    });
    let mut registration = WaitingRegistration::new(governor.clone(), waiter_id);
    drop(state);

    loop {
        notify.notified().await;
        let mut state = governor.state.lock().await;
        if !is_waiter_turn(&state, waiter_id, &parent_session_id, &provider_group) {
            continue;
        }
        let Some(index) = state
            .waiters
            .iter()
            .position(|waiter| waiter.waiter_id == waiter_id)
        else {
            continue;
        };
        state.waiters.remove(index);
        activate_session(
            &mut state,
            session_id.clone(),
            parent_session_id.clone(),
            provider_group.clone(),
        );
        notify_waiters(&state);
        registration.disarm();
        return TeamRuntimePermit { session_id };
    }
}

pub async fn release_team_runtime_permit(permit: TeamRuntimePermit) {
    let governor = shared_team_runtime_governor();
    let mut state = governor.state.lock().await;
    let removed = state.active_by_session_id.remove(&permit.session_id);
    if removed.is_some() {
        notify_waiters(&state);
    }
}

pub async fn snapshot_team_runtime_session(
    session_id: &str,
) -> Option<TeamRuntimeGovernorSnapshot> {
    let governor = shared_team_runtime_governor();
    let state = governor.state.lock().await;

    if let Some(active) = state.active_by_session_id.get(session_id) {
        let team_active_count = active_count_for_parent(&state, &active.parent_session_id);
        let team_queued_count = queued_count_for_parent(&state, &active.parent_session_id);
        let provider_parallel_budget = resolve_provider_parallel_budget(&active.provider_group);
        return Some(TeamRuntimeGovernorSnapshot {
            team_phase: "running".to_string(),
            team_parallel_budget: DEFAULT_TEAM_GLOBAL_MAX_PARALLEL,
            team_active_count,
            team_queued_count,
            provider_concurrency_group: active.provider_group.clone(),
            provider_parallel_budget,
            queue_reason: None,
            retryable_overload: provider_parallel_budget == HIGH_RISK_PROVIDER_MAX_PARALLEL,
        });
    }

    let waiter = state
        .waiters
        .iter()
        .find(|waiter| waiter.session_id == session_id)?;
    let team_active_count = active_count_for_parent(&state, &waiter.parent_session_id);
    let team_queued_count = queued_count_for_parent(&state, &waiter.parent_session_id);
    let provider_parallel_budget = resolve_provider_parallel_budget(&waiter.provider_group);
    Some(TeamRuntimeGovernorSnapshot {
        team_phase: "queued".to_string(),
        team_parallel_budget: DEFAULT_TEAM_GLOBAL_MAX_PARALLEL,
        team_active_count,
        team_queued_count,
        provider_concurrency_group: waiter.provider_group.clone(),
        provider_parallel_budget,
        queue_reason: Some(build_queue_reason(
            &state,
            &waiter.parent_session_id,
            &waiter.provider_group,
        )),
        retryable_overload: provider_parallel_budget == HIGH_RISK_PROVIDER_MAX_PARALLEL,
    })
}

#[cfg(test)]
pub async fn reset_team_runtime_governor() {
    let governor = shared_team_runtime_governor();
    let mut state = governor.state.lock().await;
    state.active_by_session_id.clear();
    state.waiters.clear();
    state.next_waiter_id = 0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::task::yield_now;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn high_risk_provider_should_wait_for_next_slot() {
        reset_team_runtime_governor().await;

        let permit_1 = acquire_team_runtime_permit("session-1", "parent-1", "glm-4.7").await;

        let waiter = tokio::spawn(async {
            acquire_team_runtime_permit("session-2", "parent-1", "zhipuai").await
        });

        yield_now().await;

        let snapshot = snapshot_team_runtime_session("session-2")
            .await
            .expect("snapshot should exist while waiting");
        assert_eq!(snapshot.team_phase, "queued");
        assert_eq!(snapshot.provider_concurrency_group, "zhipuai");
        assert_eq!(snapshot.provider_parallel_budget, 1);
        assert_eq!(snapshot.team_parallel_budget, 2);
        assert!(!waiter.is_finished());

        release_team_runtime_permit(permit_1).await;
        let permit_2 = timeout(Duration::from_secs(1), waiter)
            .await
            .expect("second acquire should finish")
            .expect("spawn should succeed");
        release_team_runtime_permit(permit_2).await;
    }

    #[tokio::test]
    async fn regular_provider_should_allow_two_parallel_members() {
        reset_team_runtime_governor().await;

        let permit_1 = acquire_team_runtime_permit("session-a", "parent-main", "openai").await;
        let permit_2 = acquire_team_runtime_permit("session-b", "parent-main", "anthropic").await;

        let waiter = tokio::spawn(async {
            acquire_team_runtime_permit("session-c", "parent-main", "openai").await
        });

        yield_now().await;

        let snapshot = snapshot_team_runtime_session("session-c")
            .await
            .expect("third session should be queued");
        assert_eq!(snapshot.team_phase, "queued");
        assert_eq!(snapshot.team_active_count, 2);
        assert!(!waiter.is_finished());

        release_team_runtime_permit(permit_2).await;
        let permit_3 = timeout(Duration::from_secs(1), waiter)
            .await
            .expect("third permit should finish")
            .expect("spawn should succeed");
        release_team_runtime_permit(permit_1).await;
        release_team_runtime_permit(permit_3).await;
    }
}
