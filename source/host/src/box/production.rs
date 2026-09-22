use std::env;

use super::box_env::{
    BoxEnvironmentUpdate, apply_box_environment_via_transport,
};
use super::box_remote_accessor::BoxEndpoint;
use super::generated_production::{
    ProductionBoxTransport, ProductionBoxTransportError,
    create_production_box_control_client,
};

pub const EXEC_DAEMON_PORT: u16 = 1337;
pub const DEFAULT_AUTH_TOKEN: &str = "local";
pub const BOX_APPLY_ENVIRONMENT_GATEWAY_METHOD: &str = "box.applyEnvironment";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionBoxEnvironment {
    transport: ProductionBoxTransport,
}

impl ProductionBoxEnvironment {
    pub fn new(
        host: impl Into<String>,
        port: u16,
        auth_token: impl Into<String>,
    ) -> Self {
        let endpoint = BoxEndpoint::new(host, port, auth_token);
        Self {
            transport: ProductionBoxTransport::from_endpoint(&endpoint),
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
        Self::new(host, port, auth_token)
    }

    pub fn transport(&self) -> &ProductionBoxTransport {
        &self.transport
    }

    pub fn apply_environment(
        &self,
        update: &BoxEnvironmentUpdate,
    ) -> Result<(), ProductionBoxTransportError> {
        apply_box_environment_via_transport(
            &(),
            &self.transport,
            update,
            create_production_box_control_client,
        )
    }
}
