#[path = "lib.rs"]
mod base;

pub use base::{
    ClarifyLockOutcome, IssuedServerRequest, JSON_RPC_VERSION, PendingServerRequest,
    REQUEST_CANCEL_METHOD, ResolvedServerRequest, SERVER_REQUEST_METHODS, ServerRequestError,
    ServerRequestRegistry, ServerRequestResponse, cancel_notification,
};

use serde_json::{Map, Value, json};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

const STATE_SCHEMA_VERSION: u64 = 1;
const STATE_FILE_NAME: &str = "gateway-state-v1.json";
const STATE_LOCK_WAIT_ATTEMPTS: usize = 200;
const STATE_LOCK_WAIT: Duration = Duration::from_millis(10);

/// Durable Rust owner for server->client requests. A configured store commits
/// each mutation before publishing its result or replacing the live snapshot.
/// The separate handoff from a resolved peer request to Host is not a shared
/// transaction; callers must not infer exactly-once Host effects from this type.
pub struct PersistentServerRequestRegistry {
    inner: ServerRequestRegistry,
    state_path: Option<PathBuf>,
    last_persistence_error: Option<String>,
    restored_requests: usize,
    #[cfg(test)]
    before_commit: Option<fn(&Path)>,
}

impl Default for PersistentServerRequestRegistry {
    fn default() -> Self {
        Self::new(default_state_path())
    }
}

impl PersistentServerRequestRegistry {
    pub fn new(state_path: Option<PathBuf>) -> Self {
        let mut registry = Self {
            inner: ServerRequestRegistry::default(),
            state_path,
            last_persistence_error: None,
            restored_requests: 0,
            #[cfg(test)]
            before_commit: None,
        };
        if let Some(path) = registry.state_path.clone() {
            match load_registry(&path) {
                Ok(restored) => {
                    registry.restored_requests = restored.len();
                    registry.inner = restored;
                }
                Err(error) => registry.last_persistence_error = Some(error.to_string()),
            }
        }
        registry
    }

    pub fn is_response_frame(frame: &Value) -> bool {
        ServerRequestRegistry::is_response_frame(frame)
    }

    pub fn issue(
        &mut self,
        session_id: impl Into<String>,
        method: impl Into<String>,
        params: Value,
    ) -> Result<IssuedServerRequest, ServerRequestError> {
        let session_id = session_id.into();
        let method = method.into();
        self.mutate(|candidate| candidate.issue(session_id, method, params))
    }

    pub fn issue_with_questions(
        &mut self,
        session_id: impl Into<String>,
        method: impl Into<String>,
        params: Value,
        question_ids: Vec<String>,
    ) -> Result<IssuedServerRequest, ServerRequestError> {
        let session_id = session_id.into();
        let method = method.into();
        self.mutate(|candidate| {
            candidate.issue_with_questions(session_id, method, params, question_ids)
        })
    }

    /// Deduplicate a runtime approval while holding the same transaction lock
    /// used to read its current pending state. A cached transport snapshot is
    /// never the admission authority for this operation.
    pub fn issue_approval_request(
        &mut self,
        session_id: impl Into<String>,
        params: Value,
    ) -> Result<Option<IssuedServerRequest>, ServerRequestError> {
        if !params.is_object() {
            return Err(ServerRequestError::ParamsMustBeObject);
        }
        let request_id = params
            .get("requestId")
            .and_then(Value::as_str)
            .filter(|id| !id.trim().is_empty())
            .ok_or(ServerRequestError::InvalidApprovalRequestId)?
            .to_string();
        let session_id = session_id.into();
        self.mutate(|candidate| {
            if candidate.export_state().iter().any(|request| {
                request.method == "approval"
                    && request.params.get("requestId").and_then(Value::as_str)
                        == Some(request_id.as_str())
            }) {
                return Ok(None);
            }
            candidate.issue(session_id, "approval", params).map(Some)
        })
    }

    pub fn resolve_response(
        &mut self,
        frame: &Value,
    ) -> Result<Option<ResolvedServerRequest>, ServerRequestError> {
        self.mutate(|candidate| Ok(candidate.resolve_response(frame)))
    }

    /// Compatibility lookup by runtime approval identity. response is the
    /// JSON-RPC result payload, not an already assembled response frame.
    pub fn resolve_approval_request(
        &mut self,
        approval_id: &str,
        response: Value,
    ) -> Result<Option<ResolvedServerRequest>, ServerRequestError> {
        if approval_id.trim().is_empty() {
            return Err(ServerRequestError::InvalidApprovalRequestId);
        }
        self.mutate(|candidate| {
            let pending = candidate.export_state().into_iter().find(|request| {
                request.method == "approval"
                    && request.params.get("requestId").and_then(Value::as_str)
                        == Some(approval_id)
            });
            let Some(request) = pending else {
                return Ok(None);
            };
            Ok(candidate.resolve_response(&json!({
                "jsonrpc": JSON_RPC_VERSION,
                "id": request.id,
                "result": response,
            })))
        })
    }

    pub fn lock_answer(
        &mut self,
        request_id: &str,
        question_id: &str,
        answer: impl Into<String>,
    ) -> Result<Option<ClarifyLockOutcome>, ServerRequestError> {
        let answer = answer.into();
        self.mutate(|candidate| candidate.lock_answer(request_id, question_id, answer))
    }

    pub fn cancel_session(
        &mut self,
        session_id: Option<&str>,
        reason: &str,
    ) -> Result<Vec<Value>, ServerRequestError> {
        self.mutate(|candidate| Ok(candidate.cancel_session(session_id, reason)))
    }

    pub fn open_requests(&self, session_id: &str) -> Vec<Value> {
        self.inner.open_requests(session_id)
    }

    pub fn pending_kind(&self, session_id: &str) -> Option<&str> {
        self.inner.pending_kind(session_id)
    }

    pub fn export_state(&self) -> Vec<PendingServerRequest> {
        self.inner.export_state()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn durability_info(&self) -> Value {
        json!({
            "enabled": self.state_path.is_some(),
            "openRequests": self.inner.len(),
            "restoredRequests": self.restored_requests,
            "lastError": self.last_persistence_error,
        })
    }

    fn mutate<T>(
        &mut self,
        operation: impl FnOnce(&mut ServerRequestRegistry) -> Result<T, ServerRequestError>,
    ) -> Result<T, ServerRequestError> {
        #[cfg(test)]
        let before_commit = self.before_commit.take();

        let transition = if let Some(path) = self.state_path.clone() {
            with_state_lock(&path, || {
                let mut document =
                    read_state_document_unlocked(&path).map_err(ServerRequestError::Persistence)?;
                let mut candidate = registry_from_document(&document)?;
                let before = candidate.export_state();
                let result = operation(&mut candidate)?;
                let requests = candidate.export_state();
                if requests != before {
                    set_document_requests(&mut document, requests)?;
                    // This hook only introduces a real fixture filesystem fault
                    // after the candidate changed; the real writer still runs.
                    #[cfg(test)]
                    if let Some(hook) = before_commit {
                        hook(&path);
                    }
                    write_state_document_unlocked(&path, &document)
                        .map_err(ServerRequestError::Persistence)?;
                }
                Ok((candidate, result))
            })
        } else {
            let mut candidate = self.inner.clone();
            operation(&mut candidate).map(|result| (candidate, result))
        };

        match transition {
            Ok((candidate, result)) => {
                self.inner = candidate;
                self.last_persistence_error = None;
                Ok(result)
            }
            Err(error) => {
                if matches!(
                    &error,
                    ServerRequestError::Persistence(_) | ServerRequestError::InvalidPersistedState(_)
                ) {
                    self.last_persistence_error = Some(error.to_string());
                }
                Err(error)
            }
        }
    }
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

fn load_registry(path: &Path) -> Result<ServerRequestRegistry, ServerRequestError> {
    with_state_lock(path, || {
        let document =
            read_state_document_unlocked(path).map_err(ServerRequestError::Persistence)?;
        registry_from_document(&document)
    })
}

fn registry_from_document(document: &Value) -> Result<ServerRequestRegistry, ServerRequestError> {
    let requests = match document.get("peer") {
        None => Vec::new(),
        Some(Value::Object(peer)) => {
            let requests = peer.get("requests").ok_or_else(|| {
                ServerRequestError::InvalidPersistedState("peer.requests is missing".into())
            })?;
            serde_json::from_value::<Vec<PendingServerRequest>>(requests.clone())
                .map_err(|error| ServerRequestError::InvalidPersistedState(error.to_string()))?
        }
        Some(_) => {
            return Err(ServerRequestError::InvalidPersistedState(
                "peer must be an object".into(),
            ));
        }
    };
    let mut candidate = ServerRequestRegistry::default();
    candidate
        .import_state(requests)
        .map_err(|error| ServerRequestError::InvalidPersistedState(error.to_string()))?;
    Ok(candidate)
}

fn set_document_requests(
    document: &mut Value,
    requests: Vec<PendingServerRequest>,
) -> Result<(), ServerRequestError> {
    let object = document.as_object_mut().ok_or_else(|| {
        ServerRequestError::InvalidPersistedState("gateway state root must be an object".into())
    })?;
    object.insert("schemaVersion".to_string(), Value::from(STATE_SCHEMA_VERSION));
    let peer = object.entry("peer".to_string()).or_insert_with(|| json!({}));
    let peer = peer.as_object_mut().ok_or_else(|| {
        ServerRequestError::InvalidPersistedState("peer must be an object".into())
    })?;
    peer.insert(
        "requests".to_string(),
        serde_json::to_value(requests)
            .map_err(|error| ServerRequestError::InvalidPersistedState(error.to_string()))?,
    );
    Ok(())
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

fn with_state_lock<T>(
    path: &Path,
    action: impl FnOnce() -> Result<T, ServerRequestError>,
) -> Result<T, ServerRequestError> {
    ensure_parent(path).map_err(ServerRequestError::Persistence)?;
    let lock_path = PathBuf::from(format!("{}.lock", path.display()));
    let _lock = acquire_state_lock(&lock_path).map_err(ServerRequestError::Persistence)?;
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
mod durability_tests;
