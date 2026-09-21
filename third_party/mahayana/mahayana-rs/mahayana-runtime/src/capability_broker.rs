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
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_store_dir(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "mahayana-capability-broker-{name}-{}-{nonce}",
            std::process::id()
        ))
    }

    fn request(capability: &str) -> CapabilityRequest {
        CapabilityRequest {
            actor: "agent:test".to_string(),
            agent_id: Some("test".to_string()),
            conversation_id: ConversationId::new("conversation:test").unwrap(),
            run_id: Some(RunId::new("run:test").unwrap()),
            capability: capability.to_string(),
            target: json!({"resource": "fixture"}),
            intent: "exercise capability policy".to_string(),
        }
    }

    #[test]
    fn broker_maps_availability_to_policy_and_audits_every_decision() {
        let dir = temp_store_dir("decisions");
        let store = Arc::new(RuntimeStore::open(Some(&dir)).unwrap());
        let broker = CapabilityBroker::new(Arc::clone(&store));

        assert_eq!(
            broker
                .authorize_request(CapabilityAvailability::Ready, None, request("ready"), 1)
                .unwrap(),
            CapabilityPolicyDecision::Allow
        );
        assert_eq!(
            broker
                .authorize_request(
                    CapabilityAvailability::PermissionRequired,
                    None,
                    request("ask"),
                    2,
                )
                .unwrap(),
            CapabilityPolicyDecision::NeedsUser
        );
        assert_eq!(
            broker
                .authorize_request(
                    CapabilityAvailability::Unavailable,
                    Some("fixture unavailable".to_string()),
                    request("deny"),
                    3,
                )
                .unwrap(),
            CapabilityPolicyDecision::Deny
        );
        assert_eq!(store.capability_audit_count().unwrap(), 3);

        drop(broker);
        drop(store);
        let _ = std::fs::remove_dir_all(dir);
    }
}
