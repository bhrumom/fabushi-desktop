use std::collections::{BTreeSet, HashMap};

use super::sand_action_audit::computer_use_audit_kind;
use super::{TurnUsage, merge_turn_usage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComputerUsePrewarmStage {
    Box,
    Browser,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputerUsePrewarmDiagnostic {
    pub kind: &'static str,
    pub stage: ComputerUsePrewarmStage,
    pub error_class: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparationState {
    Pending,
    Ready,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputerUseUsageSnapshot {
    pub model_id: Option<String>,
    pub turn_ended_count: u64,
    pub usage: Option<TurnUsage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputerControlLease {
    owner_agent_id: String,
    window_index: u32,
    generation: u64,
}

impl ComputerControlLease {
    pub fn owner_agent_id(&self) -> &str { &self.owner_agent_id }
    pub fn window_index(&self) -> u32 { self.window_index }
    pub fn generation(&self) -> u64 { self.generation }
}

#[derive(Debug, Default)]
pub struct ComputerUseCoordination {
    window_by_subagent: HashMap<String, u32>,
    control_lease: Option<ComputerControlLease>,
    next_lease_generation: u64,
    preparation_by_subagent: HashMap<String, PreparationState>,
    audit_action_counts: HashMap<String, u64>,
    model_ids: BTreeSet<String>,
    turn_ended_count: u64,
    usage: Option<TurnUsage>,
    navigation_probe_created: bool,
    audit_enabled: bool,
    diagnostics: Vec<ComputerUsePrewarmDiagnostic>,
}

impl ComputerUseCoordination {
    pub fn new(audit_enabled: bool) -> Self {
        Self {
            audit_enabled,
            ..Self::default()
        }
    }

    pub fn acquire_control_lease(&mut self, subagent_agent_id: &str) -> Option<ComputerControlLease> {
        if let Some(existing) = self.control_lease.as_ref() {
            return (existing.owner_agent_id == subagent_agent_id).then(|| existing.clone());
        }
        if self.window_by_subagent.values().any(|window| *window == 1) {
            return None;
        }
        self.next_lease_generation = self.next_lease_generation.saturating_add(1).max(1);
        let lease = ComputerControlLease {
            owner_agent_id: subagent_agent_id.to_string(),
            window_index: 1,
            generation: self.next_lease_generation,
        };
        self.window_by_subagent.insert(subagent_agent_id.to_string(), lease.window_index);
        self.control_lease = Some(lease.clone());
        Some(lease)
    }

    pub fn owns_control_lease(&self, lease: &ComputerControlLease) -> bool {
        self.control_lease.as_ref() == Some(lease)
            && self.window_by_subagent.get(lease.owner_agent_id()).is_some_and(|window| *window == lease.window_index())
    }

    pub fn release_control_lease(&mut self, lease: &ComputerControlLease) -> bool {
        if !self.owns_control_lease(lease) { return false; }
        self.window_by_subagent.remove(lease.owner_agent_id());
        self.preparation_by_subagent.remove(lease.owner_agent_id());
        self.control_lease = None;
        true
    }

    pub fn allocate_window(&mut self, subagent_agent_id: &str) -> Option<u32> {
        self.acquire_control_lease(subagent_agent_id).map(|lease| lease.window_index())
    }

    pub fn free_window(&mut self, subagent_agent_id: &str) {
        if self.control_lease.as_ref().is_some_and(|lease| lease.owner_agent_id() == subagent_agent_id) {
            self.control_lease = None;
        }
        self.window_by_subagent.remove(subagent_agent_id);
        self.preparation_by_subagent.remove(subagent_agent_id);
    }

    pub fn begin_preparation(&mut self, subagent_agent_id: &str) {
        self.preparation_by_subagent
            .insert(subagent_agent_id.to_string(), PreparationState::Pending);
    }

    pub fn mark_preparation_ready(&mut self, subagent_agent_id: &str) {
        self.preparation_by_subagent
            .insert(subagent_agent_id.to_string(), PreparationState::Ready);
    }

    pub fn mark_preparation_failed(
        &mut self,
        subagent_agent_id: &str,
        stage: ComputerUsePrewarmStage,
        error_class: impl Into<String>,
    ) {
        self.preparation_by_subagent
            .insert(subagent_agent_id.to_string(), PreparationState::Failed);
        self.diagnostics.push(ComputerUsePrewarmDiagnostic {
            kind: "computer_use_prewarm_skipped",
            stage,
            error_class: error_class.into(),
        });
    }

    pub fn preparation_for(&self, subagent_agent_id: &str) -> Option<PreparationState> {
        self.preparation_by_subagent
            .get(subagent_agent_id)
            .copied()
    }

    pub fn diagnostics(&self) -> &[ComputerUsePrewarmDiagnostic] {
        &self.diagnostics
    }

    pub fn record_audit_intent(&mut self, action_case: &str) {
        if !self.audit_enabled {
            return;
        }
        let Some(kind) = computer_use_audit_kind(action_case) else {
            return;
        };
        *self.audit_action_counts.entry(kind.to_string()).or_default() += 1;
    }

    pub fn audit_action_counts(&self) -> &HashMap<String, u64> {
        &self.audit_action_counts
    }

    pub fn record_turn_ended(&mut self, turn_usage: Option<TurnUsage>) {
        self.turn_ended_count = self.turn_ended_count.saturating_add(1);
        self.usage = merge_turn_usage(self.usage.take(), turn_usage);
    }

    pub fn record_model_id(&mut self, model_id: &str) {
        self.model_ids.insert(model_id.to_string());
    }

    pub fn usage_snapshot(&self) -> ComputerUseUsageSnapshot {
        let model_id = match self.model_ids.len() {
            0 => None,
            1 => self.model_ids.iter().next().cloned(),
            _ => Some("mixed".to_string()),
        };
        ComputerUseUsageSnapshot {
            model_id,
            turn_ended_count: self.turn_ended_count,
            usage: self.usage.clone(),
        }
    }

    pub fn get_or_create_navigation_probe(&mut self) -> bool {
        if !self.audit_enabled {
            return false;
        }
        self.navigation_probe_created = true;
        true
    }

    pub fn has_navigation_probe(&self) -> bool {
        self.navigation_probe_created
    }
}
