use std::fs;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::agents::agent_profile::SandAgentProfile;
use mahayana_host_runtime::extensions::content_search::search_index_db::{
    AttachmentKind, MediaSearchResult, MessageSearchResult,
};
use mahayana_host_runtime::extensions::session::gateway::{
    dispatch_production_session_gateway_call,
    dispatch_production_session_gateway_call_with_content_search,
};
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::transcript::roster_search::{
    AGENT_CONTENT_SEARCH_MAX_MATCHES_PER_AGENT, RosterContentSearch, build_content_snippet,
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


struct FakeIndexedRosterSearch {
    agent_id: String,
}

impl RosterContentSearch for FakeIndexedRosterSearch {
    fn is_search_ready(&self) -> bool { true }
    fn max_matches_per_agent(&self) -> usize { 5 }
    fn max_results(&self) -> usize { 50 }

    fn search_messages(&self, query: &str, _limit: usize) -> Option<Vec<MessageSearchResult>> {
        assert_eq!(query, "needle");
        Some(vec![
            MessageSearchResult {
                agent_id: self.agent_id.clone(),
                entry_id: "indexed-message".into(),
                role: "assistant".into(),
                timestamp_ms: 900,
                snippet: "indexed needle result".into(),
            },
            MessageSearchResult {
                agent_id: "deleted-agent".into(),
                entry_id: "stale-message".into(),
                role: "assistant".into(),
                timestamp_ms: 1000,
                snippet: "must be filtered".into(),
            },
        ])
    }

    fn search_media(&self, query: &str, _limit: usize) -> Option<Vec<MediaSearchResult>> {
        assert_eq!(query, "needle");
        Some(vec![
            MediaSearchResult {
                agent_id: self.agent_id.clone(),
                entry_id: "indexed-media".into(),
                file_name: "needle.png".into(),
                ext: "png".into(),
                mime: Some("image/png".into()),
                kind: AttachmentKind::Image,
                timestamp_ms: 901,
                width: Some(640),
                height: Some(480),
            },
            MediaSearchResult {
                agent_id: "deleted-agent".into(),
                entry_id: "stale-media".into(),
                file_name: "stale.png".into(),
                ext: "png".into(),
                mime: Some("image/png".into()),
                kind: AttachmentKind::Image,
                timestamp_ms: 1001,
                width: None,
                height: None,
            },
        ])
    }
}

#[test]
fn production_gateway_consumes_ready_content_search_for_indexed_messages_and_media() {
    let root = temp_root("indexed-gateway");
    let session = Arc::new(ProductionSessionWorkers::with_agents_root(&root, 500));
    let agent = session
        .materialize_new_session(Some(&profile("Indexed")), "user", None)
        .expect("indexed agent");
    let indexed = FakeIndexedRosterSearch {
        agent_id: agent.id.clone(),
    };

    let messages = dispatch_production_session_gateway_call_with_content_search(
        &session,
        Some(&indexed),
        "searchAgents",
        &json!({"query":" needle ","limit":10}),
    )
    .expect("searchAgents method")
    .expect("indexed searchAgents");
    assert_eq!(messages.as_array().map(Vec::len), Some(1));
    assert_eq!(messages[0]["agentId"], agent.id);
    assert_eq!(messages[0]["entryId"], "indexed-message");
    assert_eq!(messages[0]["snippet"], "indexed needle result");

    let media = dispatch_production_session_gateway_call_with_content_search(
        &session,
        Some(&indexed),
        "searchMedia",
        &json!({"query":"needle","limit":10}),
    )
    .expect("searchMedia method")
    .expect("indexed searchMedia");
    assert_eq!(media.as_array().map(Vec::len), Some(1));
    assert_eq!(media[0]["agentId"], agent.id);
    assert_eq!(media[0]["entryId"], "indexed-media");
    assert_eq!(media[0]["fileName"], "needle.png");
    assert_eq!(media[0]["kind"], "image");
    assert_eq!(media[0]["width"], 640);
    assert_eq!(media[0]["height"], 480);

    session.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn canonical_search_merges_authenticated_human_and_agent_conversations_and_enforces_account_isolation() {
    let root = temp_root("canonical-human-agent-search");
    let memory = Arc::new(
        mahayana_host_runtime::extensions::memory::memory_service::MemoryService::new(root.clone()),
    );
    let session = Arc::new(
        ProductionSessionWorkers::with_agents_root_identity_and_dependencies(
            &root,
            500,
            Some("human-local".into()),
            Arc::new(|| None),
            memory,
        ),
    );
    let agent = session
        .materialize_new_session(Some(&profile("IndexedAgent")), "user", None)
        .expect("indexed agent");
    let human_conversation = session
        .create_human_conversation("human-peer", "Alice")
        .expect("human conversation");
    let conversation_id = human_conversation["id"]
        .as_str()
        .expect("conversation id")
        .to_string();

    let human_first = session
        .append_human_message(
            &conversation_id,
            "needle from local human",
            "nonce-search-1",
            Some(950.0),
            None,
            &[json!({"path": "/tmp/needle-report.pdf", "name": "needle-report.pdf"})],
        )
        .expect("first human message");
    let human_agent_handoff = session
        .append_human_agent_message(
            &conversation_id,
            &agent.id,
            "stream-search-1",
            "needle from handoff agent",
            980.0,
        )
        .expect("human agent handoff message");

    let indexed = FakeIndexedRosterSearch {
        agent_id: agent.id.clone(),
    };

    let unified_messages = dispatch_production_session_gateway_call_with_content_search(
        &session,
        Some(&indexed),
        "searchAgents",
        &json!({"query":" needle ","limit":10}),
    )
    .expect("searchAgents method")
    .expect("unified searchAgents");
    let unified_array = unified_messages.as_array().expect("searchAgents array");
    assert_eq!(unified_array.len(), 3);
    assert_eq!(unified_array[0]["agentId"], conversation_id);
    assert_eq!(unified_array[0]["entryId"], human_agent_handoff["id"]);
    assert_eq!(unified_array[0]["role"], "assistant");
    assert_eq!(unified_array[1]["agentId"], conversation_id);
    assert_eq!(unified_array[1]["entryId"], human_first["id"]);
    assert_eq!(unified_array[1]["role"], "user");
    assert_eq!(unified_array[2]["agentId"], agent.id);
    assert_eq!(unified_array[2]["entryId"], "indexed-message");

    let linear_messages = dispatch_production_session_gateway_call(
        &session,
        "searchAgents",
        &json!({"query":"needle-report.pdf","limit":10}),
    )
    .expect("searchAgents linear method")
    .expect("linear searchAgents");
    let linear_array = linear_messages.as_array().expect("linear array");
    assert_eq!(linear_array.len(), 1);
    assert_eq!(linear_array[0]["agentId"], conversation_id);
    assert_eq!(linear_array[0]["entryId"], human_first["id"]);

    let unified_media = dispatch_production_session_gateway_call_with_content_search(
        &session,
        Some(&indexed),
        "searchMedia",
        &json!({"query":"needle","limit":10}),
    )
    .expect("searchMedia method")
    .expect("unified searchMedia");
    let media_array = unified_media.as_array().expect("searchMedia array");
    assert_eq!(media_array.len(), 2);
    assert_eq!(media_array[0]["agentId"], conversation_id);
    assert_eq!(media_array[0]["entryId"], human_first["id"]);
    assert_eq!(media_array[0]["fileName"], "needle-report.pdf");
    assert_eq!(media_array[0]["ext"], ".pdf");
    assert_eq!(media_array[0]["kind"], "pdf");
    assert_eq!(media_array[1]["agentId"], agent.id);
    assert_eq!(media_array[1]["entryId"], "indexed-media");

    let scoped_human_messages = dispatch_production_session_gateway_call_with_content_search(
        &session,
        Some(&indexed),
        "searchHumanMessages",
        &json!({
            "conversationId": conversation_id,
            "query": "needle",
            "limit": 10
        }),
    )
    .expect("searchHumanMessages method")
    .expect("scoped searchHumanMessages");
    assert_eq!(
        scoped_human_messages,
        json!([human_agent_handoff, human_first])
    );

    session.shutdown();

    let foreign_memory = Arc::new(
        mahayana_host_runtime::extensions::memory::memory_service::MemoryService::new(root.clone()),
    );
    let foreign = Arc::new(
        ProductionSessionWorkers::with_agents_root_identity_and_dependencies(
            &root,
            500,
            Some("human-foreign".into()),
            Arc::new(|| None),
            foreign_memory,
        ),
    );
    let foreign_messages = dispatch_production_session_gateway_call_with_content_search(
        &foreign,
        Some(&indexed),
        "searchAgents",
        &json!({"query":"needle","limit":10}),
    )
    .expect("foreign searchAgents method")
    .expect("foreign searchAgents");
    let foreign_array = foreign_messages.as_array().expect("foreign array");
    assert_eq!(foreign_array.len(), 1);
    assert_eq!(foreign_array[0]["agentId"], agent.id);

    let foreign_media = dispatch_production_session_gateway_call_with_content_search(
        &foreign,
        Some(&indexed),
        "searchMedia",
        &json!({"query":"needle","limit":10}),
    )
    .expect("foreign searchMedia method")
    .expect("foreign searchMedia");
    let foreign_media_array = foreign_media.as_array().expect("foreign media array");
    assert_eq!(foreign_media_array.len(), 1);
    assert_eq!(foreign_media_array[0]["agentId"], agent.id);

    let foreign_scoped = dispatch_production_session_gateway_call_with_content_search(
        &foreign,
        Some(&indexed),
        "searchHumanMessages",
        &json!({
            "conversationId": conversation_id,
            "query": "needle",
            "limit": 10
        }),
    )
    .expect("foreign searchHumanMessages")
    .expect_err("foreign account must be rejected from non-participant Human search");
    assert!(format!("{foreign_scoped:?}").contains("not a conversation participant"));

    foreign.shutdown();
    let _ = fs::remove_dir_all(root);
}
