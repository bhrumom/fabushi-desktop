use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde_json::Value;

use crate::extensions::session::production::ProductionSessionWorkers;

use super::ack_obligations::AckObligations;
use super::async_task_union::AsyncTask;
use super::production_runtime::{ProductionSendError, ProductionTranscriptRuntime};
use super::runner_registry::TranscriptRunnerRegistry;

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
    disposed: AtomicBool,
}

impl TranscriptManager {
    pub fn new(
        root_dir: impl AsRef<Path>,
        session_workers: Arc<ProductionSessionWorkers>,
    ) -> Self {
        let root_dir = root_dir.as_ref();
        Self {
            session_workers,
            transcript_runtime: Arc::new(ProductionTranscriptRuntime::new(Some(root_dir))),
            runner_registry: Arc::new(TranscriptRunnerRegistry::default()),
            ack_obligations: Arc::new(AckObligations::new(root_dir)),
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

    pub fn prompt_acceptance_status(
        &self,
        args: &Value,
    ) -> Result<Value, ProductionSendError> {
        self.transcript_runtime.prompt_acceptance_status(args)
    }

    pub fn switch_agent(
        &self,
        agent_id: &str,
        now_ms: f64,
    ) -> Result<Vec<Value>, String> {
        self.transcript_runtime
            .switch_agent(&self.session_workers, agent_id, now_ms)
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
        self.runner_registry
            .cancel_all("TranscriptManager disposed");
        self.session_workers.shutdown();
    }
}
