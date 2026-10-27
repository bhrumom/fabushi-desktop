use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum ActivationTrigger {
    Command(String),
    Provider(String),
    AgentHarness(String),
    Channel(String),
    Route(String),
    Capability(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifestRecord {
    pub plugin_id: String,
    pub commands: Vec<String>,
    pub providers: Vec<String>,
    pub channels: Vec<String>,
    pub capabilities: Vec<String>,
    pub is_bundled: bool,
    pub is_trusted: bool,
}

impl PluginManifestRecord {
    pub fn matches_trigger(&self, trigger: &ActivationTrigger) -> bool {
        match trigger {
            ActivationTrigger::Command(cmd) => {
                self.commands.iter().any(|c| c.eq_ignore_ascii_case(cmd))
            }
            ActivationTrigger::Provider(prov) => {
                self.providers.iter().any(|p| p.eq_ignore_ascii_case(prov))
            }
            ActivationTrigger::AgentHarness(harness) => {
                self.capabilities.iter().any(|c| c.eq_ignore_ascii_case(harness))
            }
            ActivationTrigger::Channel(chan) => {
                self.channels.iter().any(|c| c.eq_ignore_ascii_case(chan))
            }
            ActivationTrigger::Route(route) => {
                self.channels.iter().any(|c| c.eq_ignore_ascii_case(route))
            }
            ActivationTrigger::Capability(cap) => {
                self.capabilities.iter().any(|c| c.eq_ignore_ascii_case(cap))
            }
        }
    }

    pub fn is_activatable(&self) -> bool {
        self.is_bundled || self.is_trusted
    }
}

#[derive(Clone, Default)]
pub struct ActivationPlanner {
    plugins: HashMap<String, PluginManifestRecord>,
}

impl ActivationPlanner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_manifest(&mut self, manifest: PluginManifestRecord) {
        self.plugins.insert(manifest.plugin_id.clone(), manifest);
    }

    pub fn plan_activation(&self, triggers: &[ActivationTrigger]) -> Vec<String> {
        let mut activated_plugins = Vec::new();
        let mut seen = HashSet::new();

        for trigger in triggers {
            for plugin in self.plugins.values() {
                if plugin.matches_trigger(trigger) && plugin.is_activatable() {
                    if seen.insert(plugin.plugin_id.clone()) {
                        activated_plugins.push(plugin.plugin_id.clone());
                    }
                }
            }
        }

        activated_plugins.sort();
        activated_plugins
    }
}
