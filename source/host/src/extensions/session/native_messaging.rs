use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

const DEFAULT_FABUSHI_API_BASE_URL: &str = "https://api.ombhrum.com";
const FABUSHI_API_BASE_URL_ENV: &str = "FABUSHI_API_BASE_URL";
const FABUSHI_HOST_ACCESS_CREDENTIAL_FILE_ENV: &str = "FABUSHI_HOST_ACCESS_CREDENTIAL_FILE";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FabushiNativeIdentity {
    pub user_id: String,
    pub device_id: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FabushiRemoteHumanContact {
    pub id: Value,
    pub user_id: Value,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FabushiRemoteHumanAttachment {
    pub resource_id: String,
    pub name: String,
    pub content_type: String,
    pub size: u64,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FabushiRemoteHumanReaction {
    pub emoji: String,
    pub count: u64,
    pub reacted_by_me: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FabushiRemoteHumanMessage {
    pub id: Value,
    pub sender_user_id: Value,
    pub sender_username: Option<String>,
    pub recipient_user_id: Value,
    pub recipient_username: Option<String>,
    pub text: String,
    pub client_request_id: Option<String>,
    pub created_at: String,
    pub read_at: Option<String>,
    #[serde(default)]
    pub is_outgoing: bool,
    pub reply_to_message_id: Option<Value>,
    #[serde(default)]
    pub attachments: Vec<FabushiRemoteHumanAttachment>,
    #[serde(default)]
    pub reactions: Vec<FabushiRemoteHumanReaction>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FabushiRemoteHumanCall {
    pub call_id: String,
    pub creator_user_id: Value,
    pub peer_user_id: Value,
    #[serde(default)]
    pub creator_device_id: Option<String>,
    #[serde(default)]
    pub peer_device_id: Option<String>,
    pub state: String,
    pub generation: u64,
    pub event_seq: u64,
    pub terminal_state: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FabushiRemoteHumanCallEvent {
    pub call_id: String,
    pub seq: u64,
    pub generation: u64,
    pub user_id: Value,
    pub device_id: String,
    pub client_event_id: String,
    pub kind: String,
    pub payload: Value,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
struct CredentialFile {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "deviceId")]
    device_id: String,
    #[serde(rename = "userId")]
    user_id: Value,
}

#[derive(Debug, Deserialize)]
struct SendEnvelope {
    success: bool,
    message: Option<FabushiRemoteHumanMessage>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ListData {
    messages: Vec<FabushiRemoteHumanMessage>,
}

#[derive(Debug, Deserialize)]
struct ResourceEnvelope {
    success: bool,
    resource: Option<FabushiRemoteHumanAttachment>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FriendData {
    friends: Vec<FabushiRemoteHumanContact>,
}

#[derive(Debug, Deserialize)]
struct FriendEnvelope {
    success: bool,
    data: Option<FriendData>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ListEnvelope {
    success: bool,
    data: Option<ListData>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SendAttachmentRequest<'a> {
    resource_id: &'a str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SendRequest<'a> {
    target_user_id: &'a str,
    text: &'a str,
    client_request_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_to_message_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    attachments: Vec<SendAttachmentRequest<'a>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReactionRequest<'a> {
    emoji: &'a str,
    active: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReactionEnvelope {
    success: bool,
    message_id: Option<Value>,
    #[serde(default)]
    reactions: Vec<FabushiRemoteHumanReaction>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateCallRequest<'a> {
    call_id: &'a str,
    target_user_id: &'a str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppendCallEventRequest<'a> {
    client_event_id: &'a str,
    generation: u64,
    kind: &'a str,
    payload: &'a Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CallEnvelope {
    success: bool,
    call: Option<FabushiRemoteHumanCall>,
    #[serde(default)]
    calls: Vec<FabushiRemoteHumanCall>,
    #[serde(default)]
    events: Vec<FabushiRemoteHumanCallEvent>,
    event: Option<FabushiRemoteHumanCallEvent>,
    next_after_seq: Option<u64>,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FabushiCallIceServer {
    pub urls: Value,
    pub username: Option<String>,
    pub credential: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CallIceEnvelope {
    success: bool,
    #[serde(default)]
    ice_servers: Vec<FabushiCallIceServer>,
    ttl_seconds: Option<u64>,
    error: Option<String>,
}

fn valid_call_ice_urls(value: &Value) -> bool {
    let valid = |url: &str| {
        let trimmed = url.trim();
        !trimmed.is_empty()
            && trimmed.len() <= 1024
            && !trimmed.chars().any(char::is_whitespace)
            && (trimmed.starts_with("stun:")
                || trimmed.starts_with("turn:")
                || trimmed.starts_with("turns:"))
    };
    match value {
        Value::String(url) => valid(url),
        Value::Array(urls) => {
            !urls.is_empty()
                && urls.len() <= 16
                && urls.iter().all(|url| url.as_str().is_some_and(valid))
        }
        _ => false,
    }
}

#[derive(Debug, Clone)]
pub struct FabushiNativeMessagingClient {
    base_url: Url,
    credential_path: PathBuf,
    client: Client,
}

impl FabushiNativeMessagingClient {
    pub fn from_env() -> Result<Self, String> {
        let base_url = env::var(FABUSHI_API_BASE_URL_ENV)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| DEFAULT_FABUSHI_API_BASE_URL.to_string());
        let credential_path = env::var_os(FABUSHI_HOST_ACCESS_CREDENTIAL_FILE_ENV)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .ok_or_else(|| {
                "Fabushi native messaging requires the account-bound Host credential file.".to_string()
            })?;
        Self::new(&base_url, credential_path)
    }

    pub fn new(base_url: &str, credential_path: impl Into<PathBuf>) -> Result<Self, String> {
        let base_url = validate_base_url(base_url)?;
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|error| format!("could not create Fabushi messaging HTTP client: {error}"))?;
        Ok(Self {
            base_url,
            credential_path: credential_path.into(),
            client,
        })
    }

    pub fn identity(&self) -> Result<FabushiNativeIdentity, String> {
        let credentials = read_credentials(&self.credential_path)?;
        Ok(FabushiNativeIdentity {
            user_id: fabushi_identity_text(&credentials.user_id)?,
            device_id: required_trimmed(&credentials.device_id, "deviceId")?.to_string(),
        })
    }

    pub fn upload_direct_message_resource(
        &self,
        path: &Path,
        name: &str,
    ) -> Result<FabushiRemoteHumanAttachment, String> {
        let name = required_trimmed(name, "attachment name")?;
        let bytes = fs::read(path)
            .map_err(|error| format!("Fabushi Human attachment could not be read from {}: {error}", path.display()))?;
        if bytes.is_empty() {
            return Err("Fabushi Human attachment is empty".into());
        }
        let credentials = read_credentials(&self.credential_path)?;
        let part = reqwest::blocking::multipart::Part::bytes(bytes)
            .file_name(name.to_string())
            .mime_str("application/octet-stream")
            .map_err(|error| format!("Fabushi Human attachment MIME metadata was invalid: {error}"))?;
        let response = self
            .client
            .post(self.endpoint("/api/social/message-resources")?)
            .bearer_auth(required_trimmed(&credentials.access_token, "accessToken")?)
            .header(
                "x-fabushi-device-id",
                required_trimmed(&credentials.device_id, "deviceId")?,
            )
            .header("accept", "application/json")
            .multipart(reqwest::blocking::multipart::Form::new().part("file", part))
            .send()
            .map_err(|error| format!("Fabushi Human attachment upload failed: {error}"))?;
        let status = response.status();
        let body = response
            .text()
            .map_err(|error| format!("Fabushi Human attachment response could not be read: {error}"))?;
        let envelope: ResourceEnvelope = serde_json::from_str(&body).map_err(|error| {
            format!(
                "Fabushi Human attachment response was invalid JSON (HTTP {}): {error}",
                status.as_u16()
            )
        })?;
        if !status.is_success() || !envelope.success {
            return Err(format!(
                "Fabushi Human attachment upload was rejected (HTTP {}): {}",
                status.as_u16(),
                envelope.error.unwrap_or_else(|| "unknown backend error".into())
            ));
        }
        envelope
            .resource
            .ok_or_else(|| "Fabushi Human attachment response omitted the persisted resource.".to_string())
    }

    pub fn download_direct_message_resource(
        &self,
        resource_id: &str,
        expected_size: u64,
    ) -> Result<Vec<u8>, String> {
        const MAX_RESOURCE_BYTES: u64 = 32 * 1024 * 1024;
        let resource_id = required_trimmed(resource_id, "resource id")?;
        if resource_id.len() > 128 || resource_id.contains('/') || resource_id.contains('\\') {
            return Err("Fabushi Human attachment resource id is invalid".into());
        }
        if expected_size == 0 || expected_size > MAX_RESOURCE_BYTES {
            return Err("Fabushi Human attachment size exceeds the backend limit".into());
        }
        let credentials = read_credentials(&self.credential_path)?;
        let encoded_resource_id: String =
            url::form_urlencoded::byte_serialize(resource_id.as_bytes()).collect();
        let response = self
            .client
            .get(self.endpoint(&format!(
                "/api/social/message-resources/{encoded_resource_id}"
            ))?)
            .bearer_auth(required_trimmed(&credentials.access_token, "accessToken")?)
            .header(
                "x-fabushi-device-id",
                required_trimmed(&credentials.device_id, "deviceId")?,
            )
            .header("accept", "application/octet-stream")
            .send()
            .map_err(|error| format!("Fabushi Human attachment download failed: {error}"))?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().unwrap_or_default();
            return Err(format!(
                "Fabushi Human attachment download was rejected (HTTP {}): {}",
                status.as_u16(),
                body.chars().take(512).collect::<String>()
            ));
        }
        if let Some(length) = response.content_length() {
            if length != expected_size || length > MAX_RESOURCE_BYTES {
                return Err(format!(
                    "Fabushi Human attachment download length mismatch: expected {expected_size}, received {length}"
                ));
            }
        }
        let mut bytes = Vec::with_capacity(expected_size as usize);
        response
            .take(MAX_RESOURCE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| format!("Fabushi Human attachment body could not be read: {error}"))?;
        if bytes.len() as u64 != expected_size || bytes.len() as u64 > MAX_RESOURCE_BYTES {
            return Err(format!(
                "Fabushi Human attachment body length mismatch: expected {expected_size}, received {}",
                bytes.len()
            ));
        }
        Ok(bytes)
    }

    pub fn send_direct_message(
        &self,
        peer_human_id: &str,
        text: &str,
        client_request_id: &str,
        reply_to_message_id: Option<&str>,
        attachments: &[FabushiRemoteHumanAttachment],
    ) -> Result<FabushiRemoteHumanMessage, String> {
        let peer_human_id = required_trimmed(peer_human_id, "peer Human id")?;
        let text = text.trim();
        let client_request_id = required_trimmed(client_request_id, "client request id")?;
        let reply_to_message_id = reply_to_message_id
            .map(str::trim)
            .filter(|value| !value.is_empty());
        if text.is_empty() && attachments.is_empty() {
            return Err("Fabushi Human message requires text or attachments".into());
        }
        let credentials = read_credentials(&self.credential_path)?;
        let response = self
            .client
            .post(self.endpoint("/api/social/messages")?)
            .bearer_auth(required_trimmed(&credentials.access_token, "accessToken")?)
            .header(
                "x-fabushi-device-id",
                required_trimmed(&credentials.device_id, "deviceId")?,
            )
            .header("accept", "application/json")
            .json(&SendRequest {
                target_user_id: peer_human_id,
                text,
                client_request_id,
                reply_to_message_id,
                attachments: attachments
                    .iter()
                    .map(|attachment| SendAttachmentRequest {
                        resource_id: attachment.resource_id.as_str(),
                    })
                    .collect(),
            })
            .send()
            .map_err(|error| format!("Fabushi Human message send failed: {error}"))?;
        let status = response.status();
        let body = response
            .text()
            .map_err(|error| format!("Fabushi Human message response could not be read: {error}"))?;
        let envelope: SendEnvelope = serde_json::from_str(&body).map_err(|error| {
            format!(
                "Fabushi Human message response was invalid JSON (HTTP {}): {error}",
                status.as_u16()
            )
        })?;
        if !status.is_success() || !envelope.success {
            return Err(format!(
                "Fabushi Human message send was rejected (HTTP {}): {}",
                status.as_u16(),
                envelope.error.unwrap_or_else(|| "unknown backend error".into())
            ));
        }
        envelope
            .message
            .ok_or_else(|| "Fabushi Human message response omitted the persisted message.".to_string())
    }

    pub fn set_direct_message_reaction(
        &self,
        message_id: &str,
        emoji: &str,
        active: bool,
    ) -> Result<Vec<FabushiRemoteHumanReaction>, String> {
        let message_id = required_trimmed(message_id, "message id")?;
        if message_id
            .parse::<u64>()
            .ok()
            .filter(|value| *value > 0)
            .is_none()
        {
            return Err("Fabushi Human reaction requires a positive canonical message id".into());
        }
        let emoji = required_trimmed(emoji, "reaction emoji")?;
        if emoji.as_bytes().len() > 32 {
            return Err("Fabushi Human reaction emoji exceeds the backend limit".into());
        }
        let credentials = read_credentials(&self.credential_path)?;
        let path = format!("/api/social/messages/{message_id}/reactions");
        let response = self
            .client
            .post(self.endpoint(&path)?)
            .bearer_auth(required_trimmed(&credentials.access_token, "accessToken")?)
            .header(
                "x-fabushi-device-id",
                required_trimmed(&credentials.device_id, "deviceId")?,
            )
            .header("accept", "application/json")
            .json(&ReactionRequest { emoji, active })
            .send()
            .map_err(|error| format!("Fabushi Human reaction mutation failed: {error}"))?;
        let status = response.status();
        let body = response
            .text()
            .map_err(|error| format!("Fabushi Human reaction response could not be read: {error}"))?;
        let envelope: ReactionEnvelope = serde_json::from_str(&body).map_err(|error| {
            format!(
                "Fabushi Human reaction response was invalid JSON (HTTP {}): {error}",
                status.as_u16()
            )
        })?;
        if !status.is_success() || !envelope.success {
            return Err(format!(
                "Fabushi Human reaction mutation was rejected (HTTP {}): {}",
                status.as_u16(),
                envelope.error.unwrap_or_else(|| "unknown backend error".into())
            ));
        }
        let returned_id = envelope
            .message_id
            .as_ref()
            .ok_or_else(|| "Fabushi Human reaction response omitted the message id.".to_string())
            .and_then(fabushi_identity_text)?;
        if returned_id != message_id {
            return Err("Fabushi Human reaction backend returned a mismatched message id".into());
        }
        Ok(envelope.reactions)
    }

    pub fn list_friends(&self) -> Result<Vec<FabushiRemoteHumanContact>, String> {
        let credentials = read_credentials(&self.credential_path)?;
        let response = self
            .client
            .get(self.endpoint("/api/social/friends")?)
            .bearer_auth(required_trimmed(&credentials.access_token, "accessToken")?)
            .header(
                "x-fabushi-device-id",
                required_trimmed(&credentials.device_id, "deviceId")?,
            )
            .header("accept", "application/json")
            .send()
            .map_err(|error| format!("Fabushi Human contact sync failed: {error}"))?;
        let status = response.status();
        let body = response
            .text()
            .map_err(|error| format!("Fabushi Human contact sync response could not be read: {error}"))?;
        let envelope: FriendEnvelope = serde_json::from_str(&body).map_err(|error| {
            format!(
                "Fabushi Human contact sync response was invalid JSON (HTTP {}): {error}",
                status.as_u16()
            )
        })?;
        if !status.is_success() || !envelope.success {
            return Err(format!(
                "Fabushi Human contact sync was rejected (HTTP {}): {}",
                status.as_u16(),
                envelope.error.unwrap_or_else(|| "unknown backend error".into())
            ));
        }
        Ok(envelope.data.map(|data| data.friends).unwrap_or_default())
    }

    pub fn list_direct_messages(
        &self,
        peer_human_id: &str,
        before: Option<&str>,
        limit: usize,
    ) -> Result<Vec<FabushiRemoteHumanMessage>, String> {
        let peer_human_id = required_trimmed(peer_human_id, "peer Human id")?;
        let credentials = read_credentials(&self.credential_path)?;
        let mut url = self.endpoint("/api/social/messages")?;
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("contactId", peer_human_id);
            query.append_pair("limit", &limit.clamp(1, 200).to_string());
            if let Some(before) = before.map(str::trim).filter(|value| !value.is_empty()) {
                query.append_pair("before", before);
            }
        }
        let response = self
            .client
            .get(url)
            .bearer_auth(required_trimmed(&credentials.access_token, "accessToken")?)
            .header(
                "x-fabushi-device-id",
                required_trimmed(&credentials.device_id, "deviceId")?,
            )
            .header("accept", "application/json")
            .send()
            .map_err(|error| format!("Fabushi Human message sync failed: {error}"))?;
        let status = response.status();
        let body = response
            .text()
            .map_err(|error| format!("Fabushi Human message sync response could not be read: {error}"))?;
        let envelope: ListEnvelope = serde_json::from_str(&body).map_err(|error| {
            format!(
                "Fabushi Human message sync response was invalid JSON (HTTP {}): {error}",
                status.as_u16()
            )
        })?;
        if !status.is_success() || !envelope.success {
            return Err(format!(
                "Fabushi Human message sync was rejected (HTTP {}): {}",
                status.as_u16(),
                envelope.error.unwrap_or_else(|| "unknown backend error".into())
            ));
        }
        Ok(envelope.data.map(|data| data.messages).unwrap_or_default())
    }

    pub fn get_human_call_ice_servers(
        &self,
    ) -> Result<(Vec<FabushiCallIceServer>, u64), String> {
        let credentials = read_credentials(&self.credential_path)?;
        let response = self
            .client
            .get(self.endpoint("/api/social/calls/ice")?)
            .bearer_auth(required_trimmed(&credentials.access_token, "accessToken")?)
            .header(
                "x-fabushi-device-id",
                required_trimmed(&credentials.device_id, "deviceId")?,
            )
            .header("accept", "application/json")
            .send()
            .map_err(|error| format!("Fabushi Human call ICE credential request failed: {error}"))?;
        let status = response.status();
        let body = response
            .text()
            .map_err(|error| format!("Fabushi Human call ICE response could not be read: {error}"))?;
        let envelope: CallIceEnvelope = serde_json::from_str(&body).map_err(|error| {
            format!(
                "Fabushi Human call ICE response was invalid JSON (HTTP {}): {error}",
                status.as_u16()
            )
        })?;
        if !status.is_success() || !envelope.success {
            return Err(format!(
                "Fabushi Human call ICE credentials were rejected (HTTP {}): {}",
                status.as_u16(),
                envelope.error.unwrap_or_else(|| "unknown backend error".into())
            ));
        }
        let ttl_seconds = envelope
            .ttl_seconds
            .filter(|ttl| (300..=86_400).contains(ttl))
            .ok_or_else(|| "Fabushi Human call ICE response contained an invalid TTL".to_string())?;
        if envelope.ice_servers.is_empty() || envelope.ice_servers.len() > 16 {
            return Err("Fabushi Human call ICE response contained an invalid server list".into());
        }
        for server in &envelope.ice_servers {
            if !valid_call_ice_urls(&server.urls)
                || server.username.as_deref().is_some_and(|value| value.len() > 2048)
                || server.credential.as_deref().is_some_and(|value| value.len() > 2048)
            {
                return Err("Fabushi Human call ICE response contained an invalid server".into());
            }
        }
        Ok((envelope.ice_servers, ttl_seconds))
    }

    pub fn list_human_calls(&self, limit: usize) -> Result<Vec<FabushiRemoteHumanCall>, String> {
        let credentials = read_credentials(&self.credential_path)?;
        let mut url = self.endpoint("/api/social/calls")?;
        url.query_pairs_mut()
            .append_pair("limit", &limit.clamp(1, 200).to_string());
        let response = self
            .client
            .get(url)
            .bearer_auth(required_trimmed(&credentials.access_token, "accessToken")?)
            .header(
                "x-fabushi-device-id",
                required_trimmed(&credentials.device_id, "deviceId")?,
            )
            .header("accept", "application/json")
            .send()
            .map_err(|error| format!("Fabushi Human call discovery failed: {error}"))?;
        let status = response.status();
        let body = response
            .text()
            .map_err(|error| format!("Fabushi Human call discovery response could not be read: {error}"))?;
        let envelope: CallEnvelope = serde_json::from_str(&body).map_err(|error| {
            format!(
                "Fabushi Human call discovery response was invalid JSON (HTTP {}): {error}",
                status.as_u16()
            )
        })?;
        if !status.is_success() || !envelope.success {
            return Err(format!(
                "Fabushi Human call discovery was rejected (HTTP {}): {}",
                status.as_u16(),
                envelope.error.unwrap_or_else(|| "unknown backend error".into())
            ));
        }
        Ok(envelope.calls)
    }

    pub fn create_human_call(
        &self,
        call_id: &str,
        peer_human_id: &str,
    ) -> Result<FabushiRemoteHumanCall, String> {
        let call_id = required_trimmed(call_id, "call id")?;
        let peer_human_id = required_trimmed(peer_human_id, "peer Human id")?;
        let credentials = read_credentials(&self.credential_path)?;
        let response = self
            .client
            .post(self.endpoint("/api/social/calls")?)
            .bearer_auth(required_trimmed(&credentials.access_token, "accessToken")?)
            .header(
                "x-fabushi-device-id",
                required_trimmed(&credentials.device_id, "deviceId")?,
            )
            .header("accept", "application/json")
            .json(&CreateCallRequest {
                call_id,
                target_user_id: peer_human_id,
            })
            .send()
            .map_err(|error| format!("Fabushi Human call creation failed: {error}"))?;
        let status = response.status();
        let body = response
            .text()
            .map_err(|error| format!("Fabushi Human call creation response could not be read: {error}"))?;
        let envelope: CallEnvelope = serde_json::from_str(&body).map_err(|error| {
            format!(
                "Fabushi Human call creation response was invalid JSON (HTTP {}): {error}",
                status.as_u16()
            )
        })?;
        if !status.is_success() || !envelope.success {
            return Err(format!(
                "Fabushi Human call creation was rejected (HTTP {}): {}",
                status.as_u16(),
                envelope.error.unwrap_or_else(|| "unknown backend error".into())
            ));
        }
        let call = envelope
            .call
            .ok_or_else(|| "Fabushi Human call creation response omitted the call.".to_string())?;
        if call.call_id != call_id {
            return Err("Fabushi Human call backend returned a mismatched call id".into());
        }
        Ok(call)
    }

    pub fn get_human_call(
        &self,
        call_id: &str,
        after_seq: u64,
        limit: usize,
    ) -> Result<(FabushiRemoteHumanCall, Vec<FabushiRemoteHumanCallEvent>, u64), String> {
        let call_id = required_trimmed(call_id, "call id")?;
        let encoded_call_id: String =
            url::form_urlencoded::byte_serialize(call_id.as_bytes()).collect();
        let credentials = read_credentials(&self.credential_path)?;
        let mut url = self.endpoint(&format!("/api/social/calls/{encoded_call_id}"))?;
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("afterSeq", &after_seq.to_string());
            query.append_pair("limit", &limit.clamp(1, 200).to_string());
        }
        let response = self
            .client
            .get(url)
            .bearer_auth(required_trimmed(&credentials.access_token, "accessToken")?)
            .header(
                "x-fabushi-device-id",
                required_trimmed(&credentials.device_id, "deviceId")?,
            )
            .header("accept", "application/json")
            .send()
            .map_err(|error| format!("Fabushi Human call sync failed: {error}"))?;
        let status = response.status();
        let body = response
            .text()
            .map_err(|error| format!("Fabushi Human call sync response could not be read: {error}"))?;
        let envelope: CallEnvelope = serde_json::from_str(&body).map_err(|error| {
            format!(
                "Fabushi Human call sync response was invalid JSON (HTTP {}): {error}",
                status.as_u16()
            )
        })?;
        if !status.is_success() || !envelope.success {
            return Err(format!(
                "Fabushi Human call sync was rejected (HTTP {}): {}",
                status.as_u16(),
                envelope.error.unwrap_or_else(|| "unknown backend error".into())
            ));
        }
        let call = envelope
            .call
            .ok_or_else(|| "Fabushi Human call sync response omitted the call.".to_string())?;
        if call.call_id != call_id {
            return Err("Fabushi Human call backend returned a mismatched call id".into());
        }
        let next_after_seq = envelope
            .next_after_seq
            .unwrap_or_else(|| envelope.events.last().map(|event| event.seq).unwrap_or(after_seq));
        Ok((call, envelope.events, next_after_seq))
    }

    pub fn append_human_call_event(
        &self,
        call_id: &str,
        client_event_id: &str,
        generation: u64,
        kind: &str,
        payload: &Value,
    ) -> Result<(FabushiRemoteHumanCall, FabushiRemoteHumanCallEvent), String> {
        let call_id = required_trimmed(call_id, "call id")?;
        let client_event_id = required_trimmed(client_event_id, "call event id")?;
        let kind = required_trimmed(kind, "call event kind")?;
        if !payload.is_object() {
            return Err("Fabushi Human call event payload must be an object".into());
        }
        let encoded_call_id: String =
            url::form_urlencoded::byte_serialize(call_id.as_bytes()).collect();
        let credentials = read_credentials(&self.credential_path)?;
        let response = self
            .client
            .post(self.endpoint(&format!(
                "/api/social/calls/{encoded_call_id}/events"
            ))?)
            .bearer_auth(required_trimmed(&credentials.access_token, "accessToken")?)
            .header(
                "x-fabushi-device-id",
                required_trimmed(&credentials.device_id, "deviceId")?,
            )
            .header("accept", "application/json")
            .json(&AppendCallEventRequest {
                client_event_id,
                generation,
                kind,
                payload,
            })
            .send()
            .map_err(|error| format!("Fabushi Human call event send failed: {error}"))?;
        let status = response.status();
        let body = response
            .text()
            .map_err(|error| format!("Fabushi Human call event response could not be read: {error}"))?;
        let envelope: CallEnvelope = serde_json::from_str(&body).map_err(|error| {
            format!(
                "Fabushi Human call event response was invalid JSON (HTTP {}): {error}",
                status.as_u16()
            )
        })?;
        if !status.is_success() || !envelope.success {
            return Err(format!(
                "Fabushi Human call event was rejected (HTTP {}): {}",
                status.as_u16(),
                envelope.error.unwrap_or_else(|| "unknown backend error".into())
            ));
        }
        let call = envelope
            .call
            .ok_or_else(|| "Fabushi Human call event response omitted the call.".to_string())?;
        let event = envelope
            .event
            .ok_or_else(|| "Fabushi Human call event response omitted the event.".to_string())?;
        if call.call_id != call_id || event.call_id != call_id {
            return Err("Fabushi Human call backend returned a mismatched call id".into());
        }
        if event.client_event_id != client_event_id
            || event.generation != generation
            || event.kind != kind
            || event.payload != *payload
        {
            return Err("Fabushi Human call backend returned a mismatched canonical event".into());
        }
        Ok((call, event))
    }

    fn endpoint(&self, path: &str) -> Result<Url, String> {
        self.base_url
            .join(path.trim_start_matches('/'))
            .map_err(|error| format!("Fabushi messaging endpoint is invalid: {error}"))
    }
}

fn validate_base_url(raw: &str) -> Result<Url, String> {
    let normalized = raw.trim().trim_end_matches('/');
    let mut url = Url::parse(&format!("{normalized}/"))
        .map_err(|error| format!("Fabushi messaging API base URL is invalid: {error}"))?;
    let loopback = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "::1"));
    if url.scheme() != "https" && !(loopback && url.scheme() == "http") {
        return Err("Fabushi messaging API must use HTTPS outside loopback development.".into());
    }
    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(
            "Fabushi messaging API base URL must not contain credentials, query, or fragment.".into(),
        );
    }
    url.set_query(None);
    url.set_fragment(None);
    Ok(url)
}

fn read_credentials(path: &Path) -> Result<CredentialFile, String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("Fabushi messaging credential is unavailable: {error}"))?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err("Fabushi messaging credential must be a private regular file.".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err("Fabushi messaging credential must be private to the current user.".into());
        }
    }
    serde_json::from_str(
        &fs::read_to_string(path)
            .map_err(|error| format!("Fabushi messaging credential could not be read: {error}"))?,
    )
    .map_err(|error| format!("Fabushi messaging credential JSON is invalid: {error}"))
}

fn required_trimmed<'a>(value: &'a str, name: &str) -> Result<&'a str, String> {
    let value = value.trim();
    if value.is_empty() {
        Err(format!("Fabushi messaging {name} is empty."))
    } else {
        Ok(value)
    }
}

pub fn fabushi_identity_text(value: &Value) -> Result<String, String> {
    match value {
        Value::String(value) => required_trimmed(value, "userId").map(str::to_string),
        Value::Number(value) => Ok(value.to_string()),
        _ => Err("Fabushi messaging userId is invalid.".into()),
    }
}
