use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::durable_file_policy::SAND_UPGRADE_RESUME_FILE_NAME;
use crate::extensions::session::agent_db::OriginalTurnReference;
use super::sand_ack_obligation_store::SandAckObligationStore;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpgradeResumeMarker {
    pub agent_id: String,
    pub marked_at_ms: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub automation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub automation_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_work: Option<OriginalTurnReference>,
}

pub fn coerce_upgrade_resume_marker(entry: &Value) -> Option<UpgradeResumeMarker> {
    let object = entry.as_object()?;
    let agent_id = object
        .get("agentId")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())?;
    Some(UpgradeResumeMarker {
        agent_id: agent_id.to_string(),
        marked_at_ms: object
            .get("markedAtMs")
            .and_then(Value::as_f64)
            .filter(|value| value.is_finite())
            .unwrap_or(0.0),
        source: object.get("source").and_then(Value::as_str).map(ToOwned::to_owned),
        automation_id: object
            .get("automationId")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        automation_run_id: object
            .get("automationRunId")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        original_work: object.get("originalWork")
            .filter(|value| !value.is_null())
            .and_then(|value| serde_json::from_value(value.clone()).ok()),
    })
}

pub fn parse_upgrade_resume_file(raw: Option<&str>) -> Vec<UpgradeResumeMarker> {
    let Some(raw) = raw else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_str::<Value>(raw) else {
        return Vec::new();
    };
    value
        .get("pending")
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(coerce_upgrade_resume_marker).collect())
        .unwrap_or_default()
}

pub fn upgrade_resume_marker_identity_is_well_formed(marker: &UpgradeResumeMarker) -> bool {
    if marker.agent_id.trim().is_empty()
        || marker
            .automation_id
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        || marker
            .automation_run_id
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
    {
        return false;
    }
    (marker.automation_run_id.is_none() || marker.automation_id.is_some())
        && marker.original_work.as_ref().is_none_or(|original| {
            original.validate().is_ok()
                && original.agent_id == marker.agent_id
                && marker.source.as_deref() == Some("turn")
                && marker.automation_id.is_none()
                && marker.automation_run_id.is_none()
        })
}

#[derive(Debug)]
pub enum UpgradeResumeStoreError {
    OwnerPoisoned,
    MalformedDocument(String),
    UnsupportedVersion(String),
    InvalidMarker { index: usize, reason: &'static str },
    Io { operation: &'static str, error: std::io::Error },
}

impl fmt::Display for UpgradeResumeStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OwnerPoisoned => write!(formatter, "upgrade resume marker owner lock is poisoned"),
            Self::MalformedDocument(detail) => {
                write!(formatter, "upgrade resume marker document is malformed: {detail}")
            }
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported upgrade resume marker version: {version}")
            }
            Self::InvalidMarker { index, reason } => {
                write!(formatter, "invalid upgrade resume marker at row {index}: {reason}")
            }
            Self::Io { operation, error } => {
                write!(formatter, "upgrade resume marker {operation} failed: {error}")
            }
        }
    }
}

impl std::error::Error for UpgradeResumeStoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { error, .. } => Some(error),
            _ => None,
        }
    }
}

fn validate_marker(marker: &UpgradeResumeMarker, index: usize) -> Result<(), UpgradeResumeStoreError> {
    if !upgrade_resume_marker_identity_is_well_formed(marker) {
        return Err(UpgradeResumeStoreError::InvalidMarker {
            index,
            reason: "agent or automation identity is missing or inconsistent",
        });
    }
    if !marker.marked_at_ms.is_finite() {
        return Err(UpgradeResumeStoreError::InvalidMarker {
            index,
            reason: "markedAtMs must be finite",
        });
    }
    Ok(())
}

fn parse_upgrade_resume_file_checked(raw: &str) -> Result<Vec<UpgradeResumeMarker>, UpgradeResumeStoreError> {
    Ok(parse_upgrade_resume_image_checked(raw)?.1)
}

fn parse_upgrade_resume_image_checked(raw: &str) -> Result<(u64, Vec<UpgradeResumeMarker>), UpgradeResumeStoreError> {
    let value: Value = serde_json::from_str(raw)
        .map_err(|error| UpgradeResumeStoreError::MalformedDocument(error.to_string()))?;
    let object = value.as_object().ok_or_else(|| {
        UpgradeResumeStoreError::MalformedDocument("expected an object envelope".into())
    })?;
    // The historical writer also used a versionless pending envelope. Its
    // rows still need complete identities; lenient coercion is inspection-only.
    let version = match object.get("version") {
        Some(version) => {
            if !matches!(version.as_u64(), Some(1 | 2)) {
            return Err(UpgradeResumeStoreError::UnsupportedVersion(version.to_string()));
            }
            version.as_u64().expect("version was checked")
        }
        None => 1,
    };
    let rows = object.get("pending").and_then(Value::as_array).ok_or_else(|| {
        UpgradeResumeStoreError::MalformedDocument("pending must be an array".into())
    })?;
    for (index, row) in rows.iter().enumerate() {
        let invalid = |reason| UpgradeResumeStoreError::InvalidMarker { index, reason };
        let object = row.as_object().ok_or_else(|| invalid("expected an object"))?;
        if !object.get("agentId").and_then(Value::as_str).is_some_and(|value| !value.is_empty()) {
            return Err(invalid("agentId must be a nonempty string"));
        }
        if !object.get("markedAtMs").and_then(Value::as_f64).is_some_and(|value| value.is_finite()) {
            return Err(invalid("markedAtMs must be a finite number"));
        }
        for key in ["source", "automationId", "automationRunId"] {
            if object.get(key).is_some_and(|value| !value.is_null() && !value.is_string()) {
                return Err(invalid("optional identity fields must be strings or null"));
            }
        }
    }
    // Parse the original bytes again into the strict shape, so duplicate known
    // fields and unknown nested reference fields cannot disappear through Value.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Image {
        #[serde(rename = "version")]
        _version: Option<u64>,
        pending: Vec<UpgradeResumeMarker>,
    }
    let image: Image = serde_json::from_str(raw)
        .map_err(|error| UpgradeResumeStoreError::MalformedDocument(error.to_string()))?;
    let mut agents = HashSet::new();
    for (index, marker) in image.pending.iter().enumerate() {
        let invalid = |reason| UpgradeResumeStoreError::InvalidMarker { index, reason };
        validate_marker(marker, index)?;
        if version != 2 && marker.original_work.is_some() {
            return Err(invalid("legacy marker cannot carry original work authority"));
        }
        if !agents.insert(marker.agent_id.clone()) {
            return Err(invalid("duplicate agent identity"));
        }
    }
    Ok((version, image.pending))
}

pub fn upsert_resume_marker(
    existing: &[UpgradeResumeMarker],
    marker: UpgradeResumeMarker,
) -> Vec<UpgradeResumeMarker> {
    let mut next = existing
        .iter()
        .filter(|entry| entry.agent_id != marker.agent_id)
        .cloned()
        .collect::<Vec<_>>();
    next.push(marker);
    next
}

#[derive(Clone)]
pub struct SandUpgradeResumeStore {
    file_path: PathBuf,
    lock: Arc<Mutex<()>>,
}

impl SandUpgradeResumeStore {
    pub fn new(root_dir: impl AsRef<Path>) -> Self {
        Self {
            file_path: root_dir.as_ref().join(SAND_UPGRADE_RESUME_FILE_NAME),
            lock: Arc::new(Mutex::new(())),
        }
    }

    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    pub fn mark_pending(&self, marker: UpgradeResumeMarker) {
        let _ = self.mark_pending_checked(marker);
    }

    /// Compatibility inspection only. Production recovery must use the checked API.
    pub fn list_pending(&self) -> Vec<UpgradeResumeMarker> {
        self.list_pending_checked().unwrap_or_default()
    }

    pub fn list_pending_checked(&self) -> Result<Vec<UpgradeResumeMarker>, UpgradeResumeStoreError> {
        let _guard = self.lock.lock().map_err(|_| UpgradeResumeStoreError::OwnerPoisoned)?;
        self.read_pending_checked_unlocked()
    }

    /// Migrate the complete valid legacy image under the existing owner lock.
    /// Legacy rows remain unbound; migration never invents an original turn.
    pub fn migrate_legacy_checked(&self) -> Result<Vec<UpgradeResumeMarker>, UpgradeResumeStoreError> {
        self.migrate_legacy_with_replace(|part, target| fs::rename(part, target))
    }

    fn migrate_legacy_with_replace(
        &self,
        replace: impl FnOnce(&Path, &Path) -> std::io::Result<()>,
    ) -> Result<Vec<UpgradeResumeMarker>, UpgradeResumeStoreError> {
        let _guard = self.lock.lock().map_err(|_| UpgradeResumeStoreError::OwnerPoisoned)?;
        self.migrate_legacy_unlocked(replace)
    }

    fn migrate_legacy_unlocked(
        &self,
        replace: impl FnOnce(&Path, &Path) -> std::io::Result<()>,
    ) -> Result<Vec<UpgradeResumeMarker>, UpgradeResumeStoreError> {
        let raw = match fs::read_to_string(&self.file_path) {
            Ok(raw) => raw,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(UpgradeResumeStoreError::Io { operation: "read", error }),
        };
        let (version, pending) = parse_upgrade_resume_image_checked(&raw)?;
        if version != 2 {
            self.write_unlocked_with_replace(&pending, replace)?;
        }
        Ok(pending)
    }

    /// Only the Runtime admission owner uses this boundary. The action is a
    /// synchronous ACK mutation, never a provider, observer or send callback.
    /// Holding this existing lock prevents a retained marker from appearing
    /// between the deciding read and consumption of a due timer/attempt.
    pub(super) fn while_agent_unmarked_checked<T>(
        &self,
        agent_id: &str,
        action: impl FnOnce() -> T,
    ) -> Result<Option<T>, UpgradeResumeStoreError> {
        let _guard = self.lock.lock().map_err(|_| UpgradeResumeStoreError::OwnerPoisoned)?;
        let pending = self.migrate_legacy_unlocked(|part, target| fs::rename(part, target))?;
        if pending.iter().any(|marker| marker.agent_id == agent_id) {
            return Ok(None);
        }
        Ok(Some(action()))
    }

    pub fn mark_pending_checked(&self, marker: UpgradeResumeMarker) -> Result<(), UpgradeResumeStoreError> {
        self.mark_pending_batch_checked(&[marker])
    }

    pub fn mark_pending_batch_checked(&self, markers: &[UpgradeResumeMarker]) -> Result<(), UpgradeResumeStoreError> {
        self.mark_pending_batch_with_replace(markers, |part, target| fs::rename(part, target))
    }

    fn mark_pending_batch_with_replace(
        &self,
        markers: &[UpgradeResumeMarker],
        replace: impl FnOnce(&Path, &Path) -> std::io::Result<()>,
    ) -> Result<(), UpgradeResumeStoreError> {
        let _guard = self.lock.lock().map_err(|_| UpgradeResumeStoreError::OwnerPoisoned)?;
        let existing = self.read_pending_checked_unlocked()?;
        for (index, marker) in markers.iter().enumerate() {
            validate_marker(marker, index)?;
        }
        let mut pending = existing.clone();
        for marker in markers {
            if pending.iter().any(|current| {
                current.agent_id == marker.agent_id && current.marked_at_ms >= marker.marked_at_ms
            }) {
                continue;
            }
            pending = upsert_resume_marker(&pending, marker.clone());
        }
        if pending != existing {
            self.write_unlocked_with_replace(&pending, replace)?;
        }
        Ok(())
    }

    pub fn clear(&self, agent_id: &str) {
        let _ = self.clear_checked(agent_id);
    }

    pub fn clear_checked(&self, agent_id: &str) -> Result<bool, UpgradeResumeStoreError> {
        let _guard = self.lock.lock().map_err(|_| UpgradeResumeStoreError::OwnerPoisoned)?;
        let pending = self.read_pending_checked_unlocked()?;
        if !pending.iter().any(|entry| entry.agent_id == agent_id) {
            return Ok(false);
        }
        let remaining = pending
            .into_iter()
            .filter(|entry| entry.agent_id != agent_id)
            .collect::<Vec<_>>();
        if remaining.is_empty() {
            self.delete_unlocked()?;
        } else {
            self.write_unlocked_with_replace(&remaining, |part, target| fs::rename(part, target))?;
        }
        Ok(true)
    }

    /// Remove only the marker that was observed by a recovery pass.
    ///
    /// A recovery pass may race with a new marker for the same agent.  An
    /// agent-only clear would delete that newer work, so the whole marker is
    /// compared while holding the store lock and the new marker is retained.
    pub fn clear_if_matches(&self, marker: &UpgradeResumeMarker) -> bool {
        self.clear_if_matches_checked(marker).unwrap_or(false)
    }

    pub fn clear_if_matches_checked(&self, marker: &UpgradeResumeMarker) -> Result<bool, UpgradeResumeStoreError> {
        self.clear_if_matches_with_replace(marker, |part, target| fs::rename(part, target))
    }

    fn clear_if_matches_with_replace(
        &self,
        marker: &UpgradeResumeMarker,
        replace: impl FnOnce(&Path, &Path) -> std::io::Result<()>,
    ) -> Result<bool, UpgradeResumeStoreError> {
        let _guard = self.lock.lock().map_err(|_| UpgradeResumeStoreError::OwnerPoisoned)?;
        self.clear_if_matches_unlocked_with_replace(marker, replace)
    }

    pub(super) fn clear_if_matches_with_ack_absent_checked(
        &self,
        marker: &UpgradeResumeMarker,
        acknowledgments: &SandAckObligationStore,
    ) -> Result<bool, UpgradeResumeStoreError> {
        let _guard = self.lock.lock().map_err(|_| UpgradeResumeStoreError::OwnerPoisoned)?;
        let Some(_absence) = acknowledgments.checked_absence_guard(&marker.agent_id)
            .map_err(|error| UpgradeResumeStoreError::Io { operation: "check ACK absence", error })?
        else { return Ok(false); };
        self.clear_if_matches_unlocked_with_replace(marker, |part, target| fs::rename(part, target))
    }

    fn clear_if_matches_unlocked_with_replace(
        &self,
        marker: &UpgradeResumeMarker,
        replace: impl FnOnce(&Path, &Path) -> std::io::Result<()>,
    ) -> Result<bool, UpgradeResumeStoreError> {
        validate_marker(marker, 0)?;
        let pending = self.read_pending_checked_unlocked()?;
        let remaining = pending
            .iter()
            .filter(|entry| *entry != marker)
            .cloned()
            .collect::<Vec<_>>();
        if remaining.len() == pending.len() {
            return Ok(false);
        }
        if remaining.is_empty() {
            self.delete_unlocked()?;
        } else {
            self.write_unlocked_with_replace(&remaining, replace)?;
        }
        Ok(true)
    }

    pub fn clear_all(&self) {
        let _ = self.clear_all_checked();
    }

    pub fn clear_all_checked(&self) -> Result<(), UpgradeResumeStoreError> {
        let _guard = self.lock.lock().map_err(|_| UpgradeResumeStoreError::OwnerPoisoned)?;
        self.read_pending_checked_unlocked()?;
        self.delete_unlocked()
    }

    fn read_pending_checked_unlocked(&self) -> Result<Vec<UpgradeResumeMarker>, UpgradeResumeStoreError> {
        match fs::read_to_string(&self.file_path) {
            Ok(raw) => parse_upgrade_resume_file_checked(&raw),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(error) => Err(UpgradeResumeStoreError::Io { operation: "read", error }),
        }
    }

    fn write_unlocked_with_replace(
        &self,
        pending: &[UpgradeResumeMarker],
        replace: impl FnOnce(&Path, &Path) -> std::io::Result<()>,
    ) -> Result<(), UpgradeResumeStoreError> {
        #[derive(Serialize)]
        struct File<'a> {
            version: u8,
            pending: &'a [UpgradeResumeMarker],
        }
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).map_err(|error| UpgradeResumeStoreError::Io {
                operation: "create directory", error,
            })?;
        }
        let part = self.file_path.with_extension(format!("json.{}.part", uuid::Uuid::new_v4()));
        let result = (|| -> std::io::Result<()> {
            let mut file = fs::OpenOptions::new().create_new(true).write(true).open(&part)?;
            let bytes = serde_json::to_vec(&File {
                version: 2,
                pending,
            })
            .map_err(std::io::Error::other)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            drop(file);
            // Never remove the current target to make a failed rename retryable.
            replace(&part, &self.file_path)
        })();
        if result.is_err() {
            let _ = fs::remove_file(&part);
        }
        result.map_err(|error| UpgradeResumeStoreError::Io { operation: "write", error })
    }

    fn delete_unlocked(&self) -> Result<(), UpgradeResumeStoreError> {
        match fs::remove_file(&self.file_path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(UpgradeResumeStoreError::Io { operation: "delete", error }),
        }
    }
}

#[cfg(test)]
mod checked_marker_tests {
    use super::*;

    fn marker(agent_id: &str, at: f64) -> UpgradeResumeMarker {
        UpgradeResumeMarker {
            agent_id: agent_id.into(), marked_at_ms: at, source: Some("turn".into()),
            automation_id: None, automation_run_id: None,
            original_work: None,
        }
    }

    #[test]
    fn failed_batch_replace_and_exact_clear_keep_the_prior_file() {
        let root = std::env::temp_dir().join(format!("fabushi-upgrade-marker-replace-{}", uuid::Uuid::new_v4()));
        let store = SandUpgradeResumeStore::new(&root);
        let first = marker("a", 1.0);
        let second = marker("b", 2.0);
        store.mark_pending_batch_checked(&[first.clone(), second.clone()]).unwrap();
        let original = fs::read(store.file_path()).unwrap();
        let fail_replace = |part: &Path, target: &Path| {
            assert!(part.is_file(), "the real temporary-file write completed");
            assert_eq!(fs::read(target).unwrap(), original);
            Err(std::io::Error::other("controlled replacement refusal"))
        };
        let error = store.mark_pending_batch_with_replace(
            &[marker("c", 3.0), marker("d", 4.0)], fail_replace,
        ).unwrap_err();
        assert!(matches!(error, UpgradeResumeStoreError::Io { operation: "write", .. }));
        assert_eq!(fs::read(store.file_path()).unwrap(), original);
        assert_eq!(store.list_pending_checked().unwrap(), vec![first.clone(), second.clone()]);
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1, "failed temporary file is cleaned");

        let error = store.clear_if_matches_with_replace(&first, |part, target| {
            assert!(part.is_file());
            assert_eq!(fs::read(target).unwrap(), original);
            Err(std::io::Error::other("controlled exact-clear replacement refusal"))
        }).unwrap_err();
        assert!(matches!(error, UpgradeResumeStoreError::Io { operation: "write", .. }));
        assert_eq!(fs::read(store.file_path()).unwrap(), original);
        assert_eq!(store.list_pending_checked().unwrap(), vec![first.clone(), second]);
        assert!(store.clear_if_matches_checked(&first).unwrap());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn poisoned_marker_owner_refuses_reads_and_all_compatibility_mutations() {
        let root = std::env::temp_dir().join(format!("fabushi-upgrade-marker-poison-{}", uuid::Uuid::new_v4()));
        let store = SandUpgradeResumeStore::new(&root);
        let saved = marker("a", 1.0);
        store.mark_pending_checked(saved.clone()).unwrap();
        let original = fs::read(store.file_path()).unwrap();
        let poisoned = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = store.lock.lock().unwrap();
            panic!("poison marker owner");
        }));
        assert!(poisoned.is_err());
        assert!(matches!(store.list_pending_checked(), Err(UpgradeResumeStoreError::OwnerPoisoned)));
        assert!(matches!(store.migrate_legacy_checked(), Err(UpgradeResumeStoreError::OwnerPoisoned)));
        assert!(matches!(store.mark_pending_checked(marker("b", 2.0)), Err(UpgradeResumeStoreError::OwnerPoisoned)));
        assert!(matches!(store.clear_if_matches_checked(&saved), Err(UpgradeResumeStoreError::OwnerPoisoned)));
        assert!(matches!(store.clear_checked("a"), Err(UpgradeResumeStoreError::OwnerPoisoned)));
        assert!(matches!(store.clear_all_checked(), Err(UpgradeResumeStoreError::OwnerPoisoned)));
        store.mark_pending(marker("b", 2.0));
        store.clear("a");
        store.clear_all();
        assert!(!store.clear_if_matches(&saved));
        assert_eq!(fs::read(store.file_path()).unwrap(), original);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn legacy_migration_is_atomic_and_never_invents_original_work() {
        let root = std::env::temp_dir().join(format!("fabushi-upgrade-migrate-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let store = SandUpgradeResumeStore::new(&root);
        for document in [
            r#"{"pending":[{"agentId":"a","markedAtMs":1,"source":"turn"},{"agentId":"b","markedAtMs":2}]}"#,
            r#"{"version":1,"pending":[{"agentId":"a","markedAtMs":1,"source":"turn"},{"agentId":"b","markedAtMs":2}]}"#,
        ] {
            fs::write(store.file_path(), document).unwrap();
            let before = fs::read(store.file_path()).unwrap();
            assert!(store.migrate_legacy_with_replace(|part, target| {
                assert_eq!(fs::read(target).unwrap(), before);
                let staged: Value = serde_json::from_slice(&fs::read(part).unwrap()).unwrap();
                assert_eq!(staged["version"], 2);
                assert_eq!(staged["pending"].as_array().unwrap().len(), 2);
                assert!(staged["pending"].as_array().unwrap().iter().all(|row| row.get("originalWork").is_none()));
                Err(std::io::Error::other("controlled migration replace refusal"))
            }).is_err());
            assert_eq!(fs::read(store.file_path()).unwrap(), before);
            assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
            let migrated = store.migrate_legacy_checked().unwrap();
            assert_eq!(migrated.len(), 2);
            assert!(migrated.iter().all(|row| row.original_work.is_none()));
            let version_two = fs::read(store.file_path()).unwrap();
            assert_eq!(store.migrate_legacy_checked().unwrap(), migrated);
            assert_eq!(fs::read(store.file_path()).unwrap(), version_two);
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn marker_v2_keeps_strict_reference_and_exact_original_identity() {
        use crate::extensions::session::agent_db::OriginalTurnScope;
        let root = std::env::temp_dir().join(format!("fabushi-upgrade-original-{}", uuid::Uuid::new_v4()));
        let store = SandUpgradeResumeStore::new(&root);
        let mut original_marker = marker("agent", 1.0);
        original_marker.original_work = Some(OriginalTurnReference {
            version: 1, scope: OriginalTurnScope::NativeOrdinaryUserV1,
            agent_id: "agent".into(), session_key: "agent:agent:main".into(), session_id: "agent".into(),
            original_lifecycle_revision: "revision".into(), stream_id: "stream".into(),
            input_digest: "a".repeat(64), client_nonce: None,
            user_message_id: "user".into(), echo_entry_id: "user".into(), request_source: None,
            provider: "fabushi".into(), model_id: "admitted-model".into(),
        });
        store.mark_pending_checked(original_marker.clone()).unwrap();
        assert_eq!(store.migrate_legacy_checked().unwrap(), vec![original_marker.clone()]);
        let before = fs::read(store.file_path()).unwrap();
        let mut foreign = original_marker.clone();
        foreign.original_work.as_mut().unwrap().model_id = "another-model".into();
        assert!(!store.clear_if_matches_checked(&foreign).unwrap());
        assert_eq!(fs::read(store.file_path()).unwrap(), before);
        let mut document: Value = serde_json::from_slice(&before).unwrap();
        document["pending"][0]["originalWork"]["extra"] = Value::Bool(true);
        fs::write(store.file_path(), serde_json::to_vec(&document).unwrap()).unwrap();
        let corrupt = fs::read(store.file_path()).unwrap();
        assert!(store.migrate_legacy_checked().is_err());
        assert!(store.clear_if_matches_checked(&original_marker).is_err());
        assert_eq!(fs::read(store.file_path()).unwrap(), corrupt);
        document["pending"][0]["originalWork"].as_object_mut().unwrap().remove("extra");
        document["version"] = Value::from(1);
        fs::write(store.file_path(), serde_json::to_vec(&document).unwrap()).unwrap();
        assert!(store.migrate_legacy_checked().is_err(), "v1 cannot import positive original authority");
        fs::write(store.file_path(), before).unwrap();
        assert!(store.clear_if_matches_checked(&original_marker).unwrap());
        assert!(!store.file_path().exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn checked_migration_rejects_duplicate_fields_and_whole_image_corruption() {
        let root = std::env::temp_dir().join(format!("fabushi-upgrade-migrate-invalid-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let store = SandUpgradeResumeStore::new(&root);
        for raw in [
            r#"{"version":2,"version":2,"pending":[]}"#,
            r#"{"version":2,"pending":[],"extra":true}"#,
            r#"{"version":2,"pending":[{"agentId":"a","agentId":"a","markedAtMs":1}]}"#,
            r#"{"version":1,"pending":[{"agentId":"good","markedAtMs":1},{"agentId":"bad"}]}"#,
            r#"{"version":99,"pending":[]}"#,
        ] {
            fs::write(store.file_path(), raw).unwrap();
            assert!(store.migrate_legacy_checked().is_err());
            assert_eq!(fs::read_to_string(store.file_path()).unwrap(), raw);
            assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        }
        fs::remove_dir_all(root).unwrap();
    }
}
