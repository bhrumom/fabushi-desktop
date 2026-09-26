use mahayana_host_runtime::extensions::transcript::agent_run_error::{
    AgentRunErrorDescription, BackendAdditionalInfo, BackendDetail, checkout_deep_control_url,
    describe_agent_run_error, format_sand_usage_reset_in, map_error_detail_buttons,
    provider_failure_tray, with_relative_sand_included_limit_reset,
};
use serde_json::json;

const NOW: i64 = 1_700_000_000_000;

#[test]
fn usage_reset_formatting_matches_frozen_thresholds() {
    let now = chrono::DateTime::from_timestamp_millis(NOW).expect("now");
    let iso = |delta: i64| (now + chrono::Duration::milliseconds(delta)).to_rfc3339();
    assert_eq!(format_sand_usage_reset_in(&iso(-1), NOW).as_deref(), Some("less than a minute"));
    assert_eq!(format_sand_usage_reset_in(&iso(61_000), NOW).as_deref(), Some("2 minutes"));
    assert_eq!(format_sand_usage_reset_in(&iso(3_600_001), NOW).as_deref(), Some("2 hours"));
    assert_eq!(format_sand_usage_reset_in(&iso(86_400_001), NOW).as_deref(), Some("2 days"));
    assert!(format_sand_usage_reset_in("not-a-date", NOW).is_none());
}

#[test]
fn included_limit_reset_rewrites_absolute_and_relative_clauses() {
    let reset = chrono::DateTime::from_timestamp_millis(NOW + 7_200_000)
        .expect("reset")
        .to_rfc3339();
    assert_eq!(
        with_relative_sand_included_limit_reset(
            "Quota reached. It resets at 2026-09-24T12:00:00Z. Try later.",
            Some(&reset),
            NOW,
        ),
        "Quota reached. It resets in 2 hours. Try later."
    );
    assert_eq!(
        with_relative_sand_included_limit_reset(
            "Quota reached. It resets in tomorrow. Try later.",
            Some(&reset),
            NOW,
        ),
        "Quota reached. It resets in 2 hours. Try later."
    );
}

#[test]
fn action_mapping_enforces_protocols_dedupes_switch_and_caps_at_three() {
    let buttons = vec![
        json!({"label":"Docs","action":{"case":"url","value":{"url":"https://example.com/help"}}}),
        json!({"label":"Bad","action":{"case":"url","value":{"url":"file:///tmp/x"}}}),
        json!({"label":"","action":{"case":"switchModel","value":{}}}),
        json!({"label":"ignored duplicate","action":{"case":"switchModel","value":{}}}),
        json!({"label":"Raise","action":{"case":"dashboardAction","value":{"action":"requestLimitIncrease","args":{"team":"a"},"successMessage":"sent"}}}),
        json!({"label":"Pricing","action":{"case":"upgradeChoice","value":{}}}),
    ];
    assert_eq!(
        map_error_detail_buttons(&buttons),
        vec![
            json!({"kind":"open-url","label":"Docs","url":"https://example.com/help"}),
            json!({"kind":"switch-model"}),
            json!({"kind":"dashboard-action","label":"Raise","action":"requestLimitIncrease","args":{"team":"a"},"successMessage":"sent"}),
        ]
    );
}

#[test]
fn checkout_url_normalizes_tier_and_trial_flag() {
    assert_eq!(
        checkout_deep_control_url(&json!({"membershipToUpgradeTo":"ultra","allowTrial":true})),
        "https://cursor.com/api/auth/checkoutDeepControl?tier=ultra&allowTrial=true"
    );
    assert_eq!(
        checkout_deep_control_url(&json!({"membershipToUpgradeTo":"enterprise","allowTrial":false})),
        "https://cursor.com/api/auth/checkoutDeepControl?tier=pro&allowTrial=false"
    );
}

#[test]
fn structured_description_uses_backend_title_limit_reset_and_actions() {
    let reset = chrono::DateTime::from_timestamp_millis(NOW + 3_600_000)
        .expect("reset")
        .to_rfc3339();
    let detail = BackendDetail {
        title: Some("Limit reached".into()),
        detail: Some("Included usage exhausted. It resets at tomorrow.".into()),
        buttons: vec![json!({"label":"","action":{"case":"switchModel","value":{}}})],
        additional_info: Some(BackendAdditionalInfo {
            rate_limit_reason: Some("sand_included_limit".into()),
            next_reset_at: Some(reset),
        }),
    };
    assert_eq!(
        describe_agent_run_error("fallback", Some(&detail), NOW),
        AgentRunErrorDescription {
            title: Some("Limit reached".into()),
            detail: "Included usage exhausted. It resets in 1 hour.".into(),
            actions: vec![json!({"kind":"switch-model"})],
        }
    );
}

#[test]
fn provider_failure_projection_is_ready_for_shipping_trays() {
    let tray = provider_failure_tray("agent-a", "provider disconnected", NOW);
    assert_eq!(tray.agent_id.as_deref(), Some("agent-a"));
    assert_eq!(tray.title, "Agent run failed");
    assert_eq!(tray.detail, "provider disconnected");
    assert_eq!(tray.raw_detail.as_deref(), Some("provider disconnected"));
    assert_eq!(tray.error_kind.as_deref(), Some("agent_run_error"));
    assert_eq!(tray.dedupe_key.as_deref(), Some("agent-run-error:agent-a"));
}
