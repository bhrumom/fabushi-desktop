use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::Value;
use std::sync::Arc;

pub const XUSER_RELAY_POLL_INTERVAL_MS: u64 = 4_000;
pub const XUSER_RELAY_ERROR_BACKOFF_MS: u64 = 30_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandXuserRelayHttpError {
    pub status: u16,
    pub path: String,
}
impl std::fmt::Display for SandXuserRelayHttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sand sharing {} returned {}", self.path, self.status)
    }
}
impl std::error::Error for SandXuserRelayHttpError {}

pub trait XuserRelayTransport: Send + Sync {
    fn request(
        &self,
        method: &str,
        url: &str,
        token: &str,
        body: Option<&Value>,
    ) -> Result<Value, SandXuserRelayHttpError>;
}

pub struct ReqwestXuserRelayTransport {
    client: Client,
}
impl ReqwestXuserRelayTransport {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            client: Client::builder().build().map_err(|e| e.to_string())?,
        })
    }
}
impl XuserRelayTransport for ReqwestXuserRelayTransport {
    fn request(
        &self,
        method: &str,
        url: &str,
        token: &str,
        body: Option<&Value>,
    ) -> Result<Value, SandXuserRelayHttpError> {
        let mut req = match method {
            "GET" => self.client.get(url),
            "POST" => self.client.post(url),
            "DELETE" => self.client.delete(url),
            _ => self.client.post(url),
        }
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .header(CONTENT_TYPE, "application/json");
        if let Some(body) = body {
            req = req.json(body);
        }
        let response = req.send().map_err(|_| SandXuserRelayHttpError {
            status: 0,
            path: url.into(),
        })?;
        let status = response.status().as_u16();
        if !(200..300).contains(&status) {
            return Err(SandXuserRelayHttpError {
                status,
                path: url.into(),
            });
        }
        response
            .json::<Value>()
            .map_err(|_| SandXuserRelayHttpError {
                status,
                path: url.into(),
            })
    }
}

pub struct SandXuserRelayClient {
    backend_url: String,
    access_token: Arc<dyn Fn() -> Result<String, String> + Send + Sync>,
    transport: Arc<dyn XuserRelayTransport>,
}
impl SandXuserRelayClient {
    pub fn new(
        backend_url: String,
        access_token: Arc<dyn Fn() -> Result<String, String> + Send + Sync>,
        transport: Arc<dyn XuserRelayTransport>,
    ) -> Self {
        Self {
            backend_url: backend_url.trim_end_matches('/').into(),
            access_token,
            transport,
        }
    }
    fn call(&self, method: &str, path: &str, body: Option<&Value>) -> Result<Value, String> {
        let token = (self.access_token)()?;
        self.transport
            .request(
                method,
                &format!("{}{}", self.backend_url, path),
                &token,
                body,
            )
            .map_err(|e| describe_relay_error(&e))
    }
    pub fn fetch_share_state(&self) -> Result<Value, String> {
        self.call("GET", "/api/sand/xuser/state", None)
    }
    pub fn send(&self, payload: &Value) -> Result<Value, String> {
        self.call("POST", "/api/sand/xuser/relay", Some(payload))
    }
    pub fn create_room(&self, payload: &Value) -> Result<Value, String> {
        self.call("POST", "/api/sand/xuser/rooms", Some(payload))
    }
    pub fn leave_room(&self, room_id: &str) -> Result<Value, String> {
        self.call(
            "POST",
            &format!("/api/sand/xuser/rooms/{room_id}/leave"),
            Some(&serde_json::json!({})),
        )
    }
    pub fn remove_deleted_agent(&self, agent_id: &str) -> Result<Value, String> {
        self.call(
            "POST",
            "/api/sand/xuser/remove-agent",
            Some(&serde_json::json!({"agentId":agent_id})),
        )
    }
}

pub fn describe_relay_error(error: &SandXuserRelayHttpError) -> String {
    match error.status {
        403 => "Sharing isn't enabled for your account.".into(),
        429 => "You're doing that too often. Try again in a minute.".into(),
        _ => "Couldn't reach the sharing service. Check your connection and try again.".into(),
    }
}
