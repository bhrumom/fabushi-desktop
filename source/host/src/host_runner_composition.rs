use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use crate::extensions::local_tool_permission::local_tool_permission_controller::{
    SandLocalToolControllerEvent, SandLocalToolControllerEventKind,
    SandLocalToolControllerSubscription, SandLocalToolPermissionController,
    SandLocalToolRequestStatus,
};
use crate::extensions::session::production::ProductionSessionWorkers;

type PermissionEventSink = Arc<dyn Fn(&SandLocalToolControllerEvent) + Send + Sync>;

pub struct HostRunnerComposition {
    controller: Arc<SandLocalToolPermissionController>,
    sink: PermissionEventSink,
    surfaces: Mutex<HashMap<String, SandLocalToolControllerSubscription>>,
}

impl HostRunnerComposition {
    pub fn production(
        controller: Arc<SandLocalToolPermissionController>,
        sessions: Arc<ProductionSessionWorkers>,
    ) -> Self {
        let sink = Arc::new(move |event: &SandLocalToolControllerEvent| {
            if let Err(error) = persist_local_permission_event(&sessions, event) {
                eprintln!(
                    "mahayana-host local permission transcript projection failed agent={} request={} error={error}",
                    event.request.agent_id,
                    event.request.id,
                );
            }
        });
        Self::with_sink(controller, sink)
    }

    pub fn with_sink(
        controller: Arc<SandLocalToolPermissionController>,
        sink: PermissionEventSink,
    ) -> Self {
        Self {
            controller,
            sink,
            surfaces: Mutex::new(HashMap::new()),
        }
    }

    pub fn bind_local_permission_surface(&self, agent_id: &str) {
        self.unbind_local_permission_surface(agent_id);
        let wanted = agent_id.to_string();
        let sink = Arc::clone(&self.sink);
        let subscription = self.controller.subscribe(Arc::new(move |event| {
            if event.request.agent_id == wanted {
                sink(&event);
            }
        }));
        self.surfaces
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(agent_id.to_string(), subscription);
    }

    pub fn unbind_local_permission_surface(&self, agent_id: &str) {
        self.surfaces
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(agent_id);
    }

    pub fn can_ask_local_tool_permission(&self, agent_id: &str) -> bool {
        self.surfaces
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .contains_key(agent_id)
    }

    pub fn forget_local_tool_permission(&self, agent_id: &str) {
        self.unbind_local_permission_surface(agent_id);
        self.controller.forget_agent(agent_id);
    }

    pub fn active_permission_surface_count(&self) -> usize {
        self.surfaces
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }
}

fn persist_local_permission_event(
    sessions: &ProductionSessionWorkers,
    event: &SandLocalToolControllerEvent,
) -> Result<(), String> {
    match event.kind {
        SandLocalToolControllerEventKind::Created => {
            let mut ask = json!({
                "requestId": event.request.id,
                "action": event.request.action,
                "target": event.request.target,
                "status": "pending",
            });
            if let Some(description) = event.request.description.as_deref() {
                ask["description"] = Value::String(description.to_string());
            }
            let entry = json!({
                "id": event.request.id,
                "kind": "send-message",
                "message": {
                    "type": "local-tool-permission",
                    "ask": ask,
                },
                "timestampMs": now_ms(),
            });
            sessions
                .append_agent_transcript_entries(&event.request.agent_id, &[entry])
                .map(|_| ())
        }
        SandLocalToolControllerEventKind::Settled => {
            let entry_id = event.request.id.as_str();
            let Some(mut entry) = sessions
                .read_agent_transcript_entries(&event.request.agent_id)?
                .into_iter()
                .find(|entry| entry.get("id").and_then(Value::as_str) == Some(entry_id))
            else {
                return Ok(());
            };
            if let Some(status) = entry
                .get_mut("message")
                .and_then(Value::as_object_mut)
                .and_then(|message| message.get_mut("ask"))
                .and_then(Value::as_object_mut)
            {
                status.insert(
                    "status".into(),
                    Value::String(permission_status(event.request.status).to_string()),
                );
            }
            sessions
                .update_agent_transcript_entry(&event.request.agent_id, entry_id, &entry)
                .map(|_| ())
        }
    }
}

fn permission_status(status: SandLocalToolRequestStatus) -> &'static str {
    match status {
        SandLocalToolRequestStatus::Pending => "pending",
        SandLocalToolRequestStatus::Allowed => "allowed",
        SandLocalToolRequestStatus::Denied => "denied",
        SandLocalToolRequestStatus::Always => "always",
        SandLocalToolRequestStatus::Never => "never",
        SandLocalToolRequestStatus::Expired => "expired",
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}
