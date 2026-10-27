use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub const CAPABILITY_FAMILY_SPEECH_PROVIDERS: &str = "speechProviders";
pub const CAPABILITY_FAMILY_TRANSCRIPTION_PROVIDERS: &str = "realtimeTranscriptionProviders";
pub const CAPABILITY_FAMILY_VOICE_PROVIDERS: &str = "realtimeVoiceProviders";

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginOrigin {
    Bundled,
    Installed,
    Workspace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityProviderEntry {
    pub family: String,
    pub provider_id: String,
    pub plugin_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelCatalogEntry {
    pub plugin_id: String,
    pub origin: PluginOrigin,
    pub package_name: Option<String>,
    pub channel_name: String,
    pub trusted_official_install: bool,
}

#[derive(Clone, Default)]
pub struct PluginCapabilityCatalog {
    capabilities: Arc<Mutex<HashMap<(String, String), CapabilityProviderEntry>>>,
    channels: Arc<Mutex<HashMap<String, ChannelCatalogEntry>>>,
}

impl PluginCapabilityCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_capability(&self, entry: CapabilityProviderEntry) {
        let key = (entry.family.clone(), entry.provider_id.to_ascii_lowercase());
        let mut caps = self.capabilities.lock().unwrap();
        caps.insert(key, entry);
    }

    pub fn get_capability(&self, family: &str, provider_id: &str) -> Option<CapabilityProviderEntry> {
        let key = (family.to_string(), provider_id.to_ascii_lowercase());
        let caps = self.capabilities.lock().unwrap();
        caps.get(&key).cloned()
    }

    pub fn list_capabilities_for_family(&self, family: &str) -> Vec<CapabilityProviderEntry> {
        let caps = self.capabilities.lock().unwrap();
        caps.values()
            .filter(|e| e.family == family)
            .cloned()
            .collect()
    }

    pub fn register_channel(&self, entry: ChannelCatalogEntry) {
        let key = entry.channel_name.to_ascii_lowercase();
        let mut channels = self.channels.lock().unwrap();
        channels.insert(key, entry);
    }

    pub fn get_channel(&self, channel_name: &str) -> Option<ChannelCatalogEntry> {
        let key = channel_name.to_ascii_lowercase();
        let channels = self.channels.lock().unwrap();
        channels.get(&key).cloned()
    }
}
