use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::memory_service::{MemoryService, MemorySynthesisBridge};
use super::memory_synthesis_service::MemorySynthesisService;

pub const MEMORY_EXTENSION_ID: &str = "memory";
pub const MEMORY_EXTENSION_DEPENDENCIES: &[&str] = &["experiments", "inference", "telemetry"];

#[derive(Clone)]
pub struct HostMemoryExtension {
    service: Arc<MemoryService>,
    agents_root_dir: PathBuf,
    synthesis: Arc<Mutex<Option<Arc<MemorySynthesisService>>>>,
}

impl HostMemoryExtension {
    pub fn service(&self) -> Arc<MemoryService> {
        Arc::clone(&self.service)
    }

    pub fn agents_root_dir(&self) -> &Path {
        &self.agents_root_dir
    }

    pub fn enable_memory_synthesis(&self, synthesis: Arc<MemorySynthesisService>) {
        synthesis.start();
        let bridge: Arc<dyn MemorySynthesisBridge> = synthesis.clone();
        self.service.set_synthesis_bridge(Arc::downgrade(&bridge));
        let previous = self
            .synthesis
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .replace(synthesis);
        if let Some(previous) = previous {
            previous.dispose();
        }
    }

    pub fn disable_memory_synthesis(&self) {
        self.service.clear_synthesis_bridge();
        if let Some(synthesis) = self
            .synthesis
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
        {
            synthesis.dispose();
        }
    }

    pub fn memory_synthesis(&self) -> Option<Arc<MemorySynthesisService>> {
        self.synthesis
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
}

pub fn start_memory_extension(
    agents_root_dir: impl Into<PathBuf>,
) -> HostMemoryExtension {
    let agents_root_dir = agents_root_dir.into();
    HostMemoryExtension {
        service: Arc::new(MemoryService::new(agents_root_dir.clone())),
        agents_root_dir,
        synthesis: Arc::new(Mutex::new(None)),
    }
}
