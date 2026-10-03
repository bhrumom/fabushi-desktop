use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use crate::extensions::local_tool_permission::local_tool_permission_controller::{
    SandLocalToolControllerEvent, SandLocalToolControllerEventKind,
    SandLocalToolControllerSubscription, SandLocalToolPermissionController,
    SandLocalToolRequestStatus,
};
use crate::extensions::memory::agent_state::SandAgentState;
use crate::extensions::session::production::ProductionSessionWorkers;
use crate::runner::TurnUsage;
use crate::runner::computer_use::{
    ComputerControlLease, ComputerUseCoordination, ComputerUsePrewarmStage,
};
use crate::runner::sand_action_audit::ActionAuditRecord;
use crate::runner::production_agent_checkpoint::{
    AgentStateCheckpointSink, ProductionAgentStateCheckpointSink,
};
use crate::runner::production_turn_agent_owner::{
    ProductionTurnAgentBuildBindings, ProductionTurnAgentLifecycleBindings,
};
use crate::runner::prompt_collector_glue::PromptCollectorAutomationReminderState;
use crate::runner::sand_agent_runner::SandAgentRunner;
use crate::runner::subagent_runtime::SubagentRuntime;
use crate::runner::tools::sand_multitask_todo_tool::MultitaskTodoState;
use crate::runner::tools::sand_state_tool::SandStateWriter;
use crate::runner::turn_agent_composition::TurnAgentComposition;
pub use crate::runner_production_bridge::ProductionRunnerCompositionHooks as ProductionTurnCompositionHooks;
use crate::runner_production_bridge::{
    ProductionRunnerCompositionInput, create_production_runner,
    create_production_runner_composition_with_hooks,
};
use crate::transcript_mirror::conversation_state_binary::conversation_compaction_epoch;
use crate::transcript_mirror::generated_occurrence_codec::{
    GeneratedTranscriptOccurrenceCodec, RejectGeneratedToolJsonProjection,
};
use crate::transcript_mirror::production_provider::ProductionTranscriptMirrorProvider;
use crate::transcript_mirror::transcript_mirror::JournalOutcomeReporter;
use crate::transcript_mirror::transcript_mirror_router::JournalEnabledReader;

type PermissionEventSink = Arc<dyn Fn(&SandLocalToolControllerEvent) + Send + Sync>;

/// Host-resolved state surfaces consumed by the canonical Runner bridge.
///
/// Concrete state owners remain in Session/Memory; HostRunnerComposition only
/// resolves their turn-scoped handles before delegating immutable projection.
pub struct ProductionTurnStateSurfaces {
    pub state_writer: Option<Arc<dyn SandStateWriter>>,
    pub multitask_todo_state: Option<Arc<dyn MultitaskTodoState>>,
}

pub struct HostRunnerComposition {
    controller: Arc<SandLocalToolPermissionController>,
    sink: PermissionEventSink,
    surfaces: Mutex<HashMap<String, SandLocalToolControllerSubscription>>,
    prompt_automation_reminders: Mutex<HashMap<String, PromptCollectorAutomationReminderState>>,
    computer_use: Arc<Mutex<ComputerUseCoordination>>,
}

impl HostRunnerComposition {
    pub fn production(
        controller: Arc<SandLocalToolPermissionController>,
        sessions: Arc<ProductionSessionWorkers>,
    ) -> Self {
        let sink = Arc::new(move |event: &SandLocalToolControllerEvent| {
            if let Err(error) = persist_local_permission_event(&sessions, event) {
                eprintln!(
                    "mahayana-host local permission transcript projection failed agent={} request={} error={error}",
                    event.request.agent_id, event.request.id,
                );
            }
        });
        Self::with_sink(controller, sink)
    }

    pub fn with_sink(
        controller: Arc<SandLocalToolPermissionController>,
        sink: PermissionEventSink,
    ) -> Self {
        Self {
            controller,
            sink,
            surfaces: Mutex::new(HashMap::new()),
            prompt_automation_reminders: Mutex::new(HashMap::new()),
            computer_use: Arc::new(Mutex::new(ComputerUseCoordination::new(true))),
        }
    }

    /// Resolve turn-scoped state surfaces through the canonical Host composition owner.
    pub fn compose_turn_state_surfaces(
        &self,
        sessions: &ProductionSessionWorkers,
        agent_id: &str,
        group_member_turn: bool,
        multitask_enabled: bool,
    ) -> Result<ProductionTurnStateSurfaces, String> {
        if group_member_turn {
            return Ok(ProductionTurnStateSurfaces {
                state_writer: None,
                multitask_todo_state: None,
            });
        }
        let sand_root = sessions
            .memory_service()
            .agents_root_dir()
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| "production memory agents root has no sand root parent".to_string())?;
        let state_writer: Arc<dyn SandStateWriter> =
            Arc::new(SandAgentState::new(sand_root, agent_id.to_string())?);
        let multitask_todo_state = if multitask_enabled {
            Some(sessions.open_agent_db_owner(agent_id)? as Arc<dyn MultitaskTodoState>)
        } else {
            None
        };
        Ok(ProductionTurnStateSurfaces {
            state_writer: Some(state_writer),
            multitask_todo_state,
        })
    }

    /// Compose the production transcript/checkpoint boundary for one Runner session.
    ///
    /// Session storage and transcript-mirror implementations retain their own state;
    /// HostRunnerComposition owns only the one-time wiring between those owners.
    pub fn compose_production_checkpoint_sink(
        &self,
        sessions: &ProductionSessionWorkers,
        data_dir: &Path,
        agent_id: &str,
        group_member_turn: bool,
        report_outcome: JournalOutcomeReporter,
        is_journal_enabled: JournalEnabledReader,
    ) -> Result<Option<Arc<dyn AgentStateCheckpointSink>>, String> {
        if group_member_turn {
            return Ok(None);
        }
        let agent_store = sessions.open_agent_store_owner(agent_id)?;
        let blob_store = Arc::new(sessions.create_agent_blob_store(agent_id)?);
        let prior_state_bytes = agent_store.latest_checkpoint_bytes().unwrap_or_default();
        let transcript_provider = ProductionTranscriptMirrorProvider::with_reporter(
            data_dir.join("transcripts"),
            GeneratedTranscriptOccurrenceCodec::new(RejectGeneratedToolJsonProjection),
            report_outcome,
        );
        let transcript_mirror = Arc::new(transcript_provider.route_for_session(
            Arc::clone(&blob_store),
            &prior_state_bytes,
            is_journal_enabled,
        )?);
        let sink = ProductionAgentStateCheckpointSink::new(
            agent_id.to_string(),
            agent_store,
            blob_store,
            transcript_mirror,
            prior_state_bytes,
            true,
        )?;
        Ok(Some(Arc::new(sink)))
    }

    /// Read the durable conversation compaction epoch used by dynamic prompt projection.
    pub fn prompt_compaction_epoch(
        &self,
        sessions: &ProductionSessionWorkers,
        agent_id: &str,
    ) -> Result<u64, String> {
        let agent_store = sessions.open_agent_store_owner(agent_id)?;
        let state = agent_store.latest_checkpoint_bytes().unwrap_or_default();
        conversation_compaction_epoch(&state).map_err(|error| {
            format!("could not decode production conversation compaction epoch for {agent_id}: {error}")
        })
    }

    /// Resolve this turn's automation reminder through the long-lived per-Agent collector state.
    pub fn automation_status_reminder_for_turn(
        &self,
        agent_id: &str,
        rendered: Option<&str>,
        compaction_epoch: u64,
    ) -> Option<String> {
        self.prompt_automation_reminders
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .entry(agent_id.to_string())
            .or_default()
            .reminder_for_turn(rendered, compaction_epoch)
    }

    /// Commit a reminder only after the prompt collector actually injected it.
    pub fn note_automation_status_reminder(
        &self,
        agent_id: &str,
        reminder: Option<&str>,
        compaction_epoch: u64,
    ) {
        self.prompt_automation_reminders
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .entry(agent_id.to_string())
            .or_default()
            .note_reminder(reminder, compaction_epoch);
    }

    /// Construct the shipping Runner facade from the composed turn and lifecycle owners.
    pub fn compose_production_runner(
        &self,
        composition: TurnAgentComposition,
        build_bindings: ProductionTurnAgentBuildBindings,
        lifecycle_bindings: ProductionTurnAgentLifecycleBindings,
        checkpoint_sink: Option<Arc<dyn AgentStateCheckpointSink>>,
        upgrade_quiesce_signal: Arc<AtomicBool>,
        generated_agent_runtime: Arc<Mutex<SubagentRuntime>>,
    ) -> SandAgentRunner {
        create_production_runner(
            composition,
            build_bindings,
            lifecycle_bindings,
            checkpoint_sink,
            upgrade_quiesce_signal,
            generated_agent_runtime,
        )
    }

    /// Build one production turn through the canonical Host -> Runner composition owner.
    ///
    /// The entrypoint supplies concrete turn-scoped services; this method owns the
    /// composition order so app/main.rs cannot grow a second Runner assembly path.
    pub fn compose_production_turn(
        &self,
        input: ProductionRunnerCompositionInput,
        hooks: ProductionTurnCompositionHooks,
    ) -> TurnAgentComposition {
        create_production_runner_composition_with_hooks(input, hooks)
    }

    pub fn bind_local_permission_surface(&self, agent_id: &str) {
        self.unbind_local_permission_surface(agent_id);
        let wanted = agent_id.to_string();
        let sink = Arc::clone(&self.sink);
        let subscription = self.controller.subscribe(Arc::new(move |event| {
            if event.request.agent_id == wanted {
                sink(&event);
            }
        }));
        self.surfaces
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(agent_id.to_string(), subscription);
    }

    pub fn unbind_local_permission_surface(&self, agent_id: &str) {
        self.surfaces
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(agent_id);
    }

    pub fn can_ask_local_tool_permission(&self, agent_id: &str) -> bool {
        self.surfaces
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .contains_key(agent_id)
    }

    pub fn forget_local_tool_permission(&self, agent_id: &str) {
        self.unbind_local_permission_surface(agent_id);
        self.controller.forget_agent(agent_id);
    }

    pub fn active_permission_surface_count(&self) -> usize {
        self.surfaces
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }

    /// Settle HostRunnerComposition-owned shutdown state after the canonical
    /// Runner registry has interrupted active turns. Runner cancellation itself
    /// remains owned by TranscriptRunnerRegistry; this owner only drops its live
    /// local-permission subscriptions, matching the frozen composition boundary.
    pub fn dispose(&self) {
        self.surfaces
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
        self.prompt_automation_reminders
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
    }

    pub fn computer_use_coordination(&self) -> Arc<Mutex<ComputerUseCoordination>> {
        Arc::clone(&self.computer_use)
    }

    pub fn begin_computer_use_preparation(&self, agent_id: &str) -> Option<ComputerControlLease> {
        let mut owner = self
            .computer_use
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let lease = owner.acquire_control_lease(agent_id);
        if lease.is_some() {
            owner.begin_preparation(agent_id);
        } else {
            owner.mark_preparation_failed(
                agent_id,
                ComputerUsePrewarmStage::Box,
                "computer_window_busy",
            );
        }
        lease
    }

    pub fn mark_computer_use_preparation_ready(&self, agent_id: &str) {
        self.computer_use
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .mark_preparation_ready(agent_id);
    }

    pub fn mark_computer_use_preparation_failed(
        &self,
        agent_id: &str,
        stage: ComputerUsePrewarmStage,
        error_class: impl Into<String>,
    ) {
        self.computer_use
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .mark_preparation_failed(agent_id, stage, error_class);
    }

    pub fn owns_computer_control_lease(&self, lease: &ComputerControlLease) -> bool {
        self.computer_use
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .owns_control_lease(lease)
    }

    pub fn capture_computer_navigation_baseline(&self, stdout: &str) {
        self.computer_use
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .capture_navigation_baseline(stdout);
    }

    pub fn request_computer_navigation_probe(&self) -> Option<u64> {
        self.computer_use
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .request_navigation_probe()
    }

    pub fn computer_navigation_probe_wait_ms(&self, generation: u64, now_ms: u64) -> Option<u64> {
        self.computer_use
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .navigation_probe_wait_ms(generation, now_ms)
    }

    pub fn complete_computer_navigation_probe(
        &self,
        generation: u64,
        stdout: &str,
        agent_id: &str,
        turn_id: Option<&str>,
        occurred_at_ms: u64,
    ) -> Vec<ActionAuditRecord> {
        self.computer_use
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .complete_navigation_probe(generation, stdout, agent_id, turn_id, occurred_at_ms)
    }

    pub fn finish_computer_use_turn(
        &self,
        agent_id: &str,
        lease: Option<&ComputerControlLease>,
        model_id: Option<&str>,
        usage: Option<TurnUsage>,
    ) {
        let mut owner = self
            .computer_use
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(model_id) = model_id {
            owner.record_model_id(model_id);
        }
        owner.record_turn_ended(usage);
        if let Some(lease) = lease {
            let _ = owner.release_control_lease(lease);
        } else {
            owner.free_window(agent_id);
        }
    }
}

fn persist_local_permission_event(
    sessions: &ProductionSessionWorkers,
    event: &SandLocalToolControllerEvent,
) -> Result<(), String> {
    match event.kind {
        SandLocalToolControllerEventKind::Created => {
            let mut ask = json!({
                "requestId": event.request.id,
                "action": event.request.action,
                "target": event.request.target,
                "status": "pending",
            });
            if let Some(description) = event.request.description.as_deref() {
                ask["description"] = Value::String(description.to_string());
            }
            let entry = json!({
                "id": event.request.id,
                "kind": "send-message",
                "message": {
                    "type": "local-tool-permission",
                    "ask": ask,
                },
                "timestampMs": now_ms(),
            });
            sessions
                .append_agent_transcript_entries(&event.request.agent_id, &[entry])
                .map(|_| ())
        }
        SandLocalToolControllerEventKind::Settled => {
            let entry_id = event.request.id.as_str();
            let Some(mut entry) = sessions
                .read_agent_transcript_entries(&event.request.agent_id)?
                .into_iter()
                .find(|entry| entry.get("id").and_then(Value::as_str) == Some(entry_id))
            else {
                return Ok(());
            };
            if let Some(status) = entry
                .get_mut("message")
                .and_then(Value::as_object_mut)
                .and_then(|message| message.get_mut("ask"))
                .and_then(Value::as_object_mut)
            {
                status.insert(
                    "status".into(),
                    Value::String(permission_status(event.request.status).to_string()),
                );
            }
            sessions
                .update_agent_transcript_entry(&event.request.agent_id, entry_id, &entry)
                .map(|_| ())
        }
    }
}

fn permission_status(status: SandLocalToolRequestStatus) -> &'static str {
    match status {
        SandLocalToolRequestStatus::Pending => "pending",
        SandLocalToolRequestStatus::Allowed => "allowed",
        SandLocalToolRequestStatus::Denied => "denied",
        SandLocalToolRequestStatus::Always => "always",
        SandLocalToolRequestStatus::Never => "never",
        SandLocalToolRequestStatus::Expired => "expired",
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}
