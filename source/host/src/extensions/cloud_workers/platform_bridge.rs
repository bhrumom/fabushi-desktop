use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudWorkerPlatform {
    Linux,
    Macos,
    Windows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudWorkerDesktopSession {
    Headless,
    Interactive,
    BrowserCdp,
    Vnc,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudWorkerPlatformBridge {
    pub platform: CloudWorkerPlatform,
    pub session: CloudWorkerDesktopSession,
    pub launcher_id: String,
}

impl CloudWorkerPlatformBridge {
    pub fn new(
        platform: CloudWorkerPlatform,
        session: CloudWorkerDesktopSession,
        launcher_id: impl Into<String>,
    ) -> Result<Self, CloudWorkerPlatformBridgeError> {
        let launcher_id = launcher_id.into();
        if launcher_id.trim().is_empty() {
            return Err(CloudWorkerPlatformBridgeError::InvalidLauncherId);
        }
        if matches!(session, CloudWorkerDesktopSession::Vnc)
            && matches!(platform, CloudWorkerPlatform::Linux)
        {
            return Err(CloudWorkerPlatformBridgeError::UnsupportedSession);
        }
        Ok(Self {
            platform,
            session,
            launcher_id,
        })
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CloudWorkerPlatformBridgeError {
    #[error("platform launcher id must be non-empty")]
    InvalidLauncherId,
    #[error("requested desktop session is not supported by this platform bridge")]
    UnsupportedSession,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_a_thin_platform_descriptor() {
        let bridge = CloudWorkerPlatformBridge::new(
            CloudWorkerPlatform::Macos,
            CloudWorkerDesktopSession::BrowserCdp,
            "macos-launcher",
        )
        .unwrap();
        assert_eq!(bridge.platform, CloudWorkerPlatform::Macos);
    }

    #[test]
    fn rejects_an_unsupported_session_without_fallback() {
        assert_eq!(
            CloudWorkerPlatformBridge::new(
                CloudWorkerPlatform::Linux,
                CloudWorkerDesktopSession::Vnc,
                "linux-launcher",
            ),
            Err(CloudWorkerPlatformBridgeError::UnsupportedSession)
        );
    }
}
