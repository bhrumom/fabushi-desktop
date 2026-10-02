use serde_json::{Value, json};

use crate::extensions::session::agent_db_serde::AwaitingUserResponse;
use crate::extensions::session::production::ProductionSessionWorkers;
use crate::extensions::transcript::roster_emit::ProductionRosterEmit;

use super::box_request_entries::resolve_box_request_entry;

pub const BOX_HANDOFF_RESUME_PROMPT: &str =
    "[The user handed the box back to you. Please continue your task — start with the read-only Screenshot tool to see the current state of the box desktop.]";
pub const BOX_HANDOFF_DISMISSED_PROMPT: &str =
    "[The user dismissed your box help request without doing the step you asked for. Treat it as declined: do not assume the step happened, and do not immediately request the box again for the same step. Continue the task without it if you can — skip the step or find another way. If the task cannot proceed without it, send the user a brief message saying what is blocked, then stop and wait for their reply.]";
pub const BOX_HANDOFF_VIEWER_CLOSED_PROMPT: &str =
    "[The user closed the box desktop viewer without explicitly handing control back, so they may or may not have finished the step you asked for. Start with the read-only Screenshot tool to check the current state of the box desktop. If the step is clearly done, continue the task. If you can't tell, send the user a brief message asking whether they finished so you can keep going.]";

pub const MCP_AUTH_RESUME_TITLE: &str = "Agent failed to resume after MCP authorization";
pub const LISTENER_CONNECT_RESUME_TITLE: &str = "Agent failed to resume after listener connection";

pub fn mcp_auth_resume_prompt(display_name: &str) -> String {
    format!(
        "[The \"{display_name}\" MCP server finished authorizing — it's connected and its tools are available now. Your first action is a SendMessage telling the user it's connected, then pick up whatever you paused to authorize it. If there was nothing else to do, just confirm it's ready and ask what they'd like to do with it. Remember: nothing reaches the user unless it's inside a SendMessage.]"
    )
}

pub fn listener_connect_resume_prompt(platform: &str) -> String {
    let display_name = if platform.eq_ignore_ascii_case("slack") {
        "Slack"
    } else {
        "GitHub"
    };
    let reminder = if platform.eq_ignore_ascii_case("slack") {
        " For a channel listener, also remind them the Cursor bot must be in the channel (/invite @Cursor) or messages there can't reach it."
    } else {
        ""
    };
    format!(
        "[{display_name} is now connected to the user's Cursor account — {display_name} listener routines can fire. Your first action is a SendMessage telling the user it's connected, then pick up whatever you paused (e.g. finish or re-check the listener routine you were setting up).{reminder} Remember: nothing reaches the user unless it's inside a SendMessage.]"
    )
}

pub fn build_hidden_handoff_resume_send_args(
    agent_id: &str,
    prompt: &str,
    nonce_kind: &str,
    now_ms: u64,
) -> Value {
    json!({
        "agentId": agent_id,
        "prompt": prompt,
        "appendUserMessage": false,
        "awaitTurn": false,
        "directAddressedAcceptance": true,
        "requestSource": "handoff-resume",
        "hidden": true,
        "skipAckObligation": true,
        "clientNonce": format!("{nonce_kind}:{agent_id}:{now_ms}"),
    })
}

pub struct AwaitingStateSink<'a> {
    sessions: &'a ProductionSessionWorkers,
    roster: &'a ProductionRosterEmit,
}

impl<'a> AwaitingStateSink<'a> {
    pub fn new(
        sessions: &'a ProductionSessionWorkers,
        roster: &'a ProductionRosterEmit,
    ) -> Self {
        Self { sessions, roster }
    }

    fn agent_exists(&self, agent_id: &str) -> bool {
        self.sessions
            .summarize_agent_by_id(agent_id, None)
            .ok()
            .flatten()
            .is_some()
    }

    pub fn set(&self, agent_id: &str, state: &AwaitingUserResponse) -> bool {
        if !self.agent_exists(agent_id) {
            return false;
        }
        let Ok(applied) = self.sessions.set_agent_awaiting_user_response(agent_id, Some(state))
        else {
            return false;
        };
        if applied {
            let _ = self.roster.emit_agent_update(agent_id);
        }
        applied
    }

    pub fn clear(&self, agent_id: &str) -> bool {
        if !self.agent_exists(agent_id) {
            return false;
        }
        let Ok(applied) = self.sessions.set_agent_awaiting_user_response(agent_id, None) else {
            return false;
        };
        if applied {
            let _ = self.roster.emit_agent_update(agent_id);
        }
        applied
    }

    pub fn try_set_for_tab(
        &self,
        agent_id: &str,
        tab_id: &str,
        state: &AwaitingUserResponse,
    ) -> bool {
        if !self.agent_exists(agent_id) {
            return false;
        }
        let Ok(applied) = self.sessions.set_agent_awaiting_user_response_for_tab(
            agent_id,
            tab_id,
            Some(state),
            None,
        ) else {
            return false;
        };
        if applied {
            let _ = self.roster.emit_agent_update(agent_id);
        }
        applied
    }

    pub fn clear_for_tab(
        &self,
        agent_id: &str,
        tab_id: &str,
        if_since_before: Option<f64>,
    ) -> bool {
        if !self.agent_exists(agent_id) {
            return false;
        }
        let Ok(applied) = self.sessions.set_agent_awaiting_user_response_for_tab(
            agent_id,
            tab_id,
            None,
            if_since_before,
        ) else {
            return false;
        };
        if applied {
            let _ = self.roster.emit_agent_update(agent_id);
        }
        applied
    }
}

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
    build_hidden_handoff_resume_send_args(
        agent_id,
        box_handoff_resume_prompt(trigger),
        "box-handoff-resume",
        now_ms,
    )
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

pub fn settle_box_handoff_state_with_sink(
    sessions: &ProductionSessionWorkers,
    roster: &ProductionRosterEmit,
    agent_id: &str,
    request_id: &str,
    resolution: &str,
) -> Result<BoxHandoffSettlement, String> {
    let awaiting_cleared = AwaitingStateSink::new(sessions, roster).clear(agent_id);
    let resolved_entry =
        resolve_box_request_entry(sessions, agent_id, request_id, resolution)?;
    Ok(BoxHandoffSettlement {
        awaiting_cleared,
        resolved_entry,
    })
}
