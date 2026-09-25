use mahayana_host_runtime::extensions::inference::provider_session::ProviderMessage;
use mahayana_host_runtime::runner::prompt_collector_glue::project_provider_messages_for_turn;
use mahayana_host_runtime::runner::sand_prompt_markers::{
    SAND_HIDDEN_PROMPT_MARKER, SAND_TRUSTED_AUTOMATION_PROMPT_MARKER,
};
use mahayana_host_runtime::runner::system_prompt::USER_MESSAGE_REPLY_REMINDER;

fn messages() -> Vec<ProviderMessage> {
    vec![
        ProviderMessage {
            role: "system".into(),
            content: "system".into(),
        },
        ProviderMessage {
            role: "user".into(),
            content: "Please inspect it".into(),
        },
    ]
}

#[test]
fn provider_projection_shapes_address_reply_attachments_and_reply_reminder() {
    let args = serde_json::json!({
        "messageId": " t3u ",
        "replyContext": {"targetId":"t2a","quote":" prior answer "},
        "attachmentPaths": ["/tmp/report.csv"],
        "attachedFileSizes": {"/tmp/report.csv": 2048},
        "boxPathByHostPath": {"/tmp/report.csv": "/workspace/uploads/report.csv"},
        "appendReplyReminder": true
    });

    let projected = project_provider_messages_for_turn(&args, &messages());
    let user = projected.messages.last().expect("projected user");
    assert_eq!(user.role, "user");
    assert!(user.content.starts_with(
        "[t3u]\n[In reply to t2a: \"prior answer\"]\nPlease inspect it"
    ));
    assert!(user.content.contains("The user attached a file."));
    assert!(user.content.contains("/tmp/report.csv (2.0 KB)"));
    assert!(user.content.contains("/workspace/uploads/report.csv"));
    assert!(user.content.ends_with(USER_MESSAGE_REPLY_REMINDER));
    assert_eq!(
        projected.projected_user_text.as_deref(),
        Some(user.content.as_str())
    );
    assert!(!projected.prepended_unanswered_questions);
}

#[test]
fn hidden_automation_projection_matches_frozen_trust_marker_rules() {
    let trusted = project_provider_messages_for_turn(
        &serde_json::json!({
            "hidden": true,
            "automationWake": {"id":"routine-1","containsUntrustedEventText":false},
            "appendReplyReminder": true
        }),
        &messages(),
    );
    let trusted_text = &trusted.messages.last().expect("trusted hidden user").content;
    assert!(trusted_text.starts_with(&format!(
        "{SAND_HIDDEN_PROMPT_MARKER}{SAND_TRUSTED_AUTOMATION_PROMPT_MARKER}"
    )));
    assert!(!trusted_text.contains(USER_MESSAGE_REPLY_REMINDER));

    let untrusted = project_provider_messages_for_turn(
        &serde_json::json!({
            "hidden": true,
            "automationWake": {"id":"routine-2","containsUntrustedEventText":true}
        }),
        &messages(),
    );
    let untrusted_text = &untrusted.messages.last().expect("untrusted hidden user").content;
    assert!(untrusted_text.starts_with(SAND_HIDDEN_PROMPT_MARKER));
    assert!(!untrusted_text.starts_with(&format!(
        "{SAND_HIDDEN_PROMPT_MARKER}{SAND_TRUSTED_AUTOMATION_PROMPT_MARKER}"
    )));
}

#[test]
fn unanswered_questions_are_prepended_without_mutating_durable_input() {
    let original = messages();
    let projected = project_provider_messages_for_turn(
        &serde_json::json!({
            "skippedQuestionPrompts": ["Which account?"],
            "dismissedQuestionPrompts": ["Share location?"]
        }),
        &original,
    );

    assert_eq!(original.len(), 2);
    assert_eq!(original[1].content, "Please inspect it");
    assert_eq!(projected.messages.len(), 3);
    assert_eq!(projected.messages[1].role, "user");
    assert_eq!(
        projected.messages[1].content,
        "Unanswered questions:\n- Which account?\n- Share location?"
    );
    assert_eq!(projected.messages[2].content, "Please inspect it");
    assert!(projected.prepended_unanswered_questions);
}

#[test]
fn projection_without_user_message_is_a_noop() {
    let original = vec![ProviderMessage {
        role: "system".into(),
        content: "system".into(),
    }];
    let projected =
        project_provider_messages_for_turn(&serde_json::json!({"messageId":"t3u"}), &original);
    assert_eq!(projected.messages, original);
    assert_eq!(projected.projected_user_text, None);
}
