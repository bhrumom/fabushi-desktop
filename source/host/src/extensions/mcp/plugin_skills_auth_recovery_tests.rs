//! OCRM-PLUGIN-AUTH-RECOVERY-v1 oracles over the real service and file owner.
//!
//! The private image failure cuts exercise the actual staged file publication
//! before rename or after visible rename and before the directory barrier.
//! They are error instrumentation, not a claim to simulate physical power loss.
//! Separate owned-process tests cover process loss. All calls using these
//! thread-local cuts execute sync on this thread, never on Harness's worker.

use super::*;
use super::sync_tests::{
    BoundaryGate, Call, Harness, LoadStep, ServiceTestHooks, TempRoot,
    assert_skill_file_bytes, auth_block, record_ids, refresh_operation_id,
    refuse_nine_with_invalid_stage, skill_file_bytes, two_plugins,
};
use super::uninstall_state::{ImagePublishFailure, with_image_publish_failure};
use super::super::mcp_service::PluginSkillsPort;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicUsize, Ordering};
use serde_json::json;

const AUTH_ERROR: &str = "plugin skills authentication recovery state is unavailable";

// The existing channel fixture remains the concurrency oracle. This small
// inline loader is needed only to keep the storage failpoint on sync's thread.
struct InlineLoader {
    result: Mutex<Result<LoadedPlugins, String>>,
    calls: AtomicUsize,
}

impl PluginSkillsLoader for InlineLoader {
    fn load(&self) -> Result<LoadedPlugins, String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.result.lock().unwrap().clone()
    }
}

fn inline_service(
    root: &TempRoot,
    result: Result<LoadedPlugins, String>,
) -> (SandPluginSkillsService, Arc<InlineLoader>) {
    let loader = Arc::new(InlineLoader {
        result: Mutex::new(result),
        calls: AtomicUsize::new(0),
    });
    let service = SandPluginSkillsService::new(&root.0, loader.clone())
        .with_now(Arc::new(|| 1_000));
    (service, loader)
}

fn image(root: &TempRoot) -> Value {
    serde_json::from_slice(&fs::read(root.uninstall_state_path()).unwrap()).unwrap()
}

fn assert_image(root: &TempRoot, refusals: &[PluginAuthBlock], refresh: Option<&str>) {
    assert_eq!(image(root), json!({
        "formatVersion": 2,
        "confirmedPluginIds": [],
        "pending": [],
        "authRefusals": refusals,
        "refreshIntent": refresh.map(|operation_id| json!({"operationId": operation_id})),
    }));
    assert_eq!(fs::read(root.uninstall_version_path()).unwrap(), b"1\n");
}

fn assert_auth_recovery(
    service: &SandPluginSkillsService,
    refusals: &[PluginAuthBlock],
    unknown: bool,
) {
    let status = <SandPluginSkillsService as PluginSkillsPort>::status(service);
    let expected = if refusals.is_empty() && !unknown {
        None
    } else {
        Some(json!({"refusals": refusals, "refreshOutcomeUnknown": unknown}))
    };
    assert_eq!(status.get("authRecovery"), expected.as_ref());
}

fn seed(root: &TempRoot) -> (SandPluginSkillsService, Arc<InlineLoader>, Vec<PluginSkillRecord>) {
    let (service, loader) = inline_service(root, Ok(two_plugins(root, "retained")));
    let records = service.sync("seed").unwrap();
    assert_eq!(record_ids(&records), vec!["10", "9"]);
    assert_image(root, &[], None);
    assert_eq!(<SandPluginSkillsService as PluginSkillsPort>::status(&service), json!({"authBlocked": []}));
    (service, loader, records)
}

fn abandon_refresh(service: &SandPluginSkillsService, loader: &InlineLoader, root: &TempRoot) -> String {
    let calls = loader.calls.load(Ordering::SeqCst);
    assert_eq!(
        with_image_publish_failure(1, ImagePublishFailure::AfterRename, || service.sync("interrupted-reservation"))
            .unwrap_err(),
        AUTH_ERROR,
    );
    assert_eq!(loader.calls.load(Ordering::SeqCst), calls, "failed reservation never calls the loader");
    let operation_id = refresh_operation_id(root);
    let bytes = fs::read(root.uninstall_state_path()).unwrap();
    service.dispose();
    assert_eq!(fs::read(root.uninstall_state_path()).unwrap(), bytes, "dispose performs no journal I/O");
    operation_id
}

#[test]
fn auth_refresh_reservation_failure_never_calls_loader_or_changes_cache() {
    for candidate_visible in [false, true] {
        let root = TempRoot::new();
        let (service, loader, records) = seed(&root);
        let files = skill_file_bytes(&records);
        let cache = fs::read(root.cache_path()).unwrap();
        let journal = fs::read(root.uninstall_state_path()).unwrap();
        let sentinel = root.sentinel();
        let mut denied = two_plugins(&root, "must-not-load");
        denied.auth_blocked.push(auth_block("9"));
        *loader.result.lock().unwrap() = Ok(denied);
        let phase = if candidate_visible { ImagePublishFailure::AfterRename } else { ImagePublishFailure::BeforeRename };
        assert_eq!(with_image_publish_failure(1, phase, || service.sync("reservation-failure")).unwrap_err(), AUTH_ERROR);
        assert_eq!(loader.calls.load(Ordering::SeqCst), 1);
        assert_eq!(service.current(), records);
        assert!(!service.admission_reader().is_denied("9"));
        assert!(!service.admission_reader().is_denied("10"));
        assert_auth_recovery(&service, &[], false);
        assert_eq!(fs::read(root.cache_path()).unwrap(), cache);
        assert_skill_file_bytes(&files);
        assert!(sentinel.exists());
        assert!(root.staged_files().is_empty());
        if candidate_visible {
            let operation_id = refresh_operation_id(&root);
            assert_image(&root, &[], Some(&operation_id));
        } else {
            assert_eq!(fs::read(root.uninstall_state_path()).unwrap(), journal);
            assert_image(&root, &[], None);
        }
        let before_construction = fs::read(root.uninstall_state_path()).unwrap();
        let (fresh, offline) = inline_service(&root, Err("offline".into()));
        assert_eq!(offline.calls.load(Ordering::SeqCst), 0);
        assert_eq!(fs::read(root.uninstall_state_path()).unwrap(), before_construction);
        assert_eq!(fresh.admission_reader().is_denied("9"), candidate_visible);
        assert_eq!(fresh.admission_reader().is_denied("10"), candidate_visible);
        assert_auth_recovery(&fresh, &[], candidate_visible);
        assert_eq!(record_ids(&fresh.current()), if candidate_visible { vec![] } else { vec!["10", "9"] });
    }
}

#[test]
fn auth_negative_journal_failure_distinguishes_expected_and_candidate_recovery() {
    for candidate_visible in [false, true] {
        let root = TempRoot::new();
        let (service, loader, records) = seed(&root);
        let reader = service.admission_reader();
        let files = skill_file_bytes(&records);
        let cache = fs::read(root.cache_path()).unwrap();
        let sentinel = root.sentinel();
        let mut denied = two_plugins(&root, "must-not-publish");
        denied.auth_blocked.push(auth_block("9"));
        *loader.result.lock().unwrap() = Ok(denied);
        let phase = if candidate_visible { ImagePublishFailure::AfterRename } else { ImagePublishFailure::BeforeRename };
        // Publication 1 reserves custody; publication 2 imports the actual
        // negative outcome, before any cache stage exists.
        assert_eq!(with_image_publish_failure(2, phase, || service.sync("negative-image-failure")).unwrap_err(), AUTH_ERROR);
        assert_eq!(loader.calls.load(Ordering::SeqCst), 2);
        assert!(reader.is_denied("9"));
        assert!(!reader.is_denied("10"));
        assert_eq!(record_ids(&service.current()), vec!["10"]);
        assert!(service.current_auth_blocked().is_empty(), "old diagnostic cache remains unchanged");
        assert_auth_recovery(&service, &[auth_block("9")], false);
        assert_eq!(fs::read(root.cache_path()).unwrap(), cache);
        assert_skill_file_bytes(&files);
        assert!(sentinel.exists());
        assert!(root.staged_files().is_empty());
        if candidate_visible {
            assert_image(&root, &[auth_block("9")], None);
        } else {
            let operation_id = refresh_operation_id(&root);
            assert_image(&root, &[], Some(&operation_id));
        }
        let journal = fs::read(root.uninstall_state_path()).unwrap();
        let (fresh, offline) = inline_service(&root, Err("offline".into()));
        assert!(!Arc::ptr_eq(&reader, &fresh.admission_reader()));
        assert_eq!(offline.calls.load(Ordering::SeqCst), 0);
        assert_eq!(fs::read(root.uninstall_state_path()).unwrap(), journal);
        assert!(fresh.admission_reader().is_denied("9"));
        assert_eq!(fresh.admission_reader().is_denied("10"), !candidate_visible);
        let fresh_refusals = if candidate_visible { vec![auth_block("9")] } else { vec![] };
        assert_auth_recovery(&fresh, &fresh_refusals, !candidate_visible);
        assert_eq!(record_ids(&fresh.current()), if candidate_visible { vec!["10"] } else { vec![] });
        // A known returned error cannot erase either the stored refusal or an
        // inherited unknown. This is an explicit refresh, not startup replay.
        assert_eq!(fresh.sync("offline-after-negative-write-failure").unwrap_err(), "offline");
        assert_eq!(offline.calls.load(Ordering::SeqCst), 1);
        assert!(fresh.admission_reader().is_denied("9"));
        assert_eq!(fresh.admission_reader().is_denied("10"), !candidate_visible);
        assert_eq!(fs::read(root.cache_path()).unwrap(), cache);
        assert_skill_file_bytes(&files);
        assert!(sentinel.exists());
    }
}

#[test]
fn auth_live_negative_precedes_outcome_journal_while_fresh_reader_sees_unknown() {
    let gate = BoundaryGate::at_visit(2);
    let h = Harness::with_hooks(Arc::new(|| 1_000), ServiceTestHooks {
        before_auth_outcome: Some(Arc::clone(&gate.pause)),
        ..ServiceTestHooks::default()
    });
    let records = h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
    let cache = fs::read(h.root.cache_path()).unwrap();
    let files = skill_file_bytes(&records);
    let sentinel = h.root.sentinel();
    let denied = h.call("negative-before-journal");
    h.enter(2);
    h.release.send(LoadStep::Loaded(refuse_nine_with_invalid_stage(&h.root))).unwrap();
    gate.wait();
    let operation_id = refresh_operation_id(&h.root);
    assert_image(&h.root, &[], Some(&operation_id));
    assert!(h.service.admission_reader().is_denied("9"));
    assert!(!h.service.admission_reader().is_denied("10"));
    assert_eq!(record_ids(&h.service.current()), vec!["10"]);
    assert_auth_recovery(&h.service, &[auth_block("9")], false);
    assert!(matches!(h.service.publication.try_lock(), Err(std::sync::TryLockError::WouldBlock)));
    assert!(h.service.state.try_lock().is_ok());
    assert!(h.service.admission.state.try_lock().is_ok());
    let journal = fs::read(h.root.uninstall_state_path()).unwrap();
    let (fresh, offline) = inline_service(&h.root, Err("offline".into()));
    assert_eq!(offline.calls.load(Ordering::SeqCst), 0);
    assert!(fresh.current().is_empty());
    assert!(fresh.admission_reader().is_denied("9"));
    assert!(fresh.admission_reader().is_denied("10"));
    assert_auth_recovery(&fresh, &[], true);
    assert_eq!(fs::read(h.root.uninstall_state_path()).unwrap(), journal);
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), cache);
    gate.release.send(()).unwrap();
    assert_eq!(denied.finish().unwrap_err(), "invalid plugin skills cache snapshot");
    assert_image(&h.root, &[auth_block("9")], None);
    let (after, after_loader) = inline_service(&h.root, Err("offline".into()));
    assert_eq!(after_loader.calls.load(Ordering::SeqCst), 0);
    assert!(after.admission_reader().is_denied("9"));
    assert!(!after.admission_reader().is_denied("10"));
    assert_auth_recovery(&after, &[auth_block("9")], false);
    assert_eq!(record_ids(&after.current()), vec!["10"]);
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), cache);
    assert_skill_file_bytes(&files);
    assert!(sentinel.exists());
    assert!(h.root.staged_files().is_empty());
    h.assert_calls(2);
}

#[test]
fn auth_known_loader_errors_and_invalid_aggregates_preserve_offline_availability() {
    for had_refusal in [false, true] {
        for malformed in [false, true] {
            let root = TempRoot::new();
            let (service, loader, records) = seed(&root);
            let cache = fs::read(root.cache_path()).unwrap();
            let files = skill_file_bytes(&records);
            let sentinel = root.sentinel();
            let refusals = if had_refusal { vec![auth_block("9")] } else { vec![] };
            if had_refusal {
                *loader.result.lock().unwrap() = Ok(refuse_nine_with_invalid_stage(&root));
                assert_eq!(service.sync("accepted-negative").unwrap_err(), "invalid plugin skills cache snapshot");
            }
            let expected_error = if malformed {
                let mut invalid = two_plugins(&root, "invalid");
                invalid.listed_plugin_ids.push("9".into());
                invalid.auth_blocked.push(auth_block("10"));
                *loader.result.lock().unwrap() = Ok(invalid);
                "plugin skills loader returned a duplicate listed plugin ID"
            } else {
                *loader.result.lock().unwrap() = Err("offline".into());
                "offline"
            };
            let journal = fs::read(root.uninstall_state_path()).unwrap();
            assert_eq!(service.sync("known-non-admitted-outcome").unwrap_err(), expected_error);
            assert_eq!(fs::read(root.uninstall_state_path()).unwrap(), journal);
            assert_image(&root, &refusals, None);
            assert_auth_recovery(&service, &refusals, false);
            assert_eq!(service.admission_reader().is_denied("9"), had_refusal);
            assert!(!service.admission_reader().is_denied("10"));
            assert_eq!(record_ids(&service.current()), if had_refusal { vec!["10"] } else { vec!["10", "9"] });
            assert_eq!(fs::read(root.cache_path()).unwrap(), cache);
            assert_skill_file_bytes(&files);
            assert!(sentinel.exists());
            assert!(root.staged_files().is_empty());
            let (fresh, offline) = inline_service(&root, Err("offline".into()));
            assert_eq!(offline.calls.load(Ordering::SeqCst), 0);
            assert_auth_recovery(&fresh, &refusals, false);
            assert_eq!(fresh.admission_reader().is_denied("9"), had_refusal);
            assert!(!fresh.admission_reader().is_denied("10"));
            assert_eq!(fresh.current(), service.current());
        }
    }
}

#[test]
fn auth_inherited_unknown_survives_offline_and_invalid_outcomes_without_replay() {
    for malformed in [false, true] {
        let root = TempRoot::new();
        let (service, loader, records) = seed(&root);
        let cache = fs::read(root.cache_path()).unwrap();
        let files = skill_file_bytes(&records);
        let sentinel = root.sentinel();
        let old_token = abandon_refresh(&service, &loader, &root);
        let bytes = fs::read(root.uninstall_state_path()).unwrap();
        let outcome = if malformed {
            let mut invalid = two_plugins(&root, "invalid");
            invalid.listed_plugin_ids.push("9".into());
            Ok(invalid)
        } else {
            Err("offline".into())
        };
        let (fresh, fresh_loader) = inline_service(&root, outcome);
        assert_eq!(fresh_loader.calls.load(Ordering::SeqCst), 0);
        assert_eq!(fs::read(root.uninstall_state_path()).unwrap(), bytes);
        assert_image(&root, &[], Some(&old_token));
        assert_auth_recovery(&fresh, &[], true);
        assert!(fresh.current().is_empty());
        assert!(fresh.sync("explicit-failed-recovery").is_err());
        assert_eq!(fresh_loader.calls.load(Ordering::SeqCst), 1);
        let token = refresh_operation_id(&root);
        assert_ne!(token, old_token);
        assert_image(&root, &[], Some(&token));
        assert_auth_recovery(&fresh, &[], true);
        assert!(fresh.admission_reader().is_denied("9"));
        assert!(fresh.admission_reader().is_denied("10"));
        assert!(fresh.current().is_empty());
        assert_eq!(fs::read(root.cache_path()).unwrap(), cache);
        assert_skill_file_bytes(&files);
        assert!(sentinel.exists());
        assert!(root.staged_files().is_empty());
        let (again, again_loader) = inline_service(&root, Err("offline".into()));
        assert_eq!(again_loader.calls.load(Ordering::SeqCst), 0);
        assert_auth_recovery(&again, &[], true);
        assert!(again.current().is_empty());
    }
}

#[test]
fn auth_unknown_recovery_uses_only_fresh_records_while_clean_partial_keeps_fallback() {
    for unknown in [false, true] {
        let root = TempRoot::new();
        let (seed_owner, seed_loader, records) = seed(&root);
        let files = skill_file_bytes(&records);
        let sentinel = root.sentinel();
        if unknown {
            abandon_refresh(&seed_owner, &seed_loader, &root);
        } else {
            seed_owner.dispose();
        }
        let mut partial = two_plugins(&root, "fresh-ten");
        partial.plugins.remove(0);
        let (service, loader) = inline_service(&root, Ok(partial));
        let events = Arc::new(Mutex::new(Vec::<PluginSkillsSyncEvent>::new()));
        let captured = Arc::clone(&events);
        let service = service.with_reporter(Arc::new(move |event| captured.lock().unwrap().push(event)));
        assert_auth_recovery(&service, &[], unknown);
        let refreshed = service.sync("partial-recovery").unwrap();
        assert_eq!(record_ids(&refreshed), if unknown { vec!["10"] } else { vec!["10", "9"] });
        assert_eq!(service.current(), refreshed);
        let raw = service.current_index().unwrap();
        assert_eq!(raw.skills, refreshed);
        assert_eq!(raw.skills.iter().find(|record| record.plugin_id == "10").unwrap().plugin_version, "fresh-ten");
        if !unknown {
            assert_eq!(raw.skills.iter().find(|record| record.plugin_id == "9").unwrap().plugin_version, "retained");
        }
        assert_image(&root, &[], None);
        assert_auth_recovery(&service, &[], false);
        assert_eq!(<SandPluginSkillsService as PluginSkillsPort>::status(&service), json!({"authBlocked": []}));
        assert!(!service.admission_reader().is_denied("9"));
        assert!(!service.admission_reader().is_denied("10"));
        assert_skill_file_bytes(&files);
        assert!(sentinel.exists(), "unresolved materialization still prevents pruning");
        let cache = fs::read(root.cache_path()).unwrap();
        let journal = fs::read(root.uninstall_state_path()).unwrap();
        assert_eq!(service.sync("same-partial-again").unwrap(), refreshed);
        assert_eq!(fs::read(root.cache_path()).unwrap(), cache);
        assert_eq!(fs::read(root.uninstall_state_path()).unwrap(), journal);
        assert_eq!(loader.calls.load(Ordering::SeqCst), 2);
        let events = events.lock().unwrap();
        assert_eq!(events.len(), 2);
        assert!(events[0].changed);
        assert!(!events[1].changed, "settled refresh-token churn is not an admission change");
        assert_eq!(events[0].skill_count, refreshed.len());
        assert_eq!(events[1].skill_count, refreshed.len());
    }
}

#[test]
fn auth_exact_identity_zero_skill_release_keeps_current_denial_and_changes_once() {
    let h = Harness::new();
    let records = h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
    let files = skill_file_bytes(&records);
    let alias_refusal = PluginAuthBlock {
        plugin_id: "09".into(),
        plugin_name: "Exact zero-prefixed plugin".into(),
        marketplace_name: None,
    };
    let mut refused = refuse_nine_with_invalid_stage(&h.root);
    refused.listed_plugin_ids.push("09".into());
    refused.auth_blocked.push(alias_refusal.clone());
    assert_eq!(h.step("two-exact-refusals", LoadStep::Loaded(refused)).unwrap_err(), "invalid plugin skills cache snapshot");
    assert_image(&h.root, &[alias_refusal.clone(), auth_block("9")], None);
    assert_auth_recovery(&h.service, &[alias_refusal.clone(), auth_block("9")], false);
    assert!(h.service.admission_reader().is_denied("09"));
    assert!(h.service.admission_reader().is_denied("9"));
    assert!(!h.service.admission_reader().is_denied("10"));

    let mut fresh = two_plugins(&h.root, "retained");
    fresh.plugins[0].skills.clear();
    let mut alias = fresh.plugins[0].clone();
    alias.identifier.plugin_db_id = Some("09".into());
    alias.identifier.name = "zero-prefix".into();
    alias.install_path = h.root.0.join("plugins/cache/market/zero-prefix");
    fresh.plugins.push(alias);
    fresh.listed_plugin_ids.push("09".into());
    fresh.listed_cache_keys.push(ListedCacheKey {
        marketplace_slug: "market".into(),
        plugin_id: "zero-prefix".into(),
    });
    fresh.auth_blocked.push(auth_block("9"));
    assert_eq!(record_ids(&h.step("alias-zero-success-current-nine-denied", LoadStep::Loaded(fresh.clone())).unwrap()), vec!["10"]);
    assert!(!h.service.admission_reader().is_denied("09"), "fresh exact zero-Skill 09 is released");
    assert!(h.service.admission_reader().is_denied("9"), "current denial wins over fresh zero-Skill 9");
    assert_image(&h.root, &[auth_block("9")], None);
    assert_auth_recovery(&h.service, &[auth_block("9")], false);
    assert_eq!(h.service.current_auth_blocked(), vec![auth_block("9")]);
    assert_skill_file_bytes(&files);

    fresh.auth_blocked.clear();
    for (trigger, changed) in [("exact-nine-zero-restored", true), ("same-zero-success", false)] {
        assert_eq!(record_ids(&h.step(trigger, LoadStep::Loaded(fresh.clone())).unwrap()), vec!["10"]);
        assert!(!h.service.admission_reader().is_denied("9"));
        assert!(!h.service.admission_reader().is_denied("09"));
        assert!(!h.service.admission_reader().is_denied("10"));
        assert_image(&h.root, &[], None);
        assert_auth_recovery(&h.service, &[], false);
        assert_eq!(<SandPluginSkillsService as PluginSkillsPort>::status(&*h.service), json!({"authBlocked": []}));
        let events = h.events.lock().unwrap();
        let event = events.last().unwrap();
        assert_eq!(event.trigger, trigger);
        assert_eq!(event.changed, changed);
        assert_eq!(event.skill_count, 1);
        assert_eq!(event.outcome, "ok");
        assert_eq!(event.error_class, None);
    }
    assert_eq!(h.events.lock().unwrap().len(), 4);
    assert_skill_file_bytes(&files);
    h.assert_calls(5);
}

#[test]
fn auth_positive_journal_failure_follows_cache_barriers_and_preserves_actual_visibility() {
    for candidate_visible in [false, true] {
        let root = TempRoot::new();
        let (service, loader, records) = seed(&root);
        let files = skill_file_bytes(&records);
        let sentinel = root.sentinel();
        *loader.result.lock().unwrap() = Ok(refuse_nine_with_invalid_stage(&root));
        assert_eq!(service.sync("accepted-negative").unwrap_err(), "invalid plugin skills cache snapshot");
        let negative_image = fs::read(root.uninstall_state_path()).unwrap();
        assert_image(&root, &[auth_block("9")], None);
        *loader.result.lock().unwrap() = Ok(two_plugins(&root, "fresh"));
        let events = Arc::new(Mutex::new(Vec::<PluginSkillsSyncEvent>::new()));
        let captured = Arc::clone(&events);
        let service = service.with_reporter(Arc::new(move |event| captured.lock().unwrap().push(event)));
        let phase = if candidate_visible { ImagePublishFailure::AfterRename } else { ImagePublishFailure::BeforeRename };
        // Reserve, record the known outcome while retaining the refusal, then
        // release after strict cache publication: only the third image fails.
        assert_eq!(with_image_publish_failure(3, phase, || service.sync("positive-image-failure")).unwrap_err(), AUTH_ERROR);
        assert_eq!(loader.calls.load(Ordering::SeqCst), 3);
        let published = read_plugin_skills_snapshot(get_plugin_skills_dir(&root.0)).unwrap();
        assert_eq!(record_ids(&published.index.skills), vec!["10", "9"]);
        assert!(published.index.skills.iter().all(|record| record.plugin_version == "fresh"));
        assert!(published.index.auth_blocked.is_empty());
        assert!(service.admission_reader().is_denied("9"));
        assert!(!service.admission_reader().is_denied("10"));
        assert_auth_recovery(&service, &[auth_block("9")], false);
        assert_eq!(record_ids(&service.current()), vec!["10"]);
        assert!(events.lock().unwrap().is_empty(), "failed positive release is not a successful sync report");
        assert!(sentinel.exists(), "a failed release cannot prune");
        assert!(root.staged_files().is_empty());
        assert_skill_file_bytes(&files);
        if candidate_visible {
            assert_image(&root, &[], None);
        } else {
            assert_eq!(fs::read(root.uninstall_state_path()).unwrap(), negative_image);
            assert_image(&root, &[auth_block("9")], None);
        }
        let cache = fs::read(root.cache_path()).unwrap();
        let journal = fs::read(root.uninstall_state_path()).unwrap();
        let (fresh, offline) = inline_service(&root, Err("offline".into()));
        assert_eq!(offline.calls.load(Ordering::SeqCst), 0);
        assert_eq!(fresh.admission_reader().is_denied("9"), !candidate_visible);
        assert!(!fresh.admission_reader().is_denied("10"));
        let fresh_refusals = if candidate_visible { vec![] } else { vec![auth_block("9")] };
        assert_auth_recovery(&fresh, &fresh_refusals, false);
        assert_eq!(record_ids(&fresh.current()), if candidate_visible { vec!["10", "9"] } else { vec!["10"] });
        assert_eq!(fs::read(root.uninstall_state_path()).unwrap(), journal);
        assert_eq!(fs::read(root.cache_path()).unwrap(), cache);
        // The failed live owner still requires a real explicit fresh success,
        // even when a separately constructed owner can read the positive image.
        assert_eq!(record_ids(&service.sync("retry-after-visible-release").unwrap()), vec!["10", "9"]);
        assert!(!service.admission_reader().is_denied("9"));
        assert_image(&root, &[], None);
        assert_eq!(record_ids(&service.sync("stable-after-release").unwrap()), vec!["10", "9"]);
        let events = events.lock().unwrap();
        assert_eq!(events.len(), 2);
        assert!(events[0].changed, "the retained live refusal is released exactly once");
        assert!(!events[1].changed);
        assert!(!sentinel.exists());
        assert_skill_file_bytes(&files);
    }
}

#[test]
fn auth_foreign_or_unreadable_outcome_image_is_not_a_known_failed_write() {
    for corruption in ["semantic-only", "malformed", "directory", "missing"] {
        let gate = BoundaryGate::at_visit(2);
        let h = Harness::with_hooks(Arc::new(|| 1_000), ServiceTestHooks {
            before_auth_outcome: Some(Arc::clone(&gate.pause)),
            ..ServiceTestHooks::default()
        });
        let records = h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
        let files = skill_file_bytes(&records);
        let cache = fs::read(h.root.cache_path()).unwrap();
        let sentinel = h.root.sentinel();
        let active = h.call("negative-with-foreign-image");
        h.enter(2);
        let mut denied = two_plugins(&h.root, "must-not-publish");
        denied.auth_blocked.push(auth_block("9"));
        h.release.send(LoadStep::Loaded(denied)).unwrap();
        gate.wait();
        let expected_bytes = fs::read(h.root.uninstall_state_path()).unwrap();
        let expected_value: Value = serde_json::from_slice(&expected_bytes).unwrap();
        let preserved = get_plugin_skills_dir(&h.root.0).join("preserved-admission.fixture");
        match corruption {
            "semantic-only" => {
                let mut foreign = expected_bytes.clone();
                foreign.push(b'\n');
                assert_ne!(foreign, expected_bytes);
                assert_eq!(serde_json::from_slice::<Value>(&foreign).unwrap(), expected_value);
                fs::write(h.root.uninstall_state_path(), foreign).unwrap();
            }
            "malformed" => fs::write(h.root.uninstall_state_path(), b"{foreign-incomplete").unwrap(),
            "directory" => {
                fs::rename(h.root.uninstall_state_path(), &preserved).unwrap();
                fs::create_dir(h.root.uninstall_state_path()).unwrap();
                fs::write(h.root.uninstall_state_path().join("foreign.txt"), b"retain foreign directory").unwrap();
            }
            "missing" => fs::remove_file(h.root.uninstall_state_path()).unwrap(),
            _ => unreachable!(),
        }
        let foreign_bytes = fs::read(h.root.uninstall_state_path()).ok();
        let marker = fs::read(h.root.uninstall_version_path()).unwrap();
        gate.release.send(()).unwrap();
        assert_eq!(active.finish().unwrap_err(), AUTH_ERROR, "{corruption}");
        assert!(h.service.admission_reader().is_denied("9"));
        assert!(h.service.admission_reader().is_denied("10"));
        assert!(h.service.current().is_empty());
        assert_eq!(<SandPluginSkillsService as PluginSkillsPort>::status(&*h.service)["authRecovery"], json!({
            "refusals": [auth_block("9")],
            "refreshOutcomeUnknown": false,
            "fault": "admission-state-unavailable",
        }), "{corruption}");
        assert_eq!(fs::read(h.root.uninstall_state_path()).ok(), foreign_bytes);
        assert_eq!(fs::read(h.root.uninstall_version_path()).unwrap(), marker);
        assert_eq!(fs::read(h.root.cache_path()).unwrap(), cache);
        assert_skill_file_bytes(&files);
        assert!(sentinel.exists());
        assert!(h.root.staged_files().is_empty());
        if corruption == "directory" {
            assert_eq!(fs::read(&preserved).unwrap(), expected_bytes);
            assert_eq!(fs::read(h.root.uninstall_state_path().join("foreign.txt")).unwrap(), b"retain foreign directory");
        }
        assert!(h.call("must-not-retry-integrity-fault").finish().is_err());
        assert_eq!(fs::read(h.root.uninstall_state_path()).ok(), foreign_bytes);
        assert_eq!(h.events.lock().unwrap().len(), 1);
        h.assert_calls(2);
    }
}

#[test]
fn auth_old_loader_guard_and_repeated_dispose_cannot_alter_successor_token() {
    for old_panics in [false, true] {
        let h = Harness::new();
        let records = h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
        let files = skill_file_bytes(&records);
        let cache = fs::read(h.root.cache_path()).unwrap();
        let old = h.call("old-loader-held");
        h.enter(2);
        let old_token = refresh_operation_id(&h.root);
        let reader = h.service.admission_reader();
        let gate = BoundaryGate::at_visit(1);
        let mut successor = h.service.successor_with_loader(h.loader.clone()).unwrap()
            .with_now(Arc::new(|| 1_000));
        successor.test_hooks.after_refresh_reservation = Some(Arc::clone(&gate.pause));
        let successor = Arc::new(successor);
        assert!(Arc::ptr_eq(&reader, &successor.admission_reader()));
        assert_auth_recovery(&successor, &[], false);
        let replacement = Call::start(&successor, "successor-reservation-held");
        gate.wait();
        let next_token = refresh_operation_id(&h.root);
        assert_ne!(old_token, next_token);
        assert_image(&h.root, &[], Some(&next_token));
        let successor_journal = fs::read(h.root.uninstall_state_path()).unwrap();
        let marker = fs::read(h.root.uninstall_version_path()).unwrap();
        assert_eq!(h.loader.calls.load(Ordering::SeqCst), 2, "successor is paused before its loader");
        assert!(successor.publication.try_lock().is_ok());
        assert!(successor.state.try_lock().is_ok());
        assert!(successor.admission.state.try_lock().is_ok());
        if old_panics {
            h.release.send(LoadStep::Panicked).unwrap();
            assert!(old.outcome().is_err());
        } else {
            let mut stale = two_plugins(&h.root, "must-not-publish");
            stale.auth_blocked.push(auth_block("10"));
            h.release.send(LoadStep::Loaded(stale)).unwrap();
            assert_eq!(old.finish().unwrap(), records);
        }
        assert_eq!(fs::read(h.root.uninstall_state_path()).unwrap(), successor_journal);
        assert_eq!(refresh_operation_id(&h.root), next_token);
        assert_auth_recovery(&successor, &[], false);
        for _ in 0..2 {
            h.service.dispose();
            assert_eq!(h.service.sync("repeated-old-dispose").unwrap(), records);
            assert_eq!(fs::read(h.root.uninstall_state_path()).unwrap(), successor_journal);
            assert_eq!(fs::read(h.root.uninstall_version_path()).unwrap(), marker);
            assert_eq!(fs::read(h.root.cache_path()).unwrap(), cache);
            assert!(!reader.is_denied("9"));
            assert!(!reader.is_denied("10"));
            assert!(!successor.state.lock().unwrap().disposed);
        }
        let (fresh_reader_owner, fresh_loader) = inline_service(&h.root, Err("offline".into()));
        assert_eq!(fresh_loader.calls.load(Ordering::SeqCst), 0);
        assert_auth_recovery(&fresh_reader_owner, &[], true);
        assert!(fresh_reader_owner.current().is_empty());
        assert_eq!(fs::read(h.root.uninstall_state_path()).unwrap(), successor_journal);
        gate.release.send(()).unwrap();
        h.enter(3);
        h.release.send(LoadStep::Loaded(two_plugins(&h.root, "successor"))).unwrap();
        let committed = replacement.finish().unwrap();
        assert_eq!(record_ids(&committed), vec!["10", "9"]);
        assert!(committed.iter().all(|record| record.plugin_version == "successor"));
        assert_image(&h.root, &[], None);
        assert_auth_recovery(&successor, &[], false);
        assert!(!reader.is_denied("10"));
        assert_skill_file_bytes(&files);
        assert_eq!(h.events.lock().unwrap().len(), 1, "retired pass never reports success");
        h.assert_calls(3);
    }
}

#[test]
fn auth_panicking_coalesced_followup_retains_unknown_and_all_waiters_settle() {
    let h = Harness::new();
    let records = h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
    let files = skill_file_bytes(&records);
    let active = h.call("active-before-coalesced-panic");
    h.enter(2);
    let first_token = refresh_operation_id(&h.root);
    let active_image = fs::read(h.root.uninstall_state_path()).unwrap();
    let first_waiter = h.call("first-coalesced");
    h.pending("first-coalesced");
    let second_waiter = h.call("second-coalesced");
    h.pending("second-coalesced");
    first_waiter.assert_waiting();
    second_waiter.assert_waiting();
    assert_eq!(fs::read(h.root.uninstall_state_path()).unwrap(), active_image, "joining callers reserve no token");
    assert_eq!(h.loader.calls.load(Ordering::SeqCst), 2);
    h.release.send(LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
    assert_eq!(active.finish().unwrap(), records);
    h.enter(3);
    let followup_token = refresh_operation_id(&h.root);
    assert_ne!(followup_token, first_token);
    let followup_image = fs::read(h.root.uninstall_state_path()).unwrap();
    h.release.send(LoadStep::Panicked).unwrap();
    let outcomes = [first_waiter.outcome(), second_waiter.outcome()];
    assert_eq!(outcomes.iter().filter(|outcome| outcome.is_err()).count(), 1);
    assert_eq!(outcomes.into_iter().find_map(Result::ok).unwrap().unwrap_err(), "plugin skills sync aborted before completion");
    assert_eq!(fs::read(h.root.uninstall_state_path()).unwrap(), followup_image);
    assert_image(&h.root, &[], Some(&followup_token));
    assert_auth_recovery(&h.service, &[], true);
    assert!(h.service.admission_reader().is_denied("9"));
    assert!(h.service.admission_reader().is_denied("10"));
    assert!(h.service.current().is_empty());
    let (fresh, offline) = inline_service(&h.root, Err("offline".into()));
    assert_eq!(offline.calls.load(Ordering::SeqCst), 0);
    assert_auth_recovery(&fresh, &[], true);
    assert_eq!(fs::read(h.root.uninstall_state_path()).unwrap(), followup_image);
    assert_eq!(h.step("known-error-after-panic", LoadStep::Failed).unwrap_err(), "fixture loader failure");
    assert_auth_recovery(&h.service, &[], true);
    let mut partial = two_plugins(&h.root, "fresh-ten");
    partial.plugins.remove(0);
    assert_eq!(record_ids(&h.step("explicit-partial-recovery", LoadStep::Loaded(partial.clone())).unwrap()), vec!["10"]);
    assert_eq!(record_ids(&h.service.current_index().unwrap().skills), vec!["10"]);
    assert_auth_recovery(&h.service, &[], false);
    assert_image(&h.root, &[], None);
    assert!(h.events.lock().unwrap().last().unwrap().changed);
    assert_eq!(record_ids(&h.step("same-after-recovery", LoadStep::Loaded(partial)).unwrap()), vec!["10"]);
    assert!(!h.events.lock().unwrap().last().unwrap().changed);
    assert_skill_file_bytes(&files);
    h.assert_calls(6);
}

#[test]
fn auth_reporter_panic_after_settled_publication_does_not_recreate_unknown() {
    let root = TempRoot::new();
    let (service, loader) = inline_service(&root, Ok(two_plugins(&root, "fresh")));
    let service = service.with_reporter(Arc::new(|_| panic!("fixture reporter panic after settlement")));
    assert!(catch_unwind(AssertUnwindSafe(|| service.sync("panicking-report"))).is_err());
    assert_eq!(loader.calls.load(Ordering::SeqCst), 1);
    assert_image(&root, &[], None);
    assert_auth_recovery(&service, &[], false);
    assert_eq!(record_ids(&service.current()), vec!["10", "9"]);
    assert!(!service.admission_reader().is_denied("9"));
    assert!(!service.admission_reader().is_denied("10"));
    assert!(service.state.lock().unwrap().active.is_none());
    let bytes = fs::read(root.uninstall_state_path()).unwrap();
    let (fresh, offline) = inline_service(&root, Err("offline".into()));
    assert_eq!(offline.calls.load(Ordering::SeqCst), 0);
    assert_auth_recovery(&fresh, &[], false);
    assert_eq!(fresh.current(), service.current());
    assert_eq!(fs::read(root.uninstall_state_path()).unwrap(), bytes);
    assert_eq!(fresh.sync("offline-after-reporter-panic").unwrap_err(), "offline");
    assert_eq!(fresh.current(), service.current());
    assert_image(&root, &[], None);
}

#[test]
fn auth_pending_uninstall_survives_unknown_recovery_and_exact_refusal_release() {
    let root = TempRoot::new();
    let (service, loader, records) = seed(&root);
    let files = skill_file_bytes(&records);
    let sentinel = root.sentinel();
    let intent = service.begin_uninstall("9").unwrap();
    *loader.result.lock().unwrap() = Ok(refuse_nine_with_invalid_stage(&root));
    assert_eq!(service.sync("accepted-refusal-during-uninstall").unwrap_err(), "invalid plugin skills cache snapshot");
    assert_auth_recovery(&service, &[auth_block("9")], false);
    let token = abandon_refresh(&service, &loader, &root);
    assert_eq!(image(&root), json!({
        "formatVersion": 2,
        "confirmedPluginIds": [],
        "pending": [{"pluginId": "9", "operationId": intent.operation_id}],
        "authRefusals": [auth_block("9")],
        "refreshIntent": {"operationId": token},
    }));
    let mut fresh = two_plugins(&root, "fresh-zero-nine");
    fresh.plugins[0].skills.clear();
    let (recovered, fresh_loader) = inline_service(&root, Ok(fresh));
    assert_eq!(fresh_loader.calls.load(Ordering::SeqCst), 0);
    assert_auth_recovery(&recovered, &[auth_block("9")], true);
    assert!(recovered.current().is_empty());
    let restored = recovered.sync("fresh-while-uninstall-pending").unwrap();
    assert_eq!(record_ids(&restored), vec!["10"]);
    assert_auth_recovery(&recovered, &[], false);
    assert!(recovered.admission_reader().is_denied("9"), "auth success cannot settle an uninstall ticket");
    assert!(!recovered.admission_reader().is_denied("10"));
    assert_eq!(image(&root), json!({
        "formatVersion": 2,
        "confirmedPluginIds": [],
        "pending": [{"pluginId": "9", "operationId": intent.operation_id}],
        "authRefusals": [],
        "refreshIntent": null,
    }));
    assert_eq!(recovered.uninstall_recovery_status(), Some(json!({
        "confirmedPluginIds": [],
        "pending": [{"pluginId": "9", "operationId": intent.operation_id}],
    })));
    assert_eq!(fs::read(root.uninstall_version_path()).unwrap(), b"1\n");
    assert!(sentinel.exists());
    assert_skill_file_bytes(&files);
    let journal = fs::read(root.uninstall_state_path()).unwrap();
    assert_eq!(recovered.sync("same-while-pending").unwrap(), restored);
    assert_eq!(fs::read(root.uninstall_state_path()).unwrap(), journal);
    assert!(sentinel.exists());
    recovered.settle_uninstall(&intent, PluginUninstallSettlement::NotRemoved).unwrap();
    assert!(!recovered.admission_reader().is_denied("9"));
    assert_image(&root, &[], None);
    assert_eq!(record_ids(&recovered.current()), vec!["10"], "zero-Skill success did not restore an old row");
    *fresh_loader.result.lock().unwrap() = Ok(two_plugins(&root, "fresh-nine-with-skill"));
    assert_eq!(record_ids(&recovered.sync("new-materialization-after-ticket").unwrap()), vec!["10", "9"]);
    assert!(!sentinel.exists());
    assert_skill_file_bytes(&files);
}

#[test]
fn auth_clean_dispose_does_no_io_but_fresh_construction_cannot_assume_the_outcome() {
    let h = Harness::new();
    let records = h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
    let cache = fs::read(h.root.cache_path()).unwrap();
    let files = skill_file_bytes(&records);
    let old = h.call("held-before-dispose");
    h.enter(2);
    let token = refresh_operation_id(&h.root);
    let journal = fs::read(h.root.uninstall_state_path()).unwrap();
    let marker = fs::read(h.root.uninstall_version_path()).unwrap();
    h.service.dispose();
    assert_eq!(fs::read(h.root.uninstall_state_path()).unwrap(), journal);
    assert_eq!(fs::read(h.root.uninstall_version_path()).unwrap(), marker);
    assert_eq!(h.service.current(), records);
    assert_auth_recovery(&h.service, &[], false);
    assert!(!h.service.admission_reader().is_denied("9"));
    assert!(!h.service.admission_reader().is_denied("10"));
    let (fresh, offline) = inline_service(&h.root, Err("offline".into()));
    assert_eq!(offline.calls.load(Ordering::SeqCst), 0);
    assert_auth_recovery(&fresh, &[], true);
    assert!(fresh.current().is_empty());
    assert_image(&h.root, &[], Some(&token));
    let mut stale = two_plugins(&h.root, "must-not-publish");
    stale.auth_blocked.push(auth_block("10"));
    h.release.send(LoadStep::Loaded(stale)).unwrap();
    assert_eq!(old.finish().unwrap(), records);
    assert_eq!(fs::read(h.root.uninstall_state_path()).unwrap(), journal);
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), cache);
    assert!(!h.service.admission_reader().is_denied("10"));
    assert_auth_recovery(&h.service, &[], false);
    assert_eq!(h.events.lock().unwrap().len(), 1);
    assert_skill_file_bytes(&files);
    h.assert_calls(2);
}
