use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebAuthnCeremony {
    pub kind: String,
    pub origin: String,
    #[serde(default)]
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovedWebAuthnConsent {
    pub approved: bool,
    pub prompt_id: Option<String>,
    pub window_handle: Option<String>,
}

impl ApprovedWebAuthnConsent {
    pub fn approved() -> Self {
        Self {
            approved: true,
            prompt_id: None,
            window_handle: None,
        }
    }

    pub fn declined() -> Self {
        Self {
            approved: false,
            prompt_id: None,
            window_handle: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebAuthnSignerError {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WebAuthnSignerResult {
    Success { credential_json: Value },
    Failed { error: WebAuthnSignerError },
}

pub trait WebAuthnSigner {
    fn sign(
        &mut self,
        ceremony: &WebAuthnCeremony,
        approved: Option<&ApprovedWebAuthnConsent>,
    ) -> WebAuthnSignerResult;
}

#[derive(Debug, Default)]
pub struct RejectingWebAuthnSigner;

impl WebAuthnSigner for RejectingWebAuthnSigner {
    fn sign(
        &mut self,
        _ceremony: &WebAuthnCeremony,
        _approved: Option<&ApprovedWebAuthnConsent>,
    ) -> WebAuthnSignerResult {
        WebAuthnSignerResult::Failed {
            error: WebAuthnSignerError {
                name: "NotAllowedError".into(),
                code: Some("webauthn_unavailable".into()),
                message: "WebAuthn signing requires the platform signer adapter".into(),
            },
        }
    }
}

pub mod provider;
pub mod signer;
