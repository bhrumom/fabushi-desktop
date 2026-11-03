use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;
use std::time::Duration;

use mahayana_host_runtime::r#box::box_remote_accessor::{
    BoxEndpoint, ping_box_transport_classified,
};
use mahayana_host_runtime::r#box::exec_daemon_process::{
    BoxExecDaemonProcessOptions, box_exec_daemon_binary_name,
    resolve_box_exec_daemon_executable, start_box_exec_daemon_process,
};
use mahayana_host_runtime::r#box::generated_production::{
    ProductionBoxTransport, create_production_box_control_client,
};

fn free_loopback_port() -> u16 {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind free port");
    listener.local_addr().expect("local addr").port()
}

#[test]
fn sibling_binary_resolution_matches_packaged_resources_bin_contract() {
    let host = if cfg!(windows) {
        PathBuf::from(r"C:\Fabushi\resources\bin\mahayana-app-host.exe")
    } else {
        PathBuf::from("/Applications/Fabushi.app/Contents/Resources/bin/mahayana-app-host")
    };
    let expected = host
        .parent()
        .expect("host parent")
        .join(box_exec_daemon_binary_name());
    assert_eq!(
        resolve_box_exec_daemon_executable(&host).expect("resolved sibling"),
        expected
    );
}

#[test]
fn supervisor_rejects_contaminated_port_before_spawning() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("listener");
    let port = listener.local_addr().expect("addr").port();
    let root = std::env::temp_dir().join(format!(
        "fabushi-box-exec-supervisor-contaminated-{}",
        uuid::Uuid::new_v4()
    ));
    let result = start_box_exec_daemon_process(BoxExecDaemonProcessOptions {
        executable_path: root.join("does-not-matter"),
        host: "127.0.0.1".into(),
        port,
        auth_token: "token".into(),
        workspace_root: root.join("workspace"),
        terminals_directory: root.join("terminals"),
        start_timeout_ms: 500,
        stop_timeout_ms: 500,
    });
    let error = match result {
        Ok(mut daemon) => {
            let _ = daemon.close();
            panic!("contaminated startup unexpectedly succeeded");
        }
        Err(error) => error,
    };
    assert!(error.contains("already bound"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shipping_supervisor_spawns_authenticates_and_closes_real_daemon_when_provided() {
    let Some(executable) = std::env::var_os("FABUSHI_BOX_EXEC_DAEMON_TEST_BIN") else {
        eprintln!(
            "FABUSHI_BOX_EXEC_DAEMON_TEST_BIN not set; exact-head workflow runs this contract explicitly"
        );
        return;
    };
    let executable = PathBuf::from(executable);
    assert!(
        executable.is_file(),
        "daemon test binary missing: {}",
        executable.display()
    );

    let root = std::env::temp_dir().join(format!(
        "fabushi-box-exec-supervisor-{}",
        uuid::Uuid::new_v4()
    ));
    let port = free_loopback_port();
    let auth_token = format!("test-{}", uuid::Uuid::new_v4());
    let mut owned = start_box_exec_daemon_process(BoxExecDaemonProcessOptions {
        executable_path: executable.clone(),
        host: "127.0.0.1".into(),
        port,
        auth_token: auth_token.clone(),
        workspace_root: root.join("workspace"),
        terminals_directory: root.join("terminals"),
        start_timeout_ms: 5_000,
        stop_timeout_ms: 2_000,
    })
    .expect("start real daemon");
    assert!(owned.pid > 0);
    assert_eq!(owned.executable_path, executable);

    let endpoint = BoxEndpoint::new("127.0.0.1", port, auth_token);
    let transport = ProductionBoxTransport::from_endpoint(&endpoint);
    let ping = ping_box_transport_classified(
        &(),
        &transport,
        create_production_box_control_client,
        500,
    );
    assert_eq!(ping.outcome, "ok");

    owned.close().expect("graceful daemon shutdown");
    std::thread::sleep(Duration::from_millis(50));
    assert!(
        TcpListener::bind(("127.0.0.1", port)).is_ok(),
        "daemon port must be released after close"
    );
    let _ = fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
#[ignore = "requires the actual daemon binary built by the exact-H Actions lane"]
fn box_exec_response_roundtrip_uses_actual_daemon_identity_and_completion() {
    use mahayana_host_runtime::r#box::box_file_transfer::{FileTransferAccessor, WriteExecResult};
    use mahayana_host_runtime::r#box::box_shell_command::{HostShellArgsInput, build_host_shell_args};
    use mahayana_host_runtime::r#box::generated_production::{
        ProductionReadArgs, ProductionReadOutput, ProductionReadResult,
        ProductionShellResult, create_production_box_resource_accessor,
    };

    let executable = PathBuf::from(
        std::env::var_os("FABUSHI_BOX_EXEC_DAEMON_TEST_BIN")
            .expect("this explicitly selected live contract requires the real daemon binary"),
    );
    assert!(executable.is_file(), "real daemon executable is absent");
    let root = std::env::temp_dir().join(format!(
        "fabushi-box-response-roundtrip-{}",
        uuid::Uuid::new_v4(),
    ));
    let workspace = root.join("workspace");
    fs::create_dir_all(&workspace).expect("private fixture workspace");
    let port = free_loopback_port();
    let auth_token = format!("bxr-test-{}", uuid::Uuid::new_v4());
    let mut owned = start_box_exec_daemon_process(BoxExecDaemonProcessOptions {
        executable_path: executable,
        host: "127.0.0.1".into(),
        port,
        auth_token: auth_token.clone(),
        workspace_root: workspace.clone(),
        terminals_directory: root.join("terminals"),
        start_timeout_ms: 5_000,
        stop_timeout_ms: 2_000,
    }).expect("start the actual owned daemon");
    assert!(owned.pid > 0);

    let endpoint = BoxEndpoint::new("127.0.0.1", port, auth_token);
    let transport = ProductionBoxTransport::from_endpoint(&endpoint);
    let mut accessor = create_production_box_resource_accessor(&transport);

    // The same real accessor starts at proto3 id0 and then uses nonzero IDs.
    // Every returned success passes the production identity, close, Connect and
    // chunked-HTTP checks against the actual daemon producer.
    let shell = accessor.execute_shell_result(
        &(),
        build_host_shell_args(HostShellArgsInput {
            command: "printf bxr-real-daemon".into(),
            name: "printf".into(),
            working_directory: "/workspace".into(),
            tool_call_id: "bxr-live-shell".into(),
        }),
    ).expect("actual default-zero Shell response completed");
    assert_eq!(
        shell,
        ProductionShellResult::Success {
            exit_code: 0,
            stdout: "bxr-real-daemon".into(),
            stderr: String::new(),
        },
    );

    let binary = vec![0, 0xff, b'B', b'X', b'R', b'\n', 0x80];
    assert_eq!(
        accessor.execute_write(&(), "/workspace/roundtrip.bin", &binary, "bxr-live-write")
            .expect("actual nonzero Write response completed"),
        WriteExecResult::Success,
    );
    assert_eq!(fs::read(workspace.join("roundtrip.bin")).expect("real file bytes"), binary);
    let read = accessor.execute_read(&(), ProductionReadArgs {
        path: "/workspace/roundtrip.bin".into(),
        tool_call_id: "bxr-live-read".into(),
        offset: None,
        limit: None,
        encoding_hint: None,
    }).expect("actual nonzero binary Read response completed");
    match read {
        ProductionReadResult::Success { output: ProductionReadOutput::Data(bytes), .. } => {
            assert_eq!(bytes, binary);
        }
        other => panic!("binary output was lost or lossily decoded: {other:?}"),
    }

    assert_eq!(
        accessor.execute_write(&(), "/workspace/roundtrip.txt", b"readable", "bxr-live-text-write")
            .expect("actual text Write completed"),
        WriteExecResult::Success,
    );
    let text = accessor.execute_read(&(), ProductionReadArgs {
        path: "/workspace/roundtrip.txt".into(),
        tool_call_id: "bxr-live-text-read".into(),
        offset: None,
        limit: None,
        encoding_hint: None,
    }).expect("actual text Read completed");
    match text {
        ProductionReadResult::Success { output: ProductionReadOutput::Content(text), .. } => {
            assert_eq!(text, "readable");
        }
        other => panic!("text output changed representation: {other:?}"),
    }

    owned.close().expect("close only the daemon owned by this fixture");
    fs::remove_dir_all(&root).expect("remove only the owned fixture directory");
}


#[cfg(unix)]
mod foreground_capture {
    use std::path::PathBuf;
    use std::process::{Child, Command, Stdio};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;
    use std::time::{Duration, Instant};

    use mahayana_host_runtime::extensions::box_lifecycle::RecreateSandBoxResponse;
    use mahayana_host_runtime::extensions::forever_box::{
        ForeverBoxLifecycle, ForeverBoxRunnerResourcePort, ForeverBoxService, HostBox,
    };
    use mahayana_host_runtime::extensions::inference::codex_direct_responses::{
        CodexDirectCheckpoint, CodexDirectUsage,
    };
    use mahayana_host_runtime::extensions::inference::provider_session::{
        ProviderSessionError, RoutedProviderCheckpoint, RoutedToolDefinition,
    };
    use mahayana_host_runtime::r#box::production::ProductionBoxEnvironment;
    use mahayana_host_runtime::runner::box_tool_access::{
        RUNNER_BOX_SHELL_TOOL_NAME, RunnerBoxToolBridge,
    };
    use mahayana_host_runtime::runner::production_turn_run_shell_adapter::{
        ProductionTurnRunShellAdapter, ProviderRetryOutcome, RoutedProviderAttemptControl,
        RoutedProviderAttemptExecutor, ToolEffectCustodySnapshot, ToolEffectUnknownResult,
    };
    use mahayana_host_runtime::runner::routed_provider_runtime::{
        ProductionRoutedProviderCheckpointStore, RoutedProviderCancellation, RoutedToolBridge,
    };
    use mahayana_host_runtime::runner::tools::mcp_meta_tools::execute_routed_tool_with_timeout_ms;
    use mahayana_host_runtime::runner::{StreamAttemptPolicy, StreamAttemptRuntime};
    use serde_json::{Value, json};

    use super::{fs, BoxEndpoint, ProductionBoxTransport, create_production_box_control_client,
        ping_box_transport_classified};

    const PREFIX: &str = "SC_CAPTURE_PREFIX";
    const BOUND: Duration = Duration::from_secs(10);

    // This starts the actual daemon compiled with cfg(test), whose private
    // per-runtime operation fault is selected by its ignored child test. It is
    // not the normal shipping daemon, and no production fault option is added.
    struct ProbeDaemon {
        child: Child,
        root: PathBuf,
        port: u16,
        auth: String,
        finished: bool,
    }

    impl ProbeDaemon {
        fn start(mode: &str) -> Self {
            let binary = PathBuf::from(std::env::var_os(
                "FABUSHI_BOX_EXEC_DAEMON_CAPTURE_TEST_BIN",
            ).expect("exact-H capture must supply its bound actual daemon test executable"));
            assert!(binary.is_file(), "actual daemon test executable is absent");
            let root = std::env::temp_dir().join(format!(
                "fabushi-box-shell-capture-{}", uuid::Uuid::new_v4(),
            ));
            fs::create_dir(&root).expect("owned probe root");
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
                .expect("private probe root");
            let auth = format!("sc-test-{}", uuid::Uuid::new_v4());
            let log = fs::File::create(root.join("child.log")).expect("owned child log");
            let stderr = log.try_clone().expect("owned child stderr");
            let child = Command::new(binary)
                .args([
                    "--exact", "server::tests::foreground_shell_capture_probe_child",
                    "--ignored", "--nocapture", "--test-threads=1",
                ])
                .env("FABUSHI_SC_PROBE_ROOT", &root)
                .env("FABUSHI_SC_PROBE_AUTH", &auth)
                .env("FABUSHI_SC_PROBE_FAULT", mode)
                .env("FABUSHI_SC_PROBE_PORT", "0")
                .stdin(Stdio::piped())
                .stdout(log)
                .stderr(stderr)
                .spawn()
                .expect("spawn only the owned actual daemon test child");
            let mut probe = Self { child, root, port: 0, auth, finished: false };
            let deadline = Instant::now() + BOUND;
            loop {
                if let Ok(bytes) = fs::read(probe.root.join("ready.json")) {
                    if let Ok(ready) = serde_json::from_slice::<Value>(&bytes) {
                        assert_eq!(ready["pid"].as_u64(), Some(u64::from(probe.child.id())));
                        assert_eq!(ready["host"], "127.0.0.1");
                        assert_eq!(ready["workspaceRoot"].as_str(),
                            Some(probe.root.join("workspace").to_str().expect("UTF-8 fixture path")));
                        let port = ready["port"].as_u64().expect("actual bound port");
                        probe.port = u16::try_from(port).expect("valid loopback port");
                        assert_ne!(probe.port, 0);
                        break;
                    }
                }
                if let Some(status) = probe.child.try_wait().expect("observe owned startup") {
                    panic!("actual daemon child exited before readiness: {status}; {}",
                        probe.log_tail());
                }
                assert!(Instant::now() < deadline, "actual daemon readiness exceeded bound: {}",
                    probe.log_tail());
                thread::sleep(Duration::from_millis(5));
            }
            let endpoint = BoxEndpoint::new("127.0.0.1", probe.port, probe.auth.clone());
            let transport = ProductionBoxTransport::from_endpoint(&endpoint);
            let ping = ping_box_transport_classified(
                &(), &transport, create_production_box_control_client, 1_000,
            );
            assert_eq!(ping.outcome, "ok", "actual authenticated readiness");
            probe
        }

        fn workspace(&self) -> PathBuf {
            self.root.join("workspace")
        }

        fn log_tail(&self) -> String {
            let bytes = fs::read(self.root.join("child.log")).unwrap_or_default();
            String::from_utf8_lossy(&bytes[bytes.len().saturating_sub(4_096)..]).into_owned()
        }

        fn bridge(&self) -> Arc<RunnerBoxToolBridge> {
            let service = Arc::new(ForeverBoxService::new(
                HostBox::new(ProductionBoxEnvironment::new(
                    "127.0.0.1", self.port, self.auth.clone(),
                )),
                Arc::new(UnusedLifecycle), false, false, false,
            ));
            Arc::new(RunnerBoxToolBridge::new(
                Arc::new(NoUpstream),
                Arc::new(ForeverBoxRunnerResourcePort::new(service, "sc-runner")),
            ))
        }

        fn assert_one_effect(&self) {
            assert_eq!(
                fs::read(self.workspace().join("effect-count")).expect("actual child effect"),
                b"effect\n",
                "the executed command must neither disappear nor replay",
            );
        }

        fn finish(&mut self) {
            drop(self.child.stdin.take());
            let deadline = Instant::now() + BOUND;
            let status = loop {
                if let Some(status) = self.child.try_wait().expect("observe owned shutdown") {
                    break status;
                }
                assert!(Instant::now() < deadline, "actual child shutdown exceeded bound: {}",
                    self.log_tail());
                thread::sleep(Duration::from_millis(5));
            };
            assert!(status.success(), "actual daemon child failed: {status}; {}", self.log_tail());
            let stopped: Value = serde_json::from_slice(
                &fs::read(self.root.join("stopped.json")).expect("actual stop receipt"),
            ).expect("actual stop receipt JSON");
            assert_eq!(stopped["pid"].as_u64(), Some(u64::from(self.child.id())));
            assert_eq!(stopped["clean"], true);
            assert_eq!(stopped["foregroundCount"], 0, "cleanup must retain or reap actual ownership");
            self.finished = true;
        }
    }

    impl Drop for ProbeDaemon {
        fn drop(&mut self) {
            if !self.finished {
                drop(self.child.stdin.take());
                let deadline = Instant::now() + BOUND;
                loop {
                    match self.child.try_wait() {
                        Ok(Some(_)) => break,
                        _ if Instant::now() < deadline => thread::sleep(Duration::from_millis(5)),
                        _ => {
                            let _ = self.child.kill();
                            let _ = self.child.wait();
                            break;
                        }
                    }
                }
            }
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    struct UnusedLifecycle;
    impl ForeverBoxLifecycle for UnusedLifecycle {
        fn fetch_image_update_available(&self) -> Result<bool, String> { Ok(false) }
        fn recreate_in_box(
            &self, _preserve_data: bool, _force: Option<bool>,
        ) -> Result<RecreateSandBoxResponse, String> {
            Err("probe must never recreate its owned Box".into())
        }
    }

    struct NoUpstream;
    impl RoutedToolBridge for NoUpstream {
        fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
            Ok(Vec::new())
        }
        fn call_tool(
            &self, _tool: &RoutedToolDefinition, _args: Value, _id: &str,
        ) -> Result<Value, ProviderSessionError> {
            Err(ProviderSessionError::Tool("unexpected non-Box dispatch".into()))
        }
    }

    fn checkpoint(result: &Value) -> RoutedProviderCheckpoint {
        RoutedProviderCheckpoint::Fabushi(CodexDirectCheckpoint {
            model: Some("Vendor/CaseSensitive@ShellCapture".into()),
            input: vec![json!({"type":"function_call_output","call_id":"sc-tool","output":result})],
            text: String::new(),
            response_id: "sc-probe-response".into(),
            usage: CodexDirectUsage::default(),
            completed_steps: 1,
            tool_calls_completed: 1,
        })
    }

    struct ProbeAttempt {
        bridge: Arc<RunnerBoxToolBridge>,
        command: String,
        directory: String,
        expect_unknown: bool,
        attempts: usize,
        returned: Option<Result<Value, ProviderSessionError>>,
        custody: Option<ToolEffectCustodySnapshot>,
        later_dispatches: Arc<AtomicUsize>,
        checkpoint_calls: usize,
    }

    impl RoutedProviderAttemptExecutor for ProbeAttempt {
        fn run_attempt(
            &mut self,
            attempt: &RoutedProviderAttemptControl,
            resume_from: Option<&RoutedProviderCheckpoint>,
            _on_text_delta: &mut dyn FnMut(&str, &str),
            on_checkpoint: &mut dyn FnMut(&RoutedProviderCheckpoint) -> Result<(), ProviderSessionError>,
            _should_cancel: &dyn Fn() -> bool,
        ) -> Result<String, ProviderSessionError> {
            self.attempts += 1;
            assert!(resume_from.is_none(), "probe cannot resume a fabricated checkpoint");
            let tool = self.bridge.list_tools()?.into_iter()
                .find(|tool| tool.name == RUNNER_BOX_SHELL_TOOL_NAME)
                .expect("actual registered Box Shell");
            let args = json!({"command":self.command,"workingDirectory":self.directory});
            let bridge = Arc::clone(&self.bridge);
            let result = execute_routed_tool_with_timeout_ms(
                attempt, "Shell".into(), 10_000,
                move || bridge.call_tool(&tool, args, "sc-tool"),
            );
            self.custody = Some(attempt.effect_custody());
            self.returned = Some(result.clone());
            self.checkpoint_calls += 1;
            if self.expect_unknown {
                let error = result.expect_err("actual producer capture failure must stay an error");
                assert!(matches!(&error, ProviderSessionError::ToolOutcomeUnknown(_)),
                    "unknown must reach the real guard before its deadline: {error:?}");
                assert!(on_checkpoint(&checkpoint(&json!({"error":error.to_string()}))).is_err(),
                    "the production checkpoint gate must reject an unknown result");
                let later_dispatches = Arc::clone(&self.later_dispatches);
                let bridge = Arc::clone(&self.bridge);
                assert!(execute_routed_tool_with_timeout_ms(
                    attempt, "Shell".into(), 1_000, move || {
                        later_dispatches.fetch_add(1, Ordering::SeqCst);
                        let tool = bridge.list_tools()?.into_iter()
                            .find(|tool| tool.name == RUNNER_BOX_SHELL_TOOL_NAME).expect("Shell");
                        bridge.call_tool(&tool,
                            json!({"command":"printf 'next\\n' >> effect-count",
                                "workingDirectory":"/workspace"}), "sc-forbidden-next")
                    },
                ).is_err(), "further actual tool admission must fail");
                assert_eq!(self.later_dispatches.load(Ordering::SeqCst), 0);
                return Err(error);
            }
            let value = result?;
            on_checkpoint(&checkpoint(&value))?;
            Ok("completed".into())
        }
    }

    fn run_probe(probe: &ProbeDaemon, command: String, directory: &str, unknown: bool)
        -> (ProbeAttempt, ToolEffectCustodySnapshot, PathBuf)
    {
        let policy = StreamAttemptPolicy {
            first_output_timeout: Duration::from_secs(20),
            max_attempts: 2,
            initial_backoff: Duration::ZERO,
            max_backoff: Duration::ZERO,
            max_retry_after: Duration::ZERO,
        };
        let adapter = ProductionTurnRunShellAdapter {
            policy: policy.clone(),
            watchdog_poll_interval: Duration::from_millis(5),
        };
        let runtime = Arc::new(StreamAttemptRuntime::new(policy));
        let store = ProductionRoutedProviderCheckpointStore::new(
            &probe.root, "sc-runner", "sc-stream",
        );
        let mut attempt = ProbeAttempt {
            bridge: probe.bridge(), command, directory: directory.into(),
            expect_unknown: unknown, attempts: 0, returned: None, custody: None,
            later_dispatches: Arc::new(AtomicUsize::new(0)), checkpoint_calls: 0,
        };
        let mut retries = Vec::new();
        let mut reports = Vec::new();
        let result = adapter.run_with_attempt_runtime_reporting(
            Arc::clone(&runtime), &RoutedProviderCancellation::default(),
            &store, &mut attempt, &mut |_, _| {},
            &mut |event| retries.push(event.clone()),
            &mut |report| reports.push(report.clone()),
        );
        assert_eq!(attempt.attempts, 1, "the real provider retry owner must not replay");
        assert!(retries.is_empty());
        assert_eq!(attempt.checkpoint_calls, 1, "checkpoint gate actually exercised");
        let custody = runtime.effect_custody();
        assert_eq!(custody.pending_effects, 0, "the local guard worker has returned");
        assert_eq!(custody.started_revision, 1);
        if unknown {
            assert!(result.is_err());
            assert_eq!(custody.unknown_result, Some(ToolEffectUnknownResult::DownstreamOutcomeUnknown));
            assert_eq!(custody.checkpoint_revision, 0);
            assert_eq!(reports.len(), 1);
            assert_eq!(reports[0].outcome, ProviderRetryOutcome::GaveUpIneligible);
            assert_eq!(reports[0].error_code, "SAND-E0407");
            assert!(!reports[0].watchdog_expired, "producer uncertainty precedes the watchdog");
            assert!(!store.directory().exists(), "no unknown result reached actual checkpoint I/O");
            let replacement = RoutedProviderAttemptControl::new(
                Arc::clone(&runtime), runtime.begin_attempt(), RoutedProviderCancellation::default(),
            );
            assert!(replacement.admit_tool().is_err(), "replacement cannot erase existing custody");
        } else {
            assert_eq!(result.expect("definitive result"), "completed");
            assert_eq!(custody.unknown_result, None);
            assert_eq!(custody.checkpoint_revision, 1);
            assert!(reports.is_empty());
            let files = fs::read_dir(store.directory()).expect("real checkpoint directory")
                .map(|entry| entry.expect("checkpoint entry").path()).collect::<Vec<_>>();
            assert_eq!(files.len(), 1);
            let persisted: RoutedProviderCheckpoint = serde_json::from_slice(
                &fs::read(&files[0]).expect("real checkpoint bytes"),
            ).expect("actual checkpoint");
            assert_eq!(persisted, checkpoint(attempt.returned.as_ref().unwrap().as_ref().unwrap()));
        }
        let returned_custody = attempt.custody.as_ref().expect("actual guard observation");
        assert_eq!(returned_custody.started_revision, custody.started_revision);
        assert_eq!(returned_custody.pending_effects, 0);
        assert_eq!(returned_custody.unknown_result, custody.unknown_result);
        assert_eq!(returned_custody.checkpoint_revision, 0,
            "the guard observation was made before requesting a checkpoint");
        if unknown { assert_eq!(returned_custody, &custody); }
        (attempt, custody, store.directory().to_path_buf())
    }

    #[test]
    #[ignore = "requires the actual daemon test executable bound by exact-H Actions"]
    fn box_foreground_capture_faults_reach_real_runner_custody_without_replay() {
        for mode in ["stdout-read", "stderr-read", "stdout-worker", "stderr-worker",
            "stdout-wait", "stderr-wait"]
        {
            let mut probe = ProbeDaemon::start(mode);
            let redirect = if mode.starts_with("stderr-") { " >&2" } else { "" };
            let command = format!(
                "printf 'effect\\n' >> effect-count; printf '%s\\n' \"$$\" > child-pid; \
                 printf {PREFIX}{redirect}; sleep 30",
            );
            let (attempt, _, _) = run_probe(&probe, command, "/workspace", true);
            let error = attempt.returned.unwrap().unwrap_err();
            let message = error.to_string();
            let phase = if mode.ends_with("-read") {
                if mode.starts_with("stderr-") { "stderr read failed" } else { "stdout read failed" }
            } else if mode.ends_with("-worker") { "capture worker lost" } else { "child wait failed" };
            assert!(message.contains(&format!("Box Shell failed: foreground shell outcome unknown: {phase}")),
                "the actual producer Throw must survive Host projection: {message}");
            assert!(!message.contains(PREFIX), "capture payload is not a diagnostic");
            probe.assert_one_effect();
            let observed: Value = serde_json::from_slice(
                &fs::read(probe.root.join("fault-observed.json")).expect("real prefix fault witness"),
            ).expect("actual fault witness");
            assert_eq!(observed["mode"], mode);
            assert_eq!(observed["prefix"], PREFIX);
            assert_eq!(observed["stream"], if mode.starts_with("stderr-") { "stderr" } else { "stdout" });
            assert!(observed["bytesObserved"].as_u64().expect("actual captured byte count") >= PREFIX.len() as u64);
            let pid: i32 = fs::read_to_string(probe.workspace().join("child-pid"))
                .expect("actual child PID").trim().parse().expect("numeric owned child PID");
            assert!(pid > 1);
            assert_eq!(observed["childPid"].as_i64(), Some(i64::from(pid)));
            assert_eq!(unsafe { libc::kill(pid, 0) }, -1, "the actual child must have been reaped");
            assert_eq!(std::io::Error::last_os_error().raw_os_error(), Some(libc::ESRCH));
            probe.finish();
            probe.assert_one_effect();
        }
    }

    #[test]
    #[ignore = "requires the actual daemon test executable bound by exact-H Actions"]
    fn box_foreground_capture_definitive_results_remain_usable_and_checkpointed() {
        for (command, directory, expected, has_effect) in [
            (
                "printf 'effect\\n' >> effect-count; printf 'ok:\\377\\n'; printf 'stderr-complete' >&2",
                "/workspace",
                json!({"kind":"success","exitCode":0,"stdout":"ok:\u{fffd}\n","stderr":"stderr-complete"}),
                true,
            ),
            (
                "printf 'effect\\n' >> effect-count; printf 'out'; printf 'err' >&2; exit 7",
                "/workspace",
                json!({"kind":"failure","exitCode":7,"signal":"","stdout":"out","stderr":"err","aborted":false}),
                true,
            ),
            (
                "printf 'effect\\n' >> effect-count",
                "/workspace/absent-cwd",
                Value::Null,
                false,
            ),
        ] {
            let mut probe = ProbeDaemon::start("none");
            let (attempt, _, _) = run_probe(&probe, command.into(), directory, false);
            let value = attempt.returned.unwrap().expect("known result");
            if expected.is_null() {
                assert_eq!(value["kind"], "spawnError", "pre-spawn refusal stays definitive");
                assert!(!probe.workspace().join("effect-count").exists());
            } else {
                assert_eq!(value, expected, "completed bytes/exit status must retain existing display");
            }
            if has_effect { probe.assert_one_effect(); }
            assert!(!probe.root.join("fault-observed.json").exists());
            probe.finish();
        }
    }
}
