use std::path::Path;
use std::time::Duration;

use rusqlite::{Connection, OpenFlags};

use crate::storage::store_db::DB_BUSY_TIMEOUT_MS;

pub fn sqlite_vacuum_into(
    src_path: impl AsRef<Path>,
    dest_path: impl AsRef<Path>,
    busy_timeout_ms: u64,
) -> Result<(), rusqlite::Error> {
    let db = Connection::open_with_flags(
        src_path.as_ref(),
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )?;
    db.busy_timeout(Duration::from_millis(busy_timeout_ms))?;
    let escaped = dest_path
        .as_ref()
        .to_string_lossy()
        .replace('\'', "''");
    db.execute_batch(&format!("VACUUM INTO '{escaped}'"))?;
    Ok(())
}

pub fn sqlite_vacuum_into_default(
    src_path: impl AsRef<Path>,
    dest_path: impl AsRef<Path>,
) -> Result<(), rusqlite::Error> {
    sqlite_vacuum_into(src_path, dest_path, DB_BUSY_TIMEOUT_MS)
}
