//! Product-level feature controller over the direct Mahayana Runtime Host.
//!
//! `HostMode::Test` is a deterministic in-process backend for fast E2E. It uses
//! real Rust commands, state, approvals, event ordering, and lifecycle without
//! network or simulator dependencies. Production provider mappings are added
//! feature-by-feature and must never silently fall back to test behavior.

use base64::Engine as _;
use chrono::Datelike;
use chrono::NaiveDate;
use chrono::TimeZone;
use chrono::Timelike;
use chrono::Utc;

use fabushi_messaging_core::BlobId;
use fabushi_messaging_core::ClientEnvelope as MessagingClientEnvelope;
use fabushi_messaging_core::FileBlobStore;
use fabushi_messaging_core::JsonFileStateStore;
use fabushi_messaging_core::MessagingService;
use fabushi_messaging_core::{AccessGrant, AccessScope, ActorId, FileAccessTokenStore};
#[cfg(feature = "production")]
use mahayana_core::ApprovalDecision as RuntimeApprovalDecision;
#[cfg(feature = "production")]
use mahayana_core::ApprovalId;
#[cfg(feature = "production")]
use mahayana_core::ConversationId;
use mahayana_core::MAHAYANA_AI_CONVERSATION_ID;
#[cfg(feature = "production")]
use mahayana_core::MessageRole as RuntimeMessageRole;
#[cfg(feature = "production")]
use mahayana_core::OperationId;
#[cfg(feature = "production")]
use mahayana_core::RuntimeActivityStatus;
#[cfg(feature = "production")]
use mahayana_core::RuntimeCommand;
#[cfg(feature = "production")]
use mahayana_core::RuntimeEvent;
#[cfg(feature = "production")]
use mahayana_core::RuntimeResponse;
#[cfg(feature = "production")]
use mahayana_core::capability::CapabilityAvailability;
#[cfg(feature = "production")]
use mahayana_core::capability::CapabilityKind;
#[cfg(feature = "production")]
use mahayana_core::capability::CapabilityPolicyDecision;
#[cfg(feature = "production")]
use mahayana_core::capability::CapabilityRequest;
#[cfg(feature = "production")]
use mahayana_host::HostCreateConfig;
#[cfg(feature = "production")]
use mahayana_host::MahayanaHost;
use mahayana_host_protocol::AgentBroadcastResult;
use mahayana_host_protocol::AgentMode;
use mahayana_host_protocol::AgentPeerMessage;
use mahayana_host_protocol::AgentStepStatus;
#[cfg(feature = "production")]
use mahayana_host_protocol::ApprovalDecision;
use mahayana_host_protocol::ApprovalResolution;
use mahayana_host_protocol::AsyncTaskKind;
use mahayana_host_protocol::AsyncTaskStatus;
use mahayana_host_protocol::AsyncTaskSummary;
use mahayana_host_protocol::AttachmentChunkResult;
use mahayana_host_protocol::AttachmentContext;
use mahayana_host_protocol::AttachmentImageResult;
use mahayana_host_protocol::AttachmentStored;
use mahayana_host_protocol::AttachmentTextResult;
use mahayana_host_protocol::AutoReviewBehavior;
use mahayana_host_protocol::AutoReviewRule;
use mahayana_host_protocol::AutomationSummary;
use mahayana_host_protocol::AutomationTrigger;
use mahayana_host_protocol::BotSummary;
use mahayana_host_protocol::COMPUTER_CONTROL_PROTOCOL_VERSION;
use mahayana_host_protocol::CapabilitySummary;
use mahayana_host_protocol::CommandAccepted;
use mahayana_host_protocol::ComputerActionResult;
use mahayana_host_protocol::ComputerControlOrigin;
use mahayana_host_protocol::ComputerControlTarget;
use mahayana_host_protocol::ComputerSnapshot;
use mahayana_host_protocol::ComputerTargetKind;
use mahayana_host_protocol::ConnectorAccountSummary;
use mahayana_host_protocol::ConnectorStatus;
use mahayana_host_protocol::ConnectorSummary;
use mahayana_host_protocol::ConnectorToolSummary;
use mahayana_host_protocol::ConnectorTransport;
use mahayana_host_protocol::ConversationMessage;
use mahayana_host_protocol::ConversationSummary;
use mahayana_host_protocol::DraftAction;
use mahayana_host_protocol::DraftSendState;
use mahayana_host_protocol::ErrorTray;
use mahayana_host_protocol::EventCard;
use mahayana_host_protocol::EventField;
use mahayana_host_protocol::FeatureCommand;
use mahayana_host_protocol::GroupMessage;
use mahayana_host_protocol::GroupSpeaker;
use mahayana_host_protocol::GroupSummary;
use mahayana_host_protocol::HOST_PROTOCOL_VERSION;
use mahayana_host_protocol::HostConfig;
use mahayana_host_protocol::HostEvent;
use mahayana_host_protocol::HostInfo;
use mahayana_host_protocol::HostMode;
use mahayana_host_protocol::InferenceProvider;
use mahayana_host_protocol::ListenerIntegrationSummary;
use mahayana_host_protocol::ListenerPlatform;
use mahayana_host_protocol::LocalToolPermission;
use mahayana_host_protocol::MemoryKind;
use mahayana_host_protocol::MemoryRecord;
use mahayana_host_protocol::MessageDraft;
use mahayana_host_protocol::MessageRole;
use mahayana_host_protocol::ProductHostSettings;
use mahayana_host_protocol::SearchMediaMatch;
use mahayana_host_protocol::SearchMessageMatch;
use mahayana_host_protocol::SkillPublishState;
use mahayana_host_protocol::SkillSource;
use mahayana_host_protocol::SkillSummary;
use mahayana_host_protocol::SkillTeamSummary;
use mahayana_host_protocol::SubagentStatus;
use mahayana_host_protocol::SubagentSummary;
use mahayana_host_protocol::SurfacePlatform;
use mahayana_host_protocol::TEACH_MAX_DURATION_MS;
use mahayana_host_protocol::TeachEntryPoint;
use mahayana_host_protocol::TeachRecordingResult;
use mahayana_host_protocol::TeachRecordingStatus;
use mahayana_host_protocol::TranscriptCard;
use mahayana_host_protocol::TurnLifecycleState;
use mahayana_host_protocol::UpdateState;
use mahayana_host_protocol::WorkflowSource;
use mahayana_host_protocol::WorkflowSummary;
use mahayana_host_protocol::WorkflowTrigger;
#[cfg(feature = "production")]
use serde::de::DeserializeOwned;
use serde_json::Value;
use serde_json::json;
use sha2::Digest as _;
use sha2::Sha256;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::OnceLock;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum FeatureHostError {
    #[cfg(feature = "production")]
    #[error(transparent)]
    Runtime(#[from] mahayana_host::HostError),
    #[error("production Runtime support is not compiled into this Host")]
    ProductionUnavailable,
    #[error("feature Host state mutex is poisoned")]
    StatePoisoned,
    #[error("feature Host is closed")]
    Closed,
    #[error("{0}")]
    Contract(String),
}

#[cfg_attr(not(feature = "production"), allow(dead_code))]
#[derive(Debug)]
struct PendingApproval {
    mini_app_id: String,
    capability: String,
    runtime_approval_id: Option<String>,
    operation_id: Option<String>,
    agent_id: Option<String>,
}

const GROUP_MAX_MEMBER_TURNS: usize = 10;
const GROUP_MAX_ROUNDS: usize = 3;
const GROUP_PROMPT_HISTORY_LIMIT: usize = 24;
const REMOTE_DEVICE_SECRET_MAX_ENTRIES: usize = 256;
const REMOTE_DEVICE_SECRET_MAX_BYTES: u64 = 256 * 1024;

#[derive(Debug, Clone)]
struct GroupRunState {
    run_id: String,
    round: usize,
    speaker_order: Vec<String>,
    speaker_index: usize,
    total_messages: usize,
    messages_this_round: usize,
}

#[derive(Debug, Clone)]
struct GroupOperationContext {
    run_id: String,
    group_id: String,
    member_id: String,
    member_name: String,
}

#[derive(Debug, Clone)]
struct BackgroundOperationContext {
    agent_id: String,
    agent_name: String,
    source: String,
    teach_artifact: Option<String>,
}

#[derive(Debug, Clone)]
struct RemoteComputerLocalSession {
    device_id: String,
    client_id: String,
    expires_at_seconds: i64,
    generation: u64,
}

#[derive(Debug)]
struct TeachCaptureProcess {
    agent_id: String,
    entry_point: TeachEntryPoint,
    started_at_ms: i64,
    session_dir: PathBuf,
    video_path: PathBuf,
    child: Option<std::process::Child>,
}

enum MemoryAction {
    List { limit: usize },
    Add { content: String, kind: MemoryKind },
    Remove { id: String },
    Clear,
}

#[derive(Debug)]
struct FeatureState {
    events: VecDeque<HostEvent>,
    installed: BTreeMap<String, String>,
    pending_approvals: BTreeMap<String, PendingApproval>,
    operations: BTreeSet<String>,
    operation_agents: BTreeMap<String, String>,
    background_operations: BTreeMap<String, BackgroundOperationContext>,
    remote_computer_sessions: BTreeMap<String, RemoteComputerLocalSession>,
    remote_computer_device_secrets: BTreeMap<String, String>,
    subagents: BTreeMap<String, SubagentSummary>,
    async_tasks: BTreeMap<String, AsyncTaskSummary>,
    peer_messages: Vec<AgentPeerMessage>,
    workspace_state: BTreeMap<String, Value>,
    settings: ProductHostSettings,
    trays: Vec<ErrorTray>,
    sequence: u64,
    closed: bool,
    session_active: bool,
    auth_user: Option<Value>,
    automations: BTreeMap<String, AutomationSummary>,
    connectors: BTreeMap<String, ConnectorSummary>,
    skills: BTreeMap<String, SkillSummary>,
    bots: BTreeMap<String, BotSummary>,
    groups: BTreeMap<String, GroupSummary>,
    group_runs: BTreeMap<String, GroupRunState>,
    group_operations: BTreeMap<String, GroupOperationContext>,
    listeners: BTreeMap<ListenerPlatform, ListenerIntegrationSummary>,
    update_state: UpdateState,
}

impl Default for FeatureState {
    fn default() -> Self {
        Self {
            events: VecDeque::new(),
            installed: BTreeMap::new(),
            pending_approvals: BTreeMap::new(),
            operations: BTreeSet::new(),
            operation_agents: BTreeMap::new(),
            background_operations: BTreeMap::new(),
            remote_computer_sessions: BTreeMap::new(),
            remote_computer_device_secrets: BTreeMap::new(),
            subagents: BTreeMap::new(),
            async_tasks: BTreeMap::new(),
            peer_messages: Vec::new(),
            workspace_state: BTreeMap::new(),
            settings: ProductHostSettings::default(),
            trays: Vec::new(),
            sequence: 0,
            closed: false,
            session_active: true,
            auth_user: None,
            automations: BTreeMap::new(),
            connectors: default_connectors(),
            skills: default_skills(),
            bots: default_bots(),
            groups: BTreeMap::new(),
            group_runs: BTreeMap::new(),
            group_operations: BTreeMap::new(),
            listeners: default_listeners(),
            update_state: UpdateState::UpToDate {
                version: env!("CARGO_PKG_VERSION").into(),
            },
        }
    }
}

pub struct FeatureHostController {
    config: HostConfig,
    info: HostInfo,
    #[cfg(feature = "production")]
    runtime: Option<MahayanaHost>,
    automation_path: Option<PathBuf>,
    bot_state_path: Option<PathBuf>,
    group_state_path: Option<PathBuf>,
    peer_messages_path: Option<PathBuf>,
    settings_path: Option<PathBuf>,
    remote_device_state_path: Option<PathBuf>,
    test_auth_state_path: Option<PathBuf>,
    memory_root_path: Option<PathBuf>,
    workflow_root_path: Option<PathBuf>,
    teach_recording: Mutex<Option<TeachCaptureProcess>>,
    /// The authenticated account currently owning in-memory feature state.
    /// This is deliberately not a credential; it is only an identity marker
    /// used to prevent transcript/state reuse across account boundaries.
    active_account_id: Mutex<Option<String>>,
    state: Mutex<FeatureState>,
}

impl FeatureHostController {
    pub fn create(config: HostConfig, platform: SurfacePlatform) -> Result<Self, FeatureHostError> {
        validate_config(&config)?;
        match config.mode {
            HostMode::Test => Ok(Self::create_test_backend(config, platform, None)),
            HostMode::Production => {
                #[cfg(feature = "production")]
                {
                    Self::create_with_host_config(config, platform, HostCreateConfig::default())
                }
                #[cfg(not(feature = "production"))]
                {
                    Err(FeatureHostError::ProductionUnavailable)
                }
            }
        }
    }

    fn create_test_backend(
        config: HostConfig,
        platform: SurfacePlatform,
        test_data_dir: Option<&Path>,
    ) -> Self {
        let test_auth_state_path =
            test_data_dir.map(|data_dir| data_dir.join("test-auth-session.json"));
        let memory_root_path = Some(std::env::temp_dir().join(format!(
            "fabushi-feature-host-memory-{}-{}",
            config.profile_id,
            std::process::id()
        )));
        let workflow_root_path = Some(std::env::temp_dir().join(format!(
            "fabushi-feature-host-workflows-{}-{}",
            config.profile_id,
            std::process::id()
        )));
        let info = HostInfo {
            runtime_version: "mahayana-test-backend".to_string(),
            protocol_version: HOST_PROTOCOL_VERSION.to_string(),
            platform,
        };
        let mut state = FeatureState::default();
        if let Some(path) = test_auth_state_path.as_deref() {
            state.auth_user = load_test_auth_user(path);
        }
        sync_computer_control_policy(&state.settings);
        state.events.push_back(HostEvent::HostReady {
            timestamp: timestamp(),
            info: info.clone(),
        });
        Self {
            config,
            info,
            #[cfg(feature = "production")]
            runtime: None,
            automation_path: None,
            bot_state_path: None,
            group_state_path: None,
            peer_messages_path: None,
            settings_path: None,
            remote_device_state_path: None,
            test_auth_state_path,
            memory_root_path,
            workflow_root_path,
            teach_recording: Mutex::new(None),
            active_account_id: Mutex::new(None),
            state: Mutex::new(state),
        }
    }

    #[cfg(feature = "production")]
    pub fn create_with_host_config(
        config: HostConfig,
        platform: SurfacePlatform,
        host_config: HostCreateConfig,
    ) -> Result<Self, FeatureHostError> {
        validate_config(&config)?;
        if config.mode == HostMode::Test {
            let test_data_dir = host_config.runtime.data_dir.clone();
            return Ok(Self::create_test_backend(
                config,
                platform,
                test_data_dir.as_deref(),
            ));
        }
        let automation_path = host_config.automation_path.clone().or_else(|| {
            host_config
                .runtime
                .data_dir
                .as_ref()
                .map(|data_dir| data_dir.join("automations.json"))
        });
        let bot_state_path = host_config
            .runtime
            .data_dir
            .as_ref()
            .map(|data_dir| data_dir.join("bots.json"));
        let group_state_path = host_config
            .runtime
            .data_dir
            .as_ref()
            .map(|data_dir| data_dir.join("groups.json"));
        let peer_messages_path = host_config
            .runtime
            .data_dir
            .as_ref()
            .map(|data_dir| data_dir.join("peer-messages.json"));
        let settings_path = host_config
            .runtime
            .data_dir
            .as_ref()
            .map(|data_dir| data_dir.join("settings.json"));
        let remote_device_state_path = host_config
            .runtime
            .data_dir
            .as_ref()
            .map(|data_dir| data_dir.join("remote-computer-device.json"));
        let memory_root_path = host_config
            .runtime
            .data_dir
            .as_ref()
            .map(|data_dir| data_dir.join("agents"));
        let workflow_root_path = host_config
            .runtime
            .data_dir
            .as_ref()
            .map(|data_dir| data_dir.join("workflows"));
        let runtime = MahayanaHost::create(host_config)?;
        let info = HostInfo {
            runtime_version: format!("mahayana-abi-{}", runtime.status().runtime_abi_version),
            protocol_version: HOST_PROTOCOL_VERSION.to_string(),
            platform,
        };
        let mut state = FeatureState::default();
        if let Some(path) = settings_path.as_deref() {
            state.settings = load_product_host_settings(path);
            // The bundled Computer Use MCP independently rereads this canonical
            // policy before every tool call. Persist defaults during startup so
            // a first-run profile is explicit rather than relying on fail-open
            // behavior while the settings UI has not yet written the file.
            persist_product_host_settings(path, &state.settings)?;
        }
        if let Some(path) = remote_device_state_path.as_deref() {
            state.remote_computer_device_secrets = load_remote_computer_device_secrets(path);
        }
        sync_computer_control_policy(&state.settings);
        state.events.push_back(HostEvent::HostReady {
            timestamp: timestamp(),
            info: info.clone(),
        });
        let controller = Self {
            config,
            info,
            runtime: Some(runtime),
            automation_path,
            bot_state_path,
            group_state_path,
            peer_messages_path,
            settings_path,
            remote_device_state_path,
            test_auth_state_path: None,
            memory_root_path,
            workflow_root_path,
            teach_recording: Mutex::new(None),
            active_account_id: Mutex::new(None),
            state: Mutex::new(state),
        };
        controller.ensure_account_boundary(&controller.auth_status()?)?;
        controller.state()?.events.push_back(HostEvent::HostReady {
            timestamp: timestamp(),
            info: controller.info.clone(),
        });
        Ok(controller)
    }

    pub fn info(&self) -> HostInfo {
        self.info.clone()
    }

    /// Return UI-safe account state. Credentials stay inside the Rust product
    /// client and are never serialized across the presentation boundary.
    pub fn auth_status(&self) -> Result<Value, FeatureHostError> {
        match self.config.mode {
            HostMode::Test => {
                let state = self.state()?;
                Ok(match state.auth_user.as_ref() {
                    Some(user) => json!({
                        "@type": "mahayana.auth.status",
                        "loggedIn": true,
                        "provider": "test",
                        "user": user,
                    }),
                    None => json!({
                        "@type": "mahayana.auth.status",
                        "loggedIn": false,
                        "provider": "test",
                    }),
                })
            }
            HostMode::Production => {
                #[cfg(feature = "production")]
                {
                    // First paint must be local-first. Restoring the UI-safe
                    // account from the Rust-owned session is immediate and
                    // lets an offline macOS launch remain signed in. Network
                    // operations still validate the token at their boundary.
                    let session = if let Ok(session) = self
                        .runtime()?
                        .product_execute("mahayana.auth.session.restore", &json!({}))
                    {
                        session
                    } else {
                        self.runtime()?
                            .product_execute("mahayana.auth.status", &json!({}))
                            .map_err(FeatureHostError::from)?
                    };
                    self.ensure_account_boundary(&session)?;
                    Ok(session)
                }
                #[cfg(not(feature = "production"))]
                Err(FeatureHostError::ProductionUnavailable)
            }
        }
    }

    /// Issue a short-lived self-hosted messaging credential bound to the
    /// currently authenticated Fabushi account, one device, one client
    /// session, and an explicit set of scopes. The underlying account token is
    /// never exposed; only the derived messaging bearer token is returned once.
    pub fn issue_messaging_access(
        &self,
        device_id: String,
        session_id: String,
        requested_scopes: Vec<String>,
        ttl_ms: i64,
    ) -> Result<Value, FeatureHostError> {
        let device_id = required(device_id, "device id")?;
        let session_id = required(session_id, "session id")?;
        let auth = self.auth_status()?;
        if auth.get("loggedIn").and_then(Value::as_bool) != Some(true) {
            return Err(FeatureHostError::Contract(
                "messaging access requires an authenticated Fabushi account session".into(),
            ));
        }
        let user_id = stable_authenticated_account_id(&auth).ok_or_else(|| {
            FeatureHostError::Contract("authenticated account has no stable user id".into())
        })?;
        let digest = Sha256::digest(user_id.as_bytes());
        let account_fingerprint = digest[..16]
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let actor_id = ActorId::new(format!("human:account:{account_fingerprint}"));
        let mut scopes = std::collections::BTreeSet::new();
        for scope in requested_scopes {
            let parsed = match scope.as_str() {
                "messaging" => AccessScope::Messaging,
                "calls" => AccessScope::Calls,
                "blobsRead" => AccessScope::BlobsRead,
                "blobsWrite" => AccessScope::BlobsWrite,
                "payments" => AccessScope::Payments,
                "miniApps" => AccessScope::MiniApps,
                "administration" => AccessScope::Administration,
                _ => {
                    return Err(FeatureHostError::Contract(format!(
                        "unsupported messaging access scope: {scope}"
                    )));
                }
            };
            scopes.insert(parsed);
        }
        if scopes.is_empty() {
            scopes.extend([
                AccessScope::Messaging,
                AccessScope::Calls,
                AccessScope::BlobsRead,
                AccessScope::BlobsWrite,
            ]);
        }
        let now_ms = now_millis();
        let ttl_ms = ttl_ms.clamp(5 * 60 * 1000, 30 * 24 * 60 * 60 * 1000);
        let expires_at_ms = now_ms.saturating_add(ttl_ms);
        let root = self
            .memory_root_path
            .as_deref()
            .ok_or_else(|| FeatureHostError::Contract("messaging storage is unavailable".into()))?;
        let access_store = FileAccessTokenStore::new(root.join("_messaging").join("access.json"));
        let grant_id = format!("grant:{}", Uuid::new_v4().simple());
        let issued = access_store
            .issue_random(AccessGrant {
                id: grant_id,
                actor_id: actor_id.clone(),
                device_id: device_id.clone(),
                session_id: session_id.clone(),
                scopes: scopes.clone(),
                issued_at_ms: now_ms,
                expires_at_ms: Some(expires_at_ms),
                revoked_at_ms: None,
            })
            .map_err(|error| FeatureHostError::Contract(error.to_string()))?;
        Ok(json!({
            "@type": "fabushi.messaging.access",
            "actorId": actor_id.0,
            "deviceId": device_id,
            "sessionId": session_id,
            "accessToken": issued.token,
            "expiresAtMs": expires_at_ms,
            "scopes": scopes,
        }))
    }

    pub fn password_login(
        &self,
        username: String,
        password: String,
    ) -> Result<Value, FeatureHostError> {
        let username = required(username, "username")?;
        let password = required(password, "password")?;
        #[cfg(not(feature = "production"))]
        let _ = &password;
        match self.config.mode {
            HostMode::Test => {
                let user = json!({
                    "id": "fast-e2e-user",
                    "username": username,
                    "nickname": "本地测试用户",
                });
                {
                    self.state()?.auth_user = Some(user.clone());
                }
                self.persist_test_auth_user(Some(&user))?;
                Ok(json!({
                    "@type": "mahayana.auth.session",
                    "loggedIn": true,
                    "provider": "test",
                    "sessionStored": true,
                    "user": user,
                }))
            }
            HostMode::Production => {
                #[cfg(feature = "production")]
                {
                    let response = self
                        .runtime()?
                        .product_execute(
                            "mahayana.auth.password.login",
                            &json!({"username": username, "password": password}),
                        )
                        .map_err(FeatureHostError::from)?;
                    self.ensure_account_boundary(&response)?;
                    Ok(response)
                }
                #[cfg(not(feature = "production"))]
                return Err(FeatureHostError::ProductionUnavailable);
            }
        }
    }

    pub fn browser_login_start(&self) -> Result<Value, FeatureHostError> {
        match self.config.mode {
            HostMode::Test => Ok(json!({
                "attemptId": "test-browser-login",
                "loginUrl": "about:blank#fabushi-test-browser-login",
                "expiresAt": now_millis() / 1000 + 600,
                "pollAfterMs": 250,
            })),
            HostMode::Production => {
                #[cfg(feature = "production")]
                return self
                    .runtime()?
                    .product_execute(
                        "mahayana.auth.browser.start",
                        &json!({"platform": browser_login_platform(self.info.platform)}),
                    )
                    .map_err(FeatureHostError::from);
                #[cfg(not(feature = "production"))]
                return Err(FeatureHostError::ProductionUnavailable);
            }
        }
    }

    pub fn browser_login_reopen(&self, attempt_id: String) -> Result<Value, FeatureHostError> {
        let attempt_id = required(attempt_id, "attemptId")?;
        match self.config.mode {
            HostMode::Test => Ok(json!({
                "status": "pending",
                "attemptId": attempt_id,
                "loginUrl": "about:blank#fabushi-test-browser-login",
                "pollAfterMs": 120,
            })),
            HostMode::Production => {
                #[cfg(feature = "production")]
                return self
                    .runtime()?
                    .product_execute(
                        "mahayana.auth.browser.reopen",
                        &json!({"attemptId": attempt_id}),
                    )
                    .map_err(FeatureHostError::from);
                #[cfg(not(feature = "production"))]
                return Err(FeatureHostError::ProductionUnavailable);
            }
        }
    }

    pub fn browser_login_cancel(&self, attempt_id: String) -> Result<Value, FeatureHostError> {
        let attempt_id = required(attempt_id, "attemptId")?;
        match self.config.mode {
            HostMode::Test => Ok(json!({"status": "cancelled"})),
            HostMode::Production => {
                #[cfg(feature = "production")]
                return self
                    .runtime()?
                    .product_execute(
                        "mahayana.auth.browser.cancel",
                        &json!({"attemptId": attempt_id}),
                    )
                    .map_err(FeatureHostError::from);
                #[cfg(not(feature = "production"))]
                return Err(FeatureHostError::ProductionUnavailable);
            }
        }
    }

    pub fn browser_login_poll(&self, attempt_id: String) -> Result<Value, FeatureHostError> {
        let attempt_id = required(attempt_id, "attemptId")?;
        match self.config.mode {
            HostMode::Test => {
                if attempt_id != "test-browser-login" {
                    return Ok(json!({"status": "expired"}));
                }
                let user = json!({
                    "id": "fast-e2e-browser-user",
                    "email": "browser@example.test",
                    "nickname": "Browser 测试用户",
                });
                {
                    self.state()?.auth_user = Some(user.clone());
                }
                self.persist_test_auth_user(Some(&user))?;
                Ok(json!({
                    "status": "completed",
                    "provider": "browser",
                    "auth": {
                        "loggedIn": true,
                        "provider": "browser",
                        "user": user,
                    }
                }))
            }
            HostMode::Production => {
                #[cfg(feature = "production")]
                {
                    let response = self
                        .runtime()?
                        .product_execute(
                            "mahayana.auth.browser.poll",
                            &json!({"attemptId": attempt_id}),
                        )
                        .map_err(FeatureHostError::from)?;
                    if auth_payload(&response)
                        .get("loggedIn")
                        .and_then(Value::as_bool)
                        == Some(true)
                    {
                        self.ensure_account_boundary(&response)?;
                    }
                    Ok(response)
                }
                #[cfg(not(feature = "production"))]
                return Err(FeatureHostError::ProductionUnavailable);
            }
        }
    }

    pub fn auth_providers(&self) -> Result<Value, FeatureHostError> {
        match self.config.mode {
            HostMode::Test => Ok(json!([
                {"id": "google", "displayName": "Google", "enabled": true},
                {"id": "apple", "displayName": "Apple", "enabled": true},
                {"id": "microsoft", "displayName": "Microsoft", "enabled": true},
                {"id": "github", "displayName": "GitHub", "enabled": true}
            ])),
            HostMode::Production => {
                #[cfg(feature = "production")]
                return self
                    .runtime()?
                    .product_execute("mahayana.auth.oauth.providers", &json!({}))
                    .map_err(FeatureHostError::from);
                #[cfg(not(feature = "production"))]
                return Err(FeatureHostError::ProductionUnavailable);
            }
        }
    }

    pub fn oauth_start(&self, provider: String) -> Result<Value, FeatureHostError> {
        let provider = required(provider, "provider")?;
        match self.config.mode {
            HostMode::Test => Ok(json!({
                "attemptId": format!("test-oauth-{provider}"),
                "provider": provider,
                "authorizationUrl": format!("about:blank#fabushi-test-oauth-{provider}"),
            })),
            HostMode::Production => {
                #[cfg(feature = "production")]
                return self
                    .runtime()?
                    .product_execute(
                        "mahayana.auth.oauth.start",
                        &json!({"provider": provider, "platform": "macos"}),
                    )
                    .map_err(FeatureHostError::from);
                #[cfg(not(feature = "production"))]
                return Err(FeatureHostError::ProductionUnavailable);
            }
        }
    }

    pub fn oauth_poll(&self, attempt_id: String) -> Result<Value, FeatureHostError> {
        let attempt_id = required(attempt_id, "attemptId")?;
        match self.config.mode {
            HostMode::Test => {
                let user = json!({
                    "id": "fast-e2e-oauth-user",
                    "email": "oauth@example.test",
                    "nickname": "OAuth 测试用户",
                });
                {
                    self.state()?.auth_user = Some(user.clone());
                }
                self.persist_test_auth_user(Some(&user))?;
                Ok(json!({
                    "attemptId": attempt_id,
                    "status": "completed",
                    "auth": {
                        "loggedIn": true,
                        "provider": "google",
                        "user": user,
                    }
                }))
            }
            HostMode::Production => {
                #[cfg(feature = "production")]
                {
                    let response = self
                        .runtime()?
                        .product_execute(
                            "mahayana.auth.oauth.poll",
                            &json!({"attemptId": attempt_id}),
                        )
                        .map_err(FeatureHostError::from)?;
                    if auth_payload(&response)
                        .get("loggedIn")
                        .and_then(Value::as_bool)
                        == Some(true)
                    {
                        self.ensure_account_boundary(&response)?;
                    }
                    Ok(response)
                }
                #[cfg(not(feature = "production"))]
                return Err(FeatureHostError::ProductionUnavailable);
            }
        }
    }

    pub fn logout(&self) -> Result<Value, FeatureHostError> {
        match self.config.mode {
            HostMode::Test => {
                {
                    self.state()?.auth_user = None;
                }
                self.persist_test_auth_user(None)?;
                Ok(json!({
                    "@type": "mahayana.auth.session",
                    "loggedIn": false,
                    "revoked": true,
                }))
            }
            HostMode::Production => {
                #[cfg(feature = "production")]
                {
                    let response = self.runtime()?.clear_session()?;
                    self.ensure_account_boundary(&response)?;
                    Ok(response)
                }
                #[cfg(not(feature = "production"))]
                return Err(FeatureHostError::ProductionUnavailable);
            }
        }
    }

    pub fn execute(&self, command: FeatureCommand) -> Result<CommandAccepted, FeatureHostError> {
        if let FeatureCommand::MessagingExecute {
            request_id,
            envelope,
        } = &command
        {
            return self.execute_messaging(request_id.clone(), envelope.clone());
        }
        self.authorize_feature_command(&command)?;
        if matches!(
            &command,
            FeatureCommand::AutomationList { .. }
                | FeatureCommand::AutomationUpsert { .. }
                | FeatureCommand::AutomationSetEnabled { .. }
                | FeatureCommand::AutomationDelete { .. }
                | FeatureCommand::AutomationRun { .. }
        ) {
            return self.execute_automation(command);
        }
        if matches!(
            &command,
            FeatureCommand::BotCreate { .. }
                | FeatureCommand::BotUpdate { .. }
                | FeatureCommand::BotClone { .. }
                | FeatureCommand::BotDelete { .. }
                | FeatureCommand::BotSetHidden { .. }
        ) {
            return self.execute_bot_profile(command);
        }
        if matches!(
            &command,
            FeatureCommand::GroupList { .. }
                | FeatureCommand::GroupCreate { .. }
                | FeatureCommand::GroupUpdate { .. }
                | FeatureCommand::GroupDelete { .. }
                | FeatureCommand::GroupSend { .. }
        ) {
            return self.execute_group_chat(command);
        }
        if matches!(
            &command,
            FeatureCommand::AgentSend { .. }
                | FeatureCommand::AgentBroadcast { .. }
                | FeatureCommand::AgentPeerHistory { .. }
        ) {
            return self.execute_agent_messaging(command);
        }
        if matches!(
            &command,
            FeatureCommand::AgentWorkspaceStateGet { .. }
                | FeatureCommand::AgentWorkspaceStateSet { .. }
        ) {
            return self.execute_agent_workspace_state(command);
        }
        if matches!(
            &command,
            FeatureCommand::SubagentList { .. } | FeatureCommand::AsyncTaskList { .. }
        ) {
            return self.execute_subagent_observation(command);
        }
        if matches!(
            &command,
            FeatureCommand::TeachStatus { .. }
                | FeatureCommand::TeachStart { .. }
                | FeatureCommand::TeachStop { .. }
        ) {
            return self.execute_teach(command);
        }
        if matches!(
            &command,
            FeatureCommand::ComputerStatus { .. }
                | FeatureCommand::ComputerScreenshot { .. }
                | FeatureCommand::ComputerAction { .. }
        ) {
            return self.execute_computer(command);
        }
        if matches!(
            &command,
            FeatureCommand::RemoteComputerRegister { .. }
                | FeatureCommand::RemoteComputerHeartbeat { .. }
                | FeatureCommand::RemoteComputerClients { .. }
                | FeatureCommand::RemoteComputerClientRevoke { .. }
                | FeatureCommand::RemoteComputerSessions { .. }
                | FeatureCommand::RemoteComputerSessionActivate { .. }
                | FeatureCommand::RemoteComputerSessionClose { .. }
                | FeatureCommand::RemoteComputerSignal { .. }
                | FeatureCommand::RemoteComputerSignalDrain { .. }
        ) {
            return self.execute_remote_computer(command);
        }
        if matches!(
            &command,
            FeatureCommand::MemoryList { .. }
                | FeatureCommand::MemoryAdd { .. }
                | FeatureCommand::MemoryRemove { .. }
                | FeatureCommand::MemoryClear { .. }
        ) {
            return self.execute_memory(command);
        }
        if matches!(
            &command,
            FeatureCommand::TrayList { .. }
                | FeatureCommand::TrayDismiss { .. }
                | FeatureCommand::TrayClear { .. }
                | FeatureCommand::TrayClearForAgent { .. }
        ) {
            return self.execute_tray(command);
        }
        if matches!(
            &command,
            FeatureCommand::WorkflowList { .. }
                | FeatureCommand::WorkflowUpsert { .. }
                | FeatureCommand::WorkflowSetEnabled { .. }
                | FeatureCommand::WorkflowDelete { .. }
                | FeatureCommand::WorkflowRun { .. }
                | FeatureCommand::WorkflowImportMarkdown { .. }
                | FeatureCommand::WorkflowImportLiveSource { .. }
        ) {
            return self.execute_workflow(command);
        }
        if matches!(
            &command,
            FeatureCommand::AttachmentUpload { .. }
                | FeatureCommand::AttachmentReadText { .. }
                | FeatureCommand::AttachmentReadChunk { .. }
                | FeatureCommand::AttachmentReadImage { .. }
        ) {
            return self.execute_attachment(command);
        }
        if matches!(
            &command,
            FeatureCommand::SearchMessages { .. } | FeatureCommand::SearchMedia { .. }
        ) {
            return self.execute_search(command);
        }
        if matches!(
            &command,
            FeatureCommand::McpList { .. }
                | FeatureCommand::McpApps { .. }
                | FeatureCommand::McpOauthLogin { .. }
                | FeatureCommand::McpOauthLogout { .. }
                | FeatureCommand::McpRemove { .. }
                | FeatureCommand::McpSetCustomInstructions { .. }
                | FeatureCommand::McpSetToolDisabled { .. }
                | FeatureCommand::McpRefresh { .. }
                | FeatureCommand::McpToolCall { .. }
        ) {
            return self.execute_mcp(command);
        }
        if matches!(
            &command,
            FeatureCommand::SettingsGet { .. }
                | FeatureCommand::SettingsUpdate { .. }
                | FeatureCommand::AuditList { .. }
        ) {
            return self.execute_settings_and_audit(command);
        }
        if is_product_surface_command(&command) {
            return self.execute_product_surface(command);
        }
        match self.config.mode {
            HostMode::Test => self.execute_test(command),
            HostMode::Production => self.execute_production(command),
        }
    }

    fn authorize_feature_command(
        &self,
        command: &FeatureCommand,
    ) -> Result<(), FeatureHostError> {
        if self.config.mode != HostMode::Production {
            return Ok(());
        }
        #[cfg(feature = "production")]
        {
            let (capability, agent_id, target, intent) = match command {
                FeatureCommand::ComputerScreenshot { agent_id, origin, session_id, target, .. } => (
                    "computer.screen.read",
                    agent_id.clone(),
                    json!({"origin": origin, "sessionId": session_id, "target": target}),
                    "capture the local computer screen",
                ),
                FeatureCommand::ComputerAction { agent_id, origin, session_id, target, .. } => (
                    "computer.input.control",
                    agent_id.clone(),
                    json!({"origin": origin, "sessionId": session_id, "target": target}),
                    "control the local computer",
                ),
                FeatureCommand::RemoteComputerSessionActivate { device_id, session_id, .. }
                | FeatureCommand::RemoteComputerSessionClose { device_id, session_id, .. }
                | FeatureCommand::RemoteComputerSignal { device_id, session_id, .. }
                | FeatureCommand::RemoteComputerSignalDrain { device_id, session_id, .. } => (
                    "computer.remote.session",
                    None,
                    json!({"deviceId": device_id, "sessionId": session_id}),
                    "use a remote computer session",
                ),
                FeatureCommand::McpToolCall { server, tool, .. } => (
                    "mcp.tool.call",
                    None,
                    json!({"server": server, "tool": tool}),
                    "call an MCP tool",
                ),
                FeatureCommand::AgentSend { from_agent_id, target_id, .. } => (
                    "agent.handoff",
                    Some(from_agent_id.clone()),
                    json!({"targetAgent": target_id}),
                    "send durable work to another Agent",
                ),
                FeatureCommand::AgentBroadcast { target_ids, .. } => (
                    "agent.handoff.broadcast",
                    None,
                    json!({"targetAgents": target_ids}),
                    "broadcast durable work to Agents",
                ),
                FeatureCommand::AttachmentUpload { agent_id, filename, .. } => (
                    "filesystem.agent.write",
                    Some(agent_id.clone()),
                    json!({"filename": filename}),
                    "write an Agent workspace attachment",
                ),
                FeatureCommand::AttachmentReadText { agent_id, path, .. }
                | FeatureCommand::AttachmentReadChunk { agent_id, path, .. }
                | FeatureCommand::AttachmentReadImage { agent_id, path, .. } => (
                    "filesystem.agent.read",
                    Some(agent_id.clone()),
                    json!({"path": path}),
                    "read an Agent workspace attachment",
                ),
                FeatureCommand::MiniAppOpen { mini_app_id, .. } => (
                    "miniapp.open",
                    None,
                    json!({"miniAppId": mini_app_id}),
                    "open a Mini App",
                ),
                FeatureCommand::ConnectorConnect { connector_id, .. } => (
                    "connector.connect",
                    None,
                    json!({"connectorId": connector_id}),
                    "connect an external service",
                ),
                FeatureCommand::ConnectorRenameAccount { connector_id, account_id, .. }
                | FeatureCommand::ConnectorRemoveAccount { connector_id, account_id, .. } => (
                    "connector.account.manage",
                    None,
                    json!({"connectorId": connector_id, "accountId": account_id}),
                    "manage a connected external account",
                ),
                FeatureCommand::ConnectorSetToolEnabled { connector_id, tool_id, .. } => (
                    "connector.tool.manage",
                    None,
                    json!({"connectorId": connector_id, "toolId": tool_id}),
                    "change a connector tool policy",
                ),
                _ => return Ok(()),
            };
            let conversation_id = {
                let state = self.state()?;
                agent_id
                    .as_deref()
                    .and_then(|id| find_bot_by_runtime_or_surface_id(&state, id))
                    .and_then(|bot| bot.conversation_id.clone())
                    .unwrap_or_else(|| MAHAYANA_AI_CONVERSATION_ID.to_string())
            };
            let actor = agent_id
                .as_ref()
                .map(|id| format!("agent:{id}"))
                .unwrap_or_else(|| "human".to_string());
            let response = self.runtime()?.execute(RuntimeCommand::AuthorizeCapability {
                request: CapabilityRequest {
                    actor,
                    agent_id,
                    conversation_id: ConversationId(conversation_id),
                    run_id: None,
                    capability: capability.to_string(),
                    target,
                    intent: intent.to_string(),
                },
                availability: CapabilityAvailability::Ready,
                unavailable_reason: None,
            })?;
            match response {
                RuntimeResponse::CapabilityDecision {
                    decision: CapabilityPolicyDecision::Allow,
                } => Ok(()),
                RuntimeResponse::CapabilityDecision {
                    decision: CapabilityPolicyDecision::NeedsUser,
                } => Err(FeatureHostError::Contract(format!(
                    "{capability} requires explicit user approval"
                ))),
                RuntimeResponse::CapabilityDecision {
                    decision: CapabilityPolicyDecision::Deny,
                } => Err(FeatureHostError::Contract(format!(
                    "{capability} was denied by capability policy"
                ))),
                other => Err(unexpected_response("mahayana.capability.authorize", other)),
            }
        }
        #[cfg(not(feature = "production"))]
        {
            let _ = command;
            Ok(())
        }
    }

    fn execute_messaging(
        &self,
        request_id: String,
        envelope: Value,
    ) -> Result<CommandAccepted, FeatureHostError> {
        self.execute_messaging_sync(request_id.clone(), envelope)?;
        Ok(CommandAccepted {
            request_id,
            operation_id: None,
        })
    }

    /// Executes one messaging envelope against the exact same persistent
    /// `MessagingService` used by desktop and also returns the resulting server
    /// envelopes to native shells. The envelopes are still projected onto the
    /// regular FeatureHost event queue, so desktop/event-driven consumers retain
    /// their existing behavior while iOS and Android can update synchronously.
    pub fn execute_messaging_sync(
        &self,
        request_id: String,
        envelope: Value,
    ) -> Result<Vec<Value>, FeatureHostError> {
        static MESSAGING_IO_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let _io_guard = MESSAGING_IO_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .map_err(|_| FeatureHostError::Contract("messaging storage lock is poisoned".into()))?;
        let client_envelope: MessagingClientEnvelope =
            serde_json::from_value(envelope).map_err(|error| {
                FeatureHostError::Contract(format!("invalid messaging envelope: {error}"))
            })?;
        let root = self.messaging_root_for(&client_envelope)?;
        let messaging_root = root.join("_messaging");
        let store = JsonFileStateStore::new(messaging_root.join("snapshot.json"));
        let mut service = MessagingService::load_with_blob_store(
            store,
            FileBlobStore::new(messaging_root.join("blobs")),
        )
        .map_err(|error| FeatureHostError::Contract(error.to_string()))?;
        let responses = service
            .handle(client_envelope, now_millis())
            .map_err(|error| FeatureHostError::Contract(error.to_string()))?;
        let envelopes = responses
            .into_iter()
            .map(|response| {
                serde_json::to_value(response)
                    .map_err(|error| FeatureHostError::Contract(error.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut state = self.state()?;
        for envelope in &envelopes {
            state.events.push_back(HostEvent::MessagingEvent {
                timestamp: timestamp(),
                request_id: request_id.clone(),
                envelope: envelope.clone(),
            });
        }
        Ok(envelopes)
    }

    pub fn read_messaging_blob_range(
        &self,
        blob_id: &str,
        offset: u64,
        length: u64,
    ) -> Result<(fabushi_messaging_core::BlobMetadata, Vec<u8>), FeatureHostError> {
        let root = self
            .active_account_root(self.memory_root_path.as_deref())
            .ok_or_else(|| FeatureHostError::Contract("messaging storage is unavailable".into()))?;
        let blob_id = BlobId::new(blob_id.to_string())
            .map_err(|error| FeatureHostError::Contract(error.to_string()))?;
        let store = FileBlobStore::new(root.join("_messaging").join("blobs"));
        let metadata = store
            .metadata(&blob_id)
            .map_err(|error| FeatureHostError::Contract(error.to_string()))?;
        let bytes = store
            .read_range(&blob_id, offset, length.min(1024 * 1024))
            .map_err(|error| FeatureHostError::Contract(error.to_string()))?;
        Ok((metadata, bytes))
    }

    fn execute_automation(
        &self,
        command: FeatureCommand,
    ) -> Result<CommandAccepted, FeatureHostError> {
        let request_id = command.request_id().to_string();
        match command {
            FeatureCommand::AutomationList { agent_id, .. } => {
                let owner = if let Some(requested) = agent_id.as_deref() {
                    let state = self.state()?;
                    Some(canonical_runtime_agent_id(&state, requested).ok_or_else(|| {
                        FeatureHostError::Contract(format!(
                            "unknown automation agent: {requested}"
                        ))
                    })?)
                } else {
                    None
                };
                let mut automations = self
                    .state()?
                    .automations
                    .values()
                    .filter(|automation| {
                        owner.as_ref().is_none_or(|agent_id| {
                            fabu_automation_owner(automation) == agent_id
                        })
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                automations.sort_by_key(|item| item.created_at_ms);
                self.state()?.events.push_back(HostEvent::AutomationListed {
                    timestamp: timestamp(),
                    automations,
                });
                Ok(CommandAccepted {
                    request_id,
                    operation_id: None,
                })
            }
            FeatureCommand::AutomationUpsert {
                id,
                agent_id,
                name,
                prompt,
                schedule,
                trigger,
                enabled,
                ..
            } => {
                let name = required(name, "automation name")?;
                let prompt = required(prompt, "automation prompt")?;
                let trigger = trigger.unwrap_or_else(|| AutomationTrigger::Schedule {
                    schedule: schedule.clone(),
                });
                let (schedule, trigger) = match trigger {
                    AutomationTrigger::Schedule { schedule } => {
                        let schedule = normalize_automation_schedule(&schedule)?;
                        (schedule.clone(), AutomationTrigger::Schedule { schedule })
                    }
                    AutomationTrigger::Event {
                        source,
                        event,
                        filter,
                    } => {
                        let event = required(event, "automation event")?;
                        (
                            format!("event:{}:{event}", listener_platform_slug(source)),
                            AutomationTrigger::Event {
                                source,
                                event,
                                filter: filter.filter(|value| !value.trim().is_empty()),
                            },
                        )
                    }
                };
                let now = now_millis();
                let mut state = self.state()?;
                let id = id
                    .filter(|id| is_safe_automation_id(id))
                    .unwrap_or_else(|| {
                        state.sequence += 1;
                        format!("routine-{}-{}", now, state.sequence)
                    });
                let requested_agent_id = match agent_id {
                    Some(agent_id) => {
                        let requested = required(agent_id, "automation agent id")?;
                        Some(canonical_runtime_agent_id(&state, &requested).ok_or_else(|| {
                            FeatureHostError::Contract(format!(
                                "unknown automation agent: {requested}"
                            ))
                        })?)
                    }
                    None => None,
                };
                let previous_key =
                    find_automation_state_key(&state.automations, &id, requested_agent_id.as_deref());
                let previous = previous_key
                    .as_ref()
                    .and_then(|key| state.automations.get(key))
                    .cloned();
                if let (Some(previous), Some(agent_id)) =
                    (previous.as_ref(), requested_agent_id.as_deref())
                {
                    ensure_automation_agent_scope(previous, Some(agent_id))?;
                }
                let resolved_agent_id = Some(
                    requested_agent_id
                        .or_else(|| previous.as_ref().and_then(|item| item.agent_id.clone()))
                        .unwrap_or_else(|| "mahayana-assistant".into()),
                );
                let action = if previous.is_some() {
                    "updated"
                } else {
                    "created"
                };
                let automation = AutomationSummary {
                    id: id.clone(),
                    agent_id: resolved_agent_id,
                    name,
                    prompt,
                    schedule: schedule.clone(),
                    trigger: Some(trigger.clone()),
                    enabled,
                    created_at_ms: previous.as_ref().map_or(now, |item| item.created_at_ms),
                    last_run_at_ms: previous.as_ref().and_then(|item| item.last_run_at_ms),
                    next_run_at_ms: automation_next_run(&trigger, &schedule, enabled, now),
                };
                if let Some(previous_key) = previous_key {
                    state.automations.remove(&previous_key);
                }
                let state_key = automation_state_key(
                    fabu_automation_owner(&automation),
                    &automation.id,
                );
                state.automations.insert(state_key, automation.clone());
                self.persist_automations(&state.automations)?;
                state.events.push_back(HostEvent::AutomationChanged {
                    timestamp: timestamp(),
                    action: action.into(),
                    automation,
                });
                Ok(CommandAccepted {
                    request_id,
                    operation_id: None,
                })
            }
            FeatureCommand::AutomationSetEnabled {
                id,
                agent_id,
                enabled,
                ..
            } => {
                let mut state = self.state()?;
                let requested_owner = agent_id
                    .as_deref()
                    .map(|requested| {
                        canonical_runtime_agent_id(&state, requested).ok_or_else(|| {
                            FeatureHostError::Contract(format!(
                                "unknown automation agent: {requested}"
                            ))
                        })
                    })
                    .transpose()?;
                let key = find_automation_state_key(
                    &state.automations,
                    &id,
                    requested_owner.as_deref(),
                )
                .ok_or_else(|| {
                    FeatureHostError::Contract(format!("unknown automation: {id}"))
                })?;
                let automation = state.automations.get_mut(&key).ok_or_else(|| {
                    FeatureHostError::Contract(format!("unknown automation: {id}"))
                })?;
                ensure_automation_agent_scope(automation, requested_owner.as_deref())?;
                automation.enabled = enabled;
                let trigger =
                    automation
                        .trigger
                        .clone()
                        .unwrap_or_else(|| AutomationTrigger::Schedule {
                            schedule: automation.schedule.clone(),
                        });
                automation.next_run_at_ms =
                    automation_next_run(&trigger, &automation.schedule, enabled, now_millis());
                let automation = automation.clone();
                self.persist_automations(&state.automations)?;
                state.events.push_back(HostEvent::AutomationChanged {
                    timestamp: timestamp(),
                    action: if enabled { "resumed" } else { "paused" }.into(),
                    automation,
                });
                Ok(CommandAccepted {
                    request_id,
                    operation_id: None,
                })
            }
            FeatureCommand::AutomationDelete { id, agent_id, .. } => {
                let mut state = self.state()?;
                let requested_owner = agent_id
                    .as_deref()
                    .map(|requested| {
                        canonical_runtime_agent_id(&state, requested).ok_or_else(|| {
                            FeatureHostError::Contract(format!(
                                "unknown automation agent: {requested}"
                            ))
                        })
                    })
                    .transpose()?;
                let key = find_automation_state_key(
                    &state.automations,
                    &id,
                    requested_owner.as_deref(),
                )
                .ok_or_else(|| {
                    FeatureHostError::Contract(format!("unknown automation: {id}"))
                })?;
                let existing = state.automations.get(&key).ok_or_else(|| {
                    FeatureHostError::Contract(format!("unknown automation: {id}"))
                })?;
                ensure_automation_agent_scope(existing, requested_owner.as_deref())?;
                let automation = state
                    .automations
                    .remove(&key)
                    .expect("automation checked above");
                self.persist_automations(&state.automations)?;
                state.events.push_back(HostEvent::AutomationChanged {
                    timestamp: timestamp(),
                    action: "deleted".into(),
                    automation,
                });
                Ok(CommandAccepted {
                    request_id,
                    operation_id: None,
                })
            }
            FeatureCommand::AutomationRun { id, agent_id, .. } => {
                let automation =
                    {
                        let mut state = self.state()?;
                        let requested_owner = agent_id
                            .as_deref()
                            .map(|requested| {
                                canonical_runtime_agent_id(&state, requested).ok_or_else(|| {
                                    FeatureHostError::Contract(format!(
                                        "unknown automation agent: {requested}"
                                    ))
                                })
                            })
                            .transpose()?;
                        let key = find_automation_state_key(
                            &state.automations,
                            &id,
                            requested_owner.as_deref(),
                        )
                        .ok_or_else(|| {
                            FeatureHostError::Contract(format!("unknown automation: {id}"))
                        })?;
                        let automation = state.automations.get_mut(&key).ok_or_else(|| {
                            FeatureHostError::Contract(format!("unknown automation: {id}"))
                        })?;
                        ensure_automation_agent_scope(automation, requested_owner.as_deref())?;
                        let now = now_millis();
                        automation.last_run_at_ms = Some(now);
                        let trigger = automation.trigger.clone().unwrap_or_else(|| {
                            AutomationTrigger::Schedule {
                                schedule: automation.schedule.clone(),
                            }
                        });
                        automation.next_run_at_ms = automation_next_run(
                            &trigger,
                            &automation.schedule,
                            automation.enabled,
                            now,
                        );
                        let automation = automation.clone();
                        self.persist_automations(&state.automations)?;
                        state.events.push_back(HostEvent::AutomationChanged {
                            timestamp: timestamp(),
                            action: "running".into(),
                            automation: automation.clone(),
                        });
                        automation
                    };
                if let Some(AutomationTrigger::Event {
                    source,
                    event,
                    filter,
                }) = automation.trigger.as_ref()
                {
                    self.state()?.events.push_back(HostEvent::TranscriptCard {
                        timestamp: timestamp(),
                        entry_id: format!("event-{}-{}", automation.id, now_millis()),
                        operation_id: None,
                        card: TranscriptCard::Event {
                            event: EventCard {
                                source: *source,
                                event: event.clone(),
                                title: format!("{} event", listener_platform_display(*source)),
                                summary: format!("{} woke routine “{}”.", event, automation.name),
                                url: None,
                                actor: None,
                                fields: filter.as_ref().map(|filter| {
                                    vec![EventField {
                                        label: "Filter".into(),
                                        value: filter.clone(),
                                    }]
                                }),
                                occurred_at_ms: Some(now_millis()),
                            },
                        },
                    });
                }
                let trigger_context = match automation.trigger.as_ref() {
                    Some(AutomationTrigger::Event { source, event, .. }) => format!(
                        "\n触发事件：{} / {}",
                        listener_platform_display(*source),
                        event
                    ),
                    _ => String::new(),
                };
                let text = format!(
                    "[自动化例程：{}]{}\n这是用户保存的 standing instruction。请立即执行并报告结果。\n\n{}",
                    automation.name, trigger_context, automation.prompt
                );
                let target_agent_id = automation
                    .agent_id
                    .clone()
                    .unwrap_or_else(|| "mahayana-assistant".into());
                match self.config.mode {
                    HostMode::Test => self.execute_test(FeatureCommand::ChatSend {
                        request_id,
                        text,
                        agent_id: Some(target_agent_id.clone()),
                        conversation_id: None,
                        mode: AgentMode::Agent,
                        mode_statement: None,
                        model: None,
                        attachments: Vec::new(),
                    }),
                    HostMode::Production => {
                        #[cfg(feature = "production")]
                        return self.production_chat(
                            request_id,
                            text,
                            Some(target_agent_id),
                            None,
                            AgentMode::Agent,
                            None,
                            None,
                            Vec::new(),
                        );
                        #[cfg(not(feature = "production"))]
                        return Err(FeatureHostError::ProductionUnavailable);
                    }
                }
            }
            _ => unreachable!("non-automation command routed to automation executor"),
        }
    }

    fn execute_bot_profile(
        &self,
        command: FeatureCommand,
    ) -> Result<CommandAccepted, FeatureHostError> {
        let request_id = command.request_id().to_string();
        let mut state = self.state()?;
        ensure_open(&state)?;
        let (action, bot) = match command {
            FeatureCommand::BotCreate {
                name,
                description,
                title,
                avatar,
                avatar_shape,
                avatar_color,
                inference_provider,
                ..
            } => {
                let name = clamp_line(&name, 72);
                if name.is_empty() {
                    return Err(FeatureHostError::Contract(
                        "bot name must not be empty".into(),
                    ));
                }
                agent_inference_provider_key(inference_provider)?;
                let id = next_id(&mut state, "agent");
                let bot = BotSummary {
                    id: id.clone(),
                    agent_id: Some(id.clone()),
                    name,
                    description: clamp_block(&description, 2000),
                    title: title.trim().to_string(),
                    hidden: false,
                    avatar: sanitize_avatar_data_url(avatar)?,
                    avatar_shape: clean_optional_string(avatar_shape),
                    avatar_color: clean_optional_string(avatar_color),
                    notifications_enabled: true,
                    notify_on_updates: true,
                    unread: false,
                    conversation_id: Some(format!("codex:agent:{id}")),
                    inference_provider,
                };
                state.bots.insert(id, bot.clone());
                ("created", bot)
            }
            FeatureCommand::BotUpdate {
                id,
                name,
                description,
                title,
                avatar,
                avatar_shape,
                avatar_color,
                notifications_enabled,
                notify_on_updates,
                inference_provider,
                unread,
                clear_inference_provider,
                ..
            } => {
                let bot = state
                    .bots
                    .get_mut(&id)
                    .ok_or_else(|| FeatureHostError::Contract(format!("unknown bot: {id}")))?;
                if let Some(name) = name {
                    let name = clamp_line(&name, 72);
                    if name.is_empty() {
                        return Err(FeatureHostError::Contract(
                            "bot name must not be empty".into(),
                        ));
                    }
                    bot.name = name;
                }
                if let Some(description) = description {
                    bot.description = clamp_block(&description, 2000);
                }
                if let Some(title) = title {
                    bot.title = title.trim().to_string();
                }
                if avatar.is_some() {
                    bot.avatar = sanitize_avatar_data_url(avatar)?;
                }
                if avatar_shape.is_some() {
                    bot.avatar_shape = clean_optional_string(avatar_shape);
                }
                if avatar_color.is_some() {
                    bot.avatar_color = clean_optional_string(avatar_color);
                }
                if let Some(enabled) = notifications_enabled {
                    bot.notifications_enabled = enabled;
                }
                if let Some(enabled) = notify_on_updates {
                    bot.notify_on_updates = enabled;
                }
                if let Some(unread) = unread {
                    bot.unread = unread;
                }
                if clear_inference_provider && inference_provider.is_some() {
                    return Err(FeatureHostError::Contract(
                        "bot.update cannot set and clear inferenceProvider in the same request".into(),
                    ));
                }
                if clear_inference_provider {
                    bot.inference_provider = None;
                } else if let Some(provider) = inference_provider {
                    agent_inference_provider_key(Some(provider))?;
                    bot.inference_provider = Some(provider);
                }
                ("updated", bot.clone())
            }
            FeatureCommand::BotClone { id, .. } => {
                let source = state
                    .bots
                    .get(&id)
                    .cloned()
                    .ok_or_else(|| FeatureHostError::Contract(format!("unknown bot: {id}")))?;
                let new_id = next_id(&mut state, "agent");
                let source_agent_id = bot_runtime_agent_id(&source).to_string();
                let clone_name = clone_agent_display_name(&source.name);
                let bot = BotSummary {
                    id: new_id.clone(),
                    agent_id: Some(new_id.clone()),
                    name: clone_name,
                    description: source.description,
                    title: source.title,
                    hidden: false,
                    avatar: source.avatar,
                    avatar_shape: source.avatar_shape,
                    avatar_color: source.avatar_color,
                    notifications_enabled: source.notifications_enabled,
                    notify_on_updates: source.notify_on_updates,
                    unread: false,
                    conversation_id: Some(format!("codex:agent:{new_id}")),
                    inference_provider: source.inference_provider,
                };
                // Match Fabu clone semantics: copy reusable Agent-owned state
                // (memory, automations, workflow enablement) but never transcript
                // history, audits, attachments, or teach recordings.
                self.clone_agent_local_state(&source_agent_id, &new_id)?;
                state.bots.insert(new_id, bot.clone());
                ("cloned", bot)
            }
            FeatureCommand::BotDelete { id, .. } => {
                if id == "mahayana-assistant" {
                    return Err(FeatureHostError::Contract(
                        "the primary Mahayana assistant cannot be deleted".into(),
                    ));
                }
                let bot = state
                    .bots
                    .remove(&id)
                    .ok_or_else(|| FeatureHostError::Contract(format!("unknown bot: {id}")))?;
                ("deleted", bot)
            }
            FeatureCommand::BotSetHidden { id, hidden, .. } => {
                let bot = state
                    .bots
                    .get_mut(&id)
                    .ok_or_else(|| FeatureHostError::Contract(format!("unknown bot: {id}")))?;
                bot.hidden = hidden;
                ("updated", bot.clone())
            }
            _ => unreachable!("non-bot-profile command routed to bot executor"),
        };
        self.persist_bots(&state.bots)?;
        // Fabu keeps the Bot surface and Agent runtime as separate identities.
        // A Bot mutation mirrors presentation/settings into the Agent's own
        // directory, while deleting the Bot deliberately retains Agent state.
        if action != "deleted" {
            self.persist_agent_manifest(&bot)?;
        }
        state.events.push_back(HostEvent::BotChanged {
            timestamp: timestamp(),
            action: action.into(),
            bot,
        });
        Ok(CommandAccepted {
            request_id,
            operation_id: None,
        })
    }

    fn execute_group_chat(
        &self,
        command: FeatureCommand,
    ) -> Result<CommandAccepted, FeatureHostError> {
        let request_id = command.request_id().to_string();
        let mut state = self.state()?;
        ensure_open(&state)?;
        let mut kick_group_id: Option<String> = None;
        match command {
            FeatureCommand::GroupList { .. } => {
                let mut groups = state.groups.values().cloned().collect::<Vec<_>>();
                groups.sort_by_key(|group| group.created_at_ms);
                state.events.push_back(HostEvent::GroupListed {
                    timestamp: timestamp(),
                    groups,
                });
            }
            FeatureCommand::GroupCreate {
                name,
                description,
                member_ids,
                ..
            } => {
                let name = clamp_line(&name, 72);
                if name.is_empty() {
                    return Err(FeatureHostError::Contract(
                        "group name must not be empty".into(),
                    ));
                }
                let member_ids = validate_group_members(&state, member_ids)?;
                let now = now_millis();
                let id = next_id(&mut state, "group");
                let group = GroupSummary {
                    id: id.clone(),
                    name,
                    description: clamp_block(&description, 2000),
                    member_ids,
                    messages: Vec::new(),
                    created_at_ms: now,
                    updated_at_ms: now,
                };
                state.groups.insert(id, group.clone());
                self.persist_groups(&state.groups)?;
                state.events.push_back(HostEvent::GroupChanged {
                    timestamp: timestamp(),
                    action: "created".into(),
                    group,
                });
            }
            FeatureCommand::GroupUpdate {
                id,
                name,
                description,
                member_ids,
                ..
            } => {
                let validated_members = member_ids
                    .map(|member_ids| validate_group_members(&state, member_ids))
                    .transpose()?;
                let group = state
                    .groups
                    .get_mut(&id)
                    .ok_or_else(|| FeatureHostError::Contract(format!("unknown group: {id}")))?;
                if let Some(name) = name {
                    let name = clamp_line(&name, 72);
                    if name.is_empty() {
                        return Err(FeatureHostError::Contract(
                            "group name must not be empty".into(),
                        ));
                    }
                    group.name = name;
                }
                if let Some(description) = description {
                    group.description = clamp_block(&description, 2000);
                }
                if let Some(member_ids) = validated_members {
                    group.member_ids = member_ids;
                }
                group.updated_at_ms = now_millis();
                let group = group.clone();
                self.persist_groups(&state.groups)?;
                state.events.push_back(HostEvent::GroupChanged {
                    timestamp: timestamp(),
                    action: "updated".into(),
                    group,
                });
            }
            FeatureCommand::GroupDelete { id, .. } => {
                let group = state
                    .groups
                    .remove(&id)
                    .ok_or_else(|| FeatureHostError::Contract(format!("unknown group: {id}")))?;
                self.persist_groups(&state.groups)?;
                state.events.push_back(HostEvent::GroupChanged {
                    timestamp: timestamp(),
                    action: "deleted".into(),
                    group,
                });
            }
            FeatureCommand::GroupSend { id, text, .. } => {
                let text = clamp_block(&text, 8000);
                if text.is_empty() {
                    return Err(FeatureHostError::Contract(
                        "group message must not be empty".into(),
                    ));
                }
                let message_id = next_id(&mut state, "group-message");
                let now = now_millis();
                let group = state
                    .groups
                    .get_mut(&id)
                    .ok_or_else(|| FeatureHostError::Contract(format!("unknown group: {id}")))?;
                group.messages.push(GroupMessage {
                    id: message_id,
                    speaker: GroupSpeaker::User { name: None },
                    content: text,
                    created_at_ms: now,
                });
                if group.messages.len() > 500 {
                    let overflow = group.messages.len() - 500;
                    group.messages.drain(0..overflow);
                }
                group.updated_at_ms = now;
                let group = group.clone();
                let responder_ids = resolve_group_responders(&group, &state.bots);
                if !responder_ids.is_empty() {
                    let run_id = next_id(&mut state, "group-run");
                    state.group_runs.insert(
                        id.clone(),
                        GroupRunState {
                            run_id,
                            round: 0,
                            speaker_order: order_round_speakers(&responder_ids, 0),
                            speaker_index: 0,
                            total_messages: 0,
                            messages_this_round: 0,
                        },
                    );
                    kick_group_id = Some(id.clone());
                }
                self.persist_groups(&state.groups)?;
                state.events.push_back(HostEvent::GroupChanged {
                    timestamp: timestamp(),
                    action: "message".into(),
                    group,
                });
            }
            _ => unreachable!("non-group command routed to group executor"),
        }
        drop(state);
        let operation_id = match (self.config.mode, kick_group_id) {
            (HostMode::Production, Some(group_id)) => {
                #[cfg(feature = "production")]
                {
                    self.start_next_group_turn(&group_id)?
                }
                #[cfg(not(feature = "production"))]
                {
                    let _ = group_id;
                    None
                }
            }
            _ => None,
        };
        Ok(CommandAccepted {
            request_id,
            operation_id,
        })
    }

    fn execute_teach(&self, command: FeatureCommand) -> Result<CommandAccepted, FeatureHostError> {
        let request_id = command.request_id().to_string();
        self.refresh_finished_teach_recording()?;
        match command {
            FeatureCommand::TeachStatus { .. } => {
                let status = {
                    let guard = self
                        .teach_recording
                        .lock()
                        .map_err(|_| FeatureHostError::StatePoisoned)?;
                    teach_recording_status(guard.as_ref())
                };
                self.state()?.events.push_back(HostEvent::TeachChanged {
                    timestamp: timestamp(),
                    status,
                    result: None,
                });
            }
            FeatureCommand::TeachStart {
                agent_id,
                entry_point,
                ..
            } => {
                let requested_agent_id = agent_id;
                let agent_id = {
                    let state = self.state()?;
                    canonical_runtime_agent_id(&state, &requested_agent_id).ok_or_else(|| {
                        FeatureHostError::Contract(format!(
                            "unknown teach agent: {requested_agent_id}"
                        ))
                    })?
                };
                if !is_safe_memory_agent_id(&agent_id) {
                    return Err(FeatureHostError::Contract(format!(
                        "unsafe teach agent id: {agent_id}"
                    )));
                }
                let mut guard = self
                    .teach_recording
                    .lock()
                    .map_err(|_| FeatureHostError::StatePoisoned)?;
                if let Some(active) = guard.as_ref() {
                    if active.agent_id == agent_id {
                        let status = teach_recording_status(Some(active));
                        drop(guard);
                        self.state()?.events.push_back(HostEvent::TeachChanged {
                            timestamp: timestamp(),
                            status,
                            result: None,
                        });
                        return Ok(CommandAccepted {
                            request_id,
                            operation_id: None,
                        });
                    }
                    return Err(FeatureHostError::Contract(format!(
                        "teach recording is already active for {}",
                        active.agent_id
                    )));
                }

                let root = self
                    .active_account_root(self.memory_root_path.as_deref())
                    .ok_or_else(|| {
                        FeatureHostError::Contract("teach recording storage is unavailable".into())
                    })?;
                let started_at_ms = now_millis();
                let session_dir = root
                    .join(&agent_id)
                    .join("teach-sessions")
                    .join(format!("{started_at_ms}"));
                std::fs::create_dir_all(&session_dir).map_err(|error| {
                    FeatureHostError::Contract(format!("create teach session: {error}"))
                })?;
                let video_path = session_dir.join("demo.mp4");
                let child = if self.config.mode == HostMode::Production {
                    Some(spawn_teach_capture(&video_path)?)
                } else {
                    None
                };
                *guard = Some(TeachCaptureProcess {
                    agent_id: agent_id.clone(),
                    entry_point,
                    started_at_ms,
                    session_dir,
                    video_path,
                    child,
                });
                let status = teach_recording_status(guard.as_ref());
                drop(guard);
                self.state()?.events.push_back(HostEvent::TeachChanged {
                    timestamp: timestamp(),
                    status,
                    result: None,
                });
            }
            FeatureCommand::TeachStop { agent_id, save, .. } => {
                let active = {
                    let mut guard = self
                        .teach_recording
                        .lock()
                        .map_err(|_| FeatureHostError::StatePoisoned)?;
                    let Some(active) = guard.as_ref() else {
                        let status = TeachRecordingStatus::default();
                        drop(guard);
                        self.state()?.events.push_back(HostEvent::TeachChanged {
                            timestamp: timestamp(),
                            status,
                            result: None,
                        });
                        return Ok(CommandAccepted {
                            request_id,
                            operation_id: None,
                        });
                    };
                    if active.agent_id != agent_id {
                        return Err(FeatureHostError::Contract(format!(
                            "teach recording belongs to {}, not {agent_id}",
                            active.agent_id
                        )));
                    }
                    guard.take().expect("teach recording existed")
                };
                let result = self.finalize_teach_capture(active, save)?;
                self.state()?.events.push_back(HostEvent::TeachChanged {
                    timestamp: timestamp(),
                    status: TeachRecordingStatus::default(),
                    result: Some(result),
                });
            }
            _ => unreachable!("non-teach command routed to teach executor"),
        }
        Ok(CommandAccepted {
            request_id,
            operation_id: None,
        })
    }

    fn refresh_finished_teach_recording(&self) -> Result<(), FeatureHostError> {
        let finished = {
            let mut guard = self
                .teach_recording
                .lock()
                .map_err(|_| FeatureHostError::StatePoisoned)?;
            let Some(active) = guard.as_mut() else {
                return Ok(());
            };
            let process_finished = match active.child.as_mut() {
                Some(child) => child
                    .try_wait()
                    .map_err(|error| {
                        FeatureHostError::Contract(format!("poll teach capture: {error}"))
                    })?
                    .is_some(),
                None => now_millis() - active.started_at_ms >= TEACH_MAX_DURATION_MS,
            };
            process_finished.then(|| guard.take().expect("teach recording existed"))
        };
        if let Some(active) = finished {
            let result = self.finalize_teach_capture(active, true)?;
            self.state()?.events.push_back(HostEvent::TeachChanged {
                timestamp: timestamp(),
                status: TeachRecordingStatus::default(),
                result: Some(result),
            });
        }
        Ok(())
    }

    fn finalize_teach_capture(
        &self,
        mut active: TeachCaptureProcess,
        save: bool,
    ) -> Result<TeachRecordingResult, FeatureHostError> {
        if let Some(child) = active.child.as_mut() {
            stop_teach_capture(child)?;
        }
        let ended_at_ms = now_millis();
        let duration_ms = (ended_at_ms - active.started_at_ms).clamp(0, TEACH_MAX_DURATION_MS);
        if !save {
            let _ = std::fs::remove_dir_all(&active.session_dir);
            return Ok(TeachRecordingResult {
                agent_id: active.agent_id,
                video_path: active.video_path.to_string_lossy().to_string(),
                started_at_ms: active.started_at_ms,
                ended_at_ms,
                duration_ms,
                saved: false,
            });
        }

        if self.config.mode == HostMode::Test && !active.video_path.exists() {
            std::fs::write(&active.video_path, b"fabushi-test-teach-video").map_err(|error| {
                FeatureHostError::Contract(format!("write teach test fixture: {error}"))
            })?;
        }
        let metadata = std::fs::metadata(&active.video_path).map_err(|error| {
            FeatureHostError::Contract(format!("teach capture did not produce a video: {error}"))
        })?;
        if metadata.len() == 0 {
            return Err(FeatureHostError::Contract(
                "teach capture produced an empty video".into(),
            ));
        }

        let manifest = json!({
            "agentId": active.agent_id,
            "entryPoint": active.entry_point,
            "startedAtMs": active.started_at_ms,
            "endedAtMs": ended_at_ms,
            "durationMs": duration_ms,
            "videoPath": active.video_path,
        });
        std::fs::write(
            active.session_dir.join("session.json"),
            serde_json::to_vec_pretty(&manifest).map_err(|error| {
                FeatureHostError::Contract(format!("serialize teach manifest: {error}"))
            })?,
        )
        .map_err(|error| FeatureHostError::Contract(format!("write teach manifest: {error}")))?;

        if self.config.mode == HostMode::Production {
            let _ = extract_teach_frames(&active.video_path, &active.session_dir.join("frames"));
        }
        let video_path = active.video_path.to_string_lossy().to_string();
        let _ = self.schedule_teach_learning(&active.agent_id, &video_path, &active.session_dir);
        Ok(TeachRecordingResult {
            agent_id: active.agent_id,
            video_path,
            started_at_ms: active.started_at_ms,
            ended_at_ms,
            duration_ms,
            saved: true,
        })
    }

    fn schedule_teach_learning(
        &self,
        agent_id: &str,
        video_path: &str,
        session_dir: &Path,
    ) -> Result<Option<String>, FeatureHostError> {
        let bot = {
            let state = self.state()?;
            find_bot_by_runtime_or_surface_id(&state, agent_id)
                .cloned()
                .ok_or_else(|| {
                    FeatureHostError::Contract(format!("unknown teach agent: {agent_id}"))
                })?
        };
        let frames_dir = session_dir.join("frames").to_string_lossy().to_string();
        let prompt = format!(
            "[teach-recording] The user just demonstrated a repeatable task for you.\nRecording: {video_path}\nExtracted frames (when present): {frames_dir}\n\nStudy the demonstration carefully. Infer the intent, ordered steps, important UI landmarks, decision points, and safety checks. Return a reusable Markdown workflow/skill only: start with a concise # heading, then instructions another future run can follow. Do not merely summarize the recording and do not mention this hidden teach prompt."
        );
        if self.config.mode == HostMode::Test {
            return Ok(None);
        }
        #[cfg(feature = "production")]
        {
            let conversation_id = bot.conversation_id.clone().ok_or_else(|| {
                FeatureHostError::Contract(format!("bot has no conversation: {}", bot.id))
            })?;
            let response = self.runtime()?.execute(RuntimeCommand::SendMessage {
                conversation_id: ConversationId(conversation_id),
                text: prompt,
                client_message_id: Some(format!("teach:{}:{}", bot.id, now_millis())),
                inference_provider: agent_inference_provider_key(bot.inference_provider)?,
                hidden: true,
            })?;
            let operation_id = match response {
                RuntimeResponse::Accepted { operation_id } => operation_id.to_string(),
                other => return Err(unexpected_response("teach.learn", other)),
            };
            let runtime_agent_id = bot_runtime_agent_id(&bot).to_string();
            let mut state = self.state()?;
            state.background_operations.insert(
                operation_id.clone(),
                BackgroundOperationContext {
                    agent_id: runtime_agent_id.clone(),
                    agent_name: bot.name.clone(),
                    source: "teach-recording".into(),
                    teach_artifact: Some(video_path.to_string()),
                },
            );
            state.events.push_back(HostEvent::AgentBackgroundStarted {
                timestamp: timestamp(),
                agent_id: runtime_agent_id,
                agent_name: bot.name,
                operation_id: operation_id.clone(),
                source: "teach-recording".into(),
            });
            Ok(Some(operation_id))
        }
        #[cfg(not(feature = "production"))]
        Ok(None)
    }

    fn persist_teach_workflow(
        &self,
        agent_id: &str,
        artifact: &str,
        markdown: &str,
    ) -> Result<WorkflowSummary, FeatureHostError> {
        let workflow_root = self
            .active_account_root(self.workflow_root_path.as_deref())
            .ok_or_else(|| FeatureHostError::Contract("workflow storage is unavailable".into()))?;
        let body = clamp_block(markdown, 100_000);
        if body.is_empty() {
            return Err(FeatureHostError::Contract(
                "teach learning returned an empty workflow".into(),
            ));
        }
        let name = derive_teach_workflow_name(&body);
        let base_id = slugify_teach_workflow_name(&name);
        let mut id = base_id.clone();
        let mut suffix = 2usize;
        while workflow_root.join(&id).exists() {
            id = format!("{base_id}-{suffix}");
            suffix += 1;
            if suffix > 999 {
                id = format!("{base_id}-{}", now_millis());
                break;
            }
        }
        let folder = workflow_root.join(&id);
        std::fs::create_dir_all(&folder).map_err(|error| {
            FeatureHostError::Contract(format!("create learned workflow: {error}"))
        })?;
        let description = "Learned from a recorded demonstration.".to_string();
        let metadata = json!({
            "name": name,
            "description": description,
            "metadata": { "source": artifact },
        });
        let yaml = serde_yaml::to_string(&metadata).map_err(|error| {
            FeatureHostError::Contract(format!("serialize learned workflow frontmatter: {error}"))
        })?;
        let file_path = folder.join("SKILL.md");
        std::fs::write(&file_path, format!("---\n{yaml}---\n{}\n", body.trim())).map_err(
            |error| FeatureHostError::Contract(format!("write learned workflow: {error}")),
        )?;
        let created_at = now_millis();
        let workflow = WorkflowSummary {
            id,
            name,
            description,
            body,
            trigger: None,
            source_ref: Some(artifact.to_string()),
            source: WorkflowSource::Workflow,
            plugin_id: None,
            published_by_current_user: false,
            is_enabled_for_agent: true,
            disable_model_invocation: None,
            schedule_description: None,
            created_at,
            last_run_at: None,
            next_run_at: None,
            helper_scripts: Vec::new(),
            file_path: file_path.to_string_lossy().to_string(),
        };
        if let Some(agent_root) = self.active_account_root(self.memory_root_path.as_deref()) {
            let _ = set_workflow_enabled(&agent_root, agent_id, &workflow.id, true);
        }
        Ok(workflow)
    }

    fn execute_subagent_observation(
        &self,
        command: FeatureCommand,
    ) -> Result<CommandAccepted, FeatureHostError> {
        let request_id = command.request_id().to_string();
        match command {
            FeatureCommand::SubagentList { agent_id, .. } => {
                let state = self.state()?;
                ensure_open(&state)?;
                let mut subagents = state
                    .subagents
                    .values()
                    .filter(|subagent| subagent.parent_agent_id == agent_id)
                    .cloned()
                    .collect::<Vec<_>>();
                subagents.sort_by_key(|subagent| subagent.started_at_ms);
                drop(state);
                self.state()?.events.push_back(HostEvent::SubagentListed {
                    timestamp: timestamp(),
                    agent_id,
                    subagents,
                });
            }
            FeatureCommand::AsyncTaskList { agent_id, .. } => {
                let state = self.state()?;
                ensure_open(&state)?;
                let mut tasks = state
                    .async_tasks
                    .values()
                    .filter(|task| task.parent_agent_id == agent_id)
                    .cloned()
                    .collect::<Vec<_>>();
                tasks.sort_by_key(|task| task.started_at_ms);
                drop(state);
                self.state()?.events.push_back(HostEvent::AsyncTaskListed {
                    timestamp: timestamp(),
                    agent_id,
                    tasks,
                });
            }
            _ => unreachable!("non-subagent command routed to subagent observer"),
        }
        Ok(CommandAccepted {
            request_id,
            operation_id: None,
        })
    }

    fn execute_agent_workspace_state(
        &self,
        command: FeatureCommand,
    ) -> Result<CommandAccepted, FeatureHostError> {
        let request_id = command.request_id().to_string();
        let (key, value, emit_snapshot) = match command {
            FeatureCommand::AgentWorkspaceStateGet { key, .. } => {
                let value = match self.config.mode {
                    HostMode::Test => self.state()?.workspace_state.get(&key).cloned(),
                    HostMode::Production => {
                        #[cfg(feature = "production")]
                        {
                            match self.runtime()?.execute(RuntimeCommand::UiStateGet {
                                key: key.clone(),
                            })? {
                                RuntimeResponse::RuntimeUiState { value, .. } => value,
                                other => {
                                    return Err(unexpected_response(
                                        "agent.workspaceState.get",
                                        other,
                                    ));
                                }
                            }
                        }
                        #[cfg(not(feature = "production"))]
                        return Err(FeatureHostError::ProductionUnavailable);
                    }
                };
                (key, value, true)
            }
            FeatureCommand::AgentWorkspaceStateSet { key, value, .. } => {
                match self.config.mode {
                    HostMode::Test => {
                        self.state()?.workspace_state.insert(key.clone(), value.clone());
                    }
                    HostMode::Production => {
                        #[cfg(feature = "production")]
                        {
                            match self.runtime()?.execute(RuntimeCommand::UiStateSet {
                                key: key.clone(),
                                value: value.clone(),
                            })? {
                                RuntimeResponse::RuntimeUiState { .. } => {}
                                other => {
                                    return Err(unexpected_response(
                                        "agent.workspaceState.set",
                                        other,
                                    ));
                                }
                            }
                        }
                        #[cfg(not(feature = "production"))]
                        return Err(FeatureHostError::ProductionUnavailable);
                    }
                }
                (key, Some(value), false)
            }
            _ => unreachable!("non-workspace-state command routed to workspace state"),
        };
        if emit_snapshot {
            self.state()?.events.push_back(HostEvent::AgentWorkspaceState {
                timestamp: timestamp(),
                key,
                value,
            });
        }
        Ok(CommandAccepted {
            request_id,
            operation_id: None,
        })
    }

    fn execute_agent_messaging(
        &self,
        command: FeatureCommand,
    ) -> Result<CommandAccepted, FeatureHostError> {
        let request_id = command.request_id().to_string();
        match command {
            FeatureCommand::AgentPeerHistory {
                agent_id, limit, ..
            } => {
                let state = self.state()?;
                ensure_open(&state)?;
                let bot = find_bot_by_runtime_or_surface_id(&state, &agent_id)
                    .cloned()
                    .ok_or_else(|| {
                        FeatureHostError::Contract(format!("unknown bot: {agent_id}"))
                    })?;
                let surface_id = bot.id.clone();
                let agent_id = bot_runtime_agent_id(&bot).to_string();
                let mut messages = state
                    .peer_messages
                    .iter()
                    .filter(|message| {
                        message.from_agent_id == agent_id
                            || message.from_agent_id == surface_id
                            || message.target_id == agent_id
                            || message.target_id == surface_id
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                messages.sort_by_key(|message| message.created_at_ms);
                if messages.len() > limit.min(1000) {
                    let start = messages.len() - limit.min(1000);
                    messages = messages.split_off(start);
                }
                drop(state);
                self.state()?
                    .events
                    .push_back(HostEvent::AgentPeerHistoryListed {
                        timestamp: timestamp(),
                        agent_id,
                        messages,
                    });
                Ok(CommandAccepted {
                    request_id,
                    operation_id: None,
                })
            }
            FeatureCommand::AgentSend {
                from_agent_id,
                target_id,
                text,
                priority,
                ..
            } => {
                let text = clamp_block(&text, 8000);
                if text.is_empty() {
                    return Err(FeatureHostError::Contract(
                        "agent message must not be empty".into(),
                    ));
                }
                let mut kick_group: Option<String> = None;
                let direct_target = {
                    let mut state = self.state()?;
                    ensure_open(&state)?;
                    let sender = find_bot_by_runtime_or_surface_id(&state, &from_agent_id)
                        .cloned()
                        .ok_or_else(|| {
                            FeatureHostError::Contract(format!(
                                "unknown sender bot: {from_agent_id}"
                            ))
                        })?;
                    if let Some(target) =
                        find_bot_by_runtime_or_surface_id(&state, &target_id).cloned()
                    {
                        let sender_agent_id = bot_runtime_agent_id(&sender).to_string();
                        let target_agent_id = bot_runtime_agent_id(&target).to_string();
                        if sender_agent_id == target_agent_id {
                            return Err(FeatureHostError::Contract(
                                "an agent cannot message itself".into(),
                            ));
                        }
                        let peer = AgentPeerMessage {
                            id: next_id(&mut state, "agent-message"),
                            from_agent_id: sender_agent_id,
                            from_agent_name: sender.name.clone(),
                            target_id: target_agent_id,
                            target_name: target.name.clone(),
                            text: text.clone(),
                            priority,
                            created_at_ms: now_millis(),
                        };
                        state.peer_messages.push(peer.clone());
                        if state.peer_messages.len() > 5000 {
                            let overflow = state.peer_messages.len() - 5000;
                            state.peer_messages.drain(0..overflow);
                        }
                        self.persist_peer_messages(&state.peer_messages)?;
                        state.events.push_back(HostEvent::AgentPeerMessageChanged {
                            timestamp: timestamp(),
                            message: peer,
                        });
                        Some((sender, target))
                    } else if let Some(group_snapshot) = state.groups.get(&target_id).cloned() {
                        if !group_snapshot
                            .member_ids
                            .iter()
                            .any(|id| id == &sender.id)
                        {
                            return Err(FeatureHostError::Contract(format!(
                                "agent {} is not a member of group {target_id}",
                                bot_runtime_agent_id(&sender)
                            )));
                        }
                        let now = now_millis();
                        let message_id = next_id(&mut state, "group-message");
                        let group = state
                            .groups
                            .get_mut(&target_id)
                            .expect("group snapshot existed");
                        group.messages.push(GroupMessage {
                            id: message_id,
                            speaker: GroupSpeaker::Member {
                                id: sender.id.clone(),
                                name: sender.name.clone(),
                            },
                            content: text.clone(),
                            created_at_ms: now,
                        });
                        if group.messages.len() > 500 {
                            let overflow = group.messages.len() - 500;
                            group.messages.drain(0..overflow);
                        }
                        group.updated_at_ms = now;
                        let group = group.clone();
                        let mut responders = group
                            .member_ids
                            .iter()
                            .filter(|id| *id != &sender.id)
                            .cloned()
                            .collect::<Vec<_>>();
                        let lower = text.to_lowercase();
                        let mentioned = responders
                            .iter()
                            .filter(|id| {
                                state.bots.get(*id).is_some_and(|bot| {
                                    group_member_handles(&bot.name)
                                        .iter()
                                        .any(|handle| has_group_mention_at(&lower, handle))
                                })
                            })
                            .cloned()
                            .collect::<Vec<_>>();
                        if !mentioned.is_empty() && !has_everyone_group_mention(&lower) {
                            responders = mentioned;
                        }
                        if !responders.is_empty() {
                            let run_id = next_id(&mut state, "group-run");
                            state.group_runs.insert(
                                target_id.clone(),
                                GroupRunState {
                                    run_id,
                                    round: 0,
                                    speaker_order: responders,
                                    speaker_index: 0,
                                    total_messages: 0,
                                    messages_this_round: 0,
                                },
                            );
                            kick_group = Some(target_id.clone());
                        }
                        self.persist_groups(&state.groups)?;
                        state.events.push_back(HostEvent::GroupChanged {
                            timestamp: timestamp(),
                            action: "message".into(),
                            group,
                        });
                        None
                    } else {
                        return Err(FeatureHostError::Contract(format!(
                            "unknown agent or group: {target_id}"
                        )));
                    }
                };

                if let Some(group_id) = kick_group {
                    if self.config.mode == HostMode::Production {
                        #[cfg(feature = "production")]
                        {
                            let _ = self.start_next_group_turn(&group_id)?;
                        }
                    }
                    return Ok(CommandAccepted {
                        request_id,
                        operation_id: None,
                    });
                }

                if let Some((sender, target)) = direct_target {
                    let wake_prompt = build_agent_inbound_wake_prompt(&sender, &text, priority);
                    self.schedule_background_agent_turn(
                        &target,
                        if priority {
                            "agent-priority"
                        } else {
                            "agent-message"
                        },
                        wake_prompt,
                        format!("peer:{}:{}", sender.id, request_id),
                    )?;
                }
                Ok(CommandAccepted {
                    request_id,
                    operation_id: None,
                })
            }
            FeatureCommand::AgentBroadcast {
                target_ids,
                message,
                ..
            } => {
                let message = clamp_block(&message, 8000);
                if message.is_empty() {
                    return Err(FeatureHostError::Contract(
                        "broadcast message must not be empty".into(),
                    ));
                }
                let targets = {
                    let state = self.state()?;
                    ensure_open(&state)?;
                    match target_ids {
                        Some(ids) => {
                            let unique = ids.into_iter().collect::<BTreeSet<_>>();
                            unique
                                .into_iter()
                                .filter_map(|id| {
                                    find_bot_by_runtime_or_surface_id(&state, &id).cloned()
                                })
                                .collect::<Vec<_>>()
                        }
                        None => state.bots.values().cloned().collect::<Vec<_>>(),
                    }
                };
                let total = targets.len();
                let mut scheduled = 0usize;
                for target in targets {
                    let prompt = build_admin_broadcast_wake_prompt(&message);
                    if self
                        .schedule_background_agent_turn(
                            &target,
                            "broadcast",
                            prompt,
                            format!("broadcast:{}:{}", target.id, request_id),
                        )
                        .is_ok()
                    {
                        scheduled += 1;
                    }
                }
                let result = AgentBroadcastResult { total, scheduled };
                self.state()?.events.push_back(HostEvent::AgentBroadcasted {
                    timestamp: timestamp(),
                    result,
                });
                Ok(CommandAccepted {
                    request_id,
                    operation_id: None,
                })
            }
            _ => unreachable!("non-agent-messaging command routed to agent messaging executor"),
        }
    }

    fn schedule_background_agent_turn(
        &self,
        target: &BotSummary,
        source: &str,
        prompt: String,
        client_message_id: String,
    ) -> Result<Option<String>, FeatureHostError> {
        if self.config.mode == HostMode::Test {
            let runtime_agent_id = bot_runtime_agent_id(target).to_string();
            let operation_id = format!("background-test-{}-{}", runtime_agent_id, now_millis());
            let mut state = self.state()?;
            state.events.push_back(HostEvent::AgentBackgroundStarted {
                timestamp: timestamp(),
                agent_id: runtime_agent_id.clone(),
                agent_name: target.name.clone(),
                operation_id: operation_id.clone(),
                source: source.to_string(),
            });
            state.events.push_back(HostEvent::AgentBackgroundMessage {
                timestamp: timestamp(),
                agent_id: runtime_agent_id.clone(),
                agent_name: target.name.clone(),
                operation_id: operation_id.clone(),
                source: source.to_string(),
                text: format!("{} received background work.", target.name),
            });
            state.events.push_back(HostEvent::AgentBackgroundFinished {
                timestamp: timestamp(),
                agent_id: runtime_agent_id,
                agent_name: target.name.clone(),
                operation_id: operation_id.clone(),
                source: source.to_string(),
                error: None,
            });
            return Ok(Some(operation_id));
        }

        #[cfg(feature = "production")]
        {
            let conversation_id = target.conversation_id.clone().ok_or_else(|| {
                FeatureHostError::Contract(format!("bot has no conversation: {}", target.id))
            })?;
            let response = self.runtime()?.execute(RuntimeCommand::SendMessage {
                conversation_id: ConversationId(conversation_id),
                text: prompt,
                client_message_id: Some(client_message_id),
                inference_provider: agent_inference_provider_key(target.inference_provider)?,
                hidden: true,
            })?;
            let operation_id = match response {
                RuntimeResponse::Accepted { operation_id } => operation_id.to_string(),
                other => return Err(unexpected_response("agent.background", other)),
            };
            let runtime_agent_id = bot_runtime_agent_id(target).to_string();
            let mut state = self.state()?;
            state.background_operations.insert(
                operation_id.clone(),
                BackgroundOperationContext {
                    agent_id: runtime_agent_id.clone(),
                    agent_name: target.name.clone(),
                    source: source.to_string(),
                    teach_artifact: None,
                },
            );
            state.events.push_back(HostEvent::AgentBackgroundStarted {
                timestamp: timestamp(),
                agent_id: runtime_agent_id,
                agent_name: target.name.clone(),
                operation_id: operation_id.clone(),
                source: source.to_string(),
            });
            Ok(Some(operation_id))
        }
        #[cfg(not(feature = "production"))]
        Err(FeatureHostError::ProductionUnavailable)
    }

    fn execute_computer(
        &self,
        command: FeatureCommand,
    ) -> Result<CommandAccepted, FeatureHostError> {
        let request_id = command.request_id().to_string();
        let settings = self.state()?.settings.clone();
        match command {
            FeatureCommand::ComputerStatus { .. } => {
                let status = mahayana_computer::status(
                    settings.local_execution,
                    settings.route_egress_locally,
                    settings.remote_control_enabled,
                    settings.ai_computer_control_enabled,
                );
                self.state()?
                    .events
                    .push_back(HostEvent::ComputerStatusChanged {
                        timestamp: timestamp(),
                        request_id: request_id.clone(),
                        status,
                    });
            }
            FeatureCommand::ComputerScreenshot {
                origin,
                agent_id,
                session_id,
                target,
                ..
            } => {
                self.ensure_computer_origin_allowed(
                    origin,
                    session_id.as_deref(),
                    &target,
                    &settings,
                )?;
                let attributed_agent_id = if let Some(requested_agent_id) = agent_id.as_deref() {
                    let state = self.state()?;
                    Some(canonical_runtime_agent_id(&state, requested_agent_id).ok_or_else(|| {
                        FeatureHostError::Contract(format!(
                            "unknown computer-control agent: {requested_agent_id}"
                        ))
                    })?)
                } else {
                    None
                };
                let audit_agent_id = attributed_agent_id
                    .clone()
                    .unwrap_or_else(|| "mahayana-assistant".to_string());
                if !is_safe_memory_agent_id(&audit_agent_id) {
                    return Err(FeatureHostError::Contract(format!(
                        "unsafe computer-control agent: {audit_agent_id}"
                    )));
                }
                let snapshot = if self.config.mode == HostMode::Test {
                    test_computer_snapshot()
                } else {
                    mahayana_computer::capture_screen()
                        .map_err(|error| FeatureHostError::Contract(error.to_string()))?
                };
                let _ = self.append_action_audit(
                    &audit_agent_id,
                    session_id.as_deref(),
                    json!({
                        "kind": "computerScreenshot",
                        "origin": computer_origin_label(origin),
                        "sessionId": session_id,
                        "capturedAtMs": snapshot.captured_at_ms,
                    }),
                );
                self.state()?
                    .events
                    .push_back(HostEvent::ComputerSnapshotCaptured {
                        timestamp: timestamp(),
                        request_id: request_id.clone(),
                        agent_id: attributed_agent_id,
                        origin,
                        snapshot,
                    });
            }
            FeatureCommand::ComputerAction {
                origin,
                agent_id,
                session_id,
                target,
                action,
                then,
                ..
            } => {
                self.ensure_computer_origin_allowed(
                    origin,
                    session_id.as_deref(),
                    &target,
                    &settings,
                )?;
                let attributed_agent_id = if let Some(requested_agent_id) = agent_id.as_deref() {
                    let state = self.state()?;
                    Some(canonical_runtime_agent_id(&state, requested_agent_id).ok_or_else(|| {
                        FeatureHostError::Contract(format!(
                            "unknown computer-control agent: {requested_agent_id}"
                        ))
                    })?)
                } else {
                    None
                };
                let audit_agent_id = attributed_agent_id
                    .clone()
                    .unwrap_or_else(|| "mahayana-assistant".to_string());
                if !is_safe_memory_agent_id(&audit_agent_id) {
                    return Err(FeatureHostError::Contract(format!(
                        "unsafe computer-control agent: {audit_agent_id}"
                    )));
                }
                let mut actions = Vec::with_capacity(1 + then.len());
                actions.push(action);
                actions.extend(then);
                let result = if self.config.mode == HostMode::Test {
                    for action in &actions {
                        mahayana_computer::validate_action(action)
                            .map_err(|error| FeatureHostError::Contract(error.to_string()))?;
                    }
                    if actions.len() > mahayana_host_protocol::COMPUTER_MAX_ACTIONS_PER_CALL {
                        return Err(FeatureHostError::Contract(format!(
                            "at most {} computer actions may be batched",
                            mahayana_host_protocol::COMPUTER_MAX_ACTIONS_PER_CALL
                        )));
                    }
                    ComputerActionResult {
                        origin,
                        actions_executed: actions.len(),
                        snapshot: test_computer_snapshot(),
                    }
                } else {
                    match origin {
                        ComputerControlOrigin::LocalUi => mahayana_computer::execute(&actions, origin),
                        ComputerControlOrigin::RemoteMobile => {
                            let remote_session = session_id.clone().ok_or_else(|| {
                                FeatureHostError::Contract(
                                    "remote computer action requires a sessionId".into(),
                                )
                            })?;
                            let lease = mahayana_computer::ComputerControlLeaseRequest::new(
                                format!("remote:{remote_session}"),
                                remote_session,
                                target
                                    .device_id
                                    .clone()
                                    .unwrap_or_else(|| "local-desktop".to_string()),
                                origin,
                                "remote-human",
                            );
                            mahayana_computer::execute_with_lease(&actions, origin, &lease)
                        }
                        ComputerControlOrigin::Ai => {
                            let lease = mahayana_computer::ComputerControlLeaseRequest::new(
                                format!("agent:{audit_agent_id}"),
                                session_id
                                    .clone()
                                    .unwrap_or_else(|| audit_agent_id.clone()),
                                target
                                    .device_id
                                    .clone()
                                    .unwrap_or_else(|| "local-desktop".to_string()),
                                origin,
                                "agent",
                            );
                            mahayana_computer::execute_with_lease(&actions, origin, &lease)
                        }
                    }
                    .map_err(|error| FeatureHostError::Contract(error.to_string()))?
                };
                let serialized_actions = serde_json::to_value(&actions).unwrap_or(Value::Null);
                self.append_action_audit(
                    &audit_agent_id,
                    session_id.as_deref(),
                    json!({
                        "kind": "computerUse",
                        "origin": computer_origin_label(origin),
                        "sessionId": session_id,
                        "actionCount": result.actions_executed,
                        "actions": serialized_actions,
                        "status": "success",
                    }),
                )?;
                self.state()?
                    .events
                    .push_back(HostEvent::ComputerActionCompleted {
                        timestamp: timestamp(),
                        request_id: request_id.clone(),
                        agent_id: attributed_agent_id,
                        result,
                    });
            }
            _ => unreachable!("non-computer command routed to computer executor"),
        }
        Ok(CommandAccepted {
            request_id,
            operation_id: None,
        })
    }

    fn execute_remote_computer(
        &self,
        command: FeatureCommand,
    ) -> Result<CommandAccepted, FeatureHostError> {
        let request_id = command.request_id().to_string();
        let remote_enabled = self.state()?.settings.remote_control_enabled;
        let (method, action, payload, local_transition) = match command {
            FeatureCommand::RemoteComputerRegister {
                device_id,
                label,
                provider,
                platform,
                app_version,
                capabilities,
                ..
            } => {
                // Registration is device presence, not authorization to control.
                // Keeping it available while remote control is disabled lets the
                // signed-in account discover this installed desktop. Session
                // polling, activation, signaling, screenshots, and input remain
                // gated below by `remote_control_enabled`.
                let device_secret = self.remote_device_secret(&device_id, true)?;
                let provider = provider.unwrap_or_else(|| "fabushi-webrtc".to_string());
                let platform = platform.unwrap_or_else(|| "unknown".to_string());
                let app_version = app_version.unwrap_or_else(|| "unknown".to_string());
                (
                    "mahayana.remote.computer.register",
                    "registered",
                    json!({
                        "deviceId": device_id,
                        "label": label,
                        "deviceSecret": device_secret,
                        "provider": provider,
                        "platform": platform,
                        "appVersion": app_version,
                        "capabilities": capabilities,
                    }),
                    None,
                )
            }
            FeatureCommand::RemoteComputerHeartbeat { device_id, .. } => {
                let device_secret = self.remote_device_secret(&device_id, false)?;
                (
                    "mahayana.remote.computer.heartbeat",
                    "heartbeat",
                    json!({"deviceId": device_id, "deviceSecret": device_secret}),
                    None,
                )
            }
            FeatureCommand::RemoteComputerClients { device_id, .. } => (
                "mahayana.remote.computer.clients",
                "clients",
                json!({"deviceId": device_id}),
                None,
            ),
            FeatureCommand::RemoteComputerClientRevoke {
                device_id,
                client_id,
                ..
            } => (
                "mahayana.remote.computer.client.revoke",
                "clientRevoked",
                json!({"deviceId": device_id, "clientId": client_id}),
                Some(("revoke-client".to_string(), client_id)),
            ),
            FeatureCommand::RemoteComputerSessions { device_id, .. } => {
                if !remote_enabled {
                    return Err(FeatureHostError::Contract(
                        "remote computer control is disabled".into(),
                    ));
                }
                let device_secret = self.remote_device_secret(&device_id, false)?;
                (
                    "mahayana.remote.computer.sessions",
                    "sessions",
                    json!({"deviceId": device_id, "deviceSecret": device_secret}),
                    None,
                )
            }
            FeatureCommand::RemoteComputerSessionActivate {
                device_id,
                session_id,
                ..
            } => {
                if !remote_enabled {
                    return Err(FeatureHostError::Contract(
                        "remote computer control is disabled".into(),
                    ));
                }
                let device_secret = self.remote_device_secret(&device_id, false)?;
                (
                    "mahayana.remote.computer.session.activate",
                    "sessionActivated",
                    json!({"deviceId": device_id, "sessionId": session_id, "deviceSecret": device_secret}),
                    Some(("activate".to_string(), session_id)),
                )
            }
            FeatureCommand::RemoteComputerSessionClose {
                device_id,
                session_id,
                ..
            } => {
                mahayana_computer::release_control_lease(
                    &format!("remote:{session_id}"),
                    &session_id,
                );
                let device_secret = self.remote_device_secret(&device_id, false)?;
                (
                    "mahayana.remote.computer.session.close",
                    "sessionClosed",
                    json!({"deviceId": device_id, "sessionId": session_id, "role": "desktop", "deviceSecret": device_secret}),
                    Some(("close".to_string(), session_id)),
                )
            }
            FeatureCommand::RemoteComputerSignal {
                device_id,
                session_id,
                kind,
                payload,
                ..
            } => {
                if !remote_enabled {
                    return Err(FeatureHostError::Contract(
                        "remote computer control is disabled".into(),
                    ));
                }
                let device_secret = self.remote_device_secret(&device_id, false)?;
                (
                    "mahayana.remote.computer.signal",
                    "signal",
                    json!({
                        "deviceId": device_id,
                        "sessionId": session_id,
                        "senderRole": "desktop",
                        "deviceSecret": device_secret,
                        "kind": kind,
                        "payload": payload,
                    }),
                    None,
                )
            }
            FeatureCommand::RemoteComputerSignalDrain {
                device_id,
                session_id,
                after_signal_id,
                ..
            } => {
                if !remote_enabled {
                    return Err(FeatureHostError::Contract(
                        "remote computer control is disabled".into(),
                    ));
                }
                let device_secret = self.remote_device_secret(&device_id, false)?;
                (
                    "mahayana.remote.computer.signals.drain",
                    "signals",
                    json!({
                        "deviceId": device_id,
                        "sessionId": session_id,
                        "receiverRole": "desktop",
                        "deviceSecret": device_secret,
                        "afterSignalId": after_signal_id.max(0),
                    }),
                    None,
                )
            }
            _ => unreachable!("non-remote-computer command routed to remote computer executor"),
        };

        let mut data = if self.config.mode == HostMode::Test {
            match action {
                "registered" => json!({
                    "deviceId": payload.get("deviceId"),
                    "label": payload.get("label"),
                    "pairingCode": "AB12CD34EF56",
                    "pairingExpiresAt": now_millis() / 1000 + 600,
                }),
                "heartbeat" => json!({"ok": true, "lastSeenAt": now_millis() / 1000}),
                "clients" => json!({"deviceId": payload.get("deviceId"), "clients": []}),
                "sessions" => json!({"deviceId": payload.get("deviceId"), "sessions": []}),
                "sessionActivated" => json!({
                    "sessionId": payload.get("sessionId"),
                    "clientId": "remote-client-test",
                    "expiresAt": now_millis() / 1000 + 7200,
                    "state": "active",
                }),
                "sessionClosed" => {
                    json!({"sessionId": payload.get("sessionId"), "state": "closed"})
                }
                "signals" => {
                    json!({"sessionId": payload.get("sessionId"), "signals": [], "lastSignalId": payload.get("afterSignalId")})
                }
                _ => json!({"ok": true}),
            }
        } else {
            #[cfg(feature = "production")]
            {
                self.runtime()?.product_execute(method, &payload)?
            }
            #[cfg(not(feature = "production"))]
            {
                return Err(FeatureHostError::ProductionUnavailable);
            }
        };

        if let Some((transition, id)) = local_transition {
            match transition.as_str() {
                "activate" => {
                    let client_id = data
                        .get("clientId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| {
                            FeatureHostError::Contract(
                                "remote session activation did not return clientId".into(),
                            )
                        })?
                        .to_string();
                    let expires_at_seconds = data
                        .get("expiresAt")
                        .and_then(Value::as_i64)
                        .ok_or_else(|| {
                            FeatureHostError::Contract(
                                "remote session activation did not return expiresAt".into(),
                            )
                        })?;
                    let device_id = payload
                        .get("deviceId")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    let generation = {
                        let mut state = self.state()?;
                        state.sequence = state.sequence.saturating_add(1);
                        let generation = state.sequence;
                        state.remote_computer_sessions.insert(
                            id,
                            RemoteComputerLocalSession {
                                device_id,
                                client_id,
                                expires_at_seconds,
                                generation,
                            },
                        );
                        generation
                    };
                    if let Some(object) = data.as_object_mut() {
                        object.insert("generation".into(), json!(generation));
                    }
                }
                "close" => {
                    self.state()?.remote_computer_sessions.remove(&id);
                }
                "revoke-client" => {
                    self.state()?
                        .remote_computer_sessions
                        .retain(|_, session| session.client_id != id);
                }
                _ => {}
            }
        }

        self.state()?
            .events
            .push_back(HostEvent::RemoteComputerChanged {
                timestamp: timestamp(),
                request_id: request_id.clone(),
                action: action.to_string(),
                data,
            });
        Ok(CommandAccepted {
            request_id,
            operation_id: None,
        })
    }

    fn ensure_computer_origin_allowed(
        &self,
        origin: ComputerControlOrigin,
        session_id: Option<&str>,
        target: &ComputerControlTarget,
        settings: &ProductHostSettings,
    ) -> Result<(), FeatureHostError> {
        if target.protocol_version != COMPUTER_CONTROL_PROTOCOL_VERSION {
            return Err(FeatureHostError::Contract(format!(
                "unsupported computer-control protocol version: {}",
                target.protocol_version
            )));
        }
        if target.kind != ComputerTargetKind::Desktop {
            return Err(FeatureHostError::Contract(
                "desktop computer action cannot target a window or browser tab; use the target-specific capability".into(),
            ));
        }
        match origin {
            ComputerControlOrigin::LocalUi => {
                if !settings.local_execution {
                    return Err(FeatureHostError::Contract(
                        "local computer control is disabled in settings".into(),
                    ));
                }
            }
            ComputerControlOrigin::RemoteMobile => {
                if !settings.remote_control_enabled {
                    return Err(FeatureHostError::Contract(
                        "remote computer control is disabled in settings".into(),
                    ));
                }
                let session_id = session_id
                    .filter(|session| !session.trim().is_empty())
                    .ok_or_else(|| {
                        FeatureHostError::Contract(
                            "remote computer control requires an active paired session id".into(),
                        )
                    })?;
                let mut state = self.state()?;
                let now = now_millis() / 1000;
                state
                    .remote_computer_sessions
                    .retain(|_, session| session.expires_at_seconds > now);
                let Some(session) = state.remote_computer_sessions.get(session_id) else {
                    return Err(FeatureHostError::Contract(
                        "remote computer session is not active on this desktop".into(),
                    ));
                };
                if session.device_id.trim().is_empty() || session.client_id.trim().is_empty() {
                    return Err(FeatureHostError::Contract(
                        "remote computer session metadata is invalid".into(),
                    ));
                }
                if target.device_id.as_deref() != Some(session.device_id.as_str()) {
                    return Err(FeatureHostError::Contract(
                        "remote computer target device does not match the active paired session"
                            .into(),
                    ));
                }
                if target.generation != session.generation {
                    return Err(FeatureHostError::Contract(
                        "remote computer target generation is stale or mismatched".into(),
                    ));
                }
            }
            ComputerControlOrigin::Ai => {
                if !settings.local_execution || !settings.ai_computer_control_enabled {
                    return Err(FeatureHostError::Contract(
                        "AI computer control is disabled in settings".into(),
                    ));
                }
                match settings.local_tool_permission {
                    LocalToolPermission::Never => {
                        return Err(FeatureHostError::Contract(
                            "AI local-tool permission is set to Never".into(),
                        ));
                    }
                    LocalToolPermission::Ask => {
                        return Err(FeatureHostError::Contract(
                            "AI computer control requires an explicit approval while local-tool permission is Ask"
                                .into(),
                        ));
                    }
                    LocalToolPermission::Always => {}
                }
            }
        }
        Ok(())
    }

    fn execute_memory(&self, command: FeatureCommand) -> Result<CommandAccepted, FeatureHostError> {
        let request_id = command.request_id().to_string();
        let (agent_id, action) = match command {
            FeatureCommand::MemoryList {
                agent_id, limit, ..
            } => (agent_id, MemoryAction::List { limit }),
            FeatureCommand::MemoryAdd {
                agent_id,
                content,
                kind,
                ..
            } => (agent_id, MemoryAction::Add { content, kind }),
            FeatureCommand::MemoryRemove { agent_id, id, .. } => {
                (agent_id, MemoryAction::Remove { id })
            }
            FeatureCommand::MemoryClear { agent_id, .. } => (agent_id, MemoryAction::Clear),
            _ => unreachable!("non-memory command routed to memory executor"),
        };
        let requested_agent_id = agent_id;
        let agent_id = {
            let state = self.state()?;
            ensure_open(&state)?;
            canonical_runtime_agent_id(&state, &requested_agent_id).ok_or_else(|| {
                FeatureHostError::Contract(format!("unknown bot: {requested_agent_id}"))
            })?
        };
        if !is_safe_memory_agent_id(&agent_id) {
            return Err(FeatureHostError::Contract(format!(
                "unsafe memory agent id: {agent_id}"
            )));
        }
        let root = self
            .active_account_root(self.memory_root_path.as_deref())
            .ok_or_else(|| FeatureHostError::Contract("memory storage is unavailable".into()))?;
        let memory_dir = root.join(&agent_id).join("memory");
        match action {
            MemoryAction::List { limit } => {
                let memories = list_memories(&memory_dir, limit.min(1000))?;
                let count = count_memories(&memory_dir)?;
                self.state()?.events.push_back(HostEvent::MemoryListed {
                    timestamp: timestamp(),
                    agent_id,
                    memories,
                    count,
                    location: Some(memory_dir.to_string_lossy().into_owned()),
                });
            }
            MemoryAction::Add { content, kind } => {
                let memory = add_memory(&memory_dir, &content, now_millis(), kind)?;
                self.state()?.events.push_back(HostEvent::MemoryChanged {
                    timestamp: timestamp(),
                    agent_id,
                    action: if memory.is_some() {
                        "added"
                    } else {
                        "duplicate"
                    }
                    .into(),
                    memory,
                });
            }
            MemoryAction::Remove { id } => {
                let removed = remove_memory(&memory_dir, &id)?;
                self.state()?.events.push_back(HostEvent::MemoryChanged {
                    timestamp: timestamp(),
                    agent_id,
                    action: if removed { "removed" } else { "notFound" }.into(),
                    memory: None,
                });
            }
            MemoryAction::Clear => {
                if memory_dir.exists() {
                    std::fs::remove_dir_all(&memory_dir).map_err(|error| {
                        FeatureHostError::Contract(format!("clear memory: {error}"))
                    })?;
                }
                self.state()?.events.push_back(HostEvent::MemoryChanged {
                    timestamp: timestamp(),
                    agent_id,
                    action: "cleared".into(),
                    memory: None,
                });
            }
        }
        Ok(CommandAccepted {
            request_id,
            operation_id: None,
        })
    }

    fn execute_tray(&self, command: FeatureCommand) -> Result<CommandAccepted, FeatureHostError> {
        let request_id = command.request_id().to_string();
        let mut state = self.state()?;
        ensure_open(&state)?;
        match command {
            FeatureCommand::TrayList { .. } => {
                let trays = state.trays.clone();
                state.events.push_back(HostEvent::TrayListed {
                    timestamp: timestamp(),
                    trays,
                });
            }
            FeatureCommand::TrayDismiss { id, .. } => {
                let before = state.trays.len();
                state.trays.retain(|tray| tray.id != id);
                if state.trays.len() != before {
                    state.events.push_back(HostEvent::TrayChanged {
                        timestamp: timestamp(),
                        action: "dismissed".into(),
                        tray: None,
                        id: Some(id),
                    });
                }
            }
            FeatureCommand::TrayClear { .. } => {
                if !state.trays.is_empty() {
                    state.trays.clear();
                    state.events.push_back(HostEvent::TrayChanged {
                        timestamp: timestamp(),
                        action: "cleared".into(),
                        tray: None,
                        id: None,
                    });
                }
            }
            FeatureCommand::TrayClearForAgent { agent_id, .. } => {
                let removed = state
                    .trays
                    .iter()
                    .filter(|tray| tray.agent_id == agent_id)
                    .map(|tray| tray.id.clone())
                    .collect::<Vec<_>>();
                state.trays.retain(|tray| tray.agent_id != agent_id);
                for id in removed {
                    state.events.push_back(HostEvent::TrayChanged {
                        timestamp: timestamp(),
                        action: "dismissed".into(),
                        tray: None,
                        id: Some(id),
                    });
                }
            }
            _ => unreachable!("non-tray command routed to tray executor"),
        }
        Ok(CommandAccepted {
            request_id,
            operation_id: None,
        })
    }

    fn execute_workflow(
        &self,
        command: FeatureCommand,
    ) -> Result<CommandAccepted, FeatureHostError> {
        let request_id = command.request_id().to_string();
        let workflow_root_buf = self
            .active_account_root(self.workflow_root_path.as_deref())
            .ok_or_else(|| FeatureHostError::Contract("workflow storage is unavailable".into()))?;
        let agent_root_buf = self
            .active_account_root(self.memory_root_path.as_deref())
            .ok_or_else(|| FeatureHostError::Contract("agent storage is unavailable".into()))?;
        let workflow_root = workflow_root_buf.as_path();
        let agent_root = agent_root_buf.as_path();
        let agent_id = match &command {
            FeatureCommand::WorkflowList { agent_id, .. }
            | FeatureCommand::WorkflowUpsert { agent_id, .. }
            | FeatureCommand::WorkflowSetEnabled { agent_id, .. }
            | FeatureCommand::WorkflowDelete { agent_id, .. }
            | FeatureCommand::WorkflowRun { agent_id, .. }
            | FeatureCommand::WorkflowImportMarkdown { agent_id, .. }
            | FeatureCommand::WorkflowImportLiveSource { agent_id, .. } => agent_id.clone(),
            _ => unreachable!("non-workflow command routed to workflow executor"),
        };
        let requested_agent_id = agent_id;
        let agent_id = {
            let state = self.state()?;
            ensure_open(&state)?;
            canonical_runtime_agent_id(&state, &requested_agent_id).ok_or_else(|| {
                FeatureHostError::Contract(format!("unknown bot: {requested_agent_id}"))
            })?
        };
        if !is_safe_memory_agent_id(&agent_id) {
            return Err(FeatureHostError::Contract(format!(
                "unsafe workflow agent id: {agent_id}"
            )));
        }

        match command {
            FeatureCommand::WorkflowList { .. } => {
                let mut workflows = list_workflow_summaries(workflow_root, agent_root, &agent_id);
                let automations = {
                    let state = self.state()?;
                    state
                        .automations
                        .values()
                        .filter(|automation| {
                            automation
                                .agent_id
                                .as_deref()
                                .is_none_or(|owner| owner == agent_id.as_str())
                        })
                        .map(workflow_from_automation)
                        .collect::<Vec<_>>()
                };
                workflows.extend(automations);
                workflows.truncate(WORKFLOW_UI_LIMIT);
                self.state()?.events.push_back(HostEvent::WorkflowListed {
                    timestamp: timestamp(),
                    agent_id,
                    workflows,
                });
            }
            FeatureCommand::WorkflowUpsert {
                id,
                name,
                description,
                body,
                trigger,
                source_ref,
                ..
            } => {
                if let Some(mut trigger) = trigger {
                    trigger.schedule = normalize_automation_schedule(&trigger.schedule)?;
                    let now = now_millis();
                    let automation_id = id.clone().unwrap_or_else(|| slugify_workflow_name(&name));
                    let automation = {
                        let mut state = self.state()?;
                        let automation_key =
                            automation_state_key(&agent_id, &automation_id);
                        let created_at_ms = state
                            .automations
                            .get(&automation_key)
                            .map(|automation| automation.created_at_ms)
                            .unwrap_or(now);
                        let last_run_at_ms = state
                            .automations
                            .get(&automation_key)
                            .and_then(|automation| automation.last_run_at_ms);
                        let automation = AutomationSummary {
                            id: automation_id.clone(),
                            agent_id: Some(agent_id.clone()),
                            name: clamp_workflow_name(&name),
                            prompt: clamp_workflow_body(&body),
                            schedule: trigger.schedule.clone(),
                            trigger: Some(AutomationTrigger::Schedule {
                                schedule: trigger.schedule.clone(),
                            }),
                            enabled: trigger.is_enabled,
                            created_at_ms,
                            last_run_at_ms,
                            next_run_at_ms: trigger
                                .is_enabled
                                .then(|| next_automation_run(&trigger.schedule, now))
                                .flatten(),
                        };
                        state
                            .automations
                            .insert(automation_key, automation.clone());
                        self.persist_automations(&state.automations)?;
                        automation
                    };
                    let workflow_dir = workflow_root.join(&automation_id);
                    if workflow_dir.exists() {
                        let _ = std::fs::remove_dir_all(&workflow_dir);
                    }
                    let workflow = workflow_from_automation(&automation);
                    self.state()?.events.push_back(HostEvent::WorkflowChanged {
                        timestamp: timestamp(),
                        agent_id,
                        action: "saved".into(),
                        workflow: Some(workflow),
                        id: None,
                    });
                } else {
                    if let Some(existing_id) = id.as_deref() {
                        let removed_automation = {
                            let mut state = self.state()?;
                            let automation_key =
                                automation_state_key(&agent_id, existing_id);
                            if let Some(existing) = state.automations.get(&automation_key) {
                                ensure_automation_agent_scope(existing, Some(agent_id.as_str()))?;
                            }
                            let removed = state.automations.remove(&automation_key).is_some();
                            if removed {
                                self.persist_automations(&state.automations)?;
                            }
                            removed
                        };
                        if removed_automation {
                            // Converting a scheduled automation back into a trigger-less workflow.
                        }
                    }
                    let workflow = write_workflow(
                        workflow_root,
                        agent_root,
                        &agent_id,
                        id.as_deref(),
                        &name,
                        &description,
                        &body,
                        None,
                        source_ref.as_deref(),
                    )?;
                    self.state()?.events.push_back(HostEvent::WorkflowChanged {
                        timestamp: timestamp(),
                        agent_id,
                        action: "saved".into(),
                        workflow: Some(workflow),
                        id: None,
                    });
                }
            }
            FeatureCommand::WorkflowSetEnabled { id, enabled, .. } => {
                let automation = {
                    let mut state = self.state()?;
                    let automation_key = automation_state_key(&agent_id, &id);
                    if let Some(automation) = state.automations.get_mut(&automation_key) {
                        ensure_automation_agent_scope(automation, Some(agent_id.as_str()))?;
                        automation.enabled = enabled;
                        automation.next_run_at_ms = if enabled {
                            next_automation_run(&automation.schedule, now_millis())
                        } else {
                            None
                        };
                        let automation = automation.clone();
                        self.persist_automations(&state.automations)?;
                        Some(automation)
                    } else {
                        None
                    }
                };
                let workflow = if let Some(automation) = automation {
                    Some(workflow_from_automation(&automation))
                } else {
                    set_workflow_enabled(agent_root, &agent_id, &id, enabled)?;
                    load_workflow_summary(workflow_root, agent_root, &agent_id, &id)
                };
                self.state()?.events.push_back(HostEvent::WorkflowChanged {
                    timestamp: timestamp(),
                    agent_id,
                    action: "enabled".into(),
                    workflow,
                    id: Some(id),
                });
            }
            FeatureCommand::WorkflowDelete { id, .. } => {
                let removed_automation = {
                    let mut state = self.state()?;
                    let automation_key = automation_state_key(&agent_id, &id);
                    if let Some(existing) = state.automations.get(&automation_key) {
                        ensure_automation_agent_scope(existing, Some(agent_id.as_str()))?;
                    }
                    let removed = state.automations.remove(&automation_key).is_some();
                    if removed {
                        self.persist_automations(&state.automations)?;
                    }
                    removed
                };
                if !removed_automation {
                    let path = workflow_root.join(&id);
                    if path.exists() {
                        std::fs::remove_dir_all(&path).map_err(|error| {
                            FeatureHostError::Contract(format!("delete workflow: {error}"))
                        })?;
                    }
                    forget_workflow_enablement(agent_root, &agent_id, &id)?;
                }
                self.state()?.events.push_back(HostEvent::WorkflowChanged {
                    timestamp: timestamp(),
                    agent_id,
                    action: "deleted".into(),
                    workflow: None,
                    id: Some(id),
                });
            }
            FeatureCommand::WorkflowRun { id, .. } => {
                if self
                    .state()?
                    .automations
                    .contains_key(&automation_state_key(&agent_id, &id))
                {
                    return self.execute_automation(FeatureCommand::AutomationRun {
                        request_id,
                        id,
                        agent_id: Some(agent_id),
                    });
                }
                let workflow = load_workflow_summary(workflow_root, agent_root, &agent_id, &id)
                    .ok_or_else(|| FeatureHostError::Contract(format!("unknown workflow: {id}")))?;
                if !workflow.is_enabled_for_agent {
                    return Err(FeatureHostError::Contract(format!(
                        "workflow is disabled for {agent_id}: {id}"
                    )));
                }
                let visible_text = format!("@{}", workflow.name);
                let recipe = clamp_block(&workflow.body, WORKFLOW_INJECTED_BODY_LIMIT);
                let runtime_text = format!(
                    "[Workflow reference: {}]\nFollow this recipe for the current turn:\n{}\n\n[Visible user message]\n{}",
                    workflow.name, recipe, visible_text
                );
                match self.config.mode {
                    HostMode::Test => {
                        let mut state = self.state()?;
                        state.events.push_back(HostEvent::ChatMessage {
                            timestamp: timestamp(),
                            role: MessageRole::User,
                            text: visible_text,
                            operation_id: None,
                        });
                        state.events.push_back(HostEvent::ChatMessage {
                            timestamp: timestamp(),
                            role: MessageRole::Assistant,
                            text: format!("Running workflow: {}", workflow.name),
                            operation_id: None,
                        });
                        return Ok(CommandAccepted {
                            request_id,
                            operation_id: None,
                        });
                    }
                    HostMode::Production => {
                        #[cfg(feature = "production")]
                        {
                            let (conversation_id, inference_provider) = {
                                let state = self.state()?;
                                let bot = find_bot_by_runtime_or_surface_id(&state, &agent_id)
                                    .ok_or_else(|| FeatureHostError::Contract(format!(
                                        "unknown bot: {agent_id}"
                                    )))?;
                                let conversation_id = bot.conversation_id.clone().ok_or_else(|| {
                                    FeatureHostError::Contract(format!(
                                        "bot has no conversation: {agent_id}"
                                    ))
                                })?;
                                (conversation_id, agent_inference_provider_key(bot.inference_provider)?)
                            };
                            let (provider, model) = match self
                                .runtime()?
                                .execute(RuntimeCommand::Status)?
                            {
                                RuntimeResponse::Status(status) => (
                                    format!("{:?}", status.model_provider).to_lowercase(),
                                    status.model,
                                ),
                                other => return Err(unexpected_response("runtime.status", other)),
                            };
                            let response =
                                self.runtime()?.execute(RuntimeCommand::SendMessage {
                                    conversation_id: ConversationId(conversation_id),
                                    text: runtime_text,
                                    client_message_id: Some(request_id.clone()),
                                    inference_provider,
                                    hidden: true,
                                })?;
                            let operation_id = match response {
                                RuntimeResponse::Accepted { operation_id } => {
                                    operation_id.to_string()
                                }
                                other => return Err(unexpected_response("workflow.run", other)),
                            };
                            let mut state = self.state()?;
                            state.operations.insert(operation_id.clone());
                            state
                                .operation_agents
                                .insert(operation_id.clone(), agent_id);
                            state.events.push_back(HostEvent::ModelRouted {
                                timestamp: timestamp(),
                                operation_id: operation_id.clone(),
                                provider,
                                model,
                                mode: AgentMode::Agent,
                            });
                            state.events.push_back(HostEvent::ChatMessage {
                                timestamp: timestamp(),
                                role: MessageRole::User,
                                text: visible_text,
                                operation_id: None,
                            });
                            state.events.push_back(HostEvent::OperationStarted {
                                timestamp: timestamp(),
                                operation_id: operation_id.clone(),
                                label: format!("workflow:{}", workflow.id),
                                interruptible: true,
                            });
                            return Ok(CommandAccepted {
                                request_id,
                                operation_id: Some(operation_id),
                            });
                        }
                        #[cfg(not(feature = "production"))]
                        return Err(FeatureHostError::ProductionUnavailable);
                    }
                }
            }
            FeatureCommand::WorkflowImportMarkdown {
                markdown,
                fallback_name,
                ..
            } => {
                let parsed = parse_workflow_file(&markdown).ok_or_else(|| {
                    FeatureHostError::Contract("workflow markdown is empty".into())
                })?;
                let name = if parsed.name.is_empty() {
                    derive_workflow_name_from_markdown(&parsed.body)
                        .or(fallback_name.map(|name| clamp_workflow_name(&name)))
                        .unwrap_or_default()
                } else {
                    parsed.name.clone()
                };
                if name.is_empty() || parsed.body.is_empty() {
                    return Err(FeatureHostError::Contract(
                        "workflow markdown must include a name and body".into(),
                    ));
                }
                let workflow = if let Some(mut trigger) = parsed.trigger.clone() {
                    trigger.schedule = normalize_automation_schedule(&trigger.schedule)?;
                    let now = now_millis();
                    let id = slugify_workflow_name(&name);
                    let automation = AutomationSummary {
                        id: id.clone(),
                        agent_id: Some(agent_id.clone()),
                        name: name.clone(),
                        prompt: parsed.body.clone(),
                        schedule: trigger.schedule.clone(),
                        trigger: Some(AutomationTrigger::Schedule {
                            schedule: trigger.schedule.clone(),
                        }),
                        enabled: trigger.is_enabled,
                        created_at_ms: now,
                        last_run_at_ms: None,
                        next_run_at_ms: trigger
                            .is_enabled
                            .then(|| next_automation_run(&trigger.schedule, now))
                            .flatten(),
                    };
                    {
                        let mut state = self.state()?;
                        state.automations.insert(
                            automation_state_key(&agent_id, &id),
                            automation.clone(),
                        );
                        self.persist_automations(&state.automations)?;
                    }
                    workflow_from_automation(&automation)
                } else {
                    write_workflow(
                        workflow_root,
                        agent_root,
                        &agent_id,
                        None,
                        &name,
                        &parsed.description,
                        &parsed.body,
                        None,
                        parsed.source_ref.as_deref(),
                    )?
                };
                self.state()?.events.push_back(HostEvent::WorkflowChanged {
                    timestamp: timestamp(),
                    agent_id,
                    action: "imported".into(),
                    workflow: Some(workflow),
                    id: None,
                });
            }
            FeatureCommand::WorkflowImportLiveSource {
                source,
                fallback_name,
                ..
            } => {
                let source = source.trim().to_string();
                if source.is_empty() {
                    return Err(FeatureHostError::Contract(
                        "workflow live source must not be empty".into(),
                    ));
                }
                let name = fallback_name
                    .map(|name| clamp_workflow_name(&name))
                    .filter(|name| !name.is_empty())
                    .unwrap_or_else(|| derive_workflow_name_from_source(&source));
                let description = build_live_source_description(&name, &source);
                let body = build_live_source_pointer_body(&source);
                let workflow = write_workflow(
                    workflow_root,
                    agent_root,
                    &agent_id,
                    None,
                    &name,
                    &description,
                    &body,
                    None,
                    Some(&source),
                )?;
                self.state()?.events.push_back(HostEvent::WorkflowChanged {
                    timestamp: timestamp(),
                    agent_id,
                    action: "imported".into(),
                    workflow: Some(workflow),
                    id: None,
                });
            }
            _ => unreachable!("non-workflow command routed to workflow executor"),
        }
        Ok(CommandAccepted {
            request_id,
            operation_id: None,
        })
    }

    fn execute_attachment(
        &self,
        command: FeatureCommand,
    ) -> Result<CommandAccepted, FeatureHostError> {
        let request_id = command.request_id().to_string();
        let agent_root = self
            .active_account_root(self.memory_root_path.as_deref())
            .ok_or_else(|| {
                FeatureHostError::Contract("attachment storage is unavailable".into())
            })?;
        match command {
            FeatureCommand::AttachmentUpload {
                agent_id,
                filename,
                mime_type,
                bytes_base64,
                ..
            } => {
                let requested_agent_id = agent_id;
                let agent_id = {
                    let state = self.state()?;
                    canonical_runtime_agent_id(&state, &requested_agent_id).ok_or_else(|| {
                        FeatureHostError::Contract(format!(
                            "unknown attachment owner: {requested_agent_id}"
                        ))
                    })?
                };
                if !is_safe_memory_agent_id(&agent_id) {
                    return Err(FeatureHostError::Contract(format!(
                        "unsafe attachment owner: {agent_id}"
                    )));
                }
                let filename = filename.trim();
                if filename.is_empty() {
                    return Err(FeatureHostError::Contract(
                        "attachment filename must not be empty".into(),
                    ));
                }
                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(bytes_base64.trim())
                    .map_err(|error| {
                        FeatureHostError::Contract(format!("invalid attachment base64: {error}"))
                    })?;
                if bytes.is_empty() {
                    return Err(FeatureHostError::Contract("attachment is empty".into()));
                }
                let limit = attachment_byte_limit_for_name(filename);
                if bytes.len() as u64 > limit {
                    return Err(FeatureHostError::Contract(format!(
                        "attachment exceeds {} bytes",
                        limit
                    )));
                }
                let hash = format!("{:x}", Sha256::digest(&bytes));
                let extension = Path::new(filename)
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .map(|extension| extension.to_ascii_lowercase())
                    .filter(|extension| {
                        extension
                            .chars()
                            .all(|character| character.is_ascii_alphanumeric())
                    })
                    .filter(|extension| !extension.is_empty())
                    .unwrap_or_else(|| "bin".into());
                let attachments_dir = agent_root.join(&agent_id).join("attachments");
                std::fs::create_dir_all(&attachments_dir).map_err(|error| {
                    FeatureHostError::Contract(format!("create attachment directory: {error}"))
                })?;
                let path = attachments_dir.join(format!("{hash}.{extension}"));
                if !path.exists() {
                    std::fs::write(&path, &bytes).map_err(|error| {
                        FeatureHostError::Contract(format!("write attachment: {error}"))
                    })?;
                }
                let attachment = AttachmentStored {
                    id: hash.clone(),
                    agent_id,
                    name: filename.to_string(),
                    path: path.to_string_lossy().to_string(),
                    mime_type: clean_optional_string(mime_type)
                        .or_else(|| media_mime_type(filename).map(str::to_string)),
                    size_bytes: bytes.len() as u64,
                    hash,
                };
                self.state()?.events.push_back(HostEvent::AttachmentStored {
                    timestamp: timestamp(),
                    attachment,
                });
            }
            FeatureCommand::AttachmentReadText { agent_id, path, .. } => {
                let resolved = resolve_agent_attachment_path(&agent_root, &agent_id, &path)?;
                let metadata = std::fs::metadata(&resolved).map_err(|error| {
                    FeatureHostError::Contract(format!("read attachment metadata: {error}"))
                })?;
                let bytes = metadata.len();
                let preview = read_file_prefix(&resolved, ATTACHMENT_TEXT_PREVIEW_BYTE_CAP)?;
                let binary = !is_text_previewable_name(&resolved) || looks_like_binary(&preview);
                let result = AttachmentTextResult {
                    path: resolved.to_string_lossy().to_string(),
                    kind: if binary { "binary" } else { "text" }.into(),
                    text: (!binary).then(|| String::from_utf8_lossy(&preview).to_string()),
                    truncated: !binary && bytes > ATTACHMENT_TEXT_PREVIEW_BYTE_CAP as u64,
                    bytes,
                };
                self.state()?
                    .events
                    .push_back(HostEvent::AttachmentTextRead {
                        timestamp: timestamp(),
                        result,
                    });
            }
            FeatureCommand::AttachmentReadChunk {
                agent_id,
                path,
                offset,
                length,
                ..
            } => {
                let resolved = resolve_agent_attachment_path(&agent_root, &agent_id, &path)?;
                let metadata = std::fs::metadata(&resolved).map_err(|error| {
                    FeatureHostError::Contract(format!("read attachment metadata: {error}"))
                })?;
                let total_size = metadata.len();
                let start = offset.min(total_size);
                let length = length
                    .min(ATTACHMENT_CHUNK_MAX_BYTES as u64)
                    .min(total_size - start);
                let bytes = read_file_range(&resolved, start, length as usize)?;
                let result = AttachmentChunkResult {
                    path: resolved.to_string_lossy().to_string(),
                    bytes_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
                    total_size,
                    mime: media_mime_type(resolved.to_string_lossy().as_ref()).map(str::to_string),
                };
                self.state()?
                    .events
                    .push_back(HostEvent::AttachmentChunkRead {
                        timestamp: timestamp(),
                        result,
                    });
            }
            FeatureCommand::AttachmentReadImage { agent_id, path, .. } => {
                let resolved = resolve_agent_attachment_path(&agent_root, &agent_id, &path)?;
                let mime = media_mime_type(resolved.to_string_lossy().as_ref())
                    .filter(|mime| mime.starts_with("image/"))
                    .ok_or_else(|| {
                        FeatureHostError::Contract("attachment is not a supported image".into())
                    })?;
                let bytes = std::fs::read(&resolved).map_err(|error| {
                    FeatureHostError::Contract(format!("read image attachment: {error}"))
                })?;
                if bytes.len() as u64 > ATTACHMENT_BYTE_LIMIT {
                    return Err(FeatureHostError::Contract(
                        "image attachment exceeds preview limit".into(),
                    ));
                }
                let (width, height) = image_dimensions(&bytes, mime);
                let result = AttachmentImageResult {
                    path: resolved.to_string_lossy().to_string(),
                    data_url: format!(
                        "data:{mime};base64,{}",
                        base64::engine::general_purpose::STANDARD.encode(bytes)
                    ),
                    width,
                    height,
                };
                self.state()?
                    .events
                    .push_back(HostEvent::AttachmentImageRead {
                        timestamp: timestamp(),
                        result,
                    });
            }
            _ => unreachable!("non-attachment command routed to attachment executor"),
        }
        Ok(CommandAccepted {
            request_id,
            operation_id: None,
        })
    }

    fn execute_search(&self, command: FeatureCommand) -> Result<CommandAccepted, FeatureHostError> {
        let request_id = command.request_id().to_string();
        {
            let state = self.state()?;
            ensure_open(&state)?;
        }
        match command {
            FeatureCommand::SearchMessages { query, limit, .. } => {
                let query = query.trim().to_lowercase();
                let limit = limit.clamp(1, AGENT_CONTENT_SEARCH_MAX_RESULTS);
                let mut matches = Vec::new();
                if !query.is_empty() && self.config.mode == HostMode::Production {
                    #[cfg(feature = "production")]
                    {
                        let conversations =
                            match self.runtime()?.execute(RuntimeCommand::ListConversations)? {
                                RuntimeResponse::Conversations { data } => data,
                                other => {
                                    return Err(unexpected_response(
                                        "search.messages.conversations",
                                        other,
                                    ));
                                }
                            };
                        let bots = self.state()?.bots.values().cloned().collect::<Vec<_>>();
                        let bot_by_conversation = bots
                            .iter()
                            .filter_map(|bot| {
                                bot.conversation_id
                                    .as_ref()
                                    .map(|conversation_id| (conversation_id.clone(), bot))
                            })
                            .collect::<BTreeMap<_, _>>();
                        for conversation in conversations {
                            let response =
                                self.runtime()?
                                    .execute(RuntimeCommand::ConversationHistory {
                                        conversation_id: conversation.id.clone(),
                                        limit: Some(2_000),
                                    });
                            let messages = match response {
                                Ok(RuntimeResponse::History { data }) => data,
                                _ => continue,
                            };
                            let bot = bot_by_conversation.get(&conversation.id.0).copied();
                            let agent_id = bot
                                .map(|bot| bot.id.clone())
                                .unwrap_or_else(|| conversation.id.0.clone());
                            let agent_name = bot
                                .map(|bot| bot.name.clone())
                                .unwrap_or_else(|| conversation.title.clone());
                            let mut per_agent = 0usize;
                            for message in messages.into_iter().rev() {
                                if per_agent >= AGENT_CONTENT_SEARCH_MAX_MATCHES_PER_AGENT {
                                    break;
                                }
                                let Some(snippet) = build_content_snippet(&message.text, &query)
                                else {
                                    continue;
                                };
                                let role = match message.role {
                                    RuntimeMessageRole::User => MessageRole::User,
                                    RuntimeMessageRole::Assistant
                                    | RuntimeMessageRole::Contact
                                    | RuntimeMessageRole::MiniApp
                                    | RuntimeMessageRole::System => MessageRole::Assistant,
                                };
                                matches.push(SearchMessageMatch {
                                    agent_id: agent_id.clone(),
                                    agent_name: agent_name.clone(),
                                    conversation_id: conversation.id.0.clone(),
                                    entry_id: message.id.to_string(),
                                    role,
                                    timestamp_ms: message.created_at_ms,
                                    snippet,
                                });
                                per_agent += 1;
                            }
                        }
                        matches.sort_by_key(|item| std::cmp::Reverse(item.timestamp_ms));
                        matches.truncate(limit);
                    }
                }
                self.state()?
                    .events
                    .push_back(HostEvent::SearchMessagesListed {
                        timestamp: timestamp(),
                        query,
                        matches,
                    });
            }
            FeatureCommand::SearchMedia { query, limit, .. } => {
                let query = query.trim().to_lowercase();
                let limit = limit.clamp(1, AGENT_CONTENT_SEARCH_MAX_RESULTS);
                let bots = self.state()?.bots.values().cloned().collect::<Vec<_>>();
                let mut matches = Vec::new();
                if let Some(agent_root) = self.active_account_root(self.memory_root_path.as_deref())
                {
                    for bot in bots {
                        collect_agent_media_matches(
                            &agent_root.join(&bot.id).join("attachments"),
                            &bot.id,
                            &bot.name,
                            &query,
                            &mut matches,
                        );
                    }
                }
                matches.sort_by_key(|item| std::cmp::Reverse(item.timestamp_ms));
                matches.truncate(limit);
                self.state()?
                    .events
                    .push_back(HostEvent::SearchMediaListed {
                        timestamp: timestamp(),
                        query,
                        matches,
                    });
            }
            _ => unreachable!("non-search command routed to search executor"),
        }
        Ok(CommandAccepted {
            request_id,
            operation_id: None,
        })
    }

    fn execute_mcp(&self, command: FeatureCommand) -> Result<CommandAccepted, FeatureHostError> {
        let request_id = command.request_id().to_string();
        {
            let state = self.state()?;
            ensure_open(&state)?;
        }
        match command {
            FeatureCommand::McpList { .. } => {
                let servers = if self.config.mode == HostMode::Production {
                    #[cfg(feature = "production")]
                    {
                        match self.runtime()?.execute(RuntimeCommand::McpServers)? {
                            RuntimeResponse::McpServers { data } => data,
                            other => return Err(unexpected_response("mcp.list", other)),
                        }
                    }
                    #[cfg(not(feature = "production"))]
                    Vec::new()
                } else {
                    Vec::new()
                };
                self.state()?.events.push_back(HostEvent::McpListed {
                    timestamp: timestamp(),
                    servers,
                });
            }
            FeatureCommand::McpApps { .. } => {
                let apps = if self.config.mode == HostMode::Production {
                    #[cfg(feature = "production")]
                    {
                        match self.runtime()?.execute(RuntimeCommand::McpApps)? {
                            RuntimeResponse::McpApps { data } => data,
                            other => return Err(unexpected_response("mcp.apps", other)),
                        }
                    }
                    #[cfg(not(feature = "production"))]
                    Vec::new()
                } else {
                    Vec::new()
                };
                self.state()?.events.push_back(HostEvent::McpAppsListed {
                    timestamp: timestamp(),
                    apps,
                });
            }
            FeatureCommand::McpOauthLogin { server, .. } => {
                let server = required(server, "MCP server")?;
                if self.config.mode == HostMode::Production {
                    #[cfg(feature = "production")]
                    {
                        let (server, authorization_url, removed) = match self
                            .runtime()?
                            .execute(RuntimeCommand::McpOauthLogin { server })?
                        {
                            RuntimeResponse::McpOauth {
                                server,
                                authorization_url,
                                removed,
                            } => (server, authorization_url, removed),
                            other => return Err(unexpected_response("mcp.oauthLogin", other)),
                        };
                        self.state()?.events.push_back(HostEvent::McpOauthChanged {
                            timestamp: timestamp(),
                            server,
                            authorization_url,
                            removed,
                        });
                    }
                    #[cfg(not(feature = "production"))]
                    return Err(FeatureHostError::ProductionUnavailable);
                } else {
                    self.state()?.events.push_back(HostEvent::McpOauthChanged {
                        timestamp: timestamp(),
                        server,
                        authorization_url: Some("https://example.test/mcp-oauth".into()),
                        removed: false,
                    });
                }
            }
            FeatureCommand::McpOauthLogout { server, .. } => {
                let server = required(server, "MCP server")?;
                if self.config.mode == HostMode::Production {
                    #[cfg(feature = "production")]
                    {
                        let (server, authorization_url, removed) = match self
                            .runtime()?
                            .execute(RuntimeCommand::McpOauthLogout { server })?
                        {
                            RuntimeResponse::McpOauth {
                                server,
                                authorization_url,
                                removed,
                            } => (server, authorization_url, removed),
                            other => return Err(unexpected_response("mcp.oauthLogout", other)),
                        };
                        self.state()?.events.push_back(HostEvent::McpOauthChanged {
                            timestamp: timestamp(),
                            server,
                            authorization_url,
                            removed,
                        });
                    }
                    #[cfg(not(feature = "production"))]
                    return Err(FeatureHostError::ProductionUnavailable);
                } else {
                    self.state()?.events.push_back(HostEvent::McpOauthChanged {
                        timestamp: timestamp(),
                        server,
                        authorization_url: None,
                        removed: true,
                    });
                }
            }
            FeatureCommand::McpRemove { server, .. } => {
                let server = required(server, "MCP server")?;
                if self.config.mode == HostMode::Production {
                    #[cfg(feature = "production")]
                    {
                        let removed = match self.runtime()?.execute(RuntimeCommand::McpRemove {
                            server: server.clone(),
                        })? {
                            RuntimeResponse::McpRemoved { removed, .. } => removed,
                            other => return Err(unexpected_response("mcp.remove", other)),
                        };
                        if !removed {
                            return Err(FeatureHostError::Contract(format!(
                                "MCP server is not a user-managed configuration: {server}"
                            )));
                        }
                    }
                    #[cfg(not(feature = "production"))]
                    return Err(FeatureHostError::ProductionUnavailable);
                }
                self.state()?.events.push_back(HostEvent::McpRefreshed {
                    timestamp: timestamp(),
                });
            }
            FeatureCommand::McpSetCustomInstructions {
                server,
                instructions,
                ..
            } => {
                let server = required(server, "MCP server")?;
                let instructions = clamp_block(&instructions, 20_000);
                if self.config.mode == HostMode::Production {
                    #[cfg(feature = "production")]
                    match self
                        .runtime()?
                        .execute(RuntimeCommand::McpSetCustomInstructions {
                            server: server.clone(),
                            instructions,
                        })? {
                        RuntimeResponse::McpCustomInstructionsUpdated { .. } => {}
                        other => {
                            return Err(unexpected_response("mcp.setCustomInstructions", other));
                        }
                    }
                    #[cfg(not(feature = "production"))]
                    return Err(FeatureHostError::ProductionUnavailable);
                }
                self.state()?.events.push_back(HostEvent::McpRefreshed {
                    timestamp: timestamp(),
                });
            }
            FeatureCommand::McpSetToolDisabled {
                server,
                tool,
                disabled,
                ..
            } => {
                let server = required(server, "MCP server")?;
                let tool = required(tool, "MCP tool")?;
                if self.config.mode == HostMode::Production {
                    #[cfg(feature = "production")]
                    match self
                        .runtime()?
                        .execute(RuntimeCommand::McpSetToolDisabled {
                            server: server.clone(),
                            tool,
                            disabled,
                        })? {
                        RuntimeResponse::McpToolDisabledUpdated { .. } => {}
                        other => return Err(unexpected_response("mcp.setToolDisabled", other)),
                    }
                    #[cfg(not(feature = "production"))]
                    return Err(FeatureHostError::ProductionUnavailable);
                }
                self.state()?.events.push_back(HostEvent::McpRefreshed {
                    timestamp: timestamp(),
                });
            }
            FeatureCommand::McpRefresh { .. } => {
                if self.config.mode == HostMode::Production {
                    #[cfg(feature = "production")]
                    match self.runtime()?.execute(RuntimeCommand::McpRefresh)? {
                        RuntimeResponse::McpRefreshed => {}
                        other => return Err(unexpected_response("mcp.refresh", other)),
                    }
                    #[cfg(not(feature = "production"))]
                    return Err(FeatureHostError::ProductionUnavailable);
                }
                self.state()?.events.push_back(HostEvent::McpRefreshed {
                    timestamp: timestamp(),
                });
            }
            FeatureCommand::McpToolCall {
                server,
                tool,
                arguments,
                ..
            } => {
                let server = required(server, "MCP server")?;
                let tool = required(tool, "MCP tool")?;
                let result = if self.config.mode == HostMode::Production {
                    #[cfg(feature = "production")]
                    {
                        match self.runtime()?.execute(RuntimeCommand::McpToolCall {
                            server: server.clone(),
                            tool: tool.clone(),
                            arguments,
                        })? {
                            RuntimeResponse::McpToolResult { result, .. } => result,
                            other => return Err(unexpected_response("mcp.toolCall", other)),
                        }
                    }
                    #[cfg(not(feature = "production"))]
                    Value::Null
                } else {
                    json!({"ok": true, "mock": true})
                };
                let _ = self.append_action_audit(
                    "mahayana-assistant",
                    None,
                    json!({
                        "kind": "mcpToolCall",
                        "serverIdentifier": server,
                        "toolName": tool,
                        "transport": "runtime",
                        "status": "success",
                    }),
                );
                self.state()?.events.push_back(HostEvent::McpToolResult {
                    timestamp: timestamp(),
                    request_id: request_id.clone(),
                    server,
                    tool,
                    result,
                });
            }
            _ => unreachable!("non-MCP command routed to MCP executor"),
        }
        Ok(CommandAccepted {
            request_id,
            operation_id: None,
        })
    }

    fn execute_settings_and_audit(
        &self,
        command: FeatureCommand,
    ) -> Result<CommandAccepted, FeatureHostError> {
        let request_id = command.request_id().to_string();
        match command {
            FeatureCommand::SettingsGet { .. } => {
                let settings = self.state()?.settings.clone();
                self.state()?.events.push_back(HostEvent::SettingsChanged {
                    timestamp: timestamp(),
                    settings,
                });
            }
            FeatureCommand::SettingsUpdate { mut settings, .. } => {
                settings.auto_review_rules = sanitize_auto_review_rules(settings.auto_review_rules);
                {
                    let mut state = self.state()?;
                    ensure_open(&state)?;
                    state.settings = settings.clone();
                }
                if let Some(path) = self.settings_path.as_deref() {
                    persist_product_host_settings(path, &settings)?;
                }
                sync_computer_control_policy(&settings);
                if !settings.remote_control_enabled {
                    self.state()?.remote_computer_sessions.clear();
                }
                self.state()?.events.push_back(HostEvent::SettingsChanged {
                    timestamp: timestamp(),
                    settings,
                });
            }
            FeatureCommand::AuditList {
                agent_id, limit, ..
            } => {
                let requested_agent_id = agent_id;
                let agent_id = {
                    let state = self.state()?;
                    canonical_runtime_agent_id(&state, &requested_agent_id).ok_or_else(|| {
                        FeatureHostError::Contract(format!(
                            "unknown audit agent: {requested_agent_id}"
                        ))
                    })?
                };
                if !is_safe_memory_agent_id(&agent_id) {
                    return Err(FeatureHostError::Contract(format!(
                        "unsafe audit agent: {agent_id}"
                    )));
                }
                let records = self
                    .active_account_root(self.memory_root_path.as_deref())
                    .map(|root| {
                        read_action_audit(
                            &root.join(&agent_id).join("audit.jsonl"),
                            limit.min(1000),
                        )
                    })
                    .unwrap_or_default();
                self.state()?.events.push_back(HostEvent::AuditListed {
                    timestamp: timestamp(),
                    agent_id,
                    records,
                });
            }
            _ => unreachable!("non-settings command routed to settings executor"),
        }
        Ok(CommandAccepted {
            request_id,
            operation_id: None,
        })
    }

    fn append_action_audit(
        &self,
        agent_id: &str,
        turn_id: Option<&str>,
        action: Value,
    ) -> Result<(), FeatureHostError> {
        if !is_safe_memory_agent_id(agent_id) {
            return Ok(());
        }
        let Some(root) = self.active_account_root(self.memory_root_path.as_deref()) else {
            return Ok(());
        };
        let path = root.join(agent_id).join("audit.jsonl");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| {
                FeatureHostError::Contract(format!("create action audit directory: {error}"))
            })?;
        }
        let record = json!({
            "ts": timestamp(),
            "agentId": agent_id,
            "eventId": format!("audit-{}-{}", now_millis(), std::process::id()),
            "turnId": turn_id.unwrap_or(""),
            "action": action,
        });
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|error| FeatureHostError::Contract(format!("open action audit: {error}")))?;
        writeln!(file, "{record}")
            .map_err(|error| FeatureHostError::Contract(format!("append action audit: {error}")))
    }

    fn execute_product_surface(
        &self,
        command: FeatureCommand,
    ) -> Result<CommandAccepted, FeatureHostError> {
        match self.config.mode {
            HostMode::Test => self.execute_product_surface_test(command),
            HostMode::Production => {
                #[cfg(feature = "production")]
                return self.execute_product_surface_production(command);
                #[cfg(not(feature = "production"))]
                return Err(FeatureHostError::ProductionUnavailable);
            }
        }
    }

    fn execute_product_surface_test(
        &self,
        command: FeatureCommand,
    ) -> Result<CommandAccepted, FeatureHostError> {
        let request_id = command.request_id().to_string();
        let mut state = self.state()?;
        ensure_open(&state)?;
        match command {
            FeatureCommand::ConnectorList { .. } => {
                let connectors = state.connectors.values().cloned().collect();
                state.events.push_back(HostEvent::ConnectorListed {
                    timestamp: timestamp(),
                    connectors,
                });
            }
            FeatureCommand::ConnectorConnect {
                connector_id,
                account_label,
                ..
            } => {
                let connector_id = required(connector_id, "connectorId")?;
                let account_id = next_id(&mut state, "account");
                let connector = state.connectors.get_mut(&connector_id).ok_or_else(|| {
                    FeatureHostError::Contract(format!("unknown connector: {connector_id}"))
                })?;
                connector.accounts.push(ConnectorAccountSummary {
                    id: account_id,
                    label: account_label
                        .clone()
                        .filter(|label| !label.trim().is_empty())
                        .unwrap_or_else(|| "Personal".into()),
                    status: ConnectorStatus::Connected,
                    email: None,
                    team_managed: Some(false),
                    error: None,
                });
                connector.status = ConnectorStatus::Connected;
                let connector = connector.clone();
                state.events.push_back(HostEvent::ConnectorChanged {
                    timestamp: timestamp(),
                    action: "connected".into(),
                    connector,
                });
                if let Some(platform) = listener_platform_for_connector(&connector_id) {
                    if let Some(integration) = state.listeners.get_mut(&platform) {
                        integration.is_connected = true;
                        integration.account_label = account_label;
                        let integration = integration.clone();
                        state.events.push_back(HostEvent::ListenerChanged {
                            timestamp: timestamp(),
                            integration,
                        });
                    }
                }
            }
            FeatureCommand::ConnectorRenameAccount {
                connector_id,
                account_id,
                label,
                ..
            } => {
                let label = required(label, "account label")?;
                let connector = state.connectors.get_mut(&connector_id).ok_or_else(|| {
                    FeatureHostError::Contract(format!("unknown connector: {connector_id}"))
                })?;
                let account = connector
                    .accounts
                    .iter_mut()
                    .find(|account| account.id == account_id)
                    .ok_or_else(|| {
                        FeatureHostError::Contract(format!(
                            "unknown connector account: {account_id}"
                        ))
                    })?;
                if account.team_managed == Some(true) {
                    return Err(FeatureHostError::Contract(
                        "team-managed accounts cannot be renamed".into(),
                    ));
                }
                account.label = label;
                let connector = connector.clone();
                state.events.push_back(HostEvent::ConnectorChanged {
                    timestamp: timestamp(),
                    action: "updated".into(),
                    connector,
                });
            }
            FeatureCommand::ConnectorRemoveAccount {
                connector_id,
                account_id,
                ..
            } => {
                let connector = state.connectors.get_mut(&connector_id).ok_or_else(|| {
                    FeatureHostError::Contract(format!("unknown connector: {connector_id}"))
                })?;
                if connector
                    .accounts
                    .iter()
                    .any(|account| account.id == account_id && account.team_managed == Some(true))
                {
                    return Err(FeatureHostError::Contract(
                        "team-managed accounts cannot be removed".into(),
                    ));
                }
                let before = connector.accounts.len();
                connector
                    .accounts
                    .retain(|account| account.id != account_id);
                if before == connector.accounts.len() {
                    return Err(FeatureHostError::Contract(format!(
                        "unknown connector account: {account_id}"
                    )));
                }
                if connector.accounts.is_empty() {
                    connector.status = ConnectorStatus::Disconnected;
                }
                let connector = connector.clone();
                state.events.push_back(HostEvent::ConnectorChanged {
                    timestamp: timestamp(),
                    action: "removed".into(),
                    connector,
                });
            }
            FeatureCommand::ConnectorSetToolEnabled {
                connector_id,
                tool_id,
                enabled,
                ..
            } => {
                let connector = state.connectors.get_mut(&connector_id).ok_or_else(|| {
                    FeatureHostError::Contract(format!("unknown connector: {connector_id}"))
                })?;
                let tool = connector
                    .tools
                    .iter_mut()
                    .find(|tool| tool.id == tool_id)
                    .ok_or_else(|| {
                        FeatureHostError::Contract(format!("unknown connector tool: {tool_id}"))
                    })?;
                tool.enabled = enabled;
                let connector = connector.clone();
                state.events.push_back(HostEvent::ConnectorChanged {
                    timestamp: timestamp(),
                    action: "toolChanged".into(),
                    connector,
                });
            }
            FeatureCommand::SkillList { agent_id, .. } => {
                let skills = state
                    .skills
                    .values()
                    .filter(|skill| {
                        agent_id
                            .as_ref()
                            .is_none_or(|agent_id| skill.owner_agent_id.as_ref() == Some(agent_id))
                    })
                    .cloned()
                    .collect();
                state.events.push_back(HostEvent::SkillListed {
                    timestamp: timestamp(),
                    skills,
                    teams: default_skill_teams(),
                });
            }
            FeatureCommand::SkillUpsert {
                id,
                name,
                description,
                use_when,
                instructions,
                owner_agent_id,
                ..
            } => {
                let name = required(name, "skill name")?;
                let use_when = required(use_when, "skill useWhen")?;
                let instructions = required(instructions, "skill instructions")?;
                let id = id.unwrap_or_else(|| next_id(&mut state, "skill"));
                let action = if state.skills.contains_key(&id) {
                    "updated"
                } else {
                    "created"
                };
                let previous = state.skills.get(&id);
                if previous.is_some_and(|skill| skill.read_only == Some(true)) {
                    return Err(FeatureHostError::Contract(
                        "managed skills cannot be edited".into(),
                    ));
                }
                let skill = SkillSummary {
                    id: id.clone(),
                    name,
                    description,
                    use_when,
                    instructions,
                    source: previous.map_or(SkillSource::Private, |skill| skill.source),
                    publish_state: previous
                        .map_or(SkillPublishState::Local, |skill| skill.publish_state),
                    owner_agent_id,
                    team_id: previous.and_then(|skill| skill.team_id.clone()),
                    team_name: previous.and_then(|skill| skill.team_name.clone()),
                    read_only: previous.and_then(|skill| skill.read_only),
                    updated_at_ms: now_millis(),
                };
                state.skills.insert(id, skill.clone());
                state.events.push_back(HostEvent::SkillChanged {
                    timestamp: timestamp(),
                    action: action.into(),
                    skill,
                });
            }
            FeatureCommand::SkillDelete { id, .. } => {
                let skill = state
                    .skills
                    .get(&id)
                    .ok_or_else(|| FeatureHostError::Contract(format!("unknown skill: {id}")))?;
                if skill.read_only == Some(true) || skill.source == SkillSource::Team {
                    return Err(FeatureHostError::Contract(
                        "team-managed skills cannot be deleted".into(),
                    ));
                }
                let skill = state.skills.remove(&id).expect("skill checked above");
                state.events.push_back(HostEvent::SkillChanged {
                    timestamp: timestamp(),
                    action: "deleted".into(),
                    skill,
                });
            }
            FeatureCommand::SkillPublish { id, team_id, .. } => {
                let team = default_skill_teams()
                    .into_iter()
                    .find(|team| team.id == team_id)
                    .ok_or_else(|| {
                        FeatureHostError::Contract(format!("unknown skill team: {team_id}"))
                    })?;
                let skill = state
                    .skills
                    .get_mut(&id)
                    .ok_or_else(|| FeatureHostError::Contract(format!("unknown skill: {id}")))?;
                if skill.description.trim().is_empty() {
                    return Err(FeatureHostError::Contract(
                        "add a skill description before publishing".into(),
                    ));
                }
                skill.source = SkillSource::Team;
                skill.publish_state = SkillPublishState::Published;
                skill.team_id = Some(team.id);
                skill.team_name = Some(team.name);
                skill.updated_at_ms = now_millis();
                let skill = skill.clone();
                state.events.push_back(HostEvent::SkillChanged {
                    timestamp: timestamp(),
                    action: "published".into(),
                    skill,
                });
            }
            FeatureCommand::SkillUnpublish { id, .. } => {
                let skill = state
                    .skills
                    .get_mut(&id)
                    .ok_or_else(|| FeatureHostError::Contract(format!("unknown skill: {id}")))?;
                if skill.publish_state == SkillPublishState::Managed {
                    return Err(FeatureHostError::Contract(
                        "managed skills cannot be unpublished".into(),
                    ));
                }
                skill.source = SkillSource::Private;
                skill.publish_state = SkillPublishState::Local;
                skill.team_id = None;
                skill.team_name = None;
                skill.updated_at_ms = now_millis();
                let skill = skill.clone();
                state.events.push_back(HostEvent::SkillChanged {
                    timestamp: timestamp(),
                    action: "unpublished".into(),
                    skill,
                });
            }
            FeatureCommand::SkillSync { id, .. } => {
                let skill = state
                    .skills
                    .get_mut(&id)
                    .ok_or_else(|| FeatureHostError::Contract(format!("unknown skill: {id}")))?;
                if skill.team_id.is_none() {
                    return Err(FeatureHostError::Contract(
                        "only published skills can be synced".into(),
                    ));
                }
                skill.publish_state = SkillPublishState::Synced;
                skill.updated_at_ms = now_millis();
                let skill = skill.clone();
                state.events.push_back(HostEvent::SkillChanged {
                    timestamp: timestamp(),
                    action: "synced".into(),
                    skill,
                });
            }
            FeatureCommand::BotList { .. } => {
                let bots = state.bots.values().cloned().collect();
                state.events.push_back(HostEvent::BotListed {
                    timestamp: timestamp(),
                    bots,
                });
            }
            FeatureCommand::BotSetHidden { id, hidden, .. } => {
                let bot = state
                    .bots
                    .get_mut(&id)
                    .ok_or_else(|| FeatureHostError::Contract(format!("unknown bot: {id}")))?;
                bot.hidden = hidden;
                let bot = bot.clone();
                state.events.push_back(HostEvent::BotChanged {
                    timestamp: timestamp(),
                    action: "updated".into(),
                    bot,
                });
            }
            FeatureCommand::DraftResolve { draft, action, .. } => {
                let draft_id = draft.id().to_string();
                match action {
                    DraftAction::Discard => {
                        state.events.push_back(HostEvent::DraftChanged {
                            timestamp: timestamp(),
                            draft_id,
                            status: DraftSendState::Discarded,
                            error: None,
                        });
                    }
                    DraftAction::Send => {
                        validate_draft(&draft)?;
                        state.events.push_back(HostEvent::DraftChanged {
                            timestamp: timestamp(),
                            draft_id: draft_id.clone(),
                            status: DraftSendState::Sending,
                            error: None,
                        });
                        state.events.push_back(HostEvent::DraftChanged {
                            timestamp: timestamp(),
                            draft_id,
                            status: DraftSendState::Sent,
                            error: None,
                        });
                    }
                }
            }
            FeatureCommand::SecretProvide {
                secret_request_id,
                value,
                ..
            } => {
                if value.is_empty() {
                    return Err(FeatureHostError::Contract(
                        "secret value must not be empty".into(),
                    ));
                }
                state.events.push_back(HostEvent::SecretProvided {
                    timestamp: timestamp(),
                    secret_request_id,
                });
            }
            FeatureCommand::ListenerList { .. } => {
                let integrations = state.listeners.values().cloned().collect();
                state.events.push_back(HostEvent::ListenerListed {
                    timestamp: timestamp(),
                    integrations,
                });
            }
            FeatureCommand::ListenerConnect { platform, .. } => {
                let integration = state.listeners.get_mut(&platform).ok_or_else(|| {
                    FeatureHostError::Contract(format!(
                        "unsupported listener platform: {platform:?}"
                    ))
                })?;
                integration.is_connected = true;
                integration.error = None;
                let integration = integration.clone();
                state.events.push_back(HostEvent::ListenerChanged {
                    timestamp: timestamp(),
                    integration,
                });
                if let Some(connector_id) = connector_for_listener_platform(platform) {
                    if let Some(connector) = state.connectors.get_mut(connector_id) {
                        connector.status = ConnectorStatus::Connected;
                        let connector = connector.clone();
                        state.events.push_back(HostEvent::ConnectorChanged {
                            timestamp: timestamp(),
                            action: "connected".into(),
                            connector,
                        });
                    }
                }
            }
            FeatureCommand::ListenerDisconnect { platform, .. } => {
                let integration = state.listeners.get_mut(&platform).ok_or_else(|| {
                    FeatureHostError::Contract(format!(
                        "unsupported listener platform: {platform:?}"
                    ))
                })?;
                integration.is_connected = false;
                integration.account_label = None;
                integration.error = None;
                let integration = integration.clone();
                state.events.push_back(HostEvent::ListenerChanged {
                    timestamp: timestamp(),
                    integration,
                });
                if let Some(connector_id) = connector_for_listener_platform(platform) {
                    if let Some(connector) = state.connectors.get_mut(connector_id) {
                        connector.status = ConnectorStatus::Disconnected;
                        connector.accounts.clear();
                        let connector = connector.clone();
                        state.events.push_back(HostEvent::ConnectorChanged {
                            timestamp: timestamp(),
                            action: "disconnected".into(),
                            connector,
                        });
                    }
                }
            }
            FeatureCommand::UpdateStatus { .. } => {
                let update_state = state.update_state.clone();
                state.events.push_back(HostEvent::UpdateChanged {
                    timestamp: timestamp(),
                    state: update_state,
                });
            }
            FeatureCommand::UpdateCheck { .. } => {
                state.update_state = UpdateState::Checking;
                let checking = state.update_state.clone();
                state.events.push_back(HostEvent::UpdateChanged {
                    timestamp: timestamp(),
                    state: checking,
                });
                state.update_state = UpdateState::UpToDate {
                    version: env!("CARGO_PKG_VERSION").into(),
                };
                let update_state = state.update_state.clone();
                state.events.push_back(HostEvent::UpdateChanged {
                    timestamp: timestamp(),
                    state: update_state,
                });
            }
            FeatureCommand::UpdateInstall { .. } => {
                let version = match &state.update_state {
                    UpdateState::Available { version, .. }
                    | UpdateState::Ready { version }
                    | UpdateState::Downloading { version, .. }
                    | UpdateState::Staging { version } => version.clone(),
                    _ => {
                        return Err(FeatureHostError::Contract(
                            "no update is available to install".into(),
                        ));
                    }
                };
                state.update_state = UpdateState::Downloading {
                    version: version.clone(),
                    progress: Some(100),
                };
                let downloading = state.update_state.clone();
                state.events.push_back(HostEvent::UpdateChanged {
                    timestamp: timestamp(),
                    state: downloading,
                });
                state.update_state = UpdateState::Ready { version };
                let ready = state.update_state.clone();
                state.events.push_back(HostEvent::UpdateChanged {
                    timestamp: timestamp(),
                    state: ready,
                });
            }
            _ => unreachable!("non-product-surface command routed to product executor"),
        }
        Ok(CommandAccepted {
            request_id,
            operation_id: None,
        })
    }

    #[cfg(feature = "production")]
    fn production_live_connector_sources(
        &self,
    ) -> Result<(Vec<Value>, Vec<Value>), FeatureHostError> {
        let servers = match self.runtime()?.execute(RuntimeCommand::McpServers)? {
            RuntimeResponse::McpServers { data } => data,
            other => return Err(unexpected_response("mahayana.mcp.servers", other)),
        };
        let apps = match self.runtime()?.execute(RuntimeCommand::McpApps) {
            Ok(RuntimeResponse::McpApps { data }) => data,
            Ok(other) => return Err(unexpected_response("mahayana.mcp.apps", other)),
            // Connector directory discovery is feature-gated in some Codex
            // builds. Live MCP tool metadata is still authoritative for linked
            // connectors, so an unavailable directory should not hide them.
            Err(_) => Vec::new(),
        };
        Ok((servers, apps))
    }

    #[cfg(feature = "production")]
    fn production_connector_snapshot(
        &self,
    ) -> Result<
        (
            Vec<ConnectorSummary>,
            BTreeMap<String, LiveConnectorProjection>,
        ),
        FeatureHostError,
    > {
        let payload = json!({"type": "connector.list", "requestId": "connector-snapshot"});
        let response = self
            .runtime()?
            .product_execute("mahayana.connector.list", &payload)?;
        let base: Vec<ConnectorSummary> =
            decode_product_field(response, "connectors", "mahayana.connector.list")?;
        let (servers, apps) = self.production_live_connector_sources()?;
        let live = live_connector_projections(&servers, &apps);
        Ok((merge_live_connectors(base, &live), live))
    }

    #[cfg(feature = "production")]
    fn emit_connector_snapshot_change(
        &self,
        connector_id: &str,
        action: &str,
    ) -> Result<(), FeatureHostError> {
        let (connectors, _) = self.production_connector_snapshot()?;
        let connector = connectors
            .into_iter()
            .find(|connector| connector.id == connector_id)
            .ok_or_else(|| {
                FeatureHostError::Contract(format!("unknown connector: {connector_id}"))
            })?;
        self.state()?.events.push_back(HostEvent::ConnectorChanged {
            timestamp: timestamp(),
            action: action.into(),
            connector,
        });
        Ok(())
    }

    #[cfg(feature = "production")]
    fn execute_live_product_surface_production(
        &self,
        command: &FeatureCommand,
    ) -> Result<Option<CommandAccepted>, FeatureHostError> {
        let request_id = command.request_id().to_string();
        match command {
            FeatureCommand::ConnectorList { .. } => {
                let (connectors, _) = self.production_connector_snapshot()?;
                self.state()?.events.push_back(HostEvent::ConnectorListed {
                    timestamp: timestamp(),
                    connectors,
                });
                Ok(Some(CommandAccepted {
                    request_id,
                    operation_id: None,
                }))
            }
            FeatureCommand::ConnectorConnect {
                connector_id,
                account_label: _,
                ..
            } => {
                if connector_id == "git" {
                    let payload = serde_json::to_value(command).map_err(|error| {
                        FeatureHostError::Contract(format!("encode connector.connect: {error}"))
                    })?;
                    self.runtime()?
                        .product_execute("mahayana.connector.connect", &payload)?;
                    self.emit_connector_snapshot_change(connector_id, "connected")?;
                    return Ok(Some(CommandAccepted {
                        request_id,
                        operation_id: None,
                    }));
                }
                let (_, live) = self.production_connector_snapshot()?;
                let projection = live.get(connector_id).ok_or_else(|| {
                    FeatureHostError::Contract(format!(
                        "{connector_id} is not installed or discoverable in the Codex connector runtime; add its plugin from Plugins first"
                    ))
                })?;
                if let Some(url) = projection.install_url.clone() {
                    if !url.starts_with("https://") {
                        return Err(FeatureHostError::Contract(
                            "connector authorization URL must use HTTPS".into(),
                        ));
                    }
                    self.state()?
                        .events
                        .push_back(HostEvent::ConnectorOauthRequested {
                            timestamp: timestamp(),
                            connector_id: connector_id.clone(),
                            authorization_url: url,
                        });
                    return Ok(Some(CommandAccepted {
                        request_id,
                        operation_id: None,
                    }));
                }
                let server = projection.server_name.clone().ok_or_else(|| {
                    FeatureHostError::Contract(format!(
                        "{connector_id} does not expose an OAuth-capable MCP server"
                    ))
                })?;
                if projection.status == Some(ConnectorStatus::Connected) {
                    self.emit_connector_snapshot_change(connector_id, "connected")?;
                    return Ok(Some(CommandAccepted {
                        request_id,
                        operation_id: None,
                    }));
                }
                let authorization_url =
                    match self.runtime()?.execute(RuntimeCommand::McpOauthLogin {
                        server: server.clone(),
                    })? {
                        RuntimeResponse::McpOauth {
                            authorization_url: Some(url),
                            ..
                        } => url,
                        other => {
                            return Err(unexpected_response("mahayana.mcp.oauth.login", other));
                        }
                    };
                self.state()?
                    .events
                    .push_back(HostEvent::ConnectorOauthRequested {
                        timestamp: timestamp(),
                        connector_id: connector_id.clone(),
                        authorization_url,
                    });
                Ok(Some(CommandAccepted {
                    request_id,
                    operation_id: None,
                }))
            }
            FeatureCommand::ConnectorRenameAccount { connector_id, .. }
            | FeatureCommand::ConnectorSetToolEnabled { connector_id, .. } => {
                let method = product_surface_method(command);
                let payload = serde_json::to_value(command).map_err(|error| {
                    FeatureHostError::Contract(format!("encode {method}: {error}"))
                })?;
                self.runtime()?.product_execute(method, &payload)?;
                self.emit_connector_snapshot_change(
                    connector_id,
                    if matches!(command, FeatureCommand::ConnectorSetToolEnabled { .. }) {
                        "toolChanged"
                    } else {
                        "updated"
                    },
                )?;
                Ok(Some(CommandAccepted {
                    request_id,
                    operation_id: None,
                }))
            }
            FeatureCommand::ConnectorRemoveAccount {
                connector_id,
                account_id,
                ..
            } => {
                let (_, live) = self.production_connector_snapshot()?;
                if let Some(projection) = live.get(connector_id) {
                    if projection.server_name.as_deref() == Some("codex_apps") {
                        if let Some(url) = projection.install_url.clone() {
                            self.state()?
                                .events
                                .push_back(HostEvent::ConnectorOauthRequested {
                                    timestamp: timestamp(),
                                    connector_id: connector_id.clone(),
                                    authorization_url: url,
                                });
                        }
                        return Err(FeatureHostError::Contract(
                            "this linked ChatGPT App account is server-managed; its account page was opened because the current Codex connector API does not expose unlink"
                                .into(),
                        ));
                    }
                    if let Some(server) = projection.server_name.clone() {
                        match self
                            .runtime()?
                            .execute(RuntimeCommand::McpOauthLogout { server })?
                        {
                            RuntimeResponse::McpOauth { .. } => {}
                            other => {
                                return Err(unexpected_response(
                                    "mahayana.mcp.oauth.logout",
                                    other,
                                ));
                            }
                        }
                    }
                }
                let method = product_surface_method(command);
                let payload = serde_json::to_value(command).map_err(|error| {
                    FeatureHostError::Contract(format!("encode {method}: {error}"))
                })?;
                self.runtime()?.product_execute(method, &payload)?;
                let _ = account_id;
                self.emit_connector_snapshot_change(connector_id, "removed")?;
                Ok(Some(CommandAccepted {
                    request_id,
                    operation_id: None,
                }))
            }
            FeatureCommand::DraftResolve { draft, action, .. } => {
                let draft_id = draft.id().to_string();
                if *action == DraftAction::Discard {
                    self.state()?.events.push_back(HostEvent::DraftChanged {
                        timestamp: timestamp(),
                        draft_id,
                        status: DraftSendState::Discarded,
                        error: None,
                    });
                    return Ok(Some(CommandAccepted {
                        request_id,
                        operation_id: None,
                    }));
                }
                validate_draft(draft)?;
                let connector_id = match draft {
                    MessageDraft::Email { .. } => "gmail",
                    MessageDraft::Slack { .. } => "slack",
                };
                let (_, live) = self.production_connector_snapshot()?;
                let projection = live.get(connector_id).ok_or_else(|| {
                    FeatureHostError::Contract(format!(
                        "{connector_id} connector is not installed; install and authorize it before sending"
                    ))
                })?;
                if projection.status != Some(ConnectorStatus::Connected) {
                    return Err(FeatureHostError::Contract(format!(
                        "{connector_id} connector is not authorized"
                    )));
                }
                let server = projection.server_name.clone().ok_or_else(|| {
                    FeatureHostError::Contract(format!(
                        "{connector_id} connector does not expose an MCP server"
                    ))
                })?;
                let (tool, schema) =
                    projection_send_tool(projection, connector_id).ok_or_else(|| {
                        FeatureHostError::Contract(format!(
                            "{connector_id} connector has no compatible send tool"
                        ))
                    })?;
                let arguments = draft_tool_arguments(draft, schema)?;
                self.state()?.events.push_back(HostEvent::DraftChanged {
                    timestamp: timestamp(),
                    draft_id: draft_id.clone(),
                    status: DraftSendState::Sending,
                    error: None,
                });
                let result = self.runtime()?.execute(RuntimeCommand::McpToolCall {
                    server,
                    tool: tool.to_string(),
                    arguments,
                });
                match result {
                    Ok(RuntimeResponse::McpToolResult { .. }) => {
                        self.state()?.events.push_back(HostEvent::DraftChanged {
                            timestamp: timestamp(),
                            draft_id,
                            status: DraftSendState::Sent,
                            error: None,
                        });
                        Ok(Some(CommandAccepted {
                            request_id,
                            operation_id: None,
                        }))
                    }
                    Ok(other) => Err(unexpected_response("mahayana.mcp.tool.call", other)),
                    Err(error) => {
                        let message = error.to_string();
                        self.state()?.events.push_back(HostEvent::DraftChanged {
                            timestamp: timestamp(),
                            draft_id,
                            status: DraftSendState::Failed,
                            error: Some(message.clone()),
                        });
                        Err(error.into())
                    }
                }
            }
            FeatureCommand::ListenerList { .. } => {
                let payload = serde_json::to_value(command).map_err(|error| {
                    FeatureHostError::Contract(format!("encode listener.list: {error}"))
                })?;
                let response = self
                    .runtime()?
                    .product_execute("mahayana.listener.list", &payload)?;
                let mut integrations: Vec<ListenerIntegrationSummary> =
                    decode_product_field(response, "integrations", "mahayana.listener.list")?;
                let (connectors, _) = self.production_connector_snapshot()?;
                for integration in &mut integrations {
                    if integration.platform == ListenerPlatform::Git {
                        continue;
                    }
                    if let Some(connector_id) =
                        connector_for_listener_platform(integration.platform)
                    {
                        if let Some(connector) = connectors
                            .iter()
                            .find(|connector| connector.id == connector_id)
                        {
                            integration.is_connected =
                                connector.status == ConnectorStatus::Connected;
                            integration.account_label = connector
                                .accounts
                                .first()
                                .map(|account| account.label.clone());
                        }
                    }
                }
                self.state()?.events.push_back(HostEvent::ListenerListed {
                    timestamp: timestamp(),
                    integrations,
                });
                Ok(Some(CommandAccepted {
                    request_id,
                    operation_id: None,
                }))
            }
            FeatureCommand::ListenerConnect { platform, .. } => {
                if *platform == ListenerPlatform::Git {
                    let payload = serde_json::to_value(command).map_err(|error| {
                        FeatureHostError::Contract(format!("encode listener.connect: {error}"))
                    })?;
                    let response = self
                        .runtime()?
                        .product_execute("mahayana.listener.connect", &payload)?;
                    let integration =
                        decode_product_field(response, "integration", "mahayana.listener.connect")?;
                    self.state()?.events.push_back(HostEvent::ListenerChanged {
                        timestamp: timestamp(),
                        integration,
                    });
                    return Ok(Some(CommandAccepted {
                        request_id,
                        operation_id: None,
                    }));
                }
                let connector_id = connector_for_listener_platform(*platform).ok_or_else(|| {
                    FeatureHostError::Contract(format!(
                        "no connector exists for listener platform {platform:?}"
                    ))
                })?;
                let synthetic = FeatureCommand::ConnectorConnect {
                    request_id: request_id.clone(),
                    connector_id: connector_id.into(),
                    account_label: None,
                };
                self.execute_live_product_surface_production(&synthetic)?;
                Ok(Some(CommandAccepted {
                    request_id,
                    operation_id: None,
                }))
            }
            FeatureCommand::ListenerDisconnect { platform, .. } => {
                if *platform != ListenerPlatform::Git {
                    let connector_id =
                        connector_for_listener_platform(*platform).ok_or_else(|| {
                            FeatureHostError::Contract(format!(
                                "no connector exists for listener platform {platform:?}"
                            ))
                        })?;
                    let (connectors, live) = self.production_connector_snapshot()?;
                    let accounts = connectors
                        .iter()
                        .find(|connector| connector.id == connector_id)
                        .map(|connector| connector.accounts.clone())
                        .unwrap_or_default();
                    if accounts.is_empty() {
                        if let Some(projection) = live.get(connector_id) {
                            if projection.status == Some(ConnectorStatus::Connected) {
                                if let Some(server) = projection.server_name.clone() {
                                    match self
                                        .runtime()?
                                        .execute(RuntimeCommand::McpOauthLogout { server })?
                                    {
                                        RuntimeResponse::McpOauth { .. } => {}
                                        other => {
                                            return Err(unexpected_response(
                                                "mahayana.mcp.oauth.logout",
                                                other,
                                            ));
                                        }
                                    }
                                    self.emit_connector_snapshot_change(connector_id, "removed")?;
                                }
                            }
                        }
                    } else {
                        for account in accounts {
                            let synthetic = FeatureCommand::ConnectorRemoveAccount {
                                request_id: format!("{request_id}:{}", account.id),
                                connector_id: connector_id.into(),
                                account_id: account.id,
                            };
                            self.execute_live_product_surface_production(&synthetic)?;
                        }
                    }
                }
                let payload = serde_json::to_value(command).map_err(|error| {
                    FeatureHostError::Contract(format!("encode listener.disconnect: {error}"))
                })?;
                let response = self
                    .runtime()?
                    .product_execute("mahayana.listener.disconnect", &payload)?;
                let integration =
                    decode_product_field(response, "integration", "mahayana.listener.disconnect")?;
                self.state()?.events.push_back(HostEvent::ListenerChanged {
                    timestamp: timestamp(),
                    integration,
                });
                Ok(Some(CommandAccepted {
                    request_id,
                    operation_id: None,
                }))
            }
            _ => Ok(None),
        }
    }

    #[cfg(feature = "production")]
    fn execute_product_surface_production(
        &self,
        command: FeatureCommand,
    ) -> Result<CommandAccepted, FeatureHostError> {
        if let Some(accepted) = self.execute_live_product_surface_production(&command)? {
            return Ok(accepted);
        }
        let request_id = command.request_id().to_string();
        let method = product_surface_method(&command);
        let payload = serde_json::to_value(&command).map_err(|error| {
            FeatureHostError::Contract(format!("encode {method} payload: {error}"))
        })?;

        if let FeatureCommand::DraftResolve {
            draft,
            action: DraftAction::Send,
            ..
        } = &command
        {
            validate_draft(draft)?;
            self.state()?.events.push_back(HostEvent::DraftChanged {
                timestamp: timestamp(),
                draft_id: draft.id().to_string(),
                status: DraftSendState::Sending,
                error: None,
            });
        }

        let response = self
            .runtime()?
            .product_execute(method, &payload)
            .map_err(FeatureHostError::from)?;
        let mut state = self.state()?;
        match command {
            FeatureCommand::ConnectorList { .. } => {
                let connectors = decode_product_field(response, "connectors", method)?;
                state.events.push_back(HostEvent::ConnectorListed {
                    timestamp: timestamp(),
                    connectors,
                });
            }
            FeatureCommand::ConnectorConnect { connector_id, .. } => {
                if let Some(url) = response
                    .get("authorizationUrl")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                {
                    state.events.push_back(HostEvent::ConnectorOauthRequested {
                        timestamp: timestamp(),
                        connector_id,
                        authorization_url: url,
                    });
                }
                if response.get("connector").is_some() || response.get("id").is_some() {
                    let connector = decode_product_field(response, "connector", method)?;
                    state.events.push_back(HostEvent::ConnectorChanged {
                        timestamp: timestamp(),
                        action: "connected".into(),
                        connector,
                    });
                }
            }
            FeatureCommand::ConnectorRenameAccount { .. }
            | FeatureCommand::ConnectorRemoveAccount { .. }
            | FeatureCommand::ConnectorSetToolEnabled { .. } => {
                let action = match command {
                    FeatureCommand::ConnectorRemoveAccount { .. } => "removed",
                    FeatureCommand::ConnectorSetToolEnabled { .. } => "toolChanged",
                    _ => "updated",
                };
                let connector = decode_product_field(response, "connector", method)?;
                state.events.push_back(HostEvent::ConnectorChanged {
                    timestamp: timestamp(),
                    action: action.into(),
                    connector,
                });
            }
            FeatureCommand::SkillList { .. } => {
                let skills = decode_product_field(response.clone(), "skills", method)?;
                let teams = response
                    .get("teams")
                    .cloned()
                    .map(|value| decode_product_value(value, method))
                    .transpose()?
                    .unwrap_or_default();
                state.events.push_back(HostEvent::SkillListed {
                    timestamp: timestamp(),
                    skills,
                    teams,
                });
            }
            FeatureCommand::SkillUpsert { .. }
            | FeatureCommand::SkillDelete { .. }
            | FeatureCommand::SkillPublish { .. }
            | FeatureCommand::SkillUnpublish { .. }
            | FeatureCommand::SkillSync { .. } => {
                let action = match command {
                    FeatureCommand::SkillDelete { .. } => "deleted".to_string(),
                    FeatureCommand::SkillPublish { .. } => "published".to_string(),
                    FeatureCommand::SkillUnpublish { .. } => "unpublished".to_string(),
                    FeatureCommand::SkillSync { .. } => "synced".to_string(),
                    _ => response
                        .get("action")
                        .and_then(Value::as_str)
                        .unwrap_or("updated")
                        .to_string(),
                };
                let skill = decode_product_field(response, "skill", method)?;
                state.events.push_back(HostEvent::SkillChanged {
                    timestamp: timestamp(),
                    action,
                    skill,
                });
            }
            FeatureCommand::BotList { .. } => {
                let mut bots: Vec<BotSummary> = decode_product_field(response, "bots", method)?;
                let mut known = bots
                    .iter()
                    .map(|bot| bot.id.clone())
                    .collect::<BTreeSet<_>>();
                for bot in state.bots.values() {
                    if known.insert(bot.id.clone()) {
                        bots.push(bot.clone());
                    }
                }
                state.events.push_back(HostEvent::BotListed {
                    timestamp: timestamp(),
                    bots,
                });
            }
            FeatureCommand::BotSetHidden { .. } => {
                let bot = decode_product_field(response, "bot", method)?;
                state.events.push_back(HostEvent::BotChanged {
                    timestamp: timestamp(),
                    action: "updated".into(),
                    bot,
                });
            }
            FeatureCommand::DraftResolve { draft, action, .. } => {
                let status = response
                    .get("status")
                    .cloned()
                    .map(|value| decode_product_value(value, method))
                    .transpose()?
                    .unwrap_or(match action {
                        DraftAction::Send => DraftSendState::Sent,
                        DraftAction::Discard => DraftSendState::Discarded,
                    });
                state.events.push_back(HostEvent::DraftChanged {
                    timestamp: timestamp(),
                    draft_id: draft.id().to_string(),
                    status,
                    error: response
                        .get("error")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                });
            }
            FeatureCommand::SecretProvide {
                secret_request_id, ..
            } => {
                state.events.push_back(HostEvent::SecretProvided {
                    timestamp: timestamp(),
                    secret_request_id,
                });
            }
            FeatureCommand::ListenerList { .. } => {
                let integrations = decode_product_field(response, "integrations", method)?;
                state.events.push_back(HostEvent::ListenerListed {
                    timestamp: timestamp(),
                    integrations,
                });
            }
            FeatureCommand::ListenerConnect { .. } | FeatureCommand::ListenerDisconnect { .. } => {
                let integration = decode_product_field(response, "integration", method)?;
                state.events.push_back(HostEvent::ListenerChanged {
                    timestamp: timestamp(),
                    integration,
                });
            }
            FeatureCommand::UpdateStatus { .. }
            | FeatureCommand::UpdateCheck { .. }
            | FeatureCommand::UpdateInstall { .. } => {
                let update_state = decode_product_field(response, "state", method)?;
                state.events.push_back(HostEvent::UpdateChanged {
                    timestamp: timestamp(),
                    state: update_state,
                });
            }
            _ => unreachable!("non-product-surface command routed to product executor"),
        }
        Ok(CommandAccepted {
            request_id,
            operation_id: None,
        })
    }

    /// Delivers a verified listener event into the automation engine. This is
    /// intentionally not a renderer command: only the backend relay/native
    /// listener bridge may call it, so web content cannot forge trigger events.
    pub fn ingest_listener_event(&self, event: EventCard) -> Result<usize, FeatureHostError> {
        let serialized = serde_json::to_string(&event).map_err(|error| {
            FeatureHostError::Contract(format!("encode listener event: {error}"))
        })?;
        let matching_ids = {
            let state = self.state()?;
            state
                .automations
                .values()
                .filter(|automation| {
                    if !automation.enabled {
                        return false;
                    }
                    let Some(AutomationTrigger::Event {
                        source,
                        event: expected_event,
                        filter,
                    }) = automation.trigger.as_ref()
                    else {
                        return false;
                    };
                    *source == event.source
                        && (expected_event == "*" || expected_event == &event.event)
                        && filter.as_ref().is_none_or(|filter| {
                            serialized
                                .to_ascii_lowercase()
                                .contains(&filter.to_ascii_lowercase())
                        })
                })
                .map(|automation| {
                    (
                        automation_state_key(
                            fabu_automation_owner(automation),
                            &automation.id,
                        ),
                        automation.id.clone(),
                    )
                })
                .collect::<Vec<_>>()
        };

        for (state_key, id) in &matching_ids {
            let automation = {
                let mut state = self.state()?;
                let automation = state.automations.get_mut(state_key).ok_or_else(|| {
                    FeatureHostError::Contract(format!("unknown automation: {id}"))
                })?;
                automation.last_run_at_ms = Some(event.occurred_at_ms.unwrap_or_else(now_millis));
                let automation = automation.clone();
                self.persist_automations(&state.automations)?;
                state.events.push_back(HostEvent::AutomationChanged {
                    timestamp: timestamp(),
                    action: "triggered".into(),
                    automation: automation.clone(),
                });
                state.events.push_back(HostEvent::TranscriptCard {
                    timestamp: timestamp(),
                    entry_id: format!("relay-{}-{}", automation.id, now_millis()),
                    operation_id: None,
                    card: TranscriptCard::Event {
                        event: event.clone(),
                    },
                });
                automation
            };
            let details = event
                .fields
                .as_ref()
                .map(|fields| {
                    fields
                        .iter()
                        .map(|field| format!("{}: {}", field.label, field.value))
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .unwrap_or_default();
            let text = format!(
                "[事件自动化：{}]\n来源：{}\n事件：{}\n标题：{}\n摘要：{}{}\n\n这是用户保存的 standing instruction。请基于上面的真实事件立即执行并报告结果。\n\n{}",
                automation.name,
                listener_platform_display(event.source),
                event.event,
                event.title,
                event.summary,
                if details.is_empty() {
                    String::new()
                } else {
                    format!("\n{details}")
                },
                automation.prompt
            );
            let request_id = format!("listener-{}-{}", automation.id, now_millis());
            let target_agent_id = automation
                .agent_id
                .clone()
                .unwrap_or_else(|| "mahayana-assistant".into());
            match self.config.mode {
                HostMode::Test => {
                    self.execute_test(FeatureCommand::ChatSend {
                        request_id,
                        text,
                        agent_id: Some(target_agent_id.clone()),
                        conversation_id: None,
                        mode: AgentMode::Agent,
                        mode_statement: None,
                        model: None,
                        attachments: Vec::new(),
                    })?;
                }
                HostMode::Production => {
                    #[cfg(feature = "production")]
                    {
                        self.production_chat(
                            request_id,
                            text,
                            Some(target_agent_id),
                            None,
                            AgentMode::Agent,
                            None,
                            None,
                            Vec::new(),
                        )?;
                    }
                    #[cfg(not(feature = "production"))]
                    return Err(FeatureHostError::ProductionUnavailable);
                }
            }
        }
        Ok(matching_ids.len())
    }

    pub fn receive(&self) -> Result<Option<HostEvent>, FeatureHostError> {
        self.receive_with_timeout(Duration::ZERO)
    }

    pub fn receive_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Option<HostEvent>, FeatureHostError> {
        self.fire_due_automation()?;
        if let Some(event) = self.state()?.events.pop_front() {
            return Ok(Some(event));
        }
        match self.config.mode {
            HostMode::Test => Ok(None),
            HostMode::Production => self.receive_production(timeout),
        }
    }

    fn fire_due_automation(&self) -> Result<(), FeatureHostError> {
        let now = now_millis();
        let due = self
            .state()?
            .automations
            .values()
            .find(|automation| {
                automation.enabled && automation.next_run_at_ms.is_some_and(|next| next <= now)
            })
            .map(|automation| {
                (
                    automation.id.clone(),
                    Some(fabu_automation_owner(automation).to_string()),
                )
            });
        if let Some((id, agent_id)) = due {
            let _ = self.execute_automation(FeatureCommand::AutomationRun {
                request_id: format!("scheduled-{id}-{now}"),
                id,
                agent_id,
            })?;
        }
        Ok(())
    }

    fn persist_test_auth_user(&self, user: Option<&Value>) -> Result<(), FeatureHostError> {
        let Some(path) = self.test_auth_state_path.as_deref() else {
            return Ok(());
        };
        persist_test_auth_user(path, user)
    }

    fn persist_automations(
        &self,
        automations: &BTreeMap<String, AutomationSummary>,
    ) -> Result<(), FeatureHostError> {
        let Some(agent_root) = self.active_account_root(self.memory_root_path.as_deref()) else {
            return Ok(());
        };
        persist_fabu_agent_automations(&agent_root, automations)
    }

    fn persist_bots(&self, bots: &BTreeMap<String, BotSummary>) -> Result<(), FeatureHostError> {
        let Some(path) = self.active_account_root(self.bot_state_path.as_deref()) else {
            return Ok(());
        };
        persist_bots(&path, bots)
    }

    fn persist_agent_manifest(&self, bot: &BotSummary) -> Result<(), FeatureHostError> {
        let agent_id = bot.agent_id.as_deref().unwrap_or(bot.id.as_str());
        if !is_safe_memory_agent_id(agent_id) {
            return Ok(());
        }
        let Some(root) = self.active_account_root(self.memory_root_path.as_deref()) else {
            return Ok(());
        };
        persist_fabu_agent_manifest(&root.join(agent_id), bot)
    }

    fn clone_agent_local_state(
        &self,
        source_agent_id: &str,
        target_agent_id: &str,
    ) -> Result<(), FeatureHostError> {
        if !is_safe_memory_agent_id(source_agent_id)
            || !is_safe_memory_agent_id(target_agent_id)
        {
            return Err(FeatureHostError::Contract(
                "unsafe Agent id for clone".into(),
            ));
        }
        let Some(root) = self.active_account_root(self.memory_root_path.as_deref()) else {
            return Ok(());
        };
        clone_fabu_agent_local_state(&root, source_agent_id, target_agent_id)
    }

    fn persist_groups(
        &self,
        groups: &BTreeMap<String, GroupSummary>,
    ) -> Result<(), FeatureHostError> {
        let Some(path) = self.active_account_root(self.group_state_path.as_deref()) else {
            return Ok(());
        };
        persist_groups(&path, groups)
    }

    fn persist_peer_messages(&self, messages: &[AgentPeerMessage]) -> Result<(), FeatureHostError> {
        let Some(path) = self.active_account_root(self.peer_messages_path.as_deref()) else {
            return Ok(());
        };
        persist_peer_messages(&path, messages)
    }

    fn active_account_root(&self, base: Option<&Path>) -> Option<PathBuf> {
        let base = base?;
        #[cfg(feature = "production")]
        {
            // The production feature is also enabled by the cross-platform
            // test harness, while HostMode::Test deliberately has no live
            // MahayanaHost or product account. Keep that harness on its
            // isolated temporary root; real production hosts use the
            // account-scoped branch below.
            if self.config.mode == HostMode::Test {
                Some(base.to_path_buf())
            } else {
                let account_id = self
                    .active_account_id
                    .lock()
                    .ok()
                    .and_then(|account| account.clone())?;
                Some(account_scoped_path(base, &account_id))
            }
        }
        #[cfg(not(feature = "production"))]
        {
            Some(base.to_path_buf())
        }
    }

    #[cfg(feature = "production")]
    fn messaging_root_for(
        &self,
        envelope: &MessagingClientEnvelope,
    ) -> Result<PathBuf, FeatureHostError> {
        let auth_status = self.auth_status()?;
        let auth = auth_payload(&auth_status);
        if auth.get("loggedIn").and_then(Value::as_bool) != Some(true) {
            return Err(FeatureHostError::Contract(
                "messaging commands require an authenticated Fabushi account session".into(),
            ));
        }
        let account_id = auth_account_id(auth).ok_or_else(|| {
            FeatureHostError::Contract("authenticated account has no stable user id".into())
        })?;
        let expected_actor = actor_id_for_account_id(&account_id);
        if envelope.context.actor_id != expected_actor {
            return Err(FeatureHostError::Contract(
                "messaging actor does not match the authenticated Fabushi account".into(),
            ));
        }
        let base = self
            .memory_root_path
            .as_deref()
            .ok_or_else(|| FeatureHostError::Contract("messaging storage is unavailable".into()))?;
        Ok(account_scoped_path(base, &account_id))
    }

    #[cfg(not(feature = "production"))]
    fn messaging_root_for(
        &self,
        _envelope: &MessagingClientEnvelope,
    ) -> Result<PathBuf, FeatureHostError> {
        self.memory_root_path
            .clone()
            .ok_or_else(|| FeatureHostError::Contract("messaging storage is unavailable".into()))
    }

    #[cfg(feature = "production")]
    fn ensure_account_boundary(&self, response: &Value) -> Result<(), FeatureHostError> {
        let auth = auth_payload(response);
        let logged_in = auth.get("loggedIn").and_then(Value::as_bool) == Some(true);
        let next_account_id = logged_in.then(|| auth_account_id(auth)).flatten();
        let changed = {
            let active = self
                .active_account_id
                .lock()
                .map_err(|_| FeatureHostError::StatePoisoned)?;
            *active != next_account_id
        };
        if changed {
            let account_id = next_account_id.as_deref();
            let history_path = match (self.memory_root_path.as_deref(), account_id) {
                (Some(root), Some(id)) => Some(
                    account_scoped_path(root, id).join("_runtime-transcript.json"),
                ),
                _ => None,
            };
            self.runtime()?.switch_conversation_history(history_path)?;
            let mut automations = match (self.memory_root_path.as_deref(), account_id) {
                (Some(root), Some(id)) => {
                    load_fabu_agent_automations(&account_scoped_path(root, id))
                }
                _ => BTreeMap::new(),
            };
            // One-time backward-compatible migration from the pre-Fabu
            // account-wide automations.json into per-Agent automation roots.
            if let (Some(path), Some(id)) = (self.automation_path.as_deref(), account_id) {
                for (legacy_id, mut automation) in
                    load_automations(&account_scoped_path(path, id))
                {
                    if automation.agent_id.is_none() {
                        automation.agent_id = Some("mahayana-assistant".into());
                    }
                    let owner = fabu_automation_owner(&automation).to_string();
                    automations
                        .entry(automation_state_key(&owner, &legacy_id))
                        .or_insert(automation);
                }
            }
            if let (Some(root), Some(id)) = (self.memory_root_path.as_deref(), account_id) {
                persist_fabu_agent_automations(
                    &account_scoped_path(root, id),
                    &automations,
                )?;
            }
            let mut bots = default_bots();
            if let (Some(path), Some(account_id)) = (self.bot_state_path.as_deref(), account_id) {
                bots.extend(load_bots(&account_scoped_path(path, account_id)));
            }
            let groups = self
                .group_state_path
                .as_deref()
                .map(|path| {
                    account_id
                        .map(|id| load_groups(&account_scoped_path(path, id)))
                        .unwrap_or_default()
                })
                .unwrap_or_default();
            let peer_messages = self
                .peer_messages_path
                .as_deref()
                .map(|path| {
                    account_id
                        .map(|id| load_peer_messages(&account_scoped_path(path, id)))
                        .unwrap_or_default()
                })
                .unwrap_or_default();
            let remote_device_secrets = self
                .remote_device_state_path
                .as_deref()
                .map(|path| {
                    account_id
                        .map(|id| {
                            load_remote_computer_device_secrets(&account_scoped_path(path, id))
                        })
                        .unwrap_or_default()
                })
                .unwrap_or_default();
            let mut state = self.state()?;
            state.events.clear();
            state.pending_approvals.clear();
            state.operations.clear();
            state.operation_agents.clear();
            state.background_operations.clear();
            state.automations = automations;
            state.bots = bots;
            state.peer_messages = peer_messages;
            state.groups.clear();
            state.groups = groups;
            state.group_runs.clear();
            state.group_operations.clear();
            state.remote_computer_device_secrets = remote_device_secrets;
            state.auth_user = None;
            state.session_active = logged_in;
            drop(state);
            *self
                .active_account_id
                .lock()
                .map_err(|_| FeatureHostError::StatePoisoned)? = next_account_id.clone();

            // Materialize Fabu-compatible per-Agent profile/settings files after
            // the account scope becomes authoritative. This also migrates older
            // bots.json-only profiles without deleting any existing Agent data.
            if logged_in {
                let bots = self
                    .state()?
                    .bots
                    .values()
                    .cloned()
                    .collect::<Vec<_>>();
                for bot in bots {
                    self.persist_agent_manifest(&bot)?;
                }
            }
        }
        {
            let mut state = self.state()?;
            state.session_active = logged_in;
            state.auth_user = if logged_in {
                auth.get("user").cloned()
            } else {
                None
            };
        }
        if logged_in {
            // A signed-in Host is not ready for chat until the real Mahayana
            // provider session exists. This runs on restored sessions and on
            // fresh password/browser/OAuth login, so the first chat.send only
            // submits work; it never becomes the trigger that starts the
            // provider process/thread.
            self.runtime()?.warmup_conversation(ConversationId(
                MAHAYANA_AI_CONVERSATION_ID.to_string(),
            ))?;
        }
        Ok(())
    }

    fn persist_remote_device_secrets(
        &self,
        secrets: &BTreeMap<String, String>,
    ) -> Result<(), FeatureHostError> {
        let Some(path) = self.active_account_root(self.remote_device_state_path.as_deref()) else {
            return Ok(());
        };
        persist_remote_computer_device_secrets(&path, secrets)
    }

    fn remote_device_secret(
        &self,
        device_id: &str,
        create: bool,
    ) -> Result<String, FeatureHostError> {
        if !is_safe_memory_agent_id(device_id) {
            return Err(FeatureHostError::Contract(format!(
                "unsafe remote computer device id: {device_id}"
            )));
        }
        let mut state = self.state()?;
        if let Some(secret) = state.remote_computer_device_secrets.get(device_id) {
            return Ok(secret.clone());
        }
        if !create {
            return Err(FeatureHostError::Contract(
                "remote computer must be registered by this desktop before use".into(),
            ));
        }
        if state.remote_computer_device_secrets.len() >= REMOTE_DEVICE_SECRET_MAX_ENTRIES {
            return Err(FeatureHostError::Contract(
                "too many local remote computer device identities are stored; reset an old profile before registering another".into(),
            ));
        }
        let secret = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
        state
            .remote_computer_device_secrets
            .insert(device_id.to_string(), secret.clone());
        let secrets = state.remote_computer_device_secrets.clone();
        drop(state);
        self.persist_remote_device_secrets(&secrets)?;
        Ok(secret)
    }

    pub fn resolve_approval(&self, resolution: ApprovalResolution) -> Result<(), FeatureHostError> {
        let pending = {
            let mut state = self.state()?;
            ensure_open(&state)?;
            state
                .pending_approvals
                .remove(&resolution.approval_id)
                .ok_or_else(|| {
                    FeatureHostError::Contract(format!(
                        "unknown approval: {}",
                        resolution.approval_id
                    ))
                })?
        };

        #[cfg(not(feature = "production"))]
        let _ = &pending;

        if self.config.mode == HostMode::Production {
            #[cfg(feature = "production")]
            {
                if let Some(runtime_approval_id) = pending.runtime_approval_id.as_ref() {
                    let decision = match resolution.decision {
                        ApprovalDecision::AllowOnce => RuntimeApprovalDecision::Accept,
                        ApprovalDecision::AllowSession => RuntimeApprovalDecision::AcceptForSession,