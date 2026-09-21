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

    pub fn register(&self, turn_id: TurnId) -> Result<(bool, u64), String> {
        let mut state = self.state.lock().map_err(|_| "conversation actor mutex poisoned")?;
        let queued = state.active_run.is_some() || !state.queue.is_empty();
        state.queue.push_back(turn_id.clone());
        state.states.insert(turn_id, TurnState::Accepted);
        state.sequence = state.sequence.saturating_add(1);
        Ok((queued, state.sequence))
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

    #[tokio::test]
    async fn registry_reuses_actor_and_gates_only_the_same_conversation() {
        let registry = ConversationActorRegistry::default();
        let first_id = ConversationId::new("mahayana-ai:agent:first").unwrap();
        let second_id = ConversationId::new("mahayana-ai:agent:second").unwrap();
        let first = registry.actor(&first_id).unwrap();
        let first_again = registry.actor(&first_id).unwrap();
        let second = registry.actor(&second_id).unwrap();

        assert!(Arc::ptr_eq(&first, &first_again));
        assert!(!Arc::ptr_eq(&first, &second));

        let first_guard = first.gate.lock().await;
        assert!(first.gate.try_lock().is_err(), "same conversation must serialize");
        assert!(second.gate.try_lock().is_ok(), "other conversations must remain independently runnable");
        drop(first_guard);
        assert!(first.gate.try_lock().is_ok());
    }

    #[test]
    fn actor_owns_turn_state_sequence_and_active_run_lifecycle() {
        let registry = ConversationActorRegistry::default();
        let conversation = ConversationId::new("mahayana-ai:agent:lifecycle").unwrap();
        let actor = registry.actor(&conversation).unwrap();
        let turn = TurnId::new("turn:lifecycle").unwrap();
        let run = RunId::new("run:lifecycle").unwrap();

        let (_, accepted_sequence) = actor.register(turn.clone()).unwrap();
        let started_sequence = actor.start(&turn, run.clone()).unwrap();
        let thinking_sequence = actor.set_state(&turn, TurnState::Thinking).unwrap().unwrap();
        let duplicate = actor.set_state(&turn, TurnState::Thinking).unwrap();
        let finished_sequence = actor.finish(&run).unwrap();

        assert!(accepted_sequence < started_sequence);
        assert!(started_sequence < thinking_sequence);
        assert!(thinking_sequence < finished_sequence);
        assert!(duplicate.is_none(), "identical lifecycle state must not emit a new sequence");

        let state = actor.state.lock().unwrap();
        assert_eq!(state.states.get(&turn), Some(&TurnState::Thinking));
        assert!(state.active_run.is_none());
        assert!(state.active_turn.is_none());
    }

    #[test]
    fn second_turn_is_queued_until_first_run_finishes() {
        let registry = ConversationActorRegistry::default();
        let conversation = ConversationId::new("mahayana-ai:agent:test").unwrap();
        let actor = registry.actor(&conversation).unwrap();
        let first = TurnId::new("turn:first").unwrap();
        let second = TurnId::new("turn:second").unwrap();
        assert!(!actor.register(first.clone()).unwrap().0);
        actor.start(&first, RunId::new("run:first").unwrap()).unwrap();
        assert!(actor.register(second).unwrap().0);
        actor.finish(&RunId::new("run:first").unwrap()).unwrap();
    }
}
