use mahayana_host_runtime::extensions::inference::cursor_inference_transport::{
    CONNECT_END_STREAM_FLAG, CursorInferenceEvent, decode_cursor_connect_frames,
    encode_connect_envelope, encode_cursor_inference_request, encode_test_response,
};
use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderMessage, ProviderTokenUsage, RoutedToolDefinition,
};
use serde_json::json;

#[test]
fn cursor_connect_stream_decoder_preserves_text_thinking_tools_usage_and_finish() {
    let expected = vec![
        CursorInferenceEvent::TextDelta("hello".into()),
        CursorInferenceEvent::ThinkingDelta {
            text: "reason".into(),
            signature: Some("sig".into()),
        },
        CursorInferenceEvent::ToolCall {
            tool_call_id: "call-1".into(),
            tool_name: "calendar_list".into(),
            args: "{\"days\":1}".into(),
            complete: true,
            tool_index: Some(0),
        },
        CursorInferenceEvent::Usage(ProviderTokenUsage {
            input_tokens: 11,
            output_tokens: 7,
            cache_read_tokens: 3,
            cache_write_tokens: 2,
            reasoning_tokens: None,
        }),
        CursorInferenceEvent::Finished,
    ];
    let mut buffer = expected
        .iter()
        .cloned()
        .flat_map(encode_test_response)
        .collect::<Vec<_>>();
    let observed = decode_cursor_connect_frames(&mut buffer).expect("decode cursor stream");
    assert_eq!(observed, expected);
    assert!(buffer.is_empty());
}

#[test]
fn cursor_connect_decoder_waits_for_complete_frame_and_consumes_end_stream_metadata() {
    let frame = encode_test_response(CursorInferenceEvent::TextDelta("hello".into()));
    let split = frame.len() - 2;
    let mut buffer = frame[..split].to_vec();
    assert!(decode_cursor_connect_frames(&mut buffer)
        .expect("partial frame")
        .is_empty());
    buffer.extend_from_slice(&frame[split..]);
    assert_eq!(
        decode_cursor_connect_frames(&mut buffer).expect("complete frame"),
        vec![CursorInferenceEvent::TextDelta("hello".into())]
    );

    let metadata = br#"{}"#;
    let mut end = vec![CONNECT_END_STREAM_FLAG];
    end.extend_from_slice(&(metadata.len() as u32).to_be_bytes());
    end.extend_from_slice(metadata);
    assert!(decode_cursor_connect_frames(&mut end)
        .expect("end stream")
        .is_empty());
    assert!(end.is_empty());
}

#[test]
fn cursor_request_is_connect_framed_and_accepts_frozen_tool_schema() {
    let request = encode_cursor_inference_request(
        &[
            ProviderMessage {
                role: "system".into(),
                content: "system".into(),
            },
            ProviderMessage {
                role: "user".into(),
                content: "hello".into(),
            },
        ],
        &[RoutedToolDefinition {
            name: "calendar_list".into(),
            provider_identifier: "calendar".into(),
            tool_name: "list".into(),
            description: Some("List calendar entries".into()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "days": {"type": "integer"}
                }
            }),
        }],
    )
    .expect("encode request");

    assert_eq!(request[0], 0);
    let declared = u32::from_be_bytes([request[1], request[2], request[3], request[4]]) as usize;
    assert_eq!(declared, request.len() - 5);
    assert!(request.len() > 32);
}

#[test]
fn cursor_connect_decoder_rejects_server_end_stream_error() {
    let metadata = br#"{\"error\":{\"code\":\"unavailable\",\"message\":\"down\"}}"#;
    let mut frame = vec![CONNECT_END_STREAM_FLAG];
    frame.extend_from_slice(&(metadata.len() as u32).to_be_bytes());
    frame.extend_from_slice(metadata);
    let error = decode_cursor_connect_frames(&mut frame).expect_err("server error");
    assert!(error.to_string().contains("unavailable"));
}
