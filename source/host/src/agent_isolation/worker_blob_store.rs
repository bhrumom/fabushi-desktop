use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::agent_worker_pool::{AgentBlobWorkerBackend, AgentWorkerPool};

pub struct WorkerBlobStore<Backend> {
    pub pool: Arc<AgentWorkerPool<Backend>>,
    pub agent_id: String,
    pub blob_db_path: PathBuf,
    pub legacy_blob_db_path: Option<PathBuf>,
}

impl<Backend> WorkerBlobStore<Backend> {
    pub fn new(
        pool: Arc<AgentWorkerPool<Backend>>,
        agent_id: impl Into<String>,
        blob_db_path: impl Into<PathBuf>,
        legacy_blob_db_path: Option<PathBuf>,
    ) -> Self {
        Self {
            pool,
            agent_id: agent_id.into(),
            blob_db_path: blob_db_path.into(),
            legacy_blob_db_path,
        }
    }

    fn legacy_blob_db_path(&self) -> Option<&Path> {
        self.legacy_blob_db_path.as_deref()
    }
}

impl<Backend> WorkerBlobStore<Backend>
where
    Backend: AgentBlobWorkerBackend,
{
    pub async fn get_blob<Ctx>(
        &self,
        _ctx: &Ctx,
        blob_id: &[u8],
    ) -> Result<Option<Vec<u8>>, Backend::Error> {
        self.pool
            .get_blob(
                &self.agent_id,
                &self.blob_db_path,
                blob_id,
                self.legacy_blob_db_path(),
            )
            .await
    }

    pub async fn set_blob<Ctx>(
        &self,
        _ctx: &Ctx,
        blob_id: &[u8],
        blob_data: &[u8],
    ) -> Result<(), Backend::Error> {
        self.pool
            .set_blob(
                &self.agent_id,
                &self.blob_db_path,
                blob_id,
                blob_data,
                self.legacy_blob_db_path(),
            )
            .await
    }

    pub async fn set_blob_locally_only<Ctx>(
        &self,
        ctx: &Ctx,
        blob_id: &[u8],
        blob_data: &[u8],
    ) -> Result<(), Backend::Error> {
        self.set_blob(ctx, blob_id, blob_data).await
    }

    pub async fn flush<Ctx>(&self, _ctx: &Ctx) {}
}
