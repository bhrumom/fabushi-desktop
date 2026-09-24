use serde_json::{Value, json};

use crate::extensions::session::production::ProductionSessionWorkers;

use super::box_request_entries::resolve_box_request_entry;

pub const BOX_HANDOFF_RESUME_PROMPT: &str =
    "[The user handed the box back to you. Please continue your task — start with the read-only Screenshot tool to see the current state of the box desktop.]";
pub const BOX_HANDOFF_DISMISSED_PROMPT: &str =
    "[The user dismissed your box help request without doing the step you asked for. Treat it as declined: do not assume the step happened, and do not immediately request the box again for the same step. Continue the task without it if you can — skip the step or find another way. If the task cannot proceed without it, send the user a brief message saying what is blocked, then stop and wait for their reply.]";
pub const BOX_HANDOFF_VIEWER_CLOSED_PROMPT: &str =
    "[The user closed the box desktop viewer without explicitly handing control back, so they may or may not have finished the step you asked for. Start with the read-only Screenshot tool to check the current state of the box desktop. If the step is clearly done, continue the task. If you can't tell, send the user a brief message asking whether they finished so you can keep going.]";

#[derive(Debug, Clone, PartialEq)]
pub struct BoxHandoffSettlement {
    pub awaiting_cleared: bool,
    pub resolved_entry: Option<Value>,
}

pub fn box_handoff_resume_prompt(trigger: &str) -> &'static str {
    match trigger {
        "dismissed" => BOX_HANDOFF_DISMISSED_PROMPT,
        "viewer-closed" => BOX_HANDOFF_VIEWER_CLOSED_PROMPT,
        _ => BOX_HANDOFF_RESUME_PROMPT,
    }
}

pub fn build_box_handoff_resume_send_args(
    agent_id: &str,
    trigger: &str,
    now_ms: u64,
) -> Value {
    json!({
        "agentId": agent_id,
        "prompt": box_handoff_resume_prompt(trigger),
        "appendUserMessage": false,
        "awaitTurn": false,
        "directAddressedAcceptance": true,
        "requestSource": "handoff-resume",
        "hidden": true,
        "skipAckObligation": true,
        "clientNonce": format!("box-handoff-resume:{agent_id}:{now_ms}"),
    })
}

pub fn settle_box_handoff_state(
    sessions: &ProductionSessionWorkers,
    agent_id: &str,
    request_id: &str,
    resolution: &str,
) -> Result<BoxHandoffSettlement, String> {
    let awaiting_cleared = sessions
        .set_agent_awaiting_user_response(agent_id, None)
        .unwrap_or(false);
    let resolved_entry =
        resolve_box_request_entry(sessions, agent_id, request_id, resolution)?;
    Ok(BoxHandoffSettlement {
        awaiting_cleared,
        resolved_entry,
    })
}
