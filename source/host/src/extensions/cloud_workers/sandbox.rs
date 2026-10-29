use std::path::Component;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{CloudWorkerLeaseScope, LeaseId, ReplayKey};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudWorkerSandboxCapability {
    pub lease_id: LeaseId,
    pub scope: CloudWorkerLeaseScope,
    pub workspace_id: String,
    pub remote_shell: bool,
    pub filesystem: bool,
}

impl CloudWorkerSandboxCapability {
    pub fn new(
        lease_id: LeaseId,
        scope: CloudWorkerLeaseScope,
        workspace_id: impl Into<String>,
    ) -> Result<Self, CloudWorkerSandboxError> {
        let workspace_id = workspace_id.into();
        if workspace_id.trim().is_empty() {
            return Err(CloudWorkerSandboxError::InvalidWorkspace);
        }
        Ok(Self {
            lease_id,
            scope,
            workspace_id,
            remote_shell: true,
            filesystem: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudWorkerSandboxOperation {
    ReadFile { path: String },
    WriteFile { path: String },
    ExecuteArgv { argv: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudWorkerSandboxRequest {
    pub lease_id: LeaseId,
    pub workspace_id: String,
    pub operation: CloudWorkerSandboxOperation,
    pub replay_key: ReplayKey,
}

impl CloudWorkerSandboxRequest {
    pub fn validate(&self) -> Result<(), CloudWorkerSandboxError> {
        if self.workspace_id.trim().is_empty() || self.replay_key.0.trim().is_empty() {
            return Err(CloudWorkerSandboxError::InvalidRequest);
        }
        match &self.operation {
            CloudWorkerSandboxOperation::ReadFile { path }
            | CloudWorkerSandboxOperation::WriteFile { path } => validate_workspace_path(path),
            CloudWorkerSandboxOperation::ExecuteArgv { argv } => {
                if argv.is_empty() || argv.iter().any(|arg| arg.contains('\0')) {
                    return Err(CloudWorkerSandboxError::InvalidArgv);
                }
                Ok(())
            }
        }
    }
}

fn validate_workspace_path(path: &str) -> Result<(), CloudWorkerSandboxError> {
    let path = std::path::Path::new(path);
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(component, Component::ParentDir | Component::Prefix(_))
        })
    {
        return Err(CloudWorkerSandboxError::UnsafePath);
    }
    Ok(())
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CloudWorkerSandboxError {
    #[error("sandbox workspace must be non-empty")]
    InvalidWorkspace,
    #[error("sandbox request must include workspace and replay identities")]
    InvalidRequest,
    #[error("sandbox path escapes the workspace")]
    UnsafePath,
    #[error("sandbox argv must be non-empty and contain no NUL bytes")]
    InvalidArgv,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(operation: CloudWorkerSandboxOperation) -> CloudWorkerSandboxRequest {
        CloudWorkerSandboxRequest {
            lease_id: LeaseId("lease-1".into()),
            workspace_id: "workspace-1".into(),
            operation,
            replay_key: ReplayKey("request-1".into()),
        }
    }

    #[test]
    fn accepts_argv_without_shell_string_interpolation() {
        assert_eq!(
            request(CloudWorkerSandboxOperation::ExecuteArgv {
                argv: vec!["git".into(), "status".into()],
            })
            .validate(),
            Ok(())
        );
    }

    #[test]
    fn rejects_workspace_escape_paths() {
        assert_eq!(
            request(CloudWorkerSandboxOperation::ReadFile {
                path: "../secret".into(),
            })
            .validate(),
            Err(CloudWorkerSandboxError::UnsafePath)
        );
    }
}
