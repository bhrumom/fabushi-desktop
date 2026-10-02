use std::collections::HashMap;

pub const OUTLINE_STREAM_COALESCE_MS: u64 = 250;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlineUpdate<T> {
    pub agent_id: String,
    pub item: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutlineStreamDeadline {
    pub at_ms: u64,
    pub generation: u64,
}

#[derive(Debug)]
pub struct OutlineQueueResult<T> {
    pub flushed: Vec<OutlineUpdate<T>>,
    pub deadline: Option<OutlineStreamDeadline>,
}

#[derive(Debug)]
struct PendingOutlineUpdate<T> {
    item_id: String,
    update: OutlineUpdate<T>,
}

#[derive(Debug)]
pub struct OutlineStreamCoalescingPolicy<T> {
    delay_ms: u64,
    enabled: bool,
    pending: Option<PendingOutlineUpdate<T>>,
    last_flush_at_ms: u64,
    deadline: Option<OutlineStreamDeadline>,
    generation: u64,
}

impl<T> OutlineStreamCoalescingPolicy<T> {
    pub fn new(delay_ms: u64, now_ms: u64) -> Self {
        Self {
            delay_ms,
            enabled: true,
            pending: None,
            last_flush_at_ms: now_ms,
            deadline: None,
            generation: 0,
        }
    }

    pub fn delay_ms(&self) -> u64 {
        self.delay_ms
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn has_pending(&self) -> bool {
        self.pending.is_some()
    }

    pub fn next_deadline(&self) -> Option<OutlineStreamDeadline> {
        self.deadline
    }

    pub fn queue(
        &mut self,
        now_ms: u64,
        agent_id: impl Into<String>,
        item_id: impl Into<String>,
        item: T,
    ) -> OutlineQueueResult<T> {
        let agent_id = agent_id.into();
        let item_id = item_id.into();
        let update = OutlineUpdate { agent_id, item };

        if !self.enabled {
            return OutlineQueueResult {
                flushed: vec![update],
                deadline: None,
            };
        }

        let mut flushed = Vec::new();
        if self
            .pending
            .as_ref()
            .is_some_and(|pending| pending.item_id != item_id)
        {
            if let Some(update) = self.flush(now_ms) {
                flushed.push(update);
            }
        }

        self.pending = Some(PendingOutlineUpdate { item_id, update });
        let generation = self.bump_generation();

        if now_ms.saturating_sub(self.last_flush_at_ms) >= self.delay_ms {
            if let Some(update) = self.flush(now_ms) {
                flushed.push(update);
            }
            return OutlineQueueResult {
                flushed,
                deadline: None,
            };
        }

        let deadline = OutlineStreamDeadline {
            at_ms: now_ms.saturating_add(self.delay_ms),
            generation,
        };
        self.deadline = Some(deadline);
        OutlineQueueResult {
            flushed,
            deadline: Some(deadline),
        }
    }

    pub fn flush_deadline(
        &mut self,
        now_ms: u64,
        generation: u64,
    ) -> Option<OutlineUpdate<T>> {
        let deadline = self.deadline?;
        if deadline.generation != generation || now_ms < deadline.at_ms {
            return None;
        }
        self.flush(now_ms)
    }

    pub fn flush(&mut self, now_ms: u64) -> Option<OutlineUpdate<T>> {
        let pending = self.pending.take()?;
        self.deadline = None;
        self.last_flush_at_ms = now_ms;
        self.bump_generation();
        Some(pending.update)
    }

    pub fn stop(&mut self) {
        self.enabled = false;
        self.pending = None;
        self.deadline = None;
        self.bump_generation();
    }

    fn bump_generation(&mut self) -> u64 {
        self.generation = self.generation.wrapping_add(1);
        if self.generation == 0 {
            self.generation = 1;
        }
        self.generation
    }
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
