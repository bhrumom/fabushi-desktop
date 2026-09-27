use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{SecondsFormat, Utc};
use chrono_tz::Tz;
use serde_json::{Map, Value, json};

use crate::host_paths::get_sand_root_dir;

pub const MCP_CUSTOM_INSTRUCTIONS_MAX_LENGTH: usize = 500;
pub const SAND_AUTO_REVIEW_INSTRUCTION_MAX_ENTRIES: usize = 20;
pub const SAND_AUTO_REVIEW_INSTRUCTION_MAX_CHARS: usize = 1_000;
const INFERENCE_PROVIDERS: [&str; 4] = ["cursor", "claude-code", "codex", "openrouter"];

pub type UserTimeZoneListener = Arc<dyn Fn(Option<String>) + Send + Sync + 'static>;
pub type SettingsChangeListener = Arc<dyn Fn(Vec<String>) + Send + Sync + 'static>;
pub type FeatureFlagOverrideListener =
    Arc<dyn Fn(BTreeMap<String, bool>) + Send + Sync + 'static>;

pub fn is_valid_iana_time_zone(value: &str) -> bool {
    value.parse::<Tz>().is_ok()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandAutoReviewInstructions {
    pub is_enabled: bool,
    pub allow_instructions: Vec<String>,
    pub block_instructions: Vec<String>,
}
impl Default for SandAutoReviewInstructions {
    fn default() -> Self {
        Self {
            is_enabled: true,
            allow_instructions: Vec::new(),
            block_instructions: Vec::new(),
        }
    }
}
fn normalize_auto_review_instruction_list(raw: Option<&Value>) -> Vec<String> {
    let Some(items) = raw.and_then(Value::as_array) else {
        return Vec::new();
    };
    let mut result = Vec::new();
    for item in items {
        let Some(value) = item.as_str() else {
            continue;
        };
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        let clamped = trimmed
            .chars()
            .take(SAND_AUTO_REVIEW_INSTRUCTION_MAX_CHARS)
            .collect::<String>();
        if !result.contains(&clamped) {
            result.push(clamped);
        }
        if result.len() >= SAND_AUTO_REVIEW_INSTRUCTION_MAX_ENTRIES {
            break;
        }
    }
    result
}
pub fn normalize_sand_auto_review_instructions(
    raw: Option<&Value>,
) -> SandAutoReviewInstructions {
    let object = raw.and_then(Value::as_object);
    SandAutoReviewInstructions {
        is_enabled: object
            .and_then(|value| value.get("isEnabled"))
            .and_then(Value::as_bool)
            != Some(false),
        allow_instructions: normalize_auto_review_instruction_list(
            object.and_then(|value| value.get("allowInstructions")),
        ),
        block_instructions: normalize_auto_review_instruction_list(
            object.and_then(|value| value.get("blockInstructions")),
        ),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandLocalToolPermission {
    Always,
    Ask,
    Never,
}
pub const SAND_DEFAULT_LOCAL_TOOL_PERMISSION: SandLocalToolPermission =
    SandLocalToolPermission::Ask;
impl SandLocalToolPermission {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::Ask => "ask",
            Self::Never => "never",
        }
    }
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "always" => Some(Self::Always),
            "ask" => Some(Self::Ask),
            "never" => Some(Self::Never),
            _ => None,
        }
    }
    const fn rank(self) -> u8 {
        match self {
            Self::Never => 0,
            Self::Ask => 1,
            Self::Always => 2,
        }
    }
}
pub fn normalize_sand_local_tool_permission(
    value: Option<&str>,
) -> SandLocalToolPermission {
    value
        .and_then(SandLocalToolPermission::parse)
        .unwrap_or(SAND_DEFAULT_LOCAL_TOOL_PERMISSION)
}
pub fn resolve_sand_local_tool_permission(
    choice: SandLocalToolPermission,
    ceiling: Option<SandLocalToolPermission>,
) -> SandLocalToolPermission {
    match ceiling {
        Some(ceiling) if choice.rank() > ceiling.rank() => ceiling,
        _ => choice,
    }
}

pub struct SettingsSubscription<T: ?Sized> {
    id: u64,
    listeners: Arc<Mutex<BTreeMap<u64, Arc<T>>>>,
    active: bool,
}
impl<T: ?Sized> SettingsSubscription<T> {
    pub fn unsubscribe(mut self) {
        if self.active {
            if let Ok(mut listeners) = self.listeners.lock() {
                listeners.remove(&self.id);
            }
            self.active = false;
        }
    }
}
impl<T: ?Sized> Drop for SettingsSubscription<T> {
    fn drop(&mut self) {
        if self.active {
            if let Ok(mut listeners) = self.listeners.lock() {
                listeners.remove(&self.id);
            }
            self.active = false;
        }
    }
}

pub struct SettingsService {
    settings_path: PathBuf,
    user_time_zone_listeners: Arc<Mutex<BTreeMap<u64, UserTimeZoneListener>>>,
    change_listeners: Arc<Mutex<BTreeMap<u64, SettingsChangeListener>>>,
    feature_flag_override_listeners:
        Arc<Mutex<BTreeMap<u64, FeatureFlagOverrideListener>>>,
    next_listener_id: AtomicU64,
}
impl SettingsService {
    pub fn production() -> Self {
        Self::new(get_sand_root_dir().join("settings.json"))
    }
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            settings_path: path.into(),
            user_time_zone_listeners: Arc::new(Mutex::new(BTreeMap::new())),
            change_listeners: Arc::new(Mutex::new(BTreeMap::new())),
            feature_flag_override_listeners: Arc::new(Mutex::new(BTreeMap::new())),
            next_listener_id: AtomicU64::new(1),
        }
    }
    pub fn get_settings_path(&self) -> &Path {
        &self.settings_path
    }
    fn empty_settings() -> Map<String, Value> {
        Map::from_iter([
            ("version".into(), Value::from(1)),
            ("mcpBoxServers".into(), json!([])),
            ("webauthnProxyEnabled".into(), Value::Bool(true)),
            ("mcpCustomInstructions".into(), json!({})),
            ("mcpCustomInstructionsByServerId".into(), json!({})),
            ("mcpDisabledToolsByServerId".into(), json!({})),
        ])
    }
    fn load(&self) -> Map<String, Value> {
        let Ok(raw) = fs::read_to_string(&self.settings_path) else {
            return Self::empty_settings();
        };
        let Ok(Value::Object(mut value)) = serde_json::from_str::<Value>(&raw) else {
            return Self::empty_settings();
        };
        if value.get("version").and_then(Value::as_i64) != Some(1) {
            return Self::empty_settings();
        }
        for (key, default) in Self::empty_settings() {
            value.entry(key).or_insert(default);
        }
        value
    }
    fn persist(&self, value: &Map<String, Value>) -> io::Result<()> {
        if let Some(parent) = self.settings_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut raw =
            serde_json::to_string_pretty(&Value::Object(value.clone())).map_err(io::Error::other)?;
        raw.push('\n');
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let temp = PathBuf::from(format!(
            "{}.{}.{}.tmp",
            self.settings_path.display(),
            process::id(),
            nonce
        ));
        fs::write(&temp, raw)?;
        if let Err(error) = fs::rename(&temp, &self.settings_path) {
            let _ = fs::remove_file(&temp);
            return Err(error);
        }
        Ok(())
    }
    fn notify_changes(&self, fields: Vec<String>) {
        if fields.is_empty() {
            return;
        }
        let listeners = self
            .change_listeners
            .lock()
            .map(|listeners| listeners.values().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        for listener in listeners {
            listener(fields.clone());
        }
    }
    fn set_json_field(
        &self,
        field: &str,
        value: Option<Value>,
    ) -> Result<bool, String> {
        let mut settings = self.load();
        let changed = match value {
            Some(value) => {
                if settings.get(field) == Some(&value) {
                    false
                } else {
                    settings.insert(field.to_string(), value);
                    true
                }
            }
            None => settings.remove(field).is_some(),
        };
        if !changed {
            return Ok(false);
        }
        self.persist(&settings).map_err(|error| error.to_string())?;
        self.notify_changes(vec![field.to_string()]);
        Ok(true)
    }

    pub fn get_detected_user_time_zone(&self) -> Option<String> {
        self.load()
            .get("userTimeZone")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
    }
    pub fn get_user_time_zone_override(&self) -> Option<String> {
        self.load()
            .get("userTimeZoneOverride")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
    }
    pub fn get_user_time_zone(&self) -> Option<String> {
        self.get_user_time_zone_override()
            .or_else(|| self.get_detected_user_time_zone())
    }
    pub fn set_user_time_zone(&self, value: Option<&str>) -> Result<bool, String> {
        self.set_zone_field("userTimeZone", value)
    }
    pub fn set_user_time_zone_override(
        &self,
        value: Option<&str>,
    ) -> Result<bool, String> {
        self.set_zone_field("userTimeZoneOverride", value)
    }
    fn set_zone_field(
        &self,
        field: &str,
        value: Option<&str>,
    ) -> Result<bool, String> {
        let trimmed = value.map(str::trim).filter(|value| !value.is_empty());
        if let Some(zone) = trimmed {
            if !is_valid_iana_time_zone(zone) {
                return Ok(false);
            }
        }
        let before = self.get_user_time_zone();
        let mut settings = self.load();
        let changed = match trimmed {
            Some(zone) => {
                if settings.get(field).and_then(Value::as_str) == Some(zone) {
                    false
                } else {
                    settings.insert(field.into(), Value::String(zone.into()));
                    true
                }
            }
            None => settings.remove(field).is_some(),
        };
        if !changed {
            return Ok(false);
        }
        self.persist(&settings).map_err(|error| error.to_string())?;
        let after = self.get_user_time_zone();
        if after != before {
            let listeners = self
                .user_time_zone_listeners
                .lock()
                .map(|listeners| listeners.values().cloned().collect::<Vec<_>>())
                .unwrap_or_default();
            for listener in listeners {
                listener(after.clone());
            }
        }
        self.notify_changes(vec![field.to_string()]);
        Ok(true)
    }

    pub fn get_auto_review_instructions(&self) -> SandAutoReviewInstructions {
        let settings = self.load();
        normalize_sand_auto_review_instructions(settings.get("autoReviewInstructions"))
    }
    pub fn set_auto_review_instructions(
        &self,
        value: &SandAutoReviewInstructions,
    ) -> Result<bool, String> {
        let normalized = SandAutoReviewInstructions {
            is_enabled: value.is_enabled,
            allow_instructions: normalize_auto_review_instruction_list(Some(&Value::Array(
                value
                    .allow_instructions
                    .iter()
                    .cloned()
                    .map(Value::String)
                    .collect(),
            ))),
            block_instructions: normalize_auto_review_instruction_list(Some(&Value::Array(
                value
                    .block_instructions
                    .iter()
                    .cloned()
                    .map(Value::String)
                    .collect(),
            ))),
        };
        let next = json!({
            "isEnabled": normalized.is_enabled,
            "allowInstructions": normalized.allow_instructions,
            "blockInstructions": normalized.block_instructions
        });
        self.set_json_field("autoReviewInstructions", Some(next))
    }

    pub fn get_local_tool_permission_choice(&self) -> SandLocalToolPermission {
        normalize_sand_local_tool_permission(
            self.load()
                .get("localToolPermission")
                .and_then(Value::as_str),
        )
    }
    pub fn get_local_tool_permission_ceiling(&self) -> Option<SandLocalToolPermission> {
        self.load()
            .get("localToolPermissionCeiling")
            .and_then(Value::as_str)
            .and_then(SandLocalToolPermission::parse)
    }
    pub fn get_local_tool_permission(&self) -> SandLocalToolPermission {
        resolve_sand_local_tool_permission(
            self.get_local_tool_permission_choice(),
            self.get_local_tool_permission_ceiling(),
        )
    }
    pub fn set_local_tool_permission(
        &self,
        value: SandLocalToolPermission,
    ) -> Result<bool, String> {
        self.set_json_field(
            "localToolPermission",
            Some(Value::String(value.as_str().into())),
        )
    }
    pub fn set_local_tool_permission_ceiling(
        &self,
        value: Option<SandLocalToolPermission>,
    ) -> Result<bool, String> {
        self.set_json_field(
            "localToolPermissionCeiling",
            value.map(|value| Value::String(value.as_str().into())),
        )
    }

    pub fn get_webauthn_proxy_enabled(&self) -> bool {
        self.load()
            .get("webauthnProxyEnabled")
            .and_then(Value::as_bool)
            != Some(false)
    }
    pub fn set_webauthn_proxy_enabled(&self, value: bool) -> Result<bool, String> {
        self.set_json_field("webauthnProxyEnabled", Some(Value::Bool(value)))
    }

    pub fn get_agent_default_model(&self) -> Option<Value> {
        self.load()
            .get("agentDefaultModel")
            .filter(|value| is_model_selection(value))
            .cloned()
            .map(force_model_max_mode)
    }
    pub fn set_agent_default_model(
        &self,
        value: Option<&Value>,
    ) -> Result<bool, String> {
        if value.is_some_and(|value| !is_model_selection(value)) {
            return Ok(false);
        }
        self.set_json_field(
            "agentDefaultModel",
            value.cloned().map(force_model_max_mode),
        )
    }
    pub fn get_computer_use_model(&self) -> Option<Value> {
        self.load()
            .get("computerUseModel")
            .filter(|value| is_model_selection(value))
            .cloned()
    }
    pub fn set_computer_use_model(
        &self,
        value: Option<&Value>,
    ) -> Result<bool, String> {
        if value.is_some_and(|value| !is_model_selection(value)) {
            return Ok(false);
        }
        self.set_json_field("computerUseModel", value.cloned())
    }

    pub fn get_mcp_box_servers(&self) -> Vec<String> {
        unique_strings(self.load().get("mcpBoxServers"))
    }
    pub fn set_mcp_box_servers(&self, names: &[String]) -> Result<bool, String> {
        self.set_json_field(
            "mcpBoxServers",
            Some(Value::Array(
                dedupe_non_empty(names.iter().cloned())
                    .into_iter()
                    .map(Value::String)
                    .collect(),
            )),
        )
    }
    pub fn get_mcp_custom_instructions(&self) -> BTreeMap<String, String> {
        normalize_legacy_instructions(self.load().get("mcpCustomInstructions"))
    }
    pub fn set_mcp_custom_instructions(
        &self,
        value: BTreeMap<String, String>,
    ) -> Result<bool, String> {
        let normalized = value
            .into_iter()
            .filter_map(|(name, value)| {
                let value = clamp_mcp_custom_instruction(&value);
                if value.trim().is_empty() && default_mcp_custom_instruction(&name).is_empty() {
                    None
                } else {
                    Some((name, Value::String(value)))
                }
            })
            .collect::<Map<String, Value>>();
        self.set_json_field(
            "mcpCustomInstructions",
            Some(Value::Object(normalized)),
        )
    }
    pub fn get_mcp_custom_instructions_by_server_id(&self) -> BTreeMap<String, String> {
        normalize_server_instructions(
            self.load().get("mcpCustomInstructionsByServerId"),
        )
    }
    pub fn set_mcp_custom_instructions_by_server_id(
        &self,
        value: BTreeMap<String, String>,
    ) -> Result<bool, String> {
        let normalized = value
            .into_iter()
            .filter(|(id, _)| is_server_id(id))
            .map(|(id, value)| {
                (id, Value::String(clamp_mcp_custom_instruction(&value)))
            })
            .collect::<Map<String, Value>>();
        self.set_json_field(
            "mcpCustomInstructionsByServerId",
            Some(Value::Object(normalized)),
        )
    }
    pub fn get_mcp_disabled_tools_by_server_id(
        &self,
    ) -> BTreeMap<String, Vec<String>> {
        normalize_disabled_tools(self.load().get("mcpDisabledToolsByServerId"))
    }
    pub fn set_mcp_disabled_tools_by_server_id(
        &self,
        value: BTreeMap<String, Vec<String>>,
    ) -> Result<bool, String> {
        let mut normalized = Map::new();
        for (id, tools) in value {
            if !is_server_id(&id) {
                continue;
            }
            let tools = dedupe_non_empty(tools);
            if !tools.is_empty() {
                normalized.insert(
                    id,
                    Value::Array(tools.into_iter().map(Value::String).collect()),
                );
            }
        }
        self.set_json_field(
            "mcpDisabledToolsByServerId",
            Some(Value::Object(normalized)),
        )
    }
    pub fn get_mcp_custom_instructions_account_scope(&self) -> Option<String> {
        self.load()
            .get("mcpCustomInstructionsAccountScope")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    }
    pub fn scope_to_account(&self, account_scope: &str) -> Result<bool, String> {
        let account_scope = account_scope.trim();
        if account_scope.is_empty() {
            return self.clear_account_scope();
        }
        let mut settings = self.load();
        let previous = settings
            .get("mcpCustomInstructionsAccountScope")
            .and_then(Value::as_str)
            .map(str::to_string);
        if previous.as_deref() == Some(account_scope) {
            return Ok(false);
        }

        let seen = settings.get("hasSeenOnboarding").and_then(Value::as_bool);
        let seen_owner = settings
            .get("hasSeenOnboardingAccountScope")
            .and_then(Value::as_str)
            .map(str::to_string);
        settings.remove("hasSeenOnboarding");
        settings.remove("hasSeenOnboardingAccountScope");
        if let Some(seen) = seen {
            if previous.is_none() || seen_owner.as_deref() == Some(account_scope) {
                settings.insert("hasSeenOnboarding".into(), Value::Bool(seen));
                settings.insert(
                    "hasSeenOnboardingAccountScope".into(),
                    Value::String(account_scope.into()),
                );
            }
        }

        if previous.is_some() {
            clear_account_sensitive_settings(&mut settings);
        }
        settings.insert(
            "mcpCustomInstructionsAccountScope".into(),
            Value::String(account_scope.into()),
        );
        self.persist(&settings).map_err(|error| error.to_string())?;
        self.notify_changes(vec!["mcpCustomInstructionsAccountScope".into()]);
        Ok(true)
    }

    pub fn clear_account_scope(&self) -> Result<bool, String> {
        let mut settings = self.load();
        let before = settings.clone();
        settings.remove("mcpCustomInstructionsAccountScope");
        clear_account_sensitive_settings(&mut settings);
        if before == settings {
            return Ok(false);
        }
        self.persist(&settings).map_err(|error| error.to_string())?;
        self.notify_changes(vec!["mcpCustomInstructionsAccountScope".into()]);
        Ok(true)
    }
    pub fn get_raw_mcp_custom_instruction(&self, name: &str) -> Option<String> {
        self.get_mcp_custom_instructions().get(name).cloned()
    }
    pub fn get_raw_mcp_custom_instruction_by_server_id(
        &self,
        id: &str,
    ) -> Option<String> {
        self.get_mcp_custom_instructions_by_server_id()
            .get(id)
            .cloned()
    }
    pub fn migrate_mcp_custom_instruction_to_server_id(
        &self,
        server_id: &str,
        display_name: &str,
    ) -> Result<bool, String> {
        if !is_server_id(server_id) {
            return Ok(false);
        }
        let mut settings = self.load();
        let mut by_id =
            normalize_server_instructions(settings.get("mcpCustomInstructionsByServerId"));
        if by_id.contains_key(server_id) {
            return Ok(false);
        }
        let legacy = normalize_legacy_instructions(settings.get("mcpCustomInstructions"));
        let Some(value) = legacy.get(display_name).cloned() else {
            return Ok(false);
        };
        by_id.insert(server_id.into(), value);
        settings.insert(
            "mcpCustomInstructionsByServerId".into(),
            string_map_value(by_id),
        );
        self.persist(&settings).map_err(|error| error.to_string())?;
        self.notify_changes(vec!["mcpCustomInstructionsByServerId".into()]);
        Ok(true)
    }
    pub fn set_mcp_custom_instruction_by_server_id(
        &self,
        server_id: &str,
        display_name: &str,
        value: &str,
        mirror_legacy_name: bool,
    ) -> Result<bool, String> {
        if !is_server_id(server_id) {
            return Ok(false);
        }
        let value = clamp_mcp_custom_instruction(value);
        let mut settings = self.load();
        let mut by_id =
            normalize_server_instructions(settings.get("mcpCustomInstructionsByServerId"));
        let mut legacy =
            normalize_legacy_instructions(settings.get("mcpCustomInstructions"));
        by_id.insert(server_id.into(), value.clone());
        if mirror_legacy_name {
            if value.trim().is_empty()
                && default_mcp_custom_instruction(display_name).is_empty()
            {
                legacy.remove(display_name);
            } else {
                legacy.insert(display_name.into(), value);
            }
        } else {
            legacy.remove(display_name);
        }
        settings.insert(
            "mcpCustomInstructionsByServerId".into(),
            string_map_value(by_id),
        );
        settings.insert("mcpCustomInstructions".into(), string_map_value(legacy));
        self.persist(&settings).map_err(|error| error.to_string())?;
        self.notify_changes(vec![
            "mcpCustomInstructionsByServerId".into(),
            "mcpCustomInstructions".into(),
        ]);
        Ok(true)
    }
    pub fn delete_mcp_custom_instruction_by_server_id(
        &self,
        server_id: &str,
        display_name: &str,
        delete_legacy_name: bool,
    ) -> Result<bool, String> {
        let mut settings = self.load();
        let mut by_id =
            normalize_server_instructions(settings.get("mcpCustomInstructionsByServerId"));
        let mut legacy =
            normalize_legacy_instructions(settings.get("mcpCustomInstructions"));
        let mut changed = by_id.remove(server_id).is_some();
        if delete_legacy_name {
            changed |= legacy.remove(display_name).is_some();
        }
        if !changed {
            return Ok(false);
        }
        settings.insert(
            "mcpCustomInstructionsByServerId".into(),
            string_map_value(by_id),
        );
        settings.insert("mcpCustomInstructions".into(), string_map_value(legacy));
        self.persist(&settings).map_err(|error| error.to_string())?;
        self.notify_changes(vec![
            "mcpCustomInstructionsByServerId".into(),
            "mcpCustomInstructions".into(),
        ]);
        Ok(true)
    }

    pub fn get_pinned_agent_ids(&self) -> Option<Vec<String>> {
        let settings = self.load();
        settings
            .get("pinnedAgentIds")
            .and_then(Value::as_array)
            .map(|_| unique_strings(settings.get("pinnedAgentIds")))
    }
    pub fn set_pinned_agent_ids(&self, ids: &[String]) -> Result<bool, String> {
        self.set_json_field(
            "pinnedAgentIds",
            Some(Value::Array(
                dedupe_non_empty(ids.iter().cloned())
                    .into_iter()
                    .map(Value::String)
                    .collect(),
            )),
        )
    }
    pub fn get_sidebar_sections(&self) -> Option<Vec<Value>> {
        let settings = self.load();
        let raw = settings.get("sidebarSections")?.as_array()?;
        Some(normalize_sidebar_sections(raw, None))
    }
    pub fn set_sidebar_sections(&self, sections: &[Value]) -> Result<bool, String> {
        let stored = self.get_sidebar_sections();
        self.set_json_field(
            "sidebarSections",
            Some(Value::Array(normalize_sidebar_sections(
                sections,
                stored.as_deref(),
            ))),
        )
    }
    pub fn get_has_seen_onboarding(&self) -> Option<bool> {
        self.load().get("hasSeenOnboarding").and_then(Value::as_bool)
    }
    pub fn set_has_seen_onboarding(&self, value: bool) -> Result<bool, String> {
        let mut settings = self.load();
        let scope = settings
            .get("mcpCustomInstructionsAccountScope")
            .and_then(Value::as_str)
            .map(str::to_string);
        let mut changed =
            settings.get("hasSeenOnboarding").and_then(Value::as_bool) != Some(value);
        settings.insert("hasSeenOnboarding".into(), Value::Bool(value));
        settings.remove("hasSeenOnboardingAccountScope");
        if let Some(scope) = scope {
            settings.insert(
                "hasSeenOnboardingAccountScope".into(),
                Value::String(scope),
            );
            changed = true;
        }
        if !changed {
            return Ok(false);
        }
        self.persist(&settings).map_err(|error| error.to_string())?;
        self.notify_changes(vec!["hasSeenOnboarding".into()]);
        Ok(true)
    }

    pub fn get_inference_provider(&self) -> String {
        self.load()
            .get("inferenceProvider")
            .and_then(Value::as_str)
            .filter(|provider| INFERENCE_PROVIDERS.contains(provider))
            .unwrap_or("cursor")
            .to_string()
    }
    pub fn set_inference_provider(&self, value: &str) -> Result<bool, String> {
        if !INFERENCE_PROVIDERS.contains(&value) {
            return Ok(false);
        }
        self.set_json_field(
            "inferenceProvider",
            Some(Value::String(value.into())),
        )
    }
    pub fn get_inference_router_usage(&self) -> Value {
        normalize_inference_usage(self.load().get("inferenceRouterUsage"))
    }
    pub fn record_inference_usage(
        &self,
        provider: &str,
        input_tokens: Option<f64>,
        output_tokens: Option<f64>,
        cache_read_tokens: Option<f64>,
        cache_write_tokens: Option<f64>,
    ) -> Result<bool, String> {
        if !INFERENCE_PROVIDERS.contains(&provider) {
            return Ok(false);
        }
        let mut usage = self.get_inference_router_usage();
        let entry = usage
            .get_mut("providers")
            .and_then(Value::as_object_mut)
            .and_then(|providers| providers.get_mut(provider))
            .and_then(Value::as_object_mut)
            .expect("normalized inference usage provider");
        increment_counter(entry, "requests", 1);
        increment_counter(entry, "inputTokens", safe_usage(input_tokens));
        increment_counter(entry, "outputTokens", safe_usage(output_tokens));
        increment_counter(entry, "cacheReadTokens", safe_usage(cache_read_tokens));
        increment_counter(entry, "cacheWriteTokens", safe_usage(cache_write_tokens));
        entry.insert(
            "lastUsedAt".into(),
            Value::String(
                Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
            ),
        );
        self.set_json_field("inferenceRouterUsage", Some(usage))
    }

    pub fn get_notification_config(&self) -> Value {
        disabled_notification_config()
    }
    pub fn set_notification_config(&self, _input: &Value) -> Result<bool, String> {
        self.set_json_field(
            "notifications",
            Some(json!({ "isEnabled": false })),
        )
    }

    pub fn get_host_settings(&self) -> Value {
        let mut object = Map::new();
        object.insert("notifications".into(), self.get_notification_config());
        object.insert(
            "mcpCustomInstructions".into(),
            string_map_value(self.get_mcp_custom_instructions()),
        );
        object.insert(
            "mcpCustomInstructionsByServerId".into(),
            string_map_value(self.get_mcp_custom_instructions_by_server_id()),
        );
        object.insert(
            "mcpDisabledToolsByServerId".into(),
            string_list_map_value(self.get_mcp_disabled_tools_by_server_id()),
        );
        if let Some(scope) = self.get_mcp_custom_instructions_account_scope() {
            object.insert(
                "mcpCustomInstructionsAccountScope".into(),
                Value::String(scope),
            );
        }
        object.insert(
            "mcpBoxServers".into(),
            Value::Array(
                self.get_mcp_box_servers()
                    .into_iter()
                    .map(Value::String)
                    .collect(),
            ),
        );
        let auto_review = self.get_auto_review_instructions();
        object.insert(
            "autoReviewInstructions".into(),
            json!({
                "isEnabled": auto_review.is_enabled,
                "allowInstructions": auto_review.allow_instructions,
                "blockInstructions": auto_review.block_instructions
            }),
        );
        object.insert(
            "localToolPermission".into(),
            Value::String(self.get_local_tool_permission().as_str().into()),
        );
        object.insert(
            "webauthnProxyEnabled".into(),
            Value::Bool(self.get_webauthn_proxy_enabled()),
        );
        object.insert(
            "inferenceProvider".into(),
            Value::String(self.get_inference_provider()),
        );
        object.insert(
            "inferenceRouterUsage".into(),
            self.get_inference_router_usage(),
        );
        if let Some(value) = self.get_detected_user_time_zone() {
            object.insert("userTimeZone".into(), Value::String(value));
        }
        if let Some(value) = self.get_user_time_zone_override() {
            object.insert("userTimeZoneOverride".into(), Value::String(value));
        }
        if let Some(value) = self.get_agent_default_model() {
            object.insert("agentDefaultModel".into(), value);
        }
        if let Some(value) = self.get_computer_use_model() {
            object.insert("computerUseModel".into(), value);
        }
        if let Some(value) = self.get_pinned_agent_ids() {
            object.insert(
                "pinnedAgentIds".into(),
                Value::Array(value.into_iter().map(Value::String).collect()),
            );
        }
        object.insert(
            "sidebarSections".into(),
            Value::Array(self.get_sidebar_sections().unwrap_or_default()),
        );
        if let Some(value) = self.get_has_seen_onboarding() {
            object.insert("hasSeenOnboarding".into(), Value::Bool(value));
        }
        Value::Object(object)
    }

    pub fn set_host_settings(&self, update: &Value) -> Result<Value, String> {
        let Some(update) = update.as_object() else {
            return Ok(self.get_host_settings());
        };
        let previous_zone = self.get_user_time_zone();
        let mut settings = self.load();
        let changed_fields = update.keys().cloned().collect::<Vec<_>>();

        settings.insert("notifications".into(), json!({ "isEnabled": false }));

        if let Some(value) = update.get("mcpCustomInstructionsAccountScope") {
            match value {
                Value::Null => {
                    settings.remove("mcpCustomInstructionsAccountScope");
                    clear_account_sensitive_settings(&mut settings);
                }
                Value::String(scope) if !scope.trim().is_empty() => {
                    let previous = settings
                        .get("mcpCustomInstructionsAccountScope")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                    if previous.as_deref() != Some(scope.trim()) {
                        if previous.is_some() {
                            clear_account_sensitive_settings(&mut settings);
                        }
                        settings.insert(
                            "mcpCustomInstructionsAccountScope".into(),
                            Value::String(scope.trim().into()),
                        );
                    }
                }
                _ => {}
            }
        }
        if let Some(value) = update.get("mcpCustomInstructions") {
            settings.insert(
                "mcpCustomInstructions".into(),
                string_map_value(normalize_legacy_instructions(Some(value))),
            );
        }
        if let Some(value) = update.get("mcpCustomInstructionsByServerId") {
            settings.insert(
                "mcpCustomInstructionsByServerId".into(),
                string_map_value(normalize_server_instructions(Some(value))),
            );
        }
        if let Some(value) = update.get("mcpDisabledToolsByServerId") {
            settings.insert(
                "mcpDisabledToolsByServerId".into(),
                string_list_map_value(normalize_disabled_tools(Some(value))),
            );
        }
        if let Some(value) = update.get("mcpBoxServers") {
            settings.insert(
                "mcpBoxServers".into(),
                Value::Array(
                    unique_strings(Some(value))
                        .into_iter()
                        .map(Value::String)
                        .collect(),
                ),
            );
        }
        for field in ["userTimeZone", "userTimeZoneOverride"] {
            if let Some(Value::String(zone)) = update.get(field) {
                let trimmed = zone.trim();
                if trimmed.is_empty() {
                    settings.remove(field);
                } else if is_valid_iana_time_zone(trimmed) {
                    settings.insert(field.into(), Value::String(trimmed.into()));
                }
            }
        }
        if let Some(value) = update.get("agentDefaultModel") {
            if value.is_null() {
                settings.remove("agentDefaultModel");
            } else if is_model_selection(value) {
                settings.insert(
                    "agentDefaultModel".into(),
                    force_model_max_mode(value.clone()),
                );
            }
        }
        if let Some(value) = update.get("computerUseModel") {
            if value.is_null() {
                settings.remove("computerUseModel");
            } else if is_model_selection(value) {
                settings.insert("computerUseModel".into(), value.clone());
            }
        }
        if let Some(value) = update.get("autoReviewInstructions") {
            let normalized = normalize_sand_auto_review_instructions(Some(value));
            settings.insert(
                "autoReviewInstructions".into(),
                json!({
                    "isEnabled": normalized.is_enabled,
                    "allowInstructions": normalized.allow_instructions,
                    "blockInstructions": normalized.block_instructions
                }),
            );
        }
        if let Some(value) = update.get("localToolPermission") {
            let normalized =
                normalize_sand_local_tool_permission(value.as_str());
            settings.insert(
                "localToolPermission".into(),
                Value::String(normalized.as_str().into()),
            );
        }
        if let Some(Value::Bool(value)) = update.get("webauthnProxyEnabled") {
            settings.insert("webauthnProxyEnabled".into(), Value::Bool(*value));
        }
        if let Some(value) = update.get("pinnedAgentIds") {
            settings.insert(
                "pinnedAgentIds".into(),
                Value::Array(
                    unique_strings(Some(value))
                        .into_iter()
                        .map(Value::String)
                        .collect(),
                ),
            );
        }
        if let Some(Value::Array(value)) = update.get("sidebarSections") {
            let stored = settings
                .get("sidebarSections")
                .and_then(Value::as_array)
                .cloned();
            settings.insert(
                "sidebarSections".into(),
                Value::Array(normalize_sidebar_sections(value, stored.as_deref())),
            );
        }
        if let Some(Value::Bool(value)) = update.get("hasSeenOnboarding") {
            settings.insert("hasSeenOnboarding".into(), Value::Bool(*value));
        }
        if let Some(Value::String(value)) = update.get("inferenceProvider") {
            if INFERENCE_PROVIDERS.contains(&value.as_str()) {
                settings.insert(
                    "inferenceProvider".into(),
                    Value::String(value.clone()),
                );
            }
        }

        self.persist(&settings).map_err(|error| error.to_string())?;

        let next_zone = self.get_user_time_zone();
        if next_zone != previous_zone {
            let listeners = self
                .user_time_zone_listeners
                .lock()
                .map(|listeners| listeners.values().cloned().collect::<Vec<_>>())
                .unwrap_or_default();
            for listener in listeners {
                listener(next_zone.clone());
            }
        }
        if let Some(Value::Object(raw)) = update.get("featureFlagOverrides") {
            let overrides = raw
                .iter()
                .filter_map(|(key, value)| value.as_bool().map(|value| (key.clone(), value)))
                .collect::<BTreeMap<_, _>>();
            let listeners = self
                .feature_flag_override_listeners
                .lock()
                .map(|listeners| listeners.values().cloned().collect::<Vec<_>>())
                .unwrap_or_default();
            for listener in listeners {
                listener(overrides.clone());
            }
        }
        self.notify_changes(changed_fields);
        Ok(self.get_host_settings())
    }

    pub fn subscribe_to_user_time_zone(
        &self,
        listener: UserTimeZoneListener,
    ) -> SettingsSubscription<dyn Fn(Option<String>) + Send + Sync + 'static> {
        let id = self.next_listener_id.fetch_add(1, Ordering::Relaxed);
        self.user_time_zone_listeners
            .lock()
            .expect("settings timezone listeners poisoned")
            .insert(id, listener);
        SettingsSubscription {
            id,
            listeners: Arc::clone(&self.user_time_zone_listeners),
            active: true,
        }
    }
    pub fn subscribe_to_changes(
        &self,
        listener: SettingsChangeListener,
    ) -> SettingsSubscription<dyn Fn(Vec<String>) + Send + Sync + 'static> {
        let id = self.next_listener_id.fetch_add(1, Ordering::Relaxed);
        self.change_listeners
            .lock()
            .expect("settings change listeners poisoned")
            .insert(id, listener);
        SettingsSubscription {
            id,
            listeners: Arc::clone(&self.change_listeners),
            active: true,
        }
    }
    pub fn subscribe_to_feature_flag_overrides(
        &self,
        listener: FeatureFlagOverrideListener,
    ) -> SettingsSubscription<dyn Fn(BTreeMap<String, bool>) + Send + Sync + 'static> {
        let id = self.next_listener_id.fetch_add(1, Ordering::Relaxed);
        self.feature_flag_override_listeners
            .lock()
            .expect("settings feature override listeners poisoned")
            .insert(id, listener);
        SettingsSubscription {
            id,
            listeners: Arc::clone(&self.feature_flag_override_listeners),
            active: true,
        }
    }
}

fn clamp_mcp_custom_instruction(value: &str) -> String {
    value.chars().take(MCP_CUSTOM_INSTRUCTIONS_MAX_LENGTH).collect()
}
fn default_mcp_custom_instruction(name: &str) -> &'static str {
    if name.trim().eq_ignore_ascii_case("hex") {
        "When using Hex, get the underlying numbers as data: download/export the results as CSV or use the data the connector returns, and analyze those raw values directly. Don't read rendered charts or graphs from screenshots (computer-use chart reading is unreliable) — work from the actual data."
    } else {
        ""
    }
}
fn is_server_id(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('0')
        && value.bytes().all(|byte| byte.is_ascii_digit())
}
fn normalize_legacy_instructions(
    value: Option<&Value>,
) -> BTreeMap<String, String> {
    let Some(object) = value.and_then(Value::as_object) else {
        return BTreeMap::new();
    };
    object
        .iter()
        .filter_map(|(name, value)| {
            let value = clamp_mcp_custom_instruction(value.as_str()?);
            if value.trim().is_empty() && default_mcp_custom_instruction(name).is_empty() {
                None
            } else {
                Some((name.clone(), value))
            }
        })
        .collect()
}
fn normalize_server_instructions(
    value: Option<&Value>,
) -> BTreeMap<String, String> {
    let Some(object) = value.and_then(Value::as_object) else {
        return BTreeMap::new();
    };
    object
        .iter()
        .filter(|(id, _)| is_server_id(id))
        .filter_map(|(id, value)| {
            value.as_str().map(|value| {
                (id.clone(), clamp_mcp_custom_instruction(value))
            })
        })
        .collect()
}
fn normalize_disabled_tools(
    value: Option<&Value>,
) -> BTreeMap<String, Vec<String>> {
    let Some(object) = value.and_then(Value::as_object) else {
        return BTreeMap::new();
    };
    let mut result = BTreeMap::new();
    for (id, tools) in object {
        if !is_server_id(id) {
            continue;
        }
        let tools = unique_strings(Some(tools));
        if !tools.is_empty() {
            result.insert(id.clone(), tools);
        }
    }
    result
}
fn dedupe_non_empty(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for value in values {
        if !value.is_empty() && seen.insert(value.clone()) {
            result.push(value);
        }
    }
    result
}
fn unique_strings(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            dedupe_non_empty(
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string),
            )
        })
        .unwrap_or_default()
}
fn string_map_value(values: BTreeMap<String, String>) -> Value {
    Value::Object(
        values
            .into_iter()
            .map(|(key, value)| (key, Value::String(value)))
            .collect(),
    )
}
fn string_list_map_value(values: BTreeMap<String, Vec<String>>) -> Value {
    Value::Object(
        values
            .into_iter()
            .map(|(key, values)| {
                (
                    key,
                    Value::Array(values.into_iter().map(Value::String).collect()),
                )
            })
            .collect(),
    )
}
fn clear_account_sensitive_settings(settings: &mut Map<String, Value>) {
    for field in [
        "autoReviewInstructions",
        "agentDefaultModel",
        "computerUseModel",
        "localToolPermission",
        "localToolPermissionCeiling",
    ] {
        settings.remove(field);
    }
    settings.insert("mcpCustomInstructions".into(), json!({}));
    settings.insert("mcpCustomInstructionsByServerId".into(), json!({}));
    settings.insert("mcpDisabledToolsByServerId".into(), json!({}));
}
fn is_model_selection(value: &Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    if object
        .get("modelId")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .is_none()
        || object.get("maxMode").and_then(Value::as_bool).is_none()
    {
        return false;
    }
    object
        .get("parameters")
        .and_then(Value::as_array)
        .is_some_and(|parameters| {
            parameters.iter().all(|parameter| {
                parameter.as_object().is_some_and(|parameter| {
                    parameter
                        .get("id")
                        .and_then(Value::as_str)
                        .filter(|value| !value.is_empty())
                        .is_some()
                        && parameter.get("value").and_then(Value::as_str).is_some()
                })
            })
        })
}
fn force_model_max_mode(mut value: Value) -> Value {
    if let Some(object) = value.as_object_mut() {
        object.insert("maxMode".into(), Value::Bool(true));
    }
    value
}
fn normalize_sidebar_sections(
    sections: &[Value],
    stored: Option<&[Value]>,
) -> Vec<Value> {
    let mut folds = BTreeMap::<String, bool>::new();
    for source in stored.into_iter().flatten().chain(sections.iter()) {
        let Some(object) = source.as_object() else {
            continue;
        };
        let Some(id) = object.get("id").and_then(Value::as_str) else {
            continue;
        };
        if let Some(collapsed) = object.get("isCollapsed").and_then(Value::as_bool) {
            folds.insert(id.trim().to_string(), collapsed);
        }
    }
    let mut seen_sections = BTreeSet::new();
    let mut claimed_agents = BTreeSet::new();
    let mut result = Vec::new();
    for section in sections {
        let Some(object) = section.as_object() else {
            continue;
        };
        let Some(raw_id) = object.get("id").and_then(Value::as_str) else {
            continue;
        };
        let id = raw_id.trim();
        if id.is_empty() || id == "__agents__" || !seen_sections.insert(id.to_string()) {
            continue;
        }
        let name = object
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let mut agent_ids = Vec::new();
        if let Some(items) = object.get("agentIds").and_then(Value::as_array) {
            for item in items {
                let Some(agent_id) = item.as_str() else {
                    continue;
                };
                if !agent_id.is_empty() && claimed_agents.insert(agent_id.to_string()) {
                    agent_ids.push(Value::String(agent_id.to_string()));
                }
            }
        }
        result.push(json!({
            "id": id,
            "name": name,
            "agentIds": agent_ids,
            "isCollapsed": folds.get(id).copied().unwrap_or(false)
        }));
    }
    if !result.is_empty() {
        result.push(json!({
            "id": "__agents__",
            "name": "Unassigned",
            "agentIds": [],
            "isCollapsed": folds.get("__agents__").copied().unwrap_or(false)
        }));
    }
    result
}
fn empty_inference_usage() -> Value {
    let empty = || {
        json!({
            "requests": 0,
            "inputTokens": 0,
            "outputTokens": 0,
            "cacheReadTokens": 0,
            "cacheWriteTokens": 0,
            "lastUsedAt": null
        })
    };
    json!({
        "schemaVersion": 1,
        "providers": {
            "cursor": empty(),
            "claude-code": empty(),
            "codex": empty(),
            "openrouter": empty()
        }
    })
}
fn normalize_inference_usage(value: Option<&Value>) -> Value {
    let mut usage = empty_inference_usage();
    let Some(raw_providers) = value
        .and_then(|value| value.get("providers"))
        .and_then(Value::as_object)
    else {
        return usage;
    };
    for provider in INFERENCE_PROVIDERS {
        let Some(raw) = raw_providers.get(provider).and_then(Value::as_object) else {
            continue;
        };
        let target = usage
            .get_mut("providers")
            .and_then(Value::as_object_mut)
            .and_then(|providers| providers.get_mut(provider))
            .and_then(Value::as_object_mut)
            .expect("empty usage has provider");
        for key in [
            "requests",
            "inputTokens",
            "outputTokens",
            "cacheReadTokens",
            "cacheWriteTokens",
        ] {
            let count = raw.get(key).and_then(Value::as_u64).unwrap_or(0);
            target.insert(key.into(), Value::from(count));
        }
        if let Some(last_used_at) = raw.get("lastUsedAt").and_then(Value::as_str) {
            target.insert(
                "lastUsedAt".into(),
                Value::String(last_used_at.to_string()),
            );
        }
    }
    usage
}
fn safe_usage(value: Option<f64>) -> u64 {
    value
        .filter(|value| value.is_finite() && *value >= 0.0)
        .map(|value| value.round().min(u64::MAX as f64) as u64)
        .unwrap_or(0)
}
fn increment_counter(object: &mut Map<String, Value>, key: &str, by: u64) {
    let current = object.get(key).and_then(Value::as_u64).unwrap_or(0);
    object.insert(key.into(), Value::from(current.saturating_add(by)));
}
fn disabled_notification_config() -> Value {
    json!({
        "isEnabled": false,
        "allowedApps": [],
        "minIntervalMs": 5_000,
        "maxPerWindow": 10,
        "windowMs": 5 * 60_000
    })
}
