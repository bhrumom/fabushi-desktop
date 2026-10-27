use mahayana_host_runtime::extensions::session::session_turns_controller::SessionTurnsController;
use mahayana_host_runtime::extensions::shared::async_scope::{
    calculate_exponential_retry, normalize_unique_strings, AsyncWorkTracker,
};
use mahayana_host_runtime::extensions::skills::skill_runtime::{
    parse_skill_markdown, SkillRegistry,
};
use mahayana_host_runtime::extensions::snapshot::git_backup_manager::{
    create_export_envelope, verify_export_envelope,
};
use serde_json::json;
use std::time::Duration;

#[test]
fn test_ocrm_snapshot_git_backup() {
    let manifest = json!({
        "repositories": ["main", "worker"],
        "schema_version": 1
    });

    let envelope = create_export_envelope("snap-001", manifest.clone());
    assert_eq!(envelope.snapshot_id, "snap-001");
    assert!(verify_export_envelope(&envelope).is_ok());

    // Tampered checksum
    let mut tampered = envelope.clone();
    tampered.checksum = "bad_checksum".to_string();
    assert!(verify_export_envelope(&tampered).is_err());

    // Tampered manifest
    let mut tampered_manifest = envelope.clone();
    tampered_manifest.manifest = json!({"tampered": true});
    assert!(verify_export_envelope(&tampered_manifest).is_err());
}

#[test]
fn test_ocrm_skills_runtime() {
    let raw_skill = r#"---
name: code-reviewer
description: Automated code review skill
allowed_tools: git, cargo, rustc
user_invocable: true
---

# Code Review Instructions
Analyze diffs for correctness, security, and styling.
"#;

    let skill = parse_skill_markdown(raw_skill).expect("Parse skill markdown");
    assert_eq!(skill.name, "code-reviewer");
    assert_eq!(skill.description, "Automated code review skill");
    assert_eq!(skill.allowed_tools, vec!["git", "cargo", "rustc"]);
    assert!(skill.user_invocable);
    assert!(skill.instructions.contains("Analyze diffs"));

    // Skill registry & prompt injection
    let registry = SkillRegistry::new();
    registry.register_skill(skill);

    let retrieved = registry.get_skill("Code-Reviewer").expect("Skill lookup");
    assert_eq!(retrieved.name, "code-reviewer");

    let prompt = registry.format_prompt_injection();
    assert!(prompt.contains("Available skills:"));
    assert!(prompt.contains("code-reviewer: Automated code review skill"));

    // Invalid skill missing frontmatter
    assert!(parse_skill_markdown("No frontmatter").is_err());
}

#[test]
fn test_ocrm_session_turns_and_shared_async() {
    // 1. Session turns controller
    let controller = SessionTurnsController::new();
    let turn_idx = controller.record_turn_start("sess-1", "Hello agent", Some("claude-3-opus".to_string()));
    assert_eq!(turn_idx, 0);

    let turns_before = controller.get_turns("sess-1");
    assert_eq!(turns_before.len(), 1);
    assert_eq!(turns_before[0].model_override.as_deref(), Some("claude-3-opus"));
    assert!(turns_before[0].agent_output.is_none());

    assert!(controller.record_turn_complete("sess-1", 0, "Hello user!"));
    let turns_after = controller.get_turns("sess-1");
    assert_eq!(turns_after[0].agent_output.as_deref(), Some("Hello user!"));
    assert!(turns_after[0].completed_at_ms.is_some());

    // 2. Async work tracker
    let tracker = AsyncWorkTracker::new();
    assert_eq!(tracker.active_tasks(), 0);
    {
        let _guard = tracker.enter();
        assert_eq!(tracker.active_tasks(), 1);
    }
    assert_eq!(tracker.active_tasks(), 0);

    // 3. Retry calculation & string normalization
    assert_eq!(calculate_exponential_retry(0, 100, 2000), Duration::from_millis(100));
    assert_eq!(calculate_exponential_retry(1, 100, 2000), Duration::from_millis(200));
    assert_eq!(calculate_exponential_retry(10, 100, 2000), Duration::from_millis(2000));

    let norm = normalize_unique_strings(&["  Alpha ", "beta", "ALPHA", ""]);
    assert_eq!(norm, vec!["Alpha", "beta"]);
}
