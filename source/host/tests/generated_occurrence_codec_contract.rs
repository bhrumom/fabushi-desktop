use mahayana_host_runtime::transcript_mirror::generated_occurrence_codec::{
    CanonicalGeneratedToolJsonProjection, GeneratedToolJsonProjection,
    GeneratedToolProjection, GeneratedTranscriptOccurrenceCodec,
    RejectGeneratedToolJsonProjection, encode_generated_agent_tool_step,
    generated_tool_name,
};
use mahayana_host_runtime::transcript_mirror::transcript_occurrence_deriver::{
    DecodedTranscriptStep, DecodedTranscriptTurn, DecodedUserMessage,
    TranscriptOccurrenceCodec,
};
use serde_json::json;
use prost::Message;
use mahayana_host_runtime::runner::agent_v1_wire::{
    SendMessageArgs, SendMessageResult, SendMessageSuccess, SendMessageText,
    SendMessageToolCall, send_message_args, send_message_result,
};
use prost_types::{ListValue, Struct, Value as ProtobufValue, value::Kind};
use std::collections::BTreeMap;

fn encode_varint(mut value: u64, output: &mut Vec<u8>) {
    while value >= 0x80 {
        output.push((value as u8) | 0x80);
        value >>= 7;
    }
    output.push(value as u8);
}

fn field(number: u64, value: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    encode_varint((number << 3) | 2, &mut output);
    encode_varint(value.len() as u64, &mut output);
    output.extend_from_slice(value);
    output
}

fn concat(parts: &[Vec<u8>]) -> Vec<u8> {
    parts.iter().flatten().copied().collect()
}

#[derive(Clone, Copy)]
struct FixtureToolProjection;

impl GeneratedToolJsonProjection for FixtureToolProjection {
    fn project(
        &self,
        field_number: u64,
        bytes: &[u8],
    ) -> Result<Option<GeneratedToolProjection>, String> {
        assert_eq!(field_number, 15);
        assert_eq!(bytes, [0xaa, 0xbb]);
        Ok(Some(GeneratedToolProjection {
            name_override: Some("drive_search".into()),
            input: json!({"query":"quarterly"}),
            result: Some(json!({"count":2})),
        }))
    }
}

#[test]
fn generated_turn_and_user_message_shapes_match_frozen_agent_v1_fields() {
    let codec = GeneratedTranscriptOccurrenceCodec::new(RejectGeneratedToolJsonProjection);
    let agent_turn = concat(&[
        field(1, &[0x09]),
        field(2, &[0x0a]),
        field(2, &[0x0b]),
    ]);
    assert_eq!(
        codec.decode_turn(&field(1, &agent_turn)).expect("agent turn"),
        DecodedTranscriptTurn::Agent {
            user_message: vec![0x09],
            steps: vec![vec![0x0a], vec![0x0b]],
        }
    );
    assert_eq!(
        codec.decode_turn(&field(2, &[])).expect("shell turn"),
        DecodedTranscriptTurn::Shell
    );

    let user = concat(&[
        field(1, b"hello"),
        field(18, &[0xde, 0xad]),
    ]);
    assert_eq!(
        codec.decode_user_message(&user).expect("user"),
        DecodedUserMessage {
            text: "hello".into(),
            text_blob_id: Some(vec![0xde, 0xad]),
        }
    );
}

#[test]
fn generated_assistant_thinking_and_tool_boundaries_match_frozen_cases() {
    let codec = GeneratedTranscriptOccurrenceCodec::new(FixtureToolProjection);
    assert_eq!(
        codec.decode_step(&field(1, &field(1, b"answer"))).expect("assistant"),
        DecodedTranscriptStep::Assistant {
            text: "answer".into(),
        }
    );
    assert_eq!(
        codec.decode_step(&field(3, &field(1, b"thinking"))).expect("thinking"),
        DecodedTranscriptStep::Thinking {
            text: "thinking".into(),
        }
    );

    let tool_call = field(15, &[0xaa, 0xbb]);
    assert_eq!(
        codec.decode_step(&field(2, &tool_call)).expect("tool"),
        DecodedTranscriptStep::Tool {
            name: "drive_search".into(),
            input: json!({"query":"quarterly"}),
            result: Some(json!({"count":2})),
        }
    );
}

#[test]
fn tool_steps_fail_closed_without_canonical_generated_json_projection() {
    let codec = GeneratedTranscriptOccurrenceCodec::new(RejectGeneratedToolJsonProjection);
    let error = codec
        .decode_step(&field(2, &field(1, &[0x01])))
        .expect_err("tool projection must be explicit");
    assert!(error.contains("canonical generated tool JSON projection is required for shell"));
}

#[test]
fn frozen_tool_case_field_numbers_map_to_stable_names() {
    assert_eq!(generated_tool_name(1), Some("shell"));
    assert_eq!(generated_tool_name(15), Some("mcp"));
    assert_eq!(generated_tool_name(30), Some("computer_use"));
    assert_eq!(generated_tool_name(55), Some("send_message"));
    assert_eq!(generated_tool_name(77), Some("stop_agent"));
    assert_eq!(generated_tool_name(2), None);
}


#[test]
fn canonical_generated_projection_matches_frozen_send_message_to_json_semantics() {
    let tool = SendMessageToolCall {
        args: Some(SendMessageArgs {
            message: Some(send_message_args::Message::Text(SendMessageText {
                content: "hello".into(),
            })),
        }),
        result: Some(SendMessageResult {
            result: Some(send_message_result::Result::Success(SendMessageSuccess {
                timestamp: 123,
                message_id: "msg-1".into(),
            })),
        }),
    };
    let codec = GeneratedTranscriptOccurrenceCodec::new(CanonicalGeneratedToolJsonProjection);
    let step = field(2, &field(55, &tool.encode_to_vec()));
    assert_eq!(
        codec.decode_step(&step).expect("canonical send-message projection"),
        DecodedTranscriptStep::Tool {
            name: "send_message".into(),
            input: json!({"text":{"content":"hello"}}),
            result: Some(json!({
                "success":{
                    "timestamp":"123",
                    "messageId":"msg-1"
                }
            })),
        }
    );
}

#[test]
fn canonical_generated_projection_ignores_unknown_tool_message_fields() {
    let tool = SendMessageToolCall {
        args: Some(SendMessageArgs {
            message: Some(send_message_args::Message::Text(SendMessageText {
                content: "hello".into(),
            })),
        }),
        result: None,
    };
    let mut tool_bytes = tool.encode_to_vec();
    tool_bytes.extend(field(99, b"future"));
    let codec = GeneratedTranscriptOccurrenceCodec::new(CanonicalGeneratedToolJsonProjection);
    let step = field(2, &field(55, &tool_bytes));
    assert_eq!(
        codec.decode_step(&step).expect("unknown generated field"),
        DecodedTranscriptStep::Tool {
            name: "send_message".into(),
            input: json!({"text":{"content":"hello"}}),
            result: None,
        }
    );
}

#[test]
fn canonical_generated_tool_encoder_round_trips_send_message_json() {
    let step = encode_generated_agent_tool_step(
        "SendMessage",
        &json!({"text":{"content":"hello"}}),
        Some(&json!({
            "success":{
                "timestamp":"123",
                "messageId":"msg-1"
            }
        })),
        "tool-call-1",
        Some(100),
        Some(200),
    )
    .expect("encode canonical generated tool step");
    let codec = GeneratedTranscriptOccurrenceCodec::new(CanonicalGeneratedToolJsonProjection);
    assert_eq!(
        codec.decode_step(&step).expect("decode encoded tool step"),
        DecodedTranscriptStep::Tool {
            name: "send_message".into(),
            input: json!({"text":{"content":"hello"}}),
            result: Some(json!({
                "success":{
                    "timestamp":"123",
                    "messageId":"msg-1"
                }
            })),
        }
    );
}

// Independent wire oracle for the frozen agent_pb.ts / mcp_tool_pb.ts /
// mcp_exec_pb.ts fields. WKT decoding comes from prost-types, not this codec's
// generated descriptors or its JSON decoder.
#[derive(Clone, PartialEq, Message)]
struct McpWireStep {
    #[prost(message, optional, tag = "2")]
    tool_call: Option<McpWireToolCall>,
}

#[derive(Clone, PartialEq, Message)]
struct McpWireToolCall {
    #[prost(message, optional, tag = "15")]
    mcp: Option<McpWireCall>,
    #[prost(string, optional, tag = "57")]
    tool_call_id: Option<String>,
    #[prost(uint64, optional, tag = "59")]
    started_at_ms: Option<u64>,
    #[prost(uint64, optional, tag = "60")]
    completed_at_ms: Option<u64>,
}

#[derive(Clone, PartialEq, Message)]
struct McpWireCall {
    #[prost(message, optional, tag = "1")]
    args: Option<McpWireArgs>,
    #[prost(message, optional, tag = "2")]
    result: Option<McpWireResult>,
}

#[derive(Clone, PartialEq, Message)]
struct McpWireArgs {
    #[prost(string, tag = "1")]
    name: String,
    #[prost(btree_map = "string, message", tag = "2")]
    args: BTreeMap<String, ProtobufValue>,
    #[prost(string, tag = "3")]
    tool_call_id: String,
    #[prost(string, tag = "4")]
    provider_identifier: String,
    #[prost(string, tag = "5")]
    tool_name: String,
    #[prost(string, tag = "9")]
    server_identifier: String,
}

#[derive(Clone, PartialEq, Message)]
struct McpWireResult {
    #[prost(message, optional, tag = "1")]
    success: Option<McpWireSuccess>,
}

#[derive(Clone, PartialEq, Message)]
struct McpWireSuccess {
    #[prost(bool, tag = "2")]
    is_error: bool,
    #[prost(message, optional, tag = "3")]
    structured_content: Option<Struct>,
}

fn protobuf_value(kind: Kind) -> ProtobufValue {
    ProtobufValue { kind: Some(kind) }
}

#[test]
fn canonical_generated_mcp_receipt_matches_independent_protobuf_wire() {
    // The production restart bridge returns this nonempty result after its
    // one actual effect; encoding it must terminate before checkpoint storage.
    let step = encode_generated_agent_tool_step(
        "mcp",
        &json!({
            "name":"b",
            "args":{},
            "toolCallId":"original-terminal-effect",
            "providerIdentifier":"original-turn-contract",
            "toolName":"b",
            "serverIdentifier":"original-turn-contract"
        }),
        Some(&json!({"success":{"isError":false,"structuredContent":{"receipt":"committed"}}})),
        "original-terminal-effect",
        Some(100),
        Some(200),
    ).expect("encode the nonempty production MCP receipt");
    let tool_call = McpWireStep::decode(step.as_slice()).expect("independent AgentStep decode")
        .tool_call.expect("ToolCall at field 2");
    assert_eq!(tool_call.tool_call_id.as_deref(), Some("original-terminal-effect"));
    assert_eq!(tool_call.started_at_ms, Some(100));
    assert_eq!(tool_call.completed_at_ms, Some(200));
    let mcp = tool_call.mcp.expect("McpToolCall at field 15");
    let args = mcp.args.expect("McpArgs at field 1");
    assert_eq!(args.name, "b");
    assert!(args.args.is_empty());
    assert_eq!(args.tool_call_id, "original-terminal-effect");
    assert_eq!(args.provider_identifier, "original-turn-contract");
    assert_eq!(args.tool_name, "b");
    assert_eq!(args.server_identifier, "original-turn-contract");
    let success = mcp.result.expect("McpToolResult at field 2")
        .success.expect("McpSuccess at field 1");
    assert!(!success.is_error);
    assert_eq!(
        success.structured_content.expect("structuredContent at field 3"),
        Struct {
            fields: BTreeMap::from([(
                "receipt".into(),
                protobuf_value(Kind::StringValue("committed".into())),
            )]),
        }
    );
}

#[test]
fn canonical_generated_mcp_json_values_match_independent_protobuf_wire() {
    let cases = [
        ("null", json!(null), Kind::NullValue(0)),
        ("false", json!(false), Kind::BoolValue(false)),
        ("true", json!(true), Kind::BoolValue(true)),
        ("number", json!(-3.5), Kind::NumberValue(-3.5)),
        ("zero", json!(0), Kind::NumberValue(0.0)),
        ("string", json!("text"), Kind::StringValue("text".into())),
        ("empty_string", json!(""), Kind::StringValue(String::new())),
        ("empty_object", json!({}), Kind::StructValue(Struct { fields: BTreeMap::new() })),
        ("empty_list", json!([]), Kind::ListValue(ListValue { values: vec![] })),
        (
            "nested_list",
            json!([null, false, 1.25, "text", {}, [], {"values":[0]}]),
            Kind::ListValue(ListValue {
                values: vec![
                    protobuf_value(Kind::NullValue(0)),
                    protobuf_value(Kind::BoolValue(false)),
                    protobuf_value(Kind::NumberValue(1.25)),
                    protobuf_value(Kind::StringValue("text".into())),
                    protobuf_value(Kind::StructValue(Struct { fields: BTreeMap::new() })),
                    protobuf_value(Kind::ListValue(ListValue { values: vec![] })),
                    protobuf_value(Kind::StructValue(Struct {
                        fields: BTreeMap::from([(
                            "values".into(),
                            protobuf_value(Kind::ListValue(ListValue {
                                values: vec![protobuf_value(Kind::NumberValue(0.0))],
                            })),
                        )]),
                    })),
                ],
            }),
        ),
        (
            "wrapper_like_keys_are_user_data",
            json!({
                "stringValue":"literal",
                "structValue":{"fields":"literal"},
                "fields":{"values":{}},
                "values":[{"listValue":[]}]
            }),
            Kind::StructValue(Struct {
                fields: BTreeMap::from([
                    ("stringValue".into(), protobuf_value(Kind::StringValue("literal".into()))),
                    ("structValue".into(), protobuf_value(Kind::StructValue(Struct {
                        fields: BTreeMap::from([(
                            "fields".into(), protobuf_value(Kind::StringValue("literal".into())),
                        )]),
                    }))),
                    ("fields".into(), protobuf_value(Kind::StructValue(Struct {
                        fields: BTreeMap::from([(
                            "values".into(), protobuf_value(Kind::StructValue(Struct { fields: BTreeMap::new() })),
                        )]),
                    }))),
                    ("values".into(), protobuf_value(Kind::ListValue(ListValue {
                        values: vec![protobuf_value(Kind::StructValue(Struct {
                            fields: BTreeMap::from([(
                                "listValue".into(), protobuf_value(Kind::ListValue(ListValue { values: vec![] })),
                            )]),
                        }))],
                    }))),
                ]),
            }),
        ),
    ];
    for (case, value, expected_kind) in cases {
        let step = encode_generated_agent_tool_step(
            "mcp",
            &json!({"args":{"value":value.clone()}}),
            Some(&json!({"success":{"structuredContent":{"value":value}}})),
            "json-values",
            None,
            None,
        ).unwrap_or_else(|error| panic!("encode {case}: {error}"));
        let mcp = McpWireStep::decode(step.as_slice()).expect("independent AgentStep decode")
            .tool_call.expect("ToolCall").mcp.expect("McpToolCall");
        let expected = BTreeMap::from([("value".into(), protobuf_value(expected_kind))]);
        assert_eq!(mcp.args.expect("McpArgs").args, expected, "MCP args: {case}");
        assert_eq!(
            mcp.result.expect("McpToolResult").success.expect("McpSuccess")
                .structured_content.expect("structuredContent").fields,
            expected,
            "MCP structuredContent: {case}"
        );
    }
}
