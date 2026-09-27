use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use serde_json::{Value, json};

use crate::agents::agent_profile::SandAgentProfile;
use crate::extensions::session::production::ProductionSessionWorkers;
use crate::groups::group_store::{
    GROUP_CONFIG_VERSION, RemoteGroupMember, SandGroupConfig, read_sand_group_config,
    write_sand_group_config,
};
use crate::groups::remote_room_store::{
    REMOTE_ROOM_CONFIG_VERSION, RemoteRoomMember, SandRemoteRoomConfig,
    read_sand_remote_room_config, write_sand_remote_room_config,
};

pub fn decode_avatar_data_url(data_url: &str) -> Option<Vec<u8>> {
    const MARKER: &str = ";base64,";
    if !data_url.starts_with("data:image/") {
        return None;
    }
    let index = data_url.find(MARKER)?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&data_url[index + MARKER.len()..])
        .ok()?;
    (!bytes.is_empty()).then_some(bytes)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomEntryKind {
    Message,
    SendMessage,
    Other,
}

pub fn is_room_content_entry(kind: RoomEntryKind) -> bool {
    matches!(kind, RoomEntryKind::Message | RoomEntryKind::SendMessage)
}

pub fn normalize_group_message(text: &str, max_chars: usize) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed == "(pass)" {
        return None;
    }
    Some(trimmed.chars().take(max_chars).collect())
}

pub fn can_post_to_group(member_ids: &[String], from_agent_id: &str) -> bool {
    member_ids.iter().any(|id| id == from_agent_id)
}

/// Production owner for the frozen Grok shared-room/session boundary.
///
/// CrossUserSharing is deliberately not allowed to open Session databases
/// directly. It asks this Transcript-owned service to materialize room agents,
/// append room-local transcript state, and restamp entries after relay
/// settlement. This keeps Session/Transcript ownership in the Host and avoids a
/// second sharing runtime.
pub struct SharedRooms {
    sessions: Arc<ProductionSessionWorkers>,
}

impl SharedRooms {
    pub fn new(sessions: Arc<ProductionSessionWorkers>) -> Self {
        Self { sessions }
    }

    pub fn sessions(&self) -> Arc<ProductionSessionWorkers> {
        Arc::clone(&self.sessions)
    }

    pub fn get_shared_room_id_for_agent(&self, agent_id: &str) -> Option<String> {
        let dir = self.sessions.agents_root().join(agent_id);
        read_sand_group_config(&dir)
            .and_then(|config| config.shared_room_id)
            .or_else(|| read_sand_remote_room_config(&dir).map(|config| config.room_id))
    }

    pub fn list_bound_room_agents(&self, room_id: &str) -> Result<Vec<String>, String> {
        let mut bound = Vec::new();
        for agent_id in self.sessions.list_agent_record_ids()? {
            if self.get_shared_room_id_for_agent(&agent_id).as_deref() == Some(room_id) {
                bound.push(agent_id);
            }
        }
        Ok(bound)
    }

    pub fn resolve_canonical_room_agent(&self, room_id: &str) -> Result<Option<String>, String> {
        let bound = self.list_bound_room_agents(room_id)?;
        if bound.len() <= 1 {
            return Ok(bound.into_iter().next());
        }

        // Frozen Grok chooses the room agent with the most transcript content.
        // Do not auto-delete non-canonical duplicates here: deletion is a
        // lifecycle operation and remains owned by Transcript/agent lifecycle.
        let mut measured = Vec::with_capacity(bound.len());
        for agent_id in bound {
            let count = self
                .sessions
                .read_agent_transcript_entries(&agent_id)
                .map(|entries| entries.len())
                .unwrap_or_default();
            measured.push((agent_id, count));
        }
        measured.sort_by(|left, right| {
            right
                .1
                .cmp(&left.1)
                .then_with(|| left.0.cmp(&right.0))
        });
        Ok(measured.into_iter().next().map(|(agent_id, _)| agent_id))
    }

    pub fn ensure_hosted_shared_room(
        &self,
        room_id: &str,
        name: &str,
        local_member_ids: &[String],
        remote_members: &[RemoteGroupMember],
        is_creation_allowed: bool,
    ) -> Result<Option<String>, String> {
        if let Some(existing_id) = self.resolve_canonical_room_agent(room_id)? {
            let dir = self.sessions.agents_root().join(&existing_id);
            let previous = read_sand_group_config(&dir);
            let binding = SandGroupConfig {
                version: GROUP_CONFIG_VERSION,
                member_ids: self.filter_local_member_ids(local_member_ids),
                remote_members: merge_remote_member_avatars(
                    previous.as_ref().and_then(|value| value.remote_members.as_deref()),
                    remote_members,
                ),
                shared_room_id: Some(room_id.to_string()),
            };
            write_sand_group_config(&dir, &binding).map_err(|error| error.to_string())?;
            return Ok(Some(existing_id));
        }
        if !is_creation_allowed {
            return Ok(None);
        }

        let profile = SandAgentProfile {
            name: if name.trim().is_empty() {
                "Shared room".to_string()
            } else {
                name.trim().to_string()
            },
            description: String::new(),
            title: String::new(),
            avatar_shape: String::new(),
            avatar_color: String::new(),
        };
        let created = self
            .sessions
            .materialize_new_session(Some(&profile), "user", Some("shared-room-host"))?;
        let binding = SandGroupConfig {
            version: GROUP_CONFIG_VERSION,
            member_ids: self.filter_local_member_ids(local_member_ids),
            remote_members: (!remote_members.is_empty()).then(|| remote_members.to_vec()),
            shared_room_id: Some(room_id.to_string()),
        };
        write_sand_group_config(
            self.sessions.agents_root().join(&created.id),
            &binding,
        )
        .map_err(|error| error.to_string())?;
        Ok(Some(created.id))
    }

    pub fn ensure_mirror_room(
        &self,
        room_id: &str,
        name: &str,
        host_auth_id: &str,
        host_name: &str,
        host_avatar_url: Option<&str>,
        members: &[RemoteRoomMember],
        self_auth_id: &str,
    ) -> Result<String, String> {
        let existing_id = self.resolve_canonical_room_agent(room_id)?;
        let agent_id = match existing_id {
            Some(agent_id) => agent_id,
            None => {
                let profile = SandAgentProfile {
                    name: if name.trim().is_empty() {
                        "Shared room".to_string()
                    } else {
                        name.trim().to_string()
                    },
                    description: String::new(),
                    title: String::new(),
                    avatar_shape: String::new(),
                    avatar_color: String::new(),
                };
                self.sessions
                    .materialize_new_session(Some(&profile), "user", Some("shared-room-mirror"))?
                    .id
            }
        };
        let dir = self.sessions.agents_root().join(&agent_id);
        let previous = read_sand_remote_room_config(&dir);
        let filtered = members
            .iter()
            .filter(|member| member.auth_id != self_auth_id)
            .cloned()
            .collect::<Vec<_>>();
        let config = SandRemoteRoomConfig {
            version: REMOTE_ROOM_CONFIG_VERSION,
            room_id: room_id.to_string(),
            host_auth_id: host_auth_id.to_string(),
            host_name: if host_name.trim().is_empty() {
                "The host".to_string()
            } else {
                host_name.trim().to_string()
            },
            host_avatar_url: host_avatar_url
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .or_else(|| previous.and_then(|value| value.host_avatar_url)),
            members: filtered,
            is_revoked: None,
        };
        write_sand_remote_room_config(&dir, &config).map_err(|error| error.to_string())?;
        Ok(agent_id)
    }

    pub fn mark_mirror_room_revoked(&self, room_id: &str) -> Result<(), String> {
        let Some(agent_id) = self.resolve_canonical_room_agent(room_id)? else {
            return Ok(());
        };
        let dir = self.sessions.agents_root().join(agent_id);
        let Some(mut config) = read_sand_remote_room_config(&dir) else {
            return Ok(());
        };
        if config.is_revoked == Some(true) {
            return Ok(());
        }
        config.is_revoked = Some(true);
        write_sand_remote_room_config(&dir, &config).map_err(|error| error.to_string())
    }

    pub fn append_shared_room_activity_notice(
        &self,
        room_id: &str,
        text: &str,
    ) -> Result<bool, String> {
        let Some(agent_id) = self.resolve_canonical_room_agent(room_id)? else {
            return Ok(false);
        };
        let entry = json!({
            "kind": "notice",
            "id": format!("notice-xuser-turn-{}", uuid::Uuid::new_v4()),
            "text": text,
            "timestampMs": now_ms(),
        });
        self.sessions
            .append_agent_transcript_entries(&agent_id, &[entry])?;
        let _ = self.sessions.mark_agent_activity(&agent_id, now_ms());
        Ok(true)
    }

    pub fn restamp_room_entry(
        &self,
        room_id: &str,
        entry_id: &str,
        timestamp_ms: f64,
    ) -> Result<bool, String> {
        let Some(agent_id) = self.resolve_canonical_room_agent(room_id)? else {
            return Ok(false);
        };
        let Some(mut entry) = self
            .sessions
            .read_agent_transcript_entries(&agent_id)?
            .into_iter()
            .find(|entry| entry.get("id").and_then(Value::as_str) == Some(entry_id))
        else {
            return Ok(false);
        };
        if !matches!(
            entry.get("kind").and_then(Value::as_str),
            Some("message" | "send-message")
        ) {
            return Ok(false);
        }
        let Some(object) = entry.as_object_mut() else {
            return Ok(false);
        };
        object.insert("timestampMs".into(), json!(timestamp_ms));
        Ok(self
            .sessions
            .update_agent_transcript_entry(&agent_id, entry_id, &entry)?
            .is_some())
    }

    fn filter_local_member_ids(&self, requested: &[String]) -> Vec<String> {
        requested
            .iter()
            .filter_map(|id| {
                let id = id.trim();
                if id.is_empty() {
                    return None;
                }
                let summary = self.sessions.summarize_agent_by_id(id, None).ok().flatten()?;
                (!summary.is_group).then(|| summary.id)
            })
            .collect()
    }
}

fn merge_remote_member_avatars(
    previous: Option<&[RemoteGroupMember]>,
    next: &[RemoteGroupMember],
) -> Option<Vec<RemoteGroupMember>> {
    if next.is_empty() {
        return None;
    }
    let previous = previous.unwrap_or_default();
    Some(
        next.iter()
            .map(|member| {
                if member.avatar_data_url.is_some() {
                    return member.clone();
                }
                let mut merged = member.clone();
                merged.avatar_data_url = previous
                    .iter()
                    .find(|candidate| {
                        candidate.owner_auth_id == member.owner_auth_id
                            && candidate.agent_id == member.agent_id
                    })
                    .and_then(|candidate| candidate.avatar_data_url.clone());
                merged
            })
            .collect(),
    )
}

fn now_ms() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
        * 1000.0
}
