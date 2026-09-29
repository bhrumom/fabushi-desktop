use serde_json::Value;

use crate::host_request_context::HostRequestContext;
use crate::automations::automation::{AutomationRecord, render_automations_system_prompt};
use crate::extensions::inference::provider_session::ProviderMessage;
use crate::extensions::memory::memory_service::{MemoryRecall, UserMemoryRecall};
use crate::sand_multitask::SAND_MULTITASK_PROMPT_SECTION;

use super::system_prompt::{
    SAND_CLOUD_AGENTS_DISABLED_PROMPT_SECTION, build_sand_base_system_prompt,
};
use super::sand_memory::{
    MEMORY_SYSTEM_PROMPT_HEADER, MEMORY_USER_SYSTEM_PROMPT_HEADER,
    render_memory_system_prompt, render_user_memory_system_prompt,
};

pub fn render_request_context_system_prompt(
    context: &HostRequestContext,
    rules: Option<&[Value]>,
) -> String {
    render_request_context_system_prompt_with_capabilities(context, rules, false, false)
}

pub fn render_request_context_system_prompt_with_capabilities(
    context: &HostRequestContext,
    rules: Option<&[Value]>,
    cloud_agents_enabled: bool,
    multitask_enabled: bool,
) -> String {
    let mut sections = vec![build_sand_base_system_prompt(cloud_agents_enabled).to_string()];
    if !cloud_agents_enabled {
        sections.push(SAND_CLOUD_AGENTS_DISABLED_PROMPT_SECTION.to_string());
    }
    if multitask_enabled {
        sections.push(SAND_MULTITASK_PROMPT_SECTION.to_string());
    }

    if let Some(time_zone) = context
        .time_zone
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        sections.push(format!(
            "## Time\nYour box and tools run on a UTC clock, but the user lives in {time_zone}. Convert UTC timestamps you report to the user's time zone and label the converted time clearly."
        ));
    }

    if let Some(full_name) = context
        .user_full_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        sections.push(format!(
            "Your user is {full_name}; when acting through their accounts and apps, such as Slack, speak as them and never refer to them in the third person."
        ));
    }

    if let Some(rendered_rules) = render_team_rules(rules) {
        sections.push(rendered_rules);
    }

    sections.join("\n\n")
}

fn render_team_rules(rules: Option<&[Value]>) -> Option<String> {
    let rules = rules?;
    let mut rendered = Vec::new();
    for rule in rules {
        let Some(content) = rule
            .get("content")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        let name = rule
            .get("fullPath")
            .or_else(|| rule.get("name"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("team rule");
        let required = rule
            .get("isRequired")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        rendered.push(format!(
            "### {name}{}\n{content}",
            if required { " (required)" } else { "" }
        ));
    }
    (!rendered.is_empty()).then(|| {
        format!(
            "## Team rules\nApply the following organization rules to this turn.\n\n{}",
            rendered.join("\n\n")
        )
    })
}


pub fn append_automations_system_prompt(
    messages: &mut Vec<ProviderMessage>,
    automations: &[AutomationRecord],
    location: Option<&str>,
    time_zone: Option<&str>,
) {
    let prompt = render_automations_system_prompt(automations, location, time_zone);
    if prompt.is_empty() {
        return;
    }
    if messages.iter().any(|message| {
        message.role == "system"
            && (message.content.starts_with("## Routines\n")
                || message.content.contains("\n\n## Routines\n"))
    }) {
        return;
    }
    if let Some(system) = messages.iter_mut().find(|message| message.role == "system") {
        if !system.content.trim().is_empty() {
            system.content.push_str("\n\n");
        }
        system.content.push_str(&prompt);
    } else {
        messages.insert(
            0,
            ProviderMessage {
                role: "system".into(),
                content: prompt,
            },
        );
    }
}


pub fn append_user_memory_system_prompt(
    messages: &mut Vec<ProviderMessage>,
    recall: &UserMemoryRecall,
    user_memory_dir: Option<&str>,
    own_shard_dir: Option<&str>,
) {
    let memory = render_user_memory_system_prompt(recall, user_memory_dir, own_shard_dir);
    if memory.is_empty() { return; }
    if messages.iter().any(|message| {
        message.role == "system" && message.content.contains(MEMORY_USER_SYSTEM_PROMPT_HEADER)
    }) { return; }
    if let Some(system) = messages.iter_mut().find(|message| message.role == "system") {
        if !system.content.trim().is_empty() { system.content.push_str("\n\n"); }
        system.content.push_str(&memory);
    } else {
        messages.insert(0, ProviderMessage { role: "system".into(), content: memory });
    }
}

pub fn append_memory_system_prompt(
    messages: &mut Vec<ProviderMessage>,
    recall: &MemoryRecall,
    location: Option<&str>,
) {
    let memory = render_memory_system_prompt(recall, location);
    if memory.is_empty() {
        return;
    }
    if messages.iter().any(|message| {
        message.role == "system" && message.content.contains(MEMORY_SYSTEM_PROMPT_HEADER)
    }) {
        return;
    }
    if let Some(system) = messages.iter_mut().find(|message| message.role == "system") {
        if !system.content.trim().is_empty() {
            system.content.push_str("\n\n");
        }
        system.content.push_str(&memory);
    } else {
        messages.insert(
            0,
            ProviderMessage {
                role: "system".into(),
                content: memory,
            },
        );
    }
}
