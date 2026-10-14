use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use reqwest::blocking::Client;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use thiserror::Error;
use url::Url;

use crate::extensions::auth::auth_service::{
    FabushiCoreCredential, FabushiCoreCredentialProvider,
};

const CORE_REQUEST_TIMEOUT_SECS: u64 = 15;
const MAX_CORE_RESPONSE_BYTES: usize = 1024 * 1024;

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

fn required_text(value: &Value, key: &str) -> Result<String, RemoteComputerTargetError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or(RemoteComputerTargetError::InvalidResponse)
}
