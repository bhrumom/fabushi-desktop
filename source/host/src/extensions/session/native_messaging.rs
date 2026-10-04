use std::env;
use std::fs;
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
struct SendRequest<'a> {
    target_user_id: &'a str,
    text: &'a str,
    client_request_id: &'a str,
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
            user_id: value_identity(&credentials.user_id)?,
            device_id: required_trimmed(&credentials.device_id, "deviceId")?.to_string(),
        })
    }

    pub fn send_direct_message(
        &self,
        peer_human_id: &str,
        text: &str,
        client_request_id: &str,
    ) -> Result<FabushiRemoteHumanMessage, String> {
        let peer_human_id = required_trimmed(peer_human_id, "peer Human id")?;
        let text = required_trimmed(text, "message text")?;
        let client_request_id = required_trimmed(client_request_id, "client request id")?;
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

fn value_identity(value: &Value) -> Result<String, String> {
    match value {
        Value::String(value) => required_trimmed(value, "userId").map(str::to_string),
        Value::Number(value) => Ok(value.to_string()),
        _ => Err("Fabushi messaging userId is invalid.".into()),
    }
}
