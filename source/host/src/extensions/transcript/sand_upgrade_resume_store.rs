use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use serde_json::Value;

use crate::durable_file_policy::SAND_UPGRADE_RESUME_FILE_NAME;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpgradeResumeMarker {
    pub agent_id: String,
    pub marked_at_ms: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub automation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub automation_run_id: Option<String>,
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
    marker.automation_run_id.is_none() || marker.automation_id.is_some()
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
    let value: Value = serde_json::from_str(raw)
        .map_err(|error| UpgradeResumeStoreError::MalformedDocument(error.to_string()))?;
    let object = value.as_object().ok_or_else(|| {
        UpgradeResumeStoreError::MalformedDocument("expected an object envelope".into())
    })?;
    // The historical writer also used a versionless pending envelope. Its
    // rows still need complete identities; lenient coercion is inspection-only.
    if let Some(version) = object.get("version") {
        if version.as_u64() != Some(1) {
            return Err(UpgradeResumeStoreError::UnsupportedVersion(version.to_string()));
        }
    }
    let rows = object.get("pending").and_then(Value::as_array).ok_or_else(|| {
        UpgradeResumeStoreError::MalformedDocument("pending must be an array".into())
    })?;
    let mut agents = HashSet::new();
    let mut pending = Vec::with_capacity(rows.len());
    for (index, row) in rows.iter().enumerate() {
        let invalid = |reason| UpgradeResumeStoreError::InvalidMarker { index, reason };
        let object = row.as_object().ok_or_else(|| invalid("expected an object"))?;
        if !object.get("markedAtMs").and_then(Value::as_f64).is_some_and(|value| value.is_finite()) {
            return Err(invalid("markedAtMs must be a finite number"));
        }
        for key in ["source", "automationId", "automationRunId"] {
            if object.get(key).is_some_and(|value| !value.is_null() && !value.is_string()) {
                return Err(invalid("optional identity fields must be strings or null"));
            }
        }
        let marker = coerce_upgrade_resume_marker(row)
            .ok_or_else(|| invalid("agentId must be a nonempty string"))?;
        validate_marker(&marker, index)?;
        if !agents.insert(marker.agent_id.clone()) {
            return Err(invalid("duplicate agent identity"));
        }
        pending.push(marker);
    }
    Ok(pending)
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
                version: 1,
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
}
