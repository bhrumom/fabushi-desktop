use std::collections::HashMap;

use serde_json::Value;

use crate::gateway::gateway_errors::SandGatewayCommandError;
use crate::gateway::gateway_reachability::ReachabilityOutcome;
use crate::protocol::{Failure, ReplyOutcome, COORDINATOR_UNKNOWN_METHOD};

pub const GATEWAY_COMMAND_FAILED: &str = "gateway-command-failed";
pub const GATEWAY_UNREACHABLE: &str = "gateway-unreachable";
pub const GATEWAY_TRANSPORT_FAILED: &str = "gateway-transport-failed";

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GatewayDispatchError {
    #[error("{0}")]
    Command(#[from] SandGatewayCommandError),
    #[error("{message}")]
    Unreachable {
        outcome: ReachabilityOutcome,
        message: String,
    },
    #[error("{0}")]
    Transport(String),
}

pub fn failure_for(error: &GatewayDispatchError) -> Failure {
    match error {
        GatewayDispatchError::Command(error) => {
            Failure::new(GATEWAY_COMMAND_FAILED, error.message.clone())
        }
        GatewayDispatchError::Unreachable { outcome, message } => Failure::with_transport_kind(
            GATEWAY_UNREACHABLE,
            message.clone(),
            outcome.transport_kind(),
        ),
        GatewayDispatchError::Transport(message) => {
            Failure::new(GATEWAY_TRANSPORT_FAILED, message.clone())
        }
    }
}

pub type GatewayHandler =
    Box<dyn Fn(Value) -> Result<Value, GatewayDispatchError> + Send + Sync>;

#[derive(Default)]
pub struct GatewayRequestDispatcher {
    handlers: HashMap<String, GatewayHandler>,
}

impl std::fmt::Debug for GatewayRequestDispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GatewayRequestDispatcher")
            .field("method_count", &self.handlers.len())
            .finish()
    }
}

impl GatewayRequestDispatcher {
    pub fn register(&mut self, method: impl Into<String>, handler: GatewayHandler) {
        self.handlers.insert(method.into(), handler);
    }

    pub fn serves(&self, method: &str) -> bool {
        self.handlers.contains_key(method)
    }

    pub fn dispatch_reply(&self, method: &str, args: Value) -> ReplyOutcome {
        let Some(handler) = self.handlers.get(method) else {
            return ReplyOutcome::Failed {
                failure: Failure::new(
                    COORDINATOR_UNKNOWN_METHOD,
                    format!("no coordinator method named {method}"),
                ),
            };
        };

        match handler(args) {
            Ok(value) => ReplyOutcome::Ok { value },
            Err(error) => ReplyOutcome::Failed {
                failure: failure_for(&error),
            },
        }
    }

    pub fn dispatch(&self, method: &str, args: Value) -> Result<Value, Failure> {
        match self.dispatch_reply(method, args) {
            ReplyOutcome::Ok { value } => Ok(value),
            ReplyOutcome::Failed { failure } => Err(failure),
        }
    }
}
