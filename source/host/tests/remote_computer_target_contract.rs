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
