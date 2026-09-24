use mahayana_host_runtime::extensions::inference::provider_session::ProviderMessage;
use mahayana_host_runtime::runner::production_turn_input_projection::create_production_turn_input_projection;

#[test]
fn production_turn_projection_binds_stream_identity_and_recovery_shape() {
    let args = serde_json::json!({
        "messageId": "msg-1",
        "recentUserMessages": [
            {"id":"msg-0","text":"older"},
            {"id":"msg-1","text":"hello from durable history"}
        ],
        "isFork": true,
        "attachmentPaths": ["/tmp/a", "/tmp/b"],
        "selectedImages": [{"id":"image"}],
        "selectedVideos": [{"id":"video"}],
        "replyContext": {"id":"reply"}
    });
    let messages = vec![ProviderMessage {
        role: "user".into(),
        content: "hello from the real turn".into(),
    }];

    let projected = create_production_turn_input_projection(
        &args,
        "stream-request-123",
        &messages,
    )
    .expect("projection");

    assert_eq!(
        projected.options.inference_request_id.as_deref(),
        Some("stream-request-123")
    );
    assert_eq!(projected.options.message_id.as_deref(), Some("msg-1"));
    assert_eq!(
        projected.options.recent_message_text.as_deref(),
        Some("hello from durable history")
    );
    assert_eq!(
        projected
            .options
            .recent_user_messages
            .iter()
            .map(|message| (message.id.as_str(), message.text.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("msg-0", "older"),
            ("msg-1", "hello from durable history")
        ]
    );
    assert!(projected.options.is_fork);
    assert_eq!(projected.options.attachment_count, 2);
    assert_eq!(projected.options.image_count, 1);
    assert_eq!(projected.options.video_count, 1);
    assert!(projected.options.has_reply_context);
}

#[test]
fn production_turn_projection_uses_latest_user_fallback_without_durable_history() {
    let messages = vec![ProviderMessage {
        role: "user".into(),
        content: "fallback user text".into(),
    }];
    let projected = create_production_turn_input_projection(
        &serde_json::json!({"messageId":"msg-missing"}),
        "stream-fallback",
        &messages,
    )
    .expect("projection");
    assert!(projected.options.recent_user_messages.is_empty());
    assert_eq!(
        projected.options.recent_message_text.as_deref(),
        Some("fallback user text")
    );
    assert!(!projected.options.is_fork);
}

#[test]
fn production_turn_projection_rejects_missing_request_identity() {
    let messages = vec![ProviderMessage {
        role: "user".into(),
        content: "hello".into(),
    }];
    assert!(
        create_production_turn_input_projection(
            &serde_json::json!({}),
            "   ",
            &messages,
        )
        .is_err()
    );
}
