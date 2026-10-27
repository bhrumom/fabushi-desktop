//! OpenClaw Node Host Gateway Connection, Origin Scopes, and Device Token Reuse
//!
//! Mirrors:
//! - `src/node-host/gateway-auth.ts`
//! - `src/node-host/gateway-candidate-connection.ts`
//! - `src/node-host/client.ts`
//! - `src/node-host/connection.ts`
//! - `src/node-host/node-event-params.ts`
//! - `src/node-host/local-id.ts`
//! - `src/node-host/host-stats.ts`
//! - `src/node-host/startup-state-readiness.ts`

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeHostGatewayConfig {
    pub host: String,
    pub port: u16,
    pub tls: bool,
}

impl NodeHostGatewayConfig {
    pub fn new(host: impl Into<String>, port: u16, tls: bool) -> Self {
        Self {
            host: host.into(),
            port,
            tls,
        }
    }

    pub fn to_url(&self) -> String {
        let scheme = if self.tls { "https" } else { "http" };
        format!("{}://{}:{}", scheme, self.host, self.port)
    }
}

pub fn gateway_origin_scope(url_str: &str) -> String {
    let trimmed = url_str.trim();
    if let Some(idx) = trimmed.find("://") {
        let scheme = &trimmed[..idx];
        let rest = &trimmed[idx + 3..];
        let host_port = rest.split('/').next().unwrap_or(rest);
        format!("{}://{}", scheme.to_ascii_lowercase(), host_port.to_ascii_lowercase())
    } else {
        trimmed.to_ascii_lowercase()
    }
}

pub fn can_reuse_node_host_device_token(
    saved_url: Option<&str>,
    candidate_urls: &[&str],
    device_token: Option<&str>,
) -> bool {
    let token = match device_token {
        Some(t) if !t.trim().is_empty() => t,
        _ => return false,
    };

    let saved_scope = match saved_url {
        Some(u) if !u.trim().is_empty() => gateway_origin_scope(u),
        _ => return false,
    };

    if candidate_urls.is_empty() {
        return false;
    }

    let all_match = candidate_urls
        .iter()
        .all(|cand| gateway_origin_scope(cand) == saved_scope);

    all_match && !token.is_empty()
}
