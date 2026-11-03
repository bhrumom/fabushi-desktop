use mahayana_core::{ConversationId, RunId, TurnId, TurnState};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as AsyncMutex;

#[derive(Debug, Default)]
struct ConversationActorState {
    queue: VecDeque<TurnId>,
    active_turn: Option<TurnId>,
    active_run: Option<RunId>,
    states: HashMap<TurnId, TurnState>,
    sequence: u64,
}

#[derive(Debug)]
pub(crate) enum ActorAdmissionError<E> {
    Synchronization(String),
    Persistence(E),
}

#[derive(Debug)]
pub(crate) struct ActorAdmission {
    pub initial_sequence: u64,
    pub queued_sequence: Option<u64>,
}

/// One serial execution lane per conversation.
///
/// A conversation owns its queue/gate independently from other conversations,
/// so Agent A can run while Agent B runs, but retries or overlapping sends in
/// the same conversation can never concurrently mutate one provider session.
pub struct ConversationActor {
    conversation_id: ConversationId,
    pub(crate) gate: AsyncMutex<()>,
    state: Mutex<ConversationActorState>,
}

impl ConversationActor {
    fn new(conversation_id: ConversationId) -> Self {
        Self {
            conversation_id,
            gate: AsyncMutex::new(()),
            state: Mutex::new(ConversationActorState::default()),
        }
    }

    pub fn conversation_id(&self) -> &ConversationId {
        &self.conversation_id
    }

    /// Prepare a candidate without publishing it until the store commits.
    /// The private synchronous closure may only record this admission in the
    /// RuntimeStore; provider calls, event sends and extension hooks stay outside.
    pub(crate) fn admit_turn<E>(
        &self,
        turn_id: TurnId,
        retrying: bool,
        commit: impl FnOnce(TurnState) -> Result<(), E>,
    ) -> Result<ActorAdmission, ActorAdmissionError<E>> {
        let mut state = self.state.lock().map_err(|_| {
            ActorAdmissionError::Synchronization("conversation actor mutex poisoned".to_string())
        })?;
        let queued = state.active_run.is_some() || !state.queue.is_empty();
        let accepted_sequence = state.sequence.saturating_add(1);
        let initial_sequence = if retrying {
            accepted_sequence.saturating_add(1)
        } else {
            accepted_sequence
        };
        let queued_sequence = queued.then(|| initial_sequence.saturating_add(1));
        let final_state = if queued {
            TurnState::Queued
        } else if retrying {
            TurnState::Recovering
        } else {
            TurnState::Accepted
        };
        commit(final_state).map_err(ActorAdmissionError::Persistence)?;

        // No compensation by TurnId: a different accepted retry can share it.
        state.queue.push_back(turn_id.clone());
        state.states.insert(turn_id, final_state);
        state.sequence = queued_sequence.unwrap_or(initial_sequence);
        Ok(ActorAdmission { initial_sequence, queued_sequence })
    }

    pub fn start(&self, turn_id: &TurnId, run_id: RunId) -> Result<u64, String> {
        let mut state = self.state.lock().map_err(|_| "conversation actor mutex poisoned")?;
        if let Some(index) = state.queue.iter().position(|candidate| candidate == turn_id) {
            state.queue.remove(index);
        }
        state.active_turn = Some(turn_id.clone());
        state.active_run = Some(run_id);
        state.sequence = state.sequence.saturating_add(1);
        Ok(state.sequence)
    }

    pub fn finish(&self, run_id: &RunId) -> Result<u64, String> {
        let mut state = self.state.lock().map_err(|_| "conversation actor mutex poisoned")?;
        if state.active_run.as_ref() == Some(run_id) {
            state.active_run = None;
            state.active_turn = None;
        }
        state.sequence = state.sequence.saturating_add(1);
        Ok(state.sequence)
    }

    pub fn set_state(&self, turn_id: &TurnId, next: TurnState) -> Result<Option<u64>, String> {
        let mut state = self.state.lock().map_err(|_| "conversation actor mutex poisoned")?;
        if state.states.get(turn_id).copied() == Some(next) {
            return Ok(None);
        }
        state.states.insert(turn_id.clone(), next);
        state.sequence = state.sequence.saturating_add(1);
        Ok(Some(state.sequence))
    }

    #[cfg(test)]
    pub(crate) fn poison_state_for_admission_test(&self) {
        let poisoned = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _state = self.state.lock().expect("unpoisoned actor before injection");
            panic!("poison actual actor state for dispatch admission test");
        }));
        assert!(poisoned.is_err());
        assert!(self.state.is_poisoned());
    }
}

#[derive(Default)]
pub struct ConversationActorRegistry {
    actors: Mutex<HashMap<String, Arc<ConversationActor>>>,
}

impl ConversationActorRegistry {
    pub fn actor(&self, conversation_id: &ConversationId) -> Result<Arc<ConversationActor>, String> {
        let mut actors = self.actors.lock().map_err(|_| "conversation actor registry mutex poisoned")?;
        Ok(Arc::clone(
            actors
                .entry(conversation_id.to_string())
                .or_insert_with(|| Arc::new(ConversationActor::new(conversation_id.clone()))),
        ))
    }

    pub fn clear(&self) -> Result<(), String> {
        self.actors
            .lock()
            .map_err(|_| "conversation actor registry mutex poisoned")?
            .clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_turn_is_queued_until_first_run_finishes() {
        let registry = ConversationActorRegistry::default();
        let conversation = ConversationId::new("mahayana-ai:agent:test").unwrap();
        let actor = registry.actor(&conversation).unwrap();
        let first = TurnId::new("turn:first").unwrap();
        let second = TurnId::new("turn:second").unwrap();
        assert!(actor.admit_turn(first.clone(), false, |_| Ok::<_, ()>(()))
            .unwrap().queued_sequence.is_none());
        actor.start(&first, RunId::new("run:first").unwrap()).unwrap();
        assert!(actor.admit_turn(second, false, |_| Ok::<_, ()>(()))
            .unwrap().queued_sequence.is_some());
        actor.finish(&RunId::new("run:first").unwrap()).unwrap();
    }

    #[test]
    fn registry_returns_one_actor_per_conversation_and_never_globalizes_the_lane() {
        let registry = ConversationActorRegistry::default();
        let conversation_a = ConversationId::new("mahayana-ai:agent:a").unwrap();
        let conversation_b = ConversationId::new("mahayana-ai:agent:b").unwrap();
        let actor_a1 = registry.actor(&conversation_a).unwrap();
        let actor_a2 = registry.actor(&conversation_a).unwrap();
        let actor_b = registry.actor(&conversation_b).unwrap();

        assert!(Arc::ptr_eq(&actor_a1, &actor_a2));
        assert!(!Arc::ptr_eq(&actor_a1, &actor_b));
        assert_eq!(actor_a1.conversation_id(), &conversation_a);
        assert_eq!(actor_b.conversation_id(), &conversation_b);
    }

    #[test]
    fn lifecycle_sequence_advances_only_through_actor_owned_transitions() {
        let registry = ConversationActorRegistry::default();
        let conversation = ConversationId::new("mahayana-ai:agent:lifecycle").unwrap();
        let actor = registry.actor(&conversation).unwrap();
        let turn = TurnId::new("turn:lifecycle").unwrap();
        let run = RunId::new("run:lifecycle").unwrap();

        let registered_sequence = actor.admit_turn(turn.clone(), false, |_| Ok::<_, ()>(()))
            .unwrap().initial_sequence;
        let started_sequence = actor.start(&turn, run.clone()).unwrap();
        let state_sequence = actor.set_state(&turn, TurnState::Thinking).unwrap().unwrap();
        let duplicate_state_sequence = actor.set_state(&turn, TurnState::Thinking).unwrap();
        let finished_sequence = actor.finish(&run).unwrap();

        assert!(registered_sequence < started_sequence);
        assert!(started_sequence < state_sequence);
        assert_eq!(duplicate_state_sequence, None);
        assert!(state_sequence < finished_sequence);
    }

    #[test]
    fn refused_same_turn_candidate_keeps_the_existing_registration_and_sequence() {
        let registry = ConversationActorRegistry::default();
        let conversation = ConversationId::new("mahayana-ai:agent:refused-candidate").unwrap();
        let actor = registry.actor(&conversation).unwrap();
        let turn = TurnId::new("turn:shared-retry").unwrap();
        let accepted = actor.admit_turn(turn.clone(), true, |_| Ok::<_, &str>(())).unwrap();
        let refused = actor.admit_turn(turn.clone(), true, |state| {
            assert_eq!(state, TurnState::Queued);
            Err::<(), _>("store refused candidate")
        });
        assert!(matches!(refused, Err(ActorAdmissionError::Persistence("store refused candidate"))));
        let state = actor.state.lock().unwrap();
        assert_eq!(state.queue, VecDeque::from([turn.clone()]));
        assert_eq!(state.states.len(), 1);
        assert_eq!(state.states.get(&turn), Some(&TurnState::Recovering));
        assert_eq!(state.sequence, accepted.initial_sequence);
        assert_eq!(state.active_turn, None);
        assert_eq!(state.active_run, None);
    }
}
