//! Transactional install receipts in the existing injected Host SQLite owner.
//! No new database, account identity, user-data store or execution API.
use super::VerifiedPackage;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostInstallScope {
    /// Supplied by current authenticated Host account lifecycle, never renderer.
    pub account_scope: String,
    pub generation: u64,
}
#[derive(Debug, Error)]
pub enum InstallError {
    #[error("install request belongs to a departed account/session")]
    StaleScope,
    #[error("immutable package version or operation identity conflict")]
    Conflict,
    #[error("package update is not a monotonic stable release")]
    VersionPolicy,
    #[error("installed code cache quota exceeded")]
    Quota,
    #[error("Host package storage operation failed")]
    Storage(#[from] rusqlite::Error),
}
#[derive(Debug, PartialEq, Eq)]
pub struct InstalledRecord {
    pub app_id: String,
    pub version: String,
    pub digest: String,
    pub previous_digest: Option<String>,
}
pub fn initialize(db: &Connection) -> Result<(), InstallError> {
    db.execute_batch(
        "CREATE TABLE IF NOT EXISTS host_miniapp_packages (
        app_id TEXT NOT NULL, version TEXT NOT NULL, digest TEXT PRIMARY KEY,
        publisher_id TEXT NOT NULL, trust_epoch INTEGER NOT NULL, archive BLOB NOT NULL,
        UNIQUE(app_id,version));
        CREATE TABLE IF NOT EXISTS host_miniapp_installs (
        account_scope TEXT NOT NULL, app_id TEXT NOT NULL, digest TEXT NOT NULL,
        previous_digest TEXT, PRIMARY KEY(account_scope,app_id));
        CREATE TABLE IF NOT EXISTS host_miniapp_operations (
        account_scope TEXT NOT NULL, operation_id TEXT NOT NULL, app_id TEXT NOT NULL,
        digest TEXT NOT NULL, PRIMARY KEY(account_scope,operation_id));",
    )?;
    Ok(())
}
fn stable_version(version: &str) -> Option<(u64, u64, u64)> {
    let values: Vec<_> = version.split('.').collect();
    if values.len() != 3 {
        return None;
    }
    Some((
        values[0].parse().ok()?,
        values[1].parse().ok()?,
        values[2].parse().ok()?,
    ))
}
pub fn installed(
    db: &Connection,
    current: &HostInstallScope,
    app_id: &str,
) -> Result<Option<InstalledRecord>, InstallError> {
    Ok(db
        .query_row(
            "SELECT p.app_id,p.version,p.digest,i.previous_digest
        FROM host_miniapp_installs i JOIN host_miniapp_packages p ON p.digest=i.digest
        WHERE i.account_scope=?1 AND i.app_id=?2",
            params![current.account_scope, app_id],
            |r| {
                Ok(InstalledRecord {
                    app_id: r.get(0)?,
                    version: r.get(1)?,
                    digest: r.get(2)?,
                    previous_digest: r.get(3)?,
                })
            },
        )
        .optional()?)
}
/// Serializes activation with code bytes and operation receipt. Failure before
/// commit leaves both active pointer and last-good archive unchanged.
pub fn activate(
    db: &mut Connection,
    current: &HostInstallScope,
    intent: &HostInstallScope,
    operation_id: &str,
    package: VerifiedPackage,
) -> Result<InstalledRecord, InstallError> {
    if current != intent
        || current.generation == 0
        || current.account_scope.is_empty()
        || operation_id.is_empty()
        || operation_id.len() > 128
    {
        return Err(InstallError::StaleScope);
    }
    let new_version = stable_version(package.version()).ok_or(InstallError::VersionPolicy)?;
    let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let receipt: Option<(String, String)> = tx
        .query_row(
            "SELECT app_id,digest FROM host_miniapp_operations
        WHERE account_scope=?1 AND operation_id=?2",
            params![current.account_scope, operation_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    if let Some((app, digest)) = receipt {
        if app != package.app_id() || digest != package.digest() {
            return Err(InstallError::Conflict);
        }
        // An old receipt must never reactivate an older package after an update.
        return installed(&tx, current, &app)?.ok_or(InstallError::Conflict);
    }
    let existing: Option<String> = tx
        .query_row(
            "SELECT digest FROM host_miniapp_packages WHERE app_id=?1 AND version=?2",
            params![package.app_id(), package.version()],
            |r| r.get(0),
        )
        .optional()?;
    if existing
        .as_ref()
        .is_some_and(|value| value != package.digest())
    {
        return Err(InstallError::Conflict);
    }
    let old = installed(&tx, current, package.app_id())?;
    if let Some(old) = &old {
        if old.digest != package.digest()
            && stable_version(&old.version).is_none_or(|v| new_version <= v)
        {
            return Err(InstallError::VersionPolicy);
        }
    }
    let used: i64 = tx.query_row(
        "SELECT COALESCE(SUM(length(archive)),0) FROM host_miniapp_packages",
        [],
        |r| r.get(0),
    )?;
    if used < 0
        || (existing.is_none()
            && (used as u64).saturating_add(package.archive.len() as u64) > 500 * 1024 * 1024)
    {
        return Err(InstallError::Quota);
    }
    let previous = old.as_ref().and_then(|old| {
        if old.digest == package.digest() {
            old.previous_digest.clone()
        } else {
            Some(old.digest.clone())
        }
    });
    let epoch = i64::try_from(package.trust_epoch).map_err(|_| InstallError::Conflict)?;
    tx.execute("INSERT OR IGNORE INTO host_miniapp_packages(app_id,version,digest,publisher_id,trust_epoch,archive)
        VALUES(?1,?2,?3,?4,?5,?6)",params![package.app_id(),package.version(),package.digest(),package.publisher_id(),epoch,package.archive])?;
    tx.execute("INSERT INTO host_miniapp_installs(account_scope,app_id,digest,previous_digest) VALUES(?1,?2,?3,?4)
        ON CONFLICT(account_scope,app_id) DO UPDATE SET digest=excluded.digest,previous_digest=excluded.previous_digest",
        params![current.account_scope,package.app_id(),package.digest(),previous])?;
    tx.execute("INSERT INTO host_miniapp_operations(account_scope,operation_id,app_id,digest) VALUES(?1,?2,?3,?4)",
        params![current.account_scope,operation_id,package.app_id(),package.digest()])?;
    let record = installed(&tx, current, package.app_id())?.ok_or(InstallError::Conflict)?;
    tx.commit()?;
    Ok(record)
}
/// Bytes remain inert; the native launch path must reverify current catalog
/// policy with verify_catalog_bundle before serving any resource.
pub fn load_active_archive(
    db: &Connection,
    current: &HostInstallScope,
    app_id: &str,
) -> Result<Option<Vec<u8>>, InstallError> {
    Ok(db.query_row("SELECT p.archive FROM host_miniapp_installs i JOIN host_miniapp_packages p ON p.digest=i.digest
        WHERE i.account_scope=?1 AND i.app_id=?2",params![current.account_scope,app_id],|r|r.get(0)).optional()?)
}
/// Code-only removal. Per-app browser user data remains with the native data
/// owner and requires a separately explicit retain/delete UI intent.
pub fn remove(
    db: &Connection,
    current: &HostInstallScope,
    intent: &HostInstallScope,
    app_id: &str,
) -> Result<bool, InstallError> {
    if current != intent {
        return Err(InstallError::StaleScope);
    }
    Ok(db.execute(
        "DELETE FROM host_miniapp_installs WHERE account_scope=?1 AND app_id=?2",
        params![current.account_scope, app_id],
    )? > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extensions::mcp::installable_package::{
        tests::{HTML, fixture},
        verify_catalog_bundle,
    };
    fn package(version: &str, title: &str) -> VerifiedPackage {
        let (bytes, mut policy) = fixture(
            |m| {
                m["version"] = version.into();
                m["displayName"] = title.into();
            },
            HTML,
        );
        policy.version = version.into();
        verify_catalog_bundle(&bytes, &policy).unwrap()
    }
    fn scope(account: &str) -> HostInstallScope {
        HostInstallScope {
            account_scope: account.into(),
            generation: 1,
        }
    }
    #[test]
    fn activation_update_retry_and_account_isolation() {
        let mut db = Connection::open_in_memory().unwrap();
        initialize(&db).unwrap();
        let s = scope("a");
        let first = activate(&mut db, &s, &s, "one", package("1.0.0", "one")).unwrap();
        assert!(
            installed(&db, &scope("b"), &first.app_id)
                .unwrap()
                .is_none()
        );
        let next = activate(&mut db, &s, &s, "two", package("1.1.0", "two")).unwrap();
        assert_eq!(next.previous_digest, Some(first.digest));
        assert_eq!(
            activate(&mut db, &s, &s, "one", package("1.0.0", "one")).unwrap(),
            next
        );
        assert!(matches!(
            activate(
                &mut db,
                &s,
                &s,
                "changed",
                package("1.1.0", "different bytes")
            ),
            Err(InstallError::Conflict)
        ));
        assert!(
            load_active_archive(&db, &scope("b"), &next.app_id)
                .unwrap()
                .is_none()
        );
    }
    #[test]
    fn rollback_on_storage_failure_preserves_last_good() {
        let mut db = Connection::open_in_memory().unwrap();
        initialize(&db).unwrap();
        let s = scope("a");
        let first = activate(&mut db, &s, &s, "one", package("1.0.0", "one")).unwrap();
        db.execute_batch("CREATE TRIGGER stop_receipt BEFORE INSERT ON host_miniapp_operations BEGIN SELECT RAISE(ABORT,'fault'); END;").unwrap();
        assert!(activate(&mut db, &s, &s, "two", package("1.1.0", "two")).is_err());
        assert_eq!(installed(&db, &s, &first.app_id).unwrap(), Some(first));
        assert_eq!(
            db.query_row("SELECT COUNT(*) FROM host_miniapp_packages", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            1
        );
    }
    #[test]
    fn stale_generation_and_code_removal() {
        let mut db = Connection::open_in_memory().unwrap();
        initialize(&db).unwrap();
        let s = scope("a");
        let mut stale = s.clone();
        stale.generation = 0;
        assert!(matches!(
            activate(&mut db, &s, &stale, "one", package("1.0.0", "one")),
            Err(InstallError::StaleScope)
        ));
        let first = activate(&mut db, &s, &s, "one", package("1.0.0", "one")).unwrap();
        assert!(remove(&db, &s, &s, &first.app_id).unwrap());
        assert!(!remove(&db, &s, &s, &first.app_id).unwrap());
        assert!(installed(&db, &s, &first.app_id).unwrap().is_none());
    }
}
