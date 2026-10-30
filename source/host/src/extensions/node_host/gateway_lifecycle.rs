//! Single Host-owned lifecycle composition for an explicit NodeHost Gateway config.
//!
//! This owner deliberately returns retry decisions to its caller. It does not
//! discover endpoints, create a background timer, register startup work, or
//! execute node side effects.

use std::sync::Arc;

use super::gateway_transport::{
    NodeHostGatewaySession, NodeHostGatewayTransport, NodeHostGatewayTransportConfig,
    NodeHostGatewayTransportError,
};
use super::reconnect::{NodeHostReconnectDecision, NodeHostReconnectPolicy};
use super::runtime_state::NodeHostRuntimeState;

pub struct NodeHostGatewayLifecycle {
    runtime: Arc<NodeHostRuntimeState>,
    policy: NodeHostReconnectPolicy,
    session: Option<NodeHostGatewaySession>,
    last_decision: Option<NodeHostReconnectDecision>,
}
impl NodeHostGatewayLifecycle {
    pub fn new(runtime: Arc<NodeHostRuntimeState>) -> Self {
        Self {
            runtime,
            policy: NodeHostReconnectPolicy::new(),
            session: None,
            last_decision: None,
        }
    }

    /// Replace the current session only after its owned transport has closed.
    /// The config must be supplied explicitly by an approved caller.
    pub fn connect_once(
        &mut self,
        config: NodeHostGatewayTransportConfig,
    ) -> Result<(), NodeHostGatewayTransportError> {
        if let Some(session) = self.session.take() {
            session.close()?;
        }
        match NodeHostGatewayTransport::connect_with_runtime(config, Arc::clone(&self.runtime)) {
            Ok(session) => {
                self.policy.on_hello();
                self.last_decision = None;
                self.session = Some(session);
                Ok(())
            }
            Err(error) => {
                self.last_decision = Some(self.policy.observe(&error));
                Err(error)
            }
        }
    }

    pub fn session(&self) -> Option<&NodeHostGatewaySession> {
        self.session.as_ref()
    }

    pub fn session_mut(&mut self) -> Option<&mut NodeHostGatewaySession> {
        self.session.as_mut()
    }

    pub fn reconnect_decision(&self) -> Option<NodeHostReconnectDecision> {
        self.last_decision
    }

    /// Record a later read/write failure and return the bounded caller decision.
    pub fn observe_error(
        &mut self,
        error: &NodeHostGatewayTransportError,
    ) -> NodeHostReconnectDecision {
        let decision = self.policy.observe(error);
        self.last_decision = Some(decision);
        decision
    }

    pub fn close(&mut self) -> Result<(), NodeHostGatewayTransportError> {
        let Some(session) = self.session.take() else {
            return Ok(());
        };
        session.close()
    }
}
