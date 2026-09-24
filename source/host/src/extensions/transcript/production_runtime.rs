use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use crate::extensions::session::agent_session::SandAgentSessionStore;
use crate::extensions::session::production::ProductionSessionWorkers;
use crate::sand_activity::{ActivityUpdate, AgentActivity};

use super::async_task_union::{AsyncTask, merge_async_tasks};
use super::prompt_acceptance_ledger::{
    AcceptanceLookup, AcceptanceRecord, AcceptanceStatus, PromptAcceptanceError,
    PromptAcceptanceLedger, SendInput,
};
use super::run_lifecycle::RunLifecycleState;
use super::sand_pending_wake_store::SandPendingWakeStore;
use super::sand_upgrade_resume_store::SandUpgradeResumeStore;
use super::session_runtime::SessionRuntime;
use super::replica_writer::HostReplicaWriter;
use super::run_scheduler::{
    QueueAccepted, QueueDequeued, RUN_WATCHDOG_DEFAULT_MS, RUN_WATCHDOG_GRACE_DEFAULT_MS,
    RunLane, RunSettlement, WatchdogEvent,
};
use super::send_pipeline::{
    HOST_ACCOUNT_SLOT, PersistedSendContext, SendBegin, SendEchoIdentity, SendPipelineState,
};
use super::turn_runtime::{QueuedTurnRecoveryCheck, should_supersede_stale_turn};
use super::send_turn_dispatch::{ProductionTurnDispatch, UserTurnTicket};
use super::upgrade_recreate_resume::{
    UpgradeQuiesceSummary, UpgradeRecreateResume,
};

const COMPLETION_CACHE_MAX: usize = 256;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ProductionSendError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    Rejected(String),
    #[error("{0}")]
    Internal(String),
}

struct RuntimeState {
    pipeline: SendPipelineState,
    ledger: PromptAcceptanceLedger,
    lifecycle: RunLifecycleState,
    turn_dispatch: ProductionTurnDispatch,
    completions: HashMap<String, Result<Value, ProductionSendError>>,
    completion_order: VecDeque<String>,
}

pub struct ProductionTranscriptRuntime {
    state: Mutex<RuntimeState>,
    send_settled: Condvar,
    turn_ready: Condvar,
    pending_wake_store: Option<SandPendingWakeStore>,
    upgrade_resume_store: Option<SandUpgradeResumeStore>,
    upgrade_recreate_resume: UpgradeRecreateResume,
    session_runtime: SessionRuntime,
    replica_writer: HostReplicaWriter,
    roster_snapshot_seq: AtomicU64,
}

impl ProductionTranscriptRuntime {
    pub fn new(root_dir: Option<&Path>) -> Self {
        Self::with_watchdog(
            root_dir,
            RUN_WATCHDOG_DEFAULT_MS,
            RUN_WATCHDOG_GRACE_DEFAULT_MS,
        )
    }

    pub fn with_watchdog(
        root_dir: Option<&Path>,
        watchdog_ms: u64,
        watchdog_grace_ms: u64,
    ) -> Self {
        let pending_wake_store = root_dir.map(SandPendingWakeStore::new);
        let upgrade_resume_store = root_dir.map(SandUpgradeResumeStore::new);
        Self {
            state: Mutex::new(RuntimeState {
                pipeline: SendPipelineState::default(),
                ledger: PromptAcceptanceLedger::new(root_dir),
                lifecycle: RunLifecycleState::default(),
                turn_dispatch: ProductionTurnDispatch::with_watchdog(
                    watchdog_ms,
                    watchdog_grace_ms,
                ),
                completions: HashMap::new(),
                completion_order: VecDeque::new(),
            }),
            send_settled: Condvar::new(),
            turn_ready: Condvar::new(),
            pending_wake_store,
            upgrade_resume_store,
            upgrade_recreate_resume: UpgradeRecreateResume::default(),
            session_runtime: SessionRuntime::new(),
            replica_writer: HostReplicaWriter::new(),
            roster_snapshot_seq: AtomicU64::new(0),
        }
    }

    pub fn session_runtime(&self) -> &SessionRuntime {
        &self.session_runtime
    }

    pub fn switch_agent(
        &self,
        sessions: &std::sync::Arc<ProductionSessionWorkers>,
        agent_id: &str,
        now_ms: f64,
    ) -> Result<Vec<Value>, String> {
        let store = SandAgentSessionStore::new(std::sync::Arc::clone(sessions));
        let previous = store.read_active_agent_id();
        let entries = self
            .session_runtime
            .switch_agent(sessions, agent_id, now_ms)?;
        if let Some(previous) = previous
            .as_deref()
            .filter(|previous| *previous != agent_id)
        {
            let should_retire = {
                let state = self.lock_state();
                !state.lifecycle.is_running(previous)
            };
            if should_retire {
                let _ = self
                    .session_runtime
                    .retire_inactive_session(sessions, previous)?;
            }
        }
        Ok(entries)
    }

    pub fn retire_idle_live_session(
        &self,
        sessions: &Arc<ProductionSessionWorkers>,
        agent_id: &str,
    ) -> Result<bool, String> {
        if self.lock_state().lifecycle.is_running(agent_id) {
            return Ok(false);
        }
        self.session_runtime
            .retire_inactive_session(sessions, agent_id)
    }

    pub fn pending_wake_store(&self) -> Option<&SandPendingWakeStore> {
        self.pending_wake_store.as_ref()
    }

    pub fn upgrade_resume_store(&self) -> Option<&SandUpgradeResumeStore> {
        self.upgrade_resume_store.as_ref()
    }

    pub fn quiesce_for_upgrade(&self) -> UpgradeQuiesceSummary {
        let running_turns = self.lock_state().lifecycle.running_agent_ids().len();
        self.upgrade_recreate_resume
            .quiesce_for_upgrade(running_turns)
    }

    pub fn is_quiescing_for_upgrade(&self) -> bool {
        self.upgrade_recreate_resume.is_quiescing_for_upgrade()
    }

    pub fn resume_after_recreate(&self) {
        self.upgrade_recreate_resume.resume_after_recreate();
    }

    pub fn clear_agent_durable_recovery(&self, agent_id: &str) {
        if let Some(store) = self.pending_wake_store.as_ref() {
            store.clear_agent(agent_id);
        }
        if let Some(store) = self.upgrade_resume_store.as_ref() {
            store.clear(agent_id);
        }
    }

    pub fn get_async_tasks(
        &self,
        agent_id: &str,
        live_tasks: &[AsyncTask],
    ) -> Vec<AsyncTask> {
        let markers = self
            .pending_wake_store
            .as_ref()
            .map(|store| {
                store
                    .list_pending()
                    .into_iter()
                    .filter(|marker| marker.agent_id == agent_id)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        merge_async_tasks(live_tasks, &markers)
    }

    pub fn prompt_acceptance_status(
        &self,
        args: &Value,
    ) -> Result<Value, ProductionSendError> {
        let client_nonce = args
            .get("clientNonce")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                ProductionSendError::BadRequest(
                    "promptAcceptanceStatus requires clientNonce".into(),
                )
            })?;
        let account_slot = args
            .get("accountSlot")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(HOST_ACCOUNT_SLOT);
        let mut state = self.lock_state();
        Ok(match state.ledger.lookup(account_slot, client_nonce) {
            AcceptanceLookup::Found(record) => json!({
                "outcome": "found",
                "record": record,
            }),
            AcceptanceLookup::UnknownDurability => json!({
                "outcome": "unknown-durability",
            }),
            AcceptanceLookup::NotFound => json!({
                "outcome": "not-found",
            }),
        })
    }

    pub fn execute_send<Dispatch, Persist, Persisted>(
        &self,
        args: &Value,
        dispatch: Dispatch,
        persist_accepted: Persist,
    ) -> Result<Value, ProductionSendError>
    where
        Dispatch: FnOnce() -> Result<Value, ProductionSendError>,
        Persist: Fn(&Value) -> Result<Persisted, ProductionSendError>,
        Persisted: Into<PersistedSendContext>,
    {
        self.execute_send_with_watchdog(args, dispatch, persist_accepted, |_| false)
    }

    pub fn execute_send_with_watchdog<Dispatch, Persist, Persisted, Watchdog>(
        &self,
        args: &Value,
        dispatch: Dispatch,
        persist_accepted: Persist,
        on_watchdog: Watchdog,
    ) -> Result<Value, ProductionSendError>
    where
        Dispatch: FnOnce() -> Result<Value, ProductionSendError>,
        Persist: Fn(&Value) -> Result<Persisted, ProductionSendError>,
        Persisted: Into<PersistedSendContext>,
        Watchdog: Fn(&WatchdogEvent) -> bool,
    {
        self.execute_send_with_queue_observers(args, dispatch, persist_accepted, on_watchdog, |_| {}, |_| {})
    }

    pub fn execute_send_with_queue_observers<Dispatch, Persist, Persisted, Watchdog, QueueAcceptedObserver, QueueDequeuedObserver>(
        &self,
        args: &Value,
        dispatch: Dispatch,
        persist_accepted: Persist,
        on_watchdog: Watchdog,
        on_queue_accepted: QueueAcceptedObserver,
        on_queue_dequeued: QueueDequeuedObserver,
    ) -> Result<Value, ProductionSendError>
    where
        Dispatch: FnOnce() -> Result<Value, ProductionSendError>,
        Persist: Fn(&Value) -> Result<Persisted, ProductionSendError>,
        Persisted: Into<PersistedSendContext>,
        Watchdog: Fn(&WatchdogEvent) -> bool,
        QueueAcceptedObserver: Fn(&QueueAccepted),
        QueueDequeuedObserver: Fn(&QueueDequeued),
    {
        let input = parse_send_input(args)?;
        let nonce = optional_non_empty(args, "clientNonce").map(ToOwned::to_owned);
        let agent_id = input.agent_id.clone();
        let is_ack_redrive = optional_bool(args, "ackRedrive")?.unwrap_or(false)
            && optional_non_empty(args, "requestSource") == Some("handoff-resume");
        let dispatch_lane = if is_ack_redrive {
            RunLane::Background
        } else {
            RunLane::User
        };
        let dispatch_source = if is_ack_redrive { "ack-redrive" } else { "turn" };
        let dispatch_ack_token = optional_non_empty(args, "ackToken");
        let is_fork = optional_bool(args, "isFork")?.unwrap_or(false);
        let has_reply_context = optional_non_empty(args, "replyToId").is_some()
            || args.get("replyContext").is_some_and(|value| !value.is_null());
        let attachment_count = input.attachment_paths.len();
        let image_count = args
            .get("selectedImages")
            .and_then(Value::as_array)
            .map(Vec::len)
            .unwrap_or_default();
        let video_count = args
            .get("selectedVideos")
            .and_then(Value::as_array)
            .map(Vec::len)
            .unwrap_or_default();
        let mut turn_ticket: Option<UserTurnTicket> = None;
        let mut turn_generation: Option<u64> = None;
        let mut turn_epoch: Option<u64> = None;
        let mut queue_accepted_event: Option<QueueAccepted> = None;
        let mut queue_dequeued_event: Option<QueueDequeued> = None;
        let mut persisted_echo_entry_id: Option<String> = None;
        let mut persisted_send_context = PersistedSendContext::default();

        let mut state = self.lock_state();
        loop {
            let begin = {
                let RuntimeState {
                    pipeline, ledger, ..
                } = &mut *state;
                pipeline
                    .begin_send(ledger, &input, nonce.as_deref())
                    .map_err(map_acceptance_error)?
            };
            match begin {
                SendBegin::EmptyNoop => return Ok(Value::Null),
                SendBegin::Coalesced { client_nonce } => {
                    state = self
                        .send_settled
                        .wait_while(state, |state| {
                            state.pipeline.is_in_flight(&client_nonce)
                        })
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    if let Some(result) = state.completions.get(&client_nonce) {
                        return result.clone();
                    }
                }
                SendBegin::DuplicateNoop { record } => {
                    if let Some(result) = state.completions.get(&record.client_nonce) {
                        return result.clone();
                    }
                    return Ok(replay_record(&record));
                }
                SendBegin::Dispatch { .. } => {
                    // Frozen Grok durably appends the addressed user echo before
                    // the run enters the per-agent execution queue. This makes
                    // accepted user intent recoverable even if Host/Runner
                    // admission or provider execution fails afterwards.
                    let synthetic_acceptance = json!({ "accepted": true });
                    match persist_accepted(&synthetic_acceptance) {
                        Ok(persisted) => {
                            persisted_send_context = persisted.into();
                            persisted_echo_entry_id =
                                persisted_send_context.echo_entry_id.clone();
                        }
                        Err(error) => {
                            let failure = Err(error.clone());
                            {
                                let RuntimeState {
                                    pipeline, ledger, ..
                                } = &mut *state;
                                pipeline.finish_send(ledger, nonce.as_deref(), false);
                            }
                            if let Some(client_nonce) = nonce.as_deref() {
                                cache_completion(&mut state, client_nonce, failure.clone());
                            }
                            drop(state);
                            self.send_settled.notify_all();
                            return failure;
                        }
                    }

                    if let Some(client_nonce) = nonce.as_deref() {
                        let pending = {
                            let RuntimeState {
                                pipeline, ledger, ..
                            } = &mut *state;
                            pipeline.record_pending_acceptance(
                                ledger,
                                client_nonce,
                                SendEchoIdentity {
                                    agent_id: agent_id.clone().unwrap_or_default(),
                                    echo_entry_id: persisted_echo_entry_id.clone(),
                                },
                            )
                        };
                        if let Err(error) = pending {
                            let error = map_acceptance_error(error);
                            let failure = Err(error.clone());
                            {
                                let RuntimeState {
                                    pipeline, ledger, ..
                                } = &mut *state;
                                pipeline.finish_send(ledger, nonce.as_deref(), false);
                            }
                            cache_completion(&mut state, client_nonce, failure.clone());
                            drop(state);
                            self.send_settled.notify_all();
                            return failure;
                        }
                    }

                    if let Some(agent_id) = agent_id.as_deref() {
                        let accepted_at_ms = system_now_ms();
                        state
                            .lifecycle
                            .begin_session_run(agent_id, accepted_at_ms, false);
                        let epoch = state.pipeline.next_turn_epoch(agent_id);
                        state.pipeline.register_recovery_turn(
                            agent_id,
                            epoch,
                            &persisted_send_context,
                            is_fork,
                        );
                        turn_epoch = Some(epoch);
                        let (ticket, accepted, started) = state
                            .turn_dispatch
                            .enqueue_turn_with_start(
                                agent_id,
                                nonce.as_deref(),
                                accepted_at_ms,
                                accepted_at_ms,
                                dispatch_lane,
                                dispatch_source,
                                dispatch_ack_token,
                            )
                            .map_err(|error| ProductionSendError::Internal(error.to_string()))?;
                        queue_accepted_event = Some(accepted);
                        queue_dequeued_event = started;
                        turn_ticket = Some(ticket);
                    }
                    break;
                }
            }
        }
        if queue_accepted_event.is_some() || queue_dequeued_event.is_some() {
            drop(state);
            if let Some(event) = queue_accepted_event.as_ref() {
                on_queue_accepted(event);
            }
            if let Some(event) = queue_dequeued_event.as_ref() {
                on_queue_dequeued(event);
            }
            state = self.lock_state();
        }

        if let Some(ticket) = turn_ticket.as_ref() {
            loop {
                if let Some(generation) = state.turn_dispatch.active_generation_for(ticket) {
                    turn_generation = Some(generation);
                    break;
                }
                let now_ms = system_now_ms();
                let wait_ms = state
                    .turn_dispatch
                    .watchdog_wait_ms(&ticket.agent_id, now_ms)
                    .unwrap_or(1_000)
                    .max(1);
                let (next_state, _) = self
                    .turn_ready
                    .wait_timeout(state, Duration::from_millis(wait_ms))
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                state = next_state;

                if let Some(tick) = state
                    .turn_dispatch
                    .watchdog_tick(&ticket.agent_id, system_now_ms())
                {
                    let event = tick.event;
                    let started_next = tick.started_next;
                    let escaped = started_next.is_some();
                    drop(state);
                    let _ = on_watchdog(&event);
                    if let Some(event) = started_next.as_ref() {
                        on_queue_dequeued(event);
                    }
                    if escaped {
                        self.turn_ready.notify_all();
                    }
                    state = self.lock_state();
                }
            }
        }
        let suppress_stale_turn = match (agent_id.as_deref(), turn_epoch) {
            (Some(agent_id), Some(epoch)) => should_supersede_stale_turn(
                QueuedTurnRecoveryCheck {
                    epoch,
                    current_epoch: state.pipeline.current_turn_epoch(agent_id),
                    recovery_break_epoch: state.pipeline.recovery_break_epoch(agent_id),
                    prompt: &input.prompt,
                    context: &persisted_send_context,
                    latest_recovery: state.pipeline.latest_recovery_send(agent_id),
                    is_fork,
                    has_reply_context,
                    attachment_count,
                    image_count,
                    video_count,
                },
            ),
            _ => false,
        };
        drop(state);

        let mut result = if suppress_stale_turn {
            Ok(json!({
                "accepted": true,
                "superseded": true,
                "echoEntryId": persisted_send_context.echo_entry_id,
            }))
        } else {
            dispatch()
        };

        let mut state = self.lock_state();
        let accepted = result
            .as_ref()
            .ok()
            .is_some_and(|value| value.get("accepted").and_then(Value::as_bool) == Some(true));

        if accepted {
            if let Some(client_nonce) = nonce.as_deref() {
                let RuntimeState {
                    pipeline, ledger, ..
                } = &mut *state;
                pipeline.mark_send_accepted(ledger, Some(client_nonce));
            }
            if let Some(value) = result.as_ref().ok() {
                let operation_id = value
                    .get("operationId")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToOwned::to_owned);
                if let (Some(agent_id), Some(operation_id)) =
                    (agent_id.as_deref(), operation_id.as_deref())
                {
                    state.lifecycle.record_request_id(agent_id, operation_id);
                }
            }
        }

        let succeeded = result
            .as_ref()
            .ok()
            .is_some_and(|value| value.get("accepted").and_then(Value::as_bool) == Some(true));
        {
            let RuntimeState {
                pipeline, ledger, ..
            } = &mut *state;
            pipeline.finish_send(ledger, nonce.as_deref(), succeeded);
        }
        let settled_at_ms = system_now_ms();
        let mut terminal_watchdog_event = None;
        let mut settled_dequeued_event = None;
        if let (Some(ticket), Some(generation)) =
            (turn_ticket.as_ref(), turn_generation)
        {
            let (settlement, next) = state
                .turn_dispatch
                .settle_and_start_next(ticket, generation, settled_at_ms);
            settled_dequeued_event = next;
            if let RunSettlement::ZombieSettled { watchdog, .. } = settlement {
                terminal_watchdog_event = Some(watchdog);
            }
        }
        if let Some(agent_id) = agent_id.as_deref() {
            let _ = state.lifecycle.end_session_run(agent_id, settled_at_ms);
        }
        if let Some(client_nonce) = nonce.as_deref() {
            cache_completion(&mut state, client_nonce, result.clone());
        }
        drop(state);
        if let Some(event) = terminal_watchdog_event.as_ref() {
            let _ = on_watchdog(event);
        }
        if let Some(event) = settled_dequeued_event.as_ref() {
            on_queue_dequeued(event);
        }
        self.turn_ready.notify_all();
        self.send_settled.notify_all();
        result
    }

    pub fn begin_provider_run(&self, agent_id: &str) {
        self.lock_state().lifecycle.begin_provider_run(agent_id);
    }

    pub fn end_provider_run(&self, agent_id: &str) {
        self.lock_state().lifecycle.end_provider_run(agent_id);
    }

    pub fn track_runner_activity_update(
        &self,
        agent_id: &str,
        update: &ActivityUpdate,
        now_ms: u64,
    ) {
        let mut state = self.lock_state();
        state.lifecycle.track_activity_from_update(agent_id, update, now_ms);
        state.lifecycle.track_composing_from_update(agent_id, update);
        state.lifecycle.track_retrying_from_update(agent_id, update);
    }

    pub fn decorate_agent_summaries(&self, value: &mut Value) {
        let snapshot_epoch = self.replica_writer.process_epoch().to_string();
        let snapshot_seq = self
            .roster_snapshot_seq
            .fetch_add(1, Ordering::SeqCst)
            .saturating_add(1);
        let state = self.lock_state();
        let Some(rows) = value.as_array_mut() else {
            return;
        };
        for row in rows {
            let Some(object) = row.as_object_mut() else {
                continue;
            };
            let Some(agent_id) = object.get("id").and_then(Value::as_str).map(str::to_string) else {
                continue;
            };
            let running = state.lifecycle.is_running(&agent_id);
            object.insert("isRunning".into(), Value::Bool(running));
            object.insert("isRunningTurn".into(), Value::Bool(running));
            object.insert(
                "isComposingMessage".into(),
                Value::Bool(running && state.lifecycle.is_composing(&agent_id)),
            );
            object.insert(
                "isRetrying".into(),
                Value::Bool(running && state.lifecycle.is_retrying(&agent_id)),
            );
            let current_activity = if running {
                state
                    .lifecycle
                    .structured_activity(&agent_id)
                    .and_then(|activity| serde_json::to_value(activity).ok())
                    .unwrap_or(Value::Null)
            } else {
                Value::Null
            };
            object.insert("currentActivity".into(), current_activity);
            object.insert(
                "snapshotEpoch".into(),
                Value::String(snapshot_epoch.clone()),
            );
            object.insert(
                "snapshotSeq".into(),
                Value::Number(snapshot_seq.into()),
            );
        }
    }

    pub fn roster_process_epoch(&self) -> &str {
        self.replica_writer.process_epoch()
    }

    pub fn next_replica_stamp(
        &self,
        replica_key: &str,
    ) -> super::replica_writer::ReplicaStamp {
        self.replica_writer.next_stamp(replica_key)
    }

    pub fn current_turn_epoch(&self, agent_id: &str) -> u64 {
        self.lock_state().pipeline.current_turn_epoch(agent_id)
    }

    pub fn in_flight_run_count(&self, agent_id: &str) -> u64 {
        self.lock_state().lifecycle.in_flight_count(agent_id)
    }

    pub fn queued_turn_count(&self, agent_id: &str) -> usize {
        self.lock_state()
            .turn_dispatch
            .queued_task_ids(agent_id)
            .len()
    }

    pub fn active_turn_lane(&self, agent_id: &str) -> Option<RunLane> {
        self.lock_state().turn_dispatch.active_lane(agent_id)
    }

    pub fn active_turn_source(&self, agent_id: &str) -> Option<String> {
        self.lock_state()
            .turn_dispatch
            .active_source(agent_id)
            .map(ToOwned::to_owned)
    }

    pub fn is_turn_dispatch_idle(&self, agent_id: &str) -> bool {
        self.lock_state().turn_dispatch.is_idle(agent_id)
    }

    pub fn is_agent_running(&self, agent_id: &str) -> bool {
        self.lock_state().lifecycle.is_running(agent_id)
    }

    fn lock_state(&self) -> std::sync::MutexGuard<'_, RuntimeState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn cache_completion(
    state: &mut RuntimeState,
    client_nonce: &str,
    result: Result<Value, ProductionSendError>,
) {
    if !state.completions.contains_key(client_nonce) {
        state.completion_order.push_back(client_nonce.to_string());
    }
    state.completions.insert(client_nonce.to_string(), result);
    while state.completion_order.len() > COMPLETION_CACHE_MAX {
        if let Some(oldest) = state.completion_order.pop_front() {
            state.completions.remove(&oldest);
        }
    }
}

fn replay_record(record: &AcceptanceRecord) -> Value {
    let operation_id = record
        .echo_entry_id
        .as_deref()
        .and_then(|entry_id| entry_id.strip_suffix(":user"));
    json!({
        "accepted": record.status == AcceptanceStatus::Accepted,
        "duplicate": true,
        "acceptanceStatus": match record.status {
            AcceptanceStatus::Accepted => "accepted",
            AcceptanceStatus::Rejected => "rejected",
            AcceptanceStatus::Pending => "pending",
        },
        "agentId": record.agent_id,
        "clientNonce": record.client_nonce,
        "operationId": operation_id,
    })
}

fn parse_send_input(args: &Value) -> Result<SendInput, ProductionSendError> {
    let agent_id = optional_non_empty(args, "agentId")
        .or_else(|| optional_non_empty(args, "id"))
        .map(ToOwned::to_owned);
    let prompt = args
        .get("prompt")
        .or_else(|| args.get("text"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let attachment_paths = string_array(args, "attachmentPaths")?;
    let attachment_names = string_array(args, "attachmentNames")?;
    if prompt.trim().is_empty() && attachment_paths.is_empty() {
        return Ok(SendInput {
            agent_id,
            prompt,
            rich_text: optional_string(args, "richText")?,
            reply_to_id: optional_string(args, "replyToId")?,
            is_fork: optional_bool(args, "isFork")?.unwrap_or(false),
            attachment_paths,
            attachment_names,
        });
    }
    Ok(SendInput {
        agent_id,
        prompt,
        rich_text: optional_string(args, "richText")?,
        reply_to_id: optional_string(args, "replyToId")?,
        is_fork: optional_bool(args, "isFork")?.unwrap_or(false),
        attachment_paths,
        attachment_names,
    })
}

fn string_array(args: &Value, field: &str) -> Result<Vec<String>, ProductionSendError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(Value::Array(values)) => values
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(ToOwned::to_owned)
                    .ok_or_else(|| {
                        ProductionSendError::BadRequest(format!(
                            "{field} must contain only strings"
                        ))
                    })
            })
            .collect(),
        Some(_) => Err(ProductionSendError::BadRequest(format!(
            "{field} must be an array"
        ))),
    }
}

fn optional_string(args: &Value, field: &str) -> Result<Option<String>, ProductionSendError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(ProductionSendError::BadRequest(format!("invalid {field}"))),
    }
}

fn optional_bool(args: &Value, field: &str) -> Result<Option<bool>, ProductionSendError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(value)) => Ok(Some(*value)),
        Some(_) => Err(ProductionSendError::BadRequest(format!("invalid {field}"))),
    }
}

fn optional_non_empty<'a>(args: &'a Value, field: &str) -> Option<&'a str> {
    args.get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn map_acceptance_error(error: PromptAcceptanceError) -> ProductionSendError {
    match error {
        PromptAcceptanceError::DigestMismatch { .. } => {
            ProductionSendError::Conflict(error.to_string())
        }
        PromptAcceptanceError::Rejected { .. } => {
            ProductionSendError::Rejected(error.to_string())
        }
        PromptAcceptanceError::InvalidPendingIdentity => {
            ProductionSendError::Internal(error.to_string())
        }
    }
}

fn system_now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64
}
