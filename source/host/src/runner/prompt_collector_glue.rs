use std::collections::HashMap;

use serde_json::Value;

use crate::automations::automation_status_reminder::render_automation_cleared_status_reminder;

use crate::extensions::inference::provider_session::ProviderMessage;

use super::sand_agent_profile_prompt::{
    AgentProfileIdentity, agent_profile_identities_equal, render_agent_profile_update,
};
use super::sand_prompt_markers::{
    SAND_HIDDEN_PROMPT_MARKER, SAND_TRUSTED_AUTOMATION_PROMPT_MARKER,
};
use super::system_prompt::{
    ReplyContext, USER_MESSAGE_REPLY_REMINDER, append_user_reply_reminder,
    build_attached_files_note, build_reply_context_note, build_user_message_address_note,
};
use super::system_prompt_assembly::{
    ComputerPromptState, RemoteBoxPromptState, append_agent_profile_system_prompt,
    append_computer_system_prompt, append_mcp_system_prompt_sections,
    append_remote_box_system_prompt,
};

/// Provider-facing prompt projection for the frozen prompt-collector boundary.
///
/// This intentionally does not mutate the durable Session transcript. The
/// generated Grok path builds a ConversationAction from durable user state;
/// until the full generated Rust action graph is present, the shipping routed
/// provider consumes this equivalent text projection while turn ownership and
/// checkpoint identity keep using the original durable messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderPromptProjection {
    pub messages: Vec<ProviderMessage>,
    pub projected_user_text: Option<String>,
    pub prepended_unanswered_questions: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct PromptCollectorDynamicUserContext<'a> {
    pub automation_status_reminder: Option<&'a str>,
    pub profile_update: Option<&'a str>,
    pub is_silence_allowed: bool,
}

/// Runner-owned cross-turn state for frozen automation status reminder projection.
///
/// The reference prompt collector suppresses an unchanged status snapshot until
/// compaction advances, and emits one authoritative cleared snapshot when the
/// previous reminder disappears. Host composition may retain this state per
/// Agent, but the transition rules remain owned by the prompt collector.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PromptCollectorAutomationReminderState {
    last_status_reminder: Option<String>,
    last_compaction_epoch: Option<u64>,
}

impl PromptCollectorAutomationReminderState {
    pub fn reminder_for_turn(
        &self,
        rendered: Option<&str>,
        compaction_epoch: u64,
    ) -> Option<String> {
        let compaction_advanced = self
            .last_compaction_epoch
            .is_some_and(|last| compaction_epoch > last);
        match rendered {
            Some(rendered)
                if self.last_status_reminder.as_deref() == Some(rendered)
                    && !compaction_advanced =>
            {
                None
            }
            Some(rendered) => Some(rendered.to_string()),
            None if self.last_status_reminder.is_none() => None,
            None => {
                let clearing = render_automation_cleared_status_reminder();
                if self.last_status_reminder.as_deref() == Some(clearing.as_str())
                    && !compaction_advanced
                {
                    None
                } else {
                    Some(clearing)
                }
            }
        }
    }

    pub fn note_reminder(&mut self, reminder: Option<&str>, compaction_epoch: u64) {
        let Some(reminder) = reminder else {
            return;
        };
        self.last_status_reminder = Some(reminder.to_string());
        self.last_compaction_epoch = Some(compaction_epoch);
    }

    pub fn reset(&mut self) {
        self.last_status_reminder = None;
        self.last_compaction_epoch = None;
    }
}

pub fn append_profile_system_section_for_turn(
    messages: &mut Vec<ProviderMessage>,
    profile_section: &str,
) {
    append_agent_profile_system_prompt(messages, profile_section);
}

pub fn resolve_profile_update_for_turn(
    identity: &AgentProfileIdentity,
    announced_identity: &AgentProfileIdentity,
) -> Option<String> {
    (!agent_profile_identities_equal(identity, announced_identity))
        .then(|| render_agent_profile_update(identity))
}

pub fn apply_dynamic_user_context_for_turn(
    messages: &mut [ProviderMessage],
    context: PromptCollectorDynamicUserContext<'_>,
) -> bool {
    let automation = context
        .automation_status_reminder
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let profile = context
        .profile_update
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if automation.is_none() && profile.is_none() {
        return false;
    }

    let Some(user) = messages.iter_mut().rfind(|message| message.role == "user") else {
        return false;
    };

    let original = user.content.as_str();
    let mut hidden_prefix = String::new();
    let mut body = original;
    if let Some(rest) = body.strip_prefix(SAND_HIDDEN_PROMPT_MARKER) {
        hidden_prefix.push_str(SAND_HIDDEN_PROMPT_MARKER);
        body = rest;
        if let Some(rest) = body.strip_prefix(SAND_TRUSTED_AUTOMATION_PROMPT_MARKER) {
            hidden_prefix.push_str(SAND_TRUSTED_AUTOMATION_PROMPT_MARKER);
            body = rest;
        }
    }

    let reply_suffix = format!("\n\n{USER_MESSAGE_REPLY_REMINDER}");
    let mut had_reply_reminder = body.ends_with(&reply_suffix);
    if had_reply_reminder {
        body = &body[..body.len() - reply_suffix.len()];
    } else if body == USER_MESSAGE_REPLY_REMINDER {
        had_reply_reminder = true;
        body = "";
    }

    let mut rendered = body.to_string();
    for addition in [automation, profile].into_iter().flatten() {
        rendered = if rendered.is_empty() {
            addition.to_string()
        } else if context.is_silence_allowed {
            format!("{addition}\n\n{rendered}")
        } else {
            format!("{rendered}\n\n{addition}")
        };
    }
    if had_reply_reminder {
        rendered = if rendered.is_empty() {
            USER_MESSAGE_REPLY_REMINDER.to_string()
        } else {
            format!("{rendered}\n\n{USER_MESSAGE_REPLY_REMINDER}")
        };
    }

    user.content = format!("{hidden_prefix}{rendered}");
    true
}

pub fn append_mcp_runtime_sections_for_turn(
    messages: &mut Vec<ProviderMessage>,
    installed: &[Value],
    discovery_unavailable: bool,
    is_subagent_runner: bool,
) {
    append_mcp_system_prompt_sections(
        messages,
        installed,
        discovery_unavailable,
        !is_subagent_runner,
    );
}

pub fn append_remote_runtime_sections_for_turn(
    messages: &mut Vec<ProviderMessage>,
    remote_box: &RemoteBoxPromptState,
    computer: &ComputerPromptState,
) {
    append_remote_box_system_prompt(messages, remote_box);
    append_computer_system_prompt(messages, computer);
}

pub fn project_provider_messages_for_turn(
    args: &Value,
    messages: &[ProviderMessage],
) -> ProviderPromptProjection {
    let Some(user_index) = messages.iter().rposition(|message| message.role == "user") else {
        return ProviderPromptProjection {
            messages: messages.to_vec(),
            projected_user_text: None,
            prepended_unanswered_questions: false,
        };
    };

    let mut projected = messages.to_vec();
    let raw_user_text = projected[user_index].content.clone();
    let message_id = optional_non_empty(args, "messageId");
    let reply_context = parse_reply_context(args.get("replyContext"));
    let attachment_paths = string_array(args, "attachmentPaths")
        .or_else(|| string_array(args, "attachedFilePaths"))
        .unwrap_or_default();
    let attachment_sizes = number_map(args.get("attachedFileSizes"));
    let box_paths = string_map(args.get("boxPathByHostPath"));

    let address = build_user_message_address_note(message_id);
    let reply = build_reply_context_note(reply_context.as_ref());
    let mut text = [address, reply]
        .into_iter()
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    if !raw_user_text.is_empty() {
        text = if text.is_empty() {
            raw_user_text
        } else {
            format!("{text}\n{raw_user_text}")
        };
    }

    let attachments = build_attached_files_note(
        &attachment_paths,
        &box_paths,
        &attachment_sizes,
    );
    if !attachments.is_empty() {
        text = if text.is_empty() {
            attachments
        } else {
            format!("{text}\n\n{attachments}")
        };
    }

    let hidden = args.get("hidden").and_then(Value::as_bool) == Some(true);
    if args
        .get("appendReplyReminder")
        .and_then(Value::as_bool)
        == Some(true)
        && !hidden
    {
        text = append_user_reply_reminder(&text);
    }
    if hidden {
        let trusted_automation = args
            .get("automationWake")
            .and_then(Value::as_object)
            .is_some_and(|wake| {
                wake.get("containsUntrustedEventText")
                    .and_then(Value::as_bool)
                    != Some(true)
            });
        text = format!(
            "{SAND_HIDDEN_PROMPT_MARKER}{}{text}",
            if trusted_automation {
                SAND_TRUSTED_AUTOMATION_PROMPT_MARKER
            } else {
                ""
            }
        );
    }

    projected[user_index].content = text.clone();

    let unanswered = unanswered_questions_note(args);
    let prepended_unanswered_questions = !unanswered.is_empty();
    if prepended_unanswered_questions {
        projected.insert(
            user_index,
            ProviderMessage {
                role: "user".into(),
                content: unanswered,
            },
        );
    }

    ProviderPromptProjection {
        messages: projected,
        projected_user_text: Some(text),
        prepended_unanswered_questions,
    }
}

fn parse_reply_context(value: Option<&Value>) -> Option<ReplyContext> {
    let value = value?.as_object()?;
    let target_id = value.get("targetId")?.as_str()?.trim();
    let quote = value.get("quote")?.as_str()?.trim();
    if target_id.is_empty() || quote.is_empty() {
        return None;
    }
    Some(ReplyContext {
        target_id: target_id.to_string(),
        quote: quote.to_string(),
    })
}

fn optional_non_empty<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn string_array(value: &Value, field: &str) -> Option<Vec<String>> {
    let rows = value.get(field)?.as_array()?;
    Some(
        rows.iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .collect(),
    )
}

fn number_map(value: Option<&Value>) -> HashMap<String, f64> {
    value
        .and_then(Value::as_object)
        .map(|rows| {
            rows.iter()
                .filter_map(|(key, value)| value.as_f64().map(|value| (key.clone(), value)))
                .collect()
        })
        .unwrap_or_default()
}

fn string_map(value: Option<&Value>) -> HashMap<String, String> {
    value
        .and_then(Value::as_object)
        .map(|rows| {
            rows.iter()
                .filter_map(|(key, value)| {
                    value
                        .as_str()
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(|value| (key.clone(), value.to_string()))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn unanswered_questions_note(args: &Value) -> String {
    let mut questions = string_array(args, "skippedQuestionPrompts").unwrap_or_default();
    questions.extend(string_array(args, "dismissedQuestionPrompts").unwrap_or_default());
    if questions.is_empty() {
        String::new()
    } else {
        format!(
            "Unanswered questions:\n{}",
            questions
                .iter()
                .map(|question| format!("- {question}"))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}
