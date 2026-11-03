use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use mahayana_host_runtime::r#box::box_remote_accessor::{
    BoxEndpoint, BoxRemoteExecClient, BoxRemoteExecError,
};
use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderSessionError, RoutedToolDefinition,
};
use mahayana_host_runtime::runner::routed_provider_runtime::RoutedToolBridge;
use mahayana_host_runtime::r#box::box_file_transfer::{
    FileTransferAccessor, WriteExecResult,
};
use mahayana_host_runtime::r#box::box_shell_command::{
    HostShellArgs, HostShellArgsInput, build_host_shell_args,
};
use mahayana_host_runtime::r#box::box_windows::{
    ShellAccessor, ShellExecutionOutcome,
};
use mahayana_host_runtime::r#box::generated_production::{
    BACKGROUND_SHELL_SPAWN_FIELD_NUMBER, CONNECT_STREAM_CONTENT_TYPE, EXEC_PATH, PING_PATH,
    ProductionBackgroundShellSpawnArgs, ProductionBackgroundShellSpawnResult,
    ProductionBoxExecError, ProductionBoxResourceAccessor, ProductionBoxTransport,
    ProductionBoxTransportError, ProductionExecRequest, ProductionReadArgs,
    ProductionReadOutput, ProductionReadResult, create_production_box_exec_client,
    create_production_box_resource_accessor, PRODUCTION_BOX_RPC_TIMEOUT_MS,
    PRODUCTION_BOX_EXEC_TIMEOUT_MS,
    ProductionShellResult, ProductionShellStreamArgs, ProductionShellStreamEvent,
    SHELL_BACKGROUND_REASON_TIMEOUT,
};
use mahayana_host_runtime::r#box::box_factory::{
    format_sand_box_startup_summary, should_apply_shared_desktop,
};
use mahayana_host_runtime::r#box::loopback_sand_box::{
    LoopbackSandBox, LoopbackSandBoxOptions,
};
use mahayana_host_runtime::r#box::production::ProductionBoxEnvironment;
use mahayana_host_runtime::extensions::box_lifecycle::RecreateSandBoxResponse;
use mahayana_host_runtime::extensions::forever_box::{
    ForeverBoxLifecycle, ForeverBoxRunnerResourcePort, ForeverBoxService, HostBox,
};
use mahayana_host_runtime::runner::background_work::RunnerBackgroundShellWatches;
use mahayana_host_runtime::runner::box_tool_access::{
    RunnerBoxReadRequest, RunnerBoxResourcePort, RunnerBoxShellRequest, RunnerBoxToolBridge,
    RunnerBoxWriteRequest, RUNNER_BOX_SHELL_TOOL_NAME,
};

fn encode_varint(mut value: u64, out: &mut Vec<u8>) {
    while value >= 0x80 {
        out.push(((value as u8) & 0x7f) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

fn push_key(field: u32, wire: u8, out: &mut Vec<u8>) {
    encode_varint((u64::from(field) << 3) | u64::from(wire), out);
}

fn push_varint(field: u32, value: u64, out: &mut Vec<u8>) {
    push_key(field, 0, out);
    encode_varint(value, out);
}

fn push_len(field: u32, value: &[u8], out: &mut Vec<u8>) {
    push_key(field, 2, out);
    encode_varint(value.len() as u64, out);
    out.extend_from_slice(value);
}

fn connect_envelope(flags: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len() + 5);
    out.push(flags);
    out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    out.extend_from_slice(payload);
    out
}

fn background_shell_spawn_element(result: &[u8]) -> Vec<u8> {
    let mut client_message = Vec::new();
    push_len(BACKGROUND_SHELL_SPAWN_FIELD_NUMBER, result, &mut client_message);
    let mut element = Vec::new();
    push_len(1, &client_message, &mut element);
    element
}

fn background_shell_spawn_success(
    shell_id: u32,
    command: &str,
    working_directory: &str,
    pid: Option<u32>,
) -> Vec<u8> {
    let mut success = Vec::new();
    push_varint(1, u64::from(shell_id), &mut success);
    push_len(2, command.as_bytes(), &mut success);
    push_len(3, working_directory.as_bytes(), &mut success);
    if let Some(pid) = pid {
        push_varint(4, u64::from(pid), &mut success);
    }
    let mut result = Vec::new();
    push_len(1, &success, &mut result);
    background_shell_spawn_element(&result)
}

fn background_shell_spawn_error(
    case: u32,
    command: &str,
    working_directory: &str,
    detail: &str,
) -> Vec<u8> {
    let mut nested = Vec::new();
    push_len(1, command.as_bytes(), &mut nested);
    push_len(2, working_directory.as_bytes(), &mut nested);
    push_len(3, detail.as_bytes(), &mut nested);
    if case == 3 || case == 4 {
        push_varint(4, 1, &mut nested);
    }
    let mut result = Vec::new();
    push_len(case, &nested, &mut result);
    background_shell_spawn_element(&result)
}

fn background_shell_spawn_sandbox_unsupported(
    command: &str,
    working_directory: &str,
    policy_type: &str,
    reason: &str,
) -> Vec<u8> {
    let mut nested = Vec::new();
    push_len(1, command.as_bytes(), &mut nested);
    push_len(2, working_directory.as_bytes(), &mut nested);
    push_len(3, policy_type.as_bytes(), &mut nested);
    push_len(4, reason.as_bytes(), &mut nested);
    push_varint(5, 1, &mut nested);
    let mut result = Vec::new();
    push_len(5, &nested, &mut result);
    background_shell_spawn_element(&result)
}

fn shell_stream_element(event_case: u32, nested: &[u8]) -> Vec<u8> {
    let mut shell_stream = Vec::new();
    push_len(event_case, nested, &mut shell_stream);
    let mut client_message = Vec::new();
    push_len(14, &shell_stream, &mut client_message);
    let mut element = Vec::new();
    push_len(1, &client_message, &mut element);
    element
}

fn shell_stream_start_element() -> Vec<u8> {
    shell_stream_element(4, &[])
}

fn shell_stream_stdout_element(data: &str) -> Vec<u8> {
    let mut stdout = Vec::new();
    push_len(1, data.as_bytes(), &mut stdout);
    shell_stream_element(1, &stdout)
}

fn shell_stream_backgrounded_element(
    shell_id: u32,
    command: &str,
    working_directory: &str,
    pid: Option<u32>,
    ms_to_wait: Option<i32>,
    reason: Option<i32>,
) -> Vec<u8> {
    let mut backgrounded = Vec::new();
    push_varint(1, u64::from(shell_id), &mut backgrounded);
    push_len(2, command.as_bytes(), &mut backgrounded);
    push_len(3, working_directory.as_bytes(), &mut backgrounded);
    if let Some(pid) = pid {
        push_varint(4, u64::from(pid), &mut backgrounded);
    }
    if let Some(ms_to_wait) = ms_to_wait {
        push_varint(5, ms_to_wait as u32 as u64, &mut backgrounded);
    }
    if let Some(reason) = reason {
        push_varint(6, reason as u32 as u64, &mut backgrounded);
    }
    shell_stream_element(7, &backgrounded)
}

fn shell_success_element(stderr: &str) -> Vec<u8> {
    shell_success_element_with_output("", stderr)
}

fn shell_success_element_with_output(stdout: &str, stderr: &str) -> Vec<u8> {
    let mut success = Vec::new();
    push_varint(3, 0, &mut success);
    push_len(5, stdout.as_bytes(), &mut success);
    push_len(6, stderr.as_bytes(), &mut success);

    let mut shell_result = Vec::new();
    push_len(1, &success, &mut shell_result);

    let mut client_message = Vec::new();
    push_len(2, &shell_result, &mut client_message);

    let mut element = Vec::new();
    push_len(1, &client_message, &mut element);
    element
}

fn write_success_element() -> Vec<u8> {
    let mut write_result = Vec::new();
    push_len(1, &[], &mut write_result);

    let mut client_message = Vec::new();
    push_len(3, &write_result, &mut client_message);

    let mut element = Vec::new();
    push_len(1, &client_message, &mut element);
    element
}

fn read_success_element(path: &str, content: &str) -> Vec<u8> {
    let mut success = Vec::new();
    push_len(1, path.as_bytes(), &mut success);
    push_len(2, content.as_bytes(), &mut success);
    push_varint(3, 2, &mut success);
    push_varint(4, content.len() as u64, &mut success);
    push_varint(6, 0, &mut success);
    push_varint(8, 0, &mut success);

    let mut read_result = Vec::new();
    push_len(1, &success, &mut read_result);

    let mut client_message = Vec::new();
    push_len(7, &read_result, &mut client_message);

    let mut element = Vec::new();
    push_len(1, &client_message, &mut element);
    element
}

fn computer_use_success_element() -> Vec<u8> {
    let mut computer_result = Vec::new();
    push_len(1, &[], &mut computer_result);

    let mut client_message = Vec::new();
    push_len(22, &computer_result, &mut client_message);

    let mut element = Vec::new();
    push_len(1, &client_message, &mut element);
    element
}

fn raw_resource_element(field_number: u32, protobuf_result: &[u8]) -> Vec<u8> {
    let mut client_message = Vec::new();
    push_len(field_number, protobuf_result, &mut client_message);

    let mut element = Vec::new();
    push_len(1, &client_message, &mut element);
    element
}

fn read_request(stream: &mut TcpStream) -> Vec<u8> {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("read timeout");
    let mut request = Vec::new();
    let mut buffer = [0u8; 4096];
    let mut expected_total = None;
    loop {
        let read = stream.read(&mut buffer).expect("read request");
        assert!(read > 0, "client closed before request was complete");
        request.extend_from_slice(&buffer[..read]);
        if expected_total.is_none() {
            if let Some(header_end) = request
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
            {
                let headers = std::str::from_utf8(&request[..header_end])
                    .expect("UTF-8 request headers");
                let content_length = headers
                    .lines()
                    .find_map(|line| {
                        line.split_once(':').and_then(|(name, value)| {
                            name.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse::<usize>().expect("content length"))
                        })
                    })
                    .expect("content length header");
                expected_total = Some(header_end + 4 + content_length);
            }
        }
        if expected_total.is_some_and(|expected| request.len() >= expected) {
            return request;
        }
    }
}

fn serve_exec_response(
    listener: &TcpListener,
    expected_fragment: &[u8],
    element: Vec<u8>,
) {
    let (mut stream, _) = listener.accept().expect("accept ExecService request");
    let request = read_request(&mut stream);
    let header_end = request
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .expect("header end");
    let headers = std::str::from_utf8(&request[..header_end]).expect("headers");
    assert!(headers.starts_with(&format!("POST {EXEC_PATH} HTTP/1.1\r\n")));
    assert!(
        headers
            .lines()
            .any(|line| line.eq_ignore_ascii_case("Authorization: Bearer secret"))
    );
    assert!(
        headers.lines().any(|line| {
            line.eq_ignore_ascii_case(&format!(
                "Content-Type: {CONNECT_STREAM_CONTENT_TYPE}"
            ))
        })
    );

    let body = &request[header_end + 4..];
    assert_eq!(body.first().copied(), Some(0), "Connect data envelope");
    if expected_fragment == b"printf background-contract" {
        assert!(
            body.windows(2).any(|window| window == [0x82, 0x01]),
            "BackgroundShellSpawn request must use frozen ExecServerMessage field 16"
        );
    }
    assert!(
        body.windows(expected_fragment.len())
            .any(|window| window == expected_fragment),
        "request protobuf did not contain expected payload"
    );

    let identity = exec_request_identity(&request);
    let element = echo_response_identity(&element, &identity);
    let connect_body = complete_connect_body(&[element], identity.id);

    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: {CONNECT_STREAM_CONTENT_TYPE}\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n{:X}\r\n",
        connect_body.len()
    )
    .expect("write response headers");
    stream.write_all(&connect_body).expect("write Connect body");
    stream.write_all(b"\r\n0\r\n\r\n").expect("finish chunks");
    stream.flush().expect("flush response");
}

fn serve_exec_responses(
    listener: &TcpListener,
    expected_fragment: &[u8],
    elements: Vec<Vec<u8>>,
) {
    let (mut stream, _) = listener.accept().expect("accept ExecService request");
    let request = read_request(&mut stream);
    let header_end = request
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .expect("header end");
    let headers = std::str::from_utf8(&request[..header_end]).expect("headers");
    assert!(headers.starts_with(&format!("POST {EXEC_PATH} HTTP/1.1\r\n")));
    assert!(
        headers
            .lines()
            .any(|line| line.eq_ignore_ascii_case("Authorization: Bearer secret"))
    );
    assert!(
        headers.lines().any(|line| {
            line.eq_ignore_ascii_case(&format!(
                "Content-Type: {CONNECT_STREAM_CONTENT_TYPE}"
            ))
        })
    );

    let body = &request[header_end + 4..];
    assert_eq!(body.first().copied(), Some(0), "Connect data envelope");
    assert!(
        body.windows(expected_fragment.len())
            .any(|window| window == expected_fragment),
        "request protobuf did not contain expected payload"
    );
    assert!(
        body.contains(&0x72),
        "ShellStream request must use frozen ExecServerMessage field 14"
    );
    assert!(
        body.windows(2).any(|window| window == [0x68, 0x02]),
        "ShellStream timeout_behavior must be frozen BACKGROUND=2"
    );

    let identity = exec_request_identity(&request);
    let elements = elements.into_iter()
        .map(|element| echo_response_identity(&element, &identity))
        .collect::<Vec<_>>();
    let connect_body = complete_connect_body(&elements, identity.id);

    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: {CONNECT_STREAM_CONTENT_TYPE}\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n{:X}\r\n",
        connect_body.len()
    )
    .expect("write response headers");
    stream.write_all(&connect_body).expect("write Connect body");
    stream.write_all(b"\r\n0\r\n\r\n").expect("finish chunks");
    stream.flush().expect("flush response");
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RequestIdentity {
    id: u64,
    exec_id: String,
}

fn fixture_varint(input: &[u8], cursor: &mut usize) -> u64 {
    let mut value = 0u64;
    for shift in (0..=63).step_by(7) {
        let byte = *input.get(*cursor).expect("complete fixture protobuf varint");
        *cursor += 1;
        assert!(shift != 63 || byte <= 1, "fixture varint overflow");
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return value;
        }
    }
    panic!("unterminated fixture varint");
}

fn fixture_field<'a>(input: &'a [u8], cursor: &mut usize) -> (u64, u8, &'a [u8]) {
    let key = fixture_varint(input, cursor);
    let wire = (key & 7) as u8;
    let start = *cursor;
    let value = match wire {
        0 => {
            fixture_varint(input, cursor);
            &input[start..*cursor]
        }
        1 | 5 => {
            *cursor += if wire == 1 { 8 } else { 4 };
            &input[start..*cursor]
        }
        2 => {
            let length = fixture_varint(input, cursor) as usize;
            let start = *cursor;
            *cursor += length;
            &input[start..*cursor]
        }
        _ => panic!("unsupported fixture wire type"),
    };
    (key >> 3, wire, value)
}

fn fixture_message_field(input: &[u8], wanted: u64) -> &[u8] {
    let mut cursor = 0;
    while cursor < input.len() {
        let (field, wire, value) = fixture_field(input, &mut cursor);
        if field == wanted {
            assert_eq!(wire, 2, "fixture message field");
            return value;
        }
    }
    panic!("fixture field {wanted} is absent");
}

fn exec_request_identity(request: &[u8]) -> RequestIdentity {
    let header_end = request.windows(4).position(|window| window == b"\r\n\r\n")
        .expect("request header end");
    let headers = std::str::from_utf8(&request[..header_end]).expect("request headers");
    assert!(headers.starts_with(&format!("POST {EXEC_PATH} HTTP/1.1\r\n")));
    let body = &request[header_end + 4..];
    assert!(body.len() >= 5);
    assert_eq!(body[0], 0, "single uncompressed request envelope");
    let length = u32::from_be_bytes(body[1..5].try_into().expect("frame length")) as usize;
    assert_eq!(body.len(), length + 5, "exact request frame length");
    let mut identity = RequestIdentity { id: 0, exec_id: String::new() };
    let protobuf = &body[5..];
    let mut cursor = 0;
    while cursor < protobuf.len() {
        let (field, wire, value) = fixture_field(protobuf, &mut cursor);
        match field {
            1 => {
                assert_eq!(wire, 0);
                identity.id = fixture_varint(value, &mut 0);
            }
            15 => {
                assert_eq!(wire, 2);
                identity.exec_id = std::str::from_utf8(value).expect("request exec_id").into();
            }
            _ => {}
        }
    }
    identity
}

fn data_element(client: &[u8]) -> Vec<u8> {
    let mut element = Vec::new();
    push_len(1, client, &mut element);
    element
}

fn echo_response_identity(element: &[u8], identity: &RequestIdentity) -> Vec<u8> {
    let payload = fixture_message_field(element, 1);
    let mut client = Vec::new();
    if identity.id != 0 {
        push_varint(1, identity.id, &mut client);
    }
    if !identity.exec_id.is_empty() {
        push_len(15, identity.exec_id.as_bytes(), &mut client);
    }
    client.extend_from_slice(payload);
    data_element(&client)
}

fn control_element(case: u32, id: Option<u64>) -> Vec<u8> {
    let mut nested = Vec::new();
    if let Some(id) = id {
        push_varint(1, id, &mut nested);
    }
    let mut control = Vec::new();
    push_len(case, &nested, &mut control);
    let mut element = Vec::new();
    push_len(2, &control, &mut element);
    element
}

fn close_element(id: u64) -> Vec<u8> {
    // A real proto3 producer omits zero-valued scalar identity.
    control_element(1, (id != 0).then_some(id))
}

fn throw_element(id: Option<u64>, detail: &str) -> Vec<u8> {
    let mut thrown = Vec::new();
    if let Some(id) = id {
        push_varint(1, id, &mut thrown);
    }
    push_len(2, detail.as_bytes(), &mut thrown);
    push_len(3, b"remote-stack", &mut thrown);
    push_len(4, b"BOX_EXEC_TEST", &mut thrown);
    let mut control = Vec::new();
    push_len(2, &thrown, &mut control);
    let mut element = Vec::new();
    push_len(2, &control, &mut element);
    element
}

fn connect_data_body(elements: &[Vec<u8>]) -> Vec<u8> {
    let mut body = Vec::new();
    for element in elements {
        body.extend_from_slice(&connect_envelope(0, element));
    }
    body
}

fn complete_connect_body(elements: &[Vec<u8>], id: u64) -> Vec<u8> {
    let mut body = connect_data_body(elements);
    body.extend_from_slice(&connect_envelope(0, &close_element(id)));
    body.extend_from_slice(&connect_envelope(2, br#"{}"#));
    body
}

fn http_response(extra_headers: &str, body: &[u8]) -> Vec<u8> {
    let mut response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {CONNECT_STREAM_CONTENT_TYPE}\r\nConnection: close\r\n{extra_headers}\r\n"
    ).into_bytes();
    response.extend_from_slice(body);
    response
}

fn length_response(body: &[u8]) -> Vec<u8> {
    http_response(&format!("Content-Length: {}\r\n", body.len()), body)
}

fn bxr_shell_args() -> HostShellArgs {
    build_host_shell_args(HostShellArgsInput {
        command: "printf bxr-contract".into(),
        name: "printf".into(),
        working_directory: "/workspace".into(),
        tool_call_id: "bxr-contract".into(),
    })
}

fn bxr_read_args() -> ProductionReadArgs {
    ProductionReadArgs {
        path: "/workspace/bxr.txt".into(),
        tool_call_id: "bxr-read".into(),
        offset: None,
        limit: None,
        encoding_hint: None,
    }
}

fn with_exec_reply<T>(
    reply: impl FnOnce(&RequestIdentity) -> Vec<u8> + Send + 'static,
    call: impl FnOnce(&mut ProductionBoxResourceAccessor) -> Result<T, ProductionBoxExecError>,
) -> Result<T, ProductionBoxExecError> {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind BXR peer");
    let port = listener.local_addr().expect("BXR address").port();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept BXR request");
        let request = read_request(&mut stream);
        let identity = exec_request_identity(&request);
        assert_eq!(identity, RequestIdentity { id: 0, exec_id: String::new() });
        stream.write_all(&reply(&identity)).expect("write BXR response");
        stream.flush().expect("flush BXR response");
    });
    let environment = ProductionBoxEnvironment::new("127.0.0.1", port, "secret");
    let result = call(&mut environment.remote_resource_accessor());
    server.join().expect("BXR peer");
    result
}

fn shell_with_reply(response: Vec<u8>) -> Result<ProductionShellResult, ProductionBoxExecError> {
    with_exec_reply(move |_| response, |accessor| accessor.execute_shell_result(&(), bxr_shell_args()))
}

fn assert_unknown_transport(error: ProductionBoxExecError, scenario: &str) {
    assert!(error.outcome_unknown(), "{scenario}: {error:?}");
    match error {
        ProductionBoxExecError::Remote(BoxRemoteExecError::Transport(
            ProductionBoxTransportError::ExecutionOutcomeUnknown(cause),
        )) => {
            assert!(!matches!(*cause, ProductionBoxTransportError::ExecutionOutcomeUnknown(_)),
                "{scenario}: uncertainty must wrap the original cause exactly once");
        }
        other => panic!("{scenario}: expected post-write transport uncertainty, got {other:?}"),
    }
}

#[test]
fn production_exec_service_background_shell_spawn_uses_frozen_field_16_and_real_shell_id() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind fake background ExecService");
    let port = listener.local_addr().expect("local address").port();
    let server = thread::spawn(move || {
        serve_exec_response(
            &listener,
            b"printf background-contract",
            background_shell_spawn_success(
                4242,
                "printf background-contract",
                "/workspace",
                Some(31337),
            ),
        );
        serve_exec_response(
            &listener,
            b"background-error",
            background_shell_spawn_error(2, "background-error", "/workspace", "spawn failed"),
        );
        serve_exec_response(
            &listener,
            b"background-rejected",
            background_shell_spawn_error(3, "background-rejected", "/workspace", "blocked"),
        );
        serve_exec_response(
            &listener,
            b"background-permission",
            background_shell_spawn_error(4, "background-permission", "/workspace", "denied"),
        );
        serve_exec_response(
            &listener,
            b"background-sandbox",
            background_shell_spawn_sandbox_unsupported(
                "background-sandbox",
                "/workspace",
                "workspace_readonly",
                "sandbox unsupported",
            ),
        );
    });

    let environment = ProductionBoxEnvironment::new("127.0.0.1", port, "secret");
    let mut accessor = environment.remote_resource_accessor();
    let spawn = |command: &str| -> ProductionBackgroundShellSpawnArgs {
        build_host_shell_args(HostShellArgsInput {
            command: command.into(),
            name: command.split_whitespace().next().unwrap_or("shell").into(),
            working_directory: "/workspace".into(),
            tool_call_id: format!("{command}-tool"),
        })
        .into()
    };

    assert_eq!(
        accessor
            .execute_background_shell_spawn(&(), spawn("printf background-contract"))
            .expect("typed field-16 success"),
        ProductionBackgroundShellSpawnResult::Success {
            shell_id: 4242,
            command: "printf background-contract".into(),
            working_directory: "/workspace".into(),
            pid: Some(31337),
        }
    );
    assert_eq!(
        accessor
            .execute_background_shell_spawn(&(), spawn("background-error"))
            .expect("typed field-16 error"),
        ProductionBackgroundShellSpawnResult::Error {
            command: "background-error".into(),
            working_directory: "/workspace".into(),
            error: "spawn failed".into(),
        }
    );
    assert_eq!(
        accessor
            .execute_background_shell_spawn(&(), spawn("background-rejected"))
            .expect("typed field-16 rejected"),
        ProductionBackgroundShellSpawnResult::Rejected {
            command: "background-rejected".into(),
            working_directory: "/workspace".into(),
            reason: "blocked".into(),
            is_readonly: true,
        }
    );
    assert_eq!(
        accessor
            .execute_background_shell_spawn(&(), spawn("background-permission"))
            .expect("typed field-16 permission"),
        ProductionBackgroundShellSpawnResult::PermissionDenied {
            command: "background-permission".into(),
            working_directory: "/workspace".into(),
            error: "denied".into(),
            is_readonly: true,
        }
    );
    assert_eq!(
        accessor
            .execute_background_shell_spawn(&(), spawn("background-sandbox"))
            .expect("typed field-16 sandbox"),
        ProductionBackgroundShellSpawnResult::SandboxUnsupported {
            command: "background-sandbox".into(),
            working_directory: "/workspace".into(),
            sandbox_policy_type: "workspace_readonly".into(),
            reason: "sandbox unsupported".into(),
            is_readonly: true,
        }
    );

    server.join().expect("fake background ExecService thread");
}

#[test]
fn production_exec_service_preserves_shell_stdout_for_internal_consumers() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind fake ExecService");
    let port = listener.local_addr().expect("local address").port();
    let server = thread::spawn(move || {
        serve_exec_response(
            &listener,
            b"printf shipping-full-result",
            shell_success_element_with_output("pending-recording\n", "diagnostic"),
        );
    });

    let environment = ProductionBoxEnvironment::new("127.0.0.1", port, "secret");
    let mut accessor = environment.remote_resource_accessor();
    let result = accessor
        .execute_shell_result(
            &(),
            build_host_shell_args(HostShellArgsInput {
                command: "printf shipping-full-result".into(),
                name: "printf".into(),
                working_directory: "/workspace".into(),
                tool_call_id: "shipping-full-result-contract".into(),
            }),
        )
        .expect("shipping full shell result");

    assert_eq!(
        result,
        ProductionShellResult::Success {
            exit_code: 0,
            stdout: "pending-recording\n".into(),
            stderr: "diagnostic".into(),
        }
    );

    server.join().expect("fake ExecService thread");
}

#[test]
fn production_exec_service_streams_shell_and_write_through_shipping_accessor() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind fake ExecService");
    let port = listener.local_addr().expect("local address").port();
    let server = thread::spawn(move || {
        serve_exec_response(
            &listener,
            b"echo shipping-exec",
            shell_success_element("ready"),
        );
        serve_exec_response(
            &listener,
            b"/tmp/fabushi-exec-proof",
            write_success_element(),
        );
    });

    let environment = ProductionBoxEnvironment::new("127.0.0.1", port, "secret");
    let mut accessor = environment.remote_resource_accessor();

    let shell = accessor
        .execute(
            &(),
            build_host_shell_args(HostShellArgsInput {
                command: "echo shipping-exec".into(),
                name: "echo".into(),
                working_directory: "/workspace".into(),
                tool_call_id: "shipping-exec-contract".into(),
            }),
        )
        .expect("shipping shell ExecService call");
    assert_eq!(
        shell.result,
        ShellExecutionOutcome::Success {
            exit_code: 0,
            stderr: "ready".into(),
        }
    );

    let write = accessor
        .execute_write(
            &(),
            "/tmp/fabushi-exec-proof",
            b"payload",
            "shipping-write-contract",
        )
        .expect("shipping write ExecService call");
    assert_eq!(write, WriteExecResult::Success);

    server.join().expect("fake ExecService thread");
}


fn serve_unary_success(listener: &TcpListener, expected_path: &str) {
    let (mut stream, _) = listener.accept().expect("accept ControlService request");
    let request = read_request(&mut stream);
    let header_end = request
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .expect("header end");
    let headers = std::str::from_utf8(&request[..header_end]).expect("headers");
    assert!(headers.starts_with(&format!("POST {expected_path} HTTP/1.1\r\n")));
    assert!(
        headers
            .lines()
            .any(|line| line.eq_ignore_ascii_case("Authorization: Bearer secret"))
    );
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: application/proto\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    )
    .expect("write unary response");
    stream.flush().expect("flush unary response");
}

#[test]
fn production_loopback_factory_gates_exec_accessor_on_authenticated_readiness() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind fake box daemon");
    let port = listener.local_addr().expect("local address").port();
    let server = thread::spawn(move || {
        serve_unary_success(&listener, PING_PATH);
        serve_exec_response(
            &listener,
            b"echo loopback-production",
            shell_success_element("loopback-ready"),
        );
    });

    let environment = ProductionBoxEnvironment::new("127.0.0.1", port, "secret");
    assert_eq!(environment.loopback().describe(), "loopback");
    let mut ready = environment
        .ensure_ready("agent-production")
        .expect("authenticated loopback readiness");
    assert_eq!(
        ready.vnc_url,
        "http://127.0.0.1:6080/vnc.html"
    );
    assert_eq!(
        ready.terminals_folder,
        "/root/.cursor/projects/workspace/terminals"
    );

    let shell = ready
        .remote_accessor
        .execute(
            &(),
            build_host_shell_args(HostShellArgsInput {
                command: "echo loopback-production".into(),
                name: "echo".into(),
                working_directory: "/workspace".into(),
                tool_call_id: "loopback-production-contract".into(),
            }),
        )
        .expect("loopback production shell");
    assert_eq!(
        shell.result,
        ShellExecutionOutcome::Success {
            exit_code: 0,
            stderr: "loopback-ready".into(),
        }
    );

    let computer_error = ready
        .remote_accessor
        .execute_computer_use_protobuf(&(), Vec::new())
        .expect_err("standalone production primary must block monitor computer use");
    assert!(
        matches!(computer_error, ProductionBoxExecError::NoMonitor(_)),
        "standalone production primary must use the Grok no-monitor overlay"
    );

    assert!(should_apply_shared_desktop(environment.loopback().max_windows()));
    assert_eq!(
        format_sand_box_startup_summary(true, true),
        "[sand-host] agent box backend: loopback (in-box); image: host's own container; auto-update: on; build: packaged"
    );

    server.join().expect("fake box daemon thread");
}


#[test]
fn production_exec_service_reads_through_protected_shipping_accessor() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind fake read ExecService");
    let port = listener.local_addr().expect("local address").port();
    let server = thread::spawn(move || {
        serve_unary_success(&listener, PING_PATH);
        serve_exec_response(
            &listener,
            b"/workspace/safe.txt",
            read_success_element("/workspace/safe.txt", "safe\ncontent"),
        );
    });

    let loopback = LoopbackSandBox::new(LoopbackSandBoxOptions {
        host: "127.0.0.1".into(),
        auth_token: "secret".into(),
        exec_daemon_port: port,
        ready_timeout_ms: 5_000,
        poll_interval_ms: 0,
        watchdog_interval_ms: 30_000,
        protected_box_paths: vec![PathBuf::from("/workspace/private")],
    });
    let mut ready = loopback
        .ensure_ready(&(), "agent-read")
        .expect("shipping protected accessor readiness");

    let safe = ready
        .remote_accessor
        .execute_read(
            &(),
            ProductionReadArgs {
                path: "/workspace/safe.txt".into(),
                tool_call_id: "shipping-read-contract".into(),
                offset: None,
                limit: None,
                encoding_hint: None,
            },
        )
        .expect("safe read should reach ExecService");
    assert_eq!(
        safe,
        ProductionReadResult::Success {
            path: "/workspace/safe.txt".into(),
            output: ProductionReadOutput::Content("safe\ncontent".into()),
            total_lines: 2,
            file_size: 12,
            truncated: false,
            output_blob_id: None,
            range_applied: false,
        }
    );

    let protected = ready.remote_accessor.execute_read(
        &(),
        ProductionReadArgs {
            path: "/workspace/private/secret.db".into(),
            tool_call_id: "blocked-read-contract".into(),
            offset: None,
            limit: None,
            encoding_hint: None,
        },
    );
    assert!(
        protected.is_err(),
        "protected host paths must be rejected before a remote ExecService call"
    );

    server.join().expect("fake read ExecService thread");
}

#[test]
fn production_exec_service_computer_use_has_remote_and_no_monitor_paths() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind fake computer ExecService");
    let port = listener.local_addr().expect("local address").port();
    let server = thread::spawn(move || {
        serve_exec_response(
            &listener,
            b"computer-contract",
            computer_use_success_element(),
        );
        serve_exec_response(
            &listener,
            b"raw-field-44",
            raw_resource_element(44, &[0x08, 0x01]),
        );
    });

    let environment = ProductionBoxEnvironment::new("127.0.0.1", port, "secret");
    let mut accessor = environment.remote_resource_accessor();
    let protobuf_args = {
        let mut args = Vec::new();
        push_len(1, b"computer-contract", &mut args);
        args
    };
    let result = accessor
        .execute_computer_use_protobuf(&(), protobuf_args)
        .expect("shipping computer-use ExecService call");
    assert_eq!(result, vec![0x0a, 0x00]);

    let mut raw_args = Vec::new();
    push_len(1, b"raw-field-44", &mut raw_args);
    let raw_result = accessor
        .execute_raw_resource(&(), 44, raw_args)
        .expect("generic generated ExecService resource binding");
    assert_eq!(raw_result, vec![0x08, 0x01]);

    let mut denied = environment
        .remote_resource_accessor()
        .with_no_monitor_computer_use();
    let error = denied
        .execute_computer_use_protobuf(&(), Vec::new())
        .expect_err("no-monitor overlay must fail locally");
    assert!(
        error.to_string().contains("No private desktop monitor"),
        "no-monitor overlay should preserve the Grok error contract"
    );

    server.join().expect("fake computer ExecService thread");
}


#[test]
fn production_exec_service_shell_stream_uses_frozen_field_14_and_real_backgrounded_wire() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind fake stream ExecService");
    let port = listener.local_addr().expect("local address").port();
    let server = thread::spawn(move || {
        serve_exec_responses(
            &listener,
            b"printf timed-background",
            vec![
                shell_stream_start_element(),
                shell_stream_stdout_element("before\n"),
                shell_stream_backgrounded_element(
                    5150,
                    "printf timed-background",
                    "/workspace",
                    Some(27182),
                    Some(75),
                    Some(SHELL_BACKGROUND_REASON_TIMEOUT),
                ),
            ],
        );
    });

    let environment = ProductionBoxEnvironment::new("127.0.0.1", port, "secret");
    let mut accessor = environment.remote_resource_accessor();
    let events = accessor
        .execute_shell_stream(
            &(),
            ProductionShellStreamArgs {
                shell_args: build_host_shell_args(HostShellArgsInput {
                    command: "printf timed-background".into(),
                    name: "printf".into(),
                    working_directory: "/workspace".into(),
                    tool_call_id: "timed-background-direct-contract".into(),
                }),
                timeout_ms: 75,
                hard_timeout_ms: None,
            },
        )
        .expect("typed field-14 shell stream");

    assert_eq!(
        events,
        vec![
            ProductionShellStreamEvent::Start,
            ProductionShellStreamEvent::Stdout("before\n".into()),
            ProductionShellStreamEvent::Backgrounded {
                shell_id: 5150,
                command: "printf timed-background".into(),
                working_directory: "/workspace".into(),
                pid: Some(27182),
                ms_to_wait: Some(75),
                reason: Some(SHELL_BACKGROUND_REASON_TIMEOUT),
            },
        ],
        "field 14 must preserve the frozen typed event stream and real shellId"
    );

    server.join().expect("fake stream ExecService thread");
}

struct NoopForeverBoxLifecycle;

impl ForeverBoxLifecycle for NoopForeverBoxLifecycle {
    fn fetch_image_update_available(&self) -> Result<bool, String> {
        Ok(false)
    }

    fn recreate_in_box(
        &self,
        _preserve_data: bool,
        _force: Option<bool>,
    ) -> Result<RecreateSandBoxResponse, String> {
        Ok(RecreateSandBoxResponse {
            started: false,
            reason: Some("not-used-by-runner-resource-contract".into()),
        })
    }
}

#[test]
fn forever_box_runner_resource_port_background_spawn_registers_real_shell_id_with_unique_watcher() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind Runner background box daemon");
    let port = listener.local_addr().expect("local address").port();
    let server = thread::spawn(move || {
        serve_unary_success(&listener, PING_PATH);
        serve_exec_response(
            &listener,
            b"echo live-background",
            background_shell_spawn_success(
                4242,
                "echo live-background",
                "/workspace",
                Some(31337),
            ),
        );
    });

    let service = Arc::new(ForeverBoxService::new(
        HostBox::new(ProductionBoxEnvironment::new("127.0.0.1", port, "secret")),
        Arc::new(NoopForeverBoxLifecycle),
        false,
        false,
        false,
    ));
    let pending = Arc::new(Mutex::new(Vec::<(String, String, String)>::new()));
    let pending_capture = Arc::clone(&pending);
    let watches = Arc::new(RunnerBackgroundShellWatches::new(
        Arc::new(|_, _, _| None),
        Some(Arc::new(move |watch| {
            pending_capture
                .lock()
                .expect("pending background watch capture")
                .push((
                    watch.parent_agent_id.clone(),
                    watch.work_id.clone(),
                    watch.title.clone(),
                ));
        })),
        None,
        None,
    ));
    let port_adapter = ForeverBoxRunnerResourcePort::new(service, "agent-runner")
        .with_background_shell_watches(Arc::clone(&watches));

    let shell = port_adapter
        .execute_shell(RunnerBoxShellRequest {
            command: "echo live-background".into(),
            working_directory: "/workspace".into(),
            tool_call_id: "runner-background-shell-contract".into(),
            is_background: true,
            block_until_ms: None,
        })
        .expect("Runner background Shell through ForeverBox");
    assert_eq!(shell["kind"], "backgrounded");
    assert_eq!(shell["shellId"], 4242);
    assert_eq!(shell["pid"], 31337);
    assert_eq!(
        pending.lock().expect("pending background watches").as_slice(),
        &[(
            "agent-runner".into(),
            "4242".into(),
            "echo live-background".into(),
        )],
        "the real wire shellId must be registered with the unique Runner watcher"
    );

    server.join().expect("Runner background box daemon thread");
}

#[test]
fn forever_box_runner_resource_port_positive_block_backgrounds_real_shell_id_with_unique_watcher() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind timed Runner box daemon");
    let port = listener.local_addr().expect("local address").port();
    let server = thread::spawn(move || {
        serve_unary_success(&listener, PING_PATH);
        serve_exec_responses(
            &listener,
            b"printf timed-runner",
            vec![
                shell_stream_start_element(),
                shell_stream_stdout_element("before\n"),
                shell_stream_backgrounded_element(
                    5150,
                    "printf timed-runner",
                    "/workspace",
                    Some(27182),
                    Some(75),
                    Some(SHELL_BACKGROUND_REASON_TIMEOUT),
                ),
            ],
        );
    });

    let service = Arc::new(ForeverBoxService::new(
        HostBox::new(ProductionBoxEnvironment::new("127.0.0.1", port, "secret")),
        Arc::new(NoopForeverBoxLifecycle),
        false,
        false,
        false,
    ));
    let pending = Arc::new(Mutex::new(Vec::<(String, String, String)>::new()));
    let pending_capture = Arc::clone(&pending);
    let watches = Arc::new(RunnerBackgroundShellWatches::new(
        Arc::new(|_, _, _| None),
        Some(Arc::new(move |watch| {
            pending_capture
                .lock()
                .expect("timed pending background watch capture")
                .push((
                    watch.parent_agent_id.clone(),
                    watch.work_id.clone(),
                    watch.title.clone(),
                ));
        })),
        None,
        None,
    ));
    let port_adapter = ForeverBoxRunnerResourcePort::new(service, "agent-timed-runner")
        .with_background_shell_watches(Arc::clone(&watches));

    let shell = port_adapter
        .execute_shell(RunnerBoxShellRequest {
            command: "printf timed-runner".into(),
            working_directory: "/workspace".into(),
            tool_call_id: "runner-timed-shell-contract".into(),
            is_background: false,
            block_until_ms: Some(75),
        })
        .expect("Runner timed Shell through ForeverBox");
    assert_eq!(shell["kind"], "backgrounded");
    assert_eq!(shell["shellId"], 5150);
    assert_eq!(shell["pid"], 27182);
    assert_eq!(shell["blockUntilMs"], 75);
    assert_eq!(shell["backgroundReason"], SHELL_BACKGROUND_REASON_TIMEOUT);
    assert_eq!(shell["stdout"], "before\n");
    assert_eq!(
        pending.lock().expect("timed pending watches").as_slice(),
        &[(
            "agent-timed-runner".into(),
            "5150".into(),
            "printf timed-runner".into(),
        )],
        "positive block_until_ms must register the real field-14 shellId with the unique watcher"
    );

    server.join().expect("timed Runner box daemon thread");
}

#[test]
fn forever_box_runner_resource_port_reuses_authenticated_shipping_exec_service_for_turn() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind Runner box daemon");
    let port = listener.local_addr().expect("local address").port();
    let server = thread::spawn(move || {
        // Frozen remote-box-resources owns one connectionPromise per Runner.
        // The shipping adapter is turn-scoped, so Shell authenticates once and
        // the following Read reuses that guarded accessor without another ping.
        serve_unary_success(&listener, PING_PATH);
        serve_exec_response(
            &listener,
            b"echo runner-box",
            shell_success_element("runner-shell"),
        );
        serve_exec_response(
            &listener,
            b"/workspace/runner.txt",
            read_success_element("/workspace/runner.txt", "runner-read"),
        );
    });

    let service = Arc::new(ForeverBoxService::new(
        HostBox::new(ProductionBoxEnvironment::new("127.0.0.1", port, "secret")),
        Arc::new(NoopForeverBoxLifecycle),
        false,
        false,
        false,
    ));
    let port_adapter = ForeverBoxRunnerResourcePort::new(service, "agent-runner");

    let shell = port_adapter
        .execute_shell(RunnerBoxShellRequest {
            command: "echo runner-box".into(),
            working_directory: "/workspace".into(),
            tool_call_id: "runner-shell-contract".into(),
            is_background: false,
            block_until_ms: None,
        })
        .expect("Runner Shell through ForeverBox");
    assert_eq!(shell["kind"], "success");
    assert_eq!(shell["exitCode"], 0);
    assert_eq!(shell["stderr"], "runner-shell");

    let read = port_adapter
        .execute_read(RunnerBoxReadRequest {
            path: "/workspace/runner.txt".into(),
            tool_call_id: "runner-read-contract".into(),
            offset: None,
            limit: None,
            encoding_hint: None,
        })
        .expect("Runner Read through ForeverBox");
    assert_eq!(read["kind"], "success");
    assert_eq!(read["path"], "/workspace/runner.txt");
    assert_eq!(read["output"]["content"], "runner-read");

    server.join().expect("Runner box daemon thread");
}

#[test]
fn production_exec_service_accepts_default_identity_and_nonzero_sequential_echo() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind sequential BXR peer");
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        for expected in 0..=1 {
            let (mut stream, _) = listener.accept().expect("accept sequential request");
            let identity = exec_request_identity(&read_request(&mut stream));
            assert_eq!(identity.id, expected);
            assert!(identity.exec_id.is_empty(), "BXR-v1 must not mint exec_id");
            let element = echo_response_identity(
                &shell_success_element_with_output(&expected.to_string(), ""), &identity,
            );
            stream.write_all(&length_response(&complete_connect_body(&[element], identity.id)))
                .expect("write sequential response");
        }
    });
    let environment = ProductionBoxEnvironment::new("127.0.0.1", port, "secret");
    let mut accessor = environment.remote_resource_accessor();
    for expected in 0..=1 {
        assert_eq!(
            accessor.execute_shell_result(&(), bxr_shell_args()).expect("correlated result"),
            ProductionShellResult::Success {
                exit_code: 0, stdout: expected.to_string(), stderr: String::new(),
            },
        );
    }
    server.join().expect("sequential BXR peer");
}

#[test]
fn production_exec_service_decodes_metadata_on_both_sides_of_exec_and_shell_payloads() {
    for metadata_first in [false, true] {
        let result = with_exec_reply(move |identity| {
            let element = shell_success_element_with_output("preserved", "diagnostic");
            let shell_result = fixture_message_field(fixture_message_field(&element, 1), 2);
            let mut shell_metadata = Vec::new();
            push_len(101, &[], &mut shell_metadata);
            push_varint(102, 0, &mut shell_metadata);
            push_len(103, b"/workspace/terminals", &mut shell_metadata);
            push_varint(104, 123, &mut shell_metadata);
            let mut shell = Vec::new();
            if metadata_first {
                shell.extend_from_slice(&shell_metadata);
            }
            shell.extend_from_slice(shell_result);
            shell.extend_from_slice(&shell_metadata);
            let mut metadata = Vec::new();
            push_varint(1, identity.id, &mut metadata);
            push_len(15, identity.exec_id.as_bytes(), &mut metadata);
            push_varint(39, 17, &mut metadata);
            push_len(45, &[0x0a, 0x04, b'h', b'o', b'o', b'k'], &mut metadata);
            push_len(60, b"future metadata", &mut metadata);
            let mut client = Vec::new();
            if metadata_first {
                client.extend_from_slice(&metadata);
            }
            push_len(2, &shell, &mut client);
            client.extend_from_slice(&metadata);
            length_response(&complete_connect_body(&[data_element(&client)], identity.id))
        }, |accessor| accessor.execute_shell_result(&(), bxr_shell_args())).expect("metadata-independent result");
        assert_eq!(result, ProductionShellResult::Success {
            exit_code: 0, stdout: "preserved".into(), stderr: "diagnostic".into(),
        });
    }
}

#[test]
fn production_exec_service_preserves_binary_read_bytes_with_complete_identity_and_framing() {
    let bytes = vec![0, 255, 128, 10, 254];
    let expected = bytes.clone();
    let result = with_exec_reply(move |identity| {
        let mut success = Vec::new();
        push_len(1, b"/workspace/bxr.txt", &mut success);
        push_len(5, &bytes, &mut success);
        push_varint(4, bytes.len() as u64, &mut success);
        let mut result = Vec::new();
        push_len(1, &success, &mut result);
        let element = echo_response_identity(&raw_resource_element(7, &result), identity);
        length_response(&complete_connect_body(&[element], identity.id))
    }, |accessor| accessor.execute_read(&(), bxr_read_args())).expect("binary Read");
    assert_eq!(result, ProductionReadResult::Success {
        path: "/workspace/bxr.txt".into(),
        output: ProductionReadOutput::Data(expected),
        total_lines: 0, file_size: 5, truncated: false, output_blob_id: None, range_applied: false,
    });
}

#[test]
fn production_exec_service_accepts_valid_chunk_extensions_trailers_and_length_or_close_delimiting() {
    let body = complete_connect_body(&[shell_success_element_with_output("framed", "")], 0);
    let split = body.len() / 2;
    let mut chunks = format!("{split:X};token=value;quoted=\"a;b\\\"c\"\r\n").into_bytes();
    chunks.extend_from_slice(&body[..split]);
    chunks.extend_from_slice(format!("\r\n{:X};flag\r\n", body.len() - split).as_bytes());
    chunks.extend_from_slice(&body[split..]);
    chunks.extend_from_slice(b"\r\n0;done=yes\r\nX-Trace: value\r\nX-Empty:\r\nX-Obs: \xff\r\n\r\n");
    let responses = vec![
        http_response("Transfer-Encoding: chunked\r\n", &chunks),
        length_response(&body),
        http_response("", &body),
        http_response(&format!("Content-Length: {}, {}\r\nContent-Length: {}\r\n",
            body.len(), body.len(), body.len()), &body),
    ];
    for response in responses {
        assert_eq!(shell_with_reply(response).expect("complete HTTP response"),
            ProductionShellResult::Success {
                exit_code: 0, stdout: "framed".into(), stderr: String::new(),
            });
    }
    let mut with_metadata = connect_data_body(&[
        shell_success_element_with_output("metadata", ""), close_element(0),
    ]);
    with_metadata.extend_from_slice(&connect_envelope(2, br#"{"metadata":{"x-trace":["one","two"]}}"#));
    assert!(shell_with_reply(length_response(&with_metadata)).is_ok());
}

#[test]
fn production_exec_service_retains_matching_throw_and_connect_error_details() {
    let mut body = connect_data_body(&[throw_element(None, "remote failure")]);
    body.extend_from_slice(&connect_envelope(2, br#"{}"#));
    let error = shell_with_reply(length_response(&body)).expect_err("matching Throw is failure");
    assert!(error.outcome_unknown());
    match error {
        ProductionBoxExecError::Remote(BoxRemoteExecError::RemoteThrow {
            error, stack_trace, error_code,
        }) => {
            assert_eq!(error, "remote failure");
            assert_eq!(stack_trace.as_deref(), Some("remote-stack"));
            assert_eq!(error_code.as_deref(), Some("BOX_EXEC_TEST"));
        }
        other => panic!("matching Throw must retain fields: {other:?}"),
    }
    let body = connect_envelope(2, br#"{"error":{"code":"unavailable","message":"peer lost execution","details":[]}}"#);
    let error = shell_with_reply(length_response(&body)).expect_err("Connect remote error");
    let detail = error.to_string();
    assert!(detail.contains("unavailable") && detail.contains("peer lost execution"));
    assert_unknown_transport(error, "Connect remote error");
}

#[test]
fn production_exec_service_rejects_malformed_identity_metadata_and_ambiguous_oneofs() {
    let success_element = shell_success_element("");
    let payload = fixture_message_field(&success_element, 1).to_vec();
    let shell = fixture_message_field(&payload, 2).to_vec();
    let mut clients: Vec<(&str, Vec<u8>)> = Vec::new();
    for (name, field, wire) in [
        ("id wrong wire", 1, 2),
        ("exec_id wrong wire", 15, 0),
        ("local time wrong wire", 39, 2),
        ("hook context wrong wire", 45, 0),
        ("known result wrong wire", 3, 0),
    ] {
        let mut client = payload.clone();
        push_key(field, wire, &mut client);
        client.push(0);
        clients.push((name, client));
    }
    let mut repeated = payload.clone();
    push_varint(1, 0, &mut repeated);
    push_varint(1, 1, &mut repeated);
    clients.push(("contradictory id", repeated));
    let mut repeated = payload.clone();
    push_len(15, b"", &mut repeated);
    push_len(15, b"foreign", &mut repeated);
    clients.push(("contradictory exec_id", repeated));
    let mut overflow = payload.clone();
    push_varint(1, u64::from(u32::MAX) + 1, &mut overflow);
    clients.push(("uint32 identity overflow", overflow));
    let mut utf8 = payload.clone();
    push_len(15, &[255], &mut utf8);
    clients.push(("invalid UTF-8 identity", utf8));
    let mut duplicate = payload.clone();
    duplicate.extend_from_slice(&payload);
    clients.push(("duplicate Exec oneof", duplicate));
    let mut mixed = payload.clone();
    push_len(3, &[0x0a, 0], &mut mixed);
    clients.push(("mixed Exec oneof", mixed));
    let mut metadata_only = Vec::new();
    push_len(15, b"", &mut metadata_only);
    push_len(45, &[], &mut metadata_only);
    push_len(60, b"must not become RawResource", &mut metadata_only);
    clients.push(("metadata is not a result", metadata_only));
    let mut field_zero = payload.clone();
    field_zero.extend_from_slice(&[0, 0]);
    clients.push(("field zero after payload", field_zero));
    let mut truncated = payload.clone();
    push_key(60, 0, &mut truncated);
    truncated.push(0x80);
    clients.push(("truncated trailing varint", truncated));
    let mut overflow = payload.clone();
    push_key(60, 0, &mut overflow);
    overflow.extend_from_slice(&[0xff; 9]);
    overflow.push(2);
    clients.push(("overflowing trailing varint", overflow));
    let mut truncated = payload.clone();
    push_key(60, 2, &mut truncated);
    encode_varint(u64::MAX, &mut truncated);
    clients.push(("overflowing trailing length", truncated));
    for (name, client) in clients {
        let body = complete_connect_body(&[data_element(&client)], 0);
        assert_unknown_transport(shell_with_reply(length_response(&body)).expect_err(name), name);
    }

    let mut duplicate_shell = shell.clone();
    duplicate_shell.extend_from_slice(&shell);
    let mut metadata_only = Vec::new();
    push_len(103, b"/terminals", &mut metadata_only);
    let mut wrong_metadata = shell.clone();
    push_varint(103, 7, &mut wrong_metadata);
    for (name, shell_result) in [
        ("duplicate ShellResult oneof", duplicate_shell),
        ("ShellResult metadata only", metadata_only),
        ("ShellResult metadata wrong wire", wrong_metadata),
    ] {
        let mut client = Vec::new();
        push_len(2, &shell_result, &mut client);
        assert_unknown_transport(
            shell_with_reply(length_response(&complete_connect_body(&[data_element(&client)], 0)))
                .expect_err(name), name,
        );
    }

    let mut duplicate_element = success_element.clone();
    duplicate_element.extend_from_slice(&close_element(0));
    let mut empty_control = Vec::new();
    push_len(2, &[], &mut empty_control);
    let mut unsupported_control = Vec::new();
    push_len(2, &[0x22, 0], &mut unsupported_control);
    for (name, element) in [
        ("ambiguous stream envelope", duplicate_element),
        ("empty control", empty_control),
        ("unsupported control", unsupported_control),
        ("empty envelope", Vec::new()),
    ] {
        assert_unknown_transport(
            shell_with_reply(length_response(&complete_connect_body(&[element], 0))).expect_err(name),
            name,
        );
    }
}

#[test]
fn production_exec_service_rejects_foreign_data_exec_id_heartbeat_close_and_throw() {
    let original = shell_success_element("");
    let foreign_data = echo_response_identity(&original, &RequestIdentity {
        id: 1, exec_id: String::new(),
    });
    let foreign_exec = echo_response_identity(&original, &RequestIdentity {
        id: 0, exec_id: "unrequested-exec".into(),
    });
    let cases = [
        ("data id", vec![foreign_data, close_element(0)]),
        ("exec_id", vec![foreign_exec, close_element(0)]),
        ("heartbeat id", vec![control_element(3, Some(1)), original.clone(), close_element(0)]),
        ("close id", vec![original, close_element(1)]),
        ("Throw id", vec![throw_element(Some(1), "foreign failure")]),
    ];
    for (name, elements) in cases {
        let mut body = connect_data_body(&elements);
        body.extend_from_slice(&connect_envelope(2, br#"{}"#));
        let error = shell_with_reply(length_response(&body)).expect_err(name);
        assert!(error.outcome_unknown(), "{name}");
        assert!(matches!(error, ProductionBoxExecError::Remote(
            BoxRemoteExecError::ProtocolViolation(_)
        )), "{name}: {error:?}");
    }
}

#[test]
fn production_exec_service_does_not_accept_omitted_zero_identity_for_a_nonzero_request() {
    for kind in ["data", "heartbeat", "close", "throw"] {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind default-identity peer");
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            for expected in 0..=1 {
                let (mut stream, _) = listener.accept().expect("accept request");
                let identity = exec_request_identity(&read_request(&mut stream));
                assert_eq!(identity.id, expected);
                assert!(identity.exec_id.is_empty());
                let success = shell_success_element("");
                let elements = if expected == 0 {
                    vec![success, close_element(0)]
                } else {
                    let correlated = echo_response_identity(&success, &identity);
                    match kind {
                        "data" => vec![success, close_element(1)],
                        "heartbeat" => vec![control_element(3, None), correlated, close_element(1)],
                        "close" => vec![correlated, close_element(0)],
                        "throw" => vec![throw_element(None, "default-zero Throw")],
                        _ => unreachable!(),
                    }
                };
                let mut body = connect_data_body(&elements);
                body.extend_from_slice(&connect_envelope(2, br#"{}"#));
                stream.write_all(&length_response(&body)).expect("write default-identity response");
            }
        });
        let environment = ProductionBoxEnvironment::new("127.0.0.1", port, "secret");
        let mut accessor = environment.remote_resource_accessor();
        accessor.execute_shell_result(&(), bxr_shell_args()).expect("first default-zero result");
        let error = accessor.execute_shell_result(&(), bxr_shell_args()).expect_err(kind);
        assert!(error.outcome_unknown());
        assert!(matches!(error, ProductionBoxExecError::Remote(
            BoxRemoteExecError::ProtocolViolation(_)
        )), "{kind}: {error:?}");
        server.join().expect("default-identity peer");
    }
}

#[test]
fn production_exec_service_requires_exactly_one_close_after_data() {
    let data = shell_success_element("");
    for (name, elements) in [
        ("missing close", vec![data.clone()]),
        ("duplicate close", vec![data.clone(), close_element(0), close_element(0)]),
        ("data after close", vec![close_element(0), data.clone()]),
        ("heartbeat after close", vec![data.clone(), close_element(0), control_element(3, None)]),
        ("Throw after close", vec![data, close_element(0), throw_element(None, "late")]),
    ] {
        let mut body = connect_data_body(&elements);
        body.extend_from_slice(&connect_envelope(2, br#"{}"#));
        let error = shell_with_reply(length_response(&body)).expect_err(name);
        assert!(error.outcome_unknown());
        assert!(matches!(error, ProductionBoxExecError::Remote(
            BoxRemoteExecError::ProtocolViolation(_)
        )), "{name}: {error:?}");
    }
}

#[test]
fn production_exec_service_requires_one_legal_final_connect_end() {
    let data = connect_data_body(&[shell_success_element(""), close_element(0)]);
    let valid_end = connect_envelope(2, br#"{}"#);
    let mut duplicate = data.clone();
    duplicate.extend_from_slice(&valid_end);
    duplicate.extend_from_slice(&valid_end);
    let mut data_after_end = data.clone();
    data_after_end.extend_from_slice(&valid_end);
    data_after_end.extend_from_slice(&connect_envelope(0, &shell_success_element("")));
    let mut unknown_flags = data.clone();
    unknown_flags.extend_from_slice(&connect_envelope(0x82, br#"{}"#));
    let mut compressed_end = data.clone();
    compressed_end.extend_from_slice(&connect_envelope(3, br#"{}"#));
    let mut truncated_header = data.clone();
    truncated_header.extend_from_slice(&[2, 0, 0]);
    let mut truncated_payload = data.clone();
    truncated_payload.extend_from_slice(&[2, 0xff, 0xff, 0xff, 0xff, b'{']);
    let mut cases = vec![
        ("missing end", data.clone()),
        ("duplicate end", duplicate),
        ("data after end", data_after_end),
        ("unknown end flag bits", unknown_flags),
        ("compressed end", compressed_end),
        ("truncated Connect header", truncated_header),
        ("truncated Connect payload", truncated_payload),
        ("unknown data flags", connect_envelope(0x80, &shell_success_element(""))),
    ];
    for (name, payload) in [
        ("empty end", b"".as_slice()),
        ("array end", b"[]".as_slice()),
        ("null end", b"null".as_slice()),
        ("malformed JSON", b"{".as_slice()),
        ("null error", br#"{"error":null}"#.as_slice()),
        ("string error", br#"{"error":"bad"}"#.as_slice()),
        ("error without code", br#"{"error":{"message":"bad"}}"#.as_slice()),
        ("duplicate end keys", br#"{"error":{"code":"unknown"},"error":null}"#.as_slice()),
        ("invalid metadata", br#"{"metadata":{"key":"not an array"}}"#.as_slice()),
        ("null metadata", br#"{"metadata":null}"#.as_slice()),
    ] {
        let mut body = data.clone();
        body.extend_from_slice(&connect_envelope(2, payload));
        cases.push((name, body));
    }
    for (name, body) in cases {
        assert_unknown_transport(shell_with_reply(length_response(&body)).expect_err(name), name);
    }
}

#[test]
fn production_exec_service_rejects_incomplete_or_ambiguous_http_framing() {
    let body = complete_connect_body(&[shell_success_element("")], 0);
    let mut single_chunk = format!("{:X}\r\n", body.len()).into_bytes();
    single_chunk.extend_from_slice(&body);
    single_chunk.extend_from_slice(b"\r\n");
    let mut terminated = single_chunk.clone();
    terminated.extend_from_slice(b"0\r\n\r\n");
    let mut cases = vec![
        ("truncated HTTP headers", b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\n".to_vec()),
        ("invalid status", b"HTTP/1.1 bad OK\r\n\r\n".to_vec()),
        ("non-UTF-8 headers", b"HTTP/1.1 200 OK\r\nX-Test: \xff\r\n\r\n".to_vec()),
        ("short declared length", http_response(&format!("Content-Length: {}\r\n", body.len() - 1), &body)),
        ("long declared length", http_response(&format!("Content-Length: {}\r\n", body.len() + 1), &body)),
        ("conflicting lengths", http_response(&format!("Content-Length: {}\r\nContent-Length: 0\r\n", body.len()), &body)),
        ("ambiguous comma lengths", http_response(&format!("Content-Length: {}, 0\r\n", body.len()), &body)),
        ("overflowing length", http_response("Content-Length: 184467440737095516160\r\n", &body)),
        ("signed length", http_response("Content-Length: +1\r\n", &body)),
        ("length and chunked", http_response("Transfer-Encoding: chunked\r\nContent-Length: 0\r\n", &terminated)),
        ("duplicate transfer encoding", http_response("Transfer-Encoding: chunked\r\nTransfer-Encoding: chunked\r\n", &terminated)),
        ("unsupported transfer coding", http_response("Transfer-Encoding: gzip, chunked\r\n", &terminated)),
        ("substring transfer coding", http_response("Transfer-Encoding: xchunked\r\n", &terminated)),
        ("missing zero chunk", http_response("Transfer-Encoding: chunked\r\n", &single_chunk)),
        ("truncated chunk size", http_response("Transfer-Encoding: chunked\r\n", b"10")),
        ("truncated chunk payload", http_response("Transfer-Encoding: chunked\r\n", b"10\r\nshort\r\n")),
        ("overflowing chunk size", http_response("Transfer-Encoding: chunked\r\n", b"FFFFFFFFFFFFFFFFF\r\n")),
        ("invalid chunk size", http_response("Transfer-Encoding: chunked\r\n", b"+1\r\nx\r\n0\r\n\r\n")),
        ("invalid HTTP header", http_response("Bad Header: value\r\n", &body)),
    ];
    for (name, tail) in [
        ("zero chunk lacks trailer terminator", b"0\r\n".as_slice()),
        ("truncated trailer", b"0\r\nX-Test: value\r\n".as_slice()),
        ("malformed trailer", b"0\r\ninvalid\r\n\r\n".as_slice()),
        ("framing trailer", b"0\r\nContent-Length: 0\r\n\r\n".as_slice()),
        ("trailing HTTP bytes", b"0\r\n\r\nextra".as_slice()),
    ] {
        let mut chunks = single_chunk.clone();
        chunks.extend_from_slice(tail);
        cases.push((name, http_response("Transfer-Encoding: chunked\r\n", &chunks)));
    }
    let mut missing_chunk_crlf = format!("{:X}\r\n", body.len()).into_bytes();
    missing_chunk_crlf.extend_from_slice(&body);
    missing_chunk_crlf.extend_from_slice(b"0\r\n\r\n");
    cases.push(("missing chunk CRLF", http_response("Transfer-Encoding: chunked\r\n", &missing_chunk_crlf)));
    for (name, response) in cases {
        assert_unknown_transport(shell_with_reply(response).expect_err(name), name);
    }
}

#[derive(Clone, Copy, Debug)]
enum OneShotOperation {
    Shell,
    ShellAccessor,
    FileTransferShell,
    Read,
    Write,
    ComputerUse,
    RawResource,
    BackgroundShell,
}

impl OneShotOperation {
    fn success_element(self) -> Vec<u8> {
        match self {
            Self::Shell | Self::ShellAccessor | Self::FileTransferShell => shell_success_element(""),
            Self::Read => read_success_element("/workspace/bxr.txt", "read"),
            Self::Write => write_success_element(),
            Self::ComputerUse => computer_use_success_element(),
            Self::RawResource => raw_resource_element(44, &[0x08, 1]),
            Self::BackgroundShell => background_shell_spawn_success(42, "printf bxr-contract", "/workspace", Some(123)),
        }
    }

    fn call(self, accessor: &mut ProductionBoxResourceAccessor) -> Result<(), ProductionBoxExecError> {
        match self {
            Self::Shell => accessor.execute_shell_result(&(), bxr_shell_args()).map(|_| ()),
            Self::ShellAccessor => accessor.execute(&(), bxr_shell_args()).map(|_| ()),
            Self::FileTransferShell => accessor.execute_shell(&(), bxr_shell_args()).map(|_| ()),
            Self::Read => accessor.execute_read(&(), bxr_read_args()).map(|_| ()),
            Self::Write => accessor.execute_write(&(), "/workspace/bxr.txt", b"payload", "bxr-write").map(|_| ()),
            Self::ComputerUse => accessor.execute_computer_use_protobuf(&(), Vec::new()).map(|_| ()),
            Self::RawResource => accessor.execute_raw_resource(&(), 44, Vec::new()).map(|_| ()),
            Self::BackgroundShell => accessor.execute_background_shell_spawn(&(), bxr_shell_args().into()).map(|_| ()),
        }
    }
}

#[test]
fn production_exec_service_all_one_shot_accessors_require_one_expected_result() {
    for operation in [
        OneShotOperation::Shell, OneShotOperation::ShellAccessor, OneShotOperation::FileTransferShell,
        OneShotOperation::Read, OneShotOperation::Write, OneShotOperation::ComputerUse,
        OneShotOperation::RawResource, OneShotOperation::BackgroundShell,
    ] {
        let valid = operation.success_element();
        let wrong = match operation {
            OneShotOperation::Shell | OneShotOperation::ShellAccessor | OneShotOperation::FileTransferShell =>
                write_success_element(),
            _ => shell_success_element("wrong operation"),
        };
        for (name, elements) in [
            ("missing", vec![]),
            ("wrong", vec![wrong.clone()]),
            ("duplicate", vec![valid.clone(), valid.clone()]),
            ("mixed", vec![valid.clone(), wrong]),
        ] {
            let error = with_exec_reply(move |identity| {
                let elements = elements.iter()
                    .map(|element| echo_response_identity(element, identity)).collect::<Vec<_>>();
                length_response(&complete_connect_body(&elements, identity.id))
            }, |accessor| operation.call(accessor)).expect_err(name);
            assert!(error.outcome_unknown(), "{operation:?}/{name}: {error:?}");
            if name == "missing" {
                assert!(matches!(error, ProductionBoxExecError::MissingResult(_)));
            } else {
                assert!(matches!(error, ProductionBoxExecError::InvalidResult(_)));
            }
        }
    }
}

#[test]
fn production_exec_service_preserves_ordered_shell_stream_and_rejects_mixed_operations() {
    let mut stderr = Vec::new();
    push_len(1, b"warning", &mut stderr);
    let mut exit = Vec::new();
    push_varint(1, 0, &mut exit);
    push_len(2, b"/workspace", &mut exit);
    push_varint(6, 12, &mut exit);
    let elements = vec![
        control_element(3, None),
        shell_stream_start_element(),
        shell_stream_stdout_element("first"),
        shell_stream_element(2, &stderr),
        shell_stream_element(3, &exit),
    ];
    let args = || ProductionShellStreamArgs {
        shell_args: bxr_shell_args(), timeout_ms: 75, hard_timeout_ms: Some(500),
    };
    let events = with_exec_reply(move |_| length_response(&complete_connect_body(&elements, 0)),
        |accessor| accessor.execute_shell_stream(&(), args())).expect("ordered ShellStream");
    assert_eq!(events, vec![
        ProductionShellStreamEvent::Start,
        ProductionShellStreamEvent::Stdout("first".into()),
        ProductionShellStreamEvent::Stderr("warning".into()),
        ProductionShellStreamEvent::Exit {
            code: 0, cwd: "/workspace".into(), aborted: false, local_execution_time_ms: Some(12),
        },
    ]);
    let error = with_exec_reply(move |_| {
        length_response(&complete_connect_body(&[
            shell_stream_start_element(), write_success_element(),
        ], 0))
    }, |accessor| accessor.execute_shell_stream(&(), args())).expect_err("mixed stream operation");
    assert!(error.outcome_unknown());
    assert!(matches!(error, ProductionBoxExecError::InvalidResult(_)));
}

#[test]
fn production_exec_service_distinguishes_pre_write_refusal_and_local_guards_from_unknown() {
    assert_eq!(PRODUCTION_BOX_RPC_TIMEOUT_MS, 15_000);
    assert_eq!(PRODUCTION_BOX_EXEC_TIMEOUT_MS, 120_000);
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind header-refusal witness");
    listener.set_nonblocking(true).expect("nonblocking witness");
    let endpoint = BoxEndpoint::new("127.0.0.1", listener.local_addr().unwrap().port(), "bad\nheader");
    let mut accessor = create_production_box_resource_accessor(&ProductionBoxTransport::from_endpoint(&endpoint));
    let error = accessor.execute_shell_result(&(), bxr_shell_args()).expect_err("invalid auth header");
    assert!(!error.outcome_unknown());
    assert!(matches!(error, ProductionBoxExecError::Remote(BoxRemoteExecError::Transport(
        ProductionBoxTransportError::InvalidHeader(_)
    ))));
    assert_eq!(listener.accept().expect_err("invalid header must not connect").kind(), std::io::ErrorKind::WouldBlock);

    let environment = ProductionBoxEnvironment::new("127.0.0.1", 0, "secret");
    let mut accessor = environment.remote_resource_accessor();
    let error = accessor.execute_shell_result(&(), bxr_shell_args()).expect_err("pre-write connection refusal");
    assert!(!error.outcome_unknown(), "{error:?}");
    assert!(matches!(error, ProductionBoxExecError::Remote(BoxRemoteExecError::Transport(
        ProductionBoxTransportError::Io(_)
    ))));

    let error = environment.remote_resource_accessor().with_no_monitor_computer_use()
        .execute_computer_use_protobuf(&(), Vec::new()).expect_err("no monitor");
    assert!(!error.outcome_unknown());
    assert!(matches!(error, ProductionBoxExecError::NoMonitor(_)));
    let error = environment.remote_resource_accessor()
        .with_file_read_guard(vec![PathBuf::from("/workspace")])
        .execute_read(&(), bxr_read_args()).expect_err("protected path");
    assert!(!error.outcome_unknown());
    assert!(matches!(error, ProductionBoxExecError::ProtectedPath(_)));

    let exhausted = ProductionBoxExecError::Remote(BoxRemoteExecError::IdExhausted);
    assert!(!exhausted.outcome_unknown());
}

#[test]
fn production_exec_service_rejects_out_of_range_direct_client_id_before_connect() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind id-refusal witness");
    listener.set_nonblocking(true).unwrap();
    let endpoint = BoxEndpoint::new("127.0.0.1", listener.local_addr().unwrap().port(), "secret");
    let transport = ProductionBoxTransport::from_endpoint(&endpoint);
    let mut client = create_production_box_exec_client(&transport);
    let error = client.exec(&(), ProductionExecRequest::Shell {
        id: u64::from(u32::MAX) + 1, args: bxr_shell_args(),
    }).expect_err("invalid identity must not serialize or dispatch");
    assert!(!matches!(error, ProductionBoxTransportError::ExecutionOutcomeUnknown(_)));
    assert_eq!(listener.accept().expect_err("invalid id must not connect").kind(), std::io::ErrorKind::WouldBlock);
}

#[test]
fn production_exec_service_wraps_post_write_status_and_eof_once_with_original_cause() {
    assert_unknown_transport(shell_with_reply(Vec::new()).expect_err("EOF after request"), "post-write EOF");
    let response = b"HTTP/1.1 503 Unavailable\r\nContent-Length: 11\r\nConnection: close\r\n\r\npeer failed".to_vec();
    let error = shell_with_reply(response).expect_err("HTTP refusal after dispatch");
    assert!(error.outcome_unknown());
    match error {
        ProductionBoxExecError::Remote(BoxRemoteExecError::Transport(
            ProductionBoxTransportError::ExecutionOutcomeUnknown(cause)
        )) => match *cause {
            ProductionBoxTransportError::HttpStatus { status, body } => {
                assert_eq!(status, 503);
                assert_eq!(body, "peer failed");
            }
            other => panic!("status cause was lost: {other:?}"),
        },
        other => panic!("post-write status was not unknown: {other:?}"),
    }
}

#[test]
fn production_exec_service_never_replays_a_request_after_lost_completion() {
    for loss in ["eof", "missing close", "missing Connect end"] {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind single-request witness");
        listener.set_nonblocking(true).unwrap();
        let port = listener.local_addr().unwrap().port();
        let finished = Arc::new(AtomicBool::new(false));
        let requests = Arc::new(AtomicUsize::new(0));
        let server_finished = Arc::clone(&finished);
        let server_requests = Arc::clone(&requests);
        let server = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let identity = exec_request_identity(&read_request(&mut stream));
                        server_requests.fetch_add(1, Ordering::SeqCst);
                        assert!(identity.exec_id.is_empty());
                        if loss != "eof" {
                            let element = echo_response_identity(&shell_success_element(""), &identity);
                            let mut body = connect_data_body(&[element]);
                            if loss == "missing close" {
                                body.extend_from_slice(&connect_envelope(2, br#"{}"#));
                            } else {
                                body.extend_from_slice(&connect_envelope(0, &close_element(identity.id)));
                            }
                            stream.write_all(&length_response(&body)).expect("write incomplete completion");
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        if server_finished.load(Ordering::SeqCst) {
                            break;
                        }
                        assert!(Instant::now() < deadline, "single-request witness timed out");
                        thread::sleep(Duration::from_millis(1));
                    }
                    Err(error) => panic!("single-request witness failed: {error}"),
                }
            }
        });
        let environment = ProductionBoxEnvironment::new("127.0.0.1", port, "secret");
        let result = environment.remote_resource_accessor().execute_shell_result(&(), bxr_shell_args());
        finished.store(true, Ordering::SeqCst);
        server.join().expect("single-request witness");
        let error = result.expect_err(loss);
        assert!(error.outcome_unknown(), "{loss}: {error:?}");
        assert_eq!(requests.load(Ordering::SeqCst), 1, "{loss}: dispatched operation was replayed");
    }
}

fn bxr_forever_box_port(port: u16, pending: Arc<AtomicUsize>) -> ForeverBoxRunnerResourcePort {
    let service = Arc::new(ForeverBoxService::new(
        HostBox::new(ProductionBoxEnvironment::new("127.0.0.1", port, "secret")),
        Arc::new(NoopForeverBoxLifecycle), false, false, false,
    ));
    let watches = Arc::new(RunnerBackgroundShellWatches::new(
        Arc::new(|_, _, _| None),
        Some(Arc::new(move |_| { pending.fetch_add(1, Ordering::SeqCst); })),
        None, None,
    ));
    ForeverBoxRunnerResourcePort::new(service, "bxr-runner")
        .with_background_shell_watches(watches)
}

fn bxr_runner_shell(mode: &str) -> RunnerBoxShellRequest {
    RunnerBoxShellRequest {
        command: "printf bxr-contract".into(),
        working_directory: "/workspace".into(),
        tool_call_id: "bxr-runner-shell".into(),
        is_background: mode == "background",
        block_until_ms: match mode {
            "timed" => Some(75),
            "block-zero" => Some(0),
            _ => None,
        },
    }
}

#[test]
fn forever_box_runner_resource_port_preserves_unknown_shell_read_write_and_background_outcomes() {
    for mode in ["shell", "timed", "background", "block-zero", "read", "write"] {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind ForeverBox uncertainty peer");
        let port = listener.local_addr().unwrap().port();
        let calls = Arc::new(AtomicUsize::new(0));
        let server_calls = Arc::clone(&calls);
        let server = thread::spawn(move || {
            serve_unary_success(&listener, PING_PATH);
            let (mut stream, _) = listener.accept().expect("accept actual ForeverBox Exec");
            let identity = exec_request_identity(&read_request(&mut stream));
            server_calls.fetch_add(1, Ordering::SeqCst);
            let element = match mode {
                "read" => read_success_element("/workspace/bxr.txt", "unsettled"),
                "write" => write_success_element(),
                "background" | "block-zero" =>
                    background_shell_spawn_success(42, "printf bxr-contract", "/workspace", Some(123)),
                "timed" => shell_stream_backgrounded_element(42, "printf bxr-contract", "/workspace",
                    Some(123), Some(75), Some(SHELL_BACKGROUND_REASON_TIMEOUT)),
                _ => shell_success_element_with_output("unsettled", ""),
            };
            let element = echo_response_identity(&element, &identity);
            let mut body = connect_data_body(&[element]);
            // The operation output exists, but no correlated StreamClose does.
            body.extend_from_slice(&connect_envelope(2, br#"{}"#));
            stream.write_all(&length_response(&body)).expect("write unsettled response");
        });
        let pending = Arc::new(AtomicUsize::new(0));
        let resources = bxr_forever_box_port(port, Arc::clone(&pending));
        let result = match mode {
            "read" => resources.execute_read(RunnerBoxReadRequest {
                path: "/workspace/bxr.txt".into(), tool_call_id: "bxr-runner-read".into(),
                offset: None, limit: None, encoding_hint: None,
            }).map(|_| ()),
            "write" => resources.execute_write(RunnerBoxWriteRequest {
                path: "/workspace/bxr.txt".into(), data: b"payload".to_vec(),
                tool_call_id: "bxr-runner-write".into(),
            }),
            _ => resources.execute_shell(bxr_runner_shell(mode)).map(|_| ()),
        };
        server.join().expect("ForeverBox uncertainty peer");
        let error = result.expect_err(mode);
        assert!(matches!(error, ProviderSessionError::ToolOutcomeUnknown(_)),
            "{mode}: actual ForeverBox consumer lost uncertainty: {error:?}");
        assert_eq!(calls.load(Ordering::SeqCst), 1, "{mode}");
        assert_eq!(pending.load(Ordering::SeqCst), 0, "{mode}: unknown output must not register work");
    }
}

#[test]
fn forever_box_runner_resource_port_preserves_unknown_missing_terminal_and_invalid_background_id() {
    for scenario in ["missing terminal", "zero background id", "zero timed id"] {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind terminal-outcome peer");
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            serve_unary_success(&listener, PING_PATH);
            let (mut stream, _) = listener.accept().expect("accept terminal-outcome Exec");
            let identity = exec_request_identity(&read_request(&mut stream));
            let elements = match scenario {
                "missing terminal" => vec![shell_stream_start_element(), shell_stream_stdout_element("partial")],
                "zero background id" => vec![background_shell_spawn_success(0, "printf bxr-contract", "/workspace", None)],
                "zero timed id" => vec![shell_stream_backgrounded_element(0, "printf bxr-contract",
                    "/workspace", None, Some(75), Some(SHELL_BACKGROUND_REASON_TIMEOUT))],
                _ => unreachable!(),
            };
            let elements = elements.iter().map(|element| echo_response_identity(element, &identity))
                .collect::<Vec<_>>();
            stream.write_all(&length_response(&complete_connect_body(&elements, identity.id)))
                .expect("write complete RPC with unsettled operation");
        });
        let pending = Arc::new(AtomicUsize::new(0));
        let resources = bxr_forever_box_port(port, Arc::clone(&pending));
        let mode = if scenario == "zero background id" { "background" } else { "timed" };
        let result = resources.execute_shell(bxr_runner_shell(mode));
        server.join().expect("terminal-outcome peer");
        let error = result.expect_err(scenario);
        assert!(matches!(error, ProviderSessionError::ToolOutcomeUnknown(_)),
            "{scenario}: {error:?}");
        assert_eq!(pending.load(Ordering::SeqCst), 0,
            "{scenario}: invalid/unsettled operation must not register a watch");
    }
}

#[test]
fn forever_box_runner_resource_port_keeps_no_monitor_as_pre_dispatch_refusal() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind no-monitor readiness peer");
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        serve_unary_success(&listener, PING_PATH);
        listener
    });
    let resources = bxr_forever_box_port(port, Arc::new(AtomicUsize::new(0)));
    let result = resources.execute_computer_use_protobuf(Vec::new());
    let listener = server.join().expect("no-monitor readiness peer");
    listener.set_nonblocking(true).unwrap();
    let error = result.expect_err("no monitor");
    assert!(matches!(error, ProviderSessionError::Tool(_)), "{error:?}");
    assert!(error.to_string().to_ascii_lowercase().contains("monitor"));
    assert_eq!(listener.accept().expect_err("no ComputerUse request may be sent").kind(),
        std::io::ErrorKind::WouldBlock);
}

#[test]
fn production_exec_service_rejects_invalid_control_identity_and_multiple_control_cases() {
    let mut wrong_wire = Vec::new();
    push_len(1, b"", &mut wrong_wire);
    let mut contradictory = Vec::new();
    push_varint(1, 0, &mut contradictory);
    push_varint(1, 1, &mut contradictory);
    let mut overflow = Vec::new();
    push_varint(1, u64::from(u32::MAX) + 1, &mut overflow);
    for (name, nested) in [
        ("control id wrong wire", wrong_wire),
        ("control id contradictory", contradictory),
        ("control id overflow", overflow),
    ] {
        for case in [1, 2, 3] {
            let mut control = Vec::new();
            push_len(case, &nested, &mut control);
            let mut element = Vec::new();
            push_len(2, &control, &mut element);
            assert_unknown_transport(shell_with_reply(length_response(&complete_connect_body(
                &[shell_success_element(""), element], 0,
            ))).expect_err(name), name);
        }
    }
    let mut control = Vec::new();
    push_len(1, &[], &mut control);
    push_len(3, &[], &mut control);
    let mut element = Vec::new();
    push_len(2, &control, &mut element);
    assert_unknown_transport(shell_with_reply(length_response(&complete_connect_body(
        &[shell_success_element(""), element], 0,
    ))).expect_err("multiple control cases"), "multiple control cases");
}

#[test]
fn production_exec_service_rejects_ambiguous_nested_read_write_background_and_stream_results() {
    for operation in [OneShotOperation::Read, OneShotOperation::Write, OneShotOperation::BackgroundShell] {
        let element = operation.success_element();
        let client = fixture_message_field(&element, 1);
        let mut cursor = 0;
        let (field, _, result) = fixture_field(client, &mut cursor);
        let mut duplicate = result.to_vec();
        duplicate.extend_from_slice(result);
        for result in [Vec::new(), duplicate] {
            let element = raw_resource_element(field as u32, &result);
            let error = with_exec_reply(move |_| length_response(&complete_connect_body(&[element], 0)),
                |accessor| operation.call(accessor)).expect_err("ambiguous nested result");
            assert!(error.outcome_unknown(), "{operation:?}: {error:?}");
        }
    }
    let mut success = Vec::new();
    push_len(2, b"text", &mut success);
    push_len(5, b"bytes", &mut success);
    let mut read_result = Vec::new();
    push_len(1, &success, &mut read_result);
    let error = with_exec_reply(move |_| length_response(&complete_connect_body(
        &[raw_resource_element(7, &read_result)], 0,
    )), |accessor| accessor.execute_read(&(), bxr_read_args())).expect_err("two Read output cases");
    assert!(error.outcome_unknown());

    let mut stream_event = Vec::new();
    push_len(4, &[], &mut stream_event);
    push_len(3, &[], &mut stream_event);
    let error = with_exec_reply(move |_| length_response(&complete_connect_body(
        &[raw_resource_element(14, &stream_event)], 0,
    )), |accessor| accessor.execute_shell_stream(&(), ProductionShellStreamArgs {
        shell_args: bxr_shell_args(), timeout_ms: 75, hard_timeout_ms: None,
    })).expect_err("two ShellStream event cases");
    assert!(error.outcome_unknown());
}


/// Exercise complete production responses while witnessing the actual dispatch
/// count. No synthetic decoder or direct consumer projection is substituted.
fn with_counted_complete_exec_reply<T>(
    elements: Vec<Vec<u8>>,
    readiness_ping: bool,
    call: impl FnOnce(u16) -> T,
) -> T {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind counted complete-response peer");
    let port = listener.local_addr().expect("counted peer address").port();
    let finished = Arc::new(AtomicBool::new(false));
    let requests = Arc::new(AtomicUsize::new(0));
    let server_finished = Arc::clone(&finished);
    let server_requests = Arc::clone(&requests);
    let server = thread::spawn(move || {
        if readiness_ping {
            serve_unary_success(&listener, PING_PATH);
        }
        listener.set_nonblocking(true).expect("nonblocking counted peer");
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            assert!(Instant::now() < deadline, "counted complete-response peer timed out");
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let identity = exec_request_identity(&read_request(&mut stream));
                    assert!(identity.exec_id.is_empty(), "BXR must preserve the actual empty exec_id");
                    server_requests.fetch_add(1, Ordering::SeqCst);
                    let elements = elements.iter()
                        .map(|element| echo_response_identity(element, &identity))
                        .collect::<Vec<_>>();
                    let body = complete_connect_body(&elements, identity.id);
                    stream.write_all(&length_response(&body))
                        .expect("write complete correlated response");
                    stream.flush().expect("flush complete correlated response");
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if server_finished.load(Ordering::SeqCst) {
                        break;
                    }
                    thread::sleep(Duration::from_millis(1));
                }
                Err(error) => panic!("counted complete-response peer failed: {error}"),
            }
        }
    });
    let result = call(port);
    finished.store(true, Ordering::SeqCst);
    server.join().expect("counted complete-response peer");
    assert_eq!(requests.load(Ordering::SeqCst), 1,
        "the production call must dispatch exactly once and must not retry");
    result
}

#[test]
fn production_exec_service_rejects_shadowed_invalid_utf8_and_nested_metadata_once() {
    // These are raw frozen ShellResult bytes, including both adjudicated
    // counterexamples. The rest of the HTTP/Connect/RPC response is complete.
    let cases = [
        (
            "early invalid terminals_folder UTF-8",
            vec![0x0a, 0x00, 0xba, 0x06, 0x01, 0xff, 0xba, 0x06, 0x00],
            "non-UTF-8 protobuf string",
        ),
        (
            "early malformed sandbox_policy message",
            vec![0x0a, 0x00, 0xaa, 0x06, 0x01, 0x00, 0xaa, 0x06, 0x00],
            "invalid protobuf field number",
        ),
        (
            "early invalid stdout UTF-8",
            vec![0x0a, 0x07, 0x2a, 0x01, 0xff, 0x2a, 0x02, b'o', b'k'],
            "non-UTF-8 protobuf string",
        ),
        (
            "early invalid stderr UTF-8",
            vec![0x0a, 0x05, 0x32, 0x01, 0xff, 0x32, 0x00],
            "non-UTF-8 protobuf string",
        ),
        (
            "late invalid terminals_folder UTF-8 after a valid occurrence",
            vec![0x0a, 0x00, 0xba, 0x06, 0x00, 0xba, 0x06, 0x01, 0xff],
            "non-UTF-8 protobuf string",
        ),
        (
            "late malformed sandbox_policy after a valid occurrence",
            vec![0x0a, 0x00, 0xaa, 0x06, 0x00, 0xaa, 0x06, 0x01, 0x00],
            "invalid protobuf field number",
        ),
        (
            "late invalid stdout UTF-8 after a valid occurrence",
            vec![0x0a, 0x07, 0x2a, 0x02, b'o', b'k', 0x2a, 0x01, 0xff],
            "non-UTF-8 protobuf string",
        ),
        (
            "late invalid stderr UTF-8 after a valid occurrence",
            vec![0x0a, 0x05, 0x32, 0x00, 0x32, 0x01, 0xff],
            "non-UTF-8 protobuf string",
        ),
    ];
    for (scenario, shell_result, expected_cause) in cases {
        let result = with_counted_complete_exec_reply(
            vec![raw_resource_element(2, &shell_result)], false, |port| {
                ProductionBoxEnvironment::new("127.0.0.1", port, "secret")
                    .remote_resource_accessor()
                    .execute_shell_result(&(), bxr_shell_args())
            },
        );
        let error = result.expect_err(scenario);
        assert!(error.to_string().contains(expected_cause), "{scenario}: {error:?}");
        assert_unknown_transport(error, scenario);
    }
}

#[test]
fn production_exec_service_rejects_shadowed_uint32_overflow_once() {
    // Each UINT32 occurrence 80 80 80 80 10 is 2^32; a following valid value
    // must not hide it. No new int32 restriction is part of this oracle.
    let cases = [
        (
            "ShellResult pid metadata",
            2,
            vec![0x0a, 0x00, 0xc0, 0x06, 0x80, 0x80, 0x80, 0x80, 0x10, 0xc0, 0x06, 0x00],
        ),
        (
            "background shell_id",
            16,
            vec![0x0a, 0x08, 0x08, 0x80, 0x80, 0x80, 0x80, 0x10, 0x08, 0x2a],
        ),
        (
            "background pid",
            16,
            vec![0x0a, 0x0a, 0x08, 0x2a, 0x20, 0x80, 0x80, 0x80, 0x80, 0x10, 0x20, 0x7b],
        ),
        (
            "ShellStream Exit code",
            14,
            vec![0x1a, 0x08, 0x08, 0x80, 0x80, 0x80, 0x80, 0x10, 0x08, 0x00],
        ),
        (
            "ShellStream Backgrounded shell_id",
            14,
            vec![0x3a, 0x08, 0x08, 0x80, 0x80, 0x80, 0x80, 0x10, 0x08, 0x2a],
        ),
    ];
    for (scenario, field, protobuf_result) in cases {
        let result = with_counted_complete_exec_reply(
            vec![raw_resource_element(field, &protobuf_result)], false, |port| {
                let environment = ProductionBoxEnvironment::new("127.0.0.1", port, "secret");
                let mut accessor = environment.remote_resource_accessor();
                match field {
                    2 => accessor.execute_shell_result(&(), bxr_shell_args()).map(|_| ()),
                    16 => accessor.execute_background_shell_spawn(&(), bxr_shell_args().into()).map(|_| ()),
                    14 => accessor.execute_shell_stream(&(), ProductionShellStreamArgs {
                        shell_args: bxr_shell_args(), timeout_ms: 75, hard_timeout_ms: None,
                    }).map(|_| ()),
                    _ => unreachable!("declared typed occurrence case"),
                }
            },
        );
        let error = result.expect_err(scenario);
        assert!(error.outcome_unknown(), "{scenario}: {error:?}");
        assert!(error.to_string().contains("uint32 overflow"), "{scenario}: {error:?}");
    }
}

#[test]
fn production_exec_service_preserves_valid_repeated_typed_fields_and_last_values() {
    let mut success = Vec::new();
    // A normal ten-byte negative int32 followed by zero keeps existing int32
    // behavior; the uint32-specific fix must not add an int32 range policy.
    push_varint(3, u64::MAX, &mut success);
    push_varint(3, 0, &mut success);
    push_len(5, b"old stdout", &mut success);
    push_len(5, b"last stdout", &mut success);
    push_len(6, b"old stderr", &mut success);
    push_len(6, b"last stderr", &mut success);
    let mut shell = Vec::new();
    push_len(1, &success, &mut shell);
    push_len(101, &[], &mut shell);
    push_len(101, &[], &mut shell);
    push_len(103, b"/old/terminals", &mut shell);
    push_len(103, b"/new/terminals", &mut shell);
    push_varint(104, 41, &mut shell);
    push_varint(104, 42, &mut shell);
    let result = with_counted_complete_exec_reply(
        vec![raw_resource_element(2, &shell)], false, |port| {
            ProductionBoxEnvironment::new("127.0.0.1", port, "secret")
                .remote_resource_accessor().execute_shell_result(&(), bxr_shell_args())
        },
    ).expect("valid repeated Shell fields");
    assert_eq!(result, ProductionShellResult::Success {
        exit_code: 0, stdout: "last stdout".into(), stderr: "last stderr".into(),
    });

    let mut success = Vec::new();
    push_varint(1, 41, &mut success);
    push_varint(1, 42, &mut success);
    push_len(2, b"old command", &mut success);
    push_len(2, b"last command", &mut success);
    push_len(3, b"/workspace", &mut success);
    push_varint(4, 123, &mut success);
    push_varint(4, 124, &mut success);
    let mut spawn = Vec::new();
    push_len(1, &success, &mut spawn);
    let result = with_counted_complete_exec_reply(
        vec![raw_resource_element(16, &spawn)], false, |port| {
            ProductionBoxEnvironment::new("127.0.0.1", port, "secret")
                .remote_resource_accessor().execute_background_shell_spawn(&(), bxr_shell_args().into())
        },
    ).expect("valid repeated uint32 and string fields");
    assert_eq!(result, ProductionBackgroundShellSpawnResult::Success {
        shell_id: 42, command: "last command".into(), working_directory: "/workspace".into(),
        pid: Some(124),
    });

    let mut success = Vec::new();
    push_len(1, b"/workspace/bxr.txt", &mut success);
    push_len(5, &[0xff, 0x00], &mut success);
    push_len(5, &[0x80, 0xfe], &mut success);
    push_varint(4, 2, &mut success);
    let mut read = Vec::new();
    push_len(1, &success, &mut read);
    let result = with_counted_complete_exec_reply(
        vec![raw_resource_element(7, &read)], false, |port| {
            ProductionBoxEnvironment::new("127.0.0.1", port, "secret")
                .remote_resource_accessor().execute_read(&(), bxr_read_args())
        },
    ).expect("bytes remain opaque and retain the valid final value");
    assert_eq!(result, ProductionReadResult::Success {
        path: "/workspace/bxr.txt".into(), output: ProductionReadOutput::Data(vec![0x80, 0xfe]),
        total_lines: 0, file_size: 2, truncated: false, output_blob_id: None, range_applied: false,
    });
}

fn bxr_timed_terminal_element(kind: &str) -> Vec<u8> {
    match kind {
        "exit" => {
            let mut exit = Vec::new();
            push_varint(1, 0, &mut exit);
            push_len(2, b"/workspace", &mut exit);
            push_varint(6, 19, &mut exit);
            shell_stream_element(3, &exit)
        }
        "backgrounded" => shell_stream_backgrounded_element(
            42, "printf bxr-contract", "/workspace", Some(123), Some(75),
            Some(SHELL_BACKGROUND_REASON_TIMEOUT),
        ),
        "rejected" => shell_stream_element(5, &[]),
        "permission_denied" => shell_stream_element(6, &[]),
        "sandbox_unsupported" => shell_stream_element(9, &[]),
        _ => unreachable!("frozen decisive ShellStream terminal"),
    }
}

#[test]
fn forever_box_runner_timed_shell_rejects_repeated_or_conflicting_terminals_before_watch() {
    let terminals = ["exit", "backgrounded", "rejected", "permission_denied", "sandbox_unsupported"];
    for first in terminals {
        for second in terminals {
            let scenario = format!("{first} then {second}");
            let mut stderr = Vec::new();
            push_len(1, b"late warning", &mut stderr);
            let elements = vec![
                shell_stream_start_element(),
                shell_stream_stdout_element("before"),
                bxr_timed_terminal_element(first),
                shell_stream_element(8, &[]),
                shell_stream_stdout_element("between"),
                bxr_timed_terminal_element(second),
                shell_stream_element(2, &stderr),
            ];
            let pending = Arc::new(AtomicUsize::new(0));
            let result = with_counted_complete_exec_reply(elements, true, |port| {
                bxr_forever_box_port(port, Arc::clone(&pending)).execute_shell(bxr_runner_shell("timed"))
            });
            let error = result.expect_err(&scenario);
            assert!(matches!(error, ProviderSessionError::ToolOutcomeUnknown(_)),
                "{scenario}: the full received stream must be adjudicated: {error:?}");
            assert_eq!(pending.load(Ordering::SeqCst), 0,
                "{scenario}: no terminal may register work before later terminals are checked");
        }
    }
}

#[test]
fn forever_box_runner_timed_shell_preserves_post_exit_output_and_hook_metadata() {
    let mut stderr_before = Vec::new();
    push_len(1, b"before warning\n", &mut stderr_before);
    let mut stderr_after = Vec::new();
    push_len(1, b"after warning\n", &mut stderr_after);
    let elements = vec![
        shell_stream_start_element(),
        shell_stream_element(8, &[]),
        shell_stream_stdout_element("before "),
        shell_stream_element(2, &stderr_before),
        bxr_timed_terminal_element("exit"),
        shell_stream_element(8, &[]),
        shell_stream_stdout_element("after "),
        shell_stream_element(2, &stderr_after),
        shell_stream_element(8, &[]),
        shell_stream_stdout_element("tail"),
    ];
    let pending = Arc::new(AtomicUsize::new(0));
    let result = with_counted_complete_exec_reply(elements, true, |port| {
        bxr_forever_box_port(port, Arc::clone(&pending)).execute_shell(bxr_runner_shell("timed"))
    }).expect("one Exit with late output and nonterminal hook contexts");
    assert_eq!(result["kind"], "success");
    assert_eq!(result["exitCode"], 0);
    assert_eq!(result["stdout"], "before after tail");
    assert_eq!(result["stderr"], "before warning\nafter warning\n");
    assert_eq!(result["workingDirectory"], "/workspace");
    assert_eq!(result["elapsedMs"], 19);
    assert_eq!(result["aborted"], false);
    assert_eq!(pending.load(Ordering::SeqCst), 0);
}


// Frozen ShellSandboxUnsupported field3 is STRING (wire type 2), shared by
// BackgroundShellSpawnResult field5 and ShellStream field9. Policy bytes below
// deliberately include raw malformed fields; the surrounding response is valid.
fn bxr_sandbox_refusal_payload(policy_fields: &[u8]) -> Vec<u8> {
    let mut nested = Vec::new();
    push_len(1, b"printf bxr-contract", &mut nested);
    push_len(2, b"/workspace", &mut nested);
    nested.extend_from_slice(policy_fields);
    push_len(4, b"sandbox unavailable", &mut nested);
    push_varint(5, 1, &mut nested);
    nested
}

fn bxr_sandbox_refusal_element(resource_field: u32, nested: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    push_len(if resource_field == 16 { 5 } else { 9 }, nested, &mut result);
    raw_resource_element(resource_field, &result)
}

#[test]
fn production_exec_service_preserves_sandbox_refusal_string_defaults_and_last_values() {
    let mut unicode = Vec::new();
    push_len(3, "workspace_readonly/策略".as_bytes(), &mut unicode);
    let mut fallback = Vec::new();
    push_len(3, b"unknown", &mut fallback);
    let mut repeated = Vec::new();
    push_len(3, b"old-policy", &mut repeated);
    push_len(3, b"workspace_readonly", &mut repeated);
    let cases = [
        ("omitted string", Vec::new(), ""),
        ("explicit empty string", vec![0x1a, 0x00], ""),
        ("UTF-8 string", unicode, "workspace_readonly/策略"),
        ("producer fallback string", fallback, "unknown"),
        ("valid repeated strings", repeated, "workspace_readonly"),
    ];
    for (scenario, policy_fields, expected_policy) in cases {
        for field in [16, 14] {
            let element = bxr_sandbox_refusal_element(
                field, &bxr_sandbox_refusal_payload(&policy_fields),
            );
            with_counted_complete_exec_reply(vec![element], false, |port| {
                let environment = ProductionBoxEnvironment::new("127.0.0.1", port, "secret");
                let mut accessor = environment.remote_resource_accessor();
                if field == 16 {
                    let result = accessor
                        .execute_background_shell_spawn(&(), bxr_shell_args().into())
                        .expect(scenario);
                    assert_eq!(result, ProductionBackgroundShellSpawnResult::SandboxUnsupported {
                        command: "printf bxr-contract".into(),
                        working_directory: "/workspace".into(),
                        sandbox_policy_type: expected_policy.into(),
                        reason: "sandbox unavailable".into(),
                        is_readonly: true,
                    }, "{scenario}");
                } else {
                    let events = accessor.execute_shell_stream(&(), ProductionShellStreamArgs {
                        shell_args: bxr_shell_args(), timeout_ms: 75, hard_timeout_ms: None,
                    }).expect(scenario);
                    assert_eq!(events, vec![ProductionShellStreamEvent::SandboxUnsupported {
                        command: "printf bxr-contract".into(),
                        working_directory: "/workspace".into(),
                        sandbox_policy_type: expected_policy.into(),
                        reason: "sandbox unavailable".into(),
                        is_readonly: true,
                    }], "{scenario}");
                }
            });
        }
    }
}

fn bxr_malformed_sandbox_policy_cases() -> [(&'static str, Vec<u8>, &'static str); 6] {
    [
        ("varint instead of STRING", vec![0x18, 0x07], "wrong wire type"),
        ("varint before valid STRING", vec![0x18, 0x07, 0x1a, 0x00], "wrong wire type"),
        ("varint after valid STRING", vec![0x1a, 0x00, 0x18, 0x07], "wrong wire type"),
        ("invalid UTF-8", vec![0x1a, 0x01, 0xff], "non-UTF-8 protobuf string"),
        ("invalid then valid UTF-8", vec![0x1a, 0x01, 0xff, 0x1a, 0x00], "non-UTF-8 protobuf string"),
        ("valid then invalid UTF-8", vec![0x1a, 0x00, 0x1a, 0x01, 0xff], "non-UTF-8 protobuf string"),
    ]
}

#[test]
fn production_exec_service_rejects_malformed_sandbox_policy_strings_without_replay() {
    let cases = bxr_malformed_sandbox_policy_cases();
    for (scenario, policy_fields, expected_cause) in cases {
        for field in [16, 14] {
            let element = bxr_sandbox_refusal_element(
                field, &bxr_sandbox_refusal_payload(&policy_fields),
            );
            let result = with_counted_complete_exec_reply(vec![element], false, |port| {
                let environment = ProductionBoxEnvironment::new("127.0.0.1", port, "secret");
                let mut accessor = environment.remote_resource_accessor();
                if field == 16 {
                    accessor.execute_background_shell_spawn(&(), bxr_shell_args().into()).map(|_| ())
                } else {
                    accessor.execute_shell_stream(&(), ProductionShellStreamArgs {
                        shell_args: bxr_shell_args(), timeout_ms: 75, hard_timeout_ms: None,
                    }).map(|_| ())
                }
            });
            let error = result.expect_err(scenario);
            assert!(error.outcome_unknown(), "{scenario}, resource field {field}: {error:?}");
            assert!(error.to_string().contains(expected_cause),
                "{scenario}, resource field {field}: {error:?}");
        }
    }
}

struct BxrUnusedUpstream;

impl RoutedToolBridge for BxrUnusedUpstream {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        Ok(Vec::new())
    }

    fn call_tool(
        &self,
        _tool: &RoutedToolDefinition,
        _args: serde_json::Value,
        _tool_call_id: &str,
    ) -> Result<serde_json::Value, ProviderSessionError> {
        panic!("the real Box Shell route must not dispatch an upstream fallback")
    }
}

fn bxr_call_registered_runner_shell(
    port: u16,
    pending: Arc<AtomicUsize>,
    mode: &str,
) -> Result<serde_json::Value, ProviderSessionError> {
    let bridge = RunnerBoxToolBridge::new(
        Arc::new(BxrUnusedUpstream),
        Arc::new(bxr_forever_box_port(port, pending)),
    );
    let shell = bridge.list_tools().expect("real Runner Box tool catalog")
        .into_iter()
        .find(|tool| tool.name == RUNNER_BOX_SHELL_TOOL_NAME)
        .expect("real Runner Shell registration");
    let block_until_ms = match mode {
        "timed" => Some(75),
        "block-zero" => Some(0),
        _ => None,
    };
    bridge.call_tool(&shell, serde_json::json!({
        "command": "printf bxr-contract",
        "workingDirectory": "/workspace",
        "is_background": mode == "background",
        "block_until_ms": block_until_ms,
    }), "bxr-runner-shell")
}

fn bxr_sandbox_consumer_elements(mode: &str, nested: &[u8]) -> Vec<Vec<u8>> {
    if mode == "timed" {
        vec![
            shell_stream_start_element(),
            shell_stream_element(8, &[]),
            bxr_sandbox_refusal_element(14, nested),
            shell_stream_element(8, &[]),
        ]
    } else {
        vec![bxr_sandbox_refusal_element(16, nested)]
    }
}

#[test]
fn forever_box_runner_preserves_known_sandbox_refusals_without_watch_or_replay() {
    for mode in ["background", "block-zero", "timed"] {
        let mut policy_cases = Vec::new();
        for policy in ["", "unknown", "workspace_readonly", "workspace_readonly/策略"] {
            let mut policy_fields = Vec::new();
            if !policy.is_empty() {
                push_len(3, policy.as_bytes(), &mut policy_fields);
            }
            policy_cases.push((policy, policy_fields));
        }
        let mut repeated = Vec::new();
        push_len(3, b"old-policy", &mut repeated);
        push_len(3, b"workspace_readonly", &mut repeated);
        policy_cases.push(("workspace_readonly", repeated));
        for (policy, policy_fields) in policy_cases {
            let nested = bxr_sandbox_refusal_payload(&policy_fields);
            let elements = bxr_sandbox_consumer_elements(mode, &nested);
            let pending = Arc::new(AtomicUsize::new(0));
            let result = with_counted_complete_exec_reply(elements, true, |port| {
                bxr_call_registered_runner_shell(port, Arc::clone(&pending), mode)
            }).expect("complete known sandbox refusal through the real Runner bridge");
            assert_eq!(result, serde_json::json!({
                "kind": "sandboxUnsupported",
                "command": "printf bxr-contract",
                "workingDirectory": "/workspace",
                "sandboxPolicyType": policy,
                "reason": "sandbox unavailable",
                "isReadonly": true,
            }), "{mode}");
            assert_eq!(pending.load(Ordering::SeqCst), 0,
                "{mode}: a known refusal cannot register background work");
        }
    }
}


#[test]
fn forever_box_runner_preserves_unknown_malformed_sandbox_refusals_without_watch_or_replay() {
    for mode in ["background", "block-zero", "timed"] {
        for (scenario, policy_fields, expected_cause) in bxr_malformed_sandbox_policy_cases() {
            let nested = bxr_sandbox_refusal_payload(&policy_fields);
            let elements = bxr_sandbox_consumer_elements(mode, &nested);
            let pending = Arc::new(AtomicUsize::new(0));
            let result = with_counted_complete_exec_reply(elements, true, |port| {
                bxr_call_registered_runner_shell(port, Arc::clone(&pending), mode)
            });
            let error = result.expect_err(scenario);
            assert!(matches!(&error, ProviderSessionError::ToolOutcomeUnknown(_)),
                "{mode}, {scenario}: the real Runner must retain typed outcome uncertainty: {error:?}");
            assert!(error.to_string().contains(expected_cause),
                "{mode}, {scenario}: preserve the specific typed-field cause: {error:?}");
            assert_eq!(pending.load(Ordering::SeqCst), 0,
                "{mode}, {scenario}: malformed refusal cannot register background work");
        }
    }
}
