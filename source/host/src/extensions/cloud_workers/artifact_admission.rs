use std::path::Component;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudWorkerArtifactKind {
    WorkerBinary,
    RuntimeBundle,
    PluginBundle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudWorkerArtifactDigest(pub String);

impl CloudWorkerArtifactDigest {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudWorkerArtifactAdmissionError> {
        let value = value.into();
        if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(CloudWorkerArtifactAdmissionError::InvalidDigest);
        }
        Ok(Self(value.to_ascii_lowercase()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudWorkerArtifactSource {
    Https {
        url: String,
        digest: CloudWorkerArtifactDigest,
    },
    ManagedLocal {
        digest: CloudWorkerArtifactDigest,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudWorkerArtifactPolicy {
    pub allowed_https_origins: Vec<String>,
    pub max_bytes: u64,
}

impl CloudWorkerArtifactPolicy {
    pub fn validate(&self) -> Result<(), CloudWorkerArtifactAdmissionError> {
        if self.max_bytes == 0 {
            return Err(CloudWorkerArtifactAdmissionError::InvalidSizeLimit);
        }
        for origin in &self.allowed_https_origins {
            let url = Url::parse(origin)
                .map_err(|_| CloudWorkerArtifactAdmissionError::InvalidOrigin)?;
            if url.scheme() != "https"
                || url.host_str().is_none()
                || url.username() != ""
                || url.password().is_some()
                || url.path() != "/"
                || url.query().is_some()
                || url.fragment().is_some()
            {
                return Err(CloudWorkerArtifactAdmissionError::InvalidOrigin);
            }
        }
        Ok(())
    }

    pub fn validate_source(
        &self,
        source: &CloudWorkerArtifactSource,
        size_bytes: u64,
    ) -> Result<(), CloudWorkerArtifactAdmissionError> {
        self.validate()?;
        if size_bytes > self.max_bytes {
            return Err(CloudWorkerArtifactAdmissionError::ArtifactTooLarge {
                bytes: size_bytes,
                max_bytes: self.max_bytes,
            });
        }
        if let CloudWorkerArtifactSource::Https { url, .. } = source {
            let parsed = Url::parse(url)
                .map_err(|_| CloudWorkerArtifactAdmissionError::InvalidOrigin)?;
            if parsed.scheme() != "https" || parsed.username() != "" || parsed.password().is_some()
            {
                return Err(CloudWorkerArtifactAdmissionError::InvalidOrigin);
            }
            let origin = parsed.origin().ascii_serialization();
            if !self.allowed_https_origins.iter().any(|allowed| allowed == &origin) {
                return Err(CloudWorkerArtifactAdmissionError::OriginNotAllowed(origin));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudWorkerArtifactAdmissionState {
    Pending,
    Verified,
    Damaged,
    OutcomeUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudWorkerArtifactAdmission {
    pub kind: CloudWorkerArtifactKind,
    pub source: CloudWorkerArtifactSource,
    pub size_bytes: u64,
    pub state: CloudWorkerArtifactAdmissionState,
}

impl CloudWorkerArtifactAdmission {
    pub fn validate(
        &self,
        policy: &CloudWorkerArtifactPolicy,
    ) -> Result<(), CloudWorkerArtifactAdmissionError> {
        policy.validate_source(&self.source, self.size_bytes)
    }
}

pub fn validate_archive_member_path(path: &str) -> Result<(), CloudWorkerArtifactAdmissionError> {
    if path.is_empty() || path.contains('\\') {
        return Err(CloudWorkerArtifactAdmissionError::UnsafeArchivePath);
    }
    let path = std::path::Path::new(path);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(component, Component::ParentDir | Component::Prefix(_))
        })
    {
        return Err(CloudWorkerArtifactAdmissionError::UnsafeArchivePath);
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum CloudWorkerArtifactAdmissionError {
    #[error("artifact digest must be exactly 64 hexadecimal characters")]
    InvalidDigest,
    #[error("artifact size limit must be greater than zero")]
    InvalidSizeLimit,
    #[error("artifact origin must be an HTTPS origin with no path")]
    InvalidOrigin,
    #[error("artifact origin is not allowlisted: {0}")]
    OriginNotAllowed(String),
    #[error("artifact is {bytes} bytes, above the {max_bytes}-byte limit")]
    ArtifactTooLarge { bytes: u64, max_bytes: u64 },
    #[error("archive member path is unsafe")]
    UnsafeArchivePath,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest() -> CloudWorkerArtifactDigest {
        CloudWorkerArtifactDigest::new("a".repeat(64)).unwrap()
    }

    fn policy() -> CloudWorkerArtifactPolicy {
        CloudWorkerArtifactPolicy {
            allowed_https_origins: vec!["https://artifacts.example".into()],
            max_bytes: 1024,
        }
    }

    #[test]
    fn accepts_an_allowlisted_https_source() {
        let source = CloudWorkerArtifactSource::Https {
            url: "https://artifacts.example/worker.tar".into(),
            digest: digest(),
        };
        assert_eq!(policy().validate_source(&source, 10), Ok(()));
    }

    #[test]
    fn rejects_credentials_in_artifact_urls() {
        let source = CloudWorkerArtifactSource::Https {
            url: "https://user:pass@artifacts.example/worker.tar".into(),
            digest: digest(),
        };
        assert!(matches!(
            policy().validate_source(&source, 10),
            Err(CloudWorkerArtifactAdmissionError::InvalidOrigin)
        ));
    }

    #[test]
    fn rejects_parent_archive_paths() {
        assert_eq!(
            validate_archive_member_path("worker/../secret"),
            Err(CloudWorkerArtifactAdmissionError::UnsafeArchivePath)
        );
    }
}
