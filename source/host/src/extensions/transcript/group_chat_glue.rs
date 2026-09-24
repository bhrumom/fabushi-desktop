use std::collections::HashSet;
use std::sync::Arc;

use serde::Serialize;
use serde_json::Value;

use crate::agents::agent_profile::SandAgentProfile;
use crate::groups::group_chat::{
    assert_members_are_not_groups, is_same_member_set,
};
use crate::groups::group_store::{
    GROUP_CONFIG_VERSION, GROUP_MAX_MEMBERS, SandGroupConfig,
    read_sand_group_config, write_sand_group_config,
};
use crate::extensions::session::agent_session::SandAgentSessionStore;
use crate::extensions::session::production::ProductionSessionWorkers;
use crate::extensions::session::session_summaries::AgentSummary;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GroupChatGlueError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Internal(String),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupCreateResult {
    pub agent: AgentSummary,
    pub transcript: Vec<Value>,
}

pub struct GroupChatGlue {
    session: Arc<ProductionSessionWorkers>,
}

impl GroupChatGlue {
    pub fn new(session: Arc<ProductionSessionWorkers>) -> Self {
        Self { session }
    }

    pub fn create_group(
        &self,
        name: &str,
        description: &str,
        member_ids: &[String],
    ) -> Result<GroupCreateResult, GroupChatGlueError> {
        let store = SandAgentSessionStore::new(Arc::clone(&self.session));
        let all_agents = store.list_agents().map_err(GroupChatGlueError::Internal)?;
        let existing = all_agents
            .iter()
            .map(|agent| agent.id.clone())
            .collect::<HashSet<_>>();
        let group_ids = all_agents
            .iter()
            .filter(|agent| agent.is_group)
            .map(|agent| agent.id.clone())
            .collect::<HashSet<_>>();
        let requested = stable_unique(member_ids);
        assert_members_are_not_groups(&requested, |id| group_ids.contains(id))
            .map_err(|error| GroupChatGlueError::BadRequest(error.to_string()))?;
        let cleaned = requested
            .into_iter()
            .filter(|id| existing.contains(id))
            .take(GROUP_MAX_MEMBERS)
            .collect::<Vec<_>>();
        if cleaned.is_empty() {
            return Err(GroupChatGlueError::BadRequest(
                "A group needs at least one existing member agent.".into(),
            ));
        }

        if let Some(duplicate) = all_agents
            .iter()
            .find(|agent| agent.is_group && is_same_member_set(&agent.member_ids, &cleaned))
        {
            store
                .write_active_agent_id(&duplicate.id)
                .map_err(|error| GroupChatGlueError::Internal(error.to_string()))?;
            let agent = store
                .summarize_agent_by_id(&duplicate.id)
                .map_err(GroupChatGlueError::Internal)?
                .unwrap_or_else(|| duplicate.clone());
            let transcript = self
                .session
                .read_agent_transcript_entries(&duplicate.id)
                .map_err(GroupChatGlueError::Internal)?;
            return Ok(GroupCreateResult { agent, transcript });
        }

        let profile = SandAgentProfile {
            name: name.to_string(),
            description: description.to_string(),
            title: String::new(),
            avatar_shape: String::new(),
            avatar_color: String::new(),
        };
        let record = store
            .create_session(Some(&profile), "user", None)
            .map_err(GroupChatGlueError::Internal)?;
        write_sand_group_config(
            store.get_agent_dir(&record.id),
            &SandGroupConfig {
                version: GROUP_CONFIG_VERSION,
                member_ids: cleaned,
                remote_members: None,
                shared_room_id: None,
            },
        )
        .map_err(|error| GroupChatGlueError::Internal(error.to_string()))?;
        store
            .write_active_agent_id(&record.id)
            .map_err(|error| GroupChatGlueError::Internal(error.to_string()))?;
        let agent = store
            .summarize_agent_by_id(&record.id)
            .map_err(GroupChatGlueError::Internal)?
            .ok_or_else(|| {
                GroupChatGlueError::Internal("failed to summarize newly created group".into())
            })?;
        let transcript = self
            .session
            .read_agent_transcript_entries(&record.id)
            .map_err(GroupChatGlueError::Internal)?;
        Ok(GroupCreateResult { agent, transcript })
    }

    pub fn set_group_members(
        &self,
        group_id: &str,
        member_ids: &[String],
    ) -> Result<Option<AgentSummary>, GroupChatGlueError> {
        let store = SandAgentSessionStore::new(Arc::clone(&self.session));
        let dir = store.get_agent_dir(group_id);
        let Some(current) = read_sand_group_config(&dir) else {
            return Ok(None);
        };
        if current.shared_room_id.is_some() {
            return store
                .summarize_agent_by_id(group_id)
                .map_err(GroupChatGlueError::Internal);
        }

        let all_agents = store.list_agents().map_err(GroupChatGlueError::Internal)?;
        let existing = all_agents
            .iter()
            .map(|agent| agent.id.clone())
            .collect::<HashSet<_>>();
        let group_ids = all_agents
            .iter()
            .filter(|agent| agent.is_group)
            .map(|agent| agent.id.clone())
            .collect::<HashSet<_>>();
        let requested = stable_unique(member_ids);
        assert_members_are_not_groups(&requested, |id| group_ids.contains(id))
            .map_err(|error| GroupChatGlueError::BadRequest(error.to_string()))?;
        let cleaned = requested
            .into_iter()
            .filter(|id| id != group_id && existing.contains(id))
            .take(GROUP_MAX_MEMBERS)
            .collect::<Vec<_>>();
        if !cleaned.is_empty() {
            write_sand_group_config(
                &dir,
                &SandGroupConfig {
                    version: GROUP_CONFIG_VERSION,
                    member_ids: cleaned,
                    remote_members: None,
                    shared_room_id: None,
                },
            )
            .map_err(|error| GroupChatGlueError::Internal(error.to_string()))?;
        }
        store
            .summarize_agent_by_id(group_id)
            .map_err(GroupChatGlueError::Internal)
    }

    pub fn is_group_agent_id(&self, agent_id: &str) -> bool {
        let store = SandAgentSessionStore::new(Arc::clone(&self.session));
        read_sand_group_config(store.get_agent_dir(agent_id)).is_some()
    }
}

fn stable_unique(values: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .filter(|value| seen.insert((*value).to_string()))
        .map(ToOwned::to_owned)
        .collect()
}
