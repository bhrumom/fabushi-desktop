use mahayana_host_runtime::extensions::inference::provider_session::ProviderMessage;
use mahayana_host_runtime::runner::prompt_collector_glue::{
    PromptCollectorAutomationReminderState, PromptCollectorDynamicUserContext,
    append_mcp_runtime_sections_for_turn,
    append_profile_system_section_for_turn, append_remote_runtime_sections_for_turn,
    apply_dynamic_user_context_for_turn, prepend_unconfirmed_user_messages_for_turn,
    project_provider_messages_for_turn,
    resolve_profile_update_for_turn,
};
use mahayana_host_runtime::runner::sand_agent_profile_prompt::AgentProfileIdentity;
use mahayana_host_runtime::runner::system_prompt_assembly::{
    ComputerPromptState, RemoteBoxPromptState, RunnerPromptRole,
};
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


#[test]
fn dynamic_context_matches_frozen_order_before_reply_and_hidden_markers() {
    let projected = project_provider_messages_for_turn(
        &serde_json::json!({"appendReplyReminder": true}),
        &messages(),
    );
    let mut provider_messages = projected.messages;
    let changed = apply_dynamic_user_context_for_turn(
        &mut provider_messages,
        PromptCollectorDynamicUserContext {
            automation_status_reminder: Some("<automation_status>ready</automation_status>"),
            profile_update: Some("<agent_profile_update>renamed</agent_profile_update>"),
            is_silence_allowed: false,
        },
    );
    assert!(changed);
    let user = provider_messages.last().expect("dynamic user");
    let automation = user.content.find("<automation_status>").expect("automation");
    let profile = user.content.find("<agent_profile_update>").expect("profile");
    let reply = user
        .content
        .find(USER_MESSAGE_REPLY_REMINDER)
        .expect("reply reminder");
    assert!(automation < profile && profile < reply);

    let hidden = project_provider_messages_for_turn(
        &serde_json::json!({
            "hidden": true,
            "automationWake": {"id":"routine-1","containsUntrustedEventText":false}
        }),
        &messages(),
    );
    let mut hidden_messages = hidden.messages;
    apply_dynamic_user_context_for_turn(
        &mut hidden_messages,
        PromptCollectorDynamicUserContext {
            automation_status_reminder: Some("AUTOMATION"),
            profile_update: Some("PROFILE"),
            is_silence_allowed: true,
        },
    );
    assert_eq!(
        hidden_messages.last().expect("hidden user").content,
        format!(
            "{SAND_HIDDEN_PROMPT_MARKER}{SAND_TRUSTED_AUTOMATION_PROMPT_MARKER}PROFILE\n\nAUTOMATION\n\nPlease inspect it"
        )
    );
}

#[test]
fn prompt_collector_owns_profile_mcp_and_remote_runtime_sections() {
    let mut provider_messages = messages();
    append_profile_system_section_for_turn(
        &mut provider_messages,
        "Agent profile:\nTitle: Researcher",
    );
    let identity = AgentProfileIdentity {
        name: "Researcher Two".into(),
        description: "Investigates".into(),
    };
    let announced = AgentProfileIdentity {
        name: "Researcher".into(),
        description: "Investigates".into(),
    };
    assert!(resolve_profile_update_for_turn(&identity, &announced).is_some());

    append_mcp_runtime_sections_for_turn(
        &mut provider_messages,
        &[serde_json::json!({
            "name": "Acme",
            "status": "connected",
            "customInstructions": "Use exact source rows."
        })],
        true,
        false,
    );
    append_remote_runtime_sections_for_turn(
        &mut provider_messages,
        &RemoteBoxPromptState {
            role: RunnerPromptRole::Main,
            available: true,
            runtime_state: "ready".into(),
            desktop_capable: true,
            desktop_ready: true,
        },
        &ComputerPromptState {
            role: RunnerPromptRole::Main,
            box_available: true,
            desktop_capable: true,
            desktop_ready: true,
            control_lease_active: false,
            human_takeover_pending: false,
            browser_use_offered: false,
            window_index: Some(1),
        },
    );

    let system = &provider_messages
        .iter()
        .find(|message| message.role == "system")
        .expect("system")
        .content;
    assert!(system.contains("Agent profile:\nTitle: Researcher"));
    assert!(system.contains("## Connector custom instructions"));
    assert!(system.contains("<mcp_status>"));
    assert!(system.contains("## Your box"));
    assert!(system.contains("## The box desktop"));

    let mut subagent_messages = messages();
    append_mcp_runtime_sections_for_turn(
        &mut subagent_messages,
        &[serde_json::json!({
            "name": "Acme",
            "status": "connected",
            "customInstructions": "should stay hidden"
        })],
        true,
        true,
    );
    assert!(!subagent_messages[0].content.contains("Connector custom instructions"));
    assert!(!subagent_messages[0].content.contains("<mcp_status>"));
}


#[test]
fn shipping_prepend_projection_uses_durable_confirmed_transcript_watermark() {
    let args = serde_json::json!({
        "messageId": "m3",
        "recentUserMessages": [
            {"id":"m1","text":"confirmed"},
            {"id":"m2","text":"queued","richText":"{\"doc\":1}"},
            {"id":"m3","text":"current"}
        ]
    });
    let transcript = vec![
        serde_json::json!({
            "id":"m1","kind":"message","role":"user","content":"confirmed","confirmed":true
        }),
        serde_json::json!({
            "id":"m2","kind":"message","role":"user","content":"queued"
        }),
        serde_json::json!({
            "id":"m3","kind":"message","role":"user","content":"current"
        }),
    ];
    let mut provider_messages = vec![
        ProviderMessage { role: "system".into(), content: "system".into() },
        ProviderMessage { role: "user".into(), content: "current".into() },
    ];

    assert_eq!(
        prepend_unconfirmed_user_messages_for_turn(&args, &mut provider_messages, &transcript),
        1
    );
    assert_eq!(provider_messages.len(), 3);
    assert_eq!(provider_messages[1].role, "user");
    assert_eq!(provider_messages[1].content, "[m2]\nqueued");
    assert_eq!(provider_messages[2].content, "current");
}

#[test]
fn automation_status_reminder_state_matches_frozen_compaction_and_clear_semantics() {
    let mut state = PromptCollectorAutomationReminderState::default();
    let running = "<automation_status>running</automation_status>";

    assert_eq!(
        state.reminder_for_turn(Some(running), 4).as_deref(),
        Some(running)
    );
    state.note_reminder(Some(running), 4);

    assert_eq!(state.reminder_for_turn(Some(running), 4), None);
    assert_eq!(
        state.reminder_for_turn(Some(running), 5).as_deref(),
        Some(running)
    );
    state.note_reminder(Some(running), 5);

    let cleared = state
        .reminder_for_turn(None, 5)
        .expect("cleared automation snapshot");
    assert!(cleared.contains("<automation_status>"));
    assert!(cleared.contains("No current routines."));
    state.note_reminder(Some(&cleared), 5);

    assert_eq!(state.reminder_for_turn(None, 5), None);
    assert_eq!(
        state.reminder_for_turn(None, 6).as_deref(),
        Some(cleared.as_str())
    );

    state.reset();
    assert_eq!(state.reminder_for_turn(None, 6), None);
}
