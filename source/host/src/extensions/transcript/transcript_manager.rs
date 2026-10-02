use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

use serde_json::Value;

use crate::automations::automation_store::FileAutomationStore;
use crate::extensions::session::agent_db_transcript_pages::{
    TranscriptPage, TranscriptWindow, TranscriptWindowQuery,
};
use crate::extensions::attachments::attachments_service::AttachmentsService;
use crate::extensions::content_search::extension::ProductionContentSearchExtension;
use crate::extensions::memory::extension::HostMemoryExtension;
use crate::extensions::session::box_handoff_service::BoxHandoffService;
use crate::extensions::session::production::ProductionSessionWorkers;
use crate::extensions::telemetry::analytics_service::AutomationRunAnalyticsTelemetry;
use crate::extensions::telemetry::host_telemetry_service::{
    HostProductAnalytics, HostTelemetryApi,
};
use crate::extensions::trays::extension::HostTraysExtension;
use crate::extensions::turn_execution::turn_execution_service::{
    TurnExecutionError, TurnExecutionRegistry, TurnExecutor,
};

use super::ack_obligations::AckObligations;
use super::automation_runtime::AutomationRuntime;
use super::group_chat_glue::GroupChatGlue;
use super::async_task_union::AsyncTask;
use super::production_runtime::{ProductionSendError, ProductionTranscriptRuntime};
use super::runner_registry::TranscriptRunnerRegistry;
use super::shared_rooms::SharedRooms;
use super::widget_responses::WidgetResponses;
use super::workflow_commands::WorkflowCommands;

#[derive(Clone)]
pub struct TranscriptTurnExecutionPort {
    registry: Arc<Mutex<TurnExecutionRegistry>>,
}

impl TranscriptTurnExecutionPort {
    pub fn new(registry: Arc<Mutex<TurnExecutionRegistry>>) -> Self {
        Self { registry }
    }

    fn executor_handle(&self) -> Result<Arc<dyn TurnExecutor>, TurnExecutionError> {
        self.registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .executor_handle()
            .ok_or(TurnExecutionError::Unbound)
    }

    pub fn can_execute(&self) -> bool {
        self.registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .can_execute()
    }

    pub fn can_execute_group_member(&self) -> bool {
        self.can_execute()
    }

    pub async fn is_run_ready(&self) -> bool {
        let Ok(executor) = self.executor_handle() else {
            return false;
        };
        executor.is_inference_ready().await
    }

    pub fn create_runner(&self, session: Value, hooks: Value) -> Result<Value, TurnExecutionError> {
        Ok(self.executor_handle()?.create_runner(session, hooks))
    }

    pub fn create_group_member_runner(
        &self,
        session: Value,
        hooks: Value,
        overrides: Value,
    ) -> Result<Value, TurnExecutionError> {
        Ok(self
            .executor_handle()?
            .create_group_member_runner(session, hooks, overrides))
    }
}

#[derive(Clone)]
pub struct TranscriptManagerServices {
    pub telemetry: Arc<AutomationRunAnalyticsTelemetry>,
    pub product_analytics: HostProductAnalytics,
    pub trace_flusher: HostTelemetryApi,
    pub memory: HostMemoryExtension,
    pub content_search: Arc<ProductionContentSearchExtension>,
    pub attachments: Arc<AttachmentsService>,
    pub trays: Arc<HostTraysExtension>,
}

/// Production composition root for the Grok Transcript extension.
///
/// Frozen Grok builds TranscriptManager as the owner/delegation facade for
/// Session, send-pipeline, run-lifecycle and Runner registry state. The Rust
/// port keeps those owners independent, but constructs them exactly once here
/// so Host main does not create parallel runtime/ack/runner registries.
pub struct TranscriptManager {
    session_workers: Arc<ProductionSessionWorkers>,
    transcript_runtime: Arc<ProductionTranscriptRuntime>,
    runner_registry: Arc<TranscriptRunnerRegistry>,
    ack_obligations: Arc<AckObligations>,
    automation_runtime: Arc<AutomationRuntime>,
    group_chat: Arc<GroupChatGlue>,
    widget_responses: Arc<WidgetResponses>,
    workflow_commands: Arc<WorkflowCommands>,
    watched_automation_store: Mutex<Option<FileAutomationStore>>,
    handoff_service: Mutex<Option<BoxHandoffService>>,
    shared_rooms: Arc<SharedRooms>,
    services: Mutex<Option<TranscriptManagerServices>>,
    disposed: AtomicBool,
}

impl TranscriptManager {
    pub fn new(
        root_dir: impl AsRef<Path>,
        session_workers: Arc<ProductionSessionWorkers>,
    ) -> Self {
        let root_dir = root_dir.as_ref();
        let automation_runtime = Arc::new(AutomationRuntime::new(Arc::clone(&session_workers)));
        let shared_rooms = Arc::new(SharedRooms::new(Arc::clone(&session_workers)));
        let group_chat = Arc::new(GroupChatGlue::new(Arc::clone(&session_workers)));
        let widget_responses = Arc::new(WidgetResponses::new(Arc::clone(&session_workers)));
        let workflow_commands = Arc::new(WorkflowCommands::new(
            Arc::clone(&session_workers),
            Arc::clone(&automation_runtime),
        ));
        Self {
            session_workers,
            transcript_runtime: Arc::new(ProductionTranscriptRuntime::new(Some(root_dir))),
            runner_registry: Arc::new(TranscriptRunnerRegistry::default()),
            ack_obligations: Arc::new(AckObligations::new(root_dir)),
            automation_runtime,
            group_chat,
            widget_responses,
            workflow_commands,
            watched_automation_store: Mutex::new(None),
            handoff_service: Mutex::new(None),
            shared_rooms,
            services: Mutex::new(None),
            disposed: AtomicBool::new(false),
        }
    }

    pub fn session_workers(&self) -> Arc<ProductionSessionWorkers> {
        Arc::clone(&self.session_workers)
    }

    pub fn transcript_runtime(&self) -> Arc<ProductionTranscriptRuntime> {
        Arc::clone(&self.transcript_runtime)
    }

    pub fn runner_registry(&self) -> Arc<TranscriptRunnerRegistry> {
        Arc::clone(&self.runner_registry)
    }

    pub fn ack_obligations(&self) -> Arc<AckObligations> {
        Arc::clone(&self.ack_obligations)
    }

    pub fn automation_runtime(&self) -> Arc<AutomationRuntime> {
        Arc::clone(&self.automation_runtime)
    }

    pub fn shared_rooms(&self) -> Arc<SharedRooms> {
        Arc::clone(&self.shared_rooms)
    }

    pub fn group_chat(&self) -> Arc<GroupChatGlue> {
        Arc::clone(&self.group_chat)
    }

    pub fn widget_responses(&self) -> Arc<WidgetResponses> {
        Arc::clone(&self.widget_responses)
    }

    pub fn workflow_commands(&self) -> Arc<WorkflowCommands> {
        Arc::clone(&self.workflow_commands)
    }

    pub fn set_handoff_service(&self, handoff: BoxHandoffService) -> Result<(), String> {
        let mut slot = self
            .handoff_service
            .lock()
            .map_err(|_| "transcript handoff mutex poisoned".to_string())?;
        if slot.is_some() {
            return Err("transcript handoff service already configured".into());
        }
        *slot = Some(handoff);
        Ok(())
    }

    pub fn set_production_services(&self, services: TranscriptManagerServices) -> Result<(), String> {
        let mut slot = self
            .services
            .lock()
            .map_err(|_| "transcript service composition mutex poisoned".to_string())?;
        if slot.is_some() {
            return Err("transcript production services already configured".into());
        }
        *slot = Some(services);
        Ok(())
    }

    pub fn production_services(&self) -> Option<TranscriptManagerServices> {
        self.services
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub fn set_turn_execution(&self, execution: TranscriptTurnExecutionPort) {
        self.runner_registry.set_turn_execution(execution);
    }

    pub fn turn_execution(&self) -> Option<TranscriptTurnExecutionPort> {
        self.runner_registry.turn_execution()
    }

    pub fn can_execute(&self) -> bool {
        self.runner_registry.can_execute()
    }

    pub fn can_execute_group_member(&self) -> bool {
        self.runner_registry.can_execute_group_member()
    }

    pub async fn is_run_ready(&self) -> bool {
        self.runner_registry.is_run_ready().await
    }

    pub fn create_runner(
        &self,
        session: Value,
        hooks: Value,
    ) -> Result<Value, TurnExecutionError> {
        self.runner_registry.create_runner(session, hooks)
    }

    pub fn create_group_member_runner(
        &self,
        session: Value,
        hooks: Value,
        overrides: Value,
    ) -> Result<Value, TurnExecutionError> {
        self.runner_registry
            .create_group_member_runner(session, hooks, overrides)
    }

    pub fn prompt_acceptance_status(
        &self,
        args: &Value,
    ) -> Result<Value, ProductionSendError> {
        self.transcript_runtime.prompt_acceptance_status(args)
    }

    pub fn active_agent_id(&self) -> Option<String> {
        self.transcript_runtime.active_agent_id(&self.session_workers)
    }

    fn prepare_live_bounded_open(
        &self,
        agent_id: &str,
        now_ms: f64,
    ) -> Result<bool, String> {
        if self.active_agent_id().as_deref() == Some(agent_id) {
            return Ok(true);
        }
        if self
            .transcript_runtime
            .session_runtime()
            .is_live_session(agent_id)
        {
            self.switch_agent(agent_id, now_ms)?;
            return Ok(true);
        }
        Ok(false)
    }

    pub fn open_agent_tail(
        &self,
        agent_id: &str,
        query: TranscriptWindowQuery,
        now_ms: f64,
    ) -> Result<TranscriptPage, String> {
        if self.prepare_live_bounded_open(agent_id, now_ms)? {
            return self.session_workers.read_agent_transcript_tail(agent_id, query);
        }
        let page = self.session_workers.read_agent_transcript_tail(agent_id, query)?;
        self.session_workers.mark_agent_viewed(agent_id, now_ms, false)?;
        Ok(page)
    }

    pub fn open_agent_windowed(
        &self,
        agent_id: &str,
        query: TranscriptWindowQuery,
        now_ms: f64,
    ) -> Result<TranscriptWindow<BTreeMap<String, usize>>, String> {
        if self.prepare_live_bounded_open(agent_id, now_ms)? {
            return self.session_workers.read_agent_transcript_window(agent_id, query);
        }
        let window = self.session_workers.read_agent_transcript_window(agent_id, query)?;
        self.session_workers.mark_agent_viewed(agent_id, now_ms, false)?;
        Ok(window)
    }

    pub fn switch_agent(
        &self,
        agent_id: &str,
        now_ms: f64,
    ) -> Result<Vec<Value>, String> {
        let transcript = self
            .transcript_runtime
            .switch_agent(&self.session_workers, agent_id, now_ms)?;
        let next = self.automation_runtime.watch_agent_automations(agent_id)?;
        let previous = self
            .watched_automation_store
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .replace(next);
        if let Some(previous) = previous {
            previous.set_on_change(None);
        }
        self.workflow_commands.watch_agent_workflows(agent_id)?;
        Ok(transcript)
    }

    pub fn set_window_focused(
        &self,
        is_focused: bool,
        now_ms: f64,
    ) -> Result<bool, String> {
        self.transcript_runtime
            .session_runtime()
            .set_window_focused(&self.session_workers, is_focused, now_ms)
    }

    pub fn get_async_tasks(
        &self,
        agent_id: &str,
        live_tasks: &[AsyncTask],
    ) -> Vec<AsyncTask> {
        self.transcript_runtime.get_async_tasks(agent_id, live_tasks)
    }

    pub fn decorate_agent_summaries(&self, value: &mut Value) {
        self.transcript_runtime.decorate_agent_summaries(value);
    }

    pub fn clear_agent_durable_recovery(&self, agent_id: &str) {
        self.transcript_runtime.clear_agent_durable_recovery(agent_id);
    }

    pub fn is_disposed(&self) -> bool {
        self.disposed.load(Ordering::Acquire)
    }

    /// Mirrors the frozen manager's process-stop ownership without deleting
    /// durable recovery/ack state that must survive restart.
    pub fn dispose(&self) {
        if self.disposed.swap(true, Ordering::AcqRel) {
            return;
        }

        self.transcript_runtime.dispose();

        if let Some(handoff) = self
            .handoff_service
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
        {
            handoff.clear_all();
        }

        self.ack_obligations.dispose();
        self.automation_runtime.dispose();
        self.workflow_commands.dispose();

        if let Some(store) = self
            .watched_automation_store
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
        {
            store.set_on_change(None);
        }

        self.runner_registry
            .cancel_all("TranscriptManager disposed");
        self.session_workers.shutdown_with_checkpoint();
    }
}
