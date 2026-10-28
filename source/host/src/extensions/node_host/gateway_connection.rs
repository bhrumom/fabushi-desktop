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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls_fingerprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_path: Option<String>,
}

impl NodeHostGatewayConfig {
    pub fn new(host: impl Into<String>, port: u16, tls: bool) -> Self {
        Self {
            host: host.into(),
            port,
            tls,
            tls_fingerprint: None,
            context_path: None,
        }
    }

    pub fn with_tls_fingerprint(mut self, fingerprint: impl Into<String>) -> Self {
        self.tls_fingerprint = Some(fingerprint.into());
        self
    }

    pub fn with_context_path(mut self, context_path: impl Into<String>) -> Self {
        self.context_path = Some(context_path.into());
        self
    }

    pub fn to_url(&self) -> String {
        let scheme = if self.tls { "https" } else { "http" };
        format!("{}://{}:{}", scheme, self.host, self.port)
    }
}

pub fn gateway_origin_scope(url_str: &str) -> String {
    let trimmed = url_str.trim();
    if trimmed.is_empty() {
        return "default".into();
    }
    let Ok(parsed) = url::Url::parse(trimmed) else {
        return trimmed.to_string();
    };

    let scheme = parsed.scheme().to_ascii_lowercase();
    let host = parsed.host_str().unwrap_or_default().to_ascii_lowercase();
    let host = if host.contains(':') {
        format!("[{host}]")
    } else {
        host
    };
    let default_port = match scheme.as_str() {
        "http" | "ws" => Some(80),
        "https" | "wss" => Some(443),
        _ => None,
    };
    let port = parsed
        .port()
        .filter(|port| Some(*port) != default_port)
        .map(|port| format!(":{port}"))
        .unwrap_or_default();
    let path = parsed.path();
    let path = if path.is_empty() || path == "/" {
        String::new()
    } else {
        path.trim_end_matches('/').to_string()
    };
    format!("{scheme}://{host}{port}{path}")
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
