use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::agent_isolation::{
    ProductionAgentStoreWorkerBackend, WorkerBlobStore,
};
use crate::transcript_mirror::conversation_state_binary::{
    decode_conversation_state_recovery_fields,
};

use super::agent_db::SandAgentDb;

pub type ProductionWorkerBlobStore =
    WorkerBlobStore<ProductionAgentStoreWorkerBackend>;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct ProductionAgentCheckpointState {
    latest_root_blob_id: Vec<u8>,
    checkpoint_bytes: Option<Vec<u8>>,
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

    pub fn db(&self) -> Arc<SandAgentDb> {
        Arc::clone(&self.db)
    }

    /// Frozen AgentStore2.tryResetFromDb semantics: bad metadata, missing blob,
    /// or malformed protobuf resets the in-memory checkpoint and returns false.
    pub fn try_reset_from_db(&self) -> bool {
        let root = match self.db.get_latest_root_blob_id() {
            Ok(root) => root,
            Err(_) => {
                self.clear_checkpoint();
                return false;
            }
        };
        if root.is_empty() {
            self.clear_checkpoint();
            return false;
        }
        let blob = match futures::executor::block_on(
            self.blob_store.get_blob(&(), &root),
        ) {
            Ok(Some(blob)) => blob,
            _ => {
                self.clear_checkpoint();
                return false;
            }
        };
        if decode_conversation_state_recovery_fields(&blob).is_err() {
            self.clear_checkpoint();
            return false;
        }
        if let Ok(mut state) = self.state.lock() {
            state.latest_root_blob_id = root;
            state.checkpoint_bytes = Some(blob);
            true
        } else {
            false
        }
    }

    /// Frozen AgentStore2.handleCheckpoint semantics:
    /// protobuf bytes -> SHA-256 blob id -> blob write -> metadata root update.
    ///
    /// The async owner is the production checkpoint path. The synchronous
    /// wrapper remains for legacy synchronous callers, but Runner settle must
    /// await this method so it never nests a LocalPool executor.
    /// Shipping direct-user checkpoint boundary. Blob bytes are content
    /// addressed first; then SandAgentDb advances latestRootBlobId and the
    /// transcript recovery watermark in one IMMEDIATE SQLite transaction.
    pub async fn handle_checkpoint_bytes_and_confirm_user_message_async(
        &self,
        checkpoint: &[u8],
        message_id: &str,
    ) -> Result<Vec<u8>, String> {
        decode_conversation_state_recovery_fields(checkpoint)
            .map_err(|error| format!("invalid ConversationStateStructure checkpoint: {error}"))?;

        let root_blob_id = Sha256::digest(checkpoint).to_vec();
        self.blob_store
            .set_blob(&(), &root_blob_id, checkpoint)
            .await
            .map_err(|error| error.to_string())?;

        let expected_root = self.latest_root_blob_id();
        let confirmed = self
            .db
            .commit_checkpoint_root_and_confirm_user_message(
                &expected_root,
                &root_blob_id,
                message_id,
            )
            .map_err(|error| error.to_string())?;
        if confirmed.is_none() {
            return Err(
                "latestRootBlobId and confirmed user transcript entry were not committed atomically"
                    .into(),
            );
        }

        let mut state = self
            .state
            .lock()
            .map_err(|_| "production AgentStore checkpoint state poisoned".to_string())?;
        state.latest_root_blob_id = root_blob_id.clone();
        state.checkpoint_bytes = Some(checkpoint.to_vec());
        Ok(root_blob_id)
    }

    pub async fn handle_checkpoint_bytes_async(
        &self,
        checkpoint: &[u8],
    ) -> Result<Vec<u8>, String> {
        decode_conversation_state_recovery_fields(checkpoint)
            .map_err(|error| format!("invalid ConversationStateStructure checkpoint: {error}"))?;

        let root_blob_id = Sha256::digest(checkpoint).to_vec();
        self.blob_store
            .set_blob(&(), &root_blob_id, checkpoint)
            .await
            .map_err(|error| error.to_string())?;

        let advanced = self
            .db
            .set_metadata(
                "latestRootBlobId",
                Value::String(encode_hex(&root_blob_id)),
            )
            .map_err(|error| error.to_string())?;
        if !advanced {
            return Err("latestRootBlobId metadata update was not committed".into());
        }

        let mut state = self
            .state
            .lock()
            .map_err(|_| "production AgentStore checkpoint state poisoned".to_string())?;
        state.latest_root_blob_id = root_blob_id.clone();
        state.checkpoint_bytes = Some(checkpoint.to_vec());
        Ok(root_blob_id)
    }

    pub fn handle_checkpoint_bytes(
        &self,
        checkpoint: &[u8],
    ) -> Result<Vec<u8>, String> {
        futures::executor::block_on(
            self.handle_checkpoint_bytes_async(checkpoint),
        )
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

    pub fn get_blob(
        &self,
        blob_id: &[u8],
    ) -> Result<Option<Vec<u8>>, String> {
        futures::executor::block_on(
            self.blob_store.get_blob(&(), blob_id),
        )
        .map_err(|error| error.to_string())
    }

    pub fn flush(&self) {
        futures::executor::block_on(self.blob_store.flush(&()));
    }

    fn clear_checkpoint(&self) {
        if let Ok(mut state) = self.state.lock() {
            *state = ProductionAgentCheckpointState::default();
        }
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
