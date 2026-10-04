use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use url::Url;
use uuid::Uuid;

pub const ATTACHMENTS_DIRNAME: &str = "attachments";
pub const ASSETS_DIRNAME: &str = "assets";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentMediaKind {
    Image,
    Attachment,
}

pub fn get_agent_attachments_dir(agent_dir: impl AsRef<Path>) -> PathBuf {
    agent_dir.as_ref().join(ATTACHMENTS_DIRNAME)
}

pub fn get_agent_assets_dir(agent_dir: impl AsRef<Path>) -> PathBuf {
    agent_dir.as_ref().join(ASSETS_DIRNAME)
}

pub fn get_agent_media_store_roots(agent_dir: impl AsRef<Path>) -> [PathBuf; 2] {
    [get_agent_attachments_dir(&agent_dir), get_agent_assets_dir(agent_dir)]
}

fn safe_media_leaf(source_name: &str) -> String {
    let leaf = Path::new(source_name)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("attachment");
    let mut safe = leaf
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    while safe.starts_with('.') {
        safe.remove(0);
    }
    if safe.is_empty() {
        safe.push_str("attachment");
    }
    safe.truncate(160);
    safe
}

pub fn persist_agent_media_bytes(
    agent_dir: impl AsRef<Path>,
    source_name: &str,
    bytes: &[u8],
    kind: AgentMediaKind,
) -> io::Result<PathBuf> {
    let root = match kind {
        AgentMediaKind::Image => get_agent_assets_dir(agent_dir),
        AgentMediaKind::Attachment => get_agent_attachments_dir(agent_dir),
    };
    fs::create_dir_all(&root)?;
    let target = root.join(format!("{}-{}", Uuid::new_v4().simple(), safe_media_leaf(source_name)));
    let temporary = root.join(format!(".{}.tmp", Uuid::new_v4().simple()));
    fs::write(&temporary, bytes)?;
    if let Err(error) = fs::rename(&temporary, &target) {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    Ok(target)
}

pub fn file_url_for_path(path: impl AsRef<Path>) -> Option<String> {
    Url::from_file_path(path.as_ref()).ok().map(String::from)
}
