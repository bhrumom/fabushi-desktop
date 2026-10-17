//! Actions-only production-owner regression, using public synthetic credentials.
//! This is not signed-client, network authentication or factory acceptance.
#![cfg(unix)]

use mahayana_app_host::{AppHost, AppHostFeatureMode, HostRequest};
use serde_json::{Value, json};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{FileTypeExt, OpenOptionsExt, PermissionsExt};
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
        "deviceId": "public-fixture-device"
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
    result
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
    if mode == "fifo-tripwire" {
        fs::create_dir_all(root.join("app-host/feature-host/runtime/workspace"))
            .expect("create product-owned fallback workspace");
        let result = AppHost::new_with_feature_mode_and_storage_passphrase(
            root.join("app-host"),
            AppHostFeatureMode::Production,
            Some("public-handoff-unit-storage-passphrase".into()),
        );
        match result {
            Err(error) => assert!(
                error
                    .to_string()
                    .contains("CI account session path must be a regular non-symlink file"),
                "FIFO session path must fail closed with a path-validation error, got: {error}"
            ),
            Ok(_) => panic!("production AppHost accepted a FIFO CI session path"),
        }
        return;
    }
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
}

fn run_child(root: &Path, mode: &str) {
    let issuer = if mode == "production" {
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
            if mode == "production" {
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
            root.join(if mode == "fifo-tripwire" {
                "no-reader.pipe"
            } else {
                "ci-session.json"
            }),
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
        assert!(matches!(
            mode.as_str(),
            "isolated" | "production" | "isolated-return" | "fifo-tripwire"
        ));
        let root = PathBuf::from(std::env::var_os(TEST_ROOT).expect("owned test directory"));
        child_case(&root, &mode);
        return;
    }

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = TestDirectory(std::env::temp_dir().join(format!(
        "native-app-host-handoff-{}-{nonce}",
        std::process::id()
    )));
    fs::create_dir(&root.0).expect("new unique test directory");
    fs::create_dir(root.0.join("tmp")).expect("test temporary directory");
    // No writer exists. A second bootstrap reader would block and fail the
    // bounded subprocess deadline rather than quietly consuming a disk session.
    let fifo = root.0.join("no-reader.pipe");
    assert!(
        Command::new("/usr/bin/mkfifo")
            .env_clear()
            .args(["-m", "600"])
            .arg(&fifo)
            .status()
            .expect("create test FIFO")
            .success()
    );
    let fifo_metadata = fs::symlink_metadata(&fifo).expect("test FIFO metadata");
    assert!(fifo_metadata.file_type().is_fifo());
    assert_eq!(fifo_metadata.permissions().mode() & 0o777, 0o600);
    // The production Host validates the configured session path as a regular
    // non-symlink file. Keep the dead FIFO as a separate tripwire so an
    // accidental legacy reader cannot block startup.
    let ci_session = root.0.join("ci-session.json");
    let ci_session_sentinel = b"public-ci-session-must-not-be-consumed";
    let mut ci_session_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&ci_session)
        .expect("create private regular CI session sentinel");
    ci_session_file
        .write_all(ci_session_sentinel)
        .expect("write CI session sentinel");
    drop(ci_session_file);
    let ci_session_metadata =
        fs::symlink_metadata(&ci_session).expect("regular CI session metadata");
    assert!(ci_session_metadata.file_type().is_file());
    assert_eq!(ci_session_metadata.permissions().mode() & 0o777, 0o600);
    for mode in ["isolated", "production", "isolated-return", "fifo-tripwire"] {
        let started = Instant::now();
        run_child(&root.0, mode);
        if mode == "fifo-tripwire" {
            assert!(
                started.elapsed() < Duration::from_secs(5),
                "production AppHost must fail closed on the FIFO immediately, not wait for the no-reader timeout"
            );
        }
        assert_eq!(
            fs::read(&ci_session).expect("CI session sentinel remains"),
            ci_session_sentinel,
            "Native credential restoration must not consume the CI session file"
        );
    }
    let final_fifo_metadata = fs::symlink_metadata(&fifo).expect("FIFO remains present");
    assert!(final_fifo_metadata.file_type().is_fifo());
    assert_eq!(final_fifo_metadata.permissions().mode() & 0o777, 0o600);
}
