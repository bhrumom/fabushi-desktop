use std::sync::{Arc, mpsc::Sender};

use serde_json::Value;

use crate::extensions::extension_ids_generated::HostExtensionId;
use crate::gateway_server::{GatewayBridgeClose, GatewayBridgeHub};

use super::local_exec_bridge::{
    LocalExecComputer, LocalExecProviderRegistration, LocalExecProviderInfo,
    SandLocalExecBridge,
};

pub const LOCAL_EXEC_DEPENDENCIES: &[HostExtensionId] = &[
    HostExtensionId::LocalToolPermission,
    HostExtensionId::Telemetry,
];

pub fn local_exec_extension_id() -> HostExtensionId {
    HostExtensionId::LocalExec
}

/// Host owner for the desktop local-exec provider transport.
///
/// The production Gateway provider channel is live in this slice. Permission
/// resolution, generated exec protobuf decoding and the SandBox/user-computer
/// adapters remain separate mapped modules and intentionally stay non-final.
#[derive(Clone)]
pub struct HostLocalExecExtension {
    bridge: SandLocalExecBridge,
}

impl HostLocalExecExtension {
    pub fn register_provider(&self, send: Sender<Value>) -> LocalExecProviderRegistration {
        self.bridge.register_provider(send)
    }

    pub fn submit_responses(&self, batch: Value) {
        self.bridge.submit_responses(batch);
    }

    pub fn check_live_computer_for_ask(&self) -> bool {
        self.bridge.check_live_computer_for_ask()
    }

    pub fn list_computers(&self) -> Vec<LocalExecComputer> {
        self.bridge.list_computers()
    }

    pub fn active_computer(&self) -> Option<LocalExecComputer> {
        self.bridge.active_computer()
    }

    pub fn provider_info(&self) -> Option<LocalExecProviderInfo> {
        self.bridge.get_provider_info()
    }

    pub fn retire_approval(&self, approval_id: &str) {
        self.bridge.retire_approval(approval_id);
    }

    pub fn bridge(&self) -> SandLocalExecBridge {
        self.bridge.clone()
    }

    pub fn gateway_bridge(self: &Arc<Self>) -> GatewayBridgeHub {
        let registration_owner = Arc::clone(self);
        let response_owner = Arc::clone(self);
        GatewayBridgeHub::with_handlers(
            move |send| {
                let registration = registration_owner.register_provider(send);
                Box::new(move || drop(registration)) as GatewayBridgeClose
            },
            move |batch| response_owner.submit_responses(batch),
        )
    }
}

pub fn start_local_exec_extension() -> HostLocalExecExtension {
    HostLocalExecExtension {
        bridge: SandLocalExecBridge::production(),
    }
}
