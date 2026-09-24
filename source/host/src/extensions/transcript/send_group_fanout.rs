use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use futures::FutureExt;
use serde_json::{Value, json};

use crate::extensions::session::production::ProductionSessionWorkers;
use crate::groups::group_chat::{
    GroupDescription, GroupMember, GroupMessage, GroupSpeaker,
};
use crate::groups::group_store::{SandGroupConfig, read_sand_group_config};

use super::group_chat_orchestrator::{
    GroupChatOrchestrator, GroupMemberTurnRequest, GroupOrchestratorDeps,
};
use super::production_runtime::ProductionTranscriptRuntime;
use super::transcript_entry_ids::{TranscriptEntryIdKind, next_entry_id};

pub type GroupMemberTurnExecutor =
    Arc<dyn Fn(GroupMemberTurnRequest) -> Result<Vec<String>, String> + Send + Sync + 'static>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalGroupFanoutDisposition {
    NotGroup,
    DeferredRemote {
        shared_room_id: Option<String>,
        remote_member_count: usize,
    },
    Completed {
        posted_messages: usize,
        member_failures: Vec<(String, String)>,
    },
}

struct LocalGroupFanoutDeps {
    sessions: Arc<ProductionSessionWorkers>,
    runtime: Arc<ProductionTranscriptRuntime>,
    room_id: String,
    expected_epoch: u64,
    config: SandGroupConfig,
    executor: GroupMemberTurnExecutor,
    posted_messages: Arc<Mutex<usize>>,
    member_failures: Arc<Mutex<Vec<(String, String)>>>,
    post_error: Arc<Mutex<Option<String>>>,
}

impl LocalGroupFanoutDeps {
    fn read_history_from_room(&self) -> Vec<GroupMessage> {
        self.sessions
            .read_agent_transcript_entries(&self.room_id)
            .unwrap_or_default()
            .into_iter()
            .filter_map(transcript_entry_to_group_message)
            .collect()
    }

    fn append_member_message(&self, member: &GroupMember, content: &str) -> Result<(), String> {
        let entries = self.sessions.read_agent_transcript_entries(&self.room_id)?;
        let id = next_entry_id(&entries, TranscriptEntryIdKind::SendMessage);
        let entry = json!({
            "kind": "send-message",
            "id": id,
            "message": {
                "type": "text",
                "content": content,
            },
            "timestampMs": now_ms(),
            "author": {
                "id": member.id,
                "name": member.name,
            },
        });
        self.sessions
            .append_agent_transcript_entries(&self.room_id, &[entry])?;
        let _ = self.sessions.mark_agent_activity(&self.room_id, now_ms());
        if let Ok(mut posted) = self.posted_messages.lock() {
            *posted = posted.saturating_add(1);
        }
        Ok(())
    }
}

impl GroupOrchestratorDeps for LocalGroupFanoutDeps {
    fn resolve_members<'a>(
        &'a self,
        ids: &'a [String],
    ) -> futures::future::BoxFuture<'a, Vec<GroupMember>> {
        let mut members = Vec::new();
        for id in ids {
            let Ok(Some(summary)) = self.sessions.summarize_agent_by_id(id, None) else {
                continue;
            };
            if summary.is_group {
                continue;
            }
            members.push(GroupMember {
                id: summary.id,
                name: if summary.name.trim().is_empty() {
                    "Grok".to_string()
                } else {
                    summary.name
                },
                description: summary.description,
            });
        }
        futures::future::ready(members).boxed()
    }

    fn read_history(&self) -> Vec<GroupMessage> {
        self.read_history_from_room()
    }

    fn is_current(&self) -> bool {
        self.runtime.current_turn_epoch(&self.room_id) == self.expected_epoch
    }

    fn run_member_turn<'a>(
        &'a self,
        request: GroupMemberTurnRequest,
    ) -> futures::future::BoxFuture<'a, Vec<String>> {
        let member_id = request.member.id.clone();
        let output = match (self.executor)(request) {
            Ok(messages) => messages,
            Err(error) => {
                if let Ok(mut failures) = self.member_failures.lock() {
                    failures.push((member_id, error));
                }
                Vec::new()
            }
        };
        futures::future::ready(output).boxed()
    }

    fn post_member_message(&self, member: &GroupMember, content: &str) {
        if let Err(error) = self.append_member_message(member, content) {
            if let Ok(mut slot) = self.post_error.lock() {
                slot.get_or_insert(error);
            }
        }
    }

    fn finalize_member_turn(&self, _member: &GroupMember) {}

    fn is_shared_room(&self) -> bool {
        false
    }
}

pub fn dispatch_local_group_send(
    sessions: Arc<ProductionSessionWorkers>,
    runtime: Arc<ProductionTranscriptRuntime>,
    room_id: &str,
    expected_epoch: u64,
    executor: GroupMemberTurnExecutor,
) -> Result<LocalGroupFanoutDisposition, String> {
    let db_path = sessions.session_db_path(room_id)?;
    let agent_dir = db_path
        .parent()
        .ok_or_else(|| "group session database has no agent directory".to_string())?;
    let Some(config) = read_sand_group_config(agent_dir) else {
        return Ok(LocalGroupFanoutDisposition::NotGroup);
    };
    let remote_member_count = config
        .remote_members
        .as_ref()
        .map(Vec::len)
        .unwrap_or_default();
    if config.shared_room_id.is_some() || remote_member_count > 0 {
        return Ok(LocalGroupFanoutDisposition::DeferredRemote {
            shared_room_id: config.shared_room_id,
            remote_member_count,
        });
    }

    let profile = sessions
        .get_agent_profile_text(room_id)?
        .unwrap_or_else(|| crate::agents::agent_profile::SandAgentProfile {
            name: "Grok".into(),
            description: String::new(),
            title: String::new(),
            avatar_shape: String::new(),
            avatar_color: String::new(),
        });
    let group = GroupDescription {
        name: if profile.name.trim().is_empty() {
            "Grok".to_string()
        } else {
            profile.name
        },
        description: profile.description,
    };
    let posted_messages = Arc::new(Mutex::new(0usize));
    let member_failures = Arc::new(Mutex::new(Vec::new()));
    let post_error = Arc::new(Mutex::new(None));
    let deps = LocalGroupFanoutDeps {
        sessions,
        runtime,
        room_id: room_id.to_string(),
        expected_epoch,
        config: config.clone(),
        executor,
        posted_messages: Arc::clone(&posted_messages),
        member_failures: Arc::clone(&member_failures),
        post_error: Arc::clone(&post_error),
    };
    let member_ids = deps.config.member_ids.clone();
    futures::executor::block_on(
        GroupChatOrchestrator::new(deps).run(&group, &member_ids),
    );
    if let Some(error) = post_error.lock().ok().and_then(|mut slot| slot.take()) {
        return Err(error);
    }
    Ok(LocalGroupFanoutDisposition::Completed {
        posted_messages: posted_messages.lock().map(|value| *value).unwrap_or_default(),
        member_failures: member_failures.lock().map(|value| value.clone()).unwrap_or_default(),
    })
}

fn transcript_entry_to_group_message(entry: Value) -> Option<GroupMessage> {
    match entry.get("kind").and_then(Value::as_str) {
        Some("message")
            if entry.get("role").and_then(Value::as_str) == Some("user")
                && entry
                    .get("content")
                    .and_then(Value::as_str)
                    .is_some_and(|content| !content.trim().is_empty()) =>
        {
            Some(GroupMessage {
                speaker: GroupSpeaker::User {
                    name: entry
                        .get("fromUser")
                        .and_then(Value::as_object)
                        .and_then(|from_user| from_user.get("name"))
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                },
                content: entry.get("content")?.as_str()?.to_string(),
            })
        }
        Some("send-message")
            if entry
                .get("message")
                .and_then(Value::as_object)
                .and_then(|message| message.get("type"))
                .and_then(Value::as_str)
                == Some("text")
                && entry.get("streaming").and_then(Value::as_bool) != Some(true) =>
        {
            let author = entry.get("author")?.as_object()?;
            let id = author.get("id")?.as_str()?.to_string();
            let name = author.get("name")?.as_str()?.to_string();
            let content = entry
                .get("message")?
                .get("content")?
                .as_str()?
                .to_string();
            Some(GroupMessage {
                speaker: GroupSpeaker::Member { id, name },
                content,
            })
        }
        _ => None,
    }
}

fn now_ms() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
        * 1000.0
}
