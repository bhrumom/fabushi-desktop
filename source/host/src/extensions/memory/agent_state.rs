use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value};

use crate::agents::agent_avatar::{
    AVATAR_MAX_BYTES, CANONICAL_AVATAR_FILENAME, invalidate_avatar_data_url_cache,
    list_conventional_avatar_filenames, sniff_avatar_mime_type,
};
use crate::agents::agent_profile::{
    SandAgentProfile, get_sand_profile_path, read_sand_profile_file, write_sand_profile_file,
};
use crate::agents::settings_file::{get_sand_settings_path, write_sand_settings_file};
use crate::automations::automation::{AutomationRecord, AutomationSpec, describe_trigger};
use crate::extensions::session::channel_store::{FileChannelStore, get_agent_channels_dir};
use crate::storage::folder_id::is_safe_folder_id;
use crate::workflows::workflow_library::{WorkflowSpec, get_global_workflows_dir};
use crate::workflows::workflow_store::FileWorkflowStore;
use super::memory_service::{
    FileMemoryStore, MemoryKind, get_agent_memory_dir, get_project_dir,
    get_project_memory_shard_dir, get_user_memory_shard_dir, normalize_memory_content,
    project_dir_exists,
};
use super::project_membership::AgentProjectMembership;

pub const MEMORY_NOTE_PREFIX: &str = "Note: ";

pub type AgentStateClock = Arc<dyn Fn() -> i64 + Send + Sync + 'static>;
pub type AvatarChanged = Arc<dyn Fn() + Send + Sync + 'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryTier { Profile, Note, Log }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryScope { Agent, User, Project }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateWriteResult { pub ok: bool, pub message: String }

impl StateWriteResult {
    pub fn success(message: impl Into<String>) -> Self { Self { ok: true, message: message.into() } }
    pub fn failure(message: impl Into<String>) -> Self { Self { ok: false, message: message.into() } }
}

pub struct SandAgentState {
    agent_id: String,
    agent_dir: PathBuf,
    sand_root: PathBuf,
    memory: FileMemoryStore,
    membership: AgentProjectMembership,
    channels: FileChannelStore,
    workflows: FileWorkflowStore,
    now: AgentStateClock,
    on_avatar_changed: Option<AvatarChanged>,
}

impl SandAgentState {
    pub fn new(sand_root: impl Into<PathBuf>, agent_id: impl Into<String>) -> Result<Self, String> {
        let sand_root = sand_root.into();
        let agent_id = agent_id.into();
        if agent_id.trim() != agent_id || !is_safe_folder_id(&agent_id) {
            return Err(format!("invalid agent id: {agent_id}"));
        }
        let agent_dir = sand_root.join("agents").join(&agent_id);
        Ok(Self {
            memory: FileMemoryStore::new(get_agent_memory_dir(&agent_dir)),
            membership: AgentProjectMembership::new(&agent_dir),
            channels: FileChannelStore::new(get_agent_channels_dir(&agent_dir)),
            workflows: FileWorkflowStore::new(&agent_dir, get_global_workflows_dir(&sand_root)),
            agent_id, agent_dir, sand_root,
            now: Arc::new(system_now_ms),
            on_avatar_changed: None,
        })
    }

    pub fn with_clock(mut self, clock: AgentStateClock) -> Self { self.now = clock; self }
    pub fn with_avatar_changed(mut self, callback: AvatarChanged) -> Self { self.on_avatar_changed = Some(callback); self }
    pub fn agent_dir(&self) -> &Path { &self.agent_dir }

    pub fn write_memory(&self, content: &str, tier: MemoryTier, scope: MemoryScope, project: Option<&str>) -> StateWriteResult {
        let (store, label) = match self.memory_store(scope, project) { Ok(value) => value, Err(result) => return result };
        let payload = match tier {
            MemoryTier::Note => format!("{MEMORY_NOTE_PREFIX}{}", content.trim()),
            _ => content.trim().to_string(),
        };
        let kind = match tier { MemoryTier::Profile => MemoryKind::Profile, MemoryTier::Note | MemoryTier::Log => MemoryKind::Log };
        match store.add_memory(&payload, (self.now)(), kind) {
            Ok(Some(record)) => StateWriteResult::success(format!("Remembered in {label} ({}): {}", tier_label(tier), record.content)),
            Ok(None) => StateWriteResult::failure(format!("nothing was saved to {label} - the fact was empty or already recorded.")),
            Err(error) => StateWriteResult::failure(format!("could not write {label}: {error}")),
        }
    }

    pub fn remove_memory(&self, content: &str, scope: MemoryScope, project: Option<&str>) -> StateWriteResult {
        let (store, label) = match self.memory_store(scope, project) { Ok(value) => value, Err(result) => return result };
        match store.remove_memory_by_content(content) {
            Ok(true) => StateWriteResult::success(format!("Forgot from {label}: {}", normalize_memory_content(content))),
            Ok(false) => StateWriteResult::failure(format!("no fact with exactly that text is recorded in {label}.")),
            Err(error) => StateWriteResult::failure(format!("could not update {label}: {error}")),
        }
    }


    pub fn automation_record(&self, id: &str) -> Option<AutomationRecord> {
        self.workflows.automations.get(id)
    }

    pub fn create_automation(&self, spec: &AutomationSpec) -> StateWriteResult {
        match self.workflows.automations.upsert(spec, (self.now)() as f64) {
            Ok(Some(record)) => automation_result("Saved", &record),
            Ok(None) => StateWriteResult::failure(
                "the routine could not be saved - check its name, instruction, and trigger.",
            ),
            Err(error) => StateWriteResult::failure(format!("the routine could not be saved: {error}")),
        }
    }

    pub fn update_automation(&self, id: &str, spec: &AutomationSpec) -> StateWriteResult {
        match self.workflows.automations.update(id, spec) {
            Ok(Some(record)) => automation_result("Updated", &record),
            Ok(None) => StateWriteResult::failure(format!(
                "no routine with folder \"{id}\" exists, or the new fields were invalid."
            )),
            Err(error) => StateWriteResult::failure(format!("the routine could not be updated: {error}")),
        }
    }

    pub fn set_automation_enabled(&self, id: &str, enabled: bool) -> StateWriteResult {
        match self.workflows.automations.set_enabled(id, enabled) {
            Ok(Some(record)) => StateWriteResult::success(format!(
                "{} routine \"{}\" (folder {}).",
                if enabled { "Resumed" } else { "Paused" },
                record.name,
                record.id
            )),
            Ok(None) => StateWriteResult::failure(format!("no routine with folder \"{id}\" exists.")),
            Err(error) => StateWriteResult::failure(format!("the routine could not be updated: {error}")),
        }
    }

    pub fn delete_automation(&self, id: &str) -> StateWriteResult {
        let name = self.workflows.automations.get(id)
            .map(|record| record.name)
            .unwrap_or_else(|| id.to_string());
        match self.workflows.automations.remove(id) {
            Ok(true) => StateWriteResult::success(format!("Deleted routine \"{name}\" (folder {id}).")),
            Ok(false) => StateWriteResult::failure(format!("no routine with folder \"{id}\" exists.")),
            Err(error) => StateWriteResult::failure(format!("the routine could not be deleted: {error}")),
        }
    }

    pub fn write_workflow(
        &self,
        id: Option<&str>,
        name: &str,
        description: Option<&str>,
        body: &str,
    ) -> StateWriteResult {
        let spec = WorkflowSpec {
            name: name.to_string(),
            description: description.unwrap_or_default().to_string(),
            body: body.to_string(),
            trigger: None,
            source_ref: None,
        };
        let result = match id {
            Some(id) => self.workflows.update(id, &spec),
            None => self.workflows.create(&spec),
        };
        match result {
            Ok(Some(record)) => StateWriteResult::success(format!(
                "{} workflow \"{}\" (id {}).",
                if id.is_some() { "Updated" } else { "Saved" },
                record.name,
                record.id
            )),
            Ok(None) => StateWriteResult::failure(
                if let Some(id) = id {
                    format!("no workflow with id \"{id}\" exists, or the new fields were invalid.")
                } else {
                    "the workflow could not be saved - a name and non-empty body are required.".to_string()
                },
            ),
            Err(error) => StateWriteResult::failure(format!("the workflow could not be saved: {error}")),
        }
    }

    pub fn delete_workflow(&self, id: &str) -> StateWriteResult {
        match self.workflows.remove(id) {
            Ok(true) => StateWriteResult::success(format!("Deleted workflow {id}.")),
            Ok(false) => StateWriteResult::failure(format!(
                "no workflow with id \"{id}\" exists, or it is managed and cannot be deleted."
            )),
            Err(error) => StateWriteResult::failure(format!("the workflow could not be deleted: {error}")),
        }
    }

    pub fn update_profile(&self, name: Option<&str>, description: Option<&str>) -> StateWriteResult {
        if name.is_none() && description.is_none() { return StateWriteResult::failure("nothing to change - pass at least one of name or description."); }
        if name.is_some_and(|value| value.trim().is_empty()) { return StateWriteResult::failure("a blank name is not allowed."); }
        let path = get_sand_profile_path(&self.agent_dir);
        let current = read_sand_profile_file(&path).unwrap_or(SandAgentProfile {
            name: String::new(), description: String::new(), title: String::new(),
            avatar_shape: String::new(), avatar_color: String::new(),
        });
        let next = SandAgentProfile {
            name: name.map(|v| v.trim().to_string()).unwrap_or(current.name),
            description: description.map(|v| v.trim().to_string()).unwrap_or(current.description),
            title: current.title, avatar_shape: current.avatar_shape, avatar_color: current.avatar_color,
        };
        match write_sand_profile_file(&path, &next) {
            Ok(()) => StateWriteResult::success("Updated your profile."),
            Err(error) => StateWriteResult::failure(format!("could not update profile: {error}")),
        }
    }

    pub fn update_settings(&self, hidden_from_sidebar: Option<bool>, notify_on_agent_updates: Option<bool>) -> StateWriteResult {
        let mut update = Map::new();
        if let Some(value) = hidden_from_sidebar { update.insert("hiddenFromSidebar".into(), Value::Bool(value)); }
        if let Some(value) = notify_on_agent_updates { update.insert("notifyOnAgentUpdates".into(), Value::Bool(value)); }
        if update.is_empty() { return StateWriteResult::failure("nothing to change - pass at least one setting field."); }
        match write_sand_settings_file(get_sand_settings_path(&self.agent_dir), &update) {
            Ok(()) => StateWriteResult::success("Updated your settings."),
            Err(error) => StateWriteResult::failure(format!("could not update settings: {error}")),
        }
    }

    pub fn disconnect_channel(&self, platform: &str) -> StateWriteResult {
        match self.channels.remove(platform) {
            Ok(true) => StateWriteResult::success(format!("Disconnected {platform}.")),
            Ok(false) => StateWriteResult::failure(format!("{platform} is not connected.")),
            Err(error) => StateWriteResult::failure(format!("could not disconnect {platform}: {error}")),
        }
    }

    pub fn create_project(&self, slug: &str, name: &str, description: Option<&str>) -> StateWriteResult {
        let id = slug.trim();
        if !is_safe_folder_id(id) { return StateWriteResult::failure(format!("\"{id}\" is not a valid project slug.")); }
        if name.trim().is_empty() { return StateWriteResult::failure("a project needs a non-empty name."); }
        let path = get_project_dir(&self.sand_root, id);
        let existed = project_dir_exists(&self.sand_root, id);
        if !existed {
            if let Err(error) = fs::create_dir_all(&path) { return StateWriteResult::failure(format!("could not create project \"{id}\": {error}")); }
            let body = format!("---\nname: {}\ndescription: {}\n---\n", name.trim(), description.unwrap_or_default().trim());
            if let Err(error) = fs::write(path.join("project.md"), body) { return StateWriteResult::failure(format!("could not create project \"{id}\": {error}")); }
        }
        match self.membership.join(id) {
            Ok(true) => StateWriteResult::success(if existed { format!("Joined existing project \"{id}\".") } else { format!("Created and joined project \"{}\" (folder {id}).", name.trim()) }),
            Ok(false) => StateWriteResult::failure(format!("could not join project \"{id}\".")),
            Err(error) => StateWriteResult::failure(format!("could not join project \"{id}\": {error}")),
        }
    }

    pub fn join_project(&self, slug: &str) -> StateWriteResult {
        let id = slug.trim();
        if !is_safe_folder_id(id) { return StateWriteResult::failure(format!("\"{id}\" is not a valid project slug.")); }
        if !project_dir_exists(&self.sand_root, id) { return StateWriteResult::failure(format!("no project \"{id}\" exists.")); }
        match self.membership.join(id) {
            Ok(true) => StateWriteResult::success(format!("Joined project \"{id}\".")),
            Ok(false) => StateWriteResult::failure(format!("could not join project \"{id}\".")),
            Err(error) => StateWriteResult::failure(format!("could not join project \"{id}\": {error}")),
        }
    }

    pub fn leave_project(&self, slug: &str) -> StateWriteResult {
        let id = slug.trim();
        if !is_safe_folder_id(id) { return StateWriteResult::failure(format!("\"{id}\" is not a valid project slug.")); }
        match self.membership.leave(id) {
            Ok(true) => StateWriteResult::success(format!("Left project \"{id}\".")),
            Ok(false) => StateWriteResult::failure(format!("could not leave project \"{id}\".")),
            Err(error) => StateWriteResult::failure(format!("could not leave project \"{id}\": {error}")),
        }
    }

    pub fn set_avatar(&self, source: &Path) -> StateWriteResult {
        let Ok(bytes) = fs::read(source) else { return StateWriteResult::failure("could not read avatar source."); };
        if bytes.is_empty() || bytes.len() as u64 > AVATAR_MAX_BYTES { return StateWriteResult::failure("the image must be under 5 MB and non-empty."); }
        let Some(mime) = sniff_avatar_mime_type(&bytes) else { return StateWriteResult::failure("that file is not a recognized avatar image."); };
        let extension = match mime { "image/png" => "png", "image/jpeg" => "jpg", "image/webp" => "webp", "image/gif" => "gif", "image/svg+xml" => "svg", _ => return StateWriteResult::failure("that file is not a recognized avatar image.") };
        if let Err(error) = fs::create_dir_all(&self.agent_dir) { return StateWriteResult::failure(format!("could not update picture: {error}")); }
        for name in list_conventional_avatar_filenames(&self.agent_dir) { let _ = fs::remove_file(self.agent_dir.join(name)); }
        let filename = if extension == "png" { CANONICAL_AVATAR_FILENAME.to_string() } else { format!("avatar.{extension}") };
        if let Err(error) = fs::write(self.agent_dir.join(&filename), bytes) { return StateWriteResult::failure(format!("could not update picture: {error}")); }
        invalidate_avatar_data_url_cache(&self.agent_dir);
        if let Some(callback) = &self.on_avatar_changed { callback(); }
        StateWriteResult::success(format!("Updated your picture ({filename})."))
    }

    pub fn clear_avatar(&self) -> StateWriteResult {
        let files = list_conventional_avatar_filenames(&self.agent_dir);
        if files.is_empty() { return StateWriteResult::failure("you already have the default picture."); }
        for name in files {
            if let Err(error) = fs::remove_file(self.agent_dir.join(name)) { return StateWriteResult::failure(format!("could not clear picture: {error}")); }
        }
        invalidate_avatar_data_url_cache(&self.agent_dir);
        if let Some(callback) = &self.on_avatar_changed { callback(); }
        StateWriteResult::success("Cleared your picture - back to the default.")
    }

    fn memory_store(&self, scope: MemoryScope, project: Option<&str>) -> Result<(FileMemoryStore, String), StateWriteResult> {
        match scope {
            MemoryScope::Agent => Ok((self.memory.clone(), "your memory".into())),
            MemoryScope::User => Ok((FileMemoryStore::new(get_user_memory_shard_dir(&self.sand_root, &self.agent_id)), "shared user memory".into())),
            MemoryScope::Project => {
                let Some(slug) = project.map(str::trim).filter(|v| !v.is_empty()) else { return Err(StateWriteResult::failure("'project' is required when scope is project.")); };
                if !is_safe_folder_id(slug) { return Err(StateWriteResult::failure(format!("\"{slug}\" is not a valid project slug."))); }
                if !project_dir_exists(&self.sand_root, slug) { return Err(StateWriteResult::failure(format!("no project \"{slug}\" exists yet."))); }
                if !self.membership.read().contains(slug) { return Err(StateWriteResult::failure(format!("you haven't joined project \"{slug}\" yet."))); }
                Ok((FileMemoryStore::new(get_project_memory_shard_dir(&self.sand_root, slug, &self.agent_id)), format!("project \"{slug}\" memory")))
            }
        }
    }
}


fn automation_result(verb: &str, record: &AutomationRecord) -> StateWriteResult {
    StateWriteResult::success(format!(
        "{verb} routine \"{}\" (folder {}) - {}{}.",
        record.name,
        record.id,
        describe_trigger(&record.trigger),
        if record.is_enabled { "" } else { ", paused" }
    ))
}

fn tier_label(tier: MemoryTier) -> &'static str {
    match tier { MemoryTier::Profile => "profile", MemoryTier::Note => "note", MemoryTier::Log => "log" }
}

fn system_now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis().try_into().unwrap_or(i64::MAX)
}
