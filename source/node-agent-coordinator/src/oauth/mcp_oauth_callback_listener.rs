use crate::protocol::Failure;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuthCallback { pub code: String, pub state: String }

impl OAuthCallback {
    pub fn validate(self) -> Result<Self, Failure> {
        if self.code.trim().is_empty() || self.state.trim().is_empty() {
            return Err(Failure::new("MCP_OAUTH_INVALID_CALLBACK", "OAuth callback requires code and state"));
        }
        Ok(self)
    }
}
