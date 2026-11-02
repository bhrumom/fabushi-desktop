use std::fs;
use std::collections::HashSet;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::durable_file_policy::SAND_ACK_OBLIGATIONS_FILE_NAME;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AckObligation {
    pub agent_id: String,
    pub created_at_ms: f64,
    pub last_send_at_ms: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_interrupt_at_ms: Option<f64>,
    pub coalesced_count: f64,
    pub redrive_attempts: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordSendOutcome {
    pub obligation: AckObligation,
    pub created: bool,
}

pub fn finite_number(value: Option<&Value>, fallback: f64) -> f64 {
    value
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
        .unwrap_or(fallback)
}

pub fn coerce_obligation(value: &Value) -> Option<AckObligation> {
    let entry = value.as_object()?;
    let agent_id = entry
        .get("agentId")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())?
        .to_string();
    let created_at_ms = finite_number(entry.get("createdAtMs"), 0.0);
    Some(AckObligation {
        agent_id,
        created_at_ms,
        last_send_at_ms: finite_number(entry.get("lastSendAtMs"), created_at_ms),
        last_interrupt_at_ms: entry
            .get("lastInterruptAtMs")
            .and_then(Value::as_f64)
            .filter(|value| value.is_finite()),
        coalesced_count: finite_number(entry.get("coalescedCount"), 1.0).max(1.0),
        redrive_attempts: finite_number(entry.get("redriveAttempts"), 0.0).max(0.0),
    })
}

pub fn parse_ack_obligations_file(raw: Option<&str>) -> Vec<AckObligation> {
    let Some(raw) = raw else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_str::<Value>(raw) else {
        return Vec::new();
    };
    value
        .get("pending")
        .and_then(Value::as_array)
        .map(|pending| pending.iter().filter_map(coerce_obligation).collect())
        .unwrap_or_default()
}

pub struct SandAckObligationStore {
    file_path: PathBuf,
    cache: Mutex<Option<Vec<AckObligation>>>,
}

pub(super) struct CheckedAckAbsence<'a> {
    _guard: MutexGuard<'a, Option<Vec<AckObligation>>>,
}

impl SandAckObligationStore {
    pub fn new(root_dir: impl AsRef<Path>) -> Self {
        Self {
            file_path: root_dir.as_ref().join(SAND_ACK_OBLIGATIONS_FILE_NAME),
            cache: Mutex::new(None),
        }
    }

    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    pub fn get(&self, agent_id: &str) -> Option<AckObligation> {
        self.list()
            .into_iter()
            .find(|entry| entry.agent_id == agent_id)
    }

    /// A positive absence decision must read the complete current file, not a
    /// cached or lenient projection. Errors retain independent ACK custody.
    pub fn get_checked(&self, agent_id: &str) -> io::Result<Option<AckObligation>> {
        Ok(self.list_checked()?.into_iter().find(|entry| entry.agent_id == agent_id))
    }

    pub fn list_checked(&self) -> io::Result<Vec<AckObligation>> {
        let _guard = self.cache.lock().map_err(|_| io::Error::other("ACK owner is poisoned"))?;
        self.read_checked_unlocked()
    }

    pub(super) fn checked_absence_guard(&self, agent_id: &str) -> io::Result<Option<CheckedAckAbsence<'_>>> {
        let guard = self.cache.lock().map_err(|_| io::Error::other("ACK owner is poisoned"))?;
        if self.read_checked_unlocked()?.iter().any(|entry| entry.agent_id == agent_id) {
            return Ok(None);
        }
        Ok(Some(CheckedAckAbsence { _guard: guard }))
    }

    fn read_checked_unlocked(&self) -> io::Result<Vec<AckObligation>> {
        let raw = match fs::read_to_string(&self.file_path) {
            Ok(raw) => raw,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error),
        };
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Image {
            version: u64,
            pending: Vec<AckObligation>,
        }
        let image: Image = serde_json::from_str(&raw)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        if image.version != 1 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "unsupported ACK version"));
        }
        let mut agents = HashSet::new();
        for entry in &image.pending {
            if entry.agent_id.trim().is_empty()
                || !agents.insert(entry.agent_id.as_str())
                || !entry.created_at_ms.is_finite()
                || !entry.last_send_at_ms.is_finite()
                || entry.last_interrupt_at_ms.is_some_and(|value| !value.is_finite())
                || !entry.coalesced_count.is_finite()
                || entry.coalesced_count < 1.0
                || entry.coalesced_count.fract() != 0.0
                || !entry.redrive_attempts.is_finite()
                || entry.redrive_attempts < 0.0
                || entry.redrive_attempts.fract() != 0.0
            {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid or duplicate ACK obligation"));
            }
        }
        // Checked inspection does not promote a disk image into the lenient
        // compatibility cache. All owner mutations hold this same mutex.
        Ok(image.pending)
    }

    pub fn list(&self) -> Vec<AckObligation> {
        if let Ok(mut cache) = self.cache.lock() {
            return self.cached_pending_unlocked(&mut cache);
        }
        let raw = fs::read_to_string(&self.file_path).ok();
        parse_ack_obligations_file(raw.as_deref())
    }

    fn cached_pending_unlocked(&self, cache: &mut Option<Vec<AckObligation>>) -> Vec<AckObligation> {
        if cache.is_none() {
            let raw = fs::read_to_string(&self.file_path).ok();
            *cache = Some(parse_ack_obligations_file(raw.as_deref()));
        }
        cache.as_ref().expect("ACK cache initialized").clone()
    }

    fn mutate_pending<T>(
        &self,
        checked: bool,
        change: impl FnOnce(&mut Vec<AckObligation>) -> T,
    ) -> io::Result<T> {
        let mut cache = self.cache.lock().map_err(|_| io::Error::other("ACK owner is poisoned"))?;
        let mut pending = if checked {
            self.read_checked_unlocked()?
        } else {
            self.cached_pending_unlocked(&mut cache)
        };
        let before = pending.clone();
        let result = change(&mut pending);
        if pending != before {
            self.write_unlocked(&pending)?;
        }
        *cache = Some(pending);
        Ok(result)
    }

    /// A pure owner-local transition over a freshly checked obligation. The
    /// caller owns policy; this store serializes the image and atomic replace.
    pub(super) fn update_checked<T>(
        &self,
        agent_id: &str,
        change: impl FnOnce(Option<AckObligation>) -> (Option<AckObligation>, T),
    ) -> io::Result<T> {
        self.mutate_pending(true, |pending| {
            let index = pending.iter().position(|entry| entry.agent_id == agent_id);
            let previous = index.map(|index| pending[index].clone());
            let (next, result) = change(previous);
            match (index, next) {
                (Some(index), Some(next)) => pending[index] = next,
                (Some(index), None) => { pending.remove(index); }
                (None, Some(next)) => pending.push(next),
                (None, None) => {}
            }
            result
        })
    }

    pub fn record_send(&self, agent_id: &str, at_ms: f64) -> io::Result<RecordSendOutcome> {
        self.mutate_pending(false, |pending| {
            let existing = pending.iter().find(|entry| entry.agent_id == agent_id).cloned();
            let obligation = match existing.as_ref() {
                Some(existing) => AckObligation {
                    last_send_at_ms: at_ms,
                    coalesced_count: existing.coalesced_count + 1.0,
                    ..existing.clone()
                },
                None => AckObligation {
                    agent_id: agent_id.to_string(),
                    created_at_ms: at_ms,
                    last_send_at_ms: at_ms,
                    last_interrupt_at_ms: None,
                    coalesced_count: 1.0,
                    redrive_attempts: 0.0,
                },
            };
            pending.retain(|entry| entry.agent_id != agent_id);
            pending.push(obligation.clone());
            RecordSendOutcome {
                obligation,
                created: existing.is_none(),
            }
        })
    }

    pub fn record_interrupt(&self, agent_id: &str, at_ms: f64) -> io::Result<bool> {
        self.mutate_pending(false, |pending| {
            let Some(mut existing) = pending.iter().find(|entry| entry.agent_id == agent_id).cloned() else {
                return false;
            };
            existing.last_interrupt_at_ms = Some(at_ms);
            pending.retain(|entry| entry.agent_id != agent_id);
            pending.push(existing);
            true
        })
    }

    pub fn record_redrive_attempt(&self, agent_id: &str) -> io::Result<Option<AckObligation>> {
        self.mutate_pending(false, |pending| {
            let Some(mut existing) = pending.iter().find(|entry| entry.agent_id == agent_id).cloned() else {
                return None;
            };
            existing.redrive_attempts += 1.0;
            pending.retain(|entry| entry.agent_id != agent_id);
            pending.push(existing.clone());
            Some(existing)
        })
    }

    pub fn clear(&self, agent_id: &str) -> io::Result<bool> {
        self.mutate_pending(false, |pending| {
            let count = pending.len();
            pending.retain(|entry| entry.agent_id != agent_id);
            count != pending.len()
        })
    }

    pub fn upsert(&self, obligation: AckObligation) -> io::Result<()> {
        self.mutate_pending(false, |pending| {
            pending.retain(|entry| entry.agent_id != obligation.agent_id);
            pending.push(obligation);
        })
    }

    pub fn restore(&self, previous: Option<AckObligation>, agent_id: &str) -> io::Result<()> {
        match previous {
            Some(previous) => self.upsert(previous),
            None => {
                let _ = self.clear(agent_id)?;
                Ok(())
            }
        }
    }

    fn write_unlocked(&self, pending: &[AckObligation]) -> io::Result<()> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let part = self.file_path.with_extension(format!("json.{}.part", uuid::Uuid::new_v4()));
        let result = (|| -> io::Result<()> {
            let mut file = fs::OpenOptions::new().create_new(true).write(true).open(&part)?;
            let bytes = serde_json::to_vec(&json!({
                "version": 1,
                "pending": pending,
            }))
            .map_err(io::Error::other)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            drop(file);
            fs::rename(&part, &self.file_path)
        })();
        if result.is_err() {
            let _ = fs::remove_file(&part);
        }
        result
    }
}

#[cfg(test)]
mod checked_reads {
    use super::*;

    #[test]
    fn checked_absence_holds_the_real_ack_writer_until_marker_decision_finishes() {
        use std::sync::{Arc, mpsc};
        use std::time::Duration;
        let root = std::env::temp_dir().join(format!("fabushi-ack-absence-{}", uuid::Uuid::new_v4()));
        let store = Arc::new(SandAckObligationStore::new(&root));
        let absence = store.checked_absence_guard("agent").unwrap().expect("checked missing file");
        assert!(matches!(store.cache.try_lock(), Err(std::sync::TryLockError::WouldBlock)),
            "the actual mutation mutex must remain held for the decision");
        let (entered_tx, entered_rx) = mpsc::channel();
        let (finished_tx, finished_rx) = mpsc::channel();
        let writer_store = Arc::clone(&store);
        let writer = std::thread::spawn(move || {
            entered_tx.send(()).unwrap();
            let result = writer_store.record_send("agent", 10.0);
            finished_tx.send(result).unwrap();
        });
        entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(finished_rx.try_recv().is_err());
        assert!(!store.file_path().exists(), "no owner write may precede guard release");
        drop(absence);
        let result = finished_rx.recv_timeout(Duration::from_secs(5)).unwrap().unwrap();
        writer.join().unwrap();
        assert!(result.created);
        assert_eq!(store.get_checked("agent").unwrap(), Some(result.obligation));
        assert!(store.checked_absence_guard("agent").unwrap().is_none());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn checked_transition_reads_fresh_bytes_and_serialized_sends_keep_every_obligation() {
        use std::sync::{Arc, Barrier};
        let root = std::env::temp_dir().join(format!("fabushi-ack-transition-{}", uuid::Uuid::new_v4()));
        let store = Arc::new(SandAckObligationStore::new(&root));
        store.record_send("agent", 1.0).unwrap();
        let original = fs::read(store.file_path()).unwrap();
        fs::write(store.file_path(), b"{invalid owner image").unwrap();
        assert!(store.update_checked("agent", |_| -> (Option<AckObligation>, ()) {
            panic!("an unreadable fresh image must not invoke the transition");
        }).is_err());
        assert_eq!(fs::read(store.file_path()).unwrap(), b"{invalid owner image");
        fs::write(store.file_path(), &original).unwrap();
        let barrier = Arc::new(Barrier::new(4));
        let writers = (0..4).map(|worker| {
            let store = Arc::clone(&store);
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                for index in 0..8 {
                    store.record_send("agent", f64::from(worker * 8 + index + 2)).unwrap();
                }
            })
        }).collect::<Vec<_>>();
        for writer in writers { writer.join().unwrap(); }
        assert_eq!(store.get_checked("agent").unwrap().unwrap().coalesced_count, 33.0);
        let reopened = SandAckObligationStore::new(&root);
        assert_eq!(reopened.get_checked("agent").unwrap(), store.get_checked("agent").unwrap());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn checked_ack_read_refuses_poison_without_overwriting_the_durable_obligation() {
        let root = std::env::temp_dir().join(format!("fabushi-ack-poison-{}", uuid::Uuid::new_v4()));
        let store = SandAckObligationStore::new(&root);
        store.record_send("agent", 1.0).unwrap();
        let before = fs::read(store.file_path()).unwrap();
        let poisoned = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = store.cache.lock().unwrap();
            panic!("controlled ACK owner poison");
        }));
        assert!(poisoned.is_err());
        assert!(store.list_checked().is_err());
        assert!(store.get_checked("agent").is_err());
        assert_eq!(fs::read(store.file_path()).unwrap(), before);
        fs::remove_dir_all(root).unwrap();
    }
}
