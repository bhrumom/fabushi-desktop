use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use serde_json::{Map, Value, json};

use crate::agents::agent_profile::SandAgentProfile;
use crate::extensions::transcript::group_chat_glue::{
    GroupChatGlue, GroupChatGlueError,
};
use crate::selected_image_inputs::read_image_file_dimensions;
use crate::extensions::transcript::send_acceptance::{
    mark_accepted_send_activity, prepare_send_acceptance,
};
use crate::extensions::transcript::send_message_shaping::{
    UserAttachmentOptions, UserMessageOptions, create_user_attachment_entry,
    create_user_message, stat_attached_file_size,
};
use crate::extensions::transcript::send_thread_stamping::resolve_send_reply_threading;
use crate::extensions::transcript::transcript_entry_ids::{
    TranscriptEntryIdKind, next_entry_id,
};
use uuid::Uuid;

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

fn recent_recovery_user_messages(
    entries: &[Value],
) -> Vec<crate::runner::RecoveryUserMessage> {
    entries
        .iter()
        .filter(|entry| {
            entry.get("kind").and_then(Value::as_str) == Some("message")
                && entry.get("role").and_then(Value::as_str) == Some("user")
                && entry.get("fromAgent").is_none_or(Value::is_null)
                && entry.get("channel").is_none_or(Value::is_null)
        })
        .filter_map(|entry| {
            let id = entry.get("id").and_then(Value::as_str)?.trim();
            if id.is_empty() {
                return None;
            }
            Some(crate::runner::RecoveryUserMessage {
                id: id.to_string(),
                text: entry
                    .get("content")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                confirmed: entry.get("confirmed").and_then(Value::as_bool),
            })
        })
        .collect()
}

pub fn persist_accepted_send_prompt(
    session: &Arc<ProductionSessionWorkers>,
    args: &Value,
    accepted: &Value,
) -> Result<Option<String>, SessionGatewayError> {
    Ok(
        persist_accepted_send_prompt_context(session, args, accepted)?
            .echo_entry_id
    )
}

pub fn persist_accepted_send_prompt_context(
    session: &Arc<ProductionSessionWorkers>,
    args: &Value,
    accepted: &Value,
) -> Result<crate::extensions::transcript::send_pipeline::PersistedSendContext, SessionGatewayError> {
    use crate::extensions::transcript::send_pipeline::PersistedSendContext;

    if accepted.get("accepted").and_then(Value::as_bool) != Some(true) {
        return Ok(PersistedSendContext::default());
    }
    let agent_id = args
        .get("agentId")
        .or_else(|| args.get("id"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| SessionGatewayError::bad("sendPrompt requires agentId"))?;
    let prompt = args
        .get("prompt")
        .or_else(|| args.get("text"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string();
    let attachment_paths = args
        .get("attachmentPaths")
        .and_then(Value::as_array)
        .map(|values| values.iter().filter_map(Value::as_str).map(str::to_string).collect::<Vec<_>>())
        .unwrap_or_default();
    if prompt.is_empty() && attachment_paths.is_empty() {
        return Err(SessionGatewayError::bad("sendPrompt requires prompt or attachments"));
    }

    let attachment_names = args.get("attachmentNames").and_then(Value::as_array).cloned().unwrap_or_default();
    let client_nonce = optional_string(args, "clientNonce")?
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let mut existing = session
        .read_agent_transcript_entries(agent_id)
        .map_err(SessionGatewayError::internal)?;

    if let Some(client_nonce) = client_nonce.as_deref() {
        let existing_user_message_id = existing
            .iter()
            .find(|entry| {
                entry.get("clientNonce").and_then(Value::as_str) == Some(client_nonce)
                    && entry.get("kind").and_then(Value::as_str) == Some("message")
                    && entry.get("role").and_then(Value::as_str) == Some("user")
            })
            .and_then(|entry| entry.get("id"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        let existing_echo_id = existing_user_message_id.clone().or_else(|| {
            existing
                .iter()
                .find(|entry| {
                    entry.get("clientNonce").and_then(Value::as_str) == Some(client_nonce)
                })
                .and_then(|entry| entry.get("id"))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        });
        if existing_echo_id.is_some() {
            return Ok(PersistedSendContext {
                echo_entry_id: existing_echo_id,
                user_message_id: existing_user_message_id,
                recent_user_messages: recent_recovery_user_messages(&existing),
            });
        }
    }

    prepare_send_acceptance(session, agent_id, existing.len(), &prompt)
        .map_err(SessionGatewayError::internal)?;

    let threading = resolve_send_reply_threading(
        &existing,
        optional_string(args, "replyToId")?,
        optional_bool(args, "isFork")?.unwrap_or(false),
    );
    let batch_id = (!attachment_paths.is_empty()).then(|| Uuid::new_v4().to_string());
    let mut staged = Vec::new();
    let mut first_echo_id = None;

    for (index, file_path) in attachment_paths.iter().enumerate() {
        let id = next_entry_id(&existing, TranscriptEntryIdKind::UserAttachment);
        let file_name = attachment_names
            .get(index)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        let entry = create_user_attachment_entry(
            id.clone(),
            file_path.clone(),
            UserAttachmentOptions {
                file_name,
                batch_id: batch_id.clone(),
                client_nonce: client_nonce.clone(),
                byte_size: stat_attached_file_size(file_path),
                width: read_image_file_dimensions(file_path).map(|value| value.width),
                height: read_image_file_dimensions(file_path).map(|value| value.height),
                reply_to: threading.reply_to_id.clone(),
                branched: threading.is_fork,
            },
        );
        first_echo_id.get_or_insert_with(|| id.clone());
        existing.push(entry.clone());
        staged.push(entry);
    }

    let append_user_message = optional_bool(args, "appendUserMessage")?.unwrap_or(true);
    let mut user_message_id = None;
    if append_user_message && !prompt.is_empty() {
        let id = next_entry_id(&existing, TranscriptEntryIdKind::UserMessage);
        let entry = create_user_message(
            id.clone(),
            prompt,
            UserMessageOptions {
                composed_at_ms: optional_f64(args, "composedAtMs"),
                rich_text: optional_string(args, "richText")?.map(ToOwned::to_owned),
                reply_to: threading.reply_to_id.clone(),
                batch_id: batch_id.clone(),
                branched: threading.is_fork,
                client_nonce: client_nonce.clone(),
            },
            system_now_ms(),
        );
        user_message_id = Some(id.clone());
        first_echo_id.get_or_insert_with(|| id.clone());
        existing.push(entry.clone());
        staged.push(entry);
    }

    if !staged.is_empty() {
        session.append_agent_transcript_entries(agent_id, &staged).map_err(SessionGatewayError::internal)?;
        mark_accepted_send_activity(session, agent_id, system_now_ms())
            .map_err(SessionGatewayError::internal)?;
    }
    Ok(PersistedSendContext {
        echo_entry_id: user_message_id.clone().or(first_echo_id),
        user_message_id,
        recent_user_messages: recent_recovery_user_messages(&existing),
    })
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
        "createGroup" => {
            required_string(args, "name").and_then(|name| {
                let description = optional_string(args, "description")?.unwrap_or_default();
                let member_ids = required_string_array(args, "memberIds")?;
                GroupChatGlue::new(Arc::clone(session))
                    .create_group(name, description, &member_ids)
                    .map_err(map_group_chat_error)
                    .and_then(|created| {
                        serde_json::to_value(created)
                            .map_err(|error| SessionGatewayError::internal(error.to_string()))
                    })
            })
        },
        "setGroupMembers" => required_string(args, "id").and_then(|group_id| {
            let member_ids = required_string_array(args, "memberIds")?;
            GroupChatGlue::new(Arc::clone(session))
                .set_group_members(group_id, &member_ids)
                .map_err(map_group_chat_error)
                .and_then(|summary| {
                    serde_json::to_value(summary)
                        .map_err(|error| SessionGatewayError::internal(error.to_string()))
                })
        }),
        "createAgent" => {
            parse_create_agent_profile(args).and_then(|profile| {
                let origin = optional_string(args, "origin")?.unwrap_or("user");
                let purpose = optional_string(args, "purpose")?;
                let introduction_suppressed =
                    optional_bool(args, "isIntroductionSuppressed")?.unwrap_or(false);
                let record = store
                    .create_session(Some(&profile), origin, purpose)
                    .map_err(SessionGatewayError::internal)?;
                if introduction_suppressed {
                    session
                        .set_agent_introduction_pending(&record.id, false)
                        .map_err(SessionGatewayError::internal)?;
                }
                session
                    .mark_agent_viewed(&record.id, system_now_ms(), false)
                    .map_err(SessionGatewayError::internal)?;
                store
                    .write_active_agent_id(&record.id)
                    .map_err(|error| SessionGatewayError::internal(error.to_string()))?;
                let summary = store
                    .summarize_agent_by_id(&record.id)
                    .map_err(SessionGatewayError::internal)?
                    .ok_or_else(|| {
                        SessionGatewayError::internal(
                            "failed to summarize newly created agent",
                        )
                    })?;
                let transcript = session
                    .read_agent_transcript_entries(&record.id)
                    .map_err(SessionGatewayError::internal)?;
                serde_json::to_value(summary)
                    .map_err(|error| SessionGatewayError::internal(error.to_string()))
                    .map(|agent| json!({ "agent": agent, "transcript": transcript }))
            })
        },
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
        "openAgentTail" | "getAgentTranscriptTail" => required_string(args, "id").and_then(|agent_id| {
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
        "getConversationOutline" => required_string(args, "id").and_then(|agent_id| {
            session
                .read_agent_outline(agent_id)
                .and_then(|outline| serde_json::to_value(outline).map_err(|error| error.to_string()))
                .map_err(SessionGatewayError::internal)
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

fn parse_create_agent_profile(args: &Value) -> Result<SandAgentProfile, SessionGatewayError> {
    let name = required_string(args, "name")?;
    let description = optional_string(args, "description")?.unwrap_or_default();
    Ok(SandAgentProfile {
        name: name.to_string(),
        description: description.to_string(),
        title: optional_string(args, "title")?.unwrap_or_default().to_string(),
        avatar_shape: optional_string(args, "avatarShape")?
            .unwrap_or_default()
            .to_string(),
        avatar_color: optional_string(args, "avatarColor")?
            .unwrap_or_default()
            .to_string(),
    })
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

fn map_group_chat_error(error: GroupChatGlueError) -> SessionGatewayError {
    match error {
        GroupChatGlueError::BadRequest(message) => SessionGatewayError::bad(message),
        GroupChatGlueError::Internal(message) => SessionGatewayError::internal(message),
    }
}

fn required_string_array(
    args: &Value,
    field: &str,
) -> Result<Vec<String>, SessionGatewayError> {
    let values = args
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| SessionGatewayError::bad(format!("missing or invalid {field}")))?;
    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| SessionGatewayError::bad(format!("{field} must contain only strings")))
        })
        .collect()
}

fn optional_string<'a>(
    args: &'a Value,
    field: &str,
) -> Result<Option<&'a str>, SessionGatewayError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.as_str())),
        Some(_) => Err(SessionGatewayError::bad(format!("invalid {field}"))),
    }
}

fn optional_bool(args: &Value, field: &str) -> Result<Option<bool>, SessionGatewayError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(value)) => Ok(Some(*value)),
        Some(_) => Err(SessionGatewayError::bad(format!("invalid {field}"))),
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
