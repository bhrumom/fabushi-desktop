use mahayana_host_runtime::extensions::session::session_turns_controller::SessionTurnsController;
use mahayana_host_runtime::extensions::shared::async_scope::{
    calculate_exponential_retry, normalize_unique_strings, AsyncWorkTracker,
};
use mahayana_host_runtime::extensions::skills::skill_runtime::{
    parse_skill_markdown, SkillRegistry,
};
use mahayana_host_runtime::extensions::snapshot::git_backup_manager::{
    assert_database_artifact, capture_database_artifact, create_export_envelope,
    assert_fresh_restore_target, assert_no_sqlite_sidecars, parse_postgres_export_manifest,
    new_restore_namespace,
    verify_export_envelope, verify_postgres_anchor_identity, verify_postgres_export_artifacts,
    write_postgres_export_manifest,
    SnapshotGenerationFence,
};
use serde_json::json;
use std::fs;
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
fn test_ocrm_snapshot_artifact_admission_and_generation_fence() {
    let root =
        std::env::temp_dir().join(format!("fabushi-snapshot-artifact-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create temp root");
    let artifact = root.join("database.sqlite");
    fs::write(&artifact, b"snapshot-image-v1").expect("write artifact");

    let admitted = capture_database_artifact(&artifact).expect("admit regular artifact");
    assert_eq!(admitted.size_bytes, 17);
    assert_database_artifact(&admitted).expect("unchanged artifact remains valid");

    fs::write(&artifact, b"tampered-image").expect("tamper artifact");
    assert!(assert_database_artifact(&admitted).is_err());

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let symlink_path = root.join("database-link.sqlite");
        symlink(&artifact, &symlink_path).expect("create symlink fixture");
        assert!(capture_database_artifact(&symlink_path).is_err());

        let hardlink_path = root.join("database-hardlink.sqlite");
        fs::hard_link(&artifact, &hardlink_path).expect("create hardlink fixture");
        assert!(capture_database_artifact(&artifact).is_err());
    }

    let fence = SnapshotGenerationFence::new(4);
    let expected = fence.current();
    fence.assert_current(expected).expect("current generation");
    assert_eq!(fence.advance(), 5);
    assert!(fence.assert_current(expected).is_err());

    let restore_target = root.join("restore.sqlite");
    assert_fresh_restore_target(&restore_target).expect("fresh restore target");
    fs::write(root.join("restore.sqlite-wal"), b"sidecar").expect("write sidecar fixture");
    assert!(assert_fresh_restore_target(&restore_target).is_err());
    assert!(assert_no_sqlite_sidecars(&restore_target, "sidecar detected").is_err());

    let namespace_a = new_restore_namespace().expect("restore namespace entropy");
    let namespace_b = new_restore_namespace().expect("restore namespace entropy");
    assert!(namespace_a.starts_with("restore_"));
    assert_eq!(namespace_a.len(), "restore_".len() + 32);
    assert!(namespace_a[8..].bytes().all(|byte| byte.is_ascii_hexdigit()));
    assert_ne!(namespace_a, namespace_b, "independent restores need isolated namespaces");

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn test_ocrm_snapshot_postgres_manifest_schema_and_profile_oracles() {
    let valid = json!({
        "schema_version": 2,
        "snapshot_id": "snap-002",
        "created_at": "2026-10-28T00:00:00.000Z",
        "profile": "exact",
        "source": {
            "engine": "postgres",
            "store_id": "123e4567-e89b-12d3-a456-426614174000",
            "owner": "state",
            "namespace": "snapshot_002",
            "schema_version": 2
        },
        "export_schema_version": 2,
        "active_fragments": ["core"],
        "verification": {
            "tables": {"messages": {"rows": 1, "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}},
            "allocations": {"messages_id_seq": {"value": "1", "called": true}}
        },
        "artifacts": {
            "database.sqlite": {"sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", "size_bytes": 1},
            "anchor.sqlite": {"sha256": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc", "size_bytes": 1}
        },
        "sanitized_differences": []
    });
    let parsed = parse_postgres_export_manifest(&valid, "snap-002", Some("exact"))
        .expect("valid schema-v2 manifest");
    assert_eq!(parsed.export_schema_version, 2);

    let export_root = std::env::temp_dir().join(format!(
        "fabushi-export-{}",
        std::process::id()
    ));
    let export_dir = export_root.join("snap-002");
    let _ = fs::remove_dir_all(&export_root);
    fs::create_dir_all(&export_dir).expect("create export directory");
    fs::write(export_dir.join("database.sqlite"), b"database-image")
        .expect("write database artifact");
    fs::write(export_dir.join("anchor.sqlite"), b"anchor-image").expect("write anchor artifact");
    let database_artifact = capture_database_artifact(export_dir.join("database.sqlite"))
        .expect("capture database artifact");
    let anchor_artifact = capture_database_artifact(export_dir.join("anchor.sqlite"))
        .expect("capture anchor artifact");
    let mut artifact_manifest = valid.clone();
    artifact_manifest["artifacts"]["database.sqlite"]["sha256"] =
        json!(database_artifact.sha256);
    artifact_manifest["artifacts"]["database.sqlite"]["size_bytes"] =
        json!(database_artifact.size_bytes);
    artifact_manifest["artifacts"]["anchor.sqlite"]["sha256"] = json!(anchor_artifact.sha256);
    artifact_manifest["artifacts"]["anchor.sqlite"]["size_bytes"] = json!(anchor_artifact.size_bytes);
    let parsed_artifacts = parse_postgres_export_manifest(&artifact_manifest, "snap-002", Some("exact"))
        .expect("parse artifact manifest");
    verify_postgres_export_artifacts(&export_dir, &parsed_artifacts, "exact")
        .expect("verify export artifacts");
    fs::write(export_dir.join("anchor.sqlite-wal"), b"unexpected sidecar")
        .expect("write sidecar");
    assert!(verify_postgres_export_artifacts(&export_dir, &parsed_artifacts, "exact").is_err());
    fs::remove_file(export_dir.join("anchor.sqlite-wal")).expect("remove sidecar fixture");

    fs::remove_file(export_dir.join("anchor.sqlite")).expect("remove text anchor fixture");
    let anchor_connection = rusqlite::Connection::open(export_dir.join("anchor.sqlite"))
        .expect("create SQLite anchor fixture");
    anchor_connection
        .execute(
            "CREATE TABLE openclaw_engine_anchor (store_id TEXT, engine TEXT, schema TEXT)",
            [],
        )
        .expect("create anchor table");
    anchor_connection
        .execute(
            "INSERT INTO openclaw_engine_anchor VALUES (?1, 'postgres', ?2)",
            rusqlite::params!["123e4567-e89b-12d3-a456-426614174000", "snapshot_002"],
        )
        .expect("insert anchor identity");
    drop(anchor_connection);
    let anchor_artifact = capture_database_artifact(export_dir.join("anchor.sqlite"))
        .expect("capture SQLite anchor artifact");
    artifact_manifest["artifacts"]["anchor.sqlite"]["sha256"] = json!(anchor_artifact.sha256);
    artifact_manifest["artifacts"]["anchor.sqlite"]["size_bytes"] = json!(anchor_artifact.size_bytes);
    let parsed_anchor = parse_postgres_export_manifest(&artifact_manifest, "snap-002", Some("exact"))
        .expect("parse anchor manifest");
    verify_postgres_anchor_identity(export_dir.join("anchor.sqlite"), &parsed_anchor)
        .expect("verify anchor identity");
    let mut wrong_anchor = artifact_manifest.clone();
    wrong_anchor["source"]["namespace"] = json!("wrong_namespace");
    let wrong_anchor = parse_postgres_export_manifest(&wrong_anchor, "snap-002", Some("exact"))
        .expect("parse wrong-anchor manifest");
    assert!(verify_postgres_anchor_identity(export_dir.join("anchor.sqlite"), &wrong_anchor).is_err());
    let _ = fs::remove_dir_all(&export_root);

    let manifest_path = std::env::temp_dir().join(format!(
        "fabushi-snapshot-manifest-{}-{}.json",
        std::process::id(),
        parsed.snapshot_id
    ));
    let _ = fs::remove_file(&manifest_path);
    write_postgres_export_manifest(&manifest_path, &valid, "snap-002")
        .expect("exclusive manifest write");
    assert!(write_postgres_export_manifest(&manifest_path, &valid, "snap-002").is_err());
    let written: serde_json::Value = serde_json::from_slice(
        &fs::read(&manifest_path).expect("read written manifest"),
    )
    .expect("parse written manifest");
    parse_postgres_export_manifest(&written, "snap-002", Some("exact"))
        .expect("written manifest remains valid");
    let _ = fs::remove_file(&manifest_path);

    let mut sanitized = valid.clone();
    sanitized["sanitized_differences"] = json!(["transient-state"]);
    assert!(parse_postgres_export_manifest(&sanitized, "snap-002", Some("exact")).is_err());

    let mut wrong_profile = valid.clone();
    wrong_profile["profile"] = json!("user-backup");
    assert!(parse_postgres_export_manifest(&wrong_profile, "snap-002", Some("exact")).is_err());

    let mut unknown = valid;
    unknown["unexpected"] = json!(true);
    assert!(parse_postgres_export_manifest(&unknown, "snap-002", None).is_err());
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
