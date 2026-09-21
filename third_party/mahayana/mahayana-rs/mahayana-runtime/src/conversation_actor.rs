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
        if state.states.contains_key(&turn_id) || state.queue.iter().any(|candidate| candidate == &turn_id) {
            return Err(format!("turn {} is already registered", turn_id));
        }
        let queued = state.active_run.is_some() || !state.queue.is_empty();
        state.queue.push_back(turn_id.clone());
        state.states.insert(turn_id, TurnState::Accepted);
        state.sequence = state.sequence.saturating_add(1);
        Ok((queued, state.sequence))
    }

    pub fn start(&self, turn_id: &TurnId, run_id: RunId) -> Result<u64, String> {
        let mut state = self.state.lock().map_err(|_| "conversation actor mutex poisoned")?;
        if state.active_run.is_some() {
            return Err("conversation actor already has an active run".to_string());
        }
        if !state.states.contains_key(turn_id) {
            return Err(format!("turn {} was not registered with this conversation", turn_id));
        }
        let Some(index) = state.queue.iter().position(|candidate| candidate == turn_id) else {
            return Err(format!("turn {} is not queued for execution", turn_id));
        };
        state.queue.remove(index);
        state.active_turn = Some(turn_id.clone());
        state.active_run = Some(run_id);
        state.sequence = state.sequence.saturating_add(1);
        Ok(state.sequence)
    }

    pub fn finish(&self, run_id: &RunId) -> Result<u64, String> {
        let mut state = self.state.lock().map_err(|_| "conversation actor mutex poisoned")?;
        if state.active_run.as_ref() != Some(run_id) {
            return Err(format!("run {} does not own the active conversation lane", run_id));
        }
        state.active_run = None;
        state.active_turn = None;
        state.sequence = state.sequence.saturating_add(1);
        Ok(state.sequence)
    }

    pub fn set_state(&self, turn_id: &TurnId, next: TurnState) -> Result<Option<u64>, String> {
        let mut state = self.state.lock().map_err(|_| "conversation actor mutex poisoned")?;
        let Some(current) = state.states.get(turn_id).copied() else {
            return Err(format!("turn {} was not registered with this conversation", turn_id));
        };
        if current == next {
            return Ok(None);
        }
        if current.terminal() {
            return Err(format!("terminal turn {} cannot transition from {} to {}", turn_id, current.as_str(), next.as_str()));
        }
        if next == TurnState::Accepted {
            return Err(format!("turn {} cannot transition back to accepted", turn_id));
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

    #[test]
    fn registry_returns_the_same_actor_for_one_conversation_and_isolates_others() {
        let registry = ConversationActorRegistry::default();
        let first_id = ConversationId::new("conversation:first").unwrap();
        let second_id = ConversationId::new("conversation:second").unwrap();
        let first_a = registry.actor(&first_id).unwrap();
        let first_b = registry.actor(&first_id).unwrap();
        let second = registry.actor(&second_id).unwrap();
        assert!(Arc::ptr_eq(&first_a, &first_b));
        assert!(!Arc::ptr_eq(&first_a, &second));
    }

    #[test]
    fn actor_rejects_duplicate_unregistered_and_wrong_run_lifecycle_calls() {
        let conversation = ConversationId::new("conversation:guarded").unwrap();
        let actor = ConversationActor::new(conversation);
        let turn = TurnId::new("turn:guarded").unwrap();
        let run = RunId::new("run:guarded").unwrap();
        assert!(actor.start(&turn, run.clone()).is_err());
        actor.register(turn.clone()).unwrap();
        assert!(actor.register(turn.clone()).is_err());
        actor.start(&turn, run.clone()).unwrap();
        assert!(actor.start(&turn, RunId::new("run:other").unwrap()).is_err());
        assert!(actor.finish(&RunId::new("run:wrong").unwrap()).is_err());
        actor.finish(&run).unwrap();
    }

    #[test]
    fn terminal_turn_cannot_reenter_execution_lifecycle() {
        let conversation = ConversationId::new("conversation:terminal").unwrap();
        let actor = ConversationActor::new(conversation);
        let turn = TurnId::new("turn:terminal").unwrap();
        actor.register(turn.clone()).unwrap();
        actor.set_state(&turn, TurnState::Completed).unwrap();
        assert!(actor.set_state(&turn, TurnState::Thinking).is_err());
        assert!(actor.set_state(&turn, TurnState::Accepted).is_err());
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
