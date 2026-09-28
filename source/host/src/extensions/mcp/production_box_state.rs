use crate::r#box::generated_production::ProductionBoxResourceAccessor;
use super::mcp_service::BoxServerStatus;
use prost::{Message, Oneof};
use std::sync::Mutex;

pub const MCP_TOOL_EXEC_FIELD_NUMBER: u32 = 11;
pub const MCP_STATE_EXEC_FIELD_NUMBER: u32 = 36;

#[derive(Clone, PartialEq, Message)]
struct McpStateExecArgs {
    #[prost(string, repeated, tag = "1")]
    server_identifiers: Vec<String>,
    #[prost(bool, tag = "2")]
    kick_only: bool,
}

#[derive(Clone, PartialEq, Message)]
struct McpStateExecResult {
    #[prost(oneof = "mcp_state_exec_result::Result", tags = "1, 2, 3")]
    result: Option<mcp_state_exec_result::Result>,
}

mod mcp_state_exec_result {
    use super::{McpStateError, McpStateRejected, McpStateSuccess};
    use prost::Oneof;

    #[derive(Clone, PartialEq, Oneof)]
    pub enum Result {
        #[prost(message, tag = "1")]
        Success(McpStateSuccess),
        #[prost(message, tag = "2")]
        Error(McpStateError),
        #[prost(message, tag = "3")]
        Rejected(McpStateRejected),
    }
}

#[derive(Clone, PartialEq, Message)]
struct McpStateSuccess {
    #[prost(message, repeated, tag = "1")]
    servers: Vec<McpStateServer>,
}

#[derive(Clone, PartialEq, Message)]
struct McpStateError {
    #[prost(string, tag = "1")]
    error: String,
}

#[derive(Clone, PartialEq, Message)]
struct McpStateRejected {
    #[prost(string, tag = "1")]
    reason: String,
}

#[derive(Clone, PartialEq, Message)]
struct OpaqueMcpToolDefinition {}

#[derive(Clone, PartialEq, Message)]
struct McpStateServer {
    #[prost(string, tag = "1")]
    server_name: String,
    #[prost(string, tag = "2")]
    server_identifier: String,
    #[prost(message, repeated, tag = "5")]
    tools: Vec<OpaqueMcpToolDefinition>,
    #[prost(string, optional, tag = "7")]
    status: Option<String>,
    #[prost(string, optional, tag = "8")]
    error_message: Option<String>,
}

pub struct ProductionBoxMcpStateLoader {
    accessor: Mutex<ProductionBoxResourceAccessor>,
}

impl ProductionBoxMcpStateLoader {
    pub fn new(accessor: ProductionBoxResourceAccessor) -> Self {
        Self {
            accessor: Mutex::new(accessor),
        }
    }

    pub fn list_servers(
        &self,
        server_identifiers: &[String],
        kick_only: bool,
    ) -> Result<Vec<BoxServerStatus>, String> {
        let args = encode_mcp_state_args(server_identifiers, kick_only);
        let bytes = self
            .accessor
            .lock()
            .map_err(|_| "production Box MCP state accessor lock poisoned".to_string())?
            .execute_raw_resource(&(), MCP_STATE_EXEC_FIELD_NUMBER, args)
            .map_err(|error| error.to_string())?;
        decode_mcp_state_result(&bytes)
    }
}

pub fn encode_mcp_state_args(server_identifiers: &[String], kick_only: bool) -> Vec<u8> {
    McpStateExecArgs {
        server_identifiers: server_identifiers.to_vec(),
        kick_only,
    }
    .encode_to_vec()
}

pub fn decode_mcp_state_result(bytes: &[u8]) -> Result<Vec<BoxServerStatus>, String> {
    let result = McpStateExecResult::decode(bytes)
        .map_err(|error| format!("invalid mcp_state_exec_result protobuf: {error}"))?;
    match result.result {
        Some(mcp_state_exec_result::Result::Success(success)) => Ok(success
            .servers
            .into_iter()
            .map(|server| BoxServerStatus {
                server_identifier: server.server_identifier,
                status: server.status.unwrap_or_else(|| "connected".to_string()),
                status_detail: server
                    .error_message
                    .filter(|message| !message.trim().is_empty()),
                tool_count: server.tools.len(),
            })
            .collect()),
        Some(mcp_state_exec_result::Result::Error(error)) => {
            Err(format!("Box MCP state failed: {}", error.error))
        }
        Some(mcp_state_exec_result::Result::Rejected(rejected)) => {
            Err(format!("Box MCP state rejected: {}", rejected.reason))
        }
        None => Err("Box MCP state returned no result".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_mcp_state_wire_contract_is_field_36_and_preserves_kick_only() {
        assert_eq!(MCP_STATE_EXEC_FIELD_NUMBER, 36);
        let encoded = encode_mcp_state_args(&["calendar".into(), "github".into()], true);
        let decoded = McpStateExecArgs::decode(encoded.as_slice()).expect("decode args");
        assert_eq!(decoded.server_identifiers, vec!["calendar", "github"]);
        assert!(decoded.kick_only);
    }

    #[test]
    fn decodes_box_server_status_without_reimplementing_tool_schema() {
        let bytes = McpStateExecResult {
            result: Some(mcp_state_exec_result::Result::Success(McpStateSuccess {
                servers: vec![McpStateServer {
                    server_name: "Calendar".into(),
                    server_identifier: "calendar".into(),
                    tools: vec![OpaqueMcpToolDefinition {}, OpaqueMcpToolDefinition {}],
                    status: Some("connected".into()),
                    error_message: None,
                }],
            })),
        }
        .encode_to_vec();
        let servers = decode_mcp_state_result(&bytes).expect("decode state");
        assert_eq!(servers[0].server_identifier, "calendar");
        assert_eq!(servers[0].status, "connected");
        assert_eq!(servers[0].tool_count, 2);
    }
}


pub fn decode_hex_payload(raw: &str) -> Result<Vec<u8>, String> {
    let raw = raw.trim();
    if raw.len() % 2 != 0 {
        return Err("Box MCP protobuf hex payload must have even length".into());
    }
    raw.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = (pair[0] as char)
                .to_digit(16)
                .ok_or_else(|| "Box MCP protobuf payload contains non-hex data".to_string())?;
            let low = (pair[1] as char)
                .to_digit(16)
                .ok_or_else(|| "Box MCP protobuf payload contains non-hex data".to_string())?;
            Ok(((high << 4) | low) as u8)
        })
        .collect()
}

pub fn encode_hex_payload(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

pub fn execute_box_mcp_raw(
    accessor: &mut ProductionBoxResourceAccessor,
    field_number: u32,
    payload_hex: &str,
) -> Result<String, String> {
    if !matches!(field_number, MCP_TOOL_EXEC_FIELD_NUMBER | MCP_STATE_EXEC_FIELD_NUMBER) {
        return Err(format!("unsupported Box MCP ExecService field {field_number}"));
    }
    let payload = decode_hex_payload(payload_hex)?;
    let response = accessor
        .execute_raw_resource(&(), field_number, payload)
        .map_err(|error| error.to_string())?;
    Ok(encode_hex_payload(&response))
}
