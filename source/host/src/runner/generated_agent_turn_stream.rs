use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::extensions::inference::provider_session::{
    ProviderMessage, ProviderSessionError,
};

use super::inactive_turn_agent_stream::{
    InactiveTurnAgentLifecycleHooks, InactiveTurnAgentOutputSink,
    InactiveTurnAgentStreamPath, InactiveTurnAgentStreamSource,
    InactiveTurnCheckpointSink, InactiveTurnStreamFuture,
    NoopInactiveTurnAgentLifecycleHooks,
};
use super::production_agent_checkpoint::{
    AgentStateCheckpointSink, TextTurnCheckpointArtifacts,
};
use super::routed_provider_runtime::RoutedProviderCancellation;
use super::turn_agent_composition::TurnAgentComposition;
use super::{
    OuterStreamFuture, OuterStreamPersistence, StreamCancelReason,
    TurnRunOptions,
};

pub struct GeneratedAgentTurnContext<'a> {
    pub data_dir: &'a Path,
    pub lifecycle_messages: &'a [ProviderMessage],
    pub provider_messages: &'a [ProviderMessage],
    pub options: &'a TurnRunOptions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedAgentTurnState {
    pub checkpoint: TextTurnCheckpointArtifacts,
    pub assistant_content: String,
}

#[derive(Debug)]
pub enum GeneratedAgentTurnStreamError {
    Provider(ProviderSessionError),
    Persistence(String),
}

impl From<String> for GeneratedAgentTurnStreamError {
    fn from(value: String) -> Self {
        Self::Persistence(value)
    }
}

impl GeneratedAgentTurnStreamError {
    pub fn into_provider_error(self) -> ProviderSessionError {
        match self {
            Self::Provider(error) => error,
            Self::Persistence(error) => ProviderSessionError::Protocol(format!(
                "generated Agent stream persistence failed: {error}"
            )),
        }
    }
}

struct ProductionGeneratedAgentStreamSource {
    composition: TurnAgentComposition,
    checkpoint_sink: Arc<dyn AgentStateCheckpointSink>,
}

impl<'ctx>
    InactiveTurnAgentStreamSource<
        GeneratedAgentTurnContext<'ctx>,
        GeneratedAgentTurnState,
    > for ProductionGeneratedAgentStreamSource
{
    type Error = GeneratedAgentTurnStreamError;

    fn start_stream<'a>(
        &'a self,
        context: &'a GeneratedAgentTurnContext<'ctx>,
        _resume_from: Option<&'a GeneratedAgentTurnState>,
        _persist_checkpoint: &'a mut dyn InactiveTurnCheckpointSink<
            GeneratedAgentTurnContext<'ctx>,
            GeneratedAgentTurnState,
        >,
        output: &'a mut dyn InactiveTurnAgentOutputSink,
    ) -> InactiveTurnStreamFuture<
        'a,
        Result<GeneratedAgentTurnState, Self::Error>,
    > {
        Box::pin(async move {
            let mut forward_delta = |delta: &str, accumulated: &str| {
                output.on_text_delta(delta, accumulated);
            };
            let assistant_content = self
                .composition
                .run(
                    context.data_dir,
                    context.provider_messages,
                    &mut forward_delta,
                )
                .map_err(GeneratedAgentTurnStreamError::Provider)?;
            let checkpoint = self
                .checkpoint_sink
                .stage_text_turn(
                    context.lifecycle_messages,
                    context.options,
                    &assistant_content,
                )
                .map_err(GeneratedAgentTurnStreamError::Provider)?;
            Ok(GeneratedAgentTurnState {
                checkpoint,
                assistant_content,
            })
        })
    }
}

pub struct ProductionGeneratedAgentPersistence {
    checkpoint_sink: Arc<dyn AgentStateCheckpointSink>,
    cancellation: RoutedProviderCancellation,
    generation: u64,
    upgrade_quiescing: Arc<AtomicBool>,
    turn_quiesced: Arc<AtomicBool>,
}

impl ProductionGeneratedAgentPersistence {
    pub fn new(
        checkpoint_sink: Arc<dyn AgentStateCheckpointSink>,
        cancellation: RoutedProviderCancellation,
        generation: u64,
        upgrade_quiescing: Arc<AtomicBool>,
        turn_quiesced: Arc<AtomicBool>,
    ) -> Self {
        Self {
            checkpoint_sink,
            cancellation,
            generation,
            upgrade_quiescing,
            turn_quiesced,
        }
    }
}

impl<'ctx>
    OuterStreamPersistence<
        GeneratedAgentTurnContext<'ctx>,
        GeneratedAgentTurnState,
    > for ProductionGeneratedAgentPersistence
{
    fn generation(&self) -> u64 {
        self.generation
    }

    fn run_generation(&self) -> u64 {
        self.generation
    }

    fn prepare_checkpoint_for_persistence(
        &self,
        _checkpoint: &mut GeneratedAgentTurnState,
    ) {}

    fn persist_step_checkpoint<'a>(
        &'a self,
        context: &'a GeneratedAgentTurnContext<'ctx>,
        checkpoint: &'a GeneratedAgentTurnState,
    ) -> OuterStreamFuture<'a, Result<(), String>> {
        Box::pin(async move {
            self.checkpoint_sink
                .persist_staged_text_turn(context.options, &checkpoint.checkpoint)
                .map_err(|error| error.to_string())
        })
    }

    fn persist_final_state<'a>(
        &'a self,
        context: &'a GeneratedAgentTurnContext<'ctx>,
        checkpoint: &'a GeneratedAgentTurnState,
    ) -> OuterStreamFuture<'a, Result<(), String>> {
        Box::pin(async move {
            self.checkpoint_sink
                .persist_staged_text_turn(context.options, &checkpoint.checkpoint)
                .map_err(|error| error.to_string())
        })
    }

    fn commit_disk_pressure_reminder(&self) {}

    fn release_disk_pressure_reminder(&self) {}

    fn is_awaiting_user_selection(&self) -> bool {
        false
    }

    fn is_quiescing_for_upgrade(&self) -> bool {
        self.upgrade_quiescing.load(Ordering::Acquire)
    }

    fn mark_quiesced_for_upgrade(&self) {
        self.turn_quiesced.store(true, Ordering::Release);
    }

    fn cancel_run(&self, cancellation: StreamCancelReason) {
        self.cancellation.cancel(cancellation.reason);
    }
}

struct LiveDeltaSink<'a> {
    callback: &'a mut dyn FnMut(&str, &str),
}

impl InactiveTurnAgentOutputSink for LiveDeltaSink<'_> {
    fn on_text_delta(&mut self, delta: &str, accumulated: &str) {
        (self.callback)(delta, accumulated);
    }
}

pub fn run_production_generated_agent_stream(
    composition: TurnAgentComposition,
    checkpoint_sink: Arc<dyn AgentStateCheckpointSink>,
    data_dir: &Path,
    lifecycle_messages: &[ProviderMessage],
    provider_messages: &[ProviderMessage],
    options: &TurnRunOptions,
    generation: u64,
    upgrade_quiescing: Arc<AtomicBool>,
    turn_quiesced: Arc<AtomicBool>,
    on_text_delta: &mut dyn FnMut(&str, &str),
) -> Result<String, ProviderSessionError> {
    let cancellation = composition.cancellation();
    let source: Arc<
        dyn InactiveTurnAgentStreamSource<
            GeneratedAgentTurnContext<'_>,
            GeneratedAgentTurnState,
            Error = GeneratedAgentTurnStreamError,
        >,
    > = Arc::new(ProductionGeneratedAgentStreamSource {
        composition,
        checkpoint_sink: Arc::clone(&checkpoint_sink),
    });
    let path = InactiveTurnAgentStreamPath::new(source);
    let persistence = ProductionGeneratedAgentPersistence::new(
        checkpoint_sink,
        cancellation,
        generation,
        upgrade_quiescing,
        turn_quiesced,
    );
    let hooks = NoopInactiveTurnAgentLifecycleHooks;
    let context = GeneratedAgentTurnContext {
        data_dir,
        lifecycle_messages,
        provider_messages,
        options,
    };
    let mut output = LiveDeltaSink {
        callback: on_text_delta,
    };
    futures::executor::block_on(path.run_lifecycle_with_output(
        &context,
        None,
        &persistence,
        &hooks,
        &mut output,
    ))
    .map(|state| state.assistant_content)
    .map_err(GeneratedAgentTurnStreamError::into_provider_error)
}

#[allow(dead_code)]
fn _lifecycle_hook_anchor(
    hooks: &dyn InactiveTurnAgentLifecycleHooks<GeneratedAgentTurnState>,
) {
    let _ = hooks;
}
