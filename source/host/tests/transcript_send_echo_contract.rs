use std::collections::HashSet;

use mahayana_host_runtime::extensions::transcript::send_message_shaping::{
    Reaction, UserAttachmentOptions, UserMessageOptions, build_composed_offline_note,
    build_selected_videos, collect_inbound_images, create_user_attachment_entry,
    create_user_message, skippable_prompt_summary, split_attachment_paths_by_channel,
    toggle_reaction,
};
use mahayana_host_runtime::selected_image_inputs::read_image_dimensions;
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


#[test]
fn frozen_send_shaping_helpers_cover_reactions_widgets_offline_and_media_channels() {
    let first = toggle_reaction(None, "👍", "me").expect("reaction");
    assert_eq!(
        first,
        vec![Reaction {
            emoji: "👍".into(),
            by: "me".into(),
        }]
    );
    assert_eq!(toggle_reaction(Some(&first), "👍", "me"), None);

    assert_eq!(
        build_composed_offline_note(0.0),
        "[Composed offline at 1970-01-01T00:00:00.000Z]"
    );
    assert_eq!(build_composed_offline_note(f64::NAN), "");

    assert_eq!(
        skippable_prompt_summary(&json!({
            "type":"widget",
            "widget":{
                "prompt":"Choose",
                "options":[{"label":"A"},{"label":"B"}]
            }
        }))
        .as_deref(),
        Some("Choose — A / B")
    );
    assert_eq!(
        skippable_prompt_summary(&json!({"type":"widget","widget":{}})).as_deref(),
        Some("Question")
    );

    let channels = split_attachment_paths_by_channel([
        "/tmp/a.png",
        "/tmp/b.MOV",
        "/tmp/c.txt",
    ]);
    assert_eq!(channels.image_attachment_paths, vec!["/tmp/a.png"]);
    assert_eq!(channels.video_attachment_paths, vec!["/tmp/b.MOV"]);
    assert_eq!(channels.file_attachment_paths, vec!["/tmp/c.txt"]);

    let videos = build_selected_videos(["/tmp/a.mov", "/tmp/b.unknown"]);
    assert_eq!(videos[0].mime_type, "video/quicktime");
    assert_eq!(videos[0].filename, "a.mov");
    assert_eq!(videos[0].fps, 4);
    assert_eq!(videos[1].mime_type, "video/mp4");

    assert_eq!(
        collect_inbound_images(&[
            json!({"images":[{"data":"abc","mimeType":"image/png"}]}),
            json!({}),
        ]),
        vec![mahayana_host_runtime::extensions::transcript::send_message_shaping::InboundImage {
            data: "abc".into(),
            mime_type: "image/png".into(),
        }]
    );
}

#[test]
fn user_attachment_dimensions_are_projected_and_common_image_headers_are_read() {
    let mut png = vec![0u8; 24];
    png[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
    png[12..16].copy_from_slice(b"IHDR");
    png[16..20].copy_from_slice(&640u32.to_be_bytes());
    png[20..24].copy_from_slice(&480u32.to_be_bytes());
    let dimensions = read_image_dimensions(&png).expect("png dimensions");
    assert_eq!(dimensions.width, 640);
    assert_eq!(dimensions.height, 480);

    let entry = create_user_attachment_entry(
        "t0ua0",
        "/tmp/image.png",
        UserAttachmentOptions {
            width: Some(640),
            height: Some(480),
            ..UserAttachmentOptions::default()
        },
    );
    assert_eq!(entry["width"], 640);
    assert_eq!(entry["height"], 480);
}
