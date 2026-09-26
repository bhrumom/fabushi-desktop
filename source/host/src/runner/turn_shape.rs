use std::collections::HashSet;

use serde_json::Value;

use super::send_message_reminder_middleware::{
    CursorProviderOptions, MessageContent, MessageLike, MessagePart, ProviderOptions,
    SAND_SEND_MESSAGE_TOOL_NAME, is_injected_reminder_message,
};

pub const SAND_REACT_TO_MESSAGE_TOOL_NAME: &str = "ReactToMessage";

fn delivery_tool_names() -> [&'static str; 2] {
    [SAND_SEND_MESSAGE_TOOL_NAME, SAND_REACT_TO_MESSAGE_TOOL_NAME]
}

pub fn as_core_message(value: &Value) -> Option<MessageLike> {
    let object = value.as_object()?;
    let role = object.get("role")?.as_str()?.to_string();
    let content = match object.get("content")? {
        Value::String(text) => MessageContent::Text(text.clone()),
        Value::Array(parts) => MessageContent::Parts(
            parts
                .iter()
                .map(|part| {
                    let object = part.as_object();
                    MessagePart {
                        r#type: object
                            .and_then(|object| object.get("type"))
                            .and_then(Value::as_str)
                            .map(str::to_string),
                        text: object
                            .and_then(|object| object.get("text"))
                            .and_then(Value::as_str)
                            .map(str::to_string),
                        tool_name: object
                            .and_then(|object| object.get("toolName"))
                            .and_then(Value::as_str)
                            .map(str::to_string),
                        tool_call_id: object
                            .and_then(|object| object.get("toolCallId"))
                            .and_then(Value::as_str)
                            .map(str::to_string),
                        args: object.and_then(|object| object.get("args")).cloned(),
                    }
                })
                .collect(),
        ),
        _ => return None,
    };
    let cursor = object
        .get("providerOptions")
        .and_then(Value::as_object)
        .and_then(|provider| provider.get("cursor"))
        .and_then(Value::as_object)
        .map(|cursor| CursorProviderOptions {
            sand_send_message_reminder: cursor
                .get("sandSendMessageReminder")
                .and_then(Value::as_bool)
                == Some(true),
            sand_early_result_reminder: cursor
                .get("sandEarlyResultReminder")
                .and_then(Value::as_bool)
                == Some(true),
            sand_start_of_turn_ack_reminder: cursor
                .get("sandStartOfTurnAckReminder")
                .and_then(Value::as_bool)
                == Some(true),
            sand_disk_pressure_reminder: cursor
                .get("sandDiskPressureReminder")
                .and_then(Value::as_bool)
                == Some(true),
            sand_disk_pressure_reminder_episode_id: cursor
                .get("sandDiskPressureReminderEpisodeId")
                .and_then(Value::as_str)
                .map(str::to_string),
            high_level_tool_call_result: cursor.get("highLevelToolCallResult").cloned(),
        });
    Some(MessageLike {
        role,
        content,
        provider_options: cursor.map(|cursor| ProviderOptions {
            cursor: Some(cursor),
        }),
    })
}

pub fn tool_call_names(message: &MessageLike) -> Vec<String> {
    if message.role != "assistant" {
        return Vec::new();
    }
    let MessageContent::Parts(parts) = &message.content else {
        return Vec::new();
    };
    parts
        .iter()
        .filter(|part| part.r#type.as_deref() == Some("tool-call"))
        .filter_map(|part| part.tool_name.clone())
        .collect()
}

pub fn has_delivery_tool_call(names: &[String]) -> bool {
    names
        .iter()
        .any(|name| delivery_tool_names().contains(&name.as_str()))
}

fn delivery_tool_call_ids(message: &MessageLike) -> Vec<String> {
    if message.role != "assistant" {
        return Vec::new();
    }
    let MessageContent::Parts(parts) = &message.content else {
        return Vec::new();
    };
    parts
        .iter()
        .filter(|part| {
            part.r#type.as_deref() == Some("tool-call")
                && part
                    .tool_name
                    .as_deref()
                    .is_some_and(|name| delivery_tool_names().contains(&name))
        })
        .filter_map(|part| part.tool_call_id.clone())
        .collect()
}

fn errored_tool_result_ids(messages: &[Option<MessageLike>]) -> HashSet<String> {
    let mut ids = HashSet::new();
    for message in messages.iter().flatten() {
        if message.role != "tool" {
            continue;
        }
        let is_error = message
            .provider_options
            .as_ref()
            .and_then(|options| options.cursor.as_ref())
            .and_then(|cursor| cursor.high_level_tool_call_result.as_ref())
            .and_then(Value::as_object)
            .and_then(|value| value.get("isError"))
            .and_then(Value::as_bool)
            == Some(true);
        if !is_error {
            continue;
        }
        let MessageContent::Parts(parts) = &message.content else {
            continue;
        };
        for part in parts {
            if part.r#type.as_deref() == Some("tool-result") {
                if let Some(id) = &part.tool_call_id {
                    ids.insert(id.clone());
                }
            }
        }
    }
    ids
}

pub fn is_blank_assistant_message(message: &MessageLike) -> bool {
    if message.role != "assistant" {
        return false;
    }
    match &message.content {
        MessageContent::Text(text) => text.trim().is_empty(),
        MessageContent::Parts(parts) => parts.iter().all(|part| {
            part.r#type.as_deref() == Some("text")
                && part.text.as_deref().unwrap_or("").trim().is_empty()
        }),
    }
}

pub fn turn_ended_on_silent_tool_calls(raw_messages: &[Value]) -> bool {
    let messages = raw_messages
        .iter()
        .map(as_core_message)
        .collect::<Vec<_>>();
    let mut tail_index = messages.len();
    while tail_index > 0 {
        let index = tail_index - 1;
        let skip = messages[index].as_ref().is_none_or(|message| {
            message.role == "tool"
                || is_blank_assistant_message(message)
                || is_injected_reminder_message(message)
        });
        if !skip {
            break;
        }
        tail_index -= 1;
    }
    if tail_index == 0 {
        return false;
    }
    let tail_index = tail_index - 1;
    let Some(tail) = messages[tail_index].as_ref() else {
        return false;
    };
    if tail.role != "assistant" {
        return false;
    }
    let tail_names = tool_call_names(tail);
    if tail_names.is_empty() || has_delivery_tool_call(&tail_names) {
        return false;
    }

    let mut boundary: isize = -1;
    for index in (0..tail_index).rev() {
        let Some(message) = messages[index].as_ref() else {
            continue;
        };
        if is_injected_reminder_message(message) {
            continue;
        }
        if matches!(message.role.as_str(), "user" | "system") {
            boundary = index as isize;
            break;
        }
    }

    let errored_ids = errored_tool_result_ids(&messages);
    let mut acked_first = false;
    for index in ((boundary + 1) as usize)..=tail_index {
        let Some(message) = messages[index].as_ref() else {
            continue;
        };
        if message.role != "assistant" {
            continue;
        }
        let names = tool_call_names(message);
        if names.is_empty() {
            continue;
        }
        if !acked_first {
            if !has_delivery_tool_call(&names) {
                return false;
            }
            acked_first = true;
        } else if delivery_tool_call_ids(message)
            .iter()
            .any(|id| !errored_ids.contains(id))
        {
            return false;
        }
    }
    acked_first
}
