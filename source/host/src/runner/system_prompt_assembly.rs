use serde_json::Value;

use crate::host_request_context::HostRequestContext;
use crate::automations::automation::{AutomationRecord, render_automations_system_prompt};
use crate::extensions::inference::provider_session::ProviderMessage;
use crate::extensions::memory::memory_service::{
    MemoryRecall, ProjectMemoryPromptRecall, UserMemoryRecall,
};
use crate::sand_multitask::SAND_MULTITASK_PROMPT_SECTION;

use super::system_prompt::{
    SAND_CLOUD_AGENTS_DISABLED_PROMPT_SECTION, build_sand_base_system_prompt,
};
use super::sand_memory::{
    FrozenMemoryPrompt, FrozenMemorySnapshot, MEMORY_PROJECT_SYSTEM_PROMPT_HEADER,
    MEMORY_SYSTEM_PROMPT_HEADER, MEMORY_USER_SYSTEM_PROMPT_HEADER,
    render_memory_system_prompt, render_project_memory_system_prompt,
    render_user_memory_system_prompt, resolve_frozen_memory_prompt,
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


pub fn resolve_combined_memory_system_prompt(
    user_recall: &UserMemoryRecall,
    user_memory_dir: Option<&str>,
    user_own_shard_dir: Option<&str>,
    project_recall: &ProjectMemoryPromptRecall,
    projects_root_dir: Option<&str>,
    agent_recall: &MemoryRecall,
    agent_memory_dir: Option<&str>,
    snapshot: Option<&FrozenMemorySnapshot>,
    compaction_epoch: u64,
    freeze_enabled: bool,
) -> FrozenMemoryPrompt {
    let render_live = || {
        let mut parts = Vec::new();
        let mut has_facts = !user_recall.profile.is_empty() || !user_recall.recent.is_empty();

        let user = render_user_memory_system_prompt(
            user_recall,
            user_memory_dir,
            user_own_shard_dir,
        );
        if !user.is_empty() {
            parts.push(user);
        }

        let project = render_project_memory_system_prompt(project_recall, projects_root_dir);
        if !project.is_empty() {
            parts.push(project);
        }
        has_facts |= project_recall.injected.iter().any(|block| {
            !block.recall.profile.is_empty() || !block.recall.recent.is_empty()
        });

        let agent = render_memory_system_prompt(agent_recall, agent_memory_dir);
        if !agent.is_empty() {
            parts.push(agent);
        }
        has_facts |= !agent_recall.profile.is_empty() || !agent_recall.recent.is_empty();

        (parts.join("\n\n"), has_facts)
    };

    if !freeze_enabled {
        let (render, _) = render_live();
        return FrozenMemoryPrompt {
            render,
            snapshot_to_persist: None,
        };
    }
    resolve_frozen_memory_prompt(snapshot, compaction_epoch, render_live)
}

pub fn append_combined_memory_system_prompt(
    messages: &mut Vec<ProviderMessage>,
    memory: &str,
) {
    if memory.is_empty() {
        return;
    }
    if messages.iter().any(|message| {
        message.role == "system"
            && (message.content.contains(MEMORY_USER_SYSTEM_PROMPT_HEADER)
                || message.content.contains(MEMORY_PROJECT_SYSTEM_PROMPT_HEADER)
                || message.content.contains(MEMORY_SYSTEM_PROMPT_HEADER))
    }) {
        return;
    }
    if let Some(system) = messages.iter_mut().find(|message| message.role == "system") {
        if !system.content.trim().is_empty() {
            system.content.push_str("\n\n");
        }
        system.content.push_str(memory);
    } else {
        messages.insert(
            0,
            ProviderMessage {
                role: "system".into(),
                content: memory.to_string(),
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

pub fn append_project_memory_system_prompt(
    messages: &mut Vec<ProviderMessage>,
    recall: &ProjectMemoryPromptRecall,
    projects_root_dir: Option<&str>,
) {
    let memory = render_project_memory_system_prompt(recall, projects_root_dir);
    if memory.is_empty() {
        return;
    }
    if messages.iter().any(|message| {
        message.role == "system" && message.content.contains(MEMORY_PROJECT_SYSTEM_PROMPT_HEADER)
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
