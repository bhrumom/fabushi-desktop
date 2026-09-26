use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::json;

use crate::agents::agent_messaging::{
    AgentAddress, AgentMessageImage, build_agent_inbound_wake_prompt, clamp_agent_message,
};
use crate::extensions::session::production::ProductionSessionWorkers;
use super::transcript_entry_ids::{TranscriptEntryIdKind, next_entry_id};

pub const PRIORITY_AGENT_MESSAGE_INTERRUPT_REASON: &str = "superseded by a priority agent message";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInboundMessage {
    pub from: AgentAddressProjection,
    pub text: String,
    pub timestamp_ms: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub images: Vec<AgentMessageImage>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub priority: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_displayed: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_redriven: bool,
}

fn is_false(value: &bool) -> bool { !*value }

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentAddressProjection { pub id: String, pub name: String }

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentWakeRequest {
    pub agent_id: String,
    pub source_agent_id: String,
    pub prompt: String,
    pub priority: bool,
    pub member_ids: Vec<String>,
}

pub type AgentWakeSink = Arc<dyn Fn(&AgentWakeRequest) + Send + Sync + 'static>;
pub type PriorityInterruptSink = Arc<dyn Fn(&str, &str) -> usize + Send + Sync + 'static>;

pub fn partition_agent_inbound<T: Clone>(messages: &[T], is_priority: impl Fn(&T) -> bool) -> (Vec<T>, Vec<T>) {
    let mut priority = Vec::new();
    let mut rest = Vec::new();
    for message in messages {
        if is_priority(message) { priority.push(message.clone()); } else { rest.push(message.clone()); }
    }
    (priority, rest)
}

pub fn prioritize_agent_inbound(messages: &[AgentInboundMessage]) -> Vec<AgentInboundMessage> {
    let (priority, rest) = partition_agent_inbound(messages, |m| m.priority);
    priority.into_iter().chain(rest).collect()
}

pub fn merge_agent_inbound_queue(queued: &[AgentInboundMessage], deferred: &[AgentInboundMessage]) -> Vec<AgentInboundMessage> {
    let (newer_priority, newer_rest) = partition_agent_inbound(queued, |m| m.priority);
    let (older_priority, older_rest) = partition_agent_inbound(deferred, |m| m.priority);
    newer_priority.into_iter().chain(older_priority).chain(older_rest).chain(newer_rest).collect()
}

pub struct ProductionAgentToAgentMessaging {
    sessions: Arc<ProductionSessionWorkers>,
    wake_sink: AgentWakeSink,
    priority_interrupt: Option<PriorityInterruptSink>,
}

impl ProductionAgentToAgentMessaging {
    pub fn new(sessions: Arc<ProductionSessionWorkers>, wake_sink: AgentWakeSink, priority_interrupt: Option<PriorityInterruptSink>) -> Self {
        Self { sessions, wake_sink, priority_interrupt }
    }

    pub fn send_to_agent(&self, from_agent_id: &str, to_agent_id: &str, text: &str, images: &[AgentMessageImage], priority: bool) -> Result<String, String> {
        let message = clamp_agent_message(text);
        if message.is_empty() { return Ok("Message was empty; nothing was sent.".into()); }
        if to_agent_id == from_agent_id { return Ok("An agent can't message itself.".into()); }
        let roster = self.sessions.list_agent_summaries(None)?;
        let Some(target) = roster.iter().find(|agent| agent.id == to_agent_id) else { return Ok(format!("No agent found with id {to_agent_id}.")); };
        if target.remote_room.is_some() { return Ok("That is a shared chat hosted by another user; agents can't message it directly.".into()); }
        let sender = roster.iter().find(|agent| agent.id == from_agent_id);
        let sender_name = sender.map(|a| a.name.clone()).unwrap_or_else(|| "An agent".into());
        let timestamp_ms = now_ms();

        if target.is_group {
            let entries = self.sessions.read_agent_transcript_entries(to_agent_id)?;
            let entry_id = next_entry_id(&entries, TranscriptEntryIdKind::AssistantMessage);
            self.sessions.append_agent_transcript_entries(to_agent_id, &[json!({
                "kind":"message","id":entry_id,"role":"assistant","content":message,"isStreaming":false,
                "timestampMs":timestamp_ms,"fromAgent":{"id":from_agent_id,"name":sender_name},
            })])?;
            let _ = self.sessions.mark_agent_activity(to_agent_id, timestamp_ms as f64);
            (self.wake_sink)(&AgentWakeRequest {
                agent_id:to_agent_id.into(), source_agent_id:from_agent_id.into(), prompt:message.clone(),
                priority:false, member_ids:target.member_ids.clone(),
            });
            let mut notes = Vec::new();
            if !images.is_empty() {
                notes.push(format!("Note: the attached image{} {} NOT delivered — group messages are text-only for now; send images to an agent directly.",
                    if images.len()==1 {""} else {"s"}, if images.len()==1 {"was"} else {"were"}));
            }
            if priority { notes.push("Note: priority is 1:1 only — this post did not interrupt members.".into()); }
            let ack=format!("Posted to {}.",target.name);
            return Ok(if notes.is_empty(){ack}else{format!("{ack} {}",notes.join(" "))});
        }

        let _=self.sessions.add_agent_conversation_partner(from_agent_id,to_agent_id)?;
        let _=self.sessions.add_agent_conversation_partner(to_agent_id,from_agent_id)?;
        let source_entries=self.sessions.read_agent_transcript_entries(from_agent_id)?;
        let target_entries=self.sessions.read_agent_transcript_entries(to_agent_id)?;
        let outbound_id=next_entry_id(&source_entries,TranscriptEntryIdKind::AssistantMessage);
        let inbound_id=next_entry_id(&target_entries,TranscriptEntryIdKind::UserMessage);
        let image_json=serde_json::to_value(images).map_err(|e|format!("could not encode agent message images: {e}"))?;
        self.sessions.append_agent_transcript_entries(from_agent_id,&[json!({
            "kind":"message","id":outbound_id,"role":"assistant","content":message,"isStreaming":false,
            "timestampMs":timestamp_ms,"toAgent":{"id":to_agent_id,"name":target.name,"kind":"agent"},"images":image_json.clone()
        })])?;
        self.sessions.append_agent_transcript_entries(to_agent_id,&[json!({
            "kind":"message","id":inbound_id,"role":"user","content":message,"isStreaming":false,
            "timestampMs":timestamp_ms,"fromAgent":{"id":from_agent_id,"name":sender_name},"images":image_json
        })])?;
        let _=self.sessions.mark_agent_activity(to_agent_id,timestamp_ms as f64);
        if priority {
            if let Some(interrupt)=self.priority_interrupt.as_ref(){ let _=interrupt(to_agent_id,PRIORITY_AGENT_MESSAGE_INTERRUPT_REASON); }
        }
        let from=AgentAddress{
            id:from_agent_id.into(), name:sender_name,
            description:sender.and_then(|a|{let v=a.description.trim();(!v.is_empty()).then(||v.to_string())}),
            is_group:false,
        };
        (self.wake_sink)(&AgentWakeRequest{
            agent_id:to_agent_id.into(),source_agent_id:from_agent_id.into(),
            prompt:build_agent_inbound_wake_prompt(&from,&message,images,priority),priority,member_ids:Vec::new(),
        });
        Ok(if priority {
            format!("Sent to {} as a priority message — it will interrupt their current non-user work and wake them now. This is asynchronous; if they reply, it'll arrive later as a new message.",target.name)
        } else {
            format!("Sent to {}. This is asynchronous; if they reply, it'll arrive later as a new message.",target.name)
        })
    }
}

fn now_ms()->u64{
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis().min(u64::MAX as u128) as u64
}
