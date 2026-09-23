use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use serde_json::{Map, Value, json};

use super::agent_db_transcript_pages::{TranscriptPageQuery, TranscriptWindowQuery};
use super::agent_session::SandAgentSessionStore;
use super::production::ProductionSessionWorkers;
use super::session_profile_files::AgentProfileUpdate;

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
    session: &Arc<ProductionSessionWorkers>,
    method: &str,
    args: &Value,
) -> Option<Result<Value, SessionGatewayError>> {
    let store = SandAgentSessionStore::new(Arc::clone(session));
    let result = match method {
        "countAgents" => session
            .count_owned_agents()
            .map(|count| json!(count))
            .map_err(SessionGatewayError::internal),
        "listAgents" => store
            .list_agents()
            .and_then(|agents| serde_json::to_value(agents).map_err(|error| error.to_string()))
            .map_err(SessionGatewayError::internal),
        "updateAgent" => required_string(args, "id").and_then(|agent_id| {
            parse_profile_update(args)
                .and_then(|update| {
                    store
                        .update_agent_profile(agent_id, &update)
                        .map_err(SessionGatewayError::internal)
                })
                .and_then(|summary| {
                    serde_json::to_value(summary)
                        .map_err(|error| SessionGatewayError::internal(error.to_string()))
                })
        }),
        "setAgentUnread" => required_string(args, "id").and_then(|agent_id| {
            required_bool(args, "isUnread").and_then(|is_unread| {
                let at_ms = optional_f64(args, "atMs").unwrap_or_else(system_now_ms);
                store
                    .set_session_unread(agent_id, is_unread, at_ms)
                    .map(|_| Value::Null)
                    .map_err(SessionGatewayError::internal)
            })
        }),
        "setAgentNotifyOnUpdates" => required_string(args, "id").and_then(|agent_id| {
            required_bool(args, "isEnabled").and_then(|enabled| {
                store
                    .set_session_notify_on_updates(agent_id, enabled)
                    .map(|_| Value::Null)
                    .map_err(SessionGatewayError::internal)
            })
        }),
        "setAgentHiddenFromSidebar" => required_string(args, "id").and_then(|agent_id| {
            required_bool(args, "isHidden").and_then(|hidden| {
                store
                    .set_session_hidden_from_sidebar(agent_id, hidden)
                    .map(|_| Value::Null)
                    .map_err(SessionGatewayError::internal)
            })
        }),
        "getAgentAvatar" => required_string(args, "id").and_then(|agent_id| {
            store
                .get_agent_avatar(agent_id)
                .map(|avatar| {
                    json!({
                        "version": avatar.version,
                        "dataUrl": avatar.data_url,
                    })
                })
                .map_err(SessionGatewayError::internal)
        }),
        "setAgentAvatarBytes" => required_string(args, "id").and_then(|agent_id| {
            decode_optional_png(args)
                .and_then(|png| {
                    store
                        .set_agent_avatar_bytes(agent_id, png.as_deref())
                        .map_err(SessionGatewayError::internal)
                })
                .and_then(|summary| {
                    serde_json::to_value(summary)
                        .map_err(|error| SessionGatewayError::internal(error.to_string()))
                })
        }),
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

fn parse_profile_update(args: &Value) -> Result<AgentProfileUpdate, SessionGatewayError> {
    let profile = args
        .get("profile")
        .and_then(Value::as_object)
        .ok_or_else(|| SessionGatewayError::bad("missing or invalid profile"))?;
    let name = object_required_string(profile, "name")?;
    let description = object_required_string(profile, "description")?;
    Ok(AgentProfileUpdate {
        name: name.to_string(),
        description: description.to_string(),
        title: object_optional_string(profile, "title")?.map(ToOwned::to_owned),
        avatar_shape: object_optional_string(profile, "avatarShape")?.map(ToOwned::to_owned),
        avatar_color: object_optional_string(profile, "avatarColor")?.map(ToOwned::to_owned),
    })
}

fn object_required_string<'a>(
    object: &'a Map<String, Value>,
    field: &str,
) -> Result<&'a str, SessionGatewayError> {
    object
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| SessionGatewayError::bad(format!("missing or invalid profile.{field}")))
}

fn object_optional_string<'a>(
    object: &'a Map<String, Value>,
    field: &str,
) -> Result<Option<&'a str>, SessionGatewayError> {
    match object.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value)),
        Some(_) => Err(SessionGatewayError::bad(format!(
            "invalid profile.{field}"
        ))),
    }
}

fn required_bool(args: &Value, field: &str) -> Result<bool, SessionGatewayError> {
    args.get(field)
        .and_then(Value::as_bool)
        .ok_or_else(|| SessionGatewayError::bad(format!("missing or invalid {field}")))
}

fn optional_f64(args: &Value, field: &str) -> Option<f64> {
    args.get(field)
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
}

fn system_now_ms() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
        * 1000.0
}

fn decode_optional_png(args: &Value) -> Result<Option<Vec<u8>>, SessionGatewayError> {
    match args.get("pngBase64") {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(encoded)) => BASE64_STANDARD
            .decode(encoded)
            .map(Some)
            .map_err(|_| SessionGatewayError::bad("invalid pngBase64")),
        Some(_) => Err(SessionGatewayError::bad("invalid pngBase64")),
    }
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
