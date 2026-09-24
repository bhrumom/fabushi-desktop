use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::memory_service::MemoryService;

pub const MEMORY_EXTENSION_ID: &str = "memory";
pub const MEMORY_EXTENSION_DEPENDENCIES: &[&str] = &["experiments", "inference", "telemetry"];

#[derive(Debug, Clone)]
pub struct HostMemoryExtension {
    service: Arc<MemoryService>,
    agents_root_dir: PathBuf,
}

impl HostMemoryExtension {
    pub fn service(&self) -> Arc<MemoryService> {
        Arc::clone(&self.service)
    }

    pub fn agents_root_dir(&self) -> &Path {
        &self.agents_root_dir
    }
}

pub fn start_memory_extension(
    agents_root_dir: impl Into<PathBuf>,
) -> HostMemoryExtension {
    let agents_root_dir = agents_root_dir.into();
    HostMemoryExtension {
        service: Arc::new(MemoryService::new(agents_root_dir.clone())),
        agents_root_dir,
    }
}
