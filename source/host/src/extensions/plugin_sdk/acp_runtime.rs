//! OpenClaw Plugin SDK Agent Control Plane (ACP) Runtime
//!
//! Mirrors:
//! - `src/plugin-sdk/acp-runtime.ts`
//! - `src/plugin-sdk/acpx.ts`
//! - `src/plugin-sdk/acp-runtime-backend.ts`
//! - `src/plugin-sdk/acp-binding-runtime.ts`
//! - `src/plugin-sdk/acp-binding-resolve-runtime.ts`
//! - `src/plugin-sdk/acp-runtime.test.ts`
//! - `src/plugin-sdk/acp-runtime.manager-api.test.ts`

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcpRuntimeStatus {
    Ready,
    Degraded,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcpDoctorReport {
    pub backend_name: String,
    pub status: AcpRuntimeStatus,
    pub active_sessions_count: usize,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcpError {
    SessionNotFound(String),
    IncognitoSessionEnded(String),
    IncognitoSessionSyncAccessDenied(String),
    BackendError(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcpSessionEntry {
    pub session_id: String,
    pub is_incognito: bool,
    pub is_active: bool,
    pub agent_id: String,
}

#[derive(Debug, Default)]
pub struct AcpSessionManager {
    sessions: HashMap<String, AcpSessionEntry>,
}

impl AcpSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub fn register_session(&mut self, session: AcpSessionEntry) {
        self.sessions.insert(session.session_id.clone(), session);
    }

    pub fn end_session(&mut self, session_id: &str) -> Result<(), AcpError> {
        let entry = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| AcpError::SessionNotFound(session_id.to_string()))?;
        entry.is_active = false;
        Ok(())
    }

    pub fn verify_session_access(&self, session_id: &str, require_sync: bool) -> Result<&AcpSessionEntry, AcpError> {
        let entry = self
            .sessions
            .get(session_id)
            .ok_or_else(|| AcpError::SessionNotFound(session_id.to_string()))?;

        if entry.is_incognito {
            if !entry.is_active {
                return Err(AcpError::IncognitoSessionEnded(session_id.to_string()));
            }
            if require_sync {
                return Err(AcpError::IncognitoSessionSyncAccessDenied(session_id.to_string()));
            }
        }

        Ok(entry)
    }

    pub fn generate_doctor_report(&self, backend_name: &str) -> AcpDoctorReport {
        let active_count = self.sessions.values().filter(|s| s.is_active).count();
        AcpDoctorReport {
            backend_name: backend_name.to_string(),
            status: AcpRuntimeStatus::Ready,
            active_sessions_count: active_count,
            error_message: None,
        }
    }
}
