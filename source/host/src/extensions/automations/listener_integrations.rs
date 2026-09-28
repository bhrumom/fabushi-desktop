use std::collections::BTreeMap;
use std::sync::Arc;

use prost::Message;
use serde_json::Value;

use super::connect_unary::{SandConnectError, SandConnectUnaryClient};

pub const DASHBOARD_INTEGRATIONS_URL: &str = "https://cursor.com/dashboard?tab=integrations";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListenerConnectionState {
    pub kind: String,
    pub connected: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ListenerIntegrations {
    states: BTreeMap<String, bool>,
}

impl ListenerIntegrations {
    pub fn set_connected(&mut self, kind: impl Into<String>, connected: bool) {
        self.states.insert(kind.into(), connected);
    }

    pub fn is_connected(&self, kind: &str) -> Option<bool> {
        self.states.get(kind).copied()
    }

    pub fn connection_states(&self) -> Vec<ListenerConnectionState> {
        self.states
            .iter()
            .map(|(kind, connected)| ListenerConnectionState {
                kind: kind.clone(),
                connected: *connected,
            })
            .collect()
    }

    pub fn listener_kind(listener: &Value) -> Option<&str> {
        match listener.get("type").and_then(Value::as_str)? {
            "slack" => Some("slack"),
            "github" => Some("github"),
            "microsoftTeams" => Some("microsoftTeams"),
            "linear" => Some("linear"),
            "sentry" => Some("sentry"),
            "pagerduty" => Some("pagerduty"),
            _ => None,
        }
    }

    pub fn listener_is_connected(&self, listener: &Value) -> Option<bool> {
        Self::listener_kind(listener).and_then(|kind| self.is_connected(kind))
    }
}

#[derive(Clone, PartialEq, Message)]
struct GetSlackUserSettingsRequest {
    #[prost(int32, tag = "1")]
    team_id: i32,
}

#[derive(Clone, PartialEq, Message)]
struct GetSlackUserSettingsResponse {
    #[prost(bool, tag = "1")]
    has_slack_auth: bool,
}

#[derive(Clone, PartialEq, Message)]
struct GetScmConnectionStatusRequest {
    #[prost(string, optional, tag = "1")]
    ghe_application: Option<String>,
    #[prost(int32, optional, tag = "2")]
    team_id: Option<i32>,
}

#[derive(Clone, PartialEq, Message)]
struct GetScmConnectionStatusResponse {
    #[prost(bool, tag = "1")]
    connected: bool,
}

#[derive(Clone, PartialEq, Message)]
struct GetSlackInstallUrlRequest {
    #[prost(int32, tag = "1")]
    team_id: i32,
    #[prost(string, optional, tag = "2")]
    repo_url: Option<String>,
}

#[derive(Clone, PartialEq, Message)]
struct GetSlackInstallUrlResponse {
    #[prost(string, tag = "1")]
    url: String,
}

pub trait PlatformConnectionReader: Send + Sync {
    fn is_platform_connected(&self, platform: &str) -> Result<bool, SandConnectError>;
}

pub struct ProductionListenerIntegrations {
    client: Arc<SandConnectUnaryClient>,
}

impl ProductionListenerIntegrations {
    pub fn new(client: Arc<SandConnectUnaryClient>) -> Self {
        Self { client }
    }

    fn read_platform_connected(&self, platform: &str) -> Result<bool, SandConnectError> {
        match platform {
            "slack" => self
                .client
                .unary::<_, GetSlackUserSettingsResponse>(
                    "aiserver.v1.DashboardService",
                    "GetSlackUserSettings",
                    &GetSlackUserSettingsRequest { team_id: 0 },
                )
                .map(|response| response.has_slack_auth),
            "github" => self
                .client
                .unary::<_, GetScmConnectionStatusResponse>(
                    "aiserver.v1.DashboardService",
                    "GetScmConnectionStatus",
                    &GetScmConnectionStatusRequest {
                        ghe_application: None,
                        team_id: None,
                    },
                )
                .map(|response| response.connected),
            other => Err(SandConnectError::Configuration(format!(
                "unsupported listener integration platform {other}"
            ))),
        }
    }

    pub fn get_connect_url(&self, platform: &str) -> String {
        if platform != "slack" {
            return DASHBOARD_INTEGRATIONS_URL.to_string();
        }
        self.client
            .unary::<_, GetSlackInstallUrlResponse>(
                "aiserver.v1.DashboardService",
                "GetSlackInstallUrl",
                &GetSlackInstallUrlRequest {
                    team_id: 0,
                    repo_url: None,
                },
            )
            .ok()
            .map(|response| response.url)
            .filter(|url| !url.trim().is_empty())
            .unwrap_or_else(|| DASHBOARD_INTEGRATIONS_URL.to_string())
    }
}

impl PlatformConnectionReader for ProductionListenerIntegrations {
    fn is_platform_connected(&self, platform: &str) -> Result<bool, SandConnectError> {
        self.read_platform_connected(platform)
    }
}
