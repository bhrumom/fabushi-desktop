use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginStatus {
    Active,
    Disabled,
    Error(String),
    Uninstalled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUninstallReceipt {
    pub plugin_id: String,
    pub uninstalled_at_ms: u64,
    pub files_removed: usize,
    pub retained_storage_bytes: u64,
}

#[derive(Clone, Default)]
pub struct PluginRuntimeManager {
    statuses: Arc<Mutex<HashMap<String, PluginStatus>>>,
    uninstall_receipts: Arc<Mutex<HashMap<String, PluginUninstallReceipt>>>,
}

impl PluginRuntimeManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_status(&self, plugin_id: &str, status: PluginStatus) {
        let mut statuses = self.statuses.lock().unwrap();
        statuses.insert(plugin_id.to_string(), status);
    }

    pub fn get_status(&self, plugin_id: &str) -> PluginStatus {
        let statuses = self.statuses.lock().unwrap();
        statuses
            .get(plugin_id)
            .cloned()
            .unwrap_or(PluginStatus::Disabled)
    }

    pub fn uninstall_plugin(&self, plugin_id: &str, files_removed: usize) -> PluginUninstallReceipt {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let receipt = PluginUninstallReceipt {
            plugin_id: plugin_id.to_string(),
            uninstalled_at_ms: now_ms,
            files_removed,
            retained_storage_bytes: 0,
        };

        let mut statuses = self.statuses.lock().unwrap();
        statuses.insert(plugin_id.to_string(), PluginStatus::Uninstalled);

        let mut receipts = self.uninstall_receipts.lock().unwrap();
        receipts.insert(plugin_id.to_string(), receipt.clone());

        receipt
    }

    pub fn get_uninstall_receipt(&self, plugin_id: &str) -> Option<PluginUninstallReceipt> {
        let receipts = self.uninstall_receipts.lock().unwrap();
        receipts.get(plugin_id).cloned()
    }
}
