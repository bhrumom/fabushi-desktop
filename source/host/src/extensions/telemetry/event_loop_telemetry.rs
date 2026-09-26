use std::collections::BTreeMap;

use super::HostTelemetryProjection;

pub const EVENT_LOOP_RESOLUTION_MS: u64 = 20;
pub const WINDOW_MS: u64 = 60_000;
pub const PRESSURE_P95_MS: f64 = 50.0;
pub const HEARTBEAT_EVERY_N_WINDOWS: u64 = 5;
pub const EVENT_LOOP_EVENT: &str = "sand.host.event_loop";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowEmitArgs {
    pub window_index: u64,
    pub p95_ms: f64,
    pub pressure_p95_ms: Option<f64>,
    pub heartbeat_every_n_windows: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventLoopTrigger {
    Pressure,
    Heartbeat,
}

impl EventLoopTrigger {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pressure => "pressure",
            Self::Heartbeat => "heartbeat",
        }
    }
}

pub fn resolve_window_emit(args: WindowEmitArgs) -> Option<EventLoopTrigger> {
    let threshold = args.pressure_p95_ms.unwrap_or(PRESSURE_P95_MS);
    let heartbeat = args
        .heartbeat_every_n_windows
        .unwrap_or(HEARTBEAT_EVERY_N_WINDOWS);

    if args.p95_ms >= threshold {
        return Some(EventLoopTrigger::Pressure);
    }
    if heartbeat != 0 && args.window_index % heartbeat == 0 {
        return Some(EventLoopTrigger::Heartbeat);
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EventLoopWindowReport {
    pub trigger: EventLoopTrigger,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub max_ms: f64,
    pub utilization: f64,
    pub window_ms: u64,
}

fn rounded(value: f64) -> String {
    format!("{:.0}", value.round())
}

pub fn event_loop_window_telemetry(report: EventLoopWindowReport) -> HostTelemetryProjection {
    let mut metadata = BTreeMap::new();
    metadata.insert("trigger".into(), report.trigger.as_str().into());
    metadata.insert("p50_ms".into(), rounded(report.p50_ms));
    metadata.insert("p95_ms".into(), rounded(report.p95_ms));
    metadata.insert("max_ms".into(), rounded(report.max_ms));
    metadata.insert("utilization".into(), format!("{:.3}", report.utilization));
    metadata.insert("window_ms".into(), report.window_ms.to_string());

    HostTelemetryProjection {
        level: Some(match report.trigger {
            EventLoopTrigger::Pressure => "warn",
            EventLoopTrigger::Heartbeat => "info",
        }),
        event: Some(EVENT_LOOP_EVENT),
        metadata,
    }
}
