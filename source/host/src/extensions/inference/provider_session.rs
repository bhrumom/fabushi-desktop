use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::Command;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use reqwest::blocking::{Client, Response};
use serde_json::{Map, Value, json};
use thiserror::Error;
use uuid::Uuid;

use crate::host_paths::get_sand_root_dir;

use super::codex_direct_responses::{
    CodexDirectError, CodexDirectOptions, CodexDirectTool, CodexDirectTransport,
    run_codex_direct_responses,
};

pub const GROK_ROUTER_SYSTEM_PROMPT: &str =
    "You are Grok Bot, a warm, concise desktop assistant.\n\
You are running inside Grok Bot, not inside Codex CLI or Claude Code.\n\
The tools supplied with this request are Grok Bot's already-connected plugins and accounts. Use them whenever they are relevant instead of claiming that a plugin is unavailable or asking the user to reconnect it.\n\
Never ask for an API key for an already-connected plugin. Respond directly to the user in natural language after completing any necessary tool calls.";


fn assembled_provider_system_prompt(messages: &[ProviderMessage]) -> String {
    let additions = messages
        .iter()
        .filter(|message| message.role == "system")
        .map(|message| message.content.trim())
        .filter(|content| !content.is_empty())
        .collect::<Vec<_>>();
    if additions.is_empty() {
        GROK_ROUTER_SYSTEM_PROMPT.to_string()
    } else {
        format!("{GROK_ROUTER_SYSTEM_PROMPT}\n\n{}", additions.join("\n\n"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutedProvider {
    Cursor,
    Codex,
    ClaudeCode,
    OpenRouter,
}

impl RoutedProvider {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "cursor" => Some(Self::Cursor),
            "codex" => Some(Self::Codex),
            "claude-code" => Some(Self::ClaudeCode),
            "openrouter" => Some(Self::OpenRouter),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cursor => "cursor",
            Self::Codex => "codex",
            Self::ClaudeCode => "claude-code",
            Self::OpenRouter => "openrouter",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoutedToolDefinition {
    pub name: String,
    pub provider_identifier: String,
    pub tool_name: String,
    pub description: Option<String>,
    pub input_schema: Value,
}

#[derive(Debug, Error)]
pub enum ProviderSessionError {
    #[error("{0}")]
    Configuration(String),
    #[error("{0}")]
    Authentication(String),
    #[error("{0}")]
    Transport(String),
    #[error("{0}")]
    Protocol(String),
    #[error("{0}")]
    Tool(String),
}

impl From<CodexDirectError> for ProviderSessionError {
    fn from(error: CodexDirectError) -> Self {
        match error {
            CodexDirectError::Transport(message) => Self::Transport(message),
            CodexDirectError::Protocol(message) => Self::Protocol(message),
            CodexDirectError::Tool(message) => Self::Tool(message),
        }
    }
}

#[derive(Debug, Clone)]
struct CodexCredentials {
    access_token: String,
    refresh_token: String,
    id_token: String,
    account_id: String,
    path: PathBuf,
    document: Value,
}

fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn codex_home() -> PathBuf {
    env::var_os("CODEX_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".codex"))
}

fn configured_codex_model() -> String {
    env::var("SAND_CODEX_MODEL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            fs::read_to_string(codex_home().join("config.toml"))
                .ok()
                .and_then(|config| toml_string_setting(&config, "model"))
        })
        .unwrap_or_else(|| "gpt-5.4".into())
}

fn configured_codex_reasoning_effort() -> Option<String> {
    env::var("SAND_CODEX_REASONING_EFFORT")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| {
            matches!(
                value.as_str(),
                "minimal" | "low" | "medium" | "high" | "xhigh"
            )
        })
        .or_else(|| {
            fs::read_to_string(codex_home().join("config.toml"))
                .ok()
                .and_then(|config| toml_string_setting(&config, "model_reasoning_effort"))
                .filter(|value| {
                    matches!(
                        value.as_str(),
                        "minimal" | "low" | "medium" | "high" | "xhigh"
                    )
                })
        })
}

fn toml_string_setting(config: &str, key: &str) -> Option<String> {
    config.lines().find_map(|line| {
        let line = line.trim();
        if line.starts_with('#') {
            return None;
        }
        let (name, value) = line.split_once('=')?;
        if name.trim() != key {
            return None;
        }
        let value = value.trim();
        if value.len() < 2 {
            return None;
        }
        let quote = value.as_bytes()[0];
        if (quote != b'"' && quote != b'\'')
            || value.as_bytes()[value.len() - 1] != quote
        {
            return None;
        }
        let value = value[1..value.len() - 1].trim();
        (!value.is_empty()).then(|| value.to_string())
    })
}

pub fn configured_routed_provider(settings_path: &Path) -> Option<RoutedProvider> {
    let value: Value =
        serde_json::from_str(&fs::read_to_string(settings_path).ok()?).ok()?;
    [
        value.get("inferenceProvider"),
        value.get("inference_provider"),
        value.get("router").and_then(|router| router.get("provider")),
    ]
    .into_iter()
    .flatten()
    .find_map(|value| value.as_str().and_then(RoutedProvider::parse))
}

fn required_token(
    tokens: Option<&Map<String, Value>>,
    name: &str,
) -> Option<String> {
    tokens?
        .get(name)?
        .as_str()
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn read_codex_credentials(path: &Path) -> Result<CodexCredentials, ProviderSessionError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| {
        ProviderSessionError::Authentication(
            "Codex is not signed in with ChatGPT. Run `codex login`, then reopen Fabushi."
                .into(),
        )
    })?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(ProviderSessionError::Authentication(
            "Codex login credentials must be a private direct regular file.".into(),
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(ProviderSessionError::Authentication(
                "Codex login credentials must be a private direct regular file.".into(),
            ));
        }
    }

    let document: Value =
        serde_json::from_str(&fs::read_to_string(path).map_err(|error| {
            ProviderSessionError::Authentication(format!(
                "Could not read Codex login: {error}"
            ))
        })?)
        .map_err(|error| {
            ProviderSessionError::Authentication(format!(
                "Codex login JSON is invalid: {error}"
            ))
        })?;
    if document.get("auth_mode").and_then(Value::as_str) != Some("chatgpt") {
        return Err(ProviderSessionError::Authentication(
            "Codex is not signed in with ChatGPT. Run `codex login`, then reopen Fabushi."
                .into(),
        ));
    }
    let tokens = document.get("tokens").and_then(Value::as_object);
    let Some(access_token) = required_token(tokens, "access_token") else {
        return Err(ProviderSessionError::Authentication(
            "Codex login is missing an access token. Run `codex login` again.".into(),
        ));
    };
    let Some(refresh_token) = required_token(tokens, "refresh_token") else {
        return Err(ProviderSessionError::Authentication(
            "Codex login is missing a refresh token. Run `codex login` again.".into(),
        ));
    };
    let Some(id_token) = required_token(tokens, "id_token") else {
        return Err(ProviderSessionError::Authentication(
            "Codex login is missing an ID token. Run `codex login` again.".into(),
        ));
    };
    let Some(account_id) = required_token(tokens, "account_id") else {
        return Err(ProviderSessionError::Authentication(
            "Codex login is missing an account ID. Run `codex login` again.".into(),
        ));
    };
    Ok(CodexCredentials {
        access_token,
        refresh_token,
        id_token,
        account_id,
        path: path.to_path_buf(),
        document,
    })
}

fn jwt_audience(token: &str) -> Option<String> {
    let payload = token.split('.').nth(1)?;
    let decoded = URL_SAFE_NO_PAD.decode(payload.as_bytes()).ok()?;
    let value: Value = serde_json::from_slice(&decoded).ok()?;
    match value.get("aud")? {
        Value::String(value) => Some(value.clone()),
        Value::Array(values) => values
            .iter()
            .find_map(|value| value.as_str().map(str::to_string)),
        _ => None,
    }
}

fn form_encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn persist_refreshed_credentials(
    current: &CodexCredentials,
    access_token: String,
    refresh_token: String,
    id_token: String,
) -> Result<CodexCredentials, ProviderSessionError> {
    let mut document = current.document.clone();
    let root = document.as_object_mut().ok_or_else(|| {
        ProviderSessionError::Protocol("Codex auth document was not an object.".into())
    })?;
    let tokens = root
        .entry("tokens")
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .ok_or_else(|| {
            ProviderSessionError::Protocol("Codex auth tokens were not an object.".into())
        })?;
    tokens.insert(
        "access_token".into(),
        Value::String(access_token.clone()),
    );
    tokens.insert(
        "refresh_token".into(),
        Value::String(refresh_token.clone()),
    );
    tokens.insert("id_token".into(), Value::String(id_token.clone()));

    let temporary = PathBuf::from(format!(
        "{}.{}.tmp",
        current.path.to_string_lossy(),
        Uuid::new_v4()
    ));
    fs::write(
        &temporary,
        serde_json::to_vec_pretty(&document)
            .map_err(|error| ProviderSessionError::Protocol(error.to_string()))?,
    )
    .map_err(|error| {
        ProviderSessionError::Authentication(format!(
            "Could not persist refreshed Codex login: {error}"
        ))
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&temporary, fs::Permissions::from_mode(0o600)).map_err(
            |error| {
                ProviderSessionError::Authentication(format!(
                    "Could not secure refreshed Codex login: {error}"
                ))
            },
        )?;
    }
    fs::rename(&temporary, &current.path).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        ProviderSessionError::Authentication(format!(
            "Could not replace refreshed Codex login: {error}"
        ))
    })?;

    Ok(CodexCredentials {
        access_token,
        refresh_token,
        id_token,
        account_id: current.account_id.clone(),
        path: current.path.clone(),
        document,
    })
}

fn refresh_codex_credentials(
    client: &Client,
    current: &CodexCredentials,
) -> Result<CodexCredentials, ProviderSessionError> {
    let client_id = jwt_audience(&current.id_token).ok_or_else(|| {
        ProviderSessionError::Authentication(
            "Codex login expired and its refresh identity is invalid. Run `codex login` again."
                .into(),
        )
    })?;
    let body = format!(
        "grant_type=refresh_token&refresh_token={}&client_id={}",
        form_encode(&current.refresh_token),
        form_encode(&client_id),
    );
    let response = client
        .post("https://auth.openai.com/oauth/token")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .map_err(|error| {
            ProviderSessionError::Transport(format!(
                "Could not refresh Codex login: {error}"
            ))
        })?;
    if !response.status().is_success() {
        return Err(ProviderSessionError::Authentication(
            "Codex login expired and could not be refreshed. Run `codex login` again."
                .into(),
        ));
    }
    let payload: Value = response.json().map_err(|error| {
        ProviderSessionError::Protocol(format!(
            "Codex refresh response was invalid: {error}"
        ))
    })?;
    let access_token = payload
        .get("access_token")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            ProviderSessionError::Authentication(
                "Codex returned an invalid refreshed login. Run `codex login` again."
                    .into(),
            )
        })?
        .to_string();
    let refresh_token = payload
        .get("refresh_token")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or(&current.refresh_token)
        .to_string();
    let id_token = payload
        .get("id_token")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or(&current.id_token)
        .to_string();
    persist_refreshed_credentials(
        current,
        access_token,
        refresh_token,
        id_token,
    )
}

pub fn decode_sse_stream<R, F>(
    reader: R,
    mut on_event: F,
) -> Result<(), ProviderSessionError>
where
    R: Read,
    F: FnMut(Value) -> Result<(), ProviderSessionError>,
{
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    let mut data = Vec::<String>::new();

    let flush = |data: &mut Vec<String>,
                 on_event: &mut F|
     -> Result<(), ProviderSessionError> {
        if data.is_empty() {
            return Ok(());
        }
        let payload = data.join("\n");
        data.clear();
        if payload == "[DONE]" {
            return Ok(());
        }
        let value: Value = serde_json::from_str(&payload).map_err(|_| {
            ProviderSessionError::Protocol(
                "Provider stream contained malformed SSE JSON.".into(),
            )
        })?;
        on_event(value)
    };

    loop {
        line.clear();
        let count = reader
            .read_line(&mut line)
            .map_err(|error| ProviderSessionError::Transport(error.to_string()))?;
        if count == 0 {
            break;
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            flush(&mut data, &mut on_event)?;
        } else if let Some(payload) = trimmed.strip_prefix("data:") {
            data.push(payload.trim_start().to_string());
        }
    }
    flush(&mut data, &mut on_event)
}

struct CodexHttpTransport {
    client: Client,
    credentials: CodexCredentials,
}

impl CodexHttpTransport {
    fn new(auth_path: &Path) -> Result<Self, ProviderSessionError> {
        let client = Client::builder()
            .build()
            .map_err(|error| ProviderSessionError::Transport(error.to_string()))?;
        Ok(Self {
            client,
            credentials: read_codex_credentials(auth_path)?,
        })
    }

    fn send(&self, request: &Value) -> Result<Response, ProviderSessionError> {
        self.client
            .post("https://chatgpt.com/backend-api/codex/responses")
            .header(
                "authorization",
                format!("Bearer {}", self.credentials.access_token),
            )
            .header(
                "ChatGPT-Account-Id",
                self.credentials.account_id.clone(),
            )
            .header("content-type", "application/json")
            .header("accept", "text/event-stream")
            .header("user-agent", "fabushi-router/1")
            .json(request)
            .send()
            .map_err(|error| ProviderSessionError::Transport(error.to_string()))
    }
}

impl CodexDirectTransport for CodexHttpTransport {
    fn stream_response(
        &mut self,
        request: &Value,
        on_event: &mut dyn FnMut(Value) -> Result<(), CodexDirectError>,
    ) -> Result<(), CodexDirectError> {
        let mut response = self
            .send(request)
            .map_err(|error| CodexDirectError::Transport(error.to_string()))?;
        if response.status().as_u16() == 401 {
            self.credentials =
                refresh_codex_credentials(&self.client, &self.credentials)
                    .map_err(|error| {
                        CodexDirectError::Transport(error.to_string())
                    })?;
            response = self
                .send(request)
                .map_err(|error| CodexDirectError::Transport(error.to_string()))?;
        }
        if !response.status().is_success() {
            let status = response.status();
            let mut detail = String::new();
            let _ = response.take(4096).read_to_string(&mut detail);
            return Err(CodexDirectError::Transport(format!(
                "Codex direct request failed ({status}{}).",
                if detail.trim().is_empty() {
                    String::new()
                } else {
                    format!(": {}", detail.trim())
                }
            )));
        }
        decode_sse_stream(response, |event| {
            on_event(event).map_err(ProviderSessionError::from)
        })
        .map_err(|error| CodexDirectError::Protocol(error.to_string()))
    }
}

fn to_codex_tool(tool: &RoutedToolDefinition) -> CodexDirectTool {
    CodexDirectTool {
        name: tool.name.clone(),
        description: tool.description.clone(),
        parameters: tool.input_schema.clone(),
        source: json!({
            "name": tool.name,
            "providerIdentifier": tool.provider_identifier,
            "toolName": tool.tool_name,
            "description": tool.description,
            "inputSchema": tool.input_schema,
        }),
    }
}

pub fn run_codex_provider_text(
    messages: &[ProviderMessage],
    tools: &[RoutedToolDefinition],
    execute_tool: &mut dyn FnMut(
        &RoutedToolDefinition,
        Value,
        &str,
    ) -> Result<Value, ProviderSessionError>,
    on_text_delta: &mut dyn FnMut(&str, &str),
) -> Result<String, ProviderSessionError> {
    let mut transport = CodexHttpTransport::new(&codex_home().join("auth.json"))?;
    let system_prompt = assembled_provider_system_prompt(messages);
    let mut request = CodexDirectOptions::new(
        configured_codex_model(),
        system_prompt,
        messages
            .iter()
            .filter(|message| message.role != "system")
            .map(|message| {
                json!({
                    "role": if message.role == "assistant" { "assistant" } else { "user" },
                    "content": message.content,
                })
            })
            .collect(),
    );
    request.reasoning_effort = configured_codex_reasoning_effort();
    request.tools = tools.iter().map(to_codex_tool).collect();

    let tool_index = tools
        .iter()
        .map(|tool| (tool.name.clone(), tool))
        .collect::<BTreeMap<_, _>>();
    let result = run_codex_direct_responses(
        &mut transport,
        &request,
        &mut |tool, args, tool_call_id| {
            let selected = tool_index.get(&tool.name).copied().ok_or_else(|| {
                CodexDirectError::Tool(format!(
                    "Unknown Fabushi tool: {}",
                    tool.name
                ))
            })?;
            execute_tool(selected, args, tool_call_id)
                .map_err(|error| CodexDirectError::Tool(error.to_string()))
        },
        on_text_delta,
    )?;
    Ok(result.text)
}


pub struct RoutedProviderOptions<'a> {
    pub data_dir: &'a Path,
    pub tools: &'a [RoutedToolDefinition],
    pub mcp_server_url: Option<&'a str>,
    pub execute_tool: &'a mut dyn FnMut(
        &RoutedToolDefinition,
        Value,
        &str,
    ) -> Result<Value, ProviderSessionError>,
    pub on_text_delta: &'a mut dyn FnMut(&str, &str),
}

pub fn run_routed_provider_text(
    provider: RoutedProvider,
    messages: &[ProviderMessage],
    options: &mut RoutedProviderOptions<'_>,
) -> Result<String, ProviderSessionError> {
    match provider {
        RoutedProvider::Cursor => Err(ProviderSessionError::Configuration(
            "Cursor inference is owned by the Host/Gateway path and must not enter the local provider router."
                .into(),
        )),
        RoutedProvider::Codex => run_codex_provider_text(
            messages,
            options.tools,
            options.execute_tool,
            options.on_text_delta,
        ),
        RoutedProvider::OpenRouter => run_openrouter_provider_text(messages, options),
        RoutedProvider::ClaudeCode => run_claude_code_provider_text(messages, options),
    }
}

fn provider_prompt(messages: &[ProviderMessage]) -> String {
    let system_prompt = assembled_provider_system_prompt(messages);
    let rendered = messages
        .iter()
        .filter(|message| message.role != "system")
        .map(|message| {
            format!(
                "{}: {}",
                message.role.to_ascii_uppercase(),
                message.content
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    format!(
        "{system_prompt}\n\nContinue this Grok Bot conversation.\n\n{rendered}"
    )
}

fn openrouter_api_key(data_dir: &Path) -> Result<String, ProviderSessionError> {
    if let Some(value) = env::var("OPENROUTER_API_KEY")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return Ok(value);
    }
    for path in [
        data_dir.join("box-secrets.json"),
        get_sand_root_dir().join("box-secrets.json"),
    ] {
        let Ok(raw) = fs::read_to_string(path) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<Value>(&raw) else {
            continue;
        };
        if let Some(key) = value
            .get("secrets")
            .and_then(Value::as_object)
            .and_then(|secrets| secrets.get("OPENROUTER_API_KEY"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return Ok(key.to_string());
        }
    }
    Err(ProviderSessionError::Authentication(
        "OpenRouter needs OPENROUTER_API_KEY. Add it in Settings -> Router.".into(),
    ))
}

fn openrouter_tools(tools: &[RoutedToolDefinition]) -> Vec<Value> {
    tools
        .iter()
        .map(|tool| {
            json!({
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description.clone().unwrap_or_else(|| {
                        format!("{} via {}", tool.tool_name, tool.provider_identifier)
                    }),
                    "parameters": tool.input_schema,
                }
            })
        })
        .collect()
}

#[derive(Default)]
struct PartialOpenRouterToolCall {
    id: String,
    name: String,
    arguments: String,
}

fn run_openrouter_provider_text(
    messages: &[ProviderMessage],
    options: &mut RoutedProviderOptions<'_>,
) -> Result<String, ProviderSessionError> {
    let api_key = openrouter_api_key(options.data_dir)?;
    let client = Client::builder()
        .build()
        .map_err(|error| ProviderSessionError::Transport(error.to_string()))?;
    let model = env::var("SAND_OPENROUTER_MODEL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "openai/gpt-5.2".into());
    let tool_index = options
        .tools
        .iter()
        .map(|tool| (tool.name.clone(), tool))
        .collect::<BTreeMap<_, _>>();
    let declared_tools = openrouter_tools(options.tools);
    let system_prompt = assembled_provider_system_prompt(messages);
    let mut conversation = vec![json!({
        "role": "system",
        "content": system_prompt
    })];
    conversation.extend(messages.iter().filter(|message| message.role != "system").map(|message| {
        json!({
            "role": if message.role == "assistant" { "assistant" } else { "user" },
            "content": message.content,
        })
    }));
    let mut text = String::new();

    for _step in 0..8 {
        let mut request = json!({
            "model": model,
            "messages": conversation,
            "stream": true,
        });
        if !declared_tools.is_empty() {
            request["tools"] = Value::Array(declared_tools.clone());
            request["tool_choice"] = Value::String("auto".into());
        }
        let response = client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("authorization", format!("Bearer {api_key}"))
            .header("content-type", "application/json")
            .header("accept", "text/event-stream")
            .header(
                "HTTP-Referer",
                "https://github.com/bhrumom/fabushi-desktop",
            )
            .header("X-Title", "Fabushi")
            .json(&request)
            .send()
            .map_err(|error| ProviderSessionError::Transport(error.to_string()))?;
        if !response.status().is_success() {
            let status = response.status();
            let detail = response
                .text()
                .unwrap_or_default()
                .chars()
                .take(4096)
                .collect::<String>();
            return Err(ProviderSessionError::Transport(format!(
                "OpenRouter request failed ({status}{}).",
                if detail.trim().is_empty() {
                    String::new()
                } else {
                    format!(": {}", detail.trim())
                }
            )));
        }

        let mut partial_calls = BTreeMap::<usize, PartialOpenRouterToolCall>::new();
        let mut step_text = String::new();
        decode_sse_stream(response, |event| {
            if let Some(error) = event.get("error") {
                return Err(ProviderSessionError::Protocol(format!(
                    "OpenRouter stream failed: {error}"
                )));
            }
            let Some(delta) = event
                .get("choices")
                .and_then(Value::as_array)
                .and_then(|choices| choices.first())
                .and_then(|choice| choice.get("delta"))
                .and_then(Value::as_object)
            else {
                return Ok(());
            };
            if let Some(content) = delta.get("content").and_then(Value::as_str) {
                step_text.push_str(content);
                text.push_str(content);
                (options.on_text_delta)(content, &text);
            }
            if let Some(calls) = delta.get("tool_calls").and_then(Value::as_array) {
                for call in calls {
                    let index = call
                        .get("index")
                        .and_then(Value::as_u64)
                        .unwrap_or(0) as usize;
                    let partial = partial_calls.entry(index).or_default();
                    if let Some(id) = call.get("id").and_then(Value::as_str) {
                        partial.id.push_str(id);
                    }
                    if let Some(function) =
                        call.get("function").and_then(Value::as_object)
                    {
                        if let Some(name) =
                            function.get("name").and_then(Value::as_str)
                        {
                            partial.name.push_str(name);
                        }
                        if let Some(arguments) =
                            function.get("arguments").and_then(Value::as_str)
                        {
                            partial.arguments.push_str(arguments);
                        }
                    }
                }
            }
            Ok(())
        })?;

        if partial_calls.is_empty() {
            return Ok(text);
        }

        let tool_calls = partial_calls
            .iter()
            .map(|(index, call)| {
                json!({
                    "index": index,
                    "id": call.id,
                    "type": "function",
                    "function": {
                        "name": call.name,
                        "arguments": call.arguments
                    }
                })
            })
            .collect::<Vec<_>>();
        conversation.push(json!({
            "role": "assistant",
            "content": if step_text.is_empty() {
                Value::Null
            } else {
                Value::String(step_text)
            },
            "tool_calls": tool_calls,
        }));

        for (_index, call) in partial_calls {
            let tool_call_id = if call.id.is_empty() {
                Uuid::new_v4().to_string()
            } else {
                call.id
            };
            let result = match tool_index.get(&call.name).copied() {
                None => json!({
                    "isError": true,
                    "error": format!("Unknown Fabushi tool: {}", call.name)
                }),
                Some(tool) => {
                    let args =
                        serde_json::from_str::<Value>(&call.arguments)
                            .unwrap_or_else(|_| json!({}));
                    match (options.execute_tool)(tool, args, &tool_call_id) {
                        Ok(value) => value,
                        Err(error) => json!({
                            "isError": true,
                            "error": error.to_string()
                        }),
                    }
                }
            };
            conversation.push(json!({
                "role": "tool",
                "tool_call_id": tool_call_id,
                "content": serde_json::to_string(&result)
                    .unwrap_or_else(|_| "null".into()),
            }));
        }
    }

    Err(ProviderSessionError::Protocol(
        "OpenRouter exceeded Fabushi's 8-step tool limit.".into(),
    ))
}

fn resolve_claude_cli_path() -> Option<PathBuf> {
    let home = home_dir();
    let mut candidates = vec![
        env::var_os("CLAUDE_CODE_PATH").map(PathBuf::from),
        Some(home.join(".local").join("bin").join("claude")),
        Some(home.join(".claude").join("local").join("claude")),
    ];
    if let Some(path) = env::var_os("PATH") {
        candidates.extend(
            env::split_paths(&path)
                .map(|directory| Some(directory.join("claude"))),
        );
    }
    candidates.extend([
        Some(PathBuf::from("/opt/homebrew/bin/claude")),
        Some(PathBuf::from("/usr/local/bin/claude")),
    ]);
    candidates
        .into_iter()
        .flatten()
        .find(|path| path.is_file())
}

fn run_claude_code_provider_text(
    messages: &[ProviderMessage],
    options: &mut RoutedProviderOptions<'_>,
) -> Result<String, ProviderSessionError> {
    let executable = resolve_claude_cli_path().ok_or_else(|| {
        ProviderSessionError::Configuration(
            "Claude Code is not installed. Install and sign in to Claude Code, then reopen Fabushi."
                .into(),
        )
    })?;
    let mut command = Command::new(executable);
    command
        .arg("-p")
        .arg(provider_prompt(messages))
        .arg("--output-format")
        .arg("json")
        .arg("--max-turns")
        .arg(if options.mcp_server_url.is_some() {
            "8"
        } else {
            "1"
        });

    let mut mcp_config_path = None;
    if let Some(url) = options.mcp_server_url {
        let path = env::temp_dir().join(format!(
            "fabushi-claude-mcp-{}.json",
            Uuid::new_v4()
        ));
        fs::write(
            &path,
            serde_json::to_vec(&json!({
                "mcpServers": {
                    "grok_bot_plugins": {
                        "type": "http",
                        "url": url
                    }
                }
            }))
            .map_err(|error| ProviderSessionError::Protocol(error.to_string()))?,
        )
        .map_err(|error| ProviderSessionError::Transport(error.to_string()))?;
        command
            .arg("--mcp-config")
            .arg(&path)
            .arg("--strict-mcp-config")
            .arg("--allowedTools")
            .arg("mcp__grok_bot_plugins__*");
        mcp_config_path = Some(path);
    }
    if let Some(model) = env::var("SAND_CLAUDE_MODEL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        command.arg("--model").arg(model);
    }

    let output = command.output().map_err(|error| {
        ProviderSessionError::Transport(format!(
            "Could not run Claude Code: {error}"
        ))
    });
    if let Some(path) = mcp_config_path {
        let _ = fs::remove_file(path);
    }
    let output = output?;
    if !output.status.success() {
        return Err(ProviderSessionError::Transport(format!(
            "Claude Code failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    let stdout = String::from_utf8(output.stdout)
        .map_err(|error| ProviderSessionError::Protocol(error.to_string()))?;
    let payload = serde_json::from_str::<Value>(stdout.trim())
        .ok()
        .or_else(|| {
            stdout
                .lines()
                .rev()
                .find_map(|line| serde_json::from_str::<Value>(line).ok())
        })
        .ok_or_else(|| {
            ProviderSessionError::Protocol(
                "Claude Code returned invalid JSON.".into(),
            )
        })?;
    if payload
        .get("subtype")
        .and_then(Value::as_str)
        .is_some_and(|value| value != "success")
    {
        return Err(ProviderSessionError::Protocol(format!(
            "Claude Code failed: {}",
            payload
                .get("errors")
                .cloned()
                .unwrap_or_else(|| Value::String("unknown failure".into()))
        )));
    }
    let text = payload
        .get("result")
        .and_then(Value::as_str)
        .or_else(|| payload.get("content").and_then(Value::as_str))
        .ok_or_else(|| {
            ProviderSessionError::Protocol(
                "Claude Code ended without a result.".into(),
            )
        })?
        .to_string();
    if !text.is_empty() {
        (options.on_text_delta)(&text, &text);
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::{RoutedProvider, jwt_audience, toml_string_setting};
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;

    #[test]
    fn parses_reference_provider_names() {
        assert_eq!(RoutedProvider::parse("cursor"), Some(RoutedProvider::Cursor));
        assert_eq!(RoutedProvider::parse("codex"), Some(RoutedProvider::Codex));
        assert_eq!(
            RoutedProvider::parse("claude-code"),
            Some(RoutedProvider::ClaudeCode)
        );
        assert_eq!(
            RoutedProvider::parse("openrouter"),
            Some(RoutedProvider::OpenRouter)
        );
        assert_eq!(RoutedProvider::parse("other"), None);
    }

    #[test]
    fn reads_quoted_codex_settings_without_accepting_comments() {
        assert_eq!(
            toml_string_setting("model = \"gpt-5.4\"\n", "model").as_deref(),
            Some("gpt-5.4")
        );
        assert_eq!(toml_string_setting("# model = \"x\"\n", "model"), None);
    }

    #[test]
    fn extracts_oauth_client_id_from_jwt_audience() {
        let payload = URL_SAFE_NO_PAD.encode(br#"{"aud":"client-123"}"#);
        let token = format!("header.{payload}.signature");
        assert_eq!(jwt_audience(&token).as_deref(), Some("client-123"));
    }
}
