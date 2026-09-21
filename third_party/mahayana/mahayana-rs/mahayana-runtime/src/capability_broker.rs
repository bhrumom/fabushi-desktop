use crate::runtime_store::RuntimeStore;
use mahayana_core::capability::{
    CapabilityAuditRecord, CapabilityAvailability, CapabilityDescriptor, CapabilityPolicyDecision,
    CapabilityRequest,
};
use std::sync::Arc;

pub struct CapabilityBroker {
    store: Arc<RuntimeStore>,
}

impl CapabilityBroker {
    pub fn new(store: Arc<RuntimeStore>) -> Self {
        Self { store }
    }

    pub fn authorize(
        &self,
        descriptor: &CapabilityDescriptor,
        request: CapabilityRequest,
        now_ms: i64,
    ) -> Result<CapabilityPolicyDecision, String> {
        self.authorize_request(
            descriptor.availability,
            descriptor.unavailable_reason.clone(),
            request,
            now_ms,
        )
    }

    pub fn authorize_request(
        &self,
        availability: CapabilityAvailability,
        unavailable_reason: Option<String>,
        request: CapabilityRequest,
        now_ms: i64,
    ) -> Result<CapabilityPolicyDecision, String> {
        let (decision, reason) = match availability {
            CapabilityAvailability::Ready => (CapabilityPolicyDecision::Allow, None),
            CapabilityAvailability::PermissionRequired => (
                CapabilityPolicyDecision::NeedsUser,
                Some("capability requires explicit user permission".to_string()),
            ),
            CapabilityAvailability::Unavailable => (
                CapabilityPolicyDecision::Deny,
                Some(
                    unavailable_reason
                        .unwrap_or_else(|| "capability is unavailable".to_string()),
                ),
            ),
        };
        self.store
            .append_audit(&CapabilityAuditRecord {
                request,
                decision,
                decided_at_ms: now_ms,
                reason,
            })
            .map_err(|error| error.to_string())?;
        Ok(decision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mahayana_core::{ConversationId, RunId};
    use serde_json::json;

    fn request(capability: &str) -> CapabilityRequest {
        CapabilityRequest {
            actor: "agent:test".to_string(),
            agent_id: Some("test".to_string()),
            conversation_id: ConversationId::new("mahayana-ai:capability:test").unwrap(),
            run_id: Some(RunId::new("run:capability:test").unwrap()),
            capability: capability.to_string(),
            target: json!({"scope": "test"}),
            intent: "structural ownership test".to_string(),
        }
    }

    #[test]
    fn broker_is_the_single_policy_mapping_for_ready_permission_and_unavailable() {
        let store = Arc::new(RuntimeStore::open(None).unwrap());
        let broker = CapabilityBroker::new(store);

        assert_eq!(
            broker.authorize_request(
                CapabilityAvailability::Ready,
                None,
                request("filesystem.agent.read"),
                1,
            ).unwrap(),
            CapabilityPolicyDecision::Allow,
        );
        assert_eq!(
            broker.authorize_request(
                CapabilityAvailability::PermissionRequired,
                None,
                request("computer.input.control"),
                2,
            ).unwrap(),
            CapabilityPolicyDecision::NeedsUser,
        );
        assert_eq!(
            broker.authorize_request(
                CapabilityAvailability::Unavailable,
                Some("disabled for test".to_string()),
                request("shell.execute"),
                3,
            ).unwrap(),
            CapabilityPolicyDecision::Deny,
        );
    }
}
