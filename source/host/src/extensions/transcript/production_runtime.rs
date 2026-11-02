use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use crate::extensions::session::agent_session::SandAgentSessionStore;
use crate::extensions::session::agent_db::{
    OriginalTurnReference, OriginalTurnScope, SessionWorkAdmissionRecord,
};
use crate::extensions::session::session_paths::resolve_internal_send_route;
use crate::extensions::session::production::ProductionSessionWorkers;
use crate::sand_activity::{ActivityUpdate, AgentActivity};

use super::async_task_union::{AsyncTask, merge_async_tasks};
use super::ack_obligations::{AckObligations, AckRedrivePreparation, AckRedriveTrigger};
use super::sand_ack_obligation_store::SandAckObligationStore;
use super::box_request_entries::{ActiveBoxRequest, BoxRequestTrackDecision};
use super::prompt_acceptance_ledger::{
    AcceptanceLookup, AcceptanceRecord, AcceptanceStatus, PromptAcceptanceError,
    PromptAcceptanceLedger, SendInput,
    send_input_digest,
};
use super::run_lifecycle::RunLifecycleState;
use super::send_group_fanout::GroupMemberTurnExecutor;
use super::sand_pending_wake_store::{DurablePendingWakeMarker, PendingWakeKind, SandPendingWakeStore};
use super::sand_upgrade_resume_store::{SandUpgradeResumeStore, UpgradeResumeMarker};
use super::session_runtime::SessionRuntime;
use super::replica_writer::HostReplicaWriter;
use super::run_scheduler::{
    QueueAccepted, QueueDequeued, RUN_WATCHDOG_DEFAULT_MS, RUN_WATCHDOG_GRACE_DEFAULT_MS,
    RunLane, RunSettlement, WatchdogEvent,
};
use super::pending_wake_rearm::is_recreate_wake_carry_disabled;
use super::send_pipeline::{
    HOST_ACCOUNT_SLOT, PersistedSendContext, SendBegin, SendEchoIdentity, SendPipelineState,
};
use super::turn_runtime::{QueuedTurnRecoveryCheck, should_supersede_stale_turn};
use super::send_turn_dispatch::{ProductionTurnDispatch, UserTurnTicket};
use super::upgrade_recreate_resume::{
    UpgradeQuiesceSummary, UpgradeRecreateResume,
};

const COMPLETION_CACHE_MAX: usize = 256;

pub type AgentRunLifecycleObserver = Arc<dyn Fn(&str, &str) + Send + Sync>;
pub type SharedGroupRoomEntryPublisher =
    Arc<dyn Fn(&str, &Value) -> Result<(), String> + Send + Sync>;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutedSendAcceptance {
    pub duplicate: bool,
    pub context: PersistedSendContext,
}

pub fn classify_send_dispatch(
    args: &Value,
) -> Result<(RunLane, &'static str), ProductionSendError> {
    let request_source = optional_non_empty(args, "requestSource");
    let is_handoff_resume = request_source == Some("handoff-resume");
    let is_automation = request_source == Some("automation");
    let is_upgrade_resume = request_source == Some("upgrade-resume");
    let is_event = request_source == Some("event");
    let is_group_member = request_source == Some("group-member");
    let is_agent_inbound = request_source == Some("agent-inbound");
    let is_human_handoff = request_source == Some("human-handoff");
    let is_kickstart = request_source == Some("kickstart");
    let is_ack_redrive =
        optional_bool(args, "ackRedrive")?.unwrap_or(false) && is_handoff_resume;
    Ok((
        if is_handoff_resume || is_automation || is_upgrade_resume || is_event {
            RunLane::Background
        } else if is_agent_inbound {
            RunLane::Agent
        } else {
            RunLane::User
        },
        if is_ack_redrive {
            "ack-redrive"
        } else if is_handoff_resume {
            "handoff-resume"
        } else if is_automation {
            "automation"
        } else if is_upgrade_resume {
            "upgrade-resume"
        } else if is_event {
            "event"
        } else if is_agent_inbound {
            "agent-inbound"
        } else if is_human_handoff {
            "human-handoff"
        } else if is_group_member {
            "group-member"
        } else if request_source == Some("connector") {
            "connector"
        } else if is_kickstart {
            "kickstart"
        } else {
            "turn"
        },
    ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RoutedTurnLease {
    agent_id: String,
    ticket: UserTurnTicket,
    generation: u64,
    is_group_member_turn: bool,
    human_handoff_conversation_id: Option<String>,
    original_input: Option<RoutedOriginalInput>,
    original_work: Option<OriginalTurnReference>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RoutedOriginalInput {
    input_digest: String,
    client_nonce: Option<String>,
    user_message_id: String,
    echo_entry_id: String,
    request_source: Option<String>,
}

fn original_routed_input(
    args: &Value,
    input: &SendInput,
    client_nonce: Option<&str>,
    context: &PersistedSendContext,
) -> Option<RoutedOriginalInput> {
    let source = match args.get("requestSource") {
        None | Some(Value::Null) => None,
        Some(Value::String(source)) if source.trim().is_empty() => None,
        Some(Value::String(source)) if source.trim() == "turn" => Some("turn".to_string()),
        _ => return None,
    };
    if !context.accepted_durably || input.prompt.trim().is_empty() || input.is_fork
        || ["hidden", "groupMemberTurn", "upgradeResume", "ackRedrive", "skipAckObligation"]
            .into_iter().any(|key| args.get(key).is_some_and(|value| !value.is_null() && value.as_bool() != Some(false)))
        || args.get("appendUserMessage").is_some_and(|value| !value.is_null() && value.as_bool() != Some(true))
        || ["parentAgentId", "parentAgentToolCallId", "spawnedBy", "automationWake", "groupContext", "groupRoomId", "humanHandoffConversationId"]
            .into_iter().any(|key| args.get(key).is_some_and(|value| !value.is_null()))
    {
        return None;
    }
    let user_message_id = context.user_message_id.as_ref().filter(|value| !value.trim().is_empty())?;
    let echo_entry_id = context.echo_entry_id.as_ref().filter(|value| *value == user_message_id)?;
    Some(RoutedOriginalInput {
        input_digest: send_input_digest(input),
        client_nonce: client_nonce.map(ToOwned::to_owned),
        user_message_id: user_message_id.clone(),
        echo_entry_id: echo_entry_id.clone(),
        request_source: source,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutedTurnSettlement {
    pub watchdog: Option<WatchdogEvent>,
    pub next: Option<QueueDequeued>,
}

struct RuntimeState {
    pipeline: SendPipelineState,
    ledger: PromptAcceptanceLedger,
    lifecycle: RunLifecycleState,
    turn_dispatch: ProductionTurnDispatch,
    routed_turn_leases: HashMap<String, RoutedTurnLease>,
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
    shared_group_remote_executor: Mutex<Option<GroupMemberTurnExecutor>>,
    shared_group_room_entry_publisher: Mutex<Option<SharedGroupRoomEntryPublisher>>,
    roster_snapshot_seq: AtomicU64,
    agent_run_lifecycle_observer: Mutex<Option<AgentRunLifecycleObserver>>,
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
                routed_turn_leases: HashMap::new(),
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
            shared_group_remote_executor: Mutex::new(None),
            shared_group_room_entry_publisher: Mutex::new(None),
            roster_snapshot_seq: AtomicU64::new(0),
            agent_run_lifecycle_observer: Mutex::new(None),
        }
    }

    pub fn set_agent_run_lifecycle_observer(
        &self,
        observer: Option<AgentRunLifecycleObserver>,
    ) {
        *self
            .agent_run_lifecycle_observer
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = observer;
    }

    fn emit_agent_run_lifecycle(&self, event_type: &str, request_id: &str) {
        let observer = self
            .agent_run_lifecycle_observer
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        if let Some(observer) = observer {
            observer(event_type, request_id);
        }
    }

    pub fn session_runtime(&self) -> &SessionRuntime {
        &self.session_runtime
    }

    pub fn bind_shared_group_remote_executor(
        &self,
        executor: Option<GroupMemberTurnExecutor>,
    ) {
        *self
            .shared_group_remote_executor
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = executor;
    }

    pub fn shared_group_remote_executor(&self) -> Option<GroupMemberTurnExecutor> {
        self.shared_group_remote_executor
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub fn bind_shared_group_room_entry_publisher(
        &self,
        publisher: Option<SharedGroupRoomEntryPublisher>,
    ) {
        *self
            .shared_group_room_entry_publisher
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = publisher;
    }

    pub fn publish_shared_group_room_entry(
        &self,
        room_id: &str,
        entry: &Value,
    ) -> Result<bool, String> {
        let publisher = self
            .shared_group_room_entry_publisher
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        let Some(publisher) = publisher else {
            return Ok(false);
        };
        publisher(room_id, entry)?;
        Ok(true)
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

    pub fn begin_live_subagent(&self, parent_agent_id: &str) {
        self.lock_state()
            .lifecycle
            .begin_live_subagent(parent_agent_id);
    }

    pub fn end_live_subagent(&self, parent_agent_id: &str) {
        self.lock_state()
            .lifecycle
            .end_live_subagent(parent_agent_id);
    }

    pub fn has_live_subagent(&self, parent_agent_id: &str) -> bool {
        self.lock_state()
            .lifecycle
            .has_live_subagent(parent_agent_id)
    }

    /// Frozen SandHost health reads this live set instead of projecting a
    /// permanently-idle default through the gateway.
    pub fn live_running_agent_ids(&self) -> Vec<String> {
        let mut ids = self
            .lock_state()
            .lifecycle
            .running_agent_ids()
            .into_iter()
            .collect::<Vec<_>>();
        ids.sort();
        ids
    }

    /// Pending durable wakes count as background work for Host health.
    pub fn has_carryable_pending_wake(&self) -> bool {
        self.pending_wake_store
            .as_ref()
            .is_some_and(|store| !store.list_pending().is_empty())
    }

    pub fn recreate_carry_pending_wakes(&self) -> Vec<DurablePendingWakeMarker> {
        if is_recreate_wake_carry_disabled() {
            return Vec::new();
        }
        self.pending_wake_store
            .as_ref()
            .map(|store| {
                store
                    .list_pending()
                    .into_iter()
                    .filter(|marker| {
                        matches!(marker.kind, PendingWakeKind::CloudAgent | PendingWakeKind::Shell)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn active_agent_id(
        &self,
        sessions: &Arc<ProductionSessionWorkers>,
    ) -> Option<String> {
        self.session_runtime.active_agent_id(sessions)
    }

    pub fn quiesce_for_upgrade(&self) -> Result<UpgradeQuiesceSummary, String> {
        self.quiesce_for_upgrade_with_marker_writer(|store, markers| {
            store.mark_pending_batch_checked(markers).map_err(|error| error.to_string())
        })
    }

    fn quiesce_for_upgrade_with_marker_writer(
        &self,
        write_markers: impl FnOnce(&SandUpgradeResumeStore, &[UpgradeResumeMarker]) -> Result<(), String>,
    ) -> Result<UpgradeQuiesceSummary, String> {
        // Hold the existing admission owner across snapshot, marker commit
        // and flag publication. This prevents a new accepted agent from
        // appearing after the snapshot but before Manager's cancel_all.
        // Production writer is only store I/O, never an observer/provider.
        let state = self.state.lock()
            .map_err(|_| "Transcript runtime owner lock is poisoned during upgrade handoff".to_string())?;
        let running = state
                .lifecycle
                .running_agent_ids()
                .into_iter()
                .map(|agent_id| {
                    let source = state
                        .turn_dispatch
                        .active_source(&agent_id)
                        .unwrap_or("turn")
                        .to_string();
                    (agent_id, source)
                })
                .collect::<Vec<_>>();
        let store = self.upgrade_resume_store.as_ref()
            .ok_or_else(|| "upgrade resume marker store is unavailable".to_string())?;
        let marked_at_ms = system_now_ms() as f64;
        let markers = running.iter()
            .map(|(agent_id, source)| UpgradeResumeMarker {
                    agent_id: agent_id.clone(),
                    marked_at_ms,
                    // Keep the existing kickstart marker identity, but only an
                    // actual bound ordinary routed lease carries original work.
                    source: Some(if source == "kickstart" { "turn".into() } else { source.clone() }),
                    automation_id: None,
                    automation_run_id: None,
                    original_work: state.routed_turn_leases.values()
                        .find(|lease| lease.agent_id == *agent_id
                            && state.turn_dispatch.active_generation_for(&lease.ticket) == Some(lease.generation))
                        .and_then(|lease| lease.original_work.clone()),
                })
            .collect::<Vec<_>>();
        write_markers(store, &markers)?;
        let summary = self.upgrade_recreate_resume.quiesce_for_upgrade(running.len());
        drop(state);
        Ok(summary)
    }

    fn ensure_upgrade_send_admission(
        &self,
        state: &mut RuntimeState,
        input: &SendInput,
        client_nonce: Option<&str>,
        args: &Value,
    ) -> Result<(), ProductionSendError> {
        if input.prompt.trim().is_empty() && input.attachment_paths.is_empty() {
            return Ok(());
        }
        // Existing in-flight and durable identities retain their coalesced
        // result/replay ownership. begin_send still checks the normal digest
        // and rejection policy; only a fresh admission is refused here.
        if client_nonce.is_some_and(|nonce| {
            state.pipeline.is_in_flight(nonce)
                || matches!(state.ledger.lookup(HOST_ACCOUNT_SLOT, nonce), AcceptanceLookup::Found(_))
        }) {
            return Ok(());
        }
        if self.is_quiescing_for_upgrade() {
            return Err(ProductionSendError::Rejected("Transcript is quiescing for host upgrade".into()));
        }
        let automatic_continuation = matches!(optional_non_empty(args, "requestSource"), Some("upgrade-resume" | "ack-redrive"))
            || args.get("upgradeResume").and_then(Value::as_bool) == Some(true)
            || args.get("ackRedrive").and_then(Value::as_bool) == Some(true);
        if automatic_continuation {
            let store = self.upgrade_resume_store.as_ref().ok_or_else(|| {
                ProductionSendError::Rejected("automatic continuation requires the upgrade marker owner".into())
            })?;
            let pending = store.migrate_legacy_checked().map_err(|error| {
                ProductionSendError::Rejected(format!("automatic continuation retained: {error}"))
            })?;
            if input.agent_id.as_deref().is_none_or(|agent_id| pending.iter().any(|marker| marker.agent_id == agent_id)) {
                return Err(ProductionSendError::Rejected("automatic continuation retained by original upgrade work".into()));
            }
        }
        Ok(())
    }

    /// Compatibility inspection; production upgrade decisions use the checked projection.
    pub fn upgrade_resume_agent_ids(&self) -> Vec<String> {
        self.upgrade_resume_agent_ids_checked().unwrap_or_default()
    }

    pub fn upgrade_resume_agent_ids_checked(&self) -> Result<Vec<String>, String> {
        let store = self.upgrade_resume_store.as_ref()
            .ok_or_else(|| "upgrade resume marker store is unavailable".to_string())?;
        let mut agent_ids = store.list_pending_checked().map_err(|error| error.to_string())?
            .into_iter().map(|marker| marker.agent_id).collect::<Vec<_>>();
        agent_ids.sort();
        Ok(agent_ids)
    }

    pub fn is_quiescing_for_upgrade(&self) -> bool {
        self.upgrade_recreate_resume.is_quiescing_for_upgrade()
    }

    /// Serialize automatic ACK consumption with both upgrade publication
    /// paths. Admission -> marker -> ACK schedule -> ACK file/cache is the
    /// owner order. Provider, telemetry and send callbacks run after release.
    pub fn prepare_automatic_ack_redrive(
        &self,
        obligations: &AckObligations,
        agent_id: &str,
        agent_exists: bool,
        now_ms: u64,
    ) -> Result<Option<(AckRedriveTrigger, AckRedrivePreparation)>, String> {
        let state = self.state.lock()
            .map_err(|_| "Transcript admission owner is poisoned".to_string())?;
        if self.is_quiescing_for_upgrade() || state.lifecycle.is_running(agent_id) {
            return Ok(None);
        }
        let store = self.upgrade_resume_store.as_ref()
            .ok_or_else(|| "automatic ACK requires the upgrade marker owner".to_string())?;
        let preparation = store.while_agent_unmarked_checked(agent_id, || {
            obligations.prepare_due_redrive_checked(agent_id, agent_exists, now_ms)
        }).map_err(|error| error.to_string())?
            .transpose().map_err(|error| error.to_string())?.flatten();
        drop(state);
        if let Some((trigger, preparation)) = &preparation {
            obligations.report_redrive_preparation(agent_id, preparation, *trigger, now_ms as f64);
        }
        Ok(preparation)
    }

    pub fn resume_after_recreate(&self) {
        self.upgrade_recreate_resume.resume_after_recreate();
    }

    /// The shipping consumer first proves the original completed turn. This
    /// existing admission owner only serializes the final idle/ACK/marker CAS;
    /// a reference or marker alone is never a Runner completion proof.
    pub fn clear_completed_upgrade_marker_if_idle(
        &self,
        marker: &UpgradeResumeMarker,
        acknowledgments: &SandAckObligationStore,
    ) -> Result<bool, String> {
        let state = self.state.lock()
            .map_err(|_| "Transcript admission owner is poisoned".to_string())?;
        if state.lifecycle.is_running(&marker.agent_id) {
            return Ok(false);
        }
        let store = self.upgrade_resume_store.as_ref()
            .ok_or_else(|| "upgrade marker owner unavailable".to_string())?;
        store.clear_if_matches_with_ack_absent_checked(marker, acknowledgments)
            .map_err(|error| error.to_string())
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

    /// Settle process-local Transcript runtime state before Session owners close.
    pub fn dispose(&self) {
        self.session_runtime.invalidate_deferred_activation();
        self.set_agent_run_lifecycle_observer(None);
        if let Ok(mut executor) = self.shared_group_remote_executor.lock() {
            *executor = None;
        }
        if let Ok(mut publisher) = self.shared_group_room_entry_publisher.lock() {
            *publisher = None;
        }

        {
            let mut state = self.lock_state();
            state.ledger.dispose();
            state.turn_dispatch.dispose();
            state.lifecycle = RunLifecycleState::default();
            state.routed_turn_leases.clear();
            state.completions.clear();
            state.completion_order.clear();
        }
        self.turn_ready.notify_all();
        self.send_settled.notify_all();
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

    /// Admit a non-Cursor Coordinator-routed prompt into the authoritative
    /// Host/Session send pipeline without pretending provider execution happened
    /// on the Host lane. The Coordinator starts the independent Runner only
    /// after this durable admission returns.
    pub fn accept_routed_send<Persist, Persisted>(
        &self,
        args: &Value,
        persist_accepted: Persist,
    ) -> Result<RoutedSendAcceptance, ProductionSendError>
    where
        Persist: Fn(&Value) -> Result<Persisted, ProductionSendError>,
        Persisted: Into<PersistedSendContext>,
    {
        self.accept_routed_send_with_queue_observers(
            args,
            persist_accepted,
            |_| false,
            |_| {},
            |_| {},
        )
    }

    pub fn accept_routed_send_with_queue_observers<
        Persist,
        Persisted,
        Watchdog,
        QueueAcceptedObserver,
        QueueDequeuedObserver,
    >(
        &self,
        args: &Value,
        persist_accepted: Persist,
        on_watchdog: Watchdog,
        on_queue_accepted: QueueAcceptedObserver,
        on_queue_dequeued: QueueDequeuedObserver,
    ) -> Result<RoutedSendAcceptance, ProductionSendError>
    where
        Persist: Fn(&Value) -> Result<Persisted, ProductionSendError>,
        Persisted: Into<PersistedSendContext>,
        Watchdog: Fn(&WatchdogEvent) -> bool,
        QueueAcceptedObserver: Fn(&QueueAccepted),
        QueueDequeuedObserver: Fn(&QueueDequeued),
    {
        let input = parse_send_input(args)?;
        let request_source = optional_non_empty(args, "requestSource");
        let session_key = args
            .get("sessionKey")
            .or_else(|| args.get("sessionId"))
            .and_then(Value::as_str);
        let requires_selector = matches!(
            request_source,
            Some("automation" | "group-member" | "upgrade-resume" | "ack-redrive" | "subagent")
        );
        if (requires_selector || session_key.is_some()) && input.agent_id.is_some() {
            resolve_internal_send_route(
                input.agent_id.as_deref().unwrap_or_default(),
                session_key,
                args.get("chatType").and_then(Value::as_str),
                args.get("spawnedBy").and_then(Value::as_str),
                args.get("directMainFallback")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            )
            .map_err(|error| {
                ProductionSendError::BadRequest(format!(
                    "invalid trusted session route: {error:?}"
                ))
            })?;
        }
        if input.prompt.trim().is_empty() && input.attachment_paths.is_empty() {
            return Err(ProductionSendError::BadRequest(
                "routed send requires prompt or attachments".into(),
            ));
        }
        let stream_id = optional_non_empty(args, "streamId")
            .ok_or_else(|| {
                ProductionSendError::BadRequest(
                    "routed send requires streamId for Host queue ownership".into(),
                )
            })?
            .to_string();
        let nonce = optional_non_empty(args, "clientNonce").map(ToOwned::to_owned);
        let agent_id = input.agent_id.clone();
        let human_handoff_conversation_id =
            optional_non_empty(args, "humanHandoffConversationId").map(ToOwned::to_owned);
        let is_human_handoff =
            optional_non_empty(args, "requestSource") == Some("human-handoff");
        match (is_human_handoff, human_handoff_conversation_id.as_deref()) {
            (true, None) => {
                return Err(ProductionSendError::BadRequest(
                    "human-handoff routed send requires humanHandoffConversationId".into(),
                ));
            }
            (false, Some(_)) => {
                return Err(ProductionSendError::BadRequest(
                    "humanHandoffConversationId is only valid for requestSource=human-handoff".into(),
                ));
            }
            _ => {}
        }
        let (dispatch_lane, dispatch_source) = classify_send_dispatch(args)?;
        let dispatch_ack_token = optional_non_empty(args, "ackToken");
        let is_fork = optional_bool(args, "isFork")?.unwrap_or(false);
        let is_group_member_turn =
            optional_bool(args, "groupMemberTurn")?.unwrap_or(false);
        let accepted = json!({ "accepted": true, "routed": true });

        let mut state = self.lock_state();
        loop {
            self.ensure_upgrade_send_admission(&mut state, &input, nonce.as_deref(), args)?;
            let begin = {
                let RuntimeState {
                    pipeline, ledger, ..
                } = &mut *state;
                pipeline
                    .begin_send(ledger, &input, nonce.as_deref())
                    .map_err(map_acceptance_error)?
            };
            match begin {
                SendBegin::EmptyNoop => {
                    return Err(ProductionSendError::BadRequest(
                        "routed send requires prompt or attachments".into(),
                    ));
                }
                SendBegin::Coalesced { client_nonce } => {
                    state = self
                        .send_settled
                        .wait_while(state, |state| state.pipeline.is_in_flight(&client_nonce))
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    drop(state);
                    let context = persist_accepted(&accepted)?.into();
                    return Ok(RoutedSendAcceptance {
                        duplicate: true,
                        context,
                    });
                }
                SendBegin::DuplicateNoop { .. } => {
                    drop(state);
                    let context = persist_accepted(&accepted)?.into();
                    return Ok(RoutedSendAcceptance {
                        duplicate: true,
                        context,
                    });
                }
                SendBegin::Dispatch { .. } => {
                    let context = match persist_accepted(&accepted) {
                        Ok(persisted) => persisted.into(),
                        Err(error) => {
                            {
                                let RuntimeState {
                                    pipeline, ledger, ..
                                } = &mut *state;
                                pipeline.finish_send(ledger, nonce.as_deref(), false);
                            }
                            drop(state);
                            self.send_settled.notify_all();
                            return Err(error);
                        }
                    };

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
                                    echo_entry_id: context.echo_entry_id.clone(),
                                },
                            )
                        };
                        if let Err(error) = pending {
                            let failure = map_acceptance_error(error);
                            {
                                let RuntimeState {
                                    pipeline, ledger, ..
                                } = &mut *state;
                                pipeline.finish_send(ledger, nonce.as_deref(), false);
                            }
                            drop(state);
                            self.send_settled.notify_all();
                            return Err(failure);
                        }
                    }

                    if let Some(agent_id) = agent_id.as_deref() {
                        let accepted_at_ms = system_now_ms();
                        state
                            .lifecycle
                            .begin_session_run(
                                agent_id,
                                accepted_at_ms,
                                is_group_member_turn,
                            );
                        let epoch = state.pipeline.next_turn_epoch(agent_id);
                        state.pipeline.register_recovery_turn(
                            agent_id,
                            epoch,
                            &context,
                            is_fork,
                        );
                        let (ticket, queue_accepted, queue_dequeued) =
                            match state.turn_dispatch.enqueue_turn_with_start(
                                agent_id,
                                nonce.as_deref(),
                                accepted_at_ms,
                                accepted_at_ms,
                                dispatch_lane,
                                dispatch_source,
                                dispatch_ack_token,
                            ) {
                                Ok(value) => value,
                                Err(error) => {
                                    let _ = state.lifecycle.end_session_run_with_kind(
                                        agent_id,
                                        system_now_ms(),
                                        is_group_member_turn,
                                    );
                                    let RuntimeState {
                                        pipeline, ledger, ..
                                    } = &mut *state;
                                    pipeline.finish_send(ledger, nonce.as_deref(), false);
                                    drop(state);
                                    self.turn_ready.notify_all();
                                    self.send_settled.notify_all();
                                    return Err(ProductionSendError::Internal(
                                        error.to_string(),
                                    ));
                                }
                            };

                        drop(state);
                        on_queue_accepted(&queue_accepted);
                        if let Some(event) = queue_dequeued.as_ref() {
                            on_queue_dequeued(event);
                        }
                        state = self.lock_state();

                        let generation = loop {
                            if let Some(generation) =
                                state.turn_dispatch.active_generation_for(&ticket)
                            {
                                break generation;
                            }
                            let now_ms = system_now_ms();
                            let wait_ms = state
                                .turn_dispatch
                                .watchdog_wait_ms(agent_id, now_ms)
                                .unwrap_or(1_000)
                                .max(1);
                            let (next_state, _) = self
                                .turn_ready
                                .wait_timeout(state, Duration::from_millis(wait_ms))
                                .unwrap_or_else(|poisoned| poisoned.into_inner());
                            state = next_state;
                            if let Some(tick) =
                                state.turn_dispatch.watchdog_tick(agent_id, system_now_ms())
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
                        };

                        if state.routed_turn_leases.contains_key(&stream_id) {
                            let _ = state
                                .turn_dispatch
                                .settle_and_start_next(&ticket, generation, system_now_ms());
                            let _ = state.lifecycle.end_session_run_with_kind(
                                agent_id,
                                system_now_ms(),
                                is_group_member_turn,
                            );
                            let RuntimeState {
                                pipeline, ledger, ..
                            } = &mut *state;
                            pipeline.finish_send(ledger, nonce.as_deref(), false);
                            drop(state);
                            self.turn_ready.notify_all();
                            self.send_settled.notify_all();
                            return Err(ProductionSendError::Conflict(format!(
                                "routed stream already owns a Host turn lease: {stream_id}"
                            )));
                        }
                        state.routed_turn_leases.insert(
                            stream_id.clone(),
                            RoutedTurnLease {
                                agent_id: agent_id.to_string(),
                                ticket,
                                generation,
                                is_group_member_turn,
                                human_handoff_conversation_id: human_handoff_conversation_id.clone(),
                                original_input: original_routed_input(args, &input, nonce.as_deref(), &context),
                                original_work: None,
                            },
                        );
                    }

                    {
                        let RuntimeState {
                            pipeline, ledger, ..
                        } = &mut *state;
                        pipeline.mark_send_accepted(ledger, nonce.as_deref());
                        pipeline.finish_send(ledger, nonce.as_deref(), true);
                    }
                    drop(state);
                    self.emit_agent_run_lifecycle("started", &stream_id);
                    self.send_settled.notify_all();
                    return Ok(RoutedSendAcceptance {
                        duplicate: false,
                        context,
                    });
                }
            }
        }
    }

    pub fn require_routed_turn_lease(
        &self,
        agent_id: &str,
        stream_id: &str,
    ) -> Result<(), ProductionSendError> {
        let state = self.lock_state();
        match state.routed_turn_leases.get(stream_id) {
            Some(lease) if lease.agent_id == agent_id => Ok(()),
            Some(_) => Err(ProductionSendError::Conflict(format!(
                "routed stream lease agent mismatch: {stream_id}"
            ))),
            None => Err(ProductionSendError::Rejected(format!(
                "routed provider stream has no admitted Host turn lease: {stream_id}"
            ))),
        }
    }

    /// Attach the already checked Session and actual resolved Native model to
    /// the original accepted input, without reading Session under this mutex.
    pub fn bind_original_turn_model(
        &self,
        agent_id: &str,
        stream_id: &str,
        admission: &SessionWorkAdmissionRecord,
        model_id: &str,
    ) -> Result<Option<OriginalTurnReference>, ProductionSendError> {
        let mut state = self.state.lock().map_err(|_| {
            ProductionSendError::Internal("original turn admission owner is poisoned".into())
        })?;
        let lease = state.routed_turn_leases.get(stream_id).ok_or_else(|| {
            ProductionSendError::Rejected("original turn has no admitted routed lease".into())
        })?;
        if lease.agent_id != agent_id
            || state.turn_dispatch.active_generation_for(&lease.ticket) != Some(lease.generation)
        {
            return Err(ProductionSendError::Conflict("original turn lease is stale or foreign".into()));
        }
        if self.is_quiescing_for_upgrade() {
            return Ok(None);
        }
        let Some(input) = lease.original_input.as_ref() else { return Ok(None); };
        let original = OriginalTurnReference {
            version: 1,
            scope: OriginalTurnScope::NativeOrdinaryUserV1,
            agent_id: agent_id.to_string(),
            session_key: admission.session_key.clone(),
            session_id: admission.session_id.clone(),
            original_lifecycle_revision: admission.lifecycle_revision.clone(),
            stream_id: stream_id.to_string(),
            input_digest: input.input_digest.clone(),
            client_nonce: input.client_nonce.clone(),
            user_message_id: input.user_message_id.clone(),
            echo_entry_id: input.echo_entry_id.clone(),
            request_source: input.request_source.clone(),
            provider: "fabushi".into(),
            model_id: model_id.to_string(),
        };
        if admission.deletion_pending || admission.archived_at_ms.is_some() || original.validate().is_err() {
            return Ok(None);
        }
        if lease.original_work.as_ref().is_some_and(|bound| bound != &original) {
            return Err(ProductionSendError::Conflict("original turn is already bound to another identity".into()));
        }
        state.routed_turn_leases.get_mut(stream_id).expect("checked under same owner lock")
            .original_work = Some(original.clone());
        Ok(Some(original))
    }

    pub fn settle_routed_turn(
        &self,
        agent_id: &str,
        stream_id: &str,
        now_ms: u64,
    ) -> Result<Option<RoutedTurnSettlement>, ProductionSendError> {
        let mut state = self.lock_state();
        let Some(lease) = state.routed_turn_leases.remove(stream_id) else {
            return Ok(None);
        };
        if lease.agent_id != agent_id {
            state
                .routed_turn_leases
                .insert(stream_id.to_string(), lease);
            return Err(ProductionSendError::Conflict(format!(
                "routed stream lease agent mismatch: {stream_id}"
            )));
        }
        let (settlement, next) = state
            .turn_dispatch
            .settle_and_start_next(&lease.ticket, lease.generation, now_ms);
        let watchdog = match settlement {
            RunSettlement::ZombieSettled { watchdog, .. } => Some(watchdog),
            RunSettlement::ActiveSettled { .. } | RunSettlement::Unknown => None,
        };
        let _ = state.lifecycle.end_session_run_with_kind(
            agent_id,
            now_ms,
            lease.is_group_member_turn,
        );
        drop(state);
        self.emit_agent_run_lifecycle("ended", stream_id);
        self.turn_ready.notify_all();
        Ok(Some(RoutedTurnSettlement { watchdog, next }))
    }

    pub fn has_routed_turn_lease(&self, agent_id: &str, stream_id: &str) -> bool {
        self.lock_state()
            .routed_turn_leases
            .get(stream_id)
            .is_some_and(|lease| lease.agent_id == agent_id)
    }

    pub fn routed_human_handoff_conversation_id(
        &self,
        agent_id: &str,
        stream_id: &str,
    ) -> Option<String> {
        self.lock_state()
            .routed_turn_leases
            .get(stream_id)
            .filter(|lease| lease.agent_id == agent_id)
            .and_then(|lease| lease.human_handoff_conversation_id.clone())
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
        self.execute_send_with_acceptance_observer(
            args,
            dispatch,
            persist_accepted,
            |_| {},
            on_watchdog,
            on_queue_accepted,
            on_queue_dequeued,
        )
    }

    pub fn execute_send_with_acceptance_observer<
        Dispatch,
        Persist,
        Persisted,
        AcceptanceObserver,
        Watchdog,
        QueueAcceptedObserver,
        QueueDequeuedObserver,
    >(
        &self,
        args: &Value,
        dispatch: Dispatch,
        persist_accepted: Persist,
        on_accepted: AcceptanceObserver,
        on_watchdog: Watchdog,
        on_queue_accepted: QueueAcceptedObserver,
        on_queue_dequeued: QueueDequeuedObserver,
    ) -> Result<Value, ProductionSendError>
    where
        Dispatch: FnOnce() -> Result<Value, ProductionSendError>,
        Persist: Fn(&Value) -> Result<Persisted, ProductionSendError>,
        Persisted: Into<PersistedSendContext>,
        AcceptanceObserver: Fn(&PersistedSendContext),
        Watchdog: Fn(&WatchdogEvent) -> bool,
        QueueAcceptedObserver: Fn(&QueueAccepted),
        QueueDequeuedObserver: Fn(&QueueDequeued),
    {
        let input = parse_send_input(args)?;
        let nonce = optional_non_empty(args, "clientNonce").map(ToOwned::to_owned);
        let agent_id = input.agent_id.clone();
        let (dispatch_lane, dispatch_source) = classify_send_dispatch(args)?;
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
            self.ensure_upgrade_send_admission(&mut state, &input, nonce.as_deref(), args)?;
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
        // Durable acceptance side effects must run after the admission lock is
        // released and before provider dispatch. Roster projection reaches back
        // into Transcript runtime, so invoking it from persist_accepted would
        // self-deadlock this owner.
        drop(state);
        on_accepted(&persisted_send_context);
        if let Some(event) = queue_accepted_event.as_ref() {
            on_queue_accepted(event);
        }
        if let Some(event) = queue_dequeued_event.as_ref() {
            on_queue_dequeued(event);
        }
        state = self.lock_state();

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
                    state.lifecycle.track_turn_request_id(agent_id, operation_id);
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
        self.begin_provider_run_with_kind(agent_id, false);
    }

    pub fn begin_provider_run_with_kind(&self, agent_id: &str, is_group_member_turn: bool) {
        self.lock_state()
            .lifecycle
            .begin_provider_run_with_kind(agent_id, is_group_member_turn);
    }

    pub fn end_provider_run(&self, agent_id: &str) {
        self.end_provider_run_with_kind(agent_id, false);
    }

    pub fn end_provider_run_with_kind(&self, agent_id: &str, is_group_member_turn: bool) {
        self.lock_state()
            .lifecycle
            .end_provider_run_with_kind(agent_id, is_group_member_turn);
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

    pub fn set_active_remote_member(
        &self,
        agent_id: &str,
        remote_member_id: Option<&str>,
    ) {
        self.lock_state()
            .lifecycle
            .set_active_remote_member(agent_id, remote_member_id);
    }

    pub fn track_box_request_entry(
        &self,
        agent_id: &str,
        entry: &Value,
    ) -> BoxRequestTrackDecision {
        self.lock_state()
            .pipeline
            .track_box_request_entry(agent_id, entry)
    }

    pub fn append_generated_send_message_entry(
        &self,
        sessions: &ProductionSessionWorkers,
        agent_id: &str,
        session_key: &str,
        message: &Value,
        timestamp_ms: u64,
        reply_thread_target: Option<&str>,
        is_fork: bool,
    ) -> Result<Value, String> {
        let entries = sessions.read_agent_transcript_entries(agent_id)?;
        let mut entry = self
            .lock_state()
            .pipeline
            .prepare_generated_send_message_entry(
                agent_id,
                &entries,
                message,
                reply_thread_target,
                is_fork,
                timestamp_ms,
            );
        entry
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "generated send pipeline did not produce an entry id".to_string())?;
        entry["sessionKey"] = Value::String(session_key.trim().to_string());
        sessions.append_agent_transcript_entries_if_session_admitted(
            agent_id,
            session_key,
            std::slice::from_ref(&entry),
        )?;
        Ok(entry)
    }

    /// Append a generated send for an explicitly unbound compatibility
    /// producer. The generic session owner still applies its deletion and
    /// archive fence; no session identity is invented for the entry.
    pub fn append_generated_send_message_entry_compatibility(
        &self,
        sessions: &ProductionSessionWorkers,
        agent_id: &str,
        message: &Value,
        timestamp_ms: u64,
        reply_thread_target: Option<&str>,
        is_fork: bool,
    ) -> Result<Value, String> {
        let entries = sessions.read_agent_transcript_entries(agent_id)?;
        let entry = self
            .lock_state()
            .pipeline
            .prepare_generated_send_message_entry(
                agent_id,
                &entries,
                message,
                reply_thread_target,
                is_fork,
                timestamp_ms,
            );
        entry
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "generated send pipeline did not produce an entry id".to_string())?;
        sessions.append_agent_transcript_entries(
            agent_id,
            std::slice::from_ref(&entry),
        )?;
        Ok(entry)
    }

    /// Append a generated send and release its durable media fence in the same
    /// session-owner transaction.
    pub fn append_generated_send_message_entry_with_media_custody(
        &self,
        sessions: &ProductionSessionWorkers,
        agent_id: &str,
        session_key: &str,
        message: &Value,
        timestamp_ms: u64,
        reply_thread_target: Option<&str>,
        is_fork: bool,
        expected_lifecycle_revision: &str,
        media_work_id: &str,
    ) -> Result<Value, String> {
        let entries = sessions.read_agent_transcript_entries(agent_id)?;
        let mut entry = self
            .lock_state()
            .pipeline
            .prepare_generated_send_message_entry(
                agent_id,
                &entries,
                message,
                reply_thread_target,
                is_fork,
                timestamp_ms,
            );
        entry
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "generated send pipeline did not produce an entry id".to_string())?;
        entry["sessionKey"] = Value::String(session_key.trim().to_string());
        sessions.append_agent_transcript_entries_and_release_media(
            agent_id,
            session_key,
            std::slice::from_ref(&entry),
            expected_lifecycle_revision,
            media_work_id,
        )?;
        Ok(entry)
    }

    pub fn append_generated_send_message(
        &self,
        sessions: &ProductionSessionWorkers,
        agent_id: &str,
        session_key: &str,
        message: &Value,
        timestamp_ms: u64,
        reply_thread_target: Option<&str>,
        is_fork: bool,
    ) -> Result<String, String> {
        self.append_generated_send_message_entry(
            sessions,
            agent_id,
            session_key,
            message,
            timestamp_ms,
            reply_thread_target,
            is_fork,
        )?
        .get("id")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| "generated send pipeline did not produce an entry id".to_string())
    }

    pub fn resolve_box_request_tracking(&self, request_id: &str) -> bool {
        self.lock_state()
            .pipeline
            .resolve_box_request_tracking(request_id)
    }

    pub fn active_box_request(&self) -> Option<ActiveBoxRequest> {
        self.lock_state()
            .pipeline
            .active_box_request()
            .cloned()
    }

    fn durable_subagent_parent_ids(&self) -> HashSet<String> {
        self.pending_wake_store
            .as_ref()
            .map(|store| {
                store
                    .list_pending()
                    .into_iter()
                    .filter(|marker| marker.kind == PendingWakeKind::Subagent)
                    .map(|marker| marker.agent_id)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn decorate_agent_summaries(&self, value: &mut Value) {
        let durable_subagent_parents = self.durable_subagent_parent_ids();
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
            let active_remote_member_id = state
                .lifecycle
                .active_remote_member_id(&agent_id)
                .map(ToOwned::to_owned);
            let projected = state.lifecycle.project_run_state(
                &agent_id,
                durable_subagent_parents.contains(&agent_id),
                active_remote_member_id.as_deref(),
            );
            object.insert("isRunning".into(), Value::Bool(projected.is_running));
            object.insert(
                "isRunningTurn".into(),
                Value::Bool(projected.is_running_turn),
            );
            object.insert(
                "isComposingMessage".into(),
                Value::Bool(projected.is_composing_message),
            );
            object.insert(
                "isRetrying".into(),
                Value::Bool(projected.is_retrying),
            );
            object.insert(
                "currentActivity".into(),
                projected
                    .current_activity
                    .as_ref()
                    .and_then(|activity| serde_json::to_value(activity).ok())
                    .unwrap_or(Value::Null),
            );
            if let Some(active_remote_member_id) = projected.active_remote_member_id {
                object.insert(
                    "activeRemoteMemberId".into(),
                    Value::String(active_remote_member_id),
                );
            } else {
                object.insert("activeRemoteMemberId".into(), Value::Null);
            }
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

    pub fn next_turn_epoch(&self, agent_id: &str) -> u64 {
        self.lock_state().pipeline.next_turn_epoch(agent_id)
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

    pub fn settle_turn_usage(
        &self,
        agent_id: &str,
        source: &str,
    ) -> super::run_lifecycle::TurnUsageReport {
        self.lock_state().lifecycle.settle_turn_usage(agent_id, source)
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

#[cfg(test)]
mod checked_upgrade_handoff_tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::AtomicUsize;
    use std::sync::{mpsc, TryLockError};
    use std::thread;

    fn args(agent: &str, nonce: &str) -> Value {
        json!({"agentId":agent,"prompt":"retained intent","streamId":nonce,"clientNonce":nonce})
    }

    #[test]
    fn marker_write_barrier_fences_new_admission_and_refusal_reopens_it() {
        for refuse in [false, true] {
            let root = std::env::temp_dir().join(format!("fabushi-upgrade-admission-barrier-{}", uuid::Uuid::new_v4()));
            let runtime = Arc::new(ProductionTranscriptRuntime::new(Some(&root)));
            runtime.accept_routed_send(&args("before", "before"), |_| Ok(PersistedSendContext::default())).unwrap();
            let store = runtime.upgrade_resume_store().unwrap().clone();
            if refuse { fs::write(store.file_path(), b"preserved corrupt marker").unwrap(); }
            let (writer_entered_tx, writer_entered_rx) = mpsc::channel();
            let (writer_release_tx, writer_release_rx) = mpsc::channel();
            let quiesce_runtime = Arc::clone(&runtime);
            let quiesce = thread::spawn(move || {
                quiesce_runtime.quiesce_for_upgrade_with_marker_writer(|store, markers| {
                    writer_entered_tx.send(()).unwrap();
                    writer_release_rx.recv_timeout(Duration::from_secs(10)).map_err(|error| error.to_string())?;
                    store.mark_pending_batch_checked(markers).map_err(|error| error.to_string())
                })
            });
            writer_entered_rx.recv_timeout(Duration::from_secs(10)).unwrap();
            assert!(matches!(runtime.state.try_lock(), Err(TryLockError::WouldBlock)),
                "the real admission owner must remain held while marker I/O is pending");
            let persists = Arc::new(AtomicUsize::new(0));
            let dispatches = Arc::new(AtomicUsize::new(0));
            let (attempt_tx, attempt_rx) = mpsc::channel();
            let (routed_tx, routed_rx) = mpsc::channel();
            let routed_runtime = Arc::clone(&runtime);
            let routed_persists = Arc::clone(&persists);
            let routed_attempt = attempt_tx.clone();
            let routed = thread::spawn(move || {
                routed_attempt.send(()).unwrap();
                let result = routed_runtime.accept_routed_send(&args("new-routed", "new-routed"), |_| {
                    routed_persists.fetch_add(1, Ordering::SeqCst);
                    Ok(PersistedSendContext::default())
                });
                routed_tx.send(result).unwrap();
            });
            let (execute_tx, execute_rx) = mpsc::channel();
            let execute_runtime = Arc::clone(&runtime);
            let execute_persists = Arc::clone(&persists);
            let execute_dispatches = Arc::clone(&dispatches);
            let execute = thread::spawn(move || {
                attempt_tx.send(()).unwrap();
                let result = execute_runtime.execute_send(&args("new-execute", "new-execute"), || {
                    execute_dispatches.fetch_add(1, Ordering::SeqCst);
                    Ok(json!({"accepted":true,"operationId":"new-execute"}))
                }, |_| {
                    execute_persists.fetch_add(1, Ordering::SeqCst);
                    Ok(PersistedSendContext::default())
                });
                execute_tx.send(result).unwrap();
            });
            attempt_rx.recv_timeout(Duration::from_secs(10)).unwrap();
            attempt_rx.recv_timeout(Duration::from_secs(10)).unwrap();
            assert_eq!(persists.load(Ordering::SeqCst), 0);
            assert_eq!(dispatches.load(Ordering::SeqCst), 0);
            writer_release_tx.send(()).unwrap();
            let prepared = quiesce.join().unwrap();
            let admitted = routed_rx.recv_timeout(Duration::from_secs(10)).unwrap();
            let executed = execute_rx.recv_timeout(Duration::from_secs(10)).unwrap();
            routed.join().unwrap();
            execute.join().unwrap();
            assert!(runtime.has_routed_turn_lease("before", "before"));
            if refuse {
                assert!(prepared.unwrap_err().contains("malformed"));
                assert!(!runtime.is_quiescing_for_upgrade());
                assert!(admitted.is_ok());
                assert!(executed.is_ok());
                assert!(runtime.has_routed_turn_lease("new-routed", "new-routed"));
                assert_eq!(persists.load(Ordering::SeqCst), 2);
                assert_eq!(dispatches.load(Ordering::SeqCst), 1);
                assert_eq!(fs::read(store.file_path()).unwrap(), b"preserved corrupt marker");
            } else {
                assert!(prepared.unwrap().quiescing);
                assert!(matches!(admitted, Err(ProductionSendError::Rejected(_))));
                assert!(matches!(executed, Err(ProductionSendError::Rejected(_))));
                assert_eq!(persists.load(Ordering::SeqCst), 0);
                assert_eq!(dispatches.load(Ordering::SeqCst), 0);
                assert_eq!(runtime.upgrade_resume_agent_ids_checked().unwrap(), vec!["before"]);
                assert!(!runtime.has_routed_turn_lease("new-routed", "new-routed"));
                assert!(!runtime.is_agent_running("new-execute"));
            }
            let _ = fs::remove_dir_all(root);
        }
    }

    #[test]
    fn quiesce_preserves_accepted_queue_coalesced_results_and_durable_replay() {
        let root = std::env::temp_dir().join(format!("fabushi-upgrade-retained-admission-{}", uuid::Uuid::new_v4()));
        let runtime = Arc::new(ProductionTranscriptRuntime::new(Some(&root)));
        let first_args = args("agent", "first");
        let queued_args = args("agent", "queued");
        let (dispatch_entered_tx, dispatch_entered_rx) = mpsc::channel();
        let (dispatch_release_tx, dispatch_release_rx) = mpsc::channel();
        let (first_tx, first_rx) = mpsc::channel();
        let first_runtime = Arc::clone(&runtime);
        let owned_first_args = first_args.clone();
        let first = thread::spawn(move || {
            let result = first_runtime.execute_send(&owned_first_args, || {
                dispatch_entered_tx.send(()).unwrap();
                dispatch_release_rx.recv_timeout(Duration::from_secs(10)).unwrap();
                Ok(json!({"accepted":true,"operationId":"original-result"}))
            }, |_| Ok(PersistedSendContext::default()));
            first_tx.send(result).unwrap();
        });
        dispatch_entered_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        let (queued_accepted_tx, queued_accepted_rx) = mpsc::channel();
        let (queued_tx, queued_rx) = mpsc::channel();
        let queued_runtime = Arc::clone(&runtime);
        let owned_queued_args = queued_args.clone();
        let queued = thread::spawn(move || {
            let result = queued_runtime.accept_routed_send_with_queue_observers(
                &owned_queued_args, |_| Ok(PersistedSendContext::default()), |_| false,
                |_| { queued_accepted_tx.send(()).unwrap(); }, |_| {},
            );
            queued_tx.send(result).unwrap();
        });
        queued_accepted_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        assert!(runtime.quiesce_for_upgrade().unwrap().quiescing);
        let (duplicate_tx, duplicate_rx) = mpsc::channel();
        let duplicate_runtime = Arc::clone(&runtime);
        let duplicate_args = first_args.clone();
        let duplicate = thread::spawn(move || {
            let result = duplicate_runtime.execute_send(&duplicate_args,
                || panic!("coalesced result must not dispatch a second turn"),
                |_| -> Result<PersistedSendContext, ProductionSendError> { panic!("coalesced result must not persist a second echo") },
            );
            duplicate_tx.send(result).unwrap();
        });
        dispatch_release_tx.send(()).unwrap();
        let original = first_rx.recv_timeout(Duration::from_secs(10)).unwrap().unwrap();
        let duplicate_result = duplicate_rx.recv_timeout(Duration::from_secs(10)).unwrap().unwrap();
        let queued_result = queued_rx.recv_timeout(Duration::from_secs(10)).unwrap().unwrap();
        first.join().unwrap();
        duplicate.join().unwrap();
        queued.join().unwrap();
        assert_eq!(duplicate_result, original);
        assert!(!queued_result.duplicate);
        assert!(runtime.has_routed_turn_lease("agent", "queued"));
        assert_eq!(runtime.upgrade_resume_agent_ids_checked().unwrap(), vec!["agent"]);
        let replay = runtime.execute_send(&first_args,
            || panic!("durable replay must not dispatch"),
            |_| -> Result<PersistedSendContext, ProductionSendError> { panic!("durable replay must not persist") },
        ).unwrap();
        assert_eq!(replay, original);
        let mut mismatched = first_args.clone();
        mismatched["prompt"] = json!("different intent");
        let mismatch = runtime.execute_send(&mismatched,
            || panic!("mismatched replay must not dispatch"),
            |_| -> Result<PersistedSendContext, ProductionSendError> { panic!("mismatched replay must not persist") },
        );
        assert!(matches!(mismatch, Err(ProductionSendError::Conflict(_))));
        assert!(runtime.accept_routed_send(&queued_args, |_| Ok(PersistedSendContext::default())).unwrap().duplicate);
        assert!(runtime.has_routed_turn_lease("agent", "queued"));
        runtime.settle_routed_turn("agent", "queued", system_now_ms()).unwrap();
        runtime.resume_after_recreate();
        assert!(runtime.accept_routed_send(&args("after", "after"), |_| Ok(PersistedSendContext::default())).is_ok());
        let _ = fs::remove_dir_all(root);
    }
}
