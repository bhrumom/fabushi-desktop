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
