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

fn errno_token(message: &str) -> Option<String> {
    message
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .find(|token| {
            token.len() >= 2
                && token.starts_with('E')
                && token.chars().skip(1).all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit())
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
