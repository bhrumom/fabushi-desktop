//! HSA-04/HSA-05 and DPS-v1: exercise the shipping entrypoint, real profile
//! custody and durable-store recovery. No test backend or entrypoint hook is used.
//!
//! The non-loopback daemon setting is an existing production rejection after
//! both app constructors and durable pairing admission, before daemon/network
//! startup. Its exact diagnostic is required: an earlier failure or a timeout
//! cannot satisfy the control.

use mahayana_host_runtime::device_pairing_owner::DevicePairingOwner;
use mahayana_host_runtime::host_lock::{
    HostLockHandle, HostLockOutcome, acquire_host_lock, default_is_process_alive,
    is_sand_host_process, read_lock_pid, read_process_command,
};
use rusqlite::{Connection, OpenFlags, params};
use serde_json::json;
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const CHILD_TIMEOUT: Duration = Duration::from_secs(60);
const RETIRE_TIMEOUT: Duration = Duration::from_secs(5);
const POLL_INTERVAL: Duration = Duration::from_millis(10);
const MAX_DIAGNOSTIC_BYTES: usize = 256 * 1024;
const MAX_TREE_BYTES: usize = 32 * 1024 * 1024;
const MAX_TREE_ENTRIES: usize = 4096;
const STOP_HOST: &str = "192.0.2.1";
const CONVERSATION_ID: &str = "native:agent:startup-admission";
const TURN_ID: &str = "turn:startup-admission";
const RUN_ID: &str = "run:startup-admission:1";
const MESSAGE_ID: &str = "message:startup-admission";
const CREATED_AT_MS: i64 = 1_700_000_000_000;
const PAIRING_DEVICE_ID: &str = "device:startup-durability";
const PAIRING_PUBLIC_KEY: &str = "startup-durability-public-key";
const PAIRING_FAILURE_PREFIX: &str = "failed to initialize durable device pairing store:";

struct ChildResult {
    pid: u32,
    status: ExitStatus,
    stdout: String,
    stderr: String,
}

impl ChildResult {
    fn details(&self) -> String {
        format!(
            "pid={} status={} stdout={:?} stderr={:?}",
            self.pid,
            self.status,
            self.stdout.chars().take(4096).collect::<String>(),
            self.stderr.chars().take(4096).collect::<String>(),
        )
    }

    fn assert_exit_with_line(&self, code: i32, line: &str) {
        assert_eq!(self.status.code(), Some(code), "{}", self.details());
        assert!(
            self.stderr.lines().any(|actual| actual == line),
            "missing exact diagnostic {line:?}; {}",
            self.details(),
        );
    }

    fn assert_admitted_constructor_stop(&self) {
        // This path currently returns from main, so its preserved status is 0.
        self.assert_exit_with_line(
            0,
            &format!(
                "failed to start managed Grok box exec-daemon: managed box exec-daemon requires loopback SAND_BOX_HOST, got {STOP_HOST}"
            ),
        );
        assert!(!self.stderr.contains("failed to acquire Mahayana Host lock"));
    }

    fn assert_pairing_initialization_failure(&self) {
        assert_eq!(self.status.code(), Some(1), "{}", self.details());
        assert!(
            self.stderr.lines().any(|line| line.strip_prefix(PAIRING_FAILURE_PREFIX)
                .is_some_and(|error| !error.trim().is_empty())),
            "missing pairing-specific initialization error; {}", self.details(),
        );
        assert!(!self.stderr.contains("failed to acquire Mahayana Host lock"), "{}", self.details());
        assert!(!self.stderr.contains("failed to start managed Grok box exec-daemon"), "{}", self.details());
    }
}

#[derive(Debug, PartialEq, Eq)]
struct GuardSnapshot {
    bytes: Vec<u8>,
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
}

impl GuardSnapshot {
    fn read(path: &Path) -> Self {
        let metadata = fs::symlink_metadata(path).expect("observe actual physical guard file");
        assert!(metadata.is_file(), "physical guard must be a regular file");
        #[cfg(unix)]
        use std::os::unix::fs::MetadataExt;
        Self {
            bytes: read_bounded(path, 4096).expect("bounded physical guard bytes"),
            #[cfg(unix)]
            device: metadata.dev(),
            #[cfg(unix)]
            inode: metadata.ino(),
        }
    }
}

struct HeldProfile {
    lock: HostLockHandle,
    pid_bytes: Vec<u8>,
    guard: GuardSnapshot,
}

struct Fixture {
    root: PathBuf,
    child: Option<Child>,
    held_profile: Option<HeldProfile>,
    retirement_attempted: bool,
    cleaned: bool,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "fabushi-startup-admission-{label}-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4(),
        ));
        fs::create_dir(&root).expect("create exclusively owned fixture root");
        for directory in ["home", "tmp", "config", "data", "cache", "logs"] {
            fs::create_dir(root.join(directory)).expect("create isolated child environment");
        }
        Self {
            root,
            child: None,
            held_profile: None,
            retirement_attempted: false,
            cleaned: false,
        }
    }

    fn app_root(&self) -> PathBuf {
        self.root.join("app")
    }

    fn lock_path(&self) -> PathBuf {
        self.root.join("profile/host.lock")
    }

    fn guard_path(&self) -> PathBuf {
        self.root.join("profile/host.lock.guard")
    }

    fn database_path(&self) -> PathBuf {
        self.app_root().join("feature-host/runtime/runtime.sqlite3")
    }

    fn settings_path(&self) -> PathBuf {
        self.app_root().join("feature-host/runtime/settings.json")
    }

    fn pairing_database_path(&self) -> PathBuf {
        self.app_root().join("device-pairing.db")
    }

    fn hold_profile(&mut self) {
        assert!(self.child.is_none());
        assert!(self.held_profile.is_none());
        let pid = std::process::id();
        let command = read_process_command(pid).expect("parent process identity must be observable");
        assert!(default_is_process_alive(pid), "parent must be observably alive");
        assert!(
            !command.contains("host-main") && !command.contains("mahayana-app-host"),
            "unsafe test parent matches the production takeover recognizer: {command:?}",
        );
        assert!(
            !is_sand_host_process(pid),
            "refuse to expose a parent that the actual lock owner could terminate",
        );
        let acquisition = acquire_host_lock(&self.lock_path(), pid).expect("hold real profile lock");
        assert_eq!(acquisition.outcome, HostLockOutcome::Created);
        assert_eq!(acquisition.previous_pid, None);
        self.held_profile = Some(HeldProfile {
            lock: acquisition.lock,
            pid_bytes: read_bounded(&self.lock_path(), 4096).expect("read parent PID projection"),
            guard: GuardSnapshot::read(&self.guard_path()),
        });
        self.assert_held_profile_unchanged();
    }

    fn assert_held_profile_unchanged(&self) {
        assert!(self.child.is_none(), "observe custody only after exact child exit");
        let held = self.held_profile.as_ref().expect("parent owns the profile");
        assert_eq!(held.lock.pid(), std::process::id());
        assert_eq!(read_lock_pid(&self.lock_path()), Some(std::process::id()));
        assert_eq!(
            read_bounded(&self.lock_path(), 4096).expect("read held PID projection"),
            held.pid_bytes,
        );
        assert_eq!(GuardSnapshot::read(&self.guard_path()), held.guard);
        let error = acquire_host_lock(&self.lock_path(), std::process::id())
            .expect_err("a separate real acquisition must still encounter physical custody");
        assert_eq!(error.kind(), io::ErrorKind::WouldBlock);
    }

    fn release_profile(&mut self) {
        assert!(self.child.is_none());
        let held = self.held_profile.take().expect("release the exact parent guard");
        held.lock.release().expect("retire parent PID and physical custody");
        drop(held);
        assert_absent(&self.lock_path());
    }

    fn assert_retired_and_successor_can_acquire(&self) {
        assert!(self.child.is_none());
        assert!(self.held_profile.is_none());
        // The persistent guard proves admission happened; absence of the PID
        // projection distinguishes explicit retirement from mere process exit.
        let guard = GuardSnapshot::read(&self.guard_path());
        assert_absent(&self.lock_path());
        let successor = acquire_host_lock(&self.lock_path(), std::process::id())
            .expect("successor must acquire the existing physical guard");
        assert_eq!(successor.outcome, HostLockOutcome::Created);
        assert_eq!(successor.previous_pid, None);
        assert_eq!(read_lock_pid(&self.lock_path()), Some(std::process::id()));
        assert_eq!(GuardSnapshot::read(&self.guard_path()), guard);
        successor.lock.release().expect("retire successor");
        drop(successor);
        assert_absent(&self.lock_path());
        assert_eq!(GuardSnapshot::read(&self.guard_path()), guard);
    }

    fn run(&mut self, label: &str, arguments: &[&str], environment: &[(&str, &str)]) -> ChildResult {
        assert!(self.child.is_none(), "one owned child at a time");
        let stdout_path = self.root.join("logs").join(format!("{label}.stdout"));
        let stderr_path = self.root.join("logs").join(format!("{label}.stderr"));
        let stdout = OpenOptions::new().write(true).create_new(true)
            .open(&stdout_path).expect("create owned stdout file");
        let stderr = OpenOptions::new().write(true).create_new(true)
            .open(&stderr_path).expect("create owned stderr file");
        let mut command = Command::new(env!("CARGO_BIN_EXE_mahayana-app-host"));
        command.env_clear();
        // Only executable lookup and Windows loader essentials survive. No CI
        // account token, provider credential, proxy, plugin or user config does.
        for key in ["PATH", "SystemRoot", "WINDIR"] {
            if let Some(value) = std::env::var_os(key) {
                command.env(key, value);
            }
        }
        command
            .current_dir(&self.root)
            .env("HOME", self.root.join("home"))
            .env("USERPROFILE", self.root.join("home"))
            .env("APPDATA", self.root.join("config"))
            .env("LOCALAPPDATA", self.root.join("data"))
            .env("XDG_CONFIG_HOME", self.root.join("config"))
            .env("XDG_DATA_HOME", self.root.join("data"))
            .env("XDG_CACHE_HOME", self.root.join("cache"))
            .env("TMPDIR", self.root.join("tmp"))
            .env("TMP", self.root.join("tmp"))
            .env("TEMP", self.root.join("tmp"))
            .env("FABUSHI_APP_DATA", self.app_root())
            .env("SAND_DATA_ROOT", self.root.join("profile"))
            .env("FABUSHI_FEATURE_HOST_MODE", "production")
            .env("SAND_BOX_HOST", STOP_HOST)
            .args(arguments)
            .envs(environment.iter().copied())
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr));
        self.retirement_attempted = false;
        self.child = Some(command.spawn().expect("launch actual shipping Host binary"));
        let pid = self.child.as_ref().unwrap().id();
        let deadline = Instant::now() + CHILD_TIMEOUT;
        let status = loop {
            // Files avoid pipe backpressure and unjoinable reader threads. Both
            // files are size-polled, and every read is independently capped.
            for path in [&stdout_path, &stderr_path] {
                match fs::metadata(path) {
                    Ok(metadata) if metadata.len() <= MAX_DIAGNOSTIC_BYTES as u64 => {}
                    Ok(_) => self.fail_child("diagnostic file exceeded its limit", &stdout_path, &stderr_path),
                    Err(error) => self.fail_child(&format!("inspect diagnostic file: {error}"), &stdout_path, &stderr_path),
                }
            }
            match self.child.as_mut().unwrap().try_wait() {
                Ok(Some(status)) => {
                    // try_wait has reaped this exact Child. No PID-based probe
                    // or unrelated process-name cleanup authorizes root removal.
                    drop(self.child.take());
                    break status;
                }
                Ok(None) => {}
                Err(error) => self.fail_child(&format!("observe child exit: {error}"), &stdout_path, &stderr_path),
            }
            if Instant::now() >= deadline {
                self.fail_child("shipping Host exceeded its exit deadline", &stdout_path, &stderr_path);
            }
            // This polls terminal process ownership, never readiness or recovery.
            thread::sleep(POLL_INTERVAL);
        };
        ChildResult {
            pid,
            status,
            stdout: String::from_utf8(read_bounded(&stdout_path, MAX_DIAGNOSTIC_BYTES)
                .expect("bounded completed stdout")).expect("UTF-8 stdout"),
            stderr: String::from_utf8(read_bounded(&stderr_path, MAX_DIAGNOSTIC_BYTES)
                .expect("bounded completed stderr")).expect("UTF-8 stderr"),
        }
    }

    fn retire_child(&mut self) -> Result<(), String> {
        self.retirement_attempted = true;
        let Some(child) = self.child.as_mut() else { return Ok(()); };
        if matches!(child.try_wait(), Ok(Some(_))) {
            drop(self.child.take());
            return Ok(());
        }
        let pid = child.id();
        let kill_error = child.kill().err();
        let deadline = Instant::now() + RETIRE_TIMEOUT;
        let mut wait_error = None;
        loop {
            match child.try_wait() {
                Ok(Some(_)) => {
                    drop(self.child.take());
                    return match kill_error {
                        None => Ok(()),
                        Some(error) => Err(format!("child {pid} reaped after kill error: {error}")),
                    };
                }
                Ok(None) => {}
                Err(error) => wait_error = Some(error),
            }
            if Instant::now() >= deadline {
                return Err(format!(
                    "child {pid} exit remains uncertain; kill={kill_error:?}, wait={wait_error:?}"
                ));
            }
            thread::sleep(POLL_INTERVAL);
        }
    }

    fn fail_child(&mut self, reason: &str, stdout: &Path, stderr: &Path) -> ! {
        let retirement = self.retire_child();
        panic!(
            "{reason}; retirement={retirement:?}; retained fixture {}; stdout={:?}; stderr={:?}",
            self.root.display(),
            diagnostic_prefix(stdout),
            diagnostic_prefix(stderr),
        );
    }

    fn finish(mut self) {
        assert!(self.child.is_none(), "cannot remove a root with an unretired child");
        assert!(!self.retirement_attempted, "forced/uncertain retirement is not passing evidence");
        if self.held_profile.is_some() {
            self.release_profile();
        }
        fs::remove_dir_all(&self.root).expect("remove only the completed owned fixture");
        self.cleaned = true;
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if self.cleaned {
            return;
        }
        if self.child.is_some() && !self.retirement_attempted {
            let retirement = self.retire_child();
            eprintln!("startup-admission panic cleanup: {retirement:?}");
        }
        // Incomplete/failed fixtures, especially uncertain child exits, retain
        // every root and diagnostic. The parent's guard remains held throughout
        // the bounded retirement attempt, including an uncertain result.
        eprintln!("retained startup-admission fixture {}", self.root.display());
    }
}

fn read_bounded(path: &Path, maximum: usize) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    File::open(path)?.take(maximum as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > maximum {
        return Err(io::Error::other(format!("{} exceeded {maximum} bytes", path.display())));
    }
    Ok(bytes)
}

fn diagnostic_prefix(path: &Path) -> String {
    let mut bytes = Vec::new();
    match File::open(path).and_then(|file| file.take(4096).read_to_end(&mut bytes)) {
        Ok(_) => String::from_utf8_lossy(&bytes).into_owned(),
        Err(error) => format!("unavailable: {error}"),
    }
}

fn assert_absent(path: &Path) {
    assert!(
        matches!(fs::symlink_metadata(path), Err(error) if error.kind() == io::ErrorKind::NotFound),
        "expected absent path {}",
        path.display(),
    );
}

#[derive(PartialEq, Eq)]
enum TreeEntry {
    Directory,
    File(Vec<u8>),
}

fn app_tree(root: &Path) -> BTreeMap<PathBuf, TreeEntry> {
    let mut entries = BTreeMap::new();
    let mut pending = vec![root.to_path_buf()];
    let mut byte_count = 0;
    while let Some(path) = pending.pop() {
        assert!(entries.len() < MAX_TREE_ENTRIES, "app tree exceeded entry bound");
        let relative = path.strip_prefix(root).expect("stay under owned app root").to_path_buf();
        let metadata = fs::symlink_metadata(&path).expect("snapshot complete app tree");
        let entry = if metadata.is_dir() {
            for child in fs::read_dir(&path).expect("enumerate complete app directory") {
                pending.push(child.expect("observe each app entry").path());
                assert!(entries.len() + pending.len() <= MAX_TREE_ENTRIES, "app tree exceeded entry bound");
            }
            TreeEntry::Directory
        } else {
            // Never follow a symlink out of the isolated fixture or silently
            // omit an unsupported file from the byte-for-byte oracle.
            assert!(metadata.is_file(), "unexpected app entry {}", path.display());
            let bytes = read_bounded(&path, MAX_TREE_BYTES - byte_count)
                .expect("bounded complete app-file snapshot");
            byte_count += bytes.len();
            TreeEntry::File(bytes)
        };
        assert!(entries.insert(relative, entry).is_none());
    }
    entries
}

fn assert_same_tree(before: &BTreeMap<PathBuf, TreeEntry>, after: &BTreeMap<PathBuf, TreeEntry>) {
    let changed = before.keys().chain(after.keys())
        .filter(|path| before.get(*path) != after.get(*path))
        .take(32).collect::<Vec<_>>();
    assert!(before == after, "refused startup changed app-tree paths: {changed:?}");
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RecoveryRow {
    turn_id: String,
    conversation_id: String,
    message_id: Option<String>,
    created_at_ms: i64,
    turn_state: String,
    active_run_id: Option<String>,
    run_id: String,
    run_turn_id: String,
    generation: i64,
    provider: String,
    started_at_ms: i64,
    finished_at_ms: Option<i64>,
    run_state: String,
    turn_count: i64,
    run_count: i64,
}

fn seed_waiting_user(database: &Path) {
    // READ_WRITE without CREATE requires the actual admitted binary to have
    // created the schema. This fixture never supplies a competing schema.
    let mut connection = Connection::open_with_flags(database, OpenFlags::SQLITE_OPEN_READ_WRITE)
        .expect("open shipping-created RuntimeStore");
    connection.busy_timeout(Duration::from_secs(2)).expect("bound seed lock wait");
    connection.execute_batch("PRAGMA foreign_keys=ON;").expect("enforce real foreign keys");
    let transaction = connection.transaction().expect("seed one valid durable turn atomically");
    transaction.execute(
        "INSERT INTO conversations(conversation_id, updated_at_ms) VALUES (?1, ?2)",
        params![CONVERSATION_ID, CREATED_AT_MS],
    ).expect("seed conversation");
    transaction.execute(
        "INSERT INTO turns(turn_id, conversation_id, user_message_id, created_at_ms, state, active_run_id)
         VALUES (?1, ?2, ?3, ?4, 'waiting-user', ?5)",
        params![TURN_ID, CONVERSATION_ID, MESSAGE_ID, CREATED_AT_MS, RUN_ID],
    ).expect("seed waiting-user logical turn");
    transaction.execute(
        "INSERT INTO runs(run_id, turn_id, generation, provider, started_at_ms, finished_at_ms, state)
         VALUES (?1, ?2, 1, 'fabushi', ?3, NULL, 'waiting-user')",
        params![RUN_ID, TURN_ID, CREATED_AT_MS],
    ).expect("seed original waiting-user execution run");
    transaction.execute(
        "INSERT INTO turn_requests(turn_id, message_id, text, inference_provider, hidden, updated_at_ms)
         VALUES (?1, ?2, 'Wait for user input before continuing.', 'fabushi', 0, ?3)",
        params![TURN_ID, MESSAGE_ID, CREATED_AT_MS],
    ).expect("seed durable user input");
    transaction.commit().expect("commit waiting-user state");
    connection.close().expect("close seed connection before any child starts");
}

fn recovery_row(database: &Path) -> RecoveryRow {
    let connection = Connection::open_with_flags(database, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .expect("observe existing RuntimeStore without creating a database");
    connection.busy_timeout(Duration::from_secs(2)).expect("bound observer lock wait");
    let row = connection.query_row(
        "SELECT t.turn_id, t.conversation_id, t.user_message_id, t.created_at_ms, t.state,
                t.active_run_id, r.run_id, r.turn_id, r.generation, r.provider,
                r.started_at_ms, r.finished_at_ms, r.state,
                (SELECT COUNT(*) FROM turns), (SELECT COUNT(*) FROM runs)
         FROM turns t JOIN runs r ON r.turn_id = t.turn_id
         WHERE t.turn_id = ?1 AND r.run_id = ?2",
        params![TURN_ID, RUN_ID],
        |row| Ok(RecoveryRow {
            turn_id: row.get(0)?, conversation_id: row.get(1)?, message_id: row.get(2)?,
            created_at_ms: row.get(3)?, turn_state: row.get(4)?, active_run_id: row.get(5)?,
            run_id: row.get(6)?, run_turn_id: row.get(7)?, generation: row.get(8)?,
            provider: row.get(9)?, started_at_ms: row.get(10)?, finished_at_ms: row.get(11)?,
            run_state: row.get(12)?, turn_count: row.get(13)?, run_count: row.get(14)?,
        }),
    ).expect("observe the original turn and run");
    connection.close().expect("close observer before the next child starts");
    row
}

fn assert_shipping_pairing_schema(database: &Path) {
    // READ_ONLY cannot create the file or repair a missing production schema.
    let connection = Connection::open_with_flags(database, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .expect("observe pairing database created by the shipping binary");
    connection.busy_timeout(Duration::from_secs(2)).expect("bound pairing observer lock wait");
    let table_count: i64 = connection.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN (
            'device_pairing_pending', 'device_pairing_paired', 'device_bootstrap_tokens',
            'device_pairing_join_codes', 'device_pair_setup_completions', 'config_machine_state'
        )",
        [], |row| row.get(0),
    ).expect("read actual pairing schema");
    assert_eq!(table_count, 6, "shipping initializer must create the durable pairing schema");
    connection.close().expect("close pairing observer before child startup");
}

fn assert_durable_pairing_identity(database: &Path) {
    let connection = Connection::open_with_flags(database, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .expect("observe persisted pairing identity without creating it");
    connection.busy_timeout(Duration::from_secs(2)).expect("bound identity observer lock wait");
    let identity: (String, String, String) = connection.query_row(
        "SELECT public_key, display_name, platform FROM device_pairing_paired WHERE device_id = ?1",
        params![PAIRING_DEVICE_ID], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ).expect("the original paired identity must survive startup");
    assert_eq!(identity, (
        PAIRING_PUBLIC_KEY.to_owned(), "Durable device".to_owned(), "test-platform".to_owned()
    ));
    connection.close().expect("close identity observer before child startup");
}

fn rotate_pairing_token(owner: &DevicePairingOwner, role: &str, revision: u64) -> String {
    let rotated = owner.handle_pair_rotate_token(&json!({
        "deviceId": PAIRING_DEVICE_ID, "role": role
    })).expect("rotate the approved role through the durable owner");
    assert_eq!(rotated["role"], role);
    assert_eq!(rotated["revision"].as_u64(), Some(revision));
    assert_eq!(rotated["tokenDelivery"], "in-band");
    rotated["token"].as_str().expect("rotation returns the new role token").to_owned()
}

fn assert_pairing_token(owner: &DevicePairingOwner, token: &str, role: &str, scopes: &[&str]) {
    let authorization = owner.authenticate_device_token(token)
        .expect("read durable token authorization")
        .expect("the issued role token must remain authorized");
    assert_eq!(authorization.device_id, PAIRING_DEVICE_ID);
    assert_eq!(authorization.role, role);
    assert_eq!(authorization.scopes, scopes.iter().map(|scope| (*scope).to_owned()).collect::<Vec<_>>());
}

#[test]
fn refused_normal_start_keeps_missing_app_root_absent() {
    let mut fixture = Fixture::new("missing-app");
    assert_absent(&fixture.app_root());
    fixture.hold_profile();
    let result = fixture.run("refused", &[], &[]);
    result.assert_exit_with_line(0, &format!(
        "failed to acquire Mahayana Host lock at {}: Host physical custody is still held for {}",
        fixture.lock_path().display(), fixture.lock_path().display(),
    ));
    assert_absent(&fixture.app_root());
    fixture.assert_held_profile_unchanged();
    fixture.finish();
}

#[test]
fn refused_normal_start_preserves_runtime_and_settings_then_admitted_start_recovers() {
    let mut fixture = Fixture::new("durable-state");
    let bootstrap = fixture.run("create-real-schema", &[], &[]);
    bootstrap.assert_admitted_constructor_stop();
    fixture.assert_retired_and_successor_can_acquire();
    seed_waiting_user(&fixture.database_path());
    let waiting = recovery_row(&fixture.database_path());
    assert_eq!(waiting.turn_state, "waiting-user");
    assert_eq!(waiting.run_state, "waiting-user");
    assert_eq!(waiting.active_run_id.as_deref(), Some(RUN_ID));
    assert_eq!(waiting.finished_at_ms, None);
    assert_eq!((waiting.turn_count, waiting.run_count, waiting.generation), (1, 1, 1));

    let canonical_settings = read_bounded(&fixture.settings_path(), MAX_TREE_BYTES)
        .expect("settings written by the actual production constructor");
    let settings_value: serde_json::Value = serde_json::from_slice(&canonical_settings)
        .expect("production settings must be valid JSON");
    let mut seeded_settings = canonical_settings.clone();
    seeded_settings.extend_from_slice(b"\n \t\r\n");
    assert_eq!(serde_json::from_slice::<serde_json::Value>(&seeded_settings).unwrap(), settings_value);
    fs::write(fixture.settings_path(), &seeded_settings).expect("retain valid insignificant whitespace");

    fixture.hold_profile();
    let before = app_tree(&fixture.app_root());
    let refused = fixture.run("refused-with-state", &[], &[]);
    refused.assert_exit_with_line(0, &format!(
        "failed to acquire Mahayana Host lock at {}: Host physical custody is still held for {}",
        fixture.lock_path().display(), fixture.lock_path().display(),
    ));
    // Take the byte snapshot before opening a new observer, because even a
    // read-only SQLite observer can manage WAL/SHM sidecars on open/close.
    assert_same_tree(&before, &app_tree(&fixture.app_root()));
    assert_eq!(recovery_row(&fixture.database_path()), waiting);
    assert_eq!(read_bounded(&fixture.settings_path(), MAX_TREE_BYTES).unwrap(), seeded_settings);
    fixture.assert_held_profile_unchanged();

    fixture.release_profile();
    let admitted = fixture.run("admitted-recovery", &[], &[]);
    admitted.assert_admitted_constructor_stop();
    let recovered = recovery_row(&fixture.database_path());
    let finished = recovered.finished_at_ms.expect("existing run receives recovery completion time");
    assert!(finished >= CREATED_AT_MS);
    let mut expected = waiting;
    expected.turn_state = "failed".into();
    expected.active_run_id = None;
    expected.run_state = "failed".into();
    expected.finished_at_ms = Some(finished);
    assert_eq!(recovered, expected, "same logical turn, original run and run count must survive recovery");
    assert_eq!(read_bounded(&fixture.settings_path(), MAX_TREE_BYTES).unwrap(), canonical_settings);
    fixture.assert_retired_and_successor_can_acquire();
    fixture.finish();
}

#[test]
fn admitted_startup_failures_retire_profile_guard() {
    for layout_failure in [true, false] {
        let mut fixture = Fixture::new(if layout_failure { "layout-error" } else { "mode-error" });
        assert_absent(&fixture.guard_path());
        assert_absent(&fixture.lock_path());
        let result = if layout_failure {
            fs::write(fixture.app_root(), b"owned file prevents directory layout")
                .expect("create deterministic layout failure");
            fixture.run("layout-error", &[], &[])
        } else {
            fixture.run("mode-error", &[], &[("FABUSHI_FEATURE_HOST_MODE", "startup-admission-invalid")])
        };
        assert_eq!(result.status.code(), Some(1), "{}", result.details());
        if layout_failure {
            let prefix = format!(
                "failed to initialize managed Mahayana runtime layout at {}: ",
                fixture.app_root().display(),
            );
            assert!(result.stderr.lines().any(|line| line.starts_with(&prefix)), "{}", result.details());
        } else {
            result.assert_exit_with_line(1,
                "failed to initialize unified Mahayana app host: invalid request: unsupported FABUSHI_FEATURE_HOST_MODE \"startup-admission-invalid\"; expected test or production");
        }
        assert!(!result.stderr.contains("failed to acquire Mahayana Host lock"));
        fixture.assert_retired_and_successor_can_acquire();
        fixture.finish();
    }
}

#[test]
fn auxiliary_roles_reject_their_own_invalid_requests_without_normal_startup() {
    let mut fixture = Fixture::new("auxiliary");
    fixture.hold_profile();
    let local_exec = fixture.run("local-exec", &["--local-exec-file-sandbox"], &[]);
    local_exec.assert_exit_with_line(2, "FABUSHI_LOCAL_EXEC_FILE_ROOT is required");
    assert!(!local_exec.stderr.contains("failed to acquire Mahayana Host lock"));
    assert_absent(&fixture.app_root());
    fixture.assert_held_profile_unchanged();

    // Reject before auth/provider setup, the copy-in lock and the fixed /tmp
    // status path. Its only permitted write is telemetry in this owned profile.
    let copy_in = fixture.run("box-copy-in", &["--box-copy-in"], &[
        ("SAND_BOX_STORE_COPY_IN", "1"),
        ("FABUSHI_API_BASE_URL", "invalid-startup-admission-origin"),
    ]);
    copy_in.assert_exit_with_line(1,
        "[box-copy-in] failed files=0 verified=0 failures=1 reason=invalid production backend URL: invalid backend URL: invalid product service origin");
    assert!(!copy_in.stderr.contains("failed to acquire Mahayana Host lock"));
    assert_absent(&fixture.root.join("profile/box-store-sync.lock"));
    assert_absent(&fixture.app_root());
    fixture.assert_held_profile_unchanged();
    fixture.finish();
}

#[test]
fn pairing_store_failures_preserve_obstacles_and_release_profile_then_recover() {
    for directory_obstacle in [false, true] {
        let mut fixture = Fixture::new(if directory_obstacle { "pairing-directory" } else { "pairing-corrupt" });
        fs::create_dir(fixture.app_root()).expect("create only the owned fixture app root");
        let database = fixture.pairing_database_path();
        if directory_obstacle {
            fs::create_dir(&database).expect("create deterministic directory obstacle");
            fs::create_dir(database.join("retained")).expect("create sentinel child directory");
            fs::write(database.join("retained/sentinel.bin"), b"preserve sentinel\0\xff")
                .expect("write original directory sentinel");
        } else {
            fs::write(&database, b"DPS-v1: not a SQLite database\0\xff\r\n")
                .expect("write deterministic non-SQLite bytes");
        }
        // Only the pairing obstacle is compared after admitted startup. Basic
        // Host constructors may legitimately recover their own durable state.
        let obstacle = app_tree(&database);
        fixture.hold_profile();
        let refused = fixture.run("refused-profile", &[], &[]);
        refused.assert_exit_with_line(0, &format!(
            "failed to acquire Mahayana Host lock at {}: Host physical custody is still held for {}",
            fixture.lock_path().display(), fixture.lock_path().display(),
        ));
        assert!(!refused.stderr.contains(PAIRING_FAILURE_PREFIX), "{}", refused.details());
        assert_same_tree(&obstacle, &app_tree(&database));
        fixture.assert_held_profile_unchanged();
        fixture.release_profile();

        let failed = fixture.run("pairing-failure", &[], &[]);
        failed.assert_pairing_initialization_failure();
        assert_same_tree(&obstacle, &app_tree(&database));
        assert_absent(&fixture.root.join("profile/gateway.json"));
        fixture.assert_retired_and_successor_can_acquire();

        // Repair only this fixture's obstacle, preserving it for the oracle.
        let preserved = fixture.root.join("preserved-pairing-obstacle");
        fs::rename(&database, &preserved).expect("move only the owned pairing obstacle");
        assert_same_tree(&obstacle, &app_tree(&preserved));
        let recovered = fixture.run("pairing-recovered", &[], &[]);
        recovered.assert_admitted_constructor_stop();
        assert_shipping_pairing_schema(&database);
        assert_same_tree(&obstacle, &app_tree(&preserved));
        assert_absent(&fixture.root.join("profile/gateway.json"));
        fixture.assert_retired_and_successor_can_acquire();
        fixture.finish();
    }
}

#[test]
fn missing_pairing_store_is_created_before_daemon_and_reopened_on_restart() {
    let mut fixture = Fixture::new("pairing-first-run");
    let database = fixture.pairing_database_path();
    assert_absent(&database);
    let first = fixture.run("pairing-created", &[], &[]);
    first.assert_admitted_constructor_stop();
    assert_shipping_pairing_schema(&database);
    fixture.assert_retired_and_successor_can_acquire();
    {
        let owner = DevicePairingOwner::new(&database).expect("open shipping-created pairing owner");
        assert_eq!(owner.current_revision(), 0);
        assert_eq!(owner.handle_pair_list(&json!({})).unwrap(), json!({ "pending": [], "paired": [] }));
    }

    let restarted = fixture.run("pairing-reopened", &[], &[]);
    restarted.assert_admitted_constructor_stop();
    assert_shipping_pairing_schema(&database);
    assert_absent(&fixture.root.join("profile/gateway.json"));
    fixture.assert_retired_and_successor_can_acquire();
    fixture.finish();
}

#[test]
fn durable_pairing_tokens_and_revision_survive_real_startup_and_storage_recovery() {
    let mut fixture = Fixture::new("pairing-durable-tokens");
    let database = fixture.pairing_database_path();
    let first = fixture.run("create-pairing-schema", &[], &[]);
    first.assert_admitted_constructor_stop();
    assert_shipping_pairing_schema(&database);
    fixture.assert_retired_and_successor_can_acquire();

    let (operator_token, original_node_token) = {
        let owner = DevicePairingOwner::new(&database).expect("open real pairing owner");
        let pending = owner.handle_pair_request(&json!({
            "deviceId": PAIRING_DEVICE_ID,
            "publicKey": PAIRING_PUBLIC_KEY,
            "displayName": "Durable device",
            "platform": "test-platform",
            "roles": ["operator", "node"],
            "scopes": ["operator.read", "operator.write", "node.invoke"]
        })).expect("request pairing through the canonical owner");
        let approval = owner.handle_pair_approve(&json!({ "requestId": pending["requestId"] }))
            .expect("approve the durable device");
        assert_eq!(approval["revision"].as_u64(), Some(1));
        (rotate_pairing_token(&owner, "operator", 2), rotate_pairing_token(&owner, "node", 3))
    };
    assert_durable_pairing_identity(&database);
    let restarted = fixture.run("issued-tokens-restarted", &[], &[]);
    restarted.assert_admitted_constructor_stop();
    fixture.assert_retired_and_successor_can_acquire();
    assert_durable_pairing_identity(&database);
    let replacement_node_token = {
        let owner = DevicePairingOwner::new(&database).expect("reopen after real startup");
        assert_eq!(owner.current_revision(), 3);
        assert_pairing_token(&owner, &operator_token, "operator", &["operator.read", "operator.write"]);
        assert_pairing_token(&owner, &original_node_token, "node", &["node.invoke"]);
        let replacement = rotate_pairing_token(&owner, "node", 4);
        assert!(replacement != original_node_token, "node rotation must issue a different token");
        assert!(owner.authenticate_device_token(&original_node_token).unwrap().is_none());
        assert_pairing_token(&owner, &operator_token, "operator", &["operator.read", "operator.write"]);
        replacement
    };

    // All connections are closed before moving the original durable database.
    // Failure must preserve both its backup and the deterministic obstruction.
    let backup = fixture.root.join("original-durable-pairing.db");
    let durable_bytes = read_bounded(&database, MAX_TREE_BYTES).expect("retain original pairing bytes");
    fs::rename(&database, &backup).expect("retain the owned original database");
    let corrupt_bytes = b"DPS-v1 temporary storage obstacle\0\xfe";
    fs::write(&database, corrupt_bytes).expect("install owned corrupt-file obstacle");
    let failed = fixture.run("durable-storage-unavailable", &[], &[]);
    failed.assert_pairing_initialization_failure();
    assert_eq!(read_bounded(&database, 4096).unwrap().as_slice(), corrupt_bytes.as_slice());
    assert!(read_bounded(&backup, MAX_TREE_BYTES).unwrap() == durable_bytes,
        "failed startup must preserve the original durable database bytes");
    assert_absent(&fixture.root.join("profile/gateway.json"));
    fixture.assert_retired_and_successor_can_acquire();

    fs::remove_file(&database).expect("remove only this fixture's corrupt obstacle");
    fs::rename(&backup, &database).expect("restore the original owned database");
    let restored = fixture.run("original-storage-restored", &[], &[]);
    restored.assert_admitted_constructor_stop();
    fixture.assert_retired_and_successor_can_acquire();
    assert_durable_pairing_identity(&database);
    {
        let owner = DevicePairingOwner::new(&database).expect("reopen restored durable owner");
        assert_eq!(owner.current_revision(), 4);
        assert!(owner.authenticate_device_token(&original_node_token).unwrap().is_none());
        assert_pairing_token(&owner, &operator_token, "operator", &["operator.read", "operator.write"]);
        assert_pairing_token(&owner, &replacement_node_token, "node", &["node.invoke"]);
        let revoked = owner.handle_pair_revoke(&json!({ "deviceId": PAIRING_DEVICE_ID }))
            .expect("revoke the durable pairing");
        assert_eq!(revoked["revision"].as_u64(), Some(5));
    }

    let revoked_restart = fixture.run("revoked-pairing-restarted", &[], &[]);
    revoked_restart.assert_admitted_constructor_stop();
    fixture.assert_retired_and_successor_can_acquire();
    {
        let owner = DevicePairingOwner::new(&database).expect("reopen revoked pairing owner");
        assert_eq!(owner.current_revision(), 5);
        for token in [&operator_token, &original_node_token, &replacement_node_token] {
            assert!(owner.authenticate_device_token(token).unwrap().is_none(),
                "rotation/revocation must remain effective after restart");
        }
        assert_eq!(owner.handle_pair_list(&json!({})).unwrap(), json!({ "pending": [], "paired": [] }));
        assert_eq!(owner.handle_pair_status(&json!({ "deviceId": PAIRING_DEVICE_ID })).unwrap()["status"], "rejected");
    }
    assert_absent(&fixture.root.join("profile/gateway.json"));
    fixture.finish();
}
