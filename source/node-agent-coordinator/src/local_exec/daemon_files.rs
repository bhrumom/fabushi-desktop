#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonArtifact { pub path: String, pub sha256: String }

impl DaemonArtifact {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.path.trim().is_empty() { return Err("daemon artifact path is empty"); }
        if self.sha256.len() != 64 || !self.sha256.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err("daemon artifact sha256 must be a 64-character hexadecimal digest");
        }
        Ok(())
    }
}
