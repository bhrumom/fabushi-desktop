use mahayana_host_runtime::extensions::cloud_workers::extension::{
    CLOUD_WORKERS_DEPENDENCIES, CLOUD_WORKERS_EXTENSION_ID,
    start_cloud_worker_provisioning_extension,
};
use mahayana_host_runtime::extensions::cloud_workers::CloudWorkerOwnerStatus;
use mahayana_host_runtime::extensions::extension_ids_generated::HostExtensionId;
use mahayana_host_runtime::host_gateway_api::{HostGatewayOwner, host_gateway_owner};

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
    let state_path = std::env::temp_dir().join("cloud-worker-contract-state.json");
    let extension = start_cloud_worker_provisioning_extension(state_path).unwrap();
    assert_eq!(extension.status(), CloudWorkerOwnerStatus::Disabled);
}

#[test]
fn coordinator_gateway_registry_routes_only_the_read_projection() {
    assert_eq!(
        host_gateway_owner("cloudWorkers.status"),
        Some(HostGatewayOwner::CloudWorkers)
    );
    assert_eq!(host_gateway_owner("cloudWorkers.admit"), None);
}
