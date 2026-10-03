use std::fs;
use std::path::PathBuf;

fn shipping_source(relative: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
    fs::read_to_string(path).unwrap_or_else(|error| panic!("read shipping source {relative}: {error}"))
}

#[test]
fn shipping_provider_turn_consumes_dynamic_sections_in_frozen_order() {
    let source = shipping_source("app/src/main.rs");
    let collector = shipping_source("src/runner/prompt_collector_glue.rs");
    let turn_start = source
        .find("fn start_routed_provider_task(")
        .expect("shipping routed provider turn");
    let provider_start = source[turn_start..]
        .find("let mut provider_messages =")
        .map(|offset| turn_start + offset)
        .expect("shipping provider projection");
    let turn_prefix = &source[turn_start..provider_start];
    let tail = &source[provider_start..];

    let workflows = tail.find("append_workflows_system_prompt(").expect("shipping workflows section");
    let channels = tail.find("append_channels_system_prompt(").expect("shipping channels section");
    let directory = tail.find("append_agent_directory_system_prompt(").expect("shipping agent directory section");
    let mcp = tail.find("append_mcp_runtime_sections_for_turn(").expect("shipping prompt-collector MCP delegation");

    assert!(workflows < channels && channels < directory && directory < mcp);
    assert!(tail[..channels].contains("list_agent_channels(&agent_id)"));
    assert!(tail[..directory].contains("summary.member_ids") && tail[..directory].contains("summary.is_group"));
    assert!(tail[..mcp].contains("mcp_service.list_installed()"));
    assert!(tail[..mcp].contains("mcp_discovery_unavailable"));
    assert!(tail[mcp..].contains("generated_parent_agent_id.is_some()"));
    assert_eq!(source.matches("append_channels_system_prompt(").count(), 1);
    assert_eq!(source.matches("append_agent_directory_system_prompt(").count(), 1);
    assert_eq!(source.matches("append_mcp_runtime_sections_for_turn(").count(), 1);

    assert!(collector.contains("pub fn append_mcp_runtime_sections_for_turn("));
    assert!(collector.contains("append_mcp_system_prompt_sections("));
    assert!(collector.contains("!is_subagent_runner"));
    assert_eq!(source.matches("append_mcp_system_prompt_sections(").count(), 0);
    assert!(tail.contains("prompt_compaction_epoch(&session_workers, &agent_id)"));
    assert!(tail.contains("automation_status_reminder_for_turn("));
    assert!(tail.contains("note_automation_status_reminder("));
    assert!(collector.contains("pub struct PromptCollectorAutomationReminderState"));
    assert!(tail.contains("read_agent_transcript_entries(&agent_id)"));
    assert_eq!(
        source.matches("prepend_unconfirmed_user_messages_for_turn(").count(),
        1
    );
    assert!(collector.contains("pub fn prepend_unconfirmed_user_messages_for_turn("));
    assert!(collector.contains("collect_prepend_user_messages("));
    assert!(turn_prefix.contains("production_services()"));
    assert!(turn_prefix.contains("selected_media_host_paths_for_turn(&args)"));
    assert!(turn_prefix.contains("attachment_paths.push(media_path)"));
    assert!(turn_prefix.contains("services.attachments.stage_into_box(&agent_id, &attachment_paths)"));
    assert_eq!(
        source.matches("apply_staged_attachment_paths_for_turn(").count(),
        1
    );
    assert!(collector.contains("pub fn apply_staged_attachment_paths_for_turn("));
    assert!(collector.contains("pub fn collect_turn_action_for_projection("));
    assert!(collector.contains("read_subagent_video_bytes("));
    assert!(collector.contains("bytes_look_like_video_container("));
}
