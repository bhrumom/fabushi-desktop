use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use mahayana_host_runtime::extensions::transcript::pending_wake_rearm::{
    LostSubagentWake, PendingWakeRearm, PendingWakeReport, PendingWakeRuntimePort,
    PENDING_WAKE_STALE_MAX_AGE_MS,
};
use mahayana_host_runtime::extensions::transcript::sand_pending_wake_store::{
    DurablePendingWakeMarker, PendingWakeKind, QuietWakeOrigin, SandPendingWakeStore,
    coerce_quiet_origin,
};
use mahayana_host_runtime::runner::background_work::{
    CloudAgentPendingWatch, CloudAgentWatchOptions, CloudAgentWatchOutcome,
    RunnerCloudAgentWatches,
};
use uuid::Uuid;

#[derive(Default)]
struct FakeRuntime {
    pending_store: Option<SandPendingWakeStore>,
    can_execute: Mutex<bool>,
    gone: Mutex<HashSet<String>>,
    group: Mutex<HashSet<String>>,
    armed: Mutex<HashSet<(String, String)>>,
    cloud_watches: Mutex<Vec<(String, String)>>,
    admit_cloud_watch: Mutex<Option<Arc<dyn Fn(&DurablePendingWakeMarker) -> Result<bool, String> + Send + Sync>>>,
    before_cloud_armed_check: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
    after_cloud_armed_check: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
    cloud_watch_finishes: Mutex<bool>,
    shell_watches: Mutex<Vec<(String, String)>>,
    interrupted: Mutex<Vec<String>>,
    revived: Mutex<Vec<LostSubagentWake>>,
    emitted: Mutex<Vec<String>>,
    reports: Mutex<Vec<PendingWakeReport>>,
}
impl FakeRuntime {
    fn with_store(pending_store: SandPendingWakeStore) -> Self {
        Self { pending_store: Some(pending_store), ..Self::default() }
    }
}
impl PendingWakeRuntimePort for FakeRuntime {
    fn can_execute(&self) -> bool { *self.can_execute.lock().unwrap() }
    fn is_agent_gone(&self, agent_id: &str) -> bool { self.gone.lock().unwrap().contains(agent_id) }
    fn is_group_session(&self, agent_id: &str) -> Result<bool, String> { Ok(self.group.lock().unwrap().contains(agent_id)) }
    fn cloud_watch_is_armed(&self, agent_id: &str, work_id: &str) -> bool {
        let before = self.before_cloud_armed_check.lock().unwrap().take();
        if let Some(callback) = before {
            callback();
        }
        let armed = self.armed.lock().unwrap().contains(&(agent_id.into(), work_id.into()));
        if let Some(callback) = self.after_cloud_armed_check.lock().unwrap().take() {
            callback();
        }
        armed
    }
    fn watch_cloud_agent(&self, marker: &DurablePendingWakeMarker) -> Result<(), String> {
        let callback = self.admit_cloud_watch.lock().unwrap().clone();
        let admitted = match callback {
            Some(callback) => callback(marker)?,
            None => self.pending_store.as_ref().is_some_and(|store| {
                store.replace_if_matches(marker, marker.clone())
            }),
        };
        if !admitted {
            return Ok(());
        }
        self.cloud_watches.lock().unwrap().push((marker.agent_id.clone(), marker.work_id.clone()));
        if !*self.cloud_watch_finishes.lock().unwrap() {
            self.armed.lock().unwrap().insert((marker.agent_id.clone(), marker.work_id.clone()));
        }
        Ok(())
    }
    fn watch_background_shell(&self, agent_id: &str, work_id: &str, _title: Option<&str>, _quiet: Option<&QuietWakeOrigin>) -> Result<(), String> {
        self.shell_watches.lock().unwrap().push((agent_id.into(), work_id.into()));
        Ok(())
    }
    fn deliver_recreate_interrupted_shell_notice(&self, marker: &DurablePendingWakeMarker) -> Result<(), String> {
        self.interrupted.lock().unwrap().push(marker.work_id.clone());
        Ok(())
    }
    fn revive_lost_subagent(&self, wake: LostSubagentWake) -> Result<(), String> {
        self.revived.lock().unwrap().push(wake);
        Ok(())
    }
    fn revive_channel_delivery_failure(&self, _marker: &DurablePendingWakeMarker) -> Result<(), String> {
        Ok(())
    }
    fn emit_async_tasks_for_agent(&self, agent_id: &str) { self.emitted.lock().unwrap().push(agent_id.into()); }
    fn report_pending_wake(&self, report: PendingWakeReport) { self.reports.lock().unwrap().push(report); }
}

struct CloudRecoveryRuntime {
    store: SandPendingWakeStore,
    watches: RunnerCloudAgentWatches,
    before_cloud_armed_check: Mutex<Option<Box<dyn FnOnce() + Send>>>,
    admissions: Mutex<Vec<bool>>,
}

impl PendingWakeRuntimePort for CloudRecoveryRuntime {
    fn can_execute(&self) -> bool { true }
    fn is_agent_gone(&self, _agent_id: &str) -> bool { false }
    fn is_group_session(&self, _agent_id: &str) -> Result<bool, String> { Ok(false) }
    fn cloud_watch_is_armed(&self, agent_id: &str, work_id: &str) -> bool {
        let before = self.before_cloud_armed_check.lock().unwrap().take();
        if let Some(callback) = before {
            callback();
        }
        self.watches.is_cloud_watch_armed(agent_id, work_id)
    }
    fn watch_cloud_agent(&self, marker: &DurablePendingWakeMarker) -> Result<(), String> {
        let quiet_origin = marker.quiet_origin.as_ref()
            .map(serde_json::to_value).transpose().map_err(|error| error.to_string())?;
        self.watches.watch_cloud_agent_if_admitted(
            &marker.agent_id,
            &marker.work_id,
            CloudAgentWatchOptions::new(quiet_origin, false),
            &|pending| {
                let admitted = self.store.replace_if_matches(marker, cloud_pending_marker(pending));
                self.admissions.lock().unwrap().push(admitted);
                admitted
            },
        );
        Ok(())
    }
    fn watch_background_shell(&self, _: &str, _: &str, _: Option<&str>, _: Option<&QuietWakeOrigin>) -> Result<(), String> {
        panic!("cloud-only recovery fixture")
    }
    fn deliver_recreate_interrupted_shell_notice(&self, _: &DurablePendingWakeMarker) -> Result<(), String> {
        panic!("cloud-only recovery fixture")
    }
    fn revive_lost_subagent(&self, _: LostSubagentWake) -> Result<(), String> {
        panic!("cloud-only recovery fixture")
    }
    fn revive_channel_delivery_failure(&self, _: &DurablePendingWakeMarker) -> Result<(), String> {
        panic!("cloud-only recovery fixture")
    }
    fn emit_async_tasks_for_agent(&self, _: &str) {}
    fn report_pending_wake(&self, _: PendingWakeReport) {}
}

fn cloud_pending_marker(pending: &CloudAgentPendingWatch) -> DurablePendingWakeMarker {
    DurablePendingWakeMarker {
        agent_id: pending.parent_agent_id.clone(),
        kind: PendingWakeKind::CloudAgent,
        work_id: pending.work_id.clone(),
        marked_at_ms: pending.marked_at_ms,
        quiet_origin: pending.quiet_origin.as_ref().and_then(coerce_quiet_origin),
        title: Some(pending.title.clone()),
        subagent_type: Some("cursor-agent".into()),
        interrupted_by_recreate: false,
        delivery_operation_id: None,
        address_token: None,
        failure_reason: None,
        session_key: None,
    }
}

fn root() -> PathBuf {
    let path=std::env::temp_dir().join(format!("pending-wake-rearm-{}",Uuid::new_v4()));
    std::fs::create_dir_all(&path).unwrap();
    path
}
fn marker(agent:&str, kind:PendingWakeKind, work:&str, marked:f64) -> DurablePendingWakeMarker {
    DurablePendingWakeMarker {
        agent_id: agent.into(), kind, work_id: work.into(), marked_at_ms: marked,
        quiet_origin: None, title: Some("work".into()), subagent_type: Some("task".into()),
        interrupted_by_recreate: false,
        delivery_operation_id: None,
        address_token: None,
        failure_reason: None,
        session_key: None,
    }
}

#[test]
fn persists_clears_and_disarms_with_reference_telemetry() {
    let dir=root();
    let store=SandPendingWakeStore::new(&dir);
    let runtime=Arc::new(FakeRuntime::with_store(store.clone()));
    *runtime.can_execute.lock().unwrap()=true;
    let service=PendingWakeRearm::new(Some(store.clone()), runtime.clone());
    assert!(service.persist_pending_wake(marker("a",PendingWakeKind::Shell,"s1",10.0)));
    assert!(store.has_pending("a",PendingWakeKind::Shell,"s1"));
    service.clear_settled_pending_wake("a",PendingWakeKind::Shell,"s1");
    assert!(!store.has_pending("a",PendingWakeKind::Shell,"s1"));
    assert_eq!(runtime.emitted.lock().unwrap().as_slice(), &["a".to_string()]);

    service.persist_pending_wake(marker("a",PendingWakeKind::Shell,"s2",20.0));
    service.disarm_pending_wake("a",PendingWakeKind::Shell,"s2");
    let reports=runtime.reports.lock().unwrap();
    assert!(reports.iter().any(|r| r.outcome=="settled" && r.work_id=="s1" && r.reason.is_none()));
    assert!(reports.iter().any(|r| r.outcome=="settled" && r.work_id=="s2" && r.reason.as_deref()==Some("aborted")));
    drop(reports);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn rearm_prunes_stale_preserves_gone_and_restores_each_runtime_kind() {
    let dir=root();
    let store=SandPendingWakeStore::new(&dir);
    let runtime=Arc::new(FakeRuntime::with_store(store.clone()));
    *runtime.can_execute.lock().unwrap()=true;
    runtime.gone.lock().unwrap().insert("gone".into());

    store.mark_pending(marker("stale",PendingWakeKind::Shell,"old",0.0));
    store.mark_pending(marker("gone",PendingWakeKind::Shell,"held",PENDING_WAKE_STALE_MAX_AGE_MS+10.0));
    store.mark_pending(marker("cloud",PendingWakeKind::CloudAgent,"bc1",PENDING_WAKE_STALE_MAX_AGE_MS+20.0));
    store.mark_pending(marker("shell",PendingWakeKind::Shell,"sh1",PENDING_WAKE_STALE_MAX_AGE_MS+30.0));
    store.mark_pending(marker("parent",PendingWakeKind::Subagent,"sub1",PENDING_WAKE_STALE_MAX_AGE_MS+40.0));

    let now=PENDING_WAKE_STALE_MAX_AGE_MS+100.0;
    let service=PendingWakeRearm::new(Some(store.clone()),runtime.clone()).with_now(Arc::new(move||now));
    service.rearm_pending_wakes();

    assert!(!store.has_pending("stale",PendingWakeKind::Shell,"old"));
    assert!(store.has_pending("gone",PendingWakeKind::Shell,"held"));
    assert_eq!(runtime.cloud_watches.lock().unwrap().as_slice(), &[("cloud".into(),"bc1".into())]);
    assert_eq!(runtime.shell_watches.lock().unwrap().as_slice(), &[("shell".into(),"sh1".into())]);
    assert_eq!(runtime.revived.lock().unwrap().len(),1);
    assert!(store.has_pending("parent",PendingWakeKind::Subagent,"sub1"));

    let reports=runtime.reports.lock().unwrap();
    assert!(reports.iter().any(|r| r.outcome=="pruned" && r.work_id=="old" && r.reason.as_deref()==Some("stale")));
    assert!(reports.iter().any(|r| r.outcome=="rearm_skipped" && r.work_id=="held" && r.reason.as_deref()==Some("agent_gone")));
    assert!(reports.iter().any(|r| r.outcome=="rearmed" && r.work_id=="sub1" && r.reason.as_deref()==Some("interrupted_completion")));
    drop(reports);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn interrupted_recreate_shell_uses_resume_notice_and_group_session_clears_marker() {
    let dir=root();
    let store=SandPendingWakeStore::new(&dir);
    let runtime=Arc::new(FakeRuntime::with_store(store.clone()));
    *runtime.can_execute.lock().unwrap()=true;

    let mut direct=marker("a",PendingWakeKind::Shell,"resume",100.0);
    direct.interrupted_by_recreate=true;
    store.mark_pending(direct);
    let service=PendingWakeRearm::new(Some(store.clone()),runtime.clone()).with_now(Arc::new(||200.0));
    service.rearm_pending_wakes();
    assert_eq!(runtime.interrupted.lock().unwrap().as_slice(), &["resume".to_string()]);

    let mut grouped=marker("g",PendingWakeKind::Shell,"grouped",100.0);
    grouped.interrupted_by_recreate=true;
    store.mark_pending(grouped);
    runtime.group.lock().unwrap().insert("g".into());
    service.rearm_pending_wakes();
    assert!(!store.has_pending("g",PendingWakeKind::Shell,"grouped"));
    assert!(runtime.reports.lock().unwrap().iter().any(|r| r.work_id=="grouped" && r.reason.as_deref()==Some("group_session")));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn enqueue_refuses_gone_agents_and_appends_for_live_agents() {
    let runtime=Arc::new(FakeRuntime::default());
    runtime.gone.lock().unwrap().insert("gone".into());
    let service=PendingWakeRearm::new(None,runtime);
    let mut q:HashMap<String,Vec<i32>>=HashMap::new();
    assert!(!service.enqueue_pending_wake(&mut q,"gone",vec![1]));
    assert!(service.enqueue_pending_wake(&mut q,"live",vec![1,2]));
    assert!(service.enqueue_pending_wake(&mut q,"live",vec![3]));
    assert_eq!(q["live"],vec![1,2,3]);
}

#[test]
fn recreate_carry_filters_before_persisting_and_reports_frozen_outcomes() {
    let dir = root();
    let store = SandPendingWakeStore::new(&dir);
    let runtime = Arc::new(FakeRuntime::with_store(store.clone()));
    let service = PendingWakeRearm::new(Some(store.clone()), runtime.clone())
        .with_now(Arc::new(|| 500.0));

    let live_cloud = marker("cloud", PendingWakeKind::CloudAgent, "cloud-1", 100.0);
    let live_shell = marker("shell", PendingWakeKind::Shell, "shell-1", 110.0);
    let local = marker("local", PendingWakeKind::CloudAgent, "local-1", 120.0);
    let gone = marker("gone", PendingWakeKind::CloudAgent, "gone-1", 130.0);
    let group = marker("group", PendingWakeKind::CloudAgent, "group-1", 140.0);
    let subagent = marker("parent", PendingWakeKind::Subagent, "sub-1", 150.0);
    let rearmed_cloud = DurablePendingWakeMarker {
        marked_at_ms: 500.0,
        title: Some("Cloud agent cloud-1".into()),
        subagent_type: Some("cursor-agent".into()),
        ..live_cloud.clone()
    };
    let callback_store = store.clone();
    let callback_marker = rearmed_cloud.clone();
    *runtime.admit_cloud_watch.lock().unwrap() = Some(Arc::new(move |observed| {
        // Recovery transfers only the exact observed marker before awaiting.
        Ok(callback_store.replace_if_matches(observed, callback_marker.clone()))
    }));
    store.mark_pending(local.clone());
    runtime.gone.lock().unwrap().insert("gone".into());
    runtime.group.lock().unwrap().insert("group".into());

    let carried = vec![
        serde_json::to_value(&live_cloud).unwrap(),
        serde_json::to_value(&live_shell).unwrap(),
        serde_json::to_value(&local).unwrap(),
        serde_json::to_value(&gone).unwrap(),
        serde_json::to_value(&group).unwrap(),
        serde_json::to_value(&subagent).unwrap(),
    ];

    assert_eq!(service.restore_recreate_carried_pending_wakes(&carried), 0);
    assert!(!store.has_pending("cloud", PendingWakeKind::CloudAgent, "cloud-1"));
    *runtime.can_execute.lock().unwrap() = true;

    assert_eq!(service.restore_recreate_carried_pending_wakes(&carried), 2);
    assert!(store.has_pending("cloud", PendingWakeKind::CloudAgent, "cloud-1"));
    assert!(store.list_pending().contains(&rearmed_cloud));
    assert!(store.has_pending("shell", PendingWakeKind::Shell, "shell-1"));
    assert!(!store.has_pending("gone", PendingWakeKind::CloudAgent, "gone-1"));
    assert!(!store.has_pending("group", PendingWakeKind::CloudAgent, "group-1"));
    assert!(!store.has_pending("parent", PendingWakeKind::Subagent, "sub-1"));
    assert_eq!(runtime.cloud_watches.lock().unwrap().as_slice(), &[("cloud".into(), "cloud-1".into())]);
    assert_eq!(runtime.interrupted.lock().unwrap().as_slice(), &["shell-1".to_string()]);

    let reports = runtime.reports.lock().unwrap();
    assert!(reports.iter().any(|r| r.work_id == "local-1" && r.outcome == "rearm_skipped" && r.reason.as_deref() == Some("locally_owned")));
    assert!(reports.iter().any(|r| r.work_id == "gone-1" && r.outcome == "rearm_skipped" && r.reason.as_deref() == Some("agent_gone")));
    assert!(reports.iter().any(|r| r.work_id == "group-1" && r.outcome == "rearm_skipped" && r.reason.as_deref() == Some("group_session")));
    assert!(reports.iter().any(|r| r.work_id == "cloud-1" && r.outcome == "carried"));
    assert!(reports.iter().any(|r| r.work_id == "cloud-1" && r.outcome == "rearmed" && r.reason.as_deref() == Some("recreate_carry")));
    assert!(reports.iter().any(|r| r.work_id == "shell-1" && r.outcome == "carried"));
    assert!(reports.iter().any(|r| r.work_id == "shell-1" && r.outcome == "dropped_with_notice"));
    drop(reports);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cloud_wake_survives_repeated_rearm_and_store_reopen_until_settlement() {
    let dir = root();
    let store = SandPendingWakeStore::new(&dir);
    let runtime = Arc::new(FakeRuntime::with_store(store.clone()));
    *runtime.can_execute.lock().unwrap() = true;
    let pending = marker("cloud", PendingWakeKind::CloudAgent, "cloud-1", 100.0);
    assert!(store.mark_pending(pending.clone()));
    let service = PendingWakeRearm::new(Some(store.clone()), runtime.clone())
        .with_now(Arc::new(|| 500.0));

    service.rearm_pending_wakes();
    assert_eq!(store.list_pending(), vec![pending.clone()]);
    service.rearm_pending_wakes();
    assert_eq!(store.list_pending(), vec![pending.clone()]);
    assert_eq!(
        runtime.cloud_watches.lock().unwrap().as_slice(),
        &[("cloud".into(), "cloud-1".into())],
        "an armed watch is reused without admitting duplicate work",
    );

    let reopened = SandPendingWakeStore::new(&dir);
    assert_eq!(reopened.list_pending(), vec![pending.clone()]);
    let recovery = PendingWakeRearm::new(Some(reopened.clone()), runtime.clone())
        .with_now(Arc::new(|| 600.0));
    recovery.rearm_pending_wakes();
    assert_eq!(reopened.list_pending(), vec![pending]);
    assert_eq!(runtime.cloud_watches.lock().unwrap().len(), 1);

    recovery.clear_settled_pending_wake("cloud", PendingWakeKind::CloudAgent, "cloud-1");
    assert!(reopened.list_pending().is_empty());
    assert!(SandPendingWakeStore::new(&dir).list_pending().is_empty());
    assert!(runtime.reports.lock().unwrap().iter().any(|report| {
        report.work_id == "cloud-1" && report.outcome == "settled"
    }));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn failed_cloud_rearm_preserves_the_observed_marker_for_retry() {
    let dir = root();
    let store = SandPendingWakeStore::new(&dir);
    let runtime = Arc::new(FakeRuntime::with_store(store.clone()));
    *runtime.can_execute.lock().unwrap() = true;
    *runtime.admit_cloud_watch.lock().unwrap() =
        Some(Arc::new(|_| Err("cloud watch admission failed".into())));
    let pending = marker("cloud", PendingWakeKind::CloudAgent, "cloud-1", 100.0);
    assert!(store.mark_pending(pending.clone()));
    let service = PendingWakeRearm::new(Some(store.clone()), runtime.clone())
        .with_now(Arc::new(|| 500.0));

    service.rearm_pending_wakes();

    assert_eq!(SandPendingWakeStore::new(&dir).list_pending(), vec![pending]);
    assert!(!runtime.cloud_watch_is_armed("cloud", "cloud-1"));
    assert!(runtime.reports.lock().unwrap().iter().any(|report| {
        report.work_id == "cloud-1"
            && report.outcome == "rearm_failed"
            && report.reason.as_deref() == Some("error")
    }));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cloud_rearm_preserves_an_admitted_marker_on_success_and_failure() {
    for fail_watch in [false, true] {
        let dir = root();
        let store = SandPendingWakeStore::new(&dir);
        let runtime = Arc::new(FakeRuntime::with_store(store.clone()));
        *runtime.can_execute.lock().unwrap() = true;
        let pending = marker("cloud", PendingWakeKind::CloudAgent, "cloud-1", 100.0);
        let replacement = DurablePendingWakeMarker {
            marked_at_ms: 200.0,
            title: Some("Cloud agent cloud-1".into()),
            subagent_type: Some("cursor-agent".into()),
            ..pending.clone()
        };
        assert!(store.mark_pending(pending));
        let callback_store = store.clone();
        let callback_marker = replacement.clone();
        *runtime.admit_cloud_watch.lock().unwrap() = Some(Arc::new(move |observed| {
            assert!(callback_store.replace_if_matches(observed, callback_marker.clone()));
            if fail_watch {
                Err("cloud watch admission failed after replacement".into())
            } else {
                Ok(true)
            }
        }));
        let service = PendingWakeRearm::new(Some(store.clone()), runtime.clone())
            .with_now(Arc::new(|| 500.0));

        service.rearm_pending_wakes();

        assert_eq!(
            SandPendingWakeStore::new(&dir).list_pending(),
            vec![replacement.clone()],
        );
        let expected_outcome = if fail_watch { "rearm_failed" } else { "rearmed" };
        assert!(runtime.reports.lock().unwrap().iter().any(|report| {
            report.work_id == "cloud-1" && report.outcome == expected_outcome
        }));
        if !fail_watch {
            service.rearm_pending_wakes();
            assert_eq!(store.list_pending(), vec![replacement]);
            assert_eq!(runtime.cloud_watches.lock().unwrap().len(), 1);
        }
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[test]
fn cloud_rearm_rejects_a_marker_replaced_after_the_durable_snapshot() {
    let dir = root();
    let store = SandPendingWakeStore::new(&dir);
    let runtime = Arc::new(FakeRuntime::with_store(store.clone()));
    *runtime.can_execute.lock().unwrap() = true;
    let pending = marker("cloud", PendingWakeKind::CloudAgent, "cloud-1", 100.0);
    let replacement = DurablePendingWakeMarker { marked_at_ms: 200.0, ..pending.clone() };
    assert!(store.mark_pending(pending));
    let callback_store = store.clone();
    let callback_marker = replacement.clone();
    *runtime.before_cloud_armed_check.lock().unwrap() = Some(Arc::new(move || {
        // A different admission wins after recovery has captured its older row.
        assert!(callback_store.mark_pending(callback_marker.clone()));
    }));
    let service = PendingWakeRearm::new(Some(store.clone()), runtime.clone())
        .with_now(Arc::new(|| 500.0));

    service.rearm_pending_wakes();

    assert_eq!(store.list_pending(), vec![replacement.clone()]);
    assert_eq!(SandPendingWakeStore::new(&dir).list_pending(), vec![replacement]);
    assert!(!runtime.cloud_watch_is_armed("cloud", "cloud-1"));
    assert!(runtime.cloud_watches.lock().unwrap().is_empty());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cloud_completion_after_snapshot_before_first_armed_check_cannot_start_another_worker() {
    let dir = root();
    let store = SandPendingWakeStore::new(&dir);
    let (started_tx, started_rx) = mpsc::channel::<String>();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let release_rx = Mutex::new(release_rx);
    let (settled_tx, settled_rx) = mpsc::channel::<()>();
    let (changed_tx, changed_rx) = mpsc::channel::<()>();
    let fresh_pending = Arc::new(Mutex::new(0usize));
    let watches = RunnerCloudAgentWatches::new(
        Arc::new(move |work_id, _| {
            started_tx.send(work_id.to_string()).unwrap();
            release_rx.lock().unwrap().recv_timeout(Duration::from_secs(2)).unwrap();
            CloudAgentWatchOutcome { status: "completed".into(), text: "finished".into() }
        }),
        Some({
            let store = store.clone();
            let fresh_pending = Arc::clone(&fresh_pending);
            Arc::new(move |pending| {
                *fresh_pending.lock().unwrap() += 1;
                assert!(store.mark_pending(cloud_pending_marker(pending)));
            })
        }),
        Some({
            let store = store.clone();
            Arc::new(move |completion| {
                let completed = cloud_pending_marker(&CloudAgentPendingWatch {
                    parent_agent_id: completion.parent_agent_id,
                    work_id: completion.work_id,
                    title: completion.title,
                    marked_at_ms: completion.marked_at_ms,
                    quiet_origin: completion.quiet_origin,
                });
                assert!(store.clear_if_matches(&completed));
                settled_tx.send(()).unwrap();
            })
        }),
        Some(Arc::new(move |_| changed_tx.send(()).unwrap())),
    );
    assert!(watches.watch_cloud_agent("cloud", "cloud-1", CloudAgentWatchOptions::new(None, false)));
    assert_eq!(started_rx.recv_timeout(Duration::from_secs(2)).unwrap(), "cloud-1");
    changed_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    let persisted = store.list_pending();
    assert_eq!(persisted.len(), 1);
    let now_ms = persisted[0].marked_at_ms + 1.0;
    let runtime = Arc::new(CloudRecoveryRuntime {
        store: store.clone(),
        watches: watches.clone(),
        before_cloud_armed_check: Mutex::new(Some(Box::new({
            let store = store.clone();
            move || {
                // rearm_pending_wakes has captured its durable snapshot, but
                // has not yet observed the watch map. Force terminal settlement.
                release_tx.send(()).unwrap();
                settled_rx.recv_timeout(Duration::from_secs(2)).unwrap();
                changed_rx.recv_timeout(Duration::from_secs(2)).unwrap();
                assert!(store.list_pending().is_empty());
            }
        }))),
        admissions: Mutex::new(Vec::new()),
    });
    let service = PendingWakeRearm::new(Some(store.clone()), runtime.clone())
        .with_now(Arc::new(move || now_ms));

    service.rearm_pending_wakes();

    assert_eq!(*runtime.admissions.lock().unwrap(), vec![false]);
    assert_eq!(*fresh_pending.lock().unwrap(), 1, "recovery must not republish fresh pending work");
    assert!(store.list_pending().is_empty());
    assert!(SandPendingWakeStore::new(&dir).list_pending().is_empty());
    assert!(!watches.is_cloud_watch_armed("cloud", "cloud-1"));
    assert!(watches.async_task_snapshots("cloud").is_empty());
    drop(service);
    drop(runtime);
    drop(watches);
    assert_eq!(
        started_rx.recv_timeout(Duration::from_secs(2)),
        Err(mpsc::RecvTimeoutError::Disconnected),
        "the settled snapshot must not admit a replacement completion worker",
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cloud_completion_after_armed_check_is_not_resurrected_by_rearm() {
    let dir = root();
    let store = SandPendingWakeStore::new(&dir);
    let runtime = Arc::new(FakeRuntime::with_store(store.clone()));
    *runtime.can_execute.lock().unwrap() = true;
    runtime.armed.lock().unwrap().insert(("cloud".into(), "cloud-1".into()));
    let pending = marker("cloud", PendingWakeKind::CloudAgent, "cloud-1", 100.0);
    assert!(store.mark_pending(pending.clone()));
    let callback_store = store.clone();
    let callback_runtime = Arc::downgrade(&runtime);
    *runtime.after_cloud_armed_check.lock().unwrap() = Some(Arc::new(move || {
        // Completion wins after the rearm read captured an armed watch.
        callback_runtime
            .upgrade()
            .unwrap()
            .armed
            .lock()
            .unwrap()
            .remove(&("cloud".into(), "cloud-1".into()));
        assert!(callback_store.clear_if_matches(&pending));
    }));
    let service = PendingWakeRearm::new(Some(store.clone()), runtime.clone())
        .with_now(Arc::new(|| 500.0));

    service.rearm_pending_wakes();

    assert!(store.list_pending().is_empty());
    assert!(SandPendingWakeStore::new(&dir).list_pending().is_empty());
    assert!(!runtime.cloud_watch_is_armed("cloud", "cloud-1"));
    assert!(runtime.cloud_watches.lock().unwrap().is_empty());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cloud_completion_during_watch_registration_is_not_resurrected_as_failure() {
    let dir = root();
    let store = SandPendingWakeStore::new(&dir);
    let runtime = Arc::new(FakeRuntime::with_store(store.clone()));
    *runtime.can_execute.lock().unwrap() = true;
    *runtime.cloud_watch_finishes.lock().unwrap() = true;
    let pending = marker("cloud", PendingWakeKind::CloudAgent, "cloud-1", 100.0);
    let replacement = DurablePendingWakeMarker {
        marked_at_ms: 200.0,
        ..pending.clone()
    };
    assert!(store.mark_pending(pending));
    let callback_store = store.clone();
    *runtime.admit_cloud_watch.lock().unwrap() = Some(Arc::new(move |observed| {
        // Recovery transfers its marker, then settles before returning.
        assert!(callback_store.replace_if_matches(observed, replacement.clone()));
        assert!(callback_store.clear_if_matches(&replacement));
        Ok(true)
    }));
    let service = PendingWakeRearm::new(Some(store.clone()), runtime.clone())
        .with_now(Arc::new(|| 500.0));

    service.rearm_pending_wakes();

    assert!(store.list_pending().is_empty());
    assert!(SandPendingWakeStore::new(&dir).list_pending().is_empty());
    assert!(!runtime.cloud_watch_is_armed("cloud", "cloud-1"));
    assert_eq!(runtime.cloud_watches.lock().unwrap().len(), 1);
    assert!(runtime.reports.lock().unwrap().iter().any(|report| {
        report.work_id == "cloud-1" && report.outcome == "rearm_failed"
    }));
    std::fs::remove_dir_all(dir).unwrap();
}
