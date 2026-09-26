use std::fs;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::agents::agent_profile::SandAgentProfile;
use mahayana_host_runtime::extensions::session::gateway::dispatch_production_session_gateway_call;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::transcript::roster_search::{
    AGENT_CONTENT_SEARCH_MAX_MATCHES_PER_AGENT, build_content_snippet,
    find_agent_content_matches, search_agents_linear,
};
use serde_json::json;

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-roster-search-{label}-{}-{suffix}",
        std::process::id()
    ))
}

fn profile(name: &str) -> SandAgentProfile {
    SandAgentProfile {
        name: name.into(),
        description: String::new(),
        title: String::new(),
        avatar_shape: String::new(),
        avatar_color: String::new(),
    }
}

#[test]
fn frozen_content_match_rules_preserve_snippets_order_and_hidden_peer_filter() {
    let entries = vec![
        json!({"id":"one","kind":"message","role":"user","content":"before needle after","timestampMs":10}),
        json!({"id":"two","kind":"notice","text":"NEEDLE second","timestampMs":20}),
        json!({"id":"hidden","kind":"message","role":"assistant","content":"needle hidden","timestampMs":30,"hiddenOutboundAgentPeerMessage":true}),
        json!({"id":"three","kind":"send-message","message":{"type":"text","content":"needle third"},"timestampMs":40}),
    ];
    let matches = find_agent_content_matches(
        &entries,
        "needle",
        AGENT_CONTENT_SEARCH_MAX_MATCHES_PER_AGENT,
    );
    assert_eq!(
        matches.iter().map(|entry| entry.entry_id.as_str()).collect::<Vec<_>>(),
        vec!["three", "two", "one"]
    );
    assert_eq!(matches[0].role, "assistant");
    assert_eq!(matches[2].role, "user");
    assert_eq!(
        build_content_snippet("alpha   needle\n beta", "needle").as_deref(),
        Some("alpha needle beta")
    );
}

#[test]
fn production_search_agents_scans_real_session_transcripts_and_gateway_dispatches_it() {
    let root = temp_root("production");
    let session = Arc::new(ProductionSessionWorkers::with_agents_root(&root, 500));
    let first = session
        .materialize_new_session(Some(&profile("First")), "user", None)
        .expect("first agent");
    let second = session
        .materialize_new_session(Some(&profile("Second")), "user", None)
        .expect("second agent");

    session
        .append_agent_transcript_entries(
            &first.id,
            &[
                json!({"id":"first-old","kind":"message","role":"user","content":"needle old","timestampMs":100}),
                json!({"id":"first-new","kind":"message","role":"assistant","content":"needle newest","timestampMs":300}),
            ],
        )
        .expect("first transcript");
    session
        .append_agent_transcript_entries(
            &second.id,
            &[json!({"id":"second","kind":"notice","text":"needle middle","timestampMs":200})],
        )
        .expect("second transcript");

    let results = search_agents_linear(&session, "  NeEdLe  ", 10).expect("search");
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].entry_id, "first-new");
    assert_eq!(results[1].entry_id, "second");
    assert_eq!(results[2].entry_id, "first-old");

    let gateway = dispatch_production_session_gateway_call(
        &session,
        "searchAgents",
        &json!({"query":"needle","limit":2}),
    )
    .expect("gateway method")
    .expect("gateway search");
    assert_eq!(gateway.as_array().map(Vec::len), Some(2));
    assert_eq!(gateway[0]["entryId"], "first-new");

    session.shutdown();
    let _ = fs::remove_dir_all(root);
}
