use std::collections::BTreeMap;
use std::sync::Arc;

pub const SAND_MOBILE_PUSH_FOCUS_FRESHNESS_MS: u64 = 5 * 60_000;
pub const SAND_OS_NOTIFICATION_THROTTLE_MS: u64 = 5_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationAgent {
    pub id: String,
    pub name: String,
    pub is_running: bool,
    pub awaiting_reason: Option<String>,
    pub notify_enabled: bool,
    pub is_hidden_from_sidebar: bool,
    pub last_message_id: Option<String>,
    pub last_message_preview: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MobilePushInput {
    pub agent_id: String,
    pub agent_name: String,
    pub message_preview: String,
    pub last_message_id: String,
    pub awaiting_user_response: bool,
}

#[derive(Debug, Clone)]
struct Snapshot {
    agent: NotificationAgent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum TransitionKind {
    Done,
    NeedsInput,
}

#[derive(Debug, Clone)]
struct Transition {
    agent_id: String,
    agent_name: String,
    kind: TransitionKind,
    last_message_id: Option<String>,
    last_message_preview: Option<String>,
}

#[derive(Default)]
struct NotificationDecider {
    previous: BTreeMap<String, Snapshot>,
    last_notified_at_ms: BTreeMap<(String, TransitionKind), u64>,
    accounted_message_id: BTreeMap<String, Option<String>>,
}

impl NotificationDecider {
    fn seed_baseline(&mut self, agents: &[NotificationAgent]) {
        for agent in agents {
            self.previous.entry(agent.id.clone()).or_insert_with(|| Snapshot { agent: agent.clone() });
            self.accounted_message_id.entry(agent.id.clone()).or_insert_with(|| agent.last_message_id.clone());
        }
    }

    fn decide_agent(
        &mut self,
        agent: &NotificationAgent,
        now_ms: u64,
        is_window_focused: bool,
    ) -> Vec<Transition> {
        let mut out = Vec::new();
        if let Some(before) = self.previous.get(&agent.id).map(|snapshot| snapshot.agent.clone()) {
            let became_awaiting = agent.awaiting_reason.is_some() && before.awaiting_reason.is_none();
            let finished_turn = before.is_running && !agent.is_running && agent.awaiting_reason.is_none();
            let kind = if became_awaiting {
                Some(TransitionKind::NeedsInput)
            } else if finished_turn {
                Some(TransitionKind::Done)
            } else {
                None
            };
            if let Some(kind) = kind {
                let accounted = self.accounted_message_id.get(&agent.id).cloned().flatten();
                let duplicate_done = kind == TransitionKind::Done
                    && (agent.last_message_id.is_none() || agent.last_message_id == accounted);
                self.accounted_message_id.insert(agent.id.clone(), agent.last_message_id.clone());
                let key = (agent.id.clone(), kind);
                let throttled = self.last_notified_at_ms.get(&key)
                    .is_some_and(|last| now_ms.saturating_sub(*last) < SAND_OS_NOTIFICATION_THROTTLE_MS);
                if !duplicate_done
                    && !agent.is_hidden_from_sidebar
                    && agent.notify_enabled
                    && !is_window_focused
                    && !throttled
                {
                    self.last_notified_at_ms.insert(key, now_ms);
                    out.push(Transition {
                        agent_id: agent.id.clone(),
                        agent_name: agent.name.clone(),
                        kind,
                        last_message_id: agent.last_message_id.clone(),
                        last_message_preview: agent.last_message_preview.clone(),
                    });
                }
            }
        } else {
            self.accounted_message_id.insert(agent.id.clone(), agent.last_message_id.clone());
        }
        self.previous.insert(agent.id.clone(), Snapshot { agent: agent.clone() });
        out
    }

    fn forget(&mut self, agent_id: &str) {
        self.previous.remove(agent_id);
        self.accounted_message_id.remove(agent_id);
        self.last_notified_at_ms.retain(|(id, _), _| id != agent_id);
    }
}

type Notify = Arc<dyn Fn(MobilePushInput) -> Result<(), String> + Send + Sync>;

pub struct SandMobilePushNotifier {
    decider: NotificationDecider,
    notify: Notify,
    has_seeded_baseline: bool,
    pre_seed_deltas: Vec<(NotificationAgent, Option<u64>)>,
}

impl SandMobilePushNotifier {
    pub fn new(notify: Notify) -> Self {
        Self {
            decider: NotificationDecider::default(),
            notify,
            has_seeded_baseline: false,
            pre_seed_deltas: Vec::new(),
        }
    }

    pub fn seed_baseline(&mut self, agents: &[NotificationAgent], now_ms: u64) {
        self.decider.seed_baseline(agents);
        self.flush_pre_seed_deltas(now_ms);
    }

    pub fn handle_agents_event(
        &mut self,
        agents: &[NotificationAgent],
        window_focused_at_ms: Option<u64>,
        now_ms: u64,
    ) {
        let focused = is_focused(window_focused_at_ms, now_ms);
        for agent in agents {
            self.fire_transitions(self.decider.decide_agent(agent, now_ms, focused));
        }
        self.flush_pre_seed_deltas(now_ms);
    }

    pub fn handle_agent_upserted(
        &mut self,
        agent: NotificationAgent,
        window_focused_at_ms: Option<u64>,
        now_ms: u64,
    ) {
        if !self.has_seeded_baseline {
            self.pre_seed_deltas.push((agent, window_focused_at_ms));
            return;
        }
        let focused = is_focused(window_focused_at_ms, now_ms);
        let transitions = self.decider.decide_agent(&agent, now_ms, focused);
        self.fire_transitions(transitions);
    }

    pub fn forget(&mut self, agent_id: &str) {
        self.decider.forget(agent_id);
    }

    fn flush_pre_seed_deltas(&mut self, now_ms: u64) {
        if self.has_seeded_baseline {
            return;
        }
        self.has_seeded_baseline = true;
        let buffered = std::mem::take(&mut self.pre_seed_deltas);
        for (agent, focused_at) in buffered {
            self.handle_agent_upserted(agent, focused_at, now_ms);
        }
    }

    fn fire_transitions(&self, transitions: Vec<Transition>) {
        for transition in transitions {
            let _ = (self.notify)(MobilePushInput {
                agent_id: transition.agent_id,
                agent_name: transition.agent_name,
                message_preview: transition.last_message_preview.unwrap_or_default(),
                last_message_id: transition.last_message_id.unwrap_or_default(),
                awaiting_user_response: transition.kind == TransitionKind::NeedsInput,
            });
        }
    }
}

fn is_focused(window_focused_at_ms: Option<u64>, now_ms: u64) -> bool {
    window_focused_at_ms.is_some_and(|focused_at| {
        now_ms.saturating_sub(focused_at) <= SAND_MOBILE_PUSH_FOCUS_FRESHNESS_MS
    })
}
