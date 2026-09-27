use std::collections::HashMap;

pub const PRESIGN_READ_BATCH_MAX: usize = 500;

#[derive(Debug, Default, Clone)]
pub struct AgentStoreMutableWriteEtags {
    etags: HashMap<(String, String), String>,
}

impl AgentStoreMutableWriteEtags {
    pub fn get(&self, source_id: &str, rel_path: &str) -> Option<&str> {
        self.etags
            .get(&(source_id.to_string(), rel_path.to_string()))
            .map(String::as_str)
    }

    pub fn set(&mut self, source_id: &str, rel_path: &str, etag: impl Into<String>) {
        self.etags.insert(
            (source_id.to_string(), rel_path.to_string()),
            etag.into(),
        );
    }

    pub fn delete(&mut self, source_id: &str, rel_path: &str) {
        self.etags
            .remove(&(source_id.to_string(), rel_path.to_string()));
    }
}

pub fn is_conditional_write_rejection(status: u16) -> bool {
    matches!(status, 409 | 412)
}

pub fn normalize_rel_path(rel_path: &str) -> Result<String, String> {
    if rel_path.is_empty() {
        return Err("Agent store relative path must be a non-empty string".into());
    }
    if rel_path.contains('\0') {
        return Err("Agent store relative path must not contain NUL bytes".into());
    }
    let normalized = rel_path.replace('\\', "/");
    let bytes = normalized.as_bytes();
    let windows_drive = bytes.len() >= 2
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':';
    if normalized.starts_with('/') || normalized.starts_with("//") || windows_drive {
        return Err(format!(
            "Agent store relative path must not be absolute: {rel_path}"
        ));
    }

    let mut segments = Vec::new();
    for segment in normalized.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." {
            return Err(format!(
                "Agent store relative path must not contain '..': {rel_path}"
            ));
        }
        if segment.len() > 255 {
            return Err(format!(
                "Agent store path segment exceeds 255 bytes: {segment}"
            ));
        }
        if let Some(byte) = segment
            .as_bytes()
            .iter()
            .copied()
            .find(|byte| *byte < 32 || *byte == 127)
        {
            return Err(format!(
                "Agent store path segment contains control character (0x{byte:x}): {segment}"
            ));
        }
        if segment.ends_with('.') || segment.ends_with(' ') {
            return Err(format!(
                "Agent store path segment must not end with '.' or space (Windows trims these): {segment}"
            ));
        }
        let stem = segment
            .split('.')
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
        if is_windows_reserved_name(&stem) {
            return Err(format!(
                "Agent store path segment uses a reserved device name: {segment}"
            ));
        }
        segments.push(segment);
    }
    if segments.is_empty() {
        return Err(format!(
            "Agent store relative path resolved to empty: {rel_path}"
        ));
    }
    Ok(segments.join("/"))
}

fn is_windows_reserved_name(stem: &str) -> bool {
    matches!(stem, "con" | "prn" | "aux" | "nul")
        || matches!(stem.strip_prefix("com"), Some(n) if matches!(n, "1"|"2"|"3"|"4"|"5"|"6"|"7"|"8"|"9"))
        || matches!(stem.strip_prefix("lpt"), Some(n) if matches!(n, "1"|"2"|"3"|"4"|"5"|"6"|"7"|"8"|"9"))
}

pub fn batch_read_paths<I, S>(paths: I) -> Vec<Vec<String>>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let paths = paths.into_iter().map(Into::into).collect::<Vec<_>>();
    paths
        .chunks(PRESIGN_READ_BATCH_MAX)
        .map(|chunk| chunk.to_vec())
        .collect()
}
