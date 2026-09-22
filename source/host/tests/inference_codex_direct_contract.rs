use std::collections::VecDeque;

use mahayana_host_runtime::extensions::inference::codex_direct_responses::{
    CodexDirectError, CodexDirectOptions, CodexDirectTool, CodexDirectTransport,
    run_codex_direct_responses,
};
use serde_json::{Value, json};

struct FakeTransport {
    responses: VecDeque<Vec<Value>>,
    requests: Vec<Value>,
}

impl CodexDirectTransport for FakeTransport {
    fn stream_response(
        &mut self,
        request: &Value,
        on_event: &mut dyn FnMut(Value) -> Result<(), CodexDirectError>,
    ) -> Result<(), CodexDirectError> {
        self.requests.push(request.clone());
        for event in self.responses.pop_front().expect("fake response") {
            on_event(event)?;
        }
        Ok(())
    }
}

#[test]
fn codex_direct_streams_text_executes_tools_and_continues_the_same_turn() {
    let mut transport = FakeTransport {
        responses: VecDeque::from([
            vec![
                json!({"type":"response.output_text.delta","delta":"Checking "}),
                json!({"type":"response.output_item.done","item":{
                    "type":"function_call",
                    "name":"calendar_list",
                    "call_id":"call-1",
                    "arguments":"{\"days\":1}"
                }}),
                json!({"type":"response.completed","response":{
                    "id":"resp-1",
                    "usage":{"input_tokens":10,"output_tokens":2,"input_tokens_details":{"cached_tokens":3}},
                    "output":[{
                        "type":"function_call",
                        "name":"calendar_list",
                        "call_id":"call-1",
                        "arguments":"{\"days\":1}"
                    }]
                }})
            ],
            vec![
                json!({"type":"response.output_text.delta","delta":"done"}),
                json!({"type":"response.completed","response":{
                    "id":"resp-2",
                    "usage":{"input_tokens":6,"output_tokens":1,"input_tokens_details":{"cached_tokens":0}},
                    "output":[]
                }})
            ],
        ]),
        requests: Vec::new(),
    };
    let mut options = CodexDirectOptions::new(
        "gpt-test",
        "system",
        vec![json!({"role":"user","content":"calendar"})],
    );
    options.tools = vec![CodexDirectTool {
        name: "calendar_list".into(),
        description: Some("List calendar entries".into()),
        parameters: json!({"type":"object"}),
        source: json!({"providerIdentifier":"calendar","toolName":"list"}),
    }];

    let mut calls: Vec<(String, Value, String)> = Vec::new();
    let mut deltas: Vec<(String, String)> = Vec::new();
    let result = run_codex_direct_responses(
        &mut transport,
        &options,
        &mut |tool, args, call_id| {
            calls.push((tool.name.clone(), args, call_id.to_string()));
            Ok(json!({"events":["daily"]}))
        },
        &mut |delta, accumulated| {
            deltas.push((delta.to_string(), accumulated.to_string()));
        },
    )
    .expect("provider turn");

    assert_eq!(result.text, "Checking done");
    assert_eq!(result.response_id, "resp-2");
    assert_eq!(result.usage.input_tokens, 16);
    assert_eq!(result.usage.output_tokens, 3);
    assert_eq!(result.usage.cache_read_tokens, 3);
    assert_eq!(
        calls,
        vec![(
            "calendar_list".to_string(),
            json!({"days":1}),
            "call-1".to_string()
        )]
    );
    assert_eq!(
        deltas,
        vec![
            ("Checking ".to_string(), "Checking ".to_string()),
            ("done".to_string(), "Checking done".to_string())
        ]
    );
    assert_eq!(transport.requests.len(), 2);
    let second_input = transport.requests[1]["input"].as_array().expect("second input");
    assert!(second_input.iter().any(|item| item["type"] == "function_call_output"));
}

#[test]
fn codex_direct_rejects_a_stream_without_terminal_completion() {
    let mut transport = FakeTransport {
        responses: VecDeque::from([vec![json!({
            "type":"response.output_text.delta",
            "delta":"partial"
        })]]),
        requests: Vec::new(),
    };
    let options = CodexDirectOptions::new(
        "gpt-test",
        "system",
        vec![json!({"role":"user","content":"hello"})],
    );
    let error = run_codex_direct_responses(
        &mut transport,
        &options,
        &mut |_tool, _args, _call_id| Ok(Value::Null),
        &mut |_delta, _accumulated| {},
    )
    .expect_err("missing completion should fail");
    assert!(error.to_string().contains("response.completed"));
}
