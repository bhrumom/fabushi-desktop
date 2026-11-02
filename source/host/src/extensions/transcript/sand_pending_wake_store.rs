use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::durable_file_policy::SAND_PENDING_WAKE_FILE_NAME;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PendingWakeKind {
    CloudAgent,
    Subagent,
    Shell,
    ChannelDelivery,
}

impl PendingWakeKind {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "cloud-agent" => Some(Self::CloudAgent),
            "subagent" => Some(Self::Subagent),
            "shell" => Some(Self::Shell),
            "channel-delivery" => Some(Self::ChannelDelivery),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AutomationQuietOrigin {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct QuietWakeOrigin {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub automation: Option<AutomationQuietOrigin>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DurablePendingWakeMarker {
    pub agent_id: String,
    pub kind: PendingWakeKind,
    pub work_id: String,
    pub marked_at_ms: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quiet_origin: Option<QuietWakeOrigin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subagent_type: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub interrupted_by_recreate: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery_operation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
}

fn is_false(value: &bool) -> bool {
    !*value
}

pub fn coerce_quiet_origin(value: &Value) -> Option<QuietWakeOrigin> {
    let object = value.as_object()?;
    let automation = object
        .get("automation")
        .and_then(Value::as_object)
        .and_then(|automation| {
            let id = automation.get("id").and_then(Value::as_str)?;
            let name = automation.get("name").and_then(Value::as_str)?;
            (!id.is_empty()).then(|| AutomationQuietOrigin {
                id: id.to_string(),
                name: name.to_string(),
            })
        });
    Some(QuietWakeOrigin { automation })
}

pub fn coerce_marker(entry: &Value) -> Option<DurablePendingWakeMarker> {
    let object = entry.as_object()?;
    let agent_id = object
        .get("agentId")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())?;
    let work_id = object
        .get("workId")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())?;
    let kind = PendingWakeKind::parse(object.get("kind")?.as_str()?)?;
    let marked_at_ms = object
        .get("markedAtMs")
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
        .unwrap_or(0.0);
    let quiet_origin = object.get("quietOrigin").and_then(coerce_quiet_origin);
    let title = object
        .get("title")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let subagent_type = object
        .get("subagentType")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    Some(DurablePendingWakeMarker {
        agent_id: agent_id.to_string(),
        kind,
        work_id: work_id.to_string(),
        marked_at_ms,
        quiet_origin,
        title,
        subagent_type,
        interrupted_by_recreate: object
            .get("interruptedByRecreate")
            .and_then(Value::as_bool)
            == Some(true),
        delivery_operation_id: object
            .get("deliveryOperationId")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned),
        address_token: object
            .get("addressToken")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned),
        failure_reason: object
            .get("failureReason")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned),
        session_key: object
            .get("sessionKey")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned),
    })
}

pub fn coerce_pending_wake_markers(value: &Value) -> Vec<DurablePendingWakeMarker> {
    value
        .as_array()
        .map(|items| items.iter().filter_map(coerce_marker).collect())
        .unwrap_or_default()
}

pub fn parse_pending_wake_file(raw: Option<&str>) -> Vec<DurablePendingWakeMarker> {
    let Some(raw) = raw else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_str::<Value>(raw) else {
        return Vec::new();
    };
    value
        .get("pending")
        .map(coerce_pending_wake_markers)
        .unwrap_or_default()
}

#[derive(Debug)]
pub enum PendingWakeStoreError {
    OwnerPoisoned,
    MalformedDocument(String),
    UnsupportedVersion(String),
    InvalidMarker { index: usize, reason: &'static str },
    Io { operation: &'static str, error: std::io::Error },
}

impl fmt::Display for PendingWakeStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OwnerPoisoned => write!(formatter, "pending wake owner lock is poisoned"),
            Self::MalformedDocument(detail) => {
                write!(formatter, "pending wake document is malformed: {detail}")
            }
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported pending wake version: {version}")
            }
            Self::InvalidMarker { index, reason } => {
                write!(formatter, "invalid pending wake at row {index}: {reason}")
            }
            Self::Io { operation, error } => {
                write!(formatter, "pending wake {operation} failed: {error}")
            }
        }
    }
}

impl std::error::Error for PendingWakeStoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { error, .. } => Some(error),
            _ => None,
        }
    }
}

fn validate_marker(
    marker: &DurablePendingWakeMarker,
    index: usize,
) -> Result<(), PendingWakeStoreError> {
    let invalid = |reason| PendingWakeStoreError::InvalidMarker { index, reason };
    if marker.agent_id.trim().is_empty() || marker.work_id.trim().is_empty() {
        return Err(invalid("agentId and workId must be nonempty strings"));
    }
    if !marker.marked_at_ms.is_finite() {
        return Err(invalid("markedAtMs must be finite"));
    }
    if marker.quiet_origin.as_ref()
        .and_then(|origin| origin.automation.as_ref())
        .is_some_and(|automation| automation.id.trim().is_empty())
    {
        return Err(invalid("quiet automation id must be nonempty"));
    }
    // Channel-specific metadata may be absent in a legacy marker. Its
    // consumer must retain and refuse incomplete recovery, not normalize it
    // into a different operation or discard it during discovery.
    Ok(())
}

fn parse_pending_wake_file_checked(
    raw: &str,
) -> Result<Vec<DurablePendingWakeMarker>, PendingWakeStoreError> {
    let value: Value = serde_json::from_str(raw)
        .map_err(|error| PendingWakeStoreError::MalformedDocument(error.to_string()))?;
    let object = value.as_object().ok_or_else(|| {
        PendingWakeStoreError::MalformedDocument("expected an object envelope".into())
    })?;
    if let Some(version) = object.get("version") {
        if version.as_u64() != Some(1) {
            return Err(PendingWakeStoreError::UnsupportedVersion(version.to_string()));
        }
    }
    let rows = object.get("pending").and_then(Value::as_array).ok_or_else(|| {
        PendingWakeStoreError::MalformedDocument("pending must be an array".into())
    })?;
    // Keep the written JSON shapes: derived structs/enums can otherwise accept
    // positional arrays or tagged objects that no v1 writer produces.
    for (index, row) in rows.iter().enumerate() {
        let invalid = |reason| PendingWakeStoreError::InvalidMarker { index, reason };
        let row = row.as_object().ok_or_else(|| invalid("expected a marker object"))?;
        if row.get("kind").and_then(Value::as_str).and_then(PendingWakeKind::parse).is_none() {
            return Err(invalid("kind must be a known string"));
        }
        if !row.get("markedAtMs").and_then(Value::as_f64).is_some_and(|value| value.is_finite()) {
            return Err(invalid("markedAtMs must be a finite number"));
        }
        if let Some(origin) = row.get("quietOrigin").filter(|value| !value.is_null()) {
            let origin = origin.as_object().ok_or_else(|| invalid("quietOrigin must be an object or null"))?;
            if origin.get("automation").is_some_and(|automation| !automation.is_null() && !automation.is_object()) {
                return Err(invalid("quiet automation must be an object or null"));
            }
        }
    }
    // Deserialize the original bytes, not Value: duplicate members and unknown
    // fields must refuse the entire image rather than disappear in a projection.
    // Versionless envelopes are the historical v1 representation.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Image {
        #[serde(rename = "version")]
        _version: Option<u64>,
        pending: Vec<DurablePendingWakeMarker>,
    }
    let image: Image = serde_json::from_str(raw)
        .map_err(|error| PendingWakeStoreError::MalformedDocument(error.to_string()))?;
    let mut keys = HashSet::new();
    for (index, marker) in image.pending.iter().enumerate() {
        validate_marker(marker, index)?;
        if !keys.insert((marker.agent_id.clone(), marker.kind, marker.work_id.clone())) {
            return Err(PendingWakeStoreError::InvalidMarker {
                index,
                reason: "duplicate agent, kind and work identity",
            });
        }
    }
    Ok(image.pending)
}

pub fn marker_key_matches(
    marker: &DurablePendingWakeMarker,
    agent_id: &str,
    kind: PendingWakeKind,
    work_id: &str,
) -> bool {
    marker.agent_id == agent_id && marker.kind == kind && marker.work_id == work_id
}

pub fn upsert_pending_wake_marker(
    existing: &[DurablePendingWakeMarker],
    marker: DurablePendingWakeMarker,
) -> Vec<DurablePendingWakeMarker> {
    let mut next = existing
        .iter()
        .filter(|entry| {
            !marker_key_matches(entry, &marker.agent_id, marker.kind, &marker.work_id)
        })
        .cloned()
        .collect::<Vec<_>>();
    next.push(marker);
    next
}

#[derive(Clone)]
pub struct SandPendingWakeStore {
    file_path: PathBuf,
    lock: Arc<Mutex<()>>,
}

impl SandPendingWakeStore {
    pub fn new(root_dir: impl AsRef<Path>) -> Self {
        Self {
            file_path: root_dir.as_ref().join(SAND_PENDING_WAKE_FILE_NAME),
            lock: Arc::new(Mutex::new(())),
        }
    }

    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    pub fn mark_pending(&self, marker: DurablePendingWakeMarker) -> bool {
        self.mark_pending_checked(marker).is_ok()
    }

    pub fn mark_pending_checked(
        &self,
        marker: DurablePendingWakeMarker,
    ) -> Result<(), PendingWakeStoreError> {
        self.mutate_pending_checked(|pending| {
            validate_marker(&marker, 0)?;
            *pending = upsert_pending_wake_marker(pending, marker);
            Ok(())
        })
    }

    /// Compatibility inspection only. Recovery must use the checked API.
    pub fn list_pending(&self) -> Vec<DurablePendingWakeMarker> {
        self.list_pending_checked().unwrap_or_default()
    }

    pub fn list_pending_checked(
        &self,
    ) -> Result<Vec<DurablePendingWakeMarker>, PendingWakeStoreError> {
        let _guard = self.lock.lock().map_err(|_| PendingWakeStoreError::OwnerPoisoned)?;
        self.read_pending_checked_unlocked()
    }

    pub fn has_pending(
        &self,
        agent_id: &str,
        kind: PendingWakeKind,
        work_id: &str,
    ) -> bool {
        self.has_pending_checked(agent_id, kind, work_id).unwrap_or(false)
    }

    pub fn has_pending_checked(
        &self,
        agent_id: &str,
        kind: PendingWakeKind,
        work_id: &str,
    ) -> Result<bool, PendingWakeStoreError> {
        Ok(self.list_pending_checked()?
            .iter()
            .any(|entry| marker_key_matches(entry, agent_id, kind, work_id)))
    }

    pub fn clear_one(
        &self,
        agent_id: &str,
        kind: PendingWakeKind,
        work_id: &str,
    ) -> bool {
        self.clear_one_checked(agent_id, kind, work_id).unwrap_or(false)
    }

    pub fn clear_one_checked(
        &self,
        agent_id: &str,
        kind: PendingWakeKind,
        work_id: &str,
    ) -> Result<bool, PendingWakeStoreError> {
        self.mutate_pending_checked(|pending| {
            let before = pending.len();
            pending.retain(|entry| !marker_key_matches(entry, agent_id, kind, work_id));
            Ok(pending.len() != before)
        })
    }

    pub fn clear_if_matches(&self, marker: &DurablePendingWakeMarker) -> bool {
        self.clear_if_matches_checked(marker).unwrap_or(false)
    }

    pub fn clear_if_matches_checked(
        &self,
        marker: &DurablePendingWakeMarker,
    ) -> Result<bool, PendingWakeStoreError> {
        self.mutate_pending_checked(|pending| {
            validate_marker(marker, 0)?;
            let before = pending.len();
            pending.retain(|entry| entry != marker);
            Ok(pending.len() != before)
        })
    }

    /// Admit existing recovery only while its complete observed marker remains
    /// current. This validates the current image but never writes it.
    ///
    /// `admit` may only perform bounded in-memory queue admission. The lock
    /// order is this store then BackgroundWakes. It must not reenter the store,
    /// perform Session/provider I/O, notify observers or launch workers. Those
    /// operations run after this method returns and both locks are released.
    /// Serialization covers clones, not independent stores or other processes.
    pub fn with_matching_marker_checked<R>(
        &self,
        expected: &DurablePendingWakeMarker,
        admit: impl FnOnce() -> R,
    ) -> Result<Option<R>, PendingWakeStoreError> {
        let _guard = self.lock.lock().map_err(|_| PendingWakeStoreError::OwnerPoisoned)?;
        let pending = self.read_pending_checked_unlocked()?;
        validate_marker(expected, 0)?;
        Ok(pending.iter().any(|entry| entry == expected).then(admit))
    }

    pub fn mark_if_absent_or_matches(&self, marker: &DurablePendingWakeMarker) -> bool {
        self.mark_if_absent_or_matches_checked(marker).unwrap_or(false)
    }

    pub fn mark_if_absent_or_matches_checked(
        &self,
        marker: &DurablePendingWakeMarker,
    ) -> Result<bool, PendingWakeStoreError> {
        self.mutate_pending_checked(|pending| {
            validate_marker(marker, 0)?;
            if pending.iter().any(|entry| entry == marker) {
                return Ok(true);
            }
            if pending.iter().any(|entry| {
                marker_key_matches(entry, &marker.agent_id, marker.kind, &marker.work_id)
            }) {
                return Ok(false);
            }
            pending.push(marker.clone());
            Ok(true)
        })
    }

    /// Transfer an existing recovery marker to an admitted watch without
    /// recreating settled work or overwriting a newer same-key marker.
    pub fn replace_if_matches(
        &self,
        expected: &DurablePendingWakeMarker,
        replacement: DurablePendingWakeMarker,
    ) -> bool {
        self.replace_if_matches_checked(expected, replacement).unwrap_or(false)
    }

    pub fn replace_if_matches_checked(
        &self,
        expected: &DurablePendingWakeMarker,
        replacement: DurablePendingWakeMarker,
    ) -> Result<bool, PendingWakeStoreError> {
        self.mutate_pending_checked(|pending| {
            validate_marker(expected, 0)?;
            validate_marker(&replacement, 0)?;
            if !marker_key_matches(
                &replacement,
                &expected.agent_id,
                expected.kind,
                &expected.work_id,
            ) {
                return Ok(false);
            }
            let Some(index) = pending.iter().position(|entry| entry == expected) else {
                return Ok(false);
            };
            pending[index] = replacement;
            Ok(true)
        })
    }

    pub fn clear_agent(&self, agent_id: &str) {
        let _ = self.clear_agent_checked(agent_id);
    }

    pub fn clear_agent_checked(&self, agent_id: &str) -> Result<bool, PendingWakeStoreError> {
        self.mutate_pending_checked(|pending| {
            let before = pending.len();
            pending.retain(|entry| entry.agent_id != agent_id);
            Ok(pending.len() != before)
        })
    }

    pub fn clear_all(&self) {
        let _ = self.clear_all_checked();
    }

    pub fn clear_all_checked(&self) -> Result<(), PendingWakeStoreError> {
        let _guard = self.lock.lock().map_err(|_| PendingWakeStoreError::OwnerPoisoned)?;
        self.read_pending_checked_unlocked()?;
        self.delete_unlocked_with(|target| fs::remove_file(target))
    }

    pub fn prune_stale(
        &self,
        max_age_ms: f64,
        now_ms: f64,
    ) -> Vec<DurablePendingWakeMarker> {
        self.prune_stale_checked(max_age_ms, now_ms).unwrap_or_default()
    }

    pub fn prune_stale_checked(
        &self,
        max_age_ms: f64,
        now_ms: f64,
    ) -> Result<Vec<DurablePendingWakeMarker>, PendingWakeStoreError> {
        self.mutate_pending_checked(|pending| {
            let mut pruned = Vec::new();
            pending.retain(|entry| {
                // Elapsed time cannot settle a delivery failure whose follow-up
                // may still be deferred or outcome-unknown.
                if entry.kind != PendingWakeKind::ChannelDelivery
                    && now_ms - entry.marked_at_ms > max_age_ms
                {
                    pruned.push(entry.clone());
                    false
                } else {
                    true
                }
            });
            Ok(pruned)
        })
    }

    fn mutate_pending_checked<R>(
        &self,
        mutate: impl FnOnce(&mut Vec<DurablePendingWakeMarker>) -> Result<R, PendingWakeStoreError>,
    ) -> Result<R, PendingWakeStoreError> {
        self.mutate_pending_with_file_ops_checked(
            mutate,
            |part, target| fs::rename(part, target),
            |target| fs::remove_file(target),
        )
    }

    fn mutate_pending_with_file_ops_checked<R>(
        &self,
        mutate: impl FnOnce(&mut Vec<DurablePendingWakeMarker>) -> Result<R, PendingWakeStoreError>,
        replace: impl FnOnce(&Path, &Path) -> std::io::Result<()>,
        remove: impl FnOnce(&Path) -> std::io::Result<()>,
    ) -> Result<R, PendingWakeStoreError> {
        let _guard = self.lock.lock().map_err(|_| PendingWakeStoreError::OwnerPoisoned)?;
        let existing = self.read_pending_checked_unlocked()?;
        let mut pending = existing.clone();
        let result = mutate(&mut pending)?;
        if pending != existing {
            if pending.is_empty() {
                self.delete_unlocked_with(remove)?;
            } else {
                self.write_unlocked_with_replace(&pending, replace)?;
            }
        }
        // No clear/prune/admission result escapes before its publication succeeds.
        Ok(result)
    }

    fn read_pending_checked_unlocked(
        &self,
    ) -> Result<Vec<DurablePendingWakeMarker>, PendingWakeStoreError> {
        match fs::read_to_string(&self.file_path) {
            Ok(raw) => parse_pending_wake_file_checked(&raw),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                // A dangling link is unreadable retained state, not an absent
                // file that a later mutation may replace with an empty image.
                match fs::symlink_metadata(&self.file_path) {
                    Err(missing) if missing.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
                    _ => Err(PendingWakeStoreError::Io { operation: "read", error }),
                }
            }
            Err(error) => Err(PendingWakeStoreError::Io { operation: "read", error }),
        }
    }

    fn write_unlocked_with_replace(
        &self,
        pending: &[DurablePendingWakeMarker],
        replace: impl FnOnce(&Path, &Path) -> std::io::Result<()>,
    ) -> Result<(), PendingWakeStoreError> {
        #[derive(Serialize)]
        struct File<'a> {
            version: u8,
            pending: &'a [DurablePendingWakeMarker],
        }
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).map_err(|error| PendingWakeStoreError::Io {
                operation: "create directory", error,
            })?;
        }
        let part = self.file_path.with_extension(format!("json.{}.part", uuid::Uuid::new_v4()));
        let mut created = false;
        let result = (|| -> std::io::Result<()> {
            let bytes = serde_json::to_vec(&File {
                version: 1,
                pending,
            })
            .map_err(std::io::Error::other)?;
            let mut file = fs::OpenOptions::new().create_new(true).write(true).open(&part)?;
            created = true;
            file.write_all(&bytes)?;
            file.sync_all()?;
            drop(file);
            // A failed replacement must never delete the current target.
            replace(&part, &self.file_path)
        })();
        if result.is_err() && created {
            let _ = fs::remove_file(&part);
        }
        result.map_err(|error| PendingWakeStoreError::Io { operation: "write", error })
    }

    fn delete_unlocked_with(
        &self,
        remove: impl FnOnce(&Path) -> std::io::Result<()>,
    ) -> Result<(), PendingWakeStoreError> {
        match remove(&self.file_path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(PendingWakeStoreError::Io { operation: "delete", error }),
        }
    }
}

#[cfg(test)]
mod checked_pending_wake_tests {
    use super::*;
    use crate::extensions::transcript::background_wakes::BackgroundWakes;
    use std::sync::{TryLockError, mpsc};
    use std::thread;
    use std::time::Duration;

    fn marker(agent_id: &str, work_id: &str, marked_at_ms: f64) -> DurablePendingWakeMarker {
        DurablePendingWakeMarker {
            agent_id: agent_id.into(),
            kind: PendingWakeKind::ChannelDelivery,
            work_id: work_id.into(),
            marked_at_ms,
            quiet_origin: None,
            title: None,
            subagent_type: None,
            interrupted_by_recreate: false,
            delivery_operation_id: None,
            address_token: None,
            failure_reason: None,
            session_key: None,
        }
    }

    #[test]
    fn failed_pending_publication_preserves_prior_bytes_and_refuses_mutation_result() {
        let root = std::env::temp_dir().join(format!("pending-wake-publish-{}", uuid::Uuid::new_v4()));
        let store = SandPendingWakeStore::new(&root);
        let first = DurablePendingWakeMarker {
            kind: PendingWakeKind::Shell,
            ..marker("a", "first", 10.0)
        };
        let second = marker("b", "second", 20.0);
        store.mark_pending_checked(first.clone()).unwrap();
        store.mark_pending_checked(second.clone()).unwrap();
        let original = fs::read(store.file_path()).unwrap();
        let unrelated = root.join("unrelated.part");
        fs::write(&unrelated, b"not this operation's temporary file").unwrap();

        // Every checked mutation uses this exact owner commit boundary. A
        // computed return value must not escape when rename or deletion fails.
        for empty in [false, true] {
            let result = store.mutate_pending_with_file_ops_checked(
                |pending| {
                    let observed = pending.remove(0);
                    if empty {
                        pending.clear();
                    }
                    Ok(observed)
                },
                |part, target| {
                    assert!(part.is_file(), "the actual unique temporary file was written");
                    assert_ne!(part, unrelated.as_path());
                    assert_eq!(fs::read(target).unwrap(), original);
                    let staged: Value = serde_json::from_slice(&fs::read(part).unwrap()).unwrap();
                    assert_eq!(staged["version"], 1);
                    assert_eq!(staged["pending"].as_array().unwrap().len(), 1);
                    Err(std::io::Error::other("controlled pending replacement refusal"))
                },
                |target| {
                    assert_eq!(fs::read(target).unwrap(), original);
                    Err(std::io::Error::other("controlled pending deletion refusal"))
                },
            );
            let expected_operation = if empty { "delete" } else { "write" };
            assert!(matches!(result, Err(PendingWakeStoreError::Io { operation, .. })
                if operation == expected_operation));
            assert_eq!(fs::read(store.file_path()).unwrap(), original);
            assert_eq!(SandPendingWakeStore::new(&root).list_pending_checked().unwrap(),
                vec![first.clone(), second.clone()]);
            assert_eq!(fs::read_dir(&root).unwrap().count(), 2, "only the owned temporary file is removed");
            assert_eq!(fs::read(&unrelated).unwrap(), b"not this operation's temporary file");
        }

        assert_eq!(store.prune_stale_checked(5.0, 20.0).unwrap(), vec![first]);
        assert_eq!(store.list_pending_checked().unwrap(), vec![second.clone()]);
        assert!(store.clear_one_checked(&second.agent_id, second.kind, &second.work_id).unwrap());
        assert!(!store.file_path().exists());
        assert!(unrelated.is_file());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn poisoned_pending_owner_refuses_reads_admission_and_all_mutations() {
        let root = std::env::temp_dir().join(format!("pending-wake-poison-{}", uuid::Uuid::new_v4()));
        let store = SandPendingWakeStore::new(&root);
        let retained = marker("a", "work", 10.0);
        let incoming = marker("b", "new-work", 20.0);
        store.mark_pending_checked(retained.clone()).unwrap();
        let original = fs::read(store.file_path()).unwrap();
        let poisoned = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = store.with_matching_marker_checked(&retained, || -> () {
                panic!("controlled admission panic");
            });
        }));
        assert!(poisoned.is_err());
        assert!(matches!(store.list_pending_checked(), Err(PendingWakeStoreError::OwnerPoisoned)));
        assert!(matches!(store.with_matching_marker_checked(&retained, || 7), Err(PendingWakeStoreError::OwnerPoisoned)));
        assert!(store.mark_pending_checked(incoming.clone()).is_err());
        assert!(store.mark_if_absent_or_matches_checked(&incoming).is_err());
        assert!(store.replace_if_matches_checked(&retained, retained.clone()).is_err());
        assert!(store.clear_if_matches_checked(&retained).is_err());
        assert!(store.clear_one_checked("a", retained.kind, "work").is_err());
        assert!(store.clear_agent_checked("a").is_err());
        assert!(store.clear_all_checked().is_err());
        assert!(store.prune_stale_checked(1.0, 100.0).is_err());
        assert!(!store.mark_pending(incoming.clone()));
        assert!(!store.mark_if_absent_or_matches(&incoming));
        assert!(!store.replace_if_matches(&retained, retained.clone()));
        assert!(!store.clear_if_matches(&retained));
        assert!(!store.clear_one("a", retained.kind, "work"));
        store.clear_agent("a");
        store.clear_all();
        assert!(store.prune_stale(1.0, 100.0).is_empty());
        assert_eq!(fs::read(store.file_path()).unwrap(), original);
        assert_eq!(SandPendingWakeStore::new(&root).list_pending_checked().unwrap(), vec![retained]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn matching_pending_admission_holds_clone_shared_lock_until_queue_admission_finishes() {
        const WAIT: Duration = Duration::from_secs(10);
        let root = std::env::temp_dir().join(format!("pending-wake-lock-{}", uuid::Uuid::new_v4()));
        let store = SandPendingWakeStore::new(&root);
        let retained = marker("a", "work", 10.0);
        store.mark_pending_checked(retained.clone()).unwrap();
        let wakes = Arc::new(Mutex::new(BackgroundWakes::<Value>::default()));
        let queued = serde_json::to_value(&retained).unwrap();
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let (admitted_tx, admitted_rx) = mpsc::channel();
        let admitting_store = store.clone();
        let admitting_marker = retained.clone();
        let admitting_wakes = Arc::clone(&wakes);
        let admitting_value = queued.clone();
        let admission = thread::spawn(move || {
            let result = admitting_store.with_matching_marker_checked(&admitting_marker, || {
                // This hold is only an oracle. Production callbacks do not wait.
                entered_tx.send(()).unwrap();
                release_rx.recv_timeout(WAIT).expect("release exact-marker admission");
                let mut state = admitting_wakes.lock().unwrap();
                BackgroundWakes::enqueue(
                    &mut state.pending_channel_failures,
                    admitting_marker.agent_id.clone(),
                    admitting_value,
                );
                true
            });
            admitted_tx.send(result).unwrap();
        });
        entered_rx.recv_timeout(WAIT).expect("actual admission callback entered");
        let clone_store = store.clone();
        let lock_was_held = matches!(clone_store.lock.try_lock(), Err(TryLockError::WouldBlock));
        let (settled_tx, settled_rx) = mpsc::channel();
        let settling_marker = retained.clone();
        let settling_wakes = Arc::clone(&wakes);
        let settlement = thread::spawn(move || {
            let result = clone_store.clear_if_matches_checked(&settling_marker);
            let queued_before_settlement_return = settling_wakes.lock().unwrap()
                .pending_channel_failures.get(&settling_marker.agent_id)
                .is_some_and(|items| items.len() == 1);
            settled_tx.send((result, queued_before_settlement_return)).unwrap();
        });
        release_tx.send(()).unwrap();
        let admitted = admitted_rx.recv_timeout(WAIT);
        let settled = settled_rx.recv_timeout(WAIT);
        if admitted.is_ok() {
            admission.join().expect("admission worker returned");
        }
        if settled.is_ok() {
            settlement.join().expect("settlement worker returned");
        }
        assert!(lock_was_held, "exact admission holds the actual clone-shared mutex");
        assert_eq!(admitted.expect("admission completion").unwrap(), Some(true));
        let (settled, queued_before_settlement_return) = settled.expect("settlement completion");
        assert!(settled.unwrap());
        assert!(queued_before_settlement_return);
        assert_eq!(wakes.lock().unwrap().pending_channel_failures.get("a"), Some(&vec![queued]));
        assert!(store.list_pending_checked().unwrap().is_empty());
        assert_eq!(store.with_matching_marker_checked(&retained, || false).unwrap(), None);
        fs::remove_dir_all(root).unwrap();
    }
}
