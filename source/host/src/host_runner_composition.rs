use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use crate::extensions::local_tool_permission::local_tool_permission_controller::{
    SandLocalToolControllerEvent, SandLocalToolControllerEventKind,
    SandLocalToolControllerSubscription, SandLocalToolPermissionController,
    SandLocalToolRequestStatus,
};
use crate::extensions::session::production::ProductionSessionWorkers;
use crate::runner::box_tool_access::BoxShellAutoReviewCallback;
use crate::runner::computer_use::ComputerUseCoordination;
use crate::runner::production_agent_checkpoint::{
    AgentStateCheckpointSink, ProductionAgentStateCheckpointSink,
};
use crate::runner::subagent_runtime::SubagentRuntime;
use crate::runner::tools::sand_agent_management_tools::AgentManagementSink;
use crate::runner::tools::sand_multitask_todo_tool::MultitaskTodoState;
use crate::runner::tools::sand_state_tool::{
    RoutineAutoReviewCallback, RoutinePostWriteCallback, SandStateWriter,
};
use crate::runner::tools::sand_subagent_management_tools::SubagentSteerReviewCallback;
use crate::runner::tools::sand_task_subagent_tool::{SubagentTaskReviewCallback, SubagentTaskSink};
use crate::runner::turn_agent_composition::TurnAgentComposition;
use crate::runner_production_bridge::{
    ProductionRunnerCompositionInput, create_production_runner_composition,
};
use crate::transcript_mirror::generated_occurrence_codec::{
    GeneratedTranscriptOccurrenceCodec, RejectGeneratedToolJsonProjection,
};
use crate::transcript_mirror::production_provider::ProductionTranscriptMirrorProvider;
use crate::transcript_mirror::transcript_mirror::JournalOutcomeReporter;
use crate::transcript_mirror::transcript_mirror_router::JournalEnabledReader;

type PermissionEventSink = Arc<dyn Fn(&SandLocalToolControllerEvent) + Send + Sync>;

/// Per-turn Host-owned projections that decorate the canonical Runner composition.
///
/// Concrete services are resolved by the shipping Host, while this owner controls
/// the ordering and one-time projection into the Runner boundary.
pub struct ProductionTurnCompositionHooks {
    pub agent_management_sink: Arc<dyn AgentManagementSink>,
    pub state_writer: Arc<dyn SandStateWriter>,
    pub routine_auto_review: RoutineAutoReviewCallback,
    pub box_shell_review: BoxShellAutoReviewCallback,
    pub subagent_task_sink: Option<Arc<dyn SubagentTaskSink>>,
    pub subagent_task_review: Option<SubagentTaskReviewCallback>,
    pub subagent_management_runtime: Option<Arc<Mutex<SubagentRuntime>>>,
    pub subagent_steer_review: Option<SubagentSteerReviewCallback>,
    pub routine_post_write: Option<RoutinePostWriteCallback>,
    pub multitask_todo_state: Option<Arc<dyn MultitaskTodoState>>,
}

pub struct HostRunnerComposition {
    controller: Arc<SandLocalToolPermissionController>,
    sink: PermissionEventSink,
    surfaces: Mutex<HashMap<String, SandLocalToolControllerSubscription>>,
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
            computer_use: Arc::new(Mutex::new(ComputerUseCoordination::new(true))),
        }
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
        report_outcome: JournalOutcomeReporter,
        is_journal_enabled: JournalEnabledReader,
    ) -> Result<Arc<dyn AgentStateCheckpointSink>, String> {
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
        Ok(Arc::new(sink))
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
        let mut composition = create_production_runner_composition(input)
            .with_agent_management_sink(hooks.agent_management_sink)
            .with_state_writer(hooks.state_writer)
            .with_routine_auto_review(hooks.routine_auto_review)
            .with_box_shell_review(hooks.box_shell_review);

        if let Some(subagent_task_sink) = hooks.subagent_task_sink {
            composition = composition.with_subagent_task_sink(subagent_task_sink);
            if let Some(subagent_task_review) = hooks.subagent_task_review {
                composition = composition.with_subagent_task_review(subagent_task_review);
            }
        }
        if let Some(subagent_runtime) = hooks.subagent_management_runtime {
            composition =
                composition.with_subagent_management(subagent_runtime, hooks.subagent_steer_review);
        }
        if let Some(routine_post_write) = hooks.routine_post_write {
            composition = composition.with_routine_post_write(routine_post_write);
        }
        if let Some(multitask_todo_state) = hooks.multitask_todo_state {
            composition = composition.with_multitask_todo_state(multitask_todo_state);
        }

        composition
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

    pub fn computer_use_coordination(&self) -> Arc<Mutex<ComputerUseCoordination>> {
        Arc::clone(&self.computer_use)
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
