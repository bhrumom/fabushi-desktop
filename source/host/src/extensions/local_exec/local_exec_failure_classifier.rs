#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalExecFailureClass {
    Other,
    SpawnEnoent,
    SpawnPermissions,
    SpawnOther,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalExecFailureClassification {
    pub error_class: LocalExecFailureClass,
    pub errno: Option<String>,
}

fn strip_ansi_sequences(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            for next in chars.by_ref() {
                if ('@'..='~').contains(&next) {
                    break;
                }
            }
        } else {
            output.push(ch);
        }
    }
    output
}

fn truncate_utf16(input: &str, max_units: usize) -> String {
    let mut used = 0;
    input
        .chars()
        .take_while(|ch| {
            let width = ch.len_utf16();
            let fits = used + width <= max_units;
            if fits {
                used += width;
            }
            fits
        })
        .collect()
}

/// Format bounded command output for a user-facing error without leaking ANSI
/// control sequences or allowing an unbounded child-process payload.
pub fn format_command_output_for_user(output: &str, max_chars: usize) -> String {
    let plain = strip_ansi_sequences(output);
    let mut lines = Vec::new();
    for line in plain.split('\n') {
        lines.push(line.rsplit('\r').next().unwrap_or_default());
    }
    let tail = lines
        .into_iter()
        .rev()
        .take(12)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_owned();
    let truncated = truncate_utf16(&tail, max_chars);
    if truncated.encode_utf16().count() < tail.encode_utf16().count() {
        format!("…\n{truncated}")
    } else {
        truncated
    }
}

fn errno_token(message: &str) -> Option<String> {
    message
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .find(|token| {
            token.len() >= 2
                && token.starts_with('E')
                && token
                    .chars()
                    .skip(1)
                    .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit())
        })
        .map(ToOwned::to_owned)
}

pub fn classify_local_exec_failure(message: &str) -> LocalExecFailureClassification {
    let lower = message.to_ascii_lowercase();
    let is_spawn = lower
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .any(|token| token == "spawn" || token == "spawnsync");
    let errno = errno_token(message);
    let error_class = if !is_spawn {
        LocalExecFailureClass::Other
    } else {
        match errno.as_deref() {
            Some("ENOENT") => LocalExecFailureClass::SpawnEnoent,
            Some("EACCES" | "EPERM") => LocalExecFailureClass::SpawnPermissions,
            _ => LocalExecFailureClass::SpawnOther,
        }
    };
    LocalExecFailureClassification { error_class, errno }
}
