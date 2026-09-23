use serde_json::{Map, Value};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct UserMessageOptions {
    pub composed_at_ms: Option<f64>,
    pub rich_text: Option<String>,
    pub reply_to: Option<String>,
    pub batch_id: Option<String>,
    pub branched: bool,
    pub client_nonce: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UserAttachmentOptions {
    pub file_name: Option<String>,
    pub batch_id: Option<String>,
    pub client_nonce: Option<String>,
    pub byte_size: Option<u64>,
    pub reply_to: Option<String>,
    pub branched: bool,
}

pub fn is_user_message_entry(entry: &Value) -> bool {
    match entry.get("kind").and_then(Value::as_str) {
        Some("message") => entry.get("role").and_then(Value::as_str) == Some("user"),
        Some("user-attachment") => true,
        _ => false,
    }
}

pub fn quotable_entry_text(entry: &Value) -> String {
    if entry.get("kind").and_then(Value::as_str) == Some("message") {
        return entry.get("content").and_then(Value::as_str).unwrap_or_default().to_string();
    }
    if entry.get("kind").and_then(Value::as_str) == Some("send-message")
        && entry.get("message").and_then(|message| message.get("type")).and_then(Value::as_str) == Some("text")
    {
        return entry.get("message").and_then(|message| message.get("content")).and_then(Value::as_str).unwrap_or_default().to_string();
    }
    String::new()
}

pub fn describe_message_quote(entry: &Value, max_chars: Option<usize>) -> String {
    let normalized = quotable_entry_text(entry).split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return entry.get("id").and_then(Value::as_str).unwrap_or_default().to_string();
    }
    if let Some(max_chars) = max_chars {
        if normalized.chars().count() > max_chars {
            return format!("{}…", normalized.chars().take(max_chars).collect::<String>());
        }
    }
    normalized
}

pub fn describe_reacted_message_quote(entry: &Value) -> String {
    describe_message_quote(entry, Some(80))
}

pub fn describe_replied_message_quote(entry: &Value) -> String {
    describe_message_quote(entry, None)
}

pub fn create_user_message(
    id: impl Into<String>,
    content: impl Into<String>,
    options: UserMessageOptions,
    now_ms: f64,
) -> Value {
    let mut entry = Map::new();
    entry.insert("kind".into(), Value::String("message".into()));
    entry.insert("id".into(), Value::String(id.into()));
    entry.insert("role".into(), Value::String("user".into()));
    entry.insert("content".into(), Value::String(content.into()));
    entry.insert("isStreaming".into(), Value::Bool(false));
    let composed_at_ms = options.composed_at_ms.filter(|value| value.is_finite());
    insert_f64(&mut entry, "timestampMs", composed_at_ms.unwrap_or(now_ms));
    if let Some(rich_text) = options.rich_text.filter(|value| !value.is_empty()) {
        entry.insert("richText".into(), Value::String(rich_text));
    }
    if let Some(reply_to) = options.reply_to { entry.insert("replyTo".into(), Value::String(reply_to)); }
    if let Some(batch_id) = options.batch_id { entry.insert("batchId".into(), Value::String(batch_id)); }
    if options.branched { entry.insert("branched".into(), Value::Bool(true)); }
    if let Some(client_nonce) = options.client_nonce.filter(|value| !value.is_empty()) {
        entry.insert("clientNonce".into(), Value::String(client_nonce));
    }
    if let Some(composed_at_ms) = composed_at_ms { insert_f64(&mut entry, "sentWhileOfflineAtMs", composed_at_ms); }
    Value::Object(entry)
}

pub fn create_send_message_entry(id: impl Into<String>, message: Value, timestamp_ms: f64) -> Value {
    let mut entry = Map::new();
    entry.insert("kind".into(), Value::String("send-message".into()));
    entry.insert("id".into(), Value::String(id.into()));
    if let Some(reply_to) = message.get("reply_to").and_then(Value::as_str).filter(|value| !value.is_empty()) {
        entry.insert("replyTo".into(), Value::String(reply_to.to_string()));
    }
    entry.insert("message".into(), message);
    insert_f64(&mut entry, "timestampMs", timestamp_ms);
    Value::Object(entry)
}

pub fn stamp_box_request_entry(entry: &Value, request_id: &str, instruction: &str) -> Value {
    let mut next = entry.as_object().cloned().unwrap_or_default();
    next.insert("boxRequestId".into(), Value::String(request_id.to_string()));
    next.insert("boxInstruction".into(), Value::String(instruction.to_string()));
    Value::Object(next)
}

pub fn create_user_attachment_entry(
    id: impl Into<String>,
    file_path: impl Into<String>,
    options: UserAttachmentOptions,
) -> Value {
    let mut entry = Map::new();
    entry.insert("kind".into(), Value::String("user-attachment".into()));
    entry.insert("id".into(), Value::String(id.into()));
    entry.insert("file_path".into(), Value::String(file_path.into()));
    if let Some(file_name) = options.file_name.map(|value| value.trim().to_string()).filter(|value| !value.is_empty()) {
        entry.insert("file_name".into(), Value::String(file_name));
    }
    if let Some(batch_id) = options.batch_id { entry.insert("batchId".into(), Value::String(batch_id)); }
    if let Some(client_nonce) = options.client_nonce.filter(|value| !value.is_empty()) {
        entry.insert("clientNonce".into(), Value::String(client_nonce));
    }
    if let Some(byte_size) = options.byte_size { entry.insert("byteSize".into(), Value::Number(byte_size.into())); }
    if let Some(reply_to) = options.reply_to { entry.insert("replyTo".into(), Value::String(reply_to)); }
    if options.branched { entry.insert("branched".into(), Value::Bool(true)); }
    Value::Object(entry)
}

pub fn stat_attached_file_size(path: &str) -> Option<u64> {
    std::fs::metadata(path).ok().filter(|metadata| metadata.is_file()).map(|metadata| metadata.len())
}

pub fn strip_reply_to(message: &Value) -> Value {
    if protected_reply_type(message) { return message.clone(); }
    shape_threadable(message, None)
}

pub fn with_reply_to(message: &Value, reply_to: &str) -> Value {
    if protected_reply_type(message) { return message.clone(); }
    shape_threadable(message, Some(reply_to))
}

fn protected_reply_type(message: &Value) -> bool {
    matches!(
        message.get("type").and_then(Value::as_str),
        Some("auto-review-approval") | Some("local-tool-permission") | Some("email-draft") | Some("slack-draft")
    )
}

fn shape_threadable(message: &Value, reply_to: Option<&str>) -> Value {
    let Some(kind) = message.get("type").and_then(Value::as_str) else { return message.clone(); };
    let fields: &[&str] = match kind {
        "text" => &["content", "images"],
        "widget" => &["widget"],
        "cursor-agent" => &["bcId"],
        "secret-request" => &["secretRequest"],
        "permission-request" => &["permission"],
        "connector" => &["connector", "variant", "serverId", "reason", "suggestions"],
        "connectors" => &["connectors", "reason"],
        "listener-connect" => &["platform", "reason"],
        "attachment" => &["url", "file_name", "alt", "channel", "width", "height"],
        _ => return message.clone(),
    };
    let mut shaped = Map::new();
    shaped.insert("type".into(), Value::String(kind.to_string()));
    for field in fields {
        if let Some(value) = message.get(*field).filter(|value| !value.is_null()) {
            shaped.insert((*field).to_string(), value.clone());
        }
    }
    if let Some(reply_to) = reply_to { shaped.insert("reply_to".into(), Value::String(reply_to.to_string())); }
    Value::Object(shaped)
}

fn insert_f64(map: &mut Map<String, Value>, key: &str, value: f64) {
    if let Some(number) = serde_json::Number::from_f64(value) {
        map.insert(key.to_string(), Value::Number(number));
    }
}
