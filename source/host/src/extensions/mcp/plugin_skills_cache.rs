use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const PLUGIN_SKILLS_DIRNAME: &str = "plugin-skills";
pub const PLUGIN_SKILLS_CACHE_FILENAME: &str = "cache.json";
pub const AGENT_READABLE_SKILL_DIR_MODE: u32 = 0o755;
pub const AGENT_READABLE_SKILL_FILE_MODE: u32 = 0o644;
const PLUGIN_SKILLS_CACHE_FORMAT_VERSION: u64 = 1;
static NEXT_CACHE_STAGE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginSkillRecord {
    pub id: String,
    pub plugin_id: String,
    pub plugin_name: String,
    pub name: String,
    pub description: String,
    pub file_path: String,
    pub plugin_version: String,
    pub install_path: String,
    pub skill_relative_path: String,
    pub publisher_user_id: Option<u64>,
    pub marketplace_team_id: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginAuthBlock {
    pub plugin_id: String,
    pub plugin_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marketplace_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PluginSkillsCache {
    pub fetched_at: f64,
    pub current_user_id: Option<u64>,
    pub skills: Vec<PluginSkillRecord>,
    pub auth_blocked: Vec<PluginAuthBlock>,
}

#[derive(Debug, Clone)]
pub struct PluginSkillsCacheWriteIndex {
    pub current_user_id: Option<Option<u64>>,
    pub skills: Vec<Value>,
    pub auth_blocked: Option<Vec<PluginAuthBlock>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PluginSkillsAdmission {
    LegacyUnblocked,
    LegacyAmbiguous,
    Canonical,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PluginSkillsSnapshot {
    pub(crate) index: PluginSkillsCache,
    admission: PluginSkillsAdmission,
}

impl PluginSkillsSnapshot {
    pub(crate) fn eligible_skills(&self) -> impl Iterator<Item = &PluginSkillRecord> {
        self.index.skills.iter().filter(|record| match self.admission {
            PluginSkillsAdmission::LegacyUnblocked => true,
            PluginSkillsAdmission::LegacyAmbiguous => false,
            PluginSkillsAdmission::Canonical => !self
                .index
                .auth_blocked
                .iter()
                .any(|block| block.plugin_id == record.plugin_id),
        })
    }

    pub(crate) fn canonical_auth_blocked(&self) -> &[PluginAuthBlock] {
        if self.admission == PluginSkillsAdmission::Canonical {
            &self.index.auth_blocked
        } else {
            &[]
        }
    }
}

pub fn get_plugins_root_dir(sand_root: impl AsRef<Path>) -> PathBuf {
    sand_root.as_ref().join("plugins")
}

pub fn get_plugin_skills_dir(sand_root: impl AsRef<Path>) -> PathBuf {
    sand_root.as_ref().join(PLUGIN_SKILLS_DIRNAME)
}

pub fn get_plugin_skills_cache_path(cache_dir: impl AsRef<Path>) -> PathBuf {
    cache_dir.as_ref().join(PLUGIN_SKILLS_CACHE_FILENAME)
}

pub fn is_safe_plugin_skill_id(id: &str) -> bool {
    !id.is_empty()
        && id.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || byte.is_ascii_lowercase())
        })
}

fn positive_id(value: Option<&Value>) -> Option<u64> {
    value
        .and_then(Value::as_u64)
        .filter(|value| *value > 0)
}

fn string_field(record: &serde_json::Map<String, Value>, key: &str) -> Option<String> {
    record.get(key)?.as_str().map(str::to_string)
}

fn parse_skill(value: &Value) -> Option<PluginSkillRecord> {
    let record = value.as_object()?;
    let id = string_field(record, "id")?;
    let plugin_id = string_field(record, "pluginId")?;
    let plugin_name = string_field(record, "pluginName")?;
    let name = string_field(record, "name")?;
    let description = string_field(record, "description")?;
    let file_path = string_field(record, "filePath")?;
    if !is_safe_plugin_skill_id(&id)
        || plugin_id.is_empty()
        || name.is_empty()
        || !Path::new(&file_path).is_absolute()
    {
        return None;
    }
    let plugin_version = record
        .get("pluginVersion")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let install_path = record
        .get("installPath")
        .and_then(Value::as_str)
        .filter(|path| Path::new(path).is_absolute())
        .unwrap_or_default()
        .to_string();
    let skill_relative_path = record
        .get("skillRelativePath")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    Some(PluginSkillRecord {
        id,
        plugin_id,
        plugin_name,
        name,
        description,
        file_path,
        plugin_version,
        install_path,
        skill_relative_path,
        publisher_user_id: positive_id(record.get("publisherUserId")),
        marketplace_team_id: positive_id(record.get("marketplaceTeamId")),
    })
}

fn parse_auth_block(value: &Value) -> Option<PluginAuthBlock> {
    let record = value.as_object()?;
    let plugin_id = string_field(record, "pluginId")?;
    let plugin_name = string_field(record, "pluginName")?;
    let marketplace_name = match record.get("marketplaceName") {
        None => None,
        Some(Value::String(value)) => Some(value.clone()),
        Some(_) => return None,
    };
    Some(PluginAuthBlock {
        plugin_id,
        plugin_name,
        marketplace_name,
    })
}

pub fn read_plugin_skills_cache(cache_dir: impl AsRef<Path>) -> Option<PluginSkillsCache> {
    read_plugin_skills_snapshot(cache_dir).map(|snapshot| snapshot.index)
}

pub(crate) fn read_plugin_skills_snapshot(
    cache_dir: impl AsRef<Path>,
) -> Option<PluginSkillsSnapshot> {
    let text = fs::read_to_string(get_plugin_skills_cache_path(cache_dir)).ok()?;
    let value: Value = serde_json::from_str(&text).ok()?;
    parse_plugin_skills_snapshot(&value)
}

fn parse_plugin_skills_snapshot(value: &Value) -> Option<PluginSkillsSnapshot> {
    let record = value.as_object()?;
    let fetched_at = record.get("fetchedAt")?.as_f64()?;
    if !fetched_at.is_finite() {
        return None;
    }
    let raw_skills = record.get("skills")?.as_array()?;
    let mut skills = Vec::with_capacity(raw_skills.len());
    for value in raw_skills {
        skills.push(parse_skill(value)?);
    }
    let canonical = match record.get("cacheFormatVersion") {
        None => false,
        Some(value) if value.as_u64() == Some(PLUGIN_SKILLS_CACHE_FORMAT_VERSION) => true,
        Some(_) => return None,
    };
    let auth_blocked = match record.get("authBlocked") {
        None if !canonical => Vec::new(),
        Some(Value::Array(values)) => values
            .iter()
            .map(parse_auth_block)
            .collect::<Option<Vec<_>>>()?,
        _ => return None,
    };
    if canonical
        && auth_blocked
            .iter()
            .any(|block| block.plugin_id.is_empty() || block.plugin_id.trim() != block.plugin_id)
    {
        return None;
    }
    let admission = if canonical {
        PluginSkillsAdmission::Canonical
    } else if auth_blocked.is_empty() {
        PluginSkillsAdmission::LegacyUnblocked
    } else {
        PluginSkillsAdmission::LegacyAmbiguous
    };
    Some(PluginSkillsSnapshot {
        index: PluginSkillsCache {
            fetched_at,
            current_user_id: positive_id(record.get("currentUserId")),
            skills,
            auth_blocked,
        },
        admission,
    })
}

#[cfg(unix)]
fn chmod(path: &Path, mode: u32) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
}

#[cfg(not(unix))]
fn chmod(_path: &Path, _mode: u32) -> std::io::Result<()> {
    Ok(())
}

pub fn write_plugin_skills_cache(
    cache_dir: impl AsRef<Path>,
    index: &PluginSkillsCacheWriteIndex,
    now: impl FnOnce() -> f64,
) -> Result<(), String> {
    stage_plugin_skills_cache(cache_dir, index, now)?.publish()
}

pub(crate) struct StagedPluginSkillsCache {
    temporary_path: PathBuf,
    destination_path: PathBuf,
}

impl StagedPluginSkillsCache {
    pub(crate) fn publish(self) -> Result<(), String> {
        fs::rename(&self.temporary_path, &self.destination_path)
            .map_err(|error| error.to_string())
    }
}

impl Drop for StagedPluginSkillsCache {
    fn drop(&mut self) {
        // The unique file is created exclusively by this stage. Never remove
        // a shared cache directory or another publisher's staging path.
        let _ = fs::remove_file(&self.temporary_path);
    }
}

pub(crate) fn stage_plugin_skills_cache(
    cache_dir: impl AsRef<Path>,
    index: &PluginSkillsCacheWriteIndex,
    now: impl FnOnce() -> f64,
) -> Result<StagedPluginSkillsCache, String> {
    let cache_dir = cache_dir.as_ref();
    fs::create_dir_all(cache_dir).map_err(|error| error.to_string())?;
    chmod(cache_dir, AGENT_READABLE_SKILL_DIR_MODE).map_err(|error| error.to_string())?;

    let mut body = serde_json::Map::new();
    body.insert(
        "cacheFormatVersion".into(),
        Value::from(PLUGIN_SKILLS_CACHE_FORMAT_VERSION),
    );
    body.insert("fetchedAt".into(), serde_json::json!(now()));
    body.insert(
        "authBlocked".into(),
        serde_json::to_value(index.auth_blocked.as_deref().unwrap_or(&[]))
            .map_err(|error| error.to_string())?,
    );
    if let Some(current_user_id) = index.current_user_id {
        body.insert(
            "currentUserId".into(),
            current_user_id.map(Value::from).unwrap_or(Value::Null),
        );
    }
    body.insert("skills".into(), Value::Array(index.skills.clone()));
    let value = Value::Object(body);
    parse_plugin_skills_snapshot(&value)
        .ok_or_else(|| "invalid plugin skills cache snapshot".to_string())?;
    let text = format!(
        "{}\n",
        serde_json::to_string_pretty(&value)
            .map_err(|error| error.to_string())?
    );

    let path = get_plugin_skills_cache_path(cache_dir);
    let temp = cache_dir.join(format!(
        ".{PLUGIN_SKILLS_CACHE_FILENAME}.{}.{}.tmp",
        std::process::id(),
        NEXT_CACHE_STAGE.fetch_add(1, Ordering::Relaxed),
    ));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(AGENT_READABLE_SKILL_FILE_MODE);
    }
    let mut file = options.open(&temp).map_err(|error| error.to_string())?;
    let staged = StagedPluginSkillsCache {
        temporary_path: temp.clone(),
        destination_path: path,
    };
    let written = file.write_all(text.as_bytes());
    drop(file);
    written.map_err(|error| error.to_string())?;
    chmod(&temp, AGENT_READABLE_SKILL_FILE_MODE).map_err(|error| error.to_string())?;
    Ok(staged)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempCache(PathBuf);

    impl TempCache {
        fn new() -> Self {
            Self(std::env::temp_dir().join(format!("plugin-cache-stage-{}", uuid::Uuid::new_v4())))
        }
    }

    impl Drop for TempCache {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn index() -> PluginSkillsCacheWriteIndex {
        PluginSkillsCacheWriteIndex {
            current_user_id: Some(Some(7)),
            skills: Vec::new(),
            auth_blocked: Some(Vec::new()),
        }
    }

    #[test]
    fn staged_cache_has_no_public_effect_until_publish_and_discards_only_its_file() {
        let root = TempCache::new();
        write_plugin_skills_cache(&root.0, &index(), || 1.0).unwrap();
        let path = get_plugin_skills_cache_path(&root.0);
        let old = fs::read(&path).unwrap();
        let unrelated = root.0.join("cache.json.tmp");
        fs::write(&unrelated, b"another owned file").unwrap();

        let staged = stage_plugin_skills_cache(&root.0, &index(), || 2.0).unwrap();
        let temp = staged.temporary_path.clone();
        assert!(temp.is_file());
        assert_eq!(fs::read(&path).unwrap(), old);
        drop(staged);
        assert!(!temp.exists());
        assert_eq!(fs::read(&path).unwrap(), old);
        assert_eq!(fs::read(&unrelated).unwrap(), b"another owned file");

        let staged = stage_plugin_skills_cache(&root.0, &index(), || 3.0).unwrap();
        let temp = staged.temporary_path.clone();
        staged.publish().unwrap();
        assert!(!temp.exists());
        assert_eq!(read_plugin_skills_cache(&root.0).unwrap().fetched_at, 3.0);
        assert_eq!(fs::read(&unrelated).unwrap(), b"another owned file");
    }

    #[test]
    fn publication_io_failure_preserves_committed_cache() {
        let root = TempCache::new();
        write_plugin_skills_cache(&root.0, &index(), || 1.0).unwrap();
        let path = get_plugin_skills_cache_path(&root.0);
        let old = fs::read(&path).unwrap();
        let staged = stage_plugin_skills_cache(&root.0, &index(), || 2.0).unwrap();
        // Remove this exact owned stage to cause a real rename failure.
        fs::remove_file(&staged.temporary_path).unwrap();
        assert!(staged.publish().is_err());
        assert_eq!(fs::read(&path).unwrap(), old);
        assert_eq!(read_plugin_skills_cache(&root.0).unwrap().fetched_at, 1.0);
    }

    #[test]
    fn constructed_cache_must_pass_the_consumer_parser_before_staging() {
        let root = TempCache::new();
        write_plugin_skills_cache(&root.0, &index(), || 1.0).unwrap();
        let path = get_plugin_skills_cache_path(&root.0);
        let old = fs::read(&path).unwrap();
        let mut invalid = index();
        invalid.skills.push(serde_json::json!({
            "id":"Invalid-ID", "pluginId":"7", "pluginName":"P", "name":"P",
            "description":"", "filePath":"/tmp/SKILL.md"
        }));
        assert!(write_plugin_skills_cache(&root.0, &invalid, || 2.0).is_err());
        assert_eq!(fs::read(&path).unwrap(), old);
        assert_eq!(fs::read_dir(&root.0).unwrap().count(), 1);
    }
}
