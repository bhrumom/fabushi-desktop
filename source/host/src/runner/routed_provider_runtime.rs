use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use serde_json::{Value, json};
use uuid::Uuid;

use crate::extensions::inference::provider_session::{
    ProviderMessage, ProviderSessionError, RoutedProvider, RoutedProviderOptions,
    RoutedToolDefinition, run_routed_provider_text,
};
use crate::host_request_context::HostRequestContext;
use crate::runner::system_prompt_assembly::render_request_context_system_prompt;

pub const ROUTED_MCP_PROTOCOL_VERSION: &str = "2025-03-26";
pub const ROUTED_MCP_MAX_BODY_BYTES: usize = 1_048_576;

#[derive(Clone, Default)]
pub struct RoutedProviderCancellation {
    cancelled: Arc<AtomicBool>,
    reason: Arc<Mutex<Option<String>>>,
}

impl RoutedProviderCancellation {
    pub fn cancel(&self, reason: impl Into<String>) -> bool {
        let first = !self.cancelled.swap(true, Ordering::AcqRel);
        if first {
            if let Ok(mut slot) = self.reason.lock() {
                *slot = Some(reason.into());
            }
        }
        first
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    pub fn reason(&self) -> Option<String> {
        self.reason.lock().ok().and_then(|reason| reason.clone())
    }

    pub fn check(&self) -> Result<(), ProviderSessionError> {
        if self.is_cancelled() {
            Err(ProviderSessionError::Transport(
                self.reason().unwrap_or_else(|| "Runner provider request cancelled".into()),
            ))
        } else {
            Ok(())
        }
    }
}

#[derive(Default)]
pub struct RoutedProviderTaskRegistry {
    active: Mutex<HashMap<String, RoutedProviderCancellation>>,
}

impl RoutedProviderTaskRegistry {
    pub fn register(
        &self,
        stream_id: &str,
    ) -> Result<RoutedProviderCancellation, ProviderSessionError> {
        let mut active = self
            .active
            .lock()
            .map_err(|_| ProviderSessionError::Protocol(
                "Runner provider registry lock poisoned".into(),
            ))?;
        if active.contains_key(stream_id) {
            return Err(ProviderSessionError::Protocol(format!(
                "Runner provider stream already exists: {stream_id}"
            )));
        }
        let cancellation = RoutedProviderCancellation::default();
        active.insert(stream_id.to_string(), cancellation.clone());
        Ok(cancellation)
    }

    pub fn cancel(&self, stream_id: &str, reason: impl Into<String>) -> bool {
        self.active
            .lock()
            .ok()
            .and_then(|active| active.get(stream_id).cloned())
            .is_some_and(|cancellation| cancellation.cancel(reason))
    }

    pub fn finish(&self, stream_id: &str) {
        if let Ok(mut active) = self.active.lock() {
            active.remove(stream_id);
        }
    }

    pub fn cancel_all(&self, reason: &str) {
        let cancellations = self
            .active
            .lock()
            .map(|active| active.values().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        for cancellation in cancellations {
            cancellation.cancel(reason.to_string());
        }
    }

    pub fn active_count(&self) -> usize {
        self.active.lock().map(|active| active.len()).unwrap_or_default()
    }
}

pub trait RoutedToolBridge: Send + Sync {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError>;
    fn call_tool(
        &self,
        tool: &RoutedToolDefinition,
        args: Value,
        tool_call_id: &str,
    ) -> Result<Value, ProviderSessionError>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunnerRequestContextSnapshot {
    pub context: HostRequestContext,
    pub rules: Option<Vec<Value>>,
}

pub trait RunnerRequestContextSource: Send + Sync {
    fn resolve(&self) -> RunnerRequestContextSnapshot;
}

pub struct RoutedProviderRun<'a> {
    pub provider: RoutedProvider,
    pub data_dir: &'a Path,
    pub messages: &'a [ProviderMessage],
    pub bridge: Arc<dyn RoutedToolBridge>,
    pub request_context: RunnerRequestContextSnapshot,
    pub cancellation: RoutedProviderCancellation,
}

pub fn run_routed_provider_in_runner(
    run: RoutedProviderRun<'_>,
    on_text_delta: &mut dyn FnMut(&str, &str),
) -> Result<String, ProviderSessionError> {
    run.cancellation.check()?;
    if run.provider == RoutedProvider::Cursor {
        return Err(ProviderSessionError::Configuration(
            "Cursor inference is owned by the Host gateway and cannot enter the local Runner provider path."
                .into(),
        ));
    }

    let system_prompt = render_request_context_system_prompt(
        &run.request_context.context,
        run.request_context.rules.as_deref(),
    );
    let mut provider_messages = Vec::with_capacity(run.messages.len() + usize::from(!system_prompt.is_empty()));
    if !system_prompt.is_empty() {
        provider_messages.push(ProviderMessage {
            role: "system".into(),
            content: system_prompt,
        });
    }
    provider_messages.extend_from_slice(run.messages);

    let direct_tools = if run.provider == RoutedProvider::ClaudeCode {
        Vec::new()
    } else {
        run.cancellation.check()?;
        run.bridge.list_tools()?
    };
    let mut mcp_server = if run.provider == RoutedProvider::ClaudeCode {
        Some(start_routed_mcp_server_with_cancellation(
            Arc::clone(&run.bridge),
            run.cancellation.clone(),
        )?)
    } else {
        None
    };
    let mcp_url = mcp_server.as_ref().map(|server| server.url().to_string());
    let bridge = Arc::clone(&run.bridge);
    let tool_cancellation = run.cancellation.clone();
    let mut execute_tool = move |
        tool: &RoutedToolDefinition,
        args: Value,
        tool_call_id: &str,
    | {
        tool_cancellation.check()?;
        bridge.call_tool(tool, args, tool_call_id)
    };
    let delta_cancellation = run.cancellation.clone();
    let mut guarded_delta = |delta: &str, accumulated: &str| {
        if !delta_cancellation.is_cancelled() {
            on_text_delta(delta, accumulated);
        }
    };

    let result = run_routed_provider_text(
        run.provider,
        &provider_messages,
        &mut RoutedProviderOptions {
            data_dir: run.data_dir,
            tools: &direct_tools,
            mcp_server_url: mcp_url.as_deref(),
            execute_tool: &mut execute_tool,
            on_text_delta: &mut guarded_delta,
        },
    );

    if let Some(server) = mcp_server.as_mut() {
        server.close();
    }
    run.cancellation.check()?;
    result
}

pub struct RoutedMcpServer {
    url: String,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl RoutedMcpServer {
    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn close(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = TcpStream::connect(
                self.url
                    .strip_prefix("http://")
                    .and_then(|value| value.split('/').next())
                    .unwrap_or("127.0.0.1:0"),
            );
            let _ = worker.join();
        }
    }
}

impl Drop for RoutedMcpServer {
    fn drop(&mut self) {
        self.close();
    }
}

pub fn start_routed_mcp_server(
    bridge: Arc<dyn RoutedToolBridge>,
) -> Result<RoutedMcpServer, ProviderSessionError> {
    start_routed_mcp_server_with_cancellation(
        bridge,
        RoutedProviderCancellation::default(),
    )
}

pub fn start_routed_mcp_server_with_cancellation(
    bridge: Arc<dyn RoutedToolBridge>,
    cancellation: RoutedProviderCancellation,
) -> Result<RoutedMcpServer, ProviderSessionError> {
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|error| {
        ProviderSessionError::Transport(format!("could not bind routed MCP server: {error}"))
    })?;
    listener.set_nonblocking(true).map_err(|error| {
        ProviderSessionError::Transport(format!("could not configure routed MCP server: {error}"))
    })?;
    let address = listener
        .local_addr()
        .map_err(|error| ProviderSessionError::Transport(error.to_string()))?;
    let secret = Uuid::new_v4().to_string();
    let path = format!("/mcp/{secret}");
    let url = format!("http://{}{}", address, path);
    let stop = Arc::new(AtomicBool::new(false));
    let worker_stop = Arc::clone(&stop);
    let worker_cancellation = cancellation.clone();
    let worker = thread::Builder::new()
        .name("mahayana-runner-routed-mcp".into())
        .spawn(move || {
            let mut tools = HashMap::<String, RoutedToolDefinition>::new();
            while !worker_stop.load(Ordering::Acquire) && !worker_cancellation.is_cancelled() {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let _ = serve_request(
                            stream,
                            &path,
                            &mut tools,
                            bridge.as_ref(),
                            &worker_cancellation,
                        );
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => thread::sleep(Duration::from_millis(10)),
                }
            }
        })
        .map_err(|error| ProviderSessionError::Transport(format!(
            "could not start routed MCP server: {error}"
        )))?;

    Ok(RoutedMcpServer {
        url,
        stop,
        worker: Some(worker),
    })
}

fn serve_request(
    mut stream: TcpStream,
    expected_path: &str,
    tools: &mut HashMap<String, RoutedToolDefinition>,
    bridge: &dyn RoutedToolBridge,
    cancellation: &RoutedProviderCancellation,
) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut request = Vec::with_capacity(4096);
    let mut chunk = [0_u8; 4096];
    let header_end = loop {
        let count = stream.read(&mut chunk)?;
        if count == 0 {
            return Ok(());
        }
        request.extend_from_slice(&chunk[..count]);
        if let Some(index) = request.windows(4).position(|window| window == b"\r\n\r\n") {
            break index + 4;
        }
        if request.len() > 64 * 1024 {
            return write_response(&mut stream, 431, None);
        }
    };
    let headers = std::str::from_utf8(&request[..header_end])
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "headers are not UTF-8"))?;
    let mut lines = headers.lines();
    let mut request_line = lines.next().unwrap_or("").split_whitespace();
    let method = request_line.next().unwrap_or("");
    let path = request_line.next().unwrap_or("");
    if method != "POST" || path != expected_path {
        return write_response(&mut stream, 404, None);
    }
    let content_length = lines
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().ok())
                .flatten()
        })
        .unwrap_or(0);
    if content_length > ROUTED_MCP_MAX_BODY_BYTES {
        return write_response(&mut stream, 413, None);
    }
    let expected = header_end.saturating_add(content_length);
    while request.len() < expected {
        let count = stream.read(&mut chunk)?;
        if count == 0 {
            break;
        }
        request.extend_from_slice(&chunk[..count]);
    }
    if request.len() < expected {
        return write_response(&mut stream, 400, None);
    }
    let message = match serde_json::from_slice::<Value>(&request[header_end..expected]) {
        Ok(value) if value.is_object() => value,
        _ => return write_response(&mut stream, 400, None),
    };

    let rpc_method = message.get("method").and_then(Value::as_str).unwrap_or("");
    if cancellation.is_cancelled() {
        let id = message.get("id").cloned().unwrap_or(Value::Null);
        return write_json(
            &mut stream,
            json!({
                "jsonrpc":"2.0",
                "id":id,
                "result":tool_failure(
                    cancellation.reason().unwrap_or_else(|| "Runner request cancelled".into())
                )
            }),
        );
    }
    if rpc_method == "notifications/initialized" {
        return write_response(&mut stream, 202, None);
    }
    let id = message.get("id").cloned().unwrap_or(Value::Null);
    let result = match rpc_method {
        "initialize" => json!({
            "protocolVersion": ROUTED_MCP_PROTOCOL_VERSION,
            "capabilities": { "tools": { "listChanged": false } },
            "serverInfo": { "name": "fabushi-runner-plugins", "version": "1" }
        }),
        "tools/list" => match bridge.list_tools() {
            Ok(discovered) => {
                *tools = discovered.into_iter().map(|tool| (tool.name.clone(), tool)).collect();
                json!({
                    "tools": tools.values().map(|tool| json!({
                        "name": tool.name,
                        "description": tool.description.clone().unwrap_or_else(|| format!(
                            "{} via {}", tool.tool_name, tool.provider_identifier
                        )),
                        "inputSchema": tool.input_schema
                    })).collect::<Vec<_>>()
                })
            }
            Err(error) => tool_failure(error.to_string()),
        },
        "tools/call" => {
            let params = message.get("params").and_then(Value::as_object);
            let name = params.and_then(|value| value.get("name")).and_then(Value::as_str).unwrap_or("");
            let args = params.and_then(|value| value.get("arguments")).cloned().unwrap_or_else(|| json!({}));
            match tools.get(name) {
                Some(tool) => match bridge.call_tool(tool, args, &Uuid::new_v4().to_string()) {
                    Ok(value) => normalize_tool_result(value),
                    Err(error) => tool_failure(error.to_string()),
                },
                None => tool_failure(format!("Unknown Fabushi plugin tool: {name}")),
            }
        }
        _ => json!({}),
    };
    write_json(&mut stream, json!({"jsonrpc":"2.0","id":id,"result":result}))
}

fn normalize_tool_result(value: Value) -> Value {
    if let Some(root) = value.as_object() {
        if root.contains_key("content") || root.contains_key("structuredContent") {
            let mut normalized = root.clone();
            normalized.entry("isError").or_insert(Value::Bool(false));
            return Value::Object(normalized);
        }
    }
    json!({
        "isError": false,
        "content": [{"type":"text","text":value.to_string()}]
    })
}

fn tool_failure(message: impl Into<String>) -> Value {
    json!({
        "isError": true,
        "content": [{"type":"text","text":message.into()}]
    })
}

fn write_json(stream: &mut TcpStream, value: Value) -> std::io::Result<()> {
    let body = serde_json::to_vec(&value)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
    write_response(stream, 200, Some(body))
}

fn write_response(
    stream: &mut TcpStream,
    status: u16,
    body: Option<Vec<u8>>,
) -> std::io::Result<()> {
    let body = body.unwrap_or_default();
    let reason = match status {
        200 => "OK",
        202 => "Accepted",
        400 => "Bad Request",
        404 => "Not Found",
        413 => "Payload Too Large",
        431 => "Request Header Fields Too Large",
        _ => "Response",
    };
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    )?;
    if !body.is_empty() {
        write!(stream, "Content-Type: application/json\r\n")?;
    }
    write!(stream, "\r\n")?;
    stream.write_all(&body)?;
    stream.flush()
}
