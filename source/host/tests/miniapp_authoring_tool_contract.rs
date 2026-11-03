use std::sync::{Arc, Mutex};
use mahayana_host_runtime::extensions::inference::provider_session::{ProviderSessionError, RoutedToolDefinition};
use mahayana_host_runtime::runner::box_tool_access::{RunnerBoxResourcePort, RunnerBoxShellRequest, RunnerBoxReadRequest, RunnerBoxWriteRequest};
use mahayana_host_runtime::runner::routed_provider_runtime::{RoutedProviderCancellation, RoutedToolBridge};
use mahayana_host_runtime::runner::tools::turn_toolset::{build_turn_toolset, TurnToolsetDependencies, TurnToolsetRole};
use serde_json::{json, Value};

struct Base;
impl RoutedToolBridge for Base {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> { Ok(vec![]) }
    fn call_tool(&self, _: &RoutedToolDefinition, _: Value, _: &str) -> Result<Value, ProviderSessionError> {
        Err(ProviderSessionError::Tool("unexpected upstream call".into()))
    }
}
#[derive(Default)]
struct BoxPort(Mutex<Vec<RunnerBoxShellRequest>>);
impl RunnerBoxResourcePort for BoxPort {
    fn execute_shell(&self, request: RunnerBoxShellRequest) -> Result<Value, ProviderSessionError> {
        self.0.lock().unwrap().push(request);
        Ok(json!({"executed":true}))
    }
    fn execute_read(&self, _: RunnerBoxReadRequest) -> Result<Value, ProviderSessionError> { unreachable!() }
    fn execute_write(&self, _: RunnerBoxWriteRequest) -> Result<(), ProviderSessionError> { panic!("must preserve Shell review") }
}

#[test]
fn toolset_exposes_authoring_only_with_the_ordinary_bots_box() {
    for (available, shared, subagent, expected) in [(true,false,false,true),(false,false,false,false),(true,true,false,false),(true,false,true,false)] {
        let bridge = build_turn_toolset(Arc::new(Base), TurnToolsetDependencies {
            role: TurnToolsetRole { remote_box_available:available, is_shared_room_runner:shared, is_subagent_runner:subagent, ..Default::default() },
            box_resources: Some(Arc::new(BoxPort::default())), ..Default::default()
        });
        assert_eq!(bridge.list_tools().unwrap().iter().any(|t| t.name == "MiniApp"), expected);
    }
}

#[test]
fn authoring_uses_existing_shell_review_identity_and_cancellation() {
    for denied in [true,false] {
        let port = Arc::new(BoxPort::default());
        let reviews = Arc::new(Mutex::new(Vec::new()));
        let captured = reviews.clone();
        let cancellation = RoutedProviderCancellation::default();
        let bridge = build_turn_toolset(Arc::new(Base), TurnToolsetDependencies {
            box_resources:Some(port.clone()), cancellation:cancellation.clone(),
            box_shell_review:Some(Arc::new(move |request| {
                captured.lock().unwrap().push(request.tool_call_id.clone());
                Ok(denied.then(|| "review denied".into()))
            })), ..Default::default()
        });
        let tool = bridge.list_tools().unwrap().into_iter().find(|t| t.name == "MiniApp").unwrap();
        let result = bridge.call_tool(&tool,json!({"action":"inspect","project":"counter"}),"authoring-1").unwrap();
        assert_eq!(reviews.lock().unwrap().as_slice(), ["authoring-1"]);
        assert_eq!(port.0.lock().unwrap().len(), usize::from(!denied));
        if denied { assert_eq!(result, json!("review denied")); }
        else {
            let calls = port.0.lock().unwrap();
            assert_eq!(calls[0].working_directory,"/workspace");
            assert!(calls[0].command.starts_with("python3 -I -c "));
            assert!(!calls[0].is_background);
        }
        cancellation.cancel("cancelled");
        assert!(bridge.call_tool(&tool,json!({"action":"inspect","project":"counter"}),"authoring-2").is_err());
        assert_eq!(reviews.lock().unwrap().len(),1);
    }
}

struct UnavailableBox(std::sync::atomic::AtomicUsize);
impl RunnerBoxResourcePort for UnavailableBox {
    fn execute_shell(&self, _: RunnerBoxShellRequest) -> Result<Value, ProviderSessionError> {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Err(ProviderSessionError::Tool("Computer connection failed [daemon_refused]; startup state is unknown. Diagnostic: connection refused".into()))
    }
    fn execute_read(&self, _: RunnerBoxReadRequest) -> Result<Value, ProviderSessionError> { unreachable!() }
    fn execute_write(&self, _: RunnerBoxWriteRequest) -> Result<(), ProviderSessionError> { unreachable!() }
}

#[test]
fn ordinary_box_failures_remain_model_visible_without_cancelling_the_turn() {
    let port = Arc::new(UnavailableBox(std::sync::atomic::AtomicUsize::new(0)));
    let cancellation = RoutedProviderCancellation::default();
    let bridge = build_turn_toolset(Arc::new(Base), TurnToolsetDependencies {
        box_resources:Some(port.clone()), cancellation:cancellation.clone(), ..Default::default()
    });
    let tool = bridge.list_tools().unwrap().into_iter().find(|tool| tool.name == "MiniApp").unwrap();
    for attempt in 0..5 {
        let error = bridge.call_tool(&tool,json!({"action":"inspect","project":"counter"}),"tool-failure").unwrap_err();
        match error {
            ProviderSessionError::Tool(message) => {
                assert!(message.contains("Diagnostic: connection refused"));
                assert_eq!(message.contains("[Tool observation:"), attempt == 1);
            }
            other => panic!("ordinary tool failure must not cancel the turn: {other}"),
        }
        assert!(!cancellation.is_cancelled());
    }
    cancellation.cancel("user stopped the turn");
    assert!(bridge.call_tool(&tool,json!({"action":"inspect","project":"counter"}),"after-user-stop").is_err());
    assert_eq!(port.0.load(std::sync::atomic::Ordering::SeqCst), 5);
}

struct ScriptedBox(Mutex<std::collections::VecDeque<Result<Value, ProviderSessionError>>>);
impl RunnerBoxResourcePort for ScriptedBox {
    fn execute_shell(&self, _: RunnerBoxShellRequest) -> Result<Value, ProviderSessionError> {
        self.0.lock().unwrap().pop_front().expect("scripted Box outcome")
    }
    fn execute_read(&self, _: RunnerBoxReadRequest) -> Result<Value, ProviderSessionError> { unreachable!() }
    fn execute_write(&self, _: RunnerBoxWriteRequest) -> Result<(), ProviderSessionError> { unreachable!() }
}

#[test]
fn repetition_advisory_distinguishes_changed_calls_errors_and_real_recovery() {
    let failure = |text: &str| Err(ProviderSessionError::Tool(text.into()));
    let port = Arc::new(ScriptedBox(Mutex::new(std::collections::VecDeque::from([
        failure("refused"), failure("refused"), failure("refused"),
        failure("permission denied"), failure("permission denied"),
        Ok(json!({"executed":true})), failure("permission denied"), failure("permission denied"),
    ]))));
    let cancellation = RoutedProviderCancellation::default();
    let bridge = build_turn_toolset(Arc::new(Base), TurnToolsetDependencies {
        box_resources:Some(port), cancellation:cancellation.clone(), ..Default::default()
    });
    let shell = bridge.list_tools().unwrap().into_iter().find(|tool| tool.name == "Shell").unwrap();
    for (index, expected_warning) in [false, true, false, false, true, false, false, true].into_iter().enumerate() {
        // The third call is a distinct diagnostic, not a replay of the first.
        let command = if index == 2 { "diagnose changed-state" } else { "diagnose" };
        let args = if index == 1 {
            serde_json::from_str(&format!("{{\"workingDirectory\":\"/workspace\",\"command\":\"{command}\"}}")).unwrap()
        } else { json!({"command":command,"workingDirectory":"/workspace"}) };
        let result = bridge.call_tool(&shell,args,"diagnostic");
        if index == 5 { assert!(result.is_ok()); }
        else { assert_eq!(result.unwrap_err().to_string().contains("[Tool observation:"), expected_warning); }
        assert!(!cancellation.is_cancelled());
    }
}

struct ActionsBox;
impl RunnerBoxResourcePort for ActionsBox {
    fn execute_shell(&self, request: RunnerBoxShellRequest) -> Result<Value, ProviderSessionError> {
        let output = std::process::Command::new("/bin/sh")
            .args(["-c", &request.command]).current_dir(&request.working_directory)
            .output().map_err(|e| ProviderSessionError::Tool(e.to_string()))?;
        if !output.status.success() {
            return Err(ProviderSessionError::Tool(String::from_utf8_lossy(&output.stderr).into_owned()));
        }
        serde_json::from_slice(&output.stdout).map_err(|e| ProviderSessionError::Tool(e.to_string()))
    }
    fn execute_read(&self, _: RunnerBoxReadRequest) -> Result<Value, ProviderSessionError> { unreachable!() }
    fn execute_write(&self, _: RunnerBoxWriteRequest) -> Result<(), ProviderSessionError> { unreachable!() }
}

#[test]
#[cfg(feature = "actions-miniapp-bot")]
fn actions_executes_actual_bot_tool_create_modify_export_chain() {
    // This instrument needs the dedicated disposable Actions workspace. Other
    // contract suites retain their already-valid pure composition coverage.
    assert_eq!(std::env::var("FABUSHI_MINIAPP_BOT_EXECUTION").as_deref(), Ok("1"));
    assert_eq!(std::env::var("GITHUB_ACTIONS").as_deref(), Ok("true"));
    let shadow = std::path::Path::new("/workspace/base64.py");
    use std::io::Write;
    let mut poison = std::fs::OpenOptions::new().write(true).create_new(true).open(shadow).unwrap();
    poison.write_all(b"raise RuntimeError('untrusted workspace import was executed')\n").unwrap();
    let bridge = build_turn_toolset(Arc::new(Base), TurnToolsetDependencies {
        box_resources:Some(Arc::new(ActionsBox)), ..Default::default()
    });
    let tool = bridge.list_tools().unwrap().into_iter().find(|t| t.name == "MiniApp").unwrap();
    let project = format!("ci-{}",uuid::Uuid::new_v4().simple());
    let first = bridge.call_tool(&tool,json!({"action":"create","project":project,
        "appId":"org.fabushi.ci-counter","publisherId":"fabushi-ci",
        "displayName":"Bot CI counter","version":"1.0.0"}),"create").unwrap();
    let js = first["files"].as_array().unwrap().iter().find(|f| f["path"] == "src/app.js").unwrap();
    let tests = first["files"].as_array().unwrap().iter().find(|f| f["path"] == "tests/app.test.cjs").unwrap();
    let initial = bridge.call_tool(&tool,json!({"action":"export","project":project}),"inspect-source").unwrap();
    let initial_files = initial["githubFiles"].as_array().unwrap();
    let js_source = initial_files.iter().find(|f| f["path"] == "src/app.js").unwrap()["content"].as_str().unwrap().replace("++count", "count += 2");
    let test_source = initial_files.iter().find(|f| f["path"] == "tests/app.test.cjs").unwrap()["content"].as_str().unwrap()
        .replace("textContent, '2'", "textContent, '4'").replace("textContent, '1'", "textContent, '2'");
    let changed = bridge.call_tool(&tool,json!({"action":"edit","project":project,
        "expectedRevision":first["revision"],"version":"1.0.1","files":[{
            "path":"src/app.js","expectedSha256":js["sha256"],
            "content":js_source
        }, {"path":"tests/app.test.cjs","expectedSha256":tests["sha256"],"content":test_source}]}),"modify").unwrap();
    assert_ne!(first["revision"],changed["revision"]);
    let exported = bridge.call_tool(&tool,json!({"action":"export","project":project}),"export").unwrap();
    assert_eq!(exported["revision"],changed["revision"]);
    assert_eq!(exported["installable"],false);
    let destination = std::env::var("FABUSHI_MINIAPP_EXPORT").unwrap();
    std::fs::write(destination,serde_json::to_vec(&exported).unwrap()).unwrap();
    std::fs::remove_file(shadow).unwrap();
}
