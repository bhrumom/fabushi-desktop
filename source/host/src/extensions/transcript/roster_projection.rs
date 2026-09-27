use std::collections::HashMap;

pub const OUTLINE_STREAM_COALESCE_MS: u64 = 250;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlineUpdate<T> {
    pub agent_id: String,
    pub item: T,
}

#[derive(Debug, Default)]
pub struct RosterProjection<T> {
    pending_stream_outline_update: Option<OutlineUpdate<T>>,
    last_known_agent_names: HashMap<String, String>,
}

impl<T> RosterProjection<T> {
    pub fn queue_stream_outline_update(&mut self, agent_id: impl Into<String>, item: T) {
        self.pending_stream_outline_update = Some(OutlineUpdate {
            agent_id: agent_id.into(),
            item,
        });
    }

    pub fn flush_stream_outline_update(&mut self) -> Option<OutlineUpdate<T>> {
        self.pending_stream_outline_update.take()
    }

    pub fn stop_outline_stream_coalescing(&mut self) {
        self.pending_stream_outline_update = None;
    }

    pub fn note_agent_name(&mut self, agent_id: impl Into<String>, name: impl Into<String>) {
        self.last_known_agent_names.insert(agent_id.into(), name.into());
    }

    pub fn last_known_agent_name(&self, agent_id: &str) -> Option<&str> {
        self.last_known_agent_names.get(agent_id).map(String::as_str)
    }
}
