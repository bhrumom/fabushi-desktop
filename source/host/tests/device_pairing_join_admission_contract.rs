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
}

impl Fixture {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!("fabushi-join-{label}-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).expect("exclusively create owned fixture directory");
        let db_path = root.join("device-pairing.db");
        let owner = DevicePairingOwner::new(&db_path).expect("open durable pairing owner");
        Self { root, db_path, owner: Some(Arc::new(owner)), gateway: None }
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
        fs::remove_dir_all(&self.root).expect("remove only this owned fixture after handles close");
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

fn wait_for_uncommitted_write(
    journal: &Path,
    result: &mpsc::Receiver<Result<Option<Value>, GatewayCommandError>>,
) {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if fs::metadata(journal).is_ok_and(|metadata| metadata.len() > 0) {
            return;
        }
        assert!(matches!(result.try_recv(), Err(mpsc::TryRecvError::Empty)),
            "owner returned or disconnected before an actual journaled write was observed");
        assert!(Instant::now() < deadline, "no causal evidence of the blocked consumption write");
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
fn join_storage_commit_refusal_never_publishes_payload() {
    let mut fixture = Fixture::new("commit-refusal");
    let code = fixture.register(&payload());
    let before = fixture.observe(&code);
    let blocker = fixture.read_fence();
    let journal = fixture.root.join("device-pairing.db-journal");
    assert!(!journal.exists(), "fresh committed fixture has no leftover journal");
    // SQLite's SHARED reader permits the owner's immediate transaction and
    // journaled write, but refuses the EXCLUSIVE lock needed to commit. The
    // locked rusqlite 0.39.0 connection has a five-second busy timeout.
    let refusal = thread::scope(|scope| {
        let (sender, receiver) = mpsc::channel();
        let owner = fixture.owner();
        let code = &code;
        scope.spawn(move || sender.send(owner.redeem_device_pairing_join_code(code)).expect("deliver owned result"));
        wait_for_uncommitted_write(&journal, &receiver);
        receiver.recv_timeout(Duration::from_secs(8)).expect("bounded actual commit refusal")
    });
    assert_storage_error(refusal.expect_err("failed commit cannot publish a payload"), &code);
    blocker.execute_batch("ROLLBACK").expect("release only fixture's read fence");
    blocker.close().expect("close fixture reader");
    assert_eq!(fixture.observe(&code), before, "known refused commit rolled back its marker");
    fixture.reopen();
    assert_eq!(fixture.owner().redeem_device_pairing_join_code(&code).expect("retry only after explicit fixture unlock"), Some(payload()));
    fixture.assert_consumed(&before);
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
    let fixture = Fixture::new("delayed-expiry");
    let expires_at_ms = now_ms() + 3_000;
    let code = fixture.owner().register_device_pairing_join_code(&payload(), expires_at_ms).expect("register bounded future code");
    let before = fixture.observe(&code);
    let blocker = fixture.read_fence();
    let journal = fixture.root.join("device-pairing.db-journal");
    assert!(!journal.exists());
    let result = thread::scope(|scope| {
        let (sender, receiver) = mpsc::channel();
        let owner = fixture.owner();
        let code = &code;
        scope.spawn(move || sender.send(owner.redeem_device_pairing_join_code(code)).expect("deliver owned result"));
        wait_for_uncommitted_write(&journal, &receiver);
        assert!(now_ms() < expires_at_ms, "write must be witnessed while the code is still live");
        let deadline = Instant::now() + Duration::from_secs(4);
        while now_ms() <= expires_at_ms {
            assert!(Instant::now() < deadline, "fixture clock did not reach its declared expiry");
            thread::sleep(Duration::from_millis(10));
        }
        assert!(matches!(receiver.try_recv(), Err(mpsc::TryRecvError::Empty)), "held reader must prevent early publication");
        blocker.execute_batch("ROLLBACK").expect("allow the actual pending commit after expiry");
        receiver.recv_timeout(Duration::from_secs(4)).expect("bounded successful commit result")
    });
    blocker.close().expect("close expired-code reader fence");
    assert!(result.expect("released commit succeeds").is_none(), "expired payload must not escape after a delayed commit");
    fixture.assert_consumed(&before);
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
