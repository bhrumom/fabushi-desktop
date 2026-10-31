use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::Serialize;

use crate::extensions::extension_ids_generated::HostExtensionId;

use super::{
    CloudWorkerArtifactAdmission, CloudWorkerArtifactAdmissionState,
    CloudWorkerArtifactAdmissionError, CloudWorkerArtifactDigest,
    CloudWorkerArtifactKind, CloudWorkerArtifactPolicy, CloudWorkerArtifactSource,
    CloudWorkerBinaryIdentity, CloudWorkerBootstrapRequest, CloudWorkerCatalog,
    CloudWorkerCatalogError, CloudWorkerDoctorAction, CloudWorkerDoctorError,
    CloudWorkerDoctorRequest, CloudWorkerDoctorResult,
    CloudWorkerBootstrapPhase, CloudWorkerBootstrapRecord, CloudWorkerBootstrapRetryClass,
    CloudWorkerDiagnostic,
    CloudWorkerLease, CloudWorkerLeaseMutation, CloudWorkerLeaseScope,
    CloudWorkerLeasedCapability, CloudWorkerLifecycle, CloudWorkerMutationOutcome,
    CloudWorkerOwnerError, CloudWorkerOwnerStatus, CloudWorkerRuntimeIdentity, LeaseId,
    CloudWorkerSandboxCapability, CloudWorkerSandboxError,
    CloudWorkerSandboxRequest,
    CloudWorkerProvisioningConfig,
    CloudWorkerProvisioningOwner, CloudWorkerProvisioningStore, CloudWorkerStoreError,
    CloudWorkerRuntimeControlAction,
    CloudWorkerWarmImage, CloudWorkerWarmImageGeneration, CloudWorkerWarmImageOperation,
    verify_artifact_bytes,
};

pub const CLOUD_WORKERS_EXTENSION_ID: HostExtensionId = HostExtensionId::CloudWorkers;
pub const CLOUD_WORKERS_DEPENDENCIES: &[HostExtensionId] = &[
    HostExtensionId::Auth,
    HostExtensionId::Secrets,
];

#[derive(Clone)]
pub struct CloudWorkerProvisioningExtension {
    owner: Arc<Mutex<CloudWorkerProvisioningOwner>>,
    state_path: PathBuf,
    store: CloudWorkerProvisioningStore,
}

/// Read-only projection produced by the Cloud Worker owner for Gateway and
/// Coordinator consumers. It intentionally exposes readiness and mutation as
/// false until a real provider producer and side-effect owner are connected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CloudWorkerStatusSnapshot {
    pub owner: &'static str,
    pub status: &'static str,
    #[serde(rename = "providerReady")]
    pub provider_ready: bool,
    #[serde(rename = "productionMutation")]
    pub production_mutation: bool,
    #[serde(rename = "statePersisted")]
    pub state_persisted: bool,
    #[serde(rename = "leaseCount")]
    pub lease_count: usize,
    #[serde(rename = "warmImageCount")]
    pub warm_image_count: usize,
    #[serde(rename = "bootstrapCount")]
    pub bootstrap_count: usize,
    #[serde(rename = "artifactCount")]
    pub artifact_count: usize,
    #[serde(rename = "migrationReceiptCount")]
    pub migration_receipt_count: usize,
}

impl CloudWorkerProvisioningExtension {
    pub fn status(&self) -> CloudWorkerOwnerStatus {
        self.owner
            .lock()
            .map(|owner| owner.status())
            .unwrap_or(CloudWorkerOwnerStatus::Unavailable)
    }

    /// Routes provider capability discovery through the canonical owner and
    /// its exact-binary-identity cache. The loader remains the explicit
    /// provider/process boundary and may not authorize or launch a worker.
    pub fn resolve_provider_catalog<F>(
        &self,
        identity: CloudWorkerBinaryIdentity,
        load: F,
    ) -> Result<CloudWorkerCatalog, CloudWorkerCatalogError>
    where
        F: FnOnce() -> Result<CloudWorkerCatalog, CloudWorkerCatalogError>,
    {
        let mut owner = self
            .owner
            .lock()
            .map_err(|_| CloudWorkerCatalogError::OwnerUnavailable)?;
        owner.resolve_catalog(identity, load)
    }

    /// Registers one policy-validated artifact admission in the canonical
    /// owner snapshot. Acquisition and publication remain external producers.
    pub fn register_artifact(
        &self,
        artifact: CloudWorkerArtifactAdmission,
        policy: &CloudWorkerArtifactPolicy,
    ) -> Result<(), CloudWorkerMutationError> {
        self.mutate_and_persist(|owner| owner.register_artifact(artifact, policy))
    }

    /// Verifies caller-supplied bytes and atomically records the resulting
    /// verified admission. Download, credential egress and publication remain
    /// owned by the future artifact producer.
    pub fn verify_and_register_artifact(
        &self,
        kind: CloudWorkerArtifactKind,
        policy: &CloudWorkerArtifactPolicy,
        source: &CloudWorkerArtifactSource,
        bytes: &[u8],
    ) -> Result<super::CloudWorkerVerifiedArtifact, CloudWorkerMutationError> {
        let verified = verify_artifact_bytes(policy, source, bytes)
            .map_err(CloudWorkerMutationError::Artifact)?;
        let admission = CloudWorkerArtifactAdmission {
            kind,
            source: source.clone(),
            size_bytes: verified.size_bytes,
            state: CloudWorkerArtifactAdmissionState::Verified,
        };
        self.mutate_and_persist(|owner| {
            owner.register_artifact(admission, policy)?;
            Ok(verified.clone())
        })
    }

    /// Settles one persisted artifact admission without retrying or hiding an
    /// outcome-unknown state.
    pub fn settle_artifact(
        &self,
        digest: &CloudWorkerArtifactDigest,
        next: CloudWorkerArtifactAdmissionState,
    ) -> Result<CloudWorkerArtifactAdmission, CloudWorkerMutationError> {
        self.mutate_and_persist(|owner| owner.settle_artifact(digest, next))
    }

    /// Registers warm-image state through the canonical owner and its atomic
    /// persistence adapter. No provider snapshot or image process is started.
    pub fn register_warm_image(
        &self,
        image: CloudWorkerWarmImage,
    ) -> Result<(), CloudWorkerMutationError> {
        self.mutate_and_persist(|owner| owner.register_warm_image(image))
    }

    /// Applies one generation-fenced warm-image transition and persists the
    /// resulting owner snapshot. Provider side effects remain a separate
    /// adapter and are intentionally not implied by this method.
    pub fn apply_warm_image(
        &self,
        image_id: &str,
        expected_generation: CloudWorkerWarmImageGeneration,
        operation: CloudWorkerWarmImageOperation,
    ) -> Result<CloudWorkerWarmImage, CloudWorkerMutationError> {
        self.mutate_and_persist(|owner| {
            owner.apply_warm_image(image_id, expected_generation, operation)
        })
    }

    /// Persists an ambiguous capture outcome as explicit recovery debt. The
    /// provider snapshot and cleanup consumer remain outside this façade.
    pub fn mark_warm_image_capture_uncertain(
        &self,
        image_id: &str,
        expected_generation: CloudWorkerWarmImageGeneration,
    ) -> Result<CloudWorkerWarmImage, CloudWorkerMutationError> {
        self.mutate_and_persist(|owner| {
            owner.mark_warm_image_capture_uncertain(image_id, expected_generation)
        })
    }

    /// Admits one lease through the canonical owner and persists the exact
    /// owner snapshot. Provider discovery and worker launch remain separate.
    pub fn admit_lease(
        &self,
        scope: CloudWorkerLeaseScope,
        lease_id: LeaseId,
        runtime: CloudWorkerRuntimeIdentity,
    ) -> Result<CloudWorkerLease, CloudWorkerMutationError> {
        self.mutate_and_persist(|owner| owner.admit(scope, lease_id, runtime))
    }

    /// Builds a lease-fenced runtime-control mutation through the canonical
    /// owner. It does not call a provider or change persisted state.
    pub fn admit_runtime_control(
        &self,
        lease_id: &LeaseId,
        action: CloudWorkerRuntimeControlAction,
        replay_key: super::ReplayKey,
    ) -> Result<super::CloudWorkerLeaseMutation, CloudWorkerMutationError> {
        let owner = self
            .owner
            .lock()
            .map_err(|_| CloudWorkerMutationError::PoisonedOwner)?;
        owner
            .admit_runtime_control(lease_id, action, replay_key)
            .map_err(CloudWorkerMutationError::Owner)
    }

    /// Settles one fenced lease mutation and persists its lifecycle outcome,
    /// including outcome-unknown states without retrying or hiding them.
    pub fn settle_lease(
        &self,
        mutation: &CloudWorkerLeaseMutation,
        next: CloudWorkerLifecycle,
        outcome: CloudWorkerMutationOutcome,
    ) -> Result<CloudWorkerLease, CloudWorkerMutationError> {
        self.mutate_and_persist(|owner| owner.settle_lease(mutation, next, outcome))
    }

    /// Settles a runtime-control mutation through its action-specific
    /// lifecycle contract and the canonical persistence boundary.
    pub fn settle_runtime_control(
        &self,
        mutation: &CloudWorkerLeaseMutation,
        action: CloudWorkerRuntimeControlAction,
        outcome: CloudWorkerMutationOutcome,
    ) -> Result<CloudWorkerLease, CloudWorkerMutationError> {
        self.mutate_and_persist(|owner| {
            owner.settle_runtime_control(mutation, action, outcome)
        })
    }

    /// Registers a validated bootstrap record in the canonical owner snapshot.
    /// Secret values remain opaque references; no enrollment or process action
    /// is performed by this persistence façade.
    pub fn register_bootstrap(
        &self,
        record: CloudWorkerBootstrapRecord,
    ) -> Result<(), CloudWorkerMutationError> {
        self.mutate_and_persist(|owner| owner.register_bootstrap(record))
    }

    /// Settles a lease-fenced bootstrap phase and persists its bounded,
    /// redacted diagnostic state.
    pub fn settle_bootstrap(
        &self,
        mutation: &CloudWorkerLeaseMutation,
        phase: CloudWorkerBootstrapPhase,
        retry_class: CloudWorkerBootstrapRetryClass,
        diagnostic: Option<CloudWorkerDiagnostic>,
    ) -> Result<CloudWorkerBootstrapRecord, CloudWorkerMutationError> {
        self.mutate_and_persist(|owner| {
            owner.settle_bootstrap(mutation, phase, retry_class, diagnostic)
        })
    }

    /// Provides the read-only doctor projection from the canonical owner.
    /// Mutating recovery actions remain explicitly unavailable until their
    /// provider, persistence and recovery owners are wired.
    pub fn inspect_doctor(
        &self,
        request: CloudWorkerDoctorRequest,
    ) -> Result<CloudWorkerDoctorResult, CloudWorkerDoctorError> {
        request.validate()?;
        if !matches!(request.action, CloudWorkerDoctorAction::Inspect) {
            return Err(CloudWorkerDoctorError::MutationNotWired);
        }
        let owner = self
            .owner
            .lock()
            .map_err(|_| CloudWorkerDoctorError::OwnerUnavailable)?;
        let snapshot = owner.snapshot();
        Ok(CloudWorkerDoctorResult::inspection_with_snapshot(
            self.state_path.exists(),
            snapshot.leases.len(),
            snapshot.bootstrap_records.len(),
            snapshot.artifacts.len(),
            snapshot.warm_images.len(),
        ))
    }

    /// Returns a read-only snapshot for the existing Host/Coordinator status
    /// consumer. The extension remains the only owner of the live state.
    pub fn snapshot(&self) -> Option<super::CloudWorkerPersistentSnapshot> {
        self.owner.lock().ok().map(|owner| owner.snapshot())
    }

    /// Re-reads the canonical owner record before an existing executor acts.
    /// The returned capability is still ephemeral; this method does not grant
    /// a device, start a provider, or create another lease store.
    pub fn validate_current_capability(
        &self,
        capability: &CloudWorkerLeasedCapability,
    ) -> Result<CloudWorkerLeasedCapability, CloudWorkerOwnerError> {
        let owner = self
            .owner
            .lock()
            .map_err(|_| CloudWorkerOwnerError::Unavailable)?;
        if owner.status() != super::CloudWorkerOwnerStatus::Enabled {
            return Err(CloudWorkerOwnerError::Disabled);
        }
        let current = owner.leased_capability(capability.lease_id())?;
        if current != *capability {
            return Err(CloudWorkerOwnerError::StaleCapability);
        }
        Ok(current)
    }

    /// Re-reads the canonical lease before a future sandbox consumer uses a
    /// serialized capability. The sandbox descriptor owns no lifecycle state;
    /// stale, disabled or outcome-unknown leases fail closed here.
    pub fn validate_current_sandbox_capability(
        &self,
        capability: &CloudWorkerSandboxCapability,
    ) -> Result<CloudWorkerSandboxCapability, CloudWorkerOwnerError> {
        let owner = self
            .owner
            .lock()
            .map_err(|_| CloudWorkerOwnerError::Unavailable)?;
        if owner.status() != super::CloudWorkerOwnerStatus::Enabled {
            return Err(CloudWorkerOwnerError::Disabled);
        }
        let current = owner.leased_capability(&capability.lease_id)?;
        capability
            .validate_against(&current)
            .map_err(|error| match error {
                CloudWorkerSandboxError::StaleCapability => CloudWorkerOwnerError::StaleCapability,
                _ => CloudWorkerOwnerError::StaleCapability,
            })?;
        Ok(capability.clone())
    }

    /// Revalidates both the canonical lease and the requested sandbox
    /// operation before delegation. Existing sandbox owners retain all actual
    /// filesystem/process side effects.
    pub fn validate_current_sandbox_request(
        &self,
        capability: &CloudWorkerSandboxCapability,
        request: &CloudWorkerSandboxRequest,
    ) -> Result<(), CloudWorkerOwnerError> {
        self.validate_current_sandbox_capability(capability)?;
        request
            .validate_against(capability)
            .map_err(|_| CloudWorkerOwnerError::StaleCapability)
    }

    /// Re-reads the canonical lease before a bootstrap producer may act. The
    /// request carries only fencing material and typed admission proofs; this
    /// method performs no enrollment, process launch or provider mutation.
    pub fn validate_current_bootstrap_request(
        &self,
        request: &CloudWorkerBootstrapRequest,
    ) -> Result<(), CloudWorkerOwnerError> {
        request
            .validate()
            .map_err(|_| CloudWorkerOwnerError::StaleCapability)?;
        let owner = self
            .owner
            .lock()
            .map_err(|_| CloudWorkerOwnerError::Unavailable)?;
        if owner.status() != super::CloudWorkerOwnerStatus::Enabled {
            return Err(CloudWorkerOwnerError::Disabled);
        }
        let current = owner.leased_capability(&request.lease_id)?;
        if current.scope() != &request.scope
            || current.owner_epoch() != request.owner_epoch
            || current.generation() != request.generation
            || current.runtime() != &request.runtime
        {
            return Err(CloudWorkerOwnerError::StaleCapability);
        }
        owner.verified_artifact(&request.artifact)?;
        Ok(())
    }

    pub fn status_snapshot(&self) -> CloudWorkerStatusSnapshot {
        let (status, snapshot) = match self.owner.lock() {
            Ok(owner) => {
                let status = match owner.status() {
                    CloudWorkerOwnerStatus::Disabled => "disabled",
                    CloudWorkerOwnerStatus::Enabled => "enabled",
                    CloudWorkerOwnerStatus::Unavailable => "unavailable",
                };
                (status, Some(owner.snapshot()))
            }
            Err(_) => ("unavailable", None),
        };
        let state_persisted = fs::symlink_metadata(&self.state_path)
            .map(|metadata| metadata.file_type().is_file())
            .unwrap_or(false);
        CloudWorkerStatusSnapshot {
            owner: "cloud-workers",
            status,
            provider_ready: false,
            production_mutation: false,
            state_persisted,
            lease_count: snapshot.as_ref().map_or(0, |value| value.leases.len()),
            warm_image_count: snapshot.as_ref().map_or(0, |value| value.warm_images.len()),
            bootstrap_count: snapshot
                .as_ref()
                .map_or(0, |value| value.bootstrap_records.len()),
            artifact_count: snapshot.as_ref().map_or(0, |value| value.artifacts.len()),
            migration_receipt_count: snapshot
                .as_ref()
                .map_or(0, |value| value.migration_receipts.len()),
        }
    }

    pub fn state_path(&self) -> &Path {
        &self.state_path
    }

    pub fn persist(&self) -> Result<(), CloudWorkerStoreError> {
        let owner = self
            .owner
            .lock()
            .map_err(|_| CloudWorkerStoreError::PoisonedOwner)?;
        self.store.save_atomic(&self.state_path, &owner)
    }

    pub fn mutate_and_persist<T, F>(&self, mutate: F) -> Result<T, CloudWorkerMutationError>
    where
        F: FnOnce(&mut CloudWorkerProvisioningOwner) -> Result<T, CloudWorkerOwnerError>,
    {
        let mut owner = self
            .owner
            .lock()
            .map_err(|_| CloudWorkerMutationError::PoisonedOwner)?;
        let before = owner.snapshot();
        let value = mutate(&mut owner).map_err(CloudWorkerMutationError::Owner)?;
        match self.store.save_atomic(&self.state_path, &owner) {
            Ok(()) => Ok(value),
            Err(CloudWorkerStoreError::DirectoryBarrierOutcomeUnknown(error)) => {
                Err(CloudWorkerMutationError::OutcomeUnknown(
                    CloudWorkerStoreError::DirectoryBarrierOutcomeUnknown(error),
                ))
            }
            Err(error) => {
                owner
                    .restore_snapshot(before)
                    .map_err(CloudWorkerMutationError::Rollback)?;
                Err(CloudWorkerMutationError::Persistence(error))
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CloudWorkerMutationError {
    #[error(transparent)]
    Owner(#[source] CloudWorkerOwnerError),
    #[error(transparent)]
    Artifact(#[source] CloudWorkerArtifactAdmissionError),
    #[error(transparent)]
    Persistence(#[source] CloudWorkerStoreError),
    #[error("cloud-worker mutation persistence outcome is unknown: {0}")]
    OutcomeUnknown(#[source] CloudWorkerStoreError),
    #[error("cloud-worker mutation rollback failed: {0}")]
    Rollback(#[source] CloudWorkerOwnerError),
    #[error("cloud-worker owner lock is poisoned")]
    PoisonedOwner,
}

pub fn start_cloud_worker_provisioning_extension(
    state_path: impl AsRef<Path>,
) -> Result<CloudWorkerProvisioningExtension, CloudWorkerExtensionError> {
    let state_path = state_path.as_ref().to_path_buf();
    let config = CloudWorkerProvisioningConfig::default();
    let store = CloudWorkerProvisioningStore;
    let owner = if state_path.exists() {
        let metadata = fs::symlink_metadata(&state_path)
            .map_err(CloudWorkerStoreError::Read)?;
        if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
            return Err(CloudWorkerExtensionError::InvalidStatePath);
        }
        let snapshot = store.load(&state_path)?;
        CloudWorkerProvisioningOwner::from_snapshot(config, snapshot)?
    } else {
        CloudWorkerProvisioningOwner::new(config)?
    };
    Ok(CloudWorkerProvisioningExtension {
        owner: Arc::new(Mutex::new(owner)),
        state_path,
        store,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum CloudWorkerExtensionError {
    #[error("cloud-worker state path must be a regular non-symlink file")]
    InvalidStatePath,
    #[error(transparent)]
    Store(#[from] CloudWorkerStoreError),
    #[error(transparent)]
    Owner(#[from] CloudWorkerOwnerError),
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    use super::{
        CloudWorkerMutationError, CloudWorkerOwnerStatus, CloudWorkerProvisioningExtension,
        CloudWorkerProvisioningStore, start_cloud_worker_provisioning_extension,
    };
    use crate::extensions::cloud_workers::{
        CloudWorkerArtifactAdmissionError, CloudWorkerArtifactKind, CloudWorkerArtifactPolicy,
        CloudWorkerArtifactSource, CloudWorkerDoctorAction, CloudWorkerDoctorError,
        CloudWorkerDoctorRequest, CloudWorkerDoctorStatus, CloudWorkerLeaseScope,
        CloudWorkerArtifactDigest, CloudWorkerProvisioningConfig,
        CloudWorkerProvisioningOwner, ReplayKey,
    };

    fn enabled_extension(state_path: PathBuf) -> CloudWorkerProvisioningExtension {
        let owner = CloudWorkerProvisioningOwner::new(CloudWorkerProvisioningConfig {
            enabled: true,
            max_leases: 1,
            max_warm_images: 1,
            max_artifacts: 1,
        })
        .unwrap();
        CloudWorkerProvisioningExtension {
            owner: Arc::new(Mutex::new(owner)),
            state_path,
            store: CloudWorkerProvisioningStore,
        }
    }

    fn artifact_policy() -> CloudWorkerArtifactPolicy {
        CloudWorkerArtifactPolicy {
            allowed_https_origins: Vec::new(),
            max_bytes: 1024,
        }
    }

    fn test_state_path(label: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "cloud-worker-{label}-{}-{nonce}.json",
            std::process::id()
        ))
    }

    #[test]
    fn poisoned_owner_is_projected_as_unavailable_without_public_owner_access() {
        let state_path = std::env::temp_dir().join("cloud-worker-poisoned-extension-state.json");
        let extension = start_cloud_worker_provisioning_extension(state_path).unwrap();
        let owner = std::sync::Arc::clone(&extension.owner);
        let join = std::thread::spawn(move || {
            let _guard = owner
                .lock()
                .expect("owner lock should be healthy before poisoning");
            panic!("poison cloud-worker owner for the contract");
        })
        .join();
        assert!(join.is_err());
        assert_eq!(extension.status(), CloudWorkerOwnerStatus::Unavailable);
    }

    #[test]
    fn doctor_inspection_is_owner_bound_but_mutation_stays_closed() {
        let state_path = std::env::temp_dir().join("cloud-worker-doctor-inspection-state.json");
        let extension = start_cloud_worker_provisioning_extension(state_path).unwrap();
        let request = CloudWorkerDoctorRequest {
            scope: CloudWorkerLeaseScope::new("account", "device").unwrap(),
            action: CloudWorkerDoctorAction::Inspect,
            acknowledged_mutation: false,
            replay_key: ReplayKey("doctor-1".into()),
        };
        assert_eq!(
            extension.inspect_doctor(request).unwrap().status,
            CloudWorkerDoctorStatus::InspectionOnly
        );

        let request = CloudWorkerDoctorRequest {
            scope: CloudWorkerLeaseScope::new("account", "device").unwrap(),
            action: CloudWorkerDoctorAction::ReconcileLease,
            acknowledged_mutation: true,
            replay_key: ReplayKey("doctor-2".into()),
        };
        assert_eq!(
            extension.inspect_doctor(request),
            Err(CloudWorkerDoctorError::MutationNotWired)
        );
    }

    #[test]
    fn verified_artifact_bytes_are_persisted_as_one_owner_mutation() {
        let state_path = test_state_path("verified-artifact");
        let extension = enabled_extension(state_path.clone());
        let source = CloudWorkerArtifactSource::ManagedLocal {
            digest: CloudWorkerArtifactDigest::new(
                "87eba76e7f3164534045ba922e7770fb58bbd14ad732bbf5ba6f11cc56989e6e",
            )
            .unwrap(),
        };

        let verified = extension
            .verify_and_register_artifact(
                CloudWorkerArtifactKind::WorkerBinary,
                &artifact_policy(),
                &source,
                b"worker",
            )
            .unwrap();

        assert_eq!(verified.size_bytes, 6);
        assert_eq!(extension.snapshot().unwrap().artifacts.len(), 1);
        assert!(state_path.is_file());
    }

    #[test]
    fn digest_mismatch_does_not_persist_an_artifact_admission() {
        let state_path = test_state_path("rejected-artifact");
        let extension = enabled_extension(state_path.clone());
        let source = CloudWorkerArtifactSource::ManagedLocal {
            digest: CloudWorkerArtifactDigest::new("a".repeat(64)).unwrap(),
        };

        let result = extension.verify_and_register_artifact(
            CloudWorkerArtifactKind::WorkerBinary,
            &artifact_policy(),
            &source,
            b"worker",
        );

        assert!(matches!(
            result,
            Err(CloudWorkerMutationError::Artifact(
                CloudWorkerArtifactAdmissionError::DigestMismatch { .. }
            ))
        ));
        assert!(extension.snapshot().unwrap().artifacts.is_empty());
        assert!(!state_path.exists());
    }
}
