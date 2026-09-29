use std::sync::Arc;

use mahayana_host_runtime::extensions::local_exec::production::{
    GatewayExecControl, ProductionExecClientPayload, ProductionShellStreamEvent,
    PRODUCTION_LOCAL_EXEC_CODEC,
};
use serde_json::json;

#[test]
fn production_codec_accepts_exec_client_json_and_ignores_unknown_fields() {
    let decoded = PRODUCTION_LOCAL_EXEC_CODEC
        .decode_client(json!({
            "id": 9,
            "execId": "exec-9",
            "futureField": {"nested": true}
        }))
        .expect("decode client");
    assert_eq!(decoded.generated().id, 9);
    assert_eq!(decoded.generated().exec_id, "exec-9");
    assert!(matches!(decoded.payload(), ProductionExecClientPayload::Other));
    assert!(
        PRODUCTION_LOCAL_EXEC_CODEC
            .decode_client(json!(["not", "a", "message"]))
            .is_err()
    );
}

#[test]
fn production_codec_projects_frozen_control_cases() {
    assert_eq!(
        PRODUCTION_LOCAL_EXEC_CODEC
            .decode_control(&json!({
                "throw": {
                    "id": 7,
                    "error": "boom",
                    "stackTrace": "stack",
                    "errorCode": "E_BOOM",
                    "unknownField": true
                }
            }))
            .expect("throw"),
        GatewayExecControl::Throw {
            error: "boom".into(),
            stack_trace: Some("stack".into()),
        }
    );
    assert_eq!(
        PRODUCTION_LOCAL_EXEC_CODEC
            .decode_control(&json!({"streamClose": {"id": 7}}))
            .expect("stream close"),
        GatewayExecControl::StreamClose
    );
    assert_eq!(
        PRODUCTION_LOCAL_EXEC_CODEC
            .decode_control(&json!({"heartbeat": {"id": 7}, "future": true}))
            .expect("heartbeat"),
        GatewayExecControl::Unknown
    );
    assert_eq!(
        PRODUCTION_LOCAL_EXEC_CODEC
            .decode_control(&json!({"futureControl": {"id": 7}}))
            .expect("unknown"),
        GatewayExecControl::Unknown
    );
    assert!(
        PRODUCTION_LOCAL_EXEC_CODEC
            .decode_control(&json!({
                "throw": {"error": "boom"},
                "streamClose": {}
            }))
            .is_err()
    );
}

#[test]
fn production_codec_creates_package_owned_remote_accessor() {
    let manager = Arc::new(String::from("manager"));
    let accessor = PRODUCTION_LOCAL_EXEC_CODEC.create_remote_accessor(Arc::clone(&manager));
    assert!(Arc::ptr_eq(&manager, &accessor.manager()));
}


#[test]
fn production_codec_projects_generated_shell_and_read_shapes() {
    let shell_request = PRODUCTION_LOCAL_EXEC_CODEC
        .shell_stream_server_message(4, "pwd", "/tmp", "tool-shell");
    assert_eq!(shell_request["id"], 4);
    assert_eq!(shell_request["shellStreamArgs"]["command"], "pwd");
    assert_eq!(shell_request["shellStreamArgs"]["workingDirectory"], "/tmp");
    assert_eq!(shell_request["shellStreamArgs"]["toolCallId"], "tool-shell");

    let shell = PRODUCTION_LOCAL_EXEC_CODEC
        .decode_client(json!({
            "id": 4,
            "shellStream": {"stdout": {"data": "hello\n"}},
            "futureField": true
        }))
        .expect("shell stream");
    assert_eq!(
        shell.payload(),
        &ProductionExecClientPayload::ShellStream(
            ProductionShellStreamEvent::Stdout("hello\n".into())
        )
    );

    let read_request = PRODUCTION_LOCAL_EXEC_CODEC
        .read_server_message(5, "/tmp/a.txt", "tool-read", Some(3), Some(7), Some("utf-8"));
    assert_eq!(read_request["readArgs"]["path"], "/tmp/a.txt");
    assert_eq!(read_request["readArgs"]["offset"], 3);
    assert_eq!(read_request["readArgs"]["limit"], 7);
    assert_eq!(read_request["readArgs"]["encodingHint"], "utf-8");

    let read = PRODUCTION_LOCAL_EXEC_CODEC
        .decode_client(json!({
            "id": 5,
            "readResult": {"success": {"path": "/tmp/a.txt", "content": "hello"}}
        }))
        .expect("read result");
    assert!(matches!(
        read.payload(),
        ProductionExecClientPayload::ReadResult(value)
            if value["success"]["content"] == "hello"
    ));
}


#[test]
fn production_codec_uses_generated_oneof_and_permission_error_semantics() {
    assert!(
        PRODUCTION_LOCAL_EXEC_CODEC
            .decode_client(json!({
                "shellStream": {"stdout": {"data": "hello"}},
                "readResult": {"success": {"path": "/tmp/a", "content": "world"}}
            }))
            .is_err(),
        "generated ExecClientMessage oneof must reject multiple branches"
    );

    let denied = PRODUCTION_LOCAL_EXEC_CODEC
        .decode_client(json!({
            "id": 11,
            "shellStream": {
                "permissionDenied": {
                    "command": "rm",
                    "workingDirectory": "/tmp",
                    "error": "policy denied",
                    "isReadonly": false,
                    "futureField": true
                }
            },
            "futureField": true
        }))
        .expect("permission denied");
    assert_eq!(
        denied.payload(),
        &ProductionExecClientPayload::ShellStream(
            ProductionShellStreamEvent::PermissionDenied("policy denied".into())
        )
    );
}
