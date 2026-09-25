use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

use serde_json::{Map, Value};

use crate::automations::automation::{AutomationRecord, AutomationSpec};
use crate::automations::automation_store::FileAutomationStore;
use crate::extensions::session::agent_session::{AgentAutomationEntry, SandAgentSessionStore};
use crate::extensions::session::production::ProductionSessionWorkers;

use super::automation_run_path::{
    AutomationExecutionResult, AutomationRunPath, FireAutomationArgs, FireAutomationOutcome,
};
use super::automation_snapshot::{
    AutomationAction as AutomationDiffAction, AutomationSnapshot, diff_automation_action,
    snapshot_automations,
};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AutomationCommandError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Internal(String),
}

pub fn dispatch_automation_command(
    runtime: &AutomationRuntime,
    method: &str,
    args: &Value,
) -> Option<Result<Value, AutomationCommandError>> {
    if !matches!(
        method,
        "getAgentAutomations"
            | "createAgentAutomation"
            | "updateAgentAutomation"
            | "setAgentAutomationEnabled"
            | "deleteAgentAutomation"
    ) {
        return None;
    }

    let agent_id = match required_automation_string(args, &["id", "agentId"], "id") {
        Ok(value) => value,
        Err(error) => return Some(Err(error)),
    };
    let result = match method {
        "getAgentAutomations" => runtime
            .get_agent_automations(agent_id)
            .map(automation_records_value)
            .map_err(AutomationCommandError::Internal),
        "createAgentAutomation" => {
            let spec = match automation_spec(args.get("spec").unwrap_or(args)) {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            runtime
                .create_agent_automation(agent_id, &spec)
                .map(|(records, _events)| automation_records_value(records))
                .map_err(AutomationCommandError::Internal)
        }
        "updateAgentAutomation" => {
            let automation_id = match required_automation_string(
                args,
                &["automationId"],
                "automationId",
            ) {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let spec = match automation_spec(args.get("spec").unwrap_or(args)) {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            runtime
                .update_agent_automation(agent_id, automation_id, &spec)
                .map(|(records, _events)| automation_records_value(records))
                .map_err(AutomationCommandError::Internal)
        }
        "setAgentAutomationEnabled" => {
            let automation_id = match required_automation_string(
                args,
                &["automationId"],
                "automationId",
            ) {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let is_enabled = match args.get("isEnabled").and_then(Value::as_bool) {
                Some(value) => value,
                None => {
                    return Some(Err(AutomationCommandError::BadRequest(
                        "setAgentAutomationEnabled requires boolean isEnabled".into(),
                    )));
                }
            };
            runtime
                .set_agent_automation_enabled(agent_id, automation_id, is_enabled)
                .map(|(records, _events)| automation_records_value(records))
                .map_err(AutomationCommandError::Internal)
        }
        "deleteAgentAutomation" => {
            let automation_id = match required_automation_string(
                args,
                &["automationId"],
                "automationId",
            ) {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            runtime
                .delete_agent_automation(agent_id, automation_id)
                .map(|(records, _events)| automation_records_value(records))
                .map_err(AutomationCommandError::Internal)
        }
        _ => unreachable!(),
    };
    Some(result)
}

fn required_automation_string<'a>(
    value: &'a Value,
    keys: &[&str],
    label: &str,
) -> Result<&'a str, AutomationCommandError> {
    keys.iter()
        .find_map(|key| value.get(*key))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            AutomationCommandError::BadRequest(format!(
                "automation command requires non-empty {label}"
            ))
        })
}

fn automation_spec(value: &Value) -> Result<AutomationSpec, AutomationCommandError> {
    let object = value.as_object().ok_or_else(|| {
        AutomationCommandError::BadRequest("automation spec must be an object".into())
    })?;
    Ok(AutomationSpec {
        name: object
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        prompt: object
            .get("prompt")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        trigger: object.get("trigger").cloned().unwrap_or(Value::Null),
        is_enabled: object.get("isEnabled").and_then(Value::as_bool),
    })
}

pub fn automation_records_value(records: Vec<AutomationRecord>) -> Value {
    Value::Array(records.into_iter().map(automation_record_value).collect())
}

pub fn automation_record_value(record: AutomationRecord) -> Value {
    let runs = record
        .runs
        .into_iter()
        .map(|run| {
            let mut value = Map::new();
            value.insert("id".into(), Value::String(run.id));
            value.insert("trigger".into(), Value::String(run.trigger));
            value.insert("startedAt".into(), number_value(run.started_at));
            if let Some(finished_at) = run.finished_at {
                value.insert("finishedAt".into(), number_value(finished_at));
            }
            value.insert("status".into(), Value::String(run.status));
            if let Some(detail) = run.detail {
                value.insert("detail".into(), Value::String(detail));
            }
            if let Some(event) = run.event {
                value.insert("event".into(), Value::String(event));
            }
            if let Some(ids) = run.coalesced_run_ids {
                value.insert(
                    "coalescedRunIds".into(),
                    Value::Array(ids.into_iter().map(Value::String).collect()),
                );
            }
            Value::Object(value)
        })
        .collect::<Vec<_>>();

    let mut value = Map::new();
    value.insert("id".into(), Value::String(record.id));
    value.insert("name".into(), Value::String(record.name));
    value.insert("prompt".into(), Value::String(record.prompt));
    value.insert("trigger".into(), record.trigger);
    value.insert(
        "isEnabled".into(),
        Value::Bool(record.is_enabled),
    );
    value.insert("createdAt".into(), number_value(record.created_at));
    if let Some(last_run_at) = record.last_run_at {
        value.insert("lastRunAt".into(), number_value(last_run_at));
    }
    value.insert(
        "raisedNotices".into(),
        Value::Array(record.raised_notices.into_iter().map(Value::String).collect()),
    );
    value.insert("schedule".into(), Value::String(record.schedule));
    value.insert(
        "triggerDescription".into(),
        Value::String(record.trigger_description),
    );
    if let Some(next_run_at) = record.next_run_at {
        value.insert("nextRunAt".into(), number_value(next_run_at));
    }
    value.insert("runs".into(), Value::Array(runs));
    value.insert(
        "filePath".into(),
        Value::String(record.file_path.to_string_lossy().into_owned()),
    );
    Value::Object(value)
}

fn number_value(value: f64) -> Value {
    serde_json::Number::from_f64(value)
        .map(Value::Number)
        .unwrap_or(Value::Null)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutomationLifecycleSource {
    Agent,
    AutomationsUi,
    WorkflowUi,
    SpendGuard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutomationLifecycleAction {
    Created,
    Updated,
    Enabled,
    Disabled,
    Deleted,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AutomationLifecycleEvent {
    pub agent_id: String,
    pub action: AutomationLifecycleAction,
    pub automation_id: String,
    pub automation_name: String,
    pub trigger_type: String,
    pub created_at: f64,
    pub recorded_run_count: usize,
    pub source: AutomationLifecycleSource,
}

#[derive(Clone)]
pub struct AutomationRuntime {
    sessions: Arc<ProductionSessionWorkers>,
    run_path: Arc<AutomationRunPath>,
    last_known: Arc<Mutex<HashMap<String, BTreeMap<String, AutomationSnapshot>>>>,
    mutation_locks: Arc<Mutex<HashMap<String, Arc<Mutex<()>>>>>,
}

impl AutomationRuntime {
    pub fn new(sessions: Arc<ProductionSessionWorkers>) -> Self {
        Self {
            sessions,
            run_path: Arc::new(AutomationRunPath::default()),
            last_known: Arc::new(Mutex::new(HashMap::new())),
            mutation_locks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn run_path(&self) -> Arc<AutomationRunPath> {
        Arc::clone(&self.run_path)
    }

    pub fn get_agent_automations(&self, agent_id: &str) -> Result<Vec<AutomationRecord>, String> {
        SandAgentSessionStore::new(Arc::clone(&self.sessions)).list_agent_automations(agent_id)
    }

    pub fn list_all_automations(&self) -> Result<Vec<AgentAutomationEntry>, String> {
        SandAgentSessionStore::new(Arc::clone(&self.sessions)).list_all_automations()
    }

    pub fn list_all_automation_definitions(&self) -> Result<Vec<AgentAutomationEntry>, String> {
        SandAgentSessionStore::new(Arc::clone(&self.sessions)).list_all_automation_definitions()
    }

    pub fn seed_known_automations(&self, agent_id: &str) -> Result<(), String> {
        self.with_agent_mutation_lock(agent_id, || {
            let definitions = self.automation_store(agent_id)?.list_definitions();
            self.last_known
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .entry(agent_id.to_string())
                .or_insert_with(|| snapshot_automations(&definitions));
            Ok(())
        })
    }

    pub fn create_agent_automation(
        &self,
        agent_id: &str,
        spec: &AutomationSpec,
    ) -> Result<(Vec<AutomationRecord>, Vec<AutomationLifecycleEvent>), String> {
        self.mutate_agent(
            agent_id,
            AutomationLifecycleSource::AutomationsUi,
            |store| {
                store
                    .upsert(spec, now_ms())
                    .map_err(|error| error.to_string())?;
                Ok(store.list())
            },
        )
    }

    pub fn update_agent_automation(
        &self,
        agent_id: &str,
        automation_id: &str,
        spec: &AutomationSpec,
    ) -> Result<(Vec<AutomationRecord>, Vec<AutomationLifecycleEvent>), String> {
        self.mutate_agent(
            agent_id,
            AutomationLifecycleSource::AutomationsUi,
            |store| {
                store
                    .update(automation_id, spec)
                    .map_err(|error| error.to_string())?;
                Ok(store.list())
            },
        )
    }

    pub fn set_agent_automation_enabled(
        &self,
        agent_id: &str,
        automation_id: &str,
        enabled: bool,
    ) -> Result<(Vec<AutomationRecord>, Vec<AutomationLifecycleEvent>), String> {
        self.mutate_agent(
            agent_id,
            AutomationLifecycleSource::AutomationsUi,
            |store| {
                store
                    .set_enabled(automation_id, enabled)
                    .map_err(|error| error.to_string())?;
                Ok(store.list())
            },
        )
    }

    pub fn delete_agent_automation(
        &self,
        agent_id: &str,
        automation_id: &str,
    ) -> Result<(Vec<AutomationRecord>, Vec<AutomationLifecycleEvent>), String> {
        self.mutate_agent(
            agent_id,
            AutomationLifecycleSource::AutomationsUi,
            |store| {
                store
                    .remove(automation_id)
                    .map_err(|error| error.to_string())?;
                Ok(store.list())
            },
        )
    }

    pub fn run_agent_automation_now_with<Execute>(
        &self,
        agent_id: &str,
        automation_id: &str,
        execute: Execute,
    ) -> Result<Option<FireAutomationOutcome>, String>
    where
        Execute: FnOnce(&str) -> Result<AutomationExecutionResult, String>,
    {
        let Some((store, automation, before)) = self.with_agent_mutation_lock(agent_id, || {
            let store = self.automation_store(agent_id)?;
            let Some(automation) = store.get(automation_id) else {
                return Ok(None);
            };
            let before = store.list_definitions();
            self.sync_baseline(agent_id, &before);
            Ok(Some((store, automation, before)))
        })? else {
            return Ok(None);
        };

        let outcome = self.run_path.fire_automation_with(
            &store,
            FireAutomationArgs::manual(agent_id, automation, now_ms()),
            execute,
        )?;

        self.with_agent_mutation_lock(agent_id, || {
            let after = store.list_definitions();
            let _ = self.record_changes(
                agent_id,
                &before,
                &after,
                AutomationLifecycleSource::Agent,
            );
            Ok(())
        })?;
        Ok(outcome)
    }

    pub fn with_workflow_ui_mutation<T, Mutation>(
        &self,
        agent_id: &str,
        mutation: Mutation,
    ) -> Result<(T, Vec<AutomationLifecycleEvent>), String>
    where
        Mutation: FnOnce(&SandAgentSessionStore) -> Result<T, String>,
    {
        self.with_agent_mutation_lock(agent_id, || {
            let session = SandAgentSessionStore::new(Arc::clone(&self.sessions));
            let before = session.automation_store_for(agent_id)?.list_definitions();
            self.sync_baseline(agent_id, &before);
            let result = mutation(&session)?;
            let after = session.automation_store_for(agent_id)?.list_definitions();
            let events = self.record_changes(
                agent_id,
                &before,
                &after,
                AutomationLifecycleSource::WorkflowUi,
            );
            Ok((result, events))
        })
    }

    pub fn record_external_changes(
        &self,
        agent_id: &str,
        before: &[AutomationRecord],
        after: &[AutomationRecord],
        source: AutomationLifecycleSource,
    ) -> Vec<AutomationLifecycleEvent> {
        self.record_changes(agent_id, before, after, source)
    }

    fn mutate_agent<T, Mutation>(
        &self,
        agent_id: &str,
        source: AutomationLifecycleSource,
        mutation: Mutation,
    ) -> Result<(T, Vec<AutomationLifecycleEvent>), String>
    where
        Mutation: FnOnce(&FileAutomationStore) -> Result<T, String>,
    {
        self.with_agent_mutation_lock(agent_id, || {
            let store = self.automation_store(agent_id)?;
            let before = store.list_definitions();
            self.sync_baseline(agent_id, &before);
            let result = mutation(&store)?;
            let after = store.list_definitions();
            let events = self.record_changes(agent_id, &before, &after, source);
            Ok((result, events))
        })
    }

    fn record_changes(
        &self,
        agent_id: &str,
        before: &[AutomationRecord],
        after: &[AutomationRecord],
        source: AutomationLifecycleSource,
    ) -> Vec<AutomationLifecycleEvent> {
        let mut known = self
            .last_known
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous = known
            .get(agent_id)
            .cloned()
            .unwrap_or_else(|| snapshot_automations(before));
        let current = snapshot_automations(after);
        known.insert(agent_id.to_string(), current.clone());
        drop(known);

        let mut events = Vec::new();
        for (id, after) in &current {
            let action = match previous.get(id) {
                None => Some(AutomationLifecycleAction::Created),
                Some(before) => diff_automation_action(before, after).map(map_diff_action),
            };
            if let Some(action) = action {
                events.push(lifecycle_event(agent_id, after, action, source));
            }
        }
        for (id, before) in &previous {
            if !current.contains_key(id) {
                events.push(lifecycle_event(
                    agent_id,
                    before,
                    AutomationLifecycleAction::Deleted,
                    source,
                ));
            }
        }
        events
    }

    fn sync_baseline(&self, agent_id: &str, definitions: &[AutomationRecord]) {
        self.last_known
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .entry(agent_id.to_string())
            .or_insert_with(|| snapshot_automations(definitions));
    }

    fn automation_store(&self, agent_id: &str) -> Result<FileAutomationStore, String> {
        SandAgentSessionStore::new(Arc::clone(&self.sessions)).automation_store_for(agent_id)
    }

    fn with_agent_mutation_lock<T>(
        &self,
        agent_id: &str,
        operation: impl FnOnce() -> Result<T, String>,
    ) -> Result<T, String> {
        let lock = {
            let mut locks = self
                .mutation_locks
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            Arc::clone(
                locks
                    .entry(agent_id.to_string())
                    .or_insert_with(|| Arc::new(Mutex::new(()))),
            )
        };
        let _guard = lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        operation()
    }
}

fn map_diff_action(action: AutomationDiffAction) -> AutomationLifecycleAction {
    match action {
        AutomationDiffAction::Updated => AutomationLifecycleAction::Updated,
        AutomationDiffAction::Enabled => AutomationLifecycleAction::Enabled,
        AutomationDiffAction::Disabled => AutomationLifecycleAction::Disabled,
    }
}

fn lifecycle_event(
    agent_id: &str,
    snapshot: &AutomationSnapshot,
    action: AutomationLifecycleAction,
    source: AutomationLifecycleSource,
) -> AutomationLifecycleEvent {
    AutomationLifecycleEvent {
        agent_id: agent_id.to_string(),
        action,
        automation_id: snapshot.id.clone(),
        automation_name: snapshot.name.clone(),
        trigger_type: snapshot.trigger_type.clone(),
        created_at: snapshot.created_at,
        recorded_run_count: snapshot.recorded_run_count,
        source,
    }
}

fn now_ms() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as f64
}
