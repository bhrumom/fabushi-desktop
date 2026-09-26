use std::collections::HashMap;

use mahayana_host_runtime::host_request_context::HostRequestContext;
use mahayana_host_runtime::runner::system_prompt::{
    DEFAULT_SAND_SYSTEM_PROMPT, SAND_CLOUD_AGENTS_DISABLED_PROMPT_SECTION,
    SAND_MCP_MULTI_ACCOUNT_PROMPT_SECTION, SAND_SYSTEM_PROMPT_CLOUD_AGENTS_DISABLED,
    USER_MESSAGE_REPLY_REMINDER, ReplyContext, append_user_reply_reminder_with_disabled,
    build_attached_files_note, build_reply_context_note, build_sand_base_system_prompt,
    build_sand_subagent_system_prompt, build_user_message_address_note,
    format_attached_file_size, is_media_review_subagent_type,
};
use mahayana_host_runtime::runner::system_prompt_assembly::render_request_context_system_prompt;
use serde_json::json;

#[test]
fn frozen_base_prompt_variants_preserve_grok_send_message_and_cloud_agent_contracts() {
    assert!(DEFAULT_SAND_SYSTEM_PROMPT.starts_with(
        "You are Grok Bot, a warm, concise desktop assistant.\n\n## How a turn works"
    ));
    assert!(DEFAULT_SAND_SYSTEM_PROMPT.contains(
        "ALWAYS hand it to a Cursor cloud agent with the CloudAgent tool"
    ));
    assert!(!SAND_SYSTEM_PROMPT_CLOUD_AGENTS_DISABLED.contains(
        "ALWAYS hand it to a Cursor cloud agent with the CloudAgent tool"
    ));
    assert!(SAND_SYSTEM_PROMPT_CLOUD_AGENTS_DISABLED.contains(
        "Cursor cloud agents are disabled by your team's admin"
    ));
    assert_eq!(
        build_sand_base_system_prompt(true),
        DEFAULT_SAND_SYSTEM_PROMPT
    );
    assert_eq!(
        build_sand_base_system_prompt(false),
        SAND_SYSTEM_PROMPT_CLOUD_AGENTS_DISABLED
    );
    assert!(SAND_CLOUD_AGENTS_DISABLED_PROMPT_SECTION.contains("CloudAgent tool is not available"));
    assert!(SAND_MCP_MULTI_ACCOUNT_PROMPT_SECTION.contains("account_label"));
}

#[test]
fn frozen_reply_and_attachment_helpers_match_grok_shapes() {
    assert_eq!(format_attached_file_size(-1.0), "");
    assert_eq!(format_attached_file_size(1_023.0), "1023 B");
    assert_eq!(format_attached_file_size(1_024.0), "1.0 KB");
    assert_eq!(format_attached_file_size(10_240.0), "10 KB");
    assert_eq!(format_attached_file_size(1_048_576.0), "1.0 MB");
    assert_eq!(format_attached_file_size(1_073_741_824.0), "1.0 GB");

    let files = vec![" /Users/me/report.pdf ".to_string(), "".to_string()];
    let box_paths = HashMap::from([(
        "/Users/me/report.pdf".to_string(),
        "/workspace/uploads/report.pdf".to_string(),
    )]);
    let sizes = HashMap::from([("/Users/me/report.pdf".to_string(), 2_048.0)]);
    let note = build_attached_files_note(&files, &box_paths, &sizes);
    assert!(note.starts_with("The user attached a file."));
    assert!(note.contains("/Users/me/report.pdf (2.0 KB)"));
    assert!(note.contains("also copied into your box at /workspace/uploads/report.pdf"));

    assert_eq!(
        build_reply_context_note(Some(&ReplyContext {
            target_id: " t3u ".into(),
            quote: " hello ".into(),
        })),
        "[In reply to t3u: \"hello\"]"
    );
    assert_eq!(build_user_message_address_note(Some(" t3u ")), "[t3u]");
}

#[test]
fn frozen_reply_reminder_and_subagent_helpers_are_source_closed() {
    assert_eq!(
        append_user_reply_reminder_with_disabled("hello", true),
        "hello"
    );
    assert_eq!(
        append_user_reply_reminder_with_disabled("", false),
        USER_MESSAGE_REPLY_REMINDER
    );
    assert!(
        append_user_reply_reminder_with_disabled("hello", false)
            .ends_with(USER_MESSAGE_REPLY_REMINDER)
    );
    assert!(is_media_review_subagent_type(Some("watch-video")));
    assert!(is_media_review_subagent_type(Some("VideoReview")));
    assert!(!is_media_review_subagent_type(Some("generalPurpose")));

    let prompt = build_sand_subagent_system_prompt(Some("browserUse"), true);
    assert!(prompt.starts_with("You are Grok Bot running as the browserUse subagent."));
    assert!(prompt.contains("Operate in readonly mode: do not modify anything."));
    assert!(prompt.contains("## Staying safe while you work"));
}

#[test]
fn shipping_system_prompt_assembly_uses_the_frozen_supported_variant() {
    let context = HostRequestContext {
        os_version: "test".into(),
        shell: None,
        time_zone: Some("America/Los_Angeles".into()),
        transcripts_folder: "/tmp/transcripts".into(),
        user_full_name: Some("Ada Lovelace".into()),
    };
    let prompt = render_request_context_system_prompt(
        &context,
        Some(&[json!({
            "name": "security",
            "content": "Never disclose credentials.",
            "isRequired": true
        })]),
    );
    assert!(prompt.starts_with(SAND_SYSTEM_PROMPT_CLOUD_AGENTS_DISABLED));
    assert!(prompt.contains(SAND_CLOUD_AGENTS_DISABLED_PROMPT_SECTION));
    assert!(prompt.contains("the user lives in America/Los_Angeles"));
    assert!(prompt.contains("Your user is Ada Lovelace"));
    assert!(prompt.contains("### security (required)\nNever disclose credentials."));
}
