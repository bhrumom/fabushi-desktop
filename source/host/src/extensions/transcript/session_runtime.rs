use std::sync::{Arc, Mutex};

use serde_json::Value;

use crate::extensions::session::agent_session::SandAgentSessionStore;
use crate::extensions::session::production::ProductionSessionWorkers;

use super::transcript_hub::TranscriptEntry;
use super::transcript_store::TranscriptStore;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct WindowFocusState {
    is_focused: bool,
    focused_at_ms: Option<f64>,
}

/// Shipping Rust owner for the active Session/Transcript window state that
/// frozen Grok keeps in transcript/session-runtime.ts.
///
/// This slice deliberately owns only focus + active-agent switching. Windowed
/// deferred activation/catch-up and the broader live-session cache stay
/// non-final until their exact contracts are ported.
#[derive(Default)]
pub struct SessionRuntime {
    focus: Mutex<WindowFocusState>,
    transcript: TranscriptStore,
    in_memory_transcript_agent_id: Mutex<Option<String>>,
}

impl SessionRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_entries(&self) -> Vec<TranscriptEntry> {
        self.transcript.get_transcript()
    }

    pub fn set_active_transcript(
        &self,
        agent_id: &str,
        entries: &[TranscriptEntry],
    ) {
        self.transcript.set_transcript(entries);
        if let Ok(mut owner) = self.in_memory_transcript_agent_id.lock() {
            *owner = Some(agent_id.to_string());
        }
    }

    pub fn clear_active_transcript(&self, agent_id: Option<&str>) {
        self.transcript.clear_transcript();
        if let Ok(mut owner) = self.in_memory_transcript_agent_id.lock() {
            *owner = agent_id.map(ToOwned::to_owned);
        }
    }

    pub fn append_entry(&self, entry: TranscriptEntry) {
        self.transcript.append_entry(entry);
    }

    pub fn update_entry<F>(
        &self,
        id: &str,
        update: F,
    ) -> Option<TranscriptEntry>
    where
        F: FnOnce(&TranscriptEntry) -> TranscriptEntry,
    {
        self.transcript.update_entry(id, update)
    }

    pub fn remove_entry(&self, id: &str) -> bool {
        self.transcript.remove_entry(id)
    }

    fn has_loaded_agent(&self, agent_id: &str) -> bool {
        self.in_memory_transcript_agent_id
            .lock()
            .ok()
            .and_then(|owner| owner.clone())
            .as_deref()
            == Some(agent_id)
    }

    pub fn is_window_focused(&self) -> bool {
        self.focus
            .lock()
            .map(|state| state.is_focused)
            .unwrap_or(false)
    }

    pub fn window_focused_at_ms(&self) -> Option<f64> {
        self.focus
            .lock()
            .ok()
            .and_then(|state| state.focused_at_ms)
    }

    pub fn set_window_focused(
        &self,
        sessions: &Arc<ProductionSessionWorkers>,
        is_focused: bool,
        now_ms: f64,
    ) -> Result<bool, String> {
        let became_focused = {
            let mut state = self
                .focus
                .lock()
                .map_err(|_| "transcript session focus state poisoned".to_string())?;
            let was_focused = state.is_focused;
            state.is_focused = is_focused;
            state.focused_at_ms = is_focused.then_some(now_ms);
            is_focused && !was_focused
        };
        if !became_focused {
            return Ok(false);
        }

        let store = SandAgentSessionStore::new(Arc::clone(sessions));
        let Some(active_agent_id) = store.read_active_agent_id() else {
            return Ok(false);
        };
        let owner = sessions.open_agent_db_owner(&active_agent_id)?;
        let unread = owner
            .get_unread_state()
            .map_err(|error| error.to_string())?;
        if unread.is_manually_unread || unread.last_activity_at <= unread.last_viewed_at {
            return Ok(false);
        }
        sessions.mark_agent_viewed(&active_agent_id, now_ms, true)
    }

    pub fn switch_agent(
        &self,
        sessions: &Arc<ProductionSessionWorkers>,
        agent_id: &str,
        now_ms: f64,
    ) -> Result<Vec<Value>, String> {
        let store = SandAgentSessionStore::new(Arc::clone(sessions));
        let current = store.read_active_agent_id();
        if current.as_deref() == Some(agent_id) {
            if self.has_loaded_agent(agent_id) {
                return Ok(self.get_entries());
            }
            let entries = sessions.read_agent_transcript_entries(agent_id)?;
            self.set_active_transcript(agent_id, &entries);
            return Ok(entries);
        }

        if self.is_window_focused() {
            if let Some(current_agent_id) = current.as_deref() {
                sessions.mark_agent_viewed(current_agent_id, now_ms, true)?;
            }
        }

        sessions.mark_agent_viewed(agent_id, now_ms, false)?;
        let entries = sessions.read_agent_transcript_entries(agent_id)?;
        self.set_active_transcript(agent_id, &entries);
        store
            .write_active_agent_id(agent_id)
            .map_err(|error| error.to_string())?;
        Ok(entries)
    }
}
