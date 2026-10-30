use mahayana_host_runtime::extensions::cloud_workers::extension::{
    CLOUD_WORKERS_DEPENDENCIES, CLOUD_WORKERS_EXTENSION_ID,
    start_cloud_worker_provisioning_extension,
};
use mahayana_host_runtime::extensions::cloud_workers::{
    CloudWorkerLeaseScope, CloudWorkerLifecycle, CloudWorkerOwnerStatus,
    CloudWorkerProvisioningConfig, CloudWorkerProvisioningOwner,
    CloudWorkerRuntimeIdentity, CloudWorkerMutationOutcome, LeaseGeneration, LeaseId,
    OwnerEpoch, ReplayKey,
};
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
fn owner_status_snapshot_is_the_read_projection_and_stays_fail_closed() {
    let state_path = std::env::temp_dir().join(format!(
        "cloud-worker-status-projection-state-{}.json",
        std::process::id()
    ));
    let extension = start_cloud_worker_provisioning_extension(state_path).unwrap();
    let projection = extension.status_snapshot();
    assert_eq!(projection.owner, "cloud-workers");
    assert_eq!(projection.status, "disabled");
    assert!(!projection.provider_ready);
    assert!(!projection.production_mutation);
    assert!(!projection.state_persisted);
    assert_eq!(projection.lease_count, 0);
    assert_eq!(projection.warm_image_count, 0);
    let value = serde_json::to_value(projection).unwrap();
    assert_eq!(value["providerReady"], false);
    assert_eq!(value["productionMutation"], false);

    extension.persist().unwrap();
    assert!(extension.status_snapshot().state_persisted);
}

#[test]
fn coordinator_gateway_registry_routes_only_the_read_projection() {
    assert_eq!(
        host_gateway_owner("cloudWorkers.status"),
        Some(HostGatewayOwner::CloudWorkers)
    );
    assert_eq!(host_gateway_owner("cloudWorkers.admit"), None);
}

#[test]
fn owner_exposes_only_a_fenced_enrolled_consumer_capability() {
    let mut owner = CloudWorkerProvisioningOwner::new(CloudWorkerProvisioningConfig {
        enabled: true,
        max_leases: 1,
        max_warm_images: 1,
    })
    .unwrap();
    let lease_id = LeaseId("lease-contract".into());
    let runtime = CloudWorkerRuntimeIdentity::new("provider", "profile", "runtime").unwrap();
    owner
        .admit(
            CloudWorkerLeaseScope::new("account", "device").unwrap(),
            lease_id.clone(),
            runtime.clone(),
        )
        .unwrap();

    for (next, replay) in [
        (CloudWorkerLifecycle::Preparing, "prepare"),
        (CloudWorkerLifecycle::Warming, "warm"),
        (CloudWorkerLifecycle::Bootstrapping, "bootstrap"),
        (CloudWorkerLifecycle::Enrolled, "enroll"),
    ] {
        let current = owner.lease(&lease_id).unwrap().clone();
        let mutation = current.mutation(
            OwnerEpoch(0),
            current.generation,
            runtime.clone(),
            ReplayKey(replay.into()),
        );
        owner
            .settle_lease(&mutation, next, CloudWorkerMutationOutcome::Completed)
            .unwrap();
    }

    let capability = owner.leased_capability(&lease_id).unwrap();
    assert_eq!(capability.lease_id(), &lease_id);
    assert_eq!(capability.scope().account_scope, "account");
    assert_eq!(capability.runtime().runtime_id, "runtime");
}
