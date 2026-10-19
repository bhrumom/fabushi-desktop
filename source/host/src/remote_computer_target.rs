use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;
use std::net::{IpAddr, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use tungstenite::client::IntoClientRequest;
use tungstenite::http::header::AUTHORIZATION;
use tungstenite::http::HeaderValue;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket, WebSocketConfig};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use mahayana_host_protocol::{
    ComputerAction, ComputerActionKind, ComputerControlOrigin, ComputerMouseButton, ComputerPoint,
    ComputerScrollDirection,
};
use thiserror::Error;
use url::Url;
use uuid::Uuid;

use crate::extensions::auth::auth_service::{
    FabushiCoreCredential, FabushiCoreCredentialProvider,
};
use crate::network_enrollment_owner::{NetworkEnrollmentError, NetworkWorkerService};
use crate::network_worker_process::OwnedNetworkWorker;
use crate::runner::routed_provider_runtime::RoutedToolBridge;
use crate::sensitive_input_challenge::{
    SensitiveInputBinding, SensitiveInputChallengeOwner, SENSITIVE_INPUT_CHALLENGE_MAX_TTL_MS,
};
use crate::runner::tools::sand_computer_tool::{
    ComputerActionArgs, ComputerActionName, MouseButtonInput, ScrollDirectionInput,
    parse_remote_computer_action,
};

const CORE_REQUEST_TIMEOUT_SECS: u64 = 15;
const MAX_CORE_RESPONSE_BYTES: usize = 1024 * 1024;
const REMOTE_TARGET_STREAM_TIMEOUT_MS: u64 = 15_000;
const REMOTE_TARGET_SERVICE_STREAM_TIMEOUT_MS: u64 = 1_000;
const REMOTE_TARGET_INGRESS_CACHE_TTL: Duration = Duration::from_secs(10);
const REMOTE_TARGET_FRAME_MAX_BYTES: usize = 32 * 1024;
const REMOTE_TARGET_RESULT_MAX_BYTES: usize = 1024 * 1024;
const REMOTE_SENSITIVE_RESULT_MAX_BYTES: usize = 8 * 1024 * 1024;
const REMOTE_SENSITIVE_RESULT_CHUNK_BYTES: usize = 16 * 1024;
const REMOTE_SENSITIVE_RESULT_TTL_SECONDS: i64 = 60;
const REMOTE_SENSITIVE_RESULT_MAX_ENTRIES: usize = 8;
pub const REMOTE_COMPUTER_TARGET_PORT: u16 = 8792;
const REMOTE_NATIVE_TOOL_GRANT_MAX_SECONDS: i64 = 5 * 60;
const MAX_PRIVATE_WS_MESSAGE_BYTES: usize = 64 * 1024;
const MAX_ACTIVE_PRIVATE_WS_REQUESTS: usize = 16;
const PRIVATE_WS_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(20);
const PRIVATE_WS_IDENTITY_CHECK_INTERVAL: Duration = Duration::from_secs(2);
const PRIVATE_WS_NETWORK_CHECK_INTERVAL: Duration = Duration::from_secs(5);
const PRIVATE_WS_READ_TIMEOUT: Duration = Duration::from_millis(50);
const PRIVATE_WS_REQUEST_RECEIVED: u8 = 0;
const PRIVATE_WS_REQUEST_DISPATCHED: u8 = 1;
const PRIVATE_WS_REQUEST_CANCELLED_BEFORE_DISPATCH: u8 = 2;
const PRIVATE_WS_REQUEST_FINISHED: u8 = 3;
pub const FABUSHI_FDN_PRIVATE_WS_URL_ENV: &str = "FABUSHI_FDN_PRIVATE_WS_URL";

fn canonical_json(value: &Value, output: &mut String) -> Result<(), RemoteComputerTargetError> {
    match value {
        Value::Null => output.push_str("null"),
        Value::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
        Value::Number(value) => output.push_str(&value.to_string()),
        Value::String(value) => output.push_str(
            &serde_json::to_string(value).map_err(|_| RemoteComputerTargetError::InvalidResponse)?,
        ),
        Value::Array(values) => {
            output.push('[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                canonical_json(value, output)?;
            }
            output.push(']');
        }
        Value::Object(values) => {
            output.push('{');
            let mut keys = values.keys().collect::<Vec<_>>();
            keys.sort_unstable();
            for (index, key) in keys.into_iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                output.push_str(
                    &serde_json::to_string(key)
                        .map_err(|_| RemoteComputerTargetError::InvalidResponse)?,
                );
                output.push(':');
                canonical_json(&values[key], output)?;
            }
            output.push('}');
        }
    }
    Ok(())
}

pub fn operation_request_sha256(
    tool_name: &str,
    arguments: &Value,
    disk_preflight: &Value,
) -> Result<String, RemoteComputerTargetError> {
    let request = json!({
        "arguments": arguments,
        "diskPreflight": disk_preflight,
        "toolName": tool_name,
    });
    let mut canonical = String::new();
    canonical_json(&request, &mut canonical)?;
    let digest = Sha256::digest(canonical.as_bytes());
    let mut encoded = String::with_capacity(64);
    use std::fmt::Write as _;
    for byte in digest {
        write!(&mut encoded, "{byte:02x}").map_err(|_| RemoteComputerTargetError::InvalidResponse)?;
    }
    Ok(encoded)
}

pub trait RemoteComputerDeviceSecretProvider: Send + Sync {
    fn get_device_secret(&self, credential: &FabushiCoreCredential) -> Result<String, RemoteComputerTargetError>;
}

pub const FABUSHI_REMOTE_COMPUTER_DEVICE_CREDENTIAL_FILE_ENV: &str =
    "FABUSHI_REMOTE_COMPUTER_DEVICE_CREDENTIAL_FILE";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RemoteComputerDeviceCredentialFile {
    version: u32,
    user_id: serde_json::Value,
    device_id: String,
    device_secret: String,
}

#[derive(Clone)]
pub struct FileRemoteComputerDeviceSecretProvider {
    path: PathBuf,
}

impl std::fmt::Debug for FileRemoteComputerDeviceSecretProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FileRemoteComputerDeviceSecretProvider")
            .field("path", &"<host-private>")
            .finish()
    }
}

impl FileRemoteComputerDeviceSecretProvider {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn from_environment() -> Result<Self, RemoteComputerTargetError> {
        let path = std::env::var(FABUSHI_REMOTE_COMPUTER_DEVICE_CREDENTIAL_FILE_ENV)
            .map_err(|_| RemoteComputerTargetError::AuthorityUnavailable)?;
        if path.trim().is_empty() {
            return Err(RemoteComputerTargetError::AuthorityUnavailable);
        }
        Ok(Self::new(path))
    }
}

impl RemoteComputerDeviceSecretProvider for FileRemoteComputerDeviceSecretProvider {
    fn get_device_secret(
        &self,
        credential: &FabushiCoreCredential,
    ) -> Result<String, RemoteComputerTargetError> {
        let raw = fs::read_to_string(&self.path)
            .map_err(|_| RemoteComputerTargetError::AuthorityUnavailable)?;
        let parsed: RemoteComputerDeviceCredentialFile = serde_json::from_str(&raw)
            .map_err(|_| RemoteComputerTargetError::AuthorityUnavailable)?;
        let user_id = match parsed.user_id {
            Value::String(value) if !value.trim().is_empty() => value,
            Value::Number(value) => value.to_string(),
            _ => return Err(RemoteComputerTargetError::AuthorityUnavailable),
        };
        if parsed.version != 1
            || parsed.device_id != credential.device_id
            || user_id != credential.user_id
            || parsed.device_secret.len() < 48
            || parsed.device_secret.len() > 256
        {
            return Err(RemoteComputerTargetError::AuthorityUnavailable);
        }
        Ok(parsed.device_secret)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RemoteComputerTargetError {
    #[error("Fabushi target authority is unavailable.")]
    AuthorityUnavailable,
    #[error("Fabushi Core target request failed.")]
    RequestFailed,
    #[error("Fabushi Core target response failed identity validation.")]
    IdentityMismatch,
    #[error("Fabushi Core target response is invalid.")]
    InvalidResponse,
    #[error("Fabushi Remote Computer private transport failed.")]
    TransportFailed,
    #[error("Fabushi Remote Computer operation outcome is unknown; do not retry automatically.")]
    OutcomeUnknown,
}



#[derive(Clone, PartialEq, Eq)]
pub struct RemoteComputerSensitiveResult {
    pub media_type: String,
    pub bytes: Vec<u8>,
    pub captured_at_ms: i64,
    pub snapshot_generation: u64,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl std::fmt::Debug for RemoteComputerSensitiveResult {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RemoteComputerSensitiveResult")
            .field("media_type", &self.media_type)
            .field("bytes", &format_args!("<redacted:{} bytes>", self.bytes.len()))
            .field("captured_at_ms", &self.captured_at_ms)
            .field("snapshot_generation", &self.snapshot_generation)
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RemoteComputerNativeOutcome {
    Completed(Value),
    SensitiveCompleted(RemoteComputerSensitiveResult),
    Failed(Value),
    OutcomeUnknown,
}

pub trait RemoteComputerNativeExecutor: Send + Sync {
    fn execute(
        &self,
        operation: &ClaimedRemoteOperation,
        arguments: &Value,
        disk_preflight: &Value,
    ) -> RemoteComputerNativeOutcome;
}

pub trait RemoteComputerToolAuthorizer: Send + Sync {
    fn authorize_tool(
        &self,
        session_id: &str,
        client_id: &str,
        tool_name: &str,
        requested_expires_at: i64,
    ) -> Result<i64, RemoteComputerTargetError>;
}

pub trait RemoteComputerToolCatalog: Send + Sync {
    fn describe_tool(&self, tool_name: &str) -> Result<Option<Value>, RemoteComputerTargetError>;
}

pub struct RoutedRemoteComputerToolCatalog {
    bridge: Arc<dyn RoutedToolBridge>,
}

impl RoutedRemoteComputerToolCatalog {
    pub fn new(bridge: Arc<dyn RoutedToolBridge>) -> Self {
        Self { bridge }
    }
}

impl RemoteComputerToolCatalog for RoutedRemoteComputerToolCatalog {
    fn describe_tool(&self, tool_name: &str) -> Result<Option<Value>, RemoteComputerTargetError> {
        let tools = self
            .bridge
            .list_tools()
            .map_err(|_| RemoteComputerTargetError::AuthorityUnavailable)?;
        let tool = tools
            .into_iter()
            .find(|tool| tool.name == tool_name || tool.tool_name == tool_name);
        let Some(tool) = tool else {
            return Ok(None);
        };
        let effective_name = if tool.name == tool_name {
            tool.name
        } else {
            tool.tool_name
        };
        let output_schema = matches!(effective_name.as_str(), "Computer" | "Screenshot").then(|| {
            json!({
                "type": "object",
                "additionalProperties": false,
                "required": ["content", "screenshotPath"],
                "properties": {
                    "content": {"type": "string"},
                    "screenshotPath": {"type": ["string", "null"]}
                }
            })
        });
        Ok(Some(json!({
            "name": effective_name,
            "title": effective_name,
            "description": tool.description,
            "inputSchema": tool.input_schema,
            "outputSchema": output_schema,
        })))
    }
}

pub struct NativeDesktopRemoteToolCatalog;

impl RemoteComputerToolCatalog for NativeDesktopRemoteToolCatalog {
    fn describe_tool(&self, tool_name: &str) -> Result<Option<Value>, RemoteComputerTargetError> {
        if tool_name != "Computer" || !mahayana_computer::control_policy().remote_control_enabled {
            return Ok(None);
        }
        Ok(Some(json!({
            "name": "Computer",
            "title": "Computer",
            "description": "Interact with this authorized Fabushi desktop using mouse, keyboard, scrolling, waits, and target-local observation.",
            "inputSchema": {
                "type": "object",
                "additionalProperties": false,
                "required": ["action"],
                "properties": {
                    "action": {"type": "string", "enum": ["screenshot", "click", "doubleClick", "move", "drag", "type", "key", "scroll", "wait"]},
                    "x": {"type": "integer"}, "y": {"type": "integer"},
                    "x2": {"type": "integer"}, "y2": {"type": "integer"},
                    "path": {"type": "array", "items": {"type": "object", "additionalProperties": false, "required": ["x", "y"], "properties": {"x": {"type": "integer"}, "y": {"type": "integer"}}}},
                    "text": {"type": "string"}, "key": {"type": "string"},
                    "button": {"type": "string", "enum": ["left", "right", "middle"]},
                    "count": {"type": "integer", "minimum": 1, "maximum": 3},
                    "direction": {"type": "string", "enum": ["up", "down", "left", "right"]},
                    "amount": {"type": "integer"},
                    "durationMs": {"type": "integer", "minimum": 0, "maximum": 30000},
                    "snapshotGeneration": {"type": "integer", "minimum": 1},
                    "description": {"type": "string"},
                    "then": {"type": "array", "minItems": 1, "maxItems": 9, "items": {"type": "object"}}
                }
            },
            "outputSchema": {
                "type": "object",
                "additionalProperties": false,
                "required": ["content", "actionsExecuted", "capturedAtMs", "snapshotGeneration", "screenshotTransferred"],
                "properties": {
                    "content": {"type": "string"},
                    "actionsExecuted": {"type": "integer", "minimum": 1},
                    "capturedAtMs": {"type": "integer"},
                    "snapshotGeneration": {"type": "integer", "minimum": 1},
                    "width": {"type": ["integer", "null"]},
                    "height": {"type": ["integer", "null"]},
                    "screenshotTransferred": {"type": "boolean"}
                }
            }
        })))
    }
}

pub struct NativeDesktopRemoteToolAuthorizer;

impl RemoteComputerToolAuthorizer for NativeDesktopRemoteToolAuthorizer {
    fn authorize_tool(
        &self,
        session_id: &str,
        client_id: &str,
        tool_name: &str,
        requested_expires_at: i64,
    ) -> Result<i64, RemoteComputerTargetError> {
        let policy = mahayana_computer::control_policy();
        if session_id.trim().is_empty()
            || client_id.trim().is_empty()
            || tool_name != "Computer"
            || !policy.remote_control_enabled
        {
            return Err(RemoteComputerTargetError::AuthorityUnavailable);
        }
        let now = unix_seconds()?;
        if requested_expires_at <= now {
            return Err(RemoteComputerTargetError::AuthorityUnavailable);
        }
        Ok(requested_expires_at.min(now.saturating_add(REMOTE_NATIVE_TOOL_GRANT_MAX_SECONDS)))
    }
}

fn native_i32(value: Option<i64>, label: &str) -> Result<Option<i32>, String> {
    value
        .map(|value| i32::try_from(value).map_err(|_| format!("{label} is out of native desktop range")))
        .transpose()
}

fn native_computer_action(args: &ComputerActionArgs) -> Result<ComputerAction, String> {
    let action = match args.action {
        ComputerActionName::Screenshot => ComputerActionKind::Screenshot,
        ComputerActionName::Click => ComputerActionKind::Click,
        ComputerActionName::DoubleClick => ComputerActionKind::Click,
        ComputerActionName::Move => ComputerActionKind::Move,
        ComputerActionName::Drag => ComputerActionKind::Drag,
        ComputerActionName::Type => ComputerActionKind::Type,
        ComputerActionName::Key => ComputerActionKind::Key,
        ComputerActionName::Scroll => ComputerActionKind::Scroll,
        ComputerActionName::Wait => ComputerActionKind::Wait,
    };
    let path = if args.path.is_empty() {
        None
    } else {
        Some(args.path.iter().map(|point| {
            Ok(ComputerPoint {
                x: i32::try_from(point.x).map_err(|_| "drag path x is out of native desktop range".to_string())?,
                y: i32::try_from(point.y).map_err(|_| "drag path y is out of native desktop range".to_string())?,
            })
        }).collect::<Result<Vec<_>, String>>()?)
    };
    Ok(ComputerAction {
        action,
        x: native_i32(args.x, "x")?, y: native_i32(args.y, "y")?,
        x2: native_i32(args.x2, "x2")?, y2: native_i32(args.y2, "y2")?,
        path,
        text: args.text.clone(), key: args.key.clone(),
        button: args.button.map(|button| match button {
            MouseButtonInput::Left => ComputerMouseButton::Left,
            MouseButtonInput::Right => ComputerMouseButton::Right,
            MouseButtonInput::Middle => ComputerMouseButton::Middle,
        }),
        click_count: if args.action == ComputerActionName::DoubleClick {
            if args.count.is_some() {
                return Err("doubleClick has a fixed click count of 2 and does not accept count".into());
            }
            Some(2)
        } else {
            args.count
        },
        direction: args.direction.map(|direction| match direction {
            ScrollDirectionInput::Up => ComputerScrollDirection::Up,
            ScrollDirectionInput::Down => ComputerScrollDirection::Down,
            ScrollDirectionInput::Left => ComputerScrollDirection::Left,
            ScrollDirectionInput::Right => ComputerScrollDirection::Right,
        }),
        amount: native_i32(args.amount, "amount")?,
        wait_ms: args.duration_ms,
        description: args.description.clone(),
    })
}

fn native_computer_actions(args: &ComputerActionArgs) -> Result<Vec<ComputerAction>, String> {
    let mut actions = Vec::with_capacity(1 + args.then_actions.len());
    actions.push(native_computer_action(args)?);
    for follow_up in &args.then_actions {
        actions.push(native_computer_action(follow_up)?);
    }
    Ok(actions)
}

pub struct NativeDesktopRemoteExecutor;

impl RemoteComputerNativeExecutor for NativeDesktopRemoteExecutor {
    fn execute(
        &self,
        operation: &ClaimedRemoteOperation,
        arguments: &Value,
        _disk_preflight: &Value,
    ) -> RemoteComputerNativeOutcome {
        if operation.tool_name != "Computer" {
            return RemoteComputerNativeOutcome::Failed(json!({"code":"unsupported_tool","message":"Only the product Computer contract is enabled on this target path.","retryable":false}));
        }
        if !mahayana_computer::control_policy().remote_control_enabled {
            return RemoteComputerNativeOutcome::Failed(json!({"code":"remote_control_disabled","message":"Remote Computer is disabled in Fabushi settings.","retryable":false}));
        }
        let parsed = match parse_remote_computer_action(arguments) {
            Ok(action) => action,
            Err(error) => return RemoteComputerNativeOutcome::Failed(json!({"code":"invalid_arguments","message":error.to_string(),"retryable":false})),
        };
        if parsed.then_actions.iter().any(|action| action.action == ComputerActionName::Screenshot) {
            return RemoteComputerNativeOutcome::Failed(json!({"code":"invalid_arguments","message":"Screenshot may only be the primary Computer action.","retryable":false}));
        }
        let sensitive_screenshot = parsed.action == ComputerActionName::Screenshot;
        if sensitive_screenshot && !operation.sensitive_result {
            return RemoteComputerNativeOutcome::Failed(json!({"code":"sensitive_result_required","message":"Remote screenshot requires a sensitive-result operation.","retryable":false}));
        }
        let actions = match native_computer_actions(&parsed) {
            Ok(actions) => actions,
            Err(message) => return RemoteComputerNativeOutcome::Failed(json!({"code":"invalid_arguments","message":message,"retryable":false})),
        };
        let lease = mahayana_computer::ComputerControlLeaseRequest::new(
            format!("official-device-mcp:{}", operation.client_id),
            operation.session_id.clone(),
            "local-desktop",
            ComputerControlOrigin::RemoteMobile,
            "remote-human",
        );
        match mahayana_computer::execute_with_lease_fenced(
            &actions,
            ComputerControlOrigin::RemoteMobile,
            &lease,
            parsed.snapshot_generation,
        ) {
            Ok(observed) => {
                let result = observed.result;
                if sensitive_screenshot {
                    return match decode_sensitive_snapshot(
                        &result.snapshot.data_url,
                        result.snapshot.captured_at_ms,
                        observed.snapshot_generation,
                        result.snapshot.width,
                        result.snapshot.height,
                    ) {
                        Ok(sensitive) => RemoteComputerNativeOutcome::SensitiveCompleted(sensitive),
                        Err(_) => RemoteComputerNativeOutcome::Failed(json!({
                            "code":"computer_capture_invalid",
                            "message":"Target screenshot could not be prepared for the authenticated sensitive-result channel.",
                            "retryable":false
                        })),
                    };
                }
                RemoteComputerNativeOutcome::Completed(json!({
                    "content": "Computer action ran on the authorized target desktop.",
                    "actionsExecuted": result.actions_executed,
                    "capturedAtMs": result.snapshot.captured_at_ms,
                    "snapshotGeneration": observed.snapshot_generation,
                    "width": result.snapshot.width,
                    "height": result.snapshot.height,
                    "screenshotTransferred": false,
                }))
            }
            Err(mahayana_computer::ComputerError::SnapshotStale { expected, current }) => {
                RemoteComputerNativeOutcome::Failed(json!({
                    "code":"snapshot_stale",
                    "message":"The requested desktop observation is stale; observe again before retrying the action.",
                    "expectedSnapshotGeneration": expected,
                    "currentSnapshotGeneration": current,
                    "retryable":false
                }))
            }
            Err(error) => RemoteComputerNativeOutcome::Failed(json!({"code":"computer_action_failed","message":error.to_string(),"retryable":false})),
        }
    }
}

fn decode_sensitive_snapshot(
    data_url: &str,
    captured_at_ms: i64,
    snapshot_generation: u64,
    width: Option<u32>,
    height: Option<u32>,
) -> Result<RemoteComputerSensitiveResult, RemoteComputerTargetError> {
    let Some((header, encoded)) = data_url.split_once(',') else {
        return Err(RemoteComputerTargetError::InvalidResponse);
    };
    let media_type = header
        .strip_prefix("data:")
        .and_then(|value| value.strip_suffix(";base64"))
        .filter(|value| matches!(*value, "image/jpeg" | "image/png" | "image/webp"))
        .ok_or(RemoteComputerTargetError::InvalidResponse)?
        .to_string();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| RemoteComputerTargetError::InvalidResponse)?;
    if bytes.is_empty() || bytes.len() > REMOTE_SENSITIVE_RESULT_MAX_BYTES {
        return Err(RemoteComputerTargetError::InvalidResponse);
    }
    Ok(RemoteComputerSensitiveResult {
        media_type,
        bytes,
        captured_at_ms,
        snapshot_generation,
        width,
        height,
    })
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
enum RemoteTargetPrivateRequest {
    GrantController {
        device_id: String,
        network_generation: i64,
        connection_id: String,
        delegation_id: String,
        request_id: String,
        label: String,
    },
    DescribeTool {
        device_id: String,
        network_generation: i64,
        connection_id: String,
        delegation_id: String,
        request_id: String,
        label: String,
        tool_name: String,
    },
    AuthorizeTool {
        device_id: String,
        network_generation: i64,
        session_id: String,
        client_id: String,
        tool_name: String,
        expires_at: i64,
    },
    Operation {
        device_id: String,
        network_generation: i64,
        session_id: String,
        client_id: String,
        request_id: String,
        tool_name: String,
        request_sha256: String,
        grant_generation: i64,
        arguments: Value,
        disk_preflight: Value,
    },
    SensitiveInputIssue {
        device_id: String,
        network_generation: i64,
        session_id: String,
        client_id: String,
        request_id: String,
        request_sha256: String,
        grant_generation: i64,
        ttl_ms: i64,
    },
    SensitiveInputSubmit {
        device_id: String,
        network_generation: i64,
        session_id: String,
        client_id: String,
        request_id: String,
        request_sha256: String,
        grant_generation: i64,
        challenge_id: String,
        snapshot_generation: u64,
        payload_base64: String,
    },
    SensitiveResultResume {
        device_id: String,
        network_generation: i64,
        session_id: String,
        client_id: String,
        request_id: String,
        request_sha256: String,
        grant_generation: i64,
    },
    SensitiveResultChunk {
        device_id: String,
        network_generation: i64,
        session_id: String,
        client_id: String,
        request_id: String,
        request_sha256: String,
        grant_generation: i64,
        result_ref: String,
        offset: usize,
    },
    SensitiveResultRevoke {
        device_id: String,
        network_generation: i64,
        session_id: String,
        client_id: String,
        request_id: String,
        request_sha256: String,
        grant_generation: i64,
        result_ref: String,
    },
}

trait RemoteTargetResponseSink {
    fn send_response(&mut self, value: &Value) -> Result<(), RemoteComputerTargetError>;
}

struct NetworkWorkerResponseSink<'a> {
    worker: &'a mut OwnedNetworkWorker,
    stream_id: &'a str,
}

impl RemoteTargetResponseSink for NetworkWorkerResponseSink<'_> {
    fn send_response(&mut self, value: &Value) -> Result<(), RemoteComputerTargetError> {
        write_private_frame(self.worker, self.stream_id, value)
    }
}

#[derive(Default)]
struct CapturedRemoteTargetResponseSink {
    response: Option<Value>,
}

impl RemoteTargetResponseSink for CapturedRemoteTargetResponseSink {
    fn send_response(&mut self, value: &Value) -> Result<(), RemoteComputerTargetError> {
        if self.response.is_some() {
            return Err(RemoteComputerTargetError::InvalidResponse);
        }
        self.response = Some(value.clone());
        Ok(())
    }
}

struct PrivateRequestState {
    phase: AtomicU8,
    completed_at: Mutex<Option<Instant>>,
}

impl Default for PrivateRequestState {
    fn default() -> Self {
        Self {
            phase: AtomicU8::new(PRIVATE_WS_REQUEST_RECEIVED),
            completed_at: Mutex::new(None),
        }
    }
}

impl PrivateRequestState {
    fn request_cancel(&self) -> &'static str {
        loop {
            match self.phase.load(Ordering::Acquire) {
                PRIVATE_WS_REQUEST_RECEIVED => {
                    if self.phase.compare_exchange(
                        PRIVATE_WS_REQUEST_RECEIVED,
                        PRIVATE_WS_REQUEST_CANCELLED_BEFORE_DISPATCH,
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    ).is_ok() {
                        return "cancelled";
                    }
                }
                PRIVATE_WS_REQUEST_DISPATCHED => return "cancellation_pending",
                PRIVATE_WS_REQUEST_CANCELLED_BEFORE_DISPATCH => return "cancelled",
                PRIVATE_WS_REQUEST_FINISHED => return "already_completed",
                _ => return "cancellation_pending",
            }
        }
    }

    fn begin_dispatch(&self) -> bool {
        self.phase.compare_exchange(
            PRIVATE_WS_REQUEST_RECEIVED,
            PRIVATE_WS_REQUEST_DISPATCHED,
            Ordering::AcqRel,
            Ordering::Acquire,
        ).is_ok()
    }

    fn cancelled_before_dispatch(&self) -> bool {
        self.phase.load(Ordering::Acquire) == PRIVATE_WS_REQUEST_CANCELLED_BEFORE_DISPATCH
    }

    fn complete(&self) {
        if self.phase.load(Ordering::Acquire) != PRIVATE_WS_REQUEST_CANCELLED_BEFORE_DISPATCH {
            self.phase.store(PRIVATE_WS_REQUEST_FINISHED, Ordering::Release);
        }
        if let Ok(mut completed_at) = self.completed_at.lock() {
            *completed_at = Some(Instant::now());
        }
    }

    fn can_expire(&self, now: Instant) -> bool {
        let phase = self.phase.load(Ordering::Acquire);
        if phase != PRIVATE_WS_REQUEST_FINISHED
            && phase != PRIVATE_WS_REQUEST_CANCELLED_BEFORE_DISPATCH
        {
            return false;
        }
        self.completed_at.lock().ok()
            .and_then(|completed_at| *completed_at)
            .is_some_and(|completed_at| now.duration_since(completed_at) >= Duration::from_secs(60))
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct RemoteServiceClientGrant {
    pub device_id: String,
    pub client_id: String,
    client_token: String,
    pub connection_id: String,
    pub delegation_id: String,
    pub request_id: String,
    pub reconciled: bool,
}

impl std::fmt::Debug for RemoteServiceClientGrant {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RemoteServiceClientGrant")
            .field("device_id", &self.device_id)
            .field("client_id", &self.client_id)
            .field("client_token", &"<redacted>")
            .field("connection_id", &self.connection_id)
            .field("delegation_id", &self.delegation_id)
            .field("request_id", &self.request_id)
            .field("reconciled", &self.reconciled)
            .finish()
    }
}

impl RemoteServiceClientGrant {
    pub fn client_token(&self) -> &str {
        &self.client_token
    }
}

pub struct ClaimedRemoteOperation {
    pub request_id: String,
    pub session_id: String,
    pub client_id: String,
    pub tool_name: String,
    pub request_sha256: String,
    pub grant_generation: i64,
    pub sensitive_result: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteNetworkBinding {
    pub device_id: String,
    pub cell_id: String,
    pub node_id: String,
    pub proof_sha256: String,
    pub generation: i64,
    pub expires_at: i64,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteNetworkIngress {
    pub device_id: String,
    pub cell_id: String,
    pub target_generation: i64,
    pub service_id: String,
    pub node_id: String,
    pub ingress_generation: i64,
    pub expires_at: i64,
    pub active: bool,
}

fn is_fabushi_tailnet_peer(value: &str) -> bool {
    match value.parse::<IpAddr>() {
        Ok(IpAddr::V4(ip)) => {
            let [first, second, _, _] = ip.octets();
            first == 100 && (64..=127).contains(&second)
        }
        Ok(IpAddr::V6(ip)) => {
            let segments = ip.segments();
            segments[0] == 0xfd7a && segments[1] == 0x115c && segments[2] == 0xa1e0
        }
        Err(_) => false,
    }
}

#[derive(Clone)]
struct SensitiveResultEntry {
    account_user_id: String,
    device_id: String,
    network_generation: i64,
    session_id: String,
    client_id: String,
    request_id: String,
    request_sha256: String,
    tool_name: String,
    grant_generation: i64,
    result_sha256: String,
    media_type: String,
    bytes: Vec<u8>,
    captured_at_ms: i64,
    snapshot_generation: u64,
    width: Option<u32>,
    height: Option<u32>,
    expires_at: i64,
    next_offset: usize,
}

#[derive(Default)]
struct SensitiveResultVault {
    entries: HashMap<String, SensitiveResultEntry>,
}

impl SensitiveResultVault {
    fn purge_expired(&mut self, now: i64) {
        self.entries.retain(|_, entry| entry.expires_at > now);
    }

    fn insert(&mut self, result_ref: String, entry: SensitiveResultEntry, now: i64) {
        self.purge_expired(now);
        if self.entries.len() >= REMOTE_SENSITIVE_RESULT_MAX_ENTRIES {
            if let Some(oldest) = self.entries.iter()
                .min_by_key(|(_, entry)| entry.expires_at)
                .map(|(key, _)| key.clone())
            {
                self.entries.remove(&oldest);
            }
        }
        self.entries.insert(result_ref, entry);
    }
}

fn sensitive_chunk_range(
    entry: &SensitiveResultEntry,
    offset: usize,
) -> Result<(usize, bool), RemoteComputerTargetError> {
    if offset != entry.next_offset || offset >= entry.bytes.len() {
        return Err(RemoteComputerTargetError::IdentityMismatch);
    }
    let end = offset
        .saturating_add(REMOTE_SENSITIVE_RESULT_CHUNK_BYTES)
        .min(entry.bytes.len());
    Ok((end, end == entry.bytes.len()))
}

#[derive(Debug, Clone)]
struct SensitiveResultAccess {
    result_ref: String,
    result_sha256: String,
    media_type: String,
    captured_at_ms: i64,
    snapshot_generation: u64,
    width: Option<u32>,
    height: Option<u32>,
    expires_at: i64,
    next_offset: usize,
}

pub struct FabushiCoreTargetClient {
    base_url: Url,
    credential_provider: Arc<dyn FabushiCoreCredentialProvider>,
    device_secret_provider: Arc<dyn RemoteComputerDeviceSecretProvider>,
    http: Client,
    sensitive_results: Mutex<SensitiveResultVault>,
    sensitive_inputs: SensitiveInputChallengeOwner,
}

impl std::fmt::Debug for FabushiCoreTargetClient {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FabushiCoreTargetClient")
            .field("base_url", &self.base_url)
            .field("credential_provider", &"<first-party>")
            .field("device_secret_provider", &"<redacted>")
            .finish_non_exhaustive()
    }
}

impl FabushiCoreTargetClient {
    pub fn new(
        base_url: &str,
        credential_provider: Arc<dyn FabushiCoreCredentialProvider>,
        device_secret_provider: Arc<dyn RemoteComputerDeviceSecretProvider>,
    ) -> Result<Self, RemoteComputerTargetError> {
        let mut base_url = Url::parse(base_url).map_err(|_| RemoteComputerTargetError::AuthorityUnavailable)?;
        if !base_url.username().is_empty()
            || base_url.password().is_some()
            || base_url.query().is_some()
            || base_url.fragment().is_some()
        {
            return Err(RemoteComputerTargetError::AuthorityUnavailable);
        }
        let loopback = matches!(base_url.host_str(), Some("localhost" | "127.0.0.1" | "::1"));
        if base_url.scheme() != "https" && !(loopback && base_url.scheme() == "http") {
            return Err(RemoteComputerTargetError::AuthorityUnavailable);
        }
        base_url.set_path("/");
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(CORE_REQUEST_TIMEOUT_SECS))
            .build()
            .map_err(|_| RemoteComputerTargetError::AuthorityUnavailable)?;
        Ok(Self {
            base_url,
            credential_provider,
            device_secret_provider,
            http,
            sensitive_results: Mutex::new(SensitiveResultVault::default()),
            sensitive_inputs: SensitiveInputChallengeOwner::default(),
        })
    }

    fn authority(&self) -> Result<(FabushiCoreCredential, String), RemoteComputerTargetError> {
        let credential = self
            .credential_provider
            .get_fabushi_core_credential()
            .map_err(|_| RemoteComputerTargetError::AuthorityUnavailable)?;
        let device_secret = self.device_secret_provider.get_device_secret(&credential)?;
        if device_secret.len() < 48 || device_secret.len() > 256 {
            return Err(RemoteComputerTargetError::AuthorityUnavailable);
        }
        Ok((credential, device_secret))
    }

    fn endpoint(&self, segments: &[&str]) -> Result<Url, RemoteComputerTargetError> {
        let mut url = self.base_url.clone();
        {
            let mut path = url
                .path_segments_mut()
                .map_err(|_| RemoteComputerTargetError::AuthorityUnavailable)?;
            path.clear();
            for segment in segments {
                path.push(segment);
            }
        }
        Ok(url)
    }

    fn post_json(
        &self,
        credential: &FabushiCoreCredential,
        url: Url,
        body: Value,
    ) -> Result<Value, RemoteComputerTargetError> {
        let response = self
            .http
            .post(url)
            .bearer_auth(credential.access_token())
            .header("Accept", "application/json")
            .json(&body)
            .send()
            .map_err(|_| RemoteComputerTargetError::RequestFailed)?;
        if !response.status().is_success() {
            return Err(RemoteComputerTargetError::RequestFailed);
        }
        let bytes = response
            .bytes()
            .map_err(|_| RemoteComputerTargetError::RequestFailed)?;
        if bytes.len() > MAX_CORE_RESPONSE_BYTES {
            return Err(RemoteComputerTargetError::InvalidResponse);
        }
        serde_json::from_slice(&bytes).map_err(|_| RemoteComputerTargetError::InvalidResponse)
    }

    pub fn network_ingress(
        &self,
        service_id: &str,
    ) -> Result<RemoteNetworkIngress, RemoteComputerTargetError> {
        if service_id != "official-device-mcp" {
            return Err(RemoteComputerTargetError::InvalidResponse);
        }
        let (credential, device_secret) = self.authority()?;
        let response = self.post_json(
            &credential,
            self.endpoint(&["v1", "computers", "network-ingress"])?,
            json!({
                "deviceId": credential.device_id,
                "deviceSecret": device_secret,
                "serviceId": service_id,
            }),
        )?;
        if response.get("deviceId").and_then(Value::as_str)
            != Some(credential.device_id.as_str())
            || response.get("serviceId").and_then(Value::as_str) != Some(service_id)
            || response.get("active").and_then(Value::as_bool) != Some(true)
        {
            return Err(RemoteComputerTargetError::IdentityMismatch);
        }
        let cell_id = required_text(&response, "cellId")?;
        let node_id = required_text(&response, "nodeId")?;
        if !is_fabushi_tailnet_peer(&node_id) {
            return Err(RemoteComputerTargetError::IdentityMismatch);
        }
        let target_generation = response
            .get("targetGeneration")
            .and_then(Value::as_i64)
            .filter(|generation| *generation > 0)
            .ok_or(RemoteComputerTargetError::InvalidResponse)?;
        let ingress_generation = response
            .get("ingressGeneration")
            .and_then(Value::as_i64)
            .filter(|generation| *generation > 0)
            .ok_or(RemoteComputerTargetError::InvalidResponse)?;
        let now = unix_seconds()?;
        let expires_at = response
            .get("expiresAt")
            .and_then(Value::as_i64)
            .filter(|expires_at| *expires_at > now)
            .ok_or(RemoteComputerTargetError::InvalidResponse)?;
        Ok(RemoteNetworkIngress {
            device_id: credential.device_id,
            cell_id,
            target_generation,
            service_id: service_id.to_string(),
            node_id,
            ingress_generation,
            expires_at,
            active: true,
        })
    }

    pub fn grant_service_client(
        &self,
        connection_id: &str,
        delegation_id: &str,
        request_id: &str,
        label: &str,
    ) -> Result<RemoteServiceClientGrant, RemoteComputerTargetError> {
        if connection_id.trim().is_empty()
            || delegation_id.trim().is_empty()
            || request_id.trim().is_empty()
            || label.trim().is_empty()
        {
            return Err(RemoteComputerTargetError::InvalidResponse);
        }
        let (credential, device_secret) = self.authority()?;
        let response = self.post_json(
            &credential,
            self.endpoint(&["v1", "computers", &credential.device_id, "sessions", "grant"])?,
            json!({
                "deviceSecret": device_secret,
                "connectionId": connection_id,
                "delegationId": delegation_id,
                "requestId": request_id,
                "label": label
            }),
        )?;
        if response.get("deviceId").and_then(Value::as_str) != Some(credential.device_id.as_str())
            || response.get("connectionId").and_then(Value::as_str) != Some(connection_id)
            || response.get("delegationId").and_then(Value::as_str) != Some(delegation_id)
            || response.get("requestId").and_then(Value::as_str) != Some(request_id)
            || response.get("serviceBound").and_then(Value::as_bool) != Some(true)
            || response.get("targetGranted").and_then(Value::as_bool) != Some(true)
        {
            return Err(RemoteComputerTargetError::IdentityMismatch);
        }
        let client_id = required_text(&response, "clientId")?;
        let client_token = required_text(&response, "clientToken")?;
        if client_token.len() < 48 || client_token.len() > 256 {
            return Err(RemoteComputerTargetError::InvalidResponse);
        }
        Ok(RemoteServiceClientGrant {
            device_id: credential.device_id,
            client_id,
            client_token,
            connection_id: connection_id.to_string(),
            delegation_id: delegation_id.to_string(),
            request_id: request_id.to_string(),
            reconciled: response.get("reconciled").and_then(Value::as_bool).unwrap_or(false),
        })
    }

    pub fn activate_session(&self, session_id: &str) -> Result<Value, RemoteComputerTargetError> {
        let (credential, device_secret) = self.authority()?;
        let response = self.post_json(
            &credential,
            self.endpoint(&["v1", "computers", &credential.device_id, "sessions", session_id, "activate"])?,
            json!({"deviceSecret": device_secret}),
        )?;
        if response.get("sessionId").and_then(Value::as_str) != Some(session_id)
            || response.get("state").and_then(Value::as_str) != Some("active")
        {
            return Err(RemoteComputerTargetError::IdentityMismatch);
        }
        Ok(response)
    }

    pub fn grant_tool(
        &self,
        session_id: &str,
        client_id: &str,
        tool_name: &str,
        expires_at: i64,
    ) -> Result<i64, RemoteComputerTargetError> {
        let (credential, device_secret) = self.authority()?;
        let response = self.post_json(
            &credential,
            self.endpoint(&["v1", "computers", &credential.device_id, "sessions", session_id, "tool-grants", "grant"])?,
            json!({
                "deviceSecret": device_secret,
                "clientId": client_id,
                "toolName": tool_name,
                "expiresAt": expires_at
            }),
        )?;
        if response.get("toolName").and_then(Value::as_str) != Some(tool_name)
            || response.get("revoked").and_then(Value::as_bool) == Some(true)
        {
            return Err(RemoteComputerTargetError::IdentityMismatch);
        }
        response
            .get("generation")
            .and_then(Value::as_i64)
            .filter(|generation| *generation > 0)
            .ok_or(RemoteComputerTargetError::InvalidResponse)
    }

    pub fn revoke_tool(
        &self,
        session_id: &str,
        client_id: &str,
        tool_name: &str,
    ) -> Result<bool, RemoteComputerTargetError> {
        let (credential, device_secret) = self.authority()?;
        let response = self.post_json(
            &credential,
            self.endpoint(&["v1", "computers", &credential.device_id, "sessions", session_id, "tool-grants", "revoke"])?,
            json!({
                "deviceSecret": device_secret,
                "clientId": client_id,
                "toolName": tool_name
            }),
        )?;
        if response.get("toolName").and_then(Value::as_str) != Some(tool_name) {
            return Err(RemoteComputerTargetError::IdentityMismatch);
        }
        response
            .get("revoked")
            .and_then(Value::as_bool)
            .ok_or(RemoteComputerTargetError::InvalidResponse)
    }

    pub fn validate_sensitive_result_access(
        &self,
        operation: &ClaimedRemoteOperation,
        expected_result_sha256: &str,
    ) -> Result<(), RemoteComputerTargetError> {
        let (credential, device_secret) = self.authority()?;
        let response = self.post_json(
            &credential,
            self.endpoint(&[
                "v1", "computers", &credential.device_id, "sessions", &operation.session_id,
                "operations", &operation.request_id, "access",
            ])?,
            json!({"deviceSecret": device_secret}),
        )?;
        if response.get("allowed").and_then(Value::as_bool) != Some(true)
            || response.get("deviceId").and_then(Value::as_str) != Some(credential.device_id.as_str())
            || response.get("sessionId").and_then(Value::as_str) != Some(operation.session_id.as_str())
            || response.get("clientId").and_then(Value::as_str) != Some(operation.client_id.as_str())
            || response.get("requestId").and_then(Value::as_str) != Some(operation.request_id.as_str())
            || response.get("toolName").and_then(Value::as_str) != Some(operation.tool_name.as_str())
            || response.get("requestSha256").and_then(Value::as_str) != Some(operation.request_sha256.as_str())
            || response.get("grantGeneration").and_then(Value::as_i64) != Some(operation.grant_generation)
            || response.get("sensitiveResult").and_then(Value::as_bool) != Some(true)
            || response.get("state").and_then(Value::as_str) != Some("completed")
            || response.get("resultSha256").and_then(Value::as_str) != Some(expected_result_sha256)
        {
            return Err(RemoteComputerTargetError::IdentityMismatch);
        }
        Ok(())
    }

    fn sensitive_entry_matches(
        entry: &SensitiveResultEntry,
        current_account_user_id: &str,
        current_device_id: &str,
        device_id: &str,
        network_generation: i64,
        session_id: &str,
        client_id: &str,
        request_id: &str,
        request_sha256: &str,
        grant_generation: i64,
    ) -> bool {
        entry.account_user_id == current_account_user_id
            && entry.device_id == current_device_id
            && entry.device_id == device_id
            && entry.network_generation == network_generation
            && entry.session_id == session_id
            && entry.client_id == client_id
            && entry.request_id == request_id
            && entry.request_sha256 == request_sha256
            && entry.grant_generation == grant_generation
            && entry.tool_name == "Computer"
    }

    fn validate_sensitive_entry(
        &self,
        entry: &SensitiveResultEntry,
        device_id: &str,
        network_generation: i64,
        session_id: &str,
        client_id: &str,
        request_id: &str,
        request_sha256: &str,
        grant_generation: i64,
    ) -> Result<ClaimedRemoteOperation, RemoteComputerTargetError> {
        let (credential, _) = self.authority()?;
        let credential_bound = entry.account_user_id == credential.user_id
            && entry.device_id == credential.device_id;
        if !credential_bound || !Self::sensitive_entry_matches(
            entry,
            &credential.user_id,
            &credential.device_id,
            device_id,
            network_generation,
            session_id,
            client_id,
            request_id,
            request_sha256,
            grant_generation,
        ) || entry.expires_at <= unix_seconds()?
        {
            return Err(RemoteComputerTargetError::IdentityMismatch);
        }
        self.require_current_network_binding(device_id, network_generation)?;
        let operation = ClaimedRemoteOperation {
            request_id: entry.request_id.clone(),
            session_id: entry.session_id.clone(),
            client_id: entry.client_id.clone(),
            tool_name: entry.tool_name.clone(),
            request_sha256: entry.request_sha256.clone(),
            grant_generation: entry.grant_generation,
            sensitive_result: true,
        };
        self.validate_sensitive_result_access(&operation, &entry.result_sha256)?;
        Ok(operation)
    }

    pub fn claim_operation(
        &self,
        session_id: &str,
        request_id: &str,
        expected_tool_name: &str,
        expected_request_sha256: &str,
    ) -> Result<ClaimedRemoteOperation, RemoteComputerTargetError> {
        let (credential, device_secret) = self.authority()?;
        let response = self.post_json(
            &credential,
            self.endpoint(&[
                "v1", "computers", &credential.device_id, "sessions", session_id,
                "operations", request_id, "claim",
            ])?,
            json!({"deviceSecret": device_secret}),
        )?;
        if response.get("requestId").and_then(Value::as_str) != Some(request_id)
            || response.get("sessionId").and_then(Value::as_str) != Some(session_id)
            || response.get("toolName").and_then(Value::as_str) != Some(expected_tool_name)
            || response.get("requestSha256").and_then(Value::as_str) != Some(expected_request_sha256)
            || response.get("state").and_then(Value::as_str) != Some("running")
        {
            return Err(RemoteComputerTargetError::IdentityMismatch);
        }
        let client_id = response
            .get("clientId")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or(RemoteComputerTargetError::InvalidResponse)?;
        let grant_generation = response
            .get("grantGeneration")
            .and_then(Value::as_i64)
            .filter(|generation| *generation > 0)
            .ok_or(RemoteComputerTargetError::InvalidResponse)?;
        Ok(ClaimedRemoteOperation {
            request_id: request_id.to_string(),
            session_id: session_id.to_string(),
            client_id: client_id.to_string(),
            tool_name: expected_tool_name.to_string(),
            request_sha256: expected_request_sha256.to_string(),
            grant_generation,
            sensitive_result: response.get("sensitiveResult").and_then(Value::as_bool).unwrap_or(false),
        })
    }

    pub fn publish_result(
        &self,
        operation: &ClaimedRemoteOperation,
        status: &str,
        result_sha256: &str,
        result: Option<Value>,
    ) -> Result<Value, RemoteComputerTargetError> {
        if !matches!(status, "completed" | "failed") {
            return Err(RemoteComputerTargetError::InvalidResponse);
        }
        if operation.sensitive_result && result.is_some() {
            return Err(RemoteComputerTargetError::InvalidResponse);
        }
        let (credential, device_secret) = self.authority()?;
        let response = self.post_json(
            &credential,
            self.endpoint(&[
                "v1", "computers", &credential.device_id, "sessions", &operation.session_id,
                "operations", &operation.request_id, "result",
            ])?,
            json!({
                "deviceSecret": device_secret,
                "status": status,
                "resultSha256": result_sha256,
                "result": result
            }),
        )?;
        if response.get("requestId").and_then(Value::as_str) != Some(operation.request_id.as_str())
            || response.get("sessionId").and_then(Value::as_str) != Some(operation.session_id.as_str())
            || response.get("requestSha256").and_then(Value::as_str) != Some(operation.request_sha256.as_str())
            || response.get("toolName").and_then(Value::as_str) != Some(operation.tool_name.as_str())
            || response.get("state").and_then(Value::as_str) != Some(status)
            || response.get("resultSha256").and_then(Value::as_str) != Some(result_sha256)
        {
            return Err(RemoteComputerTargetError::IdentityMismatch);
        }
        Ok(response)
    }



    pub fn serve_private_request(
        &self,
        worker: &mut OwnedNetworkWorker,
        expected_peer: &str,
        local_port: u16,
        catalog: &dyn RemoteComputerToolCatalog,
        authorizer: &dyn RemoteComputerToolAuthorizer,
        executor: &dyn RemoteComputerNativeExecutor,
    ) -> Result<(), RemoteComputerTargetError> {
        if expected_peer.trim().is_empty()
            || !is_fabushi_tailnet_peer(expected_peer)
            || local_port == 0
        {
            return Err(RemoteComputerTargetError::TransportFailed);
        }
        let stream_id = worker
            .stream_listen(expected_peer, local_port, REMOTE_TARGET_STREAM_TIMEOUT_MS)
            .map_err(|_| RemoteComputerTargetError::TransportFailed)?;
        let result = self.serve_private_stream(worker, &stream_id, catalog, authorizer, executor);
        let close_result = worker.stream_close(&stream_id);
        match (result, close_result) {
            (Err(error), _) => Err(error),
            (Ok(()), Ok(())) => Ok(()),
            (Ok(()), Err(_)) => Err(RemoteComputerTargetError::TransportFailed),
        }
    }

    pub fn serve_private_request_with_timeout(
        &self,
        worker: &mut OwnedNetworkWorker,
        expected_peer: &str,
        local_port: u16,
        timeout_ms: u64,
        catalog: &dyn RemoteComputerToolCatalog,
        authorizer: &dyn RemoteComputerToolAuthorizer,
        executor: &dyn RemoteComputerNativeExecutor,
    ) -> Result<(), RemoteComputerTargetError> {
        if expected_peer.trim().is_empty()
            || !is_fabushi_tailnet_peer(expected_peer)
            || local_port == 0
            || timeout_ms == 0
            || timeout_ms > REMOTE_TARGET_STREAM_TIMEOUT_MS
        {
            return Err(RemoteComputerTargetError::TransportFailed);
        }
        let stream_id = worker
            .stream_listen(expected_peer, local_port, timeout_ms)
            .map_err(|_| RemoteComputerTargetError::TransportFailed)?;
        let result = self.serve_private_stream(worker, &stream_id, catalog, authorizer, executor);
        let close_result = worker.stream_close(&stream_id);
        match (result, close_result) {
            (Err(error), _) => Err(error),
            (Ok(()), Ok(())) => Ok(()),
            (Ok(()), Err(_)) => Err(RemoteComputerTargetError::TransportFailed),
        }
    }

    fn serve_private_stream(
        &self,
        worker: &mut OwnedNetworkWorker,
        stream_id: &str,
        catalog: &dyn RemoteComputerToolCatalog,
        authorizer: &dyn RemoteComputerToolAuthorizer,
        executor: &dyn RemoteComputerNativeExecutor,
    ) -> Result<(), RemoteComputerTargetError> {
        let frame = read_private_frame(worker, stream_id)?;
        let request: RemoteTargetPrivateRequest =
            serde_json::from_slice(&frame).map_err(|_| RemoteComputerTargetError::InvalidResponse)?;
        let mut sink = NetworkWorkerResponseSink { worker, stream_id };
        self.handle_private_request(request, catalog, authorizer, executor, &mut sink, None)
    }

    fn handle_private_request(
        &self,
        request: RemoteTargetPrivateRequest,
        catalog: &dyn RemoteComputerToolCatalog,
        authorizer: &dyn RemoteComputerToolAuthorizer,
        executor: &dyn RemoteComputerNativeExecutor,
        sink: &mut dyn RemoteTargetResponseSink,
        cancellation: Option<&PrivateRequestState>,
    ) -> Result<(), RemoteComputerTargetError> {
        if cancellation.is_some_and(PrivateRequestState::cancelled_before_dispatch) {
            return Err(RemoteComputerTargetError::RequestFailed);
        }

        match request {
            RemoteTargetPrivateRequest::GrantController {
                device_id,
                network_generation,
                connection_id,
                delegation_id,
                request_id,
                label,
            } => {
                self.require_current_network_binding(&device_id, network_generation)?;
                let grant =
                    self.grant_service_client(&connection_id, &delegation_id, &request_id, &label)?;
                sink.send_response(&json!({
                        "type": "controllerGranted",
                        "deviceId": grant.device_id,
                        "clientId": grant.client_id,
                        "clientToken": grant.client_token(),
                        "connectionId": grant.connection_id,
                        "delegationId": grant.delegation_id,
                        "requestId": grant.request_id,
                        "reconciled": grant.reconciled,
                        "networkGeneration": network_generation,
                    }),
                )
            }
            RemoteTargetPrivateRequest::DescribeTool {
                device_id,
                network_generation,
                connection_id,
                delegation_id,
                request_id,
                label,
                tool_name,
            } => {
                self.require_current_network_binding(&device_id, network_generation)?;
                let grant =
                    self.grant_service_client(&connection_id, &delegation_id, &request_id, &label)?;
                let tool = catalog.describe_tool(&tool_name)?;
                if let Some(descriptor) = tool.as_ref() {
                    if descriptor.get("name").and_then(Value::as_str) != Some(tool_name.as_str()) {
                        return Err(RemoteComputerTargetError::IdentityMismatch);
                    }
                }
                let schema_version = tool
                    .as_ref()
                    .map(tool_descriptor_sha256)
                    .transpose()?
                    .unwrap_or_default();
                sink.send_response(&json!({
                        "type": "toolDescription",
                        "deviceId": device_id,
                        "clientId": grant.client_id,
                        "connectionId": connection_id,
                        "delegationId": delegation_id,
                        "requestId": request_id,
                        "toolName": tool_name,
                        "available": tool.is_some(),
                        "schemaVersion": schema_version,
                        "tool": tool,
                        "networkGeneration": network_generation,
                    }),
                )
            }
            RemoteTargetPrivateRequest::AuthorizeTool {
                device_id,
                network_generation,
                session_id,
                client_id,
                tool_name,
                expires_at,
            } => {
                self.require_current_network_binding(&device_id, network_generation)?;
                let authorized_expires_at =
                    authorizer.authorize_tool(&session_id, &client_id, &tool_name, expires_at)?;
                let now = unix_seconds()?;
                if authorized_expires_at <= now || authorized_expires_at > expires_at {
                    return Err(RemoteComputerTargetError::AuthorityUnavailable);
                }
                self.activate_session(&session_id)?;
                let generation =
                    self.grant_tool(&session_id, &client_id, &tool_name, authorized_expires_at)?;
                sink.send_response(&json!({
                        "type": "toolAuthorized",
                        "deviceId": device_id,
                        "sessionId": session_id,
                        "clientId": client_id,
                        "toolName": tool_name,
                        "expiresAt": authorized_expires_at,
                        "grantGeneration": generation,
                        "networkGeneration": network_generation,
                    }),
                )
            }
            RemoteTargetPrivateRequest::Operation {
                device_id,
                network_generation,
                session_id,
                client_id,
                request_id,
                tool_name,
                request_sha256,
                grant_generation,
                arguments,
                disk_preflight,
            } => {
                self.require_current_network_binding(&device_id, network_generation)?;
                let locally_computed_sha256 =
                    operation_request_sha256(&tool_name, &arguments, &disk_preflight)?;
                if locally_computed_sha256 != request_sha256 {
                    return Err(RemoteComputerTargetError::IdentityMismatch);
                }
                let claimed =
                    self.claim_operation(&session_id, &request_id, &tool_name, &request_sha256)?;
                if claimed.client_id != client_id || claimed.grant_generation != grant_generation {
                    return Err(RemoteComputerTargetError::IdentityMismatch);
                }

                // Fence account switches, network revoke and generation rollover again after
                // the Core claim but before any native side effect is dispatched.
                self.require_current_network_binding(&device_id, network_generation)?;

                if let Some(cancellation) = cancellation {
                    if !cancellation.begin_dispatch() {
                        return self.finish_native_operation(
                            sink,
                            &claimed,
                            &device_id,
                            network_generation,
                            "failed",
                            json!({"code":"cancelled_before_dispatch","retryable":false}),
                        );
                    }
                }

                match executor.execute(&claimed, &arguments, &disk_preflight) {
                    RemoteComputerNativeOutcome::OutcomeUnknown => sink.send_response(&json!({
                            "type": "operationResult",
                            "deviceId": device_id,
                            "sessionId": session_id,
                            "requestId": request_id,
                            "requestSha256": request_sha256,
                            "status": "outcome_unknown",
                            "networkGeneration": network_generation,
                        }),
                    ),
                    RemoteComputerNativeOutcome::SensitiveCompleted(result) => self.finish_sensitive_operation(
                    sink,
                    &claimed,
                        &device_id,
                        network_generation,
                        result,
                    ),
                    RemoteComputerNativeOutcome::Completed(result) => self.finish_native_operation(
                    sink,
                    &claimed,
                        &device_id,
                        network_generation,
                        "completed",
                        result,
                    ),
                    RemoteComputerNativeOutcome::Failed(result) => self.finish_native_operation(
                    sink,
                    &claimed,
                        &device_id,
                        network_generation,
                        "failed",
                        result,
                    ),
                }
            }
            RemoteTargetPrivateRequest::SensitiveInputIssue {
                device_id,
                network_generation,
                session_id,
                client_id,
                request_id,
                request_sha256,
                grant_generation,
                ttl_ms,
            } => {
                if !mahayana_computer::control_policy().remote_control_enabled {
                    return Err(RemoteComputerTargetError::AuthorityUnavailable);
                }
                self.require_current_network_binding(&device_id, network_generation)?;
                if ttl_ms <= 0 || ttl_ms > SENSITIVE_INPUT_CHALLENGE_MAX_TTL_MS {
                    return Err(RemoteComputerTargetError::InvalidResponse);
                }
                let claimed =
                    self.claim_operation(&session_id, &request_id, "Computer", &request_sha256)?;
                if claimed.client_id != client_id || claimed.grant_generation != grant_generation {
                    return Err(RemoteComputerTargetError::IdentityMismatch);
                }
                self.require_current_network_binding(&device_id, network_generation)?;
                let (credential, _) = self.authority()?;
                if credential.device_id != device_id {
                    return Err(RemoteComputerTargetError::IdentityMismatch);
                }
                let binding = SensitiveInputBinding {
                    account_id: credential.user_id,
                    device_id: device_id.clone(),
                    client_id: client_id.clone(),
                    session_id: session_id.clone(),
                    operation_id: request_id.clone(),
                    request_sha256: request_sha256.clone(),
                    network_generation,
                    authorization_generation: grant_generation,
                };
                let challenge = self
                    .sensitive_inputs
                    .issue(binding, unix_millis()?, ttl_ms)
                    .map_err(|_| RemoteComputerTargetError::AuthorityUnavailable)?;
                sink.send_response(&json!({
                    "type": "sensitiveInputChallenge",
                    "deviceId": device_id,
                    "sessionId": session_id,
                    "clientId": client_id,
                    "requestId": request_id,
                    "requestSha256": request_sha256,
                    "grantGeneration": grant_generation,
                    "challengeId": challenge.challenge_id,
                    "expiresAtMs": challenge.expires_at_ms,
                    "networkGeneration": network_generation,
                }))
            }
            RemoteTargetPrivateRequest::SensitiveInputSubmit {
                device_id,
                network_generation,
                session_id,
                client_id,
                request_id,
                request_sha256,
                grant_generation,
                challenge_id,
                snapshot_generation,
                payload_base64,
            } => {
                if !mahayana_computer::control_policy().remote_control_enabled {
                    return Err(RemoteComputerTargetError::AuthorityUnavailable);
                }
                self.require_current_network_binding(&device_id, network_generation)?;
                let claimed =
                    self.claim_operation(&session_id, &request_id, "Computer", &request_sha256)?;
                if claimed.client_id != client_id || claimed.grant_generation != grant_generation {
                    return Err(RemoteComputerTargetError::IdentityMismatch);
                }
                self.require_current_network_binding(&device_id, network_generation)?;
                let (credential, _) = self.authority()?;
                if credential.device_id != device_id {
                    return Err(RemoteComputerTargetError::IdentityMismatch);
                }
                let binding = SensitiveInputBinding {
                    account_id: credential.user_id,
                    device_id: device_id.clone(),
                    client_id: client_id.clone(),
                    session_id: session_id.clone(),
                    operation_id: request_id.clone(),
                    request_sha256: request_sha256.clone(),
                    network_generation,
                    authorization_generation: grant_generation,
                };
                let mut encoded = payload_base64.into_bytes();
                let decoded = base64::engine::general_purpose::STANDARD.decode(&encoded);
                encoded.fill(0);
                let payload = decoded.map_err(|_| RemoteComputerTargetError::InvalidResponse)?;
                if let Some(cancellation) = cancellation {
                    if !cancellation.begin_dispatch() {
                        return self.finish_native_operation(
                            sink,
                            &claimed,
                            &device_id,
                            network_generation,
                            "failed",
                            json!({"code":"cancelled_before_dispatch","retryable":false}),
                        );
                    }
                }
                let delivered = self
                    .sensitive_inputs
                    .consume(
                        &challenge_id,
                        &binding,
                        unix_millis()?,
                        payload,
                        |secret| {
                            self.require_current_network_binding(&device_id, network_generation)?;
                            let bytes = std::mem::take(secret);
                            let text = match String::from_utf8(bytes) {
                                Ok(text) => text,
                                Err(error) => {
                                    let mut bytes = error.into_bytes();
                                    bytes.fill(0);
                                    return Err(RemoteComputerTargetError::InvalidResponse);
                                }
                            };
                            let mut action = ComputerAction {
                                action: ComputerActionKind::Type,
                                x: None,
                                y: None,
                                x2: None,
                                y2: None,
                                path: None,
                                text: Some(text),
                                key: None,
                                button: None,
                                click_count: None,
                                direction: None,
                                amount: None,
                                wait_ms: None,
                                description: Some("user-provided sensitive input".into()),
                            };
                            let lease = mahayana_computer::ComputerControlLeaseRequest::new(
                                format!("official-device-mcp:{}", client_id),
                                session_id.clone(),
                                "local-desktop",
                                ComputerControlOrigin::RemoteMobile,
                                "remote-human",
                            );
                            let executed = mahayana_computer::execute_with_lease_fenced(
                                std::slice::from_ref(&action),
                                ComputerControlOrigin::RemoteMobile,
                                &lease,
                                Some(snapshot_generation),
                            );
                            if let Some(text) = action.text.take() {
                                let mut bytes = text.into_bytes();
                                bytes.fill(0);
                            }
                            match executed {
                                Ok(observed) => Ok(json!({
                                    "content": "Sensitive input was delivered to the authorized target desktop.",
                                    "sensitiveInputDelivered": true,
                                    "snapshotGeneration": observed.snapshot_generation,
                                })),
                                Err(mahayana_computer::ComputerError::SnapshotStale { expected, current }) => Ok(json!({
                                    "content": "Sensitive input was not delivered because the target observation is stale.",
                                    "sensitiveInputDelivered": false,
                                    "code": "snapshot_stale",
                                    "expectedSnapshotGeneration": expected,
                                    "currentSnapshotGeneration": current,
                                })),
                                Err(_) => Ok(json!({
                                    "content": "Sensitive input delivery failed on the authorized target desktop.",
                                    "sensitiveInputDelivered": false,
                                    "code": "sensitive_input_delivery_failed",
                                })),
                            }
                        },
                    )
                    .map_err(|_| RemoteComputerTargetError::IdentityMismatch)??;
                let status = if delivered
                    .get("sensitiveInputDelivered")
                    .and_then(Value::as_bool)
                    == Some(true)
                {
                    "completed"
                } else {
                    "failed"
                };
                self.finish_native_operation(
                    sink,
                    &claimed,
                    &device_id,
                    network_generation,
                    status,
                    delivered,
                )
            }
            RemoteTargetPrivateRequest::SensitiveResultResume {
                device_id,
                network_generation,
                session_id,
                client_id,
                request_id,
                request_sha256,
                grant_generation,
            } => {
                let access = self.resume_sensitive_result(
                    &device_id,
                    network_generation,
                    &session_id,
                    &client_id,
                    &request_id,
                    &request_sha256,
                    grant_generation,
                )?;
                sink.send_response(&json!({
                    "type": "sensitiveResultReady",
                    "deviceId": device_id,
                    "sessionId": session_id,
                    "clientId": client_id,
                    "requestId": request_id,
                    "requestSha256": request_sha256,
                    "grantGeneration": grant_generation,
                    "resultRef": access.result_ref,
                    "resultSha256": access.result_sha256,
                    "mediaType": access.media_type,
                    "capturedAtMs": access.captured_at_ms,
                    "snapshotGeneration": access.snapshot_generation,
                    "width": access.width,
                    "height": access.height,
                    "expiresAt": access.expires_at,
                    "nextOffset": access.next_offset,
                    "networkGeneration": network_generation,
                }))
            }
            RemoteTargetPrivateRequest::SensitiveResultChunk {
                device_id,
                network_generation,
                session_id,
                client_id,
                request_id,
                request_sha256,
                grant_generation,
                result_ref,
                offset,
            } => {
                let chunk = self.read_sensitive_result_chunk(
                    &device_id,
                    network_generation,
                    &session_id,
                    &client_id,
                    &request_id,
                    &request_sha256,
                    grant_generation,
                    &result_ref,
                    offset,
                )?;
                sink.send_response(&chunk)
            }
            RemoteTargetPrivateRequest::SensitiveResultRevoke {
                device_id,
                network_generation,
                session_id,
                client_id,
                request_id,
                request_sha256,
                grant_generation,
                result_ref,
            } => {
                self.revoke_sensitive_result(
                    &device_id,
                    network_generation,
                    &session_id,
                    &client_id,
                    &request_id,
                    &request_sha256,
                    grant_generation,
                    &result_ref,
                )?;
                sink.send_response(&json!({
                    "type": "sensitiveResultRevoked",
                    "requestId": request_id,
                    "resultRef": result_ref,
                }))
            }
        }
    }

    fn handle_private_payload(
        &self,
        payload: &Value,
        expected_account_id: &str,
        expected_device_id: &str,
        expected_network_generation: i64,
        catalog: &dyn RemoteComputerToolCatalog,
        authorizer: &dyn RemoteComputerToolAuthorizer,
        executor: &dyn RemoteComputerNativeExecutor,
        cancellation: &PrivateRequestState,
    ) -> Result<Value, RemoteComputerTargetError> {
        validate_private_ws_payload(payload, expected_device_id, expected_network_generation)?;
        let (credential, _) = self.authority()?;
        if credential.user_id != expected_account_id || credential.device_id != expected_device_id {
            return Err(RemoteComputerTargetError::IdentityMismatch);
        }
        self.require_current_network_binding(expected_device_id, expected_network_generation)?;
        let request: RemoteTargetPrivateRequest = serde_json::from_value(payload.clone())
            .map_err(|_| RemoteComputerTargetError::InvalidResponse)?;
        let mut sink = CapturedRemoteTargetResponseSink::default();
        self.handle_private_request(request, catalog, authorizer, executor, &mut sink, Some(cancellation))?;
        let response = sink.response.ok_or(RemoteComputerTargetError::InvalidResponse)?;
        correlate_private_ws_response(response, payload, expected_device_id, expected_network_generation)
    }

    fn finish_sensitive_operation(
        &self,
        sink: &mut dyn RemoteTargetResponseSink,
        claimed: &ClaimedRemoteOperation,
        device_id: &str,
        network_generation: i64,
        result: RemoteComputerSensitiveResult,
    ) -> Result<(), RemoteComputerTargetError> {
        if !claimed.sensitive_result || result.bytes.is_empty() || result.bytes.len() > REMOTE_SENSITIVE_RESULT_MAX_BYTES {
            return Err(RemoteComputerTargetError::InvalidResponse);
        }
        let (credential, _) = self.authority()?;
        if credential.device_id != device_id {
            return Err(RemoteComputerTargetError::IdentityMismatch);
        }
        self.require_current_network_binding(device_id, network_generation)?;
        let now = unix_seconds()?;
        let result_sha256 = sha256_hex(&result.bytes);
        let snapshot_generation = result.snapshot_generation;
        let result_ref = format!("sensitive-result-{}", Uuid::new_v4());
        let expires_at = now.saturating_add(REMOTE_SENSITIVE_RESULT_TTL_SECONDS);
        let entry = SensitiveResultEntry {
            account_user_id: credential.user_id,
            device_id: device_id.to_string(),
            network_generation,
            session_id: claimed.session_id.clone(),
            client_id: claimed.client_id.clone(),
            request_id: claimed.request_id.clone(),
            request_sha256: claimed.request_sha256.clone(),
            tool_name: claimed.tool_name.clone(),
            grant_generation: claimed.grant_generation,
            result_sha256: result_sha256.clone(),
            media_type: result.media_type,
            bytes: result.bytes,
            captured_at_ms: result.captured_at_ms,
            snapshot_generation,
            width: result.width,
            height: result.height,
            expires_at,
            next_offset: 0,
        };
        {
            let mut vault = self.sensitive_results
                .lock()
                .map_err(|_| RemoteComputerTargetError::OutcomeUnknown)?;
            vault.insert(result_ref.clone(), entry, now);
        }
        if self.publish_result(claimed, "completed", &result_sha256, None).is_err() {
            if let Ok(mut vault) = self.sensitive_results.lock() {
                vault.entries.remove(&result_ref);
            }
            return Err(RemoteComputerTargetError::OutcomeUnknown);
        }
        sink.send_response(&json!({
            "type": "operationResult",
            "deviceId": device_id,
            "sessionId": claimed.session_id,
            "clientId": claimed.client_id,
            "requestId": claimed.request_id,
            "requestSha256": claimed.request_sha256,
            "grantGeneration": claimed.grant_generation,
            "resultSha256": result_sha256,
            "status": "completed",
            "sensitiveResult": true,
            "resultRef": result_ref,
            "snapshotGeneration": snapshot_generation,
            "expiresAt": expires_at,
            "networkGeneration": network_generation,
        }))
    }

    fn resume_sensitive_result(
        &self,
        device_id: &str,
        network_generation: i64,
        session_id: &str,
        client_id: &str,
        request_id: &str,
        request_sha256: &str,
        grant_generation: i64,
    ) -> Result<SensitiveResultAccess, RemoteComputerTargetError> {
        let now = unix_seconds()?;
        let mut vault = self.sensitive_results
            .lock()
            .map_err(|_| RemoteComputerTargetError::AuthorityUnavailable)?;
        vault.purge_expired(now);
        let (result_ref, entry) = vault.entries.iter()
            .find(|(_, entry)| {
                entry.device_id == device_id
                    && entry.network_generation == network_generation
                    && entry.session_id == session_id
                    && entry.client_id == client_id
                    && entry.request_id == request_id
                    && entry.request_sha256 == request_sha256
                    && entry.grant_generation == grant_generation
            })
            .map(|(result_ref, entry)| (result_ref.clone(), entry.clone()))
            .ok_or(RemoteComputerTargetError::IdentityMismatch)?;
        drop(vault);
        self.validate_sensitive_entry(
            &entry, device_id, network_generation, session_id, client_id,
            request_id, request_sha256, grant_generation,
        )?;
        Ok(SensitiveResultAccess {
            result_ref,
            result_sha256: entry.result_sha256,
            media_type: entry.media_type,
            captured_at_ms: entry.captured_at_ms,
            snapshot_generation: entry.snapshot_generation,
            width: entry.width,
            height: entry.height,
            expires_at: entry.expires_at,
            next_offset: entry.next_offset,
        })
    }

    fn read_sensitive_result_chunk(
        &self,
        device_id: &str,
        network_generation: i64,
        session_id: &str,
        client_id: &str,
        request_id: &str,
        request_sha256: &str,
        grant_generation: i64,
        result_ref: &str,
        offset: usize,
    ) -> Result<Value, RemoteComputerTargetError> {
        let now = unix_seconds()?;
        let entry = {
            let mut vault = self.sensitive_results
                .lock()
                .map_err(|_| RemoteComputerTargetError::AuthorityUnavailable)?;
            vault.purge_expired(now);
            vault.entries.get(result_ref).cloned()
                .ok_or(RemoteComputerTargetError::IdentityMismatch)?
        };
        self.validate_sensitive_entry(
            &entry, device_id, network_generation, session_id, client_id,
            request_id, request_sha256, grant_generation,
        )?;
        let (end, done) = sensitive_chunk_range(&entry, offset)?;
        let encoded = base64::engine::general_purpose::STANDARD.encode(&entry.bytes[offset..end]);
        {
            let mut vault = self.sensitive_results
                .lock()
                .map_err(|_| RemoteComputerTargetError::AuthorityUnavailable)?;
            let current = vault.entries.get_mut(result_ref)
                .ok_or(RemoteComputerTargetError::IdentityMismatch)?;
            if current.next_offset != offset {
                return Err(RemoteComputerTargetError::IdentityMismatch);
            }
            if done {
                vault.entries.remove(result_ref);
            } else {
                current.next_offset = end;
            }
        }
        Ok(json!({
            "type": "sensitiveResultChunk",
            "deviceId": device_id,
            "sessionId": session_id,
            "clientId": client_id,
            "requestId": request_id,
            "requestSha256": request_sha256,
            "grantGeneration": grant_generation,
            "resultRef": result_ref,
            "resultSha256": entry.result_sha256,
            "mediaType": entry.media_type,
            "offset": offset,
            "nextOffset": end,
            "done": done,
            "dataBase64": encoded,
            "expiresAt": entry.expires_at,
            "networkGeneration": network_generation,
        }))
    }

    fn revoke_sensitive_result(
        &self,
        device_id: &str,
        network_generation: i64,
        session_id: &str,
        client_id: &str,
        request_id: &str,
        request_sha256: &str,
        grant_generation: i64,
        result_ref: &str,
    ) -> Result<(), RemoteComputerTargetError> {
        let now = unix_seconds()?;
        let entry = {
            let mut vault = self.sensitive_results
                .lock()
                .map_err(|_| RemoteComputerTargetError::AuthorityUnavailable)?;
            vault.purge_expired(now);
            vault.entries.get(result_ref).cloned()
                .ok_or(RemoteComputerTargetError::IdentityMismatch)?
        };
        self.validate_sensitive_entry(
            &entry, device_id, network_generation, session_id, client_id,
            request_id, request_sha256, grant_generation,
        )?;
        let mut vault = self.sensitive_results
            .lock()
            .map_err(|_| RemoteComputerTargetError::AuthorityUnavailable)?;
        vault.entries.remove(result_ref);
        Ok(())
    }

    fn finish_native_operation(
        &self,
        sink: &mut dyn RemoteTargetResponseSink,
        claimed: &ClaimedRemoteOperation,
        device_id: &str,
        network_generation: i64,
        status: &str,
        result: Value,
    ) -> Result<(), RemoteComputerTargetError> {
        let result_bytes =
            serde_json::to_vec(&result).map_err(|_| RemoteComputerTargetError::OutcomeUnknown)?;
        if result_bytes.len() > REMOTE_TARGET_RESULT_MAX_BYTES {
            return Err(RemoteComputerTargetError::OutcomeUnknown);
        }
        let result_sha256 = sha256_hex(&result_bytes);
        let core_result = if claimed.sensitive_result {
            None
        } else {
            Some(result)
        };
        self.publish_result(claimed, status, &result_sha256, core_result)
            .map_err(|_| RemoteComputerTargetError::OutcomeUnknown)?;
        sink.send_response(&json!({
                "type": "operationResult",
                "deviceId": device_id,
                "sessionId": claimed.session_id,
                "requestId": claimed.request_id,
                "requestSha256": claimed.request_sha256,
                "resultSha256": result_sha256,
                "status": status,
                "networkGeneration": network_generation,
            }),
        )
    }

    fn require_current_network_binding(
        &self,
        device_id: &str,
        expected_generation: i64,
    ) -> Result<RemoteNetworkBinding, RemoteComputerTargetError> {
        let binding = self.network_binding()?;
        if binding.device_id != device_id
            || binding.generation != expected_generation
            || !binding.active
            || binding.expires_at <= unix_seconds()?
        {
            return Err(RemoteComputerTargetError::IdentityMismatch);
        }
        Ok(binding)
    }

    pub fn network_binding(&self) -> Result<RemoteNetworkBinding, RemoteComputerTargetError> {
        let (credential, device_secret) = self.authority()?;
        let response = self.post_json(
            &credential,
            self.endpoint(&["v1", "computers", "network-binding"])?,
            json!({
                "deviceId": credential.device_id,
                "deviceSecret": device_secret
            }),
        )?;
        if response.get("deviceId").and_then(Value::as_str) != Some(credential.device_id.as_str()) {
            return Err(RemoteComputerTargetError::IdentityMismatch);
        }
        let generation = response
            .get("generation")
            .and_then(Value::as_i64)
            .filter(|generation| *generation > 0)
            .ok_or(RemoteComputerTargetError::InvalidResponse)?;
        let expires_at = response
            .get("expiresAt")
            .and_then(Value::as_i64)
            .ok_or(RemoteComputerTargetError::InvalidResponse)?;
        Ok(RemoteNetworkBinding {
            device_id: credential.device_id,
            cell_id: required_text(&response, "cellId")?,
            node_id: required_text(&response, "nodeId")?,
            proof_sha256: required_text(&response, "proofSha256")?,
            generation,
            expires_at,
            active: response.get("active").and_then(Value::as_bool).unwrap_or(false),
        })
    }
}


fn unix_millis() -> Result<i64, RemoteComputerTargetError> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| RemoteComputerTargetError::AuthorityUnavailable)?
        .as_millis();
    i64::try_from(millis).map_err(|_| RemoteComputerTargetError::AuthorityUnavailable)
}

fn unix_seconds() -> Result<i64, RemoteComputerTargetError> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| RemoteComputerTargetError::AuthorityUnavailable)?
        .as_secs();
    i64::try_from(seconds).map_err(|_| RemoteComputerTargetError::AuthorityUnavailable)
}

pub struct RemoteComputerTargetNetworkService {
    client: Arc<FabushiCoreTargetClient>,
    catalog: Arc<dyn RemoteComputerToolCatalog>,
    authorizer: Arc<dyn RemoteComputerToolAuthorizer>,
    executor: Arc<dyn RemoteComputerNativeExecutor>,
    local_port: u16,
    ingress_cache: Mutex<Option<(RemoteNetworkIngress, Instant)>>,
    private_ws_url: Option<Url>,
    private_ws_stop: Arc<AtomicBool>,
    private_ws_thread: Mutex<Option<JoinHandle<()>>>,
}

impl RemoteComputerTargetNetworkService {
    pub fn new(
        client: Arc<FabushiCoreTargetClient>,
        catalog: Arc<dyn RemoteComputerToolCatalog>,
        authorizer: Arc<dyn RemoteComputerToolAuthorizer>,
        executor: Arc<dyn RemoteComputerNativeExecutor>,
        local_port: u16,
    ) -> Result<Self, RemoteComputerTargetError> {
        if local_port == 0 {
            return Err(RemoteComputerTargetError::TransportFailed);
        }
        let private_ws_url = std::env::var(FABUSHI_FDN_PRIVATE_WS_URL_ENV)
            .ok()
            .filter(|value| !value.trim().is_empty())
            .and_then(|value| match validate_private_ws_url(&value) {
                Ok(url) => Some(url),
                Err(_) => {
                    eprintln!("[fabushi-fdn] private WebSocket URL rejected; reverse fallback disabled");
                    None
                }
            });
        Ok(Self {
            client,
            catalog,
            authorizer,
            executor,
            local_port,
            ingress_cache: Mutex::new(None),
            private_ws_url,
            private_ws_stop: Arc::new(AtomicBool::new(false)),
            private_ws_thread: Mutex::new(None),
        })
    }

    fn ensure_private_ws_thread(&self) {
        let Some(url) = self.private_ws_url.clone() else { return; };
        let Ok(mut slot) = self.private_ws_thread.lock() else { return; };
        if slot.as_ref().is_some_and(|handle| !handle.is_finished()) { return; }
        if let Some(handle) = slot.take() { let _ = handle.join(); }
        if self.private_ws_stop.load(Ordering::Acquire) { return; }
        let client = Arc::clone(&self.client);
        let catalog = Arc::clone(&self.catalog);
        let authorizer = Arc::clone(&self.authorizer);
        let executor = Arc::clone(&self.executor);
        let stop = Arc::clone(&self.private_ws_stop);
        match thread::Builder::new().name("fabushi-fdn-private-ws".into())
            .spawn(move || run_private_ws_loop(url, client, catalog, authorizer, executor, stop)) {
            Ok(handle) => *slot = Some(handle),
            Err(_) => eprintln!("[fabushi-fdn] private WebSocket supervisor could not start"),
        }
    }

    fn ingress(&self) -> Result<RemoteNetworkIngress, RemoteComputerTargetError> {
        let now = unix_seconds()?;
        {
            let cache = self
                .ingress_cache
                .lock()
                .map_err(|_| RemoteComputerTargetError::AuthorityUnavailable)?;
            if let Some((ingress, fetched_at)) = cache.as_ref() {
                if fetched_at.elapsed() <= REMOTE_TARGET_INGRESS_CACHE_TTL
                    && ingress.active
                    && ingress.expires_at > now + 15
                {
                    return Ok(ingress.clone());
                }
            }
        }
        let ingress = self.client.network_ingress("official-device-mcp")?;
        let binding = self.client.network_binding()?;
        if binding.device_id != ingress.device_id
            || binding.cell_id != ingress.cell_id
            || binding.generation != ingress.target_generation
            || !binding.active
            || binding.expires_at <= now
        {
            return Err(RemoteComputerTargetError::IdentityMismatch);
        }
        let mut cache = self
            .ingress_cache
            .lock()
            .map_err(|_| RemoteComputerTargetError::AuthorityUnavailable)?;
        *cache = Some((ingress.clone(), Instant::now()));
        Ok(ingress)
    }
}

pub fn production_remote_computer_target_service(
    base_url: &str,
    credential_provider: Arc<dyn FabushiCoreCredentialProvider>,
    device_secret_provider: Arc<dyn RemoteComputerDeviceSecretProvider>,
) -> Result<Arc<dyn NetworkWorkerService>, RemoteComputerTargetError> {
    let client = Arc::new(FabushiCoreTargetClient::new(base_url, credential_provider, device_secret_provider)?);
    Ok(Arc::new(RemoteComputerTargetNetworkService::new(
        client,
        Arc::new(NativeDesktopRemoteToolCatalog),
        Arc::new(NativeDesktopRemoteToolAuthorizer),
        Arc::new(NativeDesktopRemoteExecutor),
        REMOTE_COMPUTER_TARGET_PORT,
    )?))
}

impl NetworkWorkerService for RemoteComputerTargetNetworkService {
    fn poll(&self, worker: &mut OwnedNetworkWorker) -> Result<(), NetworkEnrollmentError> {
        self.ensure_private_ws_thread();
        let ingress = self.ingress().map_err(|error| match error {
            RemoteComputerTargetError::AuthorityUnavailable => NetworkEnrollmentError::AuthorityUnavailable,
            RemoteComputerTargetError::RequestFailed | RemoteComputerTargetError::TransportFailed => {
                NetworkEnrollmentError::RequestFailed
            }
            RemoteComputerTargetError::OutcomeUnknown => NetworkEnrollmentError::OutcomeUnknown,
            RemoteComputerTargetError::IdentityMismatch | RemoteComputerTargetError::InvalidResponse => {
                NetworkEnrollmentError::RequestRejected
            }
        })?;
        self.client
            .serve_private_request_with_timeout(
                worker,
                &ingress.node_id,
                self.local_port,
                REMOTE_TARGET_SERVICE_STREAM_TIMEOUT_MS,
                self.catalog.as_ref(),
                self.authorizer.as_ref(),
                self.executor.as_ref(),
            )
            .map_err(|error| match error {
                RemoteComputerTargetError::AuthorityUnavailable => NetworkEnrollmentError::AuthorityUnavailable,
                RemoteComputerTargetError::RequestFailed | RemoteComputerTargetError::TransportFailed => {
                    NetworkEnrollmentError::RequestFailed
                }
                RemoteComputerTargetError::OutcomeUnknown => NetworkEnrollmentError::OutcomeUnknown,
                RemoteComputerTargetError::IdentityMismatch | RemoteComputerTargetError::InvalidResponse => {
                    NetworkEnrollmentError::RequestRejected
                }
            })
    }
}

fn read_private_frame(
    worker: &mut OwnedNetworkWorker,
    stream_id: &str,
) -> Result<Vec<u8>, RemoteComputerTargetError> {
    let mut frame = Vec::new();
    loop {
        let remaining = REMOTE_TARGET_FRAME_MAX_BYTES.saturating_sub(frame.len());
        if remaining == 0 {
            return Err(RemoteComputerTargetError::InvalidResponse);
        }
        let read = worker
            .stream_read(stream_id, remaining, REMOTE_TARGET_STREAM_TIMEOUT_MS)
            .map_err(|_| RemoteComputerTargetError::TransportFailed)?;
        frame.extend_from_slice(&read.data);
        if let Some(newline) = frame.iter().position(|byte| *byte == b'\n') {
            if frame[newline + 1..]
                .iter()
                .any(|byte| !byte.is_ascii_whitespace())
            {
                return Err(RemoteComputerTargetError::InvalidResponse);
            }
            frame.truncate(newline);
            if frame.is_empty() {
                return Err(RemoteComputerTargetError::InvalidResponse);
            }
            return Ok(frame);
        }
        if read.eof {
            return Err(RemoteComputerTargetError::InvalidResponse);
        }
    }
}


impl Drop for RemoteComputerTargetNetworkService {
    fn drop(&mut self) {
        self.private_ws_stop.store(true, Ordering::Release);
        if let Ok(slot) = self.private_ws_thread.get_mut() {
            if let Some(handle) = slot.take() { let _ = handle.join(); }
        }
    }
}

fn validate_private_ws_url(raw: &str) -> Result<Url, RemoteComputerTargetError> {
    let url = Url::parse(raw.trim()).map_err(|_| RemoteComputerTargetError::InvalidResponse)?;
    if url.scheme() != "wss" || url.host_str().is_none() || !url.username().is_empty()
        || url.password().is_some() || url.query().is_some() || url.fragment().is_some()
        || url.path() != "/fdn-private-agent" {
        return Err(RemoteComputerTargetError::InvalidResponse);
    }
    Ok(url)
}

fn valid_private_ws_outer_request_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn validate_private_ws_payload(payload: &Value, device_id: &str, generation: i64) -> Result<(), RemoteComputerTargetError> {
    if !payload.is_object()
        || serde_json::to_vec(payload).map_err(|_| RemoteComputerTargetError::InvalidResponse)?.len() > MAX_PRIVATE_WS_MESSAGE_BYTES
        || payload.get("deviceId").and_then(Value::as_str) != Some(device_id)
        || payload.get("networkGeneration").and_then(Value::as_i64) != Some(generation) {
        return Err(RemoteComputerTargetError::IdentityMismatch);
    }
    for key in ["sessionId", "requestId", "connectionId", "delegationId"] {
        if let Some(value) = payload.get(key) {
            let text = value.as_str().ok_or(RemoteComputerTargetError::InvalidResponse)?;
            if text.trim().is_empty() || text.len() > 512 || text.contains('\r') || text.contains('\n') {
                return Err(RemoteComputerTargetError::InvalidResponse);
            }
            if key == "requestId" && !(8..=160).contains(&text.len()) {
                return Err(RemoteComputerTargetError::InvalidResponse);
            }
        }
    }
    Ok(())
}

fn correlate_private_ws_response(mut response: Value, payload: &Value, device_id: &str, generation: i64) -> Result<Value, RemoteComputerTargetError> {
    let object = response.as_object_mut().ok_or(RemoteComputerTargetError::InvalidResponse)?;
    if let Some(value) = object.get("deviceId") {
        if value.as_str() != Some(device_id) { return Err(RemoteComputerTargetError::IdentityMismatch); }
    } else { object.insert("deviceId".into(), Value::String(device_id.to_string())); }
    if let Some(value) = object.get("networkGeneration") {
        if value.as_i64() != Some(generation) { return Err(RemoteComputerTargetError::IdentityMismatch); }
    } else { object.insert("networkGeneration".into(), Value::Number(generation.into())); }
    for key in ["sessionId", "requestId", "connectionId", "delegationId"] {
        if let Some(expected) = payload.get(key).and_then(Value::as_str) {
            if let Some(actual) = object.get(key) {
                if actual.as_str() != Some(expected) { return Err(RemoteComputerTargetError::IdentityMismatch); }
            } else { object.insert(key.into(), Value::String(expected.to_string())); }
        }
    }
    Ok(response)
}

fn set_private_ws_timeouts(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>) {
    fn set(stream: &TcpStream) {
        let _ = stream.set_read_timeout(Some(PRIVATE_WS_READ_TIMEOUT));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
    }
    match socket.get_mut() {
        MaybeTlsStream::Plain(stream) => set(stream),
        MaybeTlsStream::Rustls(stream) => set(&stream.sock),
        _ => {}
    }
}

fn connect_private_ws(url: &Url, access_token: &str) -> Result<WebSocket<MaybeTlsStream<TcpStream>>, ()> {
    if access_token.trim().is_empty() { return Err(()); }
    let mut request = url.as_str().into_client_request().map_err(|_| ())?;
    request.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {access_token}")).map_err(|_| ())?);
    let config = WebSocketConfig { max_message_size: Some(MAX_PRIVATE_WS_MESSAGE_BYTES), max_frame_size: Some(MAX_PRIVATE_WS_MESSAGE_BYTES), ..WebSocketConfig::default() };
    let (mut socket, _) = tungstenite::client::connect_with_config(request, Some(config), false).map_err(|_| ())?;
    set_private_ws_timeouts(&mut socket);
    Ok(socket)
}

fn send_private_ws_json(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>, value: &Value) -> Result<(), ()> {
    let encoded = serde_json::to_string(value).map_err(|_| ())?;
    if encoded.len() > MAX_PRIVATE_WS_MESSAGE_BYTES { return Err(()); }
    socket.send(Message::Text(encoded.into())).map_err(|_| ())
}

fn wait_private_ws_registration(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>, timeout: Duration) -> Result<Value, ()> {
    let deadline = Instant::now() + timeout;
    loop {
        if Instant::now() >= deadline { return Err(()); }
        match socket.read() {
            Ok(Message::Text(text)) => return serde_json::from_str(text.as_ref()).map_err(|_| ()),
            Ok(Message::Ping(payload)) => socket.send(Message::Pong(payload)).map_err(|_| ())?,
            Ok(Message::Pong(_)) => {},
            Ok(Message::Close(_)) => return Err(()),
            Ok(_) => return Err(()),
            Err(tungstenite::Error::Io(error)) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {},
            Err(_) => return Err(()),
        }
    }
}

fn private_ws_error_message(_error: &RemoteComputerTargetError) -> &'static str { "target request failed" }

fn dispatch_private_ws_message(
    raw: &str, client: &Arc<FabushiCoreTargetClient>,
    catalog: &Arc<dyn RemoteComputerToolCatalog>, authorizer: &Arc<dyn RemoteComputerToolAuthorizer>,
    executor: &Arc<dyn RemoteComputerNativeExecutor>, account_id: &str, device_id: &str,
    generation: i64, connection_id: &str,
    active: &Arc<Mutex<HashMap<String, Arc<PrivateRequestState>>>>, outgoing: &mpsc::Sender<Value>,
) -> bool {
    let message: Value = match serde_json::from_str(raw) { Ok(value) if value.is_object() => value, _ => return false };
    let kind = match message.get("type").and_then(Value::as_str) { Some(kind) => kind, None => return false };
    let matches = message.get("deviceId").and_then(Value::as_str) == Some(device_id)
        && message.get("networkGeneration").and_then(Value::as_i64) == Some(generation)
        && message.get("connectionId").and_then(Value::as_str) == Some(connection_id);
    match kind {
        "fdnPrivate" => {
            let Some(outer_id) = message.get("requestId").and_then(Value::as_str) else { return false; };
            if !valid_private_ws_outer_request_id(outer_id) || !matches { return false; }
            let Some(payload) = message.get("payload").filter(|value| value.is_object()).cloned() else {
                let _ = outgoing.send(json!({"type":"fdnPrivateResult","requestId":outer_id,"connectionId":connection_id,"deviceId":device_id,"networkGeneration":generation,"ok":false,"error":"private request payload is invalid"}));
                return true;
            };
            if validate_private_ws_payload(&payload, device_id, generation).is_err() {
                let _ = outgoing.send(json!({"type":"fdnPrivateResult","requestId":outer_id,"connectionId":connection_id,"deviceId":device_id,"networkGeneration":generation,"ok":false,"error":"private request identity is invalid"}));
                return true;
            }
            let state = Arc::new(PrivateRequestState::default());
            {
                let Ok(mut requests) = active.lock() else { return false; };
                requests.retain(|_, state| !state.can_expire(Instant::now()));
                if requests.contains_key(outer_id) || requests.len() >= MAX_ACTIVE_PRIVATE_WS_REQUESTS {
                    let _ = outgoing.send(json!({"type":"fdnPrivateResult","requestId":outer_id,"connectionId":connection_id,"deviceId":device_id,"networkGeneration":generation,"ok":false,"error":"private request is already active or capacity is exhausted"}));
                    return true;
                }
                requests.insert(outer_id.to_string(), Arc::clone(&state));
            }
            let request_id = outer_id.to_string();
            let request_id_thread = request_id.clone();
            let device_id_thread = device_id.to_string();
            let connection_id_thread = connection_id.to_string();
            let account_id_thread = account_id.to_string();
            let client_thread = Arc::clone(client);
            let catalog_thread = Arc::clone(catalog);
            let authorizer_thread = Arc::clone(authorizer);
            let executor_thread = Arc::clone(executor);
            let outgoing_thread = outgoing.clone();
            let state_thread = Arc::clone(&state);
            let spawn = thread::Builder::new().name("fabushi-fdn-private-request".into()).spawn(move || {
                let handled = client_thread.handle_private_payload(&payload, &account_id_thread, &device_id_thread, generation,
                    catalog_thread.as_ref(), authorizer_thread.as_ref(), executor_thread.as_ref(), state_thread.as_ref());
                let response = if state_thread.cancelled_before_dispatch() {
                    json!({"type":"fdnPrivateResult","requestId":request_id_thread,"connectionId":connection_id_thread,"deviceId":device_id_thread,"networkGeneration":generation,"ok":false,"error":"private request cancelled before native dispatch"})
                } else {
                    match handled {
                        Ok(result) => json!({"type":"fdnPrivateResult","requestId":request_id_thread,"connectionId":connection_id_thread,"deviceId":device_id_thread,"networkGeneration":generation,"ok":true,"result":result}),
                        Err(error) => json!({"type":"fdnPrivateResult","requestId":request_id_thread,"connectionId":connection_id_thread,"deviceId":device_id_thread,"networkGeneration":generation,"ok":false,"error":private_ws_error_message(&error)}),
                    }
                };
                state_thread.complete();
                let _ = outgoing_thread.send(response);
            });
            if spawn.is_err() {
                state.complete();
                let _ = outgoing.send(json!({"type":"fdnPrivateResult","requestId":request_id,"connectionId":connection_id,"deviceId":device_id,"networkGeneration":generation,"ok":false,"error":"private request worker could not start"}));
            }
            true
        }
        "fdnPrivateCancel" => {
            let Some(request_id) = message.get("requestId").and_then(Value::as_str) else { return false; };
            if !valid_private_ws_outer_request_id(request_id) || !matches { return false; }
            let status = active.lock().ok().and_then(|requests| requests.get(request_id).map(|state| state.request_cancel())).unwrap_or("not_found");
            outgoing.send(json!({"type":"fdnPrivateCancelAck","requestId":request_id,"connectionId":connection_id,"deviceId":device_id,"networkGeneration":generation,"status":status})).is_ok()
        }
        _ => false,
    }
}

fn run_private_ws_loop(
    url: Url, client: Arc<FabushiCoreTargetClient>, catalog: Arc<dyn RemoteComputerToolCatalog>,
    authorizer: Arc<dyn RemoteComputerToolAuthorizer>, executor: Arc<dyn RemoteComputerNativeExecutor>, stop: Arc<AtomicBool>,
) {
    let mut backoff = Duration::from_secs(1);
    while !stop.load(Ordering::Acquire) {
        let (credential, _) = match client.authority() {
            Ok(value) => value,
            Err(_) => { sleep_private_ws_backoff(&stop, backoff); backoff = std::cmp::min(backoff + backoff, Duration::from_secs(30)); continue; }
        };
        let binding = match client.network_binding() {
            Ok(binding) if binding.device_id == credential.device_id && binding.active && binding.expires_at > unix_seconds().unwrap_or(0) => binding,
            _ => { sleep_private_ws_backoff(&stop, backoff); backoff = std::cmp::min(backoff + backoff, Duration::from_secs(30)); continue; }
        };
        let account_id = credential.user_id.clone();
        let device_id = credential.device_id.clone();
        let generation = binding.generation;
        let token_fingerprint = sha256_hex(credential.access_token().as_bytes());
        let mut socket = match connect_private_ws(&url, credential.access_token()) {
            Ok(socket) => socket,
            Err(_) => { sleep_private_ws_backoff(&stop, backoff); backoff = std::cmp::min(backoff + backoff, Duration::from_secs(30)); continue; }
        };
        if send_private_ws_json(&mut socket, &json!({"type":"registerPrivateTransport","deviceId":device_id,"networkGeneration":generation,"leaseSeconds":120,"platform":"desktop"})).is_err() {
            let _ = socket.close(None); sleep_private_ws_backoff(&stop, backoff); backoff = std::cmp::min(backoff + backoff, Duration::from_secs(30)); continue;
        }
        let registered = match wait_private_ws_registration(&mut socket, Duration::from_secs(5)) {
            Ok(value) => value,
            Err(_) => { let _ = socket.close(None); sleep_private_ws_backoff(&stop, backoff); backoff = std::cmp::min(backoff + backoff, Duration::from_secs(30)); continue; }
        };
        let connection_id = registered.get("connectionId").and_then(Value::as_str).unwrap_or("");
        let expires_at = registered.get("expiresAt").and_then(Value::as_str)
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok()).map(|value| value.timestamp()).unwrap_or(0);
        if registered.get("type").and_then(Value::as_str) != Some("privateTransportRegistered")
            || registered.get("accountId").and_then(Value::as_str) != Some(account_id.as_str())
            || registered.get("deviceId").and_then(Value::as_str) != Some(device_id.as_str())
            || registered.get("networkGeneration").and_then(Value::as_i64) != Some(generation)
            || !valid_private_ws_outer_request_id(connection_id)
            || expires_at <= unix_seconds().unwrap_or(i64::MAX) {
            let _ = socket.close(None); sleep_private_ws_backoff(&stop, backoff); backoff = std::cmp::min(backoff + backoff, Duration::from_secs(30)); continue;
        }
        let connected_at = Instant::now();
        let active: Arc<Mutex<HashMap<String, Arc<PrivateRequestState>>>> = Arc::new(Mutex::new(HashMap::new()));
        let (outgoing_tx, outgoing_rx) = mpsc::channel::<Value>();
        let mut last_heartbeat = Instant::now();
        let mut last_identity_check = Instant::now();
        let mut last_network_check = Instant::now();
        let mut disconnected = false;
        while !stop.load(Ordering::Acquire) {
            if expires_at <= unix_seconds().unwrap_or(i64::MAX) { break; }
            if last_heartbeat.elapsed() >= PRIVATE_WS_HEARTBEAT_INTERVAL {
                if send_private_ws_json(&mut socket, &json!({"type":"privateTransportHeartbeat","deviceId":device_id,"networkGeneration":generation,"connectionId":connection_id})).is_err() { break; }
                last_heartbeat = Instant::now();
            }
            if last_identity_check.elapsed() >= PRIVATE_WS_IDENTITY_CHECK_INTERVAL {
                match client.authority() {
                    Ok((current,_)) if current.user_id == account_id && current.device_id == device_id
                        && sha256_hex(current.access_token().as_bytes()) == token_fingerprint => {},
                    _ => break,
                }
                last_identity_check = Instant::now();
            }
            if last_network_check.elapsed() >= PRIVATE_WS_NETWORK_CHECK_INTERVAL {
                match client.network_binding() {
                    Ok(current) if current.device_id == device_id && current.generation == generation && current.active && current.expires_at > unix_seconds().unwrap_or(0) => {},
                    _ => break,
                }
                last_network_check = Instant::now();
            }
            for _ in 0..32 {
                match outgoing_rx.try_recv() {
                    Ok(message) => { if send_private_ws_json(&mut socket,&message).is_err(){disconnected=true;break;} },
                    Err(mpsc::TryRecvError::Empty)=>break,
                    Err(mpsc::TryRecvError::Disconnected)=>{disconnected=true;break;}
                }
            }
            if disconnected { break; }
            if let Ok(mut requests)=active.lock(){requests.retain(|_,state|!state.can_expire(Instant::now()));}
            match socket.read() {
                Ok(Message::Text(text)) => if !dispatch_private_ws_message(text.as_ref(),&client,&catalog,&authorizer,&executor,&account_id,&device_id,generation,connection_id,&active,&outgoing_tx){disconnected=true;break;},
                Ok(Message::Ping(_)) => if socket.flush().is_err(){disconnected=true;break;},
                Ok(Message::Pong(_)) => {},
                Ok(Message::Close(_)) => {disconnected=true;break;},
                Ok(_) => {disconnected=true;break;},
                Err(tungstenite::Error::Io(error)) if matches!(error.kind(),ErrorKind::WouldBlock|ErrorKind::TimedOut)=>thread::sleep(Duration::from_millis(10)),
                Err(_) => {disconnected=true;break;}
            }
        }
        if let Ok(requests)=active.lock(){for state in requests.values(){let _=state.request_cancel();}}
        let _=socket.close(None);
        if stop.load(Ordering::Acquire){break;}
        if connected_at.elapsed()>=Duration::from_secs(60){backoff=Duration::from_secs(1);}else{backoff=std::cmp::min(backoff+backoff,Duration::from_secs(30));}
        sleep_private_ws_backoff(&stop,backoff);
    }
}

fn sleep_private_ws_backoff(stop: &AtomicBool, duration: Duration) {
    let deadline = Instant::now() + duration;
    while !stop.load(Ordering::Acquire) && Instant::now() < deadline { thread::sleep(Duration::from_millis(50)); }
}

fn write_private_frame(
    worker: &mut OwnedNetworkWorker,
    stream_id: &str,
    value: &Value,
) -> Result<(), RemoteComputerTargetError> {
    let mut encoded =
        serde_json::to_vec(value).map_err(|_| RemoteComputerTargetError::InvalidResponse)?;
    if encoded.len() >= REMOTE_TARGET_FRAME_MAX_BYTES {
        return Err(RemoteComputerTargetError::InvalidResponse);
    }
    encoded.push(b'\n');
    let written = worker
        .stream_write(stream_id, &encoded, REMOTE_TARGET_STREAM_TIMEOUT_MS)
        .map_err(|_| RemoteComputerTargetError::TransportFailed)?;
    if written != encoded.len() {
        return Err(RemoteComputerTargetError::TransportFailed);
    }
    Ok(())
}

fn tool_descriptor_sha256(value: &Value) -> Result<String, RemoteComputerTargetError> {
    let mut canonical = String::new();
    canonical_json(value, &mut canonical)?;
    Ok(sha256_hex(canonical.as_bytes()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(64);
    use std::fmt::Write as _;
    for byte in digest {
        let _ = write!(&mut encoded, "{byte:02x}");
    }
    encoded
}

fn required_text(value: &Value, key: &str) -> Result<String, RemoteComputerTargetError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or(RemoteComputerTargetError::InvalidResponse)
}


#[cfg(test)]
mod sensitive_result_tests {
    use super::*;

    fn entry(now: i64) -> SensitiveResultEntry {
        SensitiveResultEntry {
            account_user_id: "account-a".into(),
            device_id: "device-a".into(),
            network_generation: 7,
            session_id: "session-a".into(),
            client_id: "client-a".into(),
            request_id: "request-00000001".into(),
            request_sha256: "a".repeat(64),
            tool_name: "Computer".into(),
            grant_generation: 9,
            result_sha256: "b".repeat(64),
            media_type: "image/png".into(),
            bytes: vec![42; REMOTE_SENSITIVE_RESULT_CHUNK_BYTES + 3],
            captured_at_ms: 123,
            width: Some(100),
            height: Some(80),
            snapshot_generation: 1,
            expires_at: now + 60,
            next_offset: 0,
        }
    }

    #[test]
    fn sensitive_binding_rejects_wrong_account_device_session_digest_and_generation() {
        let item = entry(100);
        assert!(FabushiCoreTargetClient::sensitive_entry_matches(
            &item, "account-a", "device-a", "device-a", 7, "session-a", "client-a",
            "request-00000001", &"a".repeat(64), 9,
        ));
        assert!(!FabushiCoreTargetClient::sensitive_entry_matches(
            &item, "account-b", "device-a", "device-a", 7, "session-a", "client-a",
            "request-00000001", &"a".repeat(64), 9,
        ));
        assert!(!FabushiCoreTargetClient::sensitive_entry_matches(
            &item, "account-a", "device-b", "device-a", 7, "session-a", "client-a",
            "request-00000001", &"a".repeat(64), 9,
        ));
        assert!(!FabushiCoreTargetClient::sensitive_entry_matches(
            &item, "account-a", "device-a", "device-a", 7, "session-b", "client-a",
            "request-00000001", &"a".repeat(64), 9,
        ));
        assert!(!FabushiCoreTargetClient::sensitive_entry_matches(
            &item, "account-a", "device-a", "device-a", 7, "session-a", "client-b",
            "request-00000001", &"a".repeat(64), 9,
        ));
        assert!(!FabushiCoreTargetClient::sensitive_entry_matches(
            &item, "account-a", "device-a", "device-a", 7, "session-a", "client-a",
            "request-00000001", &"c".repeat(64), 9,
        ));
        assert!(!FabushiCoreTargetClient::sensitive_entry_matches(
            &item, "account-a", "device-a", "device-a", 8, "session-a", "client-a",
            "request-00000001", &"a".repeat(64), 9,
        ));
        assert!(!FabushiCoreTargetClient::sensitive_entry_matches(
            &item, "account-a", "device-a", "device-a", 7, "session-a", "client-a",
            "request-00000001", &"a".repeat(64), 10,
        ));
    }

    #[test]
    fn sensitive_chunk_range_is_sequential_and_replay_rejected() {
        let mut item = entry(100);
        let (end, done) = sensitive_chunk_range(&item, 0).expect("first chunk");
        assert_eq!(end, REMOTE_SENSITIVE_RESULT_CHUNK_BYTES);
        assert!(!done);
        item.next_offset = end;
        assert_eq!(
            sensitive_chunk_range(&item, 0),
            Err(RemoteComputerTargetError::IdentityMismatch),
            "old offset must not replay"
        );
        let (final_end, final_done) = sensitive_chunk_range(&item, end).expect("final chunk");
        assert_eq!(final_end, item.bytes.len());
        assert!(final_done);
    }

    #[test]
    fn sensitive_vault_purges_expired_entries_and_is_bounded() {
        let mut vault = SensitiveResultVault::default();
        vault.insert("expired".into(), entry(0), 0);
        vault.purge_expired(61);
        assert!(vault.entries.is_empty());

        for index in 0..(REMOTE_SENSITIVE_RESULT_MAX_ENTRIES + 2) {
            let mut item = entry(100);
            item.expires_at = 200 + index as i64;
            vault.insert(format!("ref-{index}"), item, 100);
        }
        assert!(vault.entries.len() <= REMOTE_SENSITIVE_RESULT_MAX_ENTRIES);
    }
}

#[cfg(test)]
mod private_websocket_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn private_websocket_url_requires_tls_and_exact_path() {
        assert!(validate_private_ws_url("wss://mcp.ombhrum.com/fdn-private-agent").is_ok());
        assert!(validate_private_ws_url("ws://mcp.ombhrum.com/fdn-private-agent").is_err());
        assert!(validate_private_ws_url("wss://mcp.ombhrum.com/agent").is_err());
        assert!(validate_private_ws_url("wss://user:secret@mcp.ombhrum.com/fdn-private-agent").is_err());
        assert!(validate_private_ws_url("wss://mcp.ombhrum.com/fdn-private-agent?token=secret").is_err());
    }

    #[test]
    fn private_websocket_payload_and_response_are_generation_bound() {
        let payload=json!({"deviceId":"device-a","networkGeneration":7,"sessionId":"session-a","requestId":"operation-00000001"});
        assert!(validate_private_ws_payload(&payload,"device-a",7).is_ok());
        assert!(validate_private_ws_payload(&payload,"device-b",7).is_err());
        assert!(validate_private_ws_payload(&payload,"device-a",8).is_err());
        let response=correlate_private_ws_response(json!({"type":"operationResult","status":"completed"}),&payload,"device-a",7).unwrap();
        assert_eq!(response["deviceId"],"device-a");
        assert_eq!(response["networkGeneration"],7);
        assert_eq!(response["sessionId"],"session-a");
        assert_eq!(response["requestId"],"operation-00000001");
        assert!(correlate_private_ws_response(json!({"deviceId":"device-b","networkGeneration":7}),&payload,"device-a",7).is_err());
    }

    #[test]
    fn private_websocket_cancel_distinguishes_queued_from_dispatched() {
        let queued=PrivateRequestState::default();
        assert_eq!(queued.request_cancel(),"cancelled");
        assert!(!queued.begin_dispatch());
        queued.complete();
        assert_eq!(queued.request_cancel(),"cancelled");
        let running=PrivateRequestState::default();
        assert!(running.begin_dispatch());
        assert_eq!(running.request_cancel(),"cancellation_pending");
        running.complete();
        assert_eq!(running.request_cancel(),"already_completed");
    }
}
