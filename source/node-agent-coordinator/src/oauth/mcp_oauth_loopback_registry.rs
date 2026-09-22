use std::collections::{HashMap, HashSet};

use crate::protocol::Failure;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopbackRedirect {
    pub origin: String,
    pub host: String,
    pub port: u16,
    pub path: String,
}

pub fn parse_loopback_redirect(value: &str) -> Result<LoopbackRedirect, Failure> {
    let rest = value.strip_prefix("http://").ok_or_else(|| {
        Failure::new(
            "MCP_OAUTH_INVALID_REDIRECT",
            "MCP OAuth redirect_uri must be a localhost HTTP URL",
        )
    })?;
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    if authority.contains('@') {
        return Err(Failure::new(
            "MCP_OAUTH_INVALID_REDIRECT",
            "MCP OAuth redirect_uri must not contain user information",
        ));
    }
    let (host, port_text) = authority.rsplit_once(':').ok_or_else(|| {
        Failure::new(
            "MCP_OAUTH_INVALID_REDIRECT",
            "MCP OAuth redirect_uri must include a port",
        )
    })?;
    if host != "127.0.0.1" && host != "localhost" {
        return Err(Failure::new(
            "MCP_OAUTH_INVALID_REDIRECT",
            "MCP OAuth redirect_uri must use localhost or 127.0.0.1",
        ));
    }
    let port = port_text.parse::<u16>().ok().filter(|port| *port > 0).ok_or_else(|| {
        Failure::new(
            "MCP_OAUTH_INVALID_REDIRECT",
            "MCP OAuth redirect_uri must include a valid non-zero port",
        )
    })?;
    let suffix = &rest[authority_end..];
    if suffix.contains('#') {
        return Err(Failure::new(
            "MCP_OAUTH_INVALID_REDIRECT",
            "MCP OAuth redirect_uri must not contain a fragment",
        ));
    }
    let path = suffix
        .split('?')
        .next()
        .filter(|path| !path.is_empty())
        .unwrap_or("/")
        .to_string();
    Ok(LoopbackRedirect {
        origin: format!("http://{host}:{port}"),
        host: host.to_string(),
        port,
        path,
    })
}

pub fn loopback_bind_hosts(redirect_host: &str) -> Vec<String> {
    if redirect_host == "localhost" {
        vec!["127.0.0.1".into(), "::1".into()]
    } else {
        vec![redirect_host.to_string()]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopbackLease {
    pub origin: String,
    pub lease_id: u64,
    pub port: u16,
    pub bind_hosts: Vec<String>,
}

#[derive(Debug)]
struct LoopbackEntry {
    port: u16,
    host: String,
    leases: HashSet<u64>,
}

#[derive(Debug, Default)]
pub struct McpOAuthLoopbackRegistry {
    origins: HashMap<String, LoopbackEntry>,
    next_lease_id: u64,
}

impl McpOAuthLoopbackRegistry {
    pub fn acquire(&mut self, redirect_url: &str) -> Result<LoopbackLease, Failure> {
        let parsed = parse_loopback_redirect(redirect_url)?;
        let entry = self.origins.entry(parsed.origin.clone()).or_insert_with(|| LoopbackEntry {
            port: parsed.port,
            host: parsed.host.clone(),
            leases: HashSet::new(),
        });
        if entry.port != parsed.port || entry.host != parsed.host {
            return Err(Failure::new(
                "MCP_OAUTH_LOOPBACK_CONFLICT",
                "loopback origin changed while it was acquired",
            ));
        }
        self.next_lease_id = self.next_lease_id.saturating_add(1).max(1);
        let lease_id = self.next_lease_id;
        entry.leases.insert(lease_id);
        Ok(LoopbackLease {
            origin: parsed.origin,
            lease_id,
            port: parsed.port,
            bind_hosts: loopback_bind_hosts(&parsed.host),
        })
    }

    pub fn release(&mut self, lease: &LoopbackLease) -> bool {
        let Some(entry) = self.origins.get_mut(&lease.origin) else {
            return false;
        };
        let removed = entry.leases.remove(&lease.lease_id);
        if entry.leases.is_empty() {
            self.origins.remove(&lease.origin);
        }
        removed
    }

    pub fn active_origin_count(&self) -> usize {
        self.origins.len()
    }

    pub fn lease_count(&self, origin: &str) -> usize {
        self.origins
            .get(origin)
            .map(|entry| entry.leases.len())
            .unwrap_or(0)
    }

    pub fn dispose(&mut self) {
        self.origins.clear();
    }
}
