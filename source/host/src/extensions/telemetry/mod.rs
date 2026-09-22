use std::collections::BTreeMap;

pub mod auto_review_approval_telemetry;
pub mod disk_pressure_telemetry;
pub mod host_diagnostic_telemetry;
pub mod host_event_bus_telemetry;
pub mod search_index_health_telemetry;
pub mod send_trace_sampler;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostTelemetryProjection {
    pub level: Option<&'static str>,
    pub event: Option<&'static str>,
    pub metadata: BTreeMap<String, String>,
}

pub mod automation_shadow_prune_telemetry;
pub mod box_log_ship_telemetry;
pub mod experiments_diagnostic_telemetry;
pub mod host_extension_diagnostic_telemetry;
pub mod revival_telemetry_mappers;
pub mod session_diagnostic_telemetry;
pub mod turn_empty_delivery_telemetry;
