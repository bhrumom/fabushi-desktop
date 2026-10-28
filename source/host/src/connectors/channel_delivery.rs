use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::extensions::transcript::channel_delivery_unregistered_error::SandChannelDeliveryUnregisteredError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelOutboundMessage {
    Text { text: String },
    Attachment { url: String, caption: Option<String> },
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
        port.deliver(agent_id, address_token, message)
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
