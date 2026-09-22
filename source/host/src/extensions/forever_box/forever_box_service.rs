use std::fmt;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;

use crate::extensions::box_lifecycle::{
    BoxLifecycleClient, BoxLifecycleService, RecreateSandBoxResponse,
};
use crate::r#box::box_env::BoxEnvironmentUpdate;
use crate::r#box::generated_production::ProductionBoxResourceAccessor;
use crate::r#box::loopback_sand_box::LoopbackSandBoxError;

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
    }

    pub fn host_bundle_auto_update_enabled(&self) -> bool {
        self.host_bundle_auto_update_enabled
    }
}
