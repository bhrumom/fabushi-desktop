use crate::protocol::Failure;

pub trait WebAuthnSigner {
    fn sign(&self, challenge: &[u8]) -> Result<Vec<u8>, Failure>;
}

pub struct RejectingWebAuthnSigner;

impl WebAuthnSigner for RejectingWebAuthnSigner {
    fn sign(&self, _challenge: &[u8]) -> Result<Vec<u8>, Failure> {
        Err(Failure::new(
            "WEBAUTHN_UNAVAILABLE",
            "WebAuthn signing requires the platform signer adapter",
        ))
    }
}
