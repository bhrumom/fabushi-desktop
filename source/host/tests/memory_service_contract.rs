use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::memory::memory_service::{
    MemoryKind, agent_memory_has_content, get_agent_memory_dir, normalize_memory_content,
    parse_facts,
};

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-memory-service-{label}-{}-{suffix}",
        std::process::id()
    ))
}

#[test]
fn memory_fact_parser_matches_frozen_markdown_contract() {
    let facts = parse_facts(
        "# About the user\n\n- (2026-09-24)  User   likes   tea  \n- (bad-date) ignored\n- (2026-13-01) invalid month\n",
        MemoryKind::Profile,
    );
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].date, "2026-09-24");
    assert_eq!(facts[0].content, "User likes tea");
    assert_eq!(facts[0].kind, MemoryKind::Profile);
    assert_eq!(
        normalize_memory_content("  many   spaces\nacross\tlines  "),
        "many spaces across lines"
    );
}

#[test]
fn durable_memory_content_requires_a_valid_fact_in_profile_or_log() {
    let root = temp_root("content");
    let agent = root.join("agent");
    let memory = get_agent_memory_dir(&agent);
    fs::create_dir_all(memory.join("log")).expect("memory dirs");

    fs::write(
        memory.join("profile.md"),
        "# About the user\n\n<!-- metadata only -->\n",
    )
    .expect("empty profile");
    fs::write(memory.join("log").join("2026-09.md"), "not a fact\n")
        .expect("empty log");
    assert!(!agent_memory_has_content(&agent));

    fs::write(
        memory.join("log").join("2026-09.md"),
        "# Memory log\n\n- (2026-09-24) Planning a release\n",
    )
    .expect("fact");
    assert!(agent_memory_has_content(&agent));

    let _ = fs::remove_dir_all(root);
}
