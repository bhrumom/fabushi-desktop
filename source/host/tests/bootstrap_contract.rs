use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use mahayana_host_runtime::gateway_config::{
    GatewayConfigError, gateway_scheme, is_loopback_host, resolve_gateway_server_config_with,
};
use mahayana_host_runtime::host_discovery::{
    GatewayDiscoveryInfo, clear_gateway_discovery, write_gateway_discovery,
};
use mahayana_host_runtime::host_paths::{
    SAND_BOX_DATA_ROOT, SAND_BOX_MODEL_VISIBLE_DATA_ROOT, get_sand_root_dir_with,
    read_user_data_dir_arg, resolve_sand_data_root_override, to_model_visible_path,
};
use uuid::Uuid;

fn temp_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("fabushi-host-bootstrap-{name}-{}", Uuid::new_v4()))
}

#[test]
fn gateway_config_matches_grok_auth_tls_and_loopback_rules() {
    assert!(is_loopback_host("127.0.0.1"));
    assert!(is_loopback_host(" LOCALHOST "));
    assert!(is_loopback_host("::1"));
    assert!(!is_loopback_host("0.0.0.0"));

    let mut env = BTreeMap::new();
    let local = resolve_gateway_server_config_with(&env, || "unused".into()).unwrap();
    assert_eq!(local.host, "127.0.0.1");
    assert_eq!(local.port, None);
    assert_eq!(local.auth_token, None);
    assert_eq!(gateway_scheme(&local), "http");

    env.insert("SAND_GATEWAY_BIND_HOST".into(), "0.0.0.0".into());
    env.insert("SAND_HOST_PORT".into(), "7777".into());
    let remote =
        resolve_gateway_server_config_with(&env, || "generated-token".into()).unwrap();
    assert_eq!(remote.port, Some(7777));
    assert_eq!(remote.auth_token.as_deref(), Some("generated-token"));

    env.insert("SAND_GATEWAY_TOKEN".into(), "pinned".into());
    let pinned = resolve_gateway_server_config_with(&env, || "wrong".into()).unwrap();
    assert_eq!(pinned.auth_token.as_deref(), Some("pinned"));

    let root = temp_dir("tls");
    fs::create_dir_all(&root).unwrap();
    let cert = root.join("cert.pem");
    let key = root.join("key.pem");
    fs::write(&cert, b"cert-bytes").unwrap();
    fs::write(&key, b"key-bytes").unwrap();

    let mut tls_env = BTreeMap::new();
    tls_env.insert(
        "SAND_GATEWAY_TLS_CERT".into(),
        cert.to_string_lossy().into_owned(),
    );
    assert!(matches!(
        resolve_gateway_server_config_with(&tls_env, || "token".into()),
        Err(GatewayConfigError::MissingTlsPair)
    ));
    tls_env.insert(
        "SAND_GATEWAY_TLS_KEY".into(),
        key.to_string_lossy().into_owned(),
    );
    let tls = resolve_gateway_server_config_with(&tls_env, || "token".into()).unwrap();
    assert_eq!(gateway_scheme(&tls), "https");
    assert_eq!(tls.tls.as_ref().unwrap().cert, b"cert-bytes");
    assert_eq!(tls.tls.as_ref().unwrap().key, b"key-bytes");

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn host_paths_match_grok_override_variant_and_model_visibility_rules() {
    let home = Path::new("/Users/example");
    let cwd = Path::new("/work");
    let mut env = BTreeMap::new();

    assert_eq!(
        get_sand_root_dir_with(home, &[], &env, cwd),
        home.join(".cursor").join("sand-dev")
    );

    env.insert("SAND_PACKAGED".into(), "1".into());
    assert_eq!(
        get_sand_root_dir_with(home, &[], &env, cwd),
        home.join(".grokbot")
    );

    env.insert("SAND_LAB".into(), "1".into());
    assert_eq!(
        get_sand_root_dir_with(home, &[], &env, cwd),
        home.join(".cursor").join("sand-lab")
    );

    let argv = vec!["host".to_string(), "--user-data-dir=profile".to_string()];
    assert_eq!(read_user_data_dir_arg(&argv).as_deref(), Some("profile"));
    assert_eq!(
        get_sand_root_dir_with(home, &argv, &env, cwd),
        cwd.join("profile").join("sand-data")
    );

    env.insert("SAND_DATA_ROOT".into(), "/tmp/grok-root".into());
    assert_eq!(
        resolve_sand_data_root_override(&env),
        Some(PathBuf::from("/tmp/grok-root"))
    );
    assert_eq!(
        get_sand_root_dir_with(home, &argv, &env, cwd),
        PathBuf::from("/tmp/grok-root")
    );

    let inside = Path::new(SAND_BOX_DATA_ROOT).join("agents/a.json");
    assert_eq!(
        to_model_visible_path(&inside),
        Path::new(SAND_BOX_MODEL_VISIBLE_DATA_ROOT).join("agents/a.json")
    );
    assert_eq!(
        to_model_visible_path(Path::new("/tmp/outside")),
        PathBuf::from("/tmp/outside")
    );
}

#[test]
fn gateway_discovery_is_validated_written_and_cleared() {
    let root = temp_dir("discovery");
    let path = root.join("gateway.json");
    let info = GatewayDiscoveryInfo {
        port: 4242,
        pid: 99,
        started_at: 123_456,
        scheme: Some("http".into()),
        host: Some("127.0.0.1".into()),
        token: Some("secret".into()),
    };

    write_gateway_discovery(&info, &path).unwrap();
    let decoded: GatewayDiscoveryInfo =
        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(decoded, info);
    assert!(
        fs::read_dir(&root)
            .unwrap()
            .flatten()
            .all(|entry| !entry.file_name().to_string_lossy().ends_with(".tmp"))
    );

    clear_gateway_discovery(&path).unwrap();
    assert!(!path.exists());
    clear_gateway_discovery(&path).unwrap();

    let invalid = GatewayDiscoveryInfo {
        port: 0,
        ..info
    };
    assert!(write_gateway_discovery(&invalid, &path).is_err());
    fs::remove_dir_all(root).unwrap();
}


#[test]
fn host_lock_takes_over_only_known_host_processes() {
    use std::cell::{Cell, RefCell};
    use mahayana_host_runtime::host_lock::{
        HostLockOutcome, acquire_host_lock_with, read_lock_pid,
    };

    let root = temp_dir("lock");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("host.lock");
    fs::write(&path, "41").unwrap();

    let alive = Cell::new(true);
    let signals = RefCell::new(Vec::<(u32, bool)>::new());
    let acquired = acquire_host_lock_with(
        &path,
        99,
        |_| alive.get(),
        |_| true,
        |pid, force| {
            signals.borrow_mut().push((pid, force));
            alive.set(false);
        },
        |_| {},
        100,
        10,
    )
    .unwrap();
    assert_eq!(acquired.outcome, HostLockOutcome::TookOver);
    assert_eq!(acquired.previous_pid, Some(41));
    assert_eq!(signals.borrow().as_slice(), &[(41, false)]);
    assert_eq!(read_lock_pid(&path), Some(99));
    acquired.lock.release().unwrap();
    assert!(!path.exists());

    fs::write(&path, "42").unwrap();
    let signals = RefCell::new(Vec::<(u32, bool)>::new());
    let foreign = acquire_host_lock_with(
        &path,
        100,
        |_| true,
        |_| false,
        |pid, force| signals.borrow_mut().push((pid, force)),
        |_| {},
        100,
        10,
    )
    .unwrap();
    assert_eq!(foreign.outcome, HostLockOutcome::ReclaimedForeign);
    assert_eq!(foreign.previous_pid, Some(42));
    assert!(signals.borrow().is_empty());
    assert_eq!(read_lock_pid(&path), Some(100));
    foreign.lock.release().unwrap();

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn host_lock_refuses_a_known_host_that_survives_both_termination_attempts() {
    use std::cell::RefCell;
    use std::io::ErrorKind;
    use mahayana_host_runtime::host_lock::{acquire_host_lock_with, read_lock_pid};

    let root = temp_dir("lock-live-refusal");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("host.lock");
    let original = b"41\n";
    fs::write(&path, original).unwrap();
    let signals = RefCell::new(Vec::<(u32, bool)>::new());

    let error = acquire_host_lock_with(
        &path,
        99,
        |pid| {
            assert_eq!(pid, 41);
            true
        },
        |pid| {
            assert_eq!(pid, 41);
            true
        },
        |pid, force| signals.borrow_mut().push((pid, force)),
        |_| {},
        20,
        10,
    )
    .expect_err("a surviving Host must prevent another Host from starting");

    assert_eq!(error.kind(), ErrorKind::TimedOut);
    assert_eq!(signals.borrow().as_slice(), &[(41, false), (41, true)]);
    assert_eq!(fs::read(&path).unwrap().as_slice(), original);
    assert_eq!(read_lock_pid(&path), Some(41));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn host_lock_accepts_forced_takeover_only_after_the_host_stops_during_polling() {
    use std::cell::{Cell, RefCell};
    use mahayana_host_runtime::host_lock::{
        HostLockOutcome, acquire_host_lock_with, read_lock_pid,
    };

    let root = temp_dir("lock-force-poll");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("host.lock");
    fs::write(&path, "41").unwrap();
    let alive = Cell::new(true);
    let force_requested = Cell::new(false);
    let forced_delays = Cell::new(0usize);
    let signals = RefCell::new(Vec::<(u32, bool)>::new());

    let acquired = acquire_host_lock_with(
        &path,
        99,
        |pid| {
            assert_eq!(pid, 41);
            alive.get()
        },
        |pid| {
            assert_eq!(pid, 41);
            true
        },
        |pid, force| {
            signals.borrow_mut().push((pid, force));
            if force {
                force_requested.set(true);
            }
        },
        |_| {
            if force_requested.get() {
                forced_delays.set(forced_delays.get() + 1);
                if forced_delays.get() == 2 {
                    alive.set(false);
                }
            }
        },
        30,
        10,
    )
    .unwrap();

    assert!(!alive.get());
    assert_eq!(forced_delays.get(), 2);
    assert_eq!(signals.borrow().as_slice(), &[(41, false), (41, true)]);
    assert_eq!(acquired.outcome, HostLockOutcome::TookOver);
    assert_eq!(acquired.previous_pid, Some(41));
    assert_eq!(read_lock_pid(&path), Some(99));
    acquired.lock.release().unwrap();
    assert!(!path.exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn host_lock_exhaustion_preserves_a_contender_that_replaces_each_observed_holder() {
    use std::cell::RefCell;
    use std::io::ErrorKind;
    use mahayana_host_runtime::host_lock::{
        MAX_ACQUIRE_ATTEMPTS, acquire_host_lock_with, read_lock_pid,
    };

    let root = temp_dir("lock-contended");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("host.lock");
    fs::write(&path, "41").unwrap();
    let observed = RefCell::new(Vec::<u32>::new());

    let error = acquire_host_lock_with(
        &path,
        99,
        |pid| {
            observed.borrow_mut().push(pid);
            assert_eq!(read_lock_pid(&path), Some(pid));
            // A contender publishes a different holder while the old PID is
            // being probed. The old observation cannot authorize its removal.
            fs::write(&path, (pid + 1).to_string()).unwrap();
            false
        },
        |_| panic!("a dead-holder probe does not need process classification"),
        |_, _| panic!("no observed holder is live"),
        |_| panic!("no termination polling is needed"),
        20,
        10,
    )
    .expect_err("contention must not bypass exclusive creation at the retry limit");

    assert_eq!(error.kind(), ErrorKind::WouldBlock);
    assert_eq!(observed.borrow().len(), MAX_ACQUIRE_ATTEMPTS);
    let last_contender = 41 + MAX_ACQUIRE_ATTEMPTS as u32;
    assert_eq!(read_lock_pid(&path), Some(last_contender));
    assert_eq!(fs::read_to_string(&path).unwrap(), last_contender.to_string());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn host_lock_preserves_fresh_dead_same_pid_and_malformed_pid_acquisition() {
    use mahayana_host_runtime::host_lock::{
        HostLockOutcome, acquire_host_lock_with, read_lock_pid,
    };

    for (name, stored_pid, expected_outcome, expected_previous_pid) in [
        ("fresh", None, HostLockOutcome::Created, None),
        ("dead", Some("41"), HostLockOutcome::ReclaimedDead, Some(41)),
        ("same", Some("99"), HostLockOutcome::Created, None),
        ("malformed", Some("invalid"), HostLockOutcome::ReclaimedDead, None),
    ] {
        let root = temp_dir(&format!("lock-control-{name}"));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("host.lock");
        if let Some(stored_pid) = stored_pid {
            fs::write(&path, stored_pid).unwrap();
        }

        let acquired = acquire_host_lock_with(
            &path,
            99,
            |pid| {
                assert_eq!(Some(pid), expected_previous_pid);
                false
            },
            |_| panic!("no live holder requires process classification"),
            |_, _| panic!("no live holder may be signalled"),
            |_| panic!("no termination polling is needed"),
            20,
            10,
        )
        .unwrap();

        assert_eq!(acquired.outcome, expected_outcome);
        assert_eq!(acquired.previous_pid, expected_previous_pid);
        assert_eq!(read_lock_pid(&path), Some(99));
        acquired.lock.release().unwrap();
        assert!(!path.exists());
        fs::remove_dir_all(root).unwrap();
    }
}
