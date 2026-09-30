use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::ports::telemetry::resolve_sand_box_identity_tags;

use super::HostTelemetryProjection;
use super::analytics_service::{
    AnalyticsClient, ProductionAnalyticsRuntime, TelemetryService, product_analytics_event,
};
use super::box_log_ship_telemetry::{BoxLogShipReport, box_log_ship_telemetry};
use super::box_log_shipper::BoxTelemetryRecord;
use super::lifecycle_telemetry::box_infrastructure_telemetry;
use super::structured_log_telemetry::{BOX_HELP_EVENT, box_help_telemetry};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersistedHostTelemetryRecord {
    pub channel: String,
    pub event: String,
    pub payload: Value,
}

struct JsonlHostTelemetrySink {
    path: PathBuf,
    file: Mutex<File>,
}

impl JsonlHostTelemetrySink {
    fn open(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path = path.into();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        Ok(Self {
            path,
            file: Mutex::new(file),
        })
    }

    fn emit(&self, record: &PersistedHostTelemetryRecord) -> io::Result<()> {
        let mut file = self
            .file
            .lock()
            .map_err(|_| io::Error::other("Host telemetry sink mutex poisoned"))?;
        serde_json::to_writer(&mut *file, record).map_err(io::Error::other)?;
        file.write_all(b"\n")?;
        file.flush()
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

#[derive(Clone)]
pub struct HostStructuredLogTelemetry {
    sink: Arc<JsonlHostTelemetrySink>,
    identity_tags: Arc<BTreeMap<String, String>>,
}

impl HostStructuredLogTelemetry {
    pub fn report_projection(&self, projection: &HostTelemetryProjection) -> io::Result<()> {
        let mut metadata = self.identity_tags.as_ref().clone();
        metadata.extend(projection.metadata.clone());
        self.sink.emit(&PersistedHostTelemetryRecord {
            channel: "structured_log".into(),
            event: projection.event.unwrap_or("sand.host.telemetry").to_string(),
            payload: json!({
                "level": projection.level.unwrap_or("info"),
                "metadata": metadata,
            }),
        })
    }

    pub fn report_box_log_record(&self, record: &BoxTelemetryRecord) -> io::Result<()> {
        match record {
            BoxTelemetryRecord::Log { source, line } => {
                let text = line.chars().take(2_048).collect::<String>();
                self.report_projection(&HostTelemetryProjection {
                    level: Some("info"),
                    event: Some("sand.box.log"),
                    metadata: BTreeMap::from([
                        ("source".into(), source.clone()),
                        ("text".into(), text),
                    ]),
                })
            }
            BoxTelemetryRecord::Infrastructure { event } => {
                self.report_projection(&box_infrastructure_telemetry(event))
            }
        }
    }

    pub fn report_box_log_ship(&self, report: &BoxLogShipReport) -> io::Result<()> {
        self.report_projection(&box_log_ship_telemetry(report))
    }

    pub fn report_box_help(&self, report: &Value) -> io::Result<()> {
        let projection = box_help_telemetry(report);
        debug_assert_eq!(projection.event, Some(BOX_HELP_EVENT));
        self.report_projection(&projection)
    }

    pub fn report_teach_recording_cap_stop_failed(&self, report: &Value) -> io::Result<()> {
        let mut metadata = BTreeMap::new();
        if let Some(error_class) = report.get("errorClass").and_then(Value::as_str) {
            metadata.insert("error_class".into(), error_class.to_string());
        }
        self.report_projection(&HostTelemetryProjection {
            level: Some("warn"),
            event: Some("sand.teach.cap_stop_failed"),
            metadata,
        })
    }

    pub fn report_teach_recording_start_failed(&self, report: &Value) -> io::Result<()> {
        let mut metadata = BTreeMap::new();
        if let Some(kind) = report.get("kind").and_then(Value::as_str) {
            metadata.insert("kind".into(), kind.to_string());
        }
        if let Some(error_class) = report.get("errorClass").and_then(Value::as_str) {
            metadata.insert("error_class".into(), error_class.to_string());
        }
        if let Some(window_index) = report.get("windowIndex").and_then(Value::as_u64) {
            metadata.insert("window_index".into(), window_index.to_string());
        }
        if let Some(entry_point) = report.get("entryPoint").and_then(Value::as_str) {
            metadata.insert("entry_point".into(), entry_point.to_string());
        }
        self.report_projection(&HostTelemetryProjection {
            level: Some("warn"),
            event: Some("sand.teach.recording_start_failed"),
            metadata,
        })
    }

    pub fn report_search_index_health(&self, report: &Value) -> io::Result<()> {
        self.sink.emit(&PersistedHostTelemetryRecord {
            channel: "structured_log".into(),
            event: "sand.search_index_health".into(),
            payload: report.clone(),
        })
    }

    pub fn report_host_extension_diagnostic(&self, diagnostic: &Value) -> io::Result<()> {
        self.sink.emit(&PersistedHostTelemetryRecord {
            channel: "structured_log".into(),
            event: "sand.host_extension_diagnostic".into(),
            payload: diagnostic.clone(),
        })
    }

    pub fn report_automation_shadow_prune(&self, report: &Value) -> io::Result<()> {
        self.sink.emit(&PersistedHostTelemetryRecord {
            channel: "structured_log".into(),
            event: "sand.automation_shadow_prune".into(),
            payload: report.clone(),
        })
    }

    pub fn report_box_boot_stage_confirmed(&self, stage: &str, duration_ms: u64) -> bool {
        let mut metadata = BTreeMap::new();
        metadata.insert("stage".into(), stage.to_string());
        metadata.insert("duration_ms".into(), duration_ms.to_string());
        self.report_projection(&HostTelemetryProjection {
            level: Some("info"),
            event: Some("sand.box.boot_stage_confirmed"),
            metadata,
        })
        .is_ok()
    }
}

#[derive(Clone)]
pub struct HostProductAnalytics {
    sink: Arc<JsonlHostTelemetrySink>,
    runtime: Arc<Mutex<Option<Arc<ProductionAnalyticsRuntime>>>>,
}

impl HostProductAnalytics {
    pub fn attach_runtime(&self, runtime: Arc<ProductionAnalyticsRuntime>) {
        *self.runtime.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(runtime);
    }

    pub fn mark_active(&self, reason: &str) {
        if let Some(runtime) = self
            .runtime
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .cloned()
        {
            runtime.mark_active(reason);
        }
    }

    pub fn can_record_events(&self) -> bool {
        self.runtime
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .is_some_and(|runtime| runtime.can_record_events())
    }

    pub fn track_event(&self, name: &str, properties: &Value) -> io::Result<()> {
        let event = product_analytics_event(name, properties);
        self.sink.emit(&PersistedHostTelemetryRecord {
            channel: "product_analytics".into(),
            event: event.name.clone(),
            payload: serde_json::to_value(&event.properties).map_err(io::Error::other)?,
        })?;
        if let Some(runtime) = self
            .runtime
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .cloned()
        {
            runtime.track(event);
        }
        Ok(())
    }
}

impl AnalyticsClient for HostProductAnalytics {
    fn track_event(&self, name: &str, properties: &Value) {
        let _ = HostProductAnalytics::track_event(self, name, properties);
    }
}

impl TelemetryService for HostStructuredLogTelemetry {
    fn report(&self, method: &'static str, report: &Value) {
        let event = match method {
            "startTurn" => "sand.turn.start",
            "reportToolCallError" => "sand.tool_call.error",
            "reportToolCallStalled" => "sand.tool_call.stalled",
            "reportToolCallStarted" => "sand.tool_call.started",
            "reportAgentError" => "sand.agent.error",
            "reportBotBlock" => "sand.bot.block",
            "reportDaemonPing" => "sand.daemon.ping",
            "reportBoxBootStage" => "sand.box.boot_stage",
            "reportExecDaemonRestart" => "sand.exec_daemon.restart",
            "reportSupervisorRestart" => "sand.supervisor.restart",
            "reportTurnInterrupt" => "sand.turn.interrupt",
            "reportTurnAwait" => "sand.turn.await",
            "reportTurnRetry" => "sand.turn.retry",
            "reportUserMessageReceived" => "sand.user_message.received",
            "reportClosingSendNudge" => "sand.closing_send_nudge",
            "reportSubagentRevival" => "sand.subagent.revival",
            "reportShellRevival" => "sand.shell.revival",
            "reportComputerUseUsage" => "sand.computer_use.usage",
            "reportTtft" => "sand.turn.ttft",
            "reportSendDispatch" => "sand.send.dispatch",
            "reportQueueAccepted" => "sand.queue.accepted",
            "reportQueueDequeued" => "sand.queue.dequeued",
            "reportQueueWatchdog" => "sand.queue.watchdog",
            "reportAckObligation" => "sand.ack_obligation",
            "reportPendingWake" => "sand.pending_wake",
            "reportTurnUsage" => "sand.turn.usage",
            "reportTurnEmptyDelivery" => "sand.turn.empty_delivery",
            "reportJournalOutcome" => "sand.journal.outcome",
            "reportAutoReviewExpireSweepFailed" => "sand.auto_review.expire_sweep_failed",
            "reportAutomationLifecycle" => "sand.automation.lifecycle",
            "reportAutomationFireDropped" => "sand.automation.fire_dropped",
            "reportAutomationRun" => "sand.automation.run",
            _ => "sand.host.telemetry",
        };
        let _ = self.sink.emit(&PersistedHostTelemetryRecord {
            channel: "structured_log".into(),
            event: event.into(),
            payload: report.clone(),
        });
    }
}

pub struct HostTelemetryService {
    pub logs: HostStructuredLogTelemetry,
    pub analytics: HostProductAnalytics,
    sink: Arc<JsonlHostTelemetrySink>,
}

impl HostTelemetryService {
    pub fn open(records_path: impl Into<PathBuf>) -> io::Result<Self> {
        Self::open_with_identity_tags(
            records_path,
            resolve_sand_box_identity_tags(),
        )
    }

    pub fn open_with_identity_tags(
        records_path: impl Into<PathBuf>,
        mut identity_tags: BTreeMap<String, String>,
    ) -> io::Result<Self> {
        identity_tags.retain(|_, value| !value.is_empty());
        let sink = Arc::new(JsonlHostTelemetrySink::open(records_path)?);
        let identity_tags = Arc::new(identity_tags);
        Ok(Self {
            logs: HostStructuredLogTelemetry {
                sink: Arc::clone(&sink),
                identity_tags,
            },
            analytics: HostProductAnalytics {
                sink: Arc::clone(&sink),
                runtime: Arc::new(Mutex::new(None)),
            },
            sink,
        })
    }

    pub fn records_path(&self) -> &Path {
        self.sink.path()
    }
}
