#[path = "lib.rs"]
mod base;

pub use base::{
    DEFAULT_REPLAY_BYTES_PER_SESSION, DEFAULT_REPLAY_EVENTS_PER_SESSION, DEFAULT_REPLAY_SESSIONS,
    DispatchResult, GatewayRuntime, ReplayLimits, ReplaySlice, RpcFailure,
};

use mahayana_core::RuntimeEvent;
use mahayana_gateway_protocol::{GATEWAY_PROTOCOL_VERSION, GatewayEvent, GatewayEventEnvelope};
use serde_json::{Map, Value, json};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

const STATE_SCHEMA_VERSION: u64 = 1;
const STATE_FILE_NAME: &str = "gateway-state-v1.json";
const STATE_LOCK_WAIT_ATTEMPTS: usize = 200;
const STATE_LOCK_WAIT: Duration = Duration::from_millis(10);

/// Rust-owned gateway state with a durable replay journal.
///
/// The transport-neutral state machine remains in `lib.rs`; this wrapper adds
/// process-restart durability without moving sequencing or transcript authority
/// into Electron/React. A restart deliberately creates a new replay epoch and
/// rehydrates retained semantic events under that epoch. Clients that still
/// hold the previous epoch therefore take the normal reset/reload path instead
/// of accepting stale sequence numbers.
pub struct GatewayState {
    inner: base::GatewayState,
    limits: ReplayLimits,
    journal: VecDeque<GatewayEventEnvelope>,
    state_path: Option<PathBuf>,
    last_persistence_error: Option<String>,
    restored_events: usize,
}

impl GatewayState {
    pub fn new(limits: ReplayLimits) -> Self {
        Self::with_path(limits, default_state_path())
    }

    pub fn with_path(limits: ReplayLimits, state_path: Option<PathBuf>) -> Self {
        let mut state = Self {
            inner: base::GatewayState::new(limits),
            limits,
            journal: VecDeque::new(),
            state_path,
            last_persistence_error: None,
            restored_events: 0,
        };
        if let Some(path) = state.state_path.clone() {
            match load_replay_events(&path) {
                Ok(events) => {
                    if let Err(error) = state.restore_events(events) {
                        state.last_persistence_error = Some(error);
                    }
                }
                Err(error) => state.last_persistence_error = Some(error),
            }
        }
        state
    }

    pub fn replay_epoch(&self) -> &str {
        self.inner.replay_epoch()
    }

    pub fn register_turn(&mut self, turn_id: impl Into<String>, session_id: impl Into<String>) {
        self.inner.register_turn(turn_id, session_id);
    }

    pub fn emit(
        &mut self,
        session_id: impl Into<String>,
        turn_id: impl Into<String>,
        timestamp_ms: i64,
        event: GatewayEvent,
    ) -> GatewayEventEnvelope {
        let envelope = self
            .inner
            .emit(session_id, turn_id, timestamp_ms, event);
        self.record_events(std::slice::from_ref(&envelope));
        envelope
    }

    pub fn ingest_runtime_event(
        &mut self,
        event: &RuntimeEvent,
        timestamp_ms: i64,
    ) -> Vec<GatewayEventEnvelope> {
        let events = self.inner.ingest_runtime_event(event, timestamp_ms);
        self.record_events(&events);
        events
    }

    pub fn replay_since(
        &self,
        session_id: &str,
        last_seen: u64,
        expected_epoch: Option<&str>,
    ) -> ReplaySlice {
        self.inner
            .replay_since(session_id, last_seen, expected_epoch)
    }

    pub fn interrupt_turn(
        &mut self,
        turn_id: &str,
        timestamp_ms: i64,
    ) -> Vec<GatewayEventEnvelope> {
        let events = self.inner.interrupt_turn(turn_id, timestamp_ms);
        self.record_events(&events);
        events
    }

    pub fn durability_info(&self) -> Value {
        json!({
            "enabled": self.state_path.is_some(),
            "retainedEvents": self.journal.len(),
            "restoredEvents": self.restored_events,
            "lastError": self.last_persistence_error,
        })
    }

    fn restore_events(&mut self, events: Vec<GatewayEventEnvelope>) -> Result<(), String> {
        let mut active_turns = HashMap::<String, String>::new();
        let mut restored = 0usize;
        for envelope in events {
            if envelope.protocol_version != GATEWAY_PROTOCOL_VERSION {
                return Err(format!(
                    "unsupported persisted gateway protocol version {}",
                    envelope.protocol_version
                ));
            }
            let timestamp_ms = chrono::DateTime::parse_from_rfc3339(&envelope.timestamp)
                .map_err(|error| format!("invalid persisted gateway timestamp: {error}"))?
                .timestamp_millis();
            let is_complete = matches!(envelope.event, GatewayEvent::MessageComplete(_));
            let session_id = envelope.session_id;
            let turn_id = envelope.turn_id;
            let normalized = self.inner.emit(
                session_id.clone(),
                turn_id.clone(),
                timestamp_ms,
                envelope.event,
            );
            if is_complete {
                active_turns.remove(&turn_id);
            } else {
                active_turns.insert(turn_id.clone(), session_id.clone());
            }
            self.journal.push_back(normalized);
            restored = restored.saturating_add(1);
        }
        for (turn_id, session_id) in active_turns {
            self.inner.register_turn(turn_id, session_id);
        }
        self.restored_events = restored;
        self.prune_journal();
        Ok(())
    }

    fn record_events(&mut self, events: &[GatewayEventEnvelope]) {
        if events.is_empty() {
            return;
        }
        self.journal.extend(events.iter().cloned());
        self.prune_journal();
        if let Some(path) = self.state_path.clone() {
            if let Err(error) = update_replay_events(&path, &self.journal) {
                self.last_persistence_error = Some(error);
            } else {
                self.last_persistence_error = None;
            }
        }
    }

    fn prune_journal(&mut self) {
        let max_sessions = self.limits.max_sessions.max(1);
        let max_events = self.limits.max_events_per_session.max(1);
        let max_bytes = self.limits.max_bytes_per_session.max(1);

        let mut newest_sessions = Vec::<String>::new();
        let mut seen = HashSet::<String>::new();
        for event in self.journal.iter().rev() {
            if seen.insert(event.session_id.clone()) {
                newest_sessions.push(event.session_id.clone());
                if newest_sessions.len() >= max_sessions {
                    break;
                }
            }
        }
        let keep = newest_sessions.into_iter().collect::<HashSet<_>>();
        self.journal.retain(|event| keep.contains(&event.session_id));

        for session_id in keep {
            loop {
                let mut count = 0usize;
                let mut bytes = 0usize;
                for event in self
                    .journal
                    .iter()
                    .filter(|event| event.session_id == session_id)
                {
                    count = count.saturating_add(1);
                    bytes = bytes.saturating_add(
                        serde_json::to_vec(event)
                            .map(|encoded| encoded.len())
                            .unwrap_or(usize::MAX),
                    );
                }
                if count <= max_events && bytes <= max_bytes {
                    break;
                }
                let Some(index) = self
                    .journal
                    .iter()
                    .position(|event| event.session_id == session_id)
                else {
                    break;
                };
                self.journal.remove(index);
            }
        }
    }
}

pub fn dispatch_request<R: GatewayRuntime>(
    method: &str,
    params: &Value,
    runtime: &mut R,
    state: &mut GatewayState,
    timestamp_ms: i64,
) -> Result<DispatchResult, RpcFailure> {
    let mut dispatched = base::dispatch_request(
        method,
        params,
        runtime,
        &mut state.inner,
        timestamp_ms,
    )?;
    state.record_events(&dispatched.events);
    if method == "gateway.info"
        && let Some(object) = dispatched.result.as_object_mut()
    {
        object.insert("durability".to_string(), state.durability_info());
    }
    Ok(dispatched)
}

fn default_state_path() -> Option<PathBuf> {
    if let Some(configured) = std::env::var_os("MAHAYANA_GATEWAY_STATE") {
        let configured = configured.to_string_lossy();
        if configured.eq_ignore_ascii_case("memory") || configured.eq_ignore_ascii_case("off") {
            return None;
        }
        if !configured.trim().is_empty() {
            return Some(PathBuf::from(configured.as_ref()));
        }
    }
    if let Some(home) = std::env::var_os("MAHAYANA_HOME") {
        return Some(PathBuf::from(home).join(STATE_FILE_NAME));
    }
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(|home| PathBuf::from(home).join(".mahayana").join(STATE_FILE_NAME))
}

fn load_replay_events(path: &Path) -> Result<Vec<GatewayEventEnvelope>, String> {
    let document = read_state_document(path)?;
    let Some(replay) = document.get("replay") else {
        return Ok(Vec::new());
    };
    let events = replay
        .get("events")
        .cloned()
        .unwrap_or_else(|| Value::Array(Vec::new()));
    serde_json::from_value(events).map_err(|error| format!("invalid gateway replay state: {error}"))
}

fn update_replay_events(
    path: &Path,
    events: &VecDeque<GatewayEventEnvelope>,
) -> Result<(), String> {
    with_state_lock(path, || {
        let mut document = read_state_document_unlocked(path)?;
        let object = document
            .as_object_mut()
            .ok_or_else(|| "gateway state root must be an object".to_string())?;
        object.insert("schemaVersion".to_string(), Value::from(STATE_SCHEMA_VERSION));
        object.insert(
            "replay".to_string(),
            json!({"events": events.iter().cloned().collect::<Vec<_>>() }),
        );
        write_state_document_unlocked(path, &document)
    })
}

fn read_state_document(path: &Path) -> Result<Value, String> {
    with_state_lock(path, || read_state_document_unlocked(path))
}

fn read_state_document_unlocked(path: &Path) -> Result<Value, String> {
    let backup = backup_path(path);
    let primary_exists = regular_file_exists(path)?;
    let backup_exists = regular_file_exists(&backup)?;
    let source = if primary_exists {
        Some(path)
    } else if backup_exists {
        Some(backup.as_path())
    } else {
        None
    };
    let Some(source) = source else {
        return Ok(Value::Object(Map::new()));
    };
    let mut file = File::open(source)
        .map_err(|error| format!("failed to open gateway state {}: {error}", source.display()))?;
    verify_open_regular_file(source, &file)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| format!("failed to read gateway state {}: {error}", source.display()))?;
    verify_open_regular_file(source, &file)?;
    let document: Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("failed to parse gateway state {}: {error}", source.display()))?;
    let object = document
        .as_object()
        .ok_or_else(|| "gateway state root must be an object".to_string())?;
    if let Some(version) = object.get("schemaVersion") {
        if version.as_u64() != Some(STATE_SCHEMA_VERSION) {
            return Err(format!(
                "unsupported gateway state schema version {version}; expected {STATE_SCHEMA_VERSION}"
            ));
        }
    }
    Ok(document)
}

fn write_state_document_unlocked(path: &Path, document: &Value) -> Result<(), String> {
    ensure_parent(path)?;
    let backup = backup_path(path);
    regular_file_exists(path)?;
    regular_file_exists(&backup)?;
    let bytes = serde_json::to_vec_pretty(document)
        .map_err(|error| format!("failed to encode gateway state: {error}"))?;
    let mut temporary = TemporaryStateFile::create(temporary_path(path))?;
    let file = temporary.file.as_mut().expect("owned temporary file is open");
    set_private_file_permissions(file)?;
    file.write_all(&bytes)
        .and_then(|_| file.write_all(b"\n"))
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("failed to flush gateway state: {error}"))?;
    drop(temporary.file.take());

    let primary_exists = regular_file_exists(path)?;
    let backup_exists = regular_file_exists(&backup)?;
    if primary_exists {
        // If only the backup exists it is the sole recoverable committed
        // document, so it must survive until the new primary is installed.
        if backup_exists {
            fs::remove_file(&backup)
                .map_err(|error| format!("failed to remove stale gateway state backup: {error}"))?;
        }
        fs::rename(path, &backup)
            .map_err(|error| format!("failed to stage gateway state backup: {error}"))?;
    }
    if let Err(error) = fs::rename(&temporary.path, path) {
        if primary_exists && !regular_file_exists(path)? && regular_file_exists(&backup)? {
            if let Err(restore_error) = fs::rename(&backup, path) {
                return Err(format!(
                    "failed to install gateway state: {error}; prior state remains in backup after restore failed: {restore_error}"
                ));
            }
        }
        return Err(format!("failed to install gateway state: {error}"));
    }
    temporary.installed = true;
    // The primary is the commit point. Backup cleanup is not a failed commit;
    // leaving a regular backup is safe because reads prefer the valid primary.
    if regular_file_exists(&backup).unwrap_or(false) {
        let _ = fs::remove_file(backup);
    }
    Ok(())
}

fn with_state_lock<T>(path: &Path, action: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    ensure_parent(path)?;
    let lock_path = PathBuf::from(format!("{}.lock", path.display()));
    let _lock = acquire_state_lock(&lock_path)?;
    action()
}

fn acquire_state_lock(path: &Path) -> Result<StateFileLock, String> {
    regular_file_exists(path)?;
    let mut options = OpenOptions::new();
    options.create(true).read(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(path)
        .map_err(|error| format!("failed to open gateway state lock: {error}"))?;
    verify_open_regular_file(path, &file)?;
    if file.metadata().map_err(|error| format!("failed to inspect gateway state lock: {error}"))?.len() != 0 {
        return Err("nonempty legacy gateway state lock requires verified owner shutdown".into());
    }
    for _ in 0..STATE_LOCK_WAIT_ATTEMPTS {
        match fs2::FileExt::try_lock_exclusive(&file) {
            Ok(()) => {
                let lock = StateFileLock { file };
                verify_open_regular_file(path, &lock.file)?;
                if lock.file.metadata().map_err(|error| format!("failed to inspect gateway state lock: {error}"))?.len() != 0 {
                    return Err("nonempty legacy gateway state lock requires verified owner shutdown".into());
                }
                set_private_file_permissions(&lock.file)?;
                return Ok(lock);
            }
            Err(error) if error.raw_os_error() == fs2::lock_contended_error().raw_os_error() => {
                thread::sleep(STATE_LOCK_WAIT);
            }
            Err(error) => {
                return Err(format!("failed to acquire gateway state lock: {error}"));
            }
        }
    }
    Err("timed out acquiring gateway state lock".to_string())
}

struct StateFileLock {
    file: File,
}

impl Drop for StateFileLock {
    fn drop(&mut self) {
        let _ = fs2::FileExt::unlock(&self.file);
    }
}

fn ensure_parent(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        // DirBuilder only applies this mode to newly created directories.
        // Never chmod an existing shared parent such as a temp directory.
        builder.create(parent)
            .map_err(|error| format!("failed to create gateway state directory: {error}"))?;
    }
    Ok(())
}

fn regular_file_exists(path: &Path) -> Result<bool, String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => Ok(true),
        Ok(_) => Err(format!("gateway state path must be a regular non-symlink file: {}", path.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!("failed to inspect gateway state path {}: {error}", path.display())),
    }
}

fn verify_open_regular_file(path: &Path, file: &File) -> Result<(), String> {
    let entry = fs::symlink_metadata(path)
        .map_err(|error| format!("failed to inspect gateway state path {}: {error}", path.display()))?;
    let opened = file.metadata()
        .map_err(|error| format!("failed to inspect open gateway state file: {error}"))?;
    if !entry.is_file() || entry.file_type().is_symlink() || !opened.is_file() {
        return Err(format!("gateway state path must be a regular non-symlink file: {}", path.display()));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if entry.dev() != opened.dev() || entry.ino() != opened.ino() {
            return Err("gateway state file identity changed while opening".into());
        }
    }
    Ok(())
}

struct TemporaryStateFile {
    path: PathBuf,
    file: Option<File>,
    installed: bool,
}

impl TemporaryStateFile {
    fn create(path: PathBuf) -> Result<Self, String> {
        let mut options = OpenOptions::new();
        options.create_new(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options.open(&path)
            .map_err(|error| format!("failed to create gateway state temp file: {error}"))?;
        Ok(Self { path, file: Some(file), installed: false })
    }
}

impl Drop for TemporaryStateFile {
    fn drop(&mut self) {
        drop(self.file.take());
        if !self.installed {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn temporary_path(path: &Path) -> PathBuf {
    PathBuf::from(format!(
        "{}.tmp-{}-{}",
        path.display(),
        std::process::id(),
        uuid::Uuid::new_v4().simple()
    ))
}

fn backup_path(path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.bak", path.display()))
}

#[cfg(unix)]
fn set_private_file_permissions(file: &File) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    file.set_permissions(fs::Permissions::from_mode(0o600))
        .map_err(|error| format!("failed to restrict gateway state permissions: {error}"))
}

#[cfg(not(unix))]
fn set_private_file_permissions(_file: &File) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod durability_tests {
    use super::*;
    use mahayana_gateway_protocol::{MessageDeltaPayload, MessageStartPayload};

    struct StateFixture {
        root: PathBuf,
    }

    impl StateFixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "mahayana-gateway-state-{}",
                uuid::Uuid::new_v4().simple()
            ));
            let mut builder = fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            builder.create(&root).expect("create an exclusively owned fixture directory");
            Self { root }
        }

        fn state_path(&self) -> PathBuf {
            self.root.join(STATE_FILE_NAME)
        }

        fn assert_no_temporary_files(&self) {
            assert!(fs::read_dir(&self.root).unwrap().all(|entry| {
                !entry.unwrap().file_name().to_string_lossy().contains(".tmp-")
            }));
        }
    }

    impl Drop for StateFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn restart_rehydrates_replay_under_a_fresh_epoch() {
        let fixture = StateFixture::new();
        let path = fixture.state_path();
        let first_epoch;
        {
            let mut state = GatewayState::with_path(ReplayLimits::default(), Some(path.clone()));
            first_epoch = state.replay_epoch().to_string();
            state.emit(
                "session-1",
                "turn-1",
                1_700_000_000_000,
                GatewayEvent::MessageStart(MessageStartPayload {
                    role: Some("assistant".to_string()),
                }),
            );
            state.emit(
                "session-1",
                "turn-1",
                1_700_000_000_001,
                GatewayEvent::MessageDelta(MessageDeltaPayload {
                    text: "hello".to_string(),
                    rendered: None,
                }),
            );
            assert_eq!(state.replay_since("session-1", 0, None).events.len(), 2);
            assert!(state.durability_info()["lastError"].is_null());
        }

        let restored = GatewayState::with_path(ReplayLimits::default(), Some(path.clone()));
        assert_ne!(restored.replay_epoch(), first_epoch);
        let replay = restored.replay_since("session-1", 0, None);
        assert_eq!(replay.latest_seq, 2);
        assert_eq!(replay.events.len(), 2);
        assert_eq!(replay.events[0].kind(), "message.start");
        assert_eq!(replay.events[1].kind(), "message.delta");
        assert_eq!(restored.restored_events, 2);
        assert!(restored.durability_info()["lastError"].is_null());
    }

    #[test]
    fn shared_state_writer_preserves_non_replay_sections() {
        let fixture = StateFixture::new();
        let path = fixture.state_path();
        ensure_parent(&path).unwrap();
        fs::write(
            &path,
            br#"{"schemaVersion":1,"peer":{"requests":[{"id":"srq-test"}]}}"#,
        )
        .unwrap();
        let mut state = GatewayState::with_path(ReplayLimits::default(), Some(path.clone()));
        state.emit(
            "session-1",
            "turn-1",
            1_700_000_000_000,
            GatewayEvent::MessageDelta(MessageDeltaPayload {
                text: "persist".to_string(),
                rendered: None,
            }),
        );
        let document: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(document["peer"]["requests"][0]["id"], "srq-test");
        assert_eq!(document["replay"]["events"].as_array().unwrap().len(), 1);
        assert!(state.durability_info()["lastError"].is_null());
    }

    #[test]
    fn shared_writer_rejects_invalid_schema_and_lock_inputs_without_writes() {
        let fixture = StateFixture::new();
        let path = fixture.state_path();
        let backup = backup_path(&path);
        let lock = PathBuf::from(format!("{}.lock", path.display()));
        let events = VecDeque::new();
        for version in [Value::Null, json!("1"), json!(-1), json!(1.0), json!(2), json!({})] {
            let bytes = serde_json::to_vec(&json!({
                "schemaVersion": version,
                "peer": { "requests": [] },
                "retained": "untouched",
            })).unwrap();
            fs::write(&path, &bytes).unwrap();
            assert!(update_replay_events(&path, &events).is_err());
            assert_eq!(fs::read(&path).unwrap(), bytes);
            fixture.assert_no_temporary_files();
        }

        let valid = br#"{"schemaVersion":1,"peer":{"requests":[]},"retained":"untouched"}"#;
        fs::write(&path, valid).unwrap();
        fs::write(&lock, b"12345\n").unwrap();
        assert!(update_replay_events(&path, &events).is_err());
        assert_eq!(fs::read(&path).unwrap(), valid);
        assert_eq!(fs::read(&lock).unwrap(), b"12345\n");
        fs::write(&lock, b"").unwrap();

        fs::create_dir(&backup).unwrap();
        assert!(update_replay_events(&path, &events).is_err());
        assert_eq!(fs::read(&path).unwrap(), valid);
        assert!(backup.is_dir());
        fs::remove_dir(&backup).unwrap();
        fs::remove_file(&path).unwrap();
        fs::create_dir(&path).unwrap();
        assert!(update_replay_events(&path, &events).is_err());
        assert!(path.is_dir());
        fs::remove_dir(&path).unwrap();
        fs::write(&path, valid).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::{PermissionsExt, symlink};
            let target = fixture.root.join("unchanged-target");
            fs::write(&target, valid).unwrap();
            for candidate in [&path, &backup, &lock] {
                if candidate.exists() {
                    fs::remove_file(candidate).unwrap();
                }
                symlink(&target, candidate).unwrap();
                assert!(update_replay_events(&path, &events).is_err());
                assert_eq!(fs::read(&target).unwrap(), valid);
                assert!(fs::symlink_metadata(candidate).unwrap().file_type().is_symlink());
                fs::remove_file(candidate).unwrap();
                if candidate == &path {
                    fs::write(&path, valid).unwrap();
                }
            }

            let existing_parent = fixture.root.join("existing-parent");
            fs::create_dir(&existing_parent).unwrap();
            File::open(&existing_parent).unwrap()
                .set_permissions(fs::Permissions::from_mode(0o755)).unwrap();
            update_replay_events(&existing_parent.join(STATE_FILE_NAME), &events).unwrap();
            assert_eq!(fs::metadata(&existing_parent).unwrap().permissions().mode() & 0o777, 0o755);
        }
        fixture.assert_no_temporary_files();
    }

    #[test]
    fn shared_writer_retains_only_backup_on_failure_then_recovers() {
        let fixture = StateFixture::new();
        let path = fixture.state_path();
        let backup = backup_path(&path);
        let lock = PathBuf::from(format!("{}.lock", path.display()));
        let committed = br#"{"schemaVersion":1,"peer":{"requests":[{"id":"srq-retained"}]},"replay":{"events":[]},"retained":{"value":true}}"#;
        fs::write(&backup, committed).unwrap();
        fs::write(&lock, b"12345\n").unwrap();
        let mut state = GatewayState::with_path(ReplayLimits::default(), Some(path.clone()));
        assert!(!state.durability_info()["lastError"].is_null());
        state.emit("session-1", "turn-1", 1_700_000_000_000,
            GatewayEvent::MessageDelta(MessageDeltaPayload { text: "before repair".into(), rendered: None }));
        assert!(!state.durability_info()["lastError"].is_null());
        assert!(!path.exists());
        assert_eq!(fs::read(&backup).unwrap(), committed);
        assert_eq!(fs::read(&lock).unwrap(), b"12345\n");
        fixture.assert_no_temporary_files();

        // This sentinel belongs exclusively to this fixture; clearing it is
        // the controlled repair, never automatic cleanup of a production lock.
        fs::write(&lock, b"").unwrap();
        state.emit("session-1", "turn-1", 1_700_000_000_001,
            GatewayEvent::MessageDelta(MessageDeltaPayload { text: "after repair".into(), rendered: None }));
        assert!(state.durability_info()["lastError"].is_null());
        let document: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(document["peer"]["requests"][0]["id"], "srq-retained");
        assert_eq!(document["retained"]["value"], true);
        assert_eq!(document["replay"]["events"].as_array().unwrap().len(), 2);
        assert!(!backup.exists());
        drop(state);
        let restored = GatewayState::with_path(ReplayLimits::default(), Some(path));
        assert!(restored.durability_info()["lastError"].is_null());
        assert_eq!(restored.restored_events, 2);
        fixture.assert_no_temporary_files();
    }
}
