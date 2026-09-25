use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;

use super::{
    OuterStreamPersistence, persist_outer_stream_checkpoint,
    persist_outer_stream_final_state, release_outer_stream_persistence,
};

pub type InactiveTurnStreamFuture<'a, T> =
    Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Rust adaptation of the frozen generated Agent stream boundary.
///
/// The shipping Rust provider bridge is synchronous at checkpoint callbacks, so
/// checkpoint persistence is deliberately drained before the callback returns.
/// That preserves the reference invariant that a checkpoint cannot authorize
/// resume until its persistence operation has settled.
pub trait InactiveTurnAgentStreamSource<Context, State>: Send + Sync {
    fn start_stream<'a>(
        &'a self,
        context: &'a Context,
        resume_from: Option<&'a State>,
        persist_checkpoint: &'a mut dyn FnMut(&Context, &mut State) -> Result<(), String>,
    ) -> InactiveTurnStreamFuture<'a, Result<State, String>>;
}

pub trait InactiveTurnAgentLifecycleHooks<State>: Send + Sync {
    fn on_completed<'a>(
        &'a self,
        _state: &'a State,
    ) -> InactiveTurnStreamFuture<'a, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }

    fn cleanup<'a>(&'a self) -> InactiveTurnStreamFuture<'a, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }
}

#[derive(Debug, Default)]
pub struct NoopInactiveTurnAgentLifecycleHooks;

impl<State> InactiveTurnAgentLifecycleHooks<State> for NoopInactiveTurnAgentLifecycleHooks {}

pub struct InactiveTurnAgentStreamPath<Context, State> {
    source: Arc<dyn InactiveTurnAgentStreamSource<Context, State>>,
    _marker: PhantomData<fn(Context, State)>,
}

impl<Context, State> Clone for InactiveTurnAgentStreamPath<Context, State> {
    fn clone(&self) -> Self {
        Self {
            source: Arc::clone(&self.source),
            _marker: PhantomData,
        }
    }
}

impl<Context, State> InactiveTurnAgentStreamPath<Context, State>
where
    Context: Send + Sync,
    State: Send + Sync,
{
    pub fn new(source: Arc<dyn InactiveTurnAgentStreamSource<Context, State>>) -> Self {
        Self {
            source,
            _marker: PhantomData,
        }
    }

    pub async fn start_stream(
        &self,
        context: &Context,
        resume_from: Option<&State>,
        persist_checkpoint: &mut dyn FnMut(&Context, &mut State) -> Result<(), String>,
    ) -> Result<State, String> {
        self.source
            .start_stream(context, resume_from, persist_checkpoint)
            .await
    }

    /// Own one dormant generated-Agent stream lifecycle.
    ///
    /// This mirrors createOuterStreamLifecycle: generation-gated step
    /// checkpoints are drained synchronously, completion hooks run before
    /// cleanup, final state is persisted only for a completed stream, and the
    /// disk-pressure claim is released exactly once on every exit path.
    pub async fn run_lifecycle<P, H>(
        &self,
        context: &Context,
        resume_from: Option<&State>,
        persistence: &P,
        hooks: &H,
    ) -> Result<State, String>
    where
        P: OuterStreamPersistence<Context, State>,
        H: InactiveTurnAgentLifecycleHooks<State>,
    {
        let mut persist = |checkpoint_context: &Context, checkpoint: &mut State| {
            futures::executor::block_on(persist_outer_stream_checkpoint(
                persistence,
                checkpoint_context,
                checkpoint,
            ))
            .map(|_| ())
        };

        let stream_result = self
            .start_stream(context, resume_from, &mut persist)
            .await;

        let result = match stream_result {
            Ok(final_state) => {
                let completion = hooks.on_completed(&final_state).await;
                let cleanup = hooks.cleanup().await;
                let final_persist = if completion.is_ok() && cleanup.is_ok() {
                    persist_outer_stream_final_state(
                        persistence,
                        context,
                        &final_state,
                    )
                    .await
                    .map(|_| ())
                } else {
                    Ok(())
                };

                if let Err(error) = completion {
                    Err(error)
                } else if let Err(error) = cleanup {
                    Err(error)
                } else if let Err(error) = final_persist {
                    Err(error)
                } else {
                    Ok(final_state)
                }
            }
            Err(error) => {
                let cleanup = hooks.cleanup().await;
                Err(cleanup.err().unwrap_or(error))
            }
        };

        release_outer_stream_persistence(persistence);
        result
    }
}
