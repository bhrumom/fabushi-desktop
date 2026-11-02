use fabushi_telegram_runtime::{
    close_client, create_client, create_persistent_client, execute_json,
    fabushi_telegram_close_client, fabushi_telegram_create_client,
    fabushi_telegram_create_persistent_client, fabushi_telegram_execute,
    fabushi_telegram_force_link, fabushi_telegram_free_string,
};
use serde_json::{json, Value};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use tempfile::tempdir;

fn execute(client_id: u64, request: Value) -> Value {
    serde_json::from_str(&execute_json(client_id, &request.to_string())).unwrap()
}

unsafe fn take_owned_ffi_json(pointer: *mut c_char) -> Value {
    assert!(!pointer.is_null(), "C ABI must return an owned JSON response");
    // Copy before freeing, then parse: even a malformed response is freed exactly once.
    let bytes = unsafe { CStr::from_ptr(pointer) }.to_bytes().to_vec();
    unsafe { fabushi_telegram_free_string(pointer) };
    serde_json::from_slice(&bytes).expect("C ABI response must be valid JSON")
}

fn execute_ffi(client_id: u64, request: Value) -> Value {
    let request = CString::new(request.to_string()).unwrap();
    unsafe { take_owned_ffi_json(fabushi_telegram_execute(client_id, request.as_ptr())) }
}

// Keep test-owned stores closed even if an assertion fails before the explicit close.
struct FfiClient(u64);

impl FfiClient {
    fn close(mut self) -> Value {
        let client_id = std::mem::replace(&mut self.0, 0);
        unsafe { take_owned_ffi_json(fabushi_telegram_close_client(client_id)) }
    }
}

impl Drop for FfiClient {
    fn drop(&mut self) {
        if self.0 != 0 {
            let response = fabushi_telegram_close_client(self.0);
            unsafe { fabushi_telegram_free_string(response) };
        }
    }
}

#[test]
fn flutter_friend_chat_payload_is_accepted_without_shape_translation() {
    assert_eq!(fabushi_telegram_force_link(), 1);
    let client = create_client();
    let chat = execute(
        client,
        json!({
            "@type": "telegram.executeCoreCommand",
            "command": {
                "type": "upsertChat",
                "chat": {
                    "id": 1_234_567_890,
                    "kind": "private",
                    "title": "Rust 好友",
                    "lastMessageId": null,
                    "lastReadInboxMessageId": null,
                    "lastReadOutboxMessageId": null,
                    "unreadCount": 0,
                    "pinnedMessageId": null,
                    "notificationSettings": {
                        "muteUntilUnixMs": null,
                        "soundId": null,
                        "showPreview": true
                    },
                    "isArchived": false,
                    "isMarkedUnread": false,
                    "draft": null,
                    "folderIds": []
                }
            }
        }),
    );
    assert_eq!(chat["ok"], true);

    let queued = execute(
        client,
        json!({
            "@type": "telegram.executeCoreCommand",
            "command": {
                "type": "queueMessage",
                "chatId": 1_234_567_890,
                "localMessageId": -1,
                "senderUserId": 1,
                "clientRequestId": "flutter-queue-1",
                "dateUnixMs": 1_720_000_000_000_i64,
                "content": {
                    "type": "text",
                    "data": {"text": "消息进入 Rust 队列", "entities": []}
                },
                "replyToMessageId": null,
                "messageThreadId": null
            }
        }),
    );
    assert_eq!(queued["ok"], true);
    assert_eq!(
        queued["data"]["state"]["messages"][0]["chatId"],
        1_234_567_890
    );
    assert_eq!(
        queued["data"]["state"]["messages"][0]["content"]["data"]["text"],
        "消息进入 Rust 队列"
    );
    assert_eq!(
        queued["data"]["state"]["messages"][0]["deliveryState"]["state"],
        "pending"
    );
    close_client(client).unwrap();
}

#[test]
fn each_client_has_isolated_core_and_authorization_state() {
    let first = create_client();
    let second = create_client();

    let result = execute(
        first,
        json!({
            "@type": "telegram.executeCoreCommand",
            "@extra": "request-1",
            "command": {
                "type": "upsertChat",
                "chat": {
                    "id": 42,
                    "kind": "private",
                    "title": "真实会话",
                    "lastMessageId": null,
                    "lastReadInboxMessageId": null,
                    "lastReadOutboxMessageId": null,
                    "unreadCount": 0,
                    "pinnedMessageId": null,
                    "notificationSettings": {
                        "muteUntilUnixMs": null,
                        "soundId": null,
                        "showPreview": true
                    },
                    "isArchived": false
                }
            }
        }),
    );
    assert_eq!(result["ok"], true);
    assert_eq!(result["data"]["@extra"], "request-1");
    assert_eq!(result["data"]["state"]["chats"]["42"]["title"], "真实会话");

    let second_state = execute(second, json!({"@type": "telegram.getState"}));
    assert_eq!(second_state["data"]["state"]["chats"], json!({}));

    let auth = execute(
        first,
        json!({
            "@type": "telegram.executeAuthorizationCommand",
            "command": {"type": "parametersAccepted"}
        }),
    );
    assert_eq!(
        auth["data"]["authorizationState"]["type"],
        "waitPhoneNumber"
    );

    close_client(first).unwrap();
    close_client(second).unwrap();
}

#[test]
fn ffi_boundary_returns_owned_json_and_handles_null_input() {
    let client = create_client();
    let request = CString::new(r#"{"@type":"telegram.getStatus"}"#).unwrap();
    let response_pointer = unsafe { fabushi_telegram_execute(client, request.as_ptr()) };
    let response = unsafe { take_owned_ffi_json(response_pointer) };
    assert_eq!(response["data"]["architecture"], "rust-command-event-core");

    let null_response = unsafe { fabushi_telegram_execute(client, std::ptr::null()) };
    let null_value = unsafe { take_owned_ffi_json(null_response) };
    assert_eq!(null_value["errorCode"], "invalid_ffi_request");
    close_client(client).unwrap();
}

#[test]
fn closed_or_unknown_clients_return_structured_errors() {
    let client = create_client();
    close_client(client).unwrap();
    let response = execute(client, json!({"@type": "telegram.getStatus"}));
    assert_eq!(response["ok"], false);
    assert_eq!(response["errorCode"], "client_not_found");
}

#[test]
fn api_connection_initialization_requires_product_configuration_and_transport() {
    let client = create_client();
    let missing = execute(client, json!({"@type": "telegram.initializeConnection"}));
    assert_eq!(missing["errorCode"], "invalid_parameter");

    let disconnected = execute(
        client,
        json!({
            "@type": "telegram.initializeConnection",
            "apiId": 12345,
            "deviceModel": "Fabushi Test",
            "systemVersion": "Rust",
            "appVersion": "0.1.0",
            "systemLangCode": "zh-Hans",
            "langPack": "",
            "langCode": "zh-hans"
        }),
    );
    assert_eq!(disconnected["errorCode"], "transport_not_connected");

    let missing_code_context = execute(
        client,
        json!({
            "@type": "telegram.submitAuthenticationCode",
            "code": "12345"
        }),
    );
    assert_eq!(missing_code_context["errorCode"], "auth_context_missing");
    let missing_resend_context = execute(
        client,
        json!({"@type": "telegram.resendAuthenticationCode"}),
    );
    assert_eq!(missing_resend_context["errorCode"], "auth_context_missing");
    let missing_cancel_context = execute(
        client,
        json!({"@type": "telegram.cancelAuthenticationCode"}),
    );
    assert_eq!(missing_cancel_context["errorCode"], "auth_context_missing");
    let missing_email_context = execute(
        client,
        json!({"@type": "telegram.sendVerifyEmailCode", "email": "a@example.test"}),
    );
    assert_eq!(missing_email_context["errorCode"], "auth_context_missing");
    let missing_password_context = execute(
        client,
        json!({
            "@type": "telegram.submitAuthenticationPassword",
            "password": "secret"
        }),
    );
    assert_eq!(
        missing_password_context["errorCode"],
        "auth_context_missing"
    );
    let sync_before_auth = execute(client, json!({"@type": "telegram.beginUpdateSync"}));
    assert_eq!(
        sync_before_auth["errorCode"],
        "authorization_command_failed"
    );
    close_client(client).unwrap();
}

#[test]
fn persistent_client_restores_committed_core_state() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("runtime.sqlite3");
    let path_string = path.to_string_lossy();
    let key = [11_u8; 32];
    let client = create_persistent_client(&path_string, &key).unwrap();
    let result = execute(
        client,
        json!({
            "@type": "telegram.executeCoreCommand",
            "command": {
                "type": "upsertChat",
                "chat": {
                    "id": 77,
                    "kind": "savedMessages",
                    "title": "持久化会话",
                    "lastMessageId": null,
                    "lastReadInboxMessageId": null,
                    "lastReadOutboxMessageId": null,
                    "unreadCount": 0,
                    "pinnedMessageId": null,
                    "notificationSettings": {
                        "muteUntilUnixMs": null,
                        "soundId": null,
                        "showPreview": true
                    },
                    "isArchived": false
                }
            }
        }),
    );
    assert_eq!(result["ok"], true);
    close_client(client).unwrap();

    let restored = create_persistent_client(&path_string, &key).unwrap();
    let status = execute(restored, json!({"@type": "telegram.getStatus"}));
    assert_eq!(status["data"]["persistentStorage"], true);
    let state = execute(restored, json!({"@type": "telegram.getState"}));
    assert_eq!(state["data"]["state"]["chats"]["77"]["title"], "持久化会话");
    close_client(restored).unwrap();
}

#[test]
#[ignore = "requires a live Telegram production data center"]
fn live_runtime_bootstrap_keeps_an_encrypted_transport_session() {
    let client = create_client();
    let ready = execute(
        client,
        json!({"@type": "telegram.bootstrapTransport", "dcId": 2}),
    );
    assert_eq!(ready["ok"], true, "{ready}");
    assert_eq!(ready["data"]["encryptedPingVerified"], true);
    assert!(ready["data"]["authKeyId"].as_str().is_some());

    let status = execute(client, json!({"@type": "telegram.getStatus"}));
    assert_eq!(status["data"]["transportConnected"], true);
    assert_eq!(status["data"]["dcId"], 2);
    close_client(client).unwrap();
}


#[test]
fn persistent_authorization_machine_state_is_restored_without_running_network() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("auth-runtime.sqlite3");
    let path_string = path.to_string_lossy();
    let key = [12_u8; 32];
    let client = create_persistent_client(&path_string, &key).unwrap();
    let configured = execute(
        client,
        json!({
            "@type": "telegram.executeAuthorizationCommand",
            "command": {"type": "parametersAccepted"}
        }),
    );
    assert_eq!(
        configured["data"]["authorizationState"]["type"],
        "waitPhoneNumber"
    );
    close_client(client).unwrap();

    let restored = create_persistent_client(&path_string, &key).unwrap();
    let state = execute(
        restored,
        json!({"@type": "telegram.getAuthorizationState"}),
    );
    assert_eq!(
        state["data"]["authorizationState"]["type"],
        "waitPhoneNumber"
    );
    close_client(restored).unwrap();
}

#[test]
fn recovery_and_challenge_commands_do_not_fabricate_ready_without_required_state() {
    let client = create_client();
    for request in [
        json!({"@type": "telegram.requestPasswordRecovery"}),
        json!({"@type": "telegram.checkRecoveryPassword", "code": "123456"}),
        json!({"@type": "telegram.refreshPasswordChallenge"}),
    ] {
        let response = execute(client, request);
        assert_eq!(response["ok"], false);
        assert_eq!(response["errorCode"], "authorization_command_failed");
    }
    close_client(client).unwrap();
}


#[test]
fn authorization_projection_exposes_canonical_self_identity_field() {
    let client = create_client();
    let state = execute(client, json!({"@type": "telegram.getAuthorizationState"}));
    assert_eq!(state["ok"], true);
    assert!(state["data"].get("selfUserId").is_some());
    assert_eq!(state["data"]["selfUserId"], Value::Null);
    close_client(client).unwrap();
}

#[test]
fn lifecycle_commands_share_the_same_runtime_identity_projection() {
    let client = create_client();
    let status = execute(client, json!({"@type": "telegram.getStatus"}));
    assert_eq!(status["ok"], true);
    assert!(status["data"].get("authorizationState").is_some());
    assert!(status["data"].get("selfUserId").is_some());

    for request in [
        json!({"@type": "telegram.resendAuthenticationCode"}),
        json!({"@type": "telegram.cancelAuthenticationCode"}),
        json!({"@type": "telegram.sendVerifyEmailCode", "email": "a@example.test"}),
    ] {
        let response = execute(client, request);
        assert_eq!(response["ok"], false);
        assert_eq!(response["errorCode"], "auth_context_missing");
    }
    close_client(client).unwrap();
}

#[test]
fn exported_c_lifecycle_returns_owned_json_and_rejects_closed_handles() {
    let client = FfiClient(fabushi_telegram_create_client());
    assert_ne!(client.0, 0);
    let client_id = client.0;
    let status = execute_ffi(client_id, json!({"@type": "telegram.getStatus"}));
    assert_eq!(status["ok"], true);
    assert_eq!(status["data"]["clientId"], client_id);
    assert_eq!(status["data"]["persistentStorage"], false);

    let configured = execute_ffi(
        client_id,
        json!({
            "@type": "telegram.executeAuthorizationCommand",
            "command": {"type": "parametersAccepted"}
        }),
    );
    assert_eq!(configured["ok"], true);
    assert_eq!(
        configured["data"]["authorizationState"]["type"],
        "waitPhoneNumber"
    );

    let closed = client.close();
    assert_eq!(
        closed,
        json!({"ok": true, "data": {"@type": "client.closed", "clientId": client_id}})
    );
    for missing_id in [client_id, 0] {
        let rejected = execute_ffi(missing_id, json!({"@type": "telegram.getStatus"}));
        assert_eq!(rejected["ok"], false);
        assert_eq!(rejected["errorCode"], "client_not_found");
        let closed_again =
            unsafe { take_owned_ffi_json(fabushi_telegram_close_client(missing_id)) };
        assert_eq!(closed_again["ok"], false);
        assert_eq!(closed_again["errorCode"], "client_not_found");
    }

    let fresh = FfiClient(fabushi_telegram_create_client());
    assert_ne!(fresh.0, 0);
    assert_ne!(fresh.0, client_id);
    let fresh_state = execute_ffi(fresh.0, json!({"@type": "telegram.getAuthorizationState"}));
    assert_eq!(fresh_state["ok"], true);
    assert_eq!(
        fresh_state["data"]["authorizationState"]["type"],
        "waitParameters"
    );
    assert_eq!(fresh.close()["ok"], true);
    unsafe { fabushi_telegram_free_string(std::ptr::null_mut()) };
}

#[test]
fn exported_c_persistent_client_restores_committed_state_after_close() {
    let directory = tempdir().unwrap();
    let database = directory.path().join("c-wrapper.sqlite3");
    let path = CString::new(database.to_str().unwrap()).unwrap();
    let key = [23_u8; 32];
    let created = unsafe {
        take_owned_ffi_json(fabushi_telegram_create_persistent_client(
            path.as_ptr(),
            key.as_ptr(),
            key.len(),
        ))
    };
    assert_eq!(created["ok"], true);
    let client = FfiClient(created["data"]["clientId"].as_u64().unwrap());
    assert_ne!(client.0, 0);
    let first_id = client.0;
    assert_eq!(created["data"]["@type"], "client.created");
    assert_eq!(created["data"]["persistentStorage"], true);
    assert_eq!(
        created["data"]["authorizationState"]["type"],
        "waitParameters"
    );
    assert!(created["data"].get("selfUserId").is_some());
    assert_eq!(created["data"]["selfUserId"], Value::Null);

    let committed = execute_ffi(
        client.0,
        json!({
            "@type": "telegram.executeCoreCommand",
            "command": {
                "type": "upsertChat",
                "chat": {
                    "id": 877,
                    "kind": "savedMessages",
                    "title": "C ABI 持久化会话",
                    "lastMessageId": null,
                    "lastReadInboxMessageId": null,
                    "lastReadOutboxMessageId": null,
                    "unreadCount": 0,
                    "pinnedMessageId": null,
                    "notificationSettings": {
                        "muteUntilUnixMs": null,
                        "soundId": null,
                        "showPreview": true
                    },
                    "isArchived": false
                }
            }
        }),
    );
    assert_eq!(committed["ok"], true);
    let configured = execute_ffi(
        client.0,
        json!({
            "@type": "telegram.executeAuthorizationCommand",
            "command": {"type": "parametersAccepted"}
        }),
    );
    assert_eq!(configured["ok"], true);
    assert_eq!(
        configured["data"]["authorizationState"]["type"],
        "waitPhoneNumber"
    );
    assert_eq!(client.close()["data"]["clientId"], first_id);
    assert!(
        database.is_file(),
        "closing a C handle must retain its synthetic store"
    );

    let reopened = unsafe {
        take_owned_ffi_json(fabushi_telegram_create_persistent_client(
            path.as_ptr(),
            key.as_ptr(),
            key.len(),
        ))
    };
    assert_eq!(reopened["ok"], true);
    let restored = FfiClient(reopened["data"]["clientId"].as_u64().unwrap());
    assert_ne!(restored.0, 0);
    assert_ne!(restored.0, first_id);
    assert_eq!(reopened["data"]["persistentStorage"], true);
    assert_eq!(
        reopened["data"]["authorizationState"]["type"],
        "waitPhoneNumber"
    );
    assert!(reopened["data"].get("selfUserId").is_some());
    assert_eq!(reopened["data"]["selfUserId"], Value::Null);
    let state = execute_ffi(restored.0, json!({"@type": "telegram.getState"}));
    assert_eq!(state["ok"], true);
    assert_eq!(
        state["data"]["state"]["chats"]["877"]["title"],
        "C ABI 持久化会话"
    );
    assert_eq!(restored.close()["ok"], true);
    assert!(database.is_file());
}

#[test]
fn exported_c_execute_rejects_null_non_utf8_and_invalid_json() {
    let client = FfiClient(fabushi_telegram_create_client());
    assert_ne!(client.0, 0);
    let null = unsafe {
        take_owned_ffi_json(fabushi_telegram_execute(client.0, std::ptr::null()))
    };
    assert_eq!(null["ok"], false);
    assert_eq!(null["errorCode"], "invalid_ffi_request");

    // CString owns a valid NUL-terminated allocation; only its UTF-8 payload is invalid.
    let non_utf8 = CString::new(vec![0xff]).unwrap();
    let rejected = unsafe {
        take_owned_ffi_json(fabushi_telegram_execute(client.0, non_utf8.as_ptr()))
    };
    assert_eq!(rejected["ok"], false);
    assert_eq!(rejected["errorCode"], "invalid_ffi_request");

    let invalid_json = CString::new("{").unwrap();
    let rejected = unsafe {
        take_owned_ffi_json(fabushi_telegram_execute(client.0, invalid_json.as_ptr()))
    };
    assert_eq!(rejected["ok"], false);
    assert_eq!(rejected["errorCode"], "invalid_json");
    let intact = execute_ffi(client.0, json!({"@type": "telegram.getAuthorizationState"}));
    assert_eq!(intact["ok"], true);
    assert_eq!(
        intact["data"]["authorizationState"]["type"],
        "waitParameters"
    );
    assert_eq!(client.close()["ok"], true);
}

#[test]
fn exported_c_persistent_create_rejects_null_and_non_utf8_inputs_before_storage() {
    let directory = tempdir().unwrap();
    let database = directory.path().join("rejected-path.sqlite3");
    let path = CString::new(database.to_str().unwrap()).unwrap();
    let key = [24_u8; 32];
    let null_path = unsafe {
        take_owned_ffi_json(fabushi_telegram_create_persistent_client(
            std::ptr::null(),
            key.as_ptr(),
            key.len(),
        ))
    };
    assert_eq!(null_path["ok"], false);
    assert_eq!(null_path["errorCode"], "null_request_pointer");

    let non_utf8_path = CString::new(vec![0xff]).unwrap();
    let rejected_path = unsafe {
        take_owned_ffi_json(fabushi_telegram_create_persistent_client(
            non_utf8_path.as_ptr(),
            key.as_ptr(),
            key.len(),
        ))
    };
    assert_eq!(rejected_path["ok"], false);
    assert_eq!(rejected_path["errorCode"], "invalid_utf8");

    let null_key = unsafe {
        take_owned_ffi_json(fabushi_telegram_create_persistent_client(
            path.as_ptr(),
            std::ptr::null(),
            key.len(),
        ))
    };
    assert_eq!(null_key["ok"], false);
    assert_eq!(null_key["errorCode"], "null_request_pointer");
    assert!(!database.exists());
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
}

#[test]
fn exported_c_persistent_create_rejects_invalid_key_lengths_before_storage() {
    let directory = tempdir().unwrap();
    let database = directory.path().join("rejected-key.sqlite3");
    let path = CString::new(database.to_str().unwrap()).unwrap();
    // Every declared length fits this real allocation, including the 33-byte negative.
    let key_buffer = [25_u8; 33];
    for length in [0_usize, 31, 33] {
        let rejected = unsafe {
            take_owned_ffi_json(fabushi_telegram_create_persistent_client(
                path.as_ptr(),
                key_buffer.as_ptr(),
                length,
            ))
        };
        assert_eq!(rejected["ok"], false, "length={length}: {rejected}");
        assert_eq!(rejected["errorCode"], "storage_failed");
        assert!(!database.exists());
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
    }

    // The same valid path and allocation succeed when exactly 32 bytes are supplied.
    let created = unsafe {
        take_owned_ffi_json(fabushi_telegram_create_persistent_client(
            path.as_ptr(),
            key_buffer.as_ptr(),
            32,
        ))
    };
    assert_eq!(created["ok"], true);
    let client = FfiClient(created["data"]["clientId"].as_u64().unwrap());
    assert_ne!(client.0, 0);
    assert_eq!(client.close()["ok"], true);
    assert!(database.is_file());
}

