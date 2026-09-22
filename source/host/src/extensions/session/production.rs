use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::agent_isolation::{
    AgentWorkerPool, ProductionAgentStoreWorkerBackend, WorkerBlobStore,
    create_production_agent_store_worker_backend,
};
use crate::storage::agent_paths::get_sand_agents_root_dir;

use super::conversation_blobs_path::conversation_blobs_path;
use super::session_paths::get_agent_db_path;

pub const PRODUCTION_BLOB_BUSY_TIMEOUT_MS: u64 = 5_000;

pub type ProductionAgentWorkerPool = AgentWorkerPool<ProductionAgentStoreWorkerBackend>;
pub type ProductionWorkerBlobStore = WorkerBlobStore<ProductionAgentStoreWorkerBackend>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedAgentBlobStore {
    pub session_db_path: PathBuf,
    pub blob_db_path: PathBuf,
    pub latest_root_blob_id: Option<Vec<u8>>,
}

/// Shipping Host owner for the Grok session materialization worker boundary.
///
/// The pool is retained for the Host lifetime so per-agent blob workers are
/// reused across turns and are closed as part of Host shutdown. The remaining
/// recovery/metadata-GC/session materialization modules stay explicitly
/// non-final until their frozen Grok contracts are ported.
pub struct ProductionSessionWorkers {
    agents_root: PathBuf,
    pool: Arc<ProductionAgentWorkerPool>,
}

impl ProductionSessionWorkers {
    pub fn production() -> Self {
        Self::with_agents_root(
            get_sand_agents_root_dir(None),
            PRODUCTION_BLOB_BUSY_TIMEOUT_MS,
        )
    }

    pub fn with_agents_root(
        agents_root: impl Into<PathBuf>,
        busy_timeout_ms: u64,
    ) -> Self {
        Self {
            agents_root: agents_root.into(),
            pool: Arc::new(AgentWorkerPool::new(
                create_production_agent_store_worker_backend(busy_timeout_ms),
            )),
        }
    }

    pub fn worker_pool(&self) -> Arc<ProductionAgentWorkerPool> {
        Arc::clone(&self.pool)
    }

    pub fn session_db_path(&self, agent_id: &str) -> Result<PathBuf, String> {
        get_agent_db_path(&self.agents_root, agent_id).map_err(|error| error.to_string())
    }

    pub fn create_agent_blob_store(
        &self,
        agent_id: &str,
    ) -> Result<ProductionWorkerBlobStore, String> {
        let session_db_path = self.session_db_path(agent_id)?;
        Ok(WorkerBlobStore::new(
            Arc::clone(&self.pool),
            agent_id,
            conversation_blobs_path(&session_db_path),
            Some(session_db_path),
        ))
    }

    /// Opens the concrete SQLite conversation-blob backend for an already
    /// materialized production agent before Runner inference begins.
    ///
    /// Missing session DBs are not created here: session creation remains owned
    /// by the Session extension. Existing sessions, however, now traverse the
    /// real AgentWorkerPool and legacy-adoption backend on the shipping path.
    pub fn prepare_existing_agent(
        &self,
        agent_id: &str,
    ) -> Result<Option<PreparedAgentBlobStore>, String> {
        let store = self.create_agent_blob_store(agent_id)?;
        let session_db_path = store
            .legacy_blob_db_path
            .as_ref()
            .cloned()
            .ok_or_else(|| "production session store is missing its session DB path".to_string())?;
        if !session_db_path.is_file() {
            return Ok(None);
        }

        let latest_root_blob_id = futures::executor::block_on(
            self.pool.find_latest_root_blob_id(
                agent_id,
                &store.blob_db_path,
                store.legacy_blob_db_path.as_deref(),
            ),
        )
        .map_err(|error| error.to_string())?;

        if let Some(blob_id) = latest_root_blob_id.as_deref() {
            let _ = futures::executor::block_on(store.get_blob(&(), blob_id))
                .map_err(|error| error.to_string())?;
        }

        Ok(Some(PreparedAgentBlobStore {
            session_db_path,
            blob_db_path: store.blob_db_path.clone(),
            latest_root_blob_id,
        }))
    }

    pub fn active_worker_count(&self) -> usize {
        self.pool.active_worker_count()
    }

    pub fn shutdown(&self) {
        futures::executor::block_on(self.pool.close_all());
    }

    pub fn agents_root(&self) -> &Path {
        &self.agents_root
    }
}
