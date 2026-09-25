use std::collections::HashMap;

use serde_json::Value;

use crate::extensions::inference::provider_session::ProviderMessage;

use super::sand_prompt_markers::{
    SAND_HIDDEN_PROMPT_MARKER, SAND_TRUSTED_AUTOMATION_PROMPT_MARKER,
};
use super::system_prompt::{
    ReplyContext, append_user_reply_reminder, build_attached_files_note,
    build_reply_context_note, build_user_message_address_note,
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
