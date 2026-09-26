use mahayana_host_runtime::transcript_mirror::conversation_state_binary::{
    TranscriptMirrorProtobufDecodeError, decode_conversation_state_recovery_fields,
    decode_summary_archive_message_ids, decode_transcript_mirror_conversation_state,
};

fn varint(mut value: u64) -> Vec<u8> {
    let mut output = Vec::new();
    while value >= 0x80 {
        output.push((value as u8) | 0x80);
        value >>= 7;
    }
    output.push(value as u8);
    output
}

fn length_delimited(field: u64, value: &[u8]) -> Vec<u8> {
    let mut output = varint((field << 3) | 2);
    output.extend(varint(value.len() as u64));
    output.extend_from_slice(value);
    output
}

#[test]
fn frozen_conversation_state_fields_decode_and_unknown_fields_skip() {
    let mut bytes = Vec::new();
    bytes.extend(length_delimited(1, b"prompt-a"));
    bytes.extend(length_delimited(8, b"turn-a"));
    bytes.extend(length_delimited(3, b"todo-a"));
    bytes.extend(length_delimited(6, b"summary-a"));
    bytes.extend(length_delimited(13, b"archive-a"));
    bytes.extend(varint((2 << 3) | 0));
    bytes.extend(varint(7));
    bytes.extend(varint((4 << 3) | 1));
    bytes.extend([0u8; 8]);
    bytes.extend(varint((5 << 3) | 5));
    bytes.extend([0u8; 4]);
    bytes.extend(length_delimited(20, b"ignored"));

    let mirror = decode_transcript_mirror_conversation_state(&bytes).expect("mirror state");
    assert_eq!(mirror.root_prompt_messages_json, vec![b"prompt-a".to_vec()]);
    assert_eq!(mirror.turns, vec![b"turn-a".to_vec()]);
    assert_eq!(mirror.summary_archives, vec![b"archive-a".to_vec()]);

    let recovery = decode_conversation_state_recovery_fields(&bytes).expect("recovery state");
    assert_eq!(recovery.todos, vec![b"todo-a".to_vec()]);
    assert_eq!(recovery.summary, Some(b"summary-a".to_vec()));
}

#[test]
fn summary_archive_decoder_reads_only_summarized_message_ids() {
    let mut bytes = Vec::new();
    bytes.extend(length_delimited(1, b"message-a"));
    bytes.extend(length_delimited(2, b"ignored"));
    bytes.extend(length_delimited(1, b"message-b"));
    assert_eq!(
        decode_summary_archive_message_ids(&bytes).expect("archive"),
        vec![b"message-a".to_vec(), b"message-b".to_vec()]
    );
}

#[test]
fn malformed_wire_shapes_fail_with_specific_decoder_errors() {
    assert_eq!(
        decode_transcript_mirror_conversation_state(&[0x80]).unwrap_err(),
        TranscriptMirrorProtobufDecodeError::TruncatedVarint
    );
    assert_eq!(
        decode_transcript_mirror_conversation_state(&[0x00]).unwrap_err(),
        TranscriptMirrorProtobufDecodeError::InvalidFieldNumber
    );

    let mut truncated = varint((8 << 3) | 2);
    truncated.extend(varint(4));
    truncated.extend([1, 2]);
    assert_eq!(
        decode_transcript_mirror_conversation_state(&truncated).unwrap_err(),
        TranscriptMirrorProtobufDecodeError::TruncatedBytes
    );

    let mut group = varint((9 << 3) | 3);
    group.extend(varint((10 << 3) | 4));
    assert_eq!(
        decode_transcript_mirror_conversation_state(&group).unwrap_err(),
        TranscriptMirrorProtobufDecodeError::MismatchedGroup
    );
}
