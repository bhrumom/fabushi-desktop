use serde_json::Value;

use crate::extensions::inference::provider_session::ProviderMessage;

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

    Ok(ProductionTurnInputProjection {
        options: TurnRunOptions {
            inference_request_id: Some(request_id.to_string()),
            message_id: optional_non_empty(args, "messageId").map(ToOwned::to_owned),
            recent_message_text: optional_non_empty(args, "recentMessageText")
                .map(ToOwned::to_owned)
                .or(latest_user_text),
            attachment_count: array_len(args, "attachmentPaths"),
            image_count: array_len(args, "selectedImages"),
            video_count: array_len(args, "selectedVideos"),
            has_reply_context: args
                .get("replyContext")
                .is_some_and(|value| !value.is_null()),
        },
    })
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
