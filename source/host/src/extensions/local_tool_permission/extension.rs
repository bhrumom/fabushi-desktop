use std::sync::Arc;

use crate::extensions::extension_ids_generated::HostExtensionId;
use crate::extensions::settings::settings_service::SettingsService;

use super::local_tool_permission_controller::{
    SandLocalToolAskRequest, SandLocalToolDecision, SandLocalToolPermissionController,
    SandLocalToolRequest, SandLocalToolScope,
};

pub const LOCAL_TOOL_PERMISSION_DEPENDENCIES: &[HostExtensionId] = &[
    HostExtensionId::Settings,
    HostExtensionId::Telemetry,
    HostExtensionId::Transcript,
];

pub fn local_tool_permission_extension_id() -> HostExtensionId {
    HostExtensionId::LocalToolPermission
}

#[derive(Clone)]
pub struct HostLocalToolPermissionExtension {
    controller: Arc<SandLocalToolPermissionController>,
}

impl HostLocalToolPermissionExtension {
    pub fn controller(&self) -> Arc<SandLocalToolPermissionController> {
        Arc::clone(&self.controller)
    }

    pub fn blocked_reason(&self) -> Option<String> {
        self.controller.blocked_reason()
    }

    pub fn requires_approval(&self) -> bool {
        self.controller.requires_approval()
    }

    pub fn authorize(
        &self,
        scope: Option<&SandLocalToolScope>,
        request: &SandLocalToolRequest,
    ) -> SandLocalToolDecision {
        self.controller.authorize(scope, request)
    }

    pub fn bind_ask_surfaces(
        &self,
        provider: Arc<dyn Fn(&str) -> bool + Send + Sync>,
    ) {
        self.controller.bind_ask_surfaces(provider);
    }

    pub fn bind_live_computer_check(
        &self,
        provider: Arc<dyn Fn(&str) -> bool + Send + Sync>,
    ) {
        self.controller.bind_live_computer_check(provider);
    }

    pub fn bind_event_sink(
        &self,
        sink: Option<Arc<dyn Fn(SandLocalToolAskRequest) + Send + Sync>>,
    ) {
        self.controller.bind_event_sink(sink);
    }
}

pub fn start_local_tool_permission_extension(
    settings: Arc<SettingsService>,
) -> HostLocalToolPermissionExtension {
    HostLocalToolPermissionExtension {
        controller: Arc::new(SandLocalToolPermissionController::production(settings)),
    }
}
