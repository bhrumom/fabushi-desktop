use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use reqwest::blocking::{Client, Response};
use serde_json::{Map, Value, json};
use thiserror::Error;
use uuid::Uuid;

use super::codex_direct_responses::{
    CodexDirectError, CodexDirectOptions, CodexDirectTool, CodexDirectTransport,
    run_codex_direct_responses,
};

pub const GROK_ROUTER_SYSTEM_PROMPT: &str =
    "You are Grok Bot, a warm, concise desktop assistant.\n\
You are running inside Grok Bot, not inside Codex CLI or Claude Code.\n\
The tools supplied with this request are Grok Bot's already-connected plugins and accounts. Use them whenever they are relevant instead of claiming that a plugin is unavailable or asking the user to reconnect it.\n\
Never ask for an API key for an already-connected plugin. Respond directly to the user in natural language after completing any necessary tool calls.";

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
    let mut request = CodexDirectOptions::new(
        configured_codex_model(),
        GROK_ROUTER_SYSTEM_PROMPT,
        messages
            .iter()
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
