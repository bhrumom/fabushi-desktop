use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DmPolicy {
    Open,
    Pairing,
    Allowlist,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupPolicy {
    Open,
    Allowlist,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub channel_id: String,
    pub dm_policy: DmPolicy,
    pub group_policy: GroupPolicy,
    pub allow_from: Vec<String>,
    pub block_streaming_coalesce: bool,
    pub markdown_enabled: bool,
}

impl ChannelConfig {
    pub fn new(channel_id: impl Into<String>) -> Self {
        Self {
            channel_id: channel_id.into(),
            dm_policy: DmPolicy::Open,
            group_policy: GroupPolicy::Open,
            allow_from: Vec::new(),
            block_streaming_coalesce: false,
            markdown_enabled: true,
        }
    }

    pub fn is_user_allowed(&self, user_id: &str) -> bool {
        if self.allow_from.iter().any(|allowed| allowed == "*" || allowed.eq_ignore_ascii_case(user_id)) {
            return true;
        }
        false
    }

    pub fn can_access_dm(&self, user_id: &str) -> bool {
        match self.dm_policy {
            DmPolicy::Open => true,
            DmPolicy::Pairing => true,
            DmPolicy::Allowlist => self.is_user_allowed(user_id),
            DmPolicy::Disabled => false,
        }
    }

    pub fn can_access_group(&self, _group_id: &str, sender_id: &str) -> bool {
        match self.group_policy {
            GroupPolicy::Open => true,
            GroupPolicy::Allowlist => self.is_user_allowed(sender_id),
            GroupPolicy::Disabled => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayClaimResult {
    Claimed,
    Duplicate,
    InFlight,
    Invalid,
}

struct ReplayEntry {
    committed_at: Instant,
    ttl: Duration,
}

#[derive(Clone, Default)]
pub struct ChannelReplayGuard {
    inner: Arc<Mutex<ChannelReplayGuardInner>>,
}

#[derive(Default)]
struct ChannelReplayGuardInner {
    inflight: HashSet<String>,
    committed: HashMap<String, ReplayEntry>,
}

impl ChannelReplayGuard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn claim(&self, key: &str, ttl: Duration) -> ReplayClaimResult {
        let trimmed = key.trim();
        if trimmed.is_empty() {
            return ReplayClaimResult::Invalid;
        }

        let mut inner = self.inner.lock().unwrap();
        let now = Instant::now();

        // Evict expired entries
        inner.committed.retain(|_, entry| now.duration_since(entry.committed_at) < entry.ttl);

        if inner.committed.contains_key(trimmed) {
            return ReplayClaimResult::Duplicate;
        }

        if inner.inflight.contains(trimmed) {
            return ReplayClaimResult::InFlight;
        }

        inner.inflight.insert(trimmed.to_string());
        ReplayClaimResult::Claimed
    }

    pub fn commit(&self, key: &str, ttl: Duration) -> bool {
        let trimmed = key.trim();
        let mut inner = self.inner.lock().unwrap();
        if inner.inflight.remove(trimmed) {
            inner.committed.insert(
                trimmed.to_string(),
                ReplayEntry {
                    committed_at: Instant::now(),
                    ttl,
                },
            );
            true
        } else {
            false
        }
    }

    pub fn release(&self, key: &str) -> bool {
        let trimmed = key.trim();
        let mut inner = self.inner.lock().unwrap();
        inner.inflight.remove(trimmed)
    }

    pub fn is_committed(&self, key: &str) -> bool {
        let trimmed = key.trim();
        let mut inner = self.inner.lock().unwrap();
        let now = Instant::now();
        inner.committed.retain(|_, entry| now.duration_since(entry.committed_at) < entry.ttl);
        inner.committed.contains_key(trimmed)
    }
}

pub fn calculate_retry_delay(attempt: u32, base_ms: u64, max_delay_ms: u64) -> Duration {
    let factor = 1u64.checked_shl(attempt.min(10)).unwrap_or(1024);
    let calculated = base_ms.saturating_mul(factor);
    Duration::from_millis(calculated.min(max_delay_ms))
}

pub fn normalize_recipient_list(raw: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut seen = HashSet::new();

    for token in raw.split(|c| c == ',' || c == ';' || c == '\n' || c == '\r') {
        let trimmed = token.trim();
        if !trimmed.is_empty() && seen.insert(trimmed.to_ascii_lowercase()) {
            result.push(trimmed.to_string());
        }
    }
    result
}
