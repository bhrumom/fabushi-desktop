use serde_json::{Value, json};

use super::agent_db_transcript_pages::{TranscriptPageQuery, TranscriptWindowQuery};
use super::production::ProductionSessionWorkers;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionGatewayError {
    BadRequest(String),
    Internal(String),
}

impl SessionGatewayError {
    fn bad(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }

    fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
}

pub fn dispatch_production_session_gateway_call(
    session: &ProductionSessionWorkers,
    method: &str,
    args: &Value,
) -> Option<Result<Value, SessionGatewayError>> {
    let result = match method {
        "countAgents" => session
            .count_owned_agents()
            .map(|count| json!(count))
            .map_err(SessionGatewayError::internal),
        "getAgentTranscript" => required_string(args, "id").and_then(|agent_id| {
            session
                .read_agent_transcript_entries(agent_id)
                .map(Value::Array)
                .map_err(SessionGatewayError::internal)
        }),
        "getAgentTranscriptPage" => required_string(args, "id").and_then(|agent_id| {
            let query = TranscriptPageQuery {
                before_seq: optional_i64(args, "beforeSeq"),
                since_ms: optional_i64(args, "sinceMs"),
                until_ms: optional_i64(args, "untilMs").unwrap_or(i64::MAX),
                limit: optional_i64(args, "limit").unwrap_or(500),
            };
            session
                .read_agent_transcript_page(agent_id, query)
                .map(|page| {
                    json!({
                        "entries": page.entries,
                        "nextBeforeSeq": page.next_before_seq,
                    })
                })
                .map_err(SessionGatewayError::internal)
        }),
        "getAgentTranscriptWindow" => required_string(args, "id").and_then(|agent_id| {
            let query = TranscriptWindowQuery {
                before_seq: optional_i64(args, "beforeSeq"),
                limit: optional_i64(args, "limit").unwrap_or(500),
            };
            session
                .read_agent_transcript_window(agent_id, query)
                .map(|window| {
                    json!({
                        "entries": window.entries,
                        "nextBeforeSeq": window.next_before_seq,
                        "threadCounts": window.thread_counts,
                    })
                })
                .map_err(SessionGatewayError::internal)
        }),
        "getAgentTranscriptTail" => required_string(args, "id").and_then(|agent_id| {
            let query = TranscriptWindowQuery {
                before_seq: optional_i64(args, "beforeSeq"),
                limit: optional_i64(args, "limit").unwrap_or(500),
            };
            session
                .read_agent_transcript_tail(agent_id, query)
                .map(|page| {
                    json!({
                        "entries": page.entries,
                        "nextBeforeSeq": page.next_before_seq,
                    })
                })
                .map_err(SessionGatewayError::internal)
        }),
        "getAgentThread" => required_string(args, "id").and_then(|agent_id| {
            required_string(args, "rootId").and_then(|root_id| {
                session
                    .read_agent_thread(agent_id, root_id)
                    .map(|thread| json!({ "entries": thread.entries }))
                    .map_err(SessionGatewayError::internal)
            })
        }),
        "getAgentChannels" | "refreshChannel" => required_string(args, "id").and_then(|agent_id| {
            session
                .list_agent_channels(agent_id)
                .and_then(|channels| serde_json::to_value(channels).map_err(|error| error.to_string()))
                .map_err(SessionGatewayError::internal)
        }),
        "connectChannel" => required_string(args, "id").and_then(|agent_id| {
            required_string(args, "platform").and_then(|platform| {
                required_string(args, "token").and_then(|token| {
                    session
                        .store_connector_credential(agent_id, platform, "token", token)
                        .map_err(SessionGatewayError::internal)
                        .and_then(|stored| {
                            if !stored {
                                return Err(SessionGatewayError::bad(
                                    "connectChannel rejected invalid agent/platform metadata",
                                ));
                            }
                            session
                                .list_agent_channels(agent_id)
                                .and_then(|channels| {
                                    serde_json::to_value(channels).map_err(|error| error.to_string())
                                })
                                .map_err(SessionGatewayError::internal)
                        })
                })
            })
        }),
        "disconnectChannel" => required_string(args, "id").and_then(|agent_id| {
            required_string(args, "platform").and_then(|platform| {
                session
                    .disconnect_channel(agent_id, platform)
                    .map_err(SessionGatewayError::internal)
                    .and_then(|_| {
                        session
                            .list_agent_channels(agent_id)
                            .and_then(|channels| {
                                serde_json::to_value(channels).map_err(|error| error.to_string())
                            })
                            .map_err(SessionGatewayError::internal)
                    })
            })
        }),
        _ => return None,
    };
    Some(result)
}

fn required_string<'a>(
    args: &'a Value,
    field: &str,
) -> Result<&'a str, SessionGatewayError> {
    args.get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| SessionGatewayError::bad(format!("missing or invalid {field}")))
}

fn optional_i64(args: &Value, field: &str) -> Option<i64> {
    let value = args.get(field)?;
    value
        .as_i64()
        .or_else(|| value.as_u64().and_then(|value| i64::try_from(value).ok()))
        .or_else(|| {
            value
                .as_f64()
                .filter(|value| value.is_finite())
                .map(|value| value.trunc() as i64)
        })
}
