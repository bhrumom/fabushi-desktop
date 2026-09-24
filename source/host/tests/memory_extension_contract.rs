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
