use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PairingChallenge {
    pub channel: String,
    pub account_id: String,
    pub code: String,
    pub sender_id: String,
}

#[derive(Clone, Default)]
pub struct ChannelPairingStore {
    challenges: Arc<Mutex<HashMap<String, (PairingChallenge, Instant)>>>,
    ttl: Duration,
}

impl ChannelPairingStore {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            challenges: Arc::new(Mutex::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    pub fn issue_challenge(
        &self,
        channel: &str,
        account_id: &str,
        sender_id: &str,
    ) -> PairingChallenge {
        // Generate 6-digit numeric challenge code
        let mut buf = [0u8; 4];
        let num = if getrandom::fill(&mut buf).is_ok() {
            u32::from_le_bytes(buf)
        } else {
            123456
        };
        let code = format!("{:06}", (num % 900_000) + 100_000);
        let challenge = PairingChallenge {
            channel: channel.to_string(),
            account_id: account_id.to_string(),
            code: code.clone(),
            sender_id: sender_id.to_string(),
        };

        let mut map = self.challenges.lock().unwrap();
        map.insert(code, (challenge.clone(), Instant::now()));
        challenge
    }

    pub fn verify_and_consume(&self, code: &str) -> Option<PairingChallenge> {
        let mut map = self.challenges.lock().unwrap();
        if let Some((challenge, created_at)) = map.remove(code) {
            if created_at.elapsed() <= self.ttl {
                return Some(challenge);
            }
        }
        None
    }
}

/// Strips standard pairing command prefixes such as "!pair " or "/pair ".
pub fn strip_pairing_command_prefix(input: &str) -> Option<&str> {
    let trimmed = input.trim();
    for prefix in &["!pair ", "/pair ", "pair "] {
        if trimmed.to_ascii_lowercase().starts_with(prefix) {
            return Some(trimmed[prefix.len()..].trim());
        }
    }
    None
}
