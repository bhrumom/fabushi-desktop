use std::cell::Cell;
use std::collections::{BTreeMap, VecDeque};
use std::fmt;
use std::io;

use mahayana_host_runtime::r#box::box_remote_accessor::{
    BoxEndpoint, BoxPingControlClient, BoxPingErrorMetadata, BoxRemoteExecClient,
    BoxRemoteExecControlMessage, BoxRemoteExecEnvelope, BoxRemoteExecError,
    BoxRemoteExecManager, BoxTransportOptions, ConnectCode, classify_ping_failure,
    create_box_authorization_interceptor, create_box_remote_resource_accessor,
    create_box_transport, ping_box_transport_classified_with_now,
};

#[derive(Debug)]
struct FakePingError {
    code: Option<ConnectCode>,
    errno: Option<&'static str>,
    message: &'static str,
}

impl fmt::Display for FakePingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message)
    }
}

impl std::error::Error for FakePingError {}

impl BoxPingErrorMetadata for FakePingError {
    fn connect_code(&self) -> Option<ConnectCode> {
        self.code.clone()
    }

    fn system_errno(&self) -> Option<&str> {
        self.errno
    }
}

struct FakePingClient {
    result: Result<(), FakePingError>,
    timeout_seen: Cell<u64>,
}

impl BoxPingControlClient<String> for FakePingClient {
    type Error = FakePingError;

    fn ping(&mut self, _ctx: &String, timeout_ms: u64) -> Result<(), Self::Error> {
        self.timeout_seen.set(timeout_ms);
        std::mem::replace(&mut self.result, Ok(()))
    }
}

#[test]
fn box_remote_accessor_builds_authenticated_binary_http11_transport() {
    let mut endpoint = BoxEndpoint::new("127.0.0.1", 1337, "secret");
    endpoint
        .headers
        .insert("x-sand-display".into(), "2".into());

    let interceptor = create_box_authorization_interceptor(&endpoint);
    let mut headers = BTreeMap::new();
    interceptor.apply(&mut headers);
    assert_eq!(headers["Authorization"], "Bearer secret");
    assert_eq!(headers["x-sand-display"], "2");

    let options = create_box_transport(&endpoint, |options| options);
    assert_eq!(
        options,
        BoxTransportOptions {
            http_version: "1.1",
            base_url: "http://127.0.0.1:1337".into(),
            use_binary_format: true,
            interceptors: vec![interceptor],
        }
    );
}

#[test]
fn box_remote_accessor_classifies_ping_failures_and_latency_like_grok() {
    let deadline = FakePingError {
        code: Some(ConnectCode::Number(4)),
        errno: None,
        message: "deadline exceeded",
    };
    let classified = classify_ping_failure(&deadline);
    assert_eq!(classified.outcome, "timeout");
    assert_eq!(classified.cause_summary, "DeadlineExceeded");

    let refused = FakePingError {
        code: Some(ConnectCode::Number(14)),
        errno: Some("ECONNREFUSED"),
        message: "connect failed",
    };
    let classified = classify_ping_failure(&refused);
    assert_eq!(classified.outcome, "refused");
    assert_eq!(classified.cause_summary, "Unavailable/ECONNREFUSED");

    let clock = Cell::new(100u64);
    let outcome = ping_box_transport_classified_with_now(
        &"ctx".into(),
        &(),
        |_| FakePingClient {
            result: Err(FakePingError {
                code: Some(ConnectCode::Text("Unavailable".into())),
                errno: None,
                message: "socket crashed",
            }),
            timeout_seen: Cell::new(0),
        },
        500,
        || {
            let value = clock.get();
            clock.set(value + 25);
            value
        },
    );
    assert_eq!(outcome.outcome, "crash");
    assert_eq!(outcome.latency_ms, 25);
    assert_eq!(outcome.cause_summary.as_deref(), Some("Unavailable"));
}

#[derive(Default)]
struct FakeExecClient {
    calls: Vec<u64>,
    throw: bool,
}

impl BoxRemoteExecClient<String, u64, String> for FakeExecClient {
    type Error = io::Error;

    fn exec(
        &mut self,
        _ctx: &String,
        args: u64,
    ) -> Result<Vec<BoxRemoteExecEnvelope<String>>, Self::Error> {
        self.calls.push(args);
        if self.throw {
            Ok(vec![
                BoxRemoteExecEnvelope::ExecClientControlMessage(
                    BoxRemoteExecControlMessage::Heartbeat { id: args },
                ),
                BoxRemoteExecEnvelope::ExecClientControlMessage(
                    BoxRemoteExecControlMessage::Throw {
                        id: Some(args),
                        error: "remote exploded".into(),
                        stack_trace: Some("stack".into()),
                        error_code: Some("E_REMOTE".into()),
                    },
                ),
            ])
        } else {
            Ok(vec![
                BoxRemoteExecEnvelope::ExecClientControlMessage(
                    BoxRemoteExecControlMessage::Heartbeat { id: args },
                ),
                BoxRemoteExecEnvelope::ExecClientMessage {
                    id: args,
                    exec_id: String::new(),
                    message: format!("message-{args}"),
                },
                BoxRemoteExecEnvelope::ExecClientControlMessage(
                    BoxRemoteExecControlMessage::StreamClose { id: args },
                ),
            ])
        }
    }
}

#[test]
fn box_remote_exec_manager_allocates_ids_filters_control_and_surfaces_remote_throw() {
    let mut manager = BoxRemoteExecManager::new(FakeExecClient::default());
    let messages = manager
        .create_exec_instance(&"ctx".into(), |id| id)
        .expect("first remote exec");
    assert_eq!(messages, vec!["message-0"]);
    assert_eq!(manager.next_id(), 1);

    let messages = manager
        .create_exec_instance(&"ctx".into(), |id| id)
        .expect("second remote exec");
    assert_eq!(messages, vec!["message-1"]);
    assert_eq!(manager.next_id(), 2);

    let mut throwing = BoxRemoteExecManager::new(FakeExecClient {
        calls: Vec::new(),
        throw: true,
    });
    let error = throwing
        .create_exec_instance(&"ctx".into(), |id| id)
        .expect_err("throw control must surface");
    match error {
        BoxRemoteExecError::RemoteThrow {
            error,
            stack_trace,
            error_code,
        } => {
            assert_eq!(error, "remote exploded");
            assert_eq!(stack_trace.as_deref(), Some("stack"));
            assert_eq!(error_code.as_deref(), Some("E_REMOTE"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn box_remote_resource_accessor_composes_transport_client_and_manager() {
    let endpoint = BoxEndpoint::new("host", 1337, "token");
    let next_id = create_box_remote_resource_accessor(
        &endpoint,
        |options| options.base_url,
        |_transport| FakeExecClient::default(),
        |manager| manager.next_id(),
    );
    assert_eq!(next_id, 0);
}

struct ScriptedExecClient {
    calls: Vec<u64>,
    responses: VecDeque<Result<Vec<BoxRemoteExecEnvelope<String>>, io::Error>>,
}

impl BoxRemoteExecClient<(), u64, String> for ScriptedExecClient {
    type Error = io::Error;

    fn exec(
        &mut self,
        _ctx: &(),
        id: u64,
    ) -> Result<Vec<BoxRemoteExecEnvelope<String>>, Self::Error> {
        self.calls.push(id);
        self.responses.pop_front().expect("one response per actual request")
    }
}

fn scripted_manager(
    responses: Vec<Result<Vec<BoxRemoteExecEnvelope<String>>, io::Error>>,
) -> BoxRemoteExecManager<ScriptedExecClient> {
    BoxRemoteExecManager::new(ScriptedExecClient {
        calls: Vec::new(),
        responses: responses.into(),
    })
}

fn data(id: u64, text: &str) -> BoxRemoteExecEnvelope<String> {
    BoxRemoteExecEnvelope::ExecClientMessage {
        id,
        exec_id: String::new(),
        message: text.into(),
    }
}

fn close(id: u64) -> BoxRemoteExecEnvelope<String> {
    BoxRemoteExecEnvelope::ExecClientControlMessage(
        BoxRemoteExecControlMessage::StreamClose { id },
    )
}

fn heartbeat(id: u64) -> BoxRemoteExecEnvelope<String> {
    BoxRemoteExecEnvelope::ExecClientControlMessage(
        BoxRemoteExecControlMessage::Heartbeat { id },
    )
}

fn remote_throw(id: Option<u64>) -> BoxRemoteExecEnvelope<String> {
    BoxRemoteExecEnvelope::ExecClientControlMessage(
        BoxRemoteExecControlMessage::Throw {
            id,
            error: "remote error detail".into(),
            stack_trace: Some("remote stack".into()),
            error_code: Some("E_REMOTE".into()),
        },
    )
}

#[test]
fn box_remote_exec_manager_preserves_ordered_data_and_heartbeat_before_close() {
    let mut manager = scripted_manager(vec![
        Ok(vec![heartbeat(0), data(0, "first"), heartbeat(0), data(0, "second"), close(0)]),
        Ok(vec![data(1, "third"), close(1)]),
    ]);
    assert_eq!(
        manager.create_exec_instance(&(), |id| id).expect("completed stream"),
        vec!["first", "second"],
    );
    assert_eq!(
        manager.create_exec_instance(&(), |id| id).expect("nonzero identity"),
        vec!["third"],
    );
    assert_eq!(manager.into_client().calls, vec![0, 1]);
}

#[test]
fn box_remote_exec_manager_requires_matching_data_and_control_identity() {
    for (case, frames) in [
        ("data", vec![data(1, "foreign"), close(0)]),
        ("exec-id", vec![
            BoxRemoteExecEnvelope::ExecClientMessage {
                id: 0,
                exec_id: "foreign-execution".into(),
                message: "foreign".into(),
            },
            close(0),
        ]),
        ("heartbeat", vec![heartbeat(1), data(0, "value"), close(0)]),
        ("close", vec![data(0, "value"), close(1)]),
    ] {
        let mut manager = scripted_manager(vec![Ok(frames)]);
        let result = manager.create_exec_instance(&(), |id| id);
        assert!(
            matches!(result, Err(BoxRemoteExecError::ProtocolViolation(_))),
            "{case}: foreign identity must not be accepted: {result:?}",
        );
        assert_eq!(manager.into_client().calls, vec![0], "{case}: no replay");
    }
}

#[test]
fn box_remote_exec_manager_requires_one_final_stream_close() {
    for (case, frames) in [
        ("empty", vec![]),
        ("missing-close", vec![data(0, "value")]),
        ("duplicate-close", vec![data(0, "value"), close(0), close(0)]),
        ("data-after-close", vec![data(0, "value"), close(0), data(0, "late")]),
        ("heartbeat-after-close", vec![data(0, "value"), close(0), heartbeat(0)]),
        ("throw-after-close", vec![data(0, "value"), close(0), remote_throw(Some(0))]),
        ("absent-envelope", vec![BoxRemoteExecEnvelope::None, close(0)]),
        ("absent-control", vec![
            BoxRemoteExecEnvelope::ExecClientControlMessage(BoxRemoteExecControlMessage::None),
            close(0),
        ]),
    ] {
        let mut manager = scripted_manager(vec![Ok(frames)]);
        let result = manager.create_exec_instance(&(), |id| id);
        assert!(
            matches!(result, Err(BoxRemoteExecError::ProtocolViolation(_))),
            "{case}: incomplete/contradictory terminal must fail: {result:?}",
        );
        assert_eq!(manager.into_client().calls, vec![0], "{case}: no replay");
    }
}

#[test]
fn box_remote_exec_manager_default_throw_id_is_only_request_zero() {
    let mut first = scripted_manager(vec![Ok(vec![remote_throw(None)])]);
    let error = first.create_exec_instance(&(), |id| id).expect_err("matching default throw");
    match error {
        BoxRemoteExecError::RemoteThrow { error, stack_trace, error_code } => {
            assert_eq!(error, "remote error detail");
            assert_eq!(stack_trace.as_deref(), Some("remote stack"));
            assert_eq!(error_code.as_deref(), Some("E_REMOTE"));
        }
        other => panic!("default-zero Throw lost its detail: {other:?}"),
    }
    assert_eq!(first.into_client().calls, vec![0]);

    let mut later = scripted_manager(vec![Ok(vec![close(0)]), Ok(vec![remote_throw(None)])]);
    assert!(later.create_exec_instance(&(), |id| id).unwrap().is_empty());
    assert!(matches!(
        later.create_exec_instance(&(), |id| id),
        Err(BoxRemoteExecError::ProtocolViolation(_)),
    ));
    assert_eq!(later.into_client().calls, vec![0, 1]);
}

#[test]
fn box_remote_exec_manager_foreign_throw_cannot_become_remote_error() {
    let mut manager = scripted_manager(vec![Ok(vec![remote_throw(Some(1))])]);
    assert!(matches!(
        manager.create_exec_instance(&(), |id| id),
        Err(BoxRemoteExecError::ProtocolViolation(_)),
    ));
    assert_eq!(manager.into_client().calls, vec![0]);
}

#[test]
fn box_remote_exec_manager_preserves_transport_loss_without_replay_or_id_reuse() {
    let mut manager = scripted_manager(vec![
        Err(io::Error::new(io::ErrorKind::UnexpectedEof, "dispatched response lost")),
        Ok(vec![data(1, "next invocation"), close(1)]),
    ]);
    let first = manager.create_exec_instance(&(), |id| id);
    match first {
        Err(BoxRemoteExecError::Transport(error)) => {
            assert_eq!(error.kind(), io::ErrorKind::UnexpectedEof);
            assert_eq!(error.to_string(), "dispatched response lost");
        }
        other => panic!("transport loss was rewritten: {other:?}"),
    }
    assert_eq!(manager.next_id(), 1);
    assert_eq!(
        manager.create_exec_instance(&(), |id| id).unwrap(),
        vec!["next invocation"],
    );
    assert_eq!(manager.into_client().calls, vec![0, 1]);
}
