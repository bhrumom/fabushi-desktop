use mahayana_host_runtime::extensions::plugin_sdk::access_groups::{
    parse_access_group_allow_from_entry, AccessGroup, AccountIdentifier,
};
use mahayana_host_runtime::extensions::plugin_sdk::acp_runtime::{
    AcpDoctorReport, AcpError, AcpRuntimeStatus, AcpSessionEntry, AcpSessionManager,
};
use mahayana_host_runtime::extensions::plugin_sdk::agent_harness::{
    AgentHarnessManager, AttemptTerminalStatus, StructuredInputForm,
};

#[test]
fn test_ocrm_plugin_sdk_access_groups_and_account_id() {
    assert_eq!(
        parse_access_group_allow_from_entry("accessGroup:admin-team"),
        Some("admin-team")
    );
    assert_eq!(
        parse_access_group_allow_from_entry("  accessGroup: dev-team  "),
        Some("dev-team")
    );
    assert_eq!(parse_access_group_allow_from_entry("regular-user"), None);

    let group = AccessGroup::new("ops")
        .with_sender("alice@example.com")
        .with_sender("bob@example.com");

    assert!(group.is_sender_allowed("alice@example.com"));
    assert!(group.is_sender_allowed("BOB@EXAMPLE.COM"));
    assert!(!group.is_sender_allowed("charlie@example.com"));

    let wildcard_group = AccessGroup::new("public").with_allow_all(true);
    assert!(wildcard_group.is_sender_allowed("anyone"));

    let acct = AccountIdentifier::new("slack", "team-123");
    assert_eq!(acct.canonical_key(), "slack:team-123");
    let parsed = AccountIdentifier::parse_canonical("slack:team-123").expect("parsed canonical");
    assert_eq!(parsed, acct);
    assert!(AccountIdentifier::parse_canonical("invalid").is_none());
}

#[test]
fn test_ocrm_plugin_sdk_acp_runtime() {
    let mut manager = AcpSessionManager::new();

    let normal_session = AcpSessionEntry {
        session_id: "sess-normal".to_string(),
        is_incognito: false,
        is_active: true,
        agent_id: "agent-1".to_string(),
    };
    manager.register_session(normal_session);

    let incognito_session = AcpSessionEntry {
        session_id: "sess-incognito".to_string(),
        is_incognito: true,
        is_active: true,
        agent_id: "agent-2".to_string(),
    };
    manager.register_session(incognito_session);

    // Normal session access ok with or without sync
    assert!(manager.verify_session_access("sess-normal", false).is_ok());
    assert!(manager.verify_session_access("sess-normal", true).is_ok());

    // Incognito session access without sync is ok
    assert!(manager.verify_session_access("sess-incognito", false).is_ok());
    // Incognito session sync access is denied
    assert_eq!(
        manager.verify_session_access("sess-incognito", true),
        Err(AcpError::IncognitoSessionSyncAccessDenied("sess-incognito".to_string()))
    );

    // End incognito session
    manager.end_session("sess-incognito").expect("ended");
    assert_eq!(
        manager.verify_session_access("sess-incognito", false),
        Err(AcpError::IncognitoSessionEnded("sess-incognito".to_string()))
    );

    // Doctor report
    let report: AcpDoctorReport = manager.generate_doctor_report("embedded-acp");
    assert_eq!(report.status, AcpRuntimeStatus::Ready);
    assert_eq!(report.active_sessions_count, 1);
}

#[test]
fn test_ocrm_plugin_sdk_agent_harness() {
    let mut manager = AgentHarnessManager::new();
    manager.register_attempt("attempt-1", "sess-1");

    assert!(manager.abort_and_drain("attempt-1", "user cancelled"));
    let attempt = manager.get_attempt("attempt-1").expect("attempt exists");
    assert_eq!(attempt.terminal_status, Some(AttemptTerminalStatus::Aborted));
    assert_eq!(attempt.error_message.as_deref(), Some("user cancelled"));

    // Structured Input Form
    let mut form = StructuredInputForm::new("login-form")
        .add_field("username", "Enter username", true)
        .add_field("notes", "Optional notes", false);

    // Missing required field
    assert!(form.validate_answers().is_err());

    // Answer required field
    assert!(form.set_answer("username", "admin"));
    assert!(form.validate_answers().is_ok());
}

#[test]
fn test_ocrm_plugin_sdk_allow_from_and_config_edit() {
    use mahayana_host_runtime::extensions::plugin_sdk::allow_from::{
        apply_account_scoped_allowlist_config_edit, format_allow_from_lowercase,
        format_normalized_allow_from_entries, is_normalized_sender_allowed,
        parse_allow_from_entries, resolve_dm_group_allowlist_config_paths,
        resolve_legacy_dm_allowlist_config_paths,
    };
    use serde_json::json;

    // 1. Lowercase formatting with prefix stripping
    let raw = vec![
        "  tg:Alice  ".to_string(),
        "TG:BOB".to_string(),
        "Charlie".to_string(),
    ];
    let formatted = format_allow_from_lowercase(&raw, Some("tg:"));
    assert_eq!(formatted, vec!["alice", "bob", "charlie"]);

    // 2. Normalized formatting
    let normalized = format_normalized_allow_from_entries(&raw, |s| {
        if s.starts_with("Charlie") {
            None
        } else {
            Some(s.to_uppercase())
        }
    });
    assert_eq!(normalized, vec!["TG:ALICE", "TG:BOB"]);

    // 3. Parsing with wildcard and error handling
    let parsed_ok = parse_allow_from_entries("user1, *; user2\nuser3", |s| Ok(s.to_string()));
    assert_eq!(
        parsed_ok.unwrap(),
        vec!["user1", "*", "user2", "user3"]
    );

    let parsed_err = parse_allow_from_entries("valid, bad-syntax", |s| {
        if s == "bad-syntax" {
            Err("invalid format".to_string())
        } else {
            Ok(s.to_string())
        }
    });
    assert!(parsed_err.is_err());

    // 4. Sender allowance checks
    let allowlist = vec!["alice".to_string(), "bob".to_string()];
    assert!(is_normalized_sender_allowed("ALICE", &allowlist, None));
    assert!(is_normalized_sender_allowed("bob", &allowlist, None));
    assert!(!is_normalized_sender_allowed("charlie", &allowlist, None));

    let allow_all = vec!["*".to_string()];
    assert!(is_normalized_sender_allowed("anyone", &allow_all, None));

    let allow_empty: Vec<String> = vec![];
    assert!(!is_normalized_sender_allowed("anyone", &allow_empty, None));

    // 5. Config path resolution
    let (dm_read, dm_write) = resolve_dm_group_allowlist_config_paths("dm");
    assert_eq!(dm_read, &["allowFrom"]);
    assert_eq!(dm_write, &["allowFrom"]);

    let legacy_dm = resolve_legacy_dm_allowlist_config_paths("dm");
    assert!(legacy_dm.is_some());
    assert!(resolve_legacy_dm_allowlist_config_paths("group").is_none());

    // 6. Config tree edit
    let mut root = json!({});
    apply_account_scoped_allowlist_config_edit(
        &mut root,
        "slack",
        Some("team-1"),
        &["allowFrom"],
        vec!["userA".to_string(), "userB".to_string()],
    );

    assert_eq!(
        root["channels"]["slack"]["accounts"]["team-1"]["allowFrom"],
        json!(["userA", "userB"])
    );
}

#[test]
fn test_ocrm_plugin_sdk_api_baseline_and_diff() {
    use mahayana_host_runtime::extensions::plugin_sdk::api_baseline::{
        diff_plugin_sdk_api, format_plugin_sdk_api_diff_report, has_plugin_sdk_api_changes,
        normalize_plugin_sdk_api_source_path, PluginSdkApiExport, PluginSdkApiExportKind,
    };
    use std::path::Path;

    let repo_root = Path::new("/workspace/project");
    let file_path = Path::new("/workspace/project/src/plugin-sdk/index.ts");
    assert_eq!(
        normalize_plugin_sdk_api_source_path(repo_root, file_path),
        "src/plugin-sdk/index.ts"
    );

    let before = vec![
        PluginSdkApiExport {
            name: "calculate".to_string(),
            kind: PluginSdkApiExportKind::Function,
            declaration: "declare function calculate(): number".to_string(),
            closure_hash: "hash1".to_string(),
            docs_path: None,
            subpath: None,
        },
        PluginSdkApiExport {
            name: "OldType".to_string(),
            kind: PluginSdkApiExportKind::Type,
            declaration: "type OldType = string".to_string(),
            closure_hash: "hash2".to_string(),
            docs_path: None,
            subpath: None,
        },
    ];

    let after = vec![
        PluginSdkApiExport {
            name: "calculate".to_string(),
            kind: PluginSdkApiExportKind::Const, // Kind changed -> breaking!
            declaration: "declare const calculate: number".to_string(),
            closure_hash: "hash1-changed".to_string(),
            docs_path: None,
            subpath: None,
        },
        PluginSdkApiExport {
            name: "NewFunction".to_string(), // Added!
            kind: PluginSdkApiExportKind::Function,
            declaration: "declare function NewFunction(): void".to_string(),
            closure_hash: "hash3".to_string(),
            docs_path: None,
            subpath: None,
        },
        // OldType was removed -> breaking!
    ];

    let diff = diff_plugin_sdk_api(&before, &after);
    assert!(has_plugin_sdk_api_changes(&diff));
    assert_eq!(diff.added_exports, vec!["NewFunction"]);
    assert_eq!(diff.removed_exports, vec!["OldType"]);
    assert_eq!(diff.breaking_changes.len(), 2);

    let report = format_plugin_sdk_api_diff_report(&diff);
    assert!(report.contains("### Breaking Changes"));
    assert!(report.contains("### Added Exports"));
    assert!(report.contains("### Removed Exports"));
}

#[test]
fn test_ocrm_plugin_sdk_agent_workspace_runtime() {
    use mahayana_host_runtime::extensions::plugin_sdk::agent_workspace_runtime::{
        WorkspaceAttachmentPreparer, WorkspaceBootstrapFilePolicy,
    };
    use std::path::Path;

    let policy = WorkspaceBootstrapFilePolicy::new(false, 1024 * 1024);
    assert!(policy.is_file_allowed(Path::new("src/main.rs"), 500));
    assert!(!policy.is_file_allowed(Path::new(".git/config"), 100)); // Hidden file rejected
    assert!(!policy.is_file_allowed(Path::new("big.bin"), 2 * 1024 * 1024)); // Exceeds size

    let preparer = WorkspaceAttachmentPreparer::new(5, 10 * 1024);
    assert!(preparer.can_admit_attachment(2, 2048, 1024));
    assert!(!preparer.can_admit_attachment(5, 2048, 1024)); // Count limit
    assert!(!preparer.can_admit_attachment(2, 8 * 1024, 4 * 1024)); // Size limit
}

