use std::future::Future;
use std::path::Path;
use std::pin::Pin;

pub type AgentWorkerFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait AgentBlobWorkerBackend: Send + Sync {
    type Error: Send + Sync + 'static;

    fn get_blob<'a>(
        &'a self,
        agent_id: &'a str,
        blob_db_path: &'a Path,
        blob_id: &'a [u8],
        legacy_blob_db_path: Option<&'a Path>,
    ) -> AgentWorkerFuture<'a, Result<Option<Vec<u8>>, Self::Error>>;

    fn set_blob<'a>(
        &'a self,
        agent_id: &'a str,
        blob_db_path: &'a Path,
        blob_id: &'a [u8],
        blob_data: &'a [u8],
        legacy_blob_db_path: Option<&'a Path>,
    ) -> AgentWorkerFuture<'a, Result<(), Self::Error>>;
}

pub struct AgentWorkerPool<Backend> {
    backend: Backend,
}

impl<Backend> AgentWorkerPool<Backend> {
    pub fn new(backend: Backend) -> Self {
        Self { backend }
    }

    pub fn backend(&self) -> &Backend {
        &self.backend
    }
}

impl<Backend> AgentWorkerPool<Backend>
where
    Backend: AgentBlobWorkerBackend,
{
    pub async fn get_blob(
        &self,
        agent_id: &str,
        blob_db_path: &Path,
        blob_id: &[u8],
        legacy_blob_db_path: Option<&Path>,
    ) -> Result<Option<Vec<u8>>, Backend::Error> {
        self.backend
            .get_blob(agent_id, blob_db_path, blob_id, legacy_blob_db_path)
            .await
    }

    pub async fn set_blob(
        &self,
        agent_id: &str,
        blob_db_path: &Path,
        blob_id: &[u8],
        blob_data: &[u8],
        legacy_blob_db_path: Option<&Path>,
    ) -> Result<(), Backend::Error> {
        self.backend
            .set_blob(
                agent_id,
                blob_db_path,
                blob_id,
                blob_data,
                legacy_blob_db_path,
            )
            .await
    }
}
