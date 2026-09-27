use super::xuser_wire_normalization::MAX_INLINE_IMAGES_PER_ENTRY;
use serde_json::{Value, json};

pub const SAND_SHARED_ROOM_IMAGE_BYTES_MAX: usize = 1_100_000;
pub const REMOTE_AGENT_ID_PREFIX: &str = "sand-remote:";

pub fn parse_remote_agent_id(id: &str) -> Option<(String, String)> {
    let body = id.strip_prefix(REMOTE_AGENT_ID_PREFIX)?;
    let (owner, agent) = body.split_once('/')?;
    let owner = urlencoding_decode(owner)?;
    let agent = urlencoding_decode(agent)?;
    if owner.is_empty() || agent.is_empty() {
        None
    } else {
        Some((owner, agent))
    }
}
fn urlencoding_decode(v: &str) -> Option<String> {
    url::form_urlencoded::parse(v.as_bytes())
        .next()
        .map(|(v, _)| v.into_owned())
}

pub fn build_publish_payload(entry: &Value, self_auth_id: &str) -> Option<Value> {
    let id = entry.get("id")?.as_str()?.trim();
    if id.is_empty() {
        return None;
    }
    let kind = entry.get("kind")?.as_str()?;
    match kind {
        "message" => {
            let content = entry
                .get("content")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim();
            let images = entry
                .get("images")
                .and_then(Value::as_array)
                .map(|v| {
                    v.iter()
                        .take(MAX_INLINE_IMAGES_PER_ENTRY)
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if content.is_empty() && images.is_empty() {
                return None;
            }
            Some(
                json!({"kind":"entry","entryId":id,"authorAuthId":self_auth_id,"text":content,"images":images,"clientNonce":entry.get("clientNonce").cloned().unwrap_or(Value::Null)}),
            )
        }
        "send-message" => {
            if entry.get("streaming").and_then(Value::as_bool) == Some(true) {
                return None;
            }
            let msg = entry.get("message")?;
            let content = msg
                .get("content")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim();
            if content.is_empty() {
                return None;
            }
            Some(json!({"kind":"entry","entryId":id,"authorAuthId":self_auth_id,"text":content}))
        }
        _ => None,
    }
}
