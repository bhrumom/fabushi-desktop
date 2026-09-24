use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::analytics_service::product_analytics_event;
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
}

impl HostStructuredLogTelemetry {
    pub fn report_box_help(&self, report: &Value) -> io::Result<()> {
        let projection = box_help_telemetry(report);
        self.sink.emit(&PersistedHostTelemetryRecord {
            channel: "structured_log".into(),
            event: projection.event.unwrap_or(BOX_HELP_EVENT).to_string(),
            payload: json!({
                "level": projection.level.unwrap_or("info"),
                "metadata": projection.metadata,
            }),
        })
    }
}

#[derive(Clone)]
pub struct HostProductAnalytics {
    sink: Arc<JsonlHostTelemetrySink>,
}

impl HostProductAnalytics {
    pub fn track_event(&self, name: &str, properties: &Value) -> io::Result<()> {
        let event = product_analytics_event(name, properties);
        self.sink.emit(&PersistedHostTelemetryRecord {
            channel: "product_analytics".into(),
            event: event.name,
            payload: serde_json::to_value(event.properties).map_err(io::Error::other)?,
        })
    }
}

pub struct HostTelemetryService {
    pub logs: HostStructuredLogTelemetry,
    pub analytics: HostProductAnalytics,
    sink: Arc<JsonlHostTelemetrySink>,
}

impl HostTelemetryService {
    pub fn open(records_path: impl Into<PathBuf>) -> io::Result<Self> {
        let sink = Arc::new(JsonlHostTelemetrySink::open(records_path)?);
        Ok(Self {
            logs: HostStructuredLogTelemetry {
                sink: Arc::clone(&sink),
            },
            analytics: HostProductAnalytics {
                sink: Arc::clone(&sink),
            },
            sink,
        })
    }

    pub fn records_path(&self) -> &Path {
        self.sink.path()
    }
}
