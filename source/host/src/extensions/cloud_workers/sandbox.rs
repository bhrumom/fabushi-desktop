use std::path::Component;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{CloudWorkerLeaseScope, LeaseId, ReplayKey};

const MAX_ID_CHARS: usize = 256;
const MAX_PATH_CHARS: usize = 4096;
const MAX_ARGV: usize = 128;
const MAX_ARG_CHARS: usize = 4096;

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
        validate_id(&lease_id.0)?;
        CloudWorkerLeaseScope::new(
            scope.account_scope.clone(),
            scope.device_scope.clone(),
        )
        .map_err(|_| CloudWorkerSandboxError::InvalidScope)?;
        if !valid_text_id(&workspace_id) {
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

    pub fn validate(&self) -> Result<(), CloudWorkerSandboxError> {
        validate_id(&self.lease_id.0)?;
        CloudWorkerLeaseScope::new(
            self.scope.account_scope.clone(),
            self.scope.device_scope.clone(),
        )
        .map_err(|_| CloudWorkerSandboxError::InvalidScope)?;
        if !valid_text_id(&self.workspace_id) {
            return Err(CloudWorkerSandboxError::InvalidWorkspace);
        }
        Ok(())
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
        validate_id(&self.lease_id.0)?;
        if !valid_text_id(&self.workspace_id) || !valid_text_id(&self.replay_key.0) {
            return Err(CloudWorkerSandboxError::InvalidRequest);
        }
        match &self.operation {
            CloudWorkerSandboxOperation::ReadFile { path }
            | CloudWorkerSandboxOperation::WriteFile { path } => validate_workspace_path(path),
            CloudWorkerSandboxOperation::ExecuteArgv { argv } => {
                if argv.is_empty()
                    || argv.len() > MAX_ARGV
                    || argv.iter().any(|arg| {
                        arg.is_empty()
                            || arg.chars().count() > MAX_ARG_CHARS
                            || arg.chars().any(|character| character.is_control())
                    })
                {
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
        || path.to_string_lossy().chars().count() > MAX_PATH_CHARS
        || path.to_string_lossy().chars().any(|character| character.is_control())
        || path.to_string_lossy().contains('\\')
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(component, Component::ParentDir | Component::Prefix(_))
        })
    {
        return Err(CloudWorkerSandboxError::UnsafePath);
    }
    Ok(())
}

fn valid_text_id(value: &str) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= MAX_ID_CHARS
        && !value.chars().any(|character| character.is_control())
}

fn validate_id(value: &str) -> Result<(), CloudWorkerSandboxError> {
    if valid_text_id(value) {
        Ok(())
    } else {
        Err(CloudWorkerSandboxError::InvalidLease)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CloudWorkerSandboxError {
    #[error("sandbox workspace must be non-empty")]
    InvalidWorkspace,
    #[error("sandbox lease id is invalid")]
    InvalidLease,
    #[error("sandbox lease scope is invalid")]
    InvalidScope,
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

    #[test]
    fn rejects_cross_platform_path_separators_and_control_text() {
        assert_eq!(
            request(CloudWorkerSandboxOperation::ReadFile {
                path: "dir\\secret".into(),
            })
            .validate(),
            Err(CloudWorkerSandboxError::UnsafePath)
        );
        assert_eq!(
            request(CloudWorkerSandboxOperation::ExecuteArgv {
                argv: vec!["git\nstatus".into()],
            })
            .validate(),
            Err(CloudWorkerSandboxError::InvalidArgv)
        );
    }

    #[test]
    fn revalidates_deserialized_capability_identity() {
        let capability = CloudWorkerSandboxCapability {
            lease_id: LeaseId("lease-1".into()),
            scope: CloudWorkerLeaseScope {
                account_scope: "account\nunsafe".into(),
                device_scope: "device".into(),
            },
            workspace_id: "workspace-1".into(),
            remote_shell: true,
            filesystem: true,
        };
        assert_eq!(
            capability.validate(),
            Err(CloudWorkerSandboxError::InvalidScope)
        );
    }
}
