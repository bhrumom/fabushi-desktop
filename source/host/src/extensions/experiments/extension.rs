use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};

use crate::extensions::browser_ua::extension::{
    BrowserUaExperimentsApi, StopSubscription,
};
use crate::extensions::extension_ids_generated::HostExtensionId;

pub const EXPERIMENTS_DEPENDENCIES: &[HostExtensionId] =
    &[HostExtensionId::Auth, HostExtensionId::Settings];

pub fn experiments_extension_id() -> HostExtensionId {
    HostExtensionId::Experiments
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostExperimentsOptions {
    pub is_dev_build: bool,
    pub env_gate_overrides: Option<String>,
}

impl HostExperimentsOptions {
    pub fn from_process_env() -> Self {
        let is_dev_build = std::env::var("SAND_PACKAGED").as_deref() != Ok("1")
            || std::env::var("SAND_HOST_DEV_ERROR_DETAIL").as_deref() == Ok("1");
        Self {
            is_dev_build,
            env_gate_overrides: std::env::var("SAND_FEATURE_GATE_OVERRIDES")
                .ok()
                .filter(|value| !value.trim().is_empty()),
        }
    }
}

struct ExperimentsState {
    options: HostExperimentsOptions,
    settings_overrides: Mutex<BTreeMap<String, bool>>,
    dynamic_config_overrides: Mutex<BTreeMap<String, BTreeMap<String, serde_json::Value>>>,
    listeners: Mutex<BTreeMap<u64, Arc<dyn Fn() + Send + Sync>>>,
    next_listener_id: AtomicU64,
}

#[derive(Clone)]
pub struct HostExperimentsExtension {
    state: Arc<ExperimentsState>,
}

impl HostExperimentsExtension {
    pub fn new(options: HostExperimentsOptions) -> Self {
        Self {
            state: Arc::new(ExperimentsState {
                options,
                settings_overrides: Mutex::new(BTreeMap::new()),
                dynamic_config_overrides: Mutex::new(BTreeMap::new()),
                listeners: Mutex::new(BTreeMap::new()),
                next_listener_id: AtomicU64::new(0),
            }),
        }
    }

    pub fn check_feature_gate(&self, name: &str) -> bool {
        if let Some(value) = self
            .state
            .settings_overrides
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(name)
            .copied()
        {
            return value;
        }
        if self.state.options.is_dev_build {
            if let Some(value) = env_gate_override(
                name,
                self.state.options.env_gate_overrides.as_deref(),
            ) {
                return value;
            }
        }
        bundled_feature_gate_default(name)
    }

    pub fn is_agent_network_enabled(&self) -> bool {
        self.check_feature_gate("sand_agent_network")
    }

    pub fn is_ua_token_kill_switch_enabled(&self) -> bool {
        self.check_feature_gate("sand_browser_ua_token_kill_switch")
    }

    pub fn get_dynamic_config(
        &self,
        name: &str,
    ) -> BTreeMap<String, serde_json::Value> {
        let mut config = bundled_dynamic_config_default(name);
        let overrides = self
            .state
            .dynamic_config_overrides
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(values) = overrides.get(name) {
            for (key, value) in values {
                config.insert(key.clone(), value.clone());
            }
        }
        config
    }

    pub fn replace_feature_flag_overrides(&self, overrides: BTreeMap<String, bool>) {
        *self
            .state
            .settings_overrides
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = overrides;
        self.notify_listeners();
    }

    pub fn replace_dynamic_config_overrides(
        &self,
        overrides: BTreeMap<String, BTreeMap<String, serde_json::Value>>,
    ) {
        *self
            .state
            .dynamic_config_overrides
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = overrides;
        self.notify_listeners();
    }

    fn notify_listeners(&self) {
        let listeners = self
            .state
            .listeners
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .values()
            .cloned()
            .collect::<Vec<_>>();
        for listener in listeners {
            listener();
        }
    }

    pub fn subscribe(&self, listener: Arc<dyn Fn() + Send + Sync>) -> StopSubscription {
        let id = self.state.next_listener_id.fetch_add(1, Ordering::SeqCst) + 1;
        self.state
            .listeners
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(id, listener);
        let state: Weak<ExperimentsState> = Arc::downgrade(&self.state);
        Box::new(move || {
            if let Some(state) = state.upgrade() {
                state
                    .listeners
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .remove(&id);
            }
        })
    }
}

impl BrowserUaExperimentsApi for HostExperimentsExtension {
    fn is_ua_token_kill_switch_enabled(&self) -> bool {
        HostExperimentsExtension::is_ua_token_kill_switch_enabled(self)
    }

    fn subscribe(&self, listener: Arc<dyn Fn() + Send + Sync>) -> StopSubscription {
        HostExperimentsExtension::subscribe(self, listener)
    }
}

pub fn start_host_experiments_extension() -> HostExperimentsExtension {
    HostExperimentsExtension::new(HostExperimentsOptions::from_process_env())
}

fn env_gate_override(name: &str, raw: Option<&str>) -> Option<bool> {
    let raw = raw?;
    for pair in raw.split(',') {
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap_or("").trim();
        let value = parts.next().unwrap_or("").trim();
        if key == name {
            return Some(matches!(value, "1" | "true" | "on"));
        }
    }
    None
}

fn bundled_feature_gate_default(name: &str) -> bool {
    match name {
        // Frozen Grok 0.18 experiment-config.gen.ts bundled defaults used by
        // Host convenience gates that are already referenced by migrated code.
        "enable_sparse_plugin_clones" | "sand_multitask" | "sand_spotlight" => true,
        _ => false,
    }
}

fn bundled_dynamic_config_default(
    name: &str,
) -> BTreeMap<String, serde_json::Value> {
    match name {
        "grok_bot_conversation_size_limits" => BTreeMap::from([
            ("soft_limit_mb".into(), serde_json::json!(256)),
            ("hard_limit_mb".into(), serde_json::json!(1024)),
        ]),
        _ => BTreeMap::new(),
    }
}
