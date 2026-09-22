use std::path::{Path, PathBuf};

pub const ATTACHMENTS_DIRNAME: &str = "attachments";
pub const ASSETS_DIRNAME: &str = "assets";

pub fn get_agent_attachments_dir(agent_dir: impl AsRef<Path>) -> PathBuf {
    agent_dir.as_ref().join(ATTACHMENTS_DIRNAME)
}

pub fn get_agent_assets_dir(agent_dir: impl AsRef<Path>) -> PathBuf {
    agent_dir.as_ref().join(ASSETS_DIRNAME)
}

pub fn get_agent_media_store_roots(agent_dir: impl AsRef<Path>) -> [PathBuf; 2] {
    [get_agent_attachments_dir(&agent_dir), get_agent_assets_dir(agent_dir)]
}
