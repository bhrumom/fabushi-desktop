use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::Connection;

pub const CONVERSATION_BLOB_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS blobs (
  id TEXT PRIMARY KEY,
  data BLOB NOT NULL
) STRICT;
"#;

pub const CONVERSATION_BLOB_MIGRATION_UNSTARTED: i64 = 0;
pub const CONVERSATION_BLOB_ADOPTION_COMPLETE: i64 = 1;
pub const CONVERSATION_BLOB_RECOVERY_REBUILT: i64 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationBlobMigrationState {
    Unstarted,
    AdoptionComplete,
    RecoveryRebuilt,
    Unknown(i64),
}

#[derive(Debug, thiserror::Error)]
pub enum ConversationBlobDbError {
    #[error("conversation blob sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("conversation blob filesystem error: {0}")]
    Io(#[from] std::io::Error),
    #[error("conversation blob database failed quick_check: {0}")]
    Corrupt(PathBuf),
}

pub fn read_conversation_blob_migration_state(
    db: &Connection,
) -> Result<ConversationBlobMigrationState, ConversationBlobDbError> {
    let version = db.query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))?;
    Ok(match version {
        CONVERSATION_BLOB_MIGRATION_UNSTARTED => ConversationBlobMigrationState::Unstarted,
        CONVERSATION_BLOB_ADOPTION_COMPLETE => ConversationBlobMigrationState::AdoptionComplete,
        CONVERSATION_BLOB_RECOVERY_REBUILT => ConversationBlobMigrationState::RecoveryRebuilt,
        other => ConversationBlobMigrationState::Unknown(other),
    })
}

pub fn set_conversation_blob_migration_state(
    db: &Connection,
    state: ConversationBlobMigrationState,
) -> Result<(), ConversationBlobDbError> {
    let value = match state {
        ConversationBlobMigrationState::Unstarted => CONVERSATION_BLOB_MIGRATION_UNSTARTED,
        ConversationBlobMigrationState::AdoptionComplete => CONVERSATION_BLOB_ADOPTION_COMPLETE,
        ConversationBlobMigrationState::RecoveryRebuilt => CONVERSATION_BLOB_RECOVERY_REBUILT,
        ConversationBlobMigrationState::Unknown(value) => value,
    };
    db.pragma_update(None, "user_version", value)?;
    Ok(())
}

pub fn open_configured_conversation_blob_db(
    db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<Connection, ConversationBlobDbError> {
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let db = Connection::open(db_path)?;
    db.busy_timeout(Duration::from_millis(busy_timeout_ms))?;
    let _ = db.pragma_update(None, "journal_mode", "WAL");
    let _ = db.pragma_update(None, "synchronous", "NORMAL");
    db.execute_batch(CONVERSATION_BLOB_SCHEMA)?;
    Ok(db)
}

pub fn run_quick_check(db: &Connection) -> Result<bool, ConversationBlobDbError> {
    let result = db.query_row("PRAGMA quick_check", [], |row| row.get::<_, String>(0))?;
    Ok(result == "ok")
}

pub fn remove_sqlite_sidecars(db_path: &Path) {
    for suffix in ["-wal", "-shm", "-journal"] {
        let sidecar = PathBuf::from(format!("{}{}", db_path.display(), suffix));
        let _ = fs::remove_file(sidecar);
    }
}

pub fn open_conversation_blob_db(
    db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<Connection, ConversationBlobDbError> {
    match open_configured_conversation_blob_db(db_path, busy_timeout_ms) {
        Ok(db) => {
            if run_quick_check(&db)? {
                Ok(db)
            } else {
                Err(ConversationBlobDbError::Corrupt(db_path.to_path_buf()))
            }
        }
        Err(ConversationBlobDbError::Sqlite(error))
            if sqlite_error_may_be_sidecar_related(&error) =>
        {
            remove_sqlite_sidecars(db_path);
            let db = open_configured_conversation_blob_db(db_path, busy_timeout_ms)?;
            if run_quick_check(&db)? {
                Ok(db)
            } else {
                Err(ConversationBlobDbError::Corrupt(db_path.to_path_buf()))
            }
        }
        Err(error) => Err(error),
    }
}

fn sqlite_error_may_be_sidecar_related(error: &rusqlite::Error) -> bool {
    let text = error.to_string().to_ascii_lowercase();
    text.contains("disk i/o")
        || text.contains("unable to open")
        || text.contains("database disk image is malformed")
}
