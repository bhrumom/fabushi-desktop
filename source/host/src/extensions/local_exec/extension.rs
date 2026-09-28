use std::sync::{Arc, mpsc::Sender};

use serde_json::Value;

use crate::extensions::extension_ids_generated::HostExtensionId;
use crate::gateway_server::{GatewayBridgeClose, GatewayBridgeHub};
use crate::extensions::telemetry::host_telemetry_service::HostStructuredLogTelemetry;
use crate::extensions::telemetry::local_exec_telemetry::{
    LocalExecProviderReport, LocalExecRefusalCause, LocalExecRefusedReport,
    local_exec_provider_telemetry, local_exec_refused_telemetry,
};

use super::local_exec_bridge::{
    LocalExecComputer, LocalExecProviderInfo, LocalExecProviderLifecycleReport,
    LocalExecProviderRegistration, SandLocalExecBridge,
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
/// The production Gateway provider channel is live and reports the frozen
/// provider lifecycle telemetry. Permission-authorized exec streaming and the
/// Runner's external-shell/file-transfer consumers remain separate mapped work.
#[derive(Clone)]
pub struct HostLocalExecExtension {
    bridge: SandLocalExecBridge,
    logs: HostStructuredLogTelemetry,
}

impl HostLocalExecExtension {
    pub fn register_provider(&self, send: Sender<Value>) -> LocalExecProviderRegistration {
        self.bridge.register_provider(send)
    }

    pub fn submit_responses(&self, batch: Value) {
        self.bridge.submit_responses(batch);
    }

    pub fn check_live_computer_for_ask(&self, agent_id: Option<&str>) -> bool {
        if self.bridge.check_live_computer_for_ask() {
            return true;
        }

        let provider_count = self.bridge.provider_count();
        let report = LocalExecRefusedReport {
            cause: if provider_count == 0 {
                LocalExecRefusalCause::NoProviders
            } else {
                LocalExecRefusalCause::StaleHeartbeat
            },
            site: "ask_gate".into(),
            conversation_id: agent_id.unwrap_or_default().to_string(),
            provider_count: provider_count.try_into().unwrap_or(i64::MAX),
            live_provider_count: 0,
            ever_registered: self.bridge.ever_registered(),
            empty_for_ms: None,
        };
        let _ = self.logs.report_projection(&local_exec_refused_telemetry(&report));
        false
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

pub fn start_local_exec_extension(
    logs: HostStructuredLogTelemetry,
) -> HostLocalExecExtension {
    let provider_logs = logs.clone();
    let provider_reporter = Arc::new(move |report: LocalExecProviderLifecycleReport| {
        let telemetry = match report {
            LocalExecProviderLifecycleReport::Registered {
                provider_id,
                provider_count,
            } => LocalExecProviderReport::Registered {
                provider_id,
                provider_count: provider_count.try_into().unwrap_or(i64::MAX),
            },
            LocalExecProviderLifecycleReport::Hello {
                provider_id,
                provider_count,
                hello_delay_ms,
                computer_id_present,
                rehello,
                supervised,
                variant,
            } => LocalExecProviderReport::Hello {
                provider_id,
                provider_count: provider_count.try_into().unwrap_or(i64::MAX),
                hello_delay_ms: hello_delay_ms as f64,
                computer_id_present,
                rehello,
                supervised,
                variant,
            },
            LocalExecProviderLifecycleReport::Detached {
                provider_id,
                provider_count,
                age_ms,
                had_hello,
                has_heartbeat,
                was_live,
                emptied,
            } => LocalExecProviderReport::Detached {
                provider_id,
                provider_count: provider_count.try_into().unwrap_or(i64::MAX),
                age_ms: age_ms as f64,
                had_hello,
                has_heartbeat,
                was_live,
                emptied,
            },
        };
        let _ = provider_logs.report_projection(&local_exec_provider_telemetry(&telemetry));
    });
    HostLocalExecExtension {
        bridge: SandLocalExecBridge::production_with_reporter(Some(provider_reporter)),
        logs,
    }
}
