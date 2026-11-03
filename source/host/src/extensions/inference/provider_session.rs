use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::future::Future;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use reqwest::{Client as AsyncClient, Response as AsyncResponse};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use thiserror::Error;
use tokio::runtime::{Builder as TokioRuntimeBuilder, Runtime};
use tokio::time::timeout;
use uuid::Uuid;

use crate::extensions::auth::auth_service::{
    FabushiCoreCredential, FabushiCredentialContext, parse_fabushi_core_credential,
};
use crate::extensions::auth::credential_renewer::system_now_ms;
use crate::host_paths::get_sand_root_dir;
use crate::extensions::session::typed_replay::TypedReplayEntry;

use super::cursor_inference_transport::{
    CursorCheckpoint, CursorCheckpointMessage, CursorInferenceAuth, CursorInferenceTransport,
};
use super::codex_direct_responses::{
    CodexDirectCheckpoint, CodexDirectError, CodexDirectOptions, CodexDirectTool,
    CodexProviderRefusal,
    CodexDirectTransport, CodexDirectUsage, run_codex_direct_responses_with_cancel,
    parse_codex_provider_refusal, run_codex_direct_responses_with_lifecycle_and_tool_step_reminder,
};
use super::typed_replay_serialization::{
    serialize_chat_replay, serialize_cursor_replay, serialize_responses_replay,
};

#[cfg(test)]
#[path = "codex_model_identity_tests.rs"]
mod codex_model_identity_tests;

#[path = "codex_request_binding.rs"]
mod codex_request_binding;
pub use codex_request_binding::{
    CodexResponsesRequestBinding, capture_codex_responses_request_binding,
};
use codex_request_binding::validate_codex_access_workspace;

#[cfg(test)]
#[path = "codex_request_workspace_tests.rs"]
mod codex_request_workspace_tests;

pub const FABUSHI_ROUTER_SYSTEM_PROMPT: &str =
    "You are Fabushi, a warm, concise desktop assistant.\n\
You are running inside Fabushi, not inside Codex CLI or Claude Code.\n\
The tools supplied with this request are Fabushi's already-connected plugins and accounts. Use them whenever they are relevant instead of claiming that a plugin is unavailable or asking the user to reconnect it.\n\
Never ask for an API key for an already-connected plugin. Respond directly to the user in natural language after completing any necessary tool calls.";


fn assembled_provider_system_prompt(messages: &[ProviderMessage]) -> String {
    let additions = messages
        .iter()
        .filter(|message| message.role == "system")
        .map(|message| message.content.trim())
        .filter(|content| !content.is_empty())
        .collect::<Vec<_>>();
    if additions.is_empty() {
        FABUSHI_ROUTER_SYSTEM_PROMPT.to_string()
    } else {
        format!("{FABUSHI_ROUTER_SYSTEM_PROMPT}\n\n{}", additions.join("\n\n"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutedProvider {
    Fabushi,
    Cursor,
    Codex,
    ClaudeCode,
    OpenRouter,
}

impl RoutedProvider {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "fabushi" => Some(Self::Fabushi),
            "cursor" => Some(Self::Cursor),
            "codex" => Some(Self::Codex),
            "claude-code" => Some(Self::ClaudeCode),
            "openrouter" => Some(Self::OpenRouter),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fabushi => "fabushi",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProviderTokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub reasoning_tokens: Option<u64>,
}

impl From<CodexDirectUsage> for ProviderTokenUsage {
    fn from(value: CodexDirectUsage) -> Self {
        Self {
            input_tokens: value.input_tokens,
            output_tokens: value.output_tokens,
            cache_read_tokens: value.cache_read_tokens,
            cache_write_tokens: value.cache_write_tokens,
            reasoning_tokens: None,
        }
    }
}

pub fn merge_provider_token_usage(
    current: Option<ProviderTokenUsage>,
    next: ProviderTokenUsage,
) -> ProviderTokenUsage {
    match current {
        None => next,
        Some(current) => ProviderTokenUsage {
            input_tokens: current.input_tokens.saturating_add(next.input_tokens),
            output_tokens: current.output_tokens.saturating_add(next.output_tokens),
            cache_read_tokens: current.cache_read_tokens.saturating_add(next.cache_read_tokens),
            cache_write_tokens: current.cache_write_tokens.saturating_add(next.cache_write_tokens),
            reasoning_tokens: match (current.reasoning_tokens, next.reasoning_tokens) {
                (None, None) => None,
                (left, right) => Some(
                    left.unwrap_or_default().saturating_add(right.unwrap_or_default()),
                ),
            },
        },
    }
}

pub fn provider_token_usage_from_openrouter_event(
    event: &Value,
) -> Option<ProviderTokenUsage> {
    let usage = event.get("usage")?.as_object()?;
    Some(ProviderTokenUsage {
        input_tokens: usage
            .get("prompt_tokens")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
        output_tokens: usage
            .get("completion_tokens")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
        cache_read_tokens: usage
            .get("prompt_tokens_details")
            .and_then(Value::as_object)
            .and_then(|details| details.get("cached_tokens"))
            .and_then(Value::as_u64)
            .unwrap_or_default(),
        cache_write_tokens: 0,
        reasoning_tokens: usage
            .get("completion_tokens_details")
            .and_then(Value::as_object)
            .and_then(|details| details.get("reasoning_tokens"))
            .and_then(Value::as_u64),
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoutedToolDefinition {
    pub name: String,
    pub provider_identifier: String,
    pub tool_name: String,
    pub description: Option<String>,
    pub input_schema: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoutedMcpMetaToolDefinition {
    pub tool: RoutedToolDefinition,
    pub plugin: Option<Value>,
    pub marketplace: Option<Value>,
    pub plugin_id: Option<String>,
    pub marketplace_id: Option<String>,
}

impl From<RoutedToolDefinition> for RoutedMcpMetaToolDefinition {
    fn from(tool: RoutedToolDefinition) -> Self {
        Self {
            tool,
            plugin: None,
            marketplace: None,
            plugin_id: None,
            marketplace_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderPartialToolCall {
    pub tool: RoutedToolDefinition,
    pub tool_call_id: String,
    pub raw_arguments: String,
    pub model_call_id: Option<String>,
}

pub type ProviderToolStepReminderHook =
    dyn Fn(&[(String, Value)]) -> Option<String> + Send + Sync;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "provider", content = "checkpoint", rename_all = "kebab-case")]
pub enum RoutedProviderCheckpoint {
    Fabushi(CodexDirectCheckpoint),
    Cursor(CursorCheckpoint),
    Codex(CodexDirectCheckpoint),
    OpenRouter(OpenRouterCheckpoint),
}

impl RoutedProviderCheckpoint {
    pub fn emitted_text_bytes(&self) -> usize {
        match self {
            Self::Fabushi(checkpoint) => checkpoint.text.len(),
            Self::Cursor(checkpoint) => checkpoint.text.len(),
            Self::Codex(checkpoint) => checkpoint.text.len(),
            Self::OpenRouter(checkpoint) => checkpoint.text.len(),
        }
    }

    pub fn tool_calls_completed(&self) -> usize {
        match self {
            Self::Fabushi(checkpoint) => checkpoint.tool_calls_completed,
            Self::Cursor(checkpoint) => checkpoint.tool_calls_completed,
            Self::Codex(checkpoint) => checkpoint.tool_calls_completed,
            Self::OpenRouter(checkpoint) => checkpoint.tool_calls_completed,
        }
    }

    pub fn provider(&self) -> RoutedProvider {
        match self {
            Self::Fabushi(_) => RoutedProvider::Fabushi,
            Self::Cursor(_) => RoutedProvider::Cursor,
            Self::Codex(_) => RoutedProvider::Codex,
            Self::OpenRouter(_) => RoutedProvider::OpenRouter,
        }
    }
}

/// A received catalog is unusable as a whole. These fixed fields deliberately
/// exclude the rejected payload and account or descriptor values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpCatalogKind {
    Tools,
    MetaTools,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpCatalogReason {
    ExpectedArray,
    NameMustBeString,
    ProviderIdentifierMustBeString,
    ToolNameMustBeString,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("MCP {kind:?} catalog refused at row {row:?}: {reason:?}")]
pub struct McpCatalogRefusal {
    pub kind: McpCatalogKind,
    pub row: Option<usize>,
    pub reason: McpCatalogReason,
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
    #[error("{0}")]
    McpCatalog(McpCatalogRefusal),
    #[error("tool execution outcome is unknown: {0}")]
    ToolOutcomeUnknown(String),
    #[error("provider refusal: {0:?}")]
    ProviderRefusal(CodexProviderRefusal),
    #[error("{0}")]
    Cancelled(String),
}

impl From<CodexDirectError> for ProviderSessionError {
    fn from(error: CodexDirectError) -> Self {
        match error {
            CodexDirectError::Authentication(message) => Self::Authentication(message),
            CodexDirectError::Configuration(message) => Self::Configuration(message),
            CodexDirectError::Transport(message) => Self::Transport(message),
            CodexDirectError::Protocol(message) => Self::Protocol(message),
            CodexDirectError::Tool(message) => Self::Tool(message),
            CodexDirectError::ProviderRefusal(refusal) => Self::ProviderRefusal(refusal),
            CodexDirectError::Cancelled(message) => Self::Cancelled(message),
        }
    }
}

pub const PROVIDER_IO_CANCEL_POLL_MS: u64 = 50;

static PROVIDER_IO_RUNTIME: OnceLock<Result<Runtime, String>> = OnceLock::new();
static RESPONSES_HTTP_CLIENT: OnceLock<Result<AsyncClient, String>> = OnceLock::new();

fn provider_io_runtime() -> Result<&'static Runtime, ProviderSessionError> {
    PROVIDER_IO_RUNTIME
        .get_or_init(|| {
            TokioRuntimeBuilder::new_multi_thread()
                .worker_threads(1)
                .thread_name("mahayana-provider-io")
                .enable_io()
                .enable_time()
                .build()
                .map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(|error| ProviderSessionError::Transport(format!(
            "Could not create provider I/O runtime: {error}"
        )))
}

fn responses_http_client() -> Result<&'static AsyncClient, ProviderSessionError> {
    RESPONSES_HTTP_CLIENT
        .get_or_init(|| {
            AsyncClient::builder()
                .pool_idle_timeout(Duration::from_secs(90))
                .tcp_keepalive(Duration::from_secs(60))
                .build()
                .map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(|error| ProviderSessionError::Transport(format!(
            "Could not create shared Responses HTTP client: {error}"
        )))
}

async fn await_cancelable_provider_io<F, T>(
    future: F,
    should_cancel: &dyn Fn() -> bool,
    label: &str,
) -> Result<T, ProviderSessionError>
where
    F: Future<Output = T>,
{
    let mut future = Box::pin(future);
    loop {
        if should_cancel() {
            return Err(ProviderSessionError::Cancelled(format!(
                "Runner cancelled {label}"
            )));
        }
        match timeout(
            Duration::from_millis(PROVIDER_IO_CANCEL_POLL_MS),
            future.as_mut(),
        )
        .await
        {
            Ok(value) => return Ok(value),
            Err(_) => continue,
        }
    }
}

async fn await_reqwest<T, F>(
    future: F,
    should_cancel: &dyn Fn() -> bool,
    label: &str,
) -> Result<T, ProviderSessionError>
where
    F: Future<Output = Result<T, reqwest::Error>>,
{
    await_cancelable_provider_io(future, should_cancel, label)
        .await?
        .map_err(|error| ProviderSessionError::Transport(error.to_string()))
}

fn codex_provider_error(error: ProviderSessionError) -> CodexDirectError {
    match error {
        ProviderSessionError::Authentication(message) => CodexDirectError::Authentication(message),
        ProviderSessionError::Configuration(message) => CodexDirectError::Configuration(message),
        ProviderSessionError::Cancelled(message) => CodexDirectError::Cancelled(message),
        ProviderSessionError::Protocol(message) => CodexDirectError::Protocol(message),
        ProviderSessionError::Tool(message) => CodexDirectError::Tool(message),
        error @ ProviderSessionError::ToolOutcomeUnknown(_) => {
            CodexDirectError::Tool(error.to_string())
        }
        ProviderSessionError::ProviderRefusal(refusal) => CodexDirectError::ProviderRefusal(refusal),
        other => CodexDirectError::Transport(other.to_string()),
    }
}

#[derive(Clone)]
struct CodexCredentials {
    access_token: String,
    refresh_token: String,
    id_token: String,
    account_id: String,
    path: PathBuf,
    document: Value,
}

impl std::fmt::Debug for CodexCredentials {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("CodexCredentials")
            .field("access_token", &"<redacted>")
            .field("refresh_token", &"<redacted>")
            .field("id_token", &"<redacted>")
            .field("account_id", &self.account_id)
            .field("path", &self.path)
            .field("document", &"<redacted>")
            .finish()
    }
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

fn normalized_fabushi_model_selection(model: &str) -> Result<&str, ProviderSessionError> {
    if model.chars().any(char::is_control) || model.trim().is_empty() {
        return Err(ProviderSessionError::Configuration(
            "Fabushi model selection must be nonblank and contain no control characters.".into(),
        ));
    }
    Ok(model.trim())
}

fn configured_fabushi_model() -> Result<String, ProviderSessionError> {
    let configured = match env::var("FABUSHI_RESPONSES_MODEL") {
        Ok(value) => value,
        Err(env::VarError::NotPresent) => return Ok("deepseek-chat".into()),
        Err(env::VarError::NotUnicode(_)) => {
            return Err(ProviderSessionError::Configuration(
                "FABUSHI_RESPONSES_MODEL must contain a valid Unicode model identity.".into(),
            ));
        }
    };
    if configured.trim().is_empty() && !configured.chars().any(char::is_control) {
        return Ok("deepseek-chat".into());
    }
    normalized_fabushi_model_selection(&configured).map(str::to_string)
}

/// Resolve one request identity without consulting authentication or transport.
/// Accepted Fabushi checkpoints pin their normalized model across retries.
pub fn resolve_fabushi_request_model(
    requested: Option<&str>,
    resume: Option<&CodexDirectCheckpoint>,
) -> Result<String, ProviderSessionError> {
    let checkpoint_model = resume
        .map(|checkpoint| {
            checkpoint
                .model
                .as_deref()
                .filter(|model| {
                    !model.is_empty()
                        && model.trim() == *model
                        && !model.chars().any(char::is_control)
                })
                .ok_or_else(|| {
                    ProviderSessionError::Protocol(
                        "Fabushi checkpoint requires a nonblank, normalized model identity without control characters.".into(),
                    )
                })
        })
        .transpose()?;
    let requested = requested
        .map(normalized_fabushi_model_selection)
        .transpose()?;
    match (requested, checkpoint_model) {
        (Some(requested), Some(checkpoint)) if requested != checkpoint => {
            Err(ProviderSessionError::Protocol(
                "Fabushi checkpoint model identity does not match this turn.".into(),
            ))
        }
        (_, Some(checkpoint)) => Ok(checkpoint.to_string()),
        (Some(requested), None) => Ok(requested.to_string()),
        (None, None) => configured_fabushi_model(),
    }
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

fn normalized_codex_model_selection(model: &str) -> Result<&str, ProviderSessionError> {
    if model.chars().any(char::is_control) || model.trim().is_empty() {
        return Err(ProviderSessionError::Configuration(
            "Codex model selection must be nonblank and contain no control characters.".into(),
        ));
    }
    Ok(model.trim())
}

/// Check Codex request identity before authentication or transport creation.
/// Independent calls keep their compatibility fallback; an absent model in a
/// legacy Codex checkpoint does not manufacture or override that selection.
pub fn resolve_codex_request_model(
    requested: Option<&str>,
    resume: Option<&CodexDirectCheckpoint>,
) -> Result<String, ProviderSessionError> {
    let checkpoint_model = resume.and_then(|checkpoint| checkpoint.model.as_deref());
    if let Some(model) = checkpoint_model {
        if model.is_empty() || model.trim() != model || model.chars().any(char::is_control) {
            return Err(ProviderSessionError::Protocol(
                "Codex checkpoint requires a nonblank, normalized model identity without control characters.".into(),
            ));
        }
    }
    let selected = requested.map(str::to_string).unwrap_or_else(configured_codex_model);
    let model = normalized_codex_model_selection(&selected)?.to_string();
    if checkpoint_model.is_some_and(|checkpoint| checkpoint != model.as_str()) {
        return Err(ProviderSessionError::Protocol(
            "Codex checkpoint model identity does not match this turn.".into(),
        ));
    }
    Ok(model)
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
    validate_codex_access_workspace(&access_token, &account_id)?;
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
    // Validate the actual new token before mutating or publishing any image.
    // The binding is a workspace boundary, not signature verification.
    validate_codex_access_workspace(&access_token, &current.account_id)?;
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

async fn refresh_codex_credentials(
    client: &AsyncClient,
    current: &CodexCredentials,
    should_cancel: &dyn Fn() -> bool,
    #[cfg(test)] request_hook: Option<&codex_request_workspace_tests::CapturedRequestHook>,
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
    let response = await_reqwest(
        execute_responses_http_request(client
            .post("https://auth.openai.com/oauth/token")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(body),
            #[cfg(test)] request_hook,
            #[cfg(test)] codex_request_workspace_tests::RequestKind::Refresh,
        ),
        should_cancel,
        "the Codex login refresh request",
    )
    .await?;
    if !response.status().is_success() {
        return Err(ProviderSessionError::Authentication(
            "Codex login expired and could not be refreshed. Run `codex login` again."
                .into(),
        ));
    }
    let payload: Value = await_reqwest(
        response.json::<Value>(),
        should_cancel,
        "the Codex login refresh response",
    )
    .await
    .map_err(|error| match error {
        ProviderSessionError::Cancelled(message) => ProviderSessionError::Cancelled(message),
        other => ProviderSessionError::Protocol(format!(
            "Codex refresh response was invalid: {other}"
        )),
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

fn flush_sse_data<F>(
    data: &mut Vec<String>,
    on_event: &mut F,
) -> Result<(), ProviderSessionError>
where
    F: FnMut(Value) -> Result<(), ProviderSessionError> + ?Sized,
{
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
}

fn push_sse_line<F>(
    line: &str,
    data: &mut Vec<String>,
    on_event: &mut F,
) -> Result<(), ProviderSessionError>
where
    F: FnMut(Value) -> Result<(), ProviderSessionError> + ?Sized,
{
    let trimmed = line.trim_end_matches(['\r', '\n']);
    if trimmed.is_empty() {
        flush_sse_data(data, on_event)?;
    } else if let Some(payload) = trimmed.strip_prefix("data:") {
        data.push(payload.trim_start().to_string());
    }
    Ok(())
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

    loop {
        line.clear();
        let count = reader
            .read_line(&mut line)
            .map_err(|error| ProviderSessionError::Transport(error.to_string()))?;
        if count == 0 {
            break;
        }
        push_sse_line(&line, &mut data, &mut on_event)?;
    }
    flush_sse_data(&mut data, &mut on_event)
}

async fn decode_async_sse_stream<F>(
    mut response: AsyncResponse,
    should_cancel: &dyn Fn() -> bool,
    on_event: &mut F,
) -> Result<(), ProviderSessionError>
where
    F: FnMut(Value) -> Result<(), ProviderSessionError> + ?Sized,
{
    let mut pending = Vec::<u8>::new();
    let mut data = Vec::<String>::new();

    loop {
        let chunk = await_reqwest(
            response.chunk(),
            should_cancel,
            "the provider response stream",
        )
        .await?;
        let Some(chunk) = chunk else {
            break;
        };
        pending.extend_from_slice(&chunk);
        while let Some(newline) = pending.iter().position(|byte| *byte == b'\n') {
            let line = pending.drain(..=newline).collect::<Vec<_>>();
            let line = std::str::from_utf8(&line).map_err(|_| {
                ProviderSessionError::Protocol(
                    "Provider stream contained invalid UTF-8.".into(),
                )
            })?;
            push_sse_line(line, &mut data, on_event)?;
        }
    }

    if !pending.is_empty() {
        let line = std::str::from_utf8(&pending).map_err(|_| {
            ProviderSessionError::Protocol(
                "Provider stream contained invalid UTF-8.".into(),
            )
        })?;
        push_sse_line(line, &mut data, on_event)?;
    }
    flush_sse_data(&mut data, on_event)
}

const FABUSHI_RESPONSES_URL_ENV: &str = "FABUSHI_RESPONSES_URL";
const FABUSHI_HOST_ACCESS_CREDENTIAL_FILE_ENV: &str = "FABUSHI_HOST_ACCESS_CREDENTIAL_FILE";
const DEFAULT_FABUSHI_NATIVE_ISSUER: &str = "https://api.ombhrum.com";
const CI_ISOLATED_FABUSHI_NATIVE_ISSUER: &str = "https://fabushi-ci-isolated-auth.bhrumom.workers.dev";

#[derive(Debug, Clone)]
enum ResponsesHttpCredentials {
    ChatGpt(CodexCredentials),
    Fabushi {
        credential: FabushiCoreCredential,
        endpoint: String,
        credential_path: PathBuf,
    },
}

fn validate_fabushi_responses_url(raw: &str) -> Result<String, ProviderSessionError> {
    let url = url::Url::parse(raw).map_err(|error| {
        ProviderSessionError::Configuration(format!(
            "Fabushi Responses URL is invalid: {error}"
        ))
    })?;
    let loopback = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "::1"));
    if url.scheme() != "https" && !(loopback && url.scheme() == "http") {
        return Err(ProviderSessionError::Configuration(
            "Fabushi Responses URL must use HTTPS outside loopback development.".into(),
        ));
    }
    if !url.username().is_empty() || url.password().is_some() || url.query().is_some() || url.fragment().is_some() {
        return Err(ProviderSessionError::Configuration(
            "Fabushi Responses URL must not contain credentials, query, or fragment.".into(),
        ));
    }
    let origin = url.origin().ascii_serialization();
    if !loopback && origin != DEFAULT_FABUSHI_NATIVE_ISSUER
        && origin != CI_ISOLATED_FABUSHI_NATIVE_ISSUER {
        return Err(ProviderSessionError::Configuration(
            "Fabushi Responses URL must use an approved Native issuer origin.".into(),
        ));
    }
    Ok(url.to_string())
}

fn ensure_responses_issuer_matches(endpoint: &str, issuer: &str) -> Result<(), ProviderSessionError> {
    let endpoint_url = url::Url::parse(endpoint).map_err(|_| ProviderSessionError::Configuration(
        "Fabushi Responses URL is invalid.".into(),
    ))?;
    if endpoint_url.origin().ascii_serialization() != issuer {
        return Err(ProviderSessionError::Authentication(
            "Fabushi account credential issuer does not match the inference service origin.".into(),
        ));
    }
    Ok(())
}

fn read_fabushi_responses_credentials(
    endpoint: &str,
    path: &Path,
) -> Result<ResponsesHttpCredentials, ProviderSessionError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        ProviderSessionError::Authentication(format!(
            "Fabushi account-bound inference credential is unavailable: {error}"
        ))
    })?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(ProviderSessionError::Authentication(
            "Fabushi account-bound inference credential must be a private regular file.".into(),
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(ProviderSessionError::Authentication(
                "Fabushi account-bound inference credential must be private to the current user.".into(),
            ));
        }
    }
    let raw = fs::read_to_string(path).map_err(|error| {
        ProviderSessionError::Authentication(format!(
            "Could not read Fabushi account-bound inference credential: {error}"
        ))
    })?;
    let credential = parse_fabushi_core_credential(&raw, system_now_ms)
        .map_err(|error| ProviderSessionError::Authentication(error.to_string()))?;
    ensure_responses_issuer_matches(endpoint, credential.issuer())?;
    Ok(ResponsesHttpCredentials::Fabushi {
        credential,
        endpoint: endpoint.to_string(),
        credential_path: path.to_path_buf(),
    })
}

fn configured_fabushi_responses_credentials() -> Result<ResponsesHttpCredentials, ProviderSessionError> {
    let endpoint = env::var(FABUSHI_RESPONSES_URL_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ProviderSessionError::Configuration(
            "Fabushi Responses transport requires the first-party Responses URL.".into(),
        ))?;
    let endpoint = validate_fabushi_responses_url(&endpoint)?;
    let credential_path = env::var_os(FABUSHI_HOST_ACCESS_CREDENTIAL_FILE_ENV)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| ProviderSessionError::Configuration(
            "Fabushi Responses transport requires the account-bound credential file.".into(),
        ))?;
    read_fabushi_responses_credentials(&endpoint, &credential_path)
}

/// Identity shared by the requests belonging to one admitted Native turn.
/// Credentials remain transport-owned and are never cached in this binding.
#[derive(Clone)]
pub struct FabushiResponsesRequestBinding {
    identity: Arc<OnceLock<FabushiResponsesRequestIdentity>>,
}

#[derive(Clone, PartialEq, Eq)]
struct FabushiResponsesRequestIdentity {
    context: FabushiCredentialContext,
    endpoint: String,
    credential_path: PathBuf,
}

impl FabushiResponsesRequestBinding {
    /// Direct Runner callers bind on their first valid transport, after
    /// checkpoint/model validation. Allocation itself performs no I/O.
    pub(crate) fn for_runner_invocation() -> Self {
        Self { identity: Arc::new(OnceLock::new()) }
    }

    fn accept_credentials(
        &self,
        credentials: &ResponsesHttpCredentials,
    ) -> Result<(), ProviderSessionError> {
        let ResponsesHttpCredentials::Fabushi {
            credential, endpoint, credential_path,
        } = credentials else {
            return Err(ProviderSessionError::Configuration(
                "Native request binding received a different provider.".into(),
            ));
        };
        let candidate = FabushiResponsesRequestIdentity {
            context: credential.context(),
            endpoint: endpoint.clone(),
            credential_path: credential_path.clone(),
        };
        // A racing first reader may lose initialization, but may never adopt
        // or replace another reader's context, endpoint or credential source.
        if let Err(candidate) = self.identity.set(candidate) {
            if self.identity.get() != Some(&candidate) {
                return Err(ProviderSessionError::Authentication(
                    "Fabushi account context changed during this turn.".into(),
                ));
            }
        }
        Ok(())
    }

    fn credentials_for_transport(
        &self,
    ) -> Result<ResponsesHttpCredentials, ProviderSessionError> {
        let credentials = match self.identity.get() {
            Some(identity) => read_fabushi_responses_credentials(
                &identity.endpoint, &identity.credential_path,
            )?,
            None => configured_fabushi_responses_credentials()?,
        };
        self.accept_credentials(&credentials)?;
        Ok(credentials)
    }
}

pub fn capture_fabushi_responses_request_binding(
) -> Result<FabushiResponsesRequestBinding, ProviderSessionError> {
    let binding = FabushiResponsesRequestBinding::for_runner_invocation();
    // Validate the actual image now, then discard its token/expiry. Each
    // transport rereads current credentials from this immutable identity.
    let _ = binding.credentials_for_transport()?;
    Ok(binding)
}

struct CodexHttpTransport {
    client: AsyncClient,
    credentials: ResponsesHttpCredentials,
    #[cfg(test)]
    request_hook: Option<codex_request_workspace_tests::CapturedRequestHook>,
}

impl CodexHttpTransport {
    fn with_credentials(credentials: ResponsesHttpCredentials) -> Result<Self, ProviderSessionError> {
        let client = responses_http_client()?.clone();
        #[cfg(test)]
        let request_hook = if matches!(&credentials, ResponsesHttpCredentials::ChatGpt(_)) {
            codex_request_workspace_tests::capture_request_hook_for_current_thread()
        } else {
            None
        };
        Ok(Self {
            client,
            credentials,
            #[cfg(test)] request_hook,
        })
    }

    fn new_fabushi() -> Result<Self, ProviderSessionError> {
        Self::with_credentials(configured_fabushi_responses_credentials()?)
    }

    fn new_fabushi_with_binding(
        binding: Option<&FabushiResponsesRequestBinding>,
    ) -> Result<Self, ProviderSessionError> {
        match binding {
            Some(binding) => Self::with_credentials(binding.credentials_for_transport()?),
            None => Self::new_fabushi(),
        }
    }
}

fn new_codex_direct_transport(
    binding: Option<&CodexResponsesRequestBinding>,
) -> Result<Box<dyn CodexDirectTransport>, ProviderSessionError> {
    // Even model-only test substitution must honor supplied admission identity.
    let credentials = binding.map(CodexResponsesRequestBinding::credentials_for_transport)
        .transpose()?;
    #[cfg(test)]
    if let Some(transport) = codex_model_identity_tests::transport_for_current_thread() {
        return Ok(transport);
    }
    let transport = match credentials {
        Some(credentials) => CodexHttpTransport::with_credentials(
            ResponsesHttpCredentials::ChatGpt(credentials),
        )?,
        None => CodexHttpTransport::with_credentials(ResponsesHttpCredentials::ChatGpt(
            CodexResponsesRequestBinding::for_runner_invocation().credentials_for_transport()?,
        ))?,
    };
    Ok(Box::new(transport))
}

// Building remains inside the cancelable future: a pre-dispatch cancellation
// must win over a stored RequestBuilder error. The original client is retained.
async fn execute_responses_http_request(
    builder: reqwest::RequestBuilder,
    #[cfg(test)] request_hook: Option<&codex_request_workspace_tests::CapturedRequestHook>,
    #[cfg(test)] request_kind: codex_request_workspace_tests::RequestKind,
) -> Result<AsyncResponse, reqwest::Error> {
    let (client, request) = builder.build_split();
    let request = request?;
    #[cfg(test)]
    let request = {
        let mut request = request;
        if let Some(hook) = request_hook {
            hook.apply(request_kind, &mut request);
        }
        request
    };
    client.execute(request).await
}

async fn send_codex_request(
    client: &AsyncClient,
    credentials: &ResponsesHttpCredentials,
    request: &Value,
    should_cancel: &dyn Fn() -> bool,
    #[cfg(test)] request_hook: Option<&codex_request_workspace_tests::CapturedRequestHook>,
) -> Result<AsyncResponse, ProviderSessionError> {
    let request_builder = match credentials {
        ResponsesHttpCredentials::ChatGpt(credentials) => client
            .post("https://chatgpt.com/backend-api/codex/responses")
            .header(
                "authorization",
                format!("Bearer {}", credentials.access_token),
            )
            .header("ChatGPT-Account-Id", credentials.account_id.clone()),
        ResponsesHttpCredentials::Fabushi {
            credential,
            endpoint,
            ..
        } => client
            .post(endpoint)
            .header("authorization", format!("Bearer {}", credential.access_token().trim())),
    };
    await_reqwest(
        execute_responses_http_request(request_builder
            .header("content-type", "application/json")
            .header("accept", "text/event-stream")
            .header("user-agent", "fabushi-router/1")
            .json(request),
            #[cfg(test)] request_hook,
            #[cfg(test)] codex_request_workspace_tests::RequestKind::Responses,
        ),
        should_cancel,
        "the Responses provider request",
    )
    .await
}

async fn refresh_responses_credentials(
    client: &AsyncClient,
    current: &ResponsesHttpCredentials,
    should_cancel: &dyn Fn() -> bool,
    #[cfg(test)] request_hook: Option<&codex_request_workspace_tests::CapturedRequestHook>,
) -> Result<ResponsesHttpCredentials, ProviderSessionError> {
    match current {
        ResponsesHttpCredentials::ChatGpt(credentials) => Ok(
            ResponsesHttpCredentials::ChatGpt(
                refresh_codex_credentials(client, credentials, should_cancel,
                    #[cfg(test)] request_hook,
                ).await?,
            ),
        ),
        ResponsesHttpCredentials::Fabushi {
            credential,
            endpoint,
            credential_path,
        } => {
            // Reread one typed image, retaining the complete endpoint and the
            // account/session binding captured by this transport. A new login
            // requires a new transport; it cannot take over a 401 retry.
            let refreshed = read_fabushi_responses_credentials(endpoint, credential_path)?;
            if let ResponsesHttpCredentials::Fabushi { credential: next, .. } = &refreshed {
                if next.context() == credential.context() {
                    return Ok(refreshed);
                }
            }
            Err(ProviderSessionError::Authentication(
                "Fabushi account credential context changed during the Responses request.".into(),
            ))
        }
    }
}

impl CodexDirectTransport for CodexHttpTransport {
    fn stream_response(
        &mut self,
        request: &Value,
        on_event: &mut dyn FnMut(Value) -> Result<(), CodexDirectError>,
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<(), CodexDirectError> {
        let runtime = provider_io_runtime().map_err(codex_provider_error)?;
        let client = self.client.clone();
        let mut credentials = self.credentials.clone();
        let result = runtime.block_on(async {
            let mut response = send_codex_request(
                &client,
                &credentials,
                request,
                should_cancel,
                #[cfg(test)] self.request_hook.as_ref(),
            )
            .await
            .map_err(codex_provider_error)?;

            if response.status().as_u16() == 401 {
                credentials = refresh_responses_credentials(
                    &client,
                    &credentials,
                    should_cancel,
                    #[cfg(test)] self.request_hook.as_ref(),
                )
                .await
                .map_err(codex_provider_error)?;
                response = send_codex_request(
                    &client,
                    &credentials,
                    request,
                    should_cancel,
                    #[cfg(test)] self.request_hook.as_ref(),
                )
                .await
                .map_err(codex_provider_error)?;
            }

            if response.status().as_u16() == 401
                && matches!(&credentials, ResponsesHttpCredentials::Fabushi { .. })
            {
                // The sole reread/retry was rejected. Do not expose a response
                // body that can echo a bearer or turn this into a new attempt.
                return Err(CodexDirectError::Authentication(
                    "Fabushi Responses authentication failed after credential reread.".into(),
                ));
            }

            if !response.status().is_success() {
                let status = response.status();
                let body = await_reqwest(
                    response.text(),
                    should_cancel,
                    "the Responses provider error response",
                )
                .await
                .map_err(codex_provider_error)?;
                let native = matches!(&credentials, ResponsesHttpCredentials::Fabushi { .. });
                if let Ok(payload) = serde_json::from_str::<Value>(&body) {
                    if let Some(refusal) = parse_codex_provider_refusal(&payload) {
                        if native {
                            // Native error bodies can echo account credentials.
                            // Keep only the parser-owned category and HTTP status.
                            return Err(CodexDirectError::ProviderRefusal(CodexProviderRefusal {
                                category: refusal.category,
                                explanation: Some(format!("Fabushi Responses request refused ({status}).")),
                                continuation_message: None,
                                error_type: None,
                            }));
                        }
                        return Err(CodexDirectError::ProviderRefusal(refusal));
                    }
                }
                if native {
                    return Err(CodexDirectError::Transport(format!(
                        "Responses request failed ({status}).",
                    )));
                }
                let detail = body.chars().take(4096).collect::<String>();
                return Err(CodexDirectError::Transport(format!(
                    "Responses request failed ({status}{}).",
                    if detail.trim().is_empty() {
                        String::new()
                    } else {
                        format!(": {}", detail.trim())
                    }
                )));
            }

            decode_async_sse_stream(
                response,
                should_cancel,
                &mut |event| {
                    on_event(event).map_err(ProviderSessionError::from)
                },
            )
            .await
            .map_err(codex_provider_error)
        });
        self.credentials = credentials;
        result
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

fn responses_input_for_turn(
    messages: &[ProviderMessage],
    typed_replay: Option<&[TypedReplayEntry]>,
) -> Result<Vec<Value>, ProviderSessionError> {
    let mut input = match typed_replay {
        Some(entries) => serialize_responses_replay(entries)
            .map_err(|error| ProviderSessionError::Protocol(error.to_string()))?,
        None => Vec::new(),
    };
    if typed_replay.is_none() {
        input.extend(messages.iter().filter(|message| message.role != "system").map(
            |message| {
                json!({
                    "role": if message.role == "assistant" { "assistant" } else { "user" },
                    "content": message.content,
                })
            },
        ));
        return Ok(input);
    }
    let message = messages
        .iter()
        .rev()
        .find(|message| message.role == "user")
        .ok_or_else(|| {
            ProviderSessionError::Protocol(
                "typed compacted replay requires a current user message".into(),
            )
        })?;
    input.push(json!({
        "role": "user",
        "content": message.content,
    }));
    Ok(input)
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
    should_cancel: &dyn Fn() -> bool,
) -> Result<String, ProviderSessionError> {
    let mut ignore_checkpoint =
        |_checkpoint: &CodexDirectCheckpoint| Ok(());
    run_codex_provider_text_with_lifecycle(
        messages,
        tools,
        execute_tool,
        on_text_delta,
        should_cancel,
        None,
        &mut ignore_checkpoint,
    )
}

pub fn run_codex_provider_text_with_lifecycle(
    messages: &[ProviderMessage],
    tools: &[RoutedToolDefinition],
    execute_tool: &mut dyn FnMut(
        &RoutedToolDefinition,
        Value,
        &str,
    ) -> Result<Value, ProviderSessionError>,
    on_text_delta: &mut dyn FnMut(&str, &str),
    should_cancel: &dyn Fn() -> bool,
    resume_from: Option<&CodexDirectCheckpoint>,
    on_checkpoint: &mut dyn FnMut(
        &CodexDirectCheckpoint,
    ) -> Result<(), ProviderSessionError>,
) -> Result<String, ProviderSessionError> {
    let mut ignore_usage = |_usage: ProviderTokenUsage| {};
    run_codex_provider_text_with_lifecycle_reporting_usage(
        messages,
        tools,
        execute_tool,
        on_text_delta,
        should_cancel,
        resume_from,
        on_checkpoint,
        &mut ignore_usage,
    )
}

pub fn run_codex_provider_text_with_lifecycle_reporting_usage(
    messages: &[ProviderMessage],
    tools: &[RoutedToolDefinition],
    execute_tool: &mut dyn FnMut(
        &RoutedToolDefinition,
        Value,
        &str,
    ) -> Result<Value, ProviderSessionError>,
    on_text_delta: &mut dyn FnMut(&str, &str),
    should_cancel: &dyn Fn() -> bool,
    resume_from: Option<&CodexDirectCheckpoint>,
    on_checkpoint: &mut dyn FnMut(
        &CodexDirectCheckpoint,
    ) -> Result<(), ProviderSessionError>,
    on_usage: &mut dyn FnMut(ProviderTokenUsage),
) -> Result<String, ProviderSessionError> {
    run_codex_provider_text_with_lifecycle_reporting_usage_with_max_steps(
        messages,
        tools,
        execute_tool,
        on_text_delta,
        should_cancel,
        resume_from,
        on_checkpoint,
        on_usage,
        8,
        None,
    )
}

pub fn run_codex_provider_text_with_lifecycle_reporting_usage_with_max_steps(
    messages: &[ProviderMessage],
    tools: &[RoutedToolDefinition],
    execute_tool: &mut dyn FnMut(
        &RoutedToolDefinition,
        Value,
        &str,
    ) -> Result<Value, ProviderSessionError>,
    on_text_delta: &mut dyn FnMut(&str, &str),
    should_cancel: &dyn Fn() -> bool,
    resume_from: Option<&CodexDirectCheckpoint>,
    on_checkpoint: &mut dyn FnMut(
        &CodexDirectCheckpoint,
    ) -> Result<(), ProviderSessionError>,
    on_usage: &mut dyn FnMut(ProviderTokenUsage),
    max_steps: usize,
    tool_step_reminder: Option<&ProviderToolStepReminderHook>,
) -> Result<String, ProviderSessionError> {
    run_codex_provider_text_with_lifecycle_reporting_usage_with_max_steps_and_replay(
        messages,
        None,
        None,
        None,
        tools,
        execute_tool,
        on_text_delta,
        should_cancel,
        resume_from,
        on_checkpoint,
        on_usage,
        max_steps,
        tool_step_reminder,
        false,
        #[cfg(test)]
        None,
    )
}

fn run_codex_provider_text_with_lifecycle_reporting_usage_with_max_steps_and_replay(
    messages: &[ProviderMessage],
    typed_replay: Option<&[TypedReplayEntry]>,
    requested_model: Option<&str>,
    request_binding: Option<&CodexResponsesRequestBinding>,
    tools: &[RoutedToolDefinition],
    execute_tool: &mut dyn FnMut(
        &RoutedToolDefinition,
        Value,
        &str,
    ) -> Result<Value, ProviderSessionError>,
    on_text_delta: &mut dyn FnMut(&str, &str),
    should_cancel: &dyn Fn() -> bool,
    resume_from: Option<&CodexDirectCheckpoint>,
    on_checkpoint: &mut dyn FnMut(
        &CodexDirectCheckpoint,
    ) -> Result<(), ProviderSessionError>,
    on_usage: &mut dyn FnMut(ProviderTokenUsage),
    max_steps: usize,
    tool_step_reminder: Option<&ProviderToolStepReminderHook>,
    preserve_attempt_failure: bool,
    #[cfg(test)] after_credentials_prepared: Option<&dyn Fn()>,
) -> Result<String, ProviderSessionError> {
    if should_cancel() {
        return Err(ProviderSessionError::Cancelled(
            "Runner cancelled before provider dispatch".into(),
        ));
    }
    let model = resolve_codex_request_model(requested_model, resume_from)?;
    if resume_from.is_some()
        && !request_binding.is_some_and(CodexResponsesRequestBinding::is_initialized)
    {
        return Err(ProviderSessionError::Configuration(
            "Codex continuation requires its retained admitted request binding.".into(),
        ));
    }
    if should_cancel() {
        return Err(ProviderSessionError::Cancelled(
            "Runner cancelled before provider dispatch".into(),
        ));
    }
    let transport = new_codex_direct_transport(request_binding);
    #[cfg(test)]
    if let Some(observe) = after_credentials_prepared {
        observe();
    }
    // The Runner predicate includes watchdog/custody stops. Its existing outer
    // adapter and shell must see preparation errors before adjudicating those
    // signals; the shell separately gives actual user cancellation priority.
    // Ordinary low-level callers still give their cancellation predicate priority.
    if should_cancel() && (!preserve_attempt_failure || transport.is_ok()) {
        return Err(ProviderSessionError::Cancelled(
            "Runner cancelled before provider dispatch".into(),
        ));
    }
    let mut transport = transport?;
    let system_prompt = assembled_provider_system_prompt(messages);
    let input = responses_input_for_turn(
        messages,
        resume_from.is_none().then_some(typed_replay).flatten(),
    )?;
    let mut request = CodexDirectOptions::new(
        model,
        system_prompt,
        input,
    );
    request.reasoning_effort = configured_codex_reasoning_effort();
    request.max_steps = max_steps.max(1);
    request.tools = tools.iter().map(to_codex_tool).collect();

    let tool_index = tools
        .iter()
        .map(|tool| (tool.name.clone(), tool))
        .collect::<BTreeMap<_, _>>();
    let result = run_codex_direct_responses_with_lifecycle_and_tool_step_reminder(
        transport.as_mut(),
        &request,
        resume_from,
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
        &mut |checkpoint| {
            on_checkpoint(checkpoint)
                .map_err(|error| CodexDirectError::Transport(error.to_string()))
        },
        should_cancel,
        tool_step_reminder,
    )?;
    on_usage(result.usage.into());
    Ok(result.text)
}


fn run_fabushi_provider_text_with_lifecycle_reporting_usage_with_max_steps(
    messages: &[ProviderMessage],
    tools: &[RoutedToolDefinition],
    execute_tool: &mut dyn FnMut(
        &RoutedToolDefinition,
        Value,
        &str,
    ) -> Result<Value, ProviderSessionError>,
    on_text_delta: &mut dyn FnMut(&str, &str),
    should_cancel: &dyn Fn() -> bool,
    resume_from: Option<&CodexDirectCheckpoint>,
    on_checkpoint: &mut dyn FnMut(
        &CodexDirectCheckpoint,
    ) -> Result<(), ProviderSessionError>,
    on_usage: &mut dyn FnMut(ProviderTokenUsage),
    max_steps: usize,
    tool_step_reminder: Option<&ProviderToolStepReminderHook>,
) -> Result<String, ProviderSessionError> {
    run_fabushi_provider_text_with_lifecycle_reporting_usage_with_max_steps_and_replay(
        messages,
        None,
        None,
        None,
        tools,
        execute_tool,
        on_text_delta,
        should_cancel,
        resume_from,
        on_checkpoint,
        on_usage,
        max_steps,
        tool_step_reminder,
    )
}

fn run_fabushi_provider_text_with_lifecycle_reporting_usage_with_max_steps_and_replay(
    messages: &[ProviderMessage],
    typed_replay: Option<&[TypedReplayEntry]>,
    requested_model: Option<&str>,
    request_binding: Option<&FabushiResponsesRequestBinding>,
    tools: &[RoutedToolDefinition],
    execute_tool: &mut dyn FnMut(
        &RoutedToolDefinition,
        Value,
        &str,
    ) -> Result<Value, ProviderSessionError>,
    on_text_delta: &mut dyn FnMut(&str, &str),
    should_cancel: &dyn Fn() -> bool,
    resume_from: Option<&CodexDirectCheckpoint>,
    on_checkpoint: &mut dyn FnMut(
        &CodexDirectCheckpoint,
    ) -> Result<(), ProviderSessionError>,
    on_usage: &mut dyn FnMut(ProviderTokenUsage),
    max_steps: usize,
    tool_step_reminder: Option<&ProviderToolStepReminderHook>,
) -> Result<String, ProviderSessionError> {
    let model = resolve_fabushi_request_model(requested_model, resume_from)?;
    let mut transport = CodexHttpTransport::new_fabushi_with_binding(request_binding)?;
    let system_prompt = format!(
        "{}\n\nRuntime inference metadata: provider=Fabushi first-party Responses API; model={model}. If the user asks which provider or model is active, answer from this runtime metadata and do not infer a different vendor identity.",
        assembled_provider_system_prompt(messages),
    );
    let input = responses_input_for_turn(
        messages,
        resume_from.is_none().then_some(typed_replay).flatten(),
    )?;
    let mut request = CodexDirectOptions::new(
        model,
        system_prompt,
        input,
    );
    request.max_steps = max_steps.max(1);
    request.tools = tools.iter().map(to_codex_tool).collect();

    let tool_index = tools
        .iter()
        .map(|tool| (tool.name.clone(), tool))
        .collect::<BTreeMap<_, _>>();
    let result = run_codex_direct_responses_with_lifecycle_and_tool_step_reminder(
        &mut transport,
        &request,
        resume_from,
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
        &mut |checkpoint| {
            on_checkpoint(checkpoint)
                .map_err(|error| CodexDirectError::Transport(error.to_string()))
        },
        should_cancel,
        tool_step_reminder,
    )?;
    on_usage(result.usage.into());
    Ok(result.text)
}


pub struct RoutedProviderOptions<'a> {
    pub data_dir: &'a Path,
    pub typed_replay: Option<&'a [TypedReplayEntry]>,
    pub fabushi_model: Option<&'a str>,
    pub codex_model: Option<&'a str>,
    pub codex_request_binding: Option<&'a CodexResponsesRequestBinding>,
    pub fabushi_request_binding: Option<&'a FabushiResponsesRequestBinding>,
    pub cursor_auth: Option<Arc<dyn CursorInferenceAuth>>,
    pub tools: &'a [RoutedToolDefinition],
    pub mcp_server_url: Option<&'a str>,
    pub execute_tool: &'a mut dyn FnMut(
        &RoutedToolDefinition,
        Value,
        &str,
    ) -> Result<Value, ProviderSessionError>,
    pub on_text_delta: &'a mut dyn FnMut(&str, &str),
    pub on_partial_tool_call: Option<&'a mut dyn FnMut(ProviderPartialToolCall) -> Result<(), ProviderSessionError>>,
    pub should_cancel: &'a dyn Fn() -> bool,
}

pub fn run_routed_provider_text(
    provider: RoutedProvider,
    messages: &[ProviderMessage],
    options: &mut RoutedProviderOptions<'_>,
) -> Result<String, ProviderSessionError> {
    let mut ignore_checkpoint =
        |_checkpoint: &RoutedProviderCheckpoint| Ok(());
    run_routed_provider_text_with_lifecycle(
        provider,
        messages,
        options,
        None,
        &mut ignore_checkpoint,
    )
}

pub fn run_routed_provider_text_with_lifecycle(
    provider: RoutedProvider,
    messages: &[ProviderMessage],
    options: &mut RoutedProviderOptions<'_>,
    resume_from: Option<&RoutedProviderCheckpoint>,
    on_checkpoint: &mut dyn FnMut(
        &RoutedProviderCheckpoint,
    ) -> Result<(), ProviderSessionError>,
) -> Result<String, ProviderSessionError> {
    let mut ignore_usage = |_usage: ProviderTokenUsage| {};
    run_routed_provider_text_with_lifecycle_reporting_usage(
        provider,
        messages,
        options,
        resume_from,
        on_checkpoint,
        &mut ignore_usage,
    )
}

pub fn run_routed_provider_text_with_lifecycle_reporting_usage(
    provider: RoutedProvider,
    messages: &[ProviderMessage],
    options: &mut RoutedProviderOptions<'_>,
    resume_from: Option<&RoutedProviderCheckpoint>,
    on_checkpoint: &mut dyn FnMut(
        &RoutedProviderCheckpoint,
    ) -> Result<(), ProviderSessionError>,
    on_usage: &mut dyn FnMut(ProviderTokenUsage),
) -> Result<String, ProviderSessionError> {
    run_routed_provider_text_with_lifecycle_impl(
        provider,
        messages,
        options,
        resume_from,
        on_checkpoint,
        on_usage,
        8,
        None,
        false,
        #[cfg(test)]
        None,
    )
}

/// Runner attempt entry: `should_cancel` also includes watchdog and custody
/// signals. Preserve explicit permanent failures for the attempt owner, which
/// separately checks actual user cancellation before retry/terminal reporting.
pub fn run_routed_provider_text_with_lifecycle_reporting_usage_with_max_steps(
    provider: RoutedProvider,
    messages: &[ProviderMessage],
    options: &mut RoutedProviderOptions<'_>,
    resume_from: Option<&RoutedProviderCheckpoint>,
    on_checkpoint: &mut dyn FnMut(
        &RoutedProviderCheckpoint,
    ) -> Result<(), ProviderSessionError>,
    on_usage: &mut dyn FnMut(ProviderTokenUsage),
    max_steps: usize,
    tool_step_reminder: Option<&ProviderToolStepReminderHook>,
) -> Result<String, ProviderSessionError> {
    run_routed_provider_text_with_lifecycle_impl(
        provider, messages, options, resume_from, on_checkpoint, on_usage,
        max_steps, tool_step_reminder, true,
        #[cfg(test)]
        None,
    )
}

fn run_routed_provider_text_with_lifecycle_impl(
    provider: RoutedProvider,
    messages: &[ProviderMessage],
    options: &mut RoutedProviderOptions<'_>,
    resume_from: Option<&RoutedProviderCheckpoint>,
    on_checkpoint: &mut dyn FnMut(
        &RoutedProviderCheckpoint,
    ) -> Result<(), ProviderSessionError>,
    on_usage: &mut dyn FnMut(ProviderTokenUsage),
    max_steps: usize,
    tool_step_reminder: Option<&ProviderToolStepReminderHook>,
    preserve_attempt_failure: bool,
    #[cfg(test)] after_codex_credentials_prepared: Option<&dyn Fn()>,
) -> Result<String, ProviderSessionError> {
    let max_steps = max_steps.max(1);
    if options.typed_replay.is_some()
        && !matches!(
            provider,
            RoutedProvider::Cursor
                | RoutedProvider::Codex
                | RoutedProvider::Fabushi
                | RoutedProvider::OpenRouter
                | RoutedProvider::ClaudeCode
        )
    {
        return Err(ProviderSessionError::Protocol(
            "typed compacted replay reached provider serialization before provider-specific message merge was enabled".into(),
        ));
    }
    if (options.should_cancel)() {
        return Err(ProviderSessionError::Cancelled(
            "Runner cancelled before provider dispatch".into(),
        ));
    }
    if let Some(checkpoint) = resume_from {
        if checkpoint.provider() != provider {
            return Err(ProviderSessionError::Protocol(format!(
                "Runner checkpoint belongs to {} but {} was selected",
                checkpoint.provider().as_str(),
                provider.as_str()
            )));
        }
    }

    let result = match provider {
        RoutedProvider::Fabushi => {
            let resume = match resume_from {
                Some(RoutedProviderCheckpoint::Fabushi(checkpoint)) => Some(checkpoint),
                _ => None,
            };
            let mut fabushi_checkpoint = |checkpoint: &CodexDirectCheckpoint| {
                on_checkpoint(&RoutedProviderCheckpoint::Fabushi(checkpoint.clone()))
            };
            run_fabushi_provider_text_with_lifecycle_reporting_usage_with_max_steps_and_replay(
                messages,
                options.typed_replay,
                options.fabushi_model,
                options.fabushi_request_binding,
                options.tools,
                options.execute_tool,
                options.on_text_delta,
                options.should_cancel,
                resume,
                &mut fabushi_checkpoint,
                on_usage,
                max_steps,
                tool_step_reminder,
            )
        }
        RoutedProvider::Cursor => {
            let auth = options.cursor_auth.clone().ok_or_else(|| {
                ProviderSessionError::Configuration(
                    "Cursor inference requires the typed Host auth binding.".into(),
                )
            })?;
            let resume = match resume_from {
                Some(RoutedProviderCheckpoint::Cursor(checkpoint)) => Some(checkpoint),
                _ => None,
            };
            let transport = CursorInferenceTransport::new(Arc::clone(&auth));
            let mut cursor_checkpoint = |checkpoint: &CursorCheckpoint| {
                on_checkpoint(&RoutedProviderCheckpoint::Cursor(checkpoint.clone()))
            };
            let mut ignore_partial = |_partial: ProviderPartialToolCall| Ok(());
            let on_partial_tool_call = options
                .on_partial_tool_call
                .as_deref_mut()
                .unwrap_or(&mut ignore_partial);
            let initial_conversation = resume
                .is_none()
                .then_some(options.typed_replay)
                .flatten()
                .map(serialize_cursor_replay)
                .transpose()
                .map_err(|error| ProviderSessionError::Protocol(error.to_string()))?
                .map(|replay| {
                    let latest_user = messages
                        .iter()
                        .rev()
                        .find(|message| message.role == "user")
                        .cloned();
                    let mut merged = messages
                        .iter()
                        .filter(|message| message.role == "system")
                        .map(|message| CursorCheckpointMessage::Text {
                            role: message.role.clone(),
                            text: message.content.clone(),
                        })
                        .collect::<Vec<_>>();
                    merged.extend(replay);
                    if let Some(message) = latest_user {
                        merged.push(CursorCheckpointMessage::Text {
                            role: message.role,
                            text: message.content,
                        });
                    }
                    merged
                });
            let result = super::cursor_inference_transport::run_cursor_with_transport_reporting_usage_and_partials_with_initial_conversation(
                &transport,
                messages,
                initial_conversation.as_deref(),
                options.tools,
                options.execute_tool,
                options.on_text_delta,
                options.should_cancel,
                resume,
                &mut cursor_checkpoint,
                on_usage,
                on_partial_tool_call,
                max_steps,
                tool_step_reminder,
            );
            result
        }
        RoutedProvider::Codex => {
            let resume = match resume_from {
                Some(RoutedProviderCheckpoint::Codex(checkpoint)) => {
                    Some(checkpoint)
                }
                _ => None,
            };
            let mut codex_checkpoint = |checkpoint: &CodexDirectCheckpoint| {
                on_checkpoint(&RoutedProviderCheckpoint::Codex(
                    checkpoint.clone(),
                ))
            };
            run_codex_provider_text_with_lifecycle_reporting_usage_with_max_steps_and_replay(
                messages,
                options.typed_replay,
                options.codex_model,
                options.codex_request_binding,
                options.tools,
                options.execute_tool,
                options.on_text_delta,
                options.should_cancel,
                resume,
                &mut codex_checkpoint,
                on_usage,
                max_steps,
                tool_step_reminder,
                preserve_attempt_failure,
                #[cfg(test)]
                after_codex_credentials_prepared,
            )
        }
        RoutedProvider::OpenRouter => {
            let resume = match resume_from {
                Some(RoutedProviderCheckpoint::OpenRouter(checkpoint)) => {
                    Some(checkpoint)
                }
                _ => None,
            };
            let mut openrouter_checkpoint =
                |checkpoint: &OpenRouterCheckpoint| {
                    on_checkpoint(&RoutedProviderCheckpoint::OpenRouter(
                        checkpoint.clone(),
                    ))
                };
            run_openrouter_provider_text_with_lifecycle_reporting_usage(
                messages,
                options,
                resume,
                &mut openrouter_checkpoint,
                on_usage,
                max_steps,
                tool_step_reminder,
            )
        }
        RoutedProvider::ClaudeCode => {
            if resume_from.is_some() {
                Err(ProviderSessionError::Protocol(
                    "Claude Code does not emit a resumable tool-boundary checkpoint."
                        .into(),
                ))
            } else {
                run_claude_code_provider_text(messages, options)
            }
        }
    };
    if (options.should_cancel)() {
        if preserve_attempt_failure && matches!(&result, Err(
            ProviderSessionError::Authentication(_)
                | ProviderSessionError::Configuration(_)
                | ProviderSessionError::ToolOutcomeUnknown(_)
        )) {
            return result;
        }
        return Err(ProviderSessionError::Cancelled(
            "Runner cancelled the provider request".into(),
        ));
    }
    result
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
        "{system_prompt}\n\nContinue this Fabushi conversation.\n\n{rendered}"
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenRouterCheckpoint {
    pub conversation: Vec<Value>,
    pub text: String,
    pub completed_steps: usize,
    pub tool_calls_completed: usize,
}

pub trait OpenRouterTransport {
    fn stream_response(
        &mut self,
        request: &Value,
        on_event: &mut dyn FnMut(Value) -> Result<(), ProviderSessionError>,
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<(), ProviderSessionError>;
}

struct OpenRouterHttpTransport {
    client: AsyncClient,
    runtime: &'static Runtime,
    api_key: String,
}

impl OpenRouterHttpTransport {
    fn new(api_key: String) -> Result<Self, ProviderSessionError> {
        Ok(Self {
            client: AsyncClient::builder()
                .build()
                .map_err(|error| ProviderSessionError::Transport(error.to_string()))?,
            runtime: provider_io_runtime()?,
            api_key,
        })
    }
}

impl OpenRouterTransport for OpenRouterHttpTransport {
    fn stream_response(
        &mut self,
        request: &Value,
        on_event: &mut dyn FnMut(Value) -> Result<(), ProviderSessionError>,
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<(), ProviderSessionError> {
        let response = self.runtime.block_on(await_reqwest(
            self.client
                .post("https://openrouter.ai/api/v1/chat/completions")
                .header("authorization", format!("Bearer {}", self.api_key))
                .header("content-type", "application/json")
                .header("accept", "text/event-stream")
                .header(
                    "HTTP-Referer",
                    "https://github.com/bhrumom/fabushi-desktop",
                )
                .header("X-Title", "Fabushi")
                .json(request)
                .send(),
            should_cancel,
            "the OpenRouter provider request",
        ))?;
        if !response.status().is_success() {
            let status = response.status();
            let detail = self
                .runtime
                .block_on(await_reqwest(
                    response.text(),
                    should_cancel,
                    "the OpenRouter provider error response",
                ))?
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
        self.runtime.block_on(decode_async_sse_stream(
            response,
            should_cancel,
            on_event,
        ))
    }
}

pub fn run_openrouter_with_transport(
    transport: &mut dyn OpenRouterTransport,
    model: &str,
    messages: &[ProviderMessage],
    tools: &[RoutedToolDefinition],
    execute_tool: &mut dyn FnMut(
        &RoutedToolDefinition,
        Value,
        &str,
    ) -> Result<Value, ProviderSessionError>,
    on_text_delta: &mut dyn FnMut(&str, &str),
    should_cancel: &dyn Fn() -> bool,
    resume_from: Option<&OpenRouterCheckpoint>,
    on_checkpoint: &mut dyn FnMut(
        &OpenRouterCheckpoint,
    ) -> Result<(), ProviderSessionError>,
) -> Result<String, ProviderSessionError> {
    let mut ignore_usage = |_usage: ProviderTokenUsage| {};
    run_openrouter_with_transport_reporting_usage(
        transport,
        model,
        messages,
        tools,
        execute_tool,
        on_text_delta,
        should_cancel,
        resume_from,
        on_checkpoint,
        &mut ignore_usage,
    )
}

pub fn run_openrouter_with_transport_reporting_usage(
    transport: &mut dyn OpenRouterTransport,
    model: &str,
    messages: &[ProviderMessage],
    tools: &[RoutedToolDefinition],
    execute_tool: &mut dyn FnMut(
        &RoutedToolDefinition,
        Value,
        &str,
    ) -> Result<Value, ProviderSessionError>,
    on_text_delta: &mut dyn FnMut(&str, &str),
    should_cancel: &dyn Fn() -> bool,
    resume_from: Option<&OpenRouterCheckpoint>,
    on_checkpoint: &mut dyn FnMut(
        &OpenRouterCheckpoint,
    ) -> Result<(), ProviderSessionError>,
    on_usage: &mut dyn FnMut(ProviderTokenUsage),
) -> Result<String, ProviderSessionError> {
    let mut ignore_partial = |_partial: ProviderPartialToolCall| Ok(());
    run_openrouter_with_transport_reporting_usage_and_partials(
        transport, model, messages, tools, execute_tool, on_text_delta,
        should_cancel, resume_from, on_checkpoint, on_usage, &mut ignore_partial,
    )
}

pub fn run_openrouter_with_transport_reporting_usage_and_partials(
    transport: &mut dyn OpenRouterTransport,
    model: &str,
    messages: &[ProviderMessage],
    tools: &[RoutedToolDefinition],
    execute_tool: &mut dyn FnMut(
        &RoutedToolDefinition,
        Value,
        &str,
    ) -> Result<Value, ProviderSessionError>,
    on_text_delta: &mut dyn FnMut(&str, &str),
    should_cancel: &dyn Fn() -> bool,
    resume_from: Option<&OpenRouterCheckpoint>,
    on_checkpoint: &mut dyn FnMut(
        &OpenRouterCheckpoint,
    ) -> Result<(), ProviderSessionError>,
    on_usage: &mut dyn FnMut(ProviderTokenUsage),
    on_partial_tool_call: &mut dyn FnMut(ProviderPartialToolCall) -> Result<(), ProviderSessionError>,
) -> Result<String, ProviderSessionError> {
    run_openrouter_with_transport_reporting_usage_and_partials_with_max_steps(
        transport,
        model,
        messages,
        tools,
        execute_tool,
        on_text_delta,
        should_cancel,
        resume_from,
        on_checkpoint,
        on_usage,
        on_partial_tool_call,
        8,
        None,
    )
}

pub fn run_openrouter_with_transport_reporting_usage_and_partials_with_max_steps(
    transport: &mut dyn OpenRouterTransport,
    model: &str,
    messages: &[ProviderMessage],
    tools: &[RoutedToolDefinition],
    execute_tool: &mut dyn FnMut(
        &RoutedToolDefinition,
        Value,
        &str,
    ) -> Result<Value, ProviderSessionError>,
    on_text_delta: &mut dyn FnMut(&str, &str),
    should_cancel: &dyn Fn() -> bool,
    resume_from: Option<&OpenRouterCheckpoint>,
    on_checkpoint: &mut dyn FnMut(
        &OpenRouterCheckpoint,
    ) -> Result<(), ProviderSessionError>,
    on_usage: &mut dyn FnMut(ProviderTokenUsage),
    on_partial_tool_call: &mut dyn FnMut(ProviderPartialToolCall) -> Result<(), ProviderSessionError>,
    max_steps: usize,
    tool_step_reminder: Option<&ProviderToolStepReminderHook>,
) -> Result<String, ProviderSessionError> {
    run_openrouter_with_transport_reporting_usage_and_partials_with_initial_conversation(
        transport,
        model,
        messages,
        None,
        tools,
        execute_tool,
        on_text_delta,
        should_cancel,
        resume_from,
        on_checkpoint,
        on_usage,
        on_partial_tool_call,
        max_steps,
        tool_step_reminder,
    )
}

pub fn run_openrouter_with_transport_reporting_usage_and_partials_with_initial_conversation(
    transport: &mut dyn OpenRouterTransport,
    model: &str,
    messages: &[ProviderMessage],
    initial_conversation: Option<&[Value]>,
    tools: &[RoutedToolDefinition],
    execute_tool: &mut dyn FnMut(
        &RoutedToolDefinition,
        Value,
        &str,
    ) -> Result<Value, ProviderSessionError>,
    on_text_delta: &mut dyn FnMut(&str, &str),
    should_cancel: &dyn Fn() -> bool,
    resume_from: Option<&OpenRouterCheckpoint>,
    on_checkpoint: &mut dyn FnMut(
        &OpenRouterCheckpoint,
    ) -> Result<(), ProviderSessionError>,
    on_usage: &mut dyn FnMut(ProviderTokenUsage),
    on_partial_tool_call: &mut dyn FnMut(ProviderPartialToolCall) -> Result<(), ProviderSessionError>,
    max_steps: usize,
    tool_step_reminder: Option<&ProviderToolStepReminderHook>,
) -> Result<String, ProviderSessionError> {
    let max_steps = max_steps.max(1);
    let tool_index = tools
        .iter()
        .map(|tool| (tool.name.clone(), tool))
        .collect::<BTreeMap<_, _>>();
    let declared_tools = openrouter_tools(tools);
    let (
        mut conversation,
        mut text,
        mut completed_steps,
        mut tool_calls_completed,
    ) = match resume_from {
        Some(checkpoint) => (
            checkpoint.conversation.clone(),
            checkpoint.text.clone(),
            checkpoint.completed_steps,
            checkpoint.tool_calls_completed,
        ),
        None => {
            let system_prompt = assembled_provider_system_prompt(messages);
            let mut conversation = vec![json!({
                "role": "system",
                "content": system_prompt
            })];
            if let Some(initial_conversation) = initial_conversation {
                conversation.extend(initial_conversation.iter().cloned());
            } else {
                conversation.extend(
                    messages
                        .iter()
                        .filter(|message| message.role != "system")
                        .map(|message| {
                            json!({
                                "role": if message.role == "assistant" {
                                    "assistant"
                                } else {
                                    "user"
                                },
                                "content": message.content,
                            })
                        }),
                );
            }
            (conversation, String::new(), 0, 0)
        }
    };

    if completed_steps >= max_steps {
        return Err(ProviderSessionError::Protocol(format!(
            "OpenRouter resume checkpoint already exhausted Fabushi's {max_steps}-step tool limit."
        )));
    }

    for _step in completed_steps..max_steps {
        if should_cancel() {
            return Err(ProviderSessionError::Cancelled(
                "Runner cancelled the OpenRouter request".into(),
            ));
        }
        let mut request = json!({
            "model": model,
            "messages": conversation,
            "stream": true,
            "usage": { "include": true },
        });
        if !declared_tools.is_empty() {
            request["tools"] = Value::Array(declared_tools.clone());
            request["tool_choice"] = Value::String("auto".into());
        }

        let mut partial_calls =
            BTreeMap::<usize, PartialOpenRouterToolCall>::new();
        let mut step_text = String::new();
        let mut step_usage = None::<ProviderTokenUsage>;
        let mut step_model_call_id = String::new();
        transport.stream_response(
            &request,
            &mut |event| {
                if should_cancel() {
                    return Err(ProviderSessionError::Cancelled(
                        "Runner cancelled the OpenRouter stream".into(),
                    ));
                }
                if let Some(error) = event.get("error") {
                    return Err(ProviderSessionError::Protocol(format!(
                        "OpenRouter stream failed: {error}"
                    )));
                }
                if let Some(id) = event.get("id").and_then(Value::as_str).filter(|id| !id.is_empty()) {
                    step_model_call_id = id.to_string();
                }
                if let Some(usage) = provider_token_usage_from_openrouter_event(&event) {
                    step_usage = Some(usage);
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
                if let Some(content) =
                    delta.get("content").and_then(Value::as_str)
                {
                    step_text.push_str(content);
                    text.push_str(content);
                    on_text_delta(content, &text);
                }
                if let Some(calls) =
                    delta.get("tool_calls").and_then(Value::as_array)
                {
                    for call in calls {
                        let index = call
                            .get("index")
                            .and_then(Value::as_u64)
                            .unwrap_or(0) as usize;
                        let partial =
                            partial_calls.entry(index).or_default();
                        if let Some(id) =
                            call.get("id").and_then(Value::as_str)
                        {
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
                        if !partial.id.is_empty() && !partial.name.is_empty() {
                            if let Some(tool) = tool_index.get(&partial.name).copied() {
                                on_partial_tool_call(ProviderPartialToolCall {
                                    tool: tool.clone(),
                                    tool_call_id: partial.id.clone(),
                                    raw_arguments: partial.arguments.clone(),
                                    model_call_id: (!step_model_call_id.is_empty())
                                        .then(|| step_model_call_id.clone()),
                                })?;
                            }
                        }
                    }
                }
                Ok(())
            },
            should_cancel,
        )?;
        if let Some(usage) = step_usage {
            on_usage(usage);
        }

        if partial_calls.is_empty() {
            return Ok(text);
        }

        let calls_in_step = partial_calls.len();
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

        let mut observed_tool_calls = Vec::with_capacity(calls_in_step);
        for (_index, call) in partial_calls {
            if should_cancel() {
                return Err(ProviderSessionError::Cancelled(
                    "Runner cancelled before OpenRouter tool execution".into(),
                ));
            }
            let tool_call_id = if call.id.is_empty() {
                Uuid::new_v4().to_string()
            } else {
                call.id
            };
            let args = serde_json::from_str::<Value>(&call.arguments)
                .unwrap_or_else(|_| json!({}));
            observed_tool_calls.push((call.name.clone(), args.clone()));
            let result = match tool_index.get(&call.name).copied() {
                None => json!({
                    "isError": true,
                    "error": format!("Unknown Fabushi tool: {}", call.name)
                }),
                Some(tool) => match execute_tool(tool, args, &tool_call_id) {
                    Ok(value) => value,
                    Err(error) => json!({
                        "isError": true,
                        "error": error.to_string()
                    }),
                },
            };
            conversation.push(json!({
                "role": "tool",
                "tool_call_id": tool_call_id,
                "content": serde_json::to_string(&result)
                    .unwrap_or_else(|_| "null".into()),
            }));
        }
        if let Some(reminder) =
            tool_step_reminder.and_then(|hook| hook(&observed_tool_calls))
        {
            conversation.push(json!({
                "role": "user",
                "content": reminder,
            }));
        }

        completed_steps = completed_steps.saturating_add(1);
        tool_calls_completed =
            tool_calls_completed.saturating_add(calls_in_step);
        on_checkpoint(&OpenRouterCheckpoint {
            conversation: conversation.clone(),
            text: text.clone(),
            completed_steps,
            tool_calls_completed,
        })?;
    }

    Err(ProviderSessionError::Protocol(format!(
        "OpenRouter exceeded Fabushi's {max_steps}-step tool limit."
    )))
}

fn run_openrouter_provider_text(
    messages: &[ProviderMessage],
    options: &mut RoutedProviderOptions<'_>,
) -> Result<String, ProviderSessionError> {
    let mut ignore_checkpoint =
        |_checkpoint: &OpenRouterCheckpoint| Ok(());
    run_openrouter_provider_text_with_lifecycle(
        messages,
        options,
        None,
        &mut ignore_checkpoint,
    )
}

fn run_openrouter_provider_text_with_lifecycle(
    messages: &[ProviderMessage],
    options: &mut RoutedProviderOptions<'_>,
    resume_from: Option<&OpenRouterCheckpoint>,
    on_checkpoint: &mut dyn FnMut(
        &OpenRouterCheckpoint,
    ) -> Result<(), ProviderSessionError>,
) -> Result<String, ProviderSessionError> {
    let mut ignore_usage = |_usage: ProviderTokenUsage| {};
    run_openrouter_provider_text_with_lifecycle_reporting_usage(
        messages,
        options,
        resume_from,
        on_checkpoint,
        &mut ignore_usage,
        8,
        None,
    )
}

fn run_openrouter_provider_text_with_lifecycle_reporting_usage(
    messages: &[ProviderMessage],
    options: &mut RoutedProviderOptions<'_>,
    resume_from: Option<&OpenRouterCheckpoint>,
    on_checkpoint: &mut dyn FnMut(
        &OpenRouterCheckpoint,
    ) -> Result<(), ProviderSessionError>,
    on_usage: &mut dyn FnMut(ProviderTokenUsage),
    max_steps: usize,
    tool_step_reminder: Option<&ProviderToolStepReminderHook>,
) -> Result<String, ProviderSessionError> {
    let api_key = openrouter_api_key(options.data_dir)?;
    let model = env::var("SAND_OPENROUTER_MODEL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "openai/gpt-5.2".into());
    let tools = options.tools;
    let should_cancel = options.should_cancel;
    let execute_tool = &mut *options.execute_tool;
    let on_text_delta = &mut *options.on_text_delta;
    let mut transport = OpenRouterHttpTransport::new(api_key)?;
    let mut ignore_partial = |_partial: ProviderPartialToolCall| Ok(());
    let on_partial_tool_call = options
        .on_partial_tool_call
        .as_deref_mut()
        .unwrap_or(&mut ignore_partial);
    let initial_conversation = options
        .typed_replay
        .filter(|_| resume_from.is_none())
        .map(serialize_chat_replay)
        .transpose()
        .map_err(|error| ProviderSessionError::Protocol(error.to_string()))?
        .map(|mut replay| {
            let message = messages
                .iter()
                .rev()
                .find(|message| message.role == "user")
                .ok_or_else(|| {
                    ProviderSessionError::Protocol(
                        "typed compacted replay requires a current user message".into(),
                    )
                })?;
            replay.push(json!({
                "role": "user",
                "content": message.content,
            }));
            Ok::<_, ProviderSessionError>(replay)
        })
        .transpose()?;
    run_openrouter_with_transport_reporting_usage_and_partials_with_initial_conversation(
        &mut transport,
        &model,
        messages,
        initial_conversation.as_deref(),
        tools,
        execute_tool,
        on_text_delta,
        should_cancel,
        resume_from,
        on_checkpoint,
        on_usage,
        on_partial_tool_call,
        max_steps,
        tool_step_reminder,
    )
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
    let prompt = claude_prompt_for_turn(messages, options.typed_replay)?;
    let executable = resolve_claude_cli_path().ok_or_else(|| {
        ProviderSessionError::Configuration(
            "Claude Code is not installed. Install and sign in to Claude Code, then reopen Fabushi."
                .into(),
        )
    })?;
    let mut command = Command::new(executable);
    command
        .arg("-p")
        .arg(prompt)
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
                    "fabushi_plugins": {
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
            .arg("mcp__fabushi_plugins__*");
        mcp_config_path = Some(path);
    }
    if let Some(model) = env::var("SAND_CLAUDE_MODEL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        command.arg("--model").arg(model);
    }

    if (options.should_cancel)() {
        if let Some(path) = mcp_config_path {
            let _ = fs::remove_file(path);
        }
        return Err(ProviderSessionError::Cancelled(
            "Runner cancelled before Claude Code dispatch".into(),
        ));
    }

    let stdout_path = env::temp_dir().join(format!(
        "fabushi-claude-stdout-{}.json",
        Uuid::new_v4()
    ));
    let stderr_path = env::temp_dir().join(format!(
        "fabushi-claude-stderr-{}.txt",
        Uuid::new_v4()
    ));
    let stdout_file = fs::File::create(&stdout_path)
        .map_err(|error| ProviderSessionError::Transport(error.to_string()))?;
    let stderr_file = fs::File::create(&stderr_path)
        .map_err(|error| ProviderSessionError::Transport(error.to_string()))?;
    command
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file));

    let mut child = command.spawn().map_err(|error| {
        ProviderSessionError::Transport(format!(
            "Could not run Claude Code: {error}"
        ))
    })?;
    let status = loop {
        if (options.should_cancel)() {
            let _ = child.kill();
            let _ = child.wait();
            if let Some(path) = mcp_config_path.as_ref() {
                let _ = fs::remove_file(path);
            }
            let _ = fs::remove_file(&stdout_path);
            let _ = fs::remove_file(&stderr_path);
            return Err(ProviderSessionError::Cancelled(
                "Runner cancelled Claude Code".into(),
            ));
        }
        match child
            .try_wait()
            .map_err(|error| ProviderSessionError::Transport(error.to_string()))?
        {
            Some(status) => break status,
            None => thread::sleep(Duration::from_millis(25)),
        }
    };

    if let Some(path) = mcp_config_path {
        let _ = fs::remove_file(path);
    }
    let stdout_bytes = fs::read(&stdout_path)
        .map_err(|error| ProviderSessionError::Transport(error.to_string()))?;
    let stderr_bytes = fs::read(&stderr_path)
        .map_err(|error| ProviderSessionError::Transport(error.to_string()))?;
    let _ = fs::remove_file(&stdout_path);
    let _ = fs::remove_file(&stderr_path);
    if !status.success() {
        return Err(ProviderSessionError::Transport(format!(
            "Claude Code failed: {}",
            String::from_utf8_lossy(&stderr_bytes).trim()
        )));
    }
    let stdout = String::from_utf8(stdout_bytes)
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

fn claude_prompt_for_turn(
    messages: &[ProviderMessage],
    typed_replay: Option<&[TypedReplayEntry]>,
) -> Result<String, ProviderSessionError> {
    let Some(entries) = typed_replay else {
        return Ok(provider_prompt(messages));
    };
    let replay = super::typed_replay_serialization::serialize_claude_replay(entries)
        .map_err(|error| ProviderSessionError::Protocol(error.to_string()))?;
    let current_user = messages
        .iter()
        .rev()
        .find(|message| message.role == "user")
        .ok_or_else(|| {
            ProviderSessionError::Protocol(
                "typed compacted replay requires a current user message".into(),
            )
        })?;
    Ok(format!(
        "{}\n\nContinue this Fabushi conversation.\n\n{}\n\nUSER: {}",
        assembled_provider_system_prompt(messages),
        replay,
        current_user.content
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        CI_ISOLATED_FABUSHI_NATIVE_ISSUER, DEFAULT_FABUSHI_NATIVE_ISSUER,
        FABUSHI_ROUTER_SYSTEM_PROMPT, RoutedProvider, await_cancelable_provider_io,
        codex_provider_error, ensure_responses_issuer_matches, jwt_audience,
        parse_fabushi_core_credential, ProviderSessionError, ResponsesHttpCredentials,
        provider_io_runtime, responses_http_client,
        toml_string_setting, validate_fabushi_responses_url,
    };
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use std::cell::Cell;
    use std::future::pending;

    #[test]
    fn fabushi_router_prompt_declares_product_identity() {
        assert!(FABUSHI_ROUTER_SYSTEM_PROMPT.contains("You are Fabushi"));
        assert!(FABUSHI_ROUTER_SYSTEM_PROMPT.contains("Fabushi's already-connected plugins"));
    }

    #[test]
    fn fabushi_inference_rejects_cross_issuer_targets_before_bearer_dispatch() {
        let production = validate_fabushi_responses_url("https://api.ombhrum.com/responses").unwrap();
        let isolated = validate_fabushi_responses_url(CI_ISOLATED_FABUSHI_NATIVE_ISSUER).unwrap();
        let mut document = serde_json::json!({
            "version": 1, "accessToken": "private-access-token", "expiresAtMs": 100_000,
            "sessionId": "session-a", "deviceId": "device-a", "userId": 42
        });
        let default = parse_fabushi_core_credential(&document.to_string(), || 0).unwrap();
        assert_eq!(default.issuer(), DEFAULT_FABUSHI_NATIVE_ISSUER);
        document["issuer"] = serde_json::json!(CI_ISOLATED_FABUSHI_NATIVE_ISSUER);
        assert_eq!(
            parse_fabushi_core_credential(&document.to_string(), || 0).unwrap().issuer(),
            CI_ISOLATED_FABUSHI_NATIVE_ISSUER,
        );
        assert!(ensure_responses_issuer_matches(&production, DEFAULT_FABUSHI_NATIVE_ISSUER).is_ok());
        assert!(ensure_responses_issuer_matches(&isolated, CI_ISOLATED_FABUSHI_NATIVE_ISSUER).is_ok());
        assert!(ensure_responses_issuer_matches(&production, CI_ISOLATED_FABUSHI_NATIVE_ISSUER).is_err());
        assert!(ensure_responses_issuer_matches(&isolated, DEFAULT_FABUSHI_NATIVE_ISSUER).is_err());
        assert!(validate_fabushi_responses_url("https://untrusted.example/responses").is_err());
        document["issuer"] = serde_json::json!("https://untrusted.example");
        assert!(parse_fabushi_core_credential(&document.to_string(), || 0).is_err());
    }

    #[test]
    fn fabushi_credential_debug_redacts_secret_and_permanent_errors_keep_their_types() {
        let raw = serde_json::json!({
            "version": 1, "accessToken": "private-access-token", "expiresAtMs": 100_000,
            "sessionId": "session-a", "deviceId": "device-a", "userId": 42
        }).to_string();
        let credential = parse_fabushi_core_credential(&raw, || 0).unwrap();
        let context = credential.context();
        let credentials = ResponsesHttpCredentials::Fabushi {
            credential, endpoint: "https://api.ombhrum.com/v1/responses".into(),
            credential_path: "credential.json".into(),
        };
        let rendered = format!("{credentials:?} {context:?}");
        assert!(rendered.contains("<redacted>"));
        assert!(!rendered.contains("private-access-token"));
        assert!(!rendered.contains(&raw));
        let chatgpt = ResponsesHttpCredentials::ChatGpt(super::CodexCredentials {
            access_token: "private-access-token".into(), refresh_token: "private-refresh-token".into(),
            id_token: "private-id-token".into(), account_id: "test-account".into(),
            path: "auth.json".into(), document: serde_json::json!({"private-document": "private-refresh-token"}),
        });
        let rendered = format!("{chatgpt:?}");
        for secret in ["private-access-token", "private-refresh-token", "private-id-token", "private-document"] {
            assert!(!rendered.contains(secret));
        }
        for error in [
            ProviderSessionError::Authentication("typed timeout-shaped auth failure".into()),
            ProviderSessionError::Configuration("typed connection-shaped config failure".into()),
        ] {
            let original = std::mem::discriminant(&error);
            let message = error.to_string();
            let round_trip = ProviderSessionError::from(codex_provider_error(error));
            assert_eq!(std::mem::discriminant(&round_trip), original);
            assert_eq!(round_trip.to_string(), message);
        }
    }

    #[test]
    fn parses_reference_provider_names() {
        assert_eq!(RoutedProvider::parse("fabushi"), Some(RoutedProvider::Fabushi));
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
    #[test]
    fn provider_io_runtime_is_process_shared() {
        let first = provider_io_runtime().expect("first provider I/O runtime");
        let second = provider_io_runtime().expect("second provider I/O runtime");
        assert!(
            std::ptr::eq(first, second),
            "provider turns must reuse one process-owned Tokio runtime so HTTP keep-alive survives between turns"
        );
    }

    #[test]
    fn responses_http_client_is_process_shared() {
        let first = responses_http_client().expect("first Responses HTTP client");
        let second = responses_http_client().expect("second Responses HTTP client");
        assert!(
            std::ptr::eq(first, second),
            "provider turns must reuse one process-owned HTTP client and connection pool"
        );
    }

    #[test]
    fn cancellable_provider_io_returns_while_the_io_future_is_silent() {
        let checks = Cell::new(0_u32);
        let should_cancel = || {
            let seen = checks.get();
            checks.set(seen.saturating_add(1));
            seen >= 1
        };
        let runtime = provider_io_runtime().expect("provider I/O runtime");
        let error = runtime
            .block_on(await_cancelable_provider_io(
                pending::<()>(),
                &should_cancel,
                "a silent provider probe",
            ))
            .expect_err("silent I/O must be interruptible by Runner cancellation");
        assert!(matches!(error, super::ProviderSessionError::Cancelled(_)));
        assert!(
            checks.get() >= 2,
            "cancellation must be re-checked without requiring provider output"
        );
    }

}

#[cfg(test)]
mod fabushi_credential_context_tests {
    use super::*;
    use std::io::{self, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::{Mutex, atomic::{AtomicBool, Ordering}};
    use std::time::Instant;

    const TEST_NAME: &str = "extensions::inference::provider_session::fabushi_credential_context_tests::captured_endpoint_and_context_survive_real_http_refresh";
    const CHILD_ROOT: &str = "FABUSHI_BOUND_TRANSPORT_CHILD_ROOT";
    const NEXT_ENDPOINT: &str = "FABUSHI_BOUND_TRANSPORT_NEXT_ENDPOINT";
    const NONCE: &str = "FABUSHI_BOUND_TRANSPORT_NONCE";
    const TOKEN_A: &str = "private-bound-transport-token-a";
    const ROTATED: &str = "private-bound-transport-token-a-rotated";
    const TOKEN_B: &str = "private-bound-transport-token-b";
    const BOUND: Duration = Duration::from_secs(20);

    fn fixture_io_error(stage: &str, error: io::Error) -> String {
        format!("{stage}: kind={:?}, os={:?}", error.kind(), error.raw_os_error())
    }

    fn captured_output_observation(output: &std::io::Result<String>) -> Value {
        match output {
            Ok(value) => {
                let panic_location = value.lines().find_map(|line| {
                    let (_, location) = line.split_once(
                        "panicked at src/extensions/inference/provider_session.rs:",
                    )?;
                    let mut numbers = location.split(':');
                    let line = numbers.next()?.trim().parse::<u32>().ok()?;
                    let column = numbers.next()?.trim().parse::<u32>().ok()?;
                    Some(json!({"line": line, "column": column}))
                });
                json!({"bytes": value.len(), "panic": panic_location})
            }
            Err(error) => json!({
                "errorKind": format!("{:?}", error.kind()),
                "osError": error.raw_os_error(),
            }),
        }
    }

    #[test]
    fn captured_output_diagnostics_preserve_status_without_credential_values() {
        let output = format!("thread 'fixture' panicked at src/extensions/inference/provider_session.rs:7:11:\nAuthorization: Bearer {TOKEN_A}; body={ROTATED}; assertion={TOKEN_B}");
        let observation = captured_output_observation(&Ok(output.clone()));
        assert_eq!(observation["bytes"], output.len());
        assert_eq!(observation["panic"]["line"], 7);
        assert_eq!(observation["panic"]["column"], 11);
        let failed_read = captured_output_observation(&Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied, output,
        )));
        assert_eq!(failed_read["errorKind"], "PermissionDenied");
        for rendered in [observation.to_string(), failed_read.to_string()] {
            for token in [TOKEN_A, ROTATED, TOKEN_B] {
                assert!(!rendered.contains(token), "credential value escaped diagnostic projection");
            }
            assert!(!rendered.contains("Authorization"), "raw output escaped diagnostic projection");
        }
    }

    fn write_credential(path: &Path, issuer: &str, user: &str, token: &str) -> Result<(), String> {
        let temporary = path.with_extension(format!("{}.tmp", Uuid::new_v4()));
        let mut options = fs::OpenOptions::new();
        options.create_new(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        options.open(&temporary).map_err(|error| fixture_io_error("replacement-create", error))?.write_all(json!({
            "version": 1, "issuer": issuer, "accessToken": token,
            "expiresAtMs": 4_102_444_800_000_u64,
            "userId": user, "deviceId": "bound-device", "sessionId": "bound-session"
        }).to_string().as_bytes()).map_err(|error| fixture_io_error("replacement-write", error))?;
        fs::rename(temporary, path).map_err(|error| fixture_io_error("replacement-rename", error))
    }

    type Request = (bool, String, String, Vec<u8>);

    fn read_request(stream: &mut TcpStream, next_endpoint: bool) -> Result<Request, String> {
        // Match the bounded integration fixture on platforms inheriting O_NONBLOCK.
        stream.set_nonblocking(false).map_err(|error| fixture_io_error("request-blocking", error))?;
        stream.set_read_timeout(Some(Duration::from_secs(5))).map_err(|error| fixture_io_error("request-read-timeout", error))?;
        stream.set_write_timeout(Some(Duration::from_secs(5))).map_err(|error| fixture_io_error("request-write-timeout", error))?;
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).map_err(|error| fixture_io_error("request-line", error))?;
        let request_line = line.trim_end().to_string();
        let mut length = None;
        let mut authorization = String::new();
        loop {
            line.clear();
            if reader.read_line(&mut line).map_err(|error| fixture_io_error("request-header", error))? == 0 {
                return Err("request headers ended early".into());
            }
            if line == "\r\n" { break; }
            let (name, value) = line.split_once(':').ok_or("invalid request header")?;
            if name.eq_ignore_ascii_case("content-length") {
                length = Some(value.trim().parse::<usize>().map_err(|_| "invalid request content-length")?);
            }
            if name.eq_ignore_ascii_case("authorization") { authorization = value.trim().into(); }
        }
        let length = length.ok_or("missing request content-length")?;
        if length >= 1024 * 1024 { return Err("oversized fixture request".into()); }
        let mut body = vec![0; length];
        reader.read_exact(&mut body).map_err(|error| fixture_io_error("request-body", error))?;
        Ok((next_endpoint, request_line, authorization, body))
    }

    #[cfg(unix)]
    #[test]
    fn bound_transport_reader_clears_forced_nonblocking_and_preserves_request() {
        use std::os::fd::AsRawFd;
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        client.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
        let (mut stream, _) = listener.accept().unwrap();
        stream.set_nonblocking(true).unwrap();
        let before = unsafe { libc::fcntl(stream.as_raw_fd(), libc::F_GETFL) };
        assert!(before >= 0);
        assert_ne!(before & libc::O_NONBLOCK, 0);
        let body = br#"{"model":"Fixture/Case@Bound"}"#;
        client.write_all(format!("POST /captured/v1/responses HTTP/1.1\r\nAuthorization: fixture-marker\r\nContent-Length: {}\r\n\r\n", body.len()).as_bytes()).unwrap();
        client.write_all(body).unwrap();
        let (next, line, authorization, actual_body) = read_request(&mut stream, true)
            .expect("bounded private fixture reader");
        let after = unsafe { libc::fcntl(stream.as_raw_fd(), libc::F_GETFL) };
        assert!(after >= 0);
        assert_eq!(after & libc::O_NONBLOCK, 0, "actual reader must restore blocking I/O");
        assert_eq!(stream.read_timeout().unwrap(), Some(Duration::from_secs(5)));
        assert_eq!(stream.write_timeout().unwrap(), Some(Duration::from_secs(5)));
        assert!(next);
        assert_eq!(line, "POST /captured/v1/responses HTTP/1.1");
        assert_eq!(authorization, "fixture-marker");
        assert_eq!(actual_body, body.to_vec());
    }

    fn write_child_receipt(root: &Path, receipt: &Value) {
        let temporary = root.join("receipt.tmp");
        fs::write(&temporary, receipt.to_string())
            .map_err(|error| fixture_io_error("child-receipt-write", error)).expect("record child phase");
        fs::rename(temporary, root.join("receipt"))
            .map_err(|error| fixture_io_error("child-receipt-rename", error)).expect("publish child phase");
    }

    fn child_phase(root: &Path, receipt: &mut Value, phase: &str) {
        receipt["phase"] = json!(phase);
        write_child_receipt(root, receipt);
    }

    fn record_transport_result(root: &Path, receipt: &mut Value, phase: &str, result: &Result<(), CodexDirectError>, events: usize) {
        let kind = match result {
            Ok(()) => "success",
            Err(CodexDirectError::Authentication(_)) => "authentication",
            Err(CodexDirectError::Configuration(_)) => "configuration",
            Err(CodexDirectError::Transport(_)) => "transport",
            Err(CodexDirectError::Protocol(_)) => "protocol",
            Err(CodexDirectError::Tool(_)) => "tool",
            Err(CodexDirectError::ProviderRefusal(_)) => "provider-refusal",
            Err(CodexDirectError::Cancelled(_)) => "cancelled",
        };
        receipt["observations"].as_array_mut().expect("receipt observations")
            .push(json!({"phase":phase, "kind":kind, "events":events}));
        child_phase(root, receipt, &format!("{phase}-observed"));
    }

    fn child(root: &Path) {
        let mut receipt = json!({
            "nonce":env::var(NONCE).unwrap(), "case":"captured-endpoint-and-context",
            "phase":"capture-credentials", "completed":false, "observations":[],
        });
        write_child_receipt(root, &receipt);
        // Capture through the actual configuration reader before changing the
        // environment. This exact-test child has not started any I/O workers;
        // the parent owns both HTTP servers. No environment mutation occurs
        // after creating the transport/runtime or while requests run.
        let captured = configured_fabushi_responses_credentials().unwrap();
        let (original_context, original_endpoint, path) = match &captured {
            ResponsesHttpCredentials::Fabushi { credential, endpoint, credential_path } =>
                (credential.context(), endpoint.clone(), credential_path.clone()),
            _ => panic!("expected Native credentials"),
        };
        let next_endpoint = env::var(NEXT_ENDPOINT).unwrap();
        unsafe { env::set_var(FABUSHI_RESPONSES_URL_ENV, &next_endpoint); }
        let mut transport = CodexHttpTransport::with_credentials(captured).unwrap();
        let request = json!({"model":"Vendor/Case@Bound", "input":[{"role":"user","content":"bound request"}], "stream":true});
        let mut events = Vec::new();
        child_phase(root, &mut receipt, "same-context-refresh-running");
        let result = transport.stream_response(&request, &mut |event| { events.push(event); Ok(()) }, &|| false);
        record_transport_result(root, &mut receipt, "same-context-refresh", &result, events.len());
        assert!(result.is_ok(), "same-context refresh must succeed; see the typed child receipt");
        assert_eq!(events.len(), 1);
        let assert_original_binding = |transport: &CodexHttpTransport| {
            match &transport.credentials {
                ResponsesHttpCredentials::Fabushi { credential, endpoint, .. } => {
                    assert!(credential.context() == original_context, "original credential context retained");
                    assert!(credential.access_token() == ROTATED, "rotated bearer retained");
                    assert_eq!(endpoint, &original_endpoint);
                }
                _ => panic!("Native binding changed provider"),
            }
            let debug = format!("{:?}", transport.credentials);
            for token in [TOKEN_A, ROTATED, TOKEN_B] { assert!(!debug.contains(token)); }
        };
        assert_original_binding(&transport);
        child_phase(root, &mut receipt, "context-refusal-running");
        let result = transport.stream_response(&request, &mut |_| panic!("refusal cannot emit an event"), &|| false);
        record_transport_result(root, &mut receipt, "context-refusal", &result, 0);
        let error = result.expect_err("changed context is refused");
        assert!(matches!(&error, CodexDirectError::Authentication(_)));
        for token in [TOKEN_A, ROTATED, TOKEN_B] {
            assert!(!format!("{error:?} {error}").contains(token));
        }
        assert_original_binding(&transport);
        // Drive both conversion directions through the actual HTTP SSE decoder,
        // rather than relying only on a conversion/classifier unit assertion.
        child_phase(root, &mut receipt, "configuration-callback-running");
        let mut configuration_events = 0;
        let result = transport.stream_response(&request,
            &mut |_| {
                configuration_events += 1;
                Err(CodexDirectError::Configuration("typed callback configuration timeout".into()))
            },
            &|| false,
        );
        record_transport_result(root, &mut receipt, "configuration-callback", &result, configuration_events);
        let configuration = result.expect_err("configuration callback fails");
        assert!(matches!(&configuration, CodexDirectError::Configuration(_)));
        assert!(matches!(ProviderSessionError::from(configuration), ProviderSessionError::Configuration(_)));
        assert_original_binding(&transport);

        let next_origin = url::Url::parse(&next_endpoint).unwrap().origin().ascii_serialization();
        child_phase(root, &mut receipt, "new-transport-running");
        write_credential(&path, &next_origin, "user-b", TOKEN_B).expect("new transport credential");
        let mut new_transport = CodexHttpTransport::new_fabushi().unwrap();
        let mut new_events = 0;
        let result = new_transport.stream_response(&request, &mut |_| { new_events += 1; Ok(()) }, &|| false);
        record_transport_result(root, &mut receipt, "new-transport", &result, new_events);
        assert!(result.is_ok(), "new transport must succeed; see the typed child receipt");
        receipt["completed"] = json!(true);
        child_phase(root, &mut receipt, "complete");
    }

    struct FixtureReport {
        cleanup: Result<(), String>,
        cleanup_observation: Option<String>,
        server: Result<(), String>,
        server_panicked: bool,
        progress: Vec<String>,
        requests: Vec<Request>,
    }

    impl FixtureReport {
        fn is_ok(&self) -> bool {
            self.cleanup.is_ok() && self.server.is_ok()
        }

        fn evidence(&self) -> Value {
            json!({
                "childCleanup": self.cleanup_observation,
                "childCleanupOutcome": self.cleanup,
                "serverOutcome": self.server,
                "serverPanicked": self.server_panicked,
                "phase": self.progress.last(),
                "accepted": self.progress.iter().filter(|event| event.ends_with(" accepted")).count(),
                "parsed": self.progress.iter().filter(|event| event.ends_with(" request-complete")).count(),
                "responded": self.progress.iter().filter(|event| event.ends_with(" response-written")).count(),
                "capturedRequests": self.requests.len(),
            })
        }
    }

    struct Fixture {
        root: PathBuf,
        stop: Arc<AtomicBool>,
        server: Option<thread::JoinHandle<Result<(), String>>>,
        child: Option<std::process::Child>,
        cleanup_observation: Option<String>,
        progress: Arc<Mutex<Vec<String>>>,
        requests: Arc<Mutex<Vec<Request>>>,
    }

    impl Fixture {
        fn wait_child(&mut self) -> Result<std::process::ExitStatus, String> {
            let child = self.child.as_mut().ok_or("owned child was not started")?;
            let deadline = Instant::now() + BOUND;
            loop {
                if let Some(status) = child.try_wait().map_err(|error| fixture_io_error("child-status", error))? {
                    return Ok(status);
                }
                if Instant::now() >= deadline { return Err("owned transport child deadline".into()); }
                thread::sleep(Duration::from_millis(5));
            }
        }

        fn finish(&mut self) -> FixtureReport {
            let mut cleanup = Ok(());
            let mut cleanup_observation = self.cleanup_observation.clone();
            let mut retired = self.child.is_none();
            if let Some(child) = self.child.as_mut() {
                let before = child.try_wait().map_err(|error| fixture_io_error("child-reap-status", error));
                if let Ok(Some(status)) = &before {
                    retired = true;
                    if let Some(prior) = &cleanup_observation {
                        cleanup = Err(format!("previous owned child cleanup failed: {prior}"));
                    } else {
                        cleanup_observation = Some(format!("already-exited={status:?}"));
                    }
                } else if let Some(prior) = &cleanup_observation {
                    // A second finalizer may observe a late exit, but cannot
                    // restart an unresolved cleanup budget or erase its failure.
                    cleanup = Err(format!("owned child remains unresolved; prior cleanup={prior}; status={before:?}"));
                } else {
                    let deadline = Instant::now() + BOUND;
                    let kill = child.kill().map_err(|error| fixture_io_error("child-kill", error));
                    let reap = loop {
                        match child.try_wait() {
                            Ok(Some(status)) => break Ok(status),
                            Ok(None) if Instant::now() < deadline => {}
                            Ok(None) => break Err("owned child reap deadline; root retained".to_string()),
                            Err(error) => break Err(fixture_io_error("child-reap-status", error)),
                        }
                        thread::sleep(Duration::from_millis(5));
                    };
                    retired = reap.is_ok();
                    cleanup_observation = Some(format!("status-before={before:?}; kill={kill:?}; reap={reap:?}"));
                    if before.is_err() || kill.is_err() || reap.is_err() {
                        cleanup = Err(format!("owned child cleanup failed: {}", cleanup_observation.as_deref().unwrap_or("unavailable")));
                    }
                }
            }
            self.cleanup_observation = cleanup_observation.clone();
            if retired { drop(self.child.take()); }
            self.stop.store(true, Ordering::Release);
            let (mut server, server_panicked) = match self.server.take() {
                Some(server) => match server.join() {
                    Ok(result) => (result, false),
                    Err(_) => (Err("fixture worker panicked; payload omitted".into()), true),
                },
                None => (Ok(()), false),
            };
            let progress = self.progress.lock().unwrap_or_else(|poisoned| {
                server = Err(format!("prior server={server:?}; fixture progress lock poisoned"));
                poisoned.into_inner()
            }).clone();
            let requests = self.requests.lock().unwrap_or_else(|poisoned| {
                server = Err(format!("prior server={server:?}; fixture request capture lock poisoned"));
                poisoned.into_inner()
            }).clone();
            FixtureReport { cleanup, cleanup_observation, server, server_panicked, progress, requests }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            if self.server.is_some() || self.child.is_some() {
                let report = self.finish();
                eprintln!("case=captured-endpoint-and-context phase=cleanup server={} progress={:?}",
                    report.evidence(), report.progress);
            }
            if self.child.is_none() {
                let _ = fs::remove_dir_all(&self.root);
            } else {
                eprintln!("private credential fixture root retained path={} owned_child_exit_unresolved", self.root.display());
            }
        }
    }

    fn record_progress(progress: &Mutex<Vec<String>>, connection: usize, next: bool, stage: &str) -> Result<(), String> {
        progress.lock().map_err(|_| "fixture progress lock poisoned")?
            .push(format!("connection={connection} endpoint={} {stage}", if next { "new" } else { "captured" }));
        Ok(())
    }

    #[test]
    fn captured_endpoint_and_context_survive_real_http_refresh() {
        if let Some(root) = env::var_os(CHILD_ROOT) { child(&PathBuf::from(root)); return; }
        let root = env::temp_dir().join(format!("fabushi-bound-transport-{}", Uuid::new_v4()));
        fs::create_dir(&root).expect("exclusively create private transport root");
        let mut fixture = Fixture {
            root, stop: Arc::new(AtomicBool::new(false)), server: None, child: None,
            cleanup_observation: None,
            progress: Arc::new(Mutex::new(vec!["connection=0 listen".into()])),
            requests: Arc::new(Mutex::new(Vec::new())),
        };
        let a = TcpListener::bind("127.0.0.1:0").unwrap();
        let b = TcpListener::bind("127.0.0.1:0").unwrap();
        a.set_nonblocking(true).unwrap();
        b.set_nonblocking(true).unwrap();
        let origin_a = format!("http://{}", a.local_addr().unwrap());
        let endpoint_a = format!("{origin_a}/captured/v1/responses");
        let endpoint_b = format!("http://{}/changed/v2/responses", b.local_addr().unwrap());
        let path = fixture.root.join("credential.json");
        write_credential(&path, &origin_a, "user-a", TOKEN_A).expect("initial transport credential");
        let stopped = Arc::clone(&fixture.stop);
        let requests = Arc::clone(&fixture.requests);
        let server_progress = Arc::clone(&fixture.progress);
        let server_path = path.clone();
        fixture.server = Some(thread::spawn(move || {
            let deadline = Instant::now() + BOUND * 2;
            let mut a_count = 0;
            let mut connections = 0;
            while !stopped.load(Ordering::Acquire) {
                if Instant::now() >= deadline { return Err("private transport fixture deadline".into()); }
                for (next, listener) in [(false, &a), (true, &b)] {
                    let mut stream = match listener.accept() {
                        Ok((stream, _)) => stream,
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => continue,
                        Err(error) => return Err(fixture_io_error("accept", error)),
                    };
                    connections += 1;
                    record_progress(&server_progress, connections, next, "accepted")?;
                    record_progress(&server_progress, connections, next, "read-request")?;
                    let request = read_request(&mut stream, next)?;
                    record_progress(&server_progress, connections, next, "request-complete")?;
                    requests.lock().map_err(|_| "fixture request capture lock poisoned")?.push(request);
                    record_progress(&server_progress, connections, next, "install-image")?;
                    let unauthorized = !next && matches!(a_count, 0 | 2);
                    if !next {
                        if a_count == 0 {
                            write_credential(&server_path, &origin_a, "user-a", ROTATED)?;
                            record_progress(&server_progress, connections, next, "same-context-replacement-installed")?;
                        }
                        if a_count == 2 {
                            write_credential(&server_path, &origin_a, "user-b", TOKEN_B)?;
                            record_progress(&server_progress, connections, next, "changed-context-replacement-installed")?;
                        }
                        a_count += 1;
                    }
                    let (status, body) = if unauthorized { ("401 Unauthorized", "{}") } else {
                        ("200 OK", "data: {\"type\":\"response.completed\",\"response\":{\"id\":\"bound-response\",\"output\":[]}}\n\n")
                    };
                    let response = format!("HTTP/1.1 {status}\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
                    record_progress(&server_progress, connections, next, &format!("response-planned status={status}"))?;
                    record_progress(&server_progress, connections, next, "write-response")?;
                    stream.write_all(response.as_bytes()).map_err(|error| fixture_io_error("response-write", error))?;
                    record_progress(&server_progress, connections, next, "response-written")?;
                }
                thread::sleep(Duration::from_millis(2));
            }
            Ok(())
        }));
        let nonce = Uuid::new_v4().to_string();
        let stdout = fixture.root.join("stdout");
        let stderr = fixture.root.join("stderr");
        let status = (|| -> Result<std::process::ExitStatus, String> {
            let executable = env::current_exe().map_err(|error| fixture_io_error("child-executable", error))?;
            fixture.child = Some(Command::new(executable)
                .args(["--exact", TEST_NAME, "--nocapture", "--test-threads=1"])
                .env(CHILD_ROOT, &fixture.root).env(NEXT_ENDPOINT, endpoint_b).env(NONCE, &nonce)
                .env(FABUSHI_RESPONSES_URL_ENV, endpoint_a).env(FABUSHI_HOST_ACCESS_CREDENTIAL_FILE_ENV, path)
                .env("NO_PROXY", "*").env("no_proxy", "*")
                .stdout(fs::File::create(&stdout).map_err(|error| fixture_io_error("child-stdout-create", error))?)
                .stderr(fs::File::create(&stderr).map_err(|error| fixture_io_error("child-stderr-create", error))?)
                .spawn().map_err(|error| fixture_io_error("child-spawn", error))?);
            fixture.wait_child()
        })();
        let report = fixture.finish();
        let stdout = fs::read_to_string(stdout);
        let stderr = fs::read_to_string(stderr);
        let receipt_raw = fs::read(fixture.root.join("receipt"))
            .map_err(|error| fixture_io_error("child-receipt-read", error));
        let receipt_bytes = receipt_raw.as_ref().ok().map(Vec::len);
        let receipt = receipt_raw.and_then(|raw| serde_json::from_slice::<Value>(&raw).map_err(|error| format!(
            "child receipt JSON: category={:?}, line={}, column={}",
            error.classify(), error.line(), error.column(),
        )));
        let evidence = json!({
            "case":"captured-endpoint-and-context", "childResult":format!("{status:?}"),
            "server":report.evidence(), "progress":report.progress,
            "stdout":captured_output_observation(&stdout), "stderr":captured_output_observation(&stderr),
            "receiptBytes":receipt_bytes,
            "receipt":receipt.as_ref().map(|value| json!({
                "nonceMatches":value["nonce"] == nonce,
                "caseMatches":value["case"] == "captured-endpoint-and-context",
                "phase":value["phase"], "completed":value["completed"], "observations":value["observations"],
            })),
        });
        eprintln!("captured-endpoint credential evidence: {evidence}");
        assert!(status.as_ref().is_ok_and(|status| status.success()) && report.is_ok()
            && stdout.is_ok() && stderr.is_ok() && receipt.is_ok(), "{evidence}");
        let receipt = receipt.expect("checked child receipt");
        assert!(receipt["nonce"] == nonce, "a zero-test child must not satisfy this oracle");
        assert!(receipt["case"] == "captured-endpoint-and-context", "child receipt scenario binding");
        assert_eq!(receipt["completed"], true, "{evidence}");
        assert_eq!(receipt["observations"].as_array().unwrap().len(), 4);
        let requests = report.requests;
        assert_eq!(requests.len(), 5, "two allowed refresh requests, one refused invocation, one callback refusal, one new transport");
        for (index, (next, line, authorization, body)) in requests.iter().enumerate() {
            assert_eq!(*next, index == 4, "only the new transport can use the changed endpoint");
            assert!(line == if *next { "POST /changed/v2/responses HTTP/1.1" } else { "POST /captured/v1/responses HTTP/1.1" }, "request must use its bound endpoint");
            assert!(authorization == &format!("Bearer {}", match index { 0 => TOKEN_A, 4 => TOKEN_B, _ => ROTATED }), "request bearer identity");
            assert!(body == &requests[0].3, "request/model bytes remain unchanged");
        }
    }

    const TURN_BINDING_TEST: &str = "extensions::inference::provider_session::fabushi_credential_context_tests::turn_binding_retains_captured_configuration_across_transports";
    const TURN_BINDING_CHILD_ROOT: &str = "FABUSHI_TURN_BINDING_CHILD_ROOT";
    const TURN_BINDING_CAPTURE_ENDPOINT: &str = "FABUSHI_TURN_BINDING_CAPTURE_ENDPOINT";
    const TURN_BINDING_CAPTURE_PATH: &str = "FABUSHI_TURN_BINDING_CAPTURE_PATH";
    const TURN_BINDING_NONCE: &str = "FABUSHI_TURN_BINDING_NONCE";
    const TURN_BINDING_ROTATED_AGAIN: &str = "private-turn-binding-token-rotated-again";
    const TURN_BINDING_COMPLETE: &str = "initial-refusals-captured-a-two-transports-fresh-b-complete";

    fn turn_binding_fixture(label: &str) -> Fixture {
        let root = env::temp_dir().join(format!("fabushi-turn-binding-{label}-{}", Uuid::new_v4()));
        fs::create_dir(&root).expect("exclusively create private turn-binding fixture");
        Fixture {
            root, stop: Arc::new(AtomicBool::new(false)), server: None, child: None,
            cleanup_observation: None,
            progress: Arc::new(Mutex::new(vec!["connection=0 listen".into()])),
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn assert_turn_binding_authentication<T>(
        result: Result<T, ProviderSessionError>,
        phase: &str,
    ) {
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("{phase}: expected permanent Authentication refusal"),
        };
        assert!(matches!(&error, ProviderSessionError::Authentication(_)),
            "{phase}: refusal lost its Authentication type");
        let rendered = format!("{error} {error:?}");
        for token in [TOKEN_A, ROTATED, TOKEN_B, TURN_BINDING_ROTATED_AGAIN] {
            assert!(!rendered.contains(token), "{phase}: refusal exposed a credential");
        }
        let direct = codex_provider_error(error);
        assert!(matches!(&direct, CodexDirectError::Authentication(_)),
            "{phase}: Responses conversion lost its permanent error type");
        assert!(matches!(ProviderSessionError::from(direct), ProviderSessionError::Authentication(_)),
            "{phase}: routed conversion lost its permanent error type");
    }

    fn turn_binding_matches_credentials(
        binding: &FabushiResponsesRequestBinding,
        credentials: &ResponsesHttpCredentials,
    ) -> bool {
        match (binding.identity.get(), credentials) {
            (Some(identity), ResponsesHttpCredentials::Fabushi { credential, endpoint, credential_path }) =>
                identity.context == credential.context()
                    && identity.endpoint == *endpoint
                    && identity.credential_path == *credential_path,
            _ => false,
        }
    }

    fn assert_turn_binding_repair(
        binding: &FabushiResponsesRequestBinding,
        original: &FabushiResponsesRequestIdentity,
        path: &Path,
        issuer: &str,
        phase: &str,
    ) {
        assert!(binding.identity.get() == Some(original), "{phase}: refusal replaced the admitted identity");
        // The existing private writer also restores permissions after the
        // missing/non-regular/non-private file controls.
        write_credential(path, issuer, "user-a", ROTATED).expect("repair original private credential");
        let mut rotated: Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        rotated["expiresAtMs"] = json!(4_102_444_900_000_u64);
        fs::write(path, rotated.to_string()).unwrap();
        let credentials = binding.credentials_for_transport().expect("repaired original context remains usable");
        assert!(turn_binding_matches_credentials(binding, &credentials), "{phase}: repair changed the binding");
        assert!(binding.identity.get() == Some(original), "{phase}: rotation replaced the admitted identity");
        assert!(matches!(&credentials, ResponsesHttpCredentials::Fabushi { credential, .. }
            if credential.access_token() == ROTATED && credential.expires_at_ms == 4_102_444_900_000_u64),
            "{phase}: transport did not reread the repaired token and expiry");
    }

    #[test]
    fn turn_binding_refusals_preserve_complete_identity() {
        let fixture = turn_binding_fixture("refusals");
        let path = fixture.root.join("credential-a.json");
        let issuer = "http://127.0.0.1:1";
        let endpoint = validate_fabushi_responses_url(&format!("{issuer}/captured/v1/responses")).unwrap();
        write_credential(&path, issuer, "user-a", TOKEN_A).expect("initial private binding credential");
        let original_image: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let binding = FabushiResponsesRequestBinding::for_runner_invocation();
        assert!(binding.identity.get().is_none(), "allocation must not acquire credentials");
        let captured = read_fabushi_responses_credentials(&endpoint, &path).unwrap();
        binding.accept_credentials(&captured).unwrap();
        let original = binding.identity.get().unwrap().clone();
        drop(captured);

        let mut invalid_images = Vec::new();
        for (phase, field, value) in [
            ("changed valid user", "userId", json!("user-b")),
            ("changed valid device", "deviceId", json!("other-device")),
            ("changed valid session", "sessionId", json!("other-session")),
            ("unsupported version", "version", json!(2)),
            ("empty token", "accessToken", json!("")),
            ("invalid user type", "userId", json!(false)),
            ("blank user", "userId", json!(" ")),
            ("blank device", "deviceId", json!(" ")),
            ("blank session", "sessionId", json!(" ")),
            ("expired image", "expiresAtMs", json!(0)),
            ("mismatched issuer origin", "issuer", json!("http://127.0.0.1:2")),
            ("unapproved issuer", "issuer", json!("https://unapproved.invalid")),
            ("malformed issuer", "issuer", json!("malformed issuer")),
            ("invalid issuer type", "issuer", json!(7)),
        ] {
            let mut image = original_image.clone();
            image[field] = value;
            invalid_images.push((phase.to_string(), image));
        }
        for field in ["accessToken", "expiresAtMs", "userId", "deviceId", "sessionId", "issuer"] {
            let mut image = original_image.clone();
            image.as_object_mut().unwrap().remove(field);
            invalid_images.push((format!("missing {field}"), image));
        }
        invalid_images.push(("non-object document".into(), json!([])));
        for (phase, image) in invalid_images {
            fs::write(&path, image.to_string()).unwrap();
            assert_turn_binding_authentication(binding.credentials_for_transport(), &phase);
            assert_turn_binding_repair(&binding, &original, &path, issuer, &phase);
        }

        fs::write(&path, b"{malformed credential image").unwrap();
        assert_turn_binding_authentication(binding.credentials_for_transport(), "malformed JSON");
        assert_turn_binding_repair(&binding, &original, &path, issuer, "malformed JSON");
        fs::remove_file(&path).unwrap();
        assert_turn_binding_authentication(binding.credentials_for_transport(), "unavailable file");
        assert_turn_binding_repair(&binding, &original, &path, issuer, "unavailable file");
        fs::remove_file(&path).unwrap();
        fs::create_dir(&path).unwrap();
        assert_turn_binding_authentication(binding.credentials_for_transport(), "directory instead of private file");
        assert!(binding.identity.get() == Some(&original), "directory refusal changed identity");
        fs::remove_dir(&path).unwrap();
        assert_turn_binding_repair(&binding, &original, &path, issuer, "directory instead of private file");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
            assert_turn_binding_authentication(binding.credentials_for_transport(), "non-private file");
            assert_turn_binding_repair(&binding, &original, &path, issuer, "non-private file");
            let target = fixture.root.join("symlink-target.json");
            write_credential(&target, issuer, "user-a", TOKEN_A).expect("private symlink control target");
            fs::remove_file(&path).unwrap();
            std::os::unix::fs::symlink(&target, &path).unwrap();
            assert_turn_binding_authentication(binding.credentials_for_transport(), "symlink instead of private file");
            assert!(binding.identity.get() == Some(&original), "symlink refusal changed identity");
            fs::remove_file(&path).unwrap();
            assert_turn_binding_repair(&binding, &original, &path, issuer, "symlink instead of private file");
        }

        // Both candidates come from the real guarded reader and contain the
        // same Auth context. Each independent configuration identity member
        // must still refuse adoption through the production initializer.
        let other_endpoint = validate_fabushi_responses_url(&format!("{issuer}/changed/v2/responses")).unwrap();
        let endpoint_candidate = read_fabushi_responses_credentials(&other_endpoint, &path).unwrap();
        assert_turn_binding_authentication(binding.accept_credentials(&endpoint_candidate), "complete endpoint mismatch");
        assert_turn_binding_repair(&binding, &original, &path, issuer, "complete endpoint mismatch");
        let other_path = fixture.root.join("credential-b.json");
        write_credential(&other_path, issuer, "user-a", TOKEN_B).expect("private alternate-path credential");
        let path_candidate = read_fabushi_responses_credentials(&endpoint, &other_path).unwrap();
        assert_turn_binding_authentication(binding.accept_credentials(&path_candidate), "credential path mismatch");
        assert_turn_binding_repair(&binding, &original, &path, issuer, "credential path mismatch");
    }

    fn race_turn_binding_images(
        binding: &FabushiResponsesRequestBinding,
        left: (&str, &Path),
        right: (&str, &Path),
    ) -> [(ResponsesHttpCredentials, Result<(), ProviderSessionError>); 2] {
        let barrier = std::sync::Barrier::new(3);
        thread::scope(|scope| {
            let read_and_accept = |(endpoint, path)| {
                // Even a failed read reaches the barrier, so a fixture error
                // cannot strand the other contender before joining it.
                let credentials = read_fabushi_responses_credentials(endpoint, path);
                barrier.wait();
                let credentials = credentials.expect("real private race candidate");
                let outcome = binding.accept_credentials(&credentials);
                (credentials, outcome)
            };
            let first = scope.spawn(move || read_and_accept(left));
            let second = scope.spawn(move || read_and_accept(right));
            barrier.wait();
            let first = first.join();
            let second = second.join();
            [first.expect("first binding contender completed"), second.expect("second binding contender completed")]
        })
    }

    #[test]
    fn turn_binding_first_valid_images_have_one_immutable_winner() {
        let fixture = turn_binding_fixture("race");
        let a = fixture.root.join("credential-a.json");
        let b = fixture.root.join("credential-b.json");
        let issuer = "http://127.0.0.1:1";
        let endpoint = validate_fabushi_responses_url(&format!("{issuer}/race/v1/responses")).unwrap();
        write_credential(&a, issuer, "user-a", TOKEN_A).expect("first private race credential");
        write_credential(&b, issuer, "user-b", TOKEN_B).expect("second private race credential");
        let binding = FabushiResponsesRequestBinding::for_runner_invocation();
        let [(first, first_result), (second, second_result)] =
            race_turn_binding_images(&binding, (&endpoint, &a), (&endpoint, &b));
        assert_eq!(usize::from(first_result.is_ok()) + usize::from(second_result.is_ok()), 1,
            "different first images must have exactly one successful initializer");
        let (winner, loser, losing_result) = if first_result.is_ok() {
            (first, second, second_result)
        } else {
            (second, first, first_result)
        };
        assert_turn_binding_authentication(losing_result, "first-image race loser");
        assert!(turn_binding_matches_credentials(&binding, &winner), "binding did not retain the race winner");
        let original = binding.identity.get().unwrap().clone();
        assert_turn_binding_authentication(binding.accept_credentials(&loser), "repeated race loser");
        assert!(binding.identity.get() == Some(&original), "loser replaced the winning identity");
        let reread = binding.credentials_for_transport().unwrap();
        assert!(turn_binding_matches_credentials(&binding, &reread), "transport did not reread the winning source");
        assert!(binding.identity.get() == Some(&original), "transport replaced the winning identity");

        let winner_path = match &winner {
            ResponsesHttpCredentials::Fabushi { credential_path, .. } => credential_path,
            _ => panic!("race winner must be Native"),
        };
        let same = FabushiResponsesRequestBinding::for_runner_invocation();
        let [(left, left_result), (right, right_result)] =
            race_turn_binding_images(&same, (&endpoint, winner_path), (&endpoint, winner_path));
        assert!(left_result.is_ok() && right_result.is_ok(), "same-identity initialization must allow both contenders");
        assert!(turn_binding_matches_credentials(&same, &left) && turn_binding_matches_credentials(&same, &right),
            "same-identity initialization lost complete identity equality");
    }

    fn turn_binding_http_request() -> Value {
        json!({"model":"Vendor/Case@TurnBinding", "input":[{"role":"user","content":"captured turn request"}], "stream":true})
    }

    fn turn_binding_http_child(root: &Path) {
        let endpoint_b = env::var(FABUSHI_RESPONSES_URL_ENV).unwrap();
        let path_b = PathBuf::from(env::var_os(FABUSHI_HOST_ACCESS_CREDENTIAL_FILE_ENV).unwrap());
        let issuer_b = url::Url::parse(&endpoint_b).unwrap().origin().ascii_serialization();
        let initial = FabushiResponsesRequestBinding::for_runner_invocation();
        assert!(initial.identity.get().is_none());
        assert_turn_binding_authentication(initial.credentials_for_transport(), "initial unavailable credential");
        assert!(initial.identity.get().is_none(), "unavailable credential admitted an identity");
        write_credential(&path_b, &issuer_b, "user-b", TOKEN_B).expect("private initial malformed control");
        fs::write(&path_b, b"{malformed initial credential").unwrap();
        assert_turn_binding_authentication(initial.credentials_for_transport(), "initial malformed credential");
        assert!(initial.identity.get().is_none(), "malformed credential admitted an identity");
        write_credential(&path_b, &issuer_b, "user-b", TOKEN_B).expect("private initial credential repair");
        let repaired_initial = initial.credentials_for_transport().unwrap();
        assert!(turn_binding_matches_credentials(&initial, &repaired_initial), "valid image after initial refusal must bind");

        let endpoint_a = env::var(TURN_BINDING_CAPTURE_ENDPOINT).unwrap();
        let path_a = PathBuf::from(env::var_os(TURN_BINDING_CAPTURE_PATH).unwrap());
        let issuer_a = url::Url::parse(&endpoint_a).unwrap().origin().ascii_serialization();
        // Narrow private fixture: use one real validated image and the very
        // initializer used by credentials_for_transport. The public eager
        // capture factory is exercised by the external production tests.
        // Environment B is fixed at process launch and never mutated here.
        let captured = read_fabushi_responses_credentials(&endpoint_a, &path_a).unwrap();
        let binding = FabushiResponsesRequestBinding::for_runner_invocation();
        binding.accept_credentials(&captured).unwrap();
        assert!(turn_binding_matches_credentials(&binding, &captured));
        drop(captured);
        for token in [ROTATED, TURN_BINDING_ROTATED_AGAIN] {
            write_credential(&path_a, &issuer_a, "user-a", token).expect("private bound token rotation");
            let shared = binding.clone();
            let mut transport = CodexHttpTransport::new_fabushi_with_binding(Some(&shared)).unwrap();
            let mut events = 0;
            transport.stream_response(&turn_binding_http_request(), &mut |_| { events += 1; Ok(()) }, &|| false).unwrap();
            assert_eq!(events, 1, "each bound transport must consume the real completed SSE event");
            assert!(turn_binding_matches_credentials(&binding, &transport.credentials));
            assert!(matches!(&transport.credentials, ResponsesHttpCredentials::Fabushi { credential, .. }
                if credential.access_token() == token), "a new transport reused a stale token");
        }
        let mut fresh = CodexHttpTransport::new_fabushi().unwrap();
        let mut events = 0;
        fresh.stream_response(&turn_binding_http_request(), &mut |_| { events += 1; Ok(()) }, &|| false).unwrap();
        assert_eq!(events, 1, "fresh unbound operation must reach endpoint B");
        assert!(turn_binding_matches_credentials(&initial, &fresh.credentials), "fresh operation did not use configuration B");
        assert!(env::var(FABUSHI_RESPONSES_URL_ENV).unwrap() == endpoint_b);
        assert!(PathBuf::from(env::var_os(FABUSHI_HOST_ACCESS_CREDENTIAL_FILE_ENV).unwrap()) == path_b);
        let receipt = format!("{}\n{}\n{TURN_BINDING_COMPLETE}\n", env::var(TURN_BINDING_NONCE).unwrap(), std::process::id());
        fs::write(root.join("turn-binding-receipt"), receipt).unwrap();
    }

    // Only nonsecret observations may enter the parent diagnostic. Raw HTTP
    // headers/bodies and raw child stdout/stderr are never rendered by this
    // new control, including on deadline, wait error or worker panic.
    #[derive(Debug, Default)]
    struct TurnBindingHttpObservation {
        accepted: usize,
        parsed: usize,
        responded: usize,
        phase: &'static str,
        failed: bool,
        requests: Vec<(bool, bool, bool, bool)>,
    }

    #[test]
    fn turn_binding_retains_captured_configuration_across_transports() {
        if let Some(root) = env::var_os(TURN_BINDING_CHILD_ROOT) {
            turn_binding_http_child(&PathBuf::from(root));
            return;
        }
        let mut fixture = turn_binding_fixture("http");
        let a = TcpListener::bind("127.0.0.1:0").unwrap();
        let b = TcpListener::bind("127.0.0.1:0").unwrap();
        a.set_nonblocking(true).unwrap();
        b.set_nonblocking(true).unwrap();
        let issuer_a = format!("http://{}", a.local_addr().unwrap());
        let endpoint_a = validate_fabushi_responses_url(&format!("{issuer_a}/turn-bound/v1/responses")).unwrap();
        let endpoint_b = validate_fabushi_responses_url(&format!("http://{}/turn-current/v2/responses", b.local_addr().unwrap())).unwrap();
        let path_a = fixture.root.join("captured-credential.json");
        let path_b = fixture.root.join("configured-credential.json");
        write_credential(&path_a, &issuer_a, "user-a", TOKEN_A).expect("private captured credential");
        let observation = Arc::new(Mutex::new(TurnBindingHttpObservation::default()));
        let server_observation = Arc::clone(&observation);
        let server_progress = Arc::clone(&fixture.progress);
        let server_requests = Arc::clone(&fixture.requests);
        let stop = Arc::clone(&fixture.stop);
        fixture.server = Some(thread::spawn(move || -> Result<(), String> {
            let deadline = Instant::now() + BOUND * 2;
            let expected_body = serde_json::to_vec(&turn_binding_http_request()).unwrap();
            while !stop.load(Ordering::Acquire) {
                if Instant::now() >= deadline {
                    let mut observed = server_observation.lock().unwrap();
                    observed.phase = "listener deadline";
                    observed.failed = true;
                    return Err("turn-binding fixture listener deadline".into());
                }
                for (next, listener) in [(false, &a), (true, &b)] {
                    let mut stream = match listener.accept() {
                        Ok((stream, _)) => stream,
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => continue,
                        Err(error) => {
                            let mut observed = server_observation.lock().unwrap();
                            observed.phase = "listener accept";
                            observed.failed = true;
                            return Err(fixture_io_error("turn-binding-listener-accept", error));
                        }
                    };
                    let connection = {
                        let mut observed = server_observation.lock().unwrap();
                        observed.accepted += 1;
                        observed.phase = "bounded blocking request read";
                        observed.accepted
                    };
                    record_progress(&server_progress, connection, next, "accepted")?;
                    record_progress(&server_progress, connection, next, "read-request")?;
                    // The established reader resets accepted sockets to
                    // blocking before its bounded read/write timeouts.
                    let request = match read_request(&mut stream, next) {
                        Ok(request) => request,
                        Err(error) => {
                            server_observation.lock().unwrap().failed = true;
                            return Err(error);
                        }
                    };
                    record_progress(&server_progress, connection, next, "request-complete")?;
                    {
                        let (_, line, authorization, body) = &request;
                        let mut observed = server_observation.lock().unwrap();
                        let expected_token = match observed.parsed {
                            0 => ROTATED,
                            1 => TURN_BINDING_ROTATED_AGAIN,
                            _ => TOKEN_B,
                        };
                        observed.requests.push((
                            next,
                            line.as_str() == if next { "POST /turn-current/v2/responses HTTP/1.1" } else { "POST /turn-bound/v1/responses HTTP/1.1" },
                            authorization == &format!("Bearer {expected_token}"),
                            body == &expected_body,
                        ));
                        observed.parsed += 1;
                        observed.phase = "bounded response write";
                    }
                    server_requests.lock().map_err(|_| "fixture request capture lock poisoned")?.push(request);
                    let body = "data: {\"type\":\"response.completed\",\"response\":{\"id\":\"turn-binding-response\",\"output\":[]}}\n\n";
                    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
                    record_progress(&server_progress, connection, next, "response-planned status=200 OK")?;
                    record_progress(&server_progress, connection, next, "write-response")?;
                    if let Err(error) = stream.write_all(response.as_bytes()) {
                        server_observation.lock().unwrap().failed = true;
                        return Err(fixture_io_error("turn-binding-response-write", error));
                    }
                    record_progress(&server_progress, connection, next, "response-written")?;
                    let mut observed = server_observation.lock().unwrap();
                    observed.responded += 1;
                    observed.phase = "observing both endpoints";
                }
                thread::sleep(Duration::from_millis(2));
            }
            Ok(())
        }));
        let nonce = Uuid::new_v4().to_string();
        let mut child_pid = None;
        let outcome = (|| -> Result<std::process::ExitStatus, String> {
            let executable = env::current_exe().map_err(|error| fixture_io_error("child-executable", error))?;
            fixture.child = Some(Command::new(executable)
                .args(["--exact", TURN_BINDING_TEST, "--nocapture", "--test-threads=1"])
                .env(TURN_BINDING_CHILD_ROOT, &fixture.root).env(TURN_BINDING_NONCE, &nonce)
                .env(TURN_BINDING_CAPTURE_ENDPOINT, endpoint_a).env(TURN_BINDING_CAPTURE_PATH, path_a)
                .env(FABUSHI_RESPONSES_URL_ENV, endpoint_b).env(FABUSHI_HOST_ACCESS_CREDENTIAL_FILE_ENV, path_b)
                .env("NO_PROXY", "*").env("no_proxy", "*")
                .stdout(Stdio::null()).stderr(Stdio::null())
                .spawn().map_err(|error| fixture_io_error("turn-binding-child-spawn", error))?);
            child_pid = Some(fixture.child.as_ref().unwrap().id());
            fixture.wait_child()
        })();
        // The same fixture owner retains the original child outcome, bounds
        // retirement, joins the server, and authorizes root deletion on exit.
        let report = fixture.finish();
        let observed = observation.lock().unwrap_or_else(|error| error.into_inner());
        // Retain the safe server outcome before adjudicating any child error.
        eprintln!("turn-binding HTTP control pid={child_pid:?} child={outcome:?} report={} progress={:?} outcome={observed:?}", report.evidence(), report.progress);
        assert!(!report.server_panicked && !observed.failed && report.is_ok(),
            "turn-binding HTTP fixture failed: {observed:?}; report={}", report.evidence());
        assert!(matches!(&outcome, Ok(status) if status.success()), "turn-binding child failed: {outcome:?}; server={observed:?}");
        let child_pid = child_pid.expect("successful owned child retained its PID");
        let expected_receipt = format!("{nonce}\n{child_pid}\n{TURN_BINDING_COMPLETE}\n");
        assert!(fs::read_to_string(fixture.root.join("turn-binding-receipt")).ok().as_deref() == Some(expected_receipt.as_str()),
            "owned child lacks the nonce/PID/completion-bound receipt; server={observed:?}");
        assert_eq!((observed.accepted, observed.parsed, observed.responded), (3, 3, 3), "both bound transports and the fresh positive must complete");
        assert_eq!(observed.requests, vec![(false, true, true, true), (false, true, true, true), (true, true, true, true)],
            "bound requests must use captured A with fresh tokens and unchanged model/body; B observes only the later unbound positive");
    }
}
