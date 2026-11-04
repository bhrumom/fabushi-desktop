use super::*;
use std::process::{Child, Command, Stdio};
use std::time::{Instant, SystemTime};

struct StateFixture {
    root: PathBuf,
}

impl StateFixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "mahayana-peer-state-{}",
            uuid::Uuid::new_v4().simple()
        ));
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(&root).expect("create an exclusively owned fixture directory");
        Self { root }
    }

    fn path(&self) -> PathBuf {
        self.root.join(STATE_FILE_NAME)
    }

    fn lock_path(&self) -> PathBuf {
        PathBuf::from(format!("{}.lock", self.path().display()))
    }

    fn open(&self) -> PersistentServerRequestRegistry {
        let registry = PersistentServerRequestRegistry::new(Some(self.path()));
        assert!(registry.durability_info()["lastError"].is_null());
        registry
    }

    fn assert_no_temporary_files(&self) {
        assert!(fs::read_dir(&self.root).unwrap().all(|entry| {
            !entry.unwrap().file_name().to_string_lossy().contains(".tmp-")
        }));
    }
}

impl Drop for StateFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

// This fault happens after the public method changed its private candidate,
// before the production writer commits. It is a real invalid filesystem input,
// not a mocked successful write or an injected error return.
fn block_backup_after_candidate(path: &Path) {
    fs::create_dir(backup_path(path)).expect("create the owned commit blocker");
}

fn block_primary_after_candidate(path: &Path) {
    fs::create_dir(path).expect("create the owned primary blocker");
}

fn response(id: &str) -> Value {
    json!({"jsonrpc":"2.0", "id":id, "result":{"decision":"deny"}})
}

fn persisted_request() -> Value {
    let mut registry = ServerRequestRegistry::default();
    let issued = registry.issue_with_id(
        "srq-existing".into(),
        "session-1".into(),
        "approval".into(),
        json!({"requestId":"approval-existing"}),
        Vec::new(),
        10,
    ).unwrap();
    serde_json::to_value(issued.request).unwrap()
}

#[test]
fn pending_request_survives_process_style_reopen() {
    let fixture = StateFixture::new();
    let original;
    {
        let mut registry = fixture.open();
        original = registry.issue("session-1", "approval", json!({"requestId":"approval-1"})).unwrap();
        assert_eq!(registry.len(), 1);
        assert!(registry.durability_info()["lastError"].is_null());
    }
    let restored = fixture.open();
    assert_eq!(restored.len(), 1);
    assert_eq!(restored.open_requests("session-1"), vec![original.frame]);
    assert_eq!(restored.restored_requests, 1);
    fixture.assert_no_temporary_files();
}

#[test]
fn resolving_request_is_durable_and_preserves_replay_section() {
    let fixture = StateFixture::new();
    let path = fixture.path();
    let retained = json!({
        "schemaVersion":1,
        "replay":{"events":[], "retainedEpoch":"not-owned-by-peer"},
        "peer":{"requests":[], "retainedPeer":{"value":"unchanged"}},
        "otherOwner":{"value":[1,2,3]}
    });
    fs::write(&path, serde_json::to_vec(&retained).unwrap()).unwrap();
    let id;
    {
        let mut registry = fixture.open();
        id = registry.issue("session-1", "approval", json!({"requestId":"approval-1"})).unwrap().request.id;
        let resolved = registry.resolve_response(&response(&id)).unwrap().unwrap();
        assert_eq!(resolved.request.id, id);
        assert!(registry.durability_info()["lastError"].is_null());
    }
    let restored = fixture.open();
    assert!(restored.is_empty());
    let document: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(document["replay"], retained["replay"]);
    assert_eq!(document["peer"]["retainedPeer"], retained["peer"]["retainedPeer"]);
    assert_eq!(document["otherOwner"], retained["otherOwner"]);
    assert_eq!(document["peer"]["requests"], json!([]));
    fixture.assert_no_temporary_files();
}

#[test]
fn failed_issue_retains_live_state_then_repairs_and_reopens() {
    let fixture = StateFixture::new();
    let path = fixture.path();
    let mut registry = fixture.open();
    registry.issue("session-keep", "approval", json!({"requestId":"keep"})).unwrap();
    let before = registry.export_state();
    let bytes = fs::read(&path).unwrap();
    registry.before_commit = Some(block_backup_after_candidate);
    assert!(matches!(
        registry.issue("session-new", "approval", json!({"requestId":"new"})),
        Err(ServerRequestError::Persistence(_))
    ));
    assert_eq!(registry.export_state(), before);
    assert_eq!(fs::read(&path).unwrap(), bytes);
    assert!(!registry.durability_info()["lastError"].is_null());
    fixture.assert_no_temporary_files();

    fs::remove_dir(backup_path(&path)).unwrap();
    let issued = registry.issue("session-new", "approval", json!({"requestId":"new"})).unwrap();
    assert!(registry.durability_info()["lastError"].is_null());
    let committed = registry.export_state();
    assert_eq!(committed.len(), 2);
    assert_eq!(committed[0], before[0]);
    assert_eq!(committed[1].id, issued.request.id);
    drop(registry);
    assert_eq!(fixture.open().export_state(), committed);
}

#[test]
fn failed_response_keeps_pending_and_stable_id_until_repaired() {
    let fixture = StateFixture::new();
    let path = fixture.path();
    let mut registry = fixture.open();
    let issued = registry.issue("session-1", "approval", json!({"requestId":"approval-1"})).unwrap();
    let before = registry.export_state();
    let bytes = fs::read(&path).unwrap();
    registry.before_commit = Some(block_backup_after_candidate);
    assert!(matches!(
        registry.resolve_response(&response(&issued.request.id)),
        Err(ServerRequestError::Persistence(_))
    ));
    assert_eq!(registry.export_state(), before);
    assert_eq!(fs::read(&path).unwrap(), bytes);
    assert!(!registry.durability_info()["lastError"].is_null());
    fs::remove_dir(backup_path(&path)).unwrap();
    assert_eq!(fixture.open().export_state(), before);
    let resolved = registry.resolve_response(&response(&issued.request.id)).unwrap().unwrap();
    assert_eq!(resolved.request.id, issued.request.id);
    assert!(registry.is_empty());
    assert!(registry.resolve_response(&response(&issued.request.id)).unwrap().is_none());
    drop(registry);
    assert!(fixture.open().is_empty());
    fixture.assert_no_temporary_files();
}

#[test]
fn failed_clarify_lock_and_completion_preserve_answers_before_retry() {
    let fixture = StateFixture::new();
    let path = fixture.path();
    let mut registry = fixture.open();
    let issued = registry.issue_with_questions(
        "session-1", "clarify", json!({"questions":[{"id":"q1"},{"id":"q2"}]}),
        vec!["q1".into(), "q2".into()],
    ).unwrap();

    for (question, answer, remaining) in [("q1", "alpha", 1usize), ("q2", "beta", 0usize)] {
        let before = registry.export_state();
        let bytes = fs::read(&path).unwrap();
        registry.before_commit = Some(block_backup_after_candidate);
        assert!(matches!(
            registry.lock_answer(&issued.request.id, question, answer),
            Err(ServerRequestError::Persistence(_))
        ));
        assert_eq!(registry.export_state(), before);
        assert_eq!(fs::read(&path).unwrap(), bytes);
        fs::remove_dir(backup_path(&path)).unwrap();
        assert_eq!(fixture.open().export_state(), before);

        let outcome = registry.lock_answer(&issued.request.id, question, answer).unwrap().unwrap();
        assert_eq!(outcome.remaining_question_ids.len(), remaining);
        if remaining == 0 {
            let resolved = outcome.resolution.expect("last answer resolves only after commit");
            assert_eq!(resolved.request.id, issued.request.id);
            assert_eq!(resolved.response, ServerRequestResponse::Result(json!({
                "answers":{"q1":"alpha","q2":"beta"}
            })));
        } else {
            assert!(outcome.resolution.is_none());
            let restored = fixture.open();
            assert_eq!(restored.export_state(), registry.export_state());
            assert_eq!(restored.open_requests("session-1")[0]["params"]["answers"]["q1"], "alpha");
        }
        fixture.assert_no_temporary_files();
    }
    drop(registry);
    assert!(fixture.open().is_empty());
}

#[test]
fn failed_cancel_has_no_notifications_and_preserves_other_sessions() {
    let fixture = StateFixture::new();
    let path = fixture.path();
    let mut registry = fixture.open();
    let first = registry.issue("session-1", "approval", json!({"requestId":"approval-1"})).unwrap();
    let other = registry.issue("session-2", "approval", json!({"requestId":"approval-2"})).unwrap();
    let before = registry.export_state();
    let bytes = fs::read(&path).unwrap();
    registry.before_commit = Some(block_backup_after_candidate);
    assert!(matches!(
        registry.cancel_session(Some("session-1"), "interrupted"),
        Err(ServerRequestError::Persistence(_))
    ));
    assert_eq!(registry.export_state(), before);
    assert_eq!(fs::read(&path).unwrap(), bytes);
    fs::remove_dir(backup_path(&path)).unwrap();
    assert_eq!(fixture.open().export_state(), before);

    let notifications = registry.cancel_session(Some("session-1"), "interrupted").unwrap();
    assert_eq!(notifications.len(), 1);
    assert_eq!(notifications[0]["method"], REQUEST_CANCEL_METHOD);
    assert_eq!(notifications[0]["params"]["id"], first.request.id);
    assert_eq!(notifications[0]["params"]["reason"], "interrupted");
    assert_eq!(registry.export_state(), vec![other.request]);
    drop(registry);
    assert_eq!(fixture.open().len(), 1);
    fixture.assert_no_temporary_files();
}

#[test]
fn real_temp_creation_failure_rolls_back_every_mutation_then_reopens() {
    #[derive(Clone, Copy, Debug)]
    enum Mutation {
        Issue,
        Resolve,
        LockPartial,
        LockComplete,
        Cancel,
    }

    fn apply(
        registry: &mut PersistentServerRequestRegistry,
        mutation: Mutation,
        approval_id: &str,
        clarify_id: &str,
    ) -> Result<(), ServerRequestError> {
        match mutation {
            Mutation::Issue => registry.issue(
                "session-new", "approval", json!({"requestId":"new"}),
            ).map(|_| ()),
            Mutation::Resolve => registry.resolve_response(&response(approval_id)).map(|_| ()),
            Mutation::LockPartial => registry.lock_answer(clarify_id, "q1", "alpha").map(|_| ()),
            Mutation::LockComplete => registry.lock_answer(clarify_id, "q2", "beta").map(|_| ()),
            Mutation::Cancel => registry.cancel_session(Some("session-approval"), "cancelled").map(|_| ()),
        }
    }

    for mutation in [
        Mutation::Issue, Mutation::Resolve, Mutation::LockPartial,
        Mutation::LockComplete, Mutation::Cancel,
    ] {
        let fixture = StateFixture::new();
        // On the supported CI filesystems each component is limited to 255
        // bytes/code units. The original, .lock, and .bak names remain valid;
        // only the real writer's UUID temp component exceeds that limit. The
        // canonical root also uses Windows' extended absolute path spelling.
        let path = fixture.root.canonicalize().unwrap().join("s".repeat(240));
        let mut seed = ServerRequestRegistry::default();
        let approval = seed.issue(
            "session-approval", "approval", json!({"requestId":"keep"}),
        ).unwrap();
        let clarify = seed.issue_with_questions(
            "session-clarify", "clarify", json!({}), vec!["q1".into(), "q2".into()],
        ).unwrap();
        if matches!(mutation, Mutation::LockComplete) {
            seed.lock_answer(&clarify.request.id, "q1", "alpha").unwrap().unwrap();
        }
        let before = seed.export_state();
        let bytes = serde_json::to_vec(&json!({
            "schemaVersion":1, "peer":{"requests":before},
            "replay":{"events":[], "retained":"temp-create-failure"}
        })).unwrap();
        fs::write(&path, &bytes).expect("the committed state component must be legal");
        let mut registry = PersistentServerRequestRegistry::new(Some(path.clone()));
        assert!(registry.durability_info()["lastError"].is_null(), "legal state/lock paths: {mutation:?}");
        assert_eq!(registry.export_state(), before);

        let error = apply(&mut registry, mutation, &approval.request.id, &clarify.request.id)
            .expect_err("the actual OS temp-file create must fail after candidate mutation");
        match error {
            ServerRequestError::Persistence(message) => assert!(
                message.contains("failed to create gateway state temp file:"),
                "must reach the writer's actual temp creation stage for {mutation:?}: {message}",
            ),
            other => panic!("unexpected error for {mutation:?}: {other}"),
        }
        assert_eq!(registry.export_state(), before, "live rollback: {mutation:?}");
        assert_eq!(fs::read(&path).unwrap(), bytes, "durable rollback: {mutation:?}");
        assert!(!registry.durability_info()["lastError"].is_null());
        assert!(!backup_path(&path).exists());
        fixture.assert_no_temporary_files();
        drop(registry);

        // Repair only this fixture's file location, then recover through the
        // production constructor and public mutation API using the same IDs.
        fs::rename(&path, fixture.path()).unwrap();
        let mut repaired = fixture.open();
        assert_eq!(repaired.export_state(), before);
        apply(&mut repaired, mutation, &approval.request.id, &clarify.request.id).unwrap();
        let committed = repaired.export_state();
        assert_ne!(committed, before, "repaired mutation must commit: {mutation:?}");
        match mutation {
            Mutation::Issue => assert_eq!(committed.len(), 3),
            Mutation::Resolve | Mutation::Cancel => {
                assert_eq!(committed.len(), 1);
                assert_eq!(committed[0].id, clarify.request.id);
            }
            Mutation::LockPartial => {
                assert_eq!(committed.len(), 2);
                assert_eq!(repaired.open_requests("session-clarify")[0]["params"]["answers"]["q1"], "alpha");
            }
            Mutation::LockComplete => {
                assert_eq!(committed.len(), 1);
                assert_eq!(committed[0].id, approval.request.id);
            }
        }
        assert!(repaired.durability_info()["lastError"].is_null());
        drop(repaired);
        assert_eq!(fixture.open().export_state(), committed);
        fixture.assert_no_temporary_files();
    }
}

#[test]
fn malformed_state_import_is_atomic_and_never_overwrites_inputs() {
    let fixture = StateFixture::new();
    let path = fixture.path();
    let mut registry = fixture.open();
    registry.issue("session-1", "approval", json!({"requestId":"keep"})).unwrap();
    let before = registry.export_state();
    let original = fs::read(&path).unwrap();
    let valid = persisted_request();
    let mut bad_tail = valid.clone();
    bad_tail["id"] = json!("not-an-srq");
    bad_tail["createdAtMs"] = json!(20);
    let valid_request: PendingServerRequest = serde_json::from_value(valid.clone()).unwrap();
    let bad_request: PendingServerRequest = serde_json::from_value(bad_tail.clone()).unwrap();
    let mut existing_base = ServerRequestRegistry::default();
    existing_base.issue("existing-session", "approval", json!({"requestId":"base-keep"})).unwrap();
    let base_before = existing_base.export_state();
    for invalid_import in [
        vec![valid_request.clone(), bad_request],
        vec![valid_request.clone(), valid_request],
    ] {
        assert!(existing_base.import_state(invalid_import).is_err());
        assert_eq!(existing_base.export_state(), base_before, "base import is independently atomic");
    }
    let mut malformed = vec![
        b"{invalid".to_vec(),
        serde_json::to_vec(&json!([])).unwrap(),
        serde_json::to_vec(&json!({"peer":null})).unwrap(),
        serde_json::to_vec(&json!({"peer":[]})).unwrap(),
        serde_json::to_vec(&json!({"peer":{}})).unwrap(),
        serde_json::to_vec(&json!({"peer":{"requests":null}})).unwrap(),
        serde_json::to_vec(&json!({"peer":{"requests":{}}})).unwrap(),
        serde_json::to_vec(&json!({"peer":{"requests":[valid.clone(), valid.clone()]}})).unwrap(),
        serde_json::to_vec(&json!({"peer":{"requests":[valid.clone(), bad_tail]}})).unwrap(),
    ];
    for version in [Value::Null, json!("1"), json!(-1), json!(1.0), json!(2), json!(true)] {
        malformed.push(serde_json::to_vec(&json!({
            "schemaVersion":version, "peer":{"requests":[valid.clone()]}
        })).unwrap());
    }
    let mut bad_params = valid.clone();
    bad_params["params"] = json!([]);
    malformed.push(serde_json::to_vec(&json!({"peer":{"requests":[bad_params]}})).unwrap());
    let mut unknown_method = valid.clone();
    unknown_method["method"] = json!("unknown");
    malformed.push(serde_json::to_vec(&json!({"peer":{"requests":[unknown_method]}})).unwrap());

    for bytes in malformed {
        fs::write(&path, &bytes).unwrap();
        let fresh = PersistentServerRequestRegistry::new(Some(path.clone()));
        assert!(fresh.is_empty(), "malformed import must not expose a valid prefix");
        assert_eq!(fresh.restored_requests, 0);
        assert!(!fresh.durability_info()["lastError"].is_null());
        assert!(registry.issue("session-new", "approval", json!({"requestId":"new"})).is_err());
        assert_eq!(registry.export_state(), before);
        assert_eq!(fs::read(&path).unwrap(), bytes);
        fixture.assert_no_temporary_files();
    }
    fs::write(&path, original).unwrap();
    registry.issue("session-new", "approval", json!({"requestId":"new"})).unwrap();
    assert!(registry.durability_info()["lastError"].is_null());
    drop(registry);
    assert_eq!(fixture.open().len(), 2);
}

#[test]
fn stale_instances_merge_changes_without_resurrecting_resolved_requests() {
    let fixture = StateFixture::new();
    let mut first = fixture.open();
    let mut stale = fixture.open();
    let a = first.issue("session-1", "approval", json!({"requestId":"a"})).unwrap();
    let b = stale.issue("session-2", "approval", json!({"requestId":"b"})).unwrap();
    assert_eq!(fixture.open().len(), 2);
    assert_eq!(stale.export_state()[0].id, a.request.id);
    assert_eq!(stale.export_state()[1].id, b.request.id);

    let resolved = first.resolve_response(&response(&a.request.id)).unwrap().unwrap();
    assert_eq!(resolved.request.id, a.request.id);
    assert_eq!(first.export_state(), vec![b.request.clone()]);
    let c = stale.issue("session-3", "approval", json!({"requestId":"c"})).unwrap();
    let current = fixture.open().export_state();
    assert_eq!(current, vec![b.request, c.request]);
    let bytes = fs::read(fixture.path()).unwrap();
    assert!(first.resolve_response(&response(&a.request.id)).unwrap().is_none());
    assert_eq!(first.export_state(), current);
    assert_eq!(fs::read(fixture.path()).unwrap(), bytes);
}

#[test]
fn approval_issue_and_response_lookup_use_latest_transaction() {
    let fixture = StateFixture::new();
    let mut first = fixture.open();
    let mut stale = fixture.open();
    let issued = first.issue_approval_request(
        "session-1", json!({"requestId":"approval-shared","title":"Approve"}),
    ).unwrap().unwrap();
    let bytes = fs::read(fixture.path()).unwrap();
    assert!(stale.issue_approval_request(
        "session-1", json!({"requestId":"approval-shared","title":"duplicate"}),
    ).unwrap().is_none());
    assert_eq!(stale.export_state(), first.export_state());
    assert_eq!(fs::read(fixture.path()).unwrap(), bytes);

    let resolved = stale.resolve_approval_request(
        "approval-shared", json!({"decision":"accept"}),
    ).unwrap().unwrap();
    assert_eq!(resolved.request.id, issued.request.id);
    assert_eq!(resolved.response, ServerRequestResponse::Result(json!({"decision":"accept"})));
    assert!(first.resolve_approval_request("approval-shared", json!({"decision":"deny"})).unwrap().is_none());
    assert!(first.is_empty());
    let committed = fs::read(fixture.path()).unwrap();
    fs::write(fixture.lock_path(), b"owned fixture legacy sentinel\n").unwrap();
    assert!(matches!(
        first.resolve_approval_request("absent", json!({"decision":"deny"})),
        Err(ServerRequestError::Persistence(_))
    ));
    assert_eq!(fs::read(fixture.path()).unwrap(), committed);
    fs::write(fixture.lock_path(), b"").unwrap();
    assert!(first.resolve_approval_request("absent", json!({"decision":"deny"})).unwrap().is_none());
    assert!(fixture.open().is_empty());
}

#[test]
fn only_backup_survives_failed_mutation_then_commits_on_repair() {
    let fixture = StateFixture::new();
    let path = fixture.path();
    let backup = backup_path(&path);
    let bytes = serde_json::to_vec(&json!({
        "schemaVersion":1,
        "peer":{"requests":[persisted_request()], "retained":"peer-metadata"},
        "replay":{"events":[], "retained":"replay-metadata"},
        "unrelated":[true, false]
    })).unwrap();
    fs::write(&backup, &bytes).unwrap();
    let mut registry = fixture.open();
    let before = registry.export_state();
    assert_eq!(before.len(), 1);
    assert!(!path.exists());
    registry.before_commit = Some(block_primary_after_candidate);
    assert!(matches!(
        registry.issue("session-2", "approval", json!({"requestId":"next"})),
        Err(ServerRequestError::Persistence(_))
    ));
    assert!(path.is_dir());
    assert_eq!(fs::read(&backup).unwrap(), bytes);
    assert_eq!(registry.export_state(), before);
    fixture.assert_no_temporary_files();
    fs::remove_dir(&path).unwrap();

    registry.issue("session-2", "approval", json!({"requestId":"next"})).unwrap();
    assert!(path.is_file());
    assert!(!backup.exists());
    let written: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(written["peer"]["retained"], "peer-metadata");
    assert_eq!(written["replay"]["retained"], "replay-metadata");
    assert_eq!(written["unrelated"], json!([true, false]));
    drop(registry);
    assert_eq!(fixture.open().len(), 2);
}

#[test]
fn nonregular_and_legacy_inputs_fail_closed_without_touching_targets() {
    let fixture = StateFixture::new();
    let path = fixture.path();
    let backup = backup_path(&path);
    let lock_path = fixture.lock_path();
    let valid = br#"{"schemaVersion":1,"peer":{"requests":[]},"unrelated":"keep"}"#;

    for candidate in [&path, &backup, &lock_path] {
        if candidate.exists() {
            fs::remove_file(candidate).unwrap();
        }
        fs::create_dir(candidate).unwrap();
        let mut registry = PersistentServerRequestRegistry::new(Some(path.clone()));
        assert!(!registry.durability_info()["lastError"].is_null());
        assert!(registry.issue("session-1", "approval", json!({"requestId":"a"})).is_err());
        assert!(candidate.is_dir());
        fs::remove_dir(candidate).unwrap();
        fixture.assert_no_temporary_files();
    }

    fs::write(&path, valid).unwrap();
    fs::write(&lock_path, b"12345\n").unwrap();
    let mut registry = PersistentServerRequestRegistry::new(Some(path.clone()));
    assert!(!registry.durability_info()["lastError"].is_null());
    assert!(registry.cancel_session(None, "test").is_err());
    assert_eq!(fs::read(&path).unwrap(), valid);
    assert_eq!(fs::read(&lock_path).unwrap(), b"12345\n");
    fs::write(&lock_path, b"").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let target = fixture.root.join("external-to-operation");
        fs::write(&target, valid).unwrap();
        for candidate in [&path, &backup, &lock_path] {
            if candidate.exists() {
                fs::remove_file(candidate).unwrap();
            }
            symlink(&target, candidate).unwrap();
            let mut rejected = PersistentServerRequestRegistry::new(Some(path.clone()));
            assert!(!rejected.durability_info()["lastError"].is_null());
            assert!(rejected.issue("session-1", "approval", json!({"requestId":"a"})).is_err());
            assert_eq!(fs::read(&target).unwrap(), valid);
            assert!(fs::symlink_metadata(candidate).unwrap().file_type().is_symlink());
            fs::remove_file(candidate).unwrap();
            if candidate == &path {
                fs::write(&path, valid).unwrap();
            }
        }

        let existing_parent = fixture.root.join("existing-parent");
        fs::create_dir(&existing_parent).unwrap();
        File::open(&existing_parent).unwrap()
            .set_permissions(fs::Permissions::from_mode(0o755)).unwrap();
        let existing_path = existing_parent.join(STATE_FILE_NAME);
        let mut healthy = PersistentServerRequestRegistry::new(Some(existing_path.clone()));
        healthy.issue("session-1", "approval", json!({"requestId":"a"})).unwrap();
        assert_eq!(fs::metadata(&existing_parent).unwrap().permissions().mode() & 0o777, 0o755);
        assert_eq!(fs::metadata(&existing_path).unwrap().permissions().mode() & 0o777, 0o600);
    }
    fixture.assert_no_temporary_files();
}

#[test]
fn memory_only_registry_preserves_behavior_without_filesystem() {
    let mut registry = PersistentServerRequestRegistry::new(None);
    assert_eq!(registry.durability_info()["enabled"], false);
    assert!(registry.state_path.is_none());
    let approval = registry.issue_approval_request(
        "session-1", json!({"requestId":"approval-memory"}),
    ).unwrap().unwrap();
    assert!(registry.issue_approval_request(
        "session-1", json!({"requestId":"approval-memory"}),
    ).unwrap().is_none());
    assert_eq!(registry.len(), 1);
    assert_eq!(registry.resolve_response(&response(&approval.request.id)).unwrap().unwrap().request.id, approval.request.id);
    let clarify = registry.issue_with_questions(
        "session-2", "clarify", json!({}), vec!["q1".into(), "q2".into()],
    ).unwrap();
    assert!(registry.lock_answer(&clarify.request.id, "q1", "yes").unwrap().unwrap().resolution.is_none());
    let notifications = registry.cancel_session(Some("session-2"), "cancelled").unwrap();
    assert_eq!(notifications.len(), 1);
    assert!(registry.is_empty());
    assert!(registry.durability_info()["lastError"].is_null());
}

struct KillOnDrop(Child);

impl Drop for KillOnDrop {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn held_old_lock_cannot_be_stolen_and_killed_child_releases_it() {
    const CHILD_PATH: &str = "FBCP_NMR48_PEER_LOCK_CHILD_STATE";
    const READY_PATH: &str = "FBCP_NMR48_PEER_LOCK_CHILD_READY";
    if let Some(path) = std::env::var_os(CHILD_PATH) {
        let path = PathBuf::from(path);
        let ready = PathBuf::from(std::env::var_os(READY_PATH).expect("owned child barrier"));
        let _held = acquire_state_lock(&path).expect("child acquires the real OS lock");
        fs::write(&ready, b"ready").unwrap();
        let deadline = Instant::now() + Duration::from_secs(30);
        while Instant::now() < deadline {
            thread::sleep(Duration::from_millis(25));
        }
        panic!("parent did not terminate its lock child within the bounded lifetime");
    }

    let fixture = StateFixture::new();
    let path = fixture.path();
    let mut registry = fixture.open();
    registry.issue("session-keep", "approval", json!({"requestId":"keep"})).unwrap();
    let before = registry.export_state();
    let bytes = fs::read(&path).unwrap();
    let lock_path = fixture.lock_path();
    let lock_file = OpenOptions::new().read(true).write(true).open(&lock_path).unwrap();
    let old = SystemTime::now() - Duration::from_secs(120);
    lock_file.set_times(fs::FileTimes::new().set_modified(old)).unwrap();
    drop(lock_file);

    let ready = fixture.root.join("child-ready");
    let mut child = KillOnDrop(Command::new(std::env::current_exe().unwrap())
        .arg("--exact")
        .arg("durability_tests::held_old_lock_cannot_be_stolen_and_killed_child_releases_it")
        .arg("--nocapture")
        .arg("--test-threads=1")
        .env(CHILD_PATH, &lock_path)
        .env(READY_PATH, &ready)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn().expect("launch bounded lock-holding child"));
    let started = Instant::now();
    while !ready.exists() {
        if let Some(status) = child.0.try_wait().unwrap() {
            panic!("lock child exited before its barrier: {status}");
        }
        assert!(started.elapsed() < Duration::from_secs(10), "lock child barrier timed out");
        thread::sleep(Duration::from_millis(10));
    }
    assert!(fs::metadata(&lock_path).unwrap().modified().unwrap().elapsed().unwrap() > Duration::from_secs(30));
    assert!(matches!(
        registry.issue("session-new", "approval", json!({"requestId":"new"})),
        Err(ServerRequestError::Persistence(_))
    ));
    assert_eq!(registry.export_state(), before);
    assert_eq!(fs::read(&path).unwrap(), bytes);
    assert!(lock_path.is_file());
    assert_eq!(fs::metadata(&lock_path).unwrap().len(), 0);
    assert!(child.0.try_wait().unwrap().is_none(), "old lock must still belong to the live child");

    child.0.kill().expect("kill the test-owned lock holder");
    assert!(!child.0.wait().unwrap().success());
    assert!(lock_path.is_file(), "release must retain the shared lock inode");
    registry.issue("session-new", "approval", json!({"requestId":"new"})).unwrap();
    assert!(registry.durability_info()["lastError"].is_null());
    drop(registry);
    assert_eq!(fixture.open().len(), 2);
    fixture.assert_no_temporary_files();
}

#[test]
fn business_errors_and_noops_leave_committed_state_unchanged() {
    let fixture = StateFixture::new();
    let mut registry = fixture.open();
    let clarify = registry.issue_with_questions(
        "session-1", "clarify", json!({}), vec!["q1".into()],
    ).unwrap();
    let before = registry.export_state();
    let bytes = fs::read(fixture.path()).unwrap();
    assert!(matches!(
        registry.issue("session-1", "unknown", json!({})),
        Err(ServerRequestError::UnsupportedMethod(_))
    ));
    assert!(matches!(
        registry.issue("session-1", "approval", json!([])),
        Err(ServerRequestError::ParamsMustBeObject)
    ));
    assert!(matches!(
        registry.lock_answer(&clarify.request.id, "absent", "answer"),
        Err(ServerRequestError::UnknownQuestion(_))
    ));
    for params in [json!({}), json!({"requestId":""}), json!({"requestId":"  "}), json!({"requestId":42})] {
        assert!(matches!(
            registry.issue_approval_request("session-1", params),
            Err(ServerRequestError::InvalidApprovalRequestId)
        ));
    }
    assert!(matches!(
        registry.resolve_approval_request("", json!({"decision":"deny"})),
        Err(ServerRequestError::InvalidApprovalRequestId)
    ));
    assert!(registry.resolve_response(&response("srq-absent")).unwrap().is_none());
    assert!(registry.cancel_session(Some("absent"), "late").unwrap().is_empty());
    assert_eq!(registry.export_state(), before);
    assert_eq!(fs::read(fixture.path()).unwrap(), bytes);
    assert_eq!(fixture.open().export_state(), before);
    fixture.assert_no_temporary_files();
}

#[test]
fn stale_clarify_writers_preserve_other_committed_answers() {
    let fixture = StateFixture::new();
    let mut first = fixture.open();
    let issued = first.issue_with_questions(
        "session-1", "clarify", json!({}), vec!["q1".into(), "q2".into()],
    ).unwrap();
    let mut stale = fixture.open();
    first.lock_answer(&issued.request.id, "q1", "alpha").unwrap().unwrap();
    let outcome = stale.lock_answer(&issued.request.id, "q2", "beta").unwrap().unwrap();
    let resolved = outcome.resolution.expect("latest first answer completes the batch");
    assert_eq!(resolved.response, ServerRequestResponse::Result(json!({
        "answers":{"q1":"alpha","q2":"beta"}
    })));
    let bytes = fs::read(fixture.path()).unwrap();
    assert!(first.lock_answer(&issued.request.id, "q1", "stale").unwrap().is_none());
    assert!(first.is_empty());
    assert_eq!(fs::read(fixture.path()).unwrap(), bytes);
    assert!(fixture.open().is_empty());
}
