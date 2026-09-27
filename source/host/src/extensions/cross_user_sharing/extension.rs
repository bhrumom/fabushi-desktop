use super::xuser_sharing_environment::resolve_xuser_sharing_environment;
use super::xuser_sharing_service::{SandSharingState, SandXuserSharingService};
use std::collections::BTreeMap;
use std::sync::Arc;

pub const SHARING_DISABLED_MESSAGE: &str = "Sharing isn't enabled for your account.";

pub struct CrossUserSharingExtension {
    service: Arc<SandXuserSharingService>,
    allowed: bool,
    reason: Option<String>,
}
impl CrossUserSharingExtension {
    pub fn new(
        service: Arc<SandXuserSharingService>,
        backend_url: &str,
        env: &BTreeMap<String, String>,
    ) -> Self {
        let gate = resolve_xuser_sharing_environment(backend_url, env);
        if !gate.is_allowed {
            service.set_enabled(false)
        }
        Self {
            service,
            allowed: gate.is_allowed,
            reason: gate.reason,
        }
    }
    pub fn set_feature_gate(&self, on: bool) {
        self.service.set_enabled(on && self.allowed)
    }
    pub fn is_enabled(&self) -> bool {
        self.service.get_state().is_enabled
    }
    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }
    pub fn get_sharing_state(&self) -> SandSharingState {
        self.service.get_state()
    }
    pub fn prepare_for_upgrade(&self) {
        self.service.set_enabled(false)
    }
}
