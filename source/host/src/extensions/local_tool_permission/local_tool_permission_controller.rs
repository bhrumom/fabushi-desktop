use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use uuid::Uuid;

use crate::extensions::settings::settings_service::{
    SandLocalToolPermission, SettingsService,
};

use super::local_tool_permission_resolution::{
    LocalToolPermissionAskStore, PendingLocalToolPermissionRequest,
    SandLocalToolResolution,
};

pub const SAND_LOCAL_TOOL_SETTLED_ID_MEMORY: usize = 64;
pub const SAND_LOCAL_TOOL_TARGET_MAX_CHARS: usize = 10_000;
pub const SAND_LOCAL_TOOL_ASK_TTL_MS: u64 = 10 * 60 * 1_000;

pub const SAND_LOCAL_TOOLS_DISABLED_MESSAGE: &str =
    "Local tools are turned off. The user has set local tool access to \"Never\", so ExternalShell, ExternalRead, AwaitExternalShell, CopyToBox, and CopyFromBox cannot run on their computer. Do not retry them while this setting remains \"Never\". Use your own computer instead (Shell, Read, AwaitShell), or ask the user to change the setting in Settings → Agent → Execution on Local Computer. If they change it away from \"Never\", you may try again.";
pub const SAND_LOCAL_TOOLS_DENIED_MESSAGE: &str =
    "The user declined this action on their computer. Do not retry it. Do something else, use your own computer instead (Shell, Read, AwaitShell), or ask them what they would prefer.";
pub const SAND_LOCAL_TOOLS_ASK_EXPIRED_MESSAGE: &str =
    "The request to run this on the user's computer went unanswered, so nothing ran. Use your own computer instead (Shell, Read, AwaitShell), or tell the user you are waiting on their approval.";
pub const SAND_LOCAL_TOOLS_ASK_UNAVAILABLE_MESSAGE: &str =
    "Using the user's computer needs their permission, and this conversation has nowhere to ask for it. Use your own computer instead (Shell, Read, AwaitShell), or do this from a direct chat with the user.";
pub const SAND_LOCAL_TOOLS_UNAPPROVED_MESSAGE: &str =
    "That action was not approved on the user's computer, so nothing ran. Ask the user to approve it (or to set Settings → Agent → Execution on Local Computer to \"Always allow\"), and use your own computer (Shell, Read, AwaitShell) in the meantime.";
pub const SAND_LOCAL_TOOLS_TARGET_TOO_LARGE_MESSAGE: &str =
    "That action is too long to show the user for approval, so it was not run. Split it into smaller steps, or use your own computer instead (Shell, Read, AwaitShell).";
pub const SAND_LOCAL_TOOLS_ASK_CANCELLED_MESSAGE: &str =
    "The request to use the user's computer was cancelled before the user answered.";
pub const SAND_NO_LOCAL_MACHINE_MESSAGE: &str =
    "Your local machine isn't connected right now (the Grok Bot desktop app must be open and online to run commands on it). Try again once it's reachable.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandLocalToolScope {
    pub agent_id: String,
    pub tool_call_id: Option<String>,
    pub action: Option<String>,
    pub direction_epoch: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandLocalToolRequest {
    pub action: String,
    pub target: String,
    pub resource_path: Option<String>,
    pub attach_to_resource_path: Option<String>,
    pub outlives_scope: bool,
    pub description: Option<String>,
}

impl SandLocalToolRequest {
    pub fn simple(action: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            action: action.into(),
            target: target.into(),
            resource_path: None,
            attach_to_resource_path: None,
            outlives_scope: false,
            description: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandLocalToolDecision {
    pub allowed: bool,
    pub reason: Option<String>,
    pub approval_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandLocalToolRequestStatus {
    Pending,
    Allowed,
    Denied,
    Always,
    Never,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandLocalToolAskRequest {
    pub id: String,
    pub agent_id: String,
    pub action: String,
    pub target: String,
    pub status: SandLocalToolRequestStatus,
    pub created_at_ms: u64,
    pub expires_at_ms: u64,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
struct StoredApproval {
    id: String,
    agent_id: String,
    tool_call_id: String,
    action: String,
    target: String,
    resource_path: Option<String>,
    outlives_scope: bool,
}

struct PendingAsk {
    request: Mutex<SandLocalToolAskRequest>,
    decision: Mutex<Option<SandLocalToolDecision>>,
    wake: Condvar,
    tool_call_id: String,
    resource_path: Option<String>,
    outlives_scope: bool,
}

#[derive(Default)]
struct ControllerState {
    pending_by_key: HashMap<String, Arc<PendingAsk>>,
    pending_by_id: HashMap<String, String>,
    approvals_by_id: HashMap<String, StoredApproval>,
    direction_epochs: HashMap<String, u64>,
    settled_ids: HashSet<String>,
    settled_order: VecDeque<String>,
}

type AgentPredicate = Arc<dyn Fn(&str) -> bool + Send + Sync>;
type EventSink = Arc<dyn Fn(SandLocalToolAskRequest) + Send + Sync>;

pub struct SandLocalToolPermissionController {
    settings: Arc<SettingsService>,
    state: Mutex<ControllerState>,
    can_ask: Mutex<AgentPredicate>,
    has_live_computer: Mutex<AgentPredicate>,
    event_sink: Mutex<Option<EventSink>>,
    ask_ttl_ms: u64,
    now_ms: Arc<dyn Fn() -> u64 + Send + Sync>,
    random_id: Arc<dyn Fn() -> String + Send + Sync>,
}

impl SandLocalToolPermissionController {
    pub fn production(settings: Arc<SettingsService>) -> Self {
        Self::with_options(
            settings,
            SAND_LOCAL_TOOL_ASK_TTL_MS,
            Arc::new(|| {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
                    .try_into()
                    .unwrap_or(u64::MAX)
            }),
            Arc::new(|| Uuid::new_v4().simple().to_string()),
        )
    }

    pub fn with_options(
        settings: Arc<SettingsService>,
        ask_ttl_ms: u64,
        now_ms: Arc<dyn Fn() -> u64 + Send + Sync>,
        random_id: Arc<dyn Fn() -> String + Send + Sync>,
    ) -> Self {
        Self {
            settings,
            state: Mutex::new(ControllerState::default()),
            can_ask: Mutex::new(Arc::new(|_| false)),
            has_live_computer: Mutex::new(Arc::new(|_| false)),
            event_sink: Mutex::new(None),
            ask_ttl_ms,
            now_ms,
            random_id,
        }
    }

    pub fn bind_ask_surfaces(&self, provider: AgentPredicate) {
        *self.can_ask.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = provider;
    }

    pub fn bind_live_computer_check(&self, provider: AgentPredicate) {
        *self
            .has_live_computer
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = provider;
    }

    pub fn bind_event_sink(&self, sink: Option<EventSink>) {
        *self.event_sink.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = sink;
    }

    pub fn permission(&self) -> SandLocalToolPermission {
        self.settings.get_local_tool_permission()
    }

    pub fn blocked_reason(&self) -> Option<String> {
        (self.permission() == SandLocalToolPermission::Never)
            .then(|| SAND_LOCAL_TOOLS_DISABLED_MESSAGE.to_string())
    }

    pub fn requires_approval(&self) -> bool {
        self.permission() == SandLocalToolPermission::Ask
    }

    pub fn direction_epoch(&self, agent_id: &str) -> u64 {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .direction_epochs
            .get(agent_id)
            .copied()
            .unwrap_or(0)
    }

    pub fn begin_turn(&self, agent_id: &str) {
        let pending = {
            let mut state = self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            let epoch = state.direction_epochs.entry(agent_id.to_string()).or_insert(0);
            *epoch = epoch.saturating_add(1);
            let approvals = state
                .approvals_by_id
                .iter()
                .filter_map(|(id, approval)| (approval.agent_id == agent_id).then_some(id.clone()))
                .collect::<Vec<_>>();
            for id in approvals {
                state.approvals_by_id.remove(&id);
            }
            state
                .pending_by_key
                .values()
                .filter(|pending| {
                    pending
                        .request
                        .lock()
                        .map(|request| request.agent_id == agent_id)
                        .unwrap_or(false)
                })
                .cloned()
                .collect::<Vec<_>>()
        };
        for pending in pending {
            let id = pending
                .request
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .id
                .clone();
            self.settle_pending(
                &id,
                SandLocalToolRequestStatus::Expired,
                SandLocalToolDecision {
                    allowed: false,
                    reason: Some(SAND_LOCAL_TOOLS_ASK_EXPIRED_MESSAGE.to_string()),
                    approval_id: None,
                },
            );
        }
    }

    pub fn forget_agent(&self, agent_id: &str) {
        self.begin_turn(agent_id);
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .direction_epochs
            .remove(agent_id);
    }

    pub fn complete_scope(&self, scope: Option<&SandLocalToolScope>) {
        let Some(scope) = scope else { return };
        let Some(tool_call_id) = scope.tool_call_id.as_deref() else { return };
        let mut state = self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        state.approvals_by_id.retain(|_, approval| {
            approval.agent_id != scope.agent_id
                || approval.tool_call_id != tool_call_id
                || approval.outlives_scope
        });
    }

    pub fn authorize(
        &self,
        scope: Option<&SandLocalToolScope>,
        request: &SandLocalToolRequest,
    ) -> SandLocalToolDecision {
        match self.permission() {
            SandLocalToolPermission::Never => {
                return denied(SAND_LOCAL_TOOLS_DISABLED_MESSAGE);
            }
            SandLocalToolPermission::Always => {
                return allowed(None);
            }
            SandLocalToolPermission::Ask => {}
        }

        let Some(scope) = scope else {
            return denied(SAND_LOCAL_TOOLS_UNAPPROVED_MESSAGE);
        };

        if request.target.chars().count() > SAND_LOCAL_TOOL_TARGET_MAX_CHARS {
            return denied(SAND_LOCAL_TOOLS_TARGET_TOO_LARGE_MESSAGE);
        }

        if let Some(decision) = self.covering_approval(scope, request) {
            return decision;
        }

        let Some(tool_call_id) = scope.tool_call_id.as_deref() else {
            return denied(SAND_LOCAL_TOOLS_UNAPPROVED_MESSAGE);
        };

        let can_ask = self
            .can_ask
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        if !can_ask(&scope.agent_id) {
            return denied(SAND_LOCAL_TOOLS_ASK_UNAVAILABLE_MESSAGE);
        }
        let has_live = self
            .has_live_computer
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        if !has_live(&scope.agent_id) {
            return denied(SAND_NO_LOCAL_MACHINE_MESSAGE);
        }

        let key = format!(
            "{}\0{}\0{}\0{}",
            scope.agent_id, tool_call_id, request.action, request.target
        );
        let pending = {
            let mut state = self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some(existing) = state.pending_by_key.get(&key) {
                Arc::clone(existing)
            } else {
                let now = (self.now_ms)();
                let id = (self.random_id)();
                let pending = Arc::new(PendingAsk {
                    request: Mutex::new(SandLocalToolAskRequest {
                        id: id.clone(),
                        agent_id: scope.agent_id.clone(),
                        action: request.action.clone(),
                        target: request.target.clone(),
                        status: SandLocalToolRequestStatus::Pending,
                        created_at_ms: now,
                        expires_at_ms: now.saturating_add(self.ask_ttl_ms),
                        description: request
                            .description
                            .as_deref()
                            .map(str::trim)
                            .filter(|value| !value.is_empty())
                            .map(str::to_string),
                    }),
                    decision: Mutex::new(None),
                    wake: Condvar::new(),
                    tool_call_id: tool_call_id.to_string(),
                    resource_path: normalize_resource_path(request.resource_path.as_deref()),
                    outlives_scope: request.outlives_scope,
                });
                state.pending_by_id.insert(id, key.clone());
                state.pending_by_key.insert(key.clone(), Arc::clone(&pending));
                if let Some(sink) = self
                    .event_sink
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .clone()
                {
                    if let Ok(request) = pending.request.lock() {
                        sink(request.clone());
                    }
                }
                pending
            }
        };

        let decision = pending
            .decision
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let (decision, wait) = pending
            .wake
            .wait_timeout_while(
                decision,
                Duration::from_millis(self.ask_ttl_ms),
                |decision| decision.is_none(),
            )
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(decision) = decision.clone() {
            return decision;
        }
        drop(decision);
        if wait.timed_out() {
            let id = pending
                .request
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .id
                .clone();
            let decision = denied(SAND_LOCAL_TOOLS_ASK_EXPIRED_MESSAGE);
            self.settle_pending(&id, SandLocalToolRequestStatus::Expired, decision.clone());
            return decision;
        }
        denied(SAND_LOCAL_TOOLS_ASK_CANCELLED_MESSAGE)
    }

    pub fn get_pending_request_for_agent(&self, agent_id: &str) -> Option<SandLocalToolAskRequest> {
        let pending = {
            let state = self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            state
                .pending_by_key
                .values()
                .find(|pending| {
                    pending
                        .request
                        .lock()
                        .map(|request| request.agent_id == agent_id)
                        .unwrap_or(false)
                })
                .cloned()
        }?;
        let request = pending
            .request
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        Some(request)
    }

    pub fn get_pending_request_by_id_full(&self, id: &str) -> Option<SandLocalToolAskRequest> {
        let pending = {
            let state = self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            let key = state.pending_by_id.get(id)?;
            state.pending_by_key.get(key).cloned()
        }?;
        let request = pending
            .request
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        Some(request)
    }

    pub fn was_settled_id(&self, id: &str) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .settled_ids
            .contains(id)
    }

    pub fn resolve_request_full(
        &self,
        request_id: &str,
        resolution: SandLocalToolResolution,
    ) -> Option<SandLocalToolAskRequest> {
        let pending = {
            let state = self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            let key = state.pending_by_id.get(request_id)?;
            state.pending_by_key.get(key).cloned()
        }?;

        if matches!(resolution, SandLocalToolResolution::Always) {
            let _ = self
                .settings
                .set_local_tool_permission(SandLocalToolPermission::Always);
        } else if matches!(resolution, SandLocalToolResolution::Never) {
            let _ = self
                .settings
                .set_local_tool_permission(SandLocalToolPermission::Never);
        }

        let (status, decision) = match resolution {
            SandLocalToolResolution::AllowOnce => (
                SandLocalToolRequestStatus::Allowed,
                allowed(Some(request_id.to_string())),
            ),
            SandLocalToolResolution::Deny => (
                SandLocalToolRequestStatus::Denied,
                denied(SAND_LOCAL_TOOLS_DENIED_MESSAGE),
            ),
            SandLocalToolResolution::Always => (
                SandLocalToolRequestStatus::Always,
                allowed(Some(request_id.to_string())),
            ),
            SandLocalToolResolution::Never => (
                SandLocalToolRequestStatus::Never,
                denied(SAND_LOCAL_TOOLS_DISABLED_MESSAGE),
            ),
        };

        if decision.allowed {
            let request = pending
                .request
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clone();
            self.state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .approvals_by_id
                .insert(
                    request_id.to_string(),
                    StoredApproval {
                        id: request_id.to_string(),
                        agent_id: request.agent_id,
                        tool_call_id: pending.tool_call_id.clone(),
                        action: request.action,
                        target: request.target,
                        resource_path: pending.resource_path.clone(),
                        outlives_scope: pending.outlives_scope,
                    },
                );
        }

        self.settle_pending(request_id, status, decision)
    }

    pub fn retire_approval(&self, id: &str) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .approvals_by_id
            .remove(id)
            .is_some()
    }

    pub fn live_approval_ids(&self) -> Vec<String> {
        let mut ids = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .approvals_by_id
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        ids.sort();
        ids
    }

    fn covering_approval(
        &self,
        scope: &SandLocalToolScope,
        request: &SandLocalToolRequest,
    ) -> Option<SandLocalToolDecision> {
        let wanted_resource = normalize_resource_path(request.attach_to_resource_path.as_deref());
        let mut state = self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let approval = state.approvals_by_id.values_mut().find(|approval| {
            approval.agent_id == scope.agent_id
                && (scope.tool_call_id.as_deref() == Some(approval.tool_call_id.as_str())
                    || approval.resource_path.is_some())
                && ((approval.action == request.action && approval.target == request.target)
                    || (approval.resource_path.is_some()
                        && approval.resource_path == wanted_resource))
        })?;
        if request.outlives_scope {
            approval.outlives_scope = true;
        }
        Some(allowed(Some(approval.id.clone())))
    }

    fn settle_pending(
        &self,
        request_id: &str,
        status: SandLocalToolRequestStatus,
        decision: SandLocalToolDecision,
    ) -> Option<SandLocalToolAskRequest> {
        let pending = {
            let mut state = self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            let key = state.pending_by_id.remove(request_id)?;
            let pending = state.pending_by_key.remove(&key)?;
            state.settled_ids.insert(request_id.to_string());
            state.settled_order.push_back(request_id.to_string());
            while state.settled_order.len() > SAND_LOCAL_TOOL_SETTLED_ID_MEMORY {
                if let Some(oldest) = state.settled_order.pop_front() {
                    state.settled_ids.remove(&oldest);
                }
            }
            pending
        };
        let settled = {
            let mut request = pending
                .request
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            request.status = status;
            request.clone()
        };
        *pending
            .decision
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(decision);
        pending.wake.notify_all();
        if let Some(sink) = self
            .event_sink
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
        {
            sink(settled.clone());
        }
        Some(settled)
    }
}

impl LocalToolPermissionAskStore for SandLocalToolPermissionController {
    fn get_pending_request_by_id(
        &self,
        request_id: &str,
    ) -> Option<PendingLocalToolPermissionRequest> {
        self.get_pending_request_by_id_full(request_id)
            .map(|request| PendingLocalToolPermissionRequest {
                agent_id: request.agent_id,
            })
    }

    fn was_settled(&self, request_id: &str) -> bool {
        self.was_settled_id(request_id)
    }

    fn resolve_request(
        &self,
        request_id: &str,
        resolution: SandLocalToolResolution,
    ) -> bool {
        self.resolve_request_full(request_id, resolution).is_some()
    }
}

fn normalize_resource_path(path: Option<&str>) -> Option<String> {
    path.map(|path| path.replace('\\', "/")).filter(|path| !path.is_empty())
}

fn allowed(approval_id: Option<String>) -> SandLocalToolDecision {
    SandLocalToolDecision {
        allowed: true,
        reason: None,
        approval_id,
    }
}

fn denied(reason: &str) -> SandLocalToolDecision {
    SandLocalToolDecision {
        allowed: false,
        reason: Some(reason.to_string()),
        approval_id: None,
    }
}
