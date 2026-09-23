use std::collections::HashSet;

use mahayana_host_runtime::extensions::transcript::send_message_shaping::{
    UserMessageOptions, create_user_message,
};
use mahayana_host_runtime::extensions::transcript::send_thread_stamping::{
    apply_auto_reply_thread, resolve_send_reply_threading, validate_ai_reply_target,
};
use mahayana_host_runtime::extensions::transcript::transcript_entry_ids::{
    TranscriptEntryIdKind, count_user_messages, first_unused_id, next_entry_id,
};
use serde_json::json;

#[test]
fn transcript_entry_ids_match_frozen_turn_and_trailing_rules() {
    let entries = vec![
        json!({"id":"t0ua0","kind":"user-attachment","file_path":"/tmp/a"}),
        json!({"id":"t0u","kind":"message","role":"user","content":"one"}),
        json!({"id":"t0a0","kind":"message","role":"assistant","content":"a"}),
        json!({"id":"t0s0","kind":"send-message","message":{"type":"text","content":"s"}}),
    ];
    assert_eq!(count_user_messages(&entries), 1);
    assert_eq!(next_entry_id(&entries, TranscriptEntryIdKind::UserMessage), "t1u");
    assert_eq!(next_entry_id(&entries, TranscriptEntryIdKind::UserAttachment), "t1ua0");
    assert_eq!(next_entry_id(&entries, TranscriptEntryIdKind::AssistantMessage), "t0a1");
    assert_eq!(next_entry_id(&entries, TranscriptEntryIdKind::SendMessage), "t0s1");

    let boot = vec![json!({"id":"tb","kind":"notice","text":"boot"})];
    assert_eq!(next_entry_id(&boot, TranscriptEntryIdKind::AssistantMessage), "tba0");
    assert_eq!(
        first_unused_id(
            &HashSet::from(["x0".to_string(), "x1".to_string()]),
            |index| format!("x{index}"),
            0,
        ),
        "x2"
    );
}

#[test]
fn reply_threading_resolves_only_real_targets_and_builds_frozen_quote() {
    let entries = vec![
        json!({"id":"t0u","kind":"message","role":"user","content":"  hello   world  "}),
        json!({"id":"t0s0","kind":"send-message","message":{"type":"text","content":"reply"}}),
    ];
    let threaded = resolve_send_reply_threading(&entries, Some("t0u"), true);
    assert_eq!(threaded.reply_to_id.as_deref(), Some("t0u"));
    assert!(threaded.is_fork);
    let context = threaded.reply_context.expect("reply context");
    assert_eq!(context.target_id, "t0u");
    assert_eq!(context.quote, "hello world");

    let missing = resolve_send_reply_threading(&entries, Some("missing"), true);
    assert_eq!(missing.reply_to_id, None);
    assert!(!missing.is_fork);
    assert_eq!(missing.reply_context, None);
}

#[test]
fn ai_reply_validation_and_auto_threading_match_threadable_message_contract() {
    let entries = vec![json!({"id":"t0u","kind":"message","role":"user","content":"one"})];
    let valid = json!({"type":"text","content":"answer","reply_to":"t0u","extra":"drop"});
    assert_eq!(validate_ai_reply_target(&valid, None, &entries), valid);

    let missing = json!({"type":"text","content":"answer","reply_to":"missing","extra":"drop"});
    assert_eq!(
        validate_ai_reply_target(&missing, None, &entries),
        json!({"type":"text","content":"answer"})
    );
    assert_eq!(
        apply_auto_reply_thread(
            &json!({"type":"connector","connector":"gmail","variant":"card","reason":"r"}),
            Some("t0u"),
            &entries,
        ),
        json!({"type":"connector","connector":"gmail","variant":"card","reason":"r","reply_to":"t0u"})
    );
    let protected = json!({
        "type":"local-tool-permission",
        "reply_to":"missing",
        "ask":{"requestId":"r","status":"pending"}
    });
    assert_eq!(validate_ai_reply_target(&protected, None, &entries), protected);
}

#[test]
fn user_message_shaping_preserves_offline_and_thread_metadata() {
    let entry = create_user_message(
        "t1u",
        "hello",
        UserMessageOptions {
            composed_at_ms: Some(1234.0),
            rich_text: Some("**hello**".into()),
            reply_to: Some("t0u".into()),
            batch_id: Some("batch".into()),
            branched: true,
            client_nonce: Some("nonce".into()),
        },
        9999.0,
    );
    assert_eq!(entry["id"], "t1u");
    assert_eq!(entry["timestampMs"], 1234.0);
    assert_eq!(entry["sentWhileOfflineAtMs"], 1234.0);
    assert_eq!(entry["replyTo"], "t0u");
    assert_eq!(entry["branched"], true);
    assert_eq!(entry["clientNonce"], "nonce");
}
