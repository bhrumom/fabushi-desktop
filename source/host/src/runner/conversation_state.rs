use std::collections::HashMap;

pub const HIDDEN_PROMPT_MARKER: &str = "[SAND_HIDDEN_PROMPT]";
pub const SUMMARIZATION_MAX_PROMPT_CHARS: usize = 2_800_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecentUserMessage {
    pub id: String,
    pub text: String,
}

pub fn select_unconfirmed_user_messages(
    recent_user_messages: &[RecentUserMessage],
    current_message_id: Option<&str>,
    last_turn_user_message_id: Option<&str>,
    has_confirmed_turns: bool,
) -> Vec<RecentUserMessage> {
    let Some(current_message_id) = current_message_id else {
        return Vec::new();
    };
    let Some(current_index) = recent_user_messages
        .iter()
        .position(|message| message.id == current_message_id)
    else {
        return Vec::new();
    };
    if current_index == 0 {
        return Vec::new();
    }

    let start = if let Some(last_turn_user_message_id) = last_turn_user_message_id {
        let Some(watermark) = recent_user_messages
            .iter()
            .position(|message| message.id == last_turn_user_message_id)
        else {
            return Vec::new();
        };
        watermark.saturating_add(1)
    } else if !has_confirmed_turns {
        0
    } else {
        return Vec::new();
    };

    if start >= current_index {
        return Vec::new();
    }
    recent_user_messages[start..current_index]
        .iter()
        .filter(|message| !message.text.trim().is_empty())
        .cloned()
        .collect()
}

pub fn build_unanswered_questions_note(skipped: &[String], dismissed: &[String]) -> String {
    let skipped = clean_prompts(skipped);
    let dismissed = clean_prompts(dismissed);
    let mut sections = Vec::new();
    match skipped.as_slice() {
        [] => {}
        [one] => sections.push(format!(
            "Earlier you prompted the user and they moved on without responding (\"{one}\") — treat it as skipped. Don't wait for or assume a response; continue with what you already know, and only ask again if you still genuinely need it."
        )),
        many => {
            let list = many.iter().map(|value| format!("\n- \"{value}\"")).collect::<String>();
            sections.push(format!(
                "Earlier you prompted the user for these and they moved on without responding — treat them as skipped:{list}\nDon't wait for or assume responses; continue with what you already know, and only ask again if you still genuinely need to."
            ));
        }
    }
    match dismissed.as_slice() {
        [] => {}
        [one] => sections.push(format!(
            "The user dismissed your question (\"{one}\") without answering — they'd rather not respond. Don't ask it again or wait for an answer; continue with what you already know and decide yourself."
        )),
        many => {
            let list = many.iter().map(|value| format!("\n- \"{value}\"")).collect::<String>();
            sections.push(format!(
                "The user dismissed these questions without answering — they'd rather not respond:{list}\nDon't ask them again or wait for answers; continue with what you already know and decide yourself."
            ));
        }
    }
    if sections.is_empty() {
        String::new()
    } else {
        format!("{HIDDEN_PROMPT_MARKER}{}", sections.join("\n\n"))
    }
}

pub fn to_safe_usage_count(value: f64) -> u64 {
    if !value.is_finite() || value <= 0.0 {
        0
    } else {
        value.floor().min(u64::MAX as f64) as u64
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SanitizedUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

pub fn sanitize_usage(prompt_tokens: f64, completion_tokens: f64, total_tokens: f64) -> SanitizedUsage {
    let prompt_tokens = to_safe_usage_count(prompt_tokens);
    let completion_tokens = to_safe_usage_count(completion_tokens);
    let total_tokens = to_safe_usage_count(total_tokens);
    SanitizedUsage {
        prompt_tokens,
        completion_tokens,
        total_tokens: if total_tokens == 0 {
            prompt_tokens.saturating_add(completion_tokens)
        } else {
            total_tokens
        },
    }
}

#[derive(Debug, Default)]
pub struct ResolvedModelTracker {
    sequence: u64,
    accepted: HashMap<String, (u64, String)>,
}

impl ResolvedModelTracker {
    pub fn resolved(&self, requested_model: &str) -> Option<&str> {
        self.accepted.get(requested_model).map(|(_, model)| model.as_str())
    }

    pub fn begin_request(&mut self, requested_model: &str) -> ModelResolutionTicket {
        self.sequence = self.sequence.saturating_add(1);
        ModelResolutionTicket {
            requested_model: requested_model.to_string(),
            sequence: self.sequence,
        }
    }

    pub fn accept_resolution(&mut self, ticket: &ModelResolutionTicket, model_id: &str) -> bool {
        let prior = self.accepted.get(&ticket.requested_model).map(|(sequence, _)| *sequence);
        if prior.is_some_and(|sequence| sequence >= ticket.sequence) {
            return false;
        }
        self.accepted.insert(
            ticket.requested_model.clone(),
            (ticket.sequence, model_id.to_string()),
        );
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelResolutionTicket {
    pub requested_model: String,
    pub sequence: u64,
}

pub fn should_use_self_summary(model_id: &str) -> bool {
    const DEPRECATED: [&str; 6] = [
        "grok-4.5-medium",
        "grok-4.5-fast-medium",
        "grok-4.5-high",
        "grok-4.5-fast-high",
        "grok-4.5-xhigh",
        "grok-4.5-fast-xhigh",
    ];
    let model = model_id.split('#').next().unwrap_or(model_id);
    let external = model
        .strip_prefix("XAIEXTERNAL--")
        .and_then(|rest| rest.split_once("--"))
        .map(|(_, name)| name);
    model == "grok-4.5"
        || DEPRECATED.contains(&model)
        || model == "cursor-grok-4.5"
        || model.starts_with("cursor-grok-4.5-")
        || model == "vega"
        || model.starts_with("vega-")
        || model.starts_with("accounts/anysphere/models/vega")
        || model.starts_with("cursor/vega")
        || model == "v9"
        || model.starts_with("v9-")
        || external.is_some_and(|value| value == "v9" || value.starts_with("v9-"))
}

fn clean_prompts(values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect()
}
