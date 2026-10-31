use fabushi_telegram_runtime::{
    authorization_snapshot, close_client, create_client, execute_json, AuthorizationState,
};
use serde_json::{json, Value};

#[test]
fn mahayana_telegram_uses_the_single_runtime_identity_projection() {
    let client_id = create_client();
    let snapshot = authorization_snapshot(client_id).unwrap();
    assert_eq!(snapshot.state, AuthorizationState::WaitParameters);
    assert_eq!(snapshot.self_user_id, None);

    let state: Value = serde_json::from_str(&execute_json(
        client_id,
        &json!({"@type": "telegram.getAuthorizationState"}).to_string(),
    ))
    .unwrap();
    assert_eq!(state["ok"], true);
    assert_eq!(state["data"]["selfUserId"], Value::Null);
    close_client(client_id).unwrap();
}

#[test]
fn auth_lifecycle_commands_share_the_same_runtime_client_id() {
    let client_id = create_client();
    for request in [
        json!({"@type": "telegram.resendAuthenticationCode"}),
        json!({"@type": "telegram.cancelAuthenticationCode"}),
        json!({"@type": "telegram.sendVerifyEmailCode", "email": "a@example.test"}),
    ] {
        let response: Value =
            serde_json::from_str(&execute_json(client_id, &request.to_string())).unwrap();
        assert_eq!(response["ok"], false);
        assert_eq!(response["errorCode"], "auth_context_missing");
    }
    let snapshot = authorization_snapshot(client_id).unwrap();
    assert_eq!(snapshot.self_user_id, None);
    close_client(client_id).unwrap();
}
