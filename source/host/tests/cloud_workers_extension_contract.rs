use mahayana_host_runtime::extensions::cloud_workers::extension::{
    CLOUD_WORKERS_DEPENDENCIES, CLOUD_WORKERS_EXTENSION_ID,
    start_cloud_worker_provisioning_extension,
};
use mahayana_host_runtime::extensions::cloud_workers::CloudWorkerOwnerStatus;
use mahayana_host_runtime::extensions::extension_ids_generated::HostExtensionId;

#[test]
fn cloud_workers_has_a_formal_slot_and_explicit_dependencies() {
    assert_eq!(CLOUD_WORKERS_EXTENSION_ID, HostExtensionId::CloudWorkers);
    assert_eq!(
        CLOUD_WORKERS_DEPENDENCIES,
        &[HostExtensionId::Auth, HostExtensionId::Secrets]
    );
}

#[test]
fn production_boundary_starts_disabled_without_provider_prerequisites() {
    let extension = start_cloud_worker_provisioning_extension();
    assert_eq!(extension.status(), CloudWorkerOwnerStatus::Disabled);
}
