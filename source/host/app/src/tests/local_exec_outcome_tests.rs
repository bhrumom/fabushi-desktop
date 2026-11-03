//! The shipping App node system.run helper, actual LocalExec manager/bridge,
//! and actual node invocation lifecycle. Controlled provider frames do not
//! establish physical-device or external-provider compatibility.
use super::super::execute_local_node_shell_invocation;
use mahayana_host_runtime::extensions::local_exec::extension::start_local_exec_extension;
use mahayana_host_runtime::extensions::local_exec::gateway_local_exec_sand_box::{
    GatewayLocalExecOperationStatus, GatewayLocalToolScope,
};
use mahayana_host_runtime::extensions::local_tool_permission::extension::start_local_tool_permission_extension;
use mahayana_host_runtime::extensions::node_host::invocation_runtime::NodeHostInvocationRuntime;
use mahayana_host_runtime::extensions::node_host::invoke_protocol::NodeInvokeRequest;
use mahayana_host_runtime::extensions::node_host::runtime_state::NodeHostRuntimeState;
use mahayana_host_runtime::extensions::node_host::session_registry::NodeHostSessionMetadata;
use mahayana_host_runtime::extensions::settings::settings_service::SettingsService;
use mahayana_host_runtime::extensions::telemetry::host_telemetry_service::HostTelemetryService;
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

const CHILD_ENV: &str = "FABUSHI_LXO_APP_CHILD";
const BOUND: Duration = Duration::from_secs(10);

// Each actual App harness child receives its root before startup. The tests
// never mutate process-global environment while production workers are alive.
fn run_owned(name: &str, case: &str) {
    if std::env::var(CHILD_ENV).ok().as_deref() == Some(name) {
        run_case(case);
        return;
    }
    let root = std::env::temp_dir().join(format!("lxo-app-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root).expect("owned App fixture");
    let stdout = root.join("child.stdout");
    let stderr = root.join("child.stderr");
    let mut child = Command::new(std::env::current_exe().expect("actual App test executable"))
        .args(["--exact", name, "--nocapture", "--test-threads=1"])
        .env(CHILD_ENV, name)
        .env("SAND_DATA_ROOT", &root)
        .stdout(Stdio::from(fs::File::create(&stdout).expect("capture stdout")))
        .stderr(Stdio::from(fs::File::create(&stderr).expect("capture stderr")))
        .spawn().expect("owned actual App test child");
    let deadline = Instant::now() + BOUND;
    let status = loop {
        if let Some(status) = child.try_wait().expect("observe exact child") { break status; }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("owned LXO App child exceeded bound: {}\n{}",
                fs::read_to_string(&stdout).unwrap_or_default(),
                fs::read_to_string(&stderr).unwrap_or_default());
        }
        thread::sleep(Duration::from_millis(2));
    };
    let output = fs::read_to_string(&stdout).expect("captured stdout");
    let errors = fs::read_to_string(&stderr).expect("captured stderr");
    assert!(status.success(), "{name}: {status}\n{output}\n{errors}");
    assert!(output.contains(&format!("test {name} ... ok")), "child must execute the exact positive: {output}");
    fs::remove_dir_all(&root).expect("remove only terminated owned fixture");
}

fn run_case(case: &str) {
    use mahayana_host_runtime::extensions::settings::settings_service::SandLocalToolPermission;
    let root = PathBuf::from(std::env::var_os("SAND_DATA_ROOT").expect("child root"));
    let settings = Arc::new(SettingsService::new(root.join("settings.json")));
    settings.set_local_tool_permission(if case == "permission" {
        SandLocalToolPermission::Never
    } else {
        SandLocalToolPermission::Always
    }).expect("persist actual permission");
    let permission = Arc::new(start_local_tool_permission_extension(settings));
    permission.controller().begin_turn("lxo-node-agent");
    let telemetry = HostTelemetryService::open_with_identity_tags(root.join("telemetry.jsonl"), Default::default())
        .expect("actual telemetry owner");
    let extension = start_local_exec_extension(telemetry.logs.clone());
    let (sender, receiver) = mpsc::channel();
    let registration = extension.register_provider(sender);
    let welcome = receiver.recv_timeout(Duration::from_secs(2)).expect("real provider welcome");
    let provider_id = welcome["providerId"].as_str().expect("unique provider identity").to_string();
    extension.submit_responses(json!({"providerId":provider_id,"frames":[{
        "kind":"hello", "localRoot":root, "terminalsFolder":"terminals",
        "computerId":"lxo-node-computer", "label":"owned node provider"
    }]}));
    let manager = extension.sandbox(permission).exec_manager();
    let observer = extension.sandbox(Arc::new(
        start_local_tool_permission_extension(Arc::new(SettingsService::new(root.join("observer-settings.json"))))
    )).exec_manager();
    let runtime = Arc::new(NodeHostRuntimeState::new());
    let lease = runtime.connect(NodeHostSessionMetadata {
        node_id:"sand-local-node".into(), display_name:"local".into(), platform:"fixture".into(),
        device_family:"fixture".into(), commands:vec!["system.run".into()],
        capabilities:vec!["exec".into()], paired:true, gateway_local:true,
    });
    assert!(runtime.set_active(&lease, true));
    let operation_id = format!("lxo-node-{case}");
    let request = NodeInvokeRequest {
        id:operation_id.clone(), node_id:"sand-local-node".into(), command:"system.run".into(),
        params:json!({"command":"fixture command"}), received_params_bytes:0,
        received_params_json:None, timeout_ms:Some(if case == "deadline" { 100 } else { 2_000 }),
        idempotency_key:Some(operation_id.clone()), session_key:None,
    };
    let invocation = NodeHostInvocationRuntime::start(Arc::clone(&runtime), lease, &request)
        .expect("actual invocation admission");
    let scope = GatewayLocalToolScope {
        agent_id:Some("lxo-node-agent".into()), tool_call_id:Some(operation_id.clone()),
        action:Some("node.invoke.system.run".into()),
    };
    let call_id = operation_id.clone();
    let worker_manager = manager.clone();
    let timeout = request.timeout_ms.map(Duration::from_millis);
    let worker = thread::spawn(move || execute_local_node_shell_invocation(
        &worker_manager, invocation, &scope, "fixture command", "/tmp", &call_id, timeout,
    ));
    if case == "permission" {
        let reply = worker.join().expect("App helper thread").expect("node response");
        assert_eq!(reply["ok"], false);
        assert_eq!(reply["error"]["code"], "EXECUTION_ERROR");
        assert!(reply["error"].get("details").is_none());
        assert!(receiver.try_recv().is_err(), "no execution request entered the provider");
        assert!(observer.operation(&operation_id).is_none());
        drop(registration);
        return;
    }
    let frame = receiver.recv_timeout(Duration::from_secs(2)).expect("actual exec entry");
    assert_eq!(frame["kind"], "exec");
    let request_id = frame["requestId"].as_str().expect("bridge request identity").to_string();
    let send = |kind: &str, message: Value| extension.submit_responses(json!({
        "providerId":provider_id, "frames":[{"kind":kind,"requestId":request_id,"message":message}]
    }));
    assert!(observer.operation(&operation_id).is_some(), "independent facade shares active owner");

    match case {
        "close" | "malformed" => {
            send("client", json!({"shellStream":{"stdout":{"data":"observed partial\n"}}}));
            if case == "close" { send("control", json!({"streamClose":{}})); }
            else { send("control", json!(["malformed"])); }
        }
        "deadline" => {
            let cancel = receiver.recv_timeout(Duration::from_secs(2)).expect("deadline requested cancel");
            assert_eq!(cancel["kind"], "cancel");
            assert_eq!(cancel["requestId"], request_id);
        }
        "preentry" => send("control", json!({"throw":{
            "error":"provider refused before execution", "errorCode":"LOCAL_EXEC_PRE_ENTRY_REFUSED"
        }})),
        "normal" | "aborted" => {
            assert!(manager.cancel_operation_for_agent("lxo-node-agent", &operation_id));
            let cancel = receiver.recv_timeout(Duration::from_secs(2)).expect("owned cancel");
            assert_eq!(cancel["requestId"], request_id);
            send("client", json!({"shellStream":{"stdout":{"data":"observed output\n"}}}));
            send("client", json!({"shellStream":{"exit":{"code":7,"cwd":"/tmp","aborted":case == "aborted"}}}));
        }
        "journal" => {
            let journal = root.join(mahayana_host_runtime::durable_file_policy::SAND_LOCAL_EXEC_OPERATION_JOURNAL_FILE_NAME);
            fs::remove_file(&journal).expect("only owned accepted journal");
            fs::create_dir(&journal).expect("real terminal publication refusal");
            send("client", json!({"shellStream":{"exit":{"code":0,"cwd":"/tmp"}}}));
        }
        _ => panic!("undeclared LXO App case"),
    }
    let reply = worker.join().expect("App helper thread").expect("node response");
    assert_eq!(reply["id"], operation_id);
    assert_eq!(reply["nodeId"], "sand-local-node");
    assert_eq!(reply["command"], "system.run");
    assert!(reply["payloadJSON"].is_null());
    let record = observer.operation(&operation_id).expect("shared terminal operation");
    assert_eq!(record.bridge_request_id, request_id);
    match case {
        "normal" | "aborted" => {
            assert_eq!(reply["ok"], true);
            assert_eq!(reply["payload"]["exitCode"], 7);
            assert_eq!(reply["payload"]["stdout"], "observed output\n");
            assert_eq!(reply["payload"]["aborted"], case == "aborted");
            assert_eq!(reply["payload"]["outcomeUnknown"], false);
            assert_eq!(record.status, if case == "aborted" {
                GatewayLocalExecOperationStatus::Cancelled
            } else { GatewayLocalExecOperationStatus::Completed });
        }
        "preentry" => {
            assert_eq!(reply["ok"], false);
            assert_eq!(reply["error"]["code"], "EXECUTION_ERROR");
            assert_eq!(record.status, GatewayLocalExecOperationStatus::Failed);
        }
        _ => {
            assert_eq!(reply["ok"], false);
            assert_eq!(reply["error"]["code"], "EXECUTION_OUTCOME_UNKNOWN");
            assert_eq!(reply["error"]["details"]["operationId"], operation_id);
            assert_eq!(reply["error"]["details"]["resultRef"], format!("fabushi-local-exec://operation/{operation_id}"));
            assert_eq!(reply["error"]["details"]["outcomeUnknown"], true);
            assert!(!record.aborted, "a delivered cancel is not an observed aborted Exit");
            if case == "journal" {
                assert_eq!(record.status, GatewayLocalExecOperationStatus::Completed);
                assert_eq!(record.exit_code, Some(0), "preserve observed Exit despite result durability failure");
                assert_eq!(reply["error"]["details"]["observed"]["exitCode"], 0);
                assert!(manager.ensure_operation_journal_healthy().is_err());
            } else {
                assert_eq!(record.status, GatewayLocalExecOperationStatus::OutcomeUnknown);
            }
            if case == "close" || case == "malformed" {
                assert_eq!(record.stdout, "observed partial\n");
                let output = if case == "close" { &reply["error"]["details"]["stdout"] }
                    else { &reply["error"]["details"]["observed"]["stdout"] };
                assert_eq!(output, "observed partial\n");
            }
        }
    }
    assert_eq!(observer.operation_history_for_agent("lxo-node-agent").len(), 1);
    drop(registration);
}

#[test]
fn local_exec_app_close_reports_unknown_with_stable_result_identity() {
    run_owned("tests::local_exec_outcome_tests::local_exec_app_close_reports_unknown_with_stable_result_identity", "close");
}
#[test]
fn local_exec_app_malformed_reply_preserves_partial_output_and_unknown_error() {
    run_owned("tests::local_exec_outcome_tests::local_exec_app_malformed_reply_preserves_partial_output_and_unknown_error", "malformed");
}
#[test]
fn local_exec_app_deadline_cancel_without_exit_remains_unknown() {
    run_owned("tests::local_exec_outcome_tests::local_exec_app_deadline_cancel_without_exit_remains_unknown", "deadline");
}
#[test]
fn local_exec_app_confirmed_normal_exit_outranks_cancel_request() {
    run_owned("tests::local_exec_outcome_tests::local_exec_app_confirmed_normal_exit_outranks_cancel_request", "normal");
}
#[test]
fn local_exec_app_confirmed_aborted_exit_remains_known() {
    run_owned("tests::local_exec_outcome_tests::local_exec_app_confirmed_aborted_exit_remains_known", "aborted");
}
#[test]
fn local_exec_app_permission_refusal_never_dispatches() {
    run_owned("tests::local_exec_outcome_tests::local_exec_app_permission_refusal_never_dispatches", "permission");
}
#[test]
fn local_exec_app_remote_preentry_refusal_remains_known() {
    run_owned("tests::local_exec_outcome_tests::local_exec_app_remote_preentry_refusal_remains_known", "preentry");
}
#[test]
fn local_exec_app_exit_with_journal_failure_retains_observed_result() {
    run_owned("tests::local_exec_outcome_tests::local_exec_app_exit_with_journal_failure_retains_observed_result", "journal");
}
