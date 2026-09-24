use serde_json::Value;

use crate::extensions::inference::provider_session::ProviderMessage;

use super::conversation_state::RecentUserMessage;
use super::TurnRunOptions;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionTurnInputProjection {
    pub options: TurnRunOptions,
}

pub fn create_production_turn_input_projection(
    args: &Value,
    stream_id: &str,
    messages: &[ProviderMessage],
) -> Result<ProductionTurnInputProjection, &'static str> {
    let request_id = stream_id.trim();
    if request_id.is_empty() {
        return Err("production turn input projection requires streamId");
    }

    let latest_user_text = messages
        .iter()
        .rev()
        .find(|message| message.role == "user" && !message.content.trim().is_empty())
        .map(|message| message.content.clone());
    let recent_user_messages = recent_user_messages(args);
    let message_id = optional_non_empty(args, "messageId").map(ToOwned::to_owned);
    let current_message_text = message_id.as_deref().and_then(|message_id| {
        recent_user_messages
            .iter()
            .find(|message| message.id == message_id)
            .map(|message| message.text.clone())
    });

    Ok(ProductionTurnInputProjection {
        options: TurnRunOptions {
            inference_request_id: Some(request_id.to_string()),
            message_id,
            recent_message_text: optional_non_empty(args, "recentMessageText")
                .map(ToOwned::to_owned)
                .or(current_message_text)
                .or(latest_user_text),
            recent_user_messages,
            is_fork: args.get("isFork").and_then(Value::as_bool).unwrap_or(false),
            attachment_count: array_len(args, "attachmentPaths"),
            image_count: array_len(args, "selectedImages"),
            video_count: array_len(args, "selectedVideos"),
            has_reply_context: args
                .get("replyContext")
                .is_some_and(|value| !value.is_null()),
        },
    })
}

fn recent_user_messages(value: &Value) -> Vec<RecentUserMessage> {
    value
        .get("recentUserMessages")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|message| {
            let id = message.get("id")?.as_str()?.trim();
            if id.is_empty() {
                return None;
            }
            let text = message
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            Some(RecentUserMessage {
                id: id.to_string(),
                text,
            })
        })
        .collect()
}

fn optional_non_empty<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn array_len(value: &Value, field: &str) -> usize {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or_default()
}
