use mahayana_host_runtime::extensions::inference::provider_session::ProviderMessage;
use mahayana_host_runtime::extensions::memory::memory_service::{
    MemoryKind, MemoryRecall, MemoryRecord,
};
use mahayana_host_runtime::runner::sand_memory::{
    MEMORY_EPISODE_PREFIX, MEMORY_NOTE_PREFIX, MemoryExtraction, build_episode_user_prompt,
    build_extraction_user_prompt, episode_interval, fact_line, gather_extraction_memories,
    is_memorable_exchange, memory_importance, parse_extracted_memories,
    render_memory_system_prompt, resolve_frozen_memory_prompt, select_relevant_memories,
};
use mahayana_host_runtime::runner::system_prompt_assembly::append_memory_system_prompt;

fn record(content: &str, created_at: i64, kind: MemoryKind) -> MemoryRecord {
    MemoryRecord {
        id: format!("id-{created_at}-{content}"),
        content: content.into(),
        created_at,
        kind,
    }
}

#[test]
fn frozen_memory_prompt_and_memorable_exchange_contracts_hold() {
    assert!(!is_memorable_exchange("hello!"));
    assert!(!is_memorable_exchange("thanks"));
    assert!(is_memorable_exchange("What city should I visit?"));
    assert!(is_memorable_exchange("I permanently moved to Portland"));

    assert_eq!(episode_interval(None), 6);
    assert_eq!(episode_interval(Some(" 9 ")), 9);
    assert_eq!(episode_interval(Some("0")), 6);

    let live = resolve_frozen_memory_prompt(None, 7, || ("facts".into(), true));
    assert_eq!(live.render, "facts");
    assert_eq!(live.snapshot_to_persist.expect("snapshot").compaction_epoch, 7);
}

#[test]
fn memory_prompt_rendering_matches_frozen_shape_and_shipping_message_merge() {
    let recall = MemoryRecall {
        profile: vec![record("The user's name is Ada", 1_700_000_000_000, MemoryKind::Profile)],
        recent: vec![record("Planning a Tokyo trip", 1_710_000_000_000, MemoryKind::Log)],
    };
    let rendered = render_memory_system_prompt(&recall, Some("/home/box/memory"));
    assert!(rendered.starts_with("Memory: durable facts you have learned"));
    assert!(rendered.contains("About the user:"));
    assert!(rendered.contains("Recently:"));
    assert!(rendered.contains("/home/box/memory"));
    assert!(fact_line(&recall.profile[0]).contains("The user's name is Ada"));

    let mut messages = vec![
        ProviderMessage { role: "system".into(), content: "base".into() },
        ProviderMessage { role: "user".into(), content: "hello".into() },
    ];
    append_memory_system_prompt(&mut messages, &recall, Some("/home/box/memory"));
    assert!(messages[0].content.starts_with("base\n\nMemory:"));
    let once = messages[0].content.clone();
    append_memory_system_prompt(&mut messages, &recall, Some("/home/box/memory"));
    assert_eq!(messages[0].content, once);
}

#[test]
fn extraction_parse_dedupe_relevance_and_episode_prompt_match_frozen_rules() {
    let existing = vec!["The user lives in London".to_string()];
    let parsed = parse_extracted_memories(
        "profile: The user lives in London\nlog: Planning Tokyo\nnote: Likes aisle seats\nremove: The user lives in London\n2. log: Planning Tokyo",
        &existing,
    );
    assert_eq!(
        parsed,
        MemoryExtraction {
            additions: vec![
                mahayana_host_runtime::runner::sand_memory::MemoryAddition {
                    content: "Planning Tokyo".into(),
                    kind: MemoryKind::Log,
                },
                mahayana_host_runtime::runner::sand_memory::MemoryAddition {
                    content: format!("{MEMORY_NOTE_PREFIX}Likes aisle seats"),
                    kind: MemoryKind::Log,
                },
            ],
            removals: vec!["The user lives in London".into()],
        }
    );

    let archive = vec![
        record("Tokyo hotel booking is pending", 30, MemoryKind::Log),
        record("Gardening project", 40, MemoryKind::Log),
    ];
    let selected = select_relevant_memories("Tokyo booking", &archive, 10);
    assert_eq!(selected.len(), 1);
    assert!(selected[0].content.contains("Tokyo"));

    let recall = MemoryRecall {
        profile: vec![record("Uses English", 50, MemoryKind::Profile)],
        recent: vec![],
    };
    let gathered = gather_extraction_memories(&recall, &archive, "Tokyo booking");
    assert_eq!(gathered, vec!["Uses English", "Tokyo hotel booking is pending"]);

    assert_eq!(memory_importance(&format!("{MEMORY_EPISODE_PREFIX}work")), 1.5);
    assert_eq!(memory_importance(&format!("{MEMORY_NOTE_PREFIX}minor")), 0.5);
    assert!(build_extraction_user_prompt(" hi ", "", &[]).contains("User: hi"));
    let episode = build_episode_user_prompt(&[
        mahayana_host_runtime::runner::sand_memory::EpisodeTurn {
            ts: 1_700_000_000_000,
            user: "question".into(),
            agent: "answer".into(),
        },
    ]);
    assert!(episode.contains("User: question"));
    assert!(episode.contains("Grok Bot: answer"));
}
