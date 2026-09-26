use std::path::Path;
use std::sync::Arc;

use serde_json::{Map, Value, json};

use crate::automations::automation::{AutomationRecord, AutomationSpec};
use crate::extensions::inference::provider_session::{
    ProviderSessionError, RoutedToolDefinition,
};
use crate::extensions::memory::agent_state::{
    MemoryScope, MemoryTier, SandAgentState, StateWriteResult,
};
use crate::runner::routed_provider_runtime::RoutedToolBridge;

pub const SAND_UPDATE_STATE_TOOL_NAME: &str = "update_state";

pub trait SandStateWriter: Send + Sync {
    fn automation_record(&self, id: &str) -> Option<AutomationRecord>;
    fn write_memory(
        &self,
        content: &str,
        tier: MemoryTier,
        scope: MemoryScope,
        project: Option<&str>,
    ) -> StateWriteResult;
    fn remove_memory(
        &self,
        content: &str,
        scope: MemoryScope,
        project: Option<&str>,
    ) -> StateWriteResult;
    fn create_automation(&self, spec: &AutomationSpec) -> StateWriteResult;
    fn update_automation(&self, id: &str, spec: &AutomationSpec) -> StateWriteResult;
    fn set_automation_enabled(&self, id: &str, enabled: bool) -> StateWriteResult;
    fn delete_automation(&self, id: &str) -> StateWriteResult;
    fn write_workflow(
        &self,
        id: Option<&str>,
        name: &str,
        description: Option<&str>,
        body: &str,
    ) -> StateWriteResult;
    fn delete_workflow(&self, id: &str) -> StateWriteResult;
    fn update_profile(&self, name: Option<&str>, description: Option<&str>) -> StateWriteResult;
    fn update_settings(
        &self,
        hidden_from_sidebar: Option<bool>,
        notify_on_agent_updates: Option<bool>,
    ) -> StateWriteResult;
    fn disconnect_channel(&self, platform: &str) -> StateWriteResult;
    fn create_project(
        &self,
        slug: &str,
        name: &str,
        description: Option<&str>,
    ) -> StateWriteResult;
    fn join_project(&self, slug: &str) -> StateWriteResult;
    fn leave_project(&self, slug: &str) -> StateWriteResult;
    fn set_avatar(&self, path: &Path) -> StateWriteResult;
    fn clear_avatar(&self) -> StateWriteResult;
}

impl SandStateWriter for SandAgentState {
    fn automation_record(&self, id: &str) -> Option<AutomationRecord> {
        self.automation_record(id)
    }

    fn write_memory(
        &self,
        content: &str,
        tier: MemoryTier,
        scope: MemoryScope,
        project: Option<&str>,
    ) -> StateWriteResult {
        self.write_memory(content, tier, scope, project)
    }

    fn remove_memory(
        &self,
        content: &str,
        scope: MemoryScope,
        project: Option<&str>,
    ) -> StateWriteResult {
        self.remove_memory(content, scope, project)
    }

    fn create_automation(&self, spec: &AutomationSpec) -> StateWriteResult {
        self.create_automation(spec)
    }

    fn update_automation(&self, id: &str, spec: &AutomationSpec) -> StateWriteResult {
        self.update_automation(id, spec)
    }

    fn set_automation_enabled(&self, id: &str, enabled: bool) -> StateWriteResult {
        self.set_automation_enabled(id, enabled)
    }

    fn delete_automation(&self, id: &str) -> StateWriteResult {
        self.delete_automation(id)
    }

    fn write_workflow(
        &self,
        id: Option<&str>,
        name: &str,
        description: Option<&str>,
        body: &str,
    ) -> StateWriteResult {
        self.write_workflow(id, name, description, body)
    }

    fn delete_workflow(&self, id: &str) -> StateWriteResult {
        self.delete_workflow(id)
    }

    fn update_profile(&self, name: Option<&str>, description: Option<&str>) -> StateWriteResult {
        self.update_profile(name, description)
    }

    fn update_settings(
        &self,
        hidden_from_sidebar: Option<bool>,
        notify_on_agent_updates: Option<bool>,
    ) -> StateWriteResult {
        self.update_settings(hidden_from_sidebar, notify_on_agent_updates)
    }

    fn disconnect_channel(&self, platform: &str) -> StateWriteResult {
        self.disconnect_channel(platform)
    }

    fn create_project(
        &self,
        slug: &str,
        name: &str,
        description: Option<&str>,
    ) -> StateWriteResult {
        self.create_project(slug, name, description)
    }

    fn join_project(&self, slug: &str) -> StateWriteResult {
        self.join_project(slug)
    }

    fn leave_project(&self, slug: &str) -> StateWriteResult {
        self.leave_project(slug)
    }

    fn set_avatar(&self, path: &Path) -> StateWriteResult {
        self.set_avatar(path)
    }

    fn clear_avatar(&self) -> StateWriteResult {
        self.clear_avatar()
    }
}

pub struct SandStateToolBridge {
    delegate: Arc<dyn RoutedToolBridge>,
    state: Arc<dyn SandStateWriter>,
}

impl SandStateToolBridge {
    pub fn new(
        delegate: Arc<dyn RoutedToolBridge>,
        state: Arc<dyn SandStateWriter>,
    ) -> Self {
        Self { delegate, state }
    }
}

impl RoutedToolBridge for SandStateToolBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        let mut tools = self.delegate.list_tools()?;
        tools.retain(|tool| {
            tool.name != SAND_UPDATE_STATE_TOOL_NAME
                && tool.tool_name != SAND_UPDATE_STATE_TOOL_NAME
        });
        tools.insert(0, update_state_definition());
        Ok(tools)
    }

    fn call_tool(
        &self,
        tool: &RoutedToolDefinition,
        args: Value,
        tool_call_id: &str,
    ) -> Result<Value, ProviderSessionError> {
        if tool.name != SAND_UPDATE_STATE_TOOL_NAME
            && tool.tool_name != SAND_UPDATE_STATE_TOOL_NAME
        {
            return self.delegate.call_tool(tool, args, tool_call_id);
        }
        let outcome = apply_state_update(&args, self.state.as_ref())?;
        Ok(Value::String(if outcome.ok {
            outcome.message
        } else {
            format!("Not saved - {}", outcome.message)
        }))
    }
}

fn update_state_definition() -> RoutedToolDefinition {
    RoutedToolDefinition {
        name: SAND_UPDATE_STATE_TOOL_NAME.into(),
        provider_identifier: "fabushi-runner".into(),
        tool_name: SAND_UPDATE_STATE_TOOL_NAME.into(),
        description: Some(
            "Change this agent's durable state: memory, routines, workflows, profile, settings, channels, projects, or avatar. State is persisted by the Host-owned agent-state service."
                .into(),
        ),
        input_schema: json!({
            "type": "object",
            "required": ["target", "action"],
            "additionalProperties": false,
            "properties": {
                "target": {"type":"string","enum":["memory","routine","workflow","profile","settings","channel","project","avatar"]},
                "action": {"type":"string","enum":["write","forget","create","update","pause","resume","delete","set","disconnect","join","leave","clear"]},
                "fact": {"type":"string"},
                "tier": {"type":"string","enum":["profile","log","note"]},
                "scope": {"type":"string","enum":["agent","user","project"]},
                "project": {"type":"string"},
                "id": {"type":"string"},
                "name": {"type":"string"},
                "prompt": {"type":"string"},
                "schedule": {"type":"string"},
                "trigger": {},
                "enabled": {"type":"boolean"},
                "description": {"type":"string"},
                "body": {"type":"string"},
                "hidden_from_sidebar": {"type":"boolean"},
                "notify_on_updates": {"type":"boolean"},
                "platform": {"type":"string"},
                "path": {"type":"string"}
            }
        }),
    }
}

pub fn apply_state_update(
    args: &Value,
    state: &dyn SandStateWriter,
) -> Result<StateWriteResult, ProviderSessionError> {
    let object = args.as_object().ok_or_else(|| tool_error(
        "update_state arguments must be an object",
    ))?;
    let target = required_string(object, "target")?;
    let action = required_string(object, "action")?;

    match (target, action) {
        ("memory", "write") => {
            let content = required_string(object, "fact")?;
            let tier = parse_tier(optional_string(object, "tier")?)?;
            let scope = parse_scope(optional_string(object, "scope")?)?;
            let project = optional_string(object, "project")?;
            Ok(state.write_memory(content, tier, scope, project))
        }
        ("memory", "forget") => {
            let content = required_string(object, "fact")?;
            let scope = parse_scope(optional_string(object, "scope")?)?;
            let project = optional_string(object, "project")?;
            Ok(state.remove_memory(content, scope, project))
        }
        ("routine", "create") => {
            let spec = automation_spec(object, None)?;
            Ok(state.create_automation(&spec))
        }
        ("routine", "update") => {
            let id = required_string(object, "id")?;
            let existing = state.automation_record(id).ok_or_else(|| tool_error(
                format!("no routine with folder \"{id}\" exists"),
            ))?;
            let spec = automation_spec(object, Some(&existing))?;
            Ok(state.update_automation(id, &spec))
        }
        ("routine", "pause") => {
            Ok(state.set_automation_enabled(required_string(object, "id")?, false))
        }
        ("routine", "resume") => {
            Ok(state.set_automation_enabled(required_string(object, "id")?, true))
        }
        ("routine", "delete") => {
            Ok(state.delete_automation(required_string(object, "id")?))
        }
        ("workflow", "write") => {
            let id = optional_string(object, "id")?;
            let name = required_string(object, "name")?;
            let description = required_present_string(object, "description")?;
            let body = required_string(object, "body")?;
            Ok(state.write_workflow(id, name, Some(description), body))
        }
        ("workflow", "delete") => {
            Ok(state.delete_workflow(required_string(object, "id")?))
        }
        ("profile", "set") => Ok(state.update_profile(
            optional_string(object, "name")?,
            optional_present_string(object, "description")?,
        )),
        ("settings", "set") => Ok(state.update_settings(
            optional_bool(object, "hidden_from_sidebar")?,
            optional_bool(object, "notify_on_updates")?,
        )),
        ("channel", "disconnect") => {
            Ok(state.disconnect_channel(required_string(object, "platform")?))
        }
        ("project", "create") => Ok(state.create_project(
            required_string(object, "project")?,
            required_string(object, "name")?,
            optional_present_string(object, "description")?,
        )),
        ("project", "join") => {
            Ok(state.join_project(required_string(object, "project")?))
        }
        ("project", "leave") => {
            Ok(state.leave_project(required_string(object, "project")?))
        }
        ("avatar", "set") => {
            Ok(state.set_avatar(Path::new(required_string(object, "path")?)))
        }
        ("avatar", "clear") => Ok(state.clear_avatar()),
        _ => Err(tool_error(format!(
            "'{action}' is not an action on {target}"
        ))),
    }
}

fn automation_spec(
    object: &Map<String, Value>,
    existing: Option<&AutomationRecord>,
) -> Result<AutomationSpec, ProviderSessionError> {
    let name = match optional_string(object, "name")? {
        Some(value) => value.to_string(),
        None => existing
            .map(|record| record.name.clone())
            .ok_or_else(|| tool_error("name is required for routine create"))?,
    };
    let prompt = match optional_string(object, "prompt")? {
        Some(value) => value.to_string(),
        None => existing
            .map(|record| record.prompt.clone())
            .ok_or_else(|| tool_error("prompt is required for routine create"))?,
    };
    let schedule = optional_string(object, "schedule")?;
    let trigger = object.get("trigger").filter(|value| !value.is_null());
    if schedule.is_some() && trigger.is_some() {
        return Err(tool_error(
            "pass either schedule or trigger for a routine, never both",
        ));
    }
    let trigger = if let Some(schedule) = schedule {
        json!({"type":"cron","schedule":schedule})
    } else if let Some(trigger) = trigger {
        if trigger.is_array() {
            json!({"type":"group","listeners":trigger})
        } else if trigger.is_object() {
            trigger.clone()
        } else {
            return Err(tool_error("trigger must be an object or array"));
        }
    } else if let Some(existing) = existing {
        existing.trigger.clone()
    } else {
        return Err(tool_error(
            "schedule or trigger is required for routine create",
        ));
    };
    let enabled = optional_bool(object, "enabled")?
        .or_else(|| existing.is_none().then_some(true));
    Ok(AutomationSpec {
        name,
        prompt,
        trigger,
        is_enabled: enabled,
    })
}

fn parse_tier(value: Option<&str>) -> Result<MemoryTier, ProviderSessionError> {
    match value.unwrap_or("log") {
        "profile" => Ok(MemoryTier::Profile),
        "log" => Ok(MemoryTier::Log),
        "note" => Ok(MemoryTier::Note),
        other => Err(tool_error(format!("unsupported memory tier: {other}"))),
    }
}

fn parse_scope(value: Option<&str>) -> Result<MemoryScope, ProviderSessionError> {
    match value.unwrap_or("agent") {
        "agent" => Ok(MemoryScope::Agent),
        "user" => Ok(MemoryScope::User),
        "project" => Ok(MemoryScope::Project),
        other => Err(tool_error(format!("unsupported memory scope: {other}"))),
    }
}

fn required_string<'a>(
    object: &'a Map<String, Value>,
    field: &str,
) -> Result<&'a str, ProviderSessionError> {
    optional_string(object, field)?
        .ok_or_else(|| tool_error(format!("{field} is required")))
}

fn optional_string<'a>(
    object: &'a Map<String, Value>,
    field: &str,
) -> Result<Option<&'a str>, ProviderSessionError> {
    match object.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => {
            let value = value.trim();
            if value.is_empty() {
                Err(tool_error(format!("{field} must not be empty")))
            } else {
                Ok(Some(value))
            }
        }
        Some(_) => Err(tool_error(format!("{field} must be a string"))),
    }
}

fn required_present_string<'a>(
    object: &'a Map<String, Value>,
    field: &str,
) -> Result<&'a str, ProviderSessionError> {
    optional_present_string(object, field)?
        .ok_or_else(|| tool_error(format!("{field} is required")))
}

fn optional_present_string<'a>(
    object: &'a Map<String, Value>,
    field: &str,
) -> Result<Option<&'a str>, ProviderSessionError> {
    match object.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.trim())),
        Some(_) => Err(tool_error(format!("{field} must be a string"))),
    }
}

fn optional_bool(
    object: &Map<String, Value>,
    field: &str,
) -> Result<Option<bool>, ProviderSessionError> {
    match object.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(value)) => Ok(Some(*value)),
        Some(_) => Err(tool_error(format!("{field} must be a boolean"))),
    }
}

fn tool_error(message: impl Into<String>) -> ProviderSessionError {
    ProviderSessionError::Tool(message.into())
}
