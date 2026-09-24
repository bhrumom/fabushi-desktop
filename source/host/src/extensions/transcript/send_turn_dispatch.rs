use super::run_scheduler::{
    QueueAccepted, QueueDequeued, QueuedRun, RunLane, RunScheduler, RunSettlement,
    WatchdogEvent, WatchdogStage,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserTurnTicket {
    pub agent_id: String,
    pub task_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnWatchdogTick {
    pub event: WatchdogEvent,
    pub started_next: Option<QueueDequeued>,
}

#[derive(Debug)]
pub struct ProductionTurnDispatch {
    scheduler: RunScheduler,
    next_task_seq: u64,
}

impl Default for ProductionTurnDispatch {
    fn default() -> Self {
        Self {
            scheduler: RunScheduler::default(),
            next_task_seq: 0,
        }
    }
}

impl ProductionTurnDispatch {
    pub fn with_watchdog(watchdog_ms: u64, watchdog_grace_ms: u64) -> Self {
        Self {
            scheduler: RunScheduler::new(watchdog_ms, watchdog_grace_ms),
            next_task_seq: 0,
        }
    }
}

impl ProductionTurnDispatch {
    pub fn watchdog_wait_ms(&self, agent_id: &str, now_ms: u64) -> Option<u64> {
        self.scheduler.next_watchdog_delay_ms(agent_id, now_ms)
    }

    pub fn watchdog_tick(
        &mut self,
        agent_id: &str,
        now_ms: u64,
    ) -> Option<TurnWatchdogTick> {
        let event = self.scheduler.watchdog_tick(agent_id, now_ms)?;
        let started_next = if event.stage == WatchdogStage::Escape {
            self.scheduler.start_next(agent_id, now_ms)
        } else {
            None
        };
        Some(TurnWatchdogTick {
            event,
            started_next,
        })
    }

    pub fn enqueue_turn(
        &mut self,
        agent_id: &str,
        client_nonce: Option<&str>,
        accepted_at_ms: u64,
        now_ms: u64,
        lane: RunLane,
        source: &str,
        ack_token: Option<&str>,
    ) -> Result<(UserTurnTicket, QueueAccepted), &'static str> {
        let (ticket, accepted, _) = self.enqueue_turn_with_start(
            agent_id, client_nonce, accepted_at_ms, now_ms, lane, source, ack_token,
        )?;
        Ok((ticket, accepted))
    }

    pub fn enqueue_turn_with_start(
        &mut self,
        agent_id: &str,
        client_nonce: Option<&str>,
        accepted_at_ms: u64,
        now_ms: u64,
        lane: RunLane,
        source: &str,
        ack_token: Option<&str>,
    ) -> Result<(UserTurnTicket, QueueAccepted, Option<QueueDequeued>), &'static str> {
        self.next_task_seq = self.next_task_seq.saturating_add(1);
        let nonce = client_nonce
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("no-nonce");
        let task_id = format!("send:{agent_id}:{}:{nonce}", self.next_task_seq);
        let accepted = self.scheduler.enqueue(
            agent_id.to_string(),
            QueuedRun {
                task_id: task_id.clone(),
                lane,
                source: source.to_string(),
                enqueued_at_ms: now_ms,
                accepted_at_ms: Some(accepted_at_ms),
                ack_token: ack_token.map(ToOwned::to_owned),
            },
        )?;
        let started = if self.scheduler.active(agent_id).is_none() {
            self.scheduler.start_next(agent_id, now_ms)
        } else {
            None
        };
        Ok((
            UserTurnTicket {
                agent_id: agent_id.to_string(),
                task_id,
            },
            accepted,
            started,
        ))
    }

    pub fn enqueue_user_turn(
        &mut self,
        agent_id: &str,
        client_nonce: Option<&str>,
        accepted_at_ms: u64,
        now_ms: u64,
    ) -> Result<(UserTurnTicket, QueueAccepted), &'static str> {
        self.enqueue_turn(
            agent_id,
            client_nonce,
            accepted_at_ms,
            now_ms,
            RunLane::User,
            "turn",
            None,
        )
    }

    pub fn active_generation_for(&self, ticket: &UserTurnTicket) -> Option<u64> {
        let active = self.scheduler.active(&ticket.agent_id)?;
        (active.item.task_id == ticket.task_id).then_some(active.generation)
    }

    pub fn settle_and_start_next(
        &mut self,
        ticket: &UserTurnTicket,
        generation: u64,
        now_ms: u64,
    ) -> (RunSettlement, Option<QueueDequeued>) {
        let settlement = self
            .scheduler
            .settle(&ticket.agent_id, generation, now_ms);
        let next = self.scheduler.start_next(&ticket.agent_id, now_ms);
        (settlement, next)
    }

    pub fn queued_task_ids(&self, agent_id: &str) -> Vec<String> {
        self.scheduler.queued_task_ids(agent_id)
    }

    pub fn active_lane(&self, agent_id: &str) -> Option<RunLane> {
        self.scheduler.get_active_lane(agent_id)
    }

    pub fn active_source(&self, agent_id: &str) -> Option<&str> {
        self.scheduler.active(agent_id).map(|active| active.item.source.as_str())
    }

    pub fn is_idle(&self, agent_id: &str) -> bool {
        self.scheduler.is_idle(agent_id)
    }
}
