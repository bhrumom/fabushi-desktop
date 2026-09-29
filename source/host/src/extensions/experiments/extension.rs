use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak, mpsc};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use reqwest::blocking::Client;
use serde_json::Value;
use url::Url;
use uuid::Uuid;

use crate::extensions::auth::credential_renewer::{
    RenewalOutcome, SAND_CLIENT_TYPE, sand_box_namespace, sand_client_version,
};
use crate::extensions::auth::extension::HostAuthExtension;
use crate::extensions::browser_ua::extension::{
    BrowserUaExperimentsApi, StopSubscription,
};
use crate::extensions::extension_ids_generated::HostExtensionId;
use crate::extensions::inference::sand_model_experiment::{
    SandModelExperimentState, read_sand_model_experiment_process_override,
};

pub const EXPERIMENTS_DEPENDENCIES: &[HostExtensionId] =
    &[HostExtensionId::Auth, HostExtensionId::Settings];
pub const STATSIG_BOOTSTRAP_CACHE_FILENAME: &str = "sand-statsig-bootstrap.json";
pub const STATSIG_BOOTSTRAP_PATH: &str = "aiserver.v1.AnalyticsService/BootstrapStatsig";
pub const STATSIG_BOOTSTRAP_TIMEOUT: Duration = Duration::from_secs(30);
pub const STATSIG_POLL_INTERVAL: Duration = Duration::from_secs(5 * 60);

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

type AuthenticatedGatePin = Arc<dyn Fn(bool) + Send + Sync>;

struct ExperimentsState {
    options: HostExperimentsOptions,
    settings_overrides: Mutex<BTreeMap<String, bool>>,
    network_feature_gates: Mutex<BTreeMap<String, bool>>,
    dynamic_config_overrides: Mutex<BTreeMap<String, BTreeMap<String, serde_json::Value>>>,
    listeners: Mutex<BTreeMap<u64, Arc<dyn Fn() + Send + Sync>>>,
    next_listener_id: AtomicU64,
    has_authenticated_statsig_bootstrap: AtomicBool,
    authenticated_gate_pins: Mutex<Vec<(String, AuthenticatedGatePin)>>,
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
                network_feature_gates: Mutex::new(BTreeMap::new()),
                dynamic_config_overrides: Mutex::new(BTreeMap::new()),
                listeners: Mutex::new(BTreeMap::new()),
                next_listener_id: AtomicU64::new(0),
                has_authenticated_statsig_bootstrap: AtomicBool::new(false),
                authenticated_gate_pins: Mutex::new(Vec::new()),
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
        if let Some(value) = self
            .state
            .network_feature_gates
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(name)
            .copied()
        {
            return value;
        }
        bundled_feature_gate_default(name)
    }

    pub fn has_authenticated_statsig_bootstrap(&self) -> bool {
        self.state
            .has_authenticated_statsig_bootstrap
            .load(Ordering::Acquire)
    }

    pub fn pin_gate_on_authenticated_bootstrap(
        &self,
        name: impl Into<String>,
        pin: Arc<dyn Fn(bool) + Send + Sync>,
    ) {
        let name = name.into();
        if self.has_authenticated_statsig_bootstrap() {
            pin(self.check_feature_gate(&name));
            return;
        }
        {
            let mut pins = self
                .state
                .authenticated_gate_pins
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if !self.has_authenticated_statsig_bootstrap() {
                pins.push((name.clone(), Arc::clone(&pin)));
                return;
            }
        }
        pin(self.check_feature_gate(&name));
    }

    pub fn hydrate_statsig_bootstrap(
        &self,
        config: &str,
        authenticated_network: bool,
    ) -> Result<bool, String> {
        let (user_id, gates) = parse_statsig_bootstrap(config)?;
        *self
            .state
            .network_feature_gates
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = gates;
        let authenticated = authenticated_network
            && user_id.as_deref().is_some_and(|value| !value.is_empty());
        if authenticated {
            self.state
                .has_authenticated_statsig_bootstrap
                .store(true, Ordering::Release);
        }
        self.notify_listeners();
        if authenticated {
            let pins = std::mem::take(
                &mut *self
                    .state
                    .authenticated_gate_pins
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()),
            );
            for (name, pin) in pins {
                pin(self.check_feature_gate(&name));
            }
        }
        Ok(authenticated)
    }

    pub fn is_agent_network_enabled(&self) -> bool {
        self.check_feature_gate("sand_agent_network")
    }

    pub fn is_ua_token_kill_switch_enabled(&self) -> bool {
        self.check_feature_gate("sand_browser_ua_token_kill_switch")
    }

    pub fn get_sand_model_experiment_state(&self) -> Option<SandModelExperimentState> {
        read_sand_model_experiment_process_override()
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

enum StatsigBootstrapCommand {
    Refresh,
    Stop,
}

pub struct ProductionStatsigBootstrapRuntime {
    stop: mpsc::Sender<StatsigBootstrapCommand>,
    worker: Mutex<Option<JoinHandle<()>>>,
    auth_stop: Mutex<Option<StopSubscription>>,
}

impl Drop for ProductionStatsigBootstrapRuntime {
    fn drop(&mut self) {
        if let Ok(stop) = self.auth_stop.get_mut() {
            if let Some(stop) = stop.take() {
                stop();
            }
        }
        let _ = self.stop.send(StatsigBootstrapCommand::Stop);
        if let Ok(worker) = self.worker.get_mut() {
            if let Some(worker) = worker.take() {
                let _ = worker.join();
            }
        }
    }
}

pub fn start_authenticated_statsig_bootstrap(
    experiments: Arc<HostExperimentsExtension>,
    auth: Arc<HostAuthExtension>,
    backend_url: String,
    cache_dir: PathBuf,
) -> Result<ProductionStatsigBootstrapRuntime, String> {
    let cache_path = cache_dir.join(STATSIG_BOOTSTRAP_CACHE_FILENAME);
    if let Some(config) = load_cached_statsig_bootstrap(&cache_path) {
        let _ = experiments.hydrate_statsig_bootstrap(&config, false);
    }

    let client = Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(STATSIG_BOOTSTRAP_TIMEOUT)
        .build()
        .map_err(|error| format!("build Statsig bootstrap client: {error}"))?;
    let (tx, rx) = mpsc::channel();
    let worker_experiments = Arc::clone(&experiments);
    let worker_auth = Arc::clone(&auth);
    let worker_cache = cache_path.clone();
    let worker = thread::Builder::new()
        .name("sand-experiments-statsig".into())
        .spawn(move || {
            let refresh = || {
                match fetch_statsig_bootstrap(
                    &client,
                    &backend_url,
                    &worker_auth,
                ) {
                    Ok(Some((config, authenticated))) => {
                        if worker_experiments
                            .hydrate_statsig_bootstrap(&config, authenticated)
                            .is_ok()
                        {
                            let _ = save_cached_statsig_bootstrap(&worker_cache, &config);
                        }
                    }
                    Ok(None) => {}
                    Err(error) => {
                        eprintln!("mahayana-host-experiments Statsig bootstrap failed: {error}");
                    }
                }
            };

            refresh();
            loop {
                match rx.recv_timeout(STATSIG_POLL_INTERVAL) {
                    Ok(StatsigBootstrapCommand::Refresh)
                    | Err(mpsc::RecvTimeoutError::Timeout) => refresh(),
                    Ok(StatsigBootstrapCommand::Stop)
                    | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        })
        .map_err(|error| format!("start Statsig bootstrap worker: {error}"))?;

    let renewal_tx = tx.clone();
    let auth_service = Arc::clone(auth.service());
    let subscription_id = auth_service.subscribe_to_renewal(Arc::new(move |event| {
        if event.result.outcome == RenewalOutcome::Renewed {
            let _ = renewal_tx.send(StatsigBootstrapCommand::Refresh);
        }
    }));
    let auth_service_for_stop = Arc::clone(&auth_service);
    let auth_stop: StopSubscription = Box::new(move || {
        auth_service_for_stop.unsubscribe_from_renewal(subscription_id);
    });

    Ok(ProductionStatsigBootstrapRuntime {
        stop: tx,
        worker: Mutex::new(Some(worker)),
        auth_stop: Mutex::new(Some(auth_stop)),
    })
}

fn fetch_statsig_bootstrap(
    client: &Client,
    backend_url: &str,
    auth: &HostAuthExtension,
) -> Result<Option<(String, bool)>, String> {
    let base = Url::parse(backend_url)
        .map_err(|error| format!("invalid Statsig backend URL: {error}"))?;
    let url = base
        .join(STATSIG_BOOTSTRAP_PATH)
        .map_err(|error| format!("join Statsig bootstrap URL: {error}"))?;
    let machine_id = auth
        .get_machine_id()
        .map_err(|error| format!("read machine id for Statsig bootstrap: {error}"))?;
    let access_token = auth.get_access_token().ok();
    let mut request = client
        .post(url)
        .header("content-type", "application/json")
        .header("x-cursor-checksum", create_cursor_checksum(&machine_id, wall_clock_now_ms()))
        .header("x-cursor-client-type", SAND_CLIENT_TYPE)
        .header("x-cursor-client-version", sand_client_version())
        .header("x-sand-box-namespace", sand_box_namespace())
        .header("x-ghost-mode", "true")
        .header("x-request-id", Uuid::new_v4().to_string())
        .body("{}");
    if std::env::var("CURSOR_AGENT_CLI_LOCAL_MODE").as_deref() == Ok("true") {
        request = request.header("local-cli-mode", "true");
    }
    if let Some(token) = access_token.as_deref() {
        request = request.bearer_auth(token);
    }
    let response = request
        .send()
        .map_err(|error| format!("Statsig bootstrap transport: {error}"))?;
    if !response.status().is_success() {
        return Ok(None);
    }
    let payload = response
        .json::<Value>()
        .map_err(|error| format!("parse Statsig bootstrap response: {error}"))?;
    Ok(payload
        .get("config")
        .and_then(Value::as_str)
        .map(|config| (config.to_string(), access_token.is_some())))
}

fn wall_clock_now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64
}

pub fn create_cursor_checksum(machine_id: &str, now_ms: u64) -> String {
    let unix_kilo_seconds = (now_ms / 1_000_000) as u32;
    let shifts = [40u32, 32, 24, 16, 8, 0];
    let mut bytes = [0u8; 6];
    for (index, shift) in shifts.into_iter().enumerate() {
        bytes[index] = ((unix_kilo_seconds >> (shift % 32)) & 0xff) as u8;
    }
    let mut last_byte = 165u8;
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = (*byte ^ last_byte).wrapping_add(index as u8);
        last_byte = *byte;
    }
    format!("{}{}", URL_SAFE_NO_PAD.encode(bytes), machine_id)
}

fn load_cached_statsig_bootstrap(path: &Path) -> Option<String> {
    let parsed = serde_json::from_str::<Value>(&fs::read_to_string(path).ok()?).ok()?;
    parsed
        .get("config")
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn save_cached_statsig_bootstrap(path: &Path, config: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("create Statsig cache directory: {error}"))?;
    }
    let (user_id, _) = parse_statsig_bootstrap(config)?;
    let payload = serde_json::to_vec(&serde_json::json!({
        "config": config,
        "userId": user_id,
        "fetchedAtMs": wall_clock_now_ms(),
    }))
    .map_err(|error| format!("serialize Statsig cache: {error}"))?;
    let temp = path.with_extension(format!("{}.tmp", std::process::id()));
    fs::write(&temp, payload)
        .map_err(|error| format!("write Statsig cache: {error}"))?;
    if path.exists() {
        let _ = fs::remove_file(path);
    }
    fs::rename(&temp, path)
        .map_err(|error| format!("commit Statsig cache: {error}"))
}

fn parse_statsig_bootstrap(
    config: &str,
) -> Result<(Option<String>, BTreeMap<String, bool>), String> {
    let parsed = serde_json::from_str::<Value>(config)
        .map_err(|error| format!("parse Statsig config: {error}"))?;
    let user_id = parsed
        .get("user")
        .and_then(Value::as_object)
        .and_then(|user| user.get("userID"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let gates = parsed
        .get("feature_gates")
        .and_then(Value::as_object)
        .map(|gates| {
            gates
                .iter()
                .filter_map(|(name, value)| {
                    statsig_gate_value(value).map(|enabled| (name.clone(), enabled))
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    Ok((user_id, gates))
}

fn statsig_gate_value(value: &Value) -> Option<bool> {
    value.as_bool().or_else(|| {
        value
            .get("value")
            .and_then(|value| value.as_bool().or_else(|| value.get("value").and_then(Value::as_bool)))
    })
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
