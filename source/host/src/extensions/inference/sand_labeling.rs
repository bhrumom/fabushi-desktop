use std::collections::{BTreeMap, BTreeSet};

pub const SAND_AGENT_MODE: &str = "sand-agent";
pub const SAND_SUMMARIZATION_REASON: &str = "agent-summarization";

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LabelMessage {
    pub inference_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FollowupClassification {
    pub request_id: String,
    pub replying_to_request_id: String,
    pub conversation_id: String,
    pub agent_mode: &'static str,
    pub model_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostTurnLabeling {
    pub request_id: String,
    pub conversation_id: String,
    pub agent_mode: &'static str,
    pub model_name: String,
}

#[derive(Debug, Default)]
pub struct FollowupLabelingTracker {
    last_request_id_by_conversation: BTreeMap<String, String>,
    recorded_request_ids: BTreeSet<String>,
}

impl FollowupLabelingTracker {
    pub fn observe_stream(
        &mut self,
        conversation_id: &str,
        request_id: &str,
        model_name: &str,
        messages: &[LabelMessage],
    ) -> Option<FollowupClassification> {
        if conversation_id.is_empty()
            || request_id.is_empty()
            || messages.is_empty()
            || messages.iter().any(|message| {
                message.inference_reason.as_deref() == Some(SAND_SUMMARIZATION_REASON)
            })
        {
            return None;
        }

        let previous_request_id = self
            .last_request_id_by_conversation
            .insert(conversation_id.to_string(), request_id.to_string());

        let previous_request_id = previous_request_id?;
        if previous_request_id.is_empty()
            || previous_request_id == request_id
            || self.recorded_request_ids.contains(request_id)
        {
            return None;
        }

        self.recorded_request_ids.insert(request_id.to_string());
        Some(FollowupClassification {
            request_id: request_id.to_string(),
            replying_to_request_id: previous_request_id,
            conversation_id: conversation_id.to_string(),
            agent_mode: SAND_AGENT_MODE,
            model_name: model_name.to_string(),
        })
    }

    pub fn last_request_id(&self, conversation_id: &str) -> Option<&str> {
        self.last_request_id_by_conversation
            .get(conversation_id)
            .map(String::as_str)
    }
}

pub fn prepare_post_turn_labeling(
    conversation_id: &str,
    request_id: &str,
    model_name: &str,
    messages: &[LabelMessage],
) -> Option<PostTurnLabeling> {
    if messages.is_empty() || request_id.trim().is_empty() || conversation_id.trim().is_empty() {
        return None;
    }

    Some(PostTurnLabeling {
        request_id: request_id.to_string(),
        conversation_id: conversation_id.to_string(),
        agent_mode: SAND_AGENT_MODE,
        model_name: model_name.to_string(),
    })
}
