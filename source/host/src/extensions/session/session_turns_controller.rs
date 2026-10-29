use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub turn_index: u32,
    pub user_input: String,
    pub agent_output: Option<String>,
    pub model_override: Option<String>,
    pub started_at_ms: u64,
    pub completed_at_ms: Option<u64>,
}

#[derive(Clone, Default)]
pub struct SessionTurnsController {
    sessions: Arc<Mutex<HashMap<String, Vec<ConversationTurn>>>>,
}

impl SessionTurnsController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_turn_start(
        &self,
        session_id: &str,
        user_input: &str,
        model_override: Option<String>,
    ) -> u32 {
        let now_ms = current_time_ms();
        let mut sessions = self.sessions.lock().unwrap();
        let turns = sessions.entry(session_id.to_string()).or_default();
        let turn_index = turns.len() as u32;

        turns.push(ConversationTurn {
            turn_index,
            user_input: user_input.to_string(),
            agent_output: None,
            model_override,
            started_at_ms: now_ms,
            completed_at_ms: None,
        });

        turn_index
    }

    pub fn record_turn_complete(&self, session_id: &str, turn_index: u32, output: &str) -> bool {
        let now_ms = current_time_ms();
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(turns) = sessions.get_mut(session_id) {
            if let Some(turn) = turns.get_mut(turn_index as usize) {
                turn.agent_output = Some(output.to_string());
                turn.completed_at_ms = Some(now_ms);
                return true;
            }
        }
        false
    }

    pub fn get_turns(&self, session_id: &str) -> Vec<ConversationTurn> {
        let sessions = self.sessions.lock().unwrap();
        sessions.get(session_id).cloned().unwrap_or_default()
    }
}

fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
