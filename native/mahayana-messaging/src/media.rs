use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TransferId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TransferDirection {
    Upload,
    Download,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TransferState {
    Queued,
    Resolving,
    Transferring,
    Paused,
    Verifying,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaDescriptor {
    pub id: String,
    pub file_name: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub content_hash: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration_ms: Option<u64>,
    pub thumbnail_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaTransfer {
    pub id: TransferId,
    pub direction: TransferDirection,
    pub media: MediaDescriptor,
    pub state: TransferState,
    pub local_path: Option<String>,
    pub remote_locator: Option<String>,
    pub chunk_size: u32,
    pub transferred_bytes: u64,
    pub verified_bytes: u64,
    pub retry_count: u32,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub error: Option<String>,
}

impl MediaTransfer {
    pub fn progress_percent(&self) -> u8 {
        if self.media.size_bytes == 0 {
            return if self.state == TransferState::Completed {
                100
            } else {
                0
            };
        }
        let percent = self
            .transferred_bytes
            .saturating_mul(100)
            .checked_div(self.media.size_bytes)
            .unwrap_or_default();
        percent.min(100) as u8
    }

    pub fn remaining_bytes(&self) -> u64 {
        self.media.size_bytes.saturating_sub(self.transferred_bytes)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaTransferQueue {
    pub transfers: BTreeMap<TransferId, MediaTransfer>,
}

impl MediaTransferQueue {
    pub fn enqueue(&mut self, transfer: MediaTransfer) -> Result<(), MediaError> {
        if transfer.chunk_size == 0 {
            return Err(MediaError::InvalidChunkSize);
        }
        if self.transfers.contains_key(&transfer.id) {
            return Err(MediaError::DuplicateTransfer(transfer.id));
        }
        self.transfers.insert(transfer.id.clone(), transfer);
        Ok(())
    }

    pub fn start(&mut self, id: &TransferId, now_ms: i64) -> Result<(), MediaError> {
        let transfer = self.require_mut(id)?;
        match transfer.state {
            TransferState::Queued | TransferState::Paused | TransferState::Failed => {
                transfer.state = TransferState::Transferring;
                transfer.updated_at_ms = now_ms;
                transfer.error = None;
                Ok(())
            }
            state => Err(MediaError::InvalidTransition {
                from: state,
                to: TransferState::Transferring,
            }),
        }
    }

    pub fn acknowledge_chunk(
        &mut self,
        id: &TransferId,
        offset: u64,
        length: u32,
        now_ms: i64,
    ) -> Result<(), MediaError> {
        let transfer = self.require_mut(id)?;
        if transfer.state != TransferState::Transferring {
            return Err(MediaError::InvalidTransition {
                from: transfer.state,
                to: TransferState::Transferring,
            });
        }
        if offset != transfer.transferred_bytes {
            return Err(MediaError::UnexpectedOffset {
                expected: transfer.transferred_bytes,
                actual: offset,
            });
        }
        transfer.transferred_bytes = transfer
            .transferred_bytes
            .saturating_add(u64::from(length))
            .min(transfer.media.size_bytes);
        transfer.updated_at_ms = now_ms;
        if transfer.transferred_bytes == transfer.media.size_bytes {
            transfer.state = TransferState::Verifying;
        }
        Ok(())
    }

    pub fn complete_verification(
        &mut self,
        id: &TransferId,
        verified_bytes: u64,
        remote_locator: Option<String>,
        now_ms: i64,
    ) -> Result<(), MediaError> {
        let transfer = self.require_mut(id)?;
        if transfer.state != TransferState::Verifying {
            return Err(MediaError::InvalidTransition {
                from: transfer.state,
                to: TransferState::Completed,
            });
        }
        if verified_bytes != transfer.media.size_bytes {
            return Err(MediaError::VerificationMismatch {
                expected: transfer.media.size_bytes,
                actual: verified_bytes,
            });
        }
        transfer.verified_bytes = verified_bytes;
        transfer.remote_locator = remote_locator.or_else(|| transfer.remote_locator.clone());
        transfer.state = TransferState::Completed;
        transfer.updated_at_ms = now_ms;
        Ok(())
    }

    pub fn pause(&mut self, id: &TransferId, now_ms: i64) -> Result<(), MediaError> {
        let transfer = self.require_mut(id)?;
        if transfer.state != TransferState::Transferring {
            return Err(MediaError::InvalidTransition {
                from: transfer.state,
                to: TransferState::Paused,
            });
        }
        transfer.state = TransferState::Paused;
        transfer.updated_at_ms = now_ms;
        Ok(())
    }

    pub fn fail(
        &mut self,
        id: &TransferId,
        error: impl Into<String>,
        now_ms: i64,
    ) -> Result<(), MediaError> {
        let transfer = self.require_mut(id)?;
        if matches!(
            transfer.state,
            TransferState::Completed | TransferState::Cancelled
        ) {
            return Err(MediaError::TerminalTransfer(id.clone()));
        }
        transfer.state = TransferState::Failed;
        transfer.retry_count = transfer.retry_count.saturating_add(1);
        transfer.error = Some(error.into());
        transfer.updated_at_ms = now_ms;
        Ok(())
    }

    pub fn cancel(&mut self, id: &TransferId, now_ms: i64) -> Result<(), MediaError> {
        let transfer = self.require_mut(id)?;
        if transfer.state == TransferState::Completed {
            return Err(MediaError::TerminalTransfer(id.clone()));
        }
        transfer.state = TransferState::Cancelled;
        transfer.updated_at_ms = now_ms;
        Ok(())
    }

    fn require_mut(&mut self, id: &TransferId) -> Result<&mut MediaTransfer, MediaError> {
        self.transfers
            .get_mut(id)
            .ok_or_else(|| MediaError::TransferNotFound(id.clone()))
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MediaResolvePriority {
    UserVisible,
    Viewport,
    Background,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaResolveState {
    Queued,
    Flight,
    Done,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaResolveLease {
    pub media_id: String,
    pub generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MediaResolveEntry {
    generation: u64,
    priority: MediaResolvePriority,
    sequence: u64,
    state: MediaResolveState,
    deadline_at_ms: Option<i64>,
}

/// Transient scheduler for remote media/resource resolution.
///
/// This belongs to the same canonical media owner as \`MediaTransferQueue\`,
/// but deliberately is not serialized: queue claims, deadlines and callback
/// generations are process-lifetime coordination. Durable media truth remains
/// in the transfer/cache/domain stores. A refreshed request receives a new
/// generation, so a late callback carrying an older lease cannot settle it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaResolveCoordinator {
    max_in_flight: usize,
    next_generation: u64,
    next_sequence: u64,
    entries: BTreeMap<String, MediaResolveEntry>,
}

impl MediaResolveCoordinator {
    pub fn new(max_in_flight: usize) -> Result<Self, MediaResolveError> {
        if max_in_flight == 0 {
            return Err(MediaResolveError::InvalidInFlightBudget);
        }
        Ok(Self {
            max_in_flight,
            next_generation: 1,
            next_sequence: 0,
            entries: BTreeMap::new(),
        })
    }

    pub fn in_flight(&self) -> usize {
        self.entries
            .values()
            .filter(|entry| entry.state == MediaResolveState::Flight)
            .count()
    }

    pub fn state(&self, media_id: &str) -> Option<MediaResolveState> {
        self.entries.get(media_id).map(|entry| entry.state)
    }

    /// Enqueue an unresolved item or promote an existing queued/flight claim.
    /// Finished/failed/cancelled entries start a fresh generation when
    /// enqueued again.
    pub fn enqueue(
        &mut self,
        media_id: impl Into<String>,
        priority: MediaResolvePriority,
    ) -> Result<u64, MediaResolveError> {
        let media_id = media_id.into();
        if media_id.trim().is_empty() {
            return Err(MediaResolveError::InvalidMediaId);
        }
        if let Some(entry) = self.entries.get_mut(&media_id) {
            match entry.state {
                MediaResolveState::Queued | MediaResolveState::Flight => {
                    if priority < entry.priority {
                        entry.priority = priority;
                    }
                    return Ok(entry.generation);
                }
                MediaResolveState::Done
                | MediaResolveState::Failed
                | MediaResolveState::Cancelled => {}
            }
        }
        Ok(self.replace_generation(media_id, priority))
    }

    /// Force a fresh source generation even if an older request is currently
    /// in flight. The previous lease immediately becomes stale and no longer
    /// occupies an in-flight slot.
    pub fn refresh(
        &mut self,
        media_id: impl Into<String>,
        priority: MediaResolvePriority,
    ) -> Result<u64, MediaResolveError> {
        let media_id = media_id.into();
        if media_id.trim().is_empty() {
            return Err(MediaResolveError::InvalidMediaId);
        }
        Ok(self.replace_generation(media_id, priority))
    }

    pub fn start_next(
        &mut self,
        now_ms: i64,
        timeout_ms: i64,
    ) -> Result<Option<MediaResolveLease>, MediaResolveError> {
        if timeout_ms <= 0 {
            return Err(MediaResolveError::InvalidTimeout);
        }
        if self.in_flight() >= self.max_in_flight {
            return Ok(None);
        }
        let candidate = self
            .entries
            .iter()
            .filter(|(_, entry)| entry.state == MediaResolveState::Queued)
            .min_by_key(|(media_id, entry)| {
                (entry.priority, entry.sequence, (*media_id).clone())
            })
            .map(|(media_id, _)| media_id.clone());
        let Some(media_id) = candidate else {
            return Ok(None);
        };
        let entry = self.entries.get_mut(&media_id).expect("candidate exists");
        entry.state = MediaResolveState::Flight;
        entry.deadline_at_ms = Some(now_ms.saturating_add(timeout_ms));
        Ok(Some(MediaResolveLease {
            media_id,
            generation: entry.generation,
        }))
    }

    pub fn complete(&mut self, lease: &MediaResolveLease) -> bool {
        self.settle(lease, MediaResolveState::Done)
    }

    pub fn fail(&mut self, lease: &MediaResolveLease) -> bool {
        self.settle(lease, MediaResolveState::Failed)
    }

    pub fn cancel(&mut self, media_id: &str) -> bool {
        let Some(entry) = self.entries.get_mut(media_id) else {
            return false;
        };
        if matches!(
            entry.state,
            MediaResolveState::Done | MediaResolveState::Cancelled
        ) {
            return false;
        }
        entry.state = MediaResolveState::Cancelled;
        entry.deadline_at_ms = None;
        true
    }

    /// Fail every current in-flight generation whose deadline has elapsed and
    /// return the exact leases that were retired. Callers can use the returned
    /// ids to cancel platform fetches; any completion arriving afterward is
    /// rejected by \`complete\`.
    pub fn expire(&mut self, now_ms: i64) -> Vec<MediaResolveLease> {
        let expired = self
            .entries
            .iter()
            .filter(|(_, entry)| {
                entry.state == MediaResolveState::Flight
                    && entry
                        .deadline_at_ms
                        .is_some_and(|deadline| now_ms >= deadline)
            })
            .map(|(media_id, entry)| MediaResolveLease {
                media_id: media_id.clone(),
                generation: entry.generation,
            })
            .collect::<Vec<_>>();
        for lease in &expired {
            let _ = self.settle(lease, MediaResolveState::Failed);
        }
        expired
    }

    fn replace_generation(
        &mut self,
        media_id: String,
        priority: MediaResolvePriority,
    ) -> u64 {
        let generation = self.next_generation;
        self.next_generation = self.next_generation.saturating_add(1);
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.saturating_add(1);
        self.entries.insert(
            media_id,
            MediaResolveEntry {
                generation,
                priority,
                sequence,
                state: MediaResolveState::Queued,
                deadline_at_ms: None,
            },
        );
        generation
    }

    fn settle(
        &mut self,
        lease: &MediaResolveLease,
        target: MediaResolveState,
    ) -> bool {
        let Some(entry) = self.entries.get_mut(&lease.media_id) else {
            return false;
        };
        if entry.generation != lease.generation
            || entry.state != MediaResolveState::Flight
        {
            return false;
        }
        entry.state = target;
        entry.deadline_at_ms = None;
        true
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum MediaResolveError {
    #[error("media resolve in-flight budget must be greater than zero")]
    InvalidInFlightBudget,
    #[error("media resolve requires a media id")]
    InvalidMediaId,
    #[error("media resolve timeout must be greater than zero")]
    InvalidTimeout,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum MediaError {
    #[error("media transfer {0:?} already exists")]
    DuplicateTransfer(TransferId),
    #[error("media transfer {0:?} was not found")]
    TransferNotFound(TransferId),
    #[error("media transfer chunk size must be greater than zero")]
    InvalidChunkSize,
    #[error("media transfer cannot transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: TransferState,
        to: TransferState,
    },
    #[error("media transfer expected offset {expected}, got {actual}")]
    UnexpectedOffset { expected: u64, actual: u64 },
    #[error("media verification expected {expected} bytes, got {actual}")]
    VerificationMismatch { expected: u64, actual: u64 },
    #[error("media transfer {0:?} is already terminal")]
    TerminalTransfer(TransferId),
}
