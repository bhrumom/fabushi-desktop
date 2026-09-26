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

#[derive(Debug, Default)]
pub struct ComputerUseCoordination {
    window_by_subagent: HashMap<String, u32>,
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

    pub fn allocate_window(&mut self, subagent_agent_id: &str) -> Option<u32> {
        if let Some(existing) = self.window_by_subagent.get(subagent_agent_id) {
            return Some(*existing);
        }
        if self.window_by_subagent.values().any(|window| *window == 1) {
            return None;
        }
        self.window_by_subagent
            .insert(subagent_agent_id.to_string(), 1);
        Some(1)
    }

    pub fn free_window(&mut self, subagent_agent_id: &str) {
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
