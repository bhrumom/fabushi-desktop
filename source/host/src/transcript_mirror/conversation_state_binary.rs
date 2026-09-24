const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TranscriptMirrorConversationState {
    pub root_prompt_messages_json: Vec<Vec<u8>>,
    pub turns: Vec<Vec<u8>>,
    pub summary_archives: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConversationStateRecoveryFields {
    pub root_prompt_messages_json: Vec<Vec<u8>>,
    pub turns: Vec<Vec<u8>>,
    pub todos: Vec<Vec<u8>>,
    pub summary: Option<Vec<u8>>,
    pub summary_archives: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TranscriptMirrorProtobufDecodeError {
    #[error("truncated protobuf varint")]
    TruncatedVarint,
    #[error("protobuf varint exceeds the safe integer range")]
    UnsafeVarint,
    #[error("invalid protobuf varint")]
    InvalidVarint,
    #[error("truncated protobuf bytes field")]
    TruncatedBytes,
    #[error("mismatched protobuf group")]
    MismatchedGroup,
    #[error("unterminated protobuf group")]
    UnterminatedGroup,
    #[error("unexpected protobuf end group")]
    UnexpectedEndGroup,
    #[error("invalid protobuf wire type")]
    InvalidWireType,
    #[error("truncated protobuf field")]
    TruncatedField,
    #[error("invalid protobuf field number")]
    InvalidFieldNumber,
}

#[derive(Debug, Clone, Copy)]
struct Cursor {
    offset: usize,
}

fn read_varint(
    bytes: &[u8],
    cursor: &mut Cursor,
) -> Result<u64, TranscriptMirrorProtobufDecodeError> {
    let mut value = 0u64;
    for shift in (0..70).step_by(7) {
        let byte = *bytes
            .get(cursor.offset)
            .ok_or(TranscriptMirrorProtobufDecodeError::TruncatedVarint)?;
        cursor.offset += 1;
        let payload = u64::from(byte & 0x7f);
        if shift == 63 && payload > 1 {
            return Err(TranscriptMirrorProtobufDecodeError::InvalidVarint);
        }
        value |= payload << shift;
        if byte & 0x80 == 0 {
            if value > MAX_SAFE_INTEGER {
                return Err(TranscriptMirrorProtobufDecodeError::UnsafeVarint);
            }
            return Ok(value);
        }
    }
    Err(TranscriptMirrorProtobufDecodeError::InvalidVarint)
}

fn read_bytes(
    bytes: &[u8],
    cursor: &mut Cursor,
) -> Result<Vec<u8>, TranscriptMirrorProtobufDecodeError> {
    let length = read_varint(bytes, cursor)?;
    let length: usize = length
        .try_into()
        .map_err(|_| TranscriptMirrorProtobufDecodeError::TruncatedBytes)?;
    let end = cursor
        .offset
        .checked_add(length)
        .ok_or(TranscriptMirrorProtobufDecodeError::TruncatedBytes)?;
    if end > bytes.len() {
        return Err(TranscriptMirrorProtobufDecodeError::TruncatedBytes);
    }
    let value = bytes[cursor.offset..end].to_vec();
    cursor.offset = end;
    Ok(value)
}

fn skip_field(
    bytes: &[u8],
    cursor: &mut Cursor,
    wire_type: u8,
    field_number: u64,
) -> Result<(), TranscriptMirrorProtobufDecodeError> {
    match wire_type {
        0 => {
            let _ = read_varint(bytes, cursor)?;
            return Ok(());
        }
        1 => {
            cursor.offset = cursor
                .offset
                .checked_add(8)
                .ok_or(TranscriptMirrorProtobufDecodeError::TruncatedField)?;
        }
        2 => {
            let length = read_varint(bytes, cursor)?;
            let length: usize = length
                .try_into()
                .map_err(|_| TranscriptMirrorProtobufDecodeError::TruncatedField)?;
            cursor.offset = cursor
                .offset
                .checked_add(length)
                .ok_or(TranscriptMirrorProtobufDecodeError::TruncatedField)?;
        }
        3 => {
            while cursor.offset < bytes.len() {
                let tag = read_varint(bytes, cursor)?;
                let nested_field = tag / 8;
                let nested_wire = (tag & 7) as u8;
                if nested_wire == 4 {
                    if nested_field != field_number {
                        return Err(TranscriptMirrorProtobufDecodeError::MismatchedGroup);
                    }
                    return Ok(());
                }
                skip_field(bytes, cursor, nested_wire, nested_field)?;
            }
            return Err(TranscriptMirrorProtobufDecodeError::UnterminatedGroup);
        }
        4 => return Err(TranscriptMirrorProtobufDecodeError::UnexpectedEndGroup),
        5 => {
            cursor.offset = cursor
                .offset
                .checked_add(4)
                .ok_or(TranscriptMirrorProtobufDecodeError::TruncatedField)?;
        }
        _ => return Err(TranscriptMirrorProtobufDecodeError::InvalidWireType),
    }
    if cursor.offset > bytes.len() {
        return Err(TranscriptMirrorProtobufDecodeError::TruncatedField);
    }
    Ok(())
}

pub fn decode_conversation_state_recovery_fields(
    bytes: &[u8],
) -> Result<ConversationStateRecoveryFields, TranscriptMirrorProtobufDecodeError> {
    let mut cursor = Cursor { offset: 0 };
    let mut state = ConversationStateRecoveryFields::default();

    while cursor.offset < bytes.len() {
        let tag = read_varint(bytes, &mut cursor)?;
        let field_number = tag / 8;
        let wire_type = (tag & 7) as u8;
        if field_number == 0 {
            return Err(TranscriptMirrorProtobufDecodeError::InvalidFieldNumber);
        }
        if wire_type == 2 {
            match field_number {
                1 => {
                    state.root_prompt_messages_json.push(read_bytes(bytes, &mut cursor)?);
                    continue;
                }
                3 => {
                    state.todos.push(read_bytes(bytes, &mut cursor)?);
                    continue;
                }
                6 => {
                    state.summary = Some(read_bytes(bytes, &mut cursor)?);
                    continue;
                }
                8 => {
                    state.turns.push(read_bytes(bytes, &mut cursor)?);
                    continue;
                }
                13 => {
                    state.summary_archives.push(read_bytes(bytes, &mut cursor)?);
                    continue;
                }
                _ => {}
            }
        }
        skip_field(bytes, &mut cursor, wire_type, field_number)?;
    }

    Ok(state)
}

pub fn decode_transcript_mirror_conversation_state(
    bytes: &[u8],
) -> Result<TranscriptMirrorConversationState, TranscriptMirrorProtobufDecodeError> {
    let decoded = decode_conversation_state_recovery_fields(bytes)?;
    Ok(TranscriptMirrorConversationState {
        root_prompt_messages_json: decoded.root_prompt_messages_json,
        turns: decoded.turns,
        summary_archives: decoded.summary_archives,
    })
}

pub fn decode_summary_archive_message_ids(
    bytes: &[u8],
) -> Result<Vec<Vec<u8>>, TranscriptMirrorProtobufDecodeError> {
    let mut cursor = Cursor { offset: 0 };
    let mut message_ids = Vec::new();
    while cursor.offset < bytes.len() {
        let tag = read_varint(bytes, &mut cursor)?;
        let field_number = tag / 8;
        let wire_type = (tag & 7) as u8;
        if field_number == 0 {
            return Err(TranscriptMirrorProtobufDecodeError::InvalidFieldNumber);
        }
        if field_number == 1 && wire_type == 2 {
            message_ids.push(read_bytes(bytes, &mut cursor)?);
            continue;
        }
        skip_field(bytes, &mut cursor, wire_type, field_number)?;
    }
    Ok(message_ids)
}
