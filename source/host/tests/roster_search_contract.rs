use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
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
use serde_json::{Value, json};

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

#[test]
fn roster_snippet_preserves_ascii_and_utf8_byte_context_windows() {
    let ascii = format!("{}Needle{}", "a".repeat(35), "b".repeat(70));
    assert_eq!(
        build_content_snippet(&ascii, "needle"),
        Some(format!("…{}Needle{}…", "a".repeat(30), "b".repeat(60))),
    );
    let multibyte = format!("{}Needlez{}", "🙂".repeat(10), "🙂".repeat(20));
    assert_eq!(
        build_content_snippet(&multibyte, "needle"),
        Some(format!("…{}Needlez{}…", "🙂".repeat(7), "🙂".repeat(14))),
        "both context boundaries round inward without cutting the matched span",
    );
    assert_eq!(
        build_content_snippet("\u{2003}東京\n\u{a0}Needle\t終\u{2003}", "needle")
            .as_deref(),
        Some("東京 Needle 終"),
    );
    assert_eq!(build_content_snippet("", "needle"), None);
    assert_eq!(build_content_snippet(" \n\t", "needle"), None);
    assert_eq!(build_content_snippet("İ", ""), None);
    assert_eq!(build_content_snippet("İ", "missing"), None);
}

fn stored_transcript_rows(path: &Path) -> Vec<(String, String)> {
    let db = rusqlite::Connection::open_with_flags(
        path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .expect("open persisted transcript without a write handle");
    let mut statement = db
        .prepare("SELECT id, entry FROM transcript_entries ORDER BY seq")
        .expect("persisted transcript query");
    statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .expect("persisted transcript rows")
        .collect::<Result<Vec<_>, _>>()
        .expect("persisted transcript text")
}

fn search_gateway(
    session: &Arc<ProductionSessionWorkers>,
    index: Option<&dyn RosterContentSearch>,
    query: &str,
    limit: usize,
) -> Value {
    dispatch_production_session_gateway_call_with_content_search(
        session,
        index,
        "searchAgents",
        &json!({"query":query,"limit":limit}),
    )
    .expect("registered searchAgents gateway")
    .expect("searchAgents reply")
}

fn assert_unicode_gateway_fallback(label: &str, index: Option<&dyn RosterContentSearch>) {
    let root = temp_root(label);
    let session = Arc::new(ProductionSessionWorkers::with_agents_root(&root, 500));
    let agent = session
        .materialize_new_session(Some(&profile("Unicode")), "user", None)
        .expect("unicode agent");
    let entries = vec![
        json!({"id":"expansion","kind":"message","role":"user","content":format!("{}Needle", "İ".repeat(90)),"timestampMs":100}),
        json!({"id":"contraction","kind":"send-message","message":{"type":"text","content":format!("{}Needle", "K".repeat(90))},"timestampMs":200}),
        json!({"id":"multibyte","kind":"notice","text":format!("{}Needlez{}", "🙂".repeat(10), "🙂".repeat(20)),"timestampMs":300}),
        json!({"id":"sigma","kind":"message","role":"user","content":format!("{} ΟΣ!", "🙂".repeat(10)),"timestampMs":400}),
        json!({"id":"partial-expansion","kind":"message","role":"user","content":format!("{}İY{}", "z".repeat(35), "界".repeat(25)),"timestampMs":500}),
        json!({"id":"hidden","kind":"message","role":"assistant","content":"Needle secret","timestampMs":900,"hiddenOutboundAgentPeerMessage":true}),
    ];
    assert_eq!(
        session.append_agent_transcript_entries(&agent.id, &entries).expect("persist fixtures"),
        entries.len(),
    );
    let persisted = stored_transcript_rows(&agent.db_path);
    for entry in &entries {
        let (_, raw) = persisted.iter().find(|(id, _)| Some(id.as_str()) == entry["id"].as_str())
            .expect("fixture exists in the actual SQLite transcript");
        assert_eq!(serde_json::from_str::<Value>(raw).expect("stored JSON"), *entry);
    }

    let expected = json!([
        {"agentId":agent.id,"entryId":"multibyte","role":"assistant","timestampMs":300.0,"snippet":format!("…{}Needlez{}…", "🙂".repeat(7), "🙂".repeat(14))},
        {"agentId":agent.id,"entryId":"contraction","role":"assistant","timestampMs":200.0,"snippet":format!("…{}Needle", "K".repeat(10))},
        {"agentId":agent.id,"entryId":"expansion","role":"user","timestampMs":100.0,"snippet":format!("…{}Needle", "İ".repeat(15))},
    ]);
    assert_eq!(search_gateway(&session, index, "  NeEdLe  ", 10), expected);
    assert_eq!(search_gateway(&session, index, "ΟΣ", 10), json!([
        {"agentId":agent.id,"entryId":"sigma","role":"user","timestampMs":400.0,"snippet":format!("…{} ΟΣ!", "🙂".repeat(7))},
    ]));
    assert_eq!(search_gateway(&session, index, "\u{307}y", 10), json!([
        {"agentId":agent.id,"entryId":"partial-expansion","role":"user","timestampMs":500.0,"snippet":format!("…{}İY{}…", "z".repeat(30), "界".repeat(20))},
    ]));
    assert_eq!(search_gateway(&session, index, "not-present", 10), json!([]));
    assert_eq!(search_gateway(&session, index, "  ", 10), json!([]));
    assert_eq!(search_gateway(&session, index, "needle", 0), json!([]));
    assert_eq!(stored_transcript_rows(&agent.db_path), persisted, "search is read-only");

    session.shutdown();
    drop(session);
    let reopened = Arc::new(ProductionSessionWorkers::with_agents_root(&root, 500));
    assert_eq!(search_gateway(&reopened, index, "needle", 10), expected);
    assert_eq!(stored_transcript_rows(&agent.db_path), persisted, "reopen retains original text");
    reopened.shutdown();
    drop(reopened);
    fs::remove_dir_all(root).expect("remove owned unicode fixture");
}

#[derive(Default)]
struct UnavailableRosterSearch {
    ready: bool,
    readiness_calls: AtomicUsize,
    message_calls: AtomicUsize,
}

impl RosterContentSearch for UnavailableRosterSearch {
    fn is_search_ready(&self) -> bool {
        self.readiness_calls.fetch_add(1, Ordering::SeqCst);
        self.ready
    }
    fn max_matches_per_agent(&self) -> usize { 5 }
    fn max_results(&self) -> usize { 50 }
    fn search_messages(&self, _query: &str, _limit: usize) -> Option<Vec<MessageSearchResult>> {
        self.message_calls.fetch_add(1, Ordering::SeqCst);
        None
    }
    fn search_media(&self, _query: &str, _limit: usize) -> Option<Vec<MediaSearchResult>> {
        panic!("message fallback must not query media")
    }
}

#[test]
fn production_gateway_unicode_fallback_without_index_preserves_storage_and_reopens() {
    assert_unicode_gateway_fallback("unicode-no-index", None);
}

#[test]
fn production_gateway_unicode_fallback_with_unready_index_never_queries_it() {
    let index = UnavailableRosterSearch::default();
    assert_unicode_gateway_fallback("unicode-unready-index", Some(&index));
    assert_eq!(index.readiness_calls.load(Ordering::SeqCst), 5);
    assert_eq!(index.message_calls.load(Ordering::SeqCst), 0);
}

#[test]
fn production_gateway_unicode_fallback_with_unavailable_index_queries_then_scans() {
    let index = UnavailableRosterSearch { ready: true, ..Default::default() };
    assert_unicode_gateway_fallback("unicode-unavailable-index", Some(&index));
    assert_eq!(index.readiness_calls.load(Ordering::SeqCst), 5);
    assert_eq!(index.message_calls.load(Ordering::SeqCst), 5);
}

#[test]
fn production_gateway_fallback_preserves_unicode_match_limits_and_recency() {
    let root = temp_root("unicode-limits");
    let session = Arc::new(ProductionSessionWorkers::with_agents_root(&root, 500));
    let first = session.materialize_new_session(Some(&profile("First")), "user", None)
        .expect("first agent");
    let second = session.materialize_new_session(Some(&profile("Second")), "user", None)
        .expect("second agent");
    let mut entries = (0..7).map(|index| json!({
        "id":format!("first-{index}"),"kind":"message","role":"user",
        "content":format!("{}Needle {index}", "İ".repeat(90)),"timestampMs":100 + index,
    })).collect::<Vec<_>>();
    entries.push(json!({"id":"hidden-newest","kind":"message","role":"user","content":"Needle hidden","timestampMs":2000,"hiddenOutboundAgentPeerMessage":true}));
    assert_eq!(session.append_agent_transcript_entries(&first.id, &entries).unwrap(), 8);
    assert_eq!(session.append_agent_transcript_entries(&second.id, &[
        json!({"id":"second","kind":"message","role":"assistant","content":"Needle second","timestampMs":1000}),
    ]).unwrap(), 1);
    let full = search_gateway(&session, None, "needle", 50);
    assert_eq!(
        full.as_array().unwrap().iter().map(|entry| entry["entryId"].as_str().unwrap()).collect::<Vec<_>>(),
        ["second", "first-6", "first-5", "first-4", "first-3", "first-2"],
    );
    assert_eq!(full[0]["agentId"], second.id);
    assert_eq!(full[1]["agentId"], first.id);
    assert_eq!(full[0]["timestampMs"], 1000.0);
    assert_eq!(full[1]["timestampMs"], 106.0);
    assert_eq!(full[1]["snippet"], format!("…{}Needle 6", "İ".repeat(15)));
    let limited = search_gateway(&session, None, "needle", 3);
    assert_eq!(limited.as_array().unwrap(), &full.as_array().unwrap()[..3]);
    session.shutdown();
    drop(session);
    fs::remove_dir_all(root).expect("remove owned limit fixture");
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
    assert_eq!(session.append_agent_transcript_entries(&agent.id, &[
        json!({"id":"linear-only","kind":"message","role":"user","content":format!("{}Needle", "İ".repeat(90)),"timestampMs":2000}),
    ]).expect("matching fallback input"), 1);
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
