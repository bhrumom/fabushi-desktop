use std::env;

use super::box_env::BoxEnvironmentUpdate;
use super::box_factory::{SandBoxComposition, apply_shared_desktop, create_sand_box};
use super::box_remote_accessor::BoxEndpoint;
use super::generated_production::{
    ProductionBoxResourceAccessor, ProductionBoxTransport,
    create_production_box_resource_accessor,
};
use super::loopback_sand_box::{
    DEFAULT_AUTH_TOKEN, EXEC_DAEMON_PORT, LoopbackReady, LoopbackSandBox,
    LoopbackSandBoxError, LoopbackSandBoxOptions,
};

pub const BOX_APPLY_ENVIRONMENT_GATEWAY_METHOD: &str = "box.applyEnvironment";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionBoxEnvironment {
    transport: ProductionBoxTransport,
    composition: SandBoxComposition,
}

impl ProductionBoxEnvironment {
    pub fn new(
        host: impl Into<String>,
        port: u16,
        auth_token: impl Into<String>,
    ) -> Self {
        let host = host.into();
        let auth_token = auth_token.into();
        let endpoint = BoxEndpoint::new(host.clone(), port, auth_token.clone());
        Self::new_with_shared_desktop(host, port, auth_token, false)
    }

    pub fn new_with_shared_desktop(
        host: impl Into<String>,
        port: u16,
        auth_token: impl Into<String>,
        shared_desktop: bool,
    ) -> Self {
        let host = host.into();
        let auth_token = auth_token.into();
        let endpoint = BoxEndpoint::new(host.clone(), port, auth_token.clone());
        let loopback = create_sand_box(LoopbackSandBoxOptions {
            host,
            auth_token,
            exec_daemon_port: port,
            ..LoopbackSandBoxOptions::default()
        });
        Self {
            transport: ProductionBoxTransport::from_endpoint(&endpoint),
            composition: apply_shared_desktop(loopback, shared_desktop, true),
        }
    }

    pub fn from_process_env() -> Self {
        let host = env::var("SAND_BOX_HOST")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "127.0.0.1".into());
        let port = env::var("SAND_BOX_EXEC_DAEMON_PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(EXEC_DAEMON_PORT);
        let auth_token = env::var("SAND_BOX_AUTH_TOKEN")
            .ok()
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| DEFAULT_AUTH_TOKEN.into());
        let shared_desktop = env::var("SAND_SHARED_DESKTOP")
            .ok()
            .map(|value| value.trim().to_ascii_lowercase())
            .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"));
        Self::new_with_shared_desktop(host, port, auth_token, shared_desktop)
    }

    pub fn transport(&self) -> &ProductionBoxTransport {
        &self.transport
    }

    pub fn loopback(&self) -> &LoopbackSandBox {
        self.composition.loopback()
    }

    pub fn shared_desktop(&self) -> Option<&super::shared_desktop_sand_box::SharedDesktopSandBox> {
        self.composition.shared_desktop()
    }

    pub fn remote_resource_accessor(&self) -> ProductionBoxResourceAccessor {
        create_production_box_resource_accessor(&self.transport)
    }

    pub fn ensure_ready(
        &self,
        agent_id: &str,
    ) -> Result<LoopbackReady, LoopbackSandBoxError> {
        self.composition.ensure_ready(&(), agent_id)
    }

    pub fn load_mcp_servers(
        &self,
        config_json: &str,
    ) -> Result<Vec<String>, LoopbackSandBoxError> {
        self.composition.load_mcp_servers(&(), config_json)
    }

    pub fn apply_environment(
        &self,
        update: &BoxEnvironmentUpdate,
    ) -> Result<(), LoopbackSandBoxError> {
        self.composition.apply_environment(&(), update)
    }
}
