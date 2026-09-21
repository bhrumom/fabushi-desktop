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