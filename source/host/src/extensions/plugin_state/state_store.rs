//! OpenClaw Plugin State Store, Journaling, and Key-Value Operations
//!
//! Mirrors:
//! - `src/plugin-state/plugin-state-store.ts`
//! - `src/plugin-state/plugin-state-store.sqlite.ts`
//! - `src/plugin-state/plugin-state-store.kernel.ts`
//! - `src/plugin-state/plugin-state-store.journal.ts`
//! - `src/plugin-state/plugin-state-store.validation.ts`
//! - `src/plugin-state/plugin-store-validation.ts`

use std::collections::HashMap;
use serde_json::Value;

pub const MAX_PLUGIN_STATE_VALUE_BYTES: usize = 1_048_576; // 1 MiB
pub const MAX_PLUGIN_STATE_BULK_DELETE_ENTRIES: usize = 512;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateStoreError {
    ValueTooLarge { size: usize, limit: usize },
    BulkDeleteLimitExceeded { count: usize, limit: usize },
    InvalidKey,
    NotFound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginStateJournalEntry {
    pub sequence_id: u64,
    pub plugin_id: String,
    pub namespace: String,
    pub key: String,
    pub timestamp_ms: u64,
}

#[derive(Debug, Default)]
pub struct PluginStateKeyedStore {
    // plugin_id -> (namespace:key) -> Value
    stores: HashMap<String, HashMap<String, Value>>,
    journal: Vec<PluginStateJournalEntry>,
    sequence_counter: u64,
}

impl PluginStateKeyedStore {
    pub fn new() -> Self {
        Self {
            stores: HashMap::new(),
            journal: Vec::new(),
            sequence_counter: 0,
        }
    }

    fn composite_key(namespace: &str, key: &str) -> String {
        format!("{}:{}", namespace.trim(), key.trim())
    }

    pub fn set_state(
        &mut self,
        plugin_id: &str,
        namespace: &str,
        key: &str,
        value: Value,
        now_ms: u64,
    ) -> Result<u64, StateStoreError> {
        if key.trim().is_empty() || namespace.trim().is_empty() {
            return Err(StateStoreError::InvalidKey);
        }

        let serialized = value.to_string();
        if serialized.len() > MAX_PLUGIN_STATE_VALUE_BYTES {
            return Err(StateStoreError::ValueTooLarge {
                size: serialized.len(),
                limit: MAX_PLUGIN_STATE_VALUE_BYTES,
            });
        }

        let plugin_store = self.stores.entry(plugin_id.to_string()).or_default();
        let comp_key = Self::composite_key(namespace, key);
        plugin_store.insert(comp_key, value);

        self.sequence_counter += 1;
        self.journal.push(PluginStateJournalEntry {
            sequence_id: self.sequence_counter,
            plugin_id: plugin_id.to_string(),
            namespace: namespace.to_string(),
            key: key.to_string(),
            timestamp_ms: now_ms,
        });

        Ok(self.sequence_counter)
    }

    pub fn get_state(
        &self,
        plugin_id: &str,
        namespace: &str,
        key: &str,
    ) -> Result<&Value, StateStoreError> {
        let plugin_store = self.stores.get(plugin_id).ok_or(StateStoreError::NotFound)?;
        let comp_key = Self::composite_key(namespace, key);
        plugin_store.get(&comp_key).ok_or(StateStoreError::NotFound)
    }

    pub fn bulk_delete(
        &mut self,
        plugin_id: &str,
        entries: &[(String, String)], // [(namespace, key)]
    ) -> Result<usize, StateStoreError> {
        if entries.len() > MAX_PLUGIN_STATE_BULK_DELETE_ENTRIES {
            return Err(StateStoreError::BulkDeleteLimitExceeded {
                count: entries.len(),
                limit: MAX_PLUGIN_STATE_BULK_DELETE_ENTRIES,
            });
        }

        let mut deleted = 0;
        if let Some(plugin_store) = self.stores.get_mut(plugin_id) {
            for (ns, key) in entries {
                let comp_key = Self::composite_key(ns, key);
                if plugin_store.remove(&comp_key).is_some() {
                    deleted += 1;
                }
            }
        }

        Ok(deleted)
    }
}
