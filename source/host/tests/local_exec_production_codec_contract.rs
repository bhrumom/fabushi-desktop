use std::sync::Arc;

use mahayana_host_runtime::extensions::local_exec::production::{
    GatewayExecControl, PRODUCTION_LOCAL_EXEC_CODEC,
};
use serde_json::json;

#[test]
fn production_codec_accepts_exec_client_json_and_ignores_unknown_fields() {
    let decoded = PRODUCTION_LOCAL_EXEC_CODEC
        .decode_client(json!({
            "message": {"shellResult": {"exitCode": 0}},
            "futureField": {"nested": true}
        }))
        .expect("decode client");
    assert_eq!(decoded.as_json()["futureField"]["nested"], true);
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
