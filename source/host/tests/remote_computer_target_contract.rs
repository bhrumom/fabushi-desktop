const TARGET_SOURCE: &str = include_str!("../src/remote_computer_target.rs");

#[test]
fn target_core_client_keeps_controller_and_device_credentials_secret() {
    for required in [
        "FabushiCoreCredentialProvider",
        "RemoteComputerDeviceSecretProvider",
        ".bearer_auth(credential.access_token())",
        ""deviceSecret": device_secret",
        ".field("client_token", &"<redacted>")",
    ] {
        assert!(TARGET_SOURCE.contains(required), "missing target credential invariant: {required}");
    }
    assert!(
        !TARGET_SOURCE.contains("#[derive(Debug, Clone, PartialEq, Eq)]\npub struct RemoteServiceClientGrant"),
        "service client grant must not derive Debug over clientToken"
    );
}

#[test]
fn target_service_grant_is_account_device_and_service_binding_correlated() {
    for required in [
        ""sessions", "grant"",
        ""connectionId": connection_id",
        ""delegationId": delegation_id",
        ""requestId": request_id",
        "response.get("serviceBound").and_then(Value::as_bool) != Some(true)",
        "response.get("targetGranted").and_then(Value::as_bool) != Some(true)",
        "response.get("connectionId").and_then(Value::as_str) != Some(connection_id)",
        "response.get("delegationId").and_then(Value::as_str) != Some(delegation_id)",
        "response.get("requestId").and_then(Value::as_str) != Some(request_id)",
    ] {
        assert!(TARGET_SOURCE.contains(required), "missing service grant correlation invariant: {required}");
    }
}

#[test]
fn target_operation_claim_and_result_keep_request_digest_and_generation_fences() {
    for required in [
        "response.get("requestId").and_then(Value::as_str) != Some(request_id)",
        "response.get("sessionId").and_then(Value::as_str) != Some(session_id)",
        "response.get("toolName").and_then(Value::as_str) != Some(expected_tool_name)",
        "response.get("requestSha256").and_then(Value::as_str) != Some(expected_request_sha256)",
        ".get("grantGeneration")",
        "response.get("resultSha256").and_then(Value::as_str) != Some(result_sha256)",
        "if operation.sensitive_result && result.is_some()",
    ] {
        assert!(TARGET_SOURCE.contains(required), "missing target operation invariant: {required}");
    }
}


#[test]
fn operation_digest_is_canonical_across_object_key_order() {
    use mahayana_host_runtime::remote_computer_target::operation_request_sha256;
    use serde_json::json;

    let left = operation_request_sha256(
        "computer_state",
        &json!({"z":1,"nested":{"b":2,"a":1},"list":[{"y":2,"x":1}]}),
        &json!({"impact":"none","reason":"no growth","targets":[]}),
    )
    .expect("canonical digest");
    let right = operation_request_sha256(
        "computer_state",
        &json!({"list":[{"x":1,"y":2}],"nested":{"a":1,"b":2},"z":1}),
        &json!({"targets":[],"reason":"no growth","impact":"none"}),
    )
    .expect("canonical digest");
    assert_eq!(left.len(), 64);
    assert_eq!(left, right);
    assert_ne!(
        left,
        operation_request_sha256(
            "computer_state",
            &json!({"list":[{"x":1,"y":2}],"nested":{"a":1,"b":3},"z":1}),
            &json!({"impact":"none","reason":"no growth","targets":[]}),
        )
        .expect("changed digest")
    );
}


#[test]
fn target_private_transport_is_peer_bound_generation_fenced_and_never_auto_replays() {
    for required in [
        "stream_listen(expected_peer, local_port, REMOTE_TARGET_STREAM_TIMEOUT_MS)",
        "RemoteTargetPrivateRequest::GrantController",
        "RemoteTargetPrivateRequest::AuthorizeTool",
        "RemoteTargetPrivateRequest::Operation",
        "operation_request_sha256(&tool_name, &arguments, &disk_preflight)",
        "claimed.client_id != client_id || claimed.grant_generation != grant_generation",
        "RemoteComputerNativeOutcome::OutcomeUnknown",
        "map_err(|_| RemoteComputerTargetError::OutcomeUnknown)",
        "binding.generation != expected_generation",
        "binding.expires_at <= unix_seconds()?",
    ] {
        assert!(TARGET_SOURCE.contains(required), "missing private target invariant: {required}");
    }
    assert!(
        TARGET_SOURCE.matches("self.require_current_network_binding(&device_id, network_generation)?;").count() >= 4,
        "target transport must fence network binding both before authorization/claim and again before native side effects"
    );
    assert_eq!(
        TARGET_SOURCE.matches("executor.execute(&claimed, &arguments, &disk_preflight)").count(),
        1,
        "a claimed side-effecting operation must be dispatched to the native executor at most once"
    );
    assert!(
        !TARGET_SOURCE.contains("executor.execute(&claimed, &arguments, &disk_preflight).retry"),
        "native execution must never be automatically retried after transport uncertainty"
    );
}
