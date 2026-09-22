use std::collections::BTreeMap;
use std::fmt;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use url::Url;

use super::box_env::{BoxEnvironmentControlClient, BoxEnvironmentUpdate};
use super::box_remote_accessor::{
    BoxEndpoint, BoxTransportOptions, create_box_transport,
};

pub const UPDATE_ENVIRONMENT_VARIABLES_PATH: &str =
    "/agent.v1.ControlService/UpdateEnvironmentVariables";
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

pub struct ProductionBoxControlClient<'a> {
    transport: &'a ProductionBoxTransport,
}

pub fn create_production_box_control_client(
    transport: &ProductionBoxTransport,
) -> ProductionBoxControlClient<'_> {
    ProductionBoxControlClient { transport }
}

impl<Ctx> BoxEnvironmentControlClient<Ctx> for ProductionBoxControlClient<'_> {
    type Error = ProductionBoxTransportError;

    fn update_environment_variables(
        &mut self,
        _ctx: &Ctx,
        request: BoxEnvironmentUpdate,
    ) -> Result<(), Self::Error> {
        let body = encode_update_environment_variables_request(&request);
        send_connect_unary(
            self.transport,
            UPDATE_ENVIRONMENT_VARIABLES_PATH,
            &body,
        )
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

fn send_connect_unary(
    transport: &ProductionBoxTransport,
    path: &str,
    body: &[u8],
) -> Result<(), ProductionBoxTransportError> {
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
        return Ok(());
    }
    let body_text = String::from_utf8_lossy(&response[header_end + 4..]);
    let body_text = body_text.chars().take(512).collect::<String>();
    Err(ProductionBoxTransportError::HttpStatus {
        status,
        body: body_text,
    })
}
