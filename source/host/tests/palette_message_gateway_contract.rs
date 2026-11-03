use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::agents::agent_profile::SandAgentProfile;
use mahayana_host_runtime::extensions::session::gateway::dispatch_production_session_gateway_call;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use serde_json::{Value, json};

fn stored_transcript_rows(path: &Path) -> Vec<(String, String)> {
    let db = rusqlite::Connection::open_with_flags(
        path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .expect("open actual persisted transcript read-only");
    let mut statement = db
        .prepare("SELECT id, entry FROM transcript_entries ORDER BY seq")
        .expect("persisted transcript query");
    statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .expect("persisted transcript rows")
        .collect::<Result<Vec<_>, _>>()
        .expect("persisted transcript text")
}

fn gateway_case(
    session: &Arc<ProductionSessionWorkers>,
    id: &str,
    query: &str,
    expected: Value,
) -> Value {
    // Match ProductionRenderer's real request shape; use the Gateway's default
    // limit and production Session search owner, not a test search algorithm.
    let results = dispatch_production_session_gateway_call(
        session,
        "searchAgents",
        &json!({"query": query}),
    )
    .expect("registered Session searchAgents Gateway")
    .expect("production searchAgents reply");
    assert_eq!(results, expected, "independent oracle for {id}");
    json!({"id": id, "query": query, "results": results})
}

#[test]
fn palette_message_gateway_exports_persisted_rust_search_results() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "fabushi-pms-v1-{}-{suffix}",
        std::process::id(),
    ));
    let session = Arc::new(ProductionSessionWorkers::with_agents_root(&root, 500));
    let agent = session
        .materialize_new_session(
            Some(&SandAgentProfile {
                name: "PMS fixture".into(),
                description: String::new(),
                title: String::new(),
                avatar_shape: String::new(),
                avatar_color: String::new(),
            }),
            "user",
            None,
        )
        .expect("fixture session");
    let entries = vec![
        json!({"id":"sigma-short","kind":"message","role":"user","content":"ΟΣ","timestampMs":100}),
        json!({"id":"sigma-context","kind":"message","role":"assistant","content":"prefix ΟΣ suffix","timestampMs":200}),
        json!({"id":"expansion","kind":"message","role":"user","content":format!("{}Needle", "İ".repeat(90)),"timestampMs":300}),
        json!({"id":"contraction","kind":"send-message","message":{"type":"text","content":format!("{}Needle", "K".repeat(90))},"timestampMs":400}),
        json!({"id":"partial-expansion","kind":"message","role":"user","content":format!("{}İY{}", "z".repeat(35), "界".repeat(25)),"timestampMs":500}),
        json!({"id":"emoji","kind":"notice","text":"route 🧭 ready","timestampMs":600}),
        json!({"id":"rank-short","kind":"message","role":"user","content":"rank","timestampMs":700}),
        json!({"id":"rank-context","kind":"message","role":"assistant","content":"a longer rank context","timestampMs":800}),
        json!({"id":"hidden","kind":"message","role":"assistant","content":"ΟΣ Needle İY 🧭 rank","timestampMs":9000,"hiddenOutboundAgentPeerMessage":true}),
    ];
    assert_eq!(
        session
            .append_agent_transcript_entries(&agent.id, &entries)
            .expect("persist fixture through actual Session owner"),
        entries.len(),
    );
    let persisted = stored_transcript_rows(&agent.db_path);
    for entry in &entries {
        let (_, raw) = persisted
            .iter()
            .find(|(id, _)| Some(id.as_str()) == entry["id"].as_str())
            .expect("fixture row exists in SQLite");
        assert_eq!(serde_json::from_str::<Value>(raw).expect("stored JSON"), *entry);
    }
    session.shutdown();
    drop(session);

    let reopened = Arc::new(ProductionSessionWorkers::with_agents_root(&root, 500));
    let cases = vec![
        gateway_case(&reopened, "final-sigma", "ος", json!([
            {"agentId":agent.id,"entryId":"sigma-context","role":"assistant","timestampMs":200.0,"snippet":"prefix ΟΣ suffix"},
            {"agentId":agent.id,"entryId":"sigma-short","role":"user","timestampMs":100.0,"snippet":"ΟΣ"},
        ])),
        gateway_case(&reopened, "unicode-offsets", "needle", json!([
            {"agentId":agent.id,"entryId":"contraction","role":"assistant","timestampMs":400.0,"snippet":format!("…{}Needle", "K".repeat(10))},
            {"agentId":agent.id,"entryId":"expansion","role":"user","timestampMs":300.0,"snippet":format!("…{}Needle", "İ".repeat(15))},
        ])),
        gateway_case(&reopened, "partial-expansion", "\u{307}y", json!([
            {"agentId":agent.id,"entryId":"partial-expansion","role":"user","timestampMs":500.0,"snippet":format!("…{}İY{}…", "z".repeat(30), "界".repeat(20))},
        ])),
        gateway_case(&reopened, "combining-only", "\u{307}", json!([
            {"agentId":agent.id,"entryId":"partial-expansion","role":"user","timestampMs":500.0,"snippet":format!("…{}İY{}…", "z".repeat(30), "界".repeat(19))},
            {"agentId":agent.id,"entryId":"expansion","role":"user","timestampMs":300.0,"snippet":format!("{}…", "İ".repeat(31))},
        ])),
        gateway_case(&reopened, "emoji-only", "🧭", json!([
            {"agentId":agent.id,"entryId":"emoji","role":"assistant","timestampMs":600.0,"snippet":"route 🧭 ready"},
        ])),
        gateway_case(&reopened, "host-order", "rank", json!([
            {"agentId":agent.id,"entryId":"rank-context","role":"assistant","timestampMs":800.0,"snippet":"a longer rank context"},
            {"agentId":agent.id,"entryId":"rank-short","role":"user","timestampMs":700.0,"snippet":"rank"},
        ])),
        gateway_case(&reopened, "nonfinal-sigma", "οσ", json!([])),
        gateway_case(&reopened, "no-match", "missing-fixture", json!([])),
        gateway_case(&reopened, "empty-query", " \t ", json!([])),
    ];
    assert_eq!(
        stored_transcript_rows(&agent.db_path),
        persisted,
        "reopen and search preserve the original SQLite transcript bytes",
    );
    reopened.shutdown();
    drop(reopened);
    fs::remove_dir_all(&root).expect("remove owned Session fixture");

    // The behavioral assertions above run even without an export destination.
    // Actions binds this exact output to the same-H Node consumer receipt.
    if let Some(path) = std::env::var_os("OCRM_PALETTE_FIXTURE_PATH") {
        let fixture = json!({"contract":"PMS-v1","formatVersion":1,"cases":cases});
        fs::write(path, serde_json::to_vec_pretty(&fixture).expect("fixture JSON"))
            .expect("export real Gateway replies for the palette consumer");
    }
}
