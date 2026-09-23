use crate::extensions::session::production::ProductionSessionWorkers;
use crate::extensions::session::session_profile_files::AgentProfileUpdate;

pub const SAND_DEFAULT_AGENT_NAME: &str = "New Bot";
pub const LEGACY_SAND_DEFAULT_AGENT_NAME: &str = "New Agent";
pub const NEW_CONVERSATION_FALLBACK_NAME: &str = "New conversation";
pub const MAX_SEEDED_AGENT_NAME_UTF16: usize = 72;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SendAcceptancePreparation {
    pub seeded_name: Option<String>,
    pub awaiting_cleared: bool,
    pub introduction_cleared: bool,
}

pub fn is_sand_default_agent_name(name: &str) -> bool {
    matches!(
        name.trim(),
        SAND_DEFAULT_AGENT_NAME | LEGACY_SAND_DEFAULT_AGENT_NAME
    )
}

pub fn build_seeded_agent_name(prompt: &str) -> String {
    let collapsed = prompt.split_whitespace().collect::<Vec<_>>().join(" ");
    let seed = if collapsed.is_empty() {
        NEW_CONVERSATION_FALLBACK_NAME
    } else {
        collapsed.as_str()
    };
    let utf16 = seed
        .encode_utf16()
        .take(MAX_SEEDED_AGENT_NAME_UTF16)
        .collect::<Vec<_>>();
    String::from_utf16_lossy(&utf16)
}

pub fn prepare_send_acceptance(
    session: &ProductionSessionWorkers,
    agent_id: &str,
    transcript_len_before: usize,
    trimmed_prompt: &str,
) -> Result<SendAcceptancePreparation, String> {
    let current_profile = session.get_agent_profile_text(agent_id)?;
    let effective_name = current_profile
        .as_ref()
        .map(|profile| profile.name.as_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or(SAND_DEFAULT_AGENT_NAME);

    let seeded_name = if transcript_len_before == 0 && is_sand_default_agent_name(effective_name) {
        let seeded_name = build_seeded_agent_name(trimmed_prompt);
        let current = current_profile.as_ref();
        session.update_agent_profile(
            agent_id,
            &AgentProfileUpdate {
                name: seeded_name.clone(),
                description: current
                    .map(|profile| profile.description.clone())
                    .unwrap_or_default(),
                title: Some(
                    current
                        .map(|profile| profile.title.clone())
                        .unwrap_or_default(),
                ),
                avatar_shape: Some(
                    current
                        .map(|profile| profile.avatar_shape.clone())
                        .unwrap_or_default(),
                ),
                avatar_color: Some(
                    current
                        .map(|profile| profile.avatar_color.clone())
                        .unwrap_or_default(),
                ),
            },
            None,
        )?;
        Some(seeded_name)
    } else {
        None
    };

    let awaiting_cleared = session.set_agent_awaiting_user_response(agent_id, None)?;
    let introduction_cleared = session.set_agent_introduction_pending(agent_id, false)?;

    Ok(SendAcceptancePreparation {
        seeded_name,
        awaiting_cleared,
        introduction_cleared,
    })
}

pub fn mark_accepted_send_activity(
    session: &ProductionSessionWorkers,
    agent_id: &str,
    at_ms: f64,
) -> Result<bool, String> {
    session.mark_agent_activity(agent_id, at_ms)
}
