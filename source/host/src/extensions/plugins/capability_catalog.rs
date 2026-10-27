use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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

// Bundled Plugin Contract Snapshot (from src/plugins/contracts/inventory/bundled-capability-metadata.ts)

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BundledPluginContractSnapshot {
    pub plugin_id: String,
    pub cli_backend_ids: Vec<String>,
    pub provider_ids: Vec<String>,
    pub provider_env_vars: std::collections::HashMap<String, Vec<String>>,
    pub worker_provider_ids: Vec<String>,
    pub storage_provider_ids: Vec<String>,
    pub embedding_provider_ids: Vec<String>,
    pub speech_provider_ids: Vec<String>,
    pub realtime_transcription_provider_ids: Vec<String>,
    pub realtime_voice_provider_ids: Vec<String>,
    pub media_understanding_provider_ids: Vec<String>,
    pub transcript_source_provider_ids: Vec<String>,
    pub document_extractor_ids: Vec<String>,
    pub image_generation_provider_ids: Vec<String>,
    pub video_generation_provider_ids: Vec<String>,
    pub music_generation_provider_ids: Vec<String>,
    pub web_content_extractor_ids: Vec<String>,
    pub web_fetch_provider_ids: Vec<String>,
    pub web_search_provider_ids: Vec<String>,
    pub migration_provider_ids: Vec<String>,
    pub tool_names: Vec<String>,
}

impl BundledPluginContractSnapshot {
    pub fn has_capabilities(&self) -> bool {
        !self.cli_backend_ids.is_empty()
            || !self.provider_ids.is_empty()
            || !self.worker_provider_ids.is_empty()
            || !self.storage_provider_ids.is_empty()
            || !self.embedding_provider_ids.is_empty()
            || !self.speech_provider_ids.is_empty()
            || !self.realtime_transcription_provider_ids.is_empty()
            || !self.realtime_voice_provider_ids.is_empty()
            || !self.media_understanding_provider_ids.is_empty()
            || !self.transcript_source_provider_ids.is_empty()
            || !self.document_extractor_ids.is_empty()
            || !self.image_generation_provider_ids.is_empty()
            || !self.video_generation_provider_ids.is_empty()
            || !self.music_generation_provider_ids.is_empty()
            || !self.web_content_extractor_ids.is_empty()
            || !self.web_fetch_provider_ids.is_empty()
            || !self.web_search_provider_ids.is_empty()
            || !self.migration_provider_ids.is_empty()
            || !self.tool_names.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Bundled Channel Config Metadata, Compat & Dir Resolvers
// (from src/plugins/bundled-channel-config-metadata.ts, bundled-compat.ts, bundled-dir.ts)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundledChannelMetadata {
    pub channel_id: String,
    pub stream_mode: Option<String>,
    pub reply_to_mode: Option<String>,
    pub history_limit: Option<u32>,
    pub media_max_mb: Option<u32>,
    pub provider_skills: Vec<String>,
}

pub struct BundledChannelMetadataValidator;

impl BundledChannelMetadataValidator {
    pub fn validate(meta: &BundledChannelMetadata) -> Result<(), String> {
        if meta.channel_id.is_empty() {
            return Err("Channel ID cannot be empty".to_string());
        }
        if let Some(limit) = meta.history_limit {
            if limit == 0 || limit > 100_000 {
                return Err(format!("history_limit {} out of valid range (1..=100000)", limit));
            }
        }
        if let Some(max_mb) = meta.media_max_mb {
            if max_mb == 0 || max_mb > 2048 {
                return Err(format!("media_max_mb {} out of valid range (1..=2048)", max_mb));
            }
        }
        Ok(())
    }
}

pub struct BundledCompatResolver;

impl BundledCompatResolver {
    pub fn normalize_plugin_id(id: &str) -> String {
        let trimmed = id.trim();
        match trimmed {
            "legacy-telegram" => "telegram".to_string(),
            "legacy-discord" => "discord".to_string(),
            "legacy-slack" => "slack".to_string(),
            other => other.to_string(),
        }
    }
}

pub struct BundledDirResolver;

impl BundledDirResolver {
    pub const CANDIDATE_DIRS: &'static [&'static str] = &[
        "bundled/plugins",
        "dist/bundled/plugins",
        "extensions",
    ];

    pub fn resolve_relative_path(base_dir: &str, relative_path: &str) -> Result<String, String> {
        if relative_path.contains("..") {
            return Err(format!(
                "Path traversal detected in bundled directory path: '{}'",
                relative_path
            ));
        }
        let clean_base = base_dir.trim_end_matches('/');
        let clean_rel = relative_path.trim_start_matches('/');
        Ok(format!("{}/{}", clean_base, clean_rel))
    }
}

pub struct BundledPluginScanner;

impl BundledPluginScanner {
    pub const RUNTIME_SIDECAR_ARTIFACTS: &'static [&'static str] = &[
        "helper-api.js",
        "light-runtime-api.js",
        "qa-runner-api.js",
        "runtime-api.js",
        "runtime-setter-api.js",
        "thread-bindings-runtime.js",
    ];

    pub const PUBLIC_SURFACE_SOURCE_EXTENSIONS: &'static [&'static str] = &[
        ".ts", ".mts", ".js", ".mjs", ".cts", ".cjs",
    ];

    pub fn rewrite_bundled_plugin_entry_to_built_path(entry: &str) -> String {
        let stripped = entry.strip_prefix("./").or_else(|| entry.strip_prefix(".\\")).unwrap_or(entry);
        if let Some(dot_idx) = stripped.rfind('.') {
            format!("{}.js", &stripped[..dot_idx])
        } else {
            format!("{}.js", stripped)
        }
    }

    pub fn derive_bundled_plugin_id_hint(
        entry_path: &str,
        manifest_id: &str,
        package_name: Option<&str>,
        has_multiple_extensions: bool,
    ) -> String {
        if !has_multiple_extensions {
            return manifest_id.to_string();
        }
        let base = std::path::Path::new(entry_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(entry_path);

        match package_name.map(|s| s.trim()).filter(|s| !s.is_empty()) {
            Some(pkg) => {
                let unscoped = pkg.rsplit('/').next().unwrap_or(pkg);
                format!("{}/{}", unscoped, base)
            }
            None => format!("{}/{}", manifest_id, base),
        }
    }

    pub fn is_top_level_public_surface_source(name: &str) -> bool {
        let ext = match name.rfind('.') {
            Some(idx) => &name[idx..],
            None => return false,
        };
        if !Self::PUBLIC_SURFACE_SOURCE_EXTENSIONS.contains(&ext) {
            return false;
        }
        if name.starts_with('.') || name.starts_with("test-") || name.contains(".test-") {
            return false;
        }
        if name.ends_with(".d.ts") {
            return false;
        }
        if name.starts_with("config-api.") {
            return false;
        }
        if name.contains(".test.") || name.contains(".spec.") {
            return false;
        }
        true
    }

    pub fn collect_bundled_plugin_runtime_sidecar_artifacts(
        public_surfaces: &[String],
    ) -> Vec<String> {
        let sidecar_set: HashSet<&str> = Self::RUNTIME_SIDECAR_ARTIFACTS.iter().copied().collect();
        public_surfaces
            .iter()
            .filter(|artifact| sidecar_set.contains(artifact.as_str()))
            .cloned()
            .collect()
    }

    pub fn has_bundled_plugin_startup_manifest(
        plugin_id: &str,
        manifest_content: Option<&str>,
    ) -> bool {
        let content = match manifest_content {
            Some(c) => c,
            None => return false,
        };
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(content) {
            if let Some(id) = val.get("id").and_then(|v| v.as_str()) {
                return id == plugin_id;
            }
        }
        false
    }
}

pub struct BundledProviderCompatValidator;

impl BundledProviderCompatValidator {
    pub const BUNDLED_PROVIDER_COMPAT_CONTRACT_KEYS: &'static [&'static str] = &[
        "speechProviders",
        "externalAuthProviders",
        "embeddingProviders",
        "mediaUnderstandingProviders",
        "transcriptSourceProviders",
        "realtimeVoiceProviders",
        "realtimeTranscriptionProviders",
        "imageGenerationProviders",
        "videoGenerationProviders",
        "musicGenerationProviders",
        "webFetchProviders",
        "webSearchProviders",
        "workerProviders",
        "storageProviders",
        "usageProviders",
        "migrationProviders",
    ];

    pub fn is_bundled_provider_compat_contract(contract: &str) -> bool {
        Self::BUNDLED_PROVIDER_COMPAT_CONTRACT_KEYS.contains(&contract)
    }

    pub fn is_bundled_provider_compat_plugin(
        origin: &str,
        providers_len: usize,
        contract_counts: &HashMap<String, usize>,
    ) -> bool {
        if origin != "bundled" {
            return false;
        }
        if providers_len > 0 {
            return true;
        }
        Self::BUNDLED_PROVIDER_COMPAT_CONTRACT_KEYS.iter().any(|key| {
            contract_counts.get(*key).copied().unwrap_or(0) > 0
        })
    }
}

pub struct BundledSourceOverlayResolver;

impl BundledSourceOverlayResolver {
    pub fn parse_linux_mount_info_mount_points(mount_info: &str) -> HashSet<String> {
        let mut mount_points = HashSet::new();
        for line in mount_info.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let fields: Vec<&str> = trimmed.split_whitespace().collect();
            if fields.len() > 4 {
                mount_points.insert(fields[4].to_string());
            }
        }
        mount_points
    }

    pub fn source_overlays_disabled(env_val: Option<&str>) -> bool {
        match env_val {
            Some(v) => {
                let lower = v.trim().to_ascii_lowercase();
                lower == "1" || lower == "true"
            }
            None => false,
        }
    }

    pub fn is_bundled_source_overlay_path(
        source_path: &str,
        mount_points: &HashSet<String>,
    ) -> bool {
        mount_points.contains(source_path)
    }

    pub fn list_bundled_source_overlay_dirs(
        bundled_root: &str,
        legacy_root: &str,
        peer_dirs: &[String],
        mount_points: &HashSet<String>,
        env_disabled: bool,
    ) -> Vec<String> {
        if env_disabled || bundled_root.is_empty() || legacy_root.is_empty() {
            return Vec::new();
        }
        let legacy_root_mounted = Self::is_bundled_source_overlay_path(legacy_root, mount_points);
        let mut overlay_dirs = Vec::new();

        for dir in peer_dirs {
            let source_dir = format!("{}/{}", legacy_root.trim_end_matches('/'), dir);
            if legacy_root_mounted || Self::is_bundled_source_overlay_path(&source_dir, mount_points) {
                overlay_dirs.push(source_dir);
            }
        }
        overlay_dirs.sort();
        overlay_dirs
    }
}

pub struct BundledPluginNamingAndArtworkValidator;

impl BundledPluginNamingAndArtworkValidator {
    pub const PLUGIN_CATEGORY_SLUGS: &'static [&'static str] = &[
        "channels",
        "models",
        "agent-runtimes",
        "memory",
        "context",
        "voice",
        "web",
        "computer-use",
        "media",
        "security",
        "integrations",
        "developer-tools",
        "infrastructure",
        "documents-files",
        "inbox-collaboration",
        "productivity",
        "scheduling",
        "finance-payments",
        "sales-marketing",
        "data-analytics",
        "agent-orchestration",
        "research",
        "other",
    ];

    pub const ALLOWED_PACKAGE_SUFFIXES: &'static [&'static str] = &[
        "",
        "-provider",
        "-plugin",
        "-speech",
        "-sandbox",
        "-media-understanding",
    ];

    pub fn validate_bundled_plugin_categories(
        categories: Option<&[String]>,
    ) -> Result<String, String> {
        let cats = categories.ok_or_else(|| "expected exactly one purpose category".to_string())?;
        if cats.len() != 1 {
            return Err("expected exactly one purpose category".to_string());
        }
        let category = &cats[0];
        if !Self::PLUGIN_CATEGORY_SLUGS.contains(&category.as_str()) {
            return Err(format!("contains unknown category '{}'", category));
        }
        Ok(category.clone())
    }

    pub fn validate_bundled_plugin_icon_png(bytes: &[u8]) -> Result<(), String> {
        const PNG_SIG: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
        if bytes.len() < 24 {
            return Err("File too short for valid PNG header".to_string());
        }
        if bytes[0..8] != PNG_SIG {
            return Err("Invalid PNG signature".to_string());
        }
        let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        if width != 512 || height != 512 {
            return Err(format!(
                "Icon must be 512x512 pixels, got {}x{}",
                width, height
            ));
        }
        Ok(())
    }

    pub fn validate_bundled_plugin_naming(
        dir_name: &str,
        manifest_id: &str,
        package_name: &str,
    ) -> Result<(), String> {
        let expected_manifest_id = if dir_name == "kimi-coding" {
            "kimi"
        } else {
            dir_name
        };

        if manifest_id != expected_manifest_id {
            return Err(format!(
                "Manifest id '{}' does not match expected '{}' for directory '{}'",
                manifest_id, expected_manifest_id, dir_name
            ));
        }

        if !package_name.starts_with("@openclaw/") {
            return Err(format!(
                "Package name '{}' must start with '@openclaw/'",
                package_name
            ));
        }

        let pkg_suffix = &package_name["@openclaw/".len()..];
        let matches_suffix = Self::ALLOWED_PACKAGE_SUFFIXES.iter().any(|suffix| {
            let expected_pkg = format!("{}{}", expected_manifest_id, suffix);
            pkg_suffix == expected_pkg
        });

        if !matches_suffix {
            return Err(format!(
                "Package name '{}' does not conform to expected prefix '@openclaw/{}' with allowed suffixes",
                package_name, expected_manifest_id
            ));
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Bundled Plugin Sources & Resolvers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundledPluginSource {
    pub plugin_id: String,
    pub local_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm_spec: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_schema: Option<serde_json::Value>,
    pub requires_config: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BundledPluginLookup<'a> {
    PluginId(&'a str),
    NpmSpec(&'a str),
    LocalPath(&'a str),
}

pub struct BundledPluginSourceResolver;
impl BundledPluginSourceResolver {
    pub fn schema_has_required_fields(schema: &serde_json::Value) -> bool {
        if let Some(obj) = schema.as_object() {
            if let Some(req) = obj.get("required").and_then(|v| v.as_array()) {
                return req.iter().any(|item| item.is_string());
            }
        }
        false
    }

    pub fn find_in_map<'a>(
        bundled: &'a HashMap<String, BundledPluginSource>,
        lookup: BundledPluginLookup<'_>,
    ) -> Option<&'a BundledPluginSource> {
        match lookup {
            BundledPluginLookup::PluginId(id) => {
                let id = id.trim();
                if id.is_empty() {
                    None
                } else {
                    bundled.get(id)
                }
            }
            BundledPluginLookup::NpmSpec(spec) => {
                let spec = spec.trim();
                if spec.is_empty() {
                    None
                } else {
                    bundled.values().find(|s| s.npm_spec.as_deref() == Some(spec))
                }
            }
            BundledPluginLookup::LocalPath(p) => {
                let p = p.trim();
                if p.is_empty() {
                    None
                } else {
                    let canonical_target = std::path::Path::new(p);
                    bundled.values().find(|s| {
                        let source_p = std::path::Path::new(&s.local_path);
                        source_p == canonical_target
                    })
                }
            }
        }
    }

    pub fn resolve_install_command_hint(source: Option<&BundledPluginSource>) -> Option<String> {
        source.map(|s| format!("openclaw plugins install {}", s.local_path))
    }
}

// ---------------------------------------------------------------------------
// Candidate Install Owner & Immutable Identity
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateInstallOwner {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_owner: Option<String>,
    #[serde(default)]
    pub ambiguous: bool,
}

pub struct CandidateInstallOwnerResolver;
impl CandidateInstallOwnerResolver {
    pub fn resolve_immutable_install_identity(
        origin: &str,
        install_owner: Option<&str>,
        source: Option<&str>,
        install_path: Option<&str>,
        spec: Option<&str>,
        resolved_spec: Option<&str>,
        integrity: Option<&str>,
        installed_at: Option<&str>,
    ) -> Option<String> {
        if origin != "global" || source != Some("npm") {
            return None;
        }
        let integrity = integrity?;
        let owner = install_owner?;
        let effective_spec = resolved_spec.or(spec).unwrap_or("");
        let arr = serde_json::json!([
            owner,
            install_path.unwrap_or(""),
            effective_spec,
            integrity,
            installed_at.unwrap_or("")
        ]);
        Some(arr.to_string())
    }
}

// ---------------------------------------------------------------------------
// Capability Catalog Families & Descriptors
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityCatalogFamily {
    SpeechProviders,
    RealtimeTranscriptionProviders,
    RealtimeVoiceProviders,
}

impl CapabilityCatalogFamily {
    pub const ALL: &'static [CapabilityCatalogFamily] = &[
        CapabilityCatalogFamily::SpeechProviders,
        CapabilityCatalogFamily::RealtimeTranscriptionProviders,
        CapabilityCatalogFamily::RealtimeVoiceProviders,
    ];

    pub fn family_name(&self) -> &'static str {
        match self {
            CapabilityCatalogFamily::SpeechProviders => "speechProviders",
            CapabilityCatalogFamily::RealtimeTranscriptionProviders => {
                "realtimeTranscriptionProviders"
            }
            CapabilityCatalogFamily::RealtimeVoiceProviders => "realtimeVoiceProviders",
        }
    }

    pub fn required_method(&self) -> &'static str {
        match self {
            CapabilityCatalogFamily::SpeechProviders => "synthesize",
            CapabilityCatalogFamily::RealtimeTranscriptionProviders => "createSession",
            CapabilityCatalogFamily::RealtimeVoiceProviders => "createBridge",
        }
    }

    pub fn from_family_name(name: &str) -> Option<CapabilityCatalogFamily> {
        match name {
            "speechProviders" => Some(CapabilityCatalogFamily::SpeechProviders),
            "realtimeTranscriptionProviders" => {
                Some(CapabilityCatalogFamily::RealtimeTranscriptionProviders)
            }
            "realtimeVoiceProviders" => Some(CapabilityCatalogFamily::RealtimeVoiceProviders),
            _ => None,
        }
    }
}

pub struct CapabilityCatalogValidator;
impl CapabilityCatalogValidator {
    pub fn validate_catalog_shape(catalog_json: &serde_json::Value) -> Result<(), String> {
        let obj = catalog_json.as_object().ok_or_else(|| {
            "default export must synchronously provide a capability descriptor collection"
                .to_string()
        })?;

        for (family_str, providers_val) in obj {
            let family = CapabilityCatalogFamily::from_family_name(family_str)
                .ok_or_else(|| format!("unknown capability catalog family: {}", family_str))?;

            let providers = providers_val.as_array().ok_or_else(|| {
                format!("{} must contain complete provider descriptors", family_str)
            })?;

            let req_method = family.required_method();
            for p in providers {
                let p_obj = p.as_object().ok_or_else(|| {
                    format!("{} must contain complete provider descriptors", family_str)
                })?;

                let id = p_obj
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim();
                if id.is_empty() {
                    return Err(format!(
                        "{} must contain complete provider descriptors",
                        family_str
                    ));
                }

                if p_obj.get("label").and_then(|v| v.as_str()).is_none() {
                    return Err(format!(
                        "{} must contain complete provider descriptors",
                        family_str
                    ));
                }

                if !p_obj.contains_key("isConfigured") || !p_obj.contains_key(req_method) {
                    return Err(format!(
                        "{} must contain complete provider descriptors",
                        family_str
                    ));
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Plugin Accepted Declared Surface & Capability Consent
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginAcceptedDeclaredSurface {
    #[serde(default)]
    pub channels: Vec<String>,
    #[serde(default)]
    pub providers: Vec<String>,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub contracts: Vec<String>,
    #[serde(default)]
    pub hooks: Vec<String>,
    #[serde(default, rename = "mcpServers")]
    pub mcp_servers: Vec<String>,
    #[serde(default, rename = "cliCommands")]
    pub cli_commands: Vec<String>,
    #[serde(default, rename = "cliBackends")]
    pub cli_backends: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default, rename = "dangerousConfigFlags")]
    pub dangerous_config_flags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginCapabilityConsentReview {
    pub plugin_id: String,
    pub review_token: String,
    pub declared: PluginAcceptedDeclaredSurface,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_declared: Option<PluginAcceptedDeclaredSurface>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub widened: Option<HashMap<String, Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepted_at: Option<String>,
}

pub struct CapabilityConsentManager;
impl CapabilityConsentManager {
    pub fn compute_declared_surface_hash(surface: &PluginAcceptedDeclaredSurface) -> String {
        let mut hasher = Sha256::new();
        let canonical_json = serde_json::to_string(surface).unwrap_or_default();
        hasher.update(canonical_json.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn resolve_plugin_install_record_integrity(
        integrity: Option<&str>,
        npm_integrity: Option<&str>,
        clawpack_sha256: Option<&str>,
        git_commit: Option<&str>,
    ) -> Option<(String, &'static str)> {
        if let Some(i) = integrity.filter(|s| !s.is_empty()) {
            return Some((i.to_string(), "ssri"));
        }
        if let Some(i) = npm_integrity.filter(|s| !s.is_empty()) {
            return Some((i.to_string(), "ssri"));
        }
        if let Some(i) = clawpack_sha256.filter(|s| !s.is_empty()) {
            return Some((i.to_string(), "sha256"));
        }
        if let Some(i) = git_commit.filter(|s| !s.is_empty()) {
            return Some((i.to_string(), "git-commit"));
        }
        None
    }

    pub fn resolve_accepted_surface_current(
        record_accepted_surface: Option<&PluginAcceptedDeclaredSurface>,
        record_accepted_surface_hash: Option<&str>,
        record_integrity: Option<&str>,
        record_accepted_surface_integrity: Option<&str>,
        current_surface: &PluginAcceptedDeclaredSurface,
    ) -> bool {
        let accepted = match record_accepted_surface {
            Some(a) => a,
            None => return false,
        };
        if accepted != current_surface {
            return false;
        }
        let expected_hash = Self::compute_declared_surface_hash(current_surface);
        if record_accepted_surface_hash != Some(expected_hash.as_str()) {
            return false;
        }
        if let Some(accepted_integrity) = record_accepted_surface_integrity {
            match record_integrity {
                Some(cur_integrity) if cur_integrity == accepted_integrity => {}
                _ => return false,
            }
        }
        true
    }

    pub fn diff_declared_surface_widening(
        previous: &PluginAcceptedDeclaredSurface,
        next: &PluginAcceptedDeclaredSurface,
    ) -> (HashMap<String, Vec<String>>, bool) {
        let mut widened = HashMap::new();

        fn diff_field(prev: &[String], nxt: &[String]) -> Vec<String> {
            let prev_set: HashSet<&str> = prev.iter().map(|s| s.as_str()).collect();
            let mut added: Vec<String> = nxt
                .iter()
                .filter(|s| !prev_set.contains(s.as_str()))
                .cloned()
                .collect();
            added.sort();
            added
        }

        let tools = diff_field(&previous.tools, &next.tools);
        if !tools.is_empty() {
            widened.insert("tools".to_string(), tools);
        }
        let channels = diff_field(&previous.channels, &next.channels);
        if !channels.is_empty() {
            widened.insert("channels".to_string(), channels);
        }
        let providers = diff_field(&previous.providers, &next.providers);
        if !providers.is_empty() {
            widened.insert("providers".to_string(), providers);
        }
        let contracts = diff_field(&previous.contracts, &next.contracts);
        if !contracts.is_empty() {
            widened.insert("contracts".to_string(), contracts);
        }
        let hooks = diff_field(&previous.hooks, &next.hooks);
        if !hooks.is_empty() {
            widened.insert("hooks".to_string(), hooks);
        }
        let mcp = diff_field(&previous.mcp_servers, &next.mcp_servers);
        if !mcp.is_empty() {
            widened.insert("mcpServers".to_string(), mcp);
        }
        let cli_cmds = diff_field(&previous.cli_commands, &next.cli_commands);
        if !cli_cmds.is_empty() {
            widened.insert("cliCommands".to_string(), cli_cmds);
        }
        let cli_backends = diff_field(&previous.cli_backends, &next.cli_backends);
        if !cli_backends.is_empty() {
            widened.insert("cliBackends".to_string(), cli_backends);
        }
        let skills = diff_field(&previous.skills, &next.skills);
        if !skills.is_empty() {
            widened.insert("skills".to_string(), skills);
        }
        let flags = diff_field(&previous.dangerous_config_flags, &next.dangerous_config_flags);
        if !flags.is_empty() {
            widened.insert("dangerousConfigFlags".to_string(), flags);
        }

        let has_widening = !widened.is_empty();
        (widened, has_widening)
    }

    pub fn redact_source_spec(spec: &str) -> String {
        if let Some(stripped) = spec.strip_prefix("git:") {
            if let Ok(mut parsed) = url::Url::parse(stripped) {
                if !parsed.username().is_empty() || parsed.password().is_some() {
                    let _ = parsed.set_username("***");
                    let _ = parsed.set_password(Some("***"));
                }
                let pairs: Vec<(String, String)> = parsed
                    .query_pairs()
                    .map(|(k, v)| {
                        if k == "token" || k == "key" || k == "secret" {
                            (k.to_string(), "***".to_string())
                        } else {
                            (k.to_string(), v.to_string())
                        }
                    })
                    .collect();
                parsed.query_pairs_mut().clear().extend_pairs(
                    pairs.iter().map(|(k, v)| (k.as_str(), v.as_str())),
                );
                return format!("git:{}", parsed);
            }
        }
        spec.to_string()
    }
}

// ---------------------------------------------------------------------------
// Capability Lease
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct PluginRuntimeCapabilityLease {
    owner: String,
    inner: Arc<Mutex<CapabilityLeaseState>>,
}

struct CapabilityLeaseState {
    active: bool,
    cleanups: Vec<Box<dyn FnOnce() + Send + 'static>>,
}

impl PluginRuntimeCapabilityLease {
    pub fn new(owner: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            inner: Arc::new(Mutex::new(CapabilityLeaseState {
                active: true,
                cleanups: Vec::new(),
            })),
        }
    }

    pub fn is_active(&self) -> bool {
        self.inner.lock().unwrap().active
    }

    pub fn assert_active(&self, capability: &str) -> Result<(), String> {
        if !self.is_active() {
            Err(format!("{} {} is no longer active", self.owner, capability))
        } else {
            Ok(())
        }
    }

    pub fn retain<F>(&self, cleanup: F) -> Result<(), String>
    where
        F: FnOnce() + Send + 'static,
    {
        let mut state = self.inner.lock().unwrap();
        if !state.active {
            cleanup();
            return Err(format!("{} capability lease is no longer active", self.owner));
        }
        state.cleanups.push(Box::new(cleanup));
        Ok(())
    }

    pub fn revoke(&self) {
        let cleanups_to_run = {
            let mut state = self.inner.lock().unwrap();
            if !state.active {
                return;
            }
            state.active = false;
            std::mem::take(&mut state.cleanups)
        };
        for cleanup in cleanups_to_run {
            cleanup();
        }
    }
}

// ---------------------------------------------------------------------------
// Operator Grants & Summary Builder
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginHookGrant {
    pub effective: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configured: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginHookGrants {
    pub allow_prompt_injection: PluginHookGrant,
    pub allow_conversation_access: PluginHookGrant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginOperatorGrants {
    pub hooks: PluginHookGrants,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subagent: Option<serde_json::Value>,
}

pub struct CapabilitySummaryBuilder;
impl CapabilitySummaryBuilder {
    pub fn merge_declared_surfaces(
        surfaces: &[PluginAcceptedDeclaredSurface],
    ) -> PluginAcceptedDeclaredSurface {
        let mut merged = PluginAcceptedDeclaredSurface::default();
        let mut channels = HashSet::new();
        let mut providers = HashSet::new();
        let mut tools = HashSet::new();
        let mut contracts = HashSet::new();
        let mut hooks = HashSet::new();
        let mut mcp = HashSet::new();
        let mut cli_cmds = HashSet::new();
        let mut cli_backends = HashSet::new();
        let mut skills = HashSet::new();
        let mut flags = HashSet::new();

        for s in surfaces {
            channels.extend(s.channels.iter().cloned());
            providers.extend(s.providers.iter().cloned());
            tools.extend(s.tools.iter().cloned());
            contracts.extend(s.contracts.iter().cloned());
            hooks.extend(s.hooks.iter().cloned());
            mcp.extend(s.mcp_servers.iter().cloned());
            cli_cmds.extend(s.cli_commands.iter().cloned());
            cli_backends.extend(s.cli_backends.iter().cloned());
            skills.extend(s.skills.iter().cloned());
            flags.extend(s.dangerous_config_flags.iter().cloned());
        }

        fn to_sorted(set: HashSet<String>) -> Vec<String> {
            let mut v: Vec<String> = set.into_iter().collect();
            v.sort();
            v
        }

        merged.channels = to_sorted(channels);
        merged.providers = to_sorted(providers);
        merged.tools = to_sorted(tools);
        merged.contracts = to_sorted(contracts);
        merged.hooks = to_sorted(hooks);
        merged.mcp_servers = to_sorted(mcp);
        merged.cli_commands = to_sorted(cli_cmds);
        merged.cli_backends = to_sorted(cli_backends);
        merged.skills = to_sorted(skills);
        merged.dangerous_config_flags = to_sorted(flags);
        merged
    }

    pub fn build_operator_grants(
        origin: &PluginOrigin,
        allow_prompt_injection: Option<bool>,
        allow_conversation_access: Option<bool>,
    ) -> PluginOperatorGrants {
        let effective_prompt_injection = allow_prompt_injection.unwrap_or(false);
        let effective_conv_access = match origin {
            PluginOrigin::Bundled => allow_conversation_access.unwrap_or(true),
            _ => allow_conversation_access.unwrap_or(false),
        };

        PluginOperatorGrants {
            hooks: PluginHookGrants {
                allow_prompt_injection: PluginHookGrant {
                    effective: effective_prompt_injection,
                    configured: allow_prompt_injection,
                },
                allow_conversation_access: PluginHookGrant {
                    effective: effective_conv_access,
                    configured: allow_conversation_access,
                },
            },
            llm: None,
            subagent: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Captured Plugin Registration
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct CapturedPluginRegistration {
    pub providers: Vec<String>,
    pub cli_commands: Vec<String>,
    pub speech_providers: Vec<String>,
    pub realtime_transcription_providers: Vec<String>,
    pub realtime_voice_providers: Vec<String>,
    pub media_understanding_providers: Vec<String>,
    pub transcript_source_providers: Vec<String>,
    pub image_generation_providers: Vec<String>,
    pub video_generation_providers: Vec<String>,
    pub music_generation_providers: Vec<String>,
    pub web_fetch_providers: Vec<String>,
    pub web_search_providers: Vec<String>,
    pub worker_providers: Vec<String>,
    pub storage_providers: Vec<String>,
    pub migration_providers: Vec<String>,
    pub embedding_providers: Vec<String>,
    pub tools: Vec<String>,
    pub session_scheduler_jobs: Vec<String>,
    pub session_actions: Vec<String>,
}

impl CapturedPluginRegistration {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_capability_provider(
        &mut self,
        family: &str,
        provider_id: &str,
    ) -> Result<(), String> {
        match family {
            "speechProviders" => self.speech_providers.push(provider_id.to_string()),
            "realtimeTranscriptionProviders" => self
                .realtime_transcription_providers
                .push(provider_id.to_string()),
            "realtimeVoiceProviders" => self.realtime_voice_providers.push(provider_id.to_string()),
            "mediaUnderstandingProviders" => {
                self.media_understanding_providers.push(provider_id.to_string())
            }
            "transcriptSourceProviders" => self
                .transcript_source_providers
                .push(provider_id.to_string()),
            "imageGenerationProviders" => self
                .image_generation_providers
                .push(provider_id.to_string()),
            "videoGenerationProviders" => self
                .video_generation_providers
                .push(provider_id.to_string()),
            "musicGenerationProviders" => self
                .music_generation_providers
                .push(provider_id.to_string()),
            "webFetchProviders" => self.web_fetch_providers.push(provider_id.to_string()),
            "webSearchProviders" => self.web_search_providers.push(provider_id.to_string()),
            "embeddingProviders" => self.embedding_providers.push(provider_id.to_string()),
            _ => return Err(format!("Unknown capability family: {}", family)),
        }
        Ok(())
    }

    pub fn total_registered_capabilities(&self) -> usize {
        self.speech_providers.len()
            + self.realtime_transcription_providers.len()
            + self.realtime_voice_providers.len()
            + self.media_understanding_providers.len()
            + self.transcript_source_providers.len()
            + self.image_generation_providers.len()
            + self.video_generation_providers.len()
            + self.music_generation_providers.len()
            + self.web_fetch_providers.len()
            + self.web_search_providers.len()
            + self.embedding_providers.len()
    }
}

// ---------------------------------------------------------------------------
// Capability Acquisition Scope & Lifetime
// ---------------------------------------------------------------------------

pub struct PluginCapabilityAcquisitionScope {
    lease: PluginRuntimeCapabilityLease,
    generation: u64,
    current_generation: Arc<std::sync::atomic::AtomicU64>,
}

impl PluginCapabilityAcquisitionScope {
    pub fn new(owner: &str, current_generation: Arc<std::sync::atomic::AtomicU64>) -> Self {
        let generation_id = current_generation.load(std::sync::atomic::Ordering::SeqCst);
        Self {
            lease: PluginRuntimeCapabilityLease::new(owner),
            generation: generation_id,
            current_generation,
        }
    }

    pub fn assert_open(&self) -> Result<(), String> {
        self.lease.assert_active("acquisition")?;
        let cur = self
            .current_generation
            .load(std::sync::atomic::Ordering::SeqCst);
        if cur != self.generation {
            return Err("The provider setup changed while preparing this request. Retry with the current provider setup.".to_string());
        }
        Ok(())
    }

    pub fn release(&self) {
        self.lease.revoke();
    }
}

// ---------------------------------------------------------------------------
// Catalog Discovery Codec & Identity Join
// ---------------------------------------------------------------------------

pub struct CatalogDiscoveryCodec;
impl CatalogDiscoveryCodec {
    pub const DISCOVERY_ID_PREFIX: &'static str = "ch_";
    pub const LOCAL_DISCOVERY_ID_PREFIX: &'static str = "local_";

    pub fn encode_discovery_id(prefix: &str, identity: &str) -> Result<String, String> {
        let normalized = identity.trim();
        if normalized.is_empty() {
            return Err("Cannot encode an empty plugin discovery identity.".to_string());
        }
        let encoded = URL_SAFE_NO_PAD.encode(normalized.as_bytes());
        Ok(format!("{}{}", prefix, encoded))
    }

    pub fn encode_plugin_discovery_id(package_name: &str) -> Result<String, String> {
        Self::encode_discovery_id(Self::DISCOVERY_ID_PREFIX, package_name)
    }

    pub fn encode_local_plugin_discovery_id(identity: &str) -> Result<String, String> {
        Self::encode_discovery_id(Self::LOCAL_DISCOVERY_ID_PREFIX, identity)
    }

    pub fn plugin_catalog_id(id: &str, clawhub_package: Option<&str>) -> Result<String, String> {
        if let Some(pkg) = clawhub_package {
            Self::encode_plugin_discovery_id(pkg)
        } else {
            Self::encode_local_plugin_discovery_id(id)
        }
    }

    pub fn resolve_plugin_discovery_identity(id: &str) -> Option<(&'static str, String)> {
        let (prefix, origin) = if id.starts_with(Self::DISCOVERY_ID_PREFIX) {
            (Self::DISCOVERY_ID_PREFIX, "clawhub")
        } else if id.starts_with(Self::LOCAL_DISCOVERY_ID_PREFIX) {
            (Self::LOCAL_DISCOVERY_ID_PREFIX, "local")
        } else {
            return None;
        };

        let payload = &id[prefix.len()..];
        if payload.is_empty() {
            return None;
        }

        if !payload
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return None;
        }

        let bytes = URL_SAFE_NO_PAD.decode(payload.as_bytes()).ok()?;
        let identity = String::from_utf8(bytes).ok()?;
        let re_encoded = Self::encode_discovery_id(prefix, &identity).ok()?;
        if re_encoded == id {
            Some((origin, identity))
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Catalog Icon Registry
// ---------------------------------------------------------------------------

pub struct CatalogIconRegistry {
    urls: Mutex<std::collections::VecDeque<String>>,
    lookup: Mutex<HashSet<String>>,
    capacity: usize,
}

impl CatalogIconRegistry {
    pub const DEFAULT_MAX_CATALOG_ICON_URLS: usize = 1024;

    pub fn new(capacity: usize) -> Self {
        Self {
            urls: Mutex::new(std::collections::VecDeque::new()),
            lookup: Mutex::new(HashSet::new()),
            capacity,
        }
    }

    pub fn normalize_catalog_icon_url(value: &str) -> Option<String> {
        if value.is_empty() || value.len() > 2048 {
            return None;
        }
        let parsed = url::Url::parse(value).ok()?;
        if parsed.scheme() != "https" {
            return None;
        }
        if parsed.host_str().is_none() {
            return None;
        }
        if !parsed.username().is_empty()
            || parsed.password().is_some()
            || parsed.fragment().is_some()
        {
            return None;
        }
        Some(parsed.to_string())
    }

    pub fn register_icon_urls<'a, I>(&self, values: I)
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut urls = self.urls.lock().unwrap();
        let mut lookup = self.lookup.lock().unwrap();
        for val in values {
            if let Some(norm) = Self::normalize_catalog_icon_url(val) {
                if !lookup.contains(&norm) {
                    if urls.len() >= self.capacity {
                        if let Some(oldest) = urls.pop_front() {
                            lookup.remove(&oldest);
                        }
                    }
                    urls.push_back(norm.clone());
                    lookup.insert(norm);
                }
            }
        }
    }

    pub fn resolve_icon_url(&self, value: &str) -> Option<String> {
        let norm = Self::normalize_catalog_icon_url(value)?;
        let lookup = self.lookup.lock().unwrap();
        if lookup.contains(&norm) {
            Some(norm)
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Catalog Search Ranking & Deduplication
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogPackageSearchResult {
    pub name: String,
    pub family: String,
    pub score: f64,
}

pub struct CatalogSearchRanking;
impl CatalogSearchRanking {
    pub const DEFAULT_PLUGIN_SEARCH_LIMIT: usize = 20;
    pub const MAX_PLUGIN_SEARCH_LIMIT: usize = 100;
    pub const ALLOWED_FAMILIES: &'static [&'static str] = &["code-plugin", "bundle-plugin"];

    pub fn clamp_limit(limit: Option<usize>) -> usize {
        match limit {
            None | Some(0) => Self::DEFAULT_PLUGIN_SEARCH_LIMIT,
            Some(l) => l.clamp(1, Self::MAX_PLUGIN_SEARCH_LIMIT),
        }
    }

    pub fn deduplicate_and_rank(
        results: Vec<CatalogPackageSearchResult>,
        limit: Option<usize>,
    ) -> Vec<CatalogPackageSearchResult> {
        let effective_limit = Self::clamp_limit(limit);
        let mut by_name: HashMap<String, CatalogPackageSearchResult> = HashMap::new();

        for entry in results {
            if !Self::ALLOWED_FAMILIES.contains(&entry.family.as_str()) {
                continue;
            }
            match by_name.get(&entry.name) {
                Some(existing) if existing.score >= entry.score => {}
                _ => {
                    by_name.insert(entry.name.clone(), entry);
                }
            }
        }

        let mut sorted: Vec<CatalogPackageSearchResult> = by_name.into_values().collect();
        sorted.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.name.cmp(&b.name))
        });

        sorted.truncate(effective_limit);
        sorted
    }
}

// ---------------------------------------------------------------------------
// Channel Catalog Resolver
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelCatalogCandidate {
    pub id_hint: Option<String>,
    pub bundled_manifest_id: Option<String>,
    pub package_plugin_id: Option<String>,
    pub package_name: Option<String>,
    pub channel_id: Option<String>,
    pub channel_name: Option<String>,
    pub origin: PluginOrigin,
    pub root_dir: String,
    pub is_trusted_official: bool,
}

pub struct ChannelCatalogResolver;
impl ChannelCatalogResolver {
    pub fn resolve_channel_catalog_entries(
        candidates: &[ChannelCatalogCandidate],
        origin_filter: Option<&PluginOrigin>,
    ) -> Vec<ChannelCatalogEntry> {
        candidates
            .iter()
            .filter_map(|c| {
                if let Some(req_origin) = origin_filter {
                    if &c.origin != req_origin {
                        return None;
                    }
                }
                let ch_id = c.channel_id.as_deref()?.trim();
                if ch_id.is_empty() {
                    return None;
                }
                let plugin_id = c
                    .bundled_manifest_id
                    .as_deref()
                    .or(c.package_plugin_id.as_deref())
                    .or(c.id_hint.as_deref())?
                    .trim();
                if plugin_id.is_empty() {
                    return None;
                }

                Some(ChannelCatalogEntry {
                    plugin_id: plugin_id.to_string(),
                    origin: c.origin.clone(),
                    package_name: c.package_name.clone(),
                    channel_name: ch_id.to_string(),
                    trusted_official_install: c.is_trusted_official,
                })
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Channel Plugin Metadata Validation & Normalization
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelPluginMeta {
    pub id: String,
    pub label: String,
    pub selection_label: String,
    pub docs_path: String,
    pub blurb: String,
    pub aliases: Vec<String>,
}

impl Default for ChannelPluginMeta {
    fn default() -> Self {
        Self {
            id: String::new(),
            label: String::new(),
            selection_label: String::new(),
            docs_path: String::new(),
            blurb: String::new(),
            aliases: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelPluginConfigHandlers {
    pub has_list_accounts: bool,
    pub has_resolve_account: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelPluginDefinition {
    pub id: String,
    pub meta: ChannelPluginMeta,
    pub config: Option<ChannelPluginConfigHandlers>,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelValidationDiagnostic {
    pub level: String, // "warn" or "error"
    pub plugin_id: String,
    pub message: String,
}

pub struct ChannelPluginValidator;
impl ChannelPluginValidator {
    pub fn validate_and_normalize(
        registered_id: &str,
        mut meta: ChannelPluginMeta,
        config: Option<&ChannelPluginConfigHandlers>,
        plugin_id: &str,
    ) -> Result<(ChannelPluginDefinition, Vec<ChannelValidationDiagnostic>), ChannelValidationDiagnostic> {
        let mut diagnostics = Vec::new();

        // 1. Check required config helpers
        match config {
            Some(c) if c.has_list_accounts && c.has_resolve_account => {}
            _ => {
                return Err(ChannelValidationDiagnostic {
                    level: "error".to_string(),
                    plugin_id: plugin_id.to_string(),
                    message: format!(
                        "channel \"{}\" registration missing required config helpers",
                        registered_id
                    ),
                });
            }
        }

        // 2. Mismatched meta.id
        if !meta.id.is_empty() && meta.id != registered_id {
            diagnostics.push(ChannelValidationDiagnostic {
                level: "warn".to_string(),
                plugin_id: plugin_id.to_string(),
                message: format!(
                    "channel \"{}\" meta.id mismatch (\"{}\"); using registered channel id",
                    registered_id, meta.id
                ),
            });
            meta.id = registered_id.to_string();
        } else if meta.id.is_empty() {
            meta.id = registered_id.to_string();
        }

        // 3. Incomplete metadata filling
        let mut missing_fields = Vec::new();
        if meta.label.is_empty() {
            meta.label = registered_id.to_string();
            missing_fields.push("label");
        }
        if meta.selection_label.is_empty() {
            meta.selection_label = registered_id.to_string();
            missing_fields.push("selectionLabel");
        }
        if meta.docs_path.is_empty() {
            meta.docs_path = format!("/channels/{}", registered_id);
            missing_fields.push("docsPath");
        }
        if meta.blurb.is_empty() {
            if registered_id == "telegram" {
                meta.blurb = "Telegram bot channel".to_string();
            }
            missing_fields.push("blurb");
        }
        if !missing_fields.is_empty() {
            diagnostics.push(ChannelValidationDiagnostic {
                level: "warn".to_string(),
                plugin_id: plugin_id.to_string(),
                message: format!(
                    "channel \"{}\" registered incomplete metadata; filled missing {}",
                    registered_id,
                    missing_fields.join(", ")
                ),
            });
        }

        // 4. Fill official external channel aliases if omitted
        if registered_id == "openclaw-weixin" && meta.aliases.is_empty() {
            meta.aliases = vec!["weixin".to_string(), "wechat".to_string(), "微信".to_string()];
        }

        Ok((
            ChannelPluginDefinition {
                id: registered_id.to_string(),
                meta,
                config: config.cloned(),
                capabilities: vec!["direct".to_string()],
            },
            diagnostics,
        ))
    }
}

// ---------------------------------------------------------------------------
// Channel Registry State & Adoption
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivePluginChannelRegistration {
    pub plugin_id: String,
    pub channel_id: String,
    pub origin: PluginOrigin,
    pub source_digest: String,
    pub accepting_calls: bool,
    pub borrowed_runtime_record: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RuntimeChannelSource {
    pub registry_id: String,
    pub is_current: bool,
}

pub struct ChannelRegistryAdoption;
impl ChannelRegistryAdoption {
    pub fn adopt_runtime_channel_registrations(
        target_registry_id: &str,
        target_channels: &[ActivePluginChannelRegistration],
        source: Option<&RuntimeChannelSource>,
        target_env_digest: &str,
        donor_env_digest: &str,
        target_gateway_resolver_id: &str,
        donor_gateway_resolver_id: &str,
        target_retired: bool,
    ) -> Result<Vec<ActivePluginChannelRegistration>, String> {
        let Some(src) = source else {
            return Ok(target_channels.to_vec());
        };
        if target_registry_id == src.registry_id || target_channels.is_empty() {
            return Ok(target_channels.to_vec());
        }
        if !src.is_current {
            return Err("Channel runtime owner changed during prepared registry admission".to_string());
        }
        if target_env_digest != donor_env_digest
            || target_gateway_resolver_id != donor_gateway_resolver_id
            || target_retired
        {
            return Ok(target_channels.to_vec());
        }

        let adopted = target_channels
            .iter()
            .map(|ch| {
                if ch.accepting_calls {
                    let mut cloned = ch.clone();
                    cloned.borrowed_runtime_record = Some(format!("{}:borrowed", ch.plugin_id));
                    cloned
                } else {
                    ch.clone()
                }
            })
            .collect();

        Ok(adopted)
    }
}

// ---------------------------------------------------------------------------
// Channel Startup Policy
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedPluginConfigEntry {
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NormalizedPluginsConfig {
    pub enabled: bool,
    pub allow: Vec<String>,
    pub deny: Vec<String>,
    pub entries: HashMap<String, NormalizedPluginConfigEntry>,
}

pub struct ChannelStartupPolicy;
impl ChannelStartupPolicy {
    pub fn can_start_configured_channel_plugin(
        plugin_id: &str,
        origin: &PluginOrigin,
        channel_ids: &[String],
        plugins_config: &NormalizedPluginsConfig,
        explicitly_configured_channels: &[String],
    ) -> bool {
        // 1. Blocked check
        let in_deny = plugins_config.deny.iter().any(|d| d.eq_ignore_ascii_case(plugin_id));
        if in_deny || !plugins_config.enabled {
            return false;
        }

        let not_in_allowlist = !plugins_config.allow.is_empty()
            && !plugins_config.allow.iter().any(|a| a.eq_ignore_ascii_case(plugin_id));

        if not_in_allowlist {
            let has_explicit_channel = channel_ids.iter().any(|ch| {
                explicitly_configured_channels.iter().any(|c| c.eq_ignore_ascii_case(ch))
            });
            if origin != &PluginOrigin::Bundled || !has_explicit_channel {
                return false;
            }
        }

        if origin == &PluginOrigin::Bundled {
            return true;
        }

        // For non-bundled plugins: requires explicit activation
        if let Some(entry) = plugins_config.entries.get(plugin_id) {
            entry.enabled
        } else {
            false
        }
    }
}

// ---------------------------------------------------------------------------
// Channel Presence Policy & ID Resolution
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChannelPresenceSource {
    ExplicitConfig,
    Env,
    ManifestEnv,
    PersistedAuth,
    Cli,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfiguredChannelPresenceEntry {
    pub channel_id: String,
    pub sources: Vec<ChannelPresenceSource>,
    pub effective: bool,
    pub plugin_ids: Vec<String>,
    pub blocked_reasons: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ChannelManifestRecord {
    pub plugin_id: String,
    pub origin: PluginOrigin,
    pub channels: Vec<String>,
    pub enabled_by_default: bool,
    pub required_env: Vec<String>,
}

pub struct ChannelPresencePolicy;
impl ChannelPresencePolicy {
    pub fn has_explicit_channel_config(
        configured_channels: &HashMap<String, serde_json::Value>,
        channel_id: &str,
    ) -> bool {
        let channel_id_lower = channel_id.to_ascii_lowercase();
        for (k, v) in configured_channels {
            if k.to_ascii_lowercase() == channel_id_lower {
                if let Some(obj) = v.as_object() {
                    if let Some(enabled) = obj.get("enabled").and_then(|e| e.as_bool()) {
                        if !enabled {
                            return false;
                        }
                    }
                    return true;
                }
            }
        }
        false
    }

    pub fn list_explicit_configured_channel_ids(
        configured_channels: &HashMap<String, serde_json::Value>,
    ) -> Vec<String> {
        let mut ids = Vec::new();
        for (k, v) in configured_channels {
            let trimmed = k.trim();
            if trimmed.is_empty() || trimmed.starts_with('_') {
                continue;
            }
            if Self::has_explicit_channel_config(configured_channels, trimmed) {
                ids.push(trimmed.to_ascii_lowercase());
            }
        }
        ids.sort();
        ids.dedup();
        ids
    }

    pub fn resolve_channel_presence_policy(
        configured_channels: &HashMap<String, serde_json::Value>,
        env_vars: &HashMap<String, String>,
        manifests: &[ChannelManifestRecord],
        plugins_config: &NormalizedPluginsConfig,
        suppress_ambient_env: bool,
    ) -> Vec<ConfiguredChannelPresenceEntry> {
        let explicit_ids = Self::list_explicit_configured_channel_ids(configured_channels);
        let mut channel_sources: HashMap<String, HashSet<ChannelPresenceSource>> = HashMap::new();

        for ch in &explicit_ids {
            channel_sources
                .entry(ch.clone())
                .or_default()
                .insert(ChannelPresenceSource::ExplicitConfig);
        }

        // Check manifests for env-based presence
        for manifest in manifests {
            for ch in &manifest.channels {
                let ch_lower = ch.to_ascii_lowercase();
                if !manifest.required_env.is_empty()
                    && manifest.required_env.iter().all(|var| env_vars.contains_key(var))
                {
                    channel_sources
                        .entry(ch_lower.clone())
                        .or_default()
                        .insert(ChannelPresenceSource::ManifestEnv);
                }
            }
        }

        // Check for explicitly disabled channels
        let mut disabled_channels: HashSet<String> = HashSet::new();
        for (k, v) in configured_channels {
            if let Some(obj) = v.as_object() {
                if obj.get("enabled").and_then(|e| e.as_bool()) == Some(false) {
                    disabled_channels.insert(k.to_ascii_lowercase());
                }
            }
        }
        for dis in &disabled_channels {
            channel_sources.remove(dis);
        }

        // Suppress ambient-only if requested
        if suppress_ambient_env {
            channel_sources.retain(|_, sources| {
                !sources.iter().all(|s| *s == ChannelPresenceSource::Env || *s == ChannelPresenceSource::ManifestEnv)
            });
        }

        let mut sorted_channels: Vec<String> = channel_sources.keys().cloned().collect();
        sorted_channels.sort();

        let mut entries = Vec::new();
        for ch in sorted_channels {
            let sources = channel_sources.get(&ch).cloned().unwrap_or_default();
            let owning_manifests: Vec<&ChannelManifestRecord> = manifests
                .iter()
                .filter(|m| m.channels.iter().any(|c| c.eq_ignore_ascii_case(&ch)))
                .collect();

            let mut effective_plugin_ids = Vec::new();
            let mut blocked_reasons = Vec::new();

            if owning_manifests.is_empty() {
                blocked_reasons.push("no-channel-owner".to_string());
            } else {
                for m in owning_manifests {
                    let can_start = ChannelStartupPolicy::can_start_configured_channel_plugin(
                        &m.plugin_id,
                        &m.origin,
                        &m.channels,
                        plugins_config,
                        &explicit_ids,
                    );
                    if can_start {
                        effective_plugin_ids.push(m.plugin_id.clone());
                    } else if plugins_config.deny.iter().any(|d| d.eq_ignore_ascii_case(&m.plugin_id)) {
                        blocked_reasons.push("blocked-by-denylist".to_string());
                    } else if !plugins_config.enabled {
                        blocked_reasons.push("plugins-disabled".to_string());
                    } else {
                        blocked_reasons.push("not-in-allowlist".to_string());
                    }
                }
            }

            let effective = !effective_plugin_ids.is_empty();
            let mut sources_vec: Vec<ChannelPresenceSource> = sources.into_iter().collect();
            sources_vec.sort_by_key(|s| format!("{:?}", s));
            effective_plugin_ids.sort();
            effective_plugin_ids.dedup();
            blocked_reasons.sort();
            blocked_reasons.dedup();

            entries.push(ConfiguredChannelPresenceEntry {
                channel_id: ch,
                sources: sources_vec,
                effective,
                plugin_ids: effective_plugin_ids,
                blocked_reasons,
            });
        }

        entries
    }

    pub fn list_configured_announce_channel_ids(
        configured_channels: &HashMap<String, serde_json::Value>,
        env_vars: &HashMap<String, String>,
        manifests: &[ChannelManifestRecord],
        plugins_config: &NormalizedPluginsConfig,
    ) -> Vec<String> {
        let policy = Self::resolve_channel_presence_policy(
            configured_channels,
            env_vars,
            manifests,
            plugins_config,
            false,
        );
        let mut announced = Vec::new();
        for entry in policy {
            if entry.effective {
                announced.push(entry.channel_id);
            }
        }
        announced.sort();
        announced.dedup();
        announced
    }

    pub fn resolve_configured_channel_plugin_ids(
        configured_channels: &HashMap<String, serde_json::Value>,
        env_vars: &HashMap<String, String>,
        manifests: &[ChannelManifestRecord],
        plugins_config: &NormalizedPluginsConfig,
    ) -> Vec<String> {
        if !plugins_config.enabled {
            return Vec::new();
        }
        let policy = Self::resolve_channel_presence_policy(
            configured_channels,
            env_vars,
            manifests,
            plugins_config,
            false,
        );
        let mut plugin_ids = Vec::new();
        for entry in policy {
            if entry.effective {
                plugin_ids.extend(entry.plugin_ids);
            }
        }
        plugin_ids.sort();
        plugin_ids.dedup();
        plugin_ids
    }
}





