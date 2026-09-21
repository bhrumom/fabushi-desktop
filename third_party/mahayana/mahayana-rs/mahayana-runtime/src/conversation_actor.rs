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
