use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const CODEX_BUNDLE_MANIFEST_RELATIVE_PATH: &str = ".codex-plugin/plugin.json";
pub const CLAUDE_BUNDLE_MANIFEST_RELATIVE_PATH: &str = ".claude-plugin/plugin.json";
pub const CURSOR_BUNDLE_MANIFEST_RELATIVE_PATH: &str = ".cursor-plugin/plugin.json";
pub const AGENT_BUNDLE_MANIFEST_RELATIVE_PATH: &str = "plugin.json";

pub const DEFAULT_MAX_ARCHIVE_BYTES_ZIP: u64 = 50 * 1024 * 1024; // 50 MB
pub const DEFAULT_MAX_EXTRACTED_BYTES: u64 = 250 * 1024 * 1024; // 250 MB
pub const DEFAULT_MAX_ENTRIES: usize = 5000;
pub const DEFAULT_MAX_ENTRY_BYTES: u64 = 50 * 1024 * 1024; // 50 MB

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginBundleFormat {
    Agent,
    Claude,
    Codex,
    Cursor,
}

impl PluginBundleFormat {
    pub fn manifest_relative_path(&self) -> &'static str {
        match self {
            PluginBundleFormat::Agent => AGENT_BUNDLE_MANIFEST_RELATIVE_PATH,
            PluginBundleFormat::Claude => CLAUDE_BUNDLE_MANIFEST_RELATIVE_PATH,
            PluginBundleFormat::Codex => CODEX_BUNDLE_MANIFEST_RELATIVE_PATH,
            PluginBundleFormat::Cursor => CURSOR_BUNDLE_MANIFEST_RELATIVE_PATH,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClawHubPluginSpec {
    pub scope: Option<String>,
    pub name: String,
    pub version: String,
}

pub fn parse_clawhub_plugin_spec(raw: &str) -> Result<ClawHubPluginSpec, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("ClawHub plugin spec cannot be empty".to_string());
    }

    let (package_part, version) = if let Some(idx) = trimmed.rfind('@') {
        if idx > 0 {
            // Version specified after '@'
            let pkg = &trimmed[..idx];
            let ver = &trimmed[idx + 1..];
            if ver.is_empty() {
                return Err("Empty version in plugin spec".to_string());
            }
            (pkg, ver.to_string())
        } else {
            // Leading '@' indicating scope, e.g. @scope/name
            (trimmed, "latest".to_string())
        }
    } else {
        (trimmed, "latest".to_string())
    };

    if package_part.starts_with('@') {
        let parts: Vec<&str> = package_part.splitn(2, '/').collect();
        if parts.len() != 2 || parts[0].len() <= 1 || parts[1].is_empty() {
            return Err(format!("Invalid scoped package name: '{package_part}'"));
        }
        let scope = parts[0][1..].to_string();
        let name = parts[1].to_string();
        Ok(ClawHubPluginSpec {
            scope: Some(scope),
            name,
            version,
        })
    } else {
        Ok(ClawHubPluginSpec {
            scope: None,
            name: package_part.to_string(),
            version,
        })
    }
}

pub fn verify_archive_limits(
    archive_bytes: u64,
    extracted_bytes: u64,
    entry_count: usize,
) -> Result<(), String> {
    if archive_bytes > DEFAULT_MAX_ARCHIVE_BYTES_ZIP {
        return Err(format!(
            "Archive size {archive_bytes} exceeds maximum allowed {DEFAULT_MAX_ARCHIVE_BYTES_ZIP}"
        ));
    }
    if extracted_bytes > DEFAULT_MAX_EXTRACTED_BYTES {
        return Err(format!(
            "Extracted size {extracted_bytes} exceeds maximum allowed {DEFAULT_MAX_EXTRACTED_BYTES}"
        ));
    }
    if entry_count > DEFAULT_MAX_ENTRIES {
        return Err(format!(
            "Entry count {entry_count} exceeds maximum allowed {DEFAULT_MAX_ENTRIES}"
        ));
    }
    Ok(())
}

pub fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

pub fn verify_sha256_integrity(data: &[u8], expected_digest: &str) -> Result<(), String> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();

    let clean_expected = expected_digest.trim();
    let expected_hex = if let Some(stripped) = clean_expected.strip_prefix("sha256-") {
        // Could be base64
        if let Ok(decoded) = STANDARD.decode(stripped) {
            encode_hex(&decoded)
        } else {
            stripped.to_ascii_lowercase()
        }
    } else {
        clean_expected.to_ascii_lowercase()
    };

    let actual_hex = encode_hex(&result);
    if actual_hex.eq_ignore_ascii_case(&expected_hex) {
        Ok(())
    } else {
        Err(format!(
            "Integrity mismatch: expected '{expected_hex}', actual '{actual_hex}'"
        ))
    }
}
