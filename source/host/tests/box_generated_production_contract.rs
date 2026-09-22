use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

use mahayana_host_runtime::r#box::box_file_transfer::{
    FileTransferAccessor, WriteExecResult,
};
use mahayana_host_runtime::r#box::box_shell_command::{
    HostShellArgsInput, build_host_shell_args,
};
use mahayana_host_runtime::r#box::box_windows::{
    ShellAccessor, ShellExecutionOutcome,
};
use mahayana_host_runtime::r#box::generated_production::{
    CONNECT_STREAM_CONTENT_TYPE, EXEC_PATH, PING_PATH,
};
use mahayana_host_runtime::r#box::box_factory::{
    format_sand_box_startup_summary, should_apply_shared_desktop,
};
use mahayana_host_runtime::r#box::production::ProductionBoxEnvironment;

fn encode_varint(mut value: u64, out: &mut Vec<u8>) {
    while value >= 0x80 {
        out.push(((value as u8) & 0x7f) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

fn push_varint(field: u8, value: u64, out: &mut Vec<u8>) {
    out.push(field << 3);
    encode_varint(value, out);
}

fn push_len(field: u8, value: &[u8], out: &mut Vec<u8>) {
    out.push((field << 3) | 2);
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

fn shell_success_element(stderr: &str) -> Vec<u8> {
    let mut success = Vec::new();
    push_varint(3, 0, &mut success);
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
    assert!(
        body.windows(expected_fragment.len())
            .any(|window| window == expected_fragment),
        "request protobuf did not contain expected payload"
    );

    let mut connect_body = connect_envelope(0, &element);
    connect_body.extend_from_slice(&connect_envelope(0x02, br#"{}"#));

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

    assert!(should_apply_shared_desktop(environment.loopback().max_windows()));
    assert_eq!(
        format_sand_box_startup_summary(true, true),
        "[sand-host] agent box backend: loopback (in-box); image: host's own container; auto-update: on; build: packaged"
    );

    server.join().expect("fake box daemon thread");
}
