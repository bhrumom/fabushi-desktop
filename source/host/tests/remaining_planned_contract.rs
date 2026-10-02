use mahayana_host_runtime::extensions::telemetry::box_log_shipper::{
    classify_offset_save_errno, is_box_log_shipping_enabled, saturating_add, to_source_name,
};
use mahayana_host_runtime::extensions::transcript::background_wakes::{
    BackgroundWakes, build_timeline_event_wake_prompt, distinct_channel_addresses,
};
use mahayana_host_runtime::extensions::transcript::client_side_tool_v2_projection::{
    ProjectedClientSideToolV2, ToolProjectionPhase, project_basic_tool_call,
};
use mahayana_host_runtime::extensions::transcript::roster_projection::{
    OUTLINE_STREAM_COALESCE_MS, RosterProjection,
};
use mahayana_host_runtime::extensions::transcript::shared_rooms::{
    RoomEntryKind, can_post_to_group, decode_avatar_data_url, is_room_content_entry,
    normalize_group_message,
};
use mahayana_host_runtime::host_gateway_api::{
    CREATE_AGENT_NONCE_LEDGER_CAP, CreateAgentNonceLedger, HOST_CAPABILITIES,
    is_sand_agent_purpose, sanitize_template_id,
};
use mahayana_host_runtime::sand_host::{
    BOX_READY_REPORT_ATTEMPTS, compute_host_health, should_report_box_ready,
};

#[test]
fn box_log_shipper_core_policy_matches_frozen_contract() {
    assert!(!is_box_log_shipping_enabled(Some("0"), None));
    assert!(is_box_log_shipping_enabled(Some("1"), None));
    assert!(!is_box_log_shipping_enabled(Some("1"), Some(" yes ")));
    assert_eq!(to_source_name("sand-host.log"), "sand-host");
    assert_eq!(to_source_name("sand-host.txt"), "sand-host.txt");
    assert_eq!(classify_offset_save_errno(Some("ENOSPC")), "no_space");
    assert_eq!(
        classify_offset_save_errno(Some("EACCES")),
        "permission_denied"
    );
    assert_eq!(classify_offset_save_errno(None), "unknown");
    assert_eq!(saturating_add(u64::MAX - 1, 3), u64::MAX);
}

#[test]
fn background_wakes_dedupe_addresses_and_fence_parallel_revival() {
    assert_eq!(
        distinct_channel_addresses(["slack:a", "slack:a", "teams:b"]),
        vec!["slack:a", "teams:b"]
    );
    let mut wakes = BackgroundWakes::<u32>::default();
    BackgroundWakes::enqueue(&mut wakes.pending_inbound, "agent", 1);
    BackgroundWakes::enqueue(&mut wakes.pending_inbound, "agent", 2);
    assert!(BackgroundWakes::<u32>::begin_revival(
        &mut wakes.reviving_inbound_agent_ids,
        "agent"
    ));
    assert!(!BackgroundWakes::<u32>::begin_revival(
        &mut wakes.reviving_inbound_agent_ids,
        "agent"
    ));
    assert_eq!(
        BackgroundWakes::take_pending(&mut wakes.pending_inbound, "agent"),
        vec![1, 2]
    );
}

#[test]
fn timeline_event_wake_prompt_matches_frozen_system_event_contract() {
    let prompt = build_timeline_event_wake_prompt(&[serde_json::json!({
        "type": "name-changed",
        "to": "Researcher"
    })]);
    assert_eq!(
        prompt,
        "[event] Something about this conversation just changed.\nThis is a system event recorded in your timeline, not the user typing in this app, and possibly something you did yourself.\n- Renamed to Researcher\nIf it is worth acknowledging to the user, reply with SendMessage; otherwise it is fine to stay silent."
    );
}

#[test]
fn client_projection_requires_identity_and_preserves_phase() {
    assert!(
        project_basic_tool_call(ToolProjectionPhase::Started, "", "shell", "{}", "", None)
            .is_none()
    );
    let projected = project_basic_tool_call(
        ToolProjectionPhase::Partial,
        "tc-1",
        "shell",
        "{\"command\":\"pwd\"}",
        "model-1",
        None,
    )
    .unwrap();
    assert!(matches!(
        projected,
        ProjectedClientSideToolV2::Call {
            is_streaming: true,
            ..
        }
    ));
    let completed = project_basic_tool_call(
        ToolProjectionPhase::Completed,
        "tc-1",
        "shell",
        "{}",
        "",
        Some(Err("denied")),
    )
    .unwrap();
    assert!(matches!(
        completed,
        ProjectedClientSideToolV2::Result { error: Some(_), .. }
    ));
}

#[test]
fn roster_coalescing_keeps_only_latest_pending_outline() {
    assert_eq!(OUTLINE_STREAM_COALESCE_MS, 250);
    let mut roster = RosterProjection::default();
    roster.queue_stream_outline_update("a", "first");
    roster.queue_stream_outline_update("a", "second");
    let update = roster.flush_stream_outline_update().unwrap();
    assert_eq!(update.item, "second");
    assert!(roster.flush_stream_outline_update().is_none());
}

#[test]
fn shared_rooms_avatar_content_and_membership_are_fail_closed() {
    assert_eq!(
        decode_avatar_data_url("data:image/png;base64,YQ=="),
        Some(b"a".to_vec())
    );
    assert!(decode_avatar_data_url("data:text/plain;base64,YQ==").is_none());
    assert!(is_room_content_entry(RoomEntryKind::Message));
    assert!(is_room_content_entry(RoomEntryKind::SendMessage));
    assert!(!is_room_content_entry(RoomEntryKind::Other));
    assert!(normalize_group_message("(pass)", 100).is_none());
    assert_eq!(
        normalize_group_message(" hello ", 100).as_deref(),
        Some("hello")
    );
    assert!(can_post_to_group(&["a".into(), "b".into()], "b"));
    assert!(!can_post_to_group(&["a".into()], "b"));
}

#[test]
fn host_gateway_nonce_and_template_fences_match_reference() {
    assert_eq!(HOST_CAPABILITIES, ["orderedReplicasV1", "sendAcceptanceV1"]);
    assert!(is_sand_agent_purpose("disk-saver"));
    assert!(!is_sand_agent_purpose("other"));
    assert_eq!(sanitize_template_id("abc-123"), Some("abc-123"));
    assert!(sanitize_template_id("ABC").is_none());

    let mut ledger = CreateAgentNonceLedger::default();
    for index in 0..(CREATE_AGENT_NONCE_LEDGER_CAP + 5) {
        ledger.insert(format!("n-{index}"), index);
    }
    assert_eq!(ledger.len(), CREATE_AGENT_NONCE_LEDGER_CAP);
    assert!(ledger.get("n-0").is_none());
    assert_eq!(ledger.get("n-68"), Some(&68));
}

#[test]
fn sand_host_health_does_not_refresh_busy_clock_for_approval_only_wait() {
    assert_eq!(BOX_READY_REPORT_ATTEMPTS, 3);
    let health = compute_host_health(
        ["a".to_string()],
        ["a".to_string()],
        false,
        Some("a".into()),
        200,
        100,
    );
    assert!(health.is_busy);
    assert!(health.busy_only_awaiting_approval);
    assert_eq!(health.last_busy_at_ms, 100);

    let active = compute_host_health(
        ["a".to_string()],
        Vec::<String>::new(),
        false,
        Some("a".into()),
        200,
        100,
    );
    assert_eq!(active.last_busy_at_ms, 200);
    assert!(should_report_box_ready(
        Some("boot-1"),
        Some(10),
        Some("boot-0")
    ));
    assert!(!should_report_box_ready(
        Some("boot-1"),
        Some(10),
        Some("boot-1")
    ));
}
