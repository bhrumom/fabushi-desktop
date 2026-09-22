use crate::protocol::Failure;
use crate::webauthn::WebAuthnSigner;

pub struct WebAuthnProvider<S: WebAuthnSigner> { signer: S }

impl<S: WebAuthnSigner> WebAuthnProvider<S> {
    pub fn new(signer: S) -> Self { Self { signer } }
    pub fn sign_challenge(&self, challenge: &[u8]) -> Result<Vec<u8>, Failure> {
        self.signer.sign(challenge)
    }
}
