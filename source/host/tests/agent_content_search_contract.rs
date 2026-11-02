use mahayana_host_runtime::extensions::content_search::agent_content_search::{
    AGENT_CONTENT_SEARCH_MAX_MATCHES_PER_AGENT, AGENT_CONTENT_SEARCH_MAX_RESULTS,
    AgentContentRole, SearchableTranscriptEntry, SearchableTranscriptEntryKind,
    build_content_snippet, entry_search_text, find_agent_content_matches,
};

fn message(
    id: &str,
    content: &str,
    role: AgentContentRole,
    timestamp_ms: Option<f64>,
) -> SearchableTranscriptEntry {
    SearchableTranscriptEntry {
        id: id.into(),
        kind: SearchableTranscriptEntryKind::Message {
            content: content.into(),
            role,
        },
        timestamp_ms,
        hidden_outbound_agent_peer_message: false,
    }
}

#[test]
fn frozen_limits_and_entry_text_projection_match_grok() {
    assert_eq!(AGENT_CONTENT_SEARCH_MAX_MATCHES_PER_AGENT, 5);
    assert_eq!(AGENT_CONTENT_SEARCH_MAX_RESULTS, 50);

    let user = message("u", "hello", AgentContentRole::User, None);
    assert_eq!(entry_search_text(&user), "hello");

    let send = SearchableTranscriptEntry {
        id: "s".into(),
        kind: SearchableTranscriptEntryKind::SendMessage {
            message_type: "text".into(),
            content: Some("delivered".into()),
        },
        timestamp_ms: None,
        hidden_outbound_agent_peer_message: false,
    };
    assert_eq!(entry_search_text(&send), "delivered");

    let image = SearchableTranscriptEntry {
        kind: SearchableTranscriptEntryKind::SendMessage {
            message_type: "image".into(),
            content: Some("must not be searchable".into()),
        },
        ..send.clone()
    };
    assert_eq!(entry_search_text(&image), "");

    let notice = SearchableTranscriptEntry {
        id: "n".into(),
        kind: SearchableTranscriptEntryKind::Notice {
            text: "notice text".into(),
        },
        timestamp_ms: None,
        hidden_outbound_agent_peer_message: false,
    };
    assert_eq!(entry_search_text(&notice), "notice text");
}

#[test]
fn snippet_collapses_whitespace_preserves_case_and_applies_frozen_window() {
    assert_eq!(
        build_content_snippet("  Alpha\n\tNeedle   tail  ", "needle").as_deref(),
        Some("Alpha Needle tail"),
    );
    assert_eq!(build_content_snippet("Alpha", ""), None);
    assert_eq!(build_content_snippet("Alpha", "missing"), None);

    let long = format!("{}Needle{}", "a".repeat(35), "b".repeat(70));
    let snippet = build_content_snippet(&long, "needle").expect("match");
    assert_eq!(
        snippet,
        format!("…{}Needle{}…", "a".repeat(30), "b".repeat(60)),
    );
}

#[test]
fn unicode_snippet_maps_expanding_and_contracting_lowercase_to_original_text() {
    for prefix in ["İ", "K"] {
        let text = format!("{}Needle", prefix.repeat(90));
        assert_eq!(
            build_content_snippet(&text, "needle"),
            Some(format!("…{}Needle", prefix.repeat(30))),
            "the typed snippet retains thirty original characters before the match",
        );
    }

    for query in ["i", "\u{307}", "i\u{307}"] {
        assert_eq!(build_content_snippet("İ", query).as_deref(), Some("İ"));
    }
    let across_expansion = format!("{}İY{}", "界".repeat(35), "🙂".repeat(70));
    assert_eq!(
        build_content_snippet(&across_expansion, "\u{307}y"),
        Some(format!("…{}İY{}…", "界".repeat(30), "🙂".repeat(60))),
        "a match beginning inside İ's lowercase expansion includes all of İ",
    );
}

#[test]
fn unicode_snippet_preserves_whole_string_contextual_sigma_matching() {
    let text = "  ΟΣ\nΟΣΑ\tΣ  ";
    for query in ["ος", "οσα", "σ"] {
        assert_eq!(
            build_content_snippet(text, query).as_deref(),
            Some("ΟΣ ΟΣΑ Σ"),
        );
    }
    assert_eq!(build_content_snippet(text, "οσ "), None);
    assert_eq!(build_content_snippet("Σ", "ς"), None);
    assert_eq!(build_content_snippet("Σ", "σ").as_deref(), Some("Σ"));
    assert_eq!(
        build_content_snippet("Kİ ΟΣ\u{301}!", "ος\u{301}").as_deref(),
        Some("Kİ ΟΣ\u{301}!"),
        "case-ignorable marks retain the whole-string final-sigma context",
    );
}

#[test]
fn unicode_snippet_keeps_character_windows_and_whitespace_contract() {
    let text = format!("{}Needle{}", "🙂".repeat(35), "界".repeat(70));
    assert_eq!(
        build_content_snippet(&text, "needle"),
        Some(format!("…{}Needle{}…", "🙂".repeat(30), "界".repeat(60))),
    );
    assert_eq!(
        build_content_snippet("\u{2003}東京\n\u{a0}Needle\t終\u{2003}", "needle")
            .as_deref(),
        Some("東京 Needle 終"),
    );
    assert_eq!(build_content_snippet("", "needle"), None);
    assert_eq!(build_content_snippet(" \n\t\u{2003}", "needle"), None);
    assert_eq!(build_content_snippet("İ", ""), None);
    assert_eq!(build_content_snippet("İ", "missing"), None);
}

#[test]
fn matches_walk_newest_first_skip_hidden_peers_and_preserve_roles() {
    let mut hidden = message(
        "hidden",
        "needle secret",
        AgentContentRole::User,
        Some(30.0),
    );
    hidden.hidden_outbound_agent_peer_message = true;

    let notice = SearchableTranscriptEntry {
        id: "notice".into(),
        kind: SearchableTranscriptEntryKind::Notice {
            text: "needle notice".into(),
        },
        timestamp_ms: None,
        hidden_outbound_agent_peer_message: false,
    };

    let entries = vec![
        message("old", "needle old", AgentContentRole::User, Some(10.0)),
        message(
            "middle",
            "needle middle",
            AgentContentRole::Assistant,
            Some(20.0),
        ),
        hidden,
        notice,
    ];
    let matches = find_agent_content_matches(&entries, "needle", 2);

    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].entry_id, "notice");
    assert_eq!(matches[0].role, AgentContentRole::Assistant);
    assert_eq!(matches[0].timestamp_ms, 0.0);
    assert_eq!(matches[1].entry_id, "middle");
    assert_eq!(matches[1].role, AgentContentRole::Assistant);
}

#[test]
fn zero_limit_and_empty_query_return_no_matches() {
    let entries = vec![message(
        "a",
        "needle",
        AgentContentRole::User,
        Some(1.0),
    )];
    assert!(find_agent_content_matches(&entries, "", 5).is_empty());
    assert!(find_agent_content_matches(&entries, "needle", 0).is_empty());
}
