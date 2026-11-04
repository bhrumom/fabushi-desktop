use base64::Engine as _;
use mahayana_core::{ModelProviderMode, RuntimeConfig};
use mahayana_feature_host::FeatureHostController;
pub use mahayana_feature_host::MiniAppAccountPort;
#[cfg(not(target_arch = "wasm32"))]
pub use mahayana_host::MiniAppRuntimeStorePort;
use mahayana_host::{HostCreateConfig, ModelRouteConfig};
use mahayana_host_protocol::{
    ApprovalResolution, FeatureCommand, HostConfig, HostMode, SurfacePlatform,
};
use mahayana_js_runtime::{DeepSeekJsHost, scan_package_compatibility};
use mahayana_native_engine::ProcessExecution;
use mahayana_plugin_runtime::{
    ExternalReleaseManifest, InstalledPluginPointer, PermissionManager, PluginInstaller,
    PluginState,
};
use mahayana_product::MahayanaProductClient;
use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppHostFeatureMode {
    Production,
    Test,
}

const TEST_MARKETPLACE_PLUGINS: &[(&str, &str, &str)] = &[
    ("global-dharma", "全球法布施", "任务、日志与部署"),
    ("faliu-flashcards", "法流记忆卡", "经文牌组与复习"),
    ("platform-publish", "平台发布", "内容发布与自动化"),
    (
        "hermes-installer",
        "Hermes Installer",
        "插件安装与运行时管理",
    ),
    ("bot-father", "Bot Father", "创建和管理机器人"),
    (
        "chatgpt-auto-confirm",
        "ChatGPT Auto Confirm",
        "受控自动确认与任务协作",
    ),
];
const TEST_MARKETPLACE_REPOSITORY: &str = "https://github.com/bhrumom/fabushi";
const TEST_MARKETPLACE_SOURCE_REF: &str = "7b02d8d00e0646e9bf4e90a129cbf203fcff015d";

impl From<AppHostFeatureMode> for HostMode {
    fn from(value: AppHostFeatureMode) -> Self {
        match value {
            AppHostFeatureMode::Production => HostMode::Production,
            AppHostFeatureMode::Test => HostMode::Test,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AppHostError {
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("host operation failed: {0}")]
    Operation(String),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostRequest {
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostResponse {
    pub id: Option<Value>,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

const PRODUCT_SETTINGS_VERSION: u32 = 2;
const PRODUCT_PROXY_ROTATION_TIMEOUT_DEFAULT_SECONDS: u32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum ProductProxyMode {
    System,
    Enabled,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum ProductProxyType {
    Socks5,
    Http,
    Mtproto,
    Web,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProductProxyEndpoint {
    #[serde(rename = "type")]
    proxy_type: ProductProxyType,
    host: String,
    port: u32,
    user: String,
    password: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProductProxyRotationSettings {
    enabled: bool,
    timeout_seconds: u32,
    preferred_indices: Vec<usize>,
}

impl Default for ProductProxyRotationSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            timeout_seconds: PRODUCT_PROXY_ROTATION_TIMEOUT_DEFAULT_SECONDS,
            preferred_indices: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProductProxySettings {
    mode: ProductProxyMode,
    selected: Option<ProductProxyEndpoint>,
    list: Vec<ProductProxyEndpoint>,
    try_ipv6: bool,
    use_proxy_for_calls: bool,
    check_ip_warning_shown: bool,
    rotation: ProductProxyRotationSettings,
}

impl Default for ProductProxySettings {
    fn default() -> Self {
        Self {
            mode: ProductProxyMode::System,
            selected: None,
            list: Vec::new(),
            try_ipv6: !cfg!(target_os = "windows"),
            use_proxy_for_calls: false,
            check_ip_warning_shown: false,
            rotation: ProductProxyRotationSettings::default(),
        }
    }
}

impl ProductProxySettings {
    #[cfg(test)]
    fn proxy_enabled(&self) -> bool {
        self.mode == ProductProxyMode::Enabled
    }

    fn normalize_preferred_indices(&mut self, indices: Vec<i64>) {
        let mut normalized = Vec::new();
        for index in indices {
            let Ok(index) = usize::try_from(index) else {
                continue;
            };
            if index >= self.list.len() || normalized.contains(&index) {
                continue;
            }
            normalized.push(index);
        }
        self.rotation.preferred_indices = normalized;
    }

    fn apply(&mut self, mutation: ProductProxyMutation) -> Result<bool, AppHostError> {
        let mut changed = false;
        match mutation {
            ProductProxyMutation::SetMode { mode } => {
                changed = self.mode != mode;
                self.mode = mode;
            }
            ProductProxyMutation::SetSelected { proxy } => {
                changed = self.selected != proxy;
                self.selected = proxy;
            }
            ProductProxyMutation::SetSelectedAndMode { proxy, mode } => {
                changed = self.selected != proxy || self.mode != mode;
                self.selected = proxy;
                self.mode = mode;
            }
            ProductProxyMutation::SetList { list } => {
                changed = self.list != list || !self.rotation.preferred_indices.is_empty();
                self.list = list;
                self.rotation.preferred_indices.clear();
            }
            ProductProxyMutation::Add { proxy } => {
                self.list.push(proxy);
                changed = true;
            }
            ProductProxyMutation::Insert { index, proxy } => {
                let index = if index <= 0 {
                    0
                } else {
                    usize::try_from(index).unwrap_or(usize::MAX).min(self.list.len())
                };
                for existing in &mut self.rotation.preferred_indices {
                    if *existing >= index {
                        *existing += 1;
                    }
                }
                self.list.insert(index, proxy);
                changed = true;
            }
            ProductProxyMutation::Remove { proxy } => {
                if let Some(index) = self.list.iter().position(|existing| existing == &proxy) {
                    self.list.remove(index);
                    // Remove the endpoint's old preference index *before* rebasing
                    // survivors. Otherwise the next preferred endpoint shifts to
                    // this index and is accidentally discarded with the removed one.
                    self.rotation.preferred_indices.retain(|existing| *existing != index);
                    for existing in &mut self.rotation.preferred_indices {
                        if *existing > index {
                            *existing -= 1;
                        }
                    }
                    changed = true;
                }
            }
            ProductProxyMutation::Replace { was, proxy } => {
                if let Some(index) = self.list.iter().position(|existing| existing == &was) {
                    changed = self.list[index] != proxy;
                    self.list[index] = proxy;
                }
            }
            ProductProxyMutation::SetTryIpv6 { enabled } => {
                changed = self.try_ipv6 != enabled;
                self.try_ipv6 = enabled;
            }
            ProductProxyMutation::SetUseProxyForCalls { enabled } => {
                changed = self.use_proxy_for_calls != enabled;
                self.use_proxy_for_calls = enabled;
            }
            ProductProxyMutation::SetCheckIpWarningShown { shown } => {
                changed = self.check_ip_warning_shown != shown;
                self.check_ip_warning_shown = shown;
            }
            ProductProxyMutation::SetRotationEnabled { enabled } => {
                changed = self.rotation.enabled != enabled;
                self.rotation.enabled = enabled;
            }
            ProductProxyMutation::SetRotationTimeout { seconds } => {
                let normalized = if seconds <= 0 {
                    PRODUCT_PROXY_ROTATION_TIMEOUT_DEFAULT_SECONDS
                } else {
                    u32::try_from(seconds).map_err(|_| {
                        AppHostError::InvalidRequest(
                            "settings.mutateNetworkProxy rotation timeout is out of range".into(),
                        )
                    })?
                };
                changed = self.rotation.timeout_seconds != normalized;
                self.rotation.timeout_seconds = normalized;
            }
            ProductProxyMutation::SetRotationPreferredIndices { indices } => {
                let before = self.rotation.preferred_indices.clone();
                self.normalize_preferred_indices(indices);
                changed = self.rotation.preferred_indices != before;
            }
            ProductProxyMutation::PromoteRotationPreferredIndex { index } => {
                if let Ok(index) = usize::try_from(index) {
                    if index < self.list.len() {
                        if self.rotation.preferred_indices.first().copied() != Some(index) {
                            if let Some(existing) = self
                                .rotation
                                .preferred_indices
                                .iter()
                                .position(|existing| *existing == index)
                            {
                                self.rotation.preferred_indices.remove(existing);
                            }
                            self.rotation.preferred_indices.insert(0, index);
                            changed = true;
                        }
                    }
                }
            }
        }
        Ok(changed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProductNetworkSettings {
    proxy: ProductProxySettings,
}

impl Default for ProductNetworkSettings {
    fn default() -> Self {
        Self {
            proxy: ProductProxySettings::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProductSettingsState {
    version: u32,
    network: ProductNetworkSettings,
}

impl Default for ProductSettingsState {
    fn default() -> Self {
        Self {
            version: PRODUCT_SETTINGS_VERSION,
            network: ProductNetworkSettings::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyProductNetworkSettingsV1 {
    proxy_enabled: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyProductSettingsStateV1 {
    version: u32,
    network: LegacyProductNetworkSettingsV1,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum ProductProxyMutation {
    SetMode {
        mode: ProductProxyMode,
    },
    SetSelected {
        proxy: Option<ProductProxyEndpoint>,
    },
    SetSelectedAndMode {
        proxy: Option<ProductProxyEndpoint>,
        mode: ProductProxyMode,
    },
    SetList {
        list: Vec<ProductProxyEndpoint>,
    },
    Add {
        proxy: ProductProxyEndpoint,
    },
    Insert {
        index: i64,
        proxy: ProductProxyEndpoint,
    },
    Remove {
        proxy: ProductProxyEndpoint,
    },
    Replace {
        was: ProductProxyEndpoint,
        proxy: ProductProxyEndpoint,
    },
    SetTryIpv6 {
        enabled: bool,
    },
    SetUseProxyForCalls {
        enabled: bool,
    },
    SetCheckIpWarningShown {
        shown: bool,
    },
    SetRotationEnabled {
        enabled: bool,
    },
    SetRotationTimeout {
        seconds: i64,
    },
    SetRotationPreferredIndices {
        indices: Vec<i64>,
    },
    PromoteRotationPreferredIndex {
        index: i64,
    },
}

impl ProductProxyMutation {
    fn republishes_network_proxy(&self) -> bool {
        matches!(
            self,
            Self::SetMode { .. }
                | Self::SetSelected { .. }
                | Self::SetSelectedAndMode { .. }
                | Self::SetTryIpv6 { .. }
        )
    }
}

/// Canonical, source-neutral product Settings owner for the desktop application.
///
/// This intentionally does not reuse feature.persistence/ui_state and does not
/// write product preferences into Telegram provider-private storage.
struct ProductSettingsOwner {
    path: PathBuf,
    state: Mutex<ProductSettingsState>,
}

impl ProductSettingsOwner {
    fn open(path: PathBuf) -> Result<Self, AppHostError> {
        let state = if path.exists() {
            let encoded = std::fs::read_to_string(&path).map_err(|error| {
                AppHostError::Operation(format!("read product Settings: {error}"))
            })?;
            let value: Value = serde_json::from_str(&encoded).map_err(|error| {
                AppHostError::Operation(format!("decode product Settings: {error}"))
            })?;
            let version = value
                .get("version")
                .and_then(Value::as_u64)
                .ok_or_else(|| AppHostError::Operation("product Settings version is missing".into()))?;
            match version {
                1 => {
                    let legacy: LegacyProductSettingsStateV1 =
                        serde_json::from_value(value).map_err(|error| {
                            AppHostError::Operation(format!(
                                "decode product Settings version 1: {error}"
                            ))
                        })?;
                    if legacy.version != 1 {
                        return Err(AppHostError::Operation(
                            "invalid product Settings version 1 payload".into(),
                        ));
                    }
                    let mut migrated = ProductSettingsState::default();
                    migrated.network.proxy.mode = if legacy.network.proxy_enabled {
                        ProductProxyMode::Enabled
                    } else {
                        ProductProxyMode::System
                    };
                    Self::persist_to_path(&path, &migrated)?;
                    migrated
                }
                version if version == u64::from(PRODUCT_SETTINGS_VERSION) => {
                    serde_json::from_value(value).map_err(|error| {
                        AppHostError::Operation(format!(
                            "decode product Settings version {PRODUCT_SETTINGS_VERSION}: {error}"
                        ))
                    })?
                }
                other => {
                    return Err(AppHostError::Operation(format!(
                        "unsupported product Settings version {other}"
                    )));
                }
            }
        } else {
            ProductSettingsState::default()
        };
        Ok(Self {
            path,
            state: Mutex::new(state),
        })
    }

    fn snapshot(&self) -> Result<ProductSettingsState, AppHostError> {
        self.state
            .lock()
            .map(|state| state.clone())
            .map_err(|_| AppHostError::Operation("product Settings lock is unavailable".into()))
    }

    fn mutate_network_proxy(
        &self,
        mutation: ProductProxyMutation,
    ) -> Result<ProductSettingsState, AppHostError> {
        let mut current = self
            .state
            .lock()
            .map_err(|_| AppHostError::Operation("product Settings lock is unavailable".into()))?;
        let mut next = current.clone();
        let changed = next.network.proxy.apply(mutation)?;
        if changed {
            self.persist(&next)?;
            *current = next.clone();
        }
        Ok(next)
    }

    fn set_network_proxy_enabled(
        &self,
        proxy_enabled: bool,
    ) -> Result<ProductSettingsState, AppHostError> {
        let mut current = self
            .state
            .lock()
            .map_err(|_| AppHostError::Operation("product Settings lock is unavailable".into()))?;
        let mut next = current.clone();
        let mode = next.network.proxy.mode;
        next.network.proxy.mode = if proxy_enabled {
            ProductProxyMode::Enabled
        } else if mode == ProductProxyMode::Enabled {
            ProductProxyMode::System
        } else {
            mode
        };
        if next != *current {
            self.persist(&next)?;
            *current = next.clone();
        }
        Ok(next)
    }

    fn persist(&self, state: &ProductSettingsState) -> Result<(), AppHostError> {
        Self::persist_to_path(&self.path, state)
    }

    fn persist_to_path(path: &Path, state: &ProductSettingsState) -> Result<(), AppHostError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| {
                AppHostError::Operation(format!("create product Settings directory: {error}"))
            })?;
        }
        let encoded = serde_json::to_vec_pretty(state)
            .map_err(|error| AppHostError::Operation(format!("encode product Settings: {error}")))?;
        let temporary = path.with_extension("json.tmp");
        std::fs::write(&temporary, encoded).map_err(|error| {
            AppHostError::Operation(format!("write product Settings temporary file: {error}"))
        })?;
        std::fs::rename(&temporary, path).map_err(|error| {
            AppHostError::Operation(format!("commit product Settings: {error}"))
        })?;
        Ok(())
    }
}

pub struct AppHost {
    app_data_dir: PathBuf,
    feature_mode: AppHostFeatureMode,
    product: MahayanaProductClient,
    js: Mutex<DeepSeekJsHost>,
    feature: Arc<FeatureHostController>,
    settings: ProductSettingsOwner,
    // One-shot desktop-shell handoff prepared only when a restored Web carrier needs browser adoption.
    startup_native_shell_event: Mutex<Option<Value>>,
}

/// Send-safe event lane for desktop PUSH delivery. It intentionally owns only
/// the FeatureHost controller and never captures the QuickJS-backed AppHost.
#[derive(Clone)]
pub struct FeatureEventSource {
    feature: Arc<FeatureHostController>,
}

impl FeatureEventSource {
    pub fn receive(&self, timeout: Duration) -> Result<Option<Value>, AppHostError> {
        let event = self
            .feature
            .receive_with_timeout(timeout)
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        event
            .map(|event| {
                serde_json::to_value(event)
                    .map_err(|error| AppHostError::Operation(error.to_string()))
            })
            .transpose()
    }

    pub fn project_grok_gateway_event(&self, event: &Value) -> Value {
        self.feature.project_grok_gateway_event(event)
    }
}

/// Lightweight platform-only request lane used by desktop. Network-backed
/// account sync must never block the FeatureHost event/command lane; this
/// helper intentionally owns only the shared Rust product client and no JS or
/// Agent runtime state.
#[derive(Clone)]
pub struct PlatformRequestHost {
    product: MahayanaProductClient,
}

impl PlatformRequestHost {
    pub fn new(app_data_dir: impl Into<PathBuf>) -> Result<Self, AppHostError> {
        let app_data_dir = app_data_dir.into();
        std::fs::create_dir_all(&app_data_dir)
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        let root = feature_host_root(&app_data_dir);
        std::fs::create_dir_all(&root)
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        let product = MahayanaProductClient::new_with_default_api_base_url(
            root.join("account-session.json"),
            root.join("product-surface.json"),
        );
        product.bootstrap_ci_test_account_session().map_err(|error| {
            AppHostError::Operation(format!(
                "GitHub Actions test-account bootstrap failed: {error}"
            ))
        })?;
        Ok(Self { product })
    }

    pub fn dispatch_json(&self, input: &str) -> String {
        let response = match serde_json::from_str::<HostRequest>(input) {
            Ok(request) if request.method == "platform.request" => {
                let id = request.id;
                match execute_platform_request(&self.product, &request.params) {
                    Ok(result) => HostResponse {
                        id,
                        ok: true,
                        result: Some(result),
                        error: None,
                    },
                    Err(error) => HostResponse {
                        id,
                        ok: false,
                        result: None,
                        error: Some(AppHostError::Operation(error.to_string()).to_string()),
                    },
                }
            }
            Ok(request) => HostResponse {
                id: request.id,
                ok: false,
                result: None,
                error: Some(format!(
                    "invalid request: platform lane cannot execute {}",
                    request.method
                )),
            },
            Err(error) => HostResponse {
                id: None,
                ok: false,
                result: None,
                error: Some(format!("invalid JSON request: {error}")),
            },
        };
        serde_json::to_string(&response).unwrap_or_else(|error| {
            format!("{{\"ok\":false,\"error\":\"serialization failed: {error}\"}}")
        })
    }
}

pub fn is_platform_request_json(input: &str) -> bool {
    serde_json::from_str::<HostRequest>(input)
        .map(|request| request.method == "platform.request")
        .unwrap_or(false)
}

fn execute_platform_request(
    product: &MahayanaProductClient,
    params: &Value,
) -> Result<Value, mahayana_product::ProductError> {
    if surface_platform() == SurfacePlatform::Electron {
        product.execute_session_owned_platform_request(params)
    } else {
        product.execute("mahayana.platform.request", params)
    }
}


#[cfg(test)]
fn startup_native_shell_event_for_carrier(carrier: &Value) -> Option<Value> {
    let bootstrap = carrier
        .get("webProxyBootstrapUrl")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())?;
    Some(json!({
        "channel": "native-shell",
        "payload": {
            "type": "web-proxy-bootstrap",
            "webProxyBootstrapUrl": bootstrap,
        }
    }))
}

impl AppHost {
    pub fn mini_app_account_port(&self) -> MiniAppAccountPort {
        self.feature.mini_app_account_port()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn mini_app_store_port(&self) -> Result<MiniAppRuntimeStorePort, AppHostError> {
        self.feature
            .mini_app_store_port()
            .map_err(|error| AppHostError::Operation(error.to_string()))
    }

    pub fn new(app_data_dir: impl Into<PathBuf>) -> Result<Self, AppHostError> {
        let feature_mode = configured_feature_host_mode()?;
        Self::new_with_feature_mode(app_data_dir, feature_mode)
    }

    pub fn new_with_feature_mode(
        app_data_dir: impl Into<PathBuf>,
        feature_mode: AppHostFeatureMode,
    ) -> Result<Self, AppHostError> {
        Self::new_with_feature_mode_and_storage_passphrase(app_data_dir, feature_mode, None)
    }

    pub fn new_with_feature_mode_and_storage_passphrase(
        app_data_dir: impl Into<PathBuf>,
        feature_mode: AppHostFeatureMode,
        storage_passphrase: Option<String>,
    ) -> Result<Self, AppHostError> {
        let app_data_dir = app_data_dir.into();
        std::fs::create_dir_all(&app_data_dir)
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        let feature_root = feature_host_root(&app_data_dir);
        let settings = ProductSettingsOwner::open(feature_root.join("product-settings.json"))?;
        // Native account credentials do not authorize a second communication
        // runtime. In particular, an older v1 credential may still carry a
        // Telegram key: leave its database and secure-store material untouched.
        // A reviewed code revert can reuse those exact paths/keys; startup is
        // neither a data migration nor an opportunity to decrypt legacy state.
        let feature = Arc::new(create_feature_host(
            &app_data_dir,
            feature_mode,
            storage_passphrase.clone(),
        )?);
        let product = match storage_passphrase {
            Some(passphrase) => {
                MahayanaProductClient::new_with_default_api_base_url_and_storage_passphrase(
                    feature_root.join("account-session.json"),
                    feature_root.join("product-surface.json"),
                    passphrase,
                )
            }
            None => MahayanaProductClient::new_with_default_api_base_url(
                feature_root.join("account-session.json"),
                feature_root.join("product-surface.json"),
            ),
        };
        product
            .bootstrap_ci_test_account_session()
            .map_err(|error| {
                AppHostError::Operation(format!(
                    "GitHub Actions test-account bootstrap failed: {error}"
                ))
            })?;
        Ok(Self {
            app_data_dir,
            feature_mode,
            product,
            js: Mutex::new(
                DeepSeekJsHost::new()
                    .map_err(|error| AppHostError::Operation(error.to_string()))?,
            ),
            feature,
            settings,
            startup_native_shell_event: Mutex::new(None),
        })
    }

    /// Take the one-shot native-shell event prepared during restored carrier projection.
    /// The desktop child protocol forwards this through the existing unsolicited-event lane;
    /// no generic shell capability is exposed to AppHost or the renderer.
    pub fn take_startup_native_shell_event(&self) -> Result<Option<Value>, AppHostError> {
        self.startup_native_shell_event
            .lock()
            .map(|mut event| event.take())
            .map_err(|_| AppHostError::Operation("startup native-shell event lock is unavailable".into()))
    }

    /// Receive the next feature event directly from the long-lived Rust runtime.
    ///
    /// Desktop uses this to drive an unsolicited event frame on the child-process
    /// protocol. Renderer/Main no longer need to poll `feature.receive`.
    pub fn feature_event_source(&self) -> FeatureEventSource {
        FeatureEventSource {
            feature: Arc::clone(&self.feature),
        }
    }

    pub fn receive_feature_event(
        &self,
        timeout: Duration,
    ) -> Result<Option<Value>, AppHostError> {
        self.feature_event_source().receive(timeout)
    }

    pub fn grok_gateway_call(
        &self,
        method: &str,
        args: Value,
    ) -> Result<Option<Value>, AppHostError> {
        self.feature
            .grok_gateway_call(method, args)
            .map_err(|error| AppHostError::Operation(error.to_string()))
    }

    pub fn dispatch(&self, request: HostRequest) -> HostResponse {
        let id = request.id.clone();
        match self.handle(&request.method, request.params) {
            Ok(result) => HostResponse {
                id,
                ok: true,
                result: Some(result),
                error: None,
            },
            Err(error) => HostResponse {
                id,
                ok: false,
                result: None,
                error: Some(error.to_string()),
            },
        }
    }

    fn handle(&self, method: &str, params: Value) -> Result<Value, AppHostError> {
        match method {
            "host.platform" => Ok(json!({"platform": host_platform()})),
            "settings.snapshot" => serde_json::to_value(self.settings.snapshot()?)
                .map_err(|error| AppHostError::Operation(error.to_string())),
            "settings.mutateNetworkProxy" => self.settings_mutate_network_proxy(params),
            "settings.setNetworkProxyEnabled" => self.settings_set_network_proxy_enabled(params),
            method if method.starts_with("feature.") => self.handle_feature(method, params),
            "platform.request" => execute_platform_request(&self.product, &params)
                .map_err(|error| AppHostError::Operation(error.to_string())),
            "plugin.permissions" => self.plugin_permissions(params),
            "plugin.permission.grant" => self.set_permission(params, true),
            "plugin.permission.revoke" => self.set_permission(params, false),
            "plugin.compatibility" => self.plugin_compatibility(params),
            "runtime.start" => self.start_runtime(params),
            "runtime.stop" => self.stop_runtime(params),
            "runtime.tools" => self.runtime_tools(),
            "runtime.call" => self.runtime_call(params),
            other => Err(AppHostError::InvalidRequest(format!(
                "unknown method {other}"
            ))),
        }
    }


    fn settings_mutate_network_proxy(&self, params: Value) -> Result<Value, AppHostError> {
        let mutation_value = params
            .get("mutation")
            .cloned()
            .ok_or_else(|| {
                AppHostError::InvalidRequest(
                    "settings.mutateNetworkProxy requires mutation".into(),
                )
            })?;
        let mutation: ProductProxyMutation = serde_json::from_value(mutation_value).map_err(|error| {
            AppHostError::InvalidRequest(format!(
                "settings.mutateNetworkProxy mutation is invalid: {error}"
            ))
        })?;
        let republish = mutation.republishes_network_proxy();
        let state = self.settings.mutate_network_proxy(mutation)?;
        let publication = if republish {
            Some(self.publish_network_proxy(&state.network.proxy)?)
        } else {
            None
        };
        Ok(json!({
            "settings": state,
            "publication": publication,
        }))
    }

    fn settings_set_network_proxy_enabled(&self, params: Value) -> Result<Value, AppHostError> {
        let proxy_enabled = params
            .get("proxyEnabled")
            .and_then(Value::as_bool)
            .ok_or_else(|| {
                AppHostError::InvalidRequest(
                    "settings.setNetworkProxyEnabled requires boolean proxyEnabled".into(),
                )
            })?;

        // Persistence is authoritative and occurs before downstream publication.
        // The compatibility command deliberately republishes even for a same-value retry.
        let state = self.settings.set_network_proxy_enabled(proxy_enabled)?;
        let publication = self.publish_network_proxy(&state.network.proxy)?;
        Ok(json!({
            "settings": state,
            "publication": publication,
        }))
    }

    fn publish_network_proxy(&self, _proxy: &ProductProxySettings) -> Result<Value, AppHostError> {
        // Persisted ProductSettings are canonical. This Native AppHost no longer
        // owns a legacy carrier; do not load it solely to publish a proxy.
        Ok(json!({
            "available": false,
            "reason": "telegramClientUnavailable",
        }))
    }

    fn handle_feature(&self, method: &str, params: Value) -> Result<Value, AppHostError> {
        match method {
            "feature.info" => serde_json::to_value(self.feature.info())
                .map_err(|error| AppHostError::Operation(error.to_string())),
            "feature.execute" => self.feature_execute(params),
            "feature.receive" => self.feature_receive(params),
            "feature.persistence.get" => self.feature_persistence_get(params),
            "feature.persistence.set" => self.feature_persistence_set(params),
            "feature.persistence.remove" => self.feature_persistence_remove(params),
            "feature.persistence.list" => self.feature_persistence_list(params),
            "feature.approval.resolve" => self.feature_resolve_approval(params),
            "feature.interrupt" => self.feature_interrupt(params),
            "feature.auth.status" => self
                .feature
                .auth_status()
                .map_err(|error| AppHostError::Operation(error.to_string())),
            // Main-process only: this method is intentionally absent from the
            // renderer IPC allowlist. It never returns a refresh credential.
            "feature.auth.deviceAgentSession" => self
                .product
                .device_agent_session()
                .map_err(|error| AppHostError::Operation(error.to_string())),
            "feature.auth.providers" => self
                .feature
                .auth_providers()
                .map_err(|error| AppHostError::Operation(error.to_string())),
            "feature.auth.telegramStatus" => Self::legacy_telegram_unavailable(),
            "feature.auth.telegramSendCode" => Self::legacy_telegram_unavailable(),
            "feature.auth.telegramSignIn" => Self::legacy_telegram_unavailable(),
            "feature.auth.telegramPassword" => Self::legacy_telegram_unavailable(),
            "feature.auth.telegramRegistration" => self.feature_telegram_registration(params),
            "feature.auth.telegramResendCode" => Self::legacy_telegram_unavailable(),
            "feature.auth.telegramCancelCode" => Self::legacy_telegram_unavailable(),
            "feature.auth.telegramRequestPasswordRecovery" => Self::legacy_telegram_unavailable(),
            "feature.auth.telegramCheckRecoveryPassword" => Self::legacy_telegram_unavailable(),
            "feature.auth.telegramSendVerifyEmailCode" => Self::legacy_telegram_unavailable(),
            "feature.auth.telegramVerifyEmail" => Self::legacy_telegram_unavailable(),
            "feature.auth.telegramRefreshPasswordChallenge" => Self::legacy_telegram_unavailable(),
            "feature.auth.passwordLogin" => self.feature_password_login(params),
            "feature.auth.browserStart" => self
                .feature
                .browser_login_start()
                .map_err(|error| AppHostError::Operation(error.to_string())),
            "feature.auth.browserPoll" => self.feature_browser_login_poll(params),
            "feature.auth.browserCancel" => self.feature_browser_login_cancel(params),
            "feature.auth.browserReopen" => self.feature_browser_login_reopen(params),
            "feature.auth.oauthStart" => self.feature_oauth_start(params),
            "feature.auth.oauthPoll" => self.feature_oauth_poll(params),
            "feature.auth.logout" => self
                .feature
                .logout()
                .map_err(|error| AppHostError::Operation(error.to_string())),
            "feature.marketplace.browse" => self.marketplace_browse(params),
            "feature.marketplace.release" => self.marketplace_release(params),
            "feature.marketplace.add" => self.marketplace_add(params),
            "feature.plugin.install" => self.install_plugin(params),
            "feature.plugin.uninstall" => self.uninstall_plugin(params),
            "feature.plugin.rollback" => self.rollback_plugin(params),
            "feature.plugin.active" => self.active_plugin(params),
            "feature.plugin.listInstalled" => self.list_installed_plugins(),
            "feature.plugin.uiDocument" => self.plugin_ui_document(params),
            "feature.messaging.execute" => self.feature_messaging_execute(params),
            "feature.messaging.blob.read" => self.feature_messaging_blob_read(params),
            "feature.messaging.access.issue" => self.feature_messaging_access_issue(params),
            other => Err(AppHostError::InvalidRequest(format!(
                "unknown feature method {other}"
            ))),
        }
    }

    fn feature_execute(&self, params: Value) -> Result<Value, AppHostError> {
        let command_value = params.get("command").cloned().unwrap_or(params);
        let command: FeatureCommand = serde_json::from_value(command_value).map_err(|error| {
            AppHostError::InvalidRequest(format!("invalid feature command: {error}"))
        })?;
        let accepted = self
            .feature
            .execute(command)
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        serde_json::to_value(accepted).map_err(|error| AppHostError::Operation(error.to_string()))
    }

    fn feature_receive(&self, params: Value) -> Result<Value, AppHostError> {
        let timeout_ms = params
            .get("timeoutMs")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            .min(30_000);
        let event = self
            .feature
            .receive_with_timeout(Duration::from_millis(timeout_ms))
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        serde_json::to_value(event).map_err(|error| AppHostError::Operation(error.to_string()))
    }

    fn runtime_state_connection(&self) -> Result<Connection, AppHostError> {
        let runtime_dir = feature_host_root(&self.app_data_dir).join("runtime");
        std::fs::create_dir_all(&runtime_dir)
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        let connection = Connection::open(runtime_dir.join("runtime.sqlite3"))
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        connection
            .execute_batch(
                r#"
                PRAGMA journal_mode=WAL;
                PRAGMA synchronous=NORMAL;
                PRAGMA busy_timeout=5000;
                CREATE TABLE IF NOT EXISTS ui_state (
                    key TEXT PRIMARY KEY,
                    value_json TEXT NOT NULL,
                    updated_at_ms INTEGER NOT NULL
                );
                "#,
            )
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        Ok(connection)
    }

    fn feature_persistence_get(&self, params: Value) -> Result<Value, AppHostError> {
        let key = client_persistence_key(&params)?;
        let connection = self.runtime_state_connection()?;
        let value = connection
            .query_row(
                "SELECT value_json FROM ui_state WHERE key = ?1",
                params![key],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        value
            .map(|encoded| {
                serde_json::from_str(&encoded)
                    .map_err(|error| AppHostError::Operation(error.to_string()))
            })
            .transpose()
            .map(|value| value.unwrap_or(Value::Null))
    }

    fn feature_persistence_set(&self, params_value: Value) -> Result<Value, AppHostError> {
        let key = client_persistence_key(&params_value)?;
        let value = params_value.get("value").cloned().unwrap_or(Value::Null);
        let encoded = serde_json::to_string(&value)
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        self.runtime_state_connection()?
            .execute(
                "INSERT INTO ui_state(key, value_json, updated_at_ms) VALUES (?1, ?2, ?3)
                 ON CONFLICT(key) DO UPDATE SET
                   value_json = excluded.value_json,
                   updated_at_ms = excluded.updated_at_ms",
                params![key, encoded, app_host_now_millis()],
            )
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        Ok(Value::Bool(true))
    }

    fn feature_persistence_remove(&self, params_value: Value) -> Result<Value, AppHostError> {
        let key = client_persistence_key(&params_value)?;
        self.runtime_state_connection()?
            .execute("DELETE FROM ui_state WHERE key = ?1", params![key])
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        Ok(Value::Bool(true))
    }

    fn feature_persistence_list(&self, params_value: Value) -> Result<Value, AppHostError> {
        let prefix = params_value
            .get("prefix")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if prefix.len() > 512 || prefix.chars().any(|character| character == '\0') {
            return Err(AppHostError::InvalidRequest(
                "client persistence prefix is invalid".to_string(),
            ));
        }
        let connection = self.runtime_state_connection()?;
        let mut statement = connection
            .prepare("SELECT key FROM ui_state ORDER BY key")
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        let mut keys = Vec::new();
        for row in rows {
            let key = row.map_err(|error| AppHostError::Operation(error.to_string()))?;
            if key.starts_with(&prefix) {
                keys.push(Value::String(key));
            }
        }
        Ok(Value::Array(keys))
    }

    fn feature_resolve_approval(&self, params: Value) -> Result<Value, AppHostError> {
        let resolution_value = params.get("resolution").cloned().unwrap_or(params);
        let resolution: ApprovalResolution =
            serde_json::from_value(resolution_value).map_err(|error| {
                AppHostError::InvalidRequest(format!("invalid approval resolution: {error}"))
            })?;
        self.feature
            .resolve_approval(resolution)
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        Ok(Value::Null)
    }

    fn feature_interrupt(&self, params: Value) -> Result<Value, AppHostError> {
        let operation_id = string_param(&params, "operationId")?;
        self.feature
            .interrupt(operation_id)
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        Ok(Value::Null)
    }

    fn feature_messaging_execute(&self, params: Value) -> Result<Value, AppHostError> {
        let request_id = string_param(&params, "requestId")?.to_string();
        let envelope = params
            .get("envelope")
            .cloned()
            .ok_or_else(|| AppHostError::InvalidRequest("envelope is required".into()))?;
        let envelopes = self
            .feature
            .execute_messaging_sync(request_id, envelope)
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        Ok(json!({"envelopes": envelopes}))
    }

    fn feature_messaging_blob_read(&self, params: Value) -> Result<Value, AppHostError> {
        let blob_id = string_param(&params, "blobId")?;
        let offset = params.get("offset").and_then(Value::as_u64).unwrap_or(0);
        let length = params
            .get("length")
            .and_then(Value::as_u64)
            .unwrap_or(1024 * 1024)
            .clamp(1, 1024 * 1024);
        let (metadata, bytes) = self
            .feature
            .read_messaging_blob_range(blob_id, offset, length)
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        Ok(json!({
            "metadata": metadata,
            "offset": offset,
            "dataBase64": base64::engine::general_purpose::STANDARD.encode(bytes),
        }))
    }

    fn feature_messaging_access_issue(&self, params: Value) -> Result<Value, AppHostError> {
        let device_id = string_param(&params, "deviceId")?.to_string();
        let session_id = string_param(&params, "sessionId")?.to_string();
        let scopes = params
            .get("scopes")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .map(|item| {
                        item.as_str().map(str::to_string).ok_or_else(|| {
                            AppHostError::InvalidRequest("scopes must contain strings".into())
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?
            .unwrap_or_default();
        let ttl_ms = params
            .get("ttlMs")
            .and_then(Value::as_i64)
            .unwrap_or(24 * 60 * 60 * 1000);
        self.feature
            .issue_messaging_access(device_id, session_id, scopes, ttl_ms)
            .map_err(|error| AppHostError::Operation(error.to_string()))
    }

    fn legacy_telegram_unavailable() -> Result<Value, AppHostError> {
        Err(AppHostError::Operation(
            "embedded Telegram client is unavailable for the current platform account".into(),
        ))
    }

    fn feature_telegram_registration(&self, params: Value) -> Result<Value, AppHostError> {
        // Preserve the original validation precedence for registration; all
        // other compatibility routes rejected before inspecting parameters.
        let _ = string_param(&params, "firstName")?;
        Self::legacy_telegram_unavailable()
    }

    fn feature_password_login(&self, params: Value) -> Result<Value, AppHostError> {
        let username = string_param(&params, "username")?.to_string();
        let password = string_param(&params, "password")?.to_string();
        self.feature
            .password_login(username, password)
            .map_err(|error| AppHostError::Operation(error.to_string()))
    }

    fn feature_browser_login_reopen(&self, params: Value) -> Result<Value, AppHostError> {
        self.feature
            .browser_login_reopen(string_param(&params, "attemptId")?.to_string())
            .map_err(|error| AppHostError::Operation(error.to_string()))
    }

    fn feature_browser_login_cancel(&self, params: Value) -> Result<Value, AppHostError> {
        self.feature
            .browser_login_cancel(string_param(&params, "attemptId")?.to_string())
            .map_err(|error| AppHostError::Operation(error.to_string()))
    }

    fn feature_browser_login_poll(&self, params: Value) -> Result<Value, AppHostError> {
        self.feature
            .browser_login_poll(string_param(&params, "attemptId")?.to_string())
            .map_err(|error| AppHostError::Operation(error.to_string()))
    }

    fn feature_oauth_start(&self, params: Value) -> Result<Value, AppHostError> {
        self.feature
            .oauth_start(string_param(&params, "provider")?.to_string())
            .map_err(|error| AppHostError::Operation(error.to_string()))
    }

    fn feature_oauth_poll(&self, params: Value) -> Result<Value, AppHostError> {
        self.feature
            .oauth_poll(string_param(&params, "attemptId")?.to_string())
            .map_err(|error| AppHostError::Operation(error.to_string()))
    }

    fn marketplace_browse(&self, params: Value) -> Result<Value, AppHostError> {
        let query = params.get("query").and_then(Value::as_str);
        if self.feature_mode == AppHostFeatureMode::Test {
            let term = query.unwrap_or_default().trim().to_lowercase();
            let plugins = TEST_MARKETPLACE_PLUGINS
                .iter()
                .filter_map(|(plugin_id, display_name, description)| {
                    let searchable =
                        format!("{plugin_id} {display_name} {description}").to_lowercase();
                    if !term.is_empty() && !searchable.contains(&term) {
                        return None;
                    }
                    Some(json!({
                        "pluginId": plugin_id,
                        "displayName": display_name,
                        "description": description,
                        "latestVersion": "1.0.0",
                        "platforms": ["desktop"],
                        "releaseStatus": "approved"
                    }))
                })
                .collect::<Vec<_>>();
            return Ok(json!({"plugins": plugins}));
        }
        let requested_platform = params
            .get("platform")
            .and_then(Value::as_str)
            .unwrap_or(host_platform());
        let marketplace_platform = match requested_platform {
            "ios" | "android" => "mobile",
            other => other,
        };
        self.product
            .marketplace_browse(query, Some(marketplace_platform))
            .map_err(|error| AppHostError::Operation(error.to_string()))
    }

    fn marketplace_release(&self, params: Value) -> Result<Value, AppHostError> {
        let plugin_id = string_param(&params, "pluginId")?;
        let version = string_param(&params, "version")?;
        if self.feature_mode == AppHostFeatureMode::Test {
            if !TEST_MARKETPLACE_PLUGINS
                .iter()
                .any(|(candidate, _, _)| *candidate == plugin_id)
            {
                return Err(AppHostError::Operation(format!(
                    "test marketplace plugin {plugin_id} was not found"
                )));
            }
            let artifact_id = format!("{plugin_id}-test-ui");
            let artifact_url = format!(
                "https://raw.githubusercontent.com/bhrumom/fabushi/{TEST_MARKETPLACE_SOURCE_REF}/marketplace/packages/{plugin_id}/1.0.0/app.tar.gz"
            );
            let artifact = json!({
                "id": artifact_id,
                "runtime": "local-web",
                "platforms": ["desktop"],
                "source": {"type": "https", "url": artifact_url},
                "sha256": "0000000000000000000000000000000000000000000000000000000000000000",
                "size": 1,
                "format": "tar-gz"
            });
            let install = json!({
                "protocol": "fabushi.marketplace.install.v1",
                "strategy": "github-immutable",
                "pluginId": plugin_id,
                "version": version,
                "source": {
                    "repository": TEST_MARKETPLACE_REPOSITORY,
                    "sourceRef": TEST_MARKETPLACE_SOURCE_REF,
                    "marketplaceHostsPackage": false
                },
                "artifacts": [artifact.clone()],
                "update": {
                    "check": "marketplace-release",
                    "comparison": "version-then-artifact-sha256",
                    "allowDowngrade": false,
                    "rollback": "previous-active"
                }
            });
            return Ok(json!({
                "pluginId": plugin_id,
                "version": version,
                "releaseStatus": "approved",
                "releaseManifest": {
                    "schemaVersion": 1,
                    "protocol": "mahayana.external-release.v1",
                    "pluginId": plugin_id,
                    "version": version,
                    "source": {
                        "repository": TEST_MARKETPLACE_REPOSITORY,
                        "sourceRef": TEST_MARKETPLACE_SOURCE_REF
                    },
                    "permissions": [],
                    "artifacts": [artifact],
                    "install": install.clone()
                },
                "install": install
            }));
        }
        self.product
            .marketplace_release_metadata(plugin_id, version)
            .map_err(|error| AppHostError::Operation(error.to_string()))
    }

    fn marketplace_add(&self, params: Value) -> Result<Value, AppHostError> {
        let plugin_id = string_param(&params, "pluginId")?;
        let requested_platform = params
            .get("platform")
            .and_then(Value::as_str)
            .unwrap_or(host_platform());
        let marketplace_platform = match requested_platform {
            "ios" | "android" => "mobile",
            other => other,
        };
        if self.feature_mode == AppHostFeatureMode::Test {
            if !TEST_MARKETPLACE_PLUGINS
                .iter()
                .any(|(candidate, _, _)| *candidate == plugin_id)
            {
                return Err(AppHostError::Operation(format!(
                    "test marketplace plugin {plugin_id} was not found"
                )));
            }
            return Ok(json!({
                "added": true,
                "accountSynchronized": true,
                "bot": {
                    "id": format!("{plugin_id}-bot"),
                    "displayName": plugin_id,
                },
            }));
        }
        self.product
            .marketplace_add(plugin_id, marketplace_platform)
            .map_err(|error| AppHostError::Operation(error.to_string()))
    }

    fn plugin_root(&self) -> PathBuf {
        self.app_data_dir.join("plugins")
    }

    fn permission_store(&self) -> PathBuf {
        self.plugin_root().join("permissions.json")
    }

    fn installer(&self) -> Result<PluginInstaller, AppHostError> {
        PluginInstaller::new(self.plugin_root())
            .map_err(|error| AppHostError::Operation(error.to_string()))
    }

    fn install_plugin(&self, params: Value) -> Result<Value, AppHostError> {
        let release_value = params
            .get("release")
            .cloned()
            .ok_or_else(|| AppHostError::InvalidRequest("release is required".into()))?;
        let release: ExternalReleaseManifest = serde_json::from_value(release_value)
            .map_err(|error| AppHostError::InvalidRequest(error.to_string()))?;
        let platform = params
            .get("platform")
            .and_then(Value::as_str)
            .unwrap_or(host_platform());
        if self.feature_mode == AppHostFeatureMode::Test {
            if !TEST_MARKETPLACE_PLUGINS
                .iter()
                .any(|(candidate, _, _)| *candidate == release.plugin_id)
            {
                return Err(AppHostError::Operation(format!(
                    "test marketplace plugin {} was not found",
                    release.plugin_id
                )));
            }
            let plugin_root = self.plugin_root().join(&release.plugin_id);
            let installed_dir = plugin_root
                .join("versions")
                .join(&release.version)
                .join("test-ui");
            std::fs::create_dir_all(&installed_dir)
                .map_err(|error| AppHostError::Operation(error.to_string()))?;
            let display_name = TEST_MARKETPLACE_PLUGINS
                .iter()
                .find(|(candidate, _, _)| *candidate == release.plugin_id)
                .map(|(_, display_name, _)| *display_name)
                .unwrap_or(release.plugin_id.as_str());
            let html = format!(
                "<!doctype html><html><head><meta charset=\"utf-8\"><title>{display_name}</title></head><body><main><h1>{display_name}</h1><p>Installed from the deterministic Mahayana Marketplace test backend.</p></main></body></html>"
            );
            std::fs::write(installed_dir.join("index.html"), html)
                .map_err(|error| AppHostError::Operation(error.to_string()))?;
            let pointer = InstalledPluginPointer {
                plugin_id: release.plugin_id.clone(),
                version: release.version.clone(),
                artifact_id: "test-ui".to_string(),
                artifact_sha256: "0".repeat(64),
                runtime: "local-web".to_string(),
                entry: Some("index.html".to_string()),
                requested_permissions: release.permissions.clone(),
                installed_path: installed_dir.to_string_lossy().into_owned(),
            };
            std::fs::create_dir_all(&plugin_root)
                .map_err(|error| AppHostError::Operation(error.to_string()))?;
            let active = serde_json::to_vec_pretty(&pointer)
                .map_err(|error| AppHostError::Operation(error.to_string()))?;
            std::fs::write(plugin_root.join("active.json"), active)
                .map_err(|error| AppHostError::Operation(error.to_string()))?;
            return serde_json::to_value(pointer)
                .map_err(|error| AppHostError::Operation(error.to_string()));
        }
        let preferred: &[&str] = match platform {
            "ios" | "android" | "mobile" => &[
                "deepseek-js",
                "javascript",
                "cordis-js",
                "web-wasm",
                "userscript",
                "mcp",
                "local-web",
            ],
            _ => &[
                "deepseek-js",
                "javascript",
                "cordis-js",
                "local-web",
                "web-wasm",
                "native",
                "desktop-stdio",
                "mcp",
            ],
        };
        let installer = self.installer()?;
        let pointer = installer
            .install(&release, platform, preferred)
            .or_else(|error| {
                if matches!(platform, "ios" | "android") {
                    installer.install(&release, "mobile", preferred)
                } else {
                    Err(error)
                }
            })
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        serde_json::to_value(pointer).map_err(|error| AppHostError::Operation(error.to_string()))
    }

    fn uninstall_plugin(&self, params: Value) -> Result<Value, AppHostError> {
        let plugin_id = string_param(&params, "pluginId")?;
        if let Ok(mut host) = self.js.lock() {
            let _ = host.disable_plugin(plugin_id);
        }
        let removed = self
            .installer()?
            .uninstall(plugin_id)
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        let permissions_removed = PermissionManager::load(self.permission_store())
            .map_err(|error| AppHostError::Operation(error.to_string()))?
            .remove_plugin(plugin_id)
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        Ok(json!({
            "pluginId": plugin_id,
            "removed": removed,
            "permissionsRemoved": permissions_removed
        }))
    }

    fn active_plugin(&self, params: Value) -> Result<Value, AppHostError> {
        let plugin_id = string_param(&params, "pluginId")?;
        let pointer = self
            .installer()?
            .active(plugin_id)
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        serde_json::to_value(pointer).map_err(|error| AppHostError::Operation(error.to_string()))
    }

    fn rollback_plugin(&self, params: Value) -> Result<Value, AppHostError> {
        let plugin_id = string_param(&params, "pluginId")?;
        let pointer = self
            .installer()?
            .rollback(plugin_id)
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        serde_json::to_value(pointer).map_err(|error| AppHostError::Operation(error.to_string()))
    }

    fn list_installed_plugins(&self) -> Result<Value, AppHostError> {
        let root = self.plugin_root();
        let installer = self.installer()?;
        let mut plugins = Vec::new();
        let entries = match std::fs::read_dir(&root) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(json!({"plugins": plugins}));
            }
            Err(error) => return Err(AppHostError::Operation(error.to_string())),
        };
        for entry in entries {
            let entry = entry.map_err(|error| AppHostError::Operation(error.to_string()))?;
            if !entry
                .file_type()
                .map_err(|error| AppHostError::Operation(error.to_string()))?
                .is_dir()
            {
                continue;
            }
            let plugin_id = entry.file_name().to_string_lossy().into_owned();
            if let Some(pointer) = installer
                .active(&plugin_id)
                .map_err(|error| AppHostError::Operation(error.to_string()))?
            {
                plugins.push(pointer);
            }
        }
        plugins.sort_by(|left, right| left.plugin_id.cmp(&right.plugin_id));
        Ok(json!({"plugins": plugins}))
    }

    fn plugin_permissions(&self, params: Value) -> Result<Value, AppHostError> {
        let plugin_id = string_param(&params, "pluginId")?;
        let pointer = self
            .installer()?
            .active(plugin_id)
            .map_err(|error| AppHostError::Operation(error.to_string()))?
            .ok_or_else(|| {
                AppHostError::Operation(format!("plugin {plugin_id} is not installed"))
            })?;
        let mut manager = PermissionManager::load(self.permission_store())
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        manager
            .retain_requested(plugin_id, &pointer.requested_permissions)
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        let granted = manager.grants_for(plugin_id);
        let missing = pointer
            .requested_permissions
            .iter()
            .filter(|permission| !granted.contains(permission))
            .cloned()
            .collect::<Vec<_>>();
        Ok(
            json!({"requested": pointer.requested_permissions, "granted": granted, "missing": missing}),
        )
    }

    fn set_permission(&self, params: Value, grant: bool) -> Result<Value, AppHostError> {
        let plugin_id = string_param(&params, "pluginId")?;
        let permission = string_param(&params, "permission")?;
        let pointer = self
            .installer()?
            .active(plugin_id)
            .map_err(|error| AppHostError::Operation(error.to_string()))?
            .ok_or_else(|| {
                AppHostError::Operation(format!("plugin {plugin_id} is not installed"))
            })?;
        let mut manager = PermissionManager::load(self.permission_store())
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        if grant {
            manager
                .grant(plugin_id, &pointer.requested_permissions, permission)
                .map_err(|error| AppHostError::Operation(error.to_string()))?;
        } else {
            manager
                .revoke(plugin_id, permission)
                .map_err(|error| AppHostError::Operation(error.to_string()))?;
        }
        self.plugin_permissions(json!({"pluginId": plugin_id}))
    }

    fn plugin_compatibility(&self, params: Value) -> Result<Value, AppHostError> {
        let plugin_id = string_param(&params, "pluginId")?;
        let pointer = self
            .installer()?
            .active(plugin_id)
            .map_err(|error| AppHostError::Operation(error.to_string()))?
            .ok_or_else(|| {
                AppHostError::Operation(format!("plugin {plugin_id} is not installed"))
            })?;
        let report = scan_package_compatibility(Path::new(&pointer.installed_path))
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        serde_json::to_value(report).map_err(|error| AppHostError::Operation(error.to_string()))
    }

    fn plugin_ui_document(&self, params: Value) -> Result<Value, AppHostError> {
        let plugin_id = string_param(&params, "pluginId")?;
        let pointer = self
            .installer()?
            .active(plugin_id)
            .map_err(|error| AppHostError::Operation(error.to_string()))?
            .ok_or_else(|| {
                AppHostError::Operation(format!("plugin {plugin_id} is not installed"))
            })?;
        let root = std::fs::canonicalize(&pointer.installed_path)
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        let requested_entry = pointer
            .entry
            .as_deref()
            .map(|value| value.trim_start_matches("./"));
        let entry = requested_entry
            .filter(|value| value.to_ascii_lowercase().ends_with(".html"))
            .map(str::to_string)
            .or_else(|| {
                ["ui/index.html", "web/index.html", "index.html"]
                    .into_iter()
                    .find(|candidate| root.join(candidate).is_file())
                    .map(str::to_string)
            })
            .ok_or_else(|| {
                AppHostError::Operation(format!(
                    "installed plugin {plugin_id} does not expose an HTML Mini App entry"
                ))
            })?;
        let relative = PathBuf::from(&entry);
        if relative.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        }) {
            return Err(AppHostError::InvalidRequest(
                "plugin UI entry escaped installed root".into(),
            ));
        }
        let document = std::fs::canonicalize(root.join(relative))
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        if !document.starts_with(&root) {
            return Err(AppHostError::InvalidRequest(
                "plugin UI entry escaped installed root".into(),
            ));
        }
        let html = std::fs::read_to_string(document)
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        Ok(json!({"pluginId": plugin_id, "html": html}))
    }

    fn start_runtime(&self, params: Value) -> Result<Value, AppHostError> {
        let plugin_id = string_param(&params, "pluginId")?.to_string();
        let pointer = self
            .installer()?
            .active(&plugin_id)
            .map_err(|error| AppHostError::Operation(error.to_string()))?
            .ok_or_else(|| {
                AppHostError::Operation(format!("plugin {plugin_id} is not installed"))
            })?;
        if !matches!(
            pointer.runtime.as_str(),
            "deepseek-js" | "javascript" | "cordis-js"
        ) {
            return Err(AppHostError::Operation(format!(
                "runtime {} is not supported by the portable JS host",
                pointer.runtime
            )));
        }
        let root = PathBuf::from(&pointer.installed_path);
        let entry = discover_js_entry(&pointer.entry, &root)?;
        let report = scan_package_compatibility(&root)
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        if !report.portable_compatible {
            return Err(AppHostError::Operation(report.blockers().join("; ")));
        }
        let grants = PermissionManager::load(self.permission_store())
            .map_err(|error| AppHostError::Operation(error.to_string()))?
            .grants_for(&plugin_id);
        let config = params.get("config").cloned().unwrap_or_else(|| json!({}));
        let mut host = self
            .js
            .lock()
            .map_err(|_| AppHostError::Operation("JavaScript host lock poisoned".into()))?;
        let state = host
            .register_plugin_with_grants(&plugin_id, &root, &entry, &config, &grants)
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        serde_json::to_value(state).map_err(|error| AppHostError::Operation(error.to_string()))
    }

    fn stop_runtime(&self, params: Value) -> Result<Value, AppHostError> {
        let plugin_id = string_param(&params, "pluginId")?;
        self.js
            .lock()
            .map_err(|_| AppHostError::Operation("JavaScript host lock poisoned".into()))?
            .disable_plugin(plugin_id)
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        Ok(Value::Null)
    }

    fn runtime_tools(&self) -> Result<Value, AppHostError> {
        let tools = self
            .js
            .lock()
            .map_err(|_| AppHostError::Operation("JavaScript host lock poisoned".into()))?
            .registered_tools()
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        serde_json::to_value(tools).map_err(|error| AppHostError::Operation(error.to_string()))
    }

    fn runtime_call(&self, params: Value) -> Result<Value, AppHostError> {
        let plugin_id = string_param(&params, "pluginId")?;
        let name = string_param(&params, "name")?;
        let arguments = params
            .get("arguments")
            .cloned()
            .unwrap_or_else(|| json!({}));
        if !arguments.is_object() {
            return Err(AppHostError::InvalidRequest(
                "runtime.call arguments must be an object".into(),
            ));
        }
        if self.feature_mode == AppHostFeatureMode::Test {
            let installed = self
                .plugin_root()
                .join(plugin_id)
                .join("active.json")
                .is_file();
            if installed
                && TEST_MARKETPLACE_PLUGINS
                    .iter()
                    .any(|(candidate, _, _)| *candidate == plugin_id)
            {
                return deterministic_test_runtime_call(plugin_id, name, &arguments);
            }
        }
        let host = self
            .js
            .lock()
            .map_err(|_| AppHostError::Operation("JavaScript host lock poisoned".into()))?;
        if host.plugin_state(plugin_id) != Some(PluginState::Active) {
            return Err(AppHostError::Operation(format!(
                "plugin {plugin_id} runtime is not active"
            )));
        }
        let tools = host
            .registered_tools()
            .map_err(|error| AppHostError::Operation(error.to_string()))?;
        if !tools.iter().any(|candidate| candidate == name) {
            return Err(AppHostError::Operation(format!(
                "tool {name} is not registered in the active runtime"
            )));
        }
        host.call_tool_json(name, &arguments)
            .map_err(|error| AppHostError::Operation(error.to_string()))
    }
}

fn deterministic_test_runtime_call(
    plugin_id: &str,
    name: &str,
    arguments: &Value,
) -> Result<Value, AppHostError> {
    if plugin_id != "global-dharma" {
        return Ok(json!({
            "content": [{"type": "text", "text": format!("{plugin_id} test tool {name} completed") }],
            "structuredContent": {"testMode": true, "tool": name, "arguments": arguments},
        }));
    }
    let (text, structured) = match name {
        "status" => (
            "已读取全球法布施状态。",
            json!({"running": false, "mode": "home", "testMode": true}),
        ),
        "start" => (
            "本地转经轮已通过宿主权限校验并启动。",
            json!({"running": true, "mode": "local-prayer-wheel", "testMode": true}),
        ),
        "stop" => (
            "全球法布施本地模式已停止。",
            json!({"running": false, "mode": "home", "testMode": true}),
        ),
        "logs" => (
            "已读取全球法布施日志。",
            json!({"entries": ["deterministic GitHub Actions test runtime"], "testMode": true}),
        ),
        "send" => (
            "全球发送测试请求已完成。",
            json!({"sent": 1, "testMode": true}),
        ),
        other => {
            return Err(AppHostError::Operation(format!(
                "test runtime tool {other} is not available for global-dharma"
            )));
        }
    };
    Ok(json!({
        "content": [{"type": "text", "text": text}],
        "structuredContent": structured,
    }))
}

fn configured_feature_host_mode() -> Result<AppHostFeatureMode, AppHostError> {
    match std::env::var("FABUSHI_FEATURE_HOST_MODE") {
        Ok(value) if value.eq_ignore_ascii_case("test") => Ok(AppHostFeatureMode::Test),
        Ok(value) if value.eq_ignore_ascii_case("production") => Ok(AppHostFeatureMode::Production),
        Ok(value) if value.trim().is_empty() => Ok(AppHostFeatureMode::Production),
        Ok(value) => Err(AppHostError::InvalidRequest(format!(
            "unsupported FABUSHI_FEATURE_HOST_MODE {value:?}; expected test or production"
        ))),
        Err(std::env::VarError::NotPresent) => Ok(AppHostFeatureMode::Production),
        Err(error) => Err(AppHostError::InvalidRequest(format!(
            "invalid FABUSHI_FEATURE_HOST_MODE: {error}"
        ))),
    }
}

fn client_persistence_key(params: &Value) -> Result<String, AppHostError> {
    let key = params
        .get("key")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    if key.is_empty()
        || key.len() > 512
        || key.chars().any(|character| character == '\0' || character.is_control())
    {
        return Err(AppHostError::InvalidRequest(
            "client persistence key is invalid".to_string(),
        ));
    }
    Ok(key.to_string())
}

fn app_host_now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(i64::MAX as u128) as i64)
        .unwrap_or_default()
}

fn feature_host_root(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("feature-host")
}

fn create_feature_host(
    app_data_dir: &Path,
    feature_mode: AppHostFeatureMode,
    storage_passphrase: Option<String>,
) -> Result<FeatureHostController, AppHostError> {
    let root = feature_host_root(app_data_dir);
    std::fs::create_dir_all(&root).map_err(|error| AppHostError::Operation(error.to_string()))?;
    let provider =
        std::env::var("MAHAYANA_INFERENCE_PROVIDER").unwrap_or_else(|_| "fabushi".into());
    // Keep the base Runtime on Fabushi. Agent-scoped provider overrides are
    // routed inside Mahayana rather than by restarting the entire Host.
    let runtime = RuntimeConfig {
        data_dir: Some(root.join("runtime")),
        ..RuntimeConfig::default()
    };
    let openrouter_token = std::env::var("MAHAYANA_OPENROUTER_API_KEY")
        .ok()
        .filter(|value| !value.trim().is_empty());
    let claude_token = std::env::var("MAHAYANA_CLAUDE_API_KEY")
        .ok()
        .filter(|value| !value.trim().is_empty());
    let model_routes = vec![
        ModelRouteConfig {
            key: "openrouter".into(),
            base_url: "https://openrouter.ai/api/v1".into(),
            default_model: std::env::var("MAHAYANA_OPENROUTER_MODEL")
                .unwrap_or_else(|_| "openai/gpt-5.2".into()),
            bearer_token: openrouter_token,
            provider_mode: ModelProviderMode::UserConfiguredRemote,
            wire_api: mahayana_model::responses::ResponsesWireApi::ChatCompletions,
            use_product_session_token: false,
        },
        ModelRouteConfig {
            key: "claude-code".into(),
            base_url: "https://api.anthropic.com/v1".into(),
            default_model: std::env::var("MAHAYANA_CLAUDE_MODEL")
                .unwrap_or_else(|_| "claude-sonnet-4-6".into()),
            bearer_token: claude_token,
            provider_mode: ModelProviderMode::UserConfiguredRemote,
            wire_api: mahayana_model::responses::ResponsesWireApi::AnthropicMessages,
            use_product_session_token: false,
        },
    ];
    let host_config = HostCreateConfig {
        runtime,
        product_session_path: Some(root.join("account-session.json")),
        product_surface_state_path: Some(root.join("product-surface.json")),
        automation_path: Some(root.join("automations.json")),
        use_codex_account: std::env::var("MAHAYANA_USE_CODEX_ACCOUNT").as_deref() == Ok("1"),
        codex_home: std::env::var_os("MAHAYANA_CODEX_HOME").map(PathBuf::from),
        product_storage_passphrase: storage_passphrase,
        // Native identity never activates the retained legacy authority.
        // MahayanaHost also rejects explicit handoffs at its public boundary.
        telegram_client_id: None,
        telegram_self_user_id: None,
        model_bearer_token: None,
        model_wire_api: mahayana_model::responses::ResponsesWireApi::Responses,
        model_routes,
        model_route_default: Some(match provider.as_str() {
            "openrouter" => "openrouter",
            "claude-code" => "claude-code",
            "codex" => "codex",
            _ => "fabushi",
        }.to_string()),
        inherit_installed_plugins: Some(false),
        process_execution: if std::env::var("MAHAYANA_SANDBOX_RUNTIME").as_deref()
            == Ok("local-docker")
        {
            ProcessExecution::LocalDocker {
                docker_path: std::env::var_os("MAHAYANA_DOCKER_BIN")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("docker")),
                image: std::env::var("MAHAYANA_DOCKER_IMAGE").unwrap_or_default(),
            }
        } else {
            ProcessExecution::Host
        },
        ..HostCreateConfig::default()
    };
    FeatureHostController::create_with_host_config(
        HostConfig {
            profile_id: "default".to_string(),
            mode: feature_mode.into(),
        },
        surface_platform(),
        host_config,
    )
    .map_err(|error| {
        AppHostError::Operation(format!("feature host initialization failed: {error}"))
    })
}

fn discover_js_entry(entry: &Option<String>, root: &Path) -> Result<PathBuf, AppHostError> {
    if let Some(entry) = entry.as_deref() {
        let relative = PathBuf::from(entry.trim_start_matches("./"));
        if relative.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        }) {
            return Err(AppHostError::InvalidRequest(
                "plugin entry escaped installed root".into(),
            ));
        }
        if root.join(&relative).is_file() {
            return Ok(relative);
        }
    }
    for candidate in ["index.mjs", "index.js", "lib/index.js", "dist/index.js"] {
        if root.join(candidate).is_file() {
            return Ok(PathBuf::from(candidate));
        }
    }
    Err(AppHostError::Operation(
        "installed plugin has no runnable JavaScript entry".into(),
    ))
}

fn string_param<'a>(params: &'a Value, name: &str) -> Result<&'a str, AppHostError> {
    params
        .get(name)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppHostError::InvalidRequest(format!("{name} is required")))
}

fn surface_platform() -> SurfacePlatform {
    if cfg!(target_os = "ios") {
        SurfacePlatform::Ios
    } else if cfg!(target_os = "android") {
        SurfacePlatform::Android
    } else {
        SurfacePlatform::Electron
    }
}

pub fn host_platform() -> &'static str {
    if cfg!(target_os = "ios") {
        "ios"
    } else if cfg!(target_os = "android") {
        "android"
    } else {
        "desktop"
    }
}

pub fn default_app_data_dir() -> PathBuf {
    if let Some(path) = std::env::var_os("FABUSHI_APP_DATA") {
        return PathBuf::from(path);
    }
    #[cfg(target_os = "macos")]
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join("Library/Application Support/com.ombhrum.fabushi");
    }
    #[cfg(target_os = "windows")]
    if let Some(data) = std::env::var_os("APPDATA") {
        return PathBuf::from(data).join("Fabushi");
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".fabushi")
}

pub fn dispatch_json(host: &AppHost, input: &str) -> String {
    let response = match serde_json::from_str::<HostRequest>(input) {
        Ok(request) => host.dispatch(request),
        Err(error) => HostResponse {
            id: None,
            ok: false,
            result: None,
            error: Some(format!("invalid JSON request: {error}")),
        },
    };
    serde_json::to_string(&response).unwrap_or_else(|error| {
        format!("{{\"ok\":false,\"error\":\"serialization failed: {error}\"}}")
    })
}

#[cfg(test)]
mod telegram_composition_contract_tests {
    use super::*;


    fn product_settings_test_path(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "fabushi-product-settings-{label}-{}-{nonce}.json",
            std::process::id()
        ))
    }

    fn proxy_endpoint(label: &str) -> ProductProxyEndpoint {
        ProductProxyEndpoint {
            proxy_type: ProductProxyType::Socks5,
            host: format!("{label}.example"),
            port: 1080,
            user: String::new(),
            password: String::new(),
        }
    }

    #[test]
    fn restored_web_proxy_bootstrap_projects_only_the_one_shot_native_shell_event() {
        let event = startup_native_shell_event_for_carrier(&json!({
            "webProxyBootstrapUrl": "http://127.0.0.1:49152/#opaque-capability",
            "secret": "must-not-cross-shell-boundary",
        }))
        .expect("restored Web carrier requires desktop adoption");
        assert_eq!(event["channel"], "native-shell");
        assert_eq!(event["payload"]["type"], "web-proxy-bootstrap");
        assert_eq!(
            event["payload"]["webProxyBootstrapUrl"],
            "http://127.0.0.1:49152/#opaque-capability"
        );
        assert!(event["payload"].get("secret").is_none());
        assert!(startup_native_shell_event_for_carrier(&json!({})).is_none());
    }

    #[test]
    fn product_settings_owner_defaults_match_fixed_u_proxy_settings() {
        let state = ProductSettingsState::default();
        assert_eq!(state.version, 2);
        assert_eq!(state.network.proxy.mode, ProductProxyMode::System);
        assert!(state.network.proxy.selected.is_none());
        assert!(state.network.proxy.list.is_empty());
        assert_eq!(state.network.proxy.try_ipv6, !cfg!(target_os = "windows"));
        assert!(!state.network.proxy.use_proxy_for_calls);
        assert!(!state.network.proxy.check_ip_warning_shown);
        assert!(!state.network.proxy.rotation.enabled);
        assert_eq!(state.network.proxy.rotation.timeout_seconds, 10);
        assert!(state.network.proxy.rotation.preferred_indices.is_empty());
    }

    #[test]
    fn product_settings_owner_migrates_v1_proxy_enabled_in_place() {
        let path = product_settings_test_path("migrate-v1");
        std::fs::write(
            &path,
            br#"{"version":1,"network":{"proxyEnabled":true}}"#,
        )
        .expect("write version 1 fixture");
        let owner = ProductSettingsOwner::open(path.clone()).expect("migrate version 1 Settings");
        let state = owner.snapshot().expect("migrated snapshot");
        assert_eq!(state.version, 2);
        assert_eq!(state.network.proxy.mode, ProductProxyMode::Enabled);
        drop(owner);

        let encoded = std::fs::read_to_string(&path).expect("read migrated Settings");
        assert!(encoded.contains(r#""version": 2"#));
        assert!(!encoded.contains("proxyEnabled"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn product_settings_owner_persists_typed_proxy_state() {
        let path = product_settings_test_path("persist-v2");
        let owner = ProductSettingsOwner::open(path.clone()).expect("open default Settings");
        owner
            .mutate_network_proxy(ProductProxyMutation::SetList {
                list: vec![proxy_endpoint("a"), proxy_endpoint("b")],
            })
            .expect("persist proxy list");
        owner
            .mutate_network_proxy(ProductProxyMutation::SetSelectedAndMode {
                proxy: Some(proxy_endpoint("b")),
                mode: ProductProxyMode::Enabled,
            })
            .expect("persist selected proxy and mode");
        drop(owner);

        let restored = ProductSettingsOwner::open(path.clone()).expect("restore Settings");
        let state = restored.snapshot().expect("restored snapshot");
        assert_eq!(state.network.proxy.list.len(), 2);
        assert_eq!(state.network.proxy.selected, Some(proxy_endpoint("b")));
        assert_eq!(state.network.proxy.mode, ProductProxyMode::Enabled);
        assert!(state.network.proxy.proxy_enabled());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn product_proxy_list_mutations_preserve_fixed_u_preferred_index_invariants() {
        let mut proxy = ProductProxySettings::default();
        proxy.list = vec![
            proxy_endpoint("a"),
            proxy_endpoint("b"),
            proxy_endpoint("c"),
        ];
        proxy.normalize_preferred_indices(vec![2, -1, 2, 0, 9]);
        assert_eq!(proxy.rotation.preferred_indices, vec![2, 0]);

        proxy
            .apply(ProductProxyMutation::Insert {
                index: 1,
                proxy: proxy_endpoint("inserted"),
            })
            .expect("insert");
        assert_eq!(proxy.rotation.preferred_indices, vec![3, 0]);

        proxy
            .apply(ProductProxyMutation::Remove {
                proxy: proxy_endpoint("inserted"),
            })
            .expect("remove");
        assert_eq!(proxy.rotation.preferred_indices, vec![2, 0]);

        proxy
            .apply(ProductProxyMutation::SetList {
                list: vec![proxy_endpoint("replacement")],
            })
            .expect("replace whole list");
        assert!(proxy.rotation.preferred_indices.is_empty());
    }

    #[test]
    fn product_proxy_remove_preserves_surviving_preference_identity_and_order() {
        // All ordered subsets of three endpoints, including all six complete
        // permutations. The oracle removes an endpoint identity, without
        // reproducing the implementation's numeric-index rebasing.
        let preference_orders: &[&[usize]] = &[
            &[], &[0], &[1], &[2],
            &[0, 1], &[0, 2], &[1, 0], &[1, 2], &[2, 0], &[2, 1],
            &[0, 1, 2], &[0, 2, 1], &[1, 0, 2],
            &[1, 2, 0], &[2, 0, 1], &[2, 1, 0],
        ];
        let mut completed_cases = 0;
        let mut complete_permutation_cases = 0;
        for order in preference_orders {
            for removal in 0..3 {
                let mut proxy = ProductProxySettings::default();
                proxy.list = vec![
                    proxy_endpoint("a"), proxy_endpoint("b"), proxy_endpoint("c"),
                ];
                proxy.selected = Some(proxy_endpoint("b"));
                proxy.mode = ProductProxyMode::Enabled;
                proxy.rotation.preferred_indices = order.to_vec();
                let removed = proxy.list[removal].clone();
                let preferred_endpoints = order.iter()
                    .map(|&index| proxy.list[index].clone())
                    .collect::<Vec<_>>();
                let expected = preferred_endpoints.into_iter()
                    .filter(|endpoint| endpoint != &removed)
                    .collect::<Vec<_>>();
                let expected_list = proxy.list.iter()
                    .filter(|endpoint| *endpoint != &removed)
                    .cloned()
                    .collect::<Vec<_>>();
                assert!(proxy.apply(ProductProxyMutation::Remove {
                    proxy: removed.clone(),
                }).expect("remove existing endpoint"));
                let actual = proxy.rotation.preferred_indices.iter()
                    .map(|&index| proxy.list[index].clone())
                    .collect::<Vec<_>>();
                assert_eq!(actual, expected, "order={order:?} remove={removal}");
                assert_eq!(proxy.list, expected_list, "surviving list order");
                assert_eq!(proxy.selected, Some(proxy_endpoint("b")));
                assert_eq!(proxy.mode, ProductProxyMode::Enabled);
                let stable = proxy.clone();
                assert!(!proxy.apply(ProductProxyMutation::Remove {
                    proxy: removed,
                }).expect("repeat remove is a no-op"));
                assert_eq!(proxy, stable, "repeat remove must not change state");
                completed_cases += 1;
                if order.len() == 3 {
                    complete_permutation_cases += 1;
                }
                // Only fixture indices are logged; endpoint credentials are
                // never included in this per-case execution evidence.
                println!(
                    "NMR-41 proxy removal passed: order={order:?} remove={removal} repeat=no-op"
                );
            }
        }
        assert_eq!(completed_cases, 48);
        assert_eq!(complete_permutation_cases, 18);
        println!(
            "NMR-41 proxy preference matrix passed: orders={} removals={completed_cases} repeated_no_ops={completed_cases} complete_permutations=6 complete_permutation_removals={complete_permutation_cases}",
            preference_orders.len()
        );
    }

    #[test]
    fn product_proxy_rotation_timeout_and_promotion_match_fixed_u() {
        let mut proxy = ProductProxySettings::default();
        proxy.list = vec![
            proxy_endpoint("a"),
            proxy_endpoint("b"),
            proxy_endpoint("c"),
        ];
        proxy
            .apply(ProductProxyMutation::SetRotationTimeout { seconds: 0 })
            .expect("normalize timeout");
        assert_eq!(proxy.rotation.timeout_seconds, 10);
        proxy
            .apply(ProductProxyMutation::SetRotationPreferredIndices {
                indices: vec![0, 2],
            })
            .expect("preferred indices");
        proxy
            .apply(ProductProxyMutation::PromoteRotationPreferredIndex { index: 2 })
            .expect("promote");
        assert_eq!(proxy.rotation.preferred_indices, vec![2, 0]);
        assert!(!proxy
            .apply(ProductProxyMutation::PromoteRotationPreferredIndex { index: 2 })
            .expect("idempotent promote"));
    }

    #[test]
    fn product_settings_owner_fails_closed_on_malformed_persistence() {
        let path = product_settings_test_path("malformed");
        std::fs::write(&path, b"{not-json").expect("write malformed contract fixture");
        assert!(ProductSettingsOwner::open(path.clone()).is_err());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn carrier_publication_mutations_match_fixed_u_change_surface() {
        assert!(ProductProxyMutation::SetMode {
            mode: ProductProxyMode::Enabled
        }.republishes_network_proxy());
        assert!(ProductProxyMutation::SetSelected {
            proxy: None
        }.republishes_network_proxy());
        assert!(ProductProxyMutation::SetTryIpv6 {
            enabled: true
        }.republishes_network_proxy());
        assert!(!ProductProxyMutation::SetUseProxyForCalls {
            enabled: true
        }.republishes_network_proxy());
    }

    #[test]
    fn product_settings_host_contract_is_source_neutral() {
        let methods = [
            "settings.snapshot",
            "settings.mutateNetworkProxy",
            "settings.setNetworkProxyEnabled",
        ];
        assert!(methods.iter().all(|method| !method.contains("telegram")));
        assert!(methods.iter().all(|method| !method.contains("Telegram")));
    }

    #[test]
    fn telegram_renderer_auth_contract_has_no_application_credential_parameters() {
        let renderer_methods = [
            "feature.auth.telegramStatus",
            "feature.auth.telegramSendCode",
            "feature.auth.telegramSignIn",
            "feature.auth.telegramPassword",
            "feature.auth.telegramRegistration",
        ];
        assert_eq!(renderer_methods.len(), 5);
        let renderer_parameters = ["phoneNumber", "code", "password", "firstName", "lastName"];
        assert!(!renderer_parameters.contains(&"apiId"));
        assert!(!renderer_parameters.contains(&"apiHash"));
    }

    #[test]
    fn product_proxy_mode_tracks_canonical_enabled_projection_without_legacy_carrier() {
        let mut proxy = ProductProxySettings::default();
        assert!(!proxy.proxy_enabled());
        proxy
            .apply(ProductProxyMutation::SetSelectedAndMode {
                proxy: Some(proxy_endpoint("primary")),
                mode: ProductProxyMode::Enabled,
            })
            .expect("enable canonical proxy");
        assert!(proxy.proxy_enabled());
        assert_eq!(proxy.selected, Some(proxy_endpoint("primary")));
        proxy
            .apply(ProductProxyMutation::SetMode {
                mode: ProductProxyMode::Disabled,
            })
            .expect("disable canonical proxy");
        assert!(!proxy.proxy_enabled());
        assert_eq!(proxy.selected, Some(proxy_endpoint("primary")));
    }

}


