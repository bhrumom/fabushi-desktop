use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::extensions::extension_ids_generated::HostExtensionId;

use super::{
    CloudWorkerOwnerError, CloudWorkerOwnerStatus, CloudWorkerProvisioningConfig,
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

impl CloudWorkerProvisioningExtension {
    pub fn owner(&self) -> Arc<Mutex<CloudWorkerProvisioningOwner>> {
        Arc::clone(&self.owner)
    }

    pub fn status(&self) -> CloudWorkerOwnerStatus {
        self.owner
            .lock()
            .expect("cloud worker provisioning owner lock")
            .status()
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
}

pub fn start_cloud_worker_provisioning_extension(
    state_path: impl AsRef<Path>,
) -> Result<CloudWorkerProvisioningExtension, CloudWorkerExtensionError> {
    let state_path = state_path.as_ref().to_path_buf();
    let config = CloudWorkerProvisioningConfig::default();
    let store = CloudWorkerProvisioningStore;
    let owner = if state_path.exists() {
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
    #[error(transparent)]
    Store(#[from] CloudWorkerStoreError),
    #[error(transparent)]
    Owner(#[from] CloudWorkerOwnerError),
}
