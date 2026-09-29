use std::collections::{HashMap, HashSet};

use serde_json::Value;
use uuid::Uuid;

use super::memory_service::{
    MemoryKind, MemoryOrigin, MemorySynthesisBridge, SynthesisApplyResult, SynthesisChange,
    SynthesisSnapshot, normalize_memory_content,
};

pub const MEMORY_SYNTHESIS_PROMPT_MARKER: &str = "<<SAND_MEMORY_SYNTHESIS_V1>>";
pub const MEMORY_SYNTHESIS_VERIFICATION_PROMPT_MARKER: &str =
    "<<SAND_MEMORY_SYNTHESIS_VERIFICATION_V1>>";
pub const MEMORY_SYNTHESIS_DEBOUNCE_MS: u64 = 15_000;
pub const MEMORY_SYNTHESIS_DEADLINE_MS: u64 = 90_000;
pub const MEMORY_SYNTHESIS_POLL_INTERVAL_MS: u64 = 3_600_000;
pub const MEMORY_SYNTHESIS_REFRESH_INTERVAL_MS: i64 = 86_400_000;
pub const MEMORY_SYNTHESIS_RETRY_ATTEMPTS: usize = 3;
pub const MEMORY_SYNTHESIS_RETRY_INITIAL_MS: u64 = 2_000;
pub const MEMORY_SYNTHESIS_RETRY_MAX_MS: u64 = 30_000;
pub const MAX_PENDING_AGENTS: usize = 64;
pub const MAX_PENDING_EVIDENCE_PER_AGENT: usize = 12;
pub const MAX_TEMPORAL_TARGETS_PER_SWEEP: usize = 4;
pub const MAX_EVIDENCE_SIDE_CHARS: usize = 8_000;
pub const MAX_SYNTHESIS_CHANGES: usize = 64;
pub const MAX_SOURCE_EVIDENCE_IDS: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryEvidence {
    pub id: String,
    pub occurred_at: i64,
    pub user: String,
    pub assistant: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryChange {
    Create {
        content: String,
        kind: MemoryKind,
        source_evidence_ids: Vec<String>,
    },
    Update {
        id: String,
        content: String,
        kind: MemoryKind,
        source_evidence_ids: Vec<String>,
    },
    Remove {
        id: String,
        source_evidence_ids: Vec<String>,
    },
}

impl MemoryChange {
    pub fn source_evidence_ids(&self) -> &[String] {
        match self {
            Self::Create {
                source_evidence_ids,
                ..
            }
            | Self::Update {
                source_evidence_ids,
                ..
            }
            | Self::Remove {
                source_evidence_ids,
                ..
            } => source_evidence_ids,
        }
    }

    pub fn to_store_change(&self) -> SynthesisChange {
        match self {
            Self::Create { content, kind, .. } => SynthesisChange::Create {
                content: content.clone(),
                kind: *kind,
            },
            Self::Update {
                id, content, kind, ..
            } => SynthesisChange::Update {
                id: id.clone(),
                content: content.clone(),
                kind: *kind,
            },
            Self::Remove { id, .. } => SynthesisChange::Remove { id: id.clone() },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SynthesisOutcome {
    Committed,
    NoWork,
    InvalidOutput,
    Rejected,
    Stale,
    Failed,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct PendingAgent {
    evidence: Vec<MemoryEvidence>,
    temporal: bool,
}

#[derive(Debug, Default)]
pub struct MemorySynthesisQueue {
    pending: HashMap<String, PendingAgent>,
    order: Vec<String>,
    needs_another_pass: bool,
}

impl MemorySynthesisQueue {
    pub fn record_turn(
        &mut self,
        agent_id: &str,
        evidence_id: Option<&str>,
        occurred_at: i64,
        user: &str,
        assistant: &str,
        active: bool,
    ) -> Option<String> {
        let user = bounded_evidence_text(user);
        let assistant = bounded_evidence_text(assistant);
        if user.is_empty() && assistant.is_empty() {
            return None;
        }

        let mut dropped_agent = None;
        if !self.pending.contains_key(agent_id) && self.pending.len() >= MAX_PENDING_AGENTS {
            if let Some(oldest) = self.order.first().cloned() {
                self.pending.remove(&oldest);
                self.order.remove(0);
                dropped_agent = Some(oldest);
            }
        }

        if !self.pending.contains_key(agent_id) {
            self.pending
                .insert(agent_id.to_string(), PendingAgent::default());
            self.order.push(agent_id.to_string());
        }
        let pending = self.pending.get_mut(agent_id).expect("pending inserted");
        pending.evidence.push(MemoryEvidence {
            id: evidence_id
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| Uuid::new_v4().to_string()),
            occurred_at,
            user,
            assistant,
        });
        if pending.evidence.len() > MAX_PENDING_EVIDENCE_PER_AGENT {
            let excess = pending.evidence.len() - MAX_PENDING_EVIDENCE_PER_AGENT;
            pending.evidence.drain(0..excess);
        }
        if active {
            self.needs_another_pass = true;
        }
        dropped_agent
    }

    pub fn queue_temporal(&mut self, agent_id: &str, active: bool) -> bool {
        if !self.pending.contains_key(agent_id) {
            self.pending
                .insert(agent_id.to_string(), PendingAgent::default());
            self.order.push(agent_id.to_string());
        }
        let pending = self.pending.get_mut(agent_id).expect("pending inserted");
        if pending.temporal {
            return false;
        }
        pending.temporal = true;
        if active {
            self.needs_another_pass = true;
        }
        true
    }

    pub fn agent_ids(&self) -> Vec<String> {
        self.order
            .iter()
            .filter(|id| self.pending.contains_key(*id))
            .cloned()
            .collect()
    }

    pub fn snapshot(&self, agent_id: &str) -> Option<(Vec<MemoryEvidence>, bool)> {
        self.pending
            .get(agent_id)
            .map(|pending| (pending.evidence.clone(), pending.temporal))
    }

    pub fn finish(
        &mut self,
        agent_id: &str,
        consumed_evidence_ids: &HashSet<String>,
        temporal: bool,
    ) {
        let Some(pending) = self.pending.get_mut(agent_id) else {
            return;
        };
        pending
            .evidence
            .retain(|item| !consumed_evidence_ids.contains(&item.id));
        if temporal {
            pending.temporal = false;
        }
        if pending.evidence.is_empty() && !pending.temporal {
            self.pending.remove(agent_id);
            self.order.retain(|id| id != agent_id);
        }
    }

    pub fn mark_stale(&mut self) {
        self.needs_another_pass = true;
    }

    pub fn take_needs_another_pass(&mut self) -> bool {
        std::mem::take(&mut self.needs_another_pass)
    }

    pub fn len(&self) -> usize {
        self.pending.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

pub fn bounded_evidence_text(raw: &str) -> String {
    let normalized = raw.trim();
    if normalized.chars().count() <= MAX_EVIDENCE_SIDE_CHARS {
        return normalized.to_string();
    }
    let half = MAX_EVIDENCE_SIDE_CHARS / 2;
    let prefix = normalized.chars().take(half).collect::<String>();
    let suffix = normalized
        .chars()
        .rev()
        .take(half)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<String>();
    format!("{prefix}\n[...middle omitted...]\n{suffix}")
}

pub fn parse_json_object(text: &str) -> Option<Value> {
    let value = text.trim();
    let start = value.find('{')?;
    let end = value.rfind('}')?;
    if end < start {
        return None;
    }
    let parsed: Value = serde_json::from_str(&value[start..=end]).ok()?;
    parsed.as_object()?;
    Some(parsed)
}

pub fn parse_memory_synthesis_changes(raw: &Value) -> Option<Vec<MemoryChange>> {
    let changes = raw.get("changes")?.as_array()?;
    if changes.len() > MAX_SYNTHESIS_CHANGES {
        return None;
    }
    let mut parsed = Vec::with_capacity(changes.len());
    for value in changes {
        let object = value.as_object()?;
        let action = object.get("action")?.as_str()?;
        let source_ids = object.get("sourceEvidenceIds")?.as_array()?;
        if source_ids.is_empty() || source_ids.len() > MAX_SOURCE_EVIDENCE_IDS {
            return None;
        }
        let mut source_evidence_ids = Vec::with_capacity(source_ids.len());
        for id in source_ids {
            let id = id.as_str()?;
            if id.is_empty() || id.len() > 64 {
                return None;
            }
            source_evidence_ids.push(id.to_string());
        }
        match action {
            "remove" => {
                let id = object.get("id")?.as_str()?;
                if id.is_empty() || id.len() > 64 {
                    return None;
                }
                parsed.push(MemoryChange::Remove {
                    id: id.to_string(),
                    source_evidence_ids,
                });
            }
            "create" | "update" => {
                let content = object.get("content")?.as_str()?;
                if content.is_empty() || content.chars().count() > 500 {
                    return None;
                }
                let kind = match object.get("kind")?.as_str()? {
                    "profile" => MemoryKind::Profile,
                    "log" => MemoryKind::Log,
                    _ => return None,
                };
                let normalized = normalize_memory_content(content);
                if normalized.is_empty() {
                    return None;
                }
                if action == "create" {
                    parsed.push(MemoryChange::Create {
                        content: normalized,
                        kind,
                        source_evidence_ids,
                    });
                } else {
                    let id = object.get("id")?.as_str()?;
                    if id.is_empty() || id.len() > 64 {
                        return None;
                    }
                    parsed.push(MemoryChange::Update {
                        id: id.to_string(),
                        content: normalized,
                        kind,
                        source_evidence_ids,
                    });
                }
            }
            _ => return None,
        }
    }
    Some(parsed)
}

pub fn uses_known_evidence(
    evidence_ids: &HashSet<String>,
    changes: &[MemoryChange],
    allow_clock: bool,
) -> bool {
    changes.iter().all(|change| {
        let ids = change.source_evidence_ids();
        !ids.is_empty()
            && ids
                .iter()
                .all(|id| evidence_ids.contains(id) || (allow_clock && id == "clock"))
            && !matches!(change, MemoryChange::Create { .. } if ids.iter().all(|id| id == "clock"))
    })
}

pub fn protects_explicit_memories(snapshot: &SynthesisSnapshot, changes: &[MemoryChange]) -> bool {
    let explicit_ids = snapshot
        .memories
        .iter()
        .filter(|memory| memory.origin == MemoryOrigin::Explicit)
        .map(|memory| memory.id.as_str())
        .collect::<HashSet<_>>();
    changes.iter().all(|change| match change {
        MemoryChange::Update { id, .. } | MemoryChange::Remove { id, .. } => {
            !explicit_ids.contains(id.as_str())
        }
        MemoryChange::Create { .. } => true,
    })
}

pub fn apply_verified_changes(
    snapshot: &SynthesisSnapshot,
    changes: &[MemoryChange],
    apply: impl FnOnce(&SynthesisSnapshot, &[SynthesisChange]) -> SynthesisApplyResult,
) -> SynthesisOutcome {
    if !protects_explicit_memories(snapshot, changes) {
        return SynthesisOutcome::Rejected;
    }
    let store_changes = changes
        .iter()
        .map(MemoryChange::to_store_change)
        .collect::<Vec<_>>();
    match apply(snapshot, &store_changes) {
        SynthesisApplyResult::Committed => SynthesisOutcome::Committed,
        SynthesisApplyResult::Stale => SynthesisOutcome::Stale,
        SynthesisApplyResult::Invalid => SynthesisOutcome::InvalidOutput,
    }
}

use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{TimeZone, Utc};

use super::memory_service::FileMemoryStore;

pub trait SynthesisTarget: Send + Sync {
    fn prepare_synthesis(&self) -> SynthesisSnapshot;
    fn apply_synthesis(
        &self,
        snapshot: &SynthesisSnapshot,
        changes: &[SynthesisChange],
        now_ms: i64,
    ) -> SynthesisApplyResult;
    fn has_memories(&self) -> bool;
    fn is_temporal_review_due(&self, now_ms: i64) -> bool;
    fn mark_temporal_review(&self, now_ms: i64);
}

impl SynthesisTarget for FileMemoryStore {
    fn prepare_synthesis(&self) -> SynthesisSnapshot {
        FileMemoryStore::prepare_synthesis(self)
    }

    fn apply_synthesis(
        &self,
        snapshot: &SynthesisSnapshot,
        changes: &[SynthesisChange],
        now_ms: i64,
    ) -> SynthesisApplyResult {
        FileMemoryStore::apply_synthesis(self, snapshot, changes, now_ms)
            .unwrap_or(SynthesisApplyResult::Invalid)
    }

    fn has_memories(&self) -> bool {
        FileMemoryStore::has_memories(self)
    }

    fn is_temporal_review_due(&self, now_ms: i64) -> bool {
        FileMemoryStore::is_temporal_review_due(self, now_ms)
    }

    fn mark_temporal_review(&self, now_ms: i64) {
        let _ = FileMemoryStore::mark_temporal_review(self, now_ms);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynthesisProposalRequest {
    pub today: String,
    pub current_memories: Vec<super::memory_service::SynthesisMemory>,
    pub new_evidence: Vec<MemoryEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynthesisVerificationRequest {
    pub today: String,
    pub current_memories: Vec<super::memory_service::SynthesisMemory>,
    pub evidence: Vec<MemoryEvidence>,
    pub proposed_changes: Vec<MemoryChange>,
}

#[derive(Debug, Clone, Default)]
pub struct SynthesisCancelSignal {
    cancelled: Arc<std::sync::atomic::AtomicBool>,
}

impl SynthesisCancelSignal {
    pub fn cancel(&self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::SeqCst)
    }
}

type TargetRef = Arc<dyn SynthesisTarget>;
type ListTargets = Arc<dyn Fn() -> Vec<(String, TargetRef)> + Send + Sync>;
type GetTarget = Arc<dyn Fn(&str) -> Option<TargetRef> + Send + Sync>;
type Propose = Arc<
    dyn Fn(SynthesisProposalRequest, SynthesisCancelSignal) -> Result<Value, String> + Send + Sync,
>;
type Verify = Arc<
    dyn Fn(SynthesisVerificationRequest, SynthesisCancelSignal) -> Result<bool, String>
        + Send
        + Sync,
>;

pub struct MemorySynthesisOptions {
    pub list_targets: ListTargets,
    pub get_target: GetTarget,
    pub propose: Propose,
    pub verify: Option<Verify>,
    pub debounce: Duration,
    pub deadline: Duration,
    pub poll_interval: Duration,
    pub retry_attempts: usize,
    pub retry_initial: Duration,
    pub retry_max: Duration,
    pub now: Arc<dyn Fn() -> i64 + Send + Sync>,
}

impl MemorySynthesisOptions {
    pub fn new(list_targets: ListTargets, get_target: GetTarget, propose: Propose) -> Self {
        Self {
            list_targets,
            get_target,
            propose,
            verify: None,
            debounce: Duration::from_millis(MEMORY_SYNTHESIS_DEBOUNCE_MS),
            deadline: Duration::from_millis(MEMORY_SYNTHESIS_DEADLINE_MS),
            poll_interval: Duration::from_millis(MEMORY_SYNTHESIS_POLL_INTERVAL_MS),
            retry_attempts: MEMORY_SYNTHESIS_RETRY_ATTEMPTS,
            retry_initial: Duration::from_millis(MEMORY_SYNTHESIS_RETRY_INITIAL_MS),
            retry_max: Duration::from_millis(MEMORY_SYNTHESIS_RETRY_MAX_MS),
            now: Arc::new(|| Utc::now().timestamp_millis()),
        }
    }
}

#[derive(Default)]
struct MemorySynthesisState {
    queue: MemorySynthesisQueue,
    started: bool,
    disposed: bool,
    active: bool,
}

pub struct MemorySynthesisService {
    options: MemorySynthesisOptions,
    state: Mutex<MemorySynthesisState>,
    cancel: SynthesisCancelSignal,
}

impl MemorySynthesisService {
    pub fn new(options: MemorySynthesisOptions) -> Self {
        Self {
            options,
            state: Mutex::new(MemorySynthesisState::default()),
            cancel: SynthesisCancelSignal::default(),
        }
    }

    pub fn start(&self) {
        let now = (self.options.now)();
        let targets = (self.options.list_targets)();
        let mut state = self.state.lock().expect("memory synthesis state poisoned");
        if state.started || state.disposed {
            return;
        }
        state.started = true;
        let mut queued = 0usize;
        for (agent_id, target) in targets {
            if queued >= MAX_TEMPORAL_TARGETS_PER_SWEEP {
                break;
            }
            if target.has_memories() && target.is_temporal_review_due(now) {
                if state.queue.queue_temporal(&agent_id, false) {
                    queued += 1;
                }
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        let state = self.state.lock().expect("memory synthesis state poisoned");
        state.started && !state.disposed
    }

    pub fn record_turn(
        &self,
        agent_id: &str,
        evidence_id: Option<String>,
        user: &str,
        assistant: &str,
        occurred_at: i64,
    ) {
        let mut state = self.state.lock().expect("memory synthesis state poisoned");
        if !state.started || state.disposed {
            return;
        }
        let active = state.active;
        state.queue.record_turn(
            agent_id,
            evidence_id.as_deref(),
            occurred_at,
            user,
            assistant,
            active,
        );
    }

    pub fn run_now(&self) -> Vec<SynthesisOutcome> {
        {
            let mut state = self.state.lock().expect("memory synthesis state poisoned");
            if !state.started || state.disposed || state.active {
                return Vec::new();
            }
            state.active = true;
        }

        let agent_ids = {
            self.state
                .lock()
                .expect("memory synthesis state poisoned")
                .queue
                .agent_ids()
        };
        let mut outcomes = Vec::with_capacity(agent_ids.len());
        for agent_id in agent_ids {
            if self.cancel.is_cancelled() {
                break;
            }
            outcomes.push(self.run_agent(&agent_id));
        }

        let mut state = self.state.lock().expect("memory synthesis state poisoned");
        state.active = false;
        outcomes
    }

    pub fn dispose(&self) {
        self.cancel.cancel();
        let mut state = self.state.lock().expect("memory synthesis state poisoned");
        state.disposed = true;
        state.queue = MemorySynthesisQueue::default();
    }

    fn run_agent(&self, agent_id: &str) -> SynthesisOutcome {
        let (evidence, temporal) = {
            let state = self.state.lock().expect("memory synthesis state poisoned");
            let Some(snapshot) = state.queue.snapshot(agent_id) else {
                return SynthesisOutcome::NoWork;
            };
            snapshot
        };
        let Some(target) = (self.options.get_target)(agent_id) else {
            self.finish(agent_id, &evidence, temporal);
            return SynthesisOutcome::NoWork;
        };
        let snapshot = target.prepare_synthesis();
        let now = (self.options.now)();

        if snapshot.memories.is_empty() && evidence.is_empty() {
            if temporal {
                target.mark_temporal_review(now);
            }
            self.finish(agent_id, &evidence, temporal);
            return SynthesisOutcome::NoWork;
        }

        let today = Utc
            .timestamp_millis_opt(now)
            .single()
            .unwrap_or_else(Utc::now)
            .format("%Y-%m-%d")
            .to_string();
        let request = SynthesisProposalRequest {
            today: today.clone(),
            current_memories: snapshot.memories.clone(),
            new_evidence: evidence.clone(),
        };

        let proposal_raw = match self.propose_with_retry(request) {
            Ok(raw) => raw,
            Err(()) => {
                if temporal {
                    target.mark_temporal_review(now);
                }
                self.finish(agent_id, &evidence, temporal);
                return SynthesisOutcome::Failed;
            }
        };
        let Some(changes) = parse_memory_synthesis_changes(&proposal_raw) else {
            if temporal {
                target.mark_temporal_review(now);
            }
            self.finish(agent_id, &evidence, temporal);
            return SynthesisOutcome::InvalidOutput;
        };
        let known = evidence
            .iter()
            .map(|item| item.id.clone())
            .collect::<HashSet<_>>();
        if !uses_known_evidence(&known, &changes, temporal) {
            if temporal {
                target.mark_temporal_review(now);
            }
            self.finish(agent_id, &evidence, temporal);
            return SynthesisOutcome::InvalidOutput;
        }

        if changes.is_empty() {
            if temporal {
                target.mark_temporal_review(now);
            }
            self.finish(agent_id, &evidence, temporal);
            return SynthesisOutcome::NoWork;
        }

        if !protects_explicit_memories(&snapshot, &changes) {
            if temporal {
                target.mark_temporal_review(now);
            }
            self.finish(agent_id, &evidence, temporal);
            return SynthesisOutcome::Rejected;
        }

        if let Some(verify) = &self.options.verify {
            let verification = SynthesisVerificationRequest {
                today,
                current_memories: snapshot.memories.clone(),
                evidence: evidence.clone(),
                proposed_changes: changes.clone(),
            };
            match verify(verification, self.cancel.clone()) {
                Ok(true) => {}
                Ok(false) => {
                    if temporal {
                        target.mark_temporal_review(now);
                    }
                    self.finish(agent_id, &evidence, temporal);
                    return SynthesisOutcome::Rejected;
                }
                Err(_) => {
                    if temporal {
                        target.mark_temporal_review(now);
                    }
                    self.finish(agent_id, &evidence, temporal);
                    return SynthesisOutcome::Failed;
                }
            }
        }

        let store_changes = changes
            .iter()
            .map(MemoryChange::to_store_change)
            .collect::<Vec<_>>();
        let outcome = match target.apply_synthesis(&snapshot, &store_changes, now) {
            SynthesisApplyResult::Committed => SynthesisOutcome::Committed,
            SynthesisApplyResult::Stale => SynthesisOutcome::Stale,
            SynthesisApplyResult::Invalid => SynthesisOutcome::InvalidOutput,
        };
        if outcome == SynthesisOutcome::Stale {
            self.state
                .lock()
                .expect("memory synthesis state poisoned")
                .queue
                .mark_stale();
        } else {
            if temporal && outcome != SynthesisOutcome::Committed {
                target.mark_temporal_review(now);
            }
            self.finish(agent_id, &evidence, temporal);
        }
        outcome
    }

    fn propose_with_retry(&self, request: SynthesisProposalRequest) -> Result<Value, ()> {
        let attempts = self.options.retry_attempts.max(1);
        let mut delay = self.options.retry_initial;
        for attempt in 0..attempts {
            if self.cancel.is_cancelled() {
                return Err(());
            }
            match (self.options.propose)(request.clone(), self.cancel.clone()) {
                Ok(value) => return Ok(value),
                Err(_) if attempt + 1 < attempts => {
                    if !delay.is_zero() {
                        std::thread::sleep(delay.min(self.options.retry_max));
                    }
                    delay = delay
                        .checked_mul(2)
                        .unwrap_or(self.options.retry_max)
                        .min(self.options.retry_max);
                }
                Err(_) => return Err(()),
            }
        }
        Err(())
    }

    fn finish(&self, agent_id: &str, evidence: &[MemoryEvidence], temporal: bool) {
        let consumed = evidence
            .iter()
            .map(|item| item.id.clone())
            .collect::<HashSet<_>>();
        self.state
            .lock()
            .expect("memory synthesis state poisoned")
            .queue
            .finish(agent_id, &consumed, temporal);
    }
}


impl MemorySynthesisBridge for MemorySynthesisService {
    fn is_enabled(&self) -> bool {
        MemorySynthesisService::is_enabled(self)
    }

    fn record_turn(
        &self,
        agent_id: &str,
        evidence_id: Option<String>,
        user: &str,
        assistant: &str,
        occurred_at: i64,
    ) {
        MemorySynthesisService::record_turn(
            self,
            agent_id,
            evidence_id,
            user,
            assistant,
            occurred_at,
        );
    }
}
