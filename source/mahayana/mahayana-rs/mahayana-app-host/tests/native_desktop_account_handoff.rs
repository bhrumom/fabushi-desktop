//! Actions-only production-owner regression, using public synthetic credentials.
//! This is not signed-client, network authentication or factory acceptance.
#![cfg(unix)]

use base64::Engine as _;
use fabushi_telegram_storage::{EncryptedSqliteStore, StorageKey};
use mahayana_app_host::{AppHost, AppHostFeatureMode, HostRequest};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{FileTypeExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const CHILD_MODE: &str = "MFA_HANDOFF_CHILD_MODE";
const TEST_ROOT: &str = "MFA_HANDOFF_TEST_ROOT";
const TEST_NAME: &str = "native_desktop_account_handoff";
const ISOLATED: &str = "https://fabushi-ci-isolated-auth.bhrumom.workers.dev";
const PRODUCTION: &str = "https://api.ombhrum.com";
const PUBLIC_ACCESS: &str = "public-unit-fixture-not-an-authentic-access-token-00000000";
const BOT_A: &str = "Public native fixture A";
const BOT_B: &str = "Public native fixture B";
const BOT_PRODUCTION: &str = "Public production fixture";
const BOT_RETAINED: &str = "Public retained Native Agent";
const LEGACY_KEY: [u8; 32] = [11; 32];
const LEGACY_METADATA: &[u8] = b"public retained encrypted metadata; no real account";

struct TestDirectory(PathBuf);

impl Drop for TestDirectory {
    fn drop(&mut self) {
        // Only the unique directory created by this test is removable.
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn replace_native_credential(path: &Path, issuer: &str, user: &str) -> Vec<u8> {
    let expires = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_millis() as u64
        + 10 * 60 * 1000;
    let bytes = serde_json::to_vec(&json!({
        "version": 1,
        "issuer": issuer,
        "accessToken": PUBLIC_ACCESS,
        "expiresAtMs": expires,
        "userId": user,
        "sessionId": format!("public-fixture-session-{user}"),
        "deviceId": "public-fixture-device",
        // An already-written v1 descriptor can outlive the Electron upgrade.
        // Its optional legacy material must never instantiate a second owner.
        "telegramStorageKey": base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(LEGACY_KEY),
        "telegramApiId": 12345,
        "telegramApiHash": "0123456789abcdef0123456789abcdef"
    }))
    .expect("serialize public fixture");
    let next = path.with_extension("next");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&next)
        .expect("create private Native credential fixture");
    file.write_all(&bytes).expect("write public Native fixture");
    drop(file);
    fs::rename(next, path).expect("replace Native-owned fixture");
    bytes
}

fn auth(host: &AppHost) -> Value {
    let response = host.dispatch(HostRequest {
        id: None,
        method: "feature.auth.status".into(),
        params: json!({}),
    });
    assert!(response.ok, "production AppHost account projection failed");
    let result = response.result.expect("account result");
    let encoded = result.to_string();
    assert!(
        !encoded.contains(PUBLIC_ACCESS),
        "UI projection leaked credential"
    );
    assert!(!encoded.contains("accessToken"));
    assert!(!encoded.contains("refreshToken"));
    assert!(!encoded.contains("telegramStorageKey"));
    assert!(!encoded.contains("telegramApiHash"));
    result
}

fn assert_legacy_unavailable(host: &AppHost) {
    // A registered-but-unauthorized provider produces a different error. This
    // exercises the actual production registry through the public command.
    let response = host.dispatch(HostRequest {
        id: None,
        method: "feature.execute".into(),
        params: json!({"command": {
            "type": "conversation.open",
            "requestId": "legacy-provider-absence",
            "conversationId": "telegram:chat:42"
        }}),
    });
    assert!(!response.ok);
    assert!(
        response.error.as_deref().unwrap_or("").contains("provider is unavailable: telegram"),
        "must reject at the missing provider boundary: {:?}", response.error
    );
    for method in [
        "telegramStatus", "telegramSendCode", "telegramSignIn", "telegramPassword",
        "telegramRegistration", "telegramResendCode", "telegramCancelCode",
        "telegramRequestPasswordRecovery", "telegramCheckRecoveryPassword",
        "telegramSendVerifyEmailCode", "telegramVerifyEmail", "telegramRefreshPasswordChallenge",
    ] {
        let response = host.dispatch(HostRequest {
            id: None,
            method: format!("feature.auth.{method}"),
            params: json!({
                "phoneNumber": "+10000000000", "code": "public-fixture-code",
                "password": "public-fixture-password", "firstName": "Public",
                "email": "public-fixture@example.invalid"
            }),
        });
        assert!(!response.ok, "legacy command cannot activate a client: {method}");
        assert_eq!(response.error.as_deref(), Some(
            "host operation failed: embedded Telegram client is unavailable for the current platform account"
        ));
    }
    assert!(host.take_startup_native_shell_event().expect("startup event").is_none());
}

fn gateway(host: &AppHost, method: &str, args: Value) -> Value {
    host.grok_gateway_call(method, args)
        .expect("production FeatureHost gateway call")
        .expect("method must be owned by production FeatureHost")
}

fn count(host: &AppHost) -> u64 {
    gateway(host, "countAgents", json!({}))
        .as_u64()
        .expect("countAgents returns a number")
}

fn contains_bot(host: &AppHost, name: &str) -> bool {
    gateway(host, "listAgents", json!({}))
        .as_array()
        .expect("listAgents array")
        .iter()
        .any(|agent| agent.get("name").and_then(Value::as_str) == Some(name))
}

fn create(host: &AppHost, name: &str) {
    let before = count(host);
    let result = gateway(
        host,
        "createAgent",
        json!({
            "name": name,
            "description": "Public integration fixture; no inference requested"
        }),
    );
    assert!(result["agent"]["id"].as_str().is_some());
    assert_eq!(count(host), before + 1);
    assert!(contains_bot(host, name));
    assert!(!result.to_string().contains(PUBLIC_ACCESS));
}

fn production_host(root: &Path) -> AppHost {
    // Match source/host/app's ensure_managed_runtime_layout composition exactly.
    fs::create_dir_all(root.join("app-host/feature-host/runtime/workspace"))
        .expect("create product-owned fallback workspace");
    // Existing production constructor; the public storage passphrase avoids
    // touching an OS account secret store. It grants no authentication bypass.
    AppHost::new_with_feature_mode_and_storage_passphrase(
        root.join("app-host"),
        AppHostFeatureMode::Production,
        Some("public-handoff-unit-storage-passphrase".into()),
    )
    .expect("production AppHost must restore Native credentials without consuming CI FIFO")
}

fn expected_isolated_id(user: &str) -> String {
    // Independent standard SHA-256 oracle available on the Linux/macOS Actions
    // images; no product test seam, new Cargo dependency or network request.
    let mut digest = Command::new("/usr/bin/openssl")
        .env_clear()
        .args(["dgst", "-sha256"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("standard SHA-256 oracle");
    let message = format!("fabushi-native-auth-id-v1\0{ISOLATED}\0{user}");
    digest
        .stdin
        .take()
        .expect("digest input")
        .write_all(message.as_bytes())
        .expect("hash public account identity");
    let output = digest.wait_with_output().expect("SHA-256 result");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("SHA-256 output encoding");
    let hash = text.split_whitespace().last().expect("SHA-256 hex");
    assert_eq!(hash.len(), 64);
    assert!(
        hash.bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    );
    format!("fabushi-ci-isolated:v1:{hash}")
}

fn child_case(root: &Path, mode: &str) {
    let credential = root.join("native-credential.json");
    let issuer = if mode == "production" {
        PRODUCTION
    } else {
        ISOLATED
    };
    let mut expected_credential = replace_native_credential(&credential, issuer, "123");
    let host = production_host(root);
    let account = host.mini_app_account_port();
    let status = auth(&host);
    assert_eq!(status["loggedIn"], true);
    assert_legacy_unavailable(&host);
    let first = account.current_scope().expect("Native account scope");

    if mode == "production" {
        assert_eq!(
            first.account_id, "123",
            "production identity must remain raw"
        );
        assert!(
            !contains_bot(&host, BOT_A),
            "same raw ID must not alias isolated A"
        );
        assert!(!contains_bot(&host, BOT_B));
        create(&host, BOT_PRODUCTION);
    } else {
        assert_eq!(first.account_id, expected_isolated_id("123"));
        assert!(!contains_bot(&host, BOT_PRODUCTION));
        if mode == "isolated-return" {
            assert!(
                contains_bot(&host, BOT_A),
                "A must survive another process and issuer"
            );
            assert!(!contains_bot(&host, BOT_B));
        } else {
            let baseline = count(&host);
            create(&host, BOT_A);
            assert_eq!(
                fs::read(&credential).expect("read fixture"),
                expected_credential
            );

            let b_credential = replace_native_credential(&credential, ISOLATED, "456");
            assert_eq!(auth(&host)["loggedIn"], true);
            assert_legacy_unavailable(&host);
            let second = account.current_scope().expect("replacement B scope");
            assert_ne!(first.account_id, second.account_id);
            assert_eq!(second.account_id, expected_isolated_id("456"));
            assert!(second.generation > first.generation);
            assert_eq!(count(&host), baseline, "B must not inherit A's roster");
            assert!(!contains_bot(&host, BOT_A));
            create(&host, BOT_B);
            assert_eq!(fs::read(&credential).expect("read B fixture"), b_credential);

            let returned_credential = replace_native_credential(&credential, ISOLATED, "123");
            assert_eq!(auth(&host)["loggedIn"], true);
            assert_legacy_unavailable(&host);
            let returned = account.current_scope().expect("returned A scope");
            assert_eq!(returned.account_id, first.account_id);
            assert!(returned.generation > second.generation);
            assert_eq!(count(&host), baseline + 1);
            assert!(contains_bot(&host, BOT_A));
            assert!(!contains_bot(&host, BOT_B));
            assert_eq!(
                fs::read(&credential).expect("read returned A fixture"),
                returned_credential
            );

            // Simulate the existing Native owner's logout, not Rust auth mutation.
            fs::remove_file(&credential).expect("Native owner removes credential");
            for _ in 0..2 {
                assert_eq!(
                    auth(&host)["loggedIn"],
                    false,
                    "removed credential must not be cached"
                );
                assert!(
                    account.current_scope().is_err(),
                    "logged-out scope must be denied"
                );
            }
            assert!(!contains_bot(&host, BOT_A));
            assert!(!contains_bot(&host, BOT_B));

            expected_credential = replace_native_credential(&credential, ISOLATED, "123");
            assert_eq!(auth(&host)["loggedIn"], true);
            assert_eq!(
                account
                    .current_scope()
                    .expect("fresh Native hand-off")
                    .account_id,
                first.account_id
            );
            assert!(contains_bot(&host, BOT_A));
            assert!(!contains_bot(&host, BOT_B));
        }
    }
    // Read-only restoration must not mutate the Native owner's file or import
    // the public fixture into the legacy Rust account-session persistence path.
    let current = fs::read(&credential).expect("Native fixture remains");
    assert_eq!(current, expected_credential);
    assert!(
        !root
            .join("app-host/feature-host/account-session.json")
            .exists()
    );
    assert!(
        !root.join("app-host/feature-host/telegram").exists(),
        "Native startup/account changes must not create a legacy directory"
    );
}

fn legacy_child_case(root: &Path, restart: bool) {
    let credential = root.join("native-credential.json");
    let expected_credential = replace_native_credential(&credential, PRODUCTION, "123");
    let host = production_host(root);
    assert_eq!(auth(&host)["loggedIn"], true);
    assert_eq!(host.mini_app_account_port().current_scope().expect("Native scope").account_id, "123");
    assert_legacy_unavailable(&host);
    if restart {
        assert!(contains_bot(&host, BOT_RETAINED), "Native Agent must survive process restart");
        let settings = host.dispatch(HostRequest {
            id: None, method: "settings.snapshot".into(), params: json!({}),
        });
        assert!(settings.ok);
        assert_eq!(settings.result.expect("retained settings")["network"]["proxy"]["mode"], "enabled");
    } else {
        assert!(!contains_bot(&host, BOT_RETAINED));
        create(&host, BOT_RETAINED);
        let changed = host.dispatch(HostRequest {
            id: None, method: "settings.setNetworkProxyEnabled".into(),
            params: json!({"proxyEnabled": true}),
        });
        assert!(changed.ok, "existing Settings owner must remain usable");
        assert_eq!(changed.result.expect("settings result")["publication"]["available"], false);
    }
    assert_eq!(fs::read(credential).expect("retained Native credential"), expected_credential);
    assert!(!root.join("app-host/feature-host/account-session.json").exists());
    drop(host);
}

fn run_child(root: &Path, mode: &str) {
    let issuer = if mode == "production" || mode.starts_with("legacy-") {
        PRODUCTION
    } else {
        ISOLATED
    };
    let mut child = Command::new(std::env::current_exe().expect("test executable"))
        .args([TEST_NAME, "--exact", "--nocapture"])
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .env(CHILD_MODE, mode)
        .env(TEST_ROOT, root)
        .env("GITHUB_ACTIONS", "true")
        .env("FABUSHI_PRODUCT_MODE", "1")
        .env("FABUSHI_API_BASE_URL", issuer)
        .env(
            "FABUSHI_ACCOUNT_ENVIRONMENT",
            if issuer == PRODUCTION {
                "production"
            } else {
                "ci-isolated"
            },
        )
        .env(
            "FABUSHI_HOST_ACCESS_CREDENTIAL_FILE",
            root.join("native-credential.json"),
        )
        .env(
            "FABUSHI_CI_ACCOUNT_SESSION_FILE",
            root.join("no-reader.pipe"),
        )
        .env("FABUSHI_CI_SESSION_DELIVERY", "private-fifo")
        .env("FABUSHI_APP_DATA", root.join("runtime"))
        .env("SAND_DATA_ROOT", root.join("data"))
        .env("TMPDIR", root.join("tmp"))
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn isolated integration-test process");
    let deadline = Instant::now() + Duration::from_secs(45);
    loop {
        if let Some(status) = child.try_wait().expect("poll test child") {
            assert!(status.success(), "Native AppHost child case failed: {mode}");
            return;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("Native AppHost child stalled: {mode}; CI FIFO has no writer");
        }
        thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn native_desktop_account_handoff() {
    if let Ok(mode) = std::env::var(CHILD_MODE) {
        let root = PathBuf::from(std::env::var_os(TEST_ROOT).expect("owned test directory"));
        if matches!(mode.as_str(), "legacy-first" | "legacy-restart") {
            legacy_child_case(&root, mode == "legacy-restart");
            return;
        }
        assert!(matches!(
            mode.as_str(),
            "isolated" | "production" | "isolated-return"
        ));
        child_case(&root, &mode);
        return;
    }

    let root = test_directory("handoff");
    for mode in ["isolated", "production", "isolated-return"] {
        run_child(&root.0, mode);
    }
}

fn test_directory(label: &str) -> TestDirectory {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = TestDirectory(std::env::temp_dir().join(format!(
        "native-app-host-{label}-{}-{nonce}",
        std::process::id()
    )));
    fs::create_dir(&root.0).expect("new unique test directory");
    fs::create_dir(root.0.join("tmp")).expect("test temporary directory");
    // No writer exists. A second bootstrap reader would block and fail the
    // bounded subprocess deadline rather than quietly consuming a disk session.
    assert!(
        Command::new("/usr/bin/mkfifo")
            .env_clear()
            .args(["-m", "600"])
            .arg(root.0.join("no-reader.pipe"))
            .status()
            .expect("create test FIFO")
            .success()
    );
    assert!(
        fs::symlink_metadata(root.0.join("no-reader.pipe"))
            .expect("test FIFO metadata")
            .file_type()
            .is_fifo()
    );
    root
}

fn retained_files(directory: &Path) -> BTreeMap<String, Vec<u8>> {
    fs::read_dir(directory).expect("retained directory")
        .map(|entry| {
            let entry = entry.expect("retained file entry");
            let name = entry.file_name().into_string().expect("fixture filename");
            (name, fs::read(entry.path()).expect("read retained bytes"))
        })
        .collect()
}

fn retained_legacy_database_path(directory: &Path, account_id: &str) -> PathBuf {
    // The old production namespace is retained as a rollback fixture, not as
    // a Native startup path. Two account files below must survive unchanged.
    let mut hasher = Sha256::new();
    hasher.update(b"fabushi-telegram-account-v1\0");
    hasher.update(account_id.as_bytes());
    directory.join(format!("{:x}.sqlite3", hasher.finalize()))
}

fn retained_legacy_case(corrupt: bool) {
    let root = test_directory(if corrupt { "legacy-corrupt" } else { "legacy-encrypted" });
    let directory = root.0.join("app-host/feature-host/telegram");
    fs::create_dir_all(&directory).expect("create legacy fixture directory");
    let database = retained_legacy_database_path(&directory, "123");
    if corrupt {
        fs::write(&database, b"public corrupt legacy SQLite fixture\0\xff").expect("corrupt fixture");
        // Sidecars and unknown files are retained, never cleaned at startup.
        fs::write(database.with_extension("sqlite3-wal"), b"public retained WAL").expect("WAL fixture");
        fs::write(database.with_extension("sqlite3-shm"), b"public retained SHM").expect("SHM fixture");
    } else {
        let mut store = EncryptedSqliteStore::open(
            &database, StorageKey::from_slice(&LEGACY_KEY).expect("fixture key"),
        ).expect("create real legacy encrypted SQLite");
        store.commit_transition(&Default::default(), &[], 0, 1000).expect("encrypted snapshot");
        store.save_private_metadata("retirement.fixture.v1", LEGACY_METADATA, 1000).expect("encrypted metadata");
        drop(store);
    }
    fs::write(retained_legacy_database_path(&directory, "456"), b"public retained other-account database").expect("other account fixture");
    fs::write(directory.join("preserved-user-data.bin"), b"public unclassified legacy data").expect("retained file");
    let before = retained_files(&directory);
    for mode in ["legacy-first", "legacy-restart"] {
        run_child(&root.0, mode);
        assert!(retained_files(&directory) == before, "Native process must leave all legacy bytes unchanged");
    }
    if !corrupt {
        // Rollback compatibility is checked with the original storage reader,
        // only after both Native processes have terminated.
        let store = EncryptedSqliteStore::open(
            &database, StorageKey::from_slice(&LEGACY_KEY).expect("original fixture key"),
        ).expect("reopen with original reader");
        assert_eq!(store.load_private_metadata("retirement.fixture.v1").expect("decrypt metadata"), Some(LEGACY_METADATA.to_vec()));
        let snapshot = store.load_snapshot().expect("decrypt snapshot").expect("retained snapshot");
        assert_eq!(snapshot.revision, 1);
        assert_eq!(snapshot.updated_at_unix_ms, 1000);
    }
}

#[test]
fn native_startup_preserves_encrypted_legacy_storage_for_rollback() {
    retained_legacy_case(false);
}

#[test]
fn native_startup_ignores_corrupt_legacy_database_and_preserves_sidecars() {
    retained_legacy_case(true);
}

