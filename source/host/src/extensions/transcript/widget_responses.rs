use std::sync::Arc;

use serde_json::Value;

use crate::extensions::local_tool_permission::local_tool_permission_resolution::{
    LocalToolPermissionWidgetResponses, StaleLocalToolPermissionCardSettlement,
};
use crate::extensions::session::production::ProductionSessionWorkers;

#[derive(Clone)]
pub struct WidgetResponses {
    workers: Arc<ProductionSessionWorkers>,
}

impl WidgetResponses {
    pub fn new(workers: Arc<ProductionSessionWorkers>) -> Self {
        Self { workers }
    }

    pub fn settle_stale_local_tool_permission_card(
        &self,
        agent_id: &str,
        entry_id: &str,
        request_id: &str,
    ) -> Result<StaleLocalToolPermissionCardSettlement, String> {
        let expired = self.workers.expire_pending_local_tool_permission_asks(
            agent_id,
            Some(request_id),
            None,
        )?;
        if !expired.is_empty() {
            return Ok(StaleLocalToolPermissionCardSettlement::Retired);
        }

        let settled = self
            .workers
            .read_agent_transcript_entries(agent_id)?
            .into_iter()
            .any(|entry| is_local_tool_permission_entry(&entry, entry_id, request_id));
        Ok(if settled {
            StaleLocalToolPermissionCardSettlement::Settled
        } else {
            StaleLocalToolPermissionCardSettlement::NotSettled
        })
    }

    pub fn expire_all_pending_local_tool_permission_cards(
        &self,
        if_pending_before_ms: Option<u64>,
    ) -> usize {
        let Ok(agent_ids) = self.workers.list_agent_record_ids() else {
            return 0;
        };
        let mut expired_count = 0usize;
        for agent_id in agent_ids {
            let expired = self.workers.expire_pending_local_tool_permission_asks(
                &agent_id,
                None,
                if_pending_before_ms.map(|value| value as f64),
            );
            if let Ok(expired) = expired {
                expired_count = expired_count.saturating_add(expired.len());
            }
        }
        expired_count
    }
}

impl LocalToolPermissionWidgetResponses for WidgetResponses {
    fn settle_stale_local_tool_permission_card(
        &self,
        agent_id: &str,
        entry_id: &str,
        request_id: &str,
    ) -> Result<StaleLocalToolPermissionCardSettlement, String> {
        WidgetResponses::settle_stale_local_tool_permission_card(
            self,
            agent_id,
            entry_id,
            request_id,
        )
    }
}

fn is_local_tool_permission_entry(
    entry: &Value,
    entry_id: &str,
    request_id: &str,
) -> bool {
    entry.get("id").and_then(Value::as_str) == Some(entry_id)
        && entry.get("kind").and_then(Value::as_str) == Some("send-message")
        && entry
            .get("message")
            .and_then(|message| message.get("type"))
            .and_then(Value::as_str)
            == Some("local-tool-permission")
        && entry
            .get("message")
            .and_then(|message| message.get("ask"))
            .and_then(|ask| ask.get("requestId"))
            .and_then(Value::as_str)
            == Some(request_id)
}
