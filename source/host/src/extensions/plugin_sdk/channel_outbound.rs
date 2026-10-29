use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const OUTBOUND_ECHO_WINDOW: Duration = Duration::from_secs(30);
pub const OUTBOUND_ECHO_MAX_ENTRIES: usize = 10_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutboundMessageIdentity {
    pub channel: String,
    pub conversation_id: String,
    pub account_id: Option<String>,
    pub message_id: Option<String>,
    pub source_id: Option<String>,
}

#[derive(Debug, Clone, Copy)]
struct OutboundEchoEntry {
    expires_at: Instant,
    sequence: u64,
}

#[derive(Clone, Default)]
pub struct OutboundEchoTracker {
    recent: Arc<Mutex<HashMap<String, OutboundEchoEntry>>>,
    window: Duration,
    next_sequence: Arc<Mutex<u64>>,
}

impl OutboundEchoTracker {
    pub fn new() -> Self {
        Self::with_window(OUTBOUND_ECHO_WINDOW)
    }

    pub fn with_window(window: Duration) -> Self {
        Self {
            recent: Arc::new(Mutex::new(HashMap::new())),
            window,
            next_sequence: Arc::new(Mutex::new(0)),
        }
    }

    pub fn record(&self, identity: OutboundMessageIdentity) {
        let keys = identity.keys();
        if keys.is_empty() {
            return;
        }
        let now = Instant::now();
        let mut recent = self.recent.lock().unwrap();
        recent.retain(|_, entry| entry.expires_at > now);
        let expires_at = now + self.window;
        for key in keys {
            recent.remove(&key);
            while recent.len() >= OUTBOUND_ECHO_MAX_ENTRIES {
                let Some(oldest_key) = recent
                    .iter()
                    .min_by_key(|(_, entry)| entry.sequence)
                    .map(|(key, _)| key.clone())
                else {
                    break;
                };
                recent.remove(&oldest_key);
            }
            let mut sequence = self.next_sequence.lock().unwrap();
            *sequence = sequence.wrapping_add(1);
            let current_sequence = *sequence;
            drop(sequence);
            recent.insert(
                key,
                OutboundEchoEntry {
                    expires_at,
                    sequence: current_sequence,
                },
            );
        }
    }

    pub fn is_recent(&self, identity: &OutboundMessageIdentity) -> bool {
        let keys = identity.keys();
        if keys.is_empty() {
            return false;
        }
        let now = Instant::now();
        let mut recent = self.recent.lock().unwrap();
        keys.into_iter().any(|key| {
            let Some(entry) = recent.get(&key).copied() else {
                return false;
            };
            if entry.expires_at <= now {
                recent.remove(&key);
                false
            } else {
                true
            }
        })
    }

    pub fn clear(&self) {
        self.recent.lock().unwrap().clear();
    }
}

impl OutboundMessageIdentity {
    fn keys(&self) -> Vec<String> {
        let channel = self.channel.trim().to_ascii_lowercase();
        let conversation_id = self.conversation_id.trim();
        if channel.is_empty() || conversation_id.is_empty() {
            return Vec::new();
        }
        let account_id = self
            .account_id
            .as_deref()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        let scope = [channel, account_id, conversation_id.to_string()];
        [
            ("message", self.message_id.as_deref()),
            ("source", self.source_id.as_deref()),
        ]
        .into_iter()
        .filter_map(|(kind, value)| {
            let value = value?.trim();
            (!value.is_empty()).then(|| {
                serde_json::to_string(&(scope.clone(), kind, value))
                    .expect("outbound echo identity key is serializable")
            })
        })
        .collect()
    }
}

/// Helper that controls draft streaming chunking boundaries.
#[derive(Debug, Clone)]
pub struct ChannelDraftStreamingChunking {
    pub min_chars: usize,
    pub max_chars: usize,
    pub break_preference: String,
}

impl Default for ChannelDraftStreamingChunking {
    fn default() -> Self {
        Self {
            min_chars: 100,
            max_chars: 1000,
            break_preference: "paragraph".to_string(),
        }
    }
}

impl ChannelDraftStreamingChunking {
    pub fn should_split_chunk(&self, current_len: usize) -> bool {
        current_len >= self.max_chars
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReplyPayload {
    pub text: String,
    pub media_paths: Vec<String>,
}

/// Preserves the source reply media selections when merging or recovering payload text.
pub fn preserve_reply_payload_media_selection(
    source: &ReplyPayload,
    recovered: &mut ReplyPayload,
) {
    if !source.media_paths.is_empty() {
        recovered.media_paths = source.media_paths.clone();
    }
}
