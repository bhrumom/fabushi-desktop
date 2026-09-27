use std::path::{Path, PathBuf};

pub const SAND_STORE_BETTER_CLI_ENV: &str = "SAND_STORE_BETTER_CLI";
pub const BOX_HOME_CATEGORY_NAME: &str = "home";
pub const BOX_HOME_DIR: &str = "/home/box";
pub const BOX_HOME_REL_PREFIX: &str = "home/box";
pub const BOX_STORE_SYNC_INTERVAL_MS: u64 = 2 * 60_000;
pub const BOX_STORE_CHROME_INTERVAL_MS: u64 = 15 * 60_000;
pub const BOX_STORE_DB_DEBOUNCE_MS: u64 = 5_000;
pub const CHROME_SESSION_CHANGE_DEBOUNCE_MS: u64 = 5_000;
pub const BOX_STORE_MANIFEST_RETRY_ATTEMPTS: usize = 3;
pub const BOX_STORE_MANIFEST_RETRY_DELAY_MS: u64 = 100;

pub const BOX_HOME_PRUNE_GUARDED_FOREIGN_TREES: [&str; 5] = [
    "home/box/cli-config",
    "home/box/chrome-profile*",
    "home/box/sand-data",
    "home/box/sand-host",
    "home/box/deps",
];

pub const BOX_HOME_REGENERABLE_JUNK_IGNORE_PATTERNS: [&str; 16] = [
    "/cli-config/",
    "/chrome-profile*/",
    "/sand-data/",
    "/sand-host/",
    "/deps/",
    ".cache/",
    "/.npm/",
    "/.nvm/",
    "/.rustup/",
    "/.cargo/registry/",
    "/.cargo/git/",
    "/.local/share/pnpm/",
    "/.local/share/Trash/",
    "/.local/state/",
    "/.docker/buildx/",
    "*.sock",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxStoreCategory {
    pub name: String,
    pub abs_root: PathBuf,
    pub rel_prefix: String,
    pub excludes: Vec<String>,
    pub idle_only: bool,
    pub stage_only: bool,
    pub contains_agent_store_dbs: bool,
}

pub fn build_box_home_category(abs_root: impl AsRef<Path>) -> BoxStoreCategory {
    BoxStoreCategory {
        name: BOX_HOME_CATEGORY_NAME.into(),
        abs_root: abs_root.as_ref().to_path_buf(),
        rel_prefix: BOX_HOME_REL_PREFIX.into(),
        excludes: BOX_HOME_PRUNE_GUARDED_FOREIGN_TREES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        idle_only: false,
        stage_only: false,
        contains_agent_store_dbs: false,
    }
}

pub fn is_better_cli_home_enabled(better_cli: Option<&str>, user_non_root: Option<&str>) -> bool {
    better_cli == Some("1") && user_non_root == Some("1")
}
