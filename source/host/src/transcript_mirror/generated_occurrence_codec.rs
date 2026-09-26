use serde_json::Value;

use super::transcript_occurrence_deriver::{
    DecodedTranscriptStep, DecodedTranscriptTurn, DecodedUserMessage,
    TranscriptOccurrenceCodec,
};

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedToolProjection {
    pub name_override: Option<String>,
    pub input: Value,
    pub result: Option<Value>,
}

pub trait GeneratedToolJsonProjection: Send + Sync {
    fn project(
        &self,
        tool_field_number: u64,
        tool_message: &[u8],
    ) -> Result<Option<GeneratedToolProjection>, String>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RejectGeneratedToolJsonProjection;

impl GeneratedToolJsonProjection for RejectGeneratedToolJsonProjection {
    fn project(
        &self,
        _tool_field_number: u64,
        _tool_message: &[u8],
    ) -> Result<Option<GeneratedToolProjection>, String> {
        Ok(None)
    }
}

pub struct GeneratedTranscriptOccurrenceCodec<Projection> {
    tool_json: Projection,
}

impl<Projection> GeneratedTranscriptOccurrenceCodec<Projection> {
    pub fn new(tool_json: Projection) -> Self {
        Self { tool_json }
    }

    pub fn tool_json_projection(&self) -> &Projection {
        &self.tool_json
    }
}

impl<Projection> TranscriptOccurrenceCodec
    for GeneratedTranscriptOccurrenceCodec<Projection>
where
    Projection: GeneratedToolJsonProjection,
{
    fn decode_turn(&self, bytes: &[u8]) -> Result<DecodedTranscriptTurn, String> {
        let mut selected = DecodedTranscriptTurn::Undefined;
        for field in length_delimited_fields(bytes)? {
            match field.number {
                1 => {
                    let mut user_message = Vec::new();
                    let mut steps = Vec::new();
                    for nested in length_delimited_fields(field.bytes)? {
                        match nested.number {
                            1 => user_message = nested.bytes.to_vec(),
                            2 => steps.push(nested.bytes.to_vec()),
                            _ => {}
                        }
                    }
                    selected = DecodedTranscriptTurn::Agent {
                        user_message,
                        steps,
                    };
                }
                2 => selected = DecodedTranscriptTurn::Shell,
                _ => {}
            }
        }
        Ok(selected)
    }

    fn decode_user_message(&self, bytes: &[u8]) -> Result<DecodedUserMessage, String> {
        let mut text = String::new();
        let mut text_blob_id = None;
        for field in length_delimited_fields(bytes)? {
            match field.number {
                1 => text = String::from_utf8_lossy(field.bytes).into_owned(),
                18 => text_blob_id = Some(field.bytes.to_vec()),
                _ => {}
            }
        }
        Ok(DecodedUserMessage { text, text_blob_id })
    }

    fn decode_step(&self, bytes: &[u8]) -> Result<DecodedTranscriptStep, String> {
        let mut selected = DecodedTranscriptStep::Undefined;
        for field in length_delimited_fields(bytes)? {
            selected = match field.number {
                1 => DecodedTranscriptStep::Assistant {
                    text: decode_text_message(field.bytes)?,
                },
                2 => self.decode_tool_call(field.bytes)?,
                3 => DecodedTranscriptStep::Thinking {
                    text: decode_text_message(field.bytes)?,
                },
                _ => selected,
            };
        }
        Ok(selected)
    }
}

impl<Projection> GeneratedTranscriptOccurrenceCodec<Projection>
where
    Projection: GeneratedToolJsonProjection,
{
    fn decode_tool_call(&self, bytes: &[u8]) -> Result<DecodedTranscriptStep, String> {
        let mut selected: Option<(u64, &[u8])> = None;
        for field in length_delimited_fields(bytes)? {
            if generated_tool_name(field.number).is_some() {
                selected = Some((field.number, field.bytes));
            }
        }
        let Some((field_number, tool_message)) = selected else {
            return Ok(DecodedTranscriptStep::Undefined);
        };
        let static_name = generated_tool_name(field_number)
            .ok_or_else(|| format!("unsupported generated tool field {field_number}"))?;
        let projection = self
            .tool_json
            .project(field_number, tool_message)?
            .ok_or_else(|| {
                format!(
                    "canonical generated tool JSON projection is required for {static_name}"
                )
            })?;
        let name = if field_number == 15 {
            projection
                .name_override
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| "mcp".to_string())
        } else {
            static_name.to_string()
        };
        Ok(DecodedTranscriptStep::Tool {
            name,
            input: projection.input,
            result: projection.result,
        })
    }
}

fn decode_text_message(bytes: &[u8]) -> Result<String, String> {
    let mut text = String::new();
    for field in length_delimited_fields(bytes)? {
        if field.number == 1 {
            text = String::from_utf8_lossy(field.bytes).into_owned();
        }
    }
    Ok(text)
}

#[derive(Clone, Copy)]
struct LengthDelimitedField<'a> {
    number: u64,
    bytes: &'a [u8],
}

fn length_delimited_fields(data: &[u8]) -> Result<Vec<LengthDelimitedField<'_>>, String> {
    let mut position = 0usize;
    let mut fields = Vec::new();
    while position < data.len() {
        let tag = read_varint(data, &mut position)?;
        let field_number = tag >> 3;
        let wire_type = (tag & 0x07) as u8;
        if field_number == 0 {
            return Err("protobuf field number 0 is invalid".into());
        }
        if wire_type == 2 {
            let bytes = read_bytes(data, &mut position)?;
            fields.push(LengthDelimitedField {
                number: field_number,
                bytes,
            });
        } else {
            skip_field(data, &mut position, wire_type, field_number)?;
        }
    }
    Ok(fields)
}

fn read_varint(data: &[u8], position: &mut usize) -> Result<u64, String> {
    let mut value = 0u64;
    for shift in (0..70).step_by(7) {
        let byte = *data
            .get(*position)
            .ok_or_else(|| "truncated protobuf varint".to_string())?;
        *position += 1;
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err("protobuf varint exceeds 64 bits".into())
}

fn read_bytes<'a>(data: &'a [u8], position: &mut usize) -> Result<&'a [u8], String> {
    let length: usize = read_varint(data, position)?
        .try_into()
        .map_err(|_| "protobuf length does not fit usize".to_string())?;
    let end = position
        .checked_add(length)
        .ok_or_else(|| "protobuf length overflow".to_string())?;
    let value = data
        .get(*position..end)
        .ok_or_else(|| "truncated protobuf length-delimited field".to_string())?;
    *position = end;
    Ok(value)
}

fn skip_field(
    data: &[u8],
    position: &mut usize,
    wire_type: u8,
    field_number: u64,
) -> Result<(), String> {
    match wire_type {
        0 => {
            let _ = read_varint(data, position)?;
        }
        1 => {
            *position = position
                .checked_add(8)
                .ok_or_else(|| "protobuf fixed64 overflow".to_string())?;
            if *position > data.len() {
                return Err("truncated protobuf fixed64".into());
            }
        }
        2 => {
            let _ = read_bytes(data, position)?;
        }
        3 => loop {
            let tag = read_varint(data, position)?;
            let nested_field = tag >> 3;
            let nested_wire = (tag & 0x07) as u8;
            if nested_wire == 4 {
                if nested_field != field_number {
                    return Err("protobuf group end field mismatch".into());
                }
                break;
            }
            skip_field(data, position, nested_wire, nested_field)?;
        },
        4 => return Err("unexpected protobuf end-group field".into()),
        5 => {
            *position = position
                .checked_add(4)
                .ok_or_else(|| "protobuf fixed32 overflow".to_string())?;
            if *position > data.len() {
                return Err("truncated protobuf fixed32".into());
            }
        }
        _ => return Err(format!("unsupported protobuf wire type {wire_type}")),
    }
    Ok(())
}

pub fn generated_tool_name(field_number: u64) -> Option<&'static str> {
    Some(match field_number {
        1 => "shell",
        3 => "delete",
        4 => "glob",
        5 => "grep",
        8 => "read",
        9 => "update_todos",
        10 => "read_todos",
        12 => "edit",
        13 => "ls",
        14 => "read_lints",
        15 => "mcp",
        16 => "sem_search",
        17 => "create_plan",
        18 => "web_search",
        19 => "task",
        20 => "list_mcp_resources",
        21 => "read_mcp_resource",
        22 => "apply_agent_diff",
        23 => "ask_question",
        24 => "fetch",
        25 => "switch_mode",
        28 => "generate_image",
        29 => "record_screen",
        30 => "computer_use",
        31 => "write_shell_stdin",
        32 => "reflect",
        33 => "setup_vm_environment",
        34 => "truncated",
        35 => "start_grind_execution",
        36 => "start_grind_planning",
        37 => "web_fetch",
        38 => "report_bugfix_results",
        39 => "ai_attribution",
        40 => "pr_management",
        41 => "mcp_auth",
        42 => "await",
        43 => "blame_by_file_path",
        44 => "get_mcp_tools",
        45 => "report_bug",
        46 => "set_active_branch",
        48 => "communicate_update",
        49 => "send_final_summary",
        50 => "update_pr_code_tour",
        51 => "replace_env",
        52 => "edit_pr_labels",
        53 => "record_ci_investigation_findings",
        55 => "send_message",
        56 => "fetch_cloud_agent_data",
        58 => "send_to_user",
        61 => "pi_read",
        62 => "pi_bash",
        63 => "pi_edit",
        64 => "pi_write",
        65 => "pi_grep",
        66 => "pi_find",
        67 => "pi_ls",
        68 => "connect_scm",
        69 => "search_conversations",
        70 => "create_goal",
        71 => "update_goal",
        72 => "adopt",
        73 => "get_agent_status",
        74 => "send_to_agent",
        75 => "read_agent_transcript",
        76 => "create_agent",
        77 => "stop_agent",
        _ => return None,
    })
}
