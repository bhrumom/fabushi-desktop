use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use url::Url;

pub const SEND_MESSAGE_TYPES: &[&str] =
    &["text", "attachment", "widget", "cursor-agent", "secret-request"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SendMessageType {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "attachment")]
    Attachment,
    #[serde(rename = "widget")]
    Widget,
    #[serde(rename = "cursor-agent")]
    CursorAgent,
    #[serde(rename = "secret-request")]
    SecretRequest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendMessageImage {
    pub url: String,
    #[serde(default)]
    pub alt: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendMessageSecret {
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    pub connector: String,
    pub field: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SendMessageInput {
    #[serde(rename = "type")]
    pub message_type: SendMessageType,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub images: Option<Vec<SendMessageImage>>,
    #[serde(default)]
    pub alt: Option<String>,
    #[serde(default, rename = "reply_to")]
    pub reply_to: Option<String>,
    #[serde(default)]
    pub channel: Option<String>,
    #[serde(default)]
    pub widget: Option<Value>,
    #[serde(default, rename = "bcId")]
    pub bc_id: Option<String>,
    #[serde(default)]
    pub secret: Option<SendMessageSecret>,
}

impl SendMessageInput {
    pub fn normalize(mut self) -> Self {
        fn trim(value: &mut Option<String>) {
            if let Some(current) = value.take() {
                *value = Some(current.trim().to_string());
            }
        }
        trim(&mut self.content);
        trim(&mut self.url);
        trim(&mut self.alt);
        trim(&mut self.reply_to);
        trim(&mut self.channel);
        trim(&mut self.bc_id);
        if let Some(images) = self.images.as_mut() {
            for image in images {
                image.url = image.url.trim().to_string();
                if let Some(alt) = image.alt.take() {
                    image.alt = Some(alt.trim().to_string());
                }
            }
        }
        if let Some(secret) = self.secret.as_mut() {
            secret.label = secret.label.trim().to_string();
            secret.connector = secret.connector.trim().to_string();
            secret.field = secret.field.trim().to_string();
            if let Some(description) = secret.description.take() {
                secret.description = Some(description.trim().to_string());
            }
        }
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendMessageIssue {
    pub path: Vec<String>,
    pub message: String,
}

pub fn parse_send_message_input(value: &Value) -> Result<SendMessageInput, String> {
    serde_json::from_value::<SendMessageInput>(value.clone())
        .map(SendMessageInput::normalize)
        .map_err(|error| format!("invalid SendMessage input: {error}"))
}

pub fn is_valid_attachment_url(value: &str) -> bool {
    Url::parse(value)
        .ok()
        .is_some_and(|url| matches!(url.scheme(), "file" | "https"))
}

fn provided_string(value: &Option<String>) -> bool {
    value.as_deref().is_some_and(|value| !value.is_empty())
}

fn issue(field: &str, message: impl Into<String>) -> SendMessageIssue {
    SendMessageIssue {
        path: vec![field.to_string()],
        message: message.into(),
    }
}

pub fn refine_send_message(value: &SendMessageInput) -> Vec<SendMessageIssue> {
    let mut issues = Vec::new();
    let type_name = match value.message_type {
        SendMessageType::Text => "text",
        SendMessageType::Attachment => "attachment",
        SendMessageType::Widget => "widget",
        SendMessageType::CursorAgent => "cursor-agent",
        SendMessageType::SecretRequest => "secret-request",
    };

    let guarded = [
        ("content", provided_string(&value.content), &["text"][..]),
        ("url", provided_string(&value.url), &["attachment"][..]),
        ("alt", provided_string(&value.alt), &["attachment"][..]),
        ("widget", value.widget.is_some(), &["widget"][..]),
        ("bcId", provided_string(&value.bc_id), &["cursor-agent"][..]),
        ("secret", value.secret.is_some(), &["secret-request"][..]),
    ];
    for (field, provided, allowed) in guarded {
        if provided && !allowed.contains(&type_name) {
            let allowed = allowed
                .iter()
                .map(|name| format!("type:{name}"))
                .collect::<Vec<_>>()
                .join(" or ");
            issues.push(issue(
                field,
                format!(
                    "{field} is only valid with {allowed} and cannot ride a type:{type_name} message - it would be silently dropped. Nothing was sent. Re-send as separate SendMessage calls, one per type: this field on its own properly-typed message ({allowed}), and any text as its own type:text message."
                ),
            ));
        }
    }

    if provided_string(&value.channel)
        && !matches!(
            value.message_type,
            SendMessageType::Text | SendMessageType::Attachment
        )
    {
        issues.push(issue(
            "channel",
            "channel can only be set for type:text or type:attachment, not widgets or cursor-agent cards",
        ));
    }
    if value.images.as_ref().is_some_and(|images| !images.is_empty())
        && value.message_type != SendMessageType::Text
    {
        issues.push(issue(
            "images",
            "images can only be set for type:text; use type:attachment for a standalone attachment",
        ));
    }

    match value.message_type {
        SendMessageType::Text => {
            if !provided_string(&value.content) {
                issues.push(issue("content", "content is required when type is text"));
            }
            for (index, image) in value.images.as_deref().unwrap_or_default().iter().enumerate() {
                if !is_valid_attachment_url(&image.url) {
                    issues.push(SendMessageIssue {
                        path: vec!["images".into(), index.to_string(), "url".into()],
                        message: "each images url must include a file:// or https:// scheme".into(),
                    });
                }
            }
        }
        SendMessageType::Attachment => {
            let Some(url) = value.url.as_deref().filter(|value| !value.is_empty()) else {
                issues.push(issue("url", "url is required when type is attachment"));
                return issues;
            };
            if !is_valid_attachment_url(url) {
                issues.push(issue(
                    "url",
                    "url must include a file:// or https:// scheme when type is attachment",
                ));
            }
        }
        _ => {}
    }
    issues
}

pub fn validate_send_message(value: &SendMessageInput) -> Result<(), Vec<SendMessageIssue>> {
    let mut issues = refine_send_message(value);
    match value.message_type {
        SendMessageType::Widget if value.widget.is_none() => {
            issues.push(issue("widget", "widget is required when type is widget"));
        }
        SendMessageType::CursorAgent if !provided_string(&value.bc_id) => {
            issues.push(issue("bcId", "bcId is required when type is cursor-agent"));
        }
        SendMessageType::SecretRequest => match value.secret.as_ref() {
            None => issues.push(issue("secret", "secret is required when type is secret-request")),
            Some(secret)
                if secret.label.is_empty()
                    || secret.connector.is_empty()
                    || secret.field.is_empty() =>
            {
                issues.push(issue(
                    "secret",
                    "secret label, connector, and field must be non-empty",
                ));
            }
            _ => {}
        },
        _ => {}
    }
    if issues.is_empty() { Ok(()) } else { Err(issues) }
}

pub fn send_message_input_schema() -> Value {
    json!({
        "type": "object",
        "required": ["type"],
        "additionalProperties": false,
        "properties": {
            "type": {"type": "string", "enum": SEND_MESSAGE_TYPES},
            "content": {"type": "string"},
            "url": {"type": "string"},
            "images": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["url"],
                    "additionalProperties": false,
                    "properties": {
                        "url": {"type": "string"},
                        "alt": {"type": "string"}
                    }
                }
            },
            "alt": {"type": "string"},
            "reply_to": {"type": "string"},
            "channel": {"type": "string"},
            "widget": {"type": "object"},
            "bcId": {"type": "string"},
            "secret": {
                "type": "object",
                "required": ["label", "connector", "field"],
                "additionalProperties": false,
                "properties": {
                    "label": {"type": "string"},
                    "description": {"type": "string"},
                    "connector": {"type": "string"},
                    "field": {"type": "string"}
                }
            }
        }
    })
}
