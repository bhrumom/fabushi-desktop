use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use crate::extensions::inference::provider_session::{
    ProviderSessionError, RoutedToolDefinition,
};
use crate::runner::routed_provider_runtime::RoutedToolBridge;

use super::send_message_encoding::encode_send_message;
use super::send_message_schema::{
    SendMessageInput, SendMessageType, parse_send_message_input, send_message_input_schema,
    validate_send_message,
};
use super::sand_secret_request::{clamp_secret_description, clamp_secret_label};

pub const SAND_SEND_MESSAGE_TOOL_NAME: &str = "SendMessage";
pub const SAND_AWAITING_USER_SEND_MESSAGE_BLOCKED: &str =
    "Cannot send another user-facing message while the current user selection is still pending.";

pub trait SendMessageSink: Send + Sync {
    fn is_awaiting_user_selection(&self) -> bool {
        false
    }

    fn send_message(
        &self,
        message: Value,
        timestamp_ms: u64,
        tool_call_id: &str,
    ) -> Result<Option<String>, ProviderSessionError>;
}

pub struct SendMessageToolBridge {
    delegate: Arc<dyn RoutedToolBridge>,
    sink: Arc<dyn SendMessageSink>,
}

impl SendMessageToolBridge {
    pub fn new(delegate: Arc<dyn RoutedToolBridge>, sink: Arc<dyn SendMessageSink>) -> Self {
        Self { delegate, sink }
    }
}

pub fn send_message_tool_definition() -> RoutedToolDefinition {
    RoutedToolDefinition {
        name: SAND_SEND_MESSAGE_TOOL_NAME.to_string(),
        provider_identifier: "fabushi-runner".to_string(),
        tool_name: SAND_SEND_MESSAGE_TOOL_NAME.to_string(),
        description: Some(
            "Send a user-visible message. Use text for normal chat, attachment for standalone files/media, widget for a selectable question, cursor-agent for a cloud-agent reference, and secret-request for secure credential input.".to_string(),
        ),
        input_schema: send_message_input_schema(),
    }
}

fn build_sand_send_message(input: &SendMessageInput) -> Result<Value, ProviderSessionError> {
    let reply = input.reply_to.as_deref().filter(|value| !value.is_empty());
    let channel = input.channel.as_deref().filter(|value| !value.is_empty());
    let message = match input.message_type {
        SendMessageType::Text => {
            let mut value = json!({
                "type": "text",
                "content": input.content.as_deref().unwrap_or_default()
            });
            if let Some(images) = input.images.as_ref().filter(|images| !images.is_empty()) {
                value["images"] = serde_json::to_value(images)
                    .map_err(|error| ProviderSessionError::Tool(error.to_string()))?;
            }
            value
        }
        SendMessageType::Attachment => json!({
            "type": "attachment",
            "url": input.url.as_deref().unwrap_or_default(),
            "alt": input.alt.as_deref()
        }),
        SendMessageType::Widget => json!({
            "type": "widget",
            "widget": input.widget.clone().unwrap_or(Value::Null)
        }),
        SendMessageType::CursorAgent => json!({
            "type": "cursor-agent",
            "bcId": input.bc_id.as_deref().unwrap_or_default()
        }),
        SendMessageType::SecretRequest => {
            let secret = input.secret.as_ref().ok_or_else(|| {
                ProviderSessionError::Tool("secret is required when type is secret-request".into())
            })?;
            let mut request = json!({
                "label": clamp_secret_label(&secret.label),
                "target": {
                    "kind": "channel-credential",
                    "platform": secret.connector,
                    "field": secret.field
                }
            });
            if let Some(description) = secret.description.as_deref().filter(|value| !value.is_empty()) {
                request["description"] = Value::String(clamp_secret_description(description));
            }
            json!({
                "type": "secret-request",
                "secretRequest": request
            })
        }
    };
    let mut message = message;
    if let Some(reply) = reply {
        message["reply_to"] = Value::String(reply.to_string());
    }
    if matches!(input.message_type, SendMessageType::Text | SendMessageType::Attachment) {
        if let Some(channel) = channel {
            message["channel"] = Value::String(channel.to_string());
        }
    }
    Ok(message)
}

impl RoutedToolBridge for SendMessageToolBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        let mut tools = self.delegate.list_tools()?;
        tools.retain(|tool| {
            tool.name != SAND_SEND_MESSAGE_TOOL_NAME
                && tool.tool_name != SAND_SEND_MESSAGE_TOOL_NAME
        });
        tools.insert(0, send_message_tool_definition());
        Ok(tools)
    }

    fn call_tool(
        &self,
        tool: &RoutedToolDefinition,
        args: Value,
        tool_call_id: &str,
    ) -> Result<Value, ProviderSessionError> {
        if tool.name != SAND_SEND_MESSAGE_TOOL_NAME
            && tool.tool_name != SAND_SEND_MESSAGE_TOOL_NAME
        {
            return self.delegate.call_tool(tool, args, tool_call_id);
        }

        let input = parse_send_message_input(&args).map_err(ProviderSessionError::Tool)?;
        if let Err(issues) = validate_send_message(&input) {
            let message = issues
                .into_iter()
                .map(|issue| format!("{}: {}", issue.path.join("."), issue.message))
                .collect::<Vec<_>>()
                .join("; ");
            return Err(ProviderSessionError::Tool(message));
        }
        if self.sink.is_awaiting_user_selection() {
            return Err(ProviderSessionError::Tool(
                SAND_AWAITING_USER_SEND_MESSAGE_BLOCKED.to_string(),
            ));
        }

        let message = build_sand_send_message(&input)?;
        let _encoded = encode_send_message(&message).map_err(ProviderSessionError::Tool)?;
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|value| value.as_millis().min(u64::MAX as u128) as u64)
            .unwrap_or_default();
        let message_id = self.sink.send_message(message, timestamp_ms, tool_call_id)?;
        Ok(json!({
            "sent": true,
            "timestampMs": timestamp_ms,
            "messageId": message_id
        }))
    }
}
