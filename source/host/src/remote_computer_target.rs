use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use thiserror::Error;
use url::Url;

use crate::extensions::auth::auth_service::{
    FabushiCoreCredential, FabushiCoreCredentialProvider,
};
use crate::network_worker_process::OwnedNetworkWorker;
use crate::runner::routed_provider_runtime::RoutedToolBridge;
use crate::runner::tools::sand_computer_tool::{
    ComputerToolExecutor, ComputerUseResult, parse_remote_computer_action,
};

const CORE_REQUEST_TIMEOUT_SECS: u64 = 15;
const MAX_CORE_RESPONSE_BYTES: usize = 1024 * 1024;
const REMOTE_TARGET_STREAM_TIMEOUT_MS: u64 = 15_000;
const REMOTE_TARGET_FRAME_MAX_BYTES: usize = 32 * 1024;
const REMOTE_TARGET_RESULT_MAX_BYTES: usize = 1024 * 1024;

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



#[derive(Debug, Clone, PartialEq)]
pub enum RemoteComputerNativeOutcome {
    Completed(Value),
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

pub struct HostComputerRemoteExecutor {
    computer: Arc<dyn ComputerToolExecutor>,
}

impl HostComputerRemoteExecutor {
    pub fn new(computer: Arc<dyn ComputerToolExecutor>) -> Self {
        Self { computer }
    }
}

impl RemoteComputerNativeExecutor for HostComputerRemoteExecutor {
    fn execute(
        &self,
        operation: &ClaimedRemoteOperation,
        arguments: &Value,
        _disk_preflight: &Value,
    ) -> RemoteComputerNativeOutcome {
        if operation.tool_name == "Screenshot" {
            return RemoteComputerNativeOutcome::Failed(json!({
                "code": "sensitive_result_channel_unavailable",
                "message": "Remote Screenshot is disabled until the authenticated chunked sensitive-result channel is active.",
                "retryable": false,
            }));
        }
        if operation.tool_name != "Computer" {
            return RemoteComputerNativeOutcome::Failed(json!({
                "code": "unsupported_tool",
                "message": "This target executor only owns the existing Host Computer tool.",
                "retryable": false,
            }));
        }
        let action = match parse_remote_computer_action(arguments) {
            Ok(action) => action,
            Err(error) => {
                return RemoteComputerNativeOutcome::Failed(json!({
                    "code": "invalid_arguments",
                    "message": error.to_string(),
                    "retryable": false,
                }));
            }
        };
        match self.computer.execute(&action, &operation.request_id) {
            Ok(ComputerUseResult::Success(success)) => RemoteComputerNativeOutcome::Completed(json!({
                "content": "Computer action ran on the target desktop.",
                "cursorPosition": success.cursor_position.map(|position| json!({
                    "x": position.x,
                    "y": position.y,
                })),
                "observedScreenshotAvailableOnTarget": success.screenshot.is_some()
                    || success.screenshot_path.is_some(),
                "screenshotTransferred": false,
            })),
            Ok(ComputerUseResult::Error(error)) => RemoteComputerNativeOutcome::Failed(json!({
                "code": "computer_action_failed",
                "message": error,
                "retryable": false,
            })),
            Ok(ComputerUseResult::Other(state)) => RemoteComputerNativeOutcome::Failed(json!({
                "code": "computer_action_unconfirmed",
                "message": state,
                "retryable": false,
            })),
            Err(error) => RemoteComputerNativeOutcome::Failed(json!({
                "code": "computer_executor_error",
                "message": error.to_string(),
                "retryable": false,
            })),
        }
    }
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

pub struct FabushiCoreTargetClient {
    base_url: Url,
    credential_provider: Arc<dyn FabushiCoreCredentialProvider>,
    device_secret_provider: Arc<dyn RemoteComputerDeviceSecretProvider>,
    http: Client,
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
        if expected_peer.trim().is_empty() || local_port == 0 {
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
                write_private_frame(
                    worker,
                    stream_id,
                    &json!({
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
                write_private_frame(
                    worker,
                    stream_id,
                    &json!({
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
                write_private_frame(
                    worker,
                    stream_id,
                    &json!({
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

                match executor.execute(&claimed, &arguments, &disk_preflight) {
                    RemoteComputerNativeOutcome::OutcomeUnknown => write_private_frame(
                        worker,
                        stream_id,
                        &json!({
                            "type": "operationResult",
                            "deviceId": device_id,
                            "sessionId": session_id,
                            "requestId": request_id,
                            "requestSha256": request_sha256,
                            "status": "outcome_unknown",
                            "networkGeneration": network_generation,
                        }),
                    ),
                    RemoteComputerNativeOutcome::Completed(result) => self.finish_native_operation(
                        worker,
                        stream_id,
                        &claimed,
                        &device_id,
                        network_generation,
                        "completed",
                        result,
                    ),
                    RemoteComputerNativeOutcome::Failed(result) => self.finish_native_operation(
                        worker,
                        stream_id,
                        &claimed,
                        &device_id,
                        network_generation,
                        "failed",
                        result,
                    ),
                }
            }
        }
    }

    fn finish_native_operation(
        &self,
        worker: &mut OwnedNetworkWorker,
        stream_id: &str,
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
        write_private_frame(
            worker,
            stream_id,
            &json!({
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


fn unix_seconds() -> Result<i64, RemoteComputerTargetError> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| RemoteComputerTargetError::AuthorityUnavailable)?
        .as_secs();
    i64::try_from(seconds).map_err(|_| RemoteComputerTargetError::AuthorityUnavailable)
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
