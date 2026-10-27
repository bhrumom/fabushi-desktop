use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
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
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelCatalogEntry {
    pub plugin_id: String,
    pub origin: PluginOrigin,
    pub package_name: Option<String>,
    pub channel_name: String,
    pub trusted_official_install: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderAuthChoice {
    pub provider: String,
    pub method: String,
    pub choice_id: String,
    pub choice_label: String,
    pub onboarding_scopes: Vec<String>,
    pub cli_flag: Option<String>,
    pub option_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginContractRegistryEntry {
    pub plugin_id: String,
    pub provider_ids: Vec<String>,
    pub worker_provider_ids: Vec<String>,
    pub web_fetch_provider_ids: Vec<String>,
    pub web_search_provider_ids: Vec<String>,
    pub speech_provider_ids: Vec<String>,
    pub realtime_voice_provider_ids: Vec<String>,
    pub realtime_transcription_provider_ids: Vec<String>,
    pub image_generation_provider_ids: Vec<String>,
}

#[derive(Clone, Default)]
pub struct PluginCapabilityCatalog {
    capabilities: Arc<Mutex<HashMap<(String, String), CapabilityProviderEntry>>>,
    channels: Arc<Mutex<HashMap<String, ChannelCatalogEntry>>>,
    canonical_aliases: Arc<Mutex<HashMap<String, Vec<String>>>>,
    auth_choices: Arc<Mutex<Vec<ProviderAuthChoice>>>,
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

    pub fn matches_provider_id_or_alias(&self, family: &str, candidate: &str) -> bool {
        let candidate_lower = candidate.to_ascii_lowercase();
        let caps = self.capabilities.lock().unwrap();
        caps.values().any(|e| {
            e.family == family
                && (e.provider_id.eq_ignore_ascii_case(&candidate_lower)
                    || e.aliases.iter().any(|a| a.eq_ignore_ascii_case(&candidate_lower)))
        })
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

    pub fn register_canonical_aliases(&self, key: &str, aliases: Vec<String>) {
        let mut map = self.canonical_aliases.lock().unwrap();
        map.insert(key.to_string(), aliases);
    }

    pub fn get_canonical_aliases(&self, key: &str) -> Option<Vec<String>> {
        let map = self.canonical_aliases.lock().unwrap();
        map.get(key).cloned()
    }

    pub fn register_auth_choice(&self, choice: ProviderAuthChoice) {
        let mut list = self.auth_choices.lock().unwrap();
        list.push(choice);
    }

    pub fn build_provider_method_choice(provider_id: &str, method_id: &str) -> String {
        format!("{}:{}", provider_id, method_id)
    }

    pub fn resolve_preferred_provider_for_auth_choice(&self, choice: &str) -> Option<String> {
        let list = self.auth_choices.lock().unwrap();
        for entry in list.iter() {
            if entry.choice_id == choice
                || Self::build_provider_method_choice(&entry.provider, &entry.method) == choice
            {
                return Some(entry.provider.clone());
            }
        }
        None
    }

    pub fn filter_text_onboarding_auth_choices(&self) -> Vec<ProviderAuthChoice> {
        let list = self.auth_choices.lock().unwrap();
        list.iter()
            .filter(|choice| {
                // If onboarding scopes are strictly limited to image-generation or video, exclude from text onboarding
                if choice.onboarding_scopes.len() == 1 && choice.onboarding_scopes[0] == "image-generation" {
                    return false;
                }
                true
            })
            .cloned()
            .collect()
    }

    pub fn validate_unique_provider_ids(entries: &[PluginContractRegistryEntry]) -> Result<(), String> {
        let mut seen_providers = HashSet::new();
        for entry in entries {
            for pid in &entry.provider_ids {
                if !seen_providers.insert(pid.clone()) {
                    return Err(format!("Duplicate provider id: {}", pid));
                }
            }
        }
        Ok(())
    }

    pub fn resolve_web_search_providers(
        bundled: &[String],
        activation_scoped: &[String],
    ) -> Vec<String> {
        let mut result = Vec::new();
        let mut seen = HashSet::new();
        for id in bundled {
            if !activation_scoped.contains(id) && seen.insert(id.clone()) {
                result.push(id.clone());
            }
        }
        for id in activation_scoped {
            if seen.insert(id.clone()) {
                result.push(id.clone());
            }
        }
        result.sort();
        result
    }

    pub fn resolve_provider_contract_providers_for_plugin_ids(
        &self,
        plugin_ids: &[&str],
        public_artifacts: &HashMap<String, Vec<String>>,
    ) -> Vec<String> {
        let mut result = Vec::new();
        let mut seen = HashSet::new();
        for plugin_id in plugin_ids {
            if let Some(providers) = public_artifacts.get(*plugin_id) {
                for p in providers {
                    if seen.insert(p.clone()) {
                        result.push(p.clone());
                    }
                }
            } else {
                let caps = self.capabilities.lock().unwrap();
                for entry in caps.values() {
                    if entry.plugin_id == *plugin_id && seen.insert(entry.provider_id.clone()) {
                        result.push(entry.provider_id.clone());
                    }
                }
            }
        }
        result
    }

    pub fn resolve_web_search_providers_for_plugin_id(
        &self,
        plugin_id: &str,
        public_artifacts: &HashMap<String, Vec<String>>,
    ) -> Vec<String> {
        if let Some(providers) = public_artifacts.get(plugin_id) {
            providers.clone()
        } else {
            let caps = self.capabilities.lock().unwrap();
            caps.values()
                .filter(|e| e.plugin_id == plugin_id && e.family == "webSearch")
                .map(|e| e.provider_id.clone())
                .collect()
        }
    }
}

// Shared upstream model catalog validations (from src/plugins/contracts/shared-upstream-model.contract.test.ts)

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelCatalogRow {
    pub provider_id: String,
    pub model_id: String,
    pub upstream_model: Option<String>,
    pub code_mode: Option<String>,
}

impl ModelCatalogRow {
    pub fn ref_id(&self) -> String {
        format!("{}/{}", self.provider_id, self.model_id)
    }

    pub fn group_key(&self) -> String {
        let target = self.upstream_model.as_deref().unwrap_or(&self.model_id);
        if let Some(pos) = target.find('/') {
            target[pos + 1..].to_ascii_lowercase()
        } else {
            target.to_ascii_lowercase()
        }
    }
}

pub struct SharedUpstreamModelCatalog;

impl SharedUpstreamModelCatalog {
    pub fn validate_upstream_model_refs(rows: &[ModelCatalogRow]) -> Result<(), Vec<String>> {
        let refs: HashSet<String> = rows.iter().map(|r| r.ref_id()).collect();
        let mut unresolved = Vec::new();

        for row in rows {
            if let Some(upstream) = &row.upstream_model {
                if upstream == &row.ref_id() || !refs.contains(upstream) {
                    unresolved.push(format!("{} -> {}", row.ref_id(), upstream));
                }
            }
        }
        if unresolved.is_empty() {
            Ok(())
        } else {
            unresolved.sort();
            Err(unresolved)
        }
    }

    pub fn validate_code_mode_tier_consistency(rows: &[ModelCatalogRow]) -> Result<(), Vec<String>> {
        let mut groups: HashMap<String, Vec<&ModelCatalogRow>> = HashMap::new();
        for row in rows {
            groups.entry(row.group_key()).or_default().push(row);
        }

        let mut silent = Vec::new();
        for (group_key, group) in groups {
            if group.len() < 2 || !group.iter().any(|r| r.code_mode.is_some()) {
                continue;
            }
            let mut declared: Vec<String> = group
                .iter()
                .filter_map(|r| r.code_mode.as_ref().map(|cm| format!("{}={}", r.ref_id(), cm)))
                .collect();
            declared.sort();
            let declared_str = declared.join(", ");

            for r in group.iter().filter(|r| r.code_mode.is_none()) {
                silent.push(format!(
                    "{} (shared model {}; siblings declare {})",
                    r.ref_id(),
                    group_key,
                    declared_str
                ));
            }
        }

        if silent.is_empty() {
            Ok(())
        } else {
            silent.sort();
            Err(silent)
        }
    }
}

// Bundled provider catalog deprecation guard (from src/plugins/contracts/provider-catalog-deprecation.contract.test.ts)

pub struct ProviderCatalogDeprecationGuard;

impl ProviderCatalogDeprecationGuard {
    pub fn assert_no_deprecated_discovery_hooks(sources: &[(&str, &str)]) -> Result<(), Vec<String>> {
        let mut offenders = Vec::new();
        for (file_path, content) in sources {
            if (content.contains("registerProvider") || content.contains("ProviderPlugin"))
                && content.contains("discovery:")
            {
                offenders.push(file_path.to_string());
            }
        }
        if offenders.is_empty() {
            Ok(())
        } else {
            offenders.sort();
            Err(offenders)
        }
    }
}

// Embedding provider registration (from src/plugins/contracts/embedding-provider.contract.test.ts)

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegisteredEmbeddingProvider {
    pub provider_id: String,
    pub plugin_id: String,
}

pub struct EmbeddingProviderRegistry {
    core_providers: std::collections::HashSet<String>,
    providers: Vec<RegisteredEmbeddingProvider>,
    diagnostics: Vec<String>,
}

impl Default for EmbeddingProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl EmbeddingProviderRegistry {
    pub fn new() -> Self {
        let mut core = std::collections::HashSet::new();
        core.insert("openai-compatible".to_string());
        Self {
            core_providers: core,
            providers: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn is_public_sdk_mutation_allowed(method_name: &str) -> bool {
        let forbidden = [
            "registerEmbeddingProvider",
            "listRegisteredEmbeddingProviders",
            "clearEmbeddingProviders",
            "restoreEmbeddingProviders",
            "restoreRegisteredEmbeddingProviders",
        ];
        !forbidden.contains(&method_name)
    }

    pub fn register(
        &mut self,
        plugin_id: &str,
        provider_id: &str,
        declared_in_contracts: bool,
    ) -> Result<(), String> {
        if !declared_in_contracts {
            let msg = format!(
                "plugin must declare contracts.embeddingProviders for adapter: {}",
                provider_id
            );
            self.diagnostics.push(format!("{}: {}", plugin_id, msg));
            return Err(msg);
        }

        if self.core_providers.contains(provider_id) {
            let msg = format!(
                "embedding provider already registered: {} (owner: core)",
                provider_id
            );
            self.diagnostics.push(format!("{}: {}", plugin_id, msg));
            return Err(msg);
        }

        if let Some(existing) = self.providers.iter().find(|p| p.provider_id == provider_id) {
            let msg = format!(
                "embedding provider already registered: {} (owner: {})",
                provider_id, existing.plugin_id
            );
            self.diagnostics.push(format!("{}: {}", plugin_id, msg));
            return Err(msg);
        }

        self.providers.push(RegisteredEmbeddingProvider {
            provider_id: provider_id.to_string(),
            plugin_id: plugin_id.to_string(),
        });
        Ok(())
    }

    pub fn providers(&self) -> &[RegisteredEmbeddingProvider] {
        &self.providers
    }

    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }
}

// Wizard choice resolution and model picker (from wizard.choice-resolution and wizard.model-picker contracts)

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WizardAuthMethod {
    pub id: String,
    pub wizard: Option<String>,
    pub model_picker: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WizardProviderEntry {
    pub id: String,
    pub setup_wizard: Option<String>,
    pub model_picker: bool,
    pub auth: Vec<WizardAuthMethod>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedProviderChoice {
    pub provider_id: String,
    pub method_id: String,
    pub wizard_source: String, // "method" | "provider"
    pub wizard: Option<String>,
}

pub struct ProviderWizardChoiceResolver;

impl ProviderWizardChoiceResolver {
    pub fn resolve_choice(
        providers: &[WizardProviderEntry],
        choice: &str,
    ) -> Option<ResolvedProviderChoice> {
        let clean_choice = choice.strip_prefix("provider-plugin:").unwrap_or(choice);

        // Try exact colon separation: provider:method
        if let Some(idx) = clean_choice.find(':') {
            let p_id = &clean_choice[..idx];
            let m_id = &clean_choice[idx + 1..];
            if let Some(provider) = providers.iter().find(|p| p.id == p_id) {
                if let Some(method) = provider.auth.iter().find(|m| m.id == m_id) {
                    let (source, wizard) = if let Some(ref w) = method.wizard {
                        ("method", Some(w.clone()))
                    } else if let Some(ref w) = provider.setup_wizard {
                        ("provider", Some(w.clone()))
                    } else {
                        ("none", None)
                    };
                    return Some(ResolvedProviderChoice {
                        provider_id: provider.id.clone(),
                        method_id: method.id.clone(),
                        wizard_source: source.to_string(),
                        wizard,
                    });
                }
            }
        }

        // Try exact provider match (defaulting to first auth method)
        if let Some(provider) = providers.iter().find(|p| p.id == clean_choice) {
            if let Some(first_method) = provider.auth.first() {
                let (source, wizard) = if let Some(ref w) = first_method.wizard {
                    ("method", Some(w.clone()))
                } else if let Some(ref w) = provider.setup_wizard {
                    ("provider", Some(w.clone()))
                } else {
                    ("none", None)
                };
                return Some(ResolvedProviderChoice {
                    provider_id: provider.id.clone(),
                    method_id: first_method.id.clone(),
                    wizard_source: source.to_string(),
                    wizard,
                });
            }
        }

        // Try hyphen separation: provider-method
        for provider in providers {
            let prefix = format!("{}-", provider.id);
            if clean_choice.starts_with(&prefix) {
                let m_id = &clean_choice[prefix.len()..];
                if let Some(method) = provider.auth.iter().find(|m| m.id == m_id) {
                    let (source, wizard) = if let Some(ref w) = method.wizard {
                        ("method", Some(w.clone()))
                    } else if let Some(ref w) = provider.setup_wizard {
                        ("provider", Some(w.clone()))
                    } else {
                        ("none", None)
                    };
                    return Some(ResolvedProviderChoice {
                        provider_id: provider.id.clone(),
                        method_id: method.id.clone(),
                        wizard_source: source.to_string(),
                        wizard,
                    });
                }
            }
        }

        None
    }

    pub fn resolve_model_picker_entries(providers: &[WizardProviderEntry]) -> Vec<String> {
        let mut entries = Vec::new();
        for provider in providers {
            if provider.model_picker {
                entries.push(provider.id.clone());
            }
            for method in &provider.auth {
                if method.model_picker {
                    entries.push(format!("provider-plugin:{}:{}", provider.id, method.id));
                }
            }
        }
        entries.sort();
        entries
    }
}

