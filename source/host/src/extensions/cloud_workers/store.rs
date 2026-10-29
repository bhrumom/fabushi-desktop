use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use super::{CloudWorkerPersistentSnapshot, CloudWorkerProvisioningOwner};

#[derive(Debug, Default, Clone, Copy)]
pub struct CloudWorkerProvisioningStore;

impl CloudWorkerProvisioningStore {
    pub fn backup_existing(
        &self,
        source: &Path,
        backup: &Path,
    ) -> Result<(), CloudWorkerStoreError> {
        if source == backup {
            return Err(CloudWorkerStoreError::InvalidBackupPath);
        }
        if !source.exists() {
            return Err(CloudWorkerStoreError::BackupSourceMissing);
        }
        if backup.exists() {
            return Err(CloudWorkerStoreError::BackupAlreadyExists);
        }
        let parent = backup.parent().ok_or(CloudWorkerStoreError::NoParent)?;
        fs::create_dir_all(parent).map_err(CloudWorkerStoreError::Write)?;
        fs::copy(source, backup).map_err(CloudWorkerStoreError::Write)?;
        #[cfg(unix)]
        fs::set_permissions(
            backup,
            std::os::unix::fs::PermissionsExt::from_mode(0o600),
        )
        .map_err(CloudWorkerStoreError::Write)?;
        Ok(())
    }

    pub fn load(
        &self,
        path: &Path,
    ) -> Result<CloudWorkerPersistentSnapshot, CloudWorkerStoreError> {
        let mut file = File::open(path).map_err(CloudWorkerStoreError::Read)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).map_err(CloudWorkerStoreError::Read)?;
        serde_json::from_slice(&bytes).map_err(CloudWorkerStoreError::Decode)
    }

    pub fn save_atomic(
        &self,
        path: &Path,
        owner: &CloudWorkerProvisioningOwner,
    ) -> Result<(), CloudWorkerStoreError> {
        let parent = path.parent().ok_or(CloudWorkerStoreError::NoParent)?;
        fs::create_dir_all(parent).map_err(CloudWorkerStoreError::Write)?;
        let temporary = temporary_path(path);
        let bytes = serde_json::to_vec_pretty(&owner.snapshot())
            .map_err(CloudWorkerStoreError::Encode)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(CloudWorkerStoreError::Write)?;
        #[cfg(unix)]
        if let Err(error) = fs::set_permissions(
            &temporary,
            std::os::unix::fs::PermissionsExt::from_mode(0o600),
        ) {
            let _ = fs::remove_file(&temporary);
            return Err(CloudWorkerStoreError::Write(error));
        }
        if let Err(error) = write_and_sync(&mut file, &bytes) {
            let _ = fs::remove_file(&temporary);
            return Err(CloudWorkerStoreError::Write(error));
        }
        drop(file);
        if let Err(error) = fs::rename(&temporary, path) {
            let _ = fs::remove_file(&temporary);
            return Err(CloudWorkerStoreError::Write(error));
        }
        Ok(())
    }
}

fn write_and_sync(file: &mut File, bytes: &[u8]) -> io::Result<()> {
    file.write_all(bytes)?;
    file.sync_all()
}

fn temporary_path(path: &Path) -> PathBuf {
    let mut temporary = path.as_os_str().to_os_string();
    temporary.push(".tmp");
    PathBuf::from(temporary)
}

#[derive(Debug, thiserror::Error)]
pub enum CloudWorkerStoreError {
    #[error("cloud-worker state has no parent directory")]
    NoParent,
    #[error("could not read cloud-worker state: {0}")]
    Read(#[source] io::Error),
    #[error("could not write cloud-worker state atomically: {0}")]
    Write(#[source] io::Error),
    #[error("could not encode cloud-worker state: {0}")]
    Encode(#[source] serde_json::Error),
    #[error("could not decode cloud-worker state: {0}")]
    Decode(#[source] serde_json::Error),
    #[error("cloud-worker owner lock is poisoned")]
    PoisonedOwner,
    #[error("backup source does not exist")]
    BackupSourceMissing,
    #[error("backup destination already exists")]
    BackupAlreadyExists,
    #[error("backup source and destination must differ")]
    InvalidBackupPath,
}
