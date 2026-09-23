use serde_json::{Value, json};

pub const GATEWAY_PREPARE_UPGRADE_PATH: &str = "/prepare-upgrade";

pub const GROK_GATEWAY_COMMANDS: &[&str] = &[
    "getTranscript", "getAgentTranscript", "getAgentTranscriptPage", "openAgentWindowed",
    "getAgentTranscriptWindow", "openAgentTail", "getAgentTranscriptTail", "getAgentThread",
    "sendPrompt", "promptAcceptanceStatus", "respondToWidget", "resolveAutoReviewApproval",
    "resolveLocalToolPermission", "dismissWidget", "submitSecret", "reactToMessage",
    "appendConnectorCard", "listAgents", "countAgents", "searchAgents", "searchMedia",
    "createAgent", "kickstartAgent", "requestDiskSaverAudit", "createGroup", "setGroupMembers",
    "updateAgent", "deleteAgent", "deleteAgents", "duplicateAgent", "setAgentUnread",
    "setAgentNotificationsEnabled", "setAgentNotifyOnUpdates", "setAgentHiddenFromSidebar",
    "openAgent", "setWindowFocused", "getAgentMemories", "deleteAgentMemory",
    "clearAgentMemories", "getAgentAutomations", "listAllAutomations", "isAgentNetworkEnabled",
    "isGlobalSearchEnabled", "isEgressTunnelAvailable", "getSharingState", "createRoomFromAgent",
    "createRoomInvite", "joinSharedRoom", "respondToRoomJoinRequest", "createSharedRoom",
    "addOwnAgentToSharedRoom", "removeOwnAgentFromSharedRoom", "setSharedRoomTyping",
    "leaveSharedRoom", "setAgentAutomationEnabled", "createAgentAutomation",
    "updateAgentAutomation", "deleteAgentAutomation", "runAgentAutomationNow",
    "broadcastToAgents", "getAgentWorkflows", "createAgentWorkflow", "updateAgentWorkflow",
    "setAgentWorkflowEnabled", "deleteAgentWorkflow", "runAgentWorkflowNow",
    "importAgentWorkflowText", "importAgentWorkflowUrl", "portAgentLocalSkills",
    "getConversationOutline", "skillsCatalog", "syncPluginSkills", "getPluginSyncStatus",
    "getSkillPublishTargets", "publishSkill", "resyncPublishedSkill", "unpublishSkill",
    "getAgentChannels", "connectChannel", "disconnectChannel", "refreshChannel",
    "getListenerIntegrations", "getListenerConnectUrl", "getSubagents", "getAsyncTasks",
    "setAgentAvatarBytes", "getAgentAvatar", "getForeverBoxStatus", "getCloudAgentInfo",
    "ensureForeverBox", "resetForeverBox", "updateForeverBox", "autoUpdateBoxNow",
    "snapshotBoxStoreNow", "getBoxStoreStatus", "clearBoxStoreNow", "updateHostNow",
    "getHostStatus", "setBoxMigrating", "prepareBoxForRecreate", "resumeBoxAfterRecreate",
    "handBackForeverBox", "startTeachRecording", "stopTeachRecording",
    "getTeachRecordingStatus", "getTrays", "dismissTray", "clearTrays", "uploadAttachment",
    "readAttachmentImage", "readAttachmentText", "readAttachmentChunk", "getHostSettings",
    "setHostSettings", "setBoxSecrets", "getBoxSecretsStatus", "completeMcpOAuth",
    "requestWebAuthnCeremony", "refreshMcp", "listRoutedMcpTools", "executeRoutedMcpTool",
    "listBoxMcpServers",
];

pub fn is_grok_gateway_command(method: &str) -> bool {
    GROK_GATEWAY_COMMANDS.contains(&method)
}

pub fn parse_command_args(body: &[u8]) -> Result<Value, serde_json::Error> {
    if body.is_empty() {
        Ok(json!({}))
    } else {
        serde_json::from_slice(body)
    }
}

pub fn slim_command_result(method: &str, mut value: Value) -> Value {
    fn strip_summary(summary: &mut Value) {
        if let Some(object) = summary.as_object_mut() {
            if object.contains_key("avatarDataUrl") {
                object.insert("avatarDataUrl".into(), Value::Null);
            }
        }
    }

    match method {
        "listAgents" => {
            if let Some(rows) = value.as_array_mut() {
                for row in rows {
                    strip_summary(row);
                }
            }
        }
        "updateAgent" | "setGroupMembers" | "setAgentAvatarBytes" => strip_summary(&mut value),
        "createAgent" | "createGroup" | "duplicateAgent" => {
            if let Some(agent) = value.get_mut("agent") {
                strip_summary(agent);
            }
        }
        _ => {}
    }
    value
}

pub fn slim_event(mut event: Value) -> Value {
    match event.get("channel").and_then(Value::as_str) {
        Some("agents") => {
            if let Some(rows) = event
                .get_mut("payload")
                .and_then(|payload| payload.get_mut("agents"))
                .and_then(Value::as_array_mut)
            {
                for row in rows {
                    if let Some(object) = row.as_object_mut() {
                        if object.contains_key("avatarDataUrl") {
                            object.insert("avatarDataUrl".into(), Value::Null);
                        }
                    }
                }
            }
        }
        Some("agent-upserted") => {
            if let Some(object) = event
                .get_mut("payload")
                .and_then(|payload| payload.get_mut("agent"))
                .and_then(Value::as_object_mut)
            {
                if object.contains_key("avatarDataUrl") {
                    object.insert("avatarDataUrl".into(), Value::Null);
                }
            }
        }
        _ => {}
    }
    event
}

