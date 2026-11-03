use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::memory::memory_service::{
    FileMemoryStore, MemoryKind, format_memory_date,
};
use mahayana_host_runtime::extensions::session::agent_db_serde::EpisodeTurn;
use mahayana_host_runtime::extensions::session::production::{
    ProductionMaterializedSession, ProductionSessionWorkers,
};
use mahayana_host_runtime::runner::sand_memory::{
    MEMORY_EPISODE_PREFIX, MEMORY_EPISODE_PROMPT_MARKER,
    MEMORY_EXTRACTION_PROMPT_MARKER,
};
use mahayana_host_runtime::runner::turn_memory::{
    EpisodeProgress, ProductionEpisodeProgress, TurnExchange, TurnMemoryMode,
    TurnMemoryReport, build_turn_memory_exchange, run_turn_memory_with,
};

fn root(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-turn-memory-{label}-{}-{nonce}",
        std::process::id()
    ))
}

#[test]
fn completed_memorable_turns_extract_memory_and_roll_episode_summary() {
    let root = root("extract");
    let workers = ProductionSessionWorkers::with_agents_root(&root, 500);
    let session = workers
        .materialize_session_with_active(None, "user", None, None)
        .expect("session");
    let calls = Arc::new(Mutex::new(Vec::<String>::new()));

    for index in 0..6 {
        let calls_for_turn = Arc::clone(&calls);
        let report = run_turn_memory_with(
            &session.memory,
            Some(session.db.as_ref()),
            1_900_000_000_000 + index,
            TurnExchange {
                user: format!("Continue the release plan with decision {index}."),
                agent: format!("Recorded decision {index}."),
            },
            TurnMemoryMode::Extract,
            move |system, _user| {
                calls_for_turn.lock().unwrap().push(system.to_string());
                if system.contains(MEMORY_EXTRACTION_PROMPT_MARKER) {
                    Ok("profile: User prefers concise release updates".into())
                } else if system.contains(MEMORY_EPISODE_PROMPT_MARKER) {
                    Ok("On 2030-03-17 the user and assistant advanced the release plan.".into())
                } else {
                    Err("unexpected prompt".into())
                }
            },
        );
        assert!(report.extraction_attempted);
        assert!(report.episode_recorded);
    }

    let memories = session.memory.list_memories(100);
    assert!(memories.iter().any(|memory| {
        memory.content == "User prefers concise release updates"
    }));
    assert!(memories.iter().any(|memory| {
        memory.content.starts_with(MEMORY_EPISODE_PREFIX)
    }));
    assert!(session.db.get_pending_episode_turns().unwrap().is_empty());
    assert_eq!(
        calls.lock().unwrap()
            .iter()
            .filter(|prompt| prompt.contains(MEMORY_EPISODE_PROMPT_MARKER))
            .count(),
        1
    );

    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn evidence_mode_clears_pending_episode_state_without_running_extraction() {
    let root = root("evidence");
    let workers = ProductionSessionWorkers::with_agents_root(&root, 500);
    let session = workers
        .materialize_session_with_active(None, "user", None, None)
        .expect("session");
    session
        .db
        .record_episode_turn(&EpisodeTurn {
            ts: 1.0,
            user: "old user".into(),
            agent: "old agent".into(),
        })
        .unwrap();

    let seen = Arc::new(Mutex::new(Vec::<TurnExchange>::new()));
    let seen_recorder = Arc::clone(&seen);
    let mut recorder = move |exchange: &TurnExchange| {
        seen_recorder.lock().unwrap().push(exchange.clone());
    };
    let report = run_turn_memory_with(
        &session.memory,
        Some(session.db.as_ref()),
        2,
        TurnExchange {
            user: "new user".into(),
            agent: "new agent".into(),
        },
        TurnMemoryMode::RecordEvidence(&mut recorder),
        |_system, _user| -> Result<String, String> {
            panic!("evidence mode must not run extraction")
        },
    );

    assert!(report.evidence_recorded);
    assert!(!report.extraction_attempted);
    assert!(session.db.get_pending_episode_turns().unwrap().is_empty());
    assert_eq!(seen.lock().unwrap().len(), 1);

    workers.shutdown();
    let _ = fs::remove_dir_all(root);
}


#[test]
fn frozen_turn_memory_exchange_keeps_agent_messages_before_final_assistant_text() {
    let exchange = build_turn_memory_exchange(
        "user prompt",
        ["sent to peer", "   ", "second peer note"],
        "final assistant reply",
    );
    assert_eq!(exchange.user, "user prompt");
    assert_eq!(
        exchange.agent,
        "sent to peer\nsecond peer note\nfinal assistant reply"
    );

    let tool_only = build_turn_memory_exchange(
        "tool-only prompt",
        ["delegated work"],
        "   ",
    );
    assert_eq!(tool_only.agent, "delegated work");
}

const EPISODE_CHILD_CASE: &str = "FABUSHI_EPISODE_APPEND_CONTRACT_CASE";
const EPISODE_CHILD_ROOT: &str = "FABUSHI_EPISODE_APPEND_CONTRACT_ROOT";

struct OwnedEpisodeChild {
    child: Child,
    root: PathBuf,
    completed: bool,
}

impl Drop for OwnedEpisodeChild {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        if self.completed {
            let _ = fs::remove_dir_all(&self.root);
        } else {
            eprintln!("retained episode fixture: {}", self.root.display());
        }
    }
}

fn isolated_episode_case(test_name: &str, run: impl FnOnce(&Path)) {
    if std::env::var(EPISODE_CHILD_CASE).ok().as_deref() == Some(test_name) {
        assert_eq!(std::env::var("SAND_MEMORY_EPISODE_INTERVAL").unwrap(), "2");
        let root = PathBuf::from(std::env::var_os(EPISODE_CHILD_ROOT).unwrap());
        run(&root);
        fs::write(root.join("completed"), test_name).expect("complete real episode assertions");
        return;
    }

    // Pin the interval only in this exact child. Captured child output keeps
    // the parent test name unique in the Actions positive-test receipts.
    let child_root = root("episode-append-child");
    fs::create_dir(&child_root).expect("exclusively own episode fixture");
    let stdout = fs::File::create(child_root.join("stdout")).unwrap();
    let stderr = fs::File::create(child_root.join("stderr")).unwrap();
    let child = Command::new(std::env::current_exe().expect("test executable"))
        .args(["--exact", test_name, "--nocapture", "--test-threads=1"])
        .env(EPISODE_CHILD_CASE, test_name)
        .env(EPISODE_CHILD_ROOT, &child_root)
        .env("SAND_MEMORY_EPISODE_INTERVAL", "2")
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .expect("start isolated episode case");
    let mut owned = OwnedEpisodeChild { child, root: child_root, completed: false };
    let deadline = Instant::now() + Duration::from_secs(45);
    let status = loop {
        if let Some(status) = owned.child.try_wait().expect("observe owned episode child") {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = owned.child.kill();
            let _ = owned.child.wait();
            panic!(
                "episode case timed out: {test_name}\nstdout:\n{}\nstderr:\n{}",
                fs::read_to_string(owned.root.join("stdout")).unwrap(),
                fs::read_to_string(owned.root.join("stderr")).unwrap(),
            );
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let stdout = fs::read_to_string(owned.root.join("stdout")).unwrap();
    let stderr = fs::read_to_string(owned.root.join("stderr")).unwrap();
    assert!(status.success(), "episode case failed: {test_name}: {status}\n{stdout}\n{stderr}");
    assert!(
        stdout.contains("test result: ok. 1 passed; 0 failed; 0 ignored;"),
        "the exact child case must pass: {stdout}\n{stderr}",
    );
    assert_eq!(
        fs::read_to_string(owned.root.join("completed")).expect("episode completion receipt"),
        test_name,
        "zero selected tests cannot establish the episode assertions",
    );
    owned.completed = true;
}

fn episode_session(
    root: &Path,
    existing_agent_id: Option<&str>,
) -> (Arc<ProductionSessionWorkers>, ProductionMaterializedSession, ProductionEpisodeProgress) {
    let workers = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
    let session = match existing_agent_id {
        Some(agent_id) => workers
            .open_materialized_session(agent_id)
            .expect("reopen actual Session storage")
            .expect("issued Session still exists"),
        None => workers
            .materialize_session_with_active(None, "user", None, None)
            .expect("materialize actual Session"),
    };
    let progress = ProductionEpisodeProgress::new(Arc::clone(&workers), &session.record.id);
    (workers, session, progress)
}

fn episode_append_time() -> i64 {
    chrono::DateTime::parse_from_rfc3339("2026-09-24T00:00:00Z")
        .expect("ordinary episode date")
        .timestamp_millis()
}

fn episode_log_path(store: &FileMemoryStore, timestamp: i64) -> PathBuf {
    let month = format_memory_date(timestamp).chars().take(7).collect::<String>();
    store.get_location().join("log").join(format!("{month}.md"))
}

fn episode_turn(timestamp: i64, user: &str, agent: &str) -> EpisodeTurn {
    EpisodeTurn { ts: timestamp as f64, user: user.into(), agent: agent.into() }
}

fn run_completed_episode_turn(
    session: &ProductionMaterializedSession,
    progress: &ProductionEpisodeProgress,
    turn: &EpisodeTurn,
    expected_pending: &[EpisodeTurn],
    summary: &str,
    on_summary: impl FnMut(),
) -> TurnMemoryReport {
    run_completed_episode_turn_with_result(
        session, progress, turn, expected_pending, Ok(summary), on_summary,
    )
}

fn run_completed_episode_turn_with_result(
    session: &ProductionMaterializedSession,
    progress: &ProductionEpisodeProgress,
    turn: &EpisodeTurn,
    expected_pending: &[EpisodeTurn],
    summary: Result<&str, &str>,
    mut on_summary: impl FnMut(),
) -> TurnMemoryReport {
    let mut extraction_calls = 0;
    let mut summary_calls = 0;
    let report = run_turn_memory_with(
        &session.memory,
        Some(progress),
        turn.ts as i64,
        TurnExchange { user: turn.user.clone(), agent: turn.agent.clone() },
        TurnMemoryMode::Extract,
        |system, user| {
            if system.contains(MEMORY_EXTRACTION_PROMPT_MARKER) {
                extraction_calls += 1;
                Ok("NONE".into())
            } else {
                assert!(system.contains(MEMORY_EPISODE_PROMPT_MARKER));
                summary_calls += 1;
                assert_eq!(
                    progress.get_pending_episode_turns().unwrap().as_slice(),
                    expected_pending,
                    "the actual Session must own every input before model completion",
                );
                // Inspect the model input independently of the prompt builder:
                // every complete user/agent value must appear in turn order.
                let mut remaining = user;
                for pending in expected_pending {
                    remaining = remaining.split_once(pending.user.as_str())
                        .expect("pending user text in episode input").1;
                    remaining = remaining.split_once(pending.agent.as_str())
                        .expect("pending agent text in episode input").1;
                }
                on_summary();
                summary.map(str::to_owned).map_err(str::to_owned)
            }
        },
    );
    assert_eq!(extraction_calls, 1);
    assert_eq!(summary_calls, 1);
    report
}

fn assert_episode_report(report: TurnMemoryReport, added: bool) {
    assert_eq!(report, TurnMemoryReport {
        extraction_attempted: true,
        episode_recorded: true,
        episode_summary_attempted: true,
        episode_memory_added: added,
        ..TurnMemoryReport::default()
    });
}

fn assert_repaired_episode(
    session: &ProductionMaterializedSession,
    progress: &ProductionEpisodeProgress,
    before: &[u8],
    timestamp: i64,
    summary: &str,
    history_id: &str,
) {
    let content = format!("{MEMORY_EPISODE_PREFIX}{summary}");
    let mut expected = before.to_vec();
    expected.extend_from_slice(format!("- ({}) {content}\n", format_memory_date(timestamp)).as_bytes());
    assert_eq!(fs::read(episode_log_path(&session.memory, timestamp)).unwrap(), expected);
    let memories = session.memory.list_memories(100);
    assert_eq!(memories.len(), 2);
    assert!(memories.iter().any(|memory| memory.id == history_id));
    assert_eq!(memories.iter().filter(|memory| memory.content == content).count(), 1);
    assert!(progress.get_pending_episode_turns().unwrap().is_empty());
}

#[test]
fn episode_append_read_failure_preserves_pending_turns_across_restart() {
    isolated_episode_case(
        "episode_append_read_failure_preserves_pending_turns_across_restart",
        |root| {
            let (workers, session, progress) = episode_session(root, None);
            let agent_id = session.record.id.clone();
            let now = episode_append_time();
            let history = session.memory.add_memory(
                "Retain the signed release checkpoint.", now - 86_400_000, MemoryKind::Log,
            ).unwrap().expect("readable history");
            let destination = episode_log_path(&session.memory, now);
            let repaired = fs::read(&destination).unwrap();
            let mut corrupt = repaired.clone();
            corrupt.extend_from_slice(b"<!-- existing history: \xff\xfe -->\n");
            fs::write(&destination, &corrupt).expect("fixture-owned unreadable destination");
            assert_eq!(fs::read(&destination).unwrap(), corrupt);
            assert_eq!(
                fs::read_to_string(&destination).expect_err("invalid UTF-8 precondition").kind(),
                io::ErrorKind::InvalidData,
            );

            let first = episode_turn(now - 1_000, "Keep the migration checkpoint.", "The checkpoint is retained.");
            let current = episode_turn(now, "Audit the résumé export.", "The résumé export needs a final review.");
            assert!(progress.record_episode_turn(&first).unwrap());
            let mut pending = vec![first, current.clone()];
            let summary = "The user and Fabushi audited the release checkpoint and résumé export.";
            let report = run_completed_episode_turn(&session, &progress, &current, &pending, summary, || {});
            assert_episode_report(report, false);
            assert_eq!(progress.get_pending_episode_turns().unwrap(), pending);
            assert_eq!(fs::read(&destination).unwrap(), corrupt);

            workers.shutdown();
            drop(progress);
            drop(session);
            drop(workers);
            let (workers, session, progress) = episode_session(root, Some(&agent_id));
            assert_eq!(progress.get_pending_episode_turns().unwrap(), pending);
            assert_eq!(fs::read(&destination).unwrap(), corrupt);
            assert_eq!(
                fs::read_to_string(&destination).expect_err("restart must not repair history").kind(),
                io::ErrorKind::InvalidData,
            );

            fs::write(&destination, &repaired).expect("explicit fixture-owned repair");
            let next = episode_turn(now + 1_000, "Continue the audit after repair.", "The prior checkpoint remains in scope.");
            pending.push(next.clone());
            let report = run_completed_episode_turn(&session, &progress, &next, &pending, summary, || {});
            assert_episode_report(report, true);
            assert_repaired_episode(&session, &progress, &repaired, now + 1_000, summary, &history.id);
            workers.shutdown();
        },
    );
}

#[test]
fn episode_append_write_failure_preserves_interleaved_turns_and_recovers() {
    isolated_episode_case(
        "episode_append_write_failure_preserves_interleaved_turns_and_recovers",
        |root| {
            let (workers, session, progress) = episode_session(root, None);
            let agent_id = session.record.id.clone();
            let now = episode_append_time();
            let history = session.memory.add_memory(
                "Preserve the deployment rollback plan.", now - 86_400_000, MemoryKind::Log,
            ).unwrap().expect("readable history");
            let destination = episode_log_path(&session.memory, now);
            let before = fs::read(&destination).unwrap();
            assert_eq!(fs::read_to_string(&destination).unwrap().as_bytes(), before);
            // WatchedDirectory's real staging write must fail after read admission.
            let temporary = PathBuf::from(format!("{}.{}.tmp", destination.display(), std::process::id()));
            fs::create_dir(&temporary).expect("fixture owns exact atomic-write obstacle");
            let sentinel = temporary.join("owned-sentinel");
            let sentinel_bytes = b"only the fixture may remove this obstacle";
            fs::write(&sentinel, sentinel_bytes).unwrap();

            let first = episode_turn(now - 1_000, "Keep the rollback plan.", "The rollback plan is ready.");
            let current = episode_turn(now, "Check the deployment window.", "The deployment window is recorded.");
            let interleaved = episode_turn(now + 1_000, "Also retain the regional freeze.", "The regional freeze blocks deployment.");
            assert!(progress.record_episode_turn(&first).unwrap());
            let mut pending = vec![first, current.clone()];
            let summary = "The user and Fabushi planned deployment around the rollback plan and regional freeze.";
            let report = run_completed_episode_turn(&session, &progress, &current, &pending, summary, || {
                assert!(workers.record_agent_episode_turn(&agent_id, &interleaved).unwrap());
            });
            pending.push(interleaved);
            assert_episode_report(report, false);
            assert_eq!(progress.get_pending_episode_turns().unwrap(), pending);
            assert_eq!(fs::read(&destination).unwrap(), before);
            assert!(fs::symlink_metadata(&temporary).unwrap().is_dir());
            assert_eq!(fs::read(&sentinel).unwrap(), sentinel_bytes);

            workers.shutdown();
            drop(progress);
            drop(session);
            drop(workers);
            let (workers, session, progress) = episode_session(root, Some(&agent_id));
            assert_eq!(progress.get_pending_episode_turns().unwrap(), pending);
            assert_eq!(fs::read(&destination).unwrap(), before);
            assert_eq!(fs::read_to_string(&destination).unwrap().as_bytes(), before);
            assert!(fs::symlink_metadata(&temporary).unwrap().is_dir());
            assert_eq!(fs::read(&sentinel).unwrap(), sentinel_bytes);

            fs::remove_file(&sentinel).expect("fixture removes its sentinel");
            fs::remove_dir(&temporary).expect("explicit fixture-owned write-path repair");
            let next = episode_turn(now + 2_000, "Resume deployment planning.", "The regional freeze remains part of the plan.");
            pending.push(next.clone());
            let report = run_completed_episode_turn(&session, &progress, &next, &pending, summary, || {});
            assert_episode_report(report, true);
            assert_repaired_episode(&session, &progress, &before, now + 2_000, summary, &history.id);
            assert!(!temporary.exists());
            workers.shutdown();
        },
    );
}

#[test]
fn episode_duplicate_append_acknowledges_existing_fact_before_clearing_pending() {
    isolated_episode_case(
        "episode_duplicate_append_acknowledges_existing_fact_before_clearing_pending",
        |root| {
            let (workers, session, progress) = episode_session(root, None);
            let agent_id = session.record.id.clone();
            let now = episode_append_time();
            let summary = "The user and Fabushi confirmed the existing release checkpoint.";
            let existing = session.memory.add_memory(
                &format!("{MEMORY_EPISODE_PREFIX}{summary}"), now - 86_400_000, MemoryKind::Log,
            ).unwrap().expect("seed one real episode fact");
            let destination = episode_log_path(&session.memory, now);
            let before = fs::read(&destination).unwrap();
            let first = episode_turn(now - 1_000, "Review the existing checkpoint.", "The existing checkpoint still applies.");
            let current = episode_turn(now, "Confirm the same checkpoint.", "The checkpoint is unchanged.");
            assert!(progress.record_episode_turn(&first).unwrap());
            let pending = vec![first, current.clone()];
            let report = run_completed_episode_turn(&session, &progress, &current, &pending, summary, || {});
            assert_episode_report(report, false);
            assert!(progress.get_pending_episode_turns().unwrap().is_empty());
            assert_eq!(fs::read(&destination).unwrap(), before);
            assert_eq!(session.memory.list_memories(100), vec![existing.clone()]);

            workers.shutdown();
            drop(progress);
            drop(session);
            drop(workers);
            let (workers, session, progress) = episode_session(root, Some(&agent_id));
            assert!(progress.get_pending_episode_turns().unwrap().is_empty());
            assert_eq!(fs::read(&destination).unwrap(), before);
            assert_eq!(session.memory.list_memories(100), vec![existing]);
            workers.shutdown();
        },
    );
}

#[test]
fn episode_summary_failure_preserves_pending_turns_and_recovers() {
    isolated_episode_case(
        "episode_summary_failure_preserves_pending_turns_and_recovers",
        |root| {
            // These are controlled model results at the real consumer boundary,
            // not live provider failures or cancellation/scheduler timing proof.
            for (label, error) in [
                ("transport", "transport error: connection reset during episode summary"),
                ("cancelled", "cancelled: episode summary request was cancelled"),
            ] {
                let case_root = root.join(label);
                let (workers, session, progress) = episode_session(&case_root, None);
                let agent_id = session.record.id.clone();
                let now = episode_append_time();
                let history = session.memory.add_memory(
                    "Preserve the reviewed release history.", now - 86_400_000, MemoryKind::Log,
                ).unwrap().expect("readable history before summary failure");
                let destination = episode_log_path(&session.memory, now);
                let before = fs::read(&destination).unwrap();
                assert_eq!(fs::read_to_string(&destination).unwrap().as_bytes(), before);

                let first = episode_turn(now - 1_000, "Keep the approved release checklist.", "The approved checklist remains available.");
                let current = episode_turn(now, "Review the résumé export.\nKeep its original title.", "The résumé export still needs review.\nIts title is preserved.");
                let interleaved = episode_turn(now + 1_000, "Also retain the regional release freeze.", "The regional freeze remains in force.");
                assert!(progress.record_episode_turn(&first).unwrap());
                let mut pending = vec![first, current.clone()];
                let report = run_completed_episode_turn_with_result(
                    &session, &progress, &current, &pending, Err(error), || {
                        assert!(workers.record_agent_episode_turn(&agent_id, &interleaved).unwrap());
                    },
                );
                pending.push(interleaved);
                assert_episode_report(report, false);
                assert_eq!(progress.get_pending_episode_turns().unwrap(), pending, "{label}");
                assert_eq!(fs::read(&destination).unwrap(), before, "{label}");
                assert_eq!(session.memory.list_memories(100), vec![history.clone()], "{label}");

                workers.shutdown();
                drop(progress);
                drop(session);
                drop(workers);
                let (workers, session, progress) = episode_session(&case_root, Some(&agent_id));
                assert_eq!(progress.get_pending_episode_turns().unwrap(), pending, "{label} after reopen");
                assert_eq!(fs::read(&destination).unwrap(), before, "{label} after reopen");
                assert_eq!(session.memory.list_memories(100), vec![history.clone()], "{label} after reopen");

                let next = episode_turn(now + 2_000, "Continue the release review.", "The checklist, résumé export and regional freeze remain in scope.");
                pending.push(next.clone());
                let summary = "The user and Fabushi reviewed the release checklist, résumé export and regional freeze.";
                let report = run_completed_episode_turn(&session, &progress, &next, &pending, summary, || {});
                assert_episode_report(report, true);
                assert_repaired_episode(&session, &progress, &before, now + 2_000, summary, &history.id);

                workers.shutdown();
                drop(progress);
                drop(session);
                drop(workers);
                let (workers, session, progress) = episode_session(&case_root, Some(&agent_id));
                assert_repaired_episode(&session, &progress, &before, now + 2_000, summary, &history.id);
                workers.shutdown();
            }
        },
    );
}

#[test]
fn episode_successful_no_summary_reply_keeps_existing_settlement() {
    isolated_episode_case(
        "episode_successful_no_summary_reply_keeps_existing_settlement",
        |root| {
            for (label, summary) in [
                ("empty", ""),
                ("whitespace", " \t\r\n "),
                ("upper_none", "NONE"),
                ("lower_none", "none"),
                ("mixed_none", " \tNoNe\r\n "),
            ] {
                let case_root = root.join(label);
                let (workers, session, progress) = episode_session(&case_root, None);
                let agent_id = session.record.id.clone();
                let now = episode_append_time();
                let history = session.memory.add_memory(
                    "Retain the existing release history without another episode.", now - 86_400_000, MemoryKind::Log,
                ).unwrap().expect("readable history before no-summary settlement");
                let destination = episode_log_path(&session.memory, now);
                let before = fs::read(&destination).unwrap();
                let first = episode_turn(now - 1_000, "Keep the earlier release note.", "The earlier release note is unchanged.");
                let current = episode_turn(now, "No new release decision is needed.", "The existing release decision still applies.");
                assert!(progress.record_episode_turn(&first).unwrap());
                let pending = vec![first, current.clone()];
                let report = run_completed_episode_turn(&session, &progress, &current, &pending, summary, || {});
                assert_episode_report(report, false);
                assert!(progress.get_pending_episode_turns().unwrap().is_empty(), "{label}");
                assert_eq!(fs::read(&destination).unwrap(), before, "{label}");
                assert_eq!(session.memory.list_memories(100), vec![history.clone()], "{label}");

                workers.shutdown();
                drop(progress);
                drop(session);
                drop(workers);
                let (workers, session, progress) = episode_session(&case_root, Some(&agent_id));
                assert!(progress.get_pending_episode_turns().unwrap().is_empty(), "{label} after reopen");
                assert_eq!(fs::read(&destination).unwrap(), before, "{label} after reopen");
                assert_eq!(session.memory.list_memories(100), vec![history], "{label} after reopen");
                workers.shutdown();
            }
        },
    );
}

#[test]
fn episode_record_refusal_preserves_archived_pending_until_restore() {
    isolated_episode_case(
        "episode_record_refusal_preserves_archived_pending_until_restore",
        |root| {
            // This composes the real public owners; it is not a Host/UI archive race.
            let (workers, session, progress) = episode_session(root, None);
            let agent_id = session.record.id.clone();
            let now = episode_append_time();
            let history = session.memory.add_memory(
                "Preserve the reviewed release history while the session is archived.",
                now - 86_400_000, MemoryKind::Log,
            ).unwrap().expect("readable history before archive");
            let destination = episode_log_path(&session.memory, now);
            let before = fs::read(&destination).unwrap();
            let pending = vec![
                episode_turn(
                    now - 2_000,
                    "Keep the résumé export title.\nRetain the regional freeze.",
                    "The résumé title is fixed.\nThe regional freeze remains active.",
                ),
                episode_turn(
                    now - 1_000,
                    "Preserve the signed release checklist.",
                    "The signed release checklist is retained.",
                ),
            ];
            for turn in &pending {
                assert!(progress.record_episode_turn(turn).unwrap());
            }
            assert_eq!(progress.get_pending_episode_turns().unwrap(), pending);
            let admission = workers.read_session_work_admission(&agent_id).unwrap().unwrap();
            let archived = workers.transition_session_work_archive(
                &agent_id, &admission.session_key, &admission.session_id,
                Some(&admission.lifecycle_revision), true, Some("fixture-user"),
                Some("archive before episode admission"), now as f64,
            ).expect("archive through the production Session transaction");
            assert_eq!(archived.archived_at_ms, Some(now as f64));
            assert_ne!(archived.lifecycle_revision, admission.lifecycle_revision);
            assert_eq!(workers.read_session_work_admission(&agent_id).unwrap(), Some(archived.clone()));

            let rejected = episode_turn(
                now + 1_000,
                "MER rejected request: publish the unapproved pricing change.",
                "MER rejected reply: the unapproved pricing change was published.",
            );
            assert_eq!(progress.record_episode_turn(&rejected), Ok(false));
            assert_eq!(progress.get_pending_episode_turns().unwrap(), pending);
            assert_eq!(fs::read(&destination).unwrap(), before);
            assert_eq!(session.memory.list_memories(100), vec![history.clone()]);
            assert_eq!(workers.read_session_work_admission(&agent_id).unwrap(), Some(archived.clone()));

            let mut extraction_calls = 0;
            let mut summary_calls = 0;
            let report = run_turn_memory_with(
                &session.memory,
                Some(&progress),
                rejected.ts as i64,
                TurnExchange { user: rejected.user.clone(), agent: rejected.agent.clone() },
                TurnMemoryMode::Extract,
                |system, _user| {
                    if system.contains(MEMORY_EXTRACTION_PROMPT_MARKER) {
                        extraction_calls += 1;
                        Ok("NONE".into())
                    } else {
                        assert!(system.contains(MEMORY_EPISODE_PROMPT_MARKER));
                        summary_calls += 1;
                        // Let a false-admission regression reach the actual file append.
                        Ok("An archived episode incorrectly reached the memory file after record refusal.".into())
                    }
                },
            );
            assert_eq!(fs::read(&destination).unwrap(), before, "a refused record must not append an episode");
            assert_eq!(session.memory.list_memories(100), vec![history.clone()]);
            assert_eq!(progress.get_pending_episode_turns().unwrap(), pending);
            assert_eq!(workers.read_session_work_admission(&agent_id).unwrap(), Some(archived.clone()));
            assert_eq!(extraction_calls, 1);
            assert_eq!(summary_calls, 0);
            assert_eq!(report, TurnMemoryReport {
                extraction_attempted: true,
                ..TurnMemoryReport::default()
            });

            workers.shutdown();
            drop(progress);
            drop(session);
            drop(workers);
            let (workers, session, progress) = episode_session(root, Some(&agent_id));
            assert_eq!(progress.get_pending_episode_turns().unwrap(), pending);
            assert_eq!(fs::read(&destination).unwrap(), before);
            assert_eq!(session.memory.list_memories(100), vec![history.clone()]);
            let current_archive = workers.read_session_work_admission(&agent_id).unwrap().unwrap();
            assert_eq!(current_archive, archived);

            let restored = workers.transition_session_work_archive(
                &agent_id, &current_archive.session_key, &current_archive.session_id,
                Some(&current_archive.lifecycle_revision), false, None, None,
                (now + 2_000) as f64,
            ).expect("restore using the current production Session revision");
            assert!(restored.archived_at_ms.is_none());
            assert_ne!(restored.lifecycle_revision, current_archive.lifecycle_revision);
            assert_eq!(workers.read_session_work_admission(&agent_id).unwrap(), Some(restored.clone()));
            let next = episode_turn(
                now + 3_000,
                "Resume the release review after restoring the session.",
                "The résumé export, regional freeze and signed checklist remain in scope.",
            );
            let mut accepted_pending = pending;
            accepted_pending.push(next.clone());
            let summary = "The user and Fabushi resumed the release review with the résumé export, regional freeze and signed checklist.";
            let mut extraction_calls = 0;
            let mut summary_calls = 0;
            let report = run_turn_memory_with(
                &session.memory,
                Some(&progress),
                next.ts as i64,
                TurnExchange { user: next.user.clone(), agent: next.agent.clone() },
                TurnMemoryMode::Extract,
                |system, user| {
                    if system.contains(MEMORY_EXTRACTION_PROMPT_MARKER) {
                        extraction_calls += 1;
                        Ok("NONE".into())
                    } else {
                        assert!(system.contains(MEMORY_EPISODE_PROMPT_MARKER));
                        summary_calls += 1;
                        assert_eq!(progress.get_pending_episode_turns().unwrap(), accepted_pending);
                        assert!(!user.contains(rejected.user.as_str()), "refused user input must not enter a later summary");
                        assert!(!user.contains(rejected.agent.as_str()), "refused agent input must not enter a later summary");
                        let mut remaining = user;
                        for turn in &accepted_pending {
                            remaining = remaining.split_once(turn.user.as_str())
                                .expect("complete admitted user text in turn order").1;
                            remaining = remaining.split_once(turn.agent.as_str())
                                .expect("complete admitted agent text in turn order").1;
                        }
                        Ok(summary.into())
                    }
                },
            );
            assert_eq!(extraction_calls, 1);
            assert_eq!(summary_calls, 1);
            assert_episode_report(report, true);
            assert_repaired_episode(&session, &progress, &before, now + 3_000, summary, &history.id);
            let settled_memories = session.memory.list_memories(100);
            assert_eq!(workers.read_session_work_admission(&agent_id).unwrap(), Some(restored.clone()));

            workers.shutdown();
            drop(progress);
            drop(session);
            drop(workers);
            let (workers, session, progress) = episode_session(root, Some(&agent_id));
            assert_repaired_episode(&session, &progress, &before, now + 3_000, summary, &history.id);
            assert_eq!(session.memory.list_memories(100), settled_memories);
            assert_eq!(workers.read_session_work_admission(&agent_id).unwrap(), Some(restored));
            workers.shutdown();
        },
    );
}
