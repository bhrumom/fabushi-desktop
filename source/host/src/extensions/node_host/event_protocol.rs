//! Strict node.event parameter construction.
//!
//! The connection/runtime owners provide the event name and optional payload;
//! this module owns only the canonical `event`/`payloadJSON` envelope. It does
//! not publish frames, own sockets, or decide which events are observable.

use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NodeEventParams {
    pub event: String,
    #[serde(rename = "payloadJSON")]
    pub payload_json: Option<String>,
}

pub fn build_node_event_params(
    event: impl Into<String>,
    payload: Option<&Value>,
) -> Result<NodeEventParams, String> {
    let payload_json = payload
        .map(serde_json::to_string)
        .transpose()
        .map_err(|error| format!("node.event payload serialization failed: {error}"))?;
    Ok(NodeEventParams {
        event: event.into(),
        payload_json,
    })
}

#[cfg(test)]
mod tests {
    use super::build_node_event_params;

    #[test]
    fn preserves_event_name_and_serializes_payload_json() {
        let params = build_node_event_params(
            "system.run.progress",
            Some(&serde_json::json!({"seq": 2, "text": "héllo"})),
        )
        .expect("payload serializes");
        assert_eq!(params.event, "system.run.progress");
        assert_eq!(
            params.payload_json.as_deref(),
            Some(r#"{"seq":2,"text":"héllo"}"#)
        );
    }

    #[test]
    fn represents_undefined_payload_as_json_null_field() {
        let params = build_node_event_params("node.ready", None).expect("event builds");
        assert_eq!(params.payload_json, None);
        assert_eq!(
            serde_json::to_value(params).expect("params serialize"),
            serde_json::json!({"event":"node.ready","payloadJSON":null})
        );
    }
}
