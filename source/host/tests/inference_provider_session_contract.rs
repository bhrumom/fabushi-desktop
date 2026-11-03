use std::cell::Cell;
use std::collections::VecDeque;
use std::fs;
use std::io::Cursor;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::inference::codex_direct_responses::CodexDirectCheckpoint;
use mahayana_host_runtime::extensions::inference::provider_session::{
    OpenRouterCheckpoint, OpenRouterTransport, ProviderMessage,
    ProviderSessionError, ProviderTokenUsage, RoutedProvider, RoutedProviderCheckpoint,
    RoutedProviderOptions, RoutedToolDefinition, configured_routed_provider, decode_sse_stream,
    resolve_fabushi_request_model,
    run_openrouter_with_transport, run_openrouter_with_transport_reporting_usage,
    run_openrouter_with_transport_reporting_usage_and_partials, run_routed_provider_text,
    run_routed_provider_text_with_lifecycle,
};
use serde_json::{Value, json};

#[test]
fn provider_session_reads_router_settings_and_streams_sse_events() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "fabushi-provider-settings-{}-{suffix}.json",
        std::process::id()
    ));
    fs::write(&path, r#"{"router":{"provider":"codex"}}"#)
        .expect("write settings");
    assert_eq!(
        configured_routed_provider(&path),
        Some(RoutedProvider::Codex)
    );
    fs::remove_file(path).expect("remove settings");

    let stream = b"event: ignored\r\ndata: {\"type\":\"response.output_text.delta\",\"delta\":\"hi\"}\r\n\r\ndata: [DONE]\r\n\r\n";
    let mut events = Vec::new();
    decode_sse_stream(Cursor::new(stream.as_slice()), |event| {
        events.push(event);
        Ok(())
    })
    .expect("decode SSE");
    assert_eq!(
        events,
        vec![json!({
            "type":"response.output_text.delta",
            "delta":"hi"
        })]
    );
}

#[test]
fn provider_session_honors_runner_cancellation_before_provider_setup() {
    let root = std::env::temp_dir();
    let messages = [];
    let tools = [];
    let should_cancel = || true;
    let mut execute_tool = |_tool: &mahayana_host_runtime::extensions::inference::provider_session::RoutedToolDefinition,
                            _args: serde_json::Value,
                            _call_id: &str| -> Result<serde_json::Value, ProviderSessionError> {
        unreachable!("cancelled provider must not execute tools")
    };
    let mut on_text_delta = |_delta: &str, _accumulated: &str| {
        panic!("cancelled provider must not emit text")
    };
    let error = run_routed_provider_text(
        RoutedProvider::Cursor,
        &messages,
        &mut RoutedProviderOptions {
            data_dir: &root,
            typed_replay: None,
            fabushi_model: None,
            codex_model: None,
            codex_request_binding: None,
            fabushi_request_binding: None,
            cursor_auth: None,
            tools: &tools,
            mcp_server_url: None,
            execute_tool: &mut execute_tool,
            on_text_delta: &mut on_text_delta,
            on_partial_tool_call: None,
            should_cancel: &should_cancel,
        },
    )
    .expect_err("runner cancellation should win before provider setup");
    assert!(matches!(error, ProviderSessionError::Cancelled(_)));
}

fn fabushi_checkpoint(model: Option<&str>) -> CodexDirectCheckpoint {
    CodexDirectCheckpoint {
        model: model.map(str::to_string),
        input: vec![
            json!({"role":"user","content":"calendar"}),
            json!({
                "type":"function_call", "name":"calendar_list",
                "call_id":"accepted-call", "arguments":"{}"
            }),
            json!({
                "type":"function_call_output", "call_id":"accepted-call",
                "output":"{\"events\":[\"daily\"]}"
            }),
        ],
        text: "Checking ".into(),
        response_id: "accepted-response".into(),
        usage: Default::default(),
        completed_steps: 1,
        tool_calls_completed: 1,
    }
}

fn refused_fabushi_dispatch(
    requested: Option<&str>,
    checkpoint: Option<&RoutedProviderCheckpoint>,
    cancelled: bool,
) -> ProviderSessionError {
    refused_provider_dispatch(
        RoutedProvider::Fabushi,
        requested,
        checkpoint,
        &|| cancelled,
    )
}

fn refused_provider_dispatch(
    provider: RoutedProvider,
    requested: Option<&str>,
    checkpoint: Option<&RoutedProviderCheckpoint>,
    should_cancel: &dyn Fn() -> bool,
) -> ProviderSessionError {
    let root = std::env::temp_dir();
    run_routed_provider_text_with_lifecycle(
        provider,
        &[],
        &mut RoutedProviderOptions {
            data_dir: &root,
            typed_replay: None,
            fabushi_model: requested,
            codex_model: None,
            codex_request_binding: None,
            fabushi_request_binding: None,
            cursor_auth: None,
            tools: &[],
            mcp_server_url: None,
            execute_tool: &mut |_tool, _args, _call_id| {
                panic!("refused provider dispatch must not execute a tool")
            },
            on_text_delta: &mut |_delta, _accumulated| {
                panic!("refused provider dispatch must not emit text")
            },
            on_partial_tool_call: None,
            should_cancel,
        },
        checkpoint,
        &mut |_accepted| panic!("refused provider dispatch must not accept a checkpoint"),
    )
    .expect_err("provider dispatch must fail before provider activity")
}

#[test]
fn fabushi_request_model_preserves_explicit_identity_and_rejects_invalid_selection() {
    for (requested, expected) in [
        ("  Org/Family/Model@2026-11  ", "Org/Family/Model@2026-11"),
        ("\u{2003}Model.Mixed+Case\u{2003}", "Model.Mixed+Case"),
        ("grok", "grok"),
        ("gemini", "gemini"),
    ] {
        assert_eq!(
            resolve_fabushi_request_model(Some(requested), None).unwrap(),
            expected,
        );
    }
    for requested in [
        "", "   ", "\u{2003}", "\tModel", "Model\n", "A\0B", "Model\u{7f}", "\u{85}Model",
    ] {
        assert!(
            matches!(
                resolve_fabushi_request_model(Some(requested), None),
                Err(ProviderSessionError::Configuration(_)),
            ),
            "invalid explicit model: {requested:?}",
        );
    }
}

#[test]
fn fabushi_checkpoint_requires_normalized_identity_and_exact_selected_model() {
    for model in [
        None, Some(""), Some(" "), Some(" Model"), Some("Model "), Some("Model\n"),
        Some("A\u{7f}B"),
    ] {
        let checkpoint = fabushi_checkpoint(model);
        for requested in [None, Some("Model"), Some("\t")] {
            assert!(
                matches!(
                    resolve_fabushi_request_model(requested, Some(&checkpoint)),
                    Err(ProviderSessionError::Protocol(_)),
                ),
                "invalid checkpoint model must fail first: {model:?}",
            );
        }
    }

    let model = "Org/Family/Model@2026-11";
    let checkpoint = fabushi_checkpoint(Some(model));
    assert_eq!(
        resolve_fabushi_request_model(None, Some(&checkpoint)).unwrap(),
        model,
    );
    assert_eq!(
        resolve_fabushi_request_model(
            Some("  Org/Family/Model@2026-11  "),
            Some(&checkpoint),
        ).unwrap(),
        model,
    );
    for requested in ["org/Family/Model@2026-11", "Org/Family/Model@other", "other"] {
        assert!(matches!(
            resolve_fabushi_request_model(Some(requested), Some(&checkpoint)),
            Err(ProviderSessionError::Protocol(_)),
        ));
    }
    for requested in ["", "  ", "\tOrg/Family/Model@2026-11"] {
        assert!(matches!(
            resolve_fabushi_request_model(Some(requested), Some(&checkpoint)),
            Err(ProviderSessionError::Configuration(_)),
        ));
    }
}

#[test]
fn fabushi_dispatch_preserves_cancellation_and_provider_tag_priority() {
    let legacy = RoutedProviderCheckpoint::Fabushi(fabushi_checkpoint(None));
    let other_provider = RoutedProviderCheckpoint::Codex(fabushi_checkpoint(None));
    for checkpoint in [&legacy, &other_provider] {
        assert!(matches!(
            refused_fabushi_dispatch(Some("\t"), Some(checkpoint), true),
            ProviderSessionError::Cancelled(_),
        ));
    }
    assert!(matches!(
        refused_fabushi_dispatch(Some("\t"), Some(&other_provider), false),
        ProviderSessionError::Protocol(message) if message.contains("belongs to codex"),
    ));
}

#[test]
fn fabushi_environment_fallback_checkpoint_pin_and_auth_order_are_isolated() {
    const CHILD_CASE: &str = "FABUSHI_PROVIDER_MODEL_CONTRACT_CASE";
    const CHILD_COMPLETED: &str = "fabushi-provider-model-contract-completed";
    if let Ok(case) = std::env::var(CHILD_CASE) {
        let result = resolve_fabushi_request_model(None, None);
        match case.as_str() {
            "missing" | "empty" | "spaces" => assert_eq!(result.unwrap(), "deepseek-chat"),
            "selected" => assert_eq!(result.unwrap(), "Env/Family/Model@Case"),
            "control" | "control_only" => assert!(matches!(
                result,
                Err(ProviderSessionError::Configuration(_)),
            )),
            _ => panic!("unknown isolated model case: {case}"),
        }

        let model = "Frozen/Family/Model@Case";
        let checkpoint = fabushi_checkpoint(Some(model));
        assert_eq!(
            resolve_fabushi_request_model(None, Some(&checkpoint)).unwrap(),
            model,
        );
        assert_eq!(resolve_fabushi_request_model(Some(model), None).unwrap(), model);

        // A valid selection reaches the deliberately unavailable credential.
        // Every model refusal below must precede this same authentication path.
        assert!(matches!(
            refused_fabushi_dispatch(Some(model), None, false),
            ProviderSessionError::Authentication(message) if message.contains("credential is unavailable"),
        ));
        let accepted = RoutedProviderCheckpoint::Fabushi(checkpoint);
        assert!(matches!(
            refused_fabushi_dispatch(None, Some(&accepted), false),
            ProviderSessionError::Authentication(message) if message.contains("credential is unavailable"),
        ));
        assert!(matches!(
            refused_fabushi_dispatch(Some("Changed/Model"), Some(&accepted), false),
            ProviderSessionError::Protocol(message) if message.contains("model identity"),
        ));
        for checkpoint_model in [None, Some(""), Some(" model "), Some("model\n")] {
            let invalid = RoutedProviderCheckpoint::Fabushi(fabushi_checkpoint(checkpoint_model));
            assert!(matches!(
                refused_fabushi_dispatch(Some(model), Some(&invalid), false),
                ProviderSessionError::Protocol(message) if message.contains("model identity"),
            ));
        }
        for requested in ["", "  ", "\tModel"] {
            assert!(matches!(
                refused_fabushi_dispatch(Some(requested), None, false),
                ProviderSessionError::Configuration(message) if message.contains("model selection"),
            ));
        }
        let checks = Cell::new(0_usize);
        let error = refused_provider_dispatch(
            RoutedProvider::Fabushi,
            Some("\tModel"),
            None,
            &|| {
                let previous = checks.get();
                checks.set(previous + 1);
                previous > 0
            },
        );
        assert!(matches!(&error, ProviderSessionError::Cancelled(_)), "{error}");
        assert_eq!(checks.get(), 2,
            "routed post-call cancellation must run after the pre-auth model refusal");
        assert!(matches!(
            refused_provider_dispatch(RoutedProvider::Cursor, Some("\tModel"), None, &|| false),
            ProviderSessionError::Configuration(message) if message.contains("Cursor inference requires"),
        ), "Cursor must ignore the Fabushi-only model field and reach its own auth boundary");
        eprintln!("{CHILD_COMPLETED}:{case}");
        return;
    }

    for (case, model) in [
        ("missing", None),
        ("empty", Some("")),
        ("spaces", Some(" \u{2003} ")),
        ("selected", Some("  Env/Family/Model@Case  ")),
        ("control", Some("\tEnv/Family/Model@Case")),
        ("control_only", Some("\n")),
    ] {
        let suffix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let missing_credential = std::env::temp_dir().join(format!(
            "fabushi-model-absent-credential-{}-{suffix}-{case}",
            std::process::id(),
        ));
        assert!(!missing_credential.exists());
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "fabushi_environment_fallback_checkpoint_pin_and_auth_order_are_isolated",
                "--nocapture",
            ])
            .env(CHILD_CASE, case)
            .env("FABUSHI_RESPONSES_URL", "http://127.0.0.1:1/responses")
            .env("FABUSHI_HOST_ACCESS_CREDENTIAL_FILE", &missing_credential)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        match model {
            Some(model) => {
                command.env("FABUSHI_RESPONSES_MODEL", model);
            }
            None => {
                command.env_remove("FABUSHI_RESPONSES_MODEL");
            }
        }
        let mut child = command.spawn().expect("start isolated model contract");
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            if child.try_wait().expect("poll isolated model contract").is_some() {
                break;
            }
            if Instant::now() >= deadline {
                child.kill().expect("stop timed out model contract");
                let output = child.wait_with_output().expect("collect timed out model contract");
                panic!("isolated model case {case} timed out: {output:?}");
            }
            thread::sleep(Duration::from_millis(10));
        }
        let output = child.wait_with_output().expect("collect isolated model contract");
        assert!(output.status.success(), "isolated model case {case} failed: {output:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(&format!("{CHILD_COMPLETED}:{case}")),
            "isolated model case {case} did not execute its assertions: {output:?}",
        );
    }
}


struct FakeOpenRouterTransport {
    responses: VecDeque<Vec<Value>>,
    requests: Vec<Value>,
}

impl OpenRouterTransport for FakeOpenRouterTransport {
    fn stream_response(
        &mut self,
        request: &Value,
        on_event: &mut dyn FnMut(Value) -> Result<(), ProviderSessionError>,
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<(), ProviderSessionError> {
        self.requests.push(request.clone());
        if should_cancel() {
            return Err(ProviderSessionError::Cancelled(
                "fake OpenRouter transport observed cancellation".into(),
            ));
        }
        for event in self.responses.pop_front().expect("fake response") {
            on_event(event)?;
        }
        Ok(())
    }
}

#[test]
fn openrouter_resumes_only_from_an_accepted_tool_boundary_checkpoint() {
    let mut first_transport = FakeOpenRouterTransport {
        responses: VecDeque::from([
            vec![json!({
                "choices":[{
                    "delta":{
                        "content":"Checking ",
                        "tool_calls":[{
                            "index":0,
                            "id":"call-resume",
                            "function":{
                                "name":"calendar_list",
                                "arguments":"{\"days\":1}"
                            }
                        }]
                    }
                }]
            })],
            vec![json!({
                "error":{"message":"transient upstream failure"}
            })],
        ]),
        requests: Vec::new(),
    };
    let messages = vec![ProviderMessage {
        role: "user".into(),
        content: "calendar".into(),
    }];
    let tools = vec![RoutedToolDefinition {
        name: "calendar_list".into(),
        provider_identifier: "calendar".into(),
        tool_name: "list".into(),
        description: Some("List calendar entries".into()),
        input_schema: json!({"type":"object"}),
    }];

    let mut checkpoint: Option<OpenRouterCheckpoint> = None;
    let mut first_tool_calls = 0_usize;
    let mut first_deltas = Vec::new();
    let error = run_openrouter_with_transport(
        &mut first_transport,
        "openai/test",
        &messages,
        &tools,
        &mut |_tool, _args, _call_id| {
            first_tool_calls += 1;
            Ok(json!({"events":["daily"]}))
        },
        &mut |delta, accumulated| {
            first_deltas.push((delta.to_string(), accumulated.to_string()));
        },
        &|| false,
        None,
        &mut |accepted| {
            checkpoint = Some(accepted.clone());
            Ok(())
        },
    )
    .expect_err("second provider step should fail after checkpoint");
    assert!(error.to_string().contains("transient upstream failure"));
    assert_eq!(first_tool_calls, 1);
    assert_eq!(
        first_deltas,
        vec![("Checking ".to_string(), "Checking ".to_string())]
    );

    let checkpoint = checkpoint.expect("OpenRouter tool boundary checkpoint");
    assert_eq!(checkpoint.text, "Checking ");
    assert_eq!(checkpoint.completed_steps, 1);
    assert_eq!(checkpoint.tool_calls_completed, 1);
    assert!(checkpoint.conversation.iter().any(|message| {
        message["role"] == "tool"
            && message["tool_call_id"] == "call-resume"
    }));

    let mut resumed_transport = FakeOpenRouterTransport {
        responses: VecDeque::from([vec![json!({
            "choices":[{"delta":{"content":"done"}}]
        })]]),
        requests: Vec::new(),
    };
    let mut resumed_tool_calls = 0_usize;
    let mut resumed_deltas = Vec::new();
    let resumed = run_openrouter_with_transport(
        &mut resumed_transport,
        "openai/test",
        &messages,
        &tools,
        &mut |_tool, _args, _call_id| {
            resumed_tool_calls += 1;
            Ok(Value::Null)
        },
        &mut |delta, accumulated| {
            resumed_deltas.push((delta.to_string(), accumulated.to_string()));
        },
        &|| false,
        Some(&checkpoint),
        &mut |_accepted| Ok(()),
    )
    .expect("resume from accepted OpenRouter checkpoint");

    assert_eq!(
        resumed_tool_calls, 0,
        "accepted OpenRouter tool work must not repeat"
    );
    assert_eq!(resumed, "Checking done");
    assert_eq!(
        resumed_deltas,
        vec![("done".to_string(), "Checking done".to_string())]
    );
    assert_eq!(resumed_transport.requests.len(), 1);
    let resumed_messages = resumed_transport.requests[0]["messages"]
        .as_array()
        .expect("resumed messages");
    assert!(resumed_messages.iter().any(|message| {
        message["role"] == "tool"
            && message["tool_call_id"] == "call-resume"
    }));
}


#[test]
fn openrouter_reports_server_usage_without_estimating_tokens() {
    let mut transport = FakeOpenRouterTransport {
        responses: VecDeque::from([vec![
            json!({"choices":[{"delta":{"content":"hello"}}]}),
            json!({
                "choices":[],
                "usage":{
                    "prompt_tokens":11,
                    "completion_tokens":7,
                    "prompt_tokens_details":{"cached_tokens":3},
                    "completion_tokens_details":{"reasoning_tokens":2}
                }
            }),
        ]]),
        requests: Vec::new(),
    };
    let messages = vec![ProviderMessage {
        role: "user".into(),
        content: "hello".into(),
    }];
    let mut observed_usage = Vec::<ProviderTokenUsage>::new();
    let output = run_openrouter_with_transport_reporting_usage(
        &mut transport,
        "openai/test",
        &messages,
        &[],
        &mut |_tool, _args, _call_id| Ok(Value::Null),
        &mut |_delta, _accumulated| {},
        &|| false,
        None,
        &mut |_accepted| Ok(()),
        &mut |usage| observed_usage.push(usage),
    )
    .expect("OpenRouter usage-reporting turn");

    assert_eq!(output, "hello");
    assert_eq!(transport.requests.len(), 1);
    assert_eq!(transport.requests[0]["usage"]["include"], true);
    assert_eq!(
        observed_usage,
        vec![ProviderTokenUsage {
            input_tokens: 11,
            output_tokens: 7,
            cache_read_tokens: 3,
            cache_write_tokens: 0,
            reasoning_tokens: Some(2),
        }]
    );
}
#[test]
fn openrouter_partial_tool_callback_reports_real_accumulated_state_and_model_call_identity() {
    let tools = vec![RoutedToolDefinition {
        name: "Shell".into(),
        provider_identifier: "mahayana-box".into(),
        tool_name: "Shell".into(),
        description: Some("shell".into()),
        input_schema: json!({"type":"object"}),
    }];
    let messages = vec![ProviderMessage {
        role: "user".into(),
        content: "run".into(),
    }];
    let mut transport = FakeOpenRouterTransport {
        responses: VecDeque::from([
            vec![
                json!({
                    "id":"or-model-call-1",
                    "choices":[{"delta":{"tool_calls":[{
                        "index":0,"id":"or-call-1","function":{"name":"Shell","arguments":r#"{"command":"pri"#}
                    }]}}]
                }),
                json!({
                    "id":"or-model-call-1",
                    "choices":[{"delta":{"tool_calls":[{
                        "index":0,"function":{"arguments":r#"ntf ok"}"#}
                    }]}}]
                }),
            ],
            vec![json!({
                "id":"or-model-call-2",
                "choices":[{"delta":{"content":"done"}}]
            })],
        ]),
        requests: Vec::new(),
    };
    let mut partials = Vec::new();
    let mut calls = 0usize;
    let output = run_openrouter_with_transport_reporting_usage_and_partials(
        &mut transport,
        "test-model",
        &messages,
        &tools,
        &mut |_tool, args, call_id| {
            calls += 1;
            assert_eq!(call_id, "or-call-1");
            assert_eq!(args, json!({"command":"printf ok"}));
            Ok(json!({"kind":"success","stdout":"ok","exitCode":0}))
        },
        &mut |_delta, _accumulated| {},
        &|| false,
        None,
        &mut |_checkpoint| Ok(()),
        &mut |_usage| {},
        &mut |partial| {
            partials.push(partial);
            Ok(())
        },
    )
    .expect("OpenRouter partial lifecycle");

    assert_eq!(output, "done");
    assert_eq!(calls, 1);
    assert_eq!(partials.len(), 2);
    assert_eq!(partials[0].tool_call_id, "or-call-1");
    assert_eq!(partials[0].tool.tool_name, "Shell");
    assert_eq!(partials[0].raw_arguments, r#"{"command":"pri"#);
    assert_eq!(
        partials[0].model_call_id.as_deref(),
        Some("or-model-call-1")
    );
    assert_eq!(partials[1].raw_arguments, r#"{"command":"printf ok"}"#);
    assert_eq!(
        partials[1].model_call_id.as_deref(),
        Some("or-model-call-1")
    );
}
