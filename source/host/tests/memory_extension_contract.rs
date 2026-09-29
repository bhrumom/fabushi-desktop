use std::path::PathBuf;
use std::sync::Arc;

use mahayana_host_runtime::extensions::memory::extension::{
    MEMORY_EXTENSION_DEPENDENCIES, MEMORY_EXTENSION_ID, start_memory_extension,
};

#[test]
fn memory_extension_owns_one_agent_scoped_service() {
    let root = PathBuf::from("/tmp/fabushi-memory-extension-contract");
    let extension = start_memory_extension(root.clone());
    assert_eq!(MEMORY_EXTENSION_ID, "memory");
    assert_eq!(
        MEMORY_EXTENSION_DEPENDENCIES,
        &["experiments", "inference", "telemetry"]
    );
    assert_eq!(extension.agents_root_dir(), root.as_path());

    let first = extension.service();
    let second = extension.service();
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(first.agents_root_dir(), root.as_path());
}


fn empty_synthesis() -> Arc<mahayana_host_runtime::extensions::memory::memory_synthesis_service::MemorySynthesisService> {
    use mahayana_host_runtime::extensions::memory::memory_synthesis_service::{
        MemorySynthesisOptions, MemorySynthesisService,
    };
    Arc::new(MemorySynthesisService::new(MemorySynthesisOptions::new(
        Arc::new(|| Vec::new()),
        Arc::new(|_| None),
        Arc::new(|_, _| Ok(serde_json::json!({"changes":[]}))),
    )))
}

#[test]
fn memory_extension_replacement_and_disable_keep_one_background_owner() {
    let root = PathBuf::from("/tmp/fabushi-memory-extension-owner-contract");
    let extension = start_memory_extension(root);

    let first = empty_synthesis();
    extension.enable_memory_synthesis(Arc::clone(&first));
    assert!(first.is_enabled());

    let second = empty_synthesis();
    extension.enable_memory_synthesis(Arc::clone(&second));
    assert!(!first.is_enabled());
    assert!(second.is_enabled());

    extension.disable_memory_synthesis();
    assert!(!second.is_enabled());
}
