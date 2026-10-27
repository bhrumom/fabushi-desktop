use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OutboundMessageIdentity {
    pub channel: String,
    pub message_id: String,
    pub account_id: Option<String>,
}

#[derive(Clone, Default)]
pub struct OutboundEchoTracker {
    recent: Arc<Mutex<HashSet<OutboundMessageIdentity>>>,
}

impl OutboundEchoTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&self, identity: OutboundMessageIdentity) {
        let mut set = self.recent.lock().unwrap();
        set.insert(identity);
    }

    pub fn is_recent(&self, identity: &OutboundMessageIdentity) -> bool {
        let set = self.recent.lock().unwrap();
        set.contains(identity)
    }

    pub fn clear(&self) {
        let mut set = self.recent.lock().unwrap();
        set.clear();
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
