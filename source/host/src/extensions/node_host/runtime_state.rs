//! Canonical Host composition for node sessions and invocation generations.
//!
//! This is a narrow owner boundary: it couples session retirement to
//! invocation draining without owning sockets, credentials, or side effects.

use super::invocation_registry::{NodeHostInvocationRegistry, NodeInvocationAdmission};
use super::session_registry::{
    NodeHostSessionLease, NodeHostSessionMetadata, NodeHostSessionRegistry,
    NodeHostSessionSnapshot,
};
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct NodeHostRuntimeState {
    sessions: Arc<NodeHostSessionRegistry>,
    invocations: Arc<NodeHostInvocationRegistry>,
}

impl NodeHostRuntimeState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn connect(&self, metadata: NodeHostSessionMetadata) -> NodeHostSessionLease {
        if let Some(previous) = self.sessions.get(&metadata.node_id) {
            let _ = self
                .invocations
                .drain_generation(&previous.node_id, previous.generation);
        }
        self.sessions.connect(metadata)
    }

    pub fn disconnect(&self, lease: &NodeHostSessionLease) -> Vec<String> {
        let drained = self
            .invocations
            .drain_generation(&lease.node_id, lease.generation);
        let _ = self.sessions.disconnect(lease);
        drained
    }

    pub fn set_active(&self, lease: &NodeHostSessionLease, active: bool) -> bool {
        self.sessions.set_active(lease, active)
    }

    pub fn get(&self, node_id: &str) -> Option<NodeHostSessionSnapshot> {
        self.sessions.get(node_id)
    }

    pub fn list(&self) -> Vec<NodeHostSessionSnapshot> {
        self.sessions.list()
    }

    pub fn admit_invocation(
        &self,
        lease: &NodeHostSessionLease,
        invoke_id: impl Into<String>,
    ) -> Result<NodeInvocationAdmission, String> {
        self.invocations.admit(lease, invoke_id)
    }

    pub fn settle_invocation(&self, admission: &NodeInvocationAdmission) -> bool {
        self.invocations.settle(admission)
    }
}

#[cfg(test)]
mod tests {
    use super::{NodeHostRuntimeState, NodeHostSessionMetadata};

    fn metadata(node_id: &str) -> NodeHostSessionMetadata {
        NodeHostSessionMetadata {
            node_id: node_id.into(),
            display_name: node_id.into(),
            platform: "test".into(),
            device_family: "test".into(),
            commands: vec!["system.run".into()],
            capabilities: vec![],
            paired: true,
            gateway_local: false,
        }
    }

    #[test]
    fn replacement_drains_predecessor_invocations_before_successor() {
        let runtime = NodeHostRuntimeState::new();
        let previous = runtime.connect(metadata("node-1"));
        let admission = runtime.admit_invocation(&previous, "invoke-1").expect("admit");
        let successor = runtime.connect(metadata("node-1"));
        assert!(!runtime.settle_invocation(&admission));
        assert!(runtime.admit_invocation(&successor, "invoke-1").is_ok());
    }
}
