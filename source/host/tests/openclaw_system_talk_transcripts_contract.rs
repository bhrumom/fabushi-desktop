use mahayana_host_runtime::extensions::state::state_persistence_engine::StatePersistenceEngine;
use mahayana_host_runtime::extensions::system_agent::system_agent_runner::{
    SystemAgentOperationKind, SystemAgentRunner,
};
use mahayana_host_runtime::extensions::talk::talk_controller::{
    TalkSessionController, TalkbackMode, VoiceSessionPhase,
};
use mahayana_host_runtime::extensions::transcripts::capture_summary::TranscriptCaptureStore;
use serde_json::json;

#[test]
fn test_ocrm_system_agent_runner() {
    let runner = SystemAgentRunner::new(true);

    // Operation parsing
    assert_eq!(
        runner.parse_operation("onboarding"),
        SystemAgentOperationKind::OnboardingSetup
    );
    assert_eq!(
        runner.parse_operation("status"),
        SystemAgentOperationKind::OverviewInspect
    );
    assert_eq!(
        runner.parse_operation("doctor"),
        SystemAgentOperationKind::DiagnosticAudit
    );
    assert_eq!(
        runner.parse_operation("custom:do-task"),
        SystemAgentOperationKind::CustomCommand("custom:do-task".to_string())
    );

    // Setup proposal
    let proposal = runner
        .prepare_setup_proposal(Some("/data/work"), Some("main-agent"), true)
        .expect("Proposal prepared");
    assert_eq!(proposal.proposed_workspace, "/data/work");
    assert_eq!(proposal.proposed_agent_name, "main-agent");
    assert_eq!(proposal.welcome_variant.as_deref(), Some("onboarding"));

    // Route unready check
    let unready_runner = SystemAgentRunner::new(false);
    assert!(unready_runner
        .prepare_setup_proposal(None, None, false)
        .is_err());
}

#[test]
fn test_ocrm_talk_voice_session() {
    let mut controller = TalkSessionController::new();
    assert_eq!(controller.phase(), VoiceSessionPhase::Inactive);

    controller.set_phase(VoiceSessionPhase::Listening);
    assert_eq!(controller.phase(), VoiceSessionPhase::Listening);

    controller.set_mode(TalkbackMode::AgentOnly);
    assert_eq!(controller.mode(), TalkbackMode::AgentOnly);

    // Audio energy RMS calculation
    // Silence: all zeroes
    let silence = vec![0u8; 100];
    assert_eq!(TalkSessionController::compute_audio_rms(&silence), 0.0);
    assert!(!controller.is_speech_detected(&silence));

    // High energy audio: alternating +16000, -16000
    let mut loud_pcm = Vec::new();
    for _ in 0..50 {
        loud_pcm.extend_from_slice(&16000i16.to_le_bytes());
        loud_pcm.extend_from_slice(&(-16000i16).to_le_bytes());
    }
    let rms = TalkSessionController::compute_audio_rms(&loud_pcm);
    assert!(rms > 0.4);
    assert!(controller.is_speech_detected(&loud_pcm));
}

#[test]
fn test_ocrm_transcripts_and_state() {
    // 1. Transcript capture store
    let store = TranscriptCaptureStore::new();
    let entry1 = store.append("Alice", "Let's review the plugin architecture");
    let entry2 = store.append("Bob", "Sounds good, looking at the code");

    assert_eq!(entry1.id, 1);
    assert_eq!(entry2.id, 2);

    let queried = store.query_entries(Some("plugin"), 10);
    assert_eq!(queried.len(), 1);
    assert_eq!(queried[0].speaker, "Alice");

    let summary = store.generate_summary_text();
    assert!(summary.contains("Alice: Let's review"));
    assert!(summary.contains("Bob: Sounds good"));

    // 2. State persistence engine
    let engine = StatePersistenceEngine::new(1);
    assert_eq!(engine.migration_version(), 1);

    assert!(engine.apply_migration(2).is_ok());
    assert_eq!(engine.migration_version(), 2);
    // Target version must be strictly greater than current
    assert!(engine.apply_migration(2).is_err());

    let v1 = engine.put_partition_entry("plugins", "state-1", json!({"active": true}));
    assert_eq!(v1, 1);

    let v2 = engine.put_partition_entry("plugins", "state-1", json!({"active": false}));
    assert_eq!(v2, 2);

    let retrieved = engine
        .get_partition_entry("plugins", "state-1")
        .expect("Entry found");
    assert_eq!(retrieved.version, 2);
    assert_eq!(retrieved.payload, json!({"active": false}));
}
