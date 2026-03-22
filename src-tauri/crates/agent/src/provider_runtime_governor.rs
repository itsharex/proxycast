use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, OnceLock};
use tokio::sync::{Mutex, Notify};

const HIGH_RISK_PROVIDER_GLOBAL_MAX_PARALLEL: usize = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderRuntimeGovernorSnapshot {
    pub provider_phase: String,
    pub provider_concurrency_group: String,
    pub provider_parallel_budget: usize,
    pub provider_active_count: usize,
    pub provider_queued_count: usize,
    pub queue_reason: Option<String>,
    pub retryable_overload: bool,
}

#[derive(Debug, Clone)]
struct ProviderRuntimeWaiter {
    waiter_id: u64,
    lease_id: String,
    provider_group: String,
    notify: Arc<Notify>,
}

#[derive(Debug, Default)]
struct ProviderRuntimeGovernorState {
    next_waiter_id: u64,
    active_by_lease_id: HashMap<String, String>,
    waiters: VecDeque<ProviderRuntimeWaiter>,
}

#[derive(Debug)]
struct ProviderRuntimeGovernorInner {
    state: Mutex<ProviderRuntimeGovernorState>,
}

#[derive(Debug)]
pub struct ProviderRuntimePermit {
    lease_id: String,
}

#[derive(Debug)]
struct WaitingRegistration {
    governor: Arc<ProviderRuntimeGovernorInner>,
    waiter_id: u64,
    disarmed: bool,
}

impl WaitingRegistration {
    fn new(governor: Arc<ProviderRuntimeGovernorInner>, waiter_id: u64) -> Self {
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

fn shared_provider_runtime_governor() -> Arc<ProviderRuntimeGovernorInner> {
    static SHARED: OnceLock<Arc<ProviderRuntimeGovernorInner>> = OnceLock::new();
    SHARED
        .get_or_init(|| {
            Arc::new(ProviderRuntimeGovernorInner {
                state: Mutex::new(ProviderRuntimeGovernorState::default()),
            })
        })
        .clone()
}

fn notify_waiters(state: &ProviderRuntimeGovernorState) {
    for waiter in &state.waiters {
        waiter.notify.notify_one();
    }
}

pub fn resolve_provider_runtime_parallel_budget(provider_group: &str) -> Option<usize> {
    if provider_group.trim().eq_ignore_ascii_case("zhipuai") {
        Some(HIGH_RISK_PROVIDER_GLOBAL_MAX_PARALLEL)
    } else {
        None
    }
}

fn active_count_for_provider(state: &ProviderRuntimeGovernorState, provider_group: &str) -> usize {
    state
        .active_by_lease_id
        .values()
        .filter(|active_group| active_group.as_str() == provider_group)
        .count()
}

fn queued_count_for_provider(state: &ProviderRuntimeGovernorState, provider_group: &str) -> usize {
    state
        .waiters
        .iter()
        .filter(|waiter| waiter.provider_group == provider_group)
        .count()
}

fn build_queue_reason(provider_group: &str) -> String {
    let _ = provider_group;
    "当前服务在同时处理过多请求时容易直接失败，系统已切换为更稳妥的顺序处理。".to_string()
}

fn can_activate(state: &ProviderRuntimeGovernorState, provider_group: &str) -> bool {
    let Some(provider_parallel_budget) = resolve_provider_runtime_parallel_budget(provider_group)
    else {
        return true;
    };
    active_count_for_provider(state, provider_group) < provider_parallel_budget
}

fn is_waiter_turn(
    state: &ProviderRuntimeGovernorState,
    waiter_id: u64,
    provider_group: &str,
) -> bool {
    let Some(first_ready_waiter) = state
        .waiters
        .iter()
        .find(|candidate| can_activate(state, &candidate.provider_group))
    else {
        return false;
    };

    first_ready_waiter.waiter_id == waiter_id && first_ready_waiter.provider_group == provider_group
}

pub async fn preview_provider_runtime_wait_snapshot(
    provider_group: &str,
) -> Option<ProviderRuntimeGovernorSnapshot> {
    let provider_parallel_budget = resolve_provider_runtime_parallel_budget(provider_group)?;
    let governor = shared_provider_runtime_governor();
    let state = governor.state.lock().await;
    if can_activate(&state, provider_group) {
        return None;
    }

    Some(ProviderRuntimeGovernorSnapshot {
        provider_phase: "queued".to_string(),
        provider_concurrency_group: provider_group.to_string(),
        provider_parallel_budget,
        provider_active_count: active_count_for_provider(&state, provider_group),
        provider_queued_count: queued_count_for_provider(&state, provider_group) + 1,
        queue_reason: Some(build_queue_reason(provider_group)),
        retryable_overload: true,
    })
}

pub async fn acquire_provider_runtime_permit(
    lease_id: impl Into<String>,
    provider_group: impl Into<String>,
) -> ProviderRuntimePermit {
    let governor = shared_provider_runtime_governor();
    let lease_id = lease_id.into();
    let provider_group = provider_group.into();

    let mut state = governor.state.lock().await;
    if can_activate(&state, &provider_group) {
        state
            .active_by_lease_id
            .insert(lease_id.clone(), provider_group.clone());
        notify_waiters(&state);
        return ProviderRuntimePermit { lease_id };
    }

    let waiter_id = state.next_waiter_id;
    state.next_waiter_id += 1;
    let notify = Arc::new(Notify::new());
    state.waiters.push_back(ProviderRuntimeWaiter {
        waiter_id,
        lease_id: lease_id.clone(),
        provider_group: provider_group.clone(),
        notify: notify.clone(),
    });
    let mut registration = WaitingRegistration::new(governor.clone(), waiter_id);
    drop(state);

    loop {
        notify.notified().await;
        let mut state = governor.state.lock().await;
        if !is_waiter_turn(&state, waiter_id, &provider_group) {
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
        state
            .active_by_lease_id
            .insert(lease_id.clone(), provider_group.clone());
        notify_waiters(&state);
        registration.disarm();
        return ProviderRuntimePermit { lease_id };
    }
}

pub async fn release_provider_runtime_permit(permit: ProviderRuntimePermit) {
    let governor = shared_provider_runtime_governor();
    let mut state = governor.state.lock().await;
    if state.active_by_lease_id.remove(&permit.lease_id).is_some() {
        notify_waiters(&state);
    }
}

pub async fn snapshot_provider_runtime_lease(
    lease_id: &str,
) -> Option<ProviderRuntimeGovernorSnapshot> {
    let governor = shared_provider_runtime_governor();
    let state = governor.state.lock().await;

    if let Some(provider_group) = state.active_by_lease_id.get(lease_id) {
        let provider_parallel_budget = resolve_provider_runtime_parallel_budget(provider_group)?;
        return Some(ProviderRuntimeGovernorSnapshot {
            provider_phase: "running".to_string(),
            provider_concurrency_group: provider_group.clone(),
            provider_parallel_budget,
            provider_active_count: active_count_for_provider(&state, provider_group),
            provider_queued_count: queued_count_for_provider(&state, provider_group),
            queue_reason: None,
            retryable_overload: true,
        });
    }

    let waiter = state
        .waiters
        .iter()
        .find(|waiter| waiter.lease_id == lease_id)?;
    let provider_parallel_budget =
        resolve_provider_runtime_parallel_budget(&waiter.provider_group)?;
    Some(ProviderRuntimeGovernorSnapshot {
        provider_phase: "queued".to_string(),
        provider_concurrency_group: waiter.provider_group.clone(),
        provider_parallel_budget,
        provider_active_count: active_count_for_provider(&state, &waiter.provider_group),
        provider_queued_count: queued_count_for_provider(&state, &waiter.provider_group),
        queue_reason: Some(build_queue_reason(&waiter.provider_group)),
        retryable_overload: true,
    })
}

#[cfg(test)]
pub async fn reset_provider_runtime_governor() {
    let governor = shared_provider_runtime_governor();
    let mut state = governor.state.lock().await;
    state.active_by_lease_id.clear();
    state.waiters.clear();
    state.next_waiter_id = 0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::task::yield_now;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn zhipuai_requests_should_run_one_by_one_globally() {
        reset_provider_runtime_governor().await;

        let permit_1 = acquire_provider_runtime_permit("lease-1", "zhipuai").await;
        let waiting_snapshot = preview_provider_runtime_wait_snapshot("zhipuai")
            .await
            .expect("second request should wait");
        assert_eq!(waiting_snapshot.provider_parallel_budget, 1);
        assert_eq!(waiting_snapshot.provider_active_count, 1);
        assert_eq!(waiting_snapshot.provider_queued_count, 1);

        let second =
            tokio::spawn(async { acquire_provider_runtime_permit("lease-2", "zhipuai").await });

        yield_now().await;
        assert!(!second.is_finished());

        release_provider_runtime_permit(permit_1).await;
        let permit_2 = timeout(Duration::from_secs(1), second)
            .await
            .expect("second request should resume")
            .expect("join should succeed");

        let running_snapshot = snapshot_provider_runtime_lease("lease-2")
            .await
            .expect("second request should be active");
        assert_eq!(running_snapshot.provider_phase, "running");
        assert_eq!(running_snapshot.provider_active_count, 1);

        release_provider_runtime_permit(permit_2).await;
    }

    #[test]
    fn only_high_risk_provider_should_use_global_guard() {
        assert_eq!(resolve_provider_runtime_parallel_budget("zhipuai"), Some(1));
        assert_eq!(resolve_provider_runtime_parallel_budget("openai"), None);
    }
}
