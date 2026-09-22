use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingOAuth {
    pub origin: String,
    pub state: String,
    pub server_name: String,
    pub expires_at_ms: u64,
}

#[derive(Debug, Default)]
pub struct OAuthPendingRegistry {
    by_key: HashMap<(String, String), PendingOAuth>,
}

impl OAuthPendingRegistry {
    pub fn track(&mut self, pending: PendingOAuth) {
        self.remove_server(&pending.server_name);
        self.by_key.insert((pending.origin.clone(), pending.state.clone()), pending);
    }

    pub fn resolve(&self, origin: &str, state: &str, now_ms: u64) -> Option<&str> {
        self.by_key
            .get(&(origin.to_string(), state.to_string()))
            .filter(|entry| entry.expires_at_ms > now_ms)
            .map(|entry| entry.server_name.as_str())
    }

    pub fn expire(&mut self, now_ms: u64) {
        self.by_key.retain(|_, entry| entry.expires_at_ms > now_ms);
    }

    pub fn remove_server(&mut self, server_name: &str) {
        self.by_key.retain(|_, entry| entry.server_name != server_name);
    }
}

pub mod mcp_oauth_callback_listener;
pub mod mcp_oauth_forwarder;
pub mod mcp_oauth_loopback_registry;
