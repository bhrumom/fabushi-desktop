use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use crate::extensions::transcript::channel_delivery_unregistered_error::SandChannelDeliveryUnregisteredError;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

pub const CHANNEL_MAX_MESSAGE_CHARS: usize = 4_000;

/// Split outbound text at channel-sized boundaries while keeping every
/// emitted chunk's markdown code fences balanced. The channel provider remains
/// responsible for transport-specific limits; this is the shared conservative
/// limit used by the Host delivery boundary.
pub fn split_channel_message(text: &str, limit: usize) -> Vec<String> {
    let limit = limit.max(1);
    if text.chars().count() <= limit {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut in_code_fence = false;
    let mut fence_lang = String::new();

    for line in text.lines() {
        let separator = usize::from(!current.is_empty());
        let projected = current.chars().count() + separator + line.chars().count();
        if projected > limit && !current.is_empty() {
            if in_code_fence {
                current.push_str("\n```");
                chunks.push(current);
                current = if fence_lang.is_empty() {
                    format!("```\n{line}")
                } else {
                    format!("```{fence_lang}\n{line}")
                };
            } else {
                chunks.push(current);
                current = line.to_string();
            }
        } else {
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(line);
        }

        if line.trim_start().starts_with("```") {
            if in_code_fence {
                in_code_fence = false;
                fence_lang.clear();
            } else {
                in_code_fence = true;
                fence_lang = line
                    .trim_start()
                    .trim_start_matches("```")
                    .trim()
                    .to_string();
            }
        }
    }

    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

/// Evaluates the channel sender policy at the connector boundary. Empty
/// allowlists are permissive, `*` allows every normalized sender, and the
/// denylist always wins over either form of allowlist access.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChannelAclGate {
    allowlist: HashSet<String>,
    denylist: HashSet<String>,
}

impl ChannelAclGate {
    pub fn new<I, J>(allowlist: I, denylist: J) -> Self
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
        J: IntoIterator,
        J::Item: AsRef<str>,
    {
        Self {
            allowlist: Self::normalize_entries(allowlist),
            denylist: Self::normalize_entries(denylist),
        }
    }

    pub fn is_authorized(&self, sender_id: &str) -> bool {
        let sender = normalize_channel_acl_value(sender_id);
        if sender.is_empty() || self.denylist.contains("*") || self.denylist.contains(&sender) {
            return false;
        }
        self.allowlist.is_empty()
            || self.allowlist.contains("*")
            || self.allowlist.contains(&sender)
    }

    fn normalize_entries<I>(entries: I) -> HashSet<String>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        entries
            .into_iter()
            .map(|entry| normalize_channel_acl_value(entry.as_ref()))
            .filter(|entry| !entry.is_empty())
            .collect()
    }
}

fn normalize_channel_acl_value(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

/// Verifies a webhook's standard HMAC-SHA256 signature. Invalid encoding and
/// mismatched lengths fail closed, and the final comparison is constant-time.
pub fn verify_webhook_hmac_sha256(
    payload: &[u8],
    secret: &str,
    expected_hex: &str,
) -> bool {
    let Some(expected) = decode_sha256_hex(expected_hex) else {
        return false;
    };

    const BLOCK_SIZE: usize = 64;
    let mut key = secret.as_bytes().to_vec();
    if key.len() > BLOCK_SIZE {
        key = Sha256::digest(&key).to_vec();
    }
    key.resize(BLOCK_SIZE, 0);

    let mut inner_pad = [0x36_u8; BLOCK_SIZE];
    let mut outer_pad = [0x5c_u8; BLOCK_SIZE];
    for (index, byte) in key.iter().enumerate() {
        inner_pad[index] ^= byte;
        outer_pad[index] ^= byte;
    }

    let mut inner = Sha256::new();
    inner.update(inner_pad);
    inner.update(payload);
    let inner_digest = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(outer_pad);
    outer.update(inner_digest);
    let actual = outer.finalize();
    actual.as_slice().ct_eq(&expected).into()
}

fn decode_sha256_hex(value: &str) -> Option<[u8; 32]> {
    let bytes = value.as_bytes();
    if bytes.len() != 64 {
        return None;
    }
    let mut decoded = [0_u8; 32];
    for (index, pair) in bytes.chunks_exact(2).enumerate() {
        decoded[index] = (hex_nibble(pair[0])? << 4) | hex_nibble(pair[1])?;
    }
    Some(decoded)
}

fn hex_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelOutboundMessage {
    Text { text: String, operation_id: Option<String> },
    Attachment {
        url: String,
        caption: Option<String>,
        operation_id: Option<String>,
    },
}

impl ChannelOutboundMessage {
    pub fn operation_id(&self) -> Option<&str> {
        match self {
            Self::Text { operation_id, .. } | Self::Attachment { operation_id, .. } => {
                operation_id.as_deref()
            }
        }
    }
}

pub trait ChannelDeliveryPort: Send + Sync {
    fn deliver(
        &self,
        agent_id: &str,
        address_token: &str,
        message: &ChannelOutboundMessage,
    ) -> Result<(), String>;
}

/// Routes channel-prefixed addresses to provider-owned delivery ports.
///
/// This is a delivery boundary only: provider ports own transport/auth and the
/// canonical HostChannelDelivery owner owns lifecycle and registration. It does
/// not create channel state, sessions, or another agent/runtime loop.
#[derive(Clone, Default)]
pub struct ChannelDeliveryRouter {
    ports: Arc<RwLock<HashMap<String, Arc<dyn ChannelDeliveryPort>>>>,
}

impl ChannelDeliveryRouter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &self,
        channel_id: impl Into<String>,
        port: Arc<dyn ChannelDeliveryPort>,
    ) -> Result<(), String> {
        let channel_id = channel_id.into().trim().to_ascii_lowercase();
        if channel_id.is_empty() {
            return Err("channel delivery registration requires a channel id".into());
        }
        let mut ports = self
            .ports
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if ports.contains_key(&channel_id) {
            return Err(format!("channel delivery provider already registered: {channel_id}"));
        }
        ports.insert(channel_id, port);
        Ok(())
    }

    pub fn replace(
        &self,
        channel_id: impl Into<String>,
        port: Arc<dyn ChannelDeliveryPort>,
    ) -> Result<(), String> {
        let channel_id = channel_id.into().trim().to_ascii_lowercase();
        if channel_id.is_empty() {
            return Err("channel delivery replacement requires a channel id".into());
        }
        self.ports
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(channel_id, port);
        Ok(())
    }

    pub fn unregister(&self, channel_id: &str) -> bool {
        self.ports
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&channel_id.trim().to_ascii_lowercase())
            .is_some()
    }

    fn resolve(&self, address_token: &str) -> Option<Arc<dyn ChannelDeliveryPort>> {
        let channel_id = address_token.split_once(':')?.0.trim().to_ascii_lowercase();
        self.ports
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&channel_id)
            .cloned()
    }
}

impl ChannelDeliveryPort for ChannelDeliveryRouter {
    fn deliver(
        &self,
        agent_id: &str,
        address_token: &str,
        message: &ChannelOutboundMessage,
    ) -> Result<(), String> {
        let Some(port) = self.resolve(address_token) else {
            return Err(SandChannelDeliveryUnregisteredError.to_string());
        };
        match message {
            ChannelOutboundMessage::Text { text, operation_id } => {
                for (index, chunk) in split_channel_message(text, CHANNEL_MAX_MESSAGE_CHARS)
                    .into_iter()
                    .enumerate()
                {
                    let chunk_operation_id = operation_id.as_ref().map(|id| {
                        format!(
                            "{id}:chunk-{index}",
                            index = index + 1,
                        )
                    });
                    let chunk_message = ChannelOutboundMessage::Text {
                        text: chunk,
                        operation_id: chunk_operation_id,
                    };
                    port.deliver(agent_id, address_token, &chunk_message)?;
                }
                Ok(())
            }
            ChannelOutboundMessage::Attachment { .. } => {
                port.deliver(agent_id, address_token, message)
            }
        }
    }
}

#[derive(Default)]
pub struct UnregisteredChannelDeliveryPort;

impl ChannelDeliveryPort for UnregisteredChannelDeliveryPort {
    fn deliver(
        &self,
        _agent_id: &str,
        _address_token: &str,
        _message: &ChannelOutboundMessage,
    ) -> Result<(), String> {
        Err(SandChannelDeliveryUnregisteredError.to_string())
    }
}

#[derive(Clone)]
pub struct HostChannelDelivery {
    port: Arc<RwLock<Arc<dyn ChannelDeliveryPort>>>,
}

impl Default for HostChannelDelivery {
    fn default() -> Self {
        Self::new(Arc::new(UnregisteredChannelDeliveryPort))
    }
}

impl HostChannelDelivery {
    pub fn new(port: Arc<dyn ChannelDeliveryPort>) -> Self {
        Self {
            port: Arc::new(RwLock::new(port)),
        }
    }

    pub fn set_port(&self, port: Arc<dyn ChannelDeliveryPort>) {
        *self
            .port
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = port;
    }

    pub fn reset(&self) {
        self.set_port(Arc::new(UnregisteredChannelDeliveryPort));
    }

    pub fn deliver(
        &self,
        agent_id: &str,
        address_token: &str,
        message: &ChannelOutboundMessage,
    ) -> Result<(), String> {
        let port = self
            .port
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        port.deliver(agent_id, address_token, message)
    }
}
