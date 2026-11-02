use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::agent_isolation::{
    ProductionAgentStoreWorkerBackend, WorkerBlobStore,
};
use crate::transcript_mirror::conversation_state_binary::{
    ConversationTurnStructureFields, SubagentPersistedStateFields,
    decode_conversation_state_recovery_fields, decode_conversation_turn_structure_fields,
    decode_subagent_persisted_state_fields,
};

use super::agent_db::{AgentDbSubscription, SandAgentDb};
use super::session_conversation_state::{
    ResolvedConversationState, SessionConversationState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionAgentMetadataKey {
    AgentId,
    LatestRootBlobId,
    Name,
    Mode,
    IsRunEverything,
    ApprovalMode,
    CreatedAt,
    LastUsedModel,
    LastDebugServerPort,
    CurrentPlanUri,
    SubagentInfo,
    BlobEncryptionKey,
}

impl ProductionAgentMetadataKey {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AgentId => "agentId",
            Self::LatestRootBlobId => "latestRootBlobId",
            Self::Name => "name",
            Self::Mode => "mode",
            Self::IsRunEverything => "isRunEverything",
            Self::ApprovalMode => "approvalMode",
            Self::CreatedAt => "createdAt",
            Self::LastUsedModel => "lastUsedModel",
            Self::LastDebugServerPort => "lastDebugServerPort",
            Self::CurrentPlanUri => "currentPlanUri",
            Self::SubagentInfo => "subagentInfo",
            Self::BlobEncryptionKey => "blobEncryptionKey",
        }
    }
}

pub type ProductionMetadataListener =
    Arc<dyn Fn(Option<Value>) + Send + Sync + 'static>;

pub type ProductionWorkerBlobStore =
    WorkerBlobStore<ProductionAgentStoreWorkerBackend>;

#[derive(Debug, Clone, PartialEq)]
pub struct ProductionResolvedSubagentState {
    pub conversation_state: ResolvedConversationState,
    pub created_timestamp_ms: u64,
    pub last_used_timestamp_ms: u64,
    pub subagent_type: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionAgentCheckpointSnapshot {
    pub root_blob_id: Vec<u8>,
    pub checkpoint_bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
enum CheckpointAuthority {
    #[default]
    Uninitialized,
    Ready,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct ProductionAgentCheckpointState {
    latest_root_blob_id: Vec<u8>,
    checkpoint_bytes: Option<Vec<u8>>,
    authority: CheckpointAuthority,
    mutation_in_progress: bool,
    revision: u64,
}

impl ProductionAgentCheckpointState {
    fn require_ready(&self) -> Result<(), String> {
        if self.mutation_in_progress {
            return Err("production AgentStore checkpoint mutation is in progress".into());
        }
        match &self.authority {
            CheckpointAuthority::Ready => Ok(()),
            CheckpointAuthority::Uninitialized => {
                Err("production AgentStore checkpoint authority is uninitialized".into())
            }
            CheckpointAuthority::Failed(error) => Err(format!(
                "production AgentStore checkpoint authority failed: {error}"
            )),
        }
    }

    fn snapshot(&self) -> Result<Option<ProductionAgentCheckpointSnapshot>, String> {
        match (&self.latest_root_blob_id[..], &self.checkpoint_bytes) {
            ([], None) => Ok(None),
            (root, Some(bytes)) if !root.is_empty() => {
                Ok(Some(ProductionAgentCheckpointSnapshot {
                    root_blob_id: root.to_vec(),
                    checkpoint_bytes: bytes.clone(),
                }))
            }
            _ => Err("production AgentStore checkpoint cache identity is inconsistent".into()),
        }
    }
}

enum CheckpointPublication {
    Ready(Option<ProductionAgentCheckpointSnapshot>),
    Legacy(Option<ProductionAgentCheckpointSnapshot>),
    Preserve,
    Failed(String),
}

/// An owner-local, nonblocking mutation claim. The cache mutex is released
/// before worker/SQLite I/O, including synchronous metadata notifications.
struct CheckpointMutation<'a> {
    store: &'a ProductionAgentStore,
    expected_root: Vec<u8>,
    expected_snapshot: Option<ProductionAgentCheckpointSnapshot>,
    revision: u64,
    finished: bool,
}

impl CheckpointMutation<'_> {
    fn finish(mut self, publication: CheckpointPublication) -> Result<(), String> {
        let mut state = self.store.state.lock()
            .map_err(|_| "production AgentStore checkpoint state poisoned".to_string())?;
        if !state.mutation_in_progress || state.revision != self.revision {
            return Err("production AgentStore checkpoint mutation ownership changed".into());
        }
        match publication {
            CheckpointPublication::Ready(snapshot) => {
                install_checkpoint_snapshot(&mut state, snapshot);
                state.authority = CheckpointAuthority::Ready;
            }
            CheckpointPublication::Legacy(snapshot) => {
                install_checkpoint_snapshot(&mut state, snapshot);
                state.authority = CheckpointAuthority::Uninitialized;
            }
            CheckpointPublication::Preserve => {}
            CheckpointPublication::Failed(error) => {
                state.authority = CheckpointAuthority::Failed(error);
            }
        }
        state.mutation_in_progress = false;
        self.finished = true;
        Ok(())
    }
}

impl Drop for CheckpointMutation<'_> {
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        if let Ok(mut state) = self.store.state.lock() {
            if state.mutation_in_progress && state.revision == self.revision {
                state.authority = CheckpointAuthority::Failed(
                    "checkpoint mutation did not complete; checked reset is required".into(),
                );
                state.mutation_in_progress = false;
            }
        }
    }
}

fn install_checkpoint_snapshot(
    state: &mut ProductionAgentCheckpointState,
    snapshot: Option<ProductionAgentCheckpointSnapshot>,
) {
    match snapshot {
        Some(snapshot) => {
            state.latest_root_blob_id = snapshot.root_blob_id;
            state.checkpoint_bytes = Some(snapshot.checkpoint_bytes);
        }
        None => {
            state.latest_root_blob_id.clear();
            state.checkpoint_bytes = None;
        }
    }
}

/// Shipping Rust owner for the frozen AgentStore2 checkpoint boundary.
///
/// WorkerBlobStore owns content-addressed bytes and SandAgentDb owns metadata.
/// This composition advances latestRootBlobId only after the checkpoint blob
/// has been durably handed to the production blob worker.
pub struct ProductionAgentStore {
    pub agent_id: String,
    pub blob_db_path: PathBuf,
    pub legacy_blob_db_path: Option<PathBuf>,
    db: Arc<SandAgentDb>,
    blob_store: ProductionWorkerBlobStore,
    state: Mutex<ProductionAgentCheckpointState>,
}

impl ProductionAgentStore {
    pub fn new(
        db: Arc<SandAgentDb>,
        blob_store: ProductionWorkerBlobStore,
    ) -> Self {
        Self {
            agent_id: blob_store.agent_id.clone(),
            blob_db_path: blob_store.blob_db_path.clone(),
            legacy_blob_db_path: blob_store.legacy_blob_db_path.clone(),
            db,
            blob_store,
            state: Mutex::new(ProductionAgentCheckpointState::default()),
        }
    }

    pub fn subscribe_to_metadata(
        &self,
        key: ProductionAgentMetadataKey,
        callback: ProductionMetadataListener,
    ) -> AgentDbSubscription {
        let db = Arc::clone(&self.db);
        let key_name = key.as_str().to_string();
        let callback_for_listener = Arc::clone(&callback);
        self.db.subscribe_metadata(
            key_name.clone(),
            Arc::new(move || {
                let value = db.get_metadata(&key_name).ok().flatten();
                callback_for_listener(value);
            }),
        )
    }

    pub fn set_metadata(
        &self,
        key: ProductionAgentMetadataKey,
        value: Value,
    ) -> Result<bool, String> {
        if key != ProductionAgentMetadataKey::LatestRootBlobId {
            return self.db
                .set_metadata(key.as_str(), value)
                .map_err(|error| error.to_string());
        }

        let mutation = self.begin_checkpoint_mutation(false)?;
        let before = self.db.read_checkpoint_root_checked();
        let result = self.db.set_metadata(key.as_str(), value)
            .map_err(|error| error.to_string());
        let after = self.db.read_checkpoint_root_checked();
        let binding_unchanged = match (&before, &after) {
            (Ok(before), Ok(after)) => {
                before == after
                    && after.as_deref().unwrap_or(&[]) == mutation.expected_root.as_slice()
            }
            _ => false,
        };
        let publication = if result.is_ok() && binding_unchanged {
            CheckpointPublication::Preserve
        } else {
            CheckpointPublication::Failed(
                "root metadata binding changed or is unavailable; checked reset is required".into(),
            )
        };
        mutation.finish(publication)?;
        result
    }

    pub fn get_metadata(
        &self,
        key: ProductionAgentMetadataKey,
    ) -> Result<Option<Value>, String> {
        self.db
            .get_metadata(key.as_str())
            .map_err(|error| error.to_string())
    }

    pub fn get_id(&self) -> String {
        self.get_metadata(ProductionAgentMetadataKey::AgentId)
            .ok()
            .flatten()
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .unwrap_or_else(|| self.agent_id.clone())
    }

    /// Frozen AgentStore2.getLastRequestIdFromConversation semantics. Missing
    /// turn blobs are skipped and the newest agent turn with a request id wins.
    pub fn get_last_request_id_from_conversation(&self) -> Result<Option<String>, String> {
        let checkpoint = self.latest_checkpoint_bytes().unwrap_or_default();
        let structure = decode_conversation_state_recovery_fields(&checkpoint)
            .map_err(|error| format!("invalid ConversationStateStructure checkpoint: {error}"))?;
        for turn_id in structure.turns.iter().rev() {
            let Some(turn_blob) = self.get_blob(turn_id)? else {
                continue;
            };
            if let Some(ConversationTurnStructureFields::Agent { request_id, .. }) =
                decode_conversation_turn_structure_fields(&turn_blob)
                    .map_err(|error| format!("invalid ConversationTurnStructure blob: {error}"))?
            {
                return Ok(request_id);
            }
        }
        Ok(None)
    }

    /// Shipping equivalent of AgentStore2.getFullConversation. It hydrates the
    /// store's current in-memory checkpoint leniently, matching the frozen API's
    /// skip-missing-blob behavior while reusing the production session decoder.
    pub fn get_full_conversation(&self) -> Result<ResolvedConversationState, String> {
        let Some(checkpoint) = self.latest_checkpoint_bytes() else {
            return Ok(ResolvedConversationState {
                turns: Vec::new(),
                todos: Vec::new(),
                summary: None,
            });
        };
        SessionConversationState::new(self.db.busy_timeout_ms())
            .resolve_conversation_state_bytes_lenient(
                Arc::clone(&self.blob_store.pool),
                &self.agent_id,
                self.db.db_path(),
                &self.blob_db_path,
                &checkpoint,
            )
            .map_err(|error| error.to_string())
    }

    /// Frozen AgentStore2.getFullConversationWithSubagents semantics. Inline
    /// persisted states are the fallback; valid ref blobs override them. A
    /// missing ref is fatal only when no inline entry exists for that subagent.
    pub fn get_full_conversation_with_subagents(
        &self,
    ) -> Result<
        (
            ResolvedConversationState,
            BTreeMap<String, ProductionResolvedSubagentState>,
        ),
        String,
    > {
        let conversation_state = self.get_full_conversation()?;
        let checkpoint = self.latest_checkpoint_bytes().unwrap_or_default();
        let structure = decode_conversation_state_recovery_fields(&checkpoint)
            .map_err(|error| format!("invalid ConversationStateStructure checkpoint: {error}"))?;

        let mut persisted = BTreeMap::<String, SubagentPersistedStateFields>::new();
        for (subagent_id, bytes) in structure.subagent_states {
            persisted.insert(
                subagent_id,
                decode_subagent_persisted_state_fields(&bytes)
                    .map_err(|error| format!("invalid inline subagent state: {error}"))?,
            );
        }

        let mut missing = Vec::new();
        for (subagent_id, blob_id) in structure.subagent_state_refs {
            match self.get_blob(&blob_id)? {
                Some(bytes) => {
                    persisted.insert(
                        subagent_id,
                        decode_subagent_persisted_state_fields(&bytes)
                            .map_err(|error| format!("invalid referenced subagent state: {error}"))?,
                    );
                }
                None if persisted.contains_key(&subagent_id) => {}
                None => missing.push(encode_hex(&blob_id)),
            }
        }
        if !missing.is_empty() {
            return Err(format!(
                "subagent state ref blob not found: {}",
                missing.join(",")
            ));
        }

        let resolver = SessionConversationState::new(self.db.busy_timeout_ms());
        let mut subagent_states = BTreeMap::new();
        for (subagent_id, state) in persisted {
            let Some(root_blob) = state.conversation_state else {
                continue;
            };
            let resolved = resolver
                .resolve_conversation_state_bytes_lenient(
                    Arc::clone(&self.blob_store.pool),
                    &self.agent_id,
                    self.db.db_path(),
                    &self.blob_db_path,
                    &root_blob,
                )
                .map_err(|error| error.to_string())?;
            subagent_states.insert(
                subagent_id,
                ProductionResolvedSubagentState {
                    conversation_state: resolved,
                    created_timestamp_ms: state.created_timestamp_ms,
                    last_used_timestamp_ms: state.last_used_timestamp_ms,
                    subagent_type: state.subagent_type,
                },
            );
        }
        Ok((conversation_state, subagent_states))
    }

    fn begin_checkpoint_mutation(
        &self,
        require_initialized: bool,
    ) -> Result<CheckpointMutation<'_>, String> {
        let mut state = self.state.lock()
            .map_err(|_| "production AgentStore checkpoint state poisoned".to_string())?;
        if state.mutation_in_progress {
            return Err("production AgentStore checkpoint mutation is in progress".into());
        }
        if require_initialized {
            state.require_ready()?;
            let _ = state.snapshot()?;
        }
        let revision = state.revision.checked_add(1)
            .ok_or_else(|| "production AgentStore checkpoint revision exhausted".to_string())?;
        let expected_snapshot = state.snapshot()?;
        state.revision = revision;
        state.mutation_in_progress = true;
        Ok(CheckpointMutation {
            store: self,
            expected_root: state.latest_root_blob_id.clone(),
            expected_snapshot,
            revision,
            finished: false,
        })
    }

    fn require_persisted_root(&self, expected_root: &[u8]) -> Result<(), String> {
        let persisted = self.db.read_checkpoint_root_for_agent_checked(&self.agent_id)
            .map_err(|error| format!("production AgentStore canonical root read failed: {error}"))?;
        if persisted.as_deref().unwrap_or(&[]) != expected_root {
            return Err(
                "production AgentStore checkpoint root changed from its captured binding".into(),
            );
        }
        Ok(())
    }

    /// Validate durable state before any Session recovery side effects. Fixed
    /// root/reference IDs are retained; new writers still content-address bytes.
    pub(super) fn validate_persisted_checkpoint_checked(
        db: &SandAgentDb,
        blob_store: &ProductionWorkerBlobStore,
    ) -> Result<(), String> {
        Self::read_persisted_checkpoint_checked(db, blob_store).map(|_| ())
    }

    fn read_persisted_checkpoint_checked(
        db: &SandAgentDb,
        blob_store: &ProductionWorkerBlobStore,
    ) -> Result<Option<ProductionAgentCheckpointSnapshot>, String> {
        let read_root = || db.read_checkpoint_root_for_agent_checked(&blob_store.agent_id)
            .map_err(|error| format!("production AgentStore canonical root read failed: {error}"));
        let root = read_root()?;
        let snapshot = match root.as_ref() {
            Some(root_blob_id) => {
                let checkpoint_bytes = blob_store.get_blob_blocking(root_blob_id)
                    .map_err(|error| format!("could not read production AgentStore checkpoint: {error}"))?
                    .ok_or_else(|| "production AgentStore checkpoint root blob is missing".to_string())?;
                let snapshot = ProductionAgentCheckpointSnapshot {
                    root_blob_id: root_blob_id.clone(), checkpoint_bytes,
                };
                decode_conversation_state_recovery_fields(&snapshot.checkpoint_bytes)
                    .map_err(|error| format!("invalid production AgentStore checkpoint: {error}"))?;
                Some(snapshot)
            }
            None => None,
        };
        if read_root()? != root {
            return Err("production AgentStore root changed during durable validation".into());
        }
        Ok(snapshot)
    }

    /// Initialize canonical authority without replacing failed durable history
    /// with an empty snapshot. Preserve the published boolean presence API.
    pub fn reset_from_db_checked(&self) -> Result<bool, String> {
        let mutation = self.begin_checkpoint_mutation(false)?;
        let loaded = (|| {
            let snapshot = Self::read_persisted_checkpoint_checked(&self.db, &self.blob_store)?;
            if let Some(snapshot) = snapshot.as_ref() {
                validate_canonical_checkpoint_snapshot(snapshot, |_, id| self.get_blob(id))?;
            }
            self.require_persisted_root(snapshot.as_ref().map(|value| value.root_blob_id.as_slice()).unwrap_or(&[]))?;
            Ok::<_, String>(snapshot)
        })();
        match loaded {
            Ok(snapshot) => {
                let present = snapshot.is_some();
                mutation.finish(CheckpointPublication::Ready(snapshot))?;
                Ok(present)
            }
            Err(error) => {
                mutation.finish(CheckpointPublication::Failed(error.clone()))?;
                Err(error)
            }
        }
    }

    pub(crate) fn validate_cached_checkpoint_checked(&self) -> Result<(), String> {
        self.checkpoint_snapshot_checked().map(|_| ())
    }

    /// Read one initialized cache image, then compare fresh durable root bytes
    /// and direct references without holding its mutex across worker/DB I/O.
    pub fn checkpoint_snapshot_checked(
        &self,
    ) -> Result<Option<ProductionAgentCheckpointSnapshot>, String> {
        let (revision, snapshot) = {
            let state = self.state.lock()
                .map_err(|_| "production AgentStore checkpoint state poisoned".to_string())?;
            state.require_ready()?;
            (state.revision, state.snapshot()?)
        };
        let persisted = Self::read_persisted_checkpoint_checked(&self.db, &self.blob_store)?;
        if persisted != snapshot {
            return Err("production AgentStore durable checkpoint changed; checked reload is required".into());
        }
        if let Some(snapshot) = snapshot.as_ref() {
            validate_canonical_checkpoint_snapshot(snapshot, |_, id| self.get_blob(id))?;
        }
        self.require_persisted_root(snapshot.as_ref().map(|value| value.root_blob_id.as_slice()).unwrap_or(&[]))?;
        {
            let state = self.state.lock()
                .map_err(|_| "production AgentStore checkpoint state poisoned".to_string())?;
            state.require_ready()?;
            if state.revision != revision || state.snapshot()? != snapshot {
                return Err("production AgentStore checkpoint cache changed during validation".into());
            }
        }
        Ok(snapshot)
    }

    /// Frozen AgentStore2.tryResetFromDb semantics: bad metadata, missing blob,
    /// or malformed protobuf resets the visible cache and returns false. This
    /// lenient operation does not establish canonical expected-root authority.
    pub fn try_reset_from_db(&self) -> bool {
        let Ok(mutation) = self.begin_checkpoint_mutation(false) else {
            return false;
        };
        let loaded = (|| {
            let root_blob_id = self.db.get_latest_root_blob_id().ok()?;
            if root_blob_id.is_empty() {
                return None;
            }
            let checkpoint_bytes = self.blob_store.get_blob_blocking(&root_blob_id).ok()??;
            decode_conversation_state_recovery_fields(&checkpoint_bytes).ok()?;
            Some(ProductionAgentCheckpointSnapshot { root_blob_id, checkpoint_bytes })
        })();
        let restored = loaded.is_some();
        mutation.finish(CheckpointPublication::Legacy(loaded)).is_ok() && restored
    }

    /// Frozen AgentStore2.handleCheckpoint semantics:
    /// protobuf bytes -> SHA-256 blob id -> blob write -> metadata root update.
    ///
    /// The async owner is the production checkpoint path. The synchronous
    /// wrapper remains for legacy synchronous callers, but Runner settle must
    /// await this method so it never nests a LocalPool executor.
    /// Shipping checkpoint boundary. Blob bytes are content addressed first;
    /// then SandAgentDb advances latestRootBlobId through the captured-root
    /// CAS. A concurrent checkpoint or lifecycle transition therefore cannot
    /// be overwritten by this stale owner.
    pub fn handle_checkpoint_bytes_and_confirm_user_message_blocking(
        &self,
        checkpoint: &[u8],
        message_id: &str,
    ) -> Result<Vec<u8>, String> {
        self.commit_checkpoint_bytes(checkpoint, Some(message_id))
    }

    pub async fn handle_checkpoint_bytes_and_confirm_user_message_async(
        &self,
        checkpoint: &[u8],
        message_id: &str,
    ) -> Result<Vec<u8>, String> {
        self.handle_checkpoint_bytes_and_confirm_user_message_blocking(
            checkpoint,
            message_id,
        )
    }

    pub fn handle_checkpoint_bytes_blocking(
        &self,
        checkpoint: &[u8],
    ) -> Result<Vec<u8>, String> {
        self.commit_checkpoint_bytes(checkpoint, None)
    }

    fn commit_checkpoint_bytes(
        &self,
        checkpoint: &[u8],
        confirmed_message_id: Option<&str>,
    ) -> Result<Vec<u8>, String> {
        self.commit_checkpoint_bytes_before_write(checkpoint, confirmed_message_id, || {})
    }

    // The ordinary writer and deterministic fault controls share this exact
    // prewrite boundary. No callback runs while the checkpoint mutex is held.
    fn commit_checkpoint_bytes_before_write(
        &self,
        checkpoint: &[u8],
        confirmed_message_id: Option<&str>,
        before_write: impl FnOnce(),
    ) -> Result<Vec<u8>, String> {
        decode_conversation_state_recovery_fields(checkpoint)
            .map_err(|error| format!("invalid ConversationStateStructure checkpoint: {error}"))?;

        // Initialization proves the expected CAS root, independently of child
        // completeness. Compatibility payloads retain protobuf-only acceptance.
        let mutation = self.begin_checkpoint_mutation(true)?;
        // Refusal here precedes any blob/DB mutation. Explicit durable repair
        // may retry against the retained cache without creating empty authority.
        let preflight = Self::read_persisted_checkpoint_checked(&self.db, &self.blob_store);
        let preflight_error = match preflight {
            Ok(persisted) if persisted == mutation.expected_snapshot => None,
            Ok(_) => Some("production AgentStore durable checkpoint changed; checked reload is required".to_string()),
            Err(error) => Some(error),
        };
        if let Some(error) = preflight_error {
            mutation.finish(CheckpointPublication::Preserve)?;
            return Err(error);
        }
        before_write();
        if let Err(error) = self.require_persisted_root(&mutation.expected_root) {
            mutation.finish(CheckpointPublication::Preserve)?;
            return Err(error);
        }
        let root_blob_id = Sha256::digest(checkpoint).to_vec();
        let committed = (|| {
            self.blob_store.set_blob_blocking(&root_blob_id, checkpoint)
                .map_err(|error| error.to_string())?;
            let committed = match confirmed_message_id {
                Some(message_id) => self.db
                    .commit_checkpoint_root_and_confirm_user_message(
                        &mutation.expected_root,
                        &root_blob_id,
                        message_id,
                    )
                    .map(|confirmed| confirmed.is_some()),
                None => self.db.compare_and_set_latest_root_blob_id(
                    &mutation.expected_root,
                    &root_blob_id,
                ),
            }
            .map_err(|error| error.to_string())?;
            if committed {
                // SQLite listeners run synchronously before the commit API
                // returns. Never publish a cache rebound by one of those calls.
                self.require_persisted_root(&root_blob_id)?;
            }
            Ok::<_, String>(committed)
        })();
        match committed {
            Ok(true) => {
                mutation.finish(CheckpointPublication::Ready(Some(
                    ProductionAgentCheckpointSnapshot {
                        root_blob_id: root_blob_id.clone(),
                        checkpoint_bytes: checkpoint.to_vec(),
                    },
                )))?;
                Ok(root_blob_id)
            }
            Ok(false) => {
                mutation.finish(CheckpointPublication::Preserve)?;
                Err(if confirmed_message_id.is_some() {
                    "latestRootBlobId and confirmed user transcript entry were not committed atomically"
                        .into()
                } else {
                    "latestRootBlobId changed before checkpoint commit; stale owner rejected".into()
                })
            }
            Err(error) => {
                mutation.finish(CheckpointPublication::Failed(error.clone()))?;
                Err(error)
            }
        }
    }

    pub async fn handle_checkpoint_bytes_async(
        &self,
        checkpoint: &[u8],
    ) -> Result<Vec<u8>, String> {
        self.handle_checkpoint_bytes_blocking(checkpoint)
    }

    pub fn handle_checkpoint_bytes(
        &self,
        checkpoint: &[u8],
    ) -> Result<Vec<u8>, String> {
        self.handle_checkpoint_bytes_blocking(checkpoint)
    }

    pub fn latest_root_blob_id(&self) -> Vec<u8> {
        self.state
            .lock()
            .map(|state| state.latest_root_blob_id.clone())
            .unwrap_or_default()
    }

    pub fn latest_checkpoint_bytes(&self) -> Option<Vec<u8>> {
        self.state
            .lock()
            .ok()
            .and_then(|state| state.checkpoint_bytes.clone())
    }

    /// Poison-aware legacy cache projections. They do not prove durable
    /// readiness; production consumers use checkpoint_snapshot_checked.
    pub fn latest_root_blob_id_checked(&self) -> Result<Vec<u8>, String> {
        self.state.lock()
            .map(|state| state.latest_root_blob_id.clone())
            .map_err(|_| "production AgentStore checkpoint state poisoned".to_string())
    }

    pub fn latest_checkpoint_bytes_checked(&self) -> Result<Option<Vec<u8>>, String> {
        self.state.lock()
            .map(|state| state.checkpoint_bytes.clone())
            .map_err(|_| "production AgentStore checkpoint state poisoned".to_string())
    }

    pub fn get_blob(
        &self,
        blob_id: &[u8],
    ) -> Result<Option<Vec<u8>>, String> {
        self.blob_store
            .get_blob_blocking(blob_id)
            .map_err(|error| error.to_string())
    }

    pub fn flush(&self) {
        self.blob_store.flush_blocking();
    }

}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CanonicalCheckpointReferenceKind {
    RootPrompt,
    Todo,
    Summary,
    Turn,
    SummaryArchive,
}

impl CanonicalCheckpointReferenceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RootPrompt => "root prompt message",
            Self::Todo => "todo",
            Self::Summary => "summary",
            Self::Turn => "conversation turn",
            Self::SummaryArchive => "summary archive",
        }
    }

    pub(crate) fn has_legacy_recovery_size_limit(self) -> bool {
        matches!(self, Self::Todo | Self::Summary | Self::Turn)
    }
}

/// CKR-v1's bounded root/direct-reference policy, shared by canonical owner
/// reads and the existing explicit-empty-root repair before its metadata CAS.
/// Unknown fields and other reference graphs retain their existing owners.
pub(crate) fn validate_canonical_checkpoint_snapshot(
    snapshot: &ProductionAgentCheckpointSnapshot,
    mut read_reference: impl FnMut(
        &[CanonicalCheckpointReferenceKind],
        &[u8],
    ) -> Result<Option<Vec<u8>>, String>,
) -> Result<(), String> {
    if snapshot.root_blob_id.is_empty() {
        return Err("production AgentStore checkpoint root blob id must be nonempty".into());
    }
    let fields = decode_conversation_state_recovery_fields(&snapshot.checkpoint_bytes)
        .map_err(|error| format!("invalid production AgentStore checkpoint: {error}"))?;
    let mut references = BTreeMap::<Vec<u8>, Vec<CanonicalCheckpointReferenceKind>>::new();
    for (kind, ids) in [
        (CanonicalCheckpointReferenceKind::RootPrompt, fields.root_prompt_messages_json),
        (CanonicalCheckpointReferenceKind::Todo, fields.todos),
        (CanonicalCheckpointReferenceKind::Summary, fields.summary.into_iter().collect()),
        (CanonicalCheckpointReferenceKind::Turn, fields.turns),
        (CanonicalCheckpointReferenceKind::SummaryArchive, fields.summary_archives),
    ] {
        for id in ids {
            if id.is_empty() {
                return Err(format!(
                    "canonical checkpoint {} blob reference must be nonempty",
                    kind.label(),
                ));
            }
            let kinds = references.entry(id).or_default();
            if !kinds.contains(&kind) {
                kinds.push(kind);
            }
        }
    }
    for (id, kinds) in references {
        // Pass every kind to the reader: deduplication must not discard a
        // stricter maintenance size bound when the same ID has several roles.
        let _bytes = read_reference(&kinds, &id)?
            .ok_or_else(|| format!(
                "canonical checkpoint {} blob is missing",
                kinds[0].label(),
            ))?;
    }
    Ok(())
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod checked_state_tests {
    use super::*;
    use crate::extensions::session::production::ProductionSessionWorkers;

    #[test]
    fn final_prewrite_refusal_preserves_authority_for_retry_after_explicit_binding_repair() {
        let root = std::env::temp_dir().join(format!(
            "fabushi-agent-store-prewrite-{}", uuid::Uuid::new_v4(),
        ));
        let workers = ProductionSessionWorkers::with_agents_root(&root, 500);
        let session = workers.materialize_session_with_active(None, "user", None, None)
            .expect("session");
        session.db.append_transcript_entry(&serde_json::json!({
            "id": "prewrite-user", "kind": "message", "role": "user",
            "content": "pending", "confirmed": false,
        })).expect("pending user");
        let foreign = SandAgentDb::open_existing_agent(&session.record.db_path, 500)
            .expect("independent persisted metadata owner");
        let next = vec![0x9a, 0x06, 0x03, b'n', b'e', b'w'];
        let next_root = Sha256::digest(&next).to_vec();
        let error = session.agent_store.commit_checkpoint_bytes_before_write(
            &next, Some("prewrite-user"), || {
                assert!(foreign.set_metadata("latestRootBlobId", serde_json::json!("dead"))
                    .expect("binding changed after durable preflight"));
            },
        ).expect_err("last root check refuses before the first persistence attempt");
        assert!(error.contains("captured binding"), "{error}");
        assert_eq!(session.agent_store.get_blob(&next_root).expect("no new blob"), None);
        assert_eq!(session.agent_store.latest_checkpoint_bytes(), None);
        assert_eq!(session.db.get_metadata("latestRootBlobId").expect("foreign binding retained"), Some(serde_json::json!("dead")));
        assert_eq!(session.db.get_transcript_entry_by_id("prewrite-user").expect("pending row")
            .and_then(|row| row.get("confirmed").and_then(Value::as_bool)), Some(false));
        assert!(foreign.set_metadata("latestRootBlobId", serde_json::json!(""))
            .expect("explicit binding repair"));
        assert_eq!(session.agent_store.handle_checkpoint_bytes_and_confirm_user_message_blocking(
            &next, "prewrite-user",
        ).expect("known prewrite refusal needs no checked reset"), next_root);
        assert_eq!(session.db.get_transcript_entry_by_id("prewrite-user").expect("confirmed row")
            .and_then(|row| row.get("confirmed").and_then(Value::as_bool)), Some(true));
        foreign.close(false);
        workers.shutdown();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn poisoned_state_cannot_be_read_or_written_as_an_empty_checkpoint() {
        let root = std::env::temp_dir().join(format!(
            "fabushi-agent-store-poison-{}", uuid::Uuid::new_v4(),
        ));
        let workers = ProductionSessionWorkers::with_agents_root(&root, 500);
        let session = workers
            .materialize_session_with_active(None, "user", None, None)
            .expect("session");
        let original_root = session.agent_store.handle_checkpoint_bytes(&[]).expect("checkpoint");
        let metadata = session.db.read_kv("metadata").expect("original metadata");
        let poisoned_store = Arc::clone(&session.agent_store);
        assert!(std::thread::spawn(move || {
            let _state = poisoned_store.state.lock().expect("initial checkpoint state");
            panic!("poison checkpoint state for the refusal oracle");
        }).join().is_err());

        assert!(session.agent_store.latest_checkpoint_bytes_checked().is_err());
        assert!(session.agent_store.latest_root_blob_id_checked().is_err());
        assert!(session.agent_store.reset_from_db_checked().is_err());
        assert!(workers.open_agent_store_owner(&session.record.id).is_err());
        let next = vec![0x9a, 0x06, 0x03, b'n', b'e', b'w'];
        assert!(session.agent_store.handle_checkpoint_bytes(&next).is_err());
        assert!(session.agent_store
            .handle_checkpoint_bytes_and_confirm_user_message_blocking(&next, "pending-message")
            .is_err());
        assert_eq!(session.db.read_kv("metadata").expect("unchanged metadata"), metadata);
        assert_eq!(session.db.get_latest_root_blob_id().expect("unchanged root"), original_root);
        assert_eq!(session.agent_store.get_blob(&Sha256::digest(&next)).expect("no failed-write blob"), None);
        // Compatibility projections still keep their old return types; the
        // production reads above must not use their empty fallback.
        assert_eq!(session.agent_store.latest_checkpoint_bytes(), None);
        assert!(session.agent_store.latest_root_blob_id().is_empty());
        workers.shutdown();
        let _ = std::fs::remove_dir_all(root);
    }
}
