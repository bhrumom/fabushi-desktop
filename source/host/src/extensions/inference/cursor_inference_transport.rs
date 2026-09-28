use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use prost::Message;
use prost_types::{ListValue, Struct, Value as ProtoValue, value::Kind};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{Value, json};
use url::Url;
use uuid::Uuid;

use crate::cursor_backend::{
    create_cursor_checksum, resolve_sand_ghost_mode_header,
};
use crate::extensions::auth::credential_renewer::{
    SAND_CLIENT_TYPE, get_configured_backend_url, sand_box_namespace, sand_client_version,
};
use crate::extensions::auth::extension::HostAuthExtension;

use super::cursor_session::sand_default_model_selection;
use super::provider_session::{
    ProviderMessage, ProviderSessionError, ProviderTokenUsage, RoutedToolDefinition,
};

pub const CURSOR_INFERENCE_STREAM_PATH: &str =
    "/aiserver.v1.InferenceService/Stream";
pub const CONNECT_STREAM_CONTENT_TYPE: &str = "application/connect+proto";
pub const CONNECT_END_STREAM_FLAG: u8 = 0x02;
pub const CONNECT_COMPRESSED_FLAG: u8 = 0x01;

pub trait CursorInferenceAuth: Send + Sync {
    fn access_token(&self) -> Result<String, String>;
    fn machine_id(&self) -> Result<String, String>;
    fn backend_url(&self) -> Result<String, String> {
        get_configured_backend_url().map_err(|error| error.to_string())
    }
}

impl CursorInferenceAuth for HostAuthExtension {
    fn access_token(&self) -> Result<String, String> {
        self.get_access_token().map_err(|error| error.to_string())
    }

    fn machine_id(&self) -> Result<String, String> {
        self.get_machine_id().map_err(|error| error.to_string())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CursorInferenceEvent {
    TextDelta(String),
    ThinkingDelta {
        text: String,
        signature: Option<String>,
    },
    ToolCall {
        tool_call_id: String,
        tool_name: String,
        args: String,
        complete: bool,
        tool_index: Option<i32>,
    },
    Usage(ProviderTokenUsage),
    Finished,
}

#[derive(Clone, PartialEq, Message)]
struct InferenceStreamRequest {
    #[prost(message, repeated, tag = "1")]
    messages: Vec<InferenceCoreMessage>,
    #[prost(message, repeated, tag = "2")]
    tools: Vec<InferenceAgentTool>,
    #[prost(message, optional, tag = "7")]
    requested_model: Option<InferenceRequestedModel>,
}

#[derive(Clone, PartialEq, Message)]
struct InferenceRequestedModel {
    #[prost(string, tag = "1")]
    model_id: String,
    #[prost(bool, tag = "2")]
    max_mode: bool,
    #[prost(message, repeated, tag = "3")]
    parameters: Vec<InferenceModelParameterValue>,
}

#[derive(Clone, PartialEq, Message)]
struct InferenceModelParameterValue {
    #[prost(string, tag = "1")]
    id: String,
    #[prost(string, tag = "2")]
    value: String,
}

#[derive(Clone, PartialEq, Message)]
struct InferenceCoreMessage {
    #[prost(int32, tag = "1")]
    role: i32,
    #[prost(oneof = "inference_core_message::Content", tags = "2, 6")]
    content: Option<inference_core_message::Content>,
    #[prost(message, repeated, tag = "4")]
    tool_calls: Vec<InferenceToolCall>,
}

mod inference_core_message {
    use prost::Oneof;

    use super::InferenceToolResultContent;

    #[derive(Clone, PartialEq, Oneof)]
    pub enum Content {
        #[prost(string, tag = "2")]
        Text(String),
        #[prost(message, tag = "6")]
        ToolContent(InferenceToolResultContent),
    }
}

#[derive(Clone, PartialEq, Message)]
struct InferenceToolCall {
    #[prost(string, tag = "1")]
    tool_call_id: String,
    #[prost(string, tag = "2")]
    tool_name: String,
    #[prost(message, optional, tag = "3")]
    args: Option<Struct>,
    #[prost(string, optional, tag = "4")]
    raw_tool_call_args: Option<String>,
}

#[derive(Clone, PartialEq, Message)]
struct InferenceAgentTool {
    #[prost(string, tag = "1")]
    name: String,
    #[prost(string, tag = "2")]
    description: String,
    #[prost(message, optional, tag = "3")]
    parameters: Option<Struct>,
}

#[derive(Clone, PartialEq, Message)]
struct InferenceToolResultContent {
    #[prost(message, repeated, tag = "1")]
    parts: Vec<InferenceToolResultPart>,
}

#[derive(Clone, PartialEq, Message)]
struct InferenceToolResultPart {
    #[prost(string, tag = "1")]
    tool_call_id: String,
    #[prost(string, tag = "2")]
    tool_name: String,
    #[prost(message, optional, tag = "3")]
    result: Option<ProtoValue>,
    #[prost(bool, tag = "4")]
    is_error: bool,
}

#[derive(Clone, PartialEq, Message)]
struct InferenceStreamResponse {
    #[prost(oneof = "inference_stream_response::Response", tags = "1, 2, 3, 5, 8, 9")]
    response: Option<inference_stream_response::Response>,
}

mod inference_stream_response {
    use prost::Oneof;

    use super::{
        InferenceExtendedUsageInfo, InferenceStreamError, InferenceTextStreamPart,
        InferenceThinkingStreamPart, InferenceToolCallStreamPart, InferenceUsageInfo,
    };

    #[derive(Clone, PartialEq, Oneof)]
    pub enum Response {
        #[prost(message, tag = "1")]
        TextPart(InferenceTextStreamPart),
        #[prost(message, tag = "2")]
        ToolCallPart(InferenceToolCallStreamPart),
        #[prost(message, tag = "3")]
        Usage(InferenceUsageInfo),
        #[prost(message, tag = "5")]
        ExtendedUsage(InferenceExtendedUsageInfo),
        #[prost(message, tag = "8")]
        Error(InferenceStreamError),
        #[prost(message, tag = "9")]
        ThinkingPart(InferenceThinkingStreamPart),
    }
}

#[derive(Clone, PartialEq, Message)]
struct InferenceTextStreamPart {
    #[prost(string, tag = "1")]
    text: String,
    #[prost(bool, tag = "2")]
    is_final: bool,
}

#[derive(Clone, PartialEq, Message)]
struct InferenceThinkingStreamPart {
    #[prost(string, tag = "1")]
    text: String,
    #[prost(string, optional, tag = "2")]
    signature: Option<String>,
    #[prost(bool, tag = "3")]
    is_final: bool,
}

#[derive(Clone, PartialEq, Message)]
struct InferenceToolCallStreamPart {
    #[prost(string, tag = "1")]
    tool_call_id: String,
    #[prost(string, tag = "2")]
    tool_name: String,
    #[prost(string, tag = "3")]
    args: String,
    #[prost(bool, tag = "4")]
    is_complete: bool,
    #[prost(int32, optional, tag = "5")]
    tool_index: Option<i32>,
}

#[derive(Clone, PartialEq, Message)]
struct InferenceUsageInfo {
    #[prost(int32, tag = "1")]
    prompt_tokens: i32,
    #[prost(int32, tag = "2")]
    completion_tokens: i32,
    #[prost(int32, optional, tag = "3")]
    total_tokens: Option<i32>,
}

#[derive(Clone, PartialEq, Message)]
struct InferenceExtendedUsageInfo {
    #[prost(int32, tag = "1")]
    input_tokens: i32,
    #[prost(int32, tag = "2")]
    output_tokens: i32,
    #[prost(int32, tag = "3")]
    cache_read_tokens: i32,
    #[prost(int32, tag = "4")]
    cache_write_tokens: i32,
    #[prost(int32, tag = "5")]
    max_tokens: i32,
}

#[derive(Clone, PartialEq, Message)]
struct InferenceStreamError {
    #[prost(string, tag = "1")]
    message: String,
    #[prost(string, tag = "2")]
    code: String,
    #[prost(bool, tag = "3")]
    is_input_token_limit_error: bool,
    #[prost(bool, tag = "4")]
    is_output_token_limit_error: bool,
    #[prost(int32, tag = "5")]
    error_type: i32,
}

fn non_negative(value: i32) -> u64 {
    u64::try_from(value).unwrap_or_default()
}

fn response_event(response: InferenceStreamResponse) -> Result<Option<CursorInferenceEvent>, ProviderSessionError> {
    use inference_stream_response::Response;

    match response.response {
        Some(Response::TextPart(part)) if part.is_final => Ok(Some(CursorInferenceEvent::Finished)),
        Some(Response::TextPart(part)) if !part.text.is_empty() => {
            Ok(Some(CursorInferenceEvent::TextDelta(part.text)))
        }
        Some(Response::ThinkingPart(part)) if !part.text.is_empty() || part.signature.is_some() => {
            Ok(Some(CursorInferenceEvent::ThinkingDelta {
                text: part.text,
                signature: part.signature,
            }))
        }
        Some(Response::ToolCallPart(part)) => Ok(Some(CursorInferenceEvent::ToolCall {
            tool_call_id: part.tool_call_id,
            tool_name: part.tool_name,
            args: part.args,
            complete: part.is_complete,
            tool_index: part.tool_index,
        })),
        Some(Response::ExtendedUsage(usage)) => Ok(Some(CursorInferenceEvent::Usage(
            ProviderTokenUsage {
                input_tokens: non_negative(usage.input_tokens),
                output_tokens: non_negative(usage.output_tokens),
                cache_read_tokens: non_negative(usage.cache_read_tokens),
                cache_write_tokens: non_negative(usage.cache_write_tokens),
                reasoning_tokens: None,
            },
        ))),
        Some(Response::Usage(usage)) => Ok(Some(CursorInferenceEvent::Usage(
            ProviderTokenUsage {
                input_tokens: non_negative(usage.prompt_tokens),
                output_tokens: non_negative(usage.completion_tokens),
                cache_read_tokens: 0,
                cache_write_tokens: 0,
                reasoning_tokens: None,
            },
        ))),
        Some(Response::Error(error)) => Err(ProviderSessionError::Protocol(format!(
            "Cursor inference stream failed code={} type={}: {}",
            error.code, error.error_type, error.message
        ))),
        _ => Ok(None),
    }
}

pub fn encode_connect_envelope(message: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(message.len() + 5);
    frame.push(0);
    frame.extend_from_slice(&(message.len() as u32).to_be_bytes());
    frame.extend_from_slice(message);
    frame
}

pub fn decode_cursor_connect_frames(
    buffer: &mut Vec<u8>,
) -> Result<Vec<CursorInferenceEvent>, ProviderSessionError> {
    let mut events = Vec::new();
    let mut consumed = 0usize;
    while buffer.len().saturating_sub(consumed) >= 5 {
        let flags = buffer[consumed];
        let length = u32::from_be_bytes([
            buffer[consumed + 1],
            buffer[consumed + 2],
            buffer[consumed + 3],
            buffer[consumed + 4],
        ]) as usize;
        if buffer.len().saturating_sub(consumed + 5) < length {
            break;
        }
        let payload_start = consumed + 5;
        let payload_end = payload_start + length;
        let payload = &buffer[payload_start..payload_end];

        if flags & CONNECT_COMPRESSED_FLAG != 0 {
            return Err(ProviderSessionError::Protocol(
                "Cursor inference returned a compressed Connect envelope without a negotiated encoding"
                    .into(),
            ));
        }
        if flags & CONNECT_END_STREAM_FLAG != 0 {
            if !payload.is_empty() {
                let metadata: Value = serde_json::from_slice(payload).map_err(|error| {
                    ProviderSessionError::Protocol(format!(
                        "Cursor inference returned invalid Connect end-stream metadata: {error}"
                    ))
                })?;
                if let Some(error) = metadata.get("error") {
                    return Err(ProviderSessionError::Protocol(format!(
                        "Cursor inference Connect stream ended with error: {error}"
                    )));
                }
            }
            consumed = payload_end;
            continue;
        }
        if flags != 0 {
            return Err(ProviderSessionError::Protocol(format!(
                "Cursor inference returned unsupported Connect envelope flags: 0x{flags:02x}"
            )));
        }

        let response = InferenceStreamResponse::decode(payload).map_err(|error| {
            ProviderSessionError::Protocol(format!(
                "Cursor inference returned invalid protobuf response: {error}"
            ))
        })?;
        if let Some(event) = response_event(response)? {
            events.push(event);
        }
        consumed = payload_end;
    }
    if consumed > 0 {
        buffer.drain(..consumed);
    }
    Ok(events)
}

fn json_to_proto(value: &Value) -> ProtoValue {
    let kind = match value {
        Value::Null => Kind::NullValue(0),
        Value::Bool(value) => Kind::BoolValue(*value),
        Value::Number(value) => Kind::NumberValue(value.as_f64().unwrap_or_default()),
        Value::String(value) => Kind::StringValue(value.clone()),
        Value::Array(values) => Kind::ListValue(ListValue {
            values: values.iter().map(json_to_proto).collect(),
        }),
        Value::Object(values) => Kind::StructValue(Struct {
            fields: values
                .iter()
                .map(|(key, value)| (key.clone(), json_to_proto(value)))
                .collect::<BTreeMap<_, _>>(),
        }),
    };
    ProtoValue { kind: Some(kind) }
}

fn json_object_to_proto(value: &Value) -> Struct {
    match json_to_proto(value).kind {
        Some(Kind::StructValue(value)) => value,
        _ => Struct::default(),
    }
}

fn provider_message_to_proto(message: &ProviderMessage) -> InferenceCoreMessage {
    let role = match message.role.as_str() {
        "user" => 1,
        "assistant" => 2,
        "tool" => 3,
        "system" => 4,
        _ => 0,
    };
    InferenceCoreMessage {
        role,
        content: Some(inference_core_message::Content::Text(message.content.clone())),
        tool_calls: Vec::new(),
    }
}

pub fn encode_cursor_inference_request(
    messages: &[ProviderMessage],
    tools: &[RoutedToolDefinition],
) -> Result<Vec<u8>, ProviderSessionError> {
    let model = sand_default_model_selection();
    let request = InferenceStreamRequest {
        messages: messages.iter().map(provider_message_to_proto).collect(),
        tools: tools
            .iter()
            .map(|tool| InferenceAgentTool {
                name: tool.name.clone(),
                description: tool.description.clone().unwrap_or_default(),
                parameters: Some(json_object_to_proto(&tool.input_schema)),
            })
            .collect(),
        requested_model: Some(InferenceRequestedModel {
            model_id: model.model_id,
            max_mode: model.max_mode,
            parameters: model
                .parameters
                .into_iter()
                .map(|parameter| InferenceModelParameterValue {
                    id: parameter.id,
                    value: parameter.value,
                })
                .collect(),
        }),
    };
    let mut payload = Vec::with_capacity(request.encoded_len());
    request.encode(&mut payload).map_err(|error| {
        ProviderSessionError::Protocol(format!(
            "could not encode Cursor inference protobuf request: {error}"
        ))
    })?;
    Ok(encode_connect_envelope(&payload))
}

pub struct CursorInferenceTransport {
    auth: Arc<dyn CursorInferenceAuth>,
}

impl CursorInferenceTransport {
    pub fn new(auth: Arc<dyn CursorInferenceAuth>) -> Self {
        Self { auth }
    }

    pub fn stream(
        &self,
        messages: &[ProviderMessage],
        tools: &[RoutedToolDefinition],
        should_cancel: &dyn Fn() -> bool,
        on_event: &mut dyn FnMut(CursorInferenceEvent) -> Result<(), ProviderSessionError>,
    ) -> Result<(), ProviderSessionError> {
        if should_cancel() {
            return Err(ProviderSessionError::Cancelled(
                "Runner cancelled Cursor inference before dispatch".into(),
            ));
        }

        let backend_url = self.auth.backend_url().map_err(ProviderSessionError::Authentication)?;
        let access_token = self.auth.access_token().map_err(ProviderSessionError::Authentication)?;
        let machine_id = self.auth.machine_id().map_err(ProviderSessionError::Authentication)?;
        let ghost_mode = resolve_sand_ghost_mode_header(
            &backend_url,
            &access_token,
            &machine_id,
        );
        let request_id = Uuid::new_v4().to_string();
        let body = encode_cursor_inference_request(messages, tools)?;

        let base = Url::parse(&backend_url)
            .map_err(|error| ProviderSessionError::Configuration(error.to_string()))?;
        let url = base
            .join(CURSOR_INFERENCE_STREAM_PATH)
            .map_err(|error| ProviderSessionError::Configuration(error.to_string()))?;
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30 * 60))
            .build()
            .map_err(|error| ProviderSessionError::Transport(error.to_string()))?;
        let request = client
            .post(url)
            .header(CONTENT_TYPE, CONNECT_STREAM_CONTENT_TYPE)
            .header(ACCEPT, CONNECT_STREAM_CONTENT_TYPE)
            .header("connect-protocol-version", "1")
            .header(AUTHORIZATION, format!("Bearer {access_token}"))
            .header(
                "x-cursor-checksum",
                create_cursor_checksum(&machine_id, crate::extensions::auth::credential_renewer::system_now_ms()),
            )
            .header("x-cursor-client-type", SAND_CLIENT_TYPE)
            .header("x-cursor-client-version", sand_client_version())
            .header("x-sand-box-namespace", sand_box_namespace())
            .header("x-ghost-mode", ghost_mode)
            .header("x-request-id", request_id)
            .body(body);

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| ProviderSessionError::Transport(error.to_string()))?;
        runtime.block_on(async {
            let response = request
                .send()
                .await
                .map_err(|error| ProviderSessionError::Transport(error.to_string()))?;
            let status = response.status();
            if !status.is_success() {
                let body = response.text().await.unwrap_or_default();
                return Err(ProviderSessionError::Transport(format!(
                    "Cursor inference returned HTTP {}: {}",
                    status.as_u16(),
                    body.chars().take(512).collect::<String>(),
                )));
            }

            let mut stream = response.bytes_stream();
            let mut buffer = Vec::new();
            loop {
                if should_cancel() {
                    return Err(ProviderSessionError::Cancelled(
                        "Runner cancelled Cursor inference stream".into(),
                    ));
                }
                let next = tokio::select! {
                    next = stream.next() => next,
                    _ = tokio::time::sleep(Duration::from_millis(10)) => continue,
                };
                let Some(next) = next else { break };
                let chunk = next
                    .map_err(|error| ProviderSessionError::Transport(error.to_string()))?;
                buffer.extend_from_slice(&chunk);
                for event in decode_cursor_connect_frames(&mut buffer)? {
                    on_event(event)?;
                }
            }
            if !buffer.is_empty() {
                return Err(ProviderSessionError::Protocol(
                    "Cursor inference stream ended with a truncated Connect envelope".into(),
                ));
            }
            Ok(())
        })
    }
}

pub fn cursor_tool_result_message(
    tool_call_id: impl Into<String>,
    tool_name: impl Into<String>,
    result: &Value,
    is_error: bool,
) -> Vec<u8> {
    let message = InferenceCoreMessage {
        role: 3,
        content: Some(inference_core_message::Content::ToolContent(
            InferenceToolResultContent {
                parts: vec![InferenceToolResultPart {
                    tool_call_id: tool_call_id.into(),
                    tool_name: tool_name.into(),
                    result: Some(json_to_proto(result)),
                    is_error,
                }],
            },
        )),
        tool_calls: Vec::new(),
    };
    message.encode_to_vec()
}

#[doc(hidden)]
pub fn encode_test_response(event: CursorInferenceEvent) -> Vec<u8> {
    use inference_stream_response::Response;

    let response = match event {
        CursorInferenceEvent::TextDelta(text) => InferenceStreamResponse {
            response: Some(Response::TextPart(InferenceTextStreamPart {
                text,
                is_final: false,
            })),
        },
        CursorInferenceEvent::ThinkingDelta { text, signature } => InferenceStreamResponse {
            response: Some(Response::ThinkingPart(InferenceThinkingStreamPart {
                text,
                signature,
                is_final: false,
            })),
        },
        CursorInferenceEvent::ToolCall {
            tool_call_id,
            tool_name,
            args,
            complete,
            tool_index,
        } => InferenceStreamResponse {
            response: Some(Response::ToolCallPart(InferenceToolCallStreamPart {
                tool_call_id,
                tool_name,
                args,
                is_complete: complete,
                tool_index,
            })),
        },
        CursorInferenceEvent::Usage(usage) => InferenceStreamResponse {
            response: Some(Response::ExtendedUsage(InferenceExtendedUsageInfo {
                input_tokens: i32::try_from(usage.input_tokens).unwrap_or(i32::MAX),
                output_tokens: i32::try_from(usage.output_tokens).unwrap_or(i32::MAX),
                cache_read_tokens: i32::try_from(usage.cache_read_tokens).unwrap_or(i32::MAX),
                cache_write_tokens: i32::try_from(usage.cache_write_tokens).unwrap_or(i32::MAX),
                max_tokens: 0,
            })),
        },
        CursorInferenceEvent::Finished => InferenceStreamResponse {
            response: Some(Response::TextPart(InferenceTextStreamPart {
                text: String::new(),
                is_final: true,
            })),
        },
    };
    encode_connect_envelope(&response.encode_to_vec())
}
