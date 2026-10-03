use std::collections::BTreeMap;
use std::env;
use std::fs::{self, Metadata};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use serde_json::Value;

use crate::extensions::session::session_diagnostics::{
    SessionDiagnostic, report_session_diagnostic,
};
use crate::host_paths::get_sand_root_dir_with;
use crate::storage::agent_paths::{
    SandInvalidAgentIdError, assert_valid_sand_agent_id, get_sand_agents_root_dir,
};

pub const STORE_FILENAME: &str = "store.db";
pub const CONVERSATION_BLOBS_FILENAME: &str = "conversation-blobs.db";
pub const SAND_CONVERSATION_ROOT_SLOT_ID: &[u8] = b"sand-live-conversation-root-v1__";
pub const STALE_ROOT_CLEANUP_VERSION: u32 = 1;
pub const ACTIVE_AGENT_FILENAME: &str = "active-agent.json";
pub const HIDDEN_ENTRY_REPAIR_VERSION: u32 = 1;
pub const LEGACY_GROUP_MEMBERS_DIRNAME: &str = "members";
pub const CONNECTOR_SECRETS_DIRNAME: &str = "connector-secrets";

static PINNED_STALE_ROOT_GC_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn pin_stale_root_gc(enabled: bool) {
    PINNED_STALE_ROOT_GC_ENABLED.store(enabled, Ordering::Release);
}

pub fn is_stale_root_gc_enabled_from_env(environment: &BTreeMap<String, String>) -> bool {
    match environment
        .get("SAND_STALE_ROOT_GC")
        .map(|value| value.trim().to_ascii_lowercase())
        .as_deref()
    {
        Some("1" | "true" | "on") => true,
        Some("0" | "false" | "off") => false,
        _ => PINNED_STALE_ROOT_GC_ENABLED.load(Ordering::Acquire),
    }
}

pub fn is_stale_root_gc_enabled() -> bool {
    is_stale_root_gc_enabled_from_env(&env::vars().collect())
}

fn default_home_dir() -> PathBuf {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn get_sand_transcripts_dir_with(
    home_dir: &Path,
    argv: &[String],
    environment: &BTreeMap<String, String>,
    cwd: &Path,
) -> PathBuf {
    get_sand_root_dir_with(home_dir, argv, environment, cwd).join("agent-transcripts")
}

pub fn get_sand_transcripts_dir(home_dir: Option<&Path>) -> PathBuf {
    let owned_home;
    let home = match home_dir {
        Some(home) => home,
        None => {
            owned_home = default_home_dir();
            &owned_home
        }
    };
    let environment = env::vars().collect::<BTreeMap<_, _>>();
    let argv = env::args().collect::<Vec<_>>();
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    get_sand_transcripts_dir_with(home, &argv, &environment, &cwd)
}

pub fn get_agent_db_path(
    root_dir: &Path,
    agent_id: &str,
) -> Result<PathBuf, SandInvalidAgentIdError> {
    assert_valid_sand_agent_id(agent_id)?;
    Ok(root_dir.join(agent_id).join(STORE_FILENAME))
}

pub fn get_connector_secrets_root(agents_root_dir: Option<&Path>) -> PathBuf {
    let owned_root;
    let root = match agents_root_dir {
        Some(root) => root,
        None => {
            owned_root = get_sand_agents_root_dir(None);
            &owned_root
        }
    };
    root.parent()
        .unwrap_or_else(|| Path::new(""))
        .join(CONNECTOR_SECRETS_DIRNAME)
}

pub fn stat_if_exists(path: &Path) -> Option<Metadata> {
    match fs::metadata(path) {
        Ok(metadata) => Some(metadata),
        Err(error) => {
            if error.kind() != std::io::ErrorKind::NotFound {
                let agent_id = path
                    .parent()
                    .and_then(Path::file_name)
                    .map(|value| value.to_string_lossy().into_owned())
                    .unwrap_or_default();
                report_session_diagnostic(&SessionDiagnostic {
                    family: "store_db".into(),
                    kind: "path_stat_failed".into(),
                    metadata: BTreeMap::from([
                        ("agentId".into(), Value::String(agent_id)),
                        (
                            "errorClass".into(),
                            Value::String(format!("{:?}", error.kind())),
                        ),
                    ]),
                });
            }
            None
        }
    }
}
