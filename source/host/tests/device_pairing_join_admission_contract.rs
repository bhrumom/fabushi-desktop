//! DJC-v1: real file-backed owner, independent SQL observations and Gateway HTTP.
//! No setup issuer/UI, companion-device, arbitrary I/O or unknown-COMMIT claim.

use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Barrier, mpsc};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::device_pairing_owner::{DevicePairingOwner, DeviceTokenAuthorization};
use mahayana_host_runtime::gateway_config::GatewayServerConfig;
use mahayana_host_runtime::gateway_server::{
    GatewayApi, GatewayCommandError, GatewayEventHub, GatewayServer, GatewayServerDeps,
    start_gateway_server,
};
use rusqlite::{Connection, OpenFlags, params, types::Value as SqlValue};
use serde_json::{Value, json};

const PRIVATE_PAYLOAD: &str = "djc-private-bootstrap-material";
const PRIVATE_TRIGGER: &str = "djc-private-trigger-diagnostic";
const STORAGE_ERROR: &str = "device pairing join-code storage unavailable";

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).expect("fixture clock").as_millis() as i64
}

fn payload() -> Value {
    json!({"bootstrapToken": PRIVATE_PAYLOAD, "legacy": [true, 7, "保留"]})
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StoredRow {
    shortcode: String,
    payload_type: String,
    payload_bytes: Vec<u8>,
    expiry_type: String,
    expiry_sql: String,
    created_at_ms: i64,
    redeemed_at_ms: Option<i64>,
}

struct PairingApi {
    owner: Option<Arc<DevicePairingOwner>>,
    retired: mpsc::Sender<()>,
}

impl GatewayApi for PairingApi {
    fn call(&self, method: &str, _args: Value) -> Result<Value, GatewayCommandError> {
        Err(GatewayCommandError::UnknownMethod(method.to_owned()))
    }

    fn authenticate_device_token(
        &self,
        token: &str,
    ) -> Result<Option<DeviceTokenAuthorization>, GatewayCommandError> {
        self.owner.as_ref().expect("live pairing API").authenticate_device_token(token)
    }

    fn redeem_device_pairing_join_code(
        &self,
        shortcode: &str,
    ) -> Result<Option<Value>, GatewayCommandError> {
        self.owner.as_ref().expect("live pairing API").redeem_device_pairing_join_code(shortcode)
    }
}

impl Drop for PairingApi {
    fn drop(&mut self) {
        // Observe retirement after every real connection has released this API,
        // and after its owner reference is gone, before reopening/removing files.
        drop(self.owner.take());
        let _ = self.retired.send(());
    }
}

struct Fixture {
    root: PathBuf,
    db_path: PathBuf,
    owner: Option<Arc<DevicePairingOwner>>,
    gateway: Option<(GatewayServer, mpsc::Receiver<()>)>,
    remove_directory_on_drop: bool,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!("fabushi-join-{label}-{}", uuid::Uuid::new_v4()));
        Self::open_root(root, true)
    }

    fn in_owned_root(root: PathBuf) -> Self {
        // The supervising parent retains the directory through completion
        // receipt validation and terminal acceptance, including child success.
        Self::open_root(root, false)
    }

    fn open_root(root: PathBuf, remove_directory_on_drop: bool) -> Self {
        fs::create_dir(&root).expect("exclusively create owned fixture directory");
        let db_path = root.join("device-pairing.db");
        let owner = DevicePairingOwner::new(&db_path).expect("open durable pairing owner");
        Self { root, db_path, owner: Some(Arc::new(owner)), gateway: None, remove_directory_on_drop }
    }

    fn owner(&self) -> &DevicePairingOwner {
        self.owner.as_deref().expect("open fixture owner")
    }

    fn sql(&self) -> Connection {
        Connection::open_with_flags(&self.db_path, OpenFlags::SQLITE_OPEN_READ_WRITE)
            .expect("open existing fixture for explicit SQL fault or repair")
    }

    fn execute(&self, sql: &str) {
        let connection = self.sql();
        connection.execute_batch(sql).expect("apply explicit fixture SQL");
        connection.close().expect("close fixture writer");
    }

    fn set(&self, shortcode: &str, payload: &SqlValue, expiry: &SqlValue) {
        let connection = self.sql();
        assert_eq!(connection.execute(
            "UPDATE device_pairing_join_codes SET payload_json = ?1, expires_at_ms = ?2
             WHERE shortcode = ?3",
            params![payload, expiry, shortcode],
        ).expect("replace only target row content"), 1);
        connection.close().expect("close content writer");
    }

    fn register(&self, payload: &Value) -> String {
        self.owner().register_device_pairing_join_code(payload, now_ms() + 600_000)
            .expect("register live legacy JSON payload")
    }

    fn observe(&self, shortcode: &str) -> StoredRow {
        self.observe_column(shortcode, "payload_json")
    }

    fn observe_column(&self, shortcode: &str, payload_column: &str) -> StoredRow {
        assert!(matches!(payload_column, "payload_json" | "payload_obstacle"));
        let connection = Connection::open_with_flags(&self.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .expect("open independent read-only row observer");
        let sql = format!(
            "SELECT shortcode, typeof({payload_column}), CAST({payload_column} AS BLOB),
                    typeof(expires_at_ms), quote(expires_at_ms), created_at_ms, redeemed_at_ms
             FROM device_pairing_join_codes WHERE shortcode = ?1"
        );
        let observed = connection.query_row(&sql, [shortcode], |row| Ok(StoredRow {
            shortcode: row.get(0)?, payload_type: row.get(1)?, payload_bytes: row.get(2)?,
            expiry_type: row.get(3)?, expiry_sql: row.get(4)?, created_at_ms: row.get(5)?,
            redeemed_at_ms: row.get(6)?,
        })).expect("observe every stored target column without decoding payload");
        connection.close().expect("close independent observer");
        observed
    }

    fn assert_consumed(&self, before: &StoredRow) -> StoredRow {
        let observed = self.observe(&before.shortcode);
        assert!(observed.redeemed_at_ms.is_some_and(|value| value > 0));
        let mut expected = before.clone();
        expected.redeemed_at_ms = observed.redeemed_at_ms;
        assert_eq!(observed, expected, "consumption changes only its existing marker");
        observed
    }

    fn close_gateway(&mut self) -> Result<(), mpsc::RecvTimeoutError> {
        if let Some((server, retired)) = self.gateway.take() {
            server.close();
            retired.recv_timeout(Duration::from_secs(10))?;
        }
        Ok(())
    }

    fn reopen(&mut self) {
        self.close_gateway().expect("retire all owned Gateway references before reopen");
        let owner = self.owner.take().expect("open owner before reopen");
        assert_eq!(Arc::strong_count(&owner), 1, "no outstanding fixture owner consumer");
        drop(owner);
        self.owner = Some(Arc::new(DevicePairingOwner::new(&self.db_path).expect("reopen same durable file")));
    }

    fn start_gateway(&mut self) {
        assert!(self.gateway.is_none());
        let (retired, receiver) = mpsc::channel();
        let server = start_gateway_server(GatewayServerDeps {
            api: Arc::new(PairingApi {
                owner: Some(Arc::clone(self.owner.as_ref().expect("open owner"))),
                retired,
            }),
            events: GatewayEventHub::default(), local_exec: None, webauthn: None,
            config: GatewayServerConfig {
                host: "127.0.0.1".into(), port: None,
                auth_token: Some("djc-native-control-secret".into()), tls: None,
            },
            started_at: 1,
        }).expect("start actual Gateway server");
        self.gateway = Some((server, receiver));
    }

    fn get(&self, path: &str) -> String {
        let port = self.gateway.as_ref().expect("running Gateway").0.port();
        let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect real Gateway");
        stream.set_read_timeout(Some(Duration::from_secs(5))).expect("bounded HTTP read");
        stream.set_write_timeout(Some(Duration::from_secs(5))).expect("bounded HTTP write");
        write!(stream, "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
            .expect("send unauthenticated public join request");
        stream.flush().expect("flush request");
        let mut response = String::new();
        stream.read_to_string(&mut response).expect("receive completed HTTP response");
        response
    }

    fn read_fence(&self) -> Connection {
        let connection = self.sql();
        assert_eq!(connection.query_row("PRAGMA journal_mode", [], |row| row.get::<_, String>(0))
            .expect("observe actual journal mode"), "delete");
        connection.execute_batch("BEGIN DEFERRED").expect("begin fixture read transaction");
        let count: i64 = connection.query_row("SELECT COUNT(*) FROM device_pairing_join_codes", [], |row| row.get(0))
            .expect("acquire a real SQLite SHARED read lock");
        assert!(count > 0);
        connection
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let retirement = self.close_gateway();
        drop(self.owner.take());
        if thread::panicking() {
            eprintln!("DJC fixture retained for failure evidence: {}", self.root.display());
            return;
        }
        retirement.expect("retire every owned Gateway connection before cleanup");
        if self.remove_directory_on_drop {
            fs::remove_dir_all(&self.root).expect("remove only this owned fixture after handles close");
        }
    }
}

fn assert_storage_error(error: GatewayCommandError, shortcode: &str) {
    assert!(matches!(&error, GatewayCommandError::Internal(_)), "{error}");
    assert_eq!(error.status(), 500);
    let diagnostic = error.to_string();
    assert_eq!(diagnostic, STORAGE_ERROR);
    for private in [shortcode, PRIVATE_PAYLOAD, PRIVATE_TRIGGER] {
        assert!(!diagnostic.contains(private), "storage diagnostic exposed private input");
    }
}

fn assert_http(response: &str, status: &str) -> Value {
    assert!(response.starts_with(status), "{response}");
    let (headers, body) = response.split_once("\r\n\r\n").expect("HTTP header/body boundary");
    assert!(headers.to_ascii_lowercase().contains("\r\ncache-control: no-store"), "{headers}");
    serde_json::from_str(body).expect("JSON Gateway body")
}

const JOIN_CHILD_CASE: &str = "FABUSHI_DJC_CHILD_CASE";
const JOIN_CHILD_ROOT: &str = "FABUSHI_DJC_CHILD_ROOT";
const JOIN_CHILD_TOKEN: &str = "FABUSHI_DJC_CHILD_TOKEN";
const JOIN_CHILD_MODE: &str = "FABUSHI_DJC_CHILD_MODE";

#[derive(Debug, PartialEq, Eq)]
enum JoinChildOutcome {
    Completed,
    MissingReceipt,
    MismatchedReceipt,
    InvalidReadyReceipt,
    FailedExit,
    TimedOut,
    WaitFailed,
}

struct JoinChildReport {
    outcome: JoinChildOutcome,
    terminal: bool,
    ready: bool,
    root: PathBuf,
    details: String,
}

impl JoinChildReport {
    fn accept(self, expected: JoinChildOutcome) {
        assert!(
            self.terminal && self.ready && self.outcome == expected,
            "DJC owned child failed; evidence retained at {}: {:?}; {}",
            self.root.display(), self.outcome, self.details,
        );
        fs::remove_dir_all(&self.root).expect("remove only terminal, accepted child evidence");
    }
}

struct OwnedJoinChild {
    child: std::process::Child,
    root: PathBuf,
    terminal: bool,
    stop_attempted: bool,
}

impl OwnedJoinChild {
    fn poll(&mut self) -> std::io::Result<Option<std::process::ExitStatus>> {
        let status = self.child.try_wait()?;
        if status.is_some() {
            self.terminal = true;
        }
        Ok(status)
    }

    fn stop_and_reap(&mut self) -> String {
        self.stop_attempted = true;
        let deadline = Instant::now() + Duration::from_secs(5);
        let kill_error = self.child.kill().err();
        loop {
            match self.poll() {
                Ok(Some(status)) => return format!(
                    "owned child reaped after stop: {status}; kill_error={kill_error:?}",
                ),
                Ok(None) if Instant::now() < deadline => {
                    thread::sleep(Duration::from_millis(10));
                }
                Ok(None) => return format!(
                    "owned child termination unconfirmed after 5s; kill_error={kill_error:?}",
                ),
                Err(error) => return format!(
                    "owned child reap failed: {error}; kill_error={kill_error:?}",
                ),
            }
        }
    }
}

impl Drop for OwnedJoinChild {
    fn drop(&mut self) {
        if !self.terminal && !self.stop_attempted {
            let diagnostic = self.stop_and_reap();
            eprintln!("DJC parent unwound; evidence retained at {}: {diagnostic}", self.root.display());
        }
    }
}

fn child_receipt(token: &str, pid: u32, case: &str, mode: &str) -> String {
    format!("{token}\n{pid}\n{case}\n{mode}\n")
}

fn write_child_receipt(root: &Path, name: &str, receipt: &str) {
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(root.join(name))
        .expect("exclusively create owned child receipt");
    file.write_all(receipt.as_bytes()).expect("write complete child receipt");
    file.flush().expect("flush child receipt");
}

fn supervise_join_child(case: &str, mode: &str) -> JoinChildReport {
    let root = std::env::temp_dir().join(format!("fabushi-join-child-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root).expect("exclusively create parent-owned child directory");
    let token = uuid::Uuid::new_v4().to_string();
    let stdout = fs::OpenOptions::new().write(true).create_new(true).open(root.join("stdout"))
        .expect("create owned child stdout");
    let stderr = fs::OpenOptions::new().write(true).create_new(true).open(root.join("stderr"))
        .expect("create owned child stderr");
    let child = std::process::Command::new(std::env::current_exe().expect("current exact test binary"))
        .args(["--exact", case, "--nocapture", "--test-threads=1"])
        .env(JOIN_CHILD_CASE, case)
        .env(JOIN_CHILD_ROOT, &root)
        .env(JOIN_CHILD_TOKEN, &token)
        .env(JOIN_CHILD_MODE, mode)
        .stdin(std::process::Stdio::null())
        .stdout(stdout)
        .stderr(stderr)
        .spawn()
        .unwrap_or_else(|error| panic!("DJC child spawn failed; retained {}: {error}", root.display()));
    let pid = child.id();
    let expected = child_receipt(&token, pid, case, mode);
    let mut owned = OwnedJoinChild { child, root: root.clone(), terminal: false, stop_attempted: false };
    let started = Instant::now();
    // The synthetic timeout control first proves child entry, then deliberately
    // parks. It qualifies this supervisor only, never SQLite refusal behavior.
    let budget = if mode == "control-timeout" { Duration::from_secs(10) } else { Duration::from_secs(120) };
    let deadline = started + budget;
    let mut control_ready_at = None;
    let (mut outcome, mut details) = loop {
        match owned.poll() {
            Ok(Some(status)) => break (
                if status.success() { JoinChildOutcome::Completed } else { JoinChildOutcome::FailedExit },
                format!("pid={pid}; elapsed_ms={}; exit={status}", started.elapsed().as_millis()),
            ),
            Err(error) => break (JoinChildOutcome::WaitFailed, format!("pid={pid}; wait failed: {error}")),
            Ok(None) => {}
        }
        if mode == "control-timeout"
            && control_ready_at.is_none()
            && fs::read_to_string(root.join("ready")).is_ok_and(|value| value == expected)
        {
            control_ready_at = Some(Instant::now());
        }
        if Instant::now() >= deadline
            || control_ready_at.is_some_and(|ready: Instant| ready.elapsed() >= Duration::from_millis(100))
        {
            break (JoinChildOutcome::TimedOut, format!("pid={pid}; outer deadline expired; elapsed_ms={}", started.elapsed().as_millis()));
        }
        thread::sleep(Duration::from_millis(10));
    };
    if !owned.terminal {
        details.push_str(&format!("; {}", owned.stop_and_reap()));
    }
    let ready = fs::read_to_string(root.join("ready")).is_ok_and(|value| value == expected);
    if outcome == JoinChildOutcome::Completed {
        outcome = if !ready {
            JoinChildOutcome::InvalidReadyReceipt
        } else {
            match fs::read_to_string(root.join("complete")) {
                Ok(value) if value == expected => JoinChildOutcome::Completed,
                Ok(_) => JoinChildOutcome::MismatchedReceipt,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => JoinChildOutcome::MissingReceipt,
                Err(error) => {
                    details.push_str(&format!("; completion receipt read failed: {error}"));
                    JoinChildOutcome::MismatchedReceipt
                }
            }
        };
    }
    // Regular owned files cannot deadlock on undrained child pipe output. These
    // diagnostics are included in the parent failure, including partial output
    // after a timeout. Successful libtest output may remain captured.
    for name in ["stdout", "stderr"] {
        let output = fs::read_to_string(root.join(name))
            .unwrap_or_else(|error| format!("owned output unavailable: {error}"));
        details.push_str(&format!("\nchild {name}:\n{output}"));
    }
    JoinChildReport { outcome, terminal: owned.terminal, ready, root, details }
}

fn complete_receipt_control_fixture(root: &Path) {
    let fixture = Fixture::in_owned_root(root.join("receipt-control"));
    let code = fixture.register(&payload());
    assert!(fixture.observe(&code).redeemed_at_ms.is_none());
    // Returning drops every fixture-owned connection before receipt handling.
    // The parent must still be able to inspect this real SQLite database.
}

fn isolated_join_case(case: &str, body: fn(&Path)) {
    if let Some(selected) = std::env::var_os(JOIN_CHILD_CASE) {
        assert_eq!(selected.to_str(), Some(case), "child must run its exact selected case");
        assert_eq!(
            std::env::args().skip(1).collect::<Vec<_>>(),
            vec!["--exact", case, "--nocapture", "--test-threads=1"],
            "child must enter through the exact same test binary selector",
        );
        let root = PathBuf::from(std::env::var_os(JOIN_CHILD_ROOT).expect("parent-owned child root"));
        assert!(root.is_dir(), "parent must create the child root");
        let token = std::env::var(JOIN_CHILD_TOKEN).expect("parent-issued invocation token");
        let mode = std::env::var(JOIN_CHILD_MODE).expect("parent-selected child mode");
        let receipt = child_receipt(&token, std::process::id(), case, &mode);
        write_child_receipt(&root, "ready", &receipt);
        match mode.as_str() {
            "actual" => body(&root),
            "control-valid" => {}
            "control-missing" => {
                complete_receipt_control_fixture(&root);
                return;
            }
            "control-mismatch" => {
                complete_receipt_control_fixture(&root);
                write_child_receipt(&root, "complete", "wrong-invocation\n");
                return;
            }
            "control-timeout" => loop { thread::park(); },
            _ => panic!("unknown DJC child mode"),
        }
        write_child_receipt(&root, "complete", &receipt);
        return;
    }

    if matches!(case,
        "join_storage_commit_refusal_never_publishes_payload"
            | "join_deferred_commit_refusal_is_qualified_and_preserves_recovery"
    ) {
        for (mode, expected) in [
            ("control-valid", JoinChildOutcome::Completed),
            ("control-missing", JoinChildOutcome::MissingReceipt),
            ("control-mismatch", JoinChildOutcome::MismatchedReceipt),
            ("control-timeout", JoinChildOutcome::TimedOut),
        ] {
            let report = supervise_join_child(case, mode);
            let retained_root = report.root.clone();
            let checks_fixture_custody = matches!(mode, "control-missing" | "control-mismatch");
            if checks_fixture_custody {
                assert!(report.terminal,
                    "DJC receipt control termination unconfirmed; retained {}: {}",
                    retained_root.display(), report.details);
                let db_path = retained_root.join("receipt-control").join("device-pairing.db");
                assert!(db_path.is_file(),
                    "completed child fixture lost before parent receipt acceptance; retained {}: {}",
                    retained_root.display(), report.details);
                let connection = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
                    .expect("inspect retained real SQLite fixture after child termination");
                let rows: i64 = connection.query_row(
                    "SELECT COUNT(*) FROM device_pairing_join_codes", [], |row| row.get(0),
                ).expect("read completed child fixture before accepting rejected receipt");
                assert_eq!(rows, 1, "retained fixture must contain the completed control's real row");
                connection.close().expect("close parent observer before accepted control cleanup");
            }
            report.accept(expected);
            if checks_fixture_custody {
                assert!(!retained_root.exists(), "accepted terminal control must remove its owned directory");
            }
        }
        eprintln!("DJC owned-child harness controls passed; these are not SQL refusal evidence");
    }
    let report = supervise_join_child(case, "actual");
    if matches!(case,
        "join_storage_commit_refusal_never_publishes_payload"
            | "join_reader_observation_notifies_before_diagnostic_release"
            | "join_reader_observation_cannot_notify_before_operation_return"
    ) && report.terminal && report.ready && report.outcome == JoinChildOutcome::Completed
    {
        let records = collect_join_reader_records(case, &report).unwrap_or_else(|failure| {
            panic!("DJC reader records rejected; evidence retained at {}: {failure}\n{}",
                report.root.display(), report.details);
        });
        // Only qualified structured records survive accepted child cleanup.
        // Nested libtest names/summaries stay in the owned raw output files.
        for record in records {
            eprintln!("{record}");
        }
    }
    report.accept(JoinChildOutcome::Completed);
}

fn join_phase(started: Instant, phase: &str) {
    eprintln!("DJC phase={phase}; elapsed_ms={}", started.elapsed().as_millis());
}

fn join_result_phase(started: Instant, result: &Result<Option<Value>, GatewayCommandError>) {
    let class = match result {
        Ok(Some(_)) => "payload",
        Ok(None) => "miss",
        Err(GatewayCommandError::Internal(_)) => "internal-error",
        Err(_) => "other-error",
    };
    eprintln!("DJC phase=owner-returned; elapsed_ms={}; result_class={class}", started.elapsed().as_millis());
}

// DJC-09 observes only the retained reader case and its two instrument controls.
// All offsets share this child case's Instant origin; none are SQL durations.
const JOIN_READER_TRACE_PREFIX: &str = "DJC_READER_TRACE ";

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
enum JoinReaderSendOutcome { Sent, Disconnected }

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
enum JoinReaderReceiveOutcome { Received, Timeout, Disconnected }

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
enum JoinReaderTryOutcome { Empty, Received, Disconnected }

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
enum JoinReaderResultClass { Payload, Miss, InternalError, OtherError }

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct JoinReaderCallRecord {
    entered_ns: Option<u64>,
    returned_ns: Option<u64>,
    send_entered_ns: Option<u64>,
    send_completed_ns: Option<u64>,
    send_outcome: JoinReaderSendOutcome,
    diagnostic_entered_ns: Option<u64>,
    diagnostic_returned_ns: Option<u64>,
    diagnostic_succeeded: bool,
    result_class: JoinReaderResultClass,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct JoinReaderObservationRecord {
    call: Option<JoinReaderCallRecord>,
    wait_entered_ns: Option<u64>,
    wait_returned_ns: Option<u64>,
    wait_outcome: JoinReaderReceiveOutcome,
    join_completed_ns: Option<u64>,
    worker_returned: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct JoinReaderReleaseRecord {
    entered_ns: Option<u64>,
    rollback_returned_ns: Option<u64>,
    rollback_succeeded: bool,
    close_entered_ns: Option<u64>,
    close_returned_ns: Option<u64>,
    close_succeeded: Option<bool>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct JoinReaderDiagnosticControlRecord {
    ready_returned_ns: Option<u64>,
    ready_outcome: JoinReaderReceiveOutcome,
    release_entered_ns: Option<u64>,
    release_completed_ns: Option<u64>,
    release_outcome: JoinReaderSendOutcome,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct JoinReaderOperationControlRecord {
    ready_returned_ns: Option<u64>,
    ready_outcome: JoinReaderReceiveOutcome,
    check_entered_ns: Option<u64>,
    check_returned_ns: Option<u64>,
    before_release: JoinReaderTryOutcome,
    release_entered_ns: Option<u64>,
    release_completed_ns: Option<u64>,
    release_outcome: JoinReaderSendOutcome,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
enum JoinReaderRecord {
    Observation(JoinReaderObservationRecord),
    ReaderRelease(JoinReaderReleaseRecord),
    DiagnosticControl(JoinReaderDiagnosticControlRecord),
    OperationControl(JoinReaderOperationControlRecord),
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct JoinReaderTrace {
    case: String,
    invocation_nonce: String,
    child_pid: u32,
    mode: String,
    instrument_control: bool,
    clock: String,
    record: JoinReaderRecord,
}

fn join_reader_offset(started: Instant, at: Instant) -> Option<u64> {
    at.checked_duration_since(started).and_then(|elapsed| elapsed.as_nanos().try_into().ok())
}

fn join_reader_receive_outcome(result: &Result<(), mpsc::RecvTimeoutError>) -> JoinReaderReceiveOutcome {
    match result {
        Ok(()) => JoinReaderReceiveOutcome::Received,
        Err(mpsc::RecvTimeoutError::Timeout) => JoinReaderReceiveOutcome::Timeout,
        Err(mpsc::RecvTimeoutError::Disconnected) => JoinReaderReceiveOutcome::Disconnected,
    }
}

fn join_reader_send_outcome(result: &Result<(), mpsc::SendError<()>>) -> JoinReaderSendOutcome {
    if result.is_ok() { JoinReaderSendOutcome::Sent } else { JoinReaderSendOutcome::Disconnected }
}

fn observe_join_reader_call(
    started: Instant,
    completion: mpsc::Sender<()>,
    operation: impl FnOnce() -> Result<Option<Value>, GatewayCommandError>,
    diagnostic: impl FnOnce(&Result<Option<Value>, GatewayCommandError>) -> Result<(), &'static str>,
) -> (Result<Option<Value>, GatewayCommandError>, JoinReaderCallRecord) {
    let entered = Instant::now();
    let result = operation();
    let returned = Instant::now();
    let send_entered = Instant::now();
    let sent = completion.send(());
    let send_completed = Instant::now();
    // Nothing formats or logs between the operation return and notification.
    let diagnostic_entered = Instant::now();
    let diagnosed = diagnostic(&result);
    let diagnostic_returned = Instant::now();
    let result_class = match &result {
        Ok(Some(_)) => JoinReaderResultClass::Payload,
        Ok(None) => JoinReaderResultClass::Miss,
        Err(GatewayCommandError::Internal(_)) => JoinReaderResultClass::InternalError,
        Err(_) => JoinReaderResultClass::OtherError,
    };
    (result, JoinReaderCallRecord {
        entered_ns: join_reader_offset(started, entered),
        returned_ns: join_reader_offset(started, returned),
        send_entered_ns: join_reader_offset(started, send_entered),
        send_completed_ns: join_reader_offset(started, send_completed),
        send_outcome: join_reader_send_outcome(&sent),
        diagnostic_entered_ns: join_reader_offset(started, diagnostic_entered),
        diagnostic_returned_ns: join_reader_offset(started, diagnostic_returned),
        diagnostic_succeeded: diagnosed.is_ok(),
        result_class,
    })
}

fn emit_join_reader_record(case: &str, instrument_control: bool, record: JoinReaderRecord) {
    let trace = JoinReaderTrace {
        case: case.to_owned(),
        invocation_nonce: std::env::var(JOIN_CHILD_TOKEN).expect("owned reader invocation nonce"),
        child_pid: std::process::id(),
        mode: std::env::var(JOIN_CHILD_MODE).expect("owned reader invocation mode"),
        instrument_control,
        clock: "child-case-start".to_owned(),
        record,
    };
    eprintln!("{JOIN_READER_TRACE_PREFIX}{}", serde_json::to_string(&trace)
        .expect("serialize credential-free reader boundary record"));
}

fn join_reader_ordered(values: &[Option<u64>]) -> bool {
    let mut previous = None;
    for value in values {
        let Some(value) = *value else { return false; };
        if previous.is_some_and(|previous| value < previous) {
            return false;
        }
        previous = Some(value);
    }
    true
}

fn join_reader_boundaries_ordered(observation: &JoinReaderObservationRecord) -> bool {
    let Some(call) = &observation.call else { return false; };
    observation.worker_returned
        && join_reader_ordered(&[
            call.entered_ns, call.returned_ns, call.send_entered_ns, call.send_completed_ns,
            call.diagnostic_entered_ns, call.diagnostic_returned_ns, observation.join_completed_ns,
        ])
        && join_reader_ordered(&[
            observation.wait_entered_ns, observation.wait_returned_ns, observation.join_completed_ns,
        ])
        // A receiver can return before the sender resumes after send(). Do not
        // impose a false send-completed-before-receive-returned ordering.
        && (observation.wait_outcome != JoinReaderReceiveOutcome::Received
            || (call.send_outcome == JoinReaderSendOutcome::Sent
                && join_reader_ordered(&[call.send_entered_ns, observation.wait_returned_ns])))
}

fn join_reader_successful_observation(observation: &JoinReaderObservationRecord) -> bool {
    join_reader_boundaries_ordered(observation)
        && observation.wait_outcome == JoinReaderReceiveOutcome::Received
        && observation.call.as_ref().is_some_and(|call|
            call.send_outcome == JoinReaderSendOutcome::Sent && call.diagnostic_succeeded)
}

fn join_reader_diagnostic_control_ordered(
    observation: &JoinReaderObservationRecord,
    control: &JoinReaderDiagnosticControlRecord,
) -> bool {
    let Some(call) = &observation.call else { return false; };
    join_reader_successful_observation(observation)
        && call.result_class == JoinReaderResultClass::Miss
        && control.ready_outcome == JoinReaderReceiveOutcome::Received
        && control.release_outcome == JoinReaderSendOutcome::Sent
        && join_reader_ordered(&[
            call.diagnostic_entered_ns, control.ready_returned_ns,
            observation.wait_entered_ns, observation.wait_returned_ns,
            control.release_entered_ns, control.release_completed_ns, observation.join_completed_ns,
        ])
        && join_reader_ordered(&[control.release_entered_ns, call.diagnostic_returned_ns])
}

fn join_reader_operation_control_ordered(
    observation: &JoinReaderObservationRecord,
    control: &JoinReaderOperationControlRecord,
) -> bool {
    let Some(call) = &observation.call else { return false; };
    join_reader_successful_observation(observation)
        && call.result_class == JoinReaderResultClass::Miss
        && control.ready_outcome == JoinReaderReceiveOutcome::Received
        && control.before_release == JoinReaderTryOutcome::Empty
        && control.release_outcome == JoinReaderSendOutcome::Sent
        && join_reader_ordered(&[
            call.entered_ns, control.ready_returned_ns,
            control.check_entered_ns, control.check_returned_ns,
            control.release_entered_ns, control.release_completed_ns,
            observation.wait_entered_ns, observation.wait_returned_ns, observation.join_completed_ns,
        ])
        // The operation may return before the releasing sender resumes.
        && join_reader_ordered(&[control.release_entered_ns, call.returned_ns])
}

fn collect_join_reader_records(case: &str, report: &JoinChildReport) -> Result<Vec<String>, String> {
    if !report.terminal || !report.ready || report.outcome != JoinChildOutcome::Completed {
        return Err("reader records require the same accepted terminal child receipt".into());
    }
    let ready = fs::read_to_string(report.root.join("ready"))
        .map_err(|_| "read retained reader ready receipt")?;
    let complete = fs::read_to_string(report.root.join("complete"))
        .map_err(|_| "read retained reader completion receipt")?;
    let identity = ready.split('\n').collect::<Vec<_>>();
    if ready != complete || identity.len() != 5 || identity[0].is_empty()
        || identity[2] != case || identity[3] != "actual" || !identity[4].is_empty()
    {
        return Err("reader records do not match the retained invocation receipts".into());
    }
    let pid: u32 = identity[1].parse().map_err(|_| "invalid retained reader child PID")?;
    let instrument_control = case != "join_storage_commit_refusal_never_publishes_payload";
    let stderr = fs::read_to_string(report.root.join("stderr"))
        .map_err(|_| "read terminal reader child stderr")?;
    let mut lines = Vec::new();
    let mut records = Vec::new();
    for line in stderr.lines() {
        let Some(raw) = line.strip_prefix(JOIN_READER_TRACE_PREFIX) else {
            if line.contains("DJC_READER_") {
                return Err("malformed reader trace prefix".into());
            }
            continue;
        };
        // Derived records reject unknown and duplicate object fields; the
        // required boundary values and outcomes are checked below.
        let trace: JoinReaderTrace = serde_json::from_str(raw)
            .map_err(|_| "malformed reader boundary record")?;
        if trace.case != case || trace.invocation_nonce != identity[0] || trace.child_pid != pid
            || trace.mode != "actual" || trace.instrument_control != instrument_control
            || trace.clock != "child-case-start"
        {
            return Err("reader record identity or kind differs from its accepted child".into());
        }
        lines.push(line.to_owned());
        records.push(trace.record);
    }
    if records.len() != 2 {
        return Err("missing or duplicate reader boundary/control records".into());
    }
    let valid = match (case, &records[0], &records[1]) {
        ("join_storage_commit_refusal_never_publishes_payload",
            JoinReaderRecord::Observation(observation), JoinReaderRecord::ReaderRelease(release)) =>
        {
            join_reader_successful_observation(observation)
                && observation.call.as_ref().is_some_and(|call|
                    call.result_class == JoinReaderResultClass::InternalError)
                && release.rollback_succeeded && release.close_succeeded == Some(true)
                && join_reader_ordered(&[
                    observation.join_completed_ns, release.entered_ns, release.rollback_returned_ns,
                    release.close_entered_ns, release.close_returned_ns,
                ])
        }
        ("join_reader_observation_notifies_before_diagnostic_release",
            JoinReaderRecord::Observation(observation), JoinReaderRecord::DiagnosticControl(control)) =>
        {
            join_reader_diagnostic_control_ordered(observation, control)
        }
        ("join_reader_observation_cannot_notify_before_operation_return",
            JoinReaderRecord::Observation(observation), JoinReaderRecord::OperationControl(control)) =>
        {
            join_reader_operation_control_ordered(observation, control)
        }
        _ => false,
    };
    if !valid {
        return Err("mismatched or inconsistent reader boundary/control records".into());
    }
    Ok(lines)
}

fn wait_for_uncommitted_write(
    journal: &Path,
    completion: &mpsc::Receiver<()>,
) -> Result<(), &'static str> {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if fs::metadata(journal).is_ok_and(|metadata| metadata.len() > 0) {
            return Ok(());
        }
        match completion.try_recv() {
            Err(mpsc::TryRecvError::Empty) => {}
            Ok(()) => return Err("owner returned before an actual journaled write was observed"),
            Err(mpsc::TryRecvError::Disconnected) => return Err("owner disconnected before an actual journaled write was observed"),
        }
        if Instant::now() >= deadline {
            return Err("no causal evidence of the blocked consumption write within 2s");
        }
        thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn join_storage_read_failure_is_distinct_and_recovers_without_consuming() {
    let mut fixture = Fixture::new("read-refusal");
    let code = fixture.register(&payload());
    let before = fixture.observe(&code);
    fixture.execute("ALTER TABLE device_pairing_join_codes RENAME COLUMN payload_json TO payload_obstacle");
    for _ in 0..2 {
        assert_storage_error(fixture.owner().redeem_device_pairing_join_code(&code).expect_err("read failure is not a miss"), &code);
        assert_eq!(fixture.observe_column(&code, "payload_obstacle"), before);
    }
    assert!(fixture.owner().redeem_device_pairing_join_code("invalid-code").expect("syntax miss before storage").is_none());
    fixture.reopen();
    assert_storage_error(fixture.owner().redeem_device_pairing_join_code(&code).expect_err("persisted obstacle still refuses"), &code);
    assert_eq!(fixture.observe_column(&code, "payload_obstacle"), before);
    fixture.execute("ALTER TABLE device_pairing_join_codes RENAME COLUMN payload_obstacle TO payload_json");
    assert_eq!(fixture.owner().redeem_device_pairing_join_code(&code).expect("explicit repair succeeds"), Some(payload()));
    let consumed = fixture.assert_consumed(&before);
    fixture.reopen();
    assert!(fixture.owner().redeem_device_pairing_join_code(&code).expect("consumed after reopen").is_none());
    assert_eq!(fixture.observe(&code), consumed);
}

#[test]
fn join_storage_write_failure_preserves_row_and_redacts_errors() {
    let mut fixture = Fixture::new("write-refusal");
    let code = fixture.register(&payload());
    let before = fixture.observe(&code);
    fixture.execute("CREATE TRIGGER refuse_join_consume BEFORE UPDATE OF redeemed_at_ms
        ON device_pairing_join_codes BEGIN SELECT RAISE(ABORT, 'djc-private-trigger-diagnostic'); END");
    for _ in 0..2 {
        assert_storage_error(fixture.owner().redeem_device_pairing_join_code(&code).expect_err("write abort refuses"), &code);
        assert_eq!(fixture.observe(&code), before);
    }
    fixture.reopen();
    assert_storage_error(fixture.owner().redeem_device_pairing_join_code(&code).expect_err("write obstacle survives reopen"), &code);
    assert_eq!(fixture.observe(&code), before);
    fixture.execute("DROP TRIGGER refuse_join_consume");
    assert_eq!(fixture.owner().redeem_device_pairing_join_code(&code).expect("repaired write"), Some(payload()));
    let consumed = fixture.assert_consumed(&before);
    fixture.reopen();
    assert_eq!(fixture.observe(&code), consumed);
    assert!(fixture.owner().redeem_device_pairing_join_code(&code).expect("one-shot after repair").is_none());
}

#[test]
fn join_reader_observation_notifies_before_diagnostic_release() {
    isolated_join_case("join_reader_observation_notifies_before_diagnostic_release", |_root| {
        const CASE: &str = "join_reader_observation_notifies_before_diagnostic_release";
        let started = Instant::now();
        thread::scope(|scope| {
            let (sender, receiver) = mpsc::channel();
            let (diagnostic_entered, diagnostic_ready) = mpsc::channel();
            let (release_diagnostic, diagnostic_release) = mpsc::channel();
            let worker = scope.spawn(move || observe_join_reader_call(
                started, sender, || Ok(None), move |_| {
                    diagnostic_entered.send(()).map_err(|_| "diagnostic readiness disconnected")?;
                    diagnostic_release.recv_timeout(Duration::from_secs(8))
                        .map_err(|_| "bounded diagnostic release failed")
                },
            ));
            let ready = diagnostic_ready.recv_timeout(Duration::from_secs(8));
            let ready_returned = Instant::now();
            let wait_entered = Instant::now();
            let waited = receiver.recv_timeout(Duration::from_secs(8));
            let wait_returned = Instant::now();
            // Always release/drop the gate and join before any assertion. A
            // failed observation cannot strand the bounded diagnostic receiver.
            let release_entered = Instant::now();
            let released = release_diagnostic.send(());
            let release_completed = Instant::now();
            drop(release_diagnostic);
            let joined = worker.join();
            let join_completed = Instant::now();
            let observation = JoinReaderObservationRecord {
                call: joined.as_ref().ok().map(|(_, call)| call.clone()),
                wait_entered_ns: join_reader_offset(started, wait_entered),
                wait_returned_ns: join_reader_offset(started, wait_returned),
                wait_outcome: join_reader_receive_outcome(&waited),
                join_completed_ns: join_reader_offset(started, join_completed),
                worker_returned: joined.is_ok(),
            };
            let control = JoinReaderDiagnosticControlRecord {
                ready_returned_ns: join_reader_offset(started, ready_returned),
                ready_outcome: join_reader_receive_outcome(&ready),
                release_entered_ns: join_reader_offset(started, release_entered),
                release_completed_ns: join_reader_offset(started, release_completed),
                release_outcome: join_reader_send_outcome(&released),
            };
            let ordered = join_reader_diagnostic_control_ordered(&observation, &control);
            emit_join_reader_record(CASE, true, JoinReaderRecord::Observation(observation));
            emit_join_reader_record(CASE, true, JoinReaderRecord::DiagnosticControl(control));
            let (result, _) = joined.expect("join bounded diagnostic observation control");
            assert!(matches!(result, Ok(None)), "observation control must return its sentinel");
            assert!(ordered, "completion must arrive while post-return diagnostics remain unreleased");
        });
    });
}

#[test]
fn join_reader_observation_cannot_notify_before_operation_return() {
    isolated_join_case("join_reader_observation_cannot_notify_before_operation_return", |_root| {
        const CASE: &str = "join_reader_observation_cannot_notify_before_operation_return";
        let started = Instant::now();
        thread::scope(|scope| {
            let (sender, receiver) = mpsc::channel();
            let (operation_entered, operation_ready) = mpsc::channel();
            let (release_operation, operation_release) = mpsc::channel();
            let worker = scope.spawn(move || observe_join_reader_call(
                started, sender, move || {
                    operation_entered.send(()).map_err(|_| GatewayCommandError::Internal(
                        "observation control operation readiness disconnected".into(),
                    ))?;
                    operation_release.recv_timeout(Duration::from_secs(8)).map_err(|_| GatewayCommandError::Internal(
                        "bounded observation control operation release failed".into(),
                    ))?;
                    Ok(None)
                }, |_| Ok(()),
            ));
            let ready = operation_ready.recv_timeout(Duration::from_secs(8));
            let ready_returned = Instant::now();
            let check_entered = Instant::now();
            let before_release = receiver.try_recv();
            let check_returned = Instant::now();
            // No assertion precedes release and retirement, even if an invalid
            // early notification was consumed by the negative observation.
            let release_entered = Instant::now();
            let released = release_operation.send(());
            let release_completed = Instant::now();
            drop(release_operation);
            let wait_entered = Instant::now();
            let waited = receiver.recv_timeout(Duration::from_secs(8));
            let wait_returned = Instant::now();
            let joined = worker.join();
            let join_completed = Instant::now();
            let observation = JoinReaderObservationRecord {
                call: joined.as_ref().ok().map(|(_, call)| call.clone()),
                wait_entered_ns: join_reader_offset(started, wait_entered),
                wait_returned_ns: join_reader_offset(started, wait_returned),
                wait_outcome: join_reader_receive_outcome(&waited),
                join_completed_ns: join_reader_offset(started, join_completed),
                worker_returned: joined.is_ok(),
            };
            let control = JoinReaderOperationControlRecord {
                ready_returned_ns: join_reader_offset(started, ready_returned),
                ready_outcome: join_reader_receive_outcome(&ready),
                check_entered_ns: join_reader_offset(started, check_entered),
                check_returned_ns: join_reader_offset(started, check_returned),
                before_release: match before_release {
                    Err(mpsc::TryRecvError::Empty) => JoinReaderTryOutcome::Empty,
                    Ok(()) => JoinReaderTryOutcome::Received,
                    Err(mpsc::TryRecvError::Disconnected) => JoinReaderTryOutcome::Disconnected,
                },
                release_entered_ns: join_reader_offset(started, release_entered),
                release_completed_ns: join_reader_offset(started, release_completed),
                release_outcome: join_reader_send_outcome(&released),
            };
            let ordered = join_reader_operation_control_ordered(&observation, &control);
            emit_join_reader_record(CASE, true, JoinReaderRecord::Observation(observation));
            emit_join_reader_record(CASE, true, JoinReaderRecord::OperationControl(control));
            let (result, _) = joined.expect("join bounded operation observation control");
            assert!(matches!(result, Ok(None)), "released observation control must return its sentinel");
            assert!(ordered, "an operation held before return cannot notify completion");
        });
    });
}

#[test]
fn join_storage_commit_refusal_never_publishes_payload() {
    isolated_join_case("join_storage_commit_refusal_never_publishes_payload", |root| {
        const CASE: &str = "join_storage_commit_refusal_never_publishes_payload";
        let started = Instant::now();
        let mut fixture = Fixture::in_owned_root(root.join("commit-refusal"));
        let code = fixture.register(&payload());
        let before = fixture.observe(&code);
        let blocker = fixture.read_fence();
        join_phase(started, "reader-fence-acquired");
        let journal = fixture.root.join("device-pairing.db-journal");
        assert!(!journal.exists(), "fresh committed fixture has no leftover journal");
        // The real SHARED reader permits the immediate transaction and journaled
        // write but prevents the EXCLUSIVE commit lock. rusqlite's five-second
        // busy timeout is not an operation-wide deadline. Keep the original 8s
        // observation failure, then collect the result under the same fence.
        let mut observation_failures = Vec::new();
        let refusal = thread::scope(|scope| {
            let (sender, receiver) = mpsc::channel();
            let owner = fixture.owner();
            let code = &code;
            let worker = scope.spawn(move || {
                join_phase(started, "owner-call-started");
                observe_join_reader_call(started, sender,
                    || owner.redeem_device_pairing_join_code(code),
                    |result| {
                        join_result_phase(started, result);
                        Ok(())
                    },
                )
            });
            match wait_for_uncommitted_write(&journal, &receiver) {
                Ok(()) => join_phase(started, "journaled-write-observed"),
                Err(failure) => {
                    join_phase(started, "journal-observation-failed");
                    observation_failures.push(failure);
                }
            }
            let wait_entered = Instant::now();
            let waited = receiver.recv_timeout(Duration::from_secs(8));
            let wait_returned = Instant::now();
            match &waited {
                Ok(()) => join_phase(started, "completion-notification-received"),
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    join_phase(started, "original-8s-observation-expired");
                    observation_failures.push("bounded actual commit refusal: original 8s observation timed out");
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    join_phase(started, "completion-notification-disconnected");
                    observation_failures.push("owner completion notification disconnected");
                }
            }
            // Joining does not release the reader fence. The parent process
            // owns the 120s diagnostic deadline and 5s stop/reap boundary.
            let joined = worker.join();
            let join_completed = Instant::now();
            let observation = JoinReaderObservationRecord {
                call: joined.as_ref().ok().map(|(_, call)| call.clone()),
                wait_entered_ns: join_reader_offset(started, wait_entered),
                wait_returned_ns: join_reader_offset(started, wait_returned),
                wait_outcome: join_reader_receive_outcome(&waited),
                join_completed_ns: join_reader_offset(started, join_completed),
                worker_returned: joined.is_ok(),
            };
            emit_join_reader_record(CASE, false, JoinReaderRecord::Observation(observation.clone()));
            let (result, call) = joined.expect("join the owned actual redemption worker");
            if !join_reader_boundaries_ordered(&observation) {
                observation_failures.push("missing or inconsistent reader completion boundaries");
            }
            if call.send_outcome != JoinReaderSendOutcome::Sent {
                observation_failures.push("owner completion send disconnected");
            }
            if !call.diagnostic_succeeded {
                observation_failures.push("owner post-return diagnostic failed");
            }
            result
        });
        let error = match refusal {
            Err(error) => error,
            Ok(None) => panic!("failed commit cannot become a miss"),
            Ok(Some(_)) => panic!("failed commit cannot publish a payload"),
        };
        assert_storage_error(error, &code);
        join_phase(started, "storage-error-status-and-redaction-checked");
        let release_entered = Instant::now();
        let rolled_back = blocker.execute_batch("ROLLBACK");
        let rollback_returned = Instant::now();
        if rolled_back.is_err() {
            emit_join_reader_record(CASE, false, JoinReaderRecord::ReaderRelease(JoinReaderReleaseRecord {
                entered_ns: join_reader_offset(started, release_entered),
                rollback_returned_ns: join_reader_offset(started, rollback_returned),
                rollback_succeeded: false,
                close_entered_ns: None, close_returned_ns: None, close_succeeded: None,
            }));
        }
        rolled_back.expect("release only fixture's read fence");
        let close_entered = Instant::now();
        let closed = blocker.close();
        let close_returned = Instant::now();
        emit_join_reader_record(CASE, false, JoinReaderRecord::ReaderRelease(JoinReaderReleaseRecord {
            entered_ns: join_reader_offset(started, release_entered),
            rollback_returned_ns: join_reader_offset(started, rollback_returned),
            rollback_succeeded: true,
            close_entered_ns: join_reader_offset(started, close_entered),
            close_returned_ns: join_reader_offset(started, close_returned),
            close_succeeded: Some(closed.is_ok()),
        }));
        closed.expect("close fixture reader");
        join_phase(started, "reader-fence-explicitly-released");
        assert_eq!(fixture.observe(&code), before, "known refused commit rolled back its marker");
        join_phase(started, "refused-row-preservation-checked");
        fixture.reopen();
        join_phase(started, "owner-reopened");
        assert_eq!(fixture.owner().redeem_device_pairing_join_code(&code).expect("retry only after explicit fixture unlock"), Some(payload()));
        join_phase(started, "fresh-redemption-after-unlock-checked");
        fixture.assert_consumed(&before);
        join_phase(started, "consumed-marker-checked");
        assert!(observation_failures.is_empty(),
            "DJC original observation failed despite later diagnostics: {observation_failures:?}");
    });
}

#[test]
fn join_deferred_commit_refusal_is_qualified_and_preserves_recovery() {
    isolated_join_case("join_deferred_commit_refusal_is_qualified_and_preserves_recovery", |root| {
        let started = Instant::now();
        let mut fixture = Fixture::in_owned_root(root.join("deferred-commit-refusal"));
        let code = fixture.register(&payload());
        let before = fixture.observe(&code);
        assert!(before.redeemed_at_ms.is_none());
        fixture.execute(
            "CREATE TABLE djc_deferred_parent (id INTEGER PRIMARY KEY);
             CREATE TABLE djc_deferred_child (
                 shortcode TEXT NOT NULL,
                 parent_id INTEGER NOT NULL REFERENCES djc_deferred_parent(id)
                     DEFERRABLE INITIALLY DEFERRED
             );
             CREATE TRIGGER djc_deferred_consume AFTER UPDATE OF redeemed_at_ms
             ON device_pairing_join_codes
             WHEN NEW.redeemed_at_ms IS NOT OLD.redeemed_at_ms
             BEGIN
                 INSERT INTO djc_deferred_child (shortcode, parent_id) VALUES (NEW.shortcode, 7);
             END;",
        );
        let assert_no_child = |fixture: &Fixture| {
            let connection = Connection::open_with_flags(&fixture.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
                .expect("open independent deferred-child observer");
            let children: i64 = connection.query_row(
                "SELECT COUNT(*) FROM djc_deferred_child", [], |row| row.get(0),
            ).expect("observe every committed fixture child");
            assert_eq!(children, 0, "refused commit must not retain its deferred child");
            connection.close().expect("close independent deferred-child observer");
        };
        assert_no_child(&fixture);

        // Qualify the same fixture through another connection in this exact
        // binary. Never enable foreign keys just for the control: the public
        // owner also relies on the locked bundled SQLite connection defaults.
        let qualifier = fixture.sql();
        assert!(qualifier.is_autocommit(), "qualification begins outside a transaction");
        assert_eq!(qualifier.query_row("PRAGMA foreign_keys", [], |row| row.get::<_, i64>(0))
            .expect("observe untouched foreign-key default"), 1);
        join_phase(started, "deferred-default-foreign-keys-qualified");
        qualifier.execute_batch("BEGIN IMMEDIATE").expect("qualification BEGIN IMMEDIATE succeeds");
        assert!(!qualifier.is_autocommit());
        let marker = now_ms();
        assert!(marker > 0);
        assert_eq!(qualifier.execute(
            "UPDATE device_pairing_join_codes SET redeemed_at_ms = ?1
             WHERE shortcode = ?2 AND redeemed_at_ms IS NULL",
            params![marker, &code],
        ).expect("qualification consumption UPDATE succeeds before COMMIT"), 1);
        let changed_marker: Option<i64> = qualifier.query_row(
            "SELECT redeemed_at_ms FROM device_pairing_join_codes WHERE shortcode = ?1",
            [&code], |row| row.get(0),
        ).expect("observe the actual uncommitted consumption marker");
        assert_eq!(changed_marker, Some(marker));
        let invalid_child: (String, i64, i64) = qualifier.query_row(
            "SELECT child.shortcode, child.parent_id,
                    (SELECT COUNT(*) FROM djc_deferred_parent WHERE id = child.parent_id)
             FROM djc_deferred_child AS child",
            [], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).expect("observe the actual deferred child and absent referenced parent");
        assert_eq!(invalid_child, (code.clone(), 7, 0));
        assert_eq!(qualifier.query_row(
            "SELECT COUNT(*) FROM djc_deferred_child", [], |row| row.get::<_, i64>(0),
        ).expect("observe one uncommitted deferred child"), 1);
        join_phase(started, "deferred-update-and-invalid-child-observed");
        let commit_error = qualifier.execute_batch("COMMIT").expect_err("only COMMIT refuses the qualified update");
        match commit_error {
            rusqlite::Error::SqliteFailure(error, _) => {
                assert_eq!(error.code, rusqlite::ErrorCode::ConstraintViolation);
                assert_eq!(error.extended_code, rusqlite::ffi::SQLITE_CONSTRAINT_FOREIGNKEY);
            }
            other => panic!("qualification requires exact SQLITE_CONSTRAINT_FOREIGNKEY: {other}"),
        }
        assert!(!qualifier.is_autocommit(), "refused deferred COMMIT leaves the transaction open");
        join_phase(started, "deferred-exact-commit-refusal-qualified");
        qualifier.execute_batch("ROLLBACK").expect("explicitly roll back only the qualification transaction");
        assert!(qualifier.is_autocommit(), "explicit qualification rollback completes");
        qualifier.close().expect("close qualification connection before public-owner calls");
        assert_eq!(fixture.observe(&code), before, "qualification rollback restores every target column");
        assert_no_child(&fixture);
        join_phase(started, "deferred-qualification-rollback-observed");

        let mut observation_failures = Vec::new();
        for attempt in ["initial", "repeat", "reopened"] {
            if attempt == "reopened" {
                fixture.reopen();
                assert_eq!(fixture.observe(&code), before);
                assert_no_child(&fixture);
            }
            eprintln!("DJC deferred public-owner attempt={attempt}");
            let refusal = thread::scope(|scope| {
                let (sender, receiver) = mpsc::channel();
                let owner = fixture.owner();
                let code = &code;
                let worker = scope.spawn(move || {
                    join_phase(started, "deferred-owner-call-started");
                    let result = owner.redeem_device_pairing_join_code(code);
                    join_result_phase(started, &result);
                    let _ = sender.send(());
                    result
                });
                match receiver.recv_timeout(Duration::from_secs(8)) {
                    Ok(()) => join_phase(started, "deferred-completion-notification-received"),
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        join_phase(started, "deferred-8s-observation-expired");
                        observation_failures.push("bounded deferred COMMIT refusal: 8s observation timed out");
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        observation_failures.push("deferred owner completion notification disconnected");
                    }
                }
                // Keep a late result as diagnostics without turning the 8s
                // observation into success. The unchanged parent owns the
                // 120s outer deadline and 5s stop/reap boundary.
                worker.join().expect("join the owned deferred-refusal redemption worker")
            });
            let error = match refusal {
                Err(error) => error,
                Ok(None) => panic!("deferred COMMIT refusal cannot become a miss"),
                Ok(Some(_)) => panic!("deferred COMMIT refusal cannot publish a payload"),
            };
            assert_storage_error(error, &code);
            assert_eq!(fixture.observe(&code), before, "public refusal preserves every target column");
            assert_no_child(&fixture);
            join_phase(started, "deferred-public-refusal-and-rollback-checked");
        }

        fixture.execute("DROP TRIGGER djc_deferred_consume");
        join_phase(started, "deferred-obstacle-explicitly-repaired");
        assert_eq!(fixture.owner().redeem_device_pairing_join_code(&code)
            .expect("ordinary redemption succeeds only after explicit trigger repair"), Some(payload()));
        let consumed = fixture.assert_consumed(&before);
        assert_no_child(&fixture);
        assert!(fixture.owner().redeem_device_pairing_join_code(&code)
            .expect("repaired redemption remains single use").is_none());
        assert_eq!(fixture.observe(&code), consumed);
        fixture.reopen();
        assert_eq!(fixture.observe(&code), consumed);
        assert_no_child(&fixture);
        assert!(fixture.owner().redeem_device_pairing_join_code(&code)
            .expect("repaired consumption survives reopen").is_none());
        assert_eq!(fixture.observe(&code), consumed);
        join_phase(started, "deferred-repair-payload-and-durable-single-use-checked");
        assert!(observation_failures.is_empty(),
            "DJC deferred refusal observation failed despite later diagnostics: {observation_failures:?}");
    });
}

#[test]
fn join_malformed_rows_are_consumed_durably_before_content_validation() {
    for fault in ["invalid-json", "invalid-utf8", "blob-payload", "oversize", "text-expiry", "real-expiry", "blob-expiry"] {
        let mut fixture = Fixture::new(fault);
        let code = fixture.register(&payload());
        let future = now_ms() + 600_000;
        let text_payload = SqlValue::Text(payload().to_string());
        match fault {
            "invalid-json" => fixture.set(&code, &SqlValue::Text(format!("{{\"token\":\"{PRIVATE_PAYLOAD}\"")), &SqlValue::Integer(future)),
            "invalid-utf8" => {
                let connection = fixture.sql();
                assert_eq!(connection.execute("UPDATE device_pairing_join_codes SET payload_json = CAST(x'FFFE' AS TEXT) WHERE shortcode = ?1", [&code])
                    .expect("inject raw invalid UTF-8 TEXT"), 1);
                connection.close().expect("close raw TEXT writer");
            }
            "blob-payload" => fixture.set(&code, &SqlValue::Blob(payload().to_string().into_bytes()), &SqlValue::Integer(future)),
            "oversize" => fixture.set(&code, &SqlValue::Text(json!("x".repeat(64 * 1024)).to_string()), &SqlValue::Integer(future)),
            "text-expiry" => fixture.set(&code, &text_payload, &SqlValue::Text("future-not-an-integer".into())),
            "real-expiry" => fixture.set(&code, &text_payload, &SqlValue::Real(future as f64 + 0.5)),
            "blob-expiry" => fixture.set(&code, &text_payload, &SqlValue::Blob(vec![1, 2, 3])),
            _ => unreachable!(),
        }
        let before = fixture.observe(&code);
        assert!(fixture.owner().redeem_device_pairing_join_code(&code).expect("malformed content is a committed miss").is_none(), "{fault}");
        let consumed = fixture.assert_consumed(&before);
        fixture.reopen();
        assert_eq!(fixture.observe(&code), consumed, "{fault}");
        assert!(fixture.owner().redeem_device_pairing_join_code(&code).expect("malformed code stays consumed").is_none());
        fixture.set(&code, &text_payload, &SqlValue::Integer(future));
        assert_eq!(fixture.observe(&code).redeemed_at_ms, consumed.redeemed_at_ms);
        assert!(fixture.owner().redeem_device_pairing_join_code(&code).expect("content repair cannot resurrect a burned code").is_none());
    }
}

#[test]
fn join_expiry_and_legacy_json_preserve_single_use_contract() {
    let mut fixture = Fixture::new("legacy-expiry");
    for expected in [payload(), json!(["legacy", true, null, 17]), json!("opaque"), json!(null), json!(7)] {
        let code = fixture.register(&expected);
        let before = fixture.observe(&code);
        assert!(fixture.owner().redeem_device_pairing_join_code("invalid-code").expect("syntax miss").is_none());
        assert_eq!(fixture.observe(&code), before);
        assert_eq!(fixture.owner().redeem_device_pairing_join_code(&format!(" {code} ")).expect("trimmed healthy redemption"), Some(expected));
        let consumed = fixture.assert_consumed(&before);
        fixture.reopen();
        assert!(fixture.owner().redeem_device_pairing_join_code(&code).expect("second redemption").is_none());
        assert_eq!(fixture.observe(&code), consumed);
    }
    let code = fixture.register(&payload());
    fixture.set(&code, &SqlValue::Text(payload().to_string()), &SqlValue::Integer(1));
    let expired = fixture.observe(&code);
    assert!(fixture.owner().redeem_device_pairing_join_code(&code).expect("expired miss").is_none());
    let consumed = fixture.assert_consumed(&expired);
    fixture.reopen();
    assert_eq!(fixture.observe(&code), consumed);
    let missing = if code == "AAAAAAAAAAAAAAAAAAAAAA" { "BBBBBBBBBBBBBBBBBBBBBB" } else { "AAAAAAAAAAAAAAAAAAAAAA" };
    assert!(fixture.owner().redeem_device_pairing_join_code(missing).expect("well-formed absent code").is_none());
    assert_eq!(fixture.observe(&code), consumed);
}

#[test]
fn join_expiry_is_rechecked_after_a_delayed_successful_commit() {
    isolated_join_case("join_expiry_is_rechecked_after_a_delayed_successful_commit", |root| {
        let started = Instant::now();
        let fixture = Fixture::in_owned_root(root.join("delayed-expiry"));
        let expires_at_ms = now_ms() + 3_000;
        let code = fixture.owner().register_device_pairing_join_code(&payload(), expires_at_ms).expect("register bounded future code");
        let before = fixture.observe(&code);
        let blocker = fixture.read_fence();
        join_phase(started, "reader-fence-acquired");
        let journal = fixture.root.join("device-pairing.db-journal");
        assert!(!journal.exists());
        let mut observation_failures = Vec::new();
        let result = thread::scope(|scope| {
            let (sender, receiver) = mpsc::channel();
            let owner = fixture.owner();
            let code = &code;
            let worker = scope.spawn(move || {
                join_phase(started, "owner-call-started");
                let result = owner.redeem_device_pairing_join_code(code);
                join_result_phase(started, &result);
                let _ = sender.send(());
                result
            });
            match wait_for_uncommitted_write(&journal, &receiver) {
                Ok(()) => join_phase(started, "journaled-write-observed"),
                Err(failure) => {
                    join_phase(started, "journal-observation-failed");
                    observation_failures.push(failure);
                }
            }
            if now_ms() >= expires_at_ms {
                observation_failures.push("write must be witnessed while the code is still live");
            }
            let deadline = Instant::now() + Duration::from_secs(4);
            while now_ms() <= expires_at_ms {
                if Instant::now() >= deadline {
                    observation_failures.push("fixture clock did not reach its declared expiry within 4s");
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            }
            if !matches!(receiver.try_recv(), Err(mpsc::TryRecvError::Empty)) {
                observation_failures.push("held reader must prevent early publication");
            }
            // A clock-observation failure never authorizes releasing the fence
            // before expiry. Preserve the actual worker result even on failure.
            if now_ms() > expires_at_ms {
                if blocker.execute_batch("ROLLBACK").is_ok() {
                    join_phase(started, "reader-fence-released-after-expiry");
                } else {
                    observation_failures.push("allow the actual pending commit after expiry: fixture rollback failed");
                }
            }
            match receiver.recv_timeout(Duration::from_secs(4)) {
                Ok(()) => join_phase(started, "completion-notification-received"),
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    join_phase(started, "original-4s-result-observation-expired");
                    observation_failures.push("bounded successful commit result: original 4s observation timed out");
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    observation_failures.push("owner completion notification disconnected");
                }
            }
            worker.join().expect("join the owned actual redemption worker")
        });
        blocker.close().expect("close expired-code reader fence");
        assert!(result.expect("released commit succeeds").is_none(), "expired payload must not escape after a delayed commit");
        join_phase(started, "expired-payload-withheld-checked");
        fixture.assert_consumed(&before);
        join_phase(started, "consumed-marker-checked");
        assert!(observation_failures.is_empty(),
            "DJC original delayed-expiry observation failed: {observation_failures:?}");
    });
}

#[test]
fn join_concurrent_file_owners_publish_one_payload() {
    let mut fixture = Fixture::new("concurrent-owners");
    let code = fixture.register(&payload());
    let before = fixture.observe(&code);
    let second = DevicePairingOwner::new(&fixture.db_path).expect("independent connection to the same existing owner store");
    let first_owner = fixture.owner();
    let start = Barrier::new(3);
    let results = thread::scope(|scope| {
        let first = scope.spawn(|| { start.wait(); first_owner.redeem_device_pairing_join_code(&code) });
        let other = scope.spawn(|| { start.wait(); second.redeem_device_pairing_join_code(&code) });
        start.wait();
        [first.join().expect("first owned thread"), other.join().expect("second owned thread")]
    });
    let values: Vec<Option<Value>> = results.into_iter().map(|result| result.expect("serialized actual SQLite redemption")).collect();
    assert_eq!(values.iter().filter(|value| value.is_some()).count(), 1);
    assert_eq!(values.into_iter().flatten().collect::<Vec<_>>(), vec![payload()]);
    drop(second);
    let consumed = fixture.assert_consumed(&before);
    fixture.reopen();
    assert!(fixture.owner().redeem_device_pairing_join_code(&code).expect("concurrent winner stays consumed").is_none());
    assert_eq!(fixture.observe(&code), consumed);
}

#[test]
fn join_gateway_reports_storage_failure_and_preserves_public_recovery() {
    let mut fixture = Fixture::new("gateway-recovery");
    let code = fixture.register(&payload());
    let before = fixture.observe(&code);
    fixture.start_gateway();
    fixture.execute("ALTER TABLE device_pairing_join_codes RENAME COLUMN payload_json TO payload_obstacle");
    for _ in 0..2 {
        let response = fixture.get(&format!("/j/{code}"));
        assert_eq!(assert_http(&response, "HTTP/1.1 500 Internal Server Error"), json!({"error": STORAGE_ERROR}));
        assert!(!response.contains(&code));
        assert!(!response.contains(PRIVATE_PAYLOAD));
        assert_eq!(fixture.observe_column(&code, "payload_obstacle"), before);
    }
    fixture.reopen();
    fixture.execute("ALTER TABLE device_pairing_join_codes RENAME COLUMN payload_obstacle TO payload_json");
    fixture.start_gateway();
    for path in [format!("/j/{code}?query=refused"), "/j".into(), "/j/invalid".into()] {
        assert_eq!(assert_http(&fixture.get(&path), "HTTP/1.1 404 Not Found"), json!({"error": "not_found"}));
        assert_eq!(fixture.observe(&code), before, "route refusal cannot consume a healthy target");
    }
    let malformed = fixture.register(&payload());
    fixture.set(&malformed, &SqlValue::Blob(PRIVATE_PAYLOAD.as_bytes().to_vec()), &SqlValue::Integer(now_ms() + 600_000));
    let malformed_before = fixture.observe(&malformed);
    let miss = fixture.get(&format!("/j/{malformed}"));
    assert_eq!(assert_http(&miss, "HTTP/1.1 404 Not Found"), json!({"error": "not_found"}));
    assert!(!miss.contains(PRIVATE_PAYLOAD));
    fixture.assert_consumed(&malformed_before);
    assert_eq!(assert_http(&fixture.get(&format!("/gateway/j/{code}")), "HTTP/1.1 200 OK"), payload());
    let consumed = fixture.assert_consumed(&before);
    assert_eq!(assert_http(&fixture.get(&format!("/j/{code}")), "HTTP/1.1 404 Not Found"), json!({"error": "not_found"}));
    fixture.reopen();
    assert_eq!(fixture.observe(&code), consumed);
    assert!(fixture.owner().redeem_device_pairing_join_code(&code).expect("HTTP consumption survives reopen").is_none());
}
