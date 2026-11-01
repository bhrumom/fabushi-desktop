use std::collections::BTreeSet;
use std::sync::Arc;

use serde_json::{Map, Value, json};

use crate::automations::automation_store::FileAutomationStore;
use crate::extensions::session::agent_db_serde::SpendGuardState;
use crate::extensions::session::production::ProductionSessionWorkers;

use super::sand_automation_spend_guard::{
    SPEND_GUARD_SNOOZE_MS, SpendGuardAnswer, SpendGuardDecision, SpendGuardEvaluation,
    SpendGuardWidget, build_spend_guard_nudge_widget, build_spend_guard_paused_widget,
    count_automation_runs_since, evaluate_automation_spend_guard, interpret_spend_guard_answer,
    render_spend_guard_answer_ack, render_spend_guard_nudge_reminder,
};
use super::transcript_entry_ids::{TranscriptEntryIdKind, next_entry_id};

#[derive(Debug, Clone, PartialEq)]
pub struct SpendGuardEvaluationSnapshot {
    pub decision: SpendGuardDecision,
    pub evaluation: SpendGuardEvaluation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpendGuardApplyResult {
    pub paused: bool,
    pub reminder: Option<String>,
    pub issued_card_entry_id: Option<String>,
    pub paused_automation_ids: Vec<String>,
}

#[derive(Clone)]
pub struct AutomationSpendGuardRuntime {
    sessions: Arc<ProductionSessionWorkers>,
}

impl AutomationSpendGuardRuntime {
    pub fn new(sessions: Arc<ProductionSessionWorkers>) -> Self {
        Self { sessions }
    }

    pub fn evaluate_with_store(
        &self,
        agent_id: &str,
        store: &FileAutomationStore,
        now_ms: f64,
    ) -> Result<SpendGuardEvaluationSnapshot, String> {
        let unread = self.sessions.get_agent_unread_state(agent_id)?;
        let state = self
            .sessions
            .get_agent_automation_spend_guard_state(agent_id)?;
        let evaluation = SpendGuardEvaluation {
            now_ms,
            last_viewed_at_ms: unread.last_viewed_at,
            unread_count: unread.unread_count.max(0.0).floor() as usize,
            fires_since_viewed_count: count_automation_runs_since(
                &store.list_definitions(),
                unread.last_viewed_at,
            ),
            nudged_at_ms: state.nudged_at_ms,
            snoozed_until_ms: state.snoozed_until_ms,
            opted_out: state.opted_out,
        };
        Ok(SpendGuardEvaluationSnapshot {
            decision: evaluate_automation_spend_guard(evaluation),
            evaluation,
        })
    }

    pub fn apply_with_store(
        &self,
        agent_id: &str,
        store: &FileAutomationStore,
        firing_automation_id: &str,
        now_ms: f64,
    ) -> Result<SpendGuardApplyResult, String> {
        let evaluated = self.evaluate_with_store(agent_id, store, now_ms)?;
        match evaluated.decision {
            SpendGuardDecision::UserActive
            | SpendGuardDecision::OptedOut
            | SpendGuardDecision::Snoozed
            | SpendGuardDecision::BelowThresholds => Ok(SpendGuardApplyResult {
                paused: firing_is_off(store, firing_automation_id),
                reminder: None,
                issued_card_entry_id: None,
                paused_automation_ids: Vec::new(),
            }),
            SpendGuardDecision::AwaitingAck => {
                let card = self.issue_card_if_none(
                    agent_id,
                    build_spend_guard_nudge_widget(),
                    now_ms,
                )?;
                Ok(SpendGuardApplyResult {
                    paused: firing_is_off(store, firing_automation_id),
                    reminder: None,
                    issued_card_entry_id: card,
                    paused_automation_ids: Vec::new(),
                })
            }
            SpendGuardDecision::Nudge => {
                let card = self.issue_guard_card(
                    agent_id,
                    &build_spend_guard_nudge_widget(),
                    now_ms,
                )?;
                self.sessions.set_agent_automation_spend_guard_state(
                    agent_id,
                    &SpendGuardState {
                        nudged_at_ms: Some(now_ms),
                        card_entry_ids: vec![card.clone()],
                        ..SpendGuardState::default()
                    },
                )?;
                Ok(SpendGuardApplyResult {
                    paused: firing_is_off(store, firing_automation_id),
                    reminder: Some(render_spend_guard_nudge_reminder(
                        evaluated.evaluation,
                        self.sessions.resolve_user_time_zone().as_deref(),
                    )),
                    issued_card_entry_id: Some(card),
                    paused_automation_ids: Vec::new(),
                })
            }
            SpendGuardDecision::Pause => {
                let current = self
                    .sessions
                    .get_agent_automation_spend_guard_state(agent_id)?;
                let recheck = self.evaluate_with_store(agent_id, store, now_ms)?;
                if recheck.decision != SpendGuardDecision::Pause {
                    return Ok(SpendGuardApplyResult {
                        paused: firing_is_off(store, firing_automation_id),
                        reminder: None,
                        issued_card_entry_id: None,
                        paused_automation_ids: Vec::new(),
                    });
                }
                let newly_paused = disable_every_enabled_routine(store)?;
                if newly_paused.is_empty() {
                    return Ok(SpendGuardApplyResult {
                        paused: firing_is_off(store, firing_automation_id),
                        reminder: None,
                        issued_card_entry_id: None,
                        paused_automation_ids: Vec::new(),
                    });
                }
                let mut paused = current
                    .paused_automation_ids
                    .iter()
                    .cloned()
                    .collect::<BTreeSet<_>>();
                paused.extend(newly_paused);
                let paused_automation_ids = paused.into_iter().collect::<Vec<_>>();
                let already_open = !current.paused_automation_ids.is_empty();
                let issued = if already_open {
                    None
                } else {
                    Some(self.issue_guard_card(
                        agent_id,
                        &build_spend_guard_paused_widget(),
                        now_ms,
                    )?)
                };
                let mut card_entry_ids = current.card_entry_ids;
                if let Some(card) = issued.as_ref() {
                    card_entry_ids.push(card.clone());
                }
                self.sessions.set_agent_automation_spend_guard_state(
                    agent_id,
                    &SpendGuardState {
                        nudged_at_ms: current.nudged_at_ms,
                        card_entry_ids,
                        paused_automation_ids: paused_automation_ids.clone(),
                        ..SpendGuardState::default()
                    },
                )?;
                Ok(SpendGuardApplyResult {
                    paused: true,
                    reminder: None,
                    issued_card_entry_id: issued,
                    paused_automation_ids,
                })
            }
        }
    }

    pub fn handle_widget_answer_with_store(
        &self,
        agent_id: &str,
        store: &FileAutomationStore,
        entry_id: &str,
        value: &str,
        now_ms: f64,
    ) -> Result<Option<String>, String> {
        let Some(answer) = interpret_spend_guard_answer(value) else {
            return Ok(None);
        };
        let current = self
            .sessions
            .get_agent_automation_spend_guard_state(agent_id)?;
        if !current.card_entry_ids.iter().any(|id| id == entry_id)
            || !self.is_host_issued_card(agent_id, entry_id, value)?
        {
            return Ok(None);
        }

        let mut next = SpendGuardState::default();
        match answer {
            SpendGuardAnswer::Keep | SpendGuardAnswer::Resume => {
                re_enable_guard_paused_routines(store, &current.paused_automation_ids)?;
                next.snoozed_until_ms = Some(now_ms + SPEND_GUARD_SNOOZE_MS);
            }
            SpendGuardAnswer::OptOut => {
                re_enable_guard_paused_routines(store, &current.paused_automation_ids)?;
                next.opted_out = true;
            }
            SpendGuardAnswer::Pause => {
                let mut paused = current
                    .paused_automation_ids
                    .iter()
                    .cloned()
                    .collect::<BTreeSet<_>>();
                paused.extend(disable_every_enabled_routine(store)?);
                next.paused_automation_ids = paused.into_iter().collect();
                if !next.paused_automation_ids.is_empty() {
                    next.card_entry_ids = current.card_entry_ids;
                }
            }
            SpendGuardAnswer::StayPaused => {}
        }
        self.sessions
            .set_agent_automation_spend_guard_state(agent_id, &next)?;
        Ok(Some(render_spend_guard_answer_ack(answer)))
    }

    fn issue_card_if_none(
        &self,
        agent_id: &str,
        widget: SpendGuardWidget,
        now_ms: f64,
    ) -> Result<Option<String>, String> {
        let mut current = self
            .sessions
            .get_agent_automation_spend_guard_state(agent_id)?;
        if !current.card_entry_ids.is_empty() {
            return Ok(None);
        }
        let card = self.issue_guard_card(agent_id, &widget, now_ms)?;
        current.card_entry_ids = vec![card.clone()];
        self.sessions
            .set_agent_automation_spend_guard_state(agent_id, &current)?;
        Ok(Some(card))
    }

    fn issue_guard_card(
        &self,
        agent_id: &str,
        widget: &SpendGuardWidget,
        now_ms: f64,
    ) -> Result<String, String> {
        let entries = self.sessions.read_agent_transcript_entries(agent_id)?;
        let id = next_entry_id(&entries, TranscriptEntryIdKind::SendMessage);
        let entry = json!({
            "kind": "send-message",
            "id": id,
            "message": {
                "type": "widget",
                "widget": widget_value(widget),
            },
            "timestampMs": now_ms,
        });
        self.sessions
            .append_agent_transcript_entries(agent_id, &[entry])?;
        let _ = self.sessions.mark_agent_activity(agent_id, now_ms)?;
        Ok(id)
    }

    fn is_host_issued_card(
        &self,
        agent_id: &str,
        entry_id: &str,
        value: &str,
    ) -> Result<bool, String> {
        let entries = self.sessions.read_agent_transcript_entries(agent_id)?;
        let Some(widget) = entries
            .iter()
            .find(|entry| entry.get("id").and_then(Value::as_str) == Some(entry_id))
            .filter(|entry| entry.get("kind").and_then(Value::as_str) == Some("send-message"))
            .and_then(|entry| entry.get("message"))
            .filter(|message| message.get("type").and_then(Value::as_str) == Some("widget"))
            .and_then(|message| message.get("widget"))
        else {
            return Ok(false);
        };
        let prompt = widget.get("prompt").and_then(Value::as_str).unwrap_or_default();
        let known_prompt = prompt == build_spend_guard_nudge_widget().prompt
            || prompt == build_spend_guard_paused_widget().prompt;
        let has_value = widget
            .get("options")
            .and_then(Value::as_array)
            .is_some_and(|options| {
                options.iter().any(|option| {
                    option.get("value").and_then(Value::as_str) == Some(value)
                })
            });
        Ok(known_prompt && has_value)
    }
}

pub fn disable_every_enabled_routine(store: &FileAutomationStore) -> Result<Vec<String>, String> {
    let mut disabled = Vec::new();
    for automation in store.list_definitions() {
        if automation.is_enabled {
            store
                .set_enabled(&automation.id, false)
                .map_err(|error| error.to_string())?;
            disabled.push(automation.id);
        }
    }
    Ok(disabled)
}

pub fn re_enable_guard_paused_routines(
    store: &FileAutomationStore,
    ids: &[String],
) -> Result<(), String> {
    let resume = ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    for automation in store.list_definitions() {
        if resume.contains(automation.id.as_str()) && !automation.is_enabled {
            store
                .set_enabled(&automation.id, true)
                .map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn firing_is_off(store: &FileAutomationStore, automation_id: &str) -> bool {
    store
        .get(automation_id)
        .is_none_or(|automation| !automation.is_enabled)
}

fn widget_value(widget: &SpendGuardWidget) -> Value {
    let options = widget
        .options
        .iter()
        .map(|option| {
            let mut value = Map::new();
            value.insert("label".into(), Value::String(option.label.clone()));
            value.insert("value".into(), Value::String(option.value.clone()));
            if let Some(style) = option.style.as_ref() {
                value.insert("style".into(), Value::String(style.clone()));
            }
            Value::Object(value)
        })
        .collect::<Vec<_>>();
    json!({
        "prompt": widget.prompt,
        "options": options,
    })
}
