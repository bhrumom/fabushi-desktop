use std::sync::Arc;
use super::settings_service::SettingsService;
pub const SETTINGS_EXTENSION_ID:&str="settings";
pub const SETTINGS_EXTENSION_DEPENDENCIES:&[&str]=&[];
pub fn start_settings_extension()->Arc<SettingsService>{Arc::new(SettingsService::production())}
