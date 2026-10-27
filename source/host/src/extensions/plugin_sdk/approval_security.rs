use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalBinding {
    pub token: String,
    pub action_name: String,
    pub session_id: String,
    pub approver_id: Option<String>,
    pub expires_at_ms: u64,
}

#[derive(Clone, Default)]
pub struct NativeApprovalControlRegistry {
    bindings: Arc<Mutex<HashMap<String, ApprovalBinding>>>,
    resolving: Arc<Mutex<HashSet<String>>>,
}

impl NativeApprovalControlRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn generate_token() -> Result<String, String> {
        let mut buf = [0u8; 18];
        getrandom::fill(&mut buf).map_err(|e| format!("Random generation failed: {e}"))?;
        Ok(URL_SAFE_NO_PAD.encode(buf))
    }

    pub fn register(&self, binding: ApprovalBinding) -> bool {
        let now_ms = current_time_ms();
        if binding.expires_at_ms <= now_ms {
            return false;
        }

        let mut bindings = self.bindings.lock().unwrap();
        bindings.insert(binding.token.clone(), binding);
        true
    }

    pub fn get(&self, token: &str) -> Option<ApprovalBinding> {
        let mut bindings = self.bindings.lock().unwrap();
        if let Some(binding) = bindings.get(token) {
            let now_ms = current_time_ms();
            if binding.expires_at_ms <= now_ms {
                bindings.remove(token);
                let mut resolving = self.resolving.lock().unwrap();
                resolving.remove(token);
                None
            } else {
                Some(binding.clone())
            }
        } else {
            None
        }
    }

    pub fn complete(&self, token: &str) -> Option<ApprovalBinding> {
        let mut bindings = self.bindings.lock().unwrap();
        let mut resolving = self.resolving.lock().unwrap();
        resolving.remove(token);
        bindings.remove(token)
    }

    pub fn prune_expired(&self) -> usize {
        let now_ms = current_time_ms();
        let mut bindings = self.bindings.lock().unwrap();
        let mut resolving = self.resolving.lock().unwrap();

        let before = bindings.len();
        bindings.retain(|token, b| {
            if b.expires_at_ms <= now_ms {
                resolving.remove(token);
                false
            } else {
                true
            }
        });
        before - bindings.len()
    }
}

pub fn resolve_approval_approvers(
    explicit: Option<&[String]>,
    allow_from: Option<&[String]>,
    extra_allow_from: Option<&[String]>,
    default_to: Option<&str>,
) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    // Explicit approvers take absolute precedence
    if let Some(explicit_list) = explicit {
        for item in explicit_list {
            let trimmed = item.trim();
            if !trimmed.is_empty() && seen.insert(trimmed.to_ascii_lowercase()) {
                result.push(trimmed.to_string());
            }
        }
        if !result.is_empty() {
            return result;
        }
    }

    // Fallbacks
    if let Some(list) = allow_from {
        for item in list {
            let trimmed = item.trim();
            if !trimmed.is_empty() && seen.insert(trimmed.to_ascii_lowercase()) {
                result.push(trimmed.to_string());
            }
        }
    }

    if let Some(list) = extra_allow_from {
        for item in list {
            let trimmed = item.trim();
            if !trimmed.is_empty() && seen.insert(trimmed.to_ascii_lowercase()) {
                result.push(trimmed.to_string());
            }
        }
    }

    if let Some(def) = default_to {
        let trimmed = def.trim();
        if !trimmed.is_empty() && seen.insert(trimmed.to_ascii_lowercase()) {
            result.push(trimmed.to_string());
        }
    }

    result
}

#[derive(Debug, PartialEq, Eq)]
pub enum SsrfCheckResult {
    Allowed,
    Disallowed(String),
}

pub fn validate_outbound_url(url_str: &str) -> SsrfCheckResult {
    let parsed = match url_str.parse::<url::Url>() {
        Ok(u) => u,
        Err(e) => return SsrfCheckResult::Disallowed(format!("Invalid URL syntax: {e}")),
    };

    let scheme = parsed.scheme().to_ascii_lowercase();
    if scheme != "http" && scheme != "https" {
        return SsrfCheckResult::Disallowed(format!("Disallowed scheme: '{scheme}' (must be http or https)"));
    }

    let host = match parsed.host_str() {
        Some(h) => h.to_ascii_lowercase(),
        None => return SsrfCheckResult::Disallowed("Missing host component".to_string()),
    };

    // Check host string
    if host == "localhost" || host.ends_with(".localhost") || host == "local" || host.ends_with(".local") {
        return SsrfCheckResult::Disallowed(format!("SSRF protection: loopback hostname '{host}' is prohibited"));
    }

    // Check if host is an IP address
    if let Ok(ip) = host.parse::<IpAddr>() {
        match ip {
            IpAddr::V4(ipv4) => {
                let octets = ipv4.octets();
                // 127.0.0.0/8 (loopback)
                if octets[0] == 127 {
                    return SsrfCheckResult::Disallowed(format!("SSRF protection: loopback IPv4 '{ip}' is prohibited"));
                }
                // 10.0.0.0/8 (private)
                if octets[0] == 10 {
                    return SsrfCheckResult::Disallowed(format!("SSRF protection: private RFC1918 IPv4 '{ip}' is prohibited"));
                }
                // 172.16.0.0/12 (private)
                if octets[0] == 172 && (16..=31).contains(&octets[1]) {
                    return SsrfCheckResult::Disallowed(format!("SSRF protection: private RFC1918 IPv4 '{ip}' is prohibited"));
                }
                // 192.168.0.0/16 (private)
                if octets[0] == 192 && octets[1] == 168 {
                    return SsrfCheckResult::Disallowed(format!("SSRF protection: private RFC1918 IPv4 '{ip}' is prohibited"));
                }
                // 169.254.0.0/16 (link-local, cloud metadata)
                if octets[0] == 169 && octets[1] == 254 {
                    return SsrfCheckResult::Disallowed(format!("SSRF protection: link-local IPv4 '{ip}' is prohibited"));
                }
                // 0.0.0.0/8
                if octets[0] == 0 {
                    return SsrfCheckResult::Disallowed(format!("SSRF protection: unspecified IPv4 '{ip}' is prohibited"));
                }
            }
            IpAddr::V6(ipv6) => {
                if ipv6.is_loopback() {
                    return SsrfCheckResult::Disallowed(format!("SSRF protection: loopback IPv6 '{ip}' is prohibited"));
                }
                if ipv6.is_unspecified() {
                    return SsrfCheckResult::Disallowed(format!("SSRF protection: unspecified IPv6 '{ip}' is prohibited"));
                }
            }
        }
    }

    SsrfCheckResult::Allowed
}

fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
