use std::collections::BTreeMap;
use std::fmt;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use url::Url;

use super::box_env::{BoxEnvironmentControlClient, BoxEnvironmentUpdate};
use super::box_mcp::{
    BoxMcpControlClient, BoxMcpLoadRequest, BoxMcpLoadResponse, ConnectErrorCode,
    ConnectErrorCodeSource,
};
use super::box_remote_accessor::{
    BoxEndpoint, BoxPingControlClient, BoxPingErrorMetadata, BoxTransportOptions, ConnectCode,
    create_box_transport,
};

pub const PING_PATH: &str = "/agent.v1.ControlService/Ping";
pub const UPDATE_ENVIRONMENT_VARIABLES_PATH: &str =
    "/agent.v1.ControlService/UpdateEnvironmentVariables";
pub const LOAD_MCP_SERVERS_PATH: &str = "/agent.v1.ControlService/LoadMcpServers";
pub const CONNECT_PROTOCOL_VERSION: &str = "1";
pub const PRODUCTION_BOX_RPC_TIMEOUT_MS: u64 = 15_000;

#[derive(Debug)]
pub enum ProductionBoxTransportError {
    InvalidBaseUrl(String),
    InvalidHeader(String),
    Io(std::io::Error),
    InvalidHttpResponse(String),
    HttpStatus { status: u16, body: String },
}

impl fmt::Display for ProductionBoxTransportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBaseUrl(message)
            | Self::InvalidHeader(message)
            | Self::InvalidHttpResponse(message) => formatter.write_str(message),
            Self::Io(error) => error.fmt(formatter),
            Self::HttpStatus { status, body } => {
                write!(formatter, "box ControlService returned HTTP {status}")?;
                if !body.is_empty() {
                    write!(formatter, ": {body}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ProductionBoxTransportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ProductionBoxTransportError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

fn http_status_connect_code(status: u16) -> Option<i64> {
    match status {
        400 => Some(3),
        401 => Some(16),
        403 => Some(7),
        404 => Some(5),
        409 => Some(10),
        429 => Some(8),
        499 => Some(1),
        501 => Some(12),
        503 => Some(14),
        504 => Some(4),
        _ => None,
    }
}

impl BoxPingErrorMetadata for ProductionBoxTransportError {
    fn connect_code(&self) -> Option<ConnectCode> {
        match self {
            Self::Io(error) if error.kind() == std::io::ErrorKind::TimedOut => {
                Some(ConnectCode::Number(4))
            }
            Self::HttpStatus { status, .. } => {
                http_status_connect_code(*status).map(ConnectCode::Number)
            }
            _ => None,
        }
    }

    fn system_errno(&self) -> Option<&str> {
        match self {
            Self::Io(error) if error.kind() == std::io::ErrorKind::ConnectionRefused => {
                Some("ECONNREFUSED")
            }
            _ => None,
        }
    }
}

impl ConnectErrorCodeSource for ProductionBoxTransportError {
    fn connect_error_code(&self) -> Option<ConnectErrorCode> {
        match self {
            Self::Io(error) if error.kind() == std::io::ErrorKind::TimedOut => {
                Some(ConnectErrorCode::Number(4))
            }
            Self::HttpStatus { status, .. } => {
                http_status_connect_code(*status).map(ConnectErrorCode::Number)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionBoxTransport {
    options: BoxTransportOptions,
}

impl ProductionBoxTransport {
    pub fn from_endpoint(endpoint: &BoxEndpoint) -> Self {
        create_box_transport(endpoint, |options| Self { options })
    }

    pub fn options(&self) -> &BoxTransportOptions {
        &self.options
    }

    fn request_headers(&self) -> Result<BTreeMap<String, String>, ProductionBoxTransportError> {
        let mut headers = BTreeMap::new();
        for interceptor in &self.options.interceptors {
            interceptor.apply(&mut headers);
        }
        for (name, value) in &headers {
            validate_header(name, value)?;
        }
        Ok(headers)
    }
}

pub struct ProductionBoxControlClient {
    transport: ProductionBoxTransport,
}

pub fn create_production_box_control_client(
    transport: &ProductionBoxTransport,
) -> ProductionBoxControlClient {
    ProductionBoxControlClient {
        transport: transport.clone(),
    }
}

impl<Ctx> BoxPingControlClient<Ctx> for ProductionBoxControlClient {
    type Error = ProductionBoxTransportError;

    fn ping(&mut self, _ctx: &Ctx, _timeout_ms: u64) -> Result<(), Self::Error> {
        send_connect_unary(&self.transport, PING_PATH, &[])
    }
}

impl<Ctx> BoxEnvironmentControlClient<Ctx> for ProductionBoxControlClient {
    type Error = ProductionBoxTransportError;

    fn update_environment_variables(
        &mut self,
        _ctx: &Ctx,
        request: BoxEnvironmentUpdate,
    ) -> Result<(), Self::Error> {
        let body = encode_update_environment_variables_request(&request);
        send_connect_unary(
            &self.transport,
            UPDATE_ENVIRONMENT_VARIABLES_PATH,
            &body,
        )
    }
}

impl<Ctx> BoxMcpControlClient<Ctx> for ProductionBoxControlClient {
    type Error = ProductionBoxTransportError;

    fn load_mcp_servers(
        &mut self,
        _ctx: &Ctx,
        request: BoxMcpLoadRequest,
    ) -> Result<BoxMcpLoadResponse, Self::Error> {
        let body = encode_load_mcp_servers_request(&request);
        let response = send_connect_unary_response(
            &self.transport,
            LOAD_MCP_SERVERS_PATH,
            &body,
        )?;
        Ok(BoxMcpLoadResponse {
            loaded_server_names: decode_load_mcp_servers_response(&response)?,
        })
    }
}

fn validate_header(name: &str, value: &str) -> Result<(), ProductionBoxTransportError> {
    if name.is_empty()
        || name.bytes().any(|byte| byte <= b' ' || byte == b':' || byte >= 0x7f)
        || value.contains('\r')
        || value.contains('\n')
    {
        return Err(ProductionBoxTransportError::InvalidHeader(format!(
            "invalid box transport header {name:?}"
        )));
    }
    Ok(())
}

fn encode_varint(mut value: usize, out: &mut Vec<u8>) {
    while value >= 0x80 {
        out.push(((value as u8) & 0x7f) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

fn encode_len_delimited(field_number: u8, value: &[u8], out: &mut Vec<u8>) {
    out.push((field_number << 3) | 2);
    encode_varint(value.len(), out);
    out.extend_from_slice(value);
}

fn encode_string_map_entry(key: &str, value: &str) -> Vec<u8> {
    let mut entry = Vec::new();
    encode_len_delimited(1, key.as_bytes(), &mut entry);
    encode_len_delimited(2, value.as_bytes(), &mut entry);
    entry
}

pub fn encode_update_environment_variables_request(update: &BoxEnvironmentUpdate) -> Vec<u8> {
    let mut body = Vec::new();
    for (name, value) in &update.env {
        let entry = encode_string_map_entry(name, value);
        encode_len_delimited(1, &entry, &mut body);
    }
    if update.replace {
        body.extend_from_slice(&[0x10, 0x01]);
    }
    body
}

pub fn encode_load_mcp_servers_request(request: &BoxMcpLoadRequest) -> Vec<u8> {
    let mut body = Vec::new();
    if !request.mcp_config_json.is_empty() {
        encode_len_delimited(1, request.mcp_config_json.as_bytes(), &mut body);
    }
    if request.remove_missing {
        body.extend_from_slice(&[0x10, 0x01]);
    }
    body
}

fn decode_varint(input: &[u8], cursor: &mut usize) -> Result<u64, ProductionBoxTransportError> {
    let mut value = 0u64;
    let mut shift = 0u32;
    while *cursor < input.len() && shift < 64 {
        let byte = input[*cursor];
        *cursor += 1;
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(value);
        }
        shift += 7;
    }
    Err(ProductionBoxTransportError::InvalidHttpResponse(
        "box ControlService returned malformed protobuf".into(),
    ))
}

fn skip_protobuf_field(
    input: &[u8],
    cursor: &mut usize,
    wire_type: u8,
) -> Result<(), ProductionBoxTransportError> {
    match wire_type {
        0 => {
            let _ = decode_varint(input, cursor)?;
        }
        1 => {
            *cursor = cursor.saturating_add(8);
        }
        2 => {
            let length = usize::try_from(decode_varint(input, cursor)?).map_err(|_| {
                ProductionBoxTransportError::InvalidHttpResponse(
                    "box ControlService protobuf length overflow".into(),
                )
            })?;
            *cursor = cursor.saturating_add(length);
        }
        5 => {
            *cursor = cursor.saturating_add(4);
        }
        _ => {
            return Err(ProductionBoxTransportError::InvalidHttpResponse(format!(
                "box ControlService returned unsupported protobuf wire type {wire_type}"
            )));
        }
    }
    if *cursor > input.len() {
        return Err(ProductionBoxTransportError::InvalidHttpResponse(
            "box ControlService returned truncated protobuf".into(),
        ));
    }
    Ok(())
}

pub fn decode_load_mcp_servers_response(
    input: &[u8],
) -> Result<Vec<String>, ProductionBoxTransportError> {
    let mut cursor = 0usize;
    let mut names = Vec::new();
    while cursor < input.len() {
        let key = decode_varint(input, &mut cursor)?;
        let field_number = key >> 3;
        let wire_type = (key & 0x07) as u8;
        if field_number == 1 && wire_type == 2 {
            let length = usize::try_from(decode_varint(input, &mut cursor)?).map_err(|_| {
                ProductionBoxTransportError::InvalidHttpResponse(
                    "box ControlService protobuf length overflow".into(),
                )
            })?;
            let end = cursor.saturating_add(length);
            if end > input.len() {
                return Err(ProductionBoxTransportError::InvalidHttpResponse(
                    "box ControlService returned truncated MCP server name".into(),
                ));
            }
            let name = std::str::from_utf8(&input[cursor..end]).map_err(|_| {
                ProductionBoxTransportError::InvalidHttpResponse(
                    "box ControlService returned non-UTF-8 MCP server name".into(),
                )
            })?;
            names.push(name.to_string());
            cursor = end;
            continue;
        }
        skip_protobuf_field(input, &mut cursor, wire_type)?;
    }
    Ok(names)
}

fn send_connect_unary(
    transport: &ProductionBoxTransport,
    path: &str,
    body: &[u8],
) -> Result<(), ProductionBoxTransportError> {
    send_connect_unary_response(transport, path, body).map(|_| ())
}

fn send_connect_unary_response(
    transport: &ProductionBoxTransport,
    path: &str,
    body: &[u8],
) -> Result<Vec<u8>, ProductionBoxTransportError> {
    let base = Url::parse(&transport.options.base_url)
        .map_err(|error| ProductionBoxTransportError::InvalidBaseUrl(error.to_string()))?;
    if base.scheme() != "http" {
        return Err(ProductionBoxTransportError::InvalidBaseUrl(format!(
            "box transport requires http:// loopback, got {}",
            transport.options.base_url
        )));
    }
    let host = base
        .host_str()
        .ok_or_else(|| ProductionBoxTransportError::InvalidBaseUrl(
            "box transport base URL has no host".into(),
        ))?;
    let port = base
        .port_or_known_default()
        .ok_or_else(|| ProductionBoxTransportError::InvalidBaseUrl(
            "box transport base URL has no port".into(),
        ))?;

    let timeout = Duration::from_millis(PRODUCTION_BOX_RPC_TIMEOUT_MS);
    let mut stream = TcpStream::connect((host, port))?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;

    let mut request = Vec::new();
    write!(
        request,
        "POST {path} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/proto\r\nConnect-Protocol-Version: {CONNECT_PROTOCOL_VERSION}\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    )
    .map_err(ProductionBoxTransportError::Io)?;
    for (name, value) in transport.request_headers()? {
        write!(request, "{name}: {value}\r\n").map_err(ProductionBoxTransportError::Io)?;
    }
    request.extend_from_slice(b"\r\n");
    request.extend_from_slice(body);
    stream.write_all(&request)?;
    stream.flush()?;

    let mut response = Vec::new();
    stream.read_to_end(&mut response)?;
    let header_end = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| ProductionBoxTransportError::InvalidHttpResponse(
            "box ControlService returned an incomplete HTTP response".into(),
        ))?;
    let header_text = std::str::from_utf8(&response[..header_end])
        .map_err(|_| ProductionBoxTransportError::InvalidHttpResponse(
            "box ControlService returned non-UTF-8 HTTP headers".into(),
        ))?;
    let status_line = header_text.lines().next().unwrap_or_default();
    let status = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or_else(|| ProductionBoxTransportError::InvalidHttpResponse(format!(
            "box ControlService returned an invalid status line: {status_line}"
        )))?;
    if (200..300).contains(&status) {
        return Ok(response[header_end + 4..].to_vec());
    }
    let body_text = String::from_utf8_lossy(&response[header_end + 4..]);
    let body_text = body_text.chars().take(512).collect::<String>();
    Err(ProductionBoxTransportError::HttpStatus {
        status,
        body: body_text,
    })
}
