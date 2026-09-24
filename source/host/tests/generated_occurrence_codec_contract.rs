use mahayana_host_runtime::transcript_mirror::generated_occurrence_codec::{
    GeneratedToolJsonProjection, GeneratedToolProjection,
    GeneratedTranscriptOccurrenceCodec, RejectGeneratedToolJsonProjection,
    generated_tool_name,
};
use mahayana_host_runtime::transcript_mirror::transcript_occurrence_deriver::{
    DecodedTranscriptStep, DecodedTranscriptTurn, DecodedUserMessage,
    TranscriptOccurrenceCodec,
};
use serde_json::json;

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
