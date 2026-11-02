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

mod physical_custody {
    use super::*;
    use std::cell::{Cell, RefCell};
    use std::fs::{File, OpenOptions};
    use std::io::{self, Read, Write};
    use std::process::{Child, Command, ExitStatus, Stdio};
    use std::thread;
    use std::time::{Duration, Instant};
    use mahayana_host_runtime::host_lock::{
        HostLockOutcome, acquire_host_lock, acquire_host_lock_with, read_lock_pid,
    };

    fn guard_path(path: &Path) -> PathBuf {
        let mut value = path.as_os_str().to_os_string();
        value.push(".guard");
        PathBuf::from(value)
    }

    #[test]
    fn host_guard_refuses_busy_incomplete_and_untrusted_pid_projections() {
        for (name, projection, outcome, previous_pid) in [
            ("missing", None, HostLockOutcome::Created, None),
            ("empty", Some(""), HostLockOutcome::ReclaimedDead, None),
            ("malformed", Some("invalid\n"), HostLockOutcome::ReclaimedDead, None),
            ("same", Some("99\n"), HostLockOutcome::Created, None),
            ("dead", Some("41\n"), HostLockOutcome::ReclaimedDead, Some(41)),
            ("foreign", Some("42\n"), HostLockOutcome::ReclaimedForeign, Some(42)),
        ] {
            let root = temp_dir(&format!("guard-busy-{name}"));
            fs::create_dir_all(&root).unwrap();
            let path = root.join("host.lock");
            let physical_guard = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(guard_path(&path))
                .unwrap();
            fs2::FileExt::try_lock_exclusive(&physical_guard).unwrap();
            if let Some(projection) = projection {
                fs::write(&path, projection).unwrap();
            }
            let acquire = || acquire_host_lock_with(
                &path,
                99,
                |pid| {
                    assert_eq!(Some(pid), previous_pid, "{name}");
                    name == "foreign"
                },
                |pid| {
                    assert_eq!((name, pid), ("foreign", 42));
                    false
                },
                |_, _| panic!("{name}: unidentified/foreign holders must not be signalled"),
                |_| panic!("{name}: no known Host takeover needs polling"),
                20,
                10,
            );

            let error = acquire().expect_err("PID metadata cannot bypass physical custody");
            assert_eq!(error.kind(), io::ErrorKind::WouldBlock, "{name}");
            match projection {
                Some(expected) => {
                    assert_eq!(fs::read(&path).unwrap().as_slice(), expected.as_bytes(), "{name}");
                }
                None => assert!(!path.exists(), "{name}"),
            }
            assert!(guard_path(&path).is_file());

            // Another lock path has independent custody even while this one is busy.
            let other = acquire_host_lock(&root.join("other.lock"), std::process::id()).unwrap();
            other.lock.release().unwrap();
            fs2::FileExt::unlock(&physical_guard).unwrap();
            drop(physical_guard);

            let admitted = acquire().unwrap();
            assert_eq!(admitted.outcome, outcome, "{name}");
            assert_eq!(admitted.previous_pid, previous_pid, "{name}");
            assert_eq!(read_lock_pid(&path), Some(99));
            admitted.lock.release().unwrap();
            assert!(!path.exists());
            assert!(guard_path(&path).is_file());
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn host_guard_known_host_takeover_requires_actual_guard_release() {
        for release_guard in [false, true] {
            let root = temp_dir("guard-known-takeover");
            let path = root.join("host.lock");
            let holder = acquire_host_lock(&path, 41).unwrap();
            let alive = Cell::new(true);
            let signals = RefCell::new(Vec::new());
            let contender = acquire_host_lock_with(
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
                    alive.set(false);
                    if release_guard {
                        holder.lock.release().unwrap();
                    }
                },
                |_| panic!("the deterministic holder stops immediately"),
                20,
                10,
            );
            assert_eq!(signals.borrow().as_slice(), &[(41, false)]);
            assert!(!alive.get());
            let successor = if release_guard {
                let admitted = contender.unwrap();
                assert_eq!(admitted.outcome, HostLockOutcome::TookOver);
                assert_eq!(admitted.previous_pid, Some(41));
                admitted
            } else {
                assert_eq!(contender.unwrap_err().kind(), io::ErrorKind::WouldBlock);
                assert_eq!(fs::read_to_string(&path).unwrap(), "41");
                holder.lock.release().unwrap();
                acquire_host_lock(&path, 99).unwrap()
            };
            // Releasing/dropping the former owner cannot disturb its successor.
            holder.lock.release().unwrap();
            drop(holder);
            assert_eq!(read_lock_pid(&path), Some(99));
            assert_eq!(
                acquire_host_lock(&path, 99).unwrap_err().kind(),
                io::ErrorKind::WouldBlock,
            );
            successor.lock.release().unwrap();
            assert!(guard_path(&path).is_file());
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn host_guard_release_is_idempotent_and_preserves_same_pid_successors() {
        let root = temp_dir("guard-release");
        let path = root.join("host.lock");
        let pid = std::process::id();
        let first = acquire_host_lock(&path, pid).unwrap();
        #[cfg(unix)]
        let initial_identity = {
            use std::os::unix::fs::MetadataExt;
            let metadata = fs::metadata(guard_path(&path)).unwrap();
            (metadata.dev(), metadata.ino())
        };
        assert_eq!(acquire_host_lock(&path, pid).unwrap_err().kind(), io::ErrorKind::WouldBlock);
        first.lock.release().unwrap();
        first.lock.release().unwrap();
        assert!(!path.exists());
        let second = acquire_host_lock(&path, pid).unwrap();
        first.lock.release().unwrap();
        drop(first);
        assert_eq!(read_lock_pid(&path), Some(pid));
        assert_eq!(acquire_host_lock(&path, pid).unwrap_err().kind(), io::ErrorKind::WouldBlock);
        second.lock.release().unwrap();
        let third = acquire_host_lock(&path, pid).unwrap();
        drop(second);
        assert_eq!(read_lock_pid(&path), Some(pid));
        assert_eq!(acquire_host_lock(&path, pid).unwrap_err().kind(), io::ErrorKind::WouldBlock);
        drop(third);
        assert!(!path.exists(), "ordinary Drop removes the owned projection");

        let changed_projection = acquire_host_lock(&path, pid).unwrap();
        fs::write(&path, "external-projection").unwrap();
        changed_projection.lock.release().unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "external-projection");
        let recovered = acquire_host_lock(&path, pid).unwrap();
        assert_eq!(recovered.outcome, HostLockOutcome::ReclaimedDead);
        drop(changed_projection);
        assert_eq!(read_lock_pid(&path), Some(pid));
        recovered.lock.release().unwrap();
        assert!(guard_path(&path).is_file());
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let metadata = fs::metadata(guard_path(&path)).unwrap();
            assert_eq!((metadata.dev(), metadata.ino()), initial_identity);
        }
        fs::remove_dir_all(root).unwrap();
    }

    const CHILD_ROOT: &str = "FABUSHI_BOOTSTRAP_PHYSICAL_CUSTODY_ROOT";
    const CHILD_NONCE: &str = "FABUSHI_BOOTSTRAP_PHYSICAL_CUSTODY_NONCE";
    const CHILD_DEADLINE: Duration = Duration::from_secs(20);

    struct OwnedGuardChild {
        child: Child,
        root: PathBuf,
        terminal: Option<ExitStatus>,
    }

    impl OwnedGuardChild {
        fn poll(&mut self) -> io::Result<Option<ExitStatus>> {
            if self.terminal.is_none() {
                self.terminal = self.child.try_wait()?;
            }
            Ok(self.terminal)
        }

        fn wait_terminal(&mut self, timeout: Duration) -> io::Result<ExitStatus> {
            let deadline = Instant::now() + timeout;
            loop {
                if let Some(status) = self.poll()? {
                    return Ok(status);
                }
                if Instant::now() >= deadline {
                    return Err(io::Error::new(io::ErrorKind::TimedOut, "owned child exit deadline"));
                }
                // Pacing only: only try_wait's terminal result proves exit.
                thread::sleep(Duration::from_millis(10));
            }
        }

        fn stop_and_reap(&mut self) -> io::Result<ExitStatus> {
            let initial_poll = self.poll();
            if let Ok(Some(status)) = &initial_poll {
                return Ok(*status);
            }
            // A wait error is not proof of exit. This exact Child remains the
            // cleanup authority; do not return before attempting its stop/reap.
            let kill_result = self.child.kill();
            let terminal = self.wait_terminal(Duration::from_secs(5));
            match terminal {
                Ok(status) => Ok(status),
                Err(error) => Err(io::Error::other(format!(
                    "owned child cleanup incomplete: initial_poll={initial_poll:?}; kill={kill_result:?}; wait={error}"
                ))),
            }
        }

        fn diagnostics(&self) -> String {
            format!(
                "pid={} root={} terminal={:?} readiness={:?}\nstdout:\n{}\nstderr:\n{}",
                self.child.id(), self.root.display(), self.terminal,
                fs::read_to_string(self.root.join("ready.json")),
                fs::read_to_string(self.root.join("child.stdout"))
                    .unwrap_or_else(|error| format!("<capture unavailable: {error}>")),
                fs::read_to_string(self.root.join("child.stderr"))
                    .unwrap_or_else(|error| format!("<capture unavailable: {error}>")),
            )
        }

        fn fail(&mut self, phase: &str, error: impl std::fmt::Display) -> ! {
            let cleanup = self.stop_and_reap();
            panic!("{phase}: {error}; cleanup={cleanup:?}; {}", self.diagnostics());
        }

        fn wait_ready(&mut self, nonce: &str) -> io::Result<()> {
            let deadline = Instant::now() + CHILD_DEADLINE;
            loop {
                if let Some(status) = self.poll()? {
                    return Err(io::Error::other(format!("child exited before readiness: {status}")));
                }
                match fs::read(self.root.join("ready.json")) {
                    Ok(bytes) => {
                        let (reported_nonce, reported_pid): (String, u32) =
                            serde_json::from_slice(&bytes).map_err(io::Error::other)?;
                        if reported_nonce != nonce || reported_pid != self.child.id() {
                            return Err(io::Error::other("child readiness nonce/PID mismatch"));
                        }
                        return Ok(());
                    }
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error),
                }
                if Instant::now() >= deadline {
                    return Err(io::Error::new(io::ErrorKind::TimedOut, "owned child readiness deadline"));
                }
                // The atomic PID/nonce receipt, not elapsed time, proves readiness.
                thread::sleep(Duration::from_millis(10));
            }
        }
    }

    impl Drop for OwnedGuardChild {
        fn drop(&mut self) {
            let cleanup = self.stop_and_reap();
            if cleanup.is_err() || thread::panicking() {
                eprintln!("Host custody child cleanup={cleanup:?}; {}", self.diagnostics());
            }
            // The caller removes the root only after confirmed exit and release.
            // Any failure leaves its readiness receipt and output available.
        }
    }

    #[test]
    fn host_guard_public_owner_lasts_until_owned_child_exit() {
        if let Some(root) = std::env::var_os(CHILD_ROOT) {
            let root = PathBuf::from(root);
            let nonce = std::env::var(CHILD_NONCE).unwrap();
            let owner = acquire_host_lock(&root.join("host.lock"), std::process::id()).unwrap();
            let ready = serde_json::to_vec(&(nonce, std::process::id())).unwrap();
            fs::write(root.join("ready.tmp"), ready).unwrap();
            fs::rename(root.join("ready.tmp"), root.join("ready.json")).unwrap();
            let mut command = [0u8; 1];
            io::stdin().read_exact(&mut command).unwrap();
            assert_eq!(command, [b'x']);
            assert_eq!(owner.lock.pid(), std::process::id());
            // Deliberately bypass Rust Drop. The OS must release the retained
            // guard on this exact process's exit; its stale PID remains on disk.
            std::process::exit(0);
        }

        let root = temp_dir("guard-real-child");
        fs::create_dir_all(&root).unwrap();
        let nonce = Uuid::new_v4().to_string();
        let process = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "physical_custody::host_guard_public_owner_lasts_until_owned_child_exit",
                "--nocapture",
                "--test-threads=1",
            ])
            .env(CHILD_ROOT, &root)
            .env(CHILD_NONCE, &nonce)
            .stdin(Stdio::piped())
            .stdout(Stdio::from(File::create(root.join("child.stdout")).unwrap()))
            .stderr(Stdio::from(File::create(root.join("child.stderr")).unwrap()))
            .spawn()
            .unwrap();
        let mut child = OwnedGuardChild {
            child: process,
            root: root.clone(),
            terminal: None,
        };
        if let Err(error) = child.wait_ready(&nonce) {
            child.fail("readiness", error);
        }
        let path = root.join("host.lock");
        let child_pid = child.child.id();
        assert_eq!(read_lock_pid(&path), Some(child_pid));
        // Exercise the real guard without classifying a test executable as a
        // shipping Host. No callback can terminate or manufacture child exit.
        let error = acquire_host_lock_with(
            &path,
            std::process::id(),
            |pid| {
                assert_eq!(pid, child_pid);
                true
            },
            |pid| {
                assert_eq!(pid, child_pid);
                false
            },
            |_, _| panic!("the live test child is not a known Host"),
            |_| panic!("no takeover is permitted"),
            20,
            10,
        )
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::WouldBlock);
        assert_eq!(read_lock_pid(&path), Some(child_pid));
        match child.poll() {
            Ok(None) => {}
            result => child.fail("holder must remain alive after contention", format!("{result:?}")),
        }
        let command_result = child.child.stdin.take().unwrap().write_all(b"x");
        if let Err(error) = command_result {
            child.fail("request exact child exit", error);
        }
        let status = match child.wait_terminal(CHILD_DEADLINE) {
            Ok(status) => status,
            Err(error) => child.fail("wait for exact child exit", error),
        };
        assert!(status.success(), "{}", child.diagnostics());
        assert_eq!(read_lock_pid(&path), Some(child_pid), "exit did not run Rust Drop");
        let successor = acquire_host_lock(&path, std::process::id()).unwrap();
        assert_eq!(successor.outcome, HostLockOutcome::ReclaimedDead);
        assert_eq!(successor.previous_pid, Some(child_pid));
        assert_eq!(read_lock_pid(&path), Some(std::process::id()));
        successor.lock.release().unwrap();
        assert!(!path.exists());
        assert!(guard_path(&path).is_file());
        drop(child);
        fs::remove_dir_all(root).unwrap();
    }
}
