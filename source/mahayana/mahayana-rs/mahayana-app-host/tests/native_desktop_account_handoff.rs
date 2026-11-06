//! Actions-only production-owner regression, using public synthetic credentials.
//! This is not signed-client, network authentication or factory acceptance.
#![cfg(unix)]

use base64::Engine as _;
use fabushi_telegram_storage::{EncryptedSqliteStore, StorageKey};
use mahayana_app_host::{AppHost, AppHostFeatureMode, HostRequest, PlatformRequestHost};
use mahayana_host::{HostCreateConfig, MahayanaHost};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::net::TcpListener;
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
const BOT_MESSAGING_A: &str = "Public messaging boundary Agent A";
const BOT_MESSAGING_B: &str = "Public messaging boundary Agent B";
const LEGACY_AUTOMATION_ID: &str = "retained-legacy-normal";
const LEGACY_AUTOMATION_NAME: &str = "Public retained normal legacy automation";
const UPSERT_COLLISION_ID: &str = "retained-upsert-collision";
const LEGACY_KEY: [u8; 32] = [11; 32];
const LEGACY_METADATA: &[u8] = b"public retained encrypted metadata; no real account";
const STANDALONE_MESSAGING_ERROR: &str = "host operation failed: standalone messaging is unavailable in the desktop host; use the Session/Transcript owner";
const SOCIAL_OWNER_ERROR: &str = "host operation failed: social conversations are unavailable in the desktop host; use the Session/Transcript owner";
const SOCIAL_HTTP_OWNER_ERROR: &str = "host operation failed: product API configuration failed: Desktop Human messages are owned by Session/Transcript";
const SOCIAL_RESOURCE_OWNER_ERROR: &str = "host operation failed: product API configuration failed: Desktop Human messages are owned by Session/Transcript";
const SOCIAL_HTTP_PATH_ERROR: &str = "host operation failed: request parameter path is invalid";
const BOT_SOCIAL: &str = "Public retained social-owner Agent";
const SOCIAL_TEXT: &str =
    "Public retained Human payload; never send or reinterpret as Agent input.\0Unicode: 人";

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
    // Preserve the public legacy refusal. The early desktop guard has the same
    // error string, so this alone is not proof of provider-registry absence.
    // assert_production_social_registry_selection checks that independently.
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
        response
            .error
            .as_deref()
            .unwrap_or("")
            .contains("provider is unavailable: telegram"),
        "must preserve the public legacy refusal: {:?}",
        response.error
    );
    assert_legacy_auth_matrix(host);
    assert!(
        host.take_startup_native_shell_event()
            .expect("startup event")
            .is_none()
    );
}

const LEGACY_UNAVAILABLE: &str = "host operation failed: embedded Telegram client is unavailable for the current platform account";

fn identified_request(
    host: &AppHost,
    id: &str,
    method: &str,
    params: Value,
) -> mahayana_app_host::HostResponse {
    let id_value = json!(id);
    let response = host.dispatch(HostRequest {
        id: Some(id_value.clone()),
        method: method.into(),
        params,
    });
    assert_eq!(
        response.id,
        Some(id_value),
        "request identity must survive {method}"
    );
    response
}

fn assert_legacy_auth_matrix(host: &AppHost) {
    // Real public AppHost dispatch; the twelve retired routes retain their
    // historical names/argument shapes, but must never restart a provider.
    let methods = [
        ("telegramStatus", json!({})),
        ("telegramSendCode", json!({"phoneNumber": "+10000000000"})),
        ("telegramSignIn", json!({"code": "public-code"})),
        ("telegramPassword", json!({"password": "public-password"})),
        (
            "telegramRegistration",
            json!({"firstName": "Public", "lastName": "Fixture"}),
        ),
        ("telegramResendCode", json!({})),
        ("telegramCancelCode", json!({})),
        ("telegramRequestPasswordRecovery", json!({})),
        (
            "telegramCheckRecoveryPassword",
            json!({"code": "public-recovery"}),
        ),
        (
            "telegramSendVerifyEmailCode",
            json!({"email": "public@example.invalid"}),
        ),
        ("telegramVerifyEmail", json!({"code": "public-email-code"})),
        ("telegramRefreshPasswordChallenge", json!({})),
    ];
    let mut checked = 0;
    for (method, valid) in methods {
        let malformed = [
            valid,
            json!({}),
            Value::Null,
            json!(false),
            json!(7),
            json!("not-an-object"),
            json!([]),
            json!({"firstName": 7, "code": {"untrusted": true}}),
            json!({"firstName": "  ", "telegramApiHash": "must-not-activate"}),
        ];
        for (variant, params) in malformed.into_iter().enumerate() {
            let registration_has_name = params
                .get("firstName")
                .and_then(Value::as_str)
                .map(|name| !name.trim().is_empty())
                .unwrap_or(false);
            let response = identified_request(
                host,
                &format!("legacy-{method}-{variant}"),
                &format!("feature.auth.{method}"),
                params,
            );
            assert!(
                !response.ok,
                "legacy method unexpectedly succeeded: {method}/{variant}"
            );
            assert!(response.result.is_none(), "legacy method emitted a result");
            let expected = if method == "telegramRegistration" && !registration_has_name {
                "invalid request: firstName is required"
            } else {
                LEGACY_UNAVAILABLE
            };
            assert_eq!(
                response.error.as_deref(),
                Some(expected),
                "legacy {method}/{variant}"
            );
            checked += 1;
        }
    }

    // Registration uniquely validates firstName first, including whitespace,
    // type confusion and non-object values. Other twelve-route refusals are
    // deliberately parameter-independent and must not parse legacy secrets.
    let extra_registration = [
        json!({"firstName": " Public "}),
        json!({"firstName": ""}),
        json!({"firstName": " \t "}),
        json!({"firstName": null}),
        json!({"firstName": 0}),
        json!({"firstName": false}),
        json!({"firstName": []}),
        json!({"firstName": {}}),
        json!({"firstName": "Public", "lastName": 999}),
        json!({"firstName": "Public", "phoneNumber": false}),
        json!({"firstName": "Public", "password": {"unexpected": true}}),
    ];
    for (variant, params) in extra_registration.into_iter().enumerate() {
        let has_name = params
            .get("firstName")
            .and_then(Value::as_str)
            .map(|name| !name.trim().is_empty())
            .unwrap_or(false);
        let response = identified_request(
            host,
            &format!("registration-extra-{variant}"),
            "feature.auth.telegramRegistration",
            params,
        );
        assert!(!response.ok);
        assert!(response.result.is_none());
        assert_eq!(
            response.error.as_deref(),
            Some(if has_name {
                LEGACY_UNAVAILABLE
            } else {
                "invalid request: firstName is required"
            }),
        );
        checked += 1;
    }
    for (variant, method) in [
        "feature.auth.telegramStatusOld",
        "feature.auth.telegramSendCodeLegacy",
        "feature.auth.telegramPasswordV2",
        "feature.auth.telegramUnknown",
        "feature.auth.TELEGRAMSTATUS",
        "feature.auth.telegramVerifyEmailLegacy",
    ]
    .into_iter()
    .enumerate()
    {
        let response = identified_request(
            host,
            &format!("legacy-unknown-{variant}"),
            method,
            json!({}),
        );
        assert!(!response.ok);
        assert!(response.result.is_none());
        assert_eq!(
            response.error.as_deref(),
            Some(format!("invalid request: unknown feature method {method}").as_str()),
        );
        checked += 1;
    }
    assert_eq!(
        checked, 125,
        "twelve historical methods, eleven registration variants and six unknown negatives"
    );
    assert_no_messaging_event(host);
}

fn assert_no_messaging_event(host: &AppHost) {
    // MessagingExecute publishes synchronously into this same event queue.
    // Drain real account/Agent events as well; no inferred source-string oracle.
    let deadline = Instant::now() + Duration::from_secs(2);
    for _ in 0..256 {
        assert!(
            Instant::now() < deadline,
            "FeatureHost event queue did not settle"
        );
        let response = host.dispatch(HostRequest {
            id: None,
            method: "feature.receive".into(),
            params: json!({"timeoutMs": 0}),
        });
        assert!(
            response.ok,
            "production event receive failed: {:?}",
            response.error
        );
        let event = response.result.expect("event receive result");
        if event.is_null() {
            return;
        }
        assert_ne!(
            event["type"], "messaging.event",
            "refused command published a MessagingEvent"
        );
    }
    panic!("FeatureHost event queue exceeded the bounded fixture event budget");
}

fn messaging_account_fingerprint(account_id: &str) -> String {
    Sha256::digest(account_id.as_bytes())[..16]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn assert_standalone_messaging_unavailable(host: &AppHost, account_id: &str) {
    let valid_envelope = json!({
        "protocolVersion": 2,
        "context": {
            "requestId": "public-boundary-sync",
            "deviceId": "public-boundary-device",
            "actorId": format!("human:account:{}", messaging_account_fingerprint(account_id)),
            "sessionId": "public-boundary-session",
            "sentAtMs": 1
        },
        "command": {"type": "sync", "cursor": null, "limit": 1}
    });
    assert_no_messaging_event(host);
    for envelope in [
        valid_envelope,
        Value::Null,
        json!({"protocolVersion": "malformed", "context": [], "command": false}),
    ] {
        let command = json!({
            "type": "messaging.execute",
            "requestId": "public-boundary-dispatch",
            "envelope": envelope
        });
        for (method, params) in [
            (
                "feature.messaging.execute",
                json!({
                    "requestId": "public-boundary-dispatch", "envelope": envelope
                }),
            ),
            ("feature.execute", json!({"command": command})),
            ("feature.execute", command),
        ] {
            let response = host.dispatch(HostRequest {
                id: None,
                method: method.into(),
                params,
            });
            assert!(
                !response.ok,
                "standalone command unexpectedly accepted: {method}"
            );
            assert!(
                response.result.is_none(),
                "refusal cannot return accepted or empty-success"
            );
            assert_eq!(response.error.as_deref(), Some(STANDALONE_MESSAGING_ERROR));
            assert_no_messaging_event(host);
        }
    }
    for params in [
        json!({"blobId": "public-retained-blob", "offset": 0, "length": 32}),
        json!({"blobId": "../public-retained-blob", "offset": u64::MAX, "length": 0}),
    ] {
        let response = host.dispatch(HostRequest {
            id: None,
            method: "feature.messaging.blob.read".into(),
            params,
        });
        assert!(!response.ok);
        assert!(response.result.is_none());
        assert_eq!(response.error.as_deref(), Some(STANDALONE_MESSAGING_ERROR));
        assert_no_messaging_event(host);
    }
    for params in [
        json!({
            "deviceId": "public-boundary-device", "sessionId": "public-boundary-session",
            "scopes": ["messaging", "blobsRead"], "ttlMs": 900000
        }),
        json!({
            "deviceId": "public-boundary-device", "sessionId": "public-boundary-session",
            "scopes": ["unsupported-boundary-scope"], "ttlMs": -1
        }),
    ] {
        let response = host.dispatch(HostRequest {
            id: None,
            method: "feature.messaging.access.issue".into(),
            params,
        });
        assert!(!response.ok);
        assert!(response.result.is_none());
        assert_eq!(response.error.as_deref(), Some(STANDALONE_MESSAGING_ERROR));
        assert_no_messaging_event(host);
    }
    // Invalid outer dispatch shapes are rejected by the existing AppHost
    // parser before the owner is reached. Preserve that validation contract.
    for (method, params) in [
        ("feature.messaging.execute", json!({"requestId": 3})),
        ("feature.messaging.blob.read", json!({"blobId": false})),
        (
            "feature.messaging.access.issue",
            json!({"deviceId": [], "sessionId": null}),
        ),
        (
            "feature.execute",
            json!({"type": "messaging.execute", "requestId": false}),
        ),
        (
            "feature.execute",
            json!({"command": {"type": "messaging.execute"}}),
        ),
    ] {
        let response = host.dispatch(HostRequest {
            id: None,
            method: method.into(),
            params,
        });
        assert!(!response.ok, "malformed request was accepted: {method}");
        assert!(response.result.is_none());
        assert!(
            response
                .error
                .as_deref()
                .unwrap_or("")
                .starts_with("invalid request:")
        );
        assert_no_messaging_event(host);
    }
}

fn assert_reserved_automations_absent(
    host: &AppHost,
    expect_legacy_record: bool,
    authenticated: bool,
) {
    // Do not drain before listing: a failed upsert must leave neither an
    // active record nor an AutomationChanged event in this same queue.
    let response = host.dispatch(HostRequest {
        id: None,
        method: "feature.execute".into(),
        params: json!({"command": {
            "type": "automation.list", "requestId": "public-retained-automation-list"
        }}),
    });
    assert!(
        response.ok,
        "public automation list failed: {:?}",
        response.error
    );
    assert_eq!(
        response.result.expect("automation command accepted")["requestId"],
        "public-retained-automation-list"
    );
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut saw_listed = false;
    for _ in 0..256 {
        assert!(
            Instant::now() < deadline,
            "public automation list event did not arrive"
        );
        let response = host.dispatch(HostRequest {
            id: None,
            method: "feature.receive".into(),
            params: json!({"timeoutMs": 0}),
        });
        assert!(
            response.ok,
            "automation event receive failed: {:?}",
            response.error
        );
        let event = response.result.expect("automation event result");
        if event.is_null() {
            assert!(
                saw_listed,
                "accepted automation list must publish its actual state"
            );
            return;
        }
        assert_ne!(
            event["type"], "messaging.event",
            "automation listing cannot reopen messaging"
        );
        assert_ne!(
            event["type"], "automation.changed",
            "failed collision cannot publish an automation mutation"
        );
        if event["type"] == "automation.listed" {
            assert!(
                !saw_listed,
                "one public list command must yield one actual state"
            );
            saw_listed = true;
            let automations = event["automations"]
                .as_array()
                .expect("actual automation list");
            if !authenticated {
                assert!(
                    automations.is_empty(),
                    "logout must clear the active automation state"
                );
            }
            let normal = automations
                .iter()
                .find(|item| item["id"] == LEGACY_AUTOMATION_ID);
            assert_eq!(
                normal.is_some(),
                expect_legacy_record,
                "normal legacy import control differs from the current account fixture"
            );
            if let Some(normal) = normal {
                assert_eq!(normal["name"], LEGACY_AUTOMATION_NAME);
                assert_eq!(normal["agentId"], "mahayana-assistant");
                assert_eq!(
                    normal["enabled"], false,
                    "archive control must never execute"
                );
            }
            for automation in automations {
                assert_ne!(
                    automation["agentId"], "_messaging",
                    "reserved owner was imported into active state"
                );
                let id = automation["id"]
                    .as_str()
                    .expect("listed automation identity");
                assert!(
                    ![
                        "retained-legacy-reserved",
                        "retained-reserved-override",
                        "retained-valid",
                        "retained-corrupt",
                        "retained-unrecognized",
                        UPSERT_COLLISION_ID,
                    ]
                    .contains(&id),
                    "retained automation marker became active: {id}"
                );
            }
        }
    }
    panic!("public automation list exceeded the bounded event budget");
}

fn assert_retained_automation_boundary(
    host: &AppHost,
    root: &Path,
    account_id: &str,
    authenticated: bool,
) {
    let fingerprint = messaging_account_fingerprint(account_id);
    let feature_root = root.join("app-host/feature-host");
    let legacy_archive = feature_root
        .join("accounts")
        .join(&fingerprint)
        .join("automations.json");
    let collision = feature_root
        .join("runtime/accounts")
        .join(&fingerprint)
        .join("agents/mahayana-assistant/automations")
        .join(UPSERT_COLLISION_ID)
        .join("automation.json");
    let before = messaging_entries(root);
    if authenticated && collision.exists() {
        assert_no_messaging_event(host);
        let response = host.dispatch(HostRequest {
            id: None,
            method: "feature.execute".into(),
            // Omitting agentId exercises the real default Agent owner.
            params: json!({"command": {
                "type": "automation.upsert", "requestId": "public-retained-upsert-collision",
                "id": UPSERT_COLLISION_ID, "name": "Public rejected collision",
                "prompt": "Do not replace retained metadata", "schedule": "@daily",
                "enabled": false
            }}),
        });
        assert!(
            !response.ok,
            "retained metadata collision must refuse the public upsert"
        );
        assert!(
            response.result.is_none(),
            "refused collision cannot return accepted"
        );
        assert_eq!(
            response.error.as_deref(),
            Some(
                "host operation failed: Agent automation path contains retained standalone messaging metadata"
            )
        );
    }
    assert_reserved_automations_absent(
        host,
        authenticated && legacy_archive.exists(),
        authenticated,
    );
    assert_eq!(
        messaging_entries(root),
        before,
        "public automation dispatch altered retained paths or bytes"
    );
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
    assert_standalone_messaging_unavailable(&host, &first.account_id);
    assert!(
        messaging_entries(root).is_empty(),
        "issuer transitions cannot create a second message store"
    );
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

fn settings_disk_path(root: &Path) -> PathBuf {
    root.join("app-host/feature-host/product-settings.json")
}

fn settings_snapshot(host: &AppHost) -> Value {
    let response = identified_request(host, "settings-snapshot", "settings.snapshot", json!({}));
    assert!(
        response.ok,
        "Settings snapshot failed: {:?}",
        response.error
    );
    assert!(response.error.is_none());
    response.result.expect("canonical Settings snapshot")
}

fn assert_settings_disk(host: &AppHost, root: &Path) -> Value {
    let snapshot = settings_snapshot(host);
    let bytes = fs::read(settings_disk_path(root)).expect("authoritative Settings file exists");
    let disk: Value = serde_json::from_slice(&bytes).expect("typed persisted Settings");
    assert_eq!(
        snapshot, disk,
        "public Settings and the single durable owner diverged"
    );
    snapshot
}

fn settings_mutation(
    host: &AppHost,
    root: &Path,
    label: &str,
    mutation: Value,
    publication: bool,
) -> Value {
    let response = identified_request(
        host,
        label,
        "settings.mutateNetworkProxy",
        json!({"mutation": mutation}),
    );
    assert!(
        response.ok,
        "Settings mutation {label} failed: {:?}",
        response.error
    );
    assert!(response.error.is_none());
    let result = response.result.expect("Settings mutation response");
    let snapshot = assert_settings_disk(host, root);
    assert_eq!(result["settings"], snapshot);
    if publication {
        assert_eq!(
            result["publication"],
            json!({
                "available": false, "reason": "telegramClientUnavailable"
            }),
            "Settings cannot activate an absent legacy proxy carrier"
        );
    } else {
        assert!(
            result["publication"].is_null(),
            "unrelated Settings changes cannot publish"
        );
    }
    assert_no_messaging_event(host);
    snapshot
}

fn settings_enabled(host: &AppHost, root: &Path, label: &str, enabled: bool) -> Value {
    let response = identified_request(
        host,
        label,
        "settings.setNetworkProxyEnabled",
        json!({"proxyEnabled": enabled}),
    );
    assert!(
        response.ok,
        "Settings compatibility toggle failed: {:?}",
        response.error
    );
    let result = response.result.expect("Settings toggle result");
    let snapshot = assert_settings_disk(host, root);
    assert_eq!(result["settings"], snapshot);
    assert_eq!(
        result["publication"],
        json!({
            "available": false, "reason": "telegramClientUnavailable"
        }),
        "same-value retries must republish the unavailability envelope"
    );
    assert_no_messaging_event(host);
    snapshot
}

fn assert_settings_restart_matrix(host: &AppHost, root: &Path, restart: bool) {
    let path = settings_disk_path(root);
    let a = json!({"type":"socks5","host":"a.example.invalid","port":1080,"user":"","password":""});
    let b = json!({"type":"http","host":"b.example.invalid","port":8080,"user":"public-b","password":"public-fixture-b"});
    let c = json!({"type":"mtproto","host":"c.example.invalid","port":443,"user":"","password":"public-fixture-c"});
    let d = json!({"type":"web","host":"d.example.invalid","port":8443,"user":"","password":"public-fixture-d"});
    let e = json!({"type":"socks5","host":"e.example.invalid","port":1081,"user":"","password":""});
    if !restart {
        let initial = settings_snapshot(host);
        assert_eq!(initial["version"], 2);
        assert_eq!(initial["network"]["proxy"]["mode"], "system");
        assert!(
            !path.exists(),
            "new owner must not create Settings until a real mutation"
        );

        // Every one of the existing fifteen mutation variants uses the
        // production AppHost dispatch and canonical product-settings.json.
        let cases = [
            (
                "list",
                json!({"type":"setList","list":[a.clone(),b.clone(),c.clone()]}),
                false,
            ),
            (
                "selected-and-mode",
                json!({"type":"setSelectedAndMode","proxy":b.clone(),"mode":"enabled"}),
                true,
            ),
            (
                "preference",
                json!({"type":"setRotationPreferredIndices","indices":[0,1,2,1,-1,7]}),
                false,
            ),
            (
                "remove-first",
                json!({"type":"remove","proxy":a.clone()}),
                false,
            ),
            (
                "insert",
                json!({"type":"insert","index":1,"proxy":d.clone()}),
                false,
            ),
            (
                "replace",
                json!({"type":"replace","was":d.clone(),"proxy":c.clone()}),
                false,
            ),
            ("add", json!({"type":"add","proxy":e.clone()}), false),
            (
                "promote",
                json!({"type":"promoteRotationPreferredIndex","index":3}),
                false,
            ),
            (
                "selected",
                json!({"type":"setSelected","proxy":e.clone()}),
                true,
            ),
            ("ipv6", json!({"type":"setTryIpv6","enabled":false}), true),
            (
                "calls",
                json!({"type":"setUseProxyForCalls","enabled":true}),
                false,
            ),
            (
                "warning",
                json!({"type":"setCheckIpWarningShown","shown":true}),
                false,
            ),
            (
                "rotate",
                json!({"type":"setRotationEnabled","enabled":true}),
                false,
            ),
            (
                "timeout-default",
                json!({"type":"setRotationTimeout","seconds":-1}),
                false,
            ),
            (
                "mode-disabled",
                json!({"type":"setMode","mode":"disabled"}),
                true,
            ),
            (
                "timeout-42",
                json!({"type":"setRotationTimeout","seconds":42}),
                false,
            ),
            (
                "mode-enabled",
                json!({"type":"setMode","mode":"enabled"}),
                true,
            ),
        ];
        for (label, mutation, publication) in cases {
            let snapshot = settings_mutation(host, root, label, mutation, publication);
            if label == "remove-first" {
                assert_eq!(
                    snapshot["network"]["proxy"]["rotation"]["preferredIndices"],
                    json!([0, 1]),
                    "removing A must retain B/C preference identity and relative priority"
                );
                assert_eq!(
                    snapshot["network"]["proxy"]["list"],
                    json!([b.clone(), c.clone()])
                );
            }
        }

        for (variant, enabled) in [true, true, false, false, true, true]
            .into_iter()
            .enumerate()
        {
            settings_enabled(host, root, &format!("compatibility-{variant}"), enabled);
        }

        // Invalid JSON/enum/typed values fail without persistence or mutation.
        let invalid = [
            (
                json!({}),
                "invalid request: settings.mutateNetworkProxy requires mutation",
            ),
            (
                json!({"mutation": null}),
                "invalid request: settings.mutateNetworkProxy mutation is invalid:",
            ),
            (
                json!({"mutation": {"type":"setMode","mode":"bogus"}}),
                "invalid request: settings.mutateNetworkProxy mutation is invalid:",
            ),
            (
                json!({"mutation": {"type":"setList","list":"not-an-array"}}),
                "invalid request: settings.mutateNetworkProxy mutation is invalid:",
            ),
            (
                json!({"mutation": {"type":"insert","index":"bad","proxy":b.clone()}}),
                "invalid request: settings.mutateNetworkProxy mutation is invalid:",
            ),
            (
                json!({"mutation": {"type":"setSelected","proxy":false}}),
                "invalid request: settings.mutateNetworkProxy mutation is invalid:",
            ),
            (
                json!({"mutation": {"type":"setRotationPreferredIndices","indices":[{}]}}),
                "invalid request: settings.mutateNetworkProxy mutation is invalid:",
            ),
            (
                json!({"mutation": {"type":"setRotationTimeout","seconds":4294967296_i64}}),
                "invalid request: settings.mutateNetworkProxy rotation timeout is out of range",
            ),
        ];
        for (variant, (params, prefix)) in invalid.into_iter().enumerate() {
            let before = assert_settings_disk(host, root);
            let raw = fs::read(&path).expect("retained valid Settings document");
            let response = identified_request(
                host,
                &format!("settings-invalid-{variant}"),
                "settings.mutateNetworkProxy",
                params,
            );
            assert!(!response.ok);
            assert!(response.result.is_none());
            assert!(
                response.error.as_deref().unwrap_or("").starts_with(prefix),
                "unexpected invalid Settings envelope: {:?}",
                response.error
            );
            assert_eq!(assert_settings_disk(host, root), before);
            assert_eq!(fs::read(&path).expect("unchanged Settings bytes"), raw);
        }
        for (variant, value) in [
            Value::Null,
            json!("false"),
            json!(0),
            json!([]),
            json!({}),
            json!({"proxyEnabled":42}),
        ]
        .into_iter()
        .enumerate()
        {
            let params = if value.is_object() {
                value
            } else {
                json!({"proxyEnabled": value})
            };
            let before = assert_settings_disk(host, root);
            let response = identified_request(
                host,
                &format!("compatibility-invalid-{variant}"),
                "settings.setNetworkProxyEnabled",
                params,
            );
            assert!(!response.ok);
            assert!(response.result.is_none());
            assert_eq!(
                response.error.as_deref(),
                Some(
                    "invalid request: settings.setNetworkProxyEnabled requires boolean proxyEnabled"
                )
            );
            assert_eq!(assert_settings_disk(host, root), before);
        }

        // A deterministic filesystem fault blocks both mutation types before
        // they can publish or change the in-memory canonical state.
        let temp = path.with_extension("json.tmp");
        fs::create_dir(&temp).expect("reserve the Settings atomic replacement path");
        let before = assert_settings_disk(host, root);
        let raw = fs::read(&path).expect("original Settings bytes");
        for (label, method, params) in [
            (
                "typed-write-fault",
                "settings.mutateNetworkProxy",
                json!({"mutation":{"type":"setMode","mode":"disabled"}}),
            ),
            (
                "compat-write-fault",
                "settings.setNetworkProxyEnabled",
                json!({"proxyEnabled":false}),
            ),
        ] {
            let response = identified_request(host, label, method, params);
            assert!(!response.ok);
            assert!(response.result.is_none());
            assert!(
                response
                    .error
                    .as_deref()
                    .unwrap_or("")
                    .contains("write product Settings temporary file:")
            );
            assert_eq!(
                settings_snapshot(host),
                before,
                "failed persistence cannot update memory"
            );
            assert_eq!(fs::read(&path).expect("read unmodified Settings"), raw);
        }
        fs::remove_dir(&temp).expect("clear only synthetic fault fixture");
        settings_mutation(
            host,
            root,
            "typed-write-recovery",
            json!({"type":"setMode","mode":"disabled"}),
            true,
        );
        settings_enabled(host, root, "compat-write-recovery", true);
    }

    // The same complete oracle is applied after a brand new process has
    // reconstructed the canonical owner, not a cached state or old test seam.
    let expected = json!({
        "version":2,
        "network":{"proxy":{
            "mode":"enabled", "selected":e.clone(),
            "list":[b,c.clone(),c,e],
            "tryIpv6":false, "useProxyForCalls":true, "checkIpWarningShown":true,
            "rotation":{"enabled":true,"timeoutSeconds":42,"preferredIndices":[3,0,2]}
        }}
    });
    let restored = assert_settings_disk(host, root);
    assert_eq!(
        restored, expected,
        "all typed Settings and surviving preference order must survive restart"
    );
    let repeated = settings_enabled(host, root, "restart-idempotent-publication", true);
    assert_eq!(
        repeated, expected,
        "restart same-value publication cannot rewrite authoritative state"
    );
    assert_no_messaging_event(host);
}

fn legacy_child_case(root: &Path, restart: bool) {
    let credential = root.join("native-credential.json");
    let expected_credential = replace_native_credential(&credential, PRODUCTION, "123");
    let host = production_host(root);
    assert_eq!(auth(&host)["loggedIn"], true);
    assert_eq!(
        host.mini_app_account_port()
            .current_scope()
            .expect("Native scope")
            .account_id,
        "123"
    );
    assert_legacy_unavailable(&host);
    if restart {
        assert!(
            contains_bot(&host, BOT_RETAINED),
            "Native Agent must survive process restart"
        );
    } else {
        assert!(!contains_bot(&host, BOT_RETAINED));
        create(&host, BOT_RETAINED);
    }
    assert_settings_restart_matrix(&host, root, restart);
    assert_eq!(
        fs::read(credential).expect("retained Native credential"),
        expected_credential
    );
    assert!(
        !root
            .join("app-host/feature-host/account-session.json")
            .exists()
    );
    drop(host);
}

fn run_child(root: &Path, mode: &str) {
    let issuer = if mode == "production"
        || mode.starts_with("legacy-")
        || mode.starts_with("social-")
        || mode.starts_with("telegram-")
        || (mode.starts_with("messaging-") && mode != "messaging-isolated")
    {
        PRODUCTION
    } else {
        ISOLATED
    };
    let mut command = Command::new(std::env::current_exe().expect("test executable"));
    command
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
        .stderr(Stdio::inherit());
    // Keep the production issuer identity needed by the registration control,
    // while routing any unexpected reqwest transport to this owned rejection
    // proxy. No inherited proxy bypass or real service is needed by the test.
    let network_fence = if mode.starts_with("social-") || mode.starts_with("telegram-") {
        let listener = TcpListener::bind("127.0.0.1:0").expect("owned rejection proxy");
        listener
            .set_nonblocking(true)
            .expect("nonblocking proxy observation");
        let proxy = format!("http://{}", listener.local_addr().expect("proxy address"));
        for name in [
            "HTTPS_PROXY",
            "https_proxy",
            "HTTP_PROXY",
            "http_proxy",
            "ALL_PROXY",
            "all_proxy",
        ] {
            command.env(name, &proxy);
        }
        command.env("NO_PROXY", "").env("no_proxy", "");
        Some(listener)
    } else {
        None
    };
    let mut child = command
        .spawn()
        .expect("spawn isolated integration-test process");
    let deadline = Instant::now() + Duration::from_secs(45);
    let mut unexpected_transports = 0;
    loop {
        if let Some(listener) = &network_fence {
            while let Ok((mut socket, _)) = listener.accept() {
                unexpected_transports += 1;
                socket
                    .set_write_timeout(Some(Duration::from_secs(1)))
                    .expect("proxy write bound");
                // Reject CONNECT and plain HTTP alike, without reading or
                // returning the synthetic credential or request body.
                let _ = socket.write_all(b"HTTP/1.1 502 Forbidden fixture transport\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
            }
        }
        if let Some(status) = child.try_wait().expect("poll test child") {
            assert!(status.success(), "Native AppHost child case failed: {mode}");
            assert_eq!(
                unexpected_transports, 0,
                "retained desktop owner dispatched unexpected HTTP: {mode}"
            );
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
        if matches!(
            mode.as_str(),
            "messaging-first" | "messaging-isolated" | "messaging-restart"
        ) {
            messaging_child_case(
                &root,
                mode == "messaging-restart",
                if mode == "messaging-isolated" {
                    ISOLATED
                } else {
                    PRODUCTION
                },
            );
            return;
        }
        if matches!(
            mode.as_str(),
            "social-first" | "social-restart" | "social-repeat"
        ) {
            social_child_case(&root, mode == "social-first");
            return;
        }
        if matches!(
            mode.as_str(),
            "telegram-first" | "telegram-restart" | "telegram-repeat"
        ) {
            telegram_child_case(&root, mode == "telegram-first");
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
    fs::read_dir(directory)
        .expect("retained directory")
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
    let root = test_directory(if corrupt {
        "legacy-corrupt"
    } else {
        "legacy-encrypted"
    });
    let directory = root.0.join("app-host/feature-host/telegram");
    fs::create_dir_all(&directory).expect("create legacy fixture directory");
    let database = retained_legacy_database_path(&directory, "123");
    if corrupt {
        fs::write(&database, b"public corrupt legacy SQLite fixture\0\xff")
            .expect("corrupt fixture");
        // Sidecars and unknown files are retained, never cleaned at startup.
        fs::write(
            database.with_extension("sqlite3-wal"),
            b"public retained WAL",
        )
        .expect("WAL fixture");
        fs::write(
            database.with_extension("sqlite3-shm"),
            b"public retained SHM",
        )
        .expect("SHM fixture");
    } else {
        let mut store = EncryptedSqliteStore::open(
            &database,
            StorageKey::from_slice(&LEGACY_KEY).expect("fixture key"),
        )
        .expect("create real legacy encrypted SQLite");
        store
            .commit_transition(&Default::default(), &[], 0, 1000)
            .expect("encrypted snapshot");
        store
            .save_private_metadata("retirement.fixture.v1", LEGACY_METADATA, 1000)
            .expect("encrypted metadata");
        drop(store);
    }
    fs::write(
        retained_legacy_database_path(&directory, "456"),
        b"public retained other-account database",
    )
    .expect("other account fixture");
    fs::write(
        directory.join("preserved-user-data.bin"),
        b"public unclassified legacy data",
    )
    .expect("retained file");
    let before = retained_files(&directory);
    for mode in ["legacy-first", "legacy-restart"] {
        run_child(&root.0, mode);
        assert!(
            retained_files(&directory) == before,
            "Native process must leave all legacy bytes unchanged"
        );
    }
    if !corrupt {
        // Rollback compatibility is checked with the original storage reader,
        // only after both Native processes have terminated.
        let store = EncryptedSqliteStore::open(
            &database,
            StorageKey::from_slice(&LEGACY_KEY).expect("original fixture key"),
        )
        .expect("reopen with original reader");
        assert_eq!(
            store
                .load_private_metadata("retirement.fixture.v1")
                .expect("decrypt metadata"),
            Some(LEGACY_METADATA.to_vec())
        );
        let snapshot = store
            .load_snapshot()
            .expect("decrypt snapshot")
            .expect("retained snapshot");
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

fn messaging_entries(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    fn visit(
        base: &Path,
        path: &Path,
        in_messaging: bool,
        entries: &mut BTreeMap<PathBuf, Option<Vec<u8>>>,
    ) {
        let metadata = fs::symlink_metadata(path).expect("inspect owned retention fixture");
        let filename = path.file_name().and_then(|name| name.to_str());
        let in_messaging = in_messaging
            || matches!(
                filename,
                Some("_messaging" | "retained-reserved-override" | UPSERT_COLLISION_ID)
            );
        let legacy_archive = filename == Some("automations.json")
            && path.parent().and_then(Path::parent) == Some(base.join("accounts").as_path());
        if metadata.is_dir() {
            if in_messaging {
                entries.insert(
                    path.strip_prefix(base)
                        .expect("relative messaging directory")
                        .into(),
                    None,
                );
            }
            for child in fs::read_dir(path).expect("read owned runtime directory") {
                visit(
                    base,
                    &child.expect("runtime entry").path(),
                    in_messaging,
                    entries,
                );
            }
        } else if in_messaging || legacy_archive {
            assert!(
                metadata.is_file(),
                "messaging fixture unexpectedly changed file kind"
            );
            entries.insert(
                path.strip_prefix(base)
                    .expect("relative messaging file")
                    .into(),
                Some(fs::read(path).expect("read retained messaging bytes")),
            );
        }
    }
    let feature_root = root.join("app-host/feature-host");
    let mut entries = BTreeMap::new();
    if feature_root.exists() {
        visit(&feature_root, &feature_root, false, &mut entries);
    }
    entries
}

fn messaging_child_case(root: &Path, restart: bool, issuer: &str) {
    let before = messaging_entries(root);
    let credential = root.join("native-credential.json");
    replace_native_credential(&credential, issuer, "123");
    let host = production_host(root);
    let account = host.mini_app_account_port();
    assert_eq!(auth(&host)["loggedIn"], true);
    let first = account
        .current_scope()
        .expect("first real Production Native scope");
    assert_eq!(
        first.account_id,
        if issuer == PRODUCTION {
            "123".to_string()
        } else {
            expected_isolated_id("123")
        }
    );
    assert_legacy_unavailable(&host);
    assert_standalone_messaging_unavailable(&host, &first.account_id);
    assert_retained_automation_boundary(&host, root, &first.account_id, true);
    assert_eq!(
        messaging_entries(root),
        before,
        "startup or refused dispatch touched retained storage"
    );
    if restart {
        assert!(
            contains_bot(&host, BOT_MESSAGING_A),
            "Agent A must survive process restart"
        );
    } else {
        assert!(!contains_bot(&host, BOT_MESSAGING_A));
        create(&host, BOT_MESSAGING_A);
    }
    assert!(
        !contains_bot(&host, BOT_MESSAGING_B),
        "account A cannot see B's Agent"
    );

    replace_native_credential(&credential, issuer, "456");
    assert_eq!(auth(&host)["loggedIn"], true);
    let second = account
        .current_scope()
        .expect("second real Production Native scope");
    assert_eq!(
        second.account_id,
        if issuer == PRODUCTION {
            "456".to_string()
        } else {
            expected_isolated_id("456")
        }
    );
    assert!(second.generation > first.generation);
    assert!(
        !contains_bot(&host, BOT_MESSAGING_A),
        "account B cannot inherit A's Agent"
    );
    if restart {
        assert!(
            contains_bot(&host, BOT_MESSAGING_B),
            "Agent B must survive process restart"
        );
    } else {
        create(&host, BOT_MESSAGING_B);
    }
    assert_standalone_messaging_unavailable(&host, &second.account_id);
    assert_retained_automation_boundary(&host, root, &second.account_id, true);
    assert_eq!(
        messaging_entries(root),
        before,
        "account switch touched standalone messaging state"
    );

    // The Native credential owner performs logout. Unauthenticated dispatch
    // must still refuse before access/session or retained-store I/O.
    fs::remove_file(&credential).expect("Native logout removes only its credential");
    assert_eq!(auth(&host)["loggedIn"], false);
    assert!(account.current_scope().is_err());
    assert_standalone_messaging_unavailable(&host, &second.account_id);
    assert_retained_automation_boundary(&host, root, &second.account_id, false);
    assert!(!contains_bot(&host, BOT_MESSAGING_A));
    assert!(!contains_bot(&host, BOT_MESSAGING_B));
    assert_eq!(
        messaging_entries(root),
        before,
        "logged-out dispatch touched retained storage"
    );

    let expected = replace_native_credential(&credential, issuer, "123");
    assert_eq!(auth(&host)["loggedIn"], true);
    let returned = account.current_scope().expect("relogged-in Native scope");
    assert_eq!(returned.account_id, first.account_id);
    assert!(returned.generation > second.generation);
    assert!(contains_bot(&host, BOT_MESSAGING_A));
    assert!(!contains_bot(&host, BOT_MESSAGING_B));
    assert_standalone_messaging_unavailable(&host, &returned.account_id);
    assert_retained_automation_boundary(&host, root, &returned.account_id, true);
    assert_eq!(
        fs::read(&credential).expect("Native credential remains"),
        expected
    );
    assert!(
        !root
            .join("app-host/feature-host/account-session.json")
            .exists()
    );
    assert_eq!(
        messaging_entries(root),
        before,
        "relogin altered retained messaging bytes"
    );
    drop(account);
    drop(host);
    assert_eq!(
        messaging_entries(root),
        before,
        "shutdown altered retained messaging bytes"
    );
}

fn seed_retained_messaging(root: &Path, corrupt: bool) {
    let runtime = root.join("app-host/feature-host/runtime");
    // Preserve the actual unscoped access namespace as well as current,
    // switched and never-selected account stores. These are byte fixtures;
    // the FeatureHost owner tests separately exercise original writer/readers.
    let mut stores = vec![runtime.join("agents/_messaging")];
    for id in [
        "123".to_string(),
        "456".to_string(),
        "789".to_string(),
        expected_isolated_id("123"),
        expected_isolated_id("456"),
    ] {
        let fingerprint = messaging_account_fingerprint(&id);
        stores.push(
            runtime
                .join("accounts")
                .join(&fingerprint)
                .join("agents/_messaging"),
        );
        // The historical account-wide archive is beside runtime, not inside
        // it. Loading that original Vec<AutomationSummary> must not import a
        // reserved owner or overwrite an external reserved-owner marker.
        let legacy_account = root
            .join("app-host/feature-host/accounts")
            .join(&fingerprint);
        fs::create_dir_all(&legacy_account).expect("retained legacy automation account directory");
        let legacy_records = json!([
            {
                "id": "retained-legacy-reserved", "agentId": "_messaging",
                "name": "Public retained legacy messaging record", "prompt": "Do not import",
                "schedule": "@daily", "enabled": false, "createdAtMs": 1000,
                "lastRunAtMs": null, "nextRunAtMs": null
            },
            {
                "id": "retained-reserved-override", "agentId": "research",
                "name": "Public legacy same-path collision", "prompt": "Do not overwrite",
                "schedule": "@daily", "enabled": false, "createdAtMs": 1001,
                "lastRunAtMs": null, "nextRunAtMs": null
            },
            {
                "id": LEGACY_AUTOMATION_ID, "agentId": "mahayana-assistant",
                "name": LEGACY_AUTOMATION_NAME, "prompt": "Public disabled migration control",
                "schedule": "@daily", "enabled": false, "createdAtMs": 1002,
                "lastRunAtMs": null, "nextRunAtMs": null
            }
        ]);
        fs::write(
            legacy_account.join("automations.json"),
            serde_json::to_vec_pretty(&legacy_records).expect("original automation archive array"),
        )
        .expect("retained legacy automation archive");
        let external_marker = runtime
            .join("accounts")
            .join(&fingerprint)
            .join("agents/research/automations/retained-reserved-override");
        fs::create_dir_all(external_marker.join("user-backup"))
            .expect("retained external automation folder");
        fs::write(
            external_marker.join("automation.json"),
            serde_json::to_vec_pretty(&json!({
                "name": "Public reserved-owner override marker", "prompt": "Preserve this folder",
                "schedule": "@daily", "enabled": false, "createdAt": 777,
                "lastRunAt": null,
                "_mahayana": {"id": "retained-reserved-override", "agentId": "_messaging"},
                "publicUnknownProperty": "original external marker"
            }))
            .expect("external reserved-owner automation marker"),
        )
        .expect("retained external automation marker");
        fs::write(
            external_marker.join("preserved.bin"),
            b"public original external sibling bytes",
        )
        .expect("retained external sibling");
        fs::write(external_marker.join("user-backup/keep.bin"), [0, 255, 9])
            .expect("retained external nested sibling");
        let collision = runtime
            .join("accounts")
            .join(&fingerprint)
            .join("agents/mahayana-assistant/automations")
            .join(UPSERT_COLLISION_ID);
        fs::create_dir_all(collision.join("user-backup"))
            .expect("retained default Agent collision folder");
        fs::write(
            collision.join("automation.json"),
            serde_json::to_vec_pretty(&json!({
                "name": "Public retained default Agent marker", "prompt": "Keep the original marker",
                "schedule": "@daily", "enabled": false, "createdAt": 778,
                "lastRunAt": null,
                "_mahayana": {"id": UPSERT_COLLISION_ID, "agentId": "_messaging"},
                "publicUnknownProperty": "original default Agent collision marker"
            })).expect("default Agent reserved-owner marker"),
        ).expect("retained default Agent marker");
        fs::write(
            collision.join("preserved.bin"),
            b"public default Agent original sibling bytes",
        )
        .expect("retained default Agent sibling");
        fs::write(collision.join("user-backup/keep.bin"), [0, 255, 10])
            .expect("retained default Agent nested sibling");
    }
    for (index, directory) in stores.iter().enumerate() {
        fs::create_dir_all(directory.join("blobs")).expect("seed retained blob directory");
        let snapshot = if corrupt {
            b"{public-corrupt-snapshot\0\xff".to_vec()
        } else {
            serde_json::to_vec(&json!({
                "schemaVersion": 1, "cursor": index + 7, "savedAtMs": 1000,
                "state": {}, "retainedPublicFixture": index
            }))
            .expect("representative retained snapshot JSON")
        };
        fs::write(directory.join("snapshot.json"), snapshot).expect("retained snapshot bytes");
        fs::write(
            directory.join("snapshot.json.tmp"),
            b"public unfinished snapshot replacement",
        )
        .expect("retained snapshot temporary");
        fs::write(
            directory.join("access.json"),
            if corrupt {
                b"public corrupt access registry\0\xff".as_slice()
            } else {
                b"{\"grants\":[],\"retainedPublicFixture\":true}".as_slice()
            },
        )
        .expect("retained access registry");
        fs::write(
            directory.join("access.json.tmp"),
            b"public unfinished access replacement",
        )
        .expect("retained access temporary");
        for (name, bytes) in [
            (
                "public-retained-blob.blob",
                b"public original blob bytes".as_slice(),
            ),
            (
                "public-retained-blob.json",
                b"{\"publicRetainedMetadata\":true}".as_slice(),
            ),
            (
                "public-upload.part",
                b"public partial upload bytes".as_slice(),
            ),
            (
                "public-upload.part.json",
                b"{\"publicUploadOffset\":7}".as_slice(),
            ),
            (
                "public-upload.part.json.tmp",
                b"public interrupted metadata replacement".as_slice(),
            ),
        ] {
            fs::write(directory.join("blobs").join(name), bytes)
                .expect("retained blob/upload fixture");
        }
        fs::create_dir(directory.join("unknown-empty-directory"))
            .expect("retained empty directory");
        fs::write(
            directory.join("unknown-user-data.bin"),
            [index as u8, 0, 255, 7],
        )
        .expect("retained unknown file");
        // Login/account reconciliation scans the containing agents directory.
        // A reserved messaging subtree must never be treated as an Agent's
        // automation store, including malformed markers eligible for cleanup.
        let automation_markers = [
            (
                "retained-valid",
                serde_json::to_vec(&json!({
                    "name": "Public retained marker", "prompt": "Keep these bytes",
                    "schedule": "0 9 * * *", "enabled": false, "createdAt": 1000,
                    "lastRunAt": null,
                    "_mahayana": {"id": "retained-valid", "agentId": "_messaging"},
                    "retainedUnknownField": index
                }))
                .expect("representative retained automation marker"),
            ),
            (
                "retained-corrupt",
                b"{public corrupt automation marker\0\xff".to_vec(),
            ),
            (
                "retained-unrecognized",
                b"{\"unknownUserFormat\":true}".to_vec(),
            ),
        ];
        for (name, marker) in automation_markers {
            let automation = directory.join("automations").join(name);
            fs::create_dir_all(automation.join("user-backup"))
                .expect("retained automation directory");
            fs::write(automation.join("automation.json"), marker)
                .expect("retained automation marker");
            fs::write(
                automation.join("preserved.bin"),
                b"public original automation-adjacent bytes",
            )
            .expect("retained automation-adjacent file");
            fs::write(
                automation.join("user-backup/item.bin"),
                [0, 255, index as u8],
            )
            .expect("retained nested automation backup");
        }
    }
}

fn retained_messaging_case(corrupt: bool) {
    let root = test_directory(if corrupt {
        "messaging-corrupt"
    } else {
        "messaging-retained"
    });
    seed_retained_messaging(&root.0, corrupt);
    let before = messaging_entries(&root.0);
    assert!(!before.is_empty());
    for mode in ["messaging-first", "messaging-isolated", "messaging-restart"] {
        run_child(&root.0, mode);
        assert_eq!(
            messaging_entries(&root.0),
            before,
            "production process changed retained messaging state"
        );
    }
}

#[test]
fn native_standalone_messaging_refuses_dispatch_without_creating_storage() {
    let root = test_directory("messaging-absent");
    assert!(messaging_entries(&root.0).is_empty());
    for mode in ["messaging-first", "messaging-isolated", "messaging-restart"] {
        run_child(&root.0, mode);
        assert!(
            messaging_entries(&root.0).is_empty(),
            "missing messaging stores must remain absent"
        );
    }
}

#[test]
fn native_standalone_messaging_preserves_snapshot_blob_access_and_other_accounts() {
    retained_messaging_case(false);
}

#[test]
fn native_standalone_messaging_preserves_corrupt_and_interrupted_stores() {
    retained_messaging_case(true);
}

fn assert_production_social_registry_selection(root: &Path) {
    let config = |label: &str| {
        let base = root.join(label);
        fs::create_dir_all(base.join("workspace")).expect("isolated Host workspace");
        HostCreateConfig {
            runtime: mahayana_core::RuntimeConfig {
                data_dir: Some(base.join("runtime")),
                workspace_roots: vec![base.join("workspace")],
                ..Default::default()
            },
            product_session_path: Some(base.join("account-session.json")),
            product_surface_state_path: Some(base.join("product-surface.json")),
            product_storage_passphrase: Some("public-social-owner-storage-passphrase".into()),
            cwd: Some(base.join("workspace")),
            inherit_installed_plugins: Some(false),
            ..Default::default()
        }
    };
    // The same production-issuer Native credential can register Social in an
    // ordinary Host. An isolated issuer would not exercise that former path.
    let generic = MahayanaHost::create(config("ordinary-host-control"))
        .expect("ordinary Host retains its supported provider composition");
    let ordinary = generic.status().providers;
    assert!(
        ordinary
            .iter()
            .any(|provider| provider == "mahayana-social")
    );
    assert!(ordinary.iter().any(|provider| provider == "mahayana-ai"));
    assert!(ordinary.iter().any(|provider| provider == "miniapp"));
    drop(generic);
    let owned =
        MahayanaHost::create_with_session_transcript_owner(config("session-owned-host-control"))
            .expect("Session/Transcript-owned Host");
    let providers = owned.status().providers;
    let expected = ordinary
        .into_iter()
        .filter(|provider| provider != "mahayana-social")
        .collect::<Vec<_>>();
    assert_eq!(
        providers, expected,
        "only the generic social writer is retired"
    );
    // Public FeatureHost denial now runs before the registry. Inspect the real
    // Session/Transcript-owned Host and call its Runtime directly as well.
    assert!(!providers.iter().any(|provider| provider == "telegram"));
    let error = owned
        .execute(mahayana_core::RuntimeCommand::ConversationHistory {
            conversation_id: mahayana_core::ConversationId("telegram:chat:42".into()),
            limit: Some(20),
        })
        .expect_err("the actual desktop Runtime must not register Telegram");
    assert_eq!(error.to_string(), "provider is unavailable: telegram");
    drop(owned);
}

fn social_sqlite_image(host: &AppHost) -> Value {
    host.mini_app_store_port().expect("real RuntimeStore port")
        .with_connection::<_, rusqlite::Error>(|connection| {
            let mut image = serde_json::Map::new();
            for table in ["conversations", "turns", "runs", "turn_requests", "pending_intents", "handoff_dispatch"] {
                let mut statement = connection.prepare(&format!("SELECT * FROM {table} ORDER BY 1"))?;
                let columns = statement.column_count();
                let rows = statement.query_map([], |row| {
                    (0..columns).map(|column| {
                        Ok(match row.get_ref(column)? {
                            rusqlite::types::ValueRef::Null => Value::Null,
                            rusqlite::types::ValueRef::Integer(value) => json!(value),
                            rusqlite::types::ValueRef::Real(value) => json!(value),
                            rusqlite::types::ValueRef::Text(bytes) => json!(String::from_utf8(bytes.to_vec()).expect("fixture SQLite UTF-8")),
                            rusqlite::types::ValueRef::Blob(bytes) => json!({"blob": base64::engine::general_purpose::STANDARD.encode(bytes)}),
                        })
                    }).collect::<Result<Vec<_>, rusqlite::Error>>()
                })?.collect::<Result<Vec<_>, _>>()?;
                image.insert(table.into(), json!(rows));
            }
            Ok(Value::Object(image))
        }).expect("read retained social rows through original RuntimeStore connection")
}

fn seed_social_waiting_turn(host: &AppHost) {
    // Use the actual initialized schema and RuntimeStore-owned connection.
    // WaitingUser is deliberate: even an old binary must fail this preservation
    // assertion without sending this synthetic text to a real service.
    host.mini_app_store_port().expect("real RuntimeStore fixture port")
        .with_connection::<_, rusqlite::Error>(|connection| {
            let transaction = connection.transaction()?;
            transaction.execute(
                "INSERT INTO conversations(conversation_id, updated_at_ms) VALUES (?1, 71)",
                ["mahayana:contact:public-retained-contact"],
            )?;
            transaction.execute(
                "INSERT INTO turns(turn_id, conversation_id, user_message_id, created_at_ms, state, active_run_id)
                 VALUES ('public-social-turn', ?1, 'public-social-message', 72, 'waiting-user', 'public-social-run')",
                ["mahayana:contact:public-retained-contact"],
            )?;
            transaction.execute(
                "INSERT INTO runs(run_id, turn_id, generation, provider, started_at_ms, finished_at_ms, state)
                 VALUES ('public-social-run', 'public-social-turn', 4, 'mahayana-social', 73, NULL, 'waiting-user')",
                [],
            )?;
            transaction.execute(
                "INSERT INTO turn_requests(turn_id, message_id, text, inference_provider, hidden, updated_at_ms)
                 VALUES ('public-social-turn', 'public-social-message', ?1, NULL, 0, 74)",
                [SOCIAL_TEXT],
            )?;
            transaction.execute(
                "INSERT INTO pending_intents(intent_id, turn_id, run_id, kind, payload_json, created_at_ms)
                 VALUES ('public-social-ask', 'public-social-turn', 'public-social-run', 'ask-user', ?1, 75)",
                [" {\"retainedUnknownField\": [1, true, \"人\"], \"publicPayload\": \"preserve original spacing\"} "],
            )?;
            transaction.commit()
        }).expect("seed durable historical social turn without dispatch");
}

fn assert_social_entries_unavailable(host: &AppHost, lane: &PlatformRequestHost) {
    for conversation in ["mahayana:contact:42", " \tmahayana:contact:42\n"] {
        for command in [
            json!({"type": "conversation.open", "requestId": "retired-open", "conversationId": conversation}),
            json!({"type": "chat.send", "requestId": "retired-send", "conversationId": conversation,
                "agentId": "mahayana-assistant", "text": SOCIAL_TEXT, "mode": "agent"}),
        ] {
            for params in [command.clone(), json!({"command": command})] {
                let response = identified_request(host, "social-owner", "feature.execute", params);
                assert!(!response.ok);
                assert!(response.result.is_none());
                assert_eq!(response.error.as_deref(), Some(SOCIAL_OWNER_ERROR));
            }
        }
    }
    for authenticated in [false, true] {
        for (method, path, expected_error) in [
            ("POST", "/api/social/messages", SOCIAL_HTTP_OWNER_ERROR),
            ("GET", "/api/social/messages", SOCIAL_HTTP_OWNER_ERROR),
            ("PATCH", "/api/social/messages/1", SOCIAL_HTTP_OWNER_ERROR),
            ("DELETE", "/api/social/messages/1", SOCIAL_HTTP_OWNER_ERROR),
            (
                "POST",
                "/api/social/messages/1/reactions",
                SOCIAL_HTTP_OWNER_ERROR,
            ),
        ]
        .into_iter()
        .chain(
            ["GET", "POST", "PUT", "PATCH", "DELETE"]
                .into_iter()
                .flat_map(|method| {
                    [
                        "/api/social/message-resources",
                        "/api/social/message-resources/12345678-1234-1234-1234-123456789abc",
                    ]
                    .into_iter()
                    .map(move |path| (method, path, SOCIAL_RESOURCE_OWNER_ERROR))
                }),
        )
        .chain([
            (
                "GET",
                "/api/other/%2e%2e/social/message-resources/12345678-1234-1234-1234-123456789abc",
                SOCIAL_RESOURCE_OWNER_ERROR,
            ),
            (
                "GET",
                "/api/social/%6dessage-resources/12345678-1234-1234-1234-123456789abc",
                SOCIAL_HTTP_PATH_ERROR,
            ),
            (
                "GET",
                "/api/social/Message-resources/12345678-1234-1234-1234-123456789abc",
                SOCIAL_HTTP_PATH_ERROR,
            ),
        ]) {
            let params = json!({"method": method, "path": path, "authenticated": authenticated,
                "sessionOwned": false,
                "body": {"text": SOCIAL_TEXT, "contact": "public-contact"}});
            let response = identified_request(
                host,
                "social-http-owner",
                "platform.request",
                params.clone(),
            );
            assert!(!response.ok);
            assert!(response.result.is_none());
            assert_eq!(response.error.as_deref(), Some(expected_error));
            let response: Value = serde_json::from_str(&lane.dispatch_json(&json!({
                "id": "social-http-lane-owner", "method": "platform.request", "params": params,
            }).to_string())).expect("decode real platform lane response");
            assert_eq!(response["id"], "social-http-lane-owner");
            assert_eq!(response["ok"], false);
            assert!(response["result"].is_null());
            assert_eq!(response["error"], expected_error);
            assert!(
                !response
                    .to_string()
                    .contains("Public retained Human payload")
            );
        }
    }
    // Rejected intents cannot publish chat/history results or copy their body
    // into the feature event stream. Ordinary account/Agent events are allowed.
    for _ in 0..256 {
        let response = identified_request(
            host,
            "social-event-check",
            "feature.receive",
            json!({"timeoutMs": 0}),
        );
        assert!(response.ok);
        let event = response.result.expect("feature event result");
        if event.is_null() {
            return;
        }
        assert_ne!(event["type"], "chat.message");
        assert_ne!(event["type"], "conversation.opened");
        assert!(!event.to_string().contains("Public retained Human payload"));
    }
    panic!("social owner fixture event queue did not settle");
}

fn social_child_case(root: &Path, first: bool) {
    let credential = root.join("native-credential.json");
    let expected_credential = replace_native_credential(&credential, PRODUCTION, "123");
    assert_production_social_registry_selection(root);
    let host = production_host(root);
    assert_eq!(auth(&host)["loggedIn"], true);
    assert_eq!(
        host.mini_app_account_port()
            .current_scope()
            .expect("production Native scope")
            .account_id,
        "123"
    );
    let baseline = root.join("public-social-retention-before.json");
    if first {
        create(&host, BOT_SOCIAL);
        seed_social_waiting_turn(&host);
        fs::write(
            &baseline,
            serde_json::to_vec(&social_sqlite_image(&host)).expect("retained SQLite baseline"),
        )
        .expect("write owned test baseline");
    } else {
        assert!(
            contains_bot(&host, BOT_SOCIAL),
            "normal Agent remains available after restart"
        );
    }
    let before: Value =
        serde_json::from_slice(&fs::read(&baseline).expect("original retained baseline"))
            .expect("decode retained baseline");
    assert_eq!(
        social_sqlite_image(&host),
        before,
        "restart changed historical social state or payload"
    );
    let lane = PlatformRequestHost::new(root.join("app-host"))
        .expect("production lightweight platform lane uses Native owner");
    assert_social_entries_unavailable(&host, &lane);
    assert_eq!(
        fs::read(&credential).expect("Native credential after denied requests"),
        expected_credential
    );
    assert_eq!(social_sqlite_image(&host), before);
    let switched_credential = replace_native_credential(&credential, PRODUCTION, "456");
    assert_eq!(auth(&host)["loggedIn"], true);
    assert_eq!(
        host.mini_app_account_port()
            .current_scope()
            .expect("switched Native scope")
            .account_id,
        "456"
    );
    assert!(
        !contains_bot(&host, BOT_SOCIAL),
        "previous account Agent must remain isolated"
    );
    assert_social_entries_unavailable(&host, &lane);
    assert_eq!(
        fs::read(&credential).expect("switched Native credential after refusal"),
        switched_credential
    );
    assert_eq!(
        social_sqlite_image(&host),
        before,
        "account change altered retained social records"
    );
    let returned_credential = replace_native_credential(&credential, PRODUCTION, "123");
    assert_eq!(auth(&host)["loggedIn"], true);
    assert_eq!(
        host.mini_app_account_port()
            .current_scope()
            .expect("returned Native scope")
            .account_id,
        "123"
    );
    assert!(
        contains_bot(&host, BOT_SOCIAL),
        "original account Agent remains durable"
    );
    assert_social_entries_unavailable(&host, &lane);
    assert_eq!(
        fs::read(&credential).expect("returned Native credential after refusal"),
        returned_credential
    );
    assert_eq!(
        social_sqlite_image(&host),
        before,
        "returning to original account altered retained social records"
    );
    fs::remove_file(&credential).expect("Native logout fixture");
    assert_eq!(auth(&host)["loggedIn"], false);
    assert_social_entries_unavailable(&host, &lane);
    assert!(
        matches!(fs::symlink_metadata(&credential), Err(error)
        if error.kind() == std::io::ErrorKind::NotFound),
        "rejected logged-out HTTP calls must not recreate the Native credential"
    );
    assert_eq!(auth(&host)["loggedIn"], false);
    assert_eq!(
        social_sqlite_image(&host),
        before,
        "unauthenticated denials changed retained data"
    );
    assert!(
        !root
            .join("app-host/feature-host/account-session.json")
            .exists()
    );
    drop(lane);
    drop(host);
}

#[test]
fn native_social_owner_preserves_sqlite_waiting_turn_and_refuses_bypass() {
    let root = test_directory("social-owner");
    for mode in ["social-first", "social-restart", "social-repeat"] {
        run_child(&root.0, mode);
    }
}

const TELEGRAM_OWNER_ERROR: &str = "host operation failed: provider is unavailable: telegram";
const TELEGRAM_RETAINED_TEXT: &str =
    "Public retained Telegram payload; never resend or reinterpret as Agent input.\0Unicode: 人";
const BOT_TELEGRAM: &str = "public-retained-owner-agent-control";

fn seed_telegram_retained_work(host: &AppHost) {
    // Seed the schema initialized by the real Production AppHost, never a
    // replacement database. The third turn predates durable turn_requests.
    host.mini_app_store_port().expect("real RuntimeStore fixture port")
        .with_connection::<_, rusqlite::Error>(|connection| {
            let transaction = connection.transaction()?;
            for (index, (suffix, conversation, state, generation, nonce)) in [
                ("active", "telegram:chat:42", "thinking", 7_i64, "public-telegram-active-nonce"),
                ("waiting", "telegram:chat:public-retained-chat", "waiting-user", 11_i64, "public-telegram-running-handoff"),
                ("legacy", "telegram:chat:-10042", "recovering", 4_i64, "public-telegram-legacy-nonce"),
                ("waiting-direct", "telegram:chat:43", "waiting-user", 13_i64, "public-telegram-waiting-direct-nonce"),
            ].into_iter().enumerate() {
                let turn_id = format!("public-telegram-{suffix}-turn");
                let run_id = format!("public-telegram-{suffix}-run");
                let at = 170_i64 + index as i64 * 10;
                transaction.execute(
                    "INSERT INTO conversations(conversation_id, updated_at_ms) VALUES (?1, ?2)",
                    rusqlite::params![conversation, at],
                )?;
                transaction.execute(
                    "INSERT INTO turns(turn_id, conversation_id, user_message_id, created_at_ms, state, active_run_id)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![turn_id, conversation, nonce, at + 1, state, run_id],
                )?;
                transaction.execute(
                    "INSERT INTO runs(run_id, turn_id, generation, provider, started_at_ms, finished_at_ms, state)
                     VALUES (?1, ?2, ?3, 'telegram', ?4, NULL, ?5)",
                    rusqlite::params![run_id, turn_id, generation, at + 2, state],
                )?;
                if suffix != "legacy" {
                    transaction.execute(
                        "INSERT INTO turn_requests(turn_id, message_id, text, inference_provider, hidden, updated_at_ms)
                         VALUES (?1, ?2, ?3, 'public-retained-route', ?4, ?5)",
                        rusqlite::params![turn_id, nonce, format!("{TELEGRAM_RETAINED_TEXT}\n{suffix}"),
                            i64::from(suffix == "waiting"), at + 3],
                    )?;
                }
            }
            // Keep exact spacing, escaped NUL and unknown fields in the stored
            // JSON. Comparing decoded objects would miss a payload rewrite.
            let question = r#"{
  "id" : "public-telegram-question", "turnId":"public-telegram-waiting-turn",
  "runId":"public-telegram-waiting-run",
  "question" : "Public retained Telegram payload; preserve question\u0000人",
  "unknown" : { "order" : [3, 1], "spaces" : "a  b" }
}"#;
            transaction.execute(
                "INSERT INTO pending_intents(intent_id, turn_id, run_id, kind, payload_json, created_at_ms)
                 VALUES ('public-telegram-question', 'public-telegram-waiting-turn',
                         'public-telegram-waiting-run', 'ask-user', ?1, 205)",
                [question],
            )?;
            // This independent waiting-user row is not excluded by the
            // RuntimeStore's agent-handoff nonce join. It must survive the
            // interrupted-turn branch before that branch can mark it Failed.
            let direct_question = r#"{
  "id" : "public-telegram-direct-question", "turnId":"public-telegram-waiting-direct-turn",
  "runId":"public-telegram-waiting-direct-run",
  "question" : "Public retained Telegram payload; direct waiting question\u0000人",
  "unknown" : { "spaces" : " keep  original " }
}"#;
            transaction.execute(
                "INSERT INTO pending_intents(intent_id, turn_id, run_id, kind, payload_json, created_at_ms)
                 VALUES ('public-telegram-direct-question', 'public-telegram-waiting-direct-turn',
                         'public-telegram-waiting-direct-run', 'ask-user', ?1, 208)",
                [direct_question],
            )?;
            for (intent_id, dispatch_state, target_operation_id, payload) in [
                (
                    "public-telegram-queued-handoff",
                    "queued",
                    None,
                    r#"{
  "id" : "public-telegram-queued-handoff", "targetAgent":"public-retained-target",
  "targetConversationId" : "telegram:chat:42",
  "task" : "Public retained Telegram payload; queued handoff\u0000人",
  "constraints" : { "literal" : " keep  spacing " }, "expectedOutput":null,
  "originRun":"public-telegram-active-run", "depth":1, "unknown":[9, 2]
}"#,
                ),
                (
                    "public-telegram-running-handoff",
                    "running",
                    Some("public-telegram-waiting-run"),
                    r#"{
  "id" : "public-telegram-running-handoff", "targetAgent":"public-retained-target",
  "targetConversationId" : "telegram:chat:public-retained-chat",
  "task" : "Public retained Telegram payload; running handoff\u0000人",
  "constraints" : { "literal" : " retain  unknown " }, "expectedOutput":"original",
  "originRun":"public-telegram-active-run", "depth":2, "unknown":{"future":true}
}"#,
                ),
            ] {
                transaction.execute(
                    "INSERT INTO pending_intents(intent_id, turn_id, run_id, kind, payload_json, created_at_ms)
                     VALUES (?1, 'public-telegram-active-turn', 'public-telegram-active-run',
                             'agent-handoff', ?2, 206)",
                    rusqlite::params![intent_id, payload],
                )?;
                transaction.execute(
                    "INSERT INTO handoff_dispatch(intent_id, target_operation_id, state, updated_at_ms)
                     VALUES (?1, ?2, ?3, 207)",
                    rusqlite::params![intent_id, target_operation_id, dispatch_state],
                )?;
            }
            transaction.commit()
        }).expect("seed retained Telegram input, question and handoff rows without dispatch");
}

fn assert_telegram_entries_unavailable(host: &AppHost) {
    for conversation in [
        "telegram:chat:42",
        "telegram:chat:",
        " \ttelegram:chat:42\n",
        " \ttelegram:chat:\n",
        "telegram:chat:public-retained-chat",
    ] {
        for command in [
            json!({"type": "conversation.open", "requestId": "retained-telegram-open",
                "conversationId": conversation}),
            json!({"type": "chat.send", "requestId": "retained-telegram-send",
                "conversationId": conversation, "agentId": "mahayana-assistant",
                "clientMessageId": "public-telegram-denied-nonce",
                "text": TELEGRAM_RETAINED_TEXT, "mode": "agent"}),
            json!({"type": "chat.send", "requestId": "retained-telegram-plugin-send",
                "conversationId": conversation, "agentId": "retained-plugin-fixture",
                "clientMessageId": "public-telegram-denied-plugin-nonce",
                "text": TELEGRAM_RETAINED_TEXT, "mode": "agent"}),
        ] {
            for params in [command.clone(), json!({"command": command})] {
                let response =
                    identified_request(host, "telegram-owner", "feature.execute", params);
                assert!(!response.ok);
                assert!(
                    response.result.is_none(),
                    "denial cannot admit an operation"
                );
                assert_eq!(response.error.as_deref(), Some(TELEGRAM_OWNER_ERROR));
            }
        }
    }
    // Native account/Agent bootstrap events are allowed. No retired input,
    // conversation result, turn transition or operation can escape this owner.
    // The six-table equality below independently rejects newly persisted work.
    for _ in 0..256 {
        let response = identified_request(
            host,
            "telegram-event-check",
            "feature.receive",
            json!({"timeoutMs": 0}),
        );
        assert!(response.ok);
        let event = response.result.expect("feature event result");
        if event.is_null() {
            return;
        }
        let kind = event["type"].as_str().expect("typed Host event");
        assert!(
            !matches!(
                kind,
                "chat.message"
                    | "chat.delta"
                    | "conversation.opened"
                    | "turn.state"
                    | "messaging.event"
            ),
            "retired Telegram intent published a message or turn event: {kind}"
        );
        assert!(
            !kind.starts_with("operation."),
            "unexpected operation event: {kind}"
        );
        assert!(
            event.get("operationId").is_none(),
            "unexpected admitted operation: {event}"
        );
        let encoded = event.to_string();
        assert!(
            !encoded.contains("Public retained Telegram payload"),
            "retained body leaked into events"
        );
        assert!(
            !encoded.contains("public-telegram-"),
            "retained identity leaked into events"
        );
    }
    panic!("Telegram owner fixture event queue did not settle");
}

fn telegram_child_case(root: &Path, first: bool) {
    let credential = root.join("native-credential.json");
    let expected_credential = replace_native_credential(&credential, PRODUCTION, "123");
    assert_production_social_registry_selection(root);
    let host = production_host(root);
    assert_eq!(auth(&host)["loggedIn"], true);
    assert_eq!(
        host.mini_app_account_port()
            .current_scope()
            .expect("production Native scope")
            .account_id,
        "123"
    );
    let baseline = root.join("public-telegram-retention-before.json");
    if first {
        create(&host, BOT_TELEGRAM);
        seed_telegram_retained_work(&host);
        fs::write(
            &baseline,
            serde_json::to_vec(&social_sqlite_image(&host)).expect("retained SQLite baseline"),
        )
        .expect("write owned Telegram test baseline");
    } else {
        assert!(
            contains_bot(&host, BOT_TELEGRAM),
            "normal Agent remains available after real process restart"
        );
    }
    let before: Value =
        serde_json::from_slice(&fs::read(&baseline).expect("original retained Telegram baseline"))
            .expect("decode retained Telegram baseline");
    assert_eq!(
        social_sqlite_image(&host),
        before,
        "startup changed retained Telegram fields or payload bytes"
    );
    assert_telegram_entries_unavailable(&host);
    assert_eq!(
        fs::read(&credential).expect("Native credential after Telegram refusal"),
        expected_credential
    );
    assert_eq!(
        social_sqlite_image(&host),
        before,
        "denied Telegram intent wrote RuntimeStore work"
    );

    let switched_credential = replace_native_credential(&credential, PRODUCTION, "456");
    assert_eq!(auth(&host)["loggedIn"], true);
    assert_eq!(
        host.mini_app_account_port()
            .current_scope()
            .expect("switched Native scope")
            .account_id,
        "456"
    );
    assert!(
        !contains_bot(&host, BOT_TELEGRAM),
        "previous account Agent must remain isolated"
    );
    assert_telegram_entries_unavailable(&host);
    assert_eq!(
        fs::read(&credential).expect("switched Native credential after Telegram refusal"),
        switched_credential
    );
    assert_eq!(
        social_sqlite_image(&host),
        before,
        "account change altered retained Telegram work"
    );

    let returned_credential = replace_native_credential(&credential, PRODUCTION, "123");
    assert_eq!(auth(&host)["loggedIn"], true);
    assert_eq!(
        host.mini_app_account_port()
            .current_scope()
            .expect("returned Native scope")
            .account_id,
        "123"
    );
    assert!(
        contains_bot(&host, BOT_TELEGRAM),
        "original account Agent remains durable"
    );
    assert_telegram_entries_unavailable(&host);
    assert_eq!(
        fs::read(&credential).expect("returned Native credential after Telegram refusal"),
        returned_credential
    );
    assert_eq!(
        social_sqlite_image(&host),
        before,
        "account return altered retained Telegram work"
    );

    fs::remove_file(&credential).expect("Native logout fixture");
    assert_eq!(auth(&host)["loggedIn"], false);
    assert_telegram_entries_unavailable(&host);
    assert!(
        matches!(fs::symlink_metadata(&credential), Err(error)
        if error.kind() == std::io::ErrorKind::NotFound),
        "logged-out Telegram refusal must not recreate a Native credential"
    );
    assert_eq!(auth(&host)["loggedIn"], false);
    assert_eq!(
        social_sqlite_image(&host),
        before,
        "logged-out refusal altered retained Telegram work"
    );
    assert!(
        !root
            .join("app-host/feature-host/account-session.json")
            .exists()
    );
    drop(host);
}

#[test]
fn native_telegram_owner_preserves_sqlite_turns_and_refuses_bypass() {
    let root = test_directory("telegram-owner");
    for mode in ["telegram-first", "telegram-restart", "telegram-repeat"] {
        run_child(&root.0, mode);
    }
}
