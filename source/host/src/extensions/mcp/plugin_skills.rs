use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::ThreadId;

use serde::Deserialize;
use serde_json::Value;

use super::legacy_live_references::remove_workflow_live_references;
use super::mcp_service::{
    PluginUninstallIntent, PluginUninstallSettlement, is_canonical_uninstall_plugin_id,
};
use super::plugin_skills_cache::{
    PluginAuthBlock, PluginSkillRecord, PluginSkillsCache, PluginSkillsCacheWriteIndex,
    PluginSkillsSnapshot,
    get_plugin_skills_dir, get_plugins_root_dir, read_plugin_skills_cache,
    read_plugin_skills_snapshot, stage_plugin_skills_cache,
};

#[path = "plugin_skills_uninstall_state.rs"]
mod uninstall_state;

use uninstall_state::{
    DurableUninstallImage, UninstallState, load_uninstall_image, persist_uninstall_image,
};

const UNINSTALL_RECOVERY_UNAVAILABLE: &str = "plugin uninstall recovery state is unavailable";

pub const EFFECTIVE_PLUGINS_RPC_TIMEOUT_MS: u64 = 15_000;
pub const CURRENT_USER_RPC_TIMEOUT_MS: u64 = 10_000;
pub const PLUGIN_SKILLS_REFRESH_INTERVAL_MS: u64 = 24 * 60 * 60 * 1_000;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginIdentifier {
    pub source: String,
    pub name: String,
    pub plugin_db_id: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledSkill {
    pub name: Option<String>,
    pub description: Option<String>,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledPlugin {
    pub identifier: PluginIdentifier,
    pub display_name: Option<String>,
    pub load_error: Option<String>,
    pub install_path: PathBuf,
    pub skills: Vec<InstalledSkill>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublisherFacts {
    pub publisher_user_id: Option<u64>,
    pub marketplace_team_id: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginLoadFailure {
    pub plugin_db_id: Option<String>,
    pub plugin_id: Option<String>,
    pub plugin_name: String,
    pub marketplace_name: Option<String>,
    pub error_message: String,
    pub error_kind: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListedCacheKey {
    pub marketplace_slug: String,
    pub plugin_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadedPlugins {
    pub plugins: Vec<InstalledPlugin>,
    pub auth_blocked: Vec<PluginAuthBlock>,
    pub listed_plugin_ids: Vec<String>,
    pub listed_cache_keys: Vec<ListedCacheKey>,
    pub publisher_facts: HashMap<String, PublisherFacts>,
    pub current_user_id: Option<u64>,
}

pub fn skill_records_identity(records: &[PluginSkillRecord]) -> String {
    let mut values = records
        .iter()
        .map(|record| format!("{}@{}", record.id, record.plugin_version))
        .collect::<Vec<_>>();
    values.sort();
    values.join("\n")
}

pub fn skill_name_from_path(relative_path: &str) -> String {
    let segments = relative_path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    if segments.is_empty() {
        return String::new();
    }
    let file_index = segments.len() - 1;
    if file_index > 0 {
        segments[file_index - 1].to_string()
    } else {
        segments[file_index].to_string()
    }
}

pub fn plugin_version_of(identifier: &PluginIdentifier) -> String {
    identifier.version.clone().unwrap_or_default()
}

fn plugin_db_id(identifier: &PluginIdentifier) -> Option<&str> {
    matches!(
        identifier.source.as_str(),
        "cursor-first-party" | "cursor-third-party"
    )
    .then_some(identifier.plugin_db_id.as_deref())
    .flatten()
}

fn is_canonical_plugin_id(id: &str) -> bool {
    !id.is_empty() && id.trim() == id
}

fn validate_loaded_plugins(loaded: &LoadedPlugins) -> Result<(), String> {
    let mut listed = HashSet::new();
    for id in &loaded.listed_plugin_ids {
        if !is_canonical_plugin_id(id) {
            return Err("plugin skills loader returned an invalid listed plugin ID".into());
        }
        if !listed.insert(id.as_str()) {
            return Err("plugin skills loader returned a duplicate listed plugin ID".into());
        }
    }
    let mut plugin_ids = HashSet::new();
    for plugin in &loaded.plugins {
        if !matches!(
            plugin.identifier.source.as_str(),
            "cursor-first-party" | "cursor-third-party"
        ) {
            continue;
        }
        let id = plugin_db_id(&plugin.identifier)
            .filter(|id| is_canonical_plugin_id(id))
            .ok_or_else(|| "plugin skills loader returned an invalid plugin ID".to_string())?;
        if !listed.contains(id) {
            return Err("plugin skills loader returned an unlisted plugin ID".into());
        }
        if !plugin_ids.insert(id) {
            return Err("plugin skills loader returned a duplicate plugin ID".into());
        }
    }
    for block in &loaded.auth_blocked {
        if !is_canonical_plugin_id(&block.plugin_id) {
            return Err("plugin skills loader returned an invalid blocked plugin ID".into());
        }
        if !listed.contains(block.plugin_id.as_str()) {
            return Err("plugin skills loader returned an unlisted blocked plugin ID".into());
        }
    }
    Ok(())
}

fn auth_blocks_identity(blocks: &[PluginAuthBlock]) -> Vec<&str> {
    let mut ids = blocks
        .iter()
        .map(|block| block.plugin_id.as_str())
        .collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();
    ids
}

fn clamp_line(value: &str, max: usize) -> String {
    value
        .replace(['\r', '\n'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(max)
        .collect()
}

fn slugify(value: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for ch in value.to_ascii_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            dash = false;
        } else if !dash && !out.is_empty() {
            out.push('-');
            dash = true;
        }
        if out.len() >= 64 {
            break;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "workflow".into()
    } else {
        out
    }
}

pub fn plugin_contents_to_skill_records(
    plugins: &[InstalledPlugin],
    publisher_facts: &HashMap<String, PublisherFacts>,
) -> Vec<PluginSkillRecord> {
    let mut records = Vec::new();
    let mut used_ids = HashSet::new();
    for plugin in plugins {
        if plugin.load_error.is_some() || plugin.install_path.as_os_str().is_empty() {
            continue;
        }
        let Some(plugin_id) = plugin_db_id(&plugin.identifier) else {
            continue;
        };
        let plugin_name = plugin
            .display_name
            .as_deref()
            .filter(|value| !value.is_empty())
            .unwrap_or(&plugin.identifier.name);
        for skill in &plugin.skills {
            let raw_name = skill
                .name
                .as_deref()
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| skill_name_from_path(&skill.path));
            let name = clamp_line(&raw_name, 80);
            if name.is_empty() {
                continue;
            }
            let slug = slugify(&name);
            let base = format!("plugin-{plugin_id}-{slug}");
            let mut id = base.clone();
            let mut suffix = 2usize;
            while used_ids.contains(&id) {
                id = format!("{base}-{suffix}");
                suffix += 1;
            }
            used_ids.insert(id.clone());
            let facts = publisher_facts.get(plugin_id).copied().unwrap_or_default();
            records.push(PluginSkillRecord {
                id,
                plugin_id: plugin_id.to_string(),
                plugin_name: plugin_name.to_string(),
                name,
                description: clamp_line(skill.description.as_deref().unwrap_or_default(), 1_536),
                file_path: plugin
                    .install_path
                    .join(&skill.path)
                    .to_string_lossy()
                    .into_owned(),
                plugin_version: plugin_version_of(&plugin.identifier),
                install_path: plugin.install_path.to_string_lossy().into_owned(),
                skill_relative_path: skill.path.clone(),
                publisher_user_id: facts.publisher_user_id,
                marketplace_team_id: facts.marketplace_team_id,
            });
        }
    }
    records
}

pub fn plugin_auth_blocks_from_failures(failures: &[PluginLoadFailure]) -> Vec<PluginAuthBlock> {
    let mut blocks = Vec::new();
    let mut seen = HashSet::new();
    for failure in failures {
        // Error prose is not an authorization oracle: local cache/filesystem
        // failures can contain "permission denied" too. Only the typed clone
        // classification from the marketplace loader may block remote access.
        let auth_failure = failure.error_kind.as_deref() == Some("user_git_access");
        if !auth_failure {
            continue;
        }
        let Some(plugin_id) = failure
            .plugin_db_id
            .as_deref()
            .filter(|id| is_canonical_plugin_id(id))
        else {
            continue;
        };
        if !seen.insert(plugin_id) {
            continue;
        }
        blocks.push(PluginAuthBlock {
            plugin_id: plugin_id.to_string(),
            plugin_name: failure.plugin_name.clone(),
            marketplace_name: failure.marketplace_name.clone(),
        });
    }
    blocks
}

fn is_within(parent: &Path, child: &Path) -> bool {
    child.strip_prefix(parent).is_ok()
}

pub fn prune_uninstalled_plugin_dirs(
    cache_root: &Path,
    listed: &[ListedCacheKey],
    indexed_file_paths: &[String],
) {
    let keep = listed
        .iter()
        .map(|key| cache_root.join(&key.marketplace_slug).join(&key.plugin_id))
        .collect::<HashSet<_>>();
    let Ok(slugs) = fs::read_dir(cache_root) else {
        return;
    };
    for slug in slugs.filter_map(Result::ok) {
        if !slug.file_type().is_ok_and(|kind| kind.is_dir()) {
            continue;
        }
        let Ok(plugins) = fs::read_dir(slug.path()) else {
            continue;
        };
        for plugin in plugins.filter_map(Result::ok) {
            if !plugin.file_type().is_ok_and(|kind| kind.is_dir()) {
                continue;
            }
            let dir = plugin.path();
            if keep.contains(&dir)
                || indexed_file_paths
                    .iter()
                    .any(|path| is_within(&dir, Path::new(path)))
            {
                continue;
            }
            let _ = fs::remove_dir_all(dir);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginSkillsSyncEvent {
    pub trigger: String,
    pub outcome: String,
    pub changed: bool,
    pub skill_count: usize,
    pub duration_ms: i64,
    pub error_class: Option<String>,
}

pub trait PluginSkillsLoader: Send + Sync {
    fn load(&self) -> Result<LoadedPlugins, String>;
}

/// Read-only negative admission from the existing Plugin Skills owner. A false
/// result is not a grant: consumers must also enforce committed eligibility and
/// their existing publisher/account rules.
pub trait PluginSkillsAdmissionQuery: Send + Sync {
    fn is_denied(&self, plugin_id: &str) -> bool;
}

pub type PluginSkillsAdmissionReader = Arc<dyn PluginSkillsAdmissionQuery>;

#[derive(Default)]
struct PluginSkillsLiveAdmission {
    state: Mutex<PluginSkillsAdmissionState>,
}

#[derive(Default)]
struct PluginSkillsAdmissionState {
    denied: BTreeMap<String, PluginAuthBlock>,
    durable: DurableUninstallImage,
    recovery_fault: bool,
    // Each observation replaces this opaque identity. An active pass keeps its
    // Arc alive, preventing identity reuse without a counter or overflow case.
    uninstall_observation: Arc<()>,
}

impl PluginSkillsAdmissionQuery for PluginSkillsLiveAdmission {
    fn is_denied(&self, plugin_id: &str) -> bool {
        self.state
            .lock()
            .map(|state| {
                state.recovery_fault
                    || state.denied.contains_key(plugin_id)
                    || state.durable.state.confirmed.contains(plugin_id)
                    || state.durable.state.pending.contains_key(plugin_id)
            })
            .unwrap_or(true)
    }
}

type SyncResult = Result<Vec<PluginSkillRecord>, String>;

#[derive(Default)]
struct SyncPass {
    // Lock ordering: service state, then pass result. A pass outlives its slot
    // so every coalesced caller can observe the same outcome.
    result: Mutex<Option<SyncResult>>,
}

struct PendingSync {
    trigger: String,
    pass: Arc<SyncPass>,
}

#[derive(Default)]
struct ServiceState {
    disposed: bool,
    replacement_claimed: bool,
    active: Option<Arc<SyncPass>>,
    active_thread: Option<ThreadId>,
    pending: Option<PendingSync>,
}

struct ActiveSyncGuard<'a> {
    service: &'a SandPluginSkillsService,
    pass: Arc<SyncPass>,
    completed: bool,
}

impl Drop for ActiveSyncGuard<'_> {
    fn drop(&mut self) {
        if !self.completed {
            // A loader/reporter panic must not orphan this pass's other callers
            // or leave every future refresh waiting on a dead executor.
            self.service.complete_pass(
                &self.pass,
                Err("plugin skills sync aborted before completion".into()),
            );
        }
    }
}

pub struct SandPluginSkillsService {
    sand_root_dir: PathBuf,
    loader: Arc<dyn PluginSkillsLoader>,
    now: Arc<dyn Fn() -> i64 + Send + Sync>,
    report_sync: Option<Arc<dyn Fn(PluginSkillsSyncEvent) + Send + Sync>>,
    publication: Mutex<()>,
    admission: Arc<PluginSkillsLiveAdmission>,
    state: Mutex<ServiceState>,
    changed: Condvar,
    #[cfg(test)]
    test_hooks: sync_tests::ServiceTestHooks,
}

impl SandPluginSkillsService {
    pub fn new(sand_root_dir: impl Into<PathBuf>, loader: Arc<dyn PluginSkillsLoader>) -> Self {
        let sand_root_dir = sand_root_dir.into();
        // Construction may read durable state. Production composition prepares
        // this fresh candidate outside its binding lock and rechecks the owner
        // before installation; same-root successors preserve the live reader.
        let state = match load_uninstall_image(&get_plugin_skills_dir(&sand_root_dir)) {
            Ok(durable) => PluginSkillsAdmissionState {
                durable,
                ..PluginSkillsAdmissionState::default()
            },
            Err(_) => PluginSkillsAdmissionState {
                recovery_fault: true,
                ..PluginSkillsAdmissionState::default()
            },
        };
        Self::with_admission(
            sand_root_dir,
            loader,
            Arc::new(PluginSkillsLiveAdmission { state: Mutex::new(state) }),
        )
    }

    fn with_admission(
        sand_root_dir: PathBuf,
        loader: Arc<dyn PluginSkillsLoader>,
        admission: Arc<PluginSkillsLiveAdmission>,
    ) -> Self {
        Self {
            sand_root_dir,
            loader,
            now: Arc::new(|| chrono::Utc::now().timestamp_millis()),
            report_sync: None,
            publication: Mutex::new(()),
            admission,
            state: Mutex::new(ServiceState::default()),
            changed: Condvar::new(),
            #[cfg(test)]
            test_hooks: sync_tests::ServiceTestHooks::default(),
        }
    }

    pub fn admission_reader(&self) -> PluginSkillsAdmissionReader {
        self.admission.clone()
    }

    /// Reserve durable negative admission before any external uninstall call.
    /// This ticket is custody, not a claim that the remote mutation occurred.
    pub fn begin_uninstall(&self, plugin_id: &str) -> Result<PluginUninstallIntent, String> {
        if !is_canonical_uninstall_plugin_id(plugin_id) {
            return Err("plugin uninstall requires a canonical plugin ID".into());
        }
        let _publication = self
            .publication
            .lock()
            .map_err(|_| "plugin skills publication poisoned".to_string())?;
        self.require_active_uninstall_owner()?;
        let expected = self.uninstall_image()?;
        if expected.state.pending.contains_key(plugin_id) {
            return Err("plugin uninstall already has an unresolved intent".into());
        }
        let operation_id = loop {
            let candidate = uuid::Uuid::new_v4().to_string();
            if !expected.state.pending.values().any(|ticket| ticket == &candidate) {
                break candidate;
            }
        };
        let intent = PluginUninstallIntent {
            plugin_id: plugin_id.to_string(),
            operation_id,
        };
        let mut next = expected.state.clone();
        next.pending.insert(intent.plugin_id.clone(), intent.operation_id.clone());
        self.persist_uninstall_transition(&expected, &next)?;
        Ok(intent)
    }

    pub fn settle_uninstall(
        &self,
        intent: &PluginUninstallIntent,
        settlement: PluginUninstallSettlement,
    ) -> Result<(), String> {
        let _publication = self
            .publication
            .lock()
            .map_err(|_| "plugin skills publication poisoned".to_string())?;
        self.require_active_uninstall_owner()?;
        let expected = self.uninstall_image()?;
        if expected.state.pending.get(&intent.plugin_id) != Some(&intent.operation_id) {
            return Err("plugin uninstall settlement does not match the current intent".into());
        }
        let mut next = expected.state.clone();
        next.pending.remove(&intent.plugin_id);
        if settlement == PluginUninstallSettlement::Removed {
            next.confirmed.insert(intent.plugin_id.clone());
        }
        // NotRemoved is a known reply, not proof of no effect. Both negative
        // settlements remove only this ticket, retaining any earlier refusal.
        self.persist_uninstall_transition(&expected, &next)
    }

    fn require_active_uninstall_owner(&self) -> Result<(), String> {
        if self
            .state
            .lock()
            .map_err(|_| "plugin skills state poisoned".to_string())?
            .disposed
        {
            return Err("plugin skills owner is disposed; uninstall state was not changed".into());
        }
        Ok(())
    }

    fn uninstall_image(&self) -> Result<DurableUninstallImage, String> {
        let live = self.admission.state.lock()
            .map_err(|_| "plugin skills live admission poisoned".to_string())?;
        if live.recovery_fault {
            return Err(UNINSTALL_RECOVERY_UNAVAILABLE.into());
        }
        Ok(live.durable.clone())
    }

    // Caller holds P, but no service-state/admission guard during filesystem
    // work. A failed write may already be visible, so latch refusal until a
    // fresh owner can inspect a repaired or completed image. Never roll it back.
    fn persist_uninstall_transition(
        &self,
        expected: &DurableUninstallImage,
        next: &UninstallState,
    ) -> Result<(), String> {
        let durable = match persist_uninstall_image(
            &get_plugin_skills_dir(&self.sand_root_dir), expected, next,
        ) {
            Ok(durable) => durable,
            Err(_) => {
                let mut live = self.admission.state.lock()
                    .map_err(|_| "plugin skills live admission poisoned".to_string())?;
                live.recovery_fault = true;
                live.uninstall_observation = Arc::new(());
                return Err(UNINSTALL_RECOVERY_UNAVAILABLE.into());
            }
        };
        let mut live = self
            .admission
            .state
            .lock()
            .map_err(|_| "plugin skills live admission poisoned".to_string())?;
        live.durable = durable;
        live.uninstall_observation = Arc::new(());
        Ok(())
    }

    pub fn uninstall_recovery_status(&self) -> Option<Value> {
        let Ok(live) = self.admission.state.lock() else {
            return Some(serde_json::json!({
                "confirmedPluginIds": [], "pending": [],
                "fault": "uninstall-state-unavailable",
            }));
        };
        if !live.recovery_fault
            && live.durable.state.confirmed.is_empty()
            && live.durable.state.pending.is_empty()
        {
            return None;
        }
        let mut status = serde_json::json!({
            "confirmedPluginIds": live.durable.state.confirmed.iter().collect::<Vec<_>>(),
            "pending": live.durable.state.pending.iter().map(|(plugin_id, operation_id)| {
                serde_json::json!({"pluginId": plugin_id, "operationId": operation_id})
            }).collect::<Vec<_>>(),
        });
        if live.recovery_fault {
            status["fault"] = Value::String("uninstall-state-unavailable".into());
        }
        Some(status)
    }

    pub(crate) fn sand_root_dir(&self) -> &Path {
        &self.sand_root_dir
    }

    pub(crate) fn successor_with_loader(
        &self,
        loader: Arc<dyn PluginSkillsLoader>,
    ) -> Result<Self, String> {
        self.retire_for_replacement()?;
        Ok(Self::with_admission(
            self.sand_root_dir.clone(),
            loader,
            Arc::clone(&self.admission),
        ))
    }

    pub(crate) fn retire_for_replacement(&self) -> Result<(), String> {
        // The predecessor's publication lock also orders committed cleanup.
        // No successor may begin materialization before this one-shot claim.
        let _publication = self
            .publication
            .lock()
            .map_err(|_| "plugin skills publication poisoned".to_string())?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| "plugin skills state poisoned".to_string())?;
        if state.replacement_claimed {
            return Err("plugin skills replacement already claimed".into());
        }
        state.disposed = true;
        state.replacement_claimed = true;
        state.pending = None;
        self.changed.notify_all();
        Ok(())
    }

    pub fn with_now(mut self, now: Arc<dyn Fn() -> i64 + Send + Sync>) -> Self {
        self.now = now;
        self
    }

    pub fn with_reporter(
        mut self,
        reporter: Arc<dyn Fn(PluginSkillsSyncEvent) + Send + Sync>,
    ) -> Self {
        self.report_sync = Some(reporter);
        self
    }

    pub fn current_index(&self) -> Option<PluginSkillsCache> {
        read_plugin_skills_cache(get_plugin_skills_dir(&self.sand_root_dir))
    }

    pub fn current(&self) -> Vec<PluginSkillRecord> {
        let records = read_plugin_skills_snapshot(get_plugin_skills_dir(&self.sand_root_dir))
            .map(|snapshot| snapshot.eligible_skills().cloned().collect())
            .unwrap_or_default();
        self.filter_live_records(records)
    }

    pub fn current_auth_blocked(&self) -> Vec<PluginAuthBlock> {
        self.current_index()
            .map(|index| index.auth_blocked)
            .unwrap_or_default()
    }

    pub fn remove_live_references(&self, source_urls: &[String]) {
        let _ = remove_workflow_live_references(&self.sand_root_dir, source_urls);
    }

    pub fn dispose(&self) {
        let _publication = self.publication.lock().expect("plugin skills publication poisoned");
        let mut state = self.state.lock().expect("plugin skills state poisoned");
        state.disposed = true;
        state.pending = None;
        self.changed.notify_all();
    }

    fn filter_live_records(&self, mut records: Vec<PluginSkillRecord>) -> Vec<PluginSkillRecord> {
        records.retain(|record| !self.admission.is_denied(&record.plugin_id));
        records
    }

    fn filter_live_result(&self, result: SyncResult) -> SyncResult {
        result.map(|records| self.filter_live_records(records))
    }

    fn uninstall_observation(&self) -> Result<Arc<()>, String> {
        let live = self.admission
            .state
            .lock()
            .map_err(|_| "plugin skills live admission poisoned".to_string())?;
        if live.recovery_fault {
            return Err(UNINSTALL_RECOVERY_UNAVAILABLE.into());
        }
        Ok(Arc::clone(&live.uninstall_observation))
    }

    fn require_current_uninstall_observation(&self, observation: &Arc<()>) -> Result<(), String> {
        let live = self
            .admission
            .state
            .lock()
            .map_err(|_| "plugin skills live admission poisoned".to_string())?;
        if live.recovery_fault {
            return Err(UNINSTALL_RECOVERY_UNAVAILABLE.into());
        }
        if !Arc::ptr_eq(&live.uninstall_observation, observation) {
            return Err("plugin skills sync superseded by an uninstall observation".into());
        }
        Ok(())
    }

    fn import_live_refusals_if_active(
        &self,
        blocks: &[PluginAuthBlock],
        observation: &Arc<()>,
    ) -> Result<Option<(Vec<PluginAuthBlock>, HashSet<String>)>, String> {
        let _publication = self
            .publication
            .lock()
            .map_err(|_| "plugin skills publication poisoned".to_string())?;
        if self
            .state
            .lock()
            .map_err(|_| "plugin skills state poisoned".to_string())?
            .disposed
        {
            return Ok(None);
        }
        // The complete loader aggregate was validated before reaching this
        // boundary. Never hold ServiceState while reading or changing admission.
        let mut live = self
            .admission
            .state
            .lock()
            .map_err(|_| "plugin skills live admission poisoned".to_string())?;
        if live.recovery_fault {
            return Err(UNINSTALL_RECOVERY_UNAVAILABLE.into());
        }
        if !Arc::ptr_eq(&live.uninstall_observation, observation) {
            return Err("plugin skills sync superseded by an uninstall observation".into());
        }
        for block in blocks {
            live.denied.insert(block.plugin_id.clone(), block.clone());
        }
        Ok(Some((
            live.denied.values().cloned().collect(),
            live.durable.state.confirmed.iter()
                .chain(live.durable.state.pending.keys()).cloned().collect(),
        )))
    }

    fn publish_snapshot_if_active(
        &self,
        staged: super::plugin_skills_cache::StagedPluginSkillsCache,
        successful_ids: &HashSet<String>,
        denied_ids: &HashSet<String>,
        observation: &Arc<()>,
    ) -> Result<Option<(PluginSkillsSnapshot, bool, Arc<()>, HashSet<String>)>, String> {
        // This service has one pass writer. Keep rename, strict readback and
        // positive release in one publication interval; a successor cannot be
        // constructed until it has claimed this predecessor's same lock.
        let _publication = self
            .publication
            .lock()
            .map_err(|_| "plugin skills publication poisoned".to_string())?;
        if self
            .state
            .lock()
            .map_err(|_| "plugin skills state poisoned".to_string())?
            .disposed
        {
            return Ok(None);
        }
        // Observations use this same publication lock. Check before rename,
        // then keep admission unlocked during filesystem work and readback.
        self.require_current_uninstall_observation(observation)?;
        staged.publish()?;
        #[cfg(test)]
        sync_tests::pause_at(&self.test_hooks.after_rename);
        let committed = read_plugin_skills_snapshot(get_plugin_skills_dir(&self.sand_root_dir))
            .ok_or_else(|| "could not read committed plugin skills snapshot".to_string())?;
        // The cache has crossed its file/directory barriers and strict readback.
        // Durable refusal release follows it, never precedes it. No admission
        // guard spans the journal write or its read-before-replace check.
        let expected = self.uninstall_image()?;
        let mut next = expected.state.clone();
        for id in successful_ids {
            if !denied_ids.contains(id) && !next.pending.contains_key(id) {
                next.confirmed.remove(id);
            }
        }
        let durable_release = next != expected.state;
        if durable_release {
            self.persist_uninstall_transition(&expected, &next)?;
        }
        let mut live = self
            .admission
            .state
            .lock()
            .map_err(|_| "plugin skills live admission poisoned".to_string())?;
        // A failed denial publication can leave the old allowed cache intact;
        // releasing its live refusal is still an observable admission change.
        let mut released_live_refusal = durable_release;
        for id in successful_ids {
            if !denied_ids.contains(id) {
                released_live_refusal |= live.denied.remove(id).is_some();
            }
        }
        let committed_observation = Arc::clone(&live.uninstall_observation);
        // Reporting describes this publication's admitted identities, even if
        // a later observation changes admission before its callback runs. Keep
        // the raw cache intact; unresolved tickets exclude both comparison sides.
        let pending_for_report = live.durable.state.pending.keys().cloned().collect();
        Ok(Some((committed, released_live_refusal, committed_observation, pending_for_report)))
    }

    fn prune_committed_if_current(
        &self,
        listed: &[ListedCacheKey],
        retained_paths: &[String],
        observation: &Arc<()>,
    ) -> Result<(), String> {
        let _publication = self
            .publication
            .lock()
            .map_err(|_| "plugin skills publication poisoned".to_string())?;
        if self
            .state
            .lock()
            .map_err(|_| "plugin skills state poisoned".to_string())?
            .replacement_claimed
        {
            return Ok(());
        }
        // A commit can precede a confirmed uninstall while its cleanup has not
        // begun. Preserve that real commit/report, but never prune using the old
        // image after the new observation. An entered prune holds P first.
        if !Arc::ptr_eq(&self.uninstall_observation()?, observation) {
            return Ok(());
        }
        if !self.uninstall_image()?.state.pending.is_empty() {
            // A fresh listing cannot prove a pending remote operation quiescent.
            // Keep materialized directories until custody is actually settled.
            return Ok(());
        }
        // Plain dispose does not revoke cleanup from an already committed pass.
        // Drop state before pure filesystem work; a replacement must wait for P.
        #[cfg(test)]
        sync_tests::pause_at(&self.test_hooks.prune_locked);
        prune_uninstalled_plugin_dirs(
            &get_plugins_root_dir(&self.sand_root_dir).join("cache"),
            listed,
            retained_paths,
        );
        Ok(())
    }

    pub fn sync(&self, trigger: &str) -> SyncResult {
        let mut state = self.state.lock().expect("plugin skills state poisoned");
        if state.disposed {
            drop(state);
            return Ok(self.current());
        }
        if state.active_thread == Some(std::thread::current().id()) {
            // Unlike a JS promise, this blocking API cannot await the callback
            // that is currently calling it. Fail explicitly instead of hanging.
            return Err("recursive plugin skills sync cannot wait on its own pass".into());
        }
        if state.active.is_none() && state.pending.is_none() {
            let pass = Arc::new(SyncPass::default());
            state.active = Some(Arc::clone(&pass));
            state.active_thread = Some(std::thread::current().id());
            drop(state);
            return self.execute_pass(pass, trigger);
        }

        // Exactly one follow-up is shared by all callers arriving during a
        // pass. New arrivals must not overtake a follow-up awaiting its worker.
        let pending = state.pending.get_or_insert_with(|| PendingSync {
            trigger: trigger.to_string(),
            pass: Arc::new(SyncPass::default()),
        });
        pending.trigger = trigger.to_string();
        let pass = Arc::clone(&pending.pass);
        self.changed.notify_all();
        loop {
            let result = pass
                .result
                .lock()
                .expect("plugin skills pass poisoned")
                .clone();
            if let Some(result) = result {
                drop(state);
                #[cfg(test)]
                sync_tests::pause_at(&self.test_hooks.coalesced_result);
                return self.filter_live_result(result);
            }
            if state.disposed {
                drop(state);
                return Ok(self.current());
            }
            if state.active.is_none()
                && state
                    .pending
                    .as_ref()
                    .is_some_and(|pending| Arc::ptr_eq(&pending.pass, &pass))
            {
                let pending = state.pending.take().expect("pending pass exists");
                state.active = Some(Arc::clone(&pass));
                state.active_thread = Some(std::thread::current().id());
                drop(state);
                return self.execute_pass(pass, &pending.trigger);
            }
            state = self
                .changed
                .wait(state)
                .expect("plugin skills state poisoned");
        }
    }

    fn execute_pass(&self, pass: Arc<SyncPass>, trigger: &str) -> SyncResult {
        let mut guard = ActiveSyncGuard {
            service: self,
            pass,
            completed: false,
        };
        // All loading, cache IO and callbacks are outside the state/result locks.
        let result = self.run_pass(trigger);
        self.complete_pass(&guard.pass, result.clone());
        guard.completed = true;
        self.filter_live_result(result)
    }

    fn complete_pass(&self, pass: &Arc<SyncPass>, result: SyncResult) {
        let mut state = self.state.lock().expect("plugin skills state poisoned");
        *pass.result.lock().expect("plugin skills pass poisoned") = Some(result);
        if state
            .active
            .as_ref()
            .is_some_and(|active| Arc::ptr_eq(active, pass))
        {
            state.active = None;
            state.active_thread = None;
        }
        self.changed.notify_all();
    }

    fn run_pass(&self, trigger: &str) -> Result<Vec<PluginSkillRecord>, String> {
        let started = (self.now)();
        // Capture for each actual load, including a queued follow-up, rather
        // than when a caller first joined a pending/coalesced pass.
        let observation = self.uninstall_observation()?;
        let loaded = match self.loader.load().and_then(|loaded| {
            validate_loaded_plugins(&loaded)?;
            Ok(loaded)
        }) {
            Ok(value) => value,
            Err(error) => {
                self.require_current_uninstall_observation(&observation)?;
                if let Some(report) = &self.report_sync {
                    report(PluginSkillsSyncEvent {
                        trigger: trigger.into(),
                        outcome: "failed".into(),
                        changed: false,
                        skill_count: self.current().len(),
                        duration_ms: (self.now)().saturating_sub(started),
                        error_class: Some("load_failed".into()),
                    });
                }
                return Err(error);
            }
        };
        let Some((live_blocks, uninstall_refusals)) =
            self.import_live_refusals_if_active(&loaded.auth_blocked, &observation)?
        else {
            return Ok(self.current());
        };
        let cache_dir = get_plugin_skills_dir(&self.sand_root_dir);
        let previous = read_plugin_skills_snapshot(&cache_dir);
        let previous_records = previous
            .as_ref()
            .map(|snapshot| snapshot.eligible_skills().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        let previous_blocks = previous
            .as_ref()
            .map(|snapshot| snapshot.canonical_auth_blocked())
            .unwrap_or_default();
        let listed = loaded
            .listed_plugin_ids
            .iter()
            .cloned()
            .collect::<HashSet<_>>();
        let loaded_ids = loaded
            .plugins
            .iter()
            .filter(|plugin| {
                plugin.load_error.is_none() && !plugin.install_path.as_os_str().is_empty()
            })
            .filter_map(|plugin| plugin_db_id(&plugin.identifier).map(str::to_string))
            .collect::<HashSet<_>>();

        // Current denial wins over success. Retained live facts also constrain
        // fallback when a prior failed publication left the disk cache unchanged.
        // Only listed, unresolved denials belong in the new committed snapshot.
        let mut denied = HashSet::new();
        let mut auth_blocked = Vec::new();
        for block in &loaded.auth_blocked {
            if denied.insert(block.plugin_id.clone()) {
                auth_blocked.push(block.clone());
            }
        }
        for block in previous_blocks.iter().chain(live_blocks.iter()) {
            if listed.contains(&block.plugin_id)
                && !loaded_ids.contains(&block.plugin_id)
                && denied.insert(block.plugin_id.clone())
            {
                auth_blocked.push(block.clone());
            }
        }
        let mut records =
            plugin_contents_to_skill_records(&loaded.plugins, &loaded.publisher_facts);
        records.retain(|record| {
            listed.contains(&record.plugin_id) && !denied.contains(&record.plugin_id)
        });
        records.extend(
            previous_records
                .iter()
                .filter(|record| {
                    listed.contains(&record.plugin_id)
                        && !loaded_ids.contains(&record.plugin_id)
                        && !denied.contains(&record.plugin_id)
                        && !uninstall_refusals.contains(&record.plugin_id)
                })
                .cloned(),
        );

        let values = records
            .iter()
            .map(|record| serde_json::to_value(record).expect("plugin skill record serializes"))
            .collect::<Vec<Value>>();
        let staged = stage_plugin_skills_cache(
            &cache_dir,
            &PluginSkillsCacheWriteIndex {
                // Publisher identity belongs to this successful pass. Unknown
                // current identity must not inherit an old account's edit grant.
                current_user_id: Some(loaded.current_user_id),
                skills: values,
                auth_blocked: Some(auth_blocked),
            },
            || (self.now)() as f64,
        )?;
        // Sample the callback before the publication fence. It may reenter or
        // dispose the service; no callback runs while publication is locked.
        let duration_ms = (self.now)().saturating_sub(started);
        let Some((committed, released_live_refusal, committed_observation, pending_for_report)) =
            self.publish_snapshot_if_active(staged, &loaded_ids, &denied, &observation)?
        else {
            return Ok(self.current());
        };
        let records = committed.eligible_skills().cloned().collect::<Vec<_>>();
        #[cfg(test)]
        sync_tests::pause_at(&self.test_hooks.after_commit);
        // A partial materialization may still publish denials or safe fallback,
        // but cannot prove that old materialized directories are uninstalled.
        if listed.is_subset(&loaded_ids) && denied.is_empty() {
            // SDK cache directory names may differ from raw listing keys. A
            // successful plugin with no skills still owns its actual install path.
            let retained_paths = records
                .iter()
                .map(|record| record.file_path.clone())
                .chain(
                    loaded
                        .plugins
                        .iter()
                        .filter(|plugin| {
                            plugin_db_id(&plugin.identifier).is_some_and(|id| {
                                listed.contains(id) && loaded_ids.contains(id)
                            })
                        })
                        .map(|plugin| plugin.install_path.to_string_lossy().into_owned()),
                )
                .collect::<Vec<_>>();
            self.prune_committed_if_current(
                &loaded.listed_cache_keys, &retained_paths, &committed_observation,
            )?;
        }
        if let Some(report) = &self.report_sync {
            let admitted_records = records.iter()
                .filter(|record| !pending_for_report.contains(&record.plugin_id))
                .cloned().collect::<Vec<_>>();
            let previous_admitted_records = previous_records.iter()
                .filter(|record| !pending_for_report.contains(&record.plugin_id))
                .cloned().collect::<Vec<_>>();
            let changed = previous.is_none()
                || released_live_refusal
                || skill_records_identity(&previous_admitted_records)
                    != skill_records_identity(&admitted_records)
                || previous.as_ref().and_then(|value| value.index.current_user_id)
                    != committed.index.current_user_id
                || auth_blocks_identity(previous_blocks)
                    != auth_blocks_identity(committed.canonical_auth_blocked());
            report(PluginSkillsSyncEvent {
                trigger: trigger.into(),
                outcome: "ok".into(),
                changed,
                skill_count: admitted_records.len(),
                duration_ms,
                error_class: None,
            });
        }
        Ok(records)
    }
}

#[cfg(test)]
#[path = "plugin_skills_sync_tests.rs"]
mod sync_tests;
