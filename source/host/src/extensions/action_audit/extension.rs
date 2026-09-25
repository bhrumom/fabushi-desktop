use std::sync::Arc;

use super::action_audit_backend::ActionAuditBackend;
use super::action_audit_service::SandActionAuditor;

#[derive(Debug, Clone)]
pub struct ActionAuditExtension {
    backend: ActionAuditBackend,
}

impl ActionAuditExtension {
    pub fn new(service: Arc<SandActionAuditor>) -> Self {
        Self { backend: ActionAuditBackend::new(service) }
    }

    pub fn backend(&self) -> &ActionAuditBackend {
        &self.backend
    }

    pub fn service(&self) -> &Arc<SandActionAuditor> {
        self.backend.service()
    }
}
