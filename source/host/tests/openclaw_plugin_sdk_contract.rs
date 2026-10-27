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
