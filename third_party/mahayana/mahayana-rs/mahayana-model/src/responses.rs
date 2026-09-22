use crate::{
    ModelError, ModelEvent, ModelEventSink, ModelRequest, ModelRuntime, ModelUsage,
    SharedModelEventSink,
};
use async_trait::async_trait;
use mahayana_core::ModelProviderMode;
use mahayana_host_runtime::{
    AttemptProgress, RetryDecision, StreamAttemptPolicy, TransientStreamError,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ResponsesWireApi {
    #[default]
    Responses,
    ChatCompletions,
    AnthropicMessages,
}

const MODEL_CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const MODEL_READ_TIMEOUT: Duration = Duration::from_secs(150);
const MODEL_WRITE_TIMEOUT: Duration = Duration::from_secs(30);

struct FirstOutputTrackingSink {
    inner: SharedModelEventSink,
    seen: AtomicBool,
}

impl FirstOutputTrackingSink {
    fn new(inner: SharedModelEventSink) -> Self {
        Self { inner, seen: AtomicBool::new(false) }
    }

    fn seen(&self) -> bool {
        self.seen.load(Ordering::SeqCst)
    }
}

impl ModelEventSink for FirstOutputTrackingSink {
    fn emit(&self, event: ModelEvent) -> Result<(), ModelError> {
        if matches!(&event, ModelEvent::OutputTextDelta(_) | ModelEvent::Completed { .. }) {
            self.seen.store(true, Ordering::SeqCst);
        }
        self.inner.emit(event)
    }
}


#[derive(Debug, Clone)]
pub struct ResponsesModelConfig {
    pub base_url: String,
    pub default_model: String,
    pub bearer_token: Option<String>,
    pub provider_mode: ModelProviderMode,
    pub wire_api: ResponsesWireApi,
}

impl ResponsesModelConfig {
    pub fn validate(&self) -> Result<(), ModelError> {
        if self.default_model.trim().is_empty() {
            return Err(ModelError::InvalidRequest(
                "default model must not be empty".into(),
            ));
        }
        if !self.base_url.starts_with("https://")
            && !matches!(self.provider_mode, ModelProviderMode::LocalLoopback)
        {
            return Err(ModelError::InvalidRequest(
                "remote model endpoints must use HTTPS".into(),
            ));
        }
        if self
            .bearer_token
            .as_deref()
            .is_some_and(|token| token.trim().is_empty() || token.contains(['\r', '\n']))
        {
            return Err(ModelError::InvalidRequest(
                "model credential contains invalid header characters".into(),
            ));
        }
        Ok(())
    }
}

/// Provider-neutral Responses API implementation of Mahayana's model boundary.
///
/// This component performs model inference only. It does not host an Agent,
/// execute tools, own workspace state, or make policy decisions; those belong
/// to `mahayana-native-engine` and the sovereign kernel.
pub struct ResponsesModelRuntime {
    config: ResponsesModelConfig,
    credential_resolver: Option<ModelCredentialResolver>,
}

/// Resolves the current product-account bearer token at inference time. A
/// long-lived desktop Host must not retain the previous account's credential
/// after logout or account switching.
pub type ModelCredentialResolver =
    Arc<dyn Fn() -> Result<Option<String>, ModelError> + Send + Sync>;

impl ResponsesModelRuntime {
    pub fn new(config: ResponsesModelConfig) -> Result<Self, ModelError> {
        config.validate()?;
        Ok(Self {
            config,
            credential_resolver: None,
        })
    }

    pub fn with_credential_resolver(mut self, resolver: ModelCredentialResolver) -> Self {
        self.credential_resolver = Some(resolver);
        self
    }
}

#[async_trait]
impl ModelRuntime for ResponsesModelRuntime {
    async fn infer(
        &self,
        mut request: ModelRequest,
        events: SharedModelEventSink,
    ) -> Result<(), ModelError> {
        if request.model.trim().is_empty() {
            request.model = self.config.default_model.clone();
        }
        if request.model.trim().is_empty() {
            return Err(ModelError::InvalidRequest("model must not be empty".into()));
        }

        let mut config = self.config.clone();
        if let Some(resolver) = self.credential_resolver.as_ref() {
            config.bearer_token = resolver()?;
        }
        let tracker = Arc::new(FirstOutputTrackingSink::new(Arc::clone(&events)));
        let tracked_events: SharedModelEventSink = tracker.clone();
        let policy = StreamAttemptPolicy::default();
        let mut attempt = 0_u32;
        let (payload, streamed_text) = loop {
            attempt += 1;
            let config_for_attempt = config.clone();
            let request_for_attempt = request.clone();
            let events_for_attempt = Arc::clone(&tracked_events);
            let first_output_timeout = policy.first_output_timeout;
            let outcome = tokio::task::spawn_blocking(move || {
                request_response(
                    &config_for_attempt,
                    request_for_attempt,
                    events_for_attempt,
                    first_output_timeout,
                )
            })
            .await
            .map_err(|error| ModelError::Inference(format!("model task failed: {error}")))
            .and_then(|result| result);

            match outcome {
                Ok(value) => break value,
                Err(error) => {
                    let mut progress = AttemptProgress::default();
                    if tracker.seen() {
                        progress.record_output(1);
                    }
                    let failure = TransientStreamError::classify(error.to_string(), None, None);
                    match policy.retry_decision(attempt, &progress, &failure) {
                        RetryDecision::RetryAfter(delay) if is_retryable_model_error(&error) => {
                            tokio::time::sleep(delay).await;
                        }
                        RetryDecision::ResumeAfter { .. } => {
                            // The Responses transport does not currently expose a durable
                            // provider cursor/checkpoint. Never replay partial visible
                            // output without one; fail closed instead.
                            return Err(error);
                        }
                        RetryDecision::RetryAfter(_) | RetryDecision::Fail => return Err(error),
                    }
                }
            }
        };

        // SSE deltas have already reached the Agent event sink. Only emit the
        // final text for JSON/fallback endpoints to avoid duplicating replies.
        if !streamed_text && let Some(text) = extract_output_text(&payload) {
            events.emit(ModelEvent::OutputTextDelta(text))?;
        }
        if let Some(usage) = extract_usage(&payload) {
            events.emit(ModelEvent::Usage(usage))?;
        }
        events.emit(ModelEvent::Completed { output: payload })
    }

    fn provider_mode(&self) -> ModelProviderMode {
        self.config.provider_mode
    }
}

fn is_retryable_model_error(error: &ModelError) -> bool {
    let (ModelError::Inference(message) | ModelError::Unavailable(message)) = error else {
        return false;
    };
    let message = message.to_ascii_lowercase();
    [
        "timeout",
        "timed out",
        "transport",
        "connection",
        "temporar",
        "unavailable",
        "overload",
        "capacity",
        "rate limit",
        "429",
        "500",
        "502",
        "503",
        "504",
    ]
    .iter()
    .any(|needle| message.contains(needle))
}


fn request_response(
    config: &ResponsesModelConfig,
    request: ModelRequest,
    events: SharedModelEventSink,
    first_output_timeout: Duration,
) -> Result<(Value, bool), ModelError> {
    if matches!(
        config.provider_mode,
        ModelProviderMode::UserConfiguredRemote
    ) && config.bearer_token.is_none()
    {
        return Err(ModelError::InvalidRequest(
            "selected model provider credential is not configured".into(),
        ));
    }
    let (endpoint, body) = match config.wire_api {
        ResponsesWireApi::Responses => {
            let endpoint = if config.base_url.ends_with("/responses") {
                config.base_url.clone()
            } else {
                format!("{}/responses", config.base_url.trim_end_matches('/'))
            };
            let mut body =
                json!({ "model": request.model, "input": request.input, "stream": true });
            for key in [
                "tools",
                "tool_choice",
                "parallel_tool_calls",
                "instructions",
                "reasoning",
                "text",
                "temperature",
                "max_output_tokens",
            ] {
                if let Some(value) = request.metadata.get(key) {
                    body[key] = value.clone();
                }
            }
            (endpoint, body)
        }
        ResponsesWireApi::ChatCompletions => {
            let endpoint = if config.base_url.ends_with("/chat/completions") {
                config.base_url.clone()
            } else {
                format!("{}/chat/completions", config.base_url.trim_end_matches('/'))
            };
            let mut messages = chat_messages(&request.input);
            if let Some(instructions) = request.metadata.get("instructions").and_then(Value::as_str)
                && !instructions.trim().is_empty()
            {
                messages.insert(0, json!({"role":"system", "content": instructions}));
            }
            let mut body = json!({ "model": request.model, "messages": messages, "stream": true });
            if let Some(tools) = request.metadata.get("tools").and_then(Value::as_array) {
                body["tools"] = Value::Array(tools.iter().filter_map(chat_tool).collect());
            }
            for key in ["tool_choice", "parallel_tool_calls", "temperature"] {
                if let Some(value) = request.metadata.get(key) {
                    body[key] = value.clone();
                }
            }
            if let Some(value) = request.metadata.get("max_output_tokens") {
                body["max_tokens"] = value.clone();
            }
            (endpoint, body)
        }
        ResponsesWireApi::AnthropicMessages => {
            let endpoint = if config.base_url.ends_with("/messages") {
                config.base_url.clone()
            } else {
                format!("{}/messages", config.base_url.trim_end_matches('/'))
            };
            let mut body = json!({
                "model": request.model,
                "messages": anthropic_messages(&request.input),
                "max_tokens": request.metadata.get("max_output_tokens").cloned().unwrap_or_else(|| json!(4096)),
                "stream": true,
            });
            let system = anthropic_system(&request.input, request.metadata.get("instructions"));
            if !system.is_empty() {
                body["system"] = json!(system);
            }
            if let Some(tools) = request.metadata.get("tools").and_then(Value::as_array) {
                body["tools"] = Value::Array(tools.iter().filter_map(anthropic_tool).collect());
            }
            if let Some(value) = request.metadata.get("temperature") {
                body["temperature"] = value.clone();
            }
            (endpoint, body)
        }
    };

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(MODEL_CONNECT_TIMEOUT)
        .timeout_read(MODEL_READ_TIMEOUT)
        .timeout_write(MODEL_WRITE_TIMEOUT)
        .build();
    let mut http = agent
        .post(&endpoint)
        .set("Accept", "text/event-stream, application/json");
    if let Some(token) = config.bearer_token.as_deref() {
        http = match config.wire_api {
            ResponsesWireApi::AnthropicMessages => http
                .set("x-api-key", token)
                .set("anthropic-version", "2023-06-01"),
            ResponsesWireApi::Responses | ResponsesWireApi::ChatCompletions => {
                http.set("Authorization", &format!("Bearer {token}"))
            }
        };
    }
    let response = http.send_json(body).map_err(redacted_http_error)?;
    if response
        .header("Content-Type")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .contains("text/event-stream")
    {
        return request_stream(response, config.wire_api, events, first_output_timeout);
    }
    let payload: Value = response
        .into_json()
        .map_err(|_| ModelError::Inference("model endpoint returned invalid JSON".into()))?;
    if let Some(error) = payload.get("error") {
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("model endpoint returned an error");
        return Err(ModelError::Inference(message.to_string()));
    }
    Ok((
        match config.wire_api {
            ResponsesWireApi::Responses => payload,
            ResponsesWireApi::ChatCompletions => normalize_chat_payload(payload),
            ResponsesWireApi::AnthropicMessages => normalize_anthropic_payload(payload),
        },
        false,
    ))
}

#[derive(Debug, Default)]
struct StreamToolCall {
    id: String,
    name: String,
    arguments: String,
}

#[derive(Debug)]
struct StreamAccumulator {
    wire_api: ResponsesWireApi,
    id: Option<Value>,
    text: String,
    streamed_text: bool,
    first_output_seen: bool,
    final_payload: Option<Value>,
    tools: BTreeMap<usize, StreamToolCall>,
    usage: Option<Value>,
    anthropic_input_tokens: u64,
    anthropic_cache_creation_input_tokens: u64,
    anthropic_cache_read_input_tokens: u64,
    anthropic_output_tokens: u64,
}

impl StreamAccumulator {
    fn new(wire_api: ResponsesWireApi) -> Self {
        Self {
            wire_api,
            id: None,
            text: String::new(),
            streamed_text: false,
            first_output_seen: false,
            final_payload: None,
            tools: BTreeMap::new(),
            usage: None,
            anthropic_input_tokens: 0,
            anthropic_cache_creation_input_tokens: 0,
            anthropic_cache_read_input_tokens: 0,
            anthropic_output_tokens: 0,
        }
    }

    fn emit_text(
        &mut self,
        delta: &str,
        events: &SharedModelEventSink,
    ) -> Result<(), ModelError> {
        if delta.is_empty() {
            return Ok(());
        }
        self.first_output_seen = true;
        self.streamed_text = true;
        self.text.push_str(delta);
        events.emit(ModelEvent::OutputTextDelta(delta.to_string()))
    }

    fn tool_mut(&mut self, index: usize) -> &mut StreamToolCall {
        self.first_output_seen = true;
        self.tools.entry(index).or_default()
    }

    fn finish(self) -> Result<(Value, bool), ModelError> {
        if let Some(payload) = self.final_payload {
            validate_response_payload(&payload)?;
            return Ok((
                match self.wire_api {
                    ResponsesWireApi::Responses => payload,
                    ResponsesWireApi::ChatCompletions => normalize_chat_payload(payload),
                    ResponsesWireApi::AnthropicMessages => normalize_anthropic_payload(payload),
                },
                self.streamed_text,
            ));
        }

        let mut output = Vec::new();
        if !self.text.is_empty() {
            output.push(json!({
                "type": "message",
                "role": "assistant",
                "content": [{"type":"output_text", "text":self.text}],
            }));
        }
        for tool in self.tools.into_values() {
            output.push(json!({
                "type": "function_call",
                "call_id": if tool.id.is_empty() { "call" } else { tool.id.as_str() },
                "name": if tool.name.is_empty() { "tool" } else { tool.name.as_str() },
                "arguments": if tool.arguments.is_empty() { "{}" } else { tool.arguments.as_str() },
            }));
        }
        let usage = match self.wire_api {
            ResponsesWireApi::AnthropicMessages => {
                let input_tokens = self
                    .anthropic_input_tokens
                    .saturating_add(self.anthropic_cache_creation_input_tokens);
                json!({
                    "input_tokens": input_tokens,
                    "cached_input_tokens": self.anthropic_cache_read_input_tokens,
                    "output_tokens": self.anthropic_output_tokens,
                    "total_tokens": input_tokens
                        .saturating_add(self.anthropic_cache_read_input_tokens)
                        .saturating_add(self.anthropic_output_tokens),
                })
            }
            _ => self.usage.unwrap_or(Value::Null),
        };
        let payload = json!({
            "id": self.id.unwrap_or_else(|| json!("resp_stream")),
            "object": "response",
            "status": "completed",
            "output": output,
            "usage": usage,
        });
        Ok((payload, self.streamed_text))
    }
}

fn request_stream(
    response: ureq::Response,
    wire_api: ResponsesWireApi,
    events: SharedModelEventSink,
    first_output_timeout: Duration,
) -> Result<(Value, bool), ModelError> {
    let mut reader = BufReader::new(response.into_reader());
    let started = Instant::now();
    let mut data = String::new();
    let mut stream = StreamAccumulator::new(wire_api);

    loop {
        if !stream.first_output_seen && started.elapsed() >= first_output_timeout {
            return Err(ModelError::Inference(format!(
                "model first-output watchdog expired after {} ms",
                first_output_timeout.as_millis()
            )));
        }

        let mut line = String::new();
        let read = reader
            .read_line(&mut line)
            .map_err(|error| ModelError::Inference(format!("model stream read failed: {error}")))?;

        if !stream.first_output_seen && started.elapsed() >= first_output_timeout {
            return Err(ModelError::Inference(format!(
                "model first-output watchdog expired after {} ms",
                first_output_timeout.as_millis()
            )));
        }

        if read == 0 {
            consume_sse_event(&data, &events, &mut stream)?;
            break;
        }
        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            consume_sse_event(&data, &events, &mut stream)?;
            data.clear();
            continue;
        }
        if line.starts_with(':') {
            continue;
        }
        if let Some(value) = line.strip_prefix("data:") {
            if !data.is_empty() {
                data.push('\n');
            }
            data.push_str(value.trim_start());
        }
    }

    stream.finish()
}

fn consume_sse_event(
    data: &str,
    events: &SharedModelEventSink,
    stream: &mut StreamAccumulator,
) -> Result<(), ModelError> {
    let data = data.trim();
    if data.is_empty() || data == "[DONE]" {
        return Ok(());
    }
    let payload: Value = serde_json::from_str(data).map_err(|error| {
        ModelError::Inference(format!("model stream returned invalid JSON: {error}"))
    })?;
    if stream.id.is_none() {
        stream.id = payload
            .get("id")
            .cloned()
            .or_else(|| payload.pointer("/message/id").cloned());
    }
    if let Some(usage) = payload.get("usage") {
        stream.usage = Some(usage.clone());
    }

    let event_type = payload.get("type").and_then(Value::as_str).unwrap_or_default();
    match event_type {
        "response.output_text.delta" => {
            if let Some(delta) = payload.get("delta").and_then(Value::as_str) {
                stream.emit_text(delta, events)?;
            }
        }
        "response.completed" => {
            stream.first_output_seen = true;
            stream.final_payload = payload.get("response").cloned().or(Some(payload));
        }
        "response.failed" | "response.incomplete" => {
            let message = payload
                .pointer("/response/error/message")
                .or_else(|| payload.pointer("/error/message"))
                .and_then(Value::as_str)
                .unwrap_or("model endpoint returned an incomplete response");
            return Err(ModelError::Inference(message.to_string()));
        }
        "message_start" => {
            // Anthropic's message_start only acknowledges stream creation; it
            // is not user-visible model output and must not disarm the
            // first-output watchdog.
            let usage = payload.pointer("/message/usage").unwrap_or(&Value::Null);
            stream.anthropic_input_tokens = usage
                .get("input_tokens")
                .and_then(Value::as_u64)
                .unwrap_or(0);
            stream.anthropic_cache_creation_input_tokens = usage
                .get("cache_creation_input_tokens")
                .and_then(Value::as_u64)
                .unwrap_or(0);
            stream.anthropic_cache_read_input_tokens = usage
                .get("cache_read_input_tokens")
                .and_then(Value::as_u64)
                .unwrap_or(0);
        }
        "content_block_start" => {
            let index = payload.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
            let block = payload.get("content_block").unwrap_or(&Value::Null);
            if block.get("type").and_then(Value::as_str) == Some("tool_use") {
                let tool = stream.tool_mut(index);
                tool.id = block.get("id").and_then(Value::as_str).unwrap_or_default().to_string();
                tool.name = block.get("name").and_then(Value::as_str).unwrap_or_default().to_string();
                if let Some(input) = block.get("input").filter(|value| !value.is_null()) {
                    let encoded = serde_json::to_string(input).unwrap_or_default();
                    if encoded != "{}" {
                        tool.arguments.push_str(&encoded);
                    }
                }
            }
        }
        "content_block_delta" => {
            let index = payload.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
            let delta = payload.get("delta").unwrap_or(&Value::Null);
            match delta.get("type").and_then(Value::as_str) {
                Some("text_delta") => {
                    if let Some(text) = delta.get("text").and_then(Value::as_str) {
                        stream.emit_text(text, events)?;
                    }
                }
                Some("input_json_delta") => {
                    if let Some(fragment) = delta.get("partial_json").and_then(Value::as_str) {
                        stream.tool_mut(index).arguments.push_str(fragment);
                    }
                }
                _ => {}
            }
        }
        "message_delta" => {
            let usage = payload.get("usage").unwrap_or(&Value::Null);
            stream.anthropic_output_tokens = usage
                .get("output_tokens")
                .and_then(Value::as_u64)
                .unwrap_or(stream.anthropic_output_tokens);
        }
        "message_stop" => {}
        _ => {
            if let Some(delta) = payload
                .pointer("/choices/0/delta/content")
                .and_then(Value::as_str)
            {
                stream.emit_text(delta, events)?;
            }
            if let Some(calls) = payload
                .pointer("/choices/0/delta/tool_calls")
                .and_then(Value::as_array)
            {
                for call in calls {
                    let index = call.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
                    let tool = stream.tool_mut(index);
                    if let Some(id) = call.get("id").and_then(Value::as_str) {
                        if tool.id.is_empty() {
                            tool.id.push_str(id);
                        }
                    }
                    if let Some(name) = call.pointer("/function/name").and_then(Value::as_str) {
                        tool.name.push_str(name);
                    }
                    if let Some(arguments) = call.pointer("/function/arguments").and_then(Value::as_str) {
                        tool.arguments.push_str(arguments);
                    }
                }
            }
        }
    }
    Ok(())
}

fn validate_response_payload(payload: &Value) -> Result<(), ModelError> {
    if let Some(error) = payload.get("error") {
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("model endpoint returned an error");
        return Err(ModelError::Inference(message.to_string()));
    }
    Ok(())
}

fn anthropic_messages(input: &Value) -> Vec<Value> {
    let mut messages: Vec<Value> = Vec::new();
    for item in input.as_array().into_iter().flatten() {
        if let Some(role) = item.get("role").and_then(Value::as_str) {
            if role == "system" {
                continue;
            }
            let content = item.get("content").cloned().unwrap_or_else(|| json!(""));
            let content = match content {
                Value::String(text) => json!([{"type":"text", "text":text}]),
                Value::Array(mut parts) => {
                    for part in &mut parts {
                        if matches!(
                            part.get("type").and_then(Value::as_str),
                            Some("input_text" | "output_text")
                        ) {
                            part["type"] = json!("text");
                        }
                    }
                    Value::Array(parts)
                }
                other => json!([{"type":"text", "text":other.to_string()}]),
            };
            messages.push(json!({"role": role, "content": content}));
            continue;
        }
        match item.get("type").and_then(Value::as_str) {
            Some("function_call") | Some("tool_call") => {
                let arguments = item
                    .get("arguments")
                    .or_else(|| item.pointer("/function/arguments"))
                    .cloned()
                    .unwrap_or_else(|| json!({}));
                let arguments = arguments
                    .as_str()
                    .and_then(|value| serde_json::from_str(value).ok())
                    .unwrap_or(arguments);
                let block = json!({
                    "type": "tool_use",
                    "id": item.get("call_id").or_else(|| item.get("id")).cloned().unwrap_or_else(|| json!("call")),
                    "name": item.get("name").or_else(|| item.pointer("/function/name")).cloned().unwrap_or_else(|| json!("tool")),
                    "input": arguments,
                });
                append_anthropic_content(&mut messages, "assistant", block);
            }
            Some("function_call_output") => {
                let content = item
                    .get("output")
                    .map(|value| match value {
                        Value::String(text) => text.clone(),
                        other => serde_json::to_string(other).unwrap_or_else(|_| "null".into()),
                    })
                    .unwrap_or_else(|| "null".into());
                let block = json!({
                    "type": "tool_result",
                    "tool_use_id": item.get("call_id").cloned().unwrap_or_else(|| json!("call")),
                    "content": content,
                });
                append_anthropic_content(&mut messages, "user", block);
            }
            _ => {}
        }
    }
    messages
}

fn anthropic_system(input: &Value, instructions: Option<&Value>) -> String {
    let mut sections = Vec::new();
    if let Some(value) = instructions.and_then(Value::as_str)
        && !value.trim().is_empty()
    {
        sections.push(value.trim().to_owned());
    }
    for item in input.as_array().into_iter().flatten() {
        if item.get("role").and_then(Value::as_str) != Some("system") {
            continue;
        }
        if let Some(value) = item.get("content").and_then(Value::as_str)
            && !value.trim().is_empty()
        {
            sections.push(value.trim().to_owned());
        }
    }
    sections.join("\n\n")
}

fn append_anthropic_content(messages: &mut Vec<Value>, role: &str, block: Value) {
    if let Some(content) = messages
        .last_mut()
        .filter(|message| message.get("role").and_then(Value::as_str) == Some(role))
        .and_then(|message| message.get_mut("content"))
        .and_then(Value::as_array_mut)
    {
        content.push(block);
    } else {
        messages.push(json!({"role": role, "content": [block]}));
    }
}

fn anthropic_tool(tool: &Value) -> Option<Value> {
    let name = tool.get("name").and_then(Value::as_str)?;
    Some(json!({
        "name": name,
        "description": tool.get("description").cloned().unwrap_or_else(|| json!("")),
        "input_schema": tool.get("parameters").cloned().unwrap_or_else(|| json!({"type":"object","properties":{}})),
    }))
}

fn normalize_anthropic_payload(payload: Value) -> Value {
    let mut output = Vec::new();
    for block in payload
        .get("content")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        match block.get("type").and_then(Value::as_str) {
            Some("text") => output.push(json!({
                "type": "message",
                "role": "assistant",
                "content": [{"type":"output_text", "text":block.get("text").cloned().unwrap_or_else(|| json!(""))}],
            })),
            Some("tool_use") => output.push(json!({
                "type": "function_call",
                "call_id": block.get("id").cloned().unwrap_or_else(|| json!("call")),
                "name": block.get("name").cloned().unwrap_or_else(|| json!("tool")),
                "arguments": serde_json::to_string(block.get("input").unwrap_or(&Value::Null)).unwrap_or_else(|_| "{}".into()),
            })),
            _ => {}
        }
    }
    let usage = payload.get("usage").cloned().unwrap_or(Value::Null);
    let uncached_input_tokens = usage
        .get("input_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let cache_creation_input_tokens = usage
        .get("cache_creation_input_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let input_tokens = uncached_input_tokens.saturating_add(cache_creation_input_tokens);
    let output_tokens = usage
        .get("output_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let cached_input_tokens = usage
        .get("cache_read_input_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    json!({
        "id": payload.get("id").cloned().unwrap_or(Value::Null),
        "output": output,
        "usage": {
            "input_tokens": input_tokens,
            "cached_input_tokens": cached_input_tokens,
            "output_tokens": output_tokens,
            "total_tokens": input_tokens.saturating_add(cached_input_tokens).saturating_add(output_tokens),
        },
    })
}

fn chat_messages(input: &Value) -> Vec<Value> {
    let mut messages = Vec::new();
    for item in input.as_array().into_iter().flatten() {
        if let Some(role) = item.get("role").and_then(Value::as_str) {
            let content = item
                .get("content")
                .cloned()
                .unwrap_or(Value::String(String::new()));
            let content = if let Some(parts) = content.as_array() {
                Value::String(
                    parts
                        .iter()
                        .filter_map(|part| part.get("text").and_then(Value::as_str))
                        .collect::<Vec<_>>()
                        .join(""),
                )
            } else {
                content
            };
            messages.push(json!({"role": role, "content": content}));
            continue;
        }
        match item.get("type").and_then(Value::as_str) {
            Some("function_call") | Some("tool_call") => {
                let tool_call = json!({
                    "id": item.get("call_id").or_else(|| item.get("id")).cloned().unwrap_or_else(|| json!("call")),
                    "type": "function",
                    "function": {
                        "name": item.get("name").or_else(|| item.pointer("/function/name")).cloned().unwrap_or_else(|| json!("tool")),
                        "arguments": item.get("arguments").or_else(|| item.pointer("/function/arguments")).cloned().unwrap_or_else(|| json!("{}")),
                    }
                });
                if let Some(calls) = messages
                    .last_mut()
                    .filter(|message| {
                        message.get("role").and_then(Value::as_str) == Some("assistant")
                    })
                    .and_then(|message| message.get_mut("tool_calls"))
                    .and_then(Value::as_array_mut)
                {
                    calls.push(tool_call);
                } else {
                    messages.push(json!({
                        "role": "assistant",
                        "content": Value::Null,
                        "tool_calls": [tool_call]
                    }));
                }
            }
            Some("function_call_output") => messages.push(json!({
                "role": "tool",
                "tool_call_id": item.get("call_id").cloned().unwrap_or_else(|| json!("call")),
                "content": item.get("output").cloned().unwrap_or_else(|| json!("null")),
            })),
            _ => {}
        }
    }
    messages
}

fn chat_tool(tool: &Value) -> Option<Value> {
    let name = tool.get("name").and_then(Value::as_str)?;
    Some(json!({
        "type": "function",
        "function": {
            "name": name,
            "description": tool.get("description").cloned().unwrap_or_else(|| json!("")),
            "parameters": tool.get("parameters").cloned().unwrap_or_else(|| json!({"type":"object","properties":{}})),
        }
    }))
}

fn normalize_chat_payload(payload: Value) -> Value {
    let message = payload
        .pointer("/choices/0/message")
        .cloned()
        .unwrap_or(Value::Null);
    let mut output = Vec::new();
    if let Some(text) = message.get("content").and_then(Value::as_str)
        && !text.is_empty()
    {
        output.push(json!({"type":"message", "role":"assistant", "content":[{"type":"output_text", "text":text}]}));
    }
    for call in message
        .get("tool_calls")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        output.push(json!({
            "type": "function_call",
            "call_id": call.get("id").cloned().unwrap_or_else(|| json!("call")),
            "name": call.pointer("/function/name").cloned().unwrap_or_else(|| json!("tool")),
            "arguments": call.pointer("/function/arguments").cloned().unwrap_or_else(|| json!("{}")),
        }));
    }
    json!({
        "id": payload.get("id").cloned().unwrap_or(Value::Null),
        "output": output,
        "usage": payload.get("usage").cloned().unwrap_or(Value::Null),
    })
}

fn redacted_http_error(error: ureq::Error) -> ModelError {
    match error {
        ureq::Error::Status(status, response) => {
            let payload = response.into_json::<Value>().unwrap_or(Value::Null);
            let message = payload
                .pointer("/error/message")
                .or_else(|| payload.get("message"))
                .and_then(Value::as_str)
                .filter(|message| !message.trim().is_empty())
                .map(str::to_owned)
                .unwrap_or_else(|| format!("model endpoint returned HTTP {status}"));
            ModelError::Inference(message)
        }
        ureq::Error::Transport(error) => {
            ModelError::Inference(format!("model transport failed: {error}"))
        }
    }
}

pub fn extract_output_text(payload: &Value) -> Option<String> {
    if let Some(text) = payload.get("output_text").and_then(Value::as_str)
        && !text.is_empty()
    {
        return Some(text.to_string());
    }
    let output = payload.get("output").and_then(Value::as_array)?;
    let text = output
        .iter()
        .filter_map(|item| item.get("content").and_then(Value::as_array))
        .flatten()
        .filter_map(|content| content.get("text").and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join("");
    (!text.is_empty()).then_some(text)
}

pub fn extract_usage(payload: &Value) -> Option<ModelUsage> {
    let usage = payload
        .get("usage")
        .or_else(|| payload.pointer("/response/usage"))?;
    let input_tokens = usage_value(usage, &["input_tokens", "prompt_tokens", "inputTokens"]);
    let output_tokens = usage_value(
        usage,
        &["output_tokens", "completion_tokens", "outputTokens"],
    );
    let cached_input_tokens = usage_value(usage, &["cached_input_tokens", "cachedInputTokens"])
        .max(
            usage
                .pointer("/input_tokens_details/cached_tokens")
                .and_then(Value::as_u64)
                .unwrap_or(0),
        );
    let reasoning_output_tokens =
        usage_value(usage, &["reasoning_output_tokens", "reasoningOutputTokens"]).max(
            usage
                .pointer("/output_tokens_details/reasoning_tokens")
                .and_then(Value::as_u64)
                .unwrap_or(0),
        );
    let total_tokens = usage_value(usage, &["total_tokens", "totalTokens"])
        .max(input_tokens.saturating_add(output_tokens));
    (total_tokens > 0).then_some(ModelUsage {
        total_tokens,
        input_tokens,
        cached_input_tokens,
        output_tokens,
        reasoning_output_tokens,
    })
}

fn usage_value(usage: &Value, keys: &[&str]) -> u64 {
    keys.iter()
        .find_map(|key| usage.get(*key).and_then(Value::as_u64))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_text_and_usage_from_responses_payload() {
        let payload = json!({
            "output": [{"content": [{"type":"output_text", "text":"hello"}]}],
            "usage": {"input_tokens": 10, "output_tokens": 4, "total_tokens": 14}
        });
        assert_eq!(extract_output_text(&payload).as_deref(), Some("hello"));
        assert_eq!(
            extract_usage(&payload),
            Some(ModelUsage {
                total_tokens: 14,
                input_tokens: 10,
                cached_input_tokens: 0,
                output_tokens: 4,
                reasoning_output_tokens: 0,
            })
        );
    }

    #[test]
    fn remote_config_requires_https_and_rejects_header_injection() {
        let mut config = ResponsesModelConfig {
            base_url: "http://example.test/v1".into(),
            default_model: "model".into(),
            bearer_token: None,
            provider_mode: ModelProviderMode::FirstPartyDacheng,
            wire_api: ResponsesWireApi::Responses,
        };
        assert!(config.validate().is_err());
        config.base_url = "https://example.test/v1".into();
        config.bearer_token = Some("token\nheader".into());
        assert!(config.validate().is_err());
    }

    #[test]
    fn normalizes_chat_completion_text_tools_and_usage() {
        let normalized = normalize_chat_payload(json!({
            "id":"chat-1",
            "choices":[{"message":{"role":"assistant","content":"善哉","tool_calls":[{"id":"call-1","type":"function","function":{"name":"search","arguments":"{\"q\":\"法\"}"}}]}}],
            "usage":{"prompt_tokens":12,"completion_tokens":4,"total_tokens":16}
        }));
        assert_eq!(extract_output_text(&normalized).as_deref(), Some("善哉"));
        assert_eq!(normalized["output"][1]["name"], "search");
        assert_eq!(extract_usage(&normalized).unwrap().total_tokens, 16);
    }

    #[test]
    fn groups_adjacent_tool_calls_into_one_assistant_message() {
        let messages = chat_messages(&json!([
            {"role":"user", "content":"search"},
            {"type":"function_call", "call_id":"call-1", "name":"first", "arguments":"{}"},
            {"type":"function_call", "call_id":"call-2", "name":"second", "arguments":"{}"},
            {"type":"function_call_output", "call_id":"call-1", "output":"one"},
            {"type":"function_call_output", "call_id":"call-2", "output":"two"}
        ]));
        assert_eq!(messages.len(), 4);
        assert_eq!(messages[1]["tool_calls"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn normalizes_anthropic_text_tools_and_cache_usage() {
        let normalized = normalize_anthropic_payload(json!({
            "id":"msg-1",
            "content":[
                {"type":"text","text":"善哉"},
                {"type":"tool_use","id":"tool-1","name":"search","input":{"q":"法"}}
            ],
            "usage":{"input_tokens":10,"cache_creation_input_tokens":2,"cache_read_input_tokens":4,"output_tokens":3}
        }));
        assert_eq!(extract_output_text(&normalized).as_deref(), Some("善哉"));
        assert_eq!(normalized["output"][1]["name"], "search");
        assert_eq!(extract_usage(&normalized).unwrap().input_tokens, 12);
        assert_eq!(extract_usage(&normalized).unwrap().cached_input_tokens, 4);
        assert_eq!(extract_usage(&normalized).unwrap().total_tokens, 19);
    }

    #[derive(Default)]
    struct RecordingSink {
        text: std::sync::Mutex<String>,
    }

    impl ModelEventSink for RecordingSink {
        fn emit(&self, event: ModelEvent) -> Result<(), ModelError> {
            if let ModelEvent::OutputTextDelta(delta) = event {
                self.text.lock().unwrap().push_str(&delta);
            }
            Ok(())
        }
    }

    #[test]
    fn parses_chat_stream_text_and_tool_calls() {
        let sink = Arc::new(RecordingSink::default());
        let events: SharedModelEventSink = sink.clone();
        let mut stream = StreamAccumulator::new(ResponsesWireApi::ChatCompletions);
        consume_sse_event(
            r#"{"id":"chat-1","choices":[{"delta":{"content":"善","tool_calls":[{"index":0,"id":"call-1","function":{"name":"search","arguments":"{\\"q\\":"}}]}}]}"#,
            &events,
            &mut stream,
        )
        .unwrap();
        consume_sse_event(
            r#"{"choices":[{"delta":{"content":"哉","tool_calls":[{"index":0,"function":{"arguments":"\\"法\\"}"}}]}}]}"#,
            &events,
            &mut stream,
        )
        .unwrap();
        let (payload, streamed) = stream.finish().unwrap();
        assert!(streamed);
        assert_eq!(extract_output_text(&payload).as_deref(), Some("善哉"));
        assert_eq!(payload["output"][1]["name"], "search");
        assert_eq!(payload["output"][1]["arguments"], "{\"q\":\"法\"}");
    }

    #[test]
    fn parses_anthropic_stream_text_and_tool_input() {
        let sink = Arc::new(RecordingSink::default());
        let events: SharedModelEventSink = sink.clone();
        let mut stream = StreamAccumulator::new(ResponsesWireApi::AnthropicMessages);
        consume_sse_event(
            r#"{"type":"message_start","message":{"id":"msg-1","usage":{"input_tokens":4}}}"#,
            &events,
            &mut stream,
        )
        .unwrap();
        consume_sse_event(
            r#"{"type":"content_block_start","index":0,"content_block":{"type":"tool_use","id":"tool-1","name":"search","input":{}}}"#,
            &events,
            &mut stream,
        )
        .unwrap();
        consume_sse_event(
            r#"{"type":"content_block_delta","index":0,"delta":{"type":"input_json_delta","partial_json":"{\\"q\\":\\"法\\"}"}}"#,
            &events,
            &mut stream,
        )
        .unwrap();
        let (payload, _) = stream.finish().unwrap();
        assert_eq!(payload["output"][0]["call_id"], "tool-1");
        assert_eq!(payload["output"][0]["name"], "search");
        assert_eq!(payload["output"][0]["arguments"], "{\"q\":\"法\"}");
    }

    #[test]
    fn groups_anthropic_text_with_tool_use_and_serializes_tool_results() {
        let messages = anthropic_messages(&json!([
            {"role":"user", "content":"search"},
            {"role":"assistant", "content":"I will search."},
            {"type":"function_call", "call_id":"tool-1", "name":"search", "arguments":"{\"q\":\"法\"}"},
            {"type":"function_call_output", "call_id":"tool-1", "output":{"ok":true}}
        ]));
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[1]["content"].as_array().unwrap().len(), 2);
        assert_eq!(messages[2]["content"][0]["content"], "{\"ok\":true}");
    }
}
