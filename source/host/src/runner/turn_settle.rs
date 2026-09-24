#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalOutcome {
    Completed,
    Failed { retryable: bool, message: String },
    Cancelled,
    WaitingUser,
}

#[derive(Debug, Default)]
pub struct TurnSettlement {
    outcome: Option<TerminalOutcome>,
}

impl TurnSettlement {
    pub fn settle(&mut self, outcome: TerminalOutcome) -> Result<(), &'static str> {
        if self.outcome.is_some() {
            return Err("turn already has a terminal outcome");
        }
        self.outcome = Some(outcome);
        Ok(())
    }

    pub fn outcome(&self) -> Option<&TerminalOutcome> {
        self.outcome.as_ref()
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self.outcome,
            Some(TerminalOutcome::Completed | TerminalOutcome::Failed { .. } | TerminalOutcome::Cancelled)
        )
    }
}

use std::future::Future;
use std::pin::Pin;

pub type TurnCheckpointFuture<'a, T> =
    Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait TranscriptCheckpointMirror<Checkpoint, BlobStore>: Send + Sync {
    fn prepare_checkpoint<'a>(
        &'a self,
        transcript_id: &'a str,
        checkpoint: &'a Checkpoint,
        blob_store: &'a BlobStore,
        finalize: bool,
        force: bool,
    ) -> TurnCheckpointFuture<'a, Result<(), String>>;

    fn abort_checkpoint<'a>(
        &'a self,
        transcript_id: &'a str,
    ) -> TurnCheckpointFuture<'a, Result<(), String>>;

    fn commit_checkpoint<'a>(
        &'a self,
        transcript_id: &'a str,
        latest_root_blob_id: Option<&'a str>,
    ) -> TurnCheckpointFuture<'a, Result<(), String>>;

    fn skip_checkpoint<'a>(
        &'a self,
        transcript_id: &'a str,
        checkpoint: &'a Checkpoint,
        blob_store: &'a BlobStore,
    ) -> TurnCheckpointFuture<'a, Result<(), String>>;
}

pub trait DurableTurnCheckpointStore<Checkpoint>: Send + Sync {
    fn handle_checkpoint<'a>(
        &'a self,
        checkpoint: &'a Checkpoint,
    ) -> TurnCheckpointFuture<'a, Result<(), String>>;

    fn latest_root_blob_id(&self) -> Option<String>;
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TurnCheckpointPersistenceError {
    #[error("transcript mirror prepare failed: {0}")]
    MirrorPrepare(String),
    #[error("durable Agent checkpoint failed: {0}")]
    Durable(String),
    #[error("transcript mirror commit failed after durable checkpoint: {0}")]
    MirrorCommit(String),
    #[error("transcript mirror skip failed: {0}")]
    MirrorSkip(String),
}

pub async fn persist_checkpoint_with_mirror<
    Checkpoint,
    BlobStore,
    Mirror,
    Store,
    LocalState,
>(
    mirror: Option<&Mirror>,
    store: Option<&Store>,
    transcript_id: &str,
    checkpoint: &Checkpoint,
    blob_store: &BlobStore,
    finalize_checkpoint: bool,
    is_subagent_runner: bool,
    transcript_persistence_enabled: bool,
    set_local_state: LocalState,
) -> Result<(), TurnCheckpointPersistenceError>
where
    Mirror: TranscriptCheckpointMirror<Checkpoint, BlobStore>,
    Store: DurableTurnCheckpointStore<Checkpoint>,
    LocalState: FnOnce(&Checkpoint) -> Result<(), String>,
{
    let prepared = transcript_persistence_enabled.then_some(mirror).flatten();
    if let Some(mirror) = prepared {
        mirror
            .prepare_checkpoint(
                transcript_id,
                checkpoint,
                blob_store,
                finalize_checkpoint,
                finalize_checkpoint || is_subagent_runner,
            )
            .await
            .map_err(TurnCheckpointPersistenceError::MirrorPrepare)?;
    }

    let durable = match store {
        Some(store) => store.handle_checkpoint(checkpoint).await,
        None => set_local_state(checkpoint),
    };
    if let Err(error) = durable {
        if let Some(mirror) = prepared {
            let _ = mirror.abort_checkpoint(transcript_id).await;
        }
        return Err(TurnCheckpointPersistenceError::Durable(error));
    }

    if let Some(mirror) = prepared {
        let root = store.and_then(DurableTurnCheckpointStore::latest_root_blob_id);
        mirror
            .commit_checkpoint(transcript_id, root.as_deref())
            .await
            .map_err(TurnCheckpointPersistenceError::MirrorCommit)?;
    } else if let Some(mirror) = mirror {
        mirror
            .skip_checkpoint(transcript_id, checkpoint, blob_store)
            .await
            .map_err(TurnCheckpointPersistenceError::MirrorSkip)?;
    }

    Ok(())
}
