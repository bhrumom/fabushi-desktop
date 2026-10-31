use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::Serialize;

use crate::extensions::extension_ids_generated::HostExtensionId;

use super::{
    CloudWorkerDoctorAction, CloudWorkerDoctorError, CloudWorkerDoctorRequest,
    CloudWorkerDoctorResult,
    CloudWorkerBootstrapRequest,
    CloudWorkerLeasedCapability, CloudWorkerOwnerError, CloudWorkerOwnerStatus,
    CloudWorkerSandboxCapability, CloudWorkerSandboxError,
    CloudWorkerSandboxRequest,
    CloudWorkerProvisioningConfig,
    CloudWorkerProvisioningOwner, CloudWorkerProvisioningStore, CloudWorkerStoreError,
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
        Ok(CloudWorkerDoctorResult::inspection())
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
    use super::{CloudWorkerOwnerStatus, start_cloud_worker_provisioning_extension};
    use crate::extensions::cloud_workers::{
        CloudWorkerDoctorAction, CloudWorkerDoctorError, CloudWorkerDoctorRequest,
        CloudWorkerDoctorStatus, CloudWorkerLeaseScope, ReplayKey,
    };

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
}
