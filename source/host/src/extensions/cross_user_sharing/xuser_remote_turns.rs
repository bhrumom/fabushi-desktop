use super::xuser_turn_dedupe_store::SandXuserTurnDedupeStore;
use std::collections::VecDeque;
pub const REMOTE_MEMBER_UNREACHABLE_BACKOFF_MS: u64 = 10 * 60_000;
pub const REMOTE_TURN_BUDGET_WINDOW_MS: u64 = 10 * 60_000;
pub const REMOTE_TURN_BUDGET_MAX: usize = 30;

#[derive(Default)]
pub struct RemoteTurnBudget {
    accepted: VecDeque<u64>,
}
impl RemoteTurnBudget {
    pub fn admit(&mut self, now_ms: u64) -> bool {
        while self
            .accepted
            .front()
            .is_some_and(|v| now_ms.saturating_sub(*v) > REMOTE_TURN_BUDGET_WINDOW_MS)
        {
            self.accepted.pop_front();
        }
        if self.accepted.len() >= REMOTE_TURN_BUDGET_MAX {
            return false;
        }
        self.accepted.push_back(now_ms);
        true
    }
}
pub fn accept_remote_turn(
    dedupe: &SandXuserTurnDedupeStore,
    budget: &mut RemoteTurnBudget,
    nonce: &str,
    now_ms: u64,
) -> bool {
    !nonce.is_empty() && dedupe.mark_seen_if_new(nonce) && budget.admit(now_ms)
}
