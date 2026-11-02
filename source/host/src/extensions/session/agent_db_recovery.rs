use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Map, Value};

use crate::storage::sqlite_busy::{is_sqlite_corrupt_error, is_sqlite_io_error};
use crate::storage::sqlite_recovery::{
    copy_salvageable_sqlite_rows, open_sqlite_for_salvage, quarantine_corrupt_sqlite_db,
};
use crate::storage::store_db::{DB_BUSY_TIMEOUT_MS, SQLITE_DB_SIDECAR_SUFFIXES};
use crate::transcript_mutation_events::publish_transcript_mutation;

use super::agent_db_schema::AGENT_DB_SCHEMA;
use super::session_diagnostics::{SessionDiagnostic, report_session_diagnostic};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SalvageCounts {
    pub kv: usize,
    pub blobs: usize,
    pub transcript: usize,
}

impl SalvageCounts {
    pub fn total(self) -> usize {
        self.kv + self.blobs + self.transcript
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorruptionRecoveredEvent {
    pub outcome: &'static str,
    pub quarantine_path: Option<PathBuf>,
    pub salvaged: SalvageCounts,
}

pub type CorruptionRecoveredCallback =
    Arc<dyn Fn(&CorruptionRecoveredEvent) + Send + Sync + 'static>;

#[derive(Clone)]
pub struct DbRecoveryOptions {
    pub recover_on_corruption: bool,
    pub reject_if_corrupt: bool,
    pub verify_integrity_on_open: bool,
    pub busy_timeout_ms: u64,
    pub on_corruption_recovered: Option<CorruptionRecoveredCallback>,
}

impl Default for DbRecoveryOptions {
    fn default() -> Self {
        Self {
            recover_on_corruption: true,
            reject_if_corrupt: false,
            verify_integrity_on_open: true,
            busy_timeout_ms: DB_BUSY_TIMEOUT_MS,
            on_corruption_recovered: None,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AgentDbRecoveryError {
    #[error("agent store sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("agent store filesystem error: {0}")]
    Io(#[from] std::io::Error),
    #[error("store.db failed integrity check for {0}")]
    Integrity(String),
    #[error("automation custody recovery authority is unavailable: {0}")]
    AutomationCustodyUnavailable(&'static str),
}

pub const AUTOMATION_CUSTODY_UNAVAILABLE_KV: &str = "automationCustodyAuthorityUnavailable";
const AUTOMATION_CUSTODY_UNAVAILABLE_VALUE: &str =
    "{\"version\":1,\"reason\":\"cron_authority_not_recovered\"}";

fn adjacent_path(db_path: &Path, suffix: &str) -> PathBuf {
    let mut path = db_path.as_os_str().to_os_string();
    path.push(suffix);
    PathBuf::from(path)
}

pub fn automation_custody_recovery_marker_path(db_path: &Path) -> PathBuf {
    adjacent_path(db_path, ".automation-custody-unavailable")
}

fn is_reparse_or_symlink(metadata: &Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

fn invalid_store_object() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "Session recovery requires regular store objects")
}

fn store_parent(db_path: &Path) -> io::Result<(PathBuf, File)> {
    let name = db_path.file_name().ok_or_else(invalid_store_object)?;
    let parent = db_path.parent().filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let metadata = fs::symlink_metadata(parent)?;
    if !metadata.is_dir() || is_reparse_or_symlink(&metadata) {
        return Err(invalid_store_object());
    }
    let parent = parent.canonicalize()?;
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // Hold the directory without sharing deletion; do not attempt a
        // Windows directory-fsync substitute.
        options.custom_flags(0x0200_0000 | 0x0020_0000).share_mode(0x1 | 0x2);
    }
    let directory = options.open(&parent)?;
    check_parent_identity(&directory, &parent)?;
    Ok((parent.join(name), directory))
}

fn check_parent_identity(directory: &File, path: &Path) -> io::Result<()> {
    let opened = directory.metadata()?;
    let current = fs::symlink_metadata(path)?;
    if !opened.is_dir() || !current.is_dir()
        || is_reparse_or_symlink(&opened) || is_reparse_or_symlink(&current)
    {
        return Err(invalid_store_object());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if (opened.dev(), opened.ino()) != (current.dev(), current.ino()) {
            return Err(invalid_store_object());
        }
    }
    Ok(())
}

fn regular_store_file(path: &Path, allow_missing: bool) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() && !is_reparse_or_symlink(&metadata) => Ok(()),
        Ok(_) => Err(invalid_store_object()),
        Err(error) if allow_missing && error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn recovery_file_options(marker: bool) -> OpenOptions {
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // Marker metadata must reach the filesystem before quarantine. The
        // stable guard is lifetime exclusion, not persistent run authority.
        let flags = 0x0020_0000 | if marker { 0x8000_0000 } else { 0 };
        options.custom_flags(flags).share_mode(if marker { 0x1 } else { 0x1 | 0x2 });
    }
    #[cfg(not(windows))]
    let _ = marker;
    options
}

fn check_opened_regular(file: &File) -> io::Result<()> {
    let metadata = file.metadata()?;
    if metadata.is_file() && !is_reparse_or_symlink(&metadata) {
        Ok(())
    } else {
        Err(invalid_store_object())
    }
}

fn open_recovery_guard(db_path: &Path) -> io::Result<File> {
    let path = adjacent_path(db_path, ".automation-custody.guard");
    regular_store_file(&path, true)?;
    let file = recovery_file_options(false).create(true).truncate(false).open(path)?;
    check_opened_regular(&file)?;
    Ok(file)
}

#[derive(Debug)]
struct LockedRecoveryFile {
    file: Option<File>,
}

impl LockedRecoveryFile {
    fn acquire(db_path: &Path, exclusive: bool) -> io::Result<Self> {
        let file = open_recovery_guard(db_path)?;
        if exclusive {
            fs2::FileExt::try_lock_exclusive(&file)?;
        } else {
            fs2::FileExt::try_lock_shared(&file)?;
        }
        Ok(Self { file: Some(file) })
    }

    fn release(&mut self) -> io::Result<()> {
        if let Some(file) = self.file.as_ref() {
            // Close alone need not release a duplicated/inherited lock. Keep
            // this resource until explicit unlock is acknowledged.
            fs2::FileExt::unlock(file)?;
            drop(self.file.take());
        }
        Ok(())
    }
}

impl Drop for LockedRecoveryFile {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

/// Holds physical exclusion against participating corruption recovery. This
/// is availability evidence only; callers must still validate their run row,
/// admission, lifecycle and effect report through the existing DB owner.
#[derive(Debug)]
pub struct AutomationCustodyRecoveryGuard {
    db_path: PathBuf,
    parent: File,
    _guard: LockedRecoveryFile,
}

impl AutomationCustodyRecoveryGuard {
    pub fn require_available(&self, db: &Connection) -> Result<(), AgentDbRecoveryError> {
        check_parent_identity(&self.parent, self.db_path.parent().ok_or_else(invalid_store_object)?)?;
        regular_store_file(&self.db_path, false)?;
        let connection_path = db.path().filter(|path| !path.is_empty())
            .ok_or(AgentDbRecoveryError::AutomationCustodyUnavailable("connection has no file identity"))?;
        if Path::new(connection_path).canonicalize()? != self.db_path {
            return Err(AgentDbRecoveryError::AutomationCustodyUnavailable("connection belongs to another store"));
        }
        match fs::symlink_metadata(automation_custody_recovery_marker_path(&self.db_path)) {
            Ok(_) => return Err(AgentDbRecoveryError::AutomationCustodyUnavailable("recovery marker is present")),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        let marked = db.query_row(
            "SELECT 1 FROM kv WHERE key = ?1 LIMIT 1",
            [AUTOMATION_CUSTODY_UNAVAILABLE_KV],
            |row| row.get::<_, i64>(0),
        ).optional()?.is_some();
        if marked {
            return Err(AgentDbRecoveryError::AutomationCustodyUnavailable("reconstructed store lacks custody authority"));
        }
        Ok(())
    }
}

pub fn acquire_automation_custody_recovery_guard(
    db_path: &Path,
    db: &Connection,
) -> Result<AutomationCustodyRecoveryGuard, AgentDbRecoveryError> {
    let (db_path, parent) = store_parent(db_path)?;
    regular_store_file(&db_path, false)?;
    let guard = LockedRecoveryFile::acquire(&db_path, false)?;
    let access = AutomationCustodyRecoveryGuard { db_path, parent, _guard: guard };
    access.require_available(db)?;
    Ok(access)
}

#[cfg(unix)]
fn sync_store_parent(parent: &File) -> io::Result<()> {
    use std::os::fd::AsRawFd;
    loop {
        // The owned directory File keeps this descriptor live throughout the
        // syscall. Apple directories need fsync, not F_FULLFSYNC.
        if unsafe { libc::fsync(parent.as_raw_fd()) } == 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}

struct RecoveryStoreBoundary {
    db_path: PathBuf,
    guard: LockedRecoveryFile,
    _marker: File,
    _parent: File,
}

impl RecoveryStoreBoundary {
    fn acquire(db_path: &Path) -> Result<Self, AgentDbRecoveryError> {
        let (db_path, parent) = store_parent(db_path)?;
        regular_store_file(&db_path, true)?;
        let guard = LockedRecoveryFile::acquire(&db_path, true)?;
        for suffix in SQLITE_DB_SIDECAR_SUFFIXES {
            regular_store_file(&adjacent_path(&db_path, suffix), true)?;
        }
        let marker_path = automation_custody_recovery_marker_path(&db_path);
        let new_marker = match fs::symlink_metadata(&marker_path) {
            Ok(metadata) if metadata.is_file() && !is_reparse_or_symlink(&metadata) => false,
            Ok(_) => return Err(invalid_store_object().into()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => true,
            Err(error) => return Err(error.into()),
        };
        let mut options = recovery_file_options(true);
        if new_marker { options.create_new(true); }
        let mut marker = options.open(&marker_path)?;
        check_opened_regular(&marker)?;
        if new_marker {
            marker.write_all(AUTOMATION_CUSTODY_UNAVAILABLE_VALUE.as_bytes())?;
        }
        marker.sync_all()?;
        #[cfg(unix)]
        sync_store_parent(&parent)?;
        #[cfg(target_os = "macos")]
        marker.sync_all()?;
        #[cfg(not(any(unix, windows)))]
        return Err(io::Error::new(io::ErrorKind::Unsupported, "Session recovery marker durability is unsupported").into());
        check_parent_identity(&parent, db_path.parent().ok_or_else(invalid_store_object)?)?;
        Ok(Self { db_path, guard, _marker: marker, _parent: parent })
    }

    fn finish(mut self) -> Result<(), AgentDbRecoveryError> {
        self.guard.release()?;
        Ok(())
    }
}

fn diagnostic(
    kind: &str,
    agent_id: &str,
    metadata: impl IntoIterator<Item = (String, Value)>,
) {
    let mut values = metadata
        .into_iter()
        .collect::<std::collections::BTreeMap<_, _>>();
    values.insert("agentId".into(), Value::String(agent_id.to_string()));
    report_session_diagnostic(&SessionDiagnostic {
        family: "store_db".into(),
        kind: kind.into(),
        metadata: values,
    });
}

fn configured_connection(
    db_path: &Path,
    agent_dir_name: &str,
    options: &DbRecoveryOptions,
) -> Result<Connection, AgentDbRecoveryError> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let db = Connection::open(db_path)?;
    db.busy_timeout(Duration::from_millis(options.busy_timeout_ms))?;
    if let Err(error) = db.pragma_update(None, "journal_mode", "WAL") {
        diagnostic(
            "wal_unavailable",
            agent_dir_name,
            [("errorClass".into(), Value::String(error.to_string()))],
        );
    }
    if let Err(error) = db.pragma_update(None, "synchronous", "NORMAL") {
        diagnostic(
            "wal_unavailable",
            agent_dir_name,
            [("errorClass".into(), Value::String(error.to_string()))],
        );
    }
    Ok(db)
}

fn initialize_schema(db: &Connection) -> Result<(), AgentDbRecoveryError> {
    db.execute_batch(AGENT_DB_SCHEMA)?;
    let has_fence_state_known = db
        .query_row(
            "SELECT 1 FROM pragma_table_info('cron_session_runs') WHERE name = 'fence_state_known' LIMIT 1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .is_some();
    if !has_fence_state_known {
        db.execute_batch(
            "ALTER TABLE cron_session_runs ADD COLUMN fence_state_known INTEGER NOT NULL DEFAULT 0",
        )?;
    }
    Ok(())
}

pub fn init_configured_db(
    db_path: &Path,
    agent_dir_name: &str,
    options: &DbRecoveryOptions,
) -> Result<Connection, AgentDbRecoveryError> {
    let db = configured_connection(db_path, agent_dir_name, options)?;
    initialize_schema(&db)?;
    Ok(db)
}

fn init_recovered_db(
    db_path: &Path,
    agent_dir_name: &str,
    options: &DbRecoveryOptions,
) -> Result<Connection, AgentDbRecoveryError> {
    let db = configured_connection(db_path, agent_dir_name, options)?;
    db.execute_batch("BEGIN IMMEDIATE")?;
    let result = (|| -> Result<(), AgentDbRecoveryError> {
        initialize_schema(&db)?;
        db.execute(
            "INSERT OR REPLACE INTO kv (key, value) VALUES (?1, ?2)",
            params![AUTOMATION_CUSTODY_UNAVAILABLE_KV, AUTOMATION_CUSTODY_UNAVAILABLE_VALUE],
        )?;
        db.execute_batch("COMMIT")?;
        Ok(())
    })();
    if let Err(error) = result {
        let _ = db.execute_batch("ROLLBACK");
        return Err(error);
    }
    Ok(db)
}

pub fn is_store_db_healthy(db: &Connection) -> bool {
    db.query_row("PRAGMA quick_check", [], |row| row.get::<_, String>(0))
        .map(|value| value == "ok")
        .unwrap_or(false)
}

fn sqlite_error(error: &AgentDbRecoveryError) -> Option<&rusqlite::Error> {
    match error {
        AgentDbRecoveryError::Sqlite(error) => Some(error),
        AgentDbRecoveryError::Io(_) | AgentDbRecoveryError::Integrity(_)
        | AgentDbRecoveryError::AutomationCustodyUnavailable(_) => None,
    }
}

pub fn open_configured_db(
    db_path: &Path,
    agent_dir_name: &str,
    options: &DbRecoveryOptions,
    has_other_live_handles: bool,
) -> Result<Connection, AgentDbRecoveryError> {
    let recover = options.recover_on_corruption && !has_other_live_handles;
    let mut opened = None;
    let mut last_error = None;

    for attempt in 0..2 {
        match init_configured_db(db_path, agent_dir_name, options) {
            Ok(db) => {
                opened = Some(db);
                break;
            }
            Err(error) => {
                let retry_io = sqlite_error(&error).is_some_and(is_sqlite_io_error) && attempt == 0;
                if retry_io {
                    diagnostic(
                        "open_io_retry",
                        agent_dir_name,
                        [("errorClass".into(), Value::String(error.to_string()))],
                    );
                    last_error = Some(error);
                    continue;
                }
                last_error = Some(error);
                break;
            }
        }
    }

    let Some(db) = opened else {
        let error = last_error.expect("configured db open records an error");
        if recover && sqlite_error(&error).is_some_and(is_sqlite_corrupt_error) {
            return recover_corrupt_store_db(db_path, agent_dir_name, options, &error.to_string());
        }
        return Err(error);
    };

    let verify = (recover || options.reject_if_corrupt) && options.verify_integrity_on_open;
    if verify && !is_store_db_healthy(&db) {
        drop(db);
        if recover {
            diagnostic(
                "quick_check_failed",
                agent_dir_name,
                Vec::<(String, Value)>::new(),
            );
            return recover_corrupt_store_db(
                db_path,
                agent_dir_name,
                options,
                "PRAGMA quick_check failed",
            );
        }
        return Err(AgentDbRecoveryError::Integrity(agent_dir_name.to_string()));
    }
    Ok(db)
}

pub fn quarantine_corrupt_db(db_path: &Path, agent_dir_name: &str) -> Option<PathBuf> {
    let boundary = match RecoveryStoreBoundary::acquire(db_path) {
        Ok(boundary) => boundary,
        Err(error) => {
            diagnostic("quarantine_barrier_failed", agent_dir_name,
                [("errorClass".into(), Value::String(error.to_string()))]);
            return None;
        }
    };
    let quarantine = quarantine_with_boundary(&boundary, agent_dir_name);
    if let Err(error) = boundary.finish() {
        diagnostic("quarantine_unlock_failed", agent_dir_name,
            [("errorClass".into(), Value::String(error.to_string()))]);
        return None;
    }
    quarantine
}

fn quarantine_with_boundary(boundary: &RecoveryStoreBoundary, agent_dir_name: &str) -> Option<PathBuf> {
    let result = quarantine_corrupt_sqlite_db(&boundary.db_path, None, false, 1, 50);
    if result.rename_error_code.is_none() {
        return result.quarantine_path;
    }
    let error_class = result
        .rename_error_code
        .clone()
        .unwrap_or_else(|| "unknown".into());
    if result.quarantine_path.is_none() {
        diagnostic(
            "quarantine_rename_failed",
            agent_dir_name,
            [("errorClass".into(), Value::String(error_class))],
        );
        return None;
    }
    diagnostic(
        "quarantine_copied",
        agent_dir_name,
        [("errorClass".into(), Value::String(error_class))],
    );
    result.quarantine_path
}

pub fn salvage_store_db(source_path: &Path, fresh_db: &Connection) -> SalvageCounts {
    let mut counts = SalvageCounts {
        kv: 0,
        blobs: 0,
        transcript: 0,
    };
    let Some(source) = open_sqlite_for_salvage(source_path, None) else {
        return counts;
    };

    counts.kv = copy_salvageable_sqlite_rows(
        &source,
        "SELECT key, value FROM kv",
        |row| {
            let key = row.get::<_, String>(0)?;
            let value = row.get::<_, String>(1)?;
            fresh_db
                .execute(
                    "INSERT OR IGNORE INTO kv (key, value) VALUES (?1, ?2)",
                    params![key, value],
                )
                .map(|_| ())
        },
    );
    counts.blobs = copy_salvageable_sqlite_rows(
        &source,
        "SELECT id, data FROM blobs",
        |row| {
            let id = row.get::<_, String>(0)?;
            let data = row.get::<_, Vec<u8>>(1)?;
            fresh_db
                .execute(
                    "INSERT OR IGNORE INTO blobs (id, data) VALUES (?1, ?2)",
                    params![id, data],
                )
                .map(|_| ())
        },
    );
    counts.transcript = copy_salvageable_sqlite_rows(
        &source,
        "SELECT seq, id, entry FROM transcript_entries",
        |row| {
            let seq = row.get::<_, i64>(0)?;
            let id = row.get::<_, String>(1)?;
            let entry = row.get::<_, String>(2)?;
            fresh_db
                .execute(
                    "INSERT OR IGNORE INTO transcript_entries (seq, id, entry) VALUES (?1, ?2, ?3)",
                    params![seq, id, entry],
                )
                .map(|_| ())
        },
    );
    counts
}

pub fn recover_corrupt_store_db(
    db_path: &Path,
    agent_dir_name: &str,
    options: &DbRecoveryOptions,
    cause: &str,
) -> Result<Connection, AgentDbRecoveryError> {
    diagnostic(
        "corrupt_recovering",
        agent_dir_name,
        [("errorClass".into(), Value::String(cause.to_string()))],
    );
    let boundary = RecoveryStoreBoundary::acquire(db_path)?;
    let quarantine_path = quarantine_with_boundary(&boundary, agent_dir_name);
    let fresh_db = init_recovered_db(&boundary.db_path, agent_dir_name, options)?;
    let salvaged = quarantine_path
        .as_deref()
        .map(|path| salvage_store_db(path, &fresh_db))
        .unwrap_or(SalvageCounts {
            kv: 0,
            blobs: 0,
            transcript: 0,
        });
    // No external callback may run while this new filesystem guard is held.
    // The durable markers remain, independently of the guard's lifetime.
    boundary.finish()?;
    let outcome = if salvaged.total() > 0 {
        "recovered"
    } else {
        "reset"
    };
    diagnostic(
        "recovery_outcome",
        agent_dir_name,
        [
            ("outcome".into(), Value::String(outcome.into())),
            (
                "quarantine".into(),
                Value::String(if quarantine_path.is_some() {
                    "preserved".into()
                } else {
                    "none".into()
                }),
            ),
            ("salvagedKv".into(), Value::from(salvaged.kv as u64)),
            ("salvagedBlobs".into(), Value::from(salvaged.blobs as u64)),
            (
                "salvagedTranscript".into(),
                Value::from(salvaged.transcript as u64),
            ),
        ],
    );
    if let Some(callback) = options.on_corruption_recovered.as_ref() {
        let event = CorruptionRecoveredEvent {
            outcome,
            quarantine_path: quarantine_path.clone(),
            salvaged,
        };
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            callback(&event)
        }));
    }

    let mut mutation = Map::new();
    mutation.insert("kind".into(), Value::String("agent-needs-reindex".into()));
    mutation.insert("agentId".into(), Value::String(agent_dir_name.to_string()));
    publish_transcript_mutation(&mutation);
    Ok(fresh_db)
}

#[cfg(all(test, unix))]
mod physical_release_tests {
    use super::*;

    fn fixture(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "fabushi-session-recovery-release-{label}-{}-{}",
            std::process::id(), uuid::Uuid::new_v4(),
        ));
        fs::create_dir(&root).expect("exclusively reserve recovery release fixture");
        eprintln!("[session-recovery-release] owned fixture {}", root.display());
        root
    }

    fn assert_contended(file: &File) {
        let error = fs2::FileExt::try_lock_exclusive(file).expect_err("physical custody must remain");
        assert_eq!(error.raw_os_error(), fs2::lock_contended_error().raw_os_error());
    }

    #[test]
    fn shared_guard_drop_unlocks_even_while_its_descriptor_duplicate_lives() {
        let root = fixture("shared");
        let path = root.join("store.db");
        let db = init_configured_db(&path, "shared-release", &DbRecoveryOptions::default())
            .expect("actual configured DB");
        let owner = acquire_automation_custody_recovery_guard(&path, &db)
            .expect("actual public availability guard");
        let duplicate = owner._guard.file.as_ref().unwrap().try_clone().expect("actual duplicate");
        let contender = open_recovery_guard(&path).expect("independent contender");
        assert_contended(&contender);
        drop(owner);
        fs2::FileExt::try_lock_exclusive(&contender)
            .expect("shared-owner Drop explicitly unlocks while duplicate lives");
        drop(duplicate);
        let later = open_recovery_guard(&path).expect("later independent contender");
        assert_contended(&later);
        fs2::FileExt::unlock(&contender).expect("release actual successor");
        drop(later);
        drop(contender);
        drop(db);
        fs::remove_dir_all(root).expect("remove completed owned fixture");
    }

    #[test]
    fn recovery_finish_unlocks_even_while_its_descriptor_duplicate_lives() {
        let root = fixture("exclusive");
        let path = root.join("store.db");
        fs::write(&path, b"owned corrupt fixture").expect("original bytes");
        let boundary = RecoveryStoreBoundary::acquire(&path).expect("actual recovery boundary");
        let duplicate = boundary.guard.file.as_ref().unwrap().try_clone().expect("actual duplicate");
        let contender = open_recovery_guard(&path).expect("independent contender");
        assert_contended(&contender);
        boundary.finish().expect("acknowledged unlock before publication");
        fs2::FileExt::try_lock_exclusive(&contender)
            .expect("successful finish explicitly unlocks while duplicate lives");
        drop(duplicate);
        let later = open_recovery_guard(&path).expect("later independent contender");
        assert_contended(&later);
        fs2::FileExt::unlock(&contender).expect("release actual successor");
        drop(later);
        drop(contender);
        assert_eq!(fs::read(&path).expect("original DB is untouched"), b"owned corrupt fixture");
        assert!(automation_custody_recovery_marker_path(&path).is_file());
        fs::remove_dir_all(root).expect("remove completed owned fixture");
    }
}
