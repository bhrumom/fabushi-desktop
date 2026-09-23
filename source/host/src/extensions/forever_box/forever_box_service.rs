use std::fmt;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};

use crate::extensions::box_lifecycle::{
    BoxLifecycleClient, BoxLifecycleService, RecreateSandBoxResponse,
};
use crate::r#box::box_env::BoxEnvironmentUpdate;
use crate::r#box::generated_production::ProductionBoxResourceAccessor;
use crate::r#box::loopback_sand_box::LoopbackSandBoxError;

use super::disk_pressure::{DiskPressureWatch};
use super::disk_pressure_guard::DiskPressureLevel;
use super::host_box::{BoxStatus, HostBox};

pub trait ForeverBoxLifecycle: Send + Sync {
    fn fetch_image_update_available(&self) -> Result<bool, String>;
    fn recreate_in_box(
        &self,
        preserve_data: bool,
        force: Option<bool>,
    ) -> Result<RecreateSandBoxResponse, String>;
}

impl<Client> ForeverBoxLifecycle for BoxLifecycleService<Client>
where
    Client: BoxLifecycleClient<()> + Send + Sync,
    Client::Error: fmt::Display,
{
    fn fetch_image_update_available(&self) -> Result<bool, String> {
        futures::executor::block_on(
            BoxLifecycleService::fetch_image_update_available(self, &()),
        )
        .map_err(|error| error.to_string())
    }

    fn recreate_in_box(
        &self,
        preserve_data: bool,
        force: Option<bool>,
    ) -> Result<RecreateSandBoxResponse, String> {
        futures::executor::block_on(
            BoxLifecycleService::recreate_in_box::<()>(self, preserve_data, force),
        )
        .map_err(|error| error.to_string())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ForeverBoxServiceError {
    #[error("box lifecycle failed: {0}")]
    Lifecycle(String),
    #[error("box operation failed: {0}")]
    Box(String),
    #[error("box recreate was declined: {0}")]
    RecreateDeclined(String),
}

pub struct ForeverBoxService {
    box_: HostBox,
    lifecycle: Arc<dyn ForeverBoxLifecycle>,
    auto_update_enabled: bool,
    host_bundle_auto_update_enabled: bool,
    is_in_box: bool,
    busy: AtomicBool,
    update_in_flight: AtomicBool,
    stopped: AtomicBool,
    disk_pressure_watch: Mutex<Option<DiskPressureWatch>>,
}

impl ForeverBoxService {
    pub fn new(
        box_: HostBox,
        lifecycle: Arc<dyn ForeverBoxLifecycle>,
        auto_update_enabled: bool,
        host_bundle_auto_update_enabled: bool,
        is_in_box: bool,
    ) -> Self {
        Self {
            box_,
            lifecycle,
            auto_update_enabled,
            host_bundle_auto_update_enabled,
            is_in_box,
            busy: AtomicBool::new(false),
            update_in_flight: AtomicBool::new(false),
            stopped: AtomicBool::new(false),
            disk_pressure_watch: Mutex::new(None),
        }
    }

    pub fn start(self: &Arc<Self>) {
        if !self.is_in_box || self.stopped.load(Ordering::Acquire) {
            return;
        }
        let service = Arc::clone(self);
        let _ = thread::Builder::new()
            .name("mahayana-forever-box-image-seed".into())
            .spawn(move || {
                let _ = service.refresh_image_update_available();
            });
    }

    pub fn box_(&self) -> &HostBox {
        &self.box_
    }

    pub fn install_disk_pressure_watch(&self, watch: DiskPressureWatch) {
        if let Ok(mut slot) = self.disk_pressure_watch.lock() {
            if let Some(previous) = slot.replace(watch) {
                previous.dispose();
            }
        }
    }

    pub fn disk_pressure_level(&self) -> Option<DiskPressureLevel> {
        self.disk_pressure_watch
            .lock()
            .ok()
            .and_then(|watch| watch.as_ref().and_then(DiskPressureWatch::level))
    }

    pub fn is_auto_update_enabled(&self) -> bool {
        self.auto_update_enabled
    }

    pub fn set_busy(&self, busy: bool) {
        self.busy.store(busy, Ordering::Release);
    }

    pub fn refresh_image_update_available(&self) -> Result<bool, ForeverBoxServiceError> {
        let available = self
            .lifecycle
            .fetch_image_update_available()
            .map_err(ForeverBoxServiceError::Lifecycle)?;
        self.box_.record_image_update_available(Some(available));
        Ok(available)
    }

    pub fn get_status(&self, agent_id: &str) -> BoxStatus {
        self.box_.get_status(agent_id)
    }

    pub fn ensure(&self, agent_id: &str) -> Result<BoxStatus, ForeverBoxServiceError> {
        self.box_
            .ensure(agent_id)
            .map_err(|error| ForeverBoxServiceError::Box(error.to_string()))
    }

    pub fn capture_screenshot(
        &self,
        agent_id: &str,
    ) -> Result<Option<Vec<u8>>, ForeverBoxServiceError> {
        let mut ready = self
            .box_
            .ensure_ready(agent_id)
            .map_err(|error| ForeverBoxServiceError::Box(error.to_string()))?;
        let request = encode_computer_use_screenshot_request(&format!(
            "sand-box-handoff-screenshot-{agent_id}"
        ));
        let response = ready
            .remote_accessor
            .execute_computer_use_protobuf(&(), request)
            .map_err(|error| ForeverBoxServiceError::Box(error.to_string()))?;
        let Some(encoded) = decode_computer_use_screenshot_base64(&response) else {
            return Ok(None);
        };
        let payload = encoded
            .strip_prefix("data:")
            .and_then(|value| value.split_once(',').map(|(_, body)| body))
            .unwrap_or(encoded.as_str());
        let bytes = BASE64_STANDARD
            .decode(payload)
            .map_err(|error| ForeverBoxServiceError::Box(format!(
                "computer screenshot base64 decode failed: {error}"
            )))?;
        Ok((!bytes.is_empty()).then_some(bytes))
    }

    pub fn reset(&self, agent_id: &str) -> Result<BoxStatus, ForeverBoxServiceError> {
        self.recreate(agent_id, false, None)
    }

    pub fn update(
        &self,
        agent_id: &str,
        force: Option<bool>,
    ) -> Result<BoxStatus, ForeverBoxServiceError> {
        self.recreate(agent_id, true, force)
    }

    pub fn auto_update_now(&self) -> Result<RecreateSandBoxResponse, ForeverBoxServiceError> {
        if !self.is_in_box {
            return Ok(RecreateSandBoxResponse {
                started: false,
                reason: Some("not-in-box".into()),
            });
        }
        if !self.auto_update_enabled {
            return Ok(RecreateSandBoxResponse {
                started: false,
                reason: Some("auto-update-disabled".into()),
            });
        }
        if self.busy.load(Ordering::Acquire) {
            return Ok(RecreateSandBoxResponse {
                started: false,
                reason: Some("busy".into()),
            });
        }
        if self.update_in_flight.swap(true, Ordering::AcqRel) {
            return Ok(RecreateSandBoxResponse {
                started: false,
                reason: Some("update-in-flight".into()),
            });
        }
        let result = (|| {
            let available = self.refresh_image_update_available()?;
            if !available {
                return Ok(RecreateSandBoxResponse {
                    started: false,
                    reason: Some("no-update-required".into()),
                });
            }
            self.lifecycle
                .recreate_in_box(true, None)
                .map_err(ForeverBoxServiceError::Lifecycle)
        })();
        self.update_in_flight.store(false, Ordering::Release);
        result
    }

    fn recreate(
        &self,
        agent_id: &str,
        preserve_data: bool,
        force: Option<bool>,
    ) -> Result<BoxStatus, ForeverBoxServiceError> {
        let result = self
            .lifecycle
            .recreate_in_box(preserve_data, force)
            .map_err(ForeverBoxServiceError::Lifecycle)?;
        if !result.started {
            return Err(ForeverBoxServiceError::RecreateDeclined(
                result.reason.unwrap_or_else(|| "service declined the recreate".into()),
            ));
        }
        Ok(BoxStatus {
            agent_id: agent_id.to_string(),
            state: "running".into(),
            vnc_url: None,
            windows: None,
            image_update_available: self.box_.image_update_available(),
            pull_percent: Some(0),
        })
    }

    pub fn release_agent(&self, agent_id: &str) {
        self.box_.release_window(agent_id);
    }

    pub fn apply_environment(
        &self,
        update: &BoxEnvironmentUpdate,
    ) -> Result<(), LoopbackSandBoxError> {
        self.box_.apply_environment(update)
    }

    pub fn load_mcp_servers(
        &self,
        config_json: &str,
    ) -> Result<Vec<String>, LoopbackSandBoxError> {
        self.box_.load_mcp_servers(config_json)
    }

    pub fn mcp_resource_accessor(
        &self,
    ) -> Result<ProductionBoxResourceAccessor, LoopbackSandBoxError> {
        self.box_.mcp_resource_accessor()
    }

    pub fn dispose(&self) {
        self.stopped.store(true, Ordering::Release);
        if let Some(watch) = self
            .disk_pressure_watch
            .lock()
            .ok()
            .and_then(|mut slot| slot.take())
        {
            watch.dispose();
        }
    }

    pub fn host_bundle_auto_update_enabled(&self) -> bool {
        self.host_bundle_auto_update_enabled
    }
}

pub fn encode_computer_use_screenshot_request(tool_call_id: &str) -> Vec<u8> {
    let mut action = Vec::new();
    push_length_delimited_field(10, &[], &mut action);
    let mut request = Vec::new();
    push_length_delimited_field(1, tool_call_id.as_bytes(), &mut request);
    push_length_delimited_field(2, &action, &mut request);
    request
}

pub fn decode_computer_use_screenshot_base64(payload: &[u8]) -> Option<String> {
    let mut outer = 0usize;
    while outer < payload.len() {
        let tag = read_proto_varint(payload, &mut outer)?;
        let field = tag >> 3;
        let wire = (tag & 0x07) as u8;
        if field == 1 && wire == 2 {
            let success = read_proto_bytes(payload, &mut outer)?;
            let mut inner = 0usize;
            while inner < success.len() {
                let tag = read_proto_varint(success, &mut inner)?;
                let field = tag >> 3;
                let wire = (tag & 0x07) as u8;
                if field == 3 && wire == 2 {
                    return String::from_utf8(read_proto_bytes(success, &mut inner)?.to_vec())
                        .ok()
                        .filter(|value| !value.is_empty());
                }
                skip_proto_field(success, &mut inner, wire)?;
            }
            return None;
        }
        skip_proto_field(payload, &mut outer, wire)?;
    }
    None
}

fn push_proto_varint(mut value: u64, output: &mut Vec<u8>) {
    while value >= 0x80 {
        output.push((value as u8) | 0x80);
        value >>= 7;
    }
    output.push(value as u8);
}

fn push_length_delimited_field(field: u64, value: &[u8], output: &mut Vec<u8>) {
    push_proto_varint((field << 3) | 2, output);
    push_proto_varint(value.len() as u64, output);
    output.extend_from_slice(value);
}

fn read_proto_varint(data: &[u8], position: &mut usize) -> Option<u64> {
    let mut value = 0u64;
    for shift in (0..70).step_by(7) {
        let byte = *data.get(*position)?;
        *position += 1;
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Some(value);
        }
    }
    None
}

fn read_proto_bytes<'a>(data: &'a [u8], position: &mut usize) -> Option<&'a [u8]> {
    let length: usize = read_proto_varint(data, position)?.try_into().ok()?;
    let end = position.checked_add(length)?;
    let value = data.get(*position..end)?;
    *position = end;
    Some(value)
}

fn skip_proto_field(data: &[u8], position: &mut usize, wire: u8) -> Option<()> {
    match wire {
        0 => {
            let _ = read_proto_varint(data, position)?;
        }
        1 => {
            *position = position.checked_add(8)?;
            if *position > data.len() {
                return None;
            }
        }
        2 => {
            let _ = read_proto_bytes(data, position)?;
        }
        5 => {
            *position = position.checked_add(4)?;
            if *position > data.len() {
                return None;
            }
        }
        _ => return None,
    }
    Some(())
}
