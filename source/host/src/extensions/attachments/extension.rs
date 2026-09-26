use std::sync::Arc;

use serde_json::Value;

use crate::extensions::auth::extension::HostAuthExtension;
use crate::extensions::extension_ids_generated::HostExtensionId;
use crate::extensions::forever_box::ForeverBoxService;

use super::attachments_service::{
    AttachmentDiagnosticReporter, AttachmentsBox, AttachmentsService,
};
use super::generate_image_service::GenerateImageAuth;

pub const ATTACHMENTS_EXTENSION_ID: HostExtensionId = HostExtensionId::Attachments;
pub const ATTACHMENTS_DEPENDENCIES: &[HostExtensionId] = &[
    HostExtensionId::Auth,
    HostExtensionId::ForeverBox,
    HostExtensionId::Telemetry,
];

#[derive(Clone)]
pub struct HostAttachmentsExtension {
    service: Arc<AttachmentsService>,
}

impl HostAttachmentsExtension {
    pub fn new(service: Arc<AttachmentsService>) -> Self {
        Self { service }
    }

    pub fn service(&self) -> Arc<AttachmentsService> {
        Arc::clone(&self.service)
    }

    pub fn dispatch_gateway(
        &self,
        method: &str,
        args: &Value,
    ) -> Option<Result<Value, String>> {
        self.service.dispatch_gateway(method, args)
    }
}

pub fn start_attachments_extension(
    auth: Arc<HostAuthExtension>,
    box_: Arc<ForeverBoxService>,
    report: Option<AttachmentDiagnosticReporter>,
) -> HostAttachmentsExtension {
    let auth: Arc<dyn GenerateImageAuth> = auth;
    let box_: Arc<dyn AttachmentsBox> = box_;
    HostAttachmentsExtension::new(Arc::new(AttachmentsService::production(
        auth, box_, report,
    )))
}
