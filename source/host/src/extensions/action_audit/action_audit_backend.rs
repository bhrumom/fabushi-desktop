use std::sync::Arc;

use super::action_audit_service::SandActionAuditor;

#[derive(Debug, Clone)]
pub struct ActionAuditBackend {
    service: Arc<SandActionAuditor>,
}

impl ActionAuditBackend {
    pub fn new(service: Arc<SandActionAuditor>) -> Self {
        Self { service }
    }

    pub fn service(&self) -> &Arc<SandActionAuditor> {
        &self.service
    }
}
