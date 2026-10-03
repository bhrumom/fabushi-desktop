use std::collections::BTreeMap;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::settings::settings_service::{
    MCP_CUSTOM_INSTRUCTIONS_MAX_LENGTH, SAND_AUTO_REVIEW_INSTRUCTION_MAX_CHARS,
    SAND_AUTO_REVIEW_INSTRUCTION_MAX_ENTRIES, SandAutoReviewInstructions,
    SandLocalToolPermission, SettingsService, is_valid_iana_time_zone,
};

fn temp_path(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-settings-{label}-{}-{suffix}.json",
        std::process::id()
    ))
}

#[test]
fn timezone_settings_validate_persist_preserve_unknown_fields_and_notify_effective_changes() {
    let path = temp_path("timezone");
    fs::write(
        &path,
        r#"{"version":1,"unrelated":{"keep":true},"userTimeZone":"UTC"}"#,
    )
    .expect("seed settings");
    let service = SettingsService::new(&path);
    assert!(is_valid_iana_time_zone("America/Los_Angeles"));
    assert!(!is_valid_iana_time_zone("Not/AZone"));
    assert_eq!(service.get_user_time_zone().as_deref(), Some("UTC"));

    let observed = Arc::new(Mutex::new(Vec::<Option<String>>::new()));
    let observed_for_listener = Arc::clone(&observed);
    let _subscription = service.subscribe_to_user_time_zone(Arc::new(move |zone| {
        observed_for_listener.lock().expect("observed lock").push(zone);
    }));

    assert!(!service
        .set_user_time_zone_override(Some("Not/AZone"))
        .expect("invalid zone ignored"));
    assert!(service
        .set_user_time_zone_override(Some(" America/Los_Angeles "))
        .expect("set override"));
    assert_eq!(
        service.get_user_time_zone().as_deref(),
        Some("America/Los_Angeles")
    );

    assert!(service
        .set_user_time_zone(Some("Europe/Berlin"))
        .expect("set detected zone"));
    assert_eq!(
        service.get_user_time_zone().as_deref(),
        Some("America/Los_Angeles")
    );

    assert!(service
        .set_user_time_zone_override(Some(""))
        .expect("clear override"));
    assert_eq!(service.get_user_time_zone().as_deref(), Some("Europe/Berlin"));

    let persisted: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("settings text"))
            .expect("settings json");
    assert_eq!(persisted["unrelated"]["keep"], true);
    assert_eq!(
        observed.lock().expect("observed lock").as_slice(),
        &[
            Some("America/Los_Angeles".to_string()),
            Some("Europe/Berlin".to_string())
        ]
    );
    let _ = fs::remove_file(path);
}

#[test]
fn auto_review_instructions_default_normalize_persist_and_notify() {
    let path = temp_path("auto-review");
    let service = SettingsService::new(&path);
    assert_eq!(
        service.get_auto_review_instructions(),
        SandAutoReviewInstructions::default()
    );

    let changed = Arc::new(Mutex::new(Vec::<Vec<String>>::new()));
    let changed_for_listener = Arc::clone(&changed);
    let _subscription = service.subscribe_to_changes(Arc::new(move |keys| {
        changed_for_listener.lock().expect("changed lock").push(keys);
    }));

    let oversized = "x".repeat(SAND_AUTO_REVIEW_INSTRUCTION_MAX_CHARS + 50);
    let mut allow = vec!["  allow shell  ".to_string(), "allow shell".to_string(), oversized];
    for index in 0..(SAND_AUTO_REVIEW_INSTRUCTION_MAX_ENTRIES + 10) {
        allow.push(format!("allow-{index}"));
    }
    let instructions = SandAutoReviewInstructions {
        is_enabled: false,
        allow_instructions: allow,
        block_instructions: vec!["  block delete  ".into(), "".into()],
    };
    assert!(service
        .set_auto_review_instructions(&instructions)
        .expect("persist auto-review instructions"));

    let normalized = service.get_auto_review_instructions();
    assert!(!normalized.is_enabled);
    assert_eq!(normalized.allow_instructions[0], "allow shell");
    assert_eq!(normalized.allow_instructions.len(), SAND_AUTO_REVIEW_INSTRUCTION_MAX_ENTRIES);
    assert_eq!(
        normalized.allow_instructions[1].chars().count(),
        SAND_AUTO_REVIEW_INSTRUCTION_MAX_CHARS
    );
    assert_eq!(normalized.block_instructions, vec!["block delete".to_string()]);
    assert_eq!(
        changed.lock().expect("changed lock").as_slice(),
        &[vec!["autoReviewInstructions".to_string()]]
    );
    assert!(!service
        .set_auto_review_instructions(&normalized)
        .expect("identical normalized settings are stable"));

    let host = service.get_host_settings();
    assert_eq!(host["autoReviewInstructions"]["isEnabled"], false);
    let _ = fs::remove_file(path);
}


#[test]
fn frozen_mcp_settings_normalize_migrate_delete_and_account_scope() {
    let path = temp_path("mcp");
    let service = SettingsService::new(&path);
    let oversized = "x".repeat(MCP_CUSTOM_INSTRUCTIONS_MAX_LENGTH + 32);

    service
        .set_mcp_custom_instructions(BTreeMap::from([
            ("hex".into(), "".into()),
            ("plain".into(), "".into()),
            ("alpha".into(), oversized),
        ]))
        .expect("legacy instructions");
    let legacy = service.get_mcp_custom_instructions();
    assert_eq!(legacy.get("hex").map(String::as_str), Some(""));
    assert!(!legacy.contains_key("plain"));
    assert_eq!(
        legacy.get("alpha").map(|value| value.chars().count()),
        Some(MCP_CUSTOM_INSTRUCTIONS_MAX_LENGTH)
    );

    service
        .set_mcp_custom_instructions(BTreeMap::from([(
            "Legacy".into(),
            "instruction".into(),
        )]))
        .expect("legacy seed");
    assert!(service
        .migrate_mcp_custom_instruction_to_server_id("11", "Legacy")
        .expect("migrate"));
    assert_eq!(
        service
            .get_raw_mcp_custom_instruction_by_server_id("11")
            .as_deref(),
        Some("instruction")
    );
    assert!(service
        .delete_mcp_custom_instruction_by_server_id("11", "Legacy", true)
        .expect("delete"));
    assert!(service.get_raw_mcp_custom_instruction("Legacy").is_none());

    service
        .set_mcp_disabled_tools_by_server_id(BTreeMap::from([
            (
                "12".into(),
                vec!["A".into(), "A".into(), "".into(), "B".into()],
            ),
            ("bad".into(), vec!["X".into()]),
        ]))
        .expect("disabled tools");
    assert_eq!(
        service.get_mcp_disabled_tools_by_server_id(),
        BTreeMap::from([("12".into(), vec!["A".into(), "B".into()])])
    );

    service.scope_to_account("account-a").expect("initial scope");
    service
        .set_mcp_custom_instructions(BTreeMap::from([("x".into(), "y".into())]))
        .expect("scoped mcp");
    service
        .set_local_tool_permission(SandLocalToolPermission::Always)
        .expect("permission");
    assert!(service.scope_to_account("account-b").expect("scope change"));
    assert!(service.get_mcp_custom_instructions().is_empty());
    assert_eq!(
        service.get_local_tool_permission(),
        SandLocalToolPermission::Ask
    );
    let _ = fs::remove_file(path);
}

#[test]
fn permission_ceiling_feature_override_and_inference_usage_match_frozen_contract() {
    let path = temp_path("runtime");
    let service = SettingsService::new(&path);
    service
        .set_local_tool_permission(SandLocalToolPermission::Always)
        .expect("choice");
    service
        .set_local_tool_permission_ceiling(Some(SandLocalToolPermission::Ask))
        .expect("ceiling");
    assert_eq!(
        service.get_local_tool_permission(),
        SandLocalToolPermission::Ask
    );

    let observed = Arc::new(Mutex::new(Vec::<BTreeMap<String, bool>>::new()));
    let sink = Arc::clone(&observed);
    let _subscription =
        service.subscribe_to_feature_flag_overrides(Arc::new(move |overrides| {
            sink.lock().expect("feature overrides").push(overrides);
        }));
    service
        .set_host_settings(&serde_json::json!({
            "featureFlagOverrides": {
                "sand_multitask": false,
                "sand_spotlight": true
            }
        }))
        .expect("host settings");
    assert_eq!(
        observed.lock().expect("feature overrides").as_slice(),
        &[BTreeMap::from([
            ("sand_multitask".into(), false),
            ("sand_spotlight".into(), true),
        ])]
    );

    assert_eq!(service.get_inference_provider(), "cursor");
    assert!(service
        .set_inference_provider("codex")
        .expect("set provider"));
    assert!(!service
        .set_inference_provider("invalid")
        .expect("reject provider"));
    service
        .record_inference_usage(
            "codex",
            Some(10.4),
            Some(4.6),
            Some(-1.0),
            Some(f64::NAN),
        )
        .expect("record usage");
    let usage = service.get_inference_router_usage();
    assert_eq!(usage["providers"]["codex"]["requests"], 1);
    assert_eq!(usage["providers"]["codex"]["inputTokens"], 10);
    assert_eq!(usage["providers"]["codex"]["outputTokens"], 5);
    assert_eq!(usage["providers"]["codex"]["cacheReadTokens"], 0);
    assert_eq!(usage["providers"]["codex"]["cacheWriteTokens"], 0);
    assert!(usage["providers"]["codex"]["lastUsedAt"].is_string());

    let host = service.get_host_settings();
    assert_eq!(host["notifications"]["isEnabled"], false);
    assert_eq!(host["notifications"]["minIntervalMs"], 5000);
    let _ = fs::remove_file(path);
}


#[test]
fn host_settings_switch_scope_before_applying_same_update_and_normalizes_invalid_permission() {
    let path = temp_path("host-update-order");
    let service = SettingsService::new(&path);
    service.scope_to_account("account-a").expect("first scope");
    service
        .set_mcp_custom_instructions(BTreeMap::from([("old".into(), "old-value".into())]))
        .expect("old instructions");
    service
        .set_has_seen_onboarding(true)
        .expect("onboarding owner");

    let host = service
        .set_host_settings(&serde_json::json!({
            "mcpCustomInstructionsAccountScope": "account-b",
            "mcpCustomInstructions": { "new": "new-value" },
            "localToolPermission": { "not": "a permission" }
        }))
        .expect("scoped update");

    assert_eq!(
        host["mcpCustomInstructions"],
        serde_json::json!({ "new": "new-value" })
    );
    assert_eq!(host["localToolPermission"], "ask");
    assert!(host.get("hasSeenOnboarding").is_none());
    let _ = fs::remove_file(path);
}


#[test]
fn frozen_notification_and_default_auto_review_persistence_are_normalized() {
    let path = temp_path("normalization");
    fs::write(
        &path,
        r#"{"version":1,"notifications":{"isEnabled":true,"extra":"drop"},"autoReviewInstructions":{"isEnabled":false,"allowInstructions":["x"],"blockInstructions":[]}}"#,
    )
    .expect("seed settings");
    let service = SettingsService::new(&path);

    let notifications = service.get_notification_config();
    assert_eq!(notifications["isEnabled"], false);
    let persisted: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("settings text"))
            .expect("settings json");
    assert_eq!(
        persisted["notifications"],
        serde_json::json!({"isEnabled": false})
    );

    assert!(service
        .set_auto_review_instructions(&SandAutoReviewInstructions::default())
        .expect("clear default auto review override"));
    let persisted: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("settings text"))
            .expect("settings json");
    assert!(persisted.get("autoReviewInstructions").is_none());
    let _ = fs::remove_file(path);
}
