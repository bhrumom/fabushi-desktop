use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use url::Url;

use serde_json::{Value, json};

use crate::extensions::inference::provider_session::{
    ProviderSessionError, RoutedToolDefinition,
};
use crate::runner::routed_provider_runtime::RoutedToolBridge;

use super::box_help_tool::{BoxHelpOutcome, BoxHelpRequest};
use super::send_message_encoding::encode_send_message;
use super::send_message_schema::{
    SendMessageInput, SendMessageType, parse_send_message_input, send_message_input_schema,
    validate_send_message,
};
use super::sand_secret_request::{clamp_secret_description, clamp_secret_label};

pub const SAND_SEND_MESSAGE_TOOL_NAME: &str = "SendMessage";
pub const SAND_AWAITING_USER_SEND_MESSAGE_BLOCKED: &str =
    "This turn is already waiting on the user (you sent a question widget or handed the box back to them), so this message was not delivered. Wait for the user — their response arrives as the next message — then say this on your next turn.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedAttachmentSource {
    pub url: String,
    pub file_name: Option<String>,
}

pub fn file_path_from_file_url(raw: &str) -> Option<PathBuf> {
    let url = Url::parse(raw).ok()?;
    if url.scheme() != "file" {
        return None;
    }
    url.to_file_path().ok()
}

pub fn identity_attachment_source(raw: &str) -> ResolvedAttachmentSource {
    let file_name = file_path_from_file_url(raw)
        .and_then(|path| path.file_name().map(|value| value.to_string_lossy().into_owned()))
        .filter(|value| !value.is_empty());
    ResolvedAttachmentSource {
        url: raw.to_string(),
        file_name,
    }
}

pub trait SendMessageSink: Send + Sync {
    fn is_awaiting_user_selection(&self) -> bool {
        false
    }

    fn request_box_help(
        &self,
        _request: BoxHelpRequest,
        _timestamp_ms: u64,
        _tool_call_id: &str,
    ) -> Result<BoxHelpOutcome, ProviderSessionError> {
        Err(ProviderSessionError::Tool(
            "request_box_help is unavailable for this Runner".into(),
        ))
    }

    fn resolve_attachment_source(
        &self,
        source_url: &str,
        _tool_call_id: &str,
    ) -> Result<ResolvedAttachmentSource, ProviderSessionError> {
        Ok(identity_attachment_source(source_url))
    }

    fn read_media_dimensions(&self, _resolved_url: &str) -> Option<(u32, u32)> {
        None
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

fn build_sand_send_message(
    input: &SendMessageInput,
    sink: &dyn SendMessageSink,
    tool_call_id: &str,
) -> Result<Value, ProviderSessionError> {
    let reply = input.reply_to.as_deref().filter(|value| !value.is_empty());
    let channel = input.channel.as_deref().filter(|value| !value.is_empty());
    let message = match input.message_type {
        SendMessageType::Text => {
            let mut value = json!({
                "type": "text",
                "content": input.content.as_deref().unwrap_or_default()
            });
            if let Some(images) = input.images.as_ref().filter(|images| !images.is_empty()) {
                let resolved = images
                    .iter()
                    .map(|image| {
                        let source = sink.resolve_attachment_source(&image.url, tool_call_id)?;
                        let dimensions = sink.read_media_dimensions(&source.url);
                        let mut image_value = json!({
                            "url": source.url,
                            "alt": image.alt,
                        });
                        if let Some((width, height)) = dimensions {
                            image_value["width"] = Value::Number(width.into());
                            image_value["height"] = Value::Number(height.into());
                        }
                        Ok::<_, ProviderSessionError>(image_value)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                value["images"] = Value::Array(resolved);
            }
            value
        }
        SendMessageType::Attachment => {
            let source = sink.resolve_attachment_source(
                input.url.as_deref().unwrap_or_default(),
                tool_call_id,
            )?;
            let mut value = json!({
                "type": "attachment",
                "url": source.url,
                "alt": input.alt.as_deref()
            });
            if let Some(file_name) = source.file_name {
                value["file_name"] = Value::String(file_name);
            }
            if let Some((width, height)) = sink.read_media_dimensions(&source.url) {
                value["width"] = Value::Number(width.into());
                value["height"] = Value::Number(height.into());
            }
            value
        },
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

        let message = build_sand_send_message(&input, self.sink.as_ref(), tool_call_id)?;
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
