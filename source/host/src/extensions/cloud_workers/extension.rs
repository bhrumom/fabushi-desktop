use std::sync::{Arc, Mutex};

use crate::extensions::extension_ids_generated::HostExtensionId;

use super::{CloudWorkerOwnerStatus, CloudWorkerProvisioningConfig, CloudWorkerProvisioningOwner};

pub const CLOUD_WORKERS_EXTENSION_ID: HostExtensionId = HostExtensionId::CloudWorkers;
pub const CLOUD_WORKERS_DEPENDENCIES: &[HostExtensionId] = &[
    HostExtensionId::Auth,
    HostExtensionId::Secrets,
];

#[derive(Clone)]
pub struct CloudWorkerProvisioningExtension {
    owner: Arc<Mutex<CloudWorkerProvisioningOwner>>,
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
}

pub fn start_cloud_worker_provisioning_extension() -> CloudWorkerProvisioningExtension {
    let owner = CloudWorkerProvisioningOwner::new(CloudWorkerProvisioningConfig::default())
        .expect("default cloud worker provisioning configuration is valid");
    CloudWorkerProvisioningExtension {
        owner: Arc::new(Mutex::new(owner)),
    }
}
