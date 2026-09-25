use std::sync::Arc;

use serde_json::{Map, Value};

use crate::extensions::session::agent_session::{
    SandAgentSessionStore, WorkflowImportResponse,
};
use crate::extensions::session::production::ProductionSessionWorkers;
use crate::workflows::workflow_library::{WorkflowSpec, WorkflowTrigger};
use crate::workflows::workflow_store::{WorkflowImportBatch, WorkflowRecord};

pub const WORKFLOW_REFERENCE_NODE_TYPE: &str = "workflowReference";
pub const WORKFLOW_INJECTED_BODY_LIMIT: usize = 8_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowRunNowPlan {
    Automation {
        agent_id: String,
        automation_id: String,
    },
    Reference {
        agent_id: String,
        workflow_id: String,
        workflow_name: String,
        visible_prompt: String,
        rich_text: String,
        runtime_prompt: String,
    },
}

pub fn prepare_workflow_run_now(
    workers: Arc<ProductionSessionWorkers>,
    args: &Value,
) -> Result<Option<WorkflowRunNowPlan>, WorkflowCommandError> {
    let agent_id = agent_id(args)?;
    let workflow_id = required_string(args, &["workflowId"], "workflowId")?;
    let store = SandAgentSessionStore::new(workers);
    let Some(workflow) = store
        .get_agent_workflow(agent_id, workflow_id)
        .map_err(WorkflowCommandError::Internal)?
    else {
        return Ok(None);
    };

    if workflow.source == "automation" {
        return Ok(Some(WorkflowRunNowPlan::Automation {
            agent_id: agent_id.to_string(),
            automation_id: workflow.id,
        }));
    }

    let visible_prompt = format!("@{}", workflow.name);
    let rich_text = workflow_reference_rich_text(&workflow)?;
    let runtime_prompt = if workflow.is_enabled_for_agent {
        format!(
            "{}\n\n{}",
            build_workflow_run_prompt(&workflow),
            visible_prompt
        )
    } else {
        visible_prompt.clone()
    };
    Ok(Some(WorkflowRunNowPlan::Reference {
        agent_id: agent_id.to_string(),
        workflow_id: workflow.id,
        workflow_name: workflow.name,
        visible_prompt,
        rich_text,
        runtime_prompt,
    }))
}

pub fn build_workflow_run_prompt(workflow: &WorkflowRecord) -> String {
    let identity = match workflow.source.as_str() {
        "managed" => format!("managed skill id {}", workflow.id),
        "plugin" => format!(
            "plugin skill id {}, file {}",
            workflow.id,
            workflow.file_path.display()
        ),
        _ => format!("folder {}", workflow.id),
    };
    let mut lines = vec![format!(
        "The user invoked the \"{}\" workflow ({}). Run it now.",
        workflow.name, identity
    )];
    if !workflow.description.is_empty() {
        lines.push(format!("What it does: {}", workflow.description));
    }
    lines.push("Recipe to follow:".into());
    lines.push(
        workflow
            .body
            .trim()
            .chars()
            .take(WORKFLOW_INJECTED_BODY_LIMIT)
            .collect(),
    );
    if !workflow.helper_scripts.is_empty() {
        let workflow_dir = workflow
            .file_path
            .parent()
            .unwrap_or(workflow.file_path.as_path());
        lines.push(format!(
            "Helper files live beside this workflow in {}: {}. Use them with Shell as the recipe directs.",
            workflow_dir.display(),
            workflow.helper_scripts.join(", ")
        ));
    }
    lines.push(
        "Carry out the recipe now, adapting it to anything else the user said in this message."
            .into(),
    );
    lines.join("\n")
}

fn workflow_reference_rich_text(
    workflow: &WorkflowRecord,
) -> Result<String, WorkflowCommandError> {
    serde_json::to_string(&serde_json::json!({
        "type": "doc",
        "content": [{
            "type": "paragraph",
            "content": [{
                "type": WORKFLOW_REFERENCE_NODE_TYPE,
                "attrs": {
                    "id": workflow.id,
                    "label": workflow.name,
                }
            }]
        }]
    }))
    .map_err(|error| WorkflowCommandError::Internal(error.to_string()))
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum WorkflowCommandError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Internal(String),
}

pub fn dispatch_workflow_command(
    workers: Arc<ProductionSessionWorkers>,
    method: &str,
    args: &Value,
) -> Option<Result<Value, WorkflowCommandError>> {
    if !matches!(
        method,
        "getAgentWorkflows"
            | "createAgentWorkflow"
            | "updateAgentWorkflow"
            | "setAgentWorkflowEnabled"
            | "deleteAgentWorkflow"
            | "importAgentWorkflowText"
            | "importAgentWorkflowUrl"
            | "portAgentLocalSkills"
    ) {
        return None;
    }

    let store = SandAgentSessionStore::new(workers);
    let result = match method {
        "getAgentWorkflows" => agent_id(args)
            .and_then(|agent_id| {
                store
                    .list_agent_workflows(agent_id)
                    .map_err(WorkflowCommandError::Internal)
            })
            .map(workflow_records_value),
        "createAgentWorkflow" => {
            let agent_id = match agent_id(args) {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let spec = match workflow_spec(args.get("spec").unwrap_or(args)) {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            store
                .create_agent_workflow(agent_id, &spec)
                .map(workflow_records_value)
                .map_err(WorkflowCommandError::Internal)
        }
        "updateAgentWorkflow" => {
            let agent_id = match agent_id(args) {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let workflow_id = match required_string(args, &["workflowId"], "workflowId") {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let spec = match workflow_spec(args.get("spec").unwrap_or(args)) {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            store
                .update_agent_workflow(agent_id, workflow_id, &spec)
                .map(workflow_records_value)
                .map_err(WorkflowCommandError::Internal)
        }
        "setAgentWorkflowEnabled" => {
            let agent_id = match agent_id(args) {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let workflow_id = match required_string(args, &["workflowId"], "workflowId") {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let enabled = match args.get("isEnabled").and_then(Value::as_bool) {
                Some(value) => value,
                None => {
                    return Some(Err(WorkflowCommandError::BadRequest(
                        "setAgentWorkflowEnabled requires boolean isEnabled".into(),
                    )));
                }
            };
            store
                .set_agent_workflow_enabled(agent_id, workflow_id, enabled)
                .map(workflow_records_value)
                .map_err(WorkflowCommandError::Internal)
        }
        "deleteAgentWorkflow" => {
            let agent_id = match agent_id(args) {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let workflow_id = match required_string(args, &["workflowId"], "workflowId") {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            store
                .remove_agent_workflow(agent_id, workflow_id)
                .map(workflow_records_value)
                .map_err(WorkflowCommandError::Internal)
        }
        "importAgentWorkflowText" => {
            let agent_id = match agent_id(args) {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let markdown = match required_string(args, &["markdown"], "markdown") {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let name = optional_string(args, &["name"]);
            store
                .import_agent_workflow_markdown(agent_id, markdown, name)
                .map(workflow_import_response_value)
                .map_err(WorkflowCommandError::Internal)
        }
        "importAgentWorkflowUrl" => {
            let agent_id = match agent_id(args) {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let url = match required_string(args, &["url"], "url") {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let name = optional_string(args, &["name"]);
            store
                .import_agent_workflow_source(agent_id, url, name)
                .map(workflow_import_response_value)
                .map_err(WorkflowCommandError::Internal)
        }
        "portAgentLocalSkills" => agent_id(args)
            .and_then(|agent_id| {
                store
                    .port_agent_local_skills(agent_id)
                    .map_err(WorkflowCommandError::Internal)
            })
            .map(workflow_import_response_value),
        _ => unreachable!(),
    };
    Some(result)
}

fn agent_id(args: &Value) -> Result<&str, WorkflowCommandError> {
    required_string(args, &["id", "agentId"], "id")
}

fn required_string<'a>(
    value: &'a Value,
    keys: &[&str],
    label: &str,
) -> Result<&'a str, WorkflowCommandError> {
    optional_string(value, keys).ok_or_else(|| {
        WorkflowCommandError::BadRequest(format!("workflow command requires non-empty {label}"))
    })
}

fn optional_string<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| value.get(*key))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn workflow_spec(value: &Value) -> Result<WorkflowSpec, WorkflowCommandError> {
    let object = value.as_object().ok_or_else(|| {
        WorkflowCommandError::BadRequest("workflow spec must be an object".into())
    })?;
    let name = object
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let description = object
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let body = object
        .get("body")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let source_ref = object
        .get("sourceRef")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let trigger = match object.get("trigger") {
        None | Some(Value::Null) => None,
        Some(Value::Object(trigger)) => {
            let schedule = trigger
                .get("schedule")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            Some(WorkflowTrigger {
                schedule,
                is_enabled: trigger
                    .get("isEnabled")
                    .or_else(|| trigger.get("enabled"))
                    .and_then(Value::as_bool)
                    .unwrap_or(true),
            })
        }
        Some(_) => {
            return Err(WorkflowCommandError::BadRequest(
                "workflow trigger must be an object or null".into(),
            ));
        }
    };
    Ok(WorkflowSpec {
        name,
        description,
        body,
        trigger,
        source_ref,
    })
}

fn workflow_records_value(records: Vec<WorkflowRecord>) -> Value {
    Value::Array(records.into_iter().map(workflow_record_value).collect())
}

fn workflow_record_value(record: WorkflowRecord) -> Value {
    let mut value = Map::new();
    value.insert("id".into(), Value::String(record.id));
    value.insert("name".into(), Value::String(record.name));
    value.insert("description".into(), Value::String(record.description));
    value.insert("body".into(), Value::String(record.body));
    if let Some(trigger) = record.trigger {
        value.insert(
            "trigger".into(),
            serde_json::json!({
                "schedule": trigger.schedule,
                "isEnabled": trigger.is_enabled,
            }),
        );
    }
    value.insert("source".into(), Value::String(record.source));
    if let Some(source_ref) = record.source_ref {
        value.insert("sourceRef".into(), Value::String(source_ref));
    }
    value.insert(
        "isEnabledForAgent".into(),
        Value::Bool(record.is_enabled_for_agent),
    );
    value.insert("createdAt".into(), number_value(record.created_at));
    if let Some(last_run_at) = record.last_run_at {
        value.insert("lastRunAt".into(), number_value(last_run_at));
    }
    if let Some(next_run_at) = record.next_run_at {
        value.insert("nextRunAt".into(), number_value(next_run_at));
    }
    value.insert(
        "helperScripts".into(),
        Value::Array(
            record
                .helper_scripts
                .into_iter()
                .map(Value::String)
                .collect(),
        ),
    );
    value.insert(
        "filePath".into(),
        Value::String(record.file_path.to_string_lossy().into_owned()),
    );
    Value::Object(value)
}

fn workflow_import_response_value(response: WorkflowImportResponse) -> Value {
    serde_json::json!({
        "workflows": response
            .workflows
            .into_iter()
            .map(workflow_record_value)
            .collect::<Vec<_>>(),
        "result": workflow_import_batch_value(response.result),
    })
}

fn workflow_import_batch_value(batch: WorkflowImportBatch) -> Value {
    serde_json::json!({
        "imported": batch
            .imported
            .into_iter()
            .map(|item| serde_json::json!({"id": item.id, "name": item.name}))
            .collect::<Vec<_>>(),
        "skipped": batch
            .skipped
            .into_iter()
            .map(|item| serde_json::json!({"source": item.source, "reason": item.reason}))
            .collect::<Vec<_>>(),
    })
}

fn number_value(value: f64) -> Value {
    serde_json::Number::from_f64(value)
        .map(Value::Number)
        .unwrap_or(Value::Null)
}
