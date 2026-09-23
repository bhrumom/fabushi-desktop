use std::collections::HashMap;
use std::io;
use std::path::Path;
use std::sync::Mutex;

use uuid::Uuid;

use super::sand_ack_obligation_store::{
    AckObligation, RecordSendOutcome, SandAckObligationStore,
};

pub const MAX_ACK_REDRIVES: u64 = 3;
pub const ACK_REDRIVE_IDLE_DELAY_MS: u64 = 5_000;
pub const RUNNER_PREPARE_ACK_OBLIGATION_GATEWAY_METHOD: &str =
    "runner.prepareAckObligation";
pub const RUNNER_ROLLBACK_ACK_OBLIGATION_GATEWAY_METHOD: &str =
    "runner.rollbackAckObligation";

pub fn build_ack_redrive_prompt() -> &'static str {
    "[System recovery] The user sent one or more messages that were never visibly acknowledged — the turns handling them were interrupted, or the app restarted before a reply went out. Their newest message may be MISSING from your context entirely. Respond now by actually invoking the SendMessage tool: if you can see their latest message and already completed what it asked, send a brief confirmation with the result; if you can see it but the work is not done, acknowledge them and continue the work; if you cannot be certain what they last asked, say you may have missed their latest message and ask them to resend it — NEVER guess or claim completion of work you cannot see. Plain assistant text is NEVER shown to the user; only a real SendMessage tool invocation reaches them. Do NOT end this turn with only thinking, an empty reply, or a plan to send later — ending the turn without a real SendMessage invocation delivers nothing and is a failure. Invoke SendMessage now, even if all you can send is a brief status update."
}

#[derive(Debug, Clone, PartialEq)]
pub struct AckReservation {
    pub ack_token: String,
    pub obligation: AckObligation,
    pub created: bool,
}

#[derive(Debug, Clone)]
struct ReservationState {
    agent_id: String,
    previous: Option<AckObligation>,
    recorded: AckObligation,
}

pub struct AckObligations {
    store: SandAckObligationStore,
    reservations: Mutex<HashMap<String, ReservationState>>,
}

impl AckObligations {
    pub fn new(root_dir: impl AsRef<Path>) -> Self {
        Self {
            store: SandAckObligationStore::new(root_dir),
            reservations: Mutex::new(HashMap::new()),
        }
    }

    pub fn store(&self) -> &SandAckObligationStore {
        &self.store
    }

    pub fn record_send(
        &self,
        agent_id: &str,
        at_ms: f64,
    ) -> io::Result<RecordSendOutcome> {
        self.store.record_send(agent_id, at_ms)
    }

    pub fn mint_ack_run_token(
        &self,
        agent_id: &str,
    ) -> io::Result<Option<String>> {
        let Some(recorded) = self.store.get(agent_id) else {
            return Ok(None);
        };
        let ack_token = Uuid::new_v4().to_string();
        self.reservations
            .lock()
            .map_err(|_| io::Error::other("ack reservation registry poisoned"))?
            .insert(
                ack_token.clone(),
                ReservationState {
                    agent_id: agent_id.to_string(),
                    previous: Some(recorded.clone()),
                    recorded,
                },
            );
        Ok(Some(ack_token))
    }

    pub fn record_send_and_mint_token(
        &self,
        agent_id: &str,
        at_ms: f64,
    ) -> io::Result<AckReservation> {
        let previous = self.store.get(agent_id);
        let RecordSendOutcome {
            obligation,
            created,
        } = self.store.record_send(agent_id, at_ms)?;
        let ack_token = Uuid::new_v4().to_string();
        self.reservations
            .lock()
            .map_err(|_| io::Error::other("ack reservation registry poisoned"))?
            .insert(
                ack_token.clone(),
                ReservationState {
                    agent_id: agent_id.to_string(),
                    previous,
                    recorded: obligation.clone(),
                },
            );
        Ok(AckReservation {
            ack_token,
            obligation,
            created,
        })
    }

    pub fn fulfill_ack_obligation(&self, agent_id: &str, ack_token: &str) -> io::Result<bool> {
        if !self.token_matches_agent(agent_id, ack_token)? {
            return Ok(false);
        }
        self.store.clear(agent_id)
    }

    pub fn rollback_ack_reservation(&self, agent_id: &str, ack_token: &str) -> io::Result<bool> {
        let reservation = {
            let mut reservations = self
                .reservations
                .lock()
                .map_err(|_| io::Error::other("ack reservation registry poisoned"))?;
            let Some(reservation) = reservations.get(ack_token) else {
                return Ok(false);
            };
            if reservation.agent_id != agent_id {
                return Ok(false);
            }
            reservations.remove(ack_token).expect("reservation existed")
        };
        if self.store.get(agent_id).as_ref() != Some(&reservation.recorded) {
            return Ok(false);
        }
        self.store.restore(reservation.previous, agent_id)?;
        Ok(true)
    }

    pub fn retire_ack_run_token(&self, agent_id: &str, ack_token: Option<&str>) -> bool {
        let Some(ack_token) = ack_token else {
            return false;
        };
        let Ok(mut reservations) = self.reservations.lock() else {
            return false;
        };
        if reservations
            .get(ack_token)
            .is_some_and(|reservation| reservation.agent_id == agent_id)
        {
            reservations.remove(ack_token);
            true
        } else {
            false
        }
    }

    pub fn record_interrupt(&self, agent_id: &str, at_ms: f64) -> io::Result<bool> {
        self.store.record_interrupt(agent_id, at_ms)
    }

    pub fn record_redrive_attempt(&self, agent_id: &str) -> io::Result<Option<AckObligation>> {
        self.store.record_redrive_attempt(agent_id)
    }

    pub fn clear_lost(&self, agent_id: &str) -> io::Result<bool> {
        self.store.clear(agent_id)
    }

    pub fn forget_agent(&self, agent_id: &str) -> io::Result<bool> {
        let cleared = self.store.clear(agent_id)?;
        let mut reservations = self
            .reservations
            .lock()
            .map_err(|_| io::Error::other("ack reservation registry poisoned"))?;
        let before = reservations.len();
        reservations.retain(|_, reservation| reservation.agent_id != agent_id);
        Ok(cleared || reservations.len() != before)
    }

    fn token_matches_agent(&self, agent_id: &str, ack_token: &str) -> io::Result<bool> {
        Ok(self
            .reservations
            .lock()
            .map_err(|_| io::Error::other("ack reservation registry poisoned"))?
            .get(ack_token)
            .is_some_and(|reservation| reservation.agent_id == agent_id))
    }
}
