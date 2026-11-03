use super::*;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Weak;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde_json::json;

use super::super::mcp_service::{PluginSkillsPort, PluginUninstallIntent, PluginUninstallSettlement};
use super::super::plugin_skills_cache::get_plugin_skills_cache_path;

const DEADLINE: Duration = Duration::from_secs(5);

// These private channel checkpoints pause actual service boundaries. Shipping
// builds contain neither the hooks nor a callback under the publication mutex.
#[derive(Default)]
pub(super) struct ServiceTestHooks {
    pub(super) after_refresh_reservation: Option<Arc<BoundaryPause>>,
    pub(super) before_auth_outcome: Option<Arc<BoundaryPause>>,
    pub(super) after_rename: Option<Arc<BoundaryPause>>,
    pub(super) after_commit: Option<Arc<BoundaryPause>>,
    pub(super) prune_locked: Option<Arc<BoundaryPause>>,
    pub(super) coalesced_result: Option<Arc<BoundaryPause>>,
}

pub(super) struct BoundaryPause {
    visit: usize,
    calls: AtomicUsize,
    entered: Sender<()>,
    release: Mutex<Receiver<()>>,
}

pub(super) fn pause_at(pause: &Option<Arc<BoundaryPause>>) {
    if let Some(pause) = pause {
        if pause.calls.fetch_add(1, Ordering::SeqCst) + 1 == pause.visit {
            pause.entered.send(()).unwrap();
            pause.release.lock().unwrap().recv_timeout(DEADLINE).unwrap();
        }
    }
}

pub(super) struct BoundaryGate {
    pub(super) pause: Arc<BoundaryPause>,
    entered: Receiver<()>,
    pub(super) release: Sender<()>,
}

impl BoundaryGate {
    pub(super) fn at_visit(visit: usize) -> Self {
        let (entered_tx, entered) = mpsc::channel();
        let (release, release_rx) = mpsc::channel();
        Self {
            pause: Arc::new(BoundaryPause {
                visit,
                calls: AtomicUsize::new(0),
                entered: entered_tx,
                release: Mutex::new(release_rx),
            }),
            entered,
            release,
        }
    }

    pub(super) fn wait(&self) {
        self.entered.recv_timeout(DEADLINE).expect("service boundary not reached");
    }
}

pub(super) enum LoadStep {
    Loaded(LoadedPlugins),
    Failed,
    Panicked,
}

pub(super) struct GateLoader {
    entered: Sender<usize>,
    release: Mutex<Receiver<LoadStep>>,
    pub(super) calls: AtomicUsize,
    active: AtomicUsize,
    max_active: AtomicUsize,
}

struct ActiveLoad<'a>(&'a AtomicUsize);
impl Drop for ActiveLoad<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

impl PluginSkillsLoader for GateLoader {
    fn load(&self) -> Result<LoadedPlugins, String> {
        let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        let _active = ActiveLoad(&self.active);
        self.max_active.fetch_max(active, Ordering::SeqCst);
        let call = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
        self.entered.send(call).unwrap();
        // Release this fixture lock before deliberately unwinding.
        let step = self.release.lock().unwrap().recv_timeout(DEADLINE).unwrap();
        match step {
            LoadStep::Loaded(loaded) => Ok(loaded),
            LoadStep::Failed => Err("fixture loader failure".into()),
            LoadStep::Panicked => panic!("fixture loader panic"),
        }
    }
}

pub(super) struct TempRoot(pub(super) PathBuf);
impl TempRoot {
    pub(super) fn new() -> Self {
        let root = std::env::temp_dir().join(format!("plugin-sync-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(root.join("plugins/cache/market/plugin/skills/test")).unwrap();
        Self(root)
    }

    fn loaded(&self, version: &str) -> LoadedPlugins {
        LoadedPlugins {
            plugins: vec![InstalledPlugin {
                identifier: PluginIdentifier {
                    source: "cursor-third-party".into(),
                    name: "plugin".into(),
                    plugin_db_id: Some("9".into()),
                    version: Some(version.into()),
                },
                display_name: None,
                load_error: None,
                install_path: self.0.join("plugins/cache/market/plugin"),
                skills: vec![InstalledSkill {
                    name: Some("Test".into()),
                    description: None,
                    path: "skills/test/SKILL.md".into(),
                }],
            }],
            auth_blocked: vec![],
            listed_plugin_ids: vec!["9".into()],
            listed_cache_keys: vec![ListedCacheKey {
                marketplace_slug: "market".into(),
                plugin_id: "plugin".into(),
            }],
            publisher_facts: HashMap::new(),
            current_user_id: Some(12),
        }
    }

    pub(super) fn cache_path(&self) -> PathBuf {
        get_plugin_skills_cache_path(get_plugin_skills_dir(&self.0))
    }

    pub(super) fn uninstall_state_path(&self) -> PathBuf {
        get_plugin_skills_dir(&self.0).join("uninstall-state.json")
    }

    pub(super) fn uninstall_version_path(&self) -> PathBuf {
        get_plugin_skills_dir(&self.0).join("uninstall-state.version")
    }

    fn cache_fixture(&self) -> Value {
        let loaded = self.loaded("retained");
        json!({
            "cacheFormatVersion": 1,
            "fetchedAt": 1_000,
            "currentUserId": 12,
            "skills": plugin_contents_to_skill_records(&loaded.plugins, &loaded.publisher_facts),
            "authBlocked": [],
        })
    }

    fn write_cache(&self, value: &Value) {
        let path = self.cache_path();
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
    }

    pub(super) fn sentinel(&self) -> PathBuf {
        let path = self.0.join("plugins/cache/market/unlisted/keep.txt");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "must survive an incomplete pass").unwrap();
        path
    }

    pub(super) fn staged_files(&self) -> Vec<PathBuf> {
        fs::read_dir(get_plugin_skills_dir(&self.0))
            .unwrap()
            .map(Result::unwrap)
            .filter(|entry| entry.file_name().to_string_lossy().ends_with(".tmp"))
            .map(|entry| entry.path())
            .collect()
    }
}
impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub(super) struct Call {
    result: Receiver<Result<SyncResult, ()>>,
    thread: JoinHandle<()>,
}
impl Call {
    pub(super) fn start(service: &Arc<SandPluginSkillsService>, trigger: &'static str) -> Self {
        let service = Arc::clone(service);
        let (tx, result) = mpsc::channel();
        let thread = thread::spawn(move || {
            let outcome = catch_unwind(AssertUnwindSafe(|| service.sync(trigger))).map_err(|_| ());
            let _ = tx.send(outcome);
        });
        Self { result, thread }
    }

    pub(super) fn assert_waiting(&self) {
        assert!(matches!(self.result.try_recv(), Err(TryRecvError::Empty)));
    }

    pub(super) fn outcome(self) -> Result<SyncResult, ()> {
        let result = self
            .result
            .recv_timeout(DEADLINE)
            .expect("sync did not settle");
        self.thread.join().unwrap();
        result
    }

    pub(super) fn finish(self) -> SyncResult {
        self.outcome().expect("unexpected sync panic")
    }
}

pub(super) struct Harness {
    pub(super) root: TempRoot,
    pub(super) service: Arc<SandPluginSkillsService>,
    pub(super) loader: Arc<GateLoader>,
    entered: Receiver<usize>,
    pub(super) release: Sender<LoadStep>,
    pub(super) events: Arc<Mutex<Vec<PluginSkillsSyncEvent>>>,
}
impl Harness {
    pub(super) fn new() -> Self {
        Self::with_now(Arc::new(|| 1_000))
    }

    fn with_now(now: Arc<dyn Fn() -> i64 + Send + Sync>) -> Self {
        Self::with_hooks(now, ServiceTestHooks::default())
    }

    pub(super) fn with_hooks(now: Arc<dyn Fn() -> i64 + Send + Sync>, hooks: ServiceTestHooks) -> Self {
        let root = TempRoot::new();
        let (tx, entered) = mpsc::channel();
        let (release, rx) = mpsc::channel();
        let loader = Arc::new(GateLoader {
            entered: tx,
            release: Mutex::new(rx),
            calls: AtomicUsize::new(0),
            active: AtomicUsize::new(0),
            max_active: AtomicUsize::new(0),
        });
        let events = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&events);
        let mut service = SandPluginSkillsService::new(&root.0, loader.clone())
            .with_now(now)
            .with_reporter(Arc::new(move |event| captured.lock().unwrap().push(event)));
        service.test_hooks = hooks;
        let service = Arc::new(service);
        Self {
            root,
            service,
            loader,
            entered,
            release,
            events,
        }
    }

    pub(super) fn call(&self, trigger: &'static str) -> Call {
        Call::start(&self.service, trigger)
    }

    pub(super) fn enter(&self, pass: usize) {
        assert_eq!(
            self.entered
                .recv_timeout(DEADLINE)
                .expect("loader did not start"),
            pass
        );
    }

    pub(super) fn pending(&self, trigger: &str) {
        // Wait for actual enrollment, not an assumed OS scheduling delay.
        let until = Instant::now() + DEADLINE;
        let mut state = self.service.state.lock().unwrap();
        loop {
            if state
                .pending
                .as_ref()
                .is_some_and(|pending| pending.trigger == trigger)
            {
                return;
            }
            let remaining = until
                .checked_duration_since(Instant::now())
                .expect("caller did not enroll");
            state = self
                .service
                .changed
                .wait_timeout(state, remaining)
                .unwrap()
                .0;
        }
    }

    fn loaded(&self, version: &str) {
        self.release
            .send(LoadStep::Loaded(self.root.loaded(version)))
            .unwrap();
    }

    pub(super) fn step(&self, trigger: &'static str, step: LoadStep) -> SyncResult {
        let next = self.loader.calls.load(Ordering::SeqCst) + 1;
        let call = self.call(trigger);
        self.enter(next);
        self.release.send(step).unwrap();
        call.finish()
    }

    pub(super) fn assert_calls(&self, calls: usize) {
        assert_eq!(self.loader.calls.load(Ordering::SeqCst), calls);
        assert_eq!(self.loader.max_active.load(Ordering::SeqCst), 1);
    }
}

fn assert_version(result: SyncResult, version: &str) {
    let records = result.unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].plugin_version, version);
}

fn assert_empty(result: SyncResult) {
    assert!(result.unwrap().is_empty());
}

#[test]
fn overlapping_callers_wait_for_one_followup_and_last_trigger_wins() {
    let h = Harness::new();
    let first = h.call("startup");
    h.enter(1);
    let second = h.call("install");
    h.pending("install");
    let third = h.call("auth_change");
    h.pending("auth_change");
    second.assert_waiting();
    third.assert_waiting();
    h.loaded("v1");
    assert_version(first.finish(), "v1");
    h.enter(2);
    second.assert_waiting();
    third.assert_waiting();
    h.loaded("v2");
    assert_version(second.finish(), "v2");
    assert_version(third.finish(), "v2");
    h.assert_calls(2);
    let events = h.events.lock().unwrap();
    assert_eq!(
        events
            .iter()
            .map(|event| event.trigger.as_str())
            .collect::<Vec<_>>(),
        vec!["startup", "auth_change"]
    );
}

#[test]
fn followup_failure_reaches_every_waiter_without_replacing_active_success() {
    let h = Harness::new();
    let first = h.call("startup");
    h.enter(1);
    let second = h.call("install");
    h.pending("install");
    let third = h.call("refresh");
    h.pending("refresh");
    h.loaded("v1");
    assert_version(first.finish(), "v1");
    h.enter(2);
    h.release.send(LoadStep::Failed).unwrap();
    assert_eq!(second.finish().unwrap_err(), "fixture loader failure");
    assert_eq!(third.finish().unwrap_err(), "fixture loader failure");
    assert_version(Ok(h.service.current()), "v1");
    h.assert_calls(2);
}

#[test]
fn active_failure_does_not_cancel_a_pending_success_or_borrow_its_result() {
    let h = Harness::new();
    let first = h.call("startup");
    h.enter(1);
    let second = h.call("install");
    h.pending("install");
    h.release.send(LoadStep::Failed).unwrap();
    assert_eq!(first.finish().unwrap_err(), "fixture loader failure");
    h.enter(2);
    h.loaded("v2");
    assert_version(second.finish(), "v2");
    h.assert_calls(2);
}

#[test]
fn disposal_wakes_queued_callers_without_loading_or_overwriting_retained_cache() {
    let h = Harness::new();
    let seed = h.call("seed");
    h.enter(1);
    h.loaded("retained");
    assert_version(seed.finish(), "retained");
    let active = h.call("refresh");
    h.enter(2);
    let queued = h.call("auth_change");
    h.pending("auth_change");
    h.service.dispose();
    assert_version(queued.finish(), "retained");
    assert_version(h.call("after-dispose").finish(), "retained");
    active.assert_waiting();
    h.loaded("must-not-write");
    assert_version(active.finish(), "retained");
    assert_version(Ok(h.service.current()), "retained");
    h.assert_calls(2);
    assert_eq!(h.events.lock().unwrap().len(), 1);
}

#[test]
fn requests_during_a_followup_have_their_own_next_pass() {
    let h = Harness::new();
    let first = h.call("startup");
    h.enter(1);
    let second = h.call("install");
    h.pending("install");
    h.loaded("v1");
    assert_version(first.finish(), "v1");
    h.enter(2);
    let third = h.call("refresh");
    h.pending("refresh");
    h.loaded("v2");
    assert_version(second.finish(), "v2");
    h.enter(3);
    third.assert_waiting();
    h.loaded("v3");
    assert_version(third.finish(), "v3");
    h.assert_calls(3);
}

#[test]
fn panicking_followup_settles_shared_waiters_and_releases_future_sync() {
    let h = Harness::new();
    let first = h.call("startup");
    h.enter(1);
    let second = h.call("install");
    h.pending("install");
    let third = h.call("refresh");
    h.pending("refresh");
    h.loaded("v1");
    assert_version(first.finish(), "v1");
    h.enter(2);
    h.release.send(LoadStep::Panicked).unwrap();
    let outcomes = [second.outcome(), third.outcome()];
    assert_eq!(
        outcomes.iter().filter(|outcome| outcome.is_err()).count(),
        1
    );
    let settled = outcomes.into_iter().find_map(Result::ok).unwrap();
    assert_eq!(
        settled.unwrap_err(),
        "plugin skills sync aborted before completion"
    );
    let recovery = h.call("recovery");
    h.enter(3);
    h.loaded("v3");
    assert_version(recovery.finish(), "v3");
    h.assert_calls(3);
}

struct ReentrantLoader {
    service: Mutex<Weak<SandPluginSkillsService>>,
    loaded: LoadedPlugins,
    observed: Mutex<Option<String>>,
}
impl PluginSkillsLoader for ReentrantLoader {
    fn load(&self) -> Result<LoadedPlugins, String> {
        let service = self.service.lock().unwrap().upgrade().unwrap();
        let error = service.sync("recursive").unwrap_err();
        *self.observed.lock().unwrap() = Some(error);
        Ok(self.loaded.clone())
    }
}

#[test]
fn same_thread_recursive_sync_fails_explicitly_instead_of_waiting_on_itself() {
    let root = TempRoot::new();
    let loader = Arc::new(ReentrantLoader {
        service: Mutex::new(Weak::new()),
        loaded: root.loaded("v1"),
        observed: Mutex::new(None),
    });
    let service = Arc::new(SandPluginSkillsService::new(&root.0, loader.clone()));
    *loader.service.lock().unwrap() = Arc::downgrade(&service);
    assert_version(Call::start(&service, "startup").finish(), "v1");
    assert_eq!(
        loader.observed.lock().unwrap().as_deref(),
        Some("recursive plugin skills sync cannot wait on its own pass")
    );
    assert!(service.state.lock().unwrap().pending.is_none());
}

pub(super) fn auth_block(id: &str) -> PluginAuthBlock {
    PluginAuthBlock {
        plugin_id: id.into(),
        plugin_name: "Plugin".into(),
        marketplace_name: Some("market".into()),
    }
}

fn unmaterialized(root: &TempRoot) -> LoadedPlugins {
    let mut loaded = root.loaded("unmaterialized");
    loaded.plugins.clear();
    loaded
}

fn blocked_ids(service: &SandPluginSkillsService) -> Vec<String> {
    let mut ids = service
        .current_auth_blocked()
        .into_iter()
        .map(|block| block.plugin_id)
        .collect::<Vec<_>>();
    ids.sort();
    ids
}

#[test]
fn canonical_denial_survives_listing_and_materialization_failures_until_fresh_success() {
    let h = Harness::new();
    assert_version(h.step("seed", LoadStep::Loaded(h.root.loaded("v1"))), "v1");
    let sentinel = h.root.sentinel();
    let mut denied = unmaterialized(&h.root);
    denied.auth_blocked.push(auth_block("9"));
    assert_empty(h.step("denied", LoadStep::Loaded(denied)));
    assert!(h.service.current().is_empty());
    assert_eq!(blocked_ids(&h.service), vec!["9"]);
    let denied_bytes = fs::read(h.root.cache_path()).unwrap();

    assert_eq!(
        h.step("offline", LoadStep::Failed).unwrap_err(),
        "fixture loader failure",
    );
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), denied_bytes);
    assert_empty(h.step("missing-version", LoadStep::Loaded(unmaterialized(&h.root))));
    let mut failed = h.root.loaded("must-not-revive");
    failed.plugins[0].load_error = Some("transient checkout failure".into());
    assert_empty(h.step("failed-checkout", LoadStep::Loaded(failed)));
    assert_eq!(blocked_ids(&h.service), vec!["9"]);
    assert!(sentinel.exists());

    assert_version(
        h.step("recovered", LoadStep::Loaded(h.root.loaded("v1"))),
        "v1",
    );
    assert!(blocked_ids(&h.service).is_empty());
    assert!(!sentinel.exists());
    let events = h.events.lock().unwrap();
    assert_eq!(
        events.iter().map(|event| event.changed).collect::<Vec<_>>(),
        vec![true, true, false, false, false, true],
    );
    assert_eq!(events[2].outcome, "failed");
    assert_eq!(events[2].error_class.as_deref(), Some("load_failed"));
}

#[test]
fn successful_zero_skill_materialization_replaces_old_records_and_clears_denial() {
    let h = Harness::new();
    assert_version(h.step("seed", LoadStep::Loaded(h.root.loaded("v1"))), "v1");
    let mut empty_success = h.root.loaded("v1");
    empty_success.plugins[0].skills.clear();
    assert_empty(h.step("empty-success", LoadStep::Loaded(empty_success.clone())));
    assert!(h.service.current_index().unwrap().skills.is_empty());

    let mut denied = unmaterialized(&h.root);
    denied.auth_blocked.push(auth_block("9"));
    assert_empty(h.step("denied", LoadStep::Loaded(denied)));
    assert_empty(h.step("transient", LoadStep::Loaded(unmaterialized(&h.root))));
    assert_eq!(blocked_ids(&h.service), vec!["9"]);
    assert_empty(h.step("recovered-empty", LoadStep::Loaded(empty_success)));
    assert!(blocked_ids(&h.service).is_empty());
    assert_eq!(
        h.events
            .lock()
            .unwrap()
            .iter()
            .map(|event| event.changed)
            .collect::<Vec<_>>(),
        vec![true, true, true, false, true],
    );
}

#[test]
fn zero_skill_success_retains_actual_install_path_until_authoritative_empty() {
    let h = Harness::new();
    let mut loaded = h.root.loaded("v1");
    loaded.listed_plugin_ids = vec!["db-zero-skill".into()];
    loaded.listed_cache_keys = vec![ListedCacheKey {
        marketplace_slug: "market.place".into(),
        plugin_id: "review.tools".into(),
    }];
    let plugin = &mut loaded.plugins[0];
    plugin.identifier.name = "review.tools".into();
    plugin.identifier.plugin_db_id = Some("db-zero-skill".into());
    // The actual SDK producer oracle supplies these sanitized directory names;
    // this service fixture passes its returned path without normalizing keys.
    plugin.install_path = h
        .root
        .0
        .join("plugins/cache/market_place/review_tools/v1");
    plugin.skills.clear();
    let install_path = plugin.install_path.clone();
    let command_path = install_path.join("commands/review.md");
    let command_bytes = b"Review the current request without changing unrelated files.";
    fs::create_dir_all(command_path.parent().unwrap()).unwrap();
    fs::write(&command_path, command_bytes).unwrap();
    let sentinel_path = install_path.join(".cache-complete");
    fs::write(&sentinel_path, b"").unwrap();

    let unlisted = h
        .root
        .0
        .join("plugins/cache/market_place/unlisted/v1/commands/old.md");
    fs::create_dir_all(unlisted.parent().unwrap()).unwrap();
    fs::write(&unlisted, b"uninstalled command").unwrap();
    // An ignored source with the same ID as the successful canonical plugin
    // cannot borrow that plugin's authority to retain another install path.
    let mut unknown = plugin.clone();
    unknown.identifier.source = "unknown-source".into();
    unknown.install_path = h
        .root
        .0
        .join("plugins/cache/market_place/unknown_source/v1");
    let unknown_path = unknown.install_path.clone();
    fs::create_dir_all(&unknown_path).unwrap();
    fs::write(unknown_path.join("unowned.txt"), b"unowned bytes").unwrap();
    loaded.plugins.push(unknown);

    assert_empty(h.step("zero-skill-success", LoadStep::Loaded(loaded)));
    let committed = h.service.current_index().unwrap();
    assert!(committed.skills.is_empty());
    assert!(committed.auth_blocked.is_empty());
    assert!(h.service.current().is_empty());
    let published: Value =
        serde_json::from_slice(&fs::read(h.root.cache_path()).unwrap()).unwrap();
    assert_eq!(published["cacheFormatVersion"], json!(1));
    assert_eq!(fs::read(&command_path).unwrap(), command_bytes);
    assert_eq!(fs::read(&sentinel_path).unwrap(), b"");
    assert!(!unlisted.exists());
    assert!(!unknown_path.exists());

    let mut empty = unmaterialized(&h.root);
    empty.listed_plugin_ids.clear();
    empty.listed_cache_keys.clear();
    assert_empty(h.step("authoritative-empty", LoadStep::Loaded(empty)));
    assert!(h.service.current_index().unwrap().skills.is_empty());
    assert!(!install_path.parent().unwrap().exists());
    assert!(!command_path.exists());
    let events = h.events.lock().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].trigger, "zero-skill-success");
    assert_eq!(events[1].trigger, "authoritative-empty");
    assert!(
        events
            .iter()
            .all(|event| event.outcome == "ok" && event.skill_count == 0),
    );
    h.assert_calls(2);
}

#[test]
fn current_denial_wins_over_success_and_absent_denial_retires_without_hiding_other_plugins() {
    let h = Harness::new();
    let sentinel = h.root.sentinel();
    let mut loaded = h.root.loaded("v1");
    let mut other = loaded.plugins[0].clone();
    other.identifier.plugin_db_id = Some("10".into());
    other.install_path = h.root.0.join("plugins/cache/market/other");
    fs::create_dir_all(&other.install_path).unwrap();
    loaded.plugins.push(other.clone());
    loaded.listed_plugin_ids.push("10".into());
    loaded.listed_cache_keys.push(ListedCacheKey {
        marketplace_slug: "market".into(),
        plugin_id: "other".into(),
    });
    loaded.auth_blocked = vec![auth_block("9"), auth_block("9")];
    let records = h
        .step("conflicting-denial", LoadStep::Loaded(loaded))
        .unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].plugin_id, "10");
    assert_eq!(h.service.current(), records);
    assert_eq!(h.service.current_index().unwrap().skills, records);
    assert_eq!(blocked_ids(&h.service), vec!["9"]);
    assert!(sentinel.exists());

    let mut remaining = h.root.loaded("v1");
    remaining.plugins = vec![other];
    remaining.listed_plugin_ids = vec!["10".into()];
    remaining.listed_cache_keys[0].plugin_id = "other".into();
    assert_eq!(
        h.step("uninstalled", LoadStep::Loaded(remaining)).unwrap(),
        records,
    );
    assert!(blocked_ids(&h.service).is_empty());
    assert!(!sentinel.exists());
    assert!(!h.root.0.join("plugins/cache/market/plugin").exists());
    assert!(h.events.lock().unwrap()[1].changed);
}

#[test]
fn malformed_authoritative_loaded_snapshots_preserve_cache_and_materialized_files() {
    let h = Harness::new();
    assert_version(
        h.step("seed", LoadStep::Loaded(h.root.loaded("retained"))),
        "retained",
    );
    let bytes = fs::read(h.root.cache_path()).unwrap();
    let sentinel = h.root.sentinel();
    let cases: &[(&'static str, fn(&mut LoadedPlugins))] = &[
        ("empty-listed-id", |loaded| loaded.listed_plugin_ids[0].clear()),
        ("whitespace-listed-id", |loaded| {
            loaded.listed_plugin_ids[0] = " 9".into();
        }),
        ("duplicate-listed-id", |loaded| {
            loaded.listed_plugin_ids.push("9".into());
        }),
        ("missing-plugin-id", |loaded| {
            loaded.plugins[0].identifier.plugin_db_id = None;
        }),
        ("empty-plugin-id", |loaded| {
            loaded.plugins[0].identifier.plugin_db_id = Some(String::new());
        }),
        ("whitespace-plugin-id", |loaded| {
            loaded.plugins[0].identifier.plugin_db_id = Some("9 ".into());
        }),
        ("unlisted-plugin", |loaded| loaded.listed_plugin_ids.clear()),
        ("duplicate-plugin", |loaded| {
            loaded.plugins.push(loaded.plugins[0].clone());
        }),
        ("empty-block-id", |loaded| {
            loaded.auth_blocked.push(auth_block(""));
        }),
        ("whitespace-block-id", |loaded| {
            loaded.auth_blocked.push(auth_block("9\n"));
        }),
        ("unlisted-block", |loaded| {
            loaded.auth_blocked.push(auth_block("10"));
        }),
    ];
    for &(trigger, corrupt) in cases {
        let mut loaded = h.root.loaded("must-not-publish");
        corrupt(&mut loaded);
        let error = h.step(trigger, LoadStep::Loaded(loaded)).unwrap_err();
        assert!(
            error.starts_with("plugin skills loader returned"),
            "{trigger}: {error}",
        );
        assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes, "{trigger}");
        assert_version(Ok(h.service.current()), "retained");
        assert!(sentinel.exists(), "{trigger}");
    }
    let events = h.events.lock().unwrap();
    assert_eq!(
        events.iter().filter(|event| event.outcome == "ok").count(),
        1,
    );
    assert!(events[1..].iter().all(|event| !event.changed));
}

#[test]
fn partial_materialization_preserves_files_and_complete_or_empty_listing_may_prune() {
    let h = Harness::new();
    assert_version(
        h.step("seed", LoadStep::Loaded(h.root.loaded("retained"))),
        "retained",
    );
    let sentinel = h.root.sentinel();
    let mut partial = unmaterialized(&h.root);
    partial.listed_plugin_ids.push("10".into());
    partial.listed_cache_keys.clear();
    assert_version(h.step("partial", LoadStep::Loaded(partial)), "retained");
    assert!(sentinel.exists());
    let mut no_install_path = h.root.loaded("must-not-replace");
    no_install_path.plugins[0].install_path = PathBuf::new();
    assert_version(
        h.step("missing-path", LoadStep::Loaded(no_install_path)),
        "retained",
    );
    assert!(sentinel.exists());

    assert_version(
        h.step("complete", LoadStep::Loaded(h.root.loaded("fresh"))),
        "fresh",
    );
    assert!(!sentinel.exists());
    let sentinel = h.root.sentinel();
    let mut empty = unmaterialized(&h.root);
    empty.listed_plugin_ids.clear();
    empty.listed_cache_keys.clear();
    assert_empty(h.step("authoritative-empty", LoadStep::Loaded(empty)));
    assert!(h.service.current().is_empty());
    assert!(!sentinel.exists());
    assert!(!h.root.0.join("plugins/cache/market/plugin").exists());
}

#[test]
fn service_current_and_disposed_sync_share_canonical_admission_with_raw_diagnostics() {
    let h = Harness::new();
    let mut value = h.root.cache_fixture();
    let mut other = value["skills"][0].clone();
    other["id"] = json!("plugin-10-test");
    other["pluginId"] = json!("10");
    value["skills"].as_array_mut().unwrap().push(other);
    value["authBlocked"] = json!([auth_block("9")]);
    h.root.write_cache(&value);
    let bytes = fs::read(h.root.cache_path()).unwrap();
    let records = h.service.current();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].plugin_id, "10");
    assert_eq!(h.service.current_index().unwrap().skills.len(), 2);
    assert_eq!(blocked_ids(&h.service), vec!["9"]);
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
    h.service.dispose();
    assert_eq!(h.call("disposed").finish().unwrap(), records);
    assert_eq!(h.loader.calls.load(Ordering::SeqCst), 0);
}

#[test]
fn ambiguous_legacy_denials_never_supply_service_current_or_fallback() {
    for legacy_id in ["plugin", "9"] {
        let h = Harness::new();
        let mut value = h.root.cache_fixture();
        value.as_object_mut().unwrap().remove("cacheFormatVersion");
        value["authBlocked"] = json!([auth_block(legacy_id)]);
        h.root.write_cache(&value);
        let bytes = fs::read(h.root.cache_path()).unwrap();
        assert!(h.service.current().is_empty());
        assert_eq!(h.service.current_index().unwrap().skills.len(), 1);
        assert_eq!(blocked_ids(&h.service), vec![legacy_id]);
        assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
        assert!(h.step("offline", LoadStep::Failed).is_err());
        assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);

        assert_empty(h.step("canonical-partial", LoadStep::Loaded(unmaterialized(&h.root))));
        assert!(h.service.current_index().unwrap().skills.is_empty());
        assert!(blocked_ids(&h.service).is_empty());
        let migrated: Value =
            serde_json::from_slice(&fs::read(h.root.cache_path()).unwrap()).unwrap();
        assert_eq!(migrated["cacheFormatVersion"], json!(1));
        assert_version(
            h.step("canonical-success", LoadStep::Loaded(h.root.loaded("fresh"))),
            "fresh",
        );
    }
}

#[test]
fn legacy_cache_without_denials_keeps_eligible_fallback_during_canonical_refresh() {
    for missing_blocks in [true, false] {
        let h = Harness::new();
        let mut value = h.root.cache_fixture();
        value.as_object_mut().unwrap().remove("cacheFormatVersion");
        if missing_blocks {
            value.as_object_mut().unwrap().remove("authBlocked");
        }
        h.root.write_cache(&value);
        assert_version(Ok(h.service.current()), "retained");
        assert_version(
            h.step("canonical-partial", LoadStep::Loaded(unmaterialized(&h.root))),
            "retained",
        );
        assert!(blocked_ids(&h.service).is_empty());
    }
}

#[test]
fn invalid_cache_is_read_only_and_never_supplies_fallback() {
    let cases: &[(&'static str, fn(&mut Value))] = &[
        ("unknown-version", |value| value["cacheFormatVersion"] = json!(2)),
        ("string-version", |value| value["cacheFormatVersion"] = json!("1")),
        ("null-version", |value| value["cacheFormatVersion"] = Value::Null),
        ("missing-blocks", |value| {
            value.as_object_mut().unwrap().remove("authBlocked");
        }),
        ("null-blocks", |value| value["authBlocked"] = Value::Null),
        ("wrong-blocks-shape", |value| value["authBlocked"] = json!({})),
        ("invalid-block-entry", |value| {
            value["authBlocked"] = json!([{"pluginId": "9"}]);
        }),
        ("empty-block-id", |value| {
            value["authBlocked"] = json!([auth_block("")]);
        }),
        ("whitespace-block-id", |value| {
            value["authBlocked"] = json!([auth_block("9 ")]);
        }),
        ("legacy-invalid-blocks", |value| {
            value.as_object_mut().unwrap().remove("cacheFormatVersion");
            value["authBlocked"] = json!("not a denial list");
        }),
    ];
    for &(trigger, corrupt) in cases {
        let h = Harness::new();
        let mut value = h.root.cache_fixture();
        corrupt(&mut value);
        h.root.write_cache(&value);
        let bytes = fs::read(h.root.cache_path()).unwrap();
        let sentinel = h.root.sentinel();
        assert!(h.service.current().is_empty(), "{trigger}");
        assert!(h.service.current_index().is_none(), "{trigger}");
        assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes, "{trigger}");
        assert!(h.step(trigger, LoadStep::Failed).is_err());
        assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes, "{trigger}");
        assert!(sentinel.exists(), "{trigger}");
        assert_empty(h.step("canonical-partial", LoadStep::Loaded(unmaterialized(&h.root))));
        assert!(h.service.current_index().unwrap().skills.is_empty());
        assert!(sentinel.exists());
    }
}

#[test]
fn invalid_generated_records_fail_before_publication_without_changing_cache_or_pruning() {
    let h = Harness::new();
    assert_version(
        h.step("seed", LoadStep::Loaded(h.root.loaded("retained"))),
        "retained",
    );
    let bytes = fs::read(h.root.cache_path()).unwrap();
    let sentinel = h.root.sentinel();
    let mut opaque = h.root.loaded("must-not-publish");
    opaque.plugins[0].identifier.plugin_db_id = Some("opaque:plugin".into());
    opaque.listed_plugin_ids = vec!["opaque:plugin".into()];
    let mut relative = h.root.loaded("must-not-publish");
    relative.plugins[0].install_path = PathBuf::from("relative/plugin");
    for loaded in [opaque.clone(), relative] {
        assert!(h.step("invalid-record", LoadStep::Loaded(loaded)).is_err());
        assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
        assert_version(Ok(h.service.current()), "retained");
        assert!(sentinel.exists());
        assert!(h.root.staged_files().is_empty());
    }
    assert_eq!(h.events.lock().unwrap().len(), 1);
    // Opaque canonical IDs themselves remain valid when there is no invalid
    // generated workflow ID to persist.
    opaque.plugins[0].skills.clear();
    assert_empty(h.step("opaque-empty-success", LoadStep::Loaded(opaque)));
}

#[test]
fn staging_failure_preserves_existing_path_and_materialized_files_without_success() {
    let gate = PublicationGate::at_clock_call(2);
    let h = Harness::with_now(Arc::clone(&gate.now));
    let cache_dir = get_plugin_skills_dir(&h.root.0);
    let sentinel = h.root.sentinel();
    let active = h.call("stage-failure");
    h.enter(1);
    h.loaded("v1");
    gate.wait();
    // Refresh custody now precedes content staging. Introduce the original
    // non-directory obstruction at the actual stage's clock boundary, after
    // its settled journal is durable, so this still exercises cache staging.
    let journal = fs::read(h.root.uninstall_state_path()).unwrap();
    let preserved = h.root.0.join("preserved-plugin-skills.fixture");
    fs::rename(&cache_dir, &preserved).unwrap();
    fs::write(&cache_dir, "existing non-directory cache path").unwrap();
    let bytes = fs::read(&cache_dir).unwrap();
    gate.release.send(()).unwrap();
    assert!(active.finish().is_err());
    assert_eq!(fs::read(&cache_dir).unwrap(), bytes);
    assert_eq!(fs::read(preserved.join("uninstall-state.json")).unwrap(), journal);
    assert_eq!(fs::read(preserved.join("uninstall-state.version")).unwrap(), b"1\n");
    assert!(sentinel.exists());
    assert!(h.root.0.join("plugins/cache/market/plugin").exists());
    assert!(h.events.lock().unwrap().is_empty());
    h.assert_calls(1);
}

struct PublicationGate {
    now: Arc<dyn Fn() -> i64 + Send + Sync>,
    entered: Receiver<()>,
    release: Sender<()>,
}

impl PublicationGate {
    fn after_seed() -> Self {
        Self::at_clock_call(6)
    }

    fn at_clock_call(call: usize) -> Self {
        let (entered_tx, entered) = mpsc::channel();
        let (release, release_rx) = mpsc::channel();
        let release_rx = Mutex::new(release_rx);
        let calls = AtomicUsize::new(0);
        let now = Arc::new(move || {
            // A successful pass samples start, serialization, then duration.
            // Invalid generated records stop after serialization. The caller
            // chooses a pre-publication duration boundary, never elapsed time.
            if calls.fetch_add(1, Ordering::SeqCst) + 1 == call {
                entered_tx.send(()).unwrap();
                release_rx.lock().unwrap().recv_timeout(DEADLINE).unwrap();
            }
            1_000
        });
        Self {
            now,
            entered,
            release,
        }
    }

    fn wait(&self) {
        self.entered
            .recv_timeout(DEADLINE)
            .expect("publication boundary not reached");
    }
}

#[test]
fn disposal_after_staging_prevents_rename_pruning_and_success_and_wakes_queued_sync() {
    let gate = PublicationGate::after_seed();
    let h = Harness::with_now(Arc::clone(&gate.now));
    assert_version(
        h.step("seed", LoadStep::Loaded(h.root.loaded("retained"))),
        "retained",
    );
    let bytes = fs::read(h.root.cache_path()).unwrap();
    let sentinel = h.root.sentinel();
    let active = h.call("refresh");
    h.enter(2);
    h.loaded("must-not-publish");
    gate.wait();
    assert_eq!(h.root.staged_files().len(), 1);
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
    let queued = h.call("queued");
    h.pending("queued");

    let service = Arc::clone(&h.service);
    let (tx, disposed) = mpsc::channel();
    let disposing = thread::spawn(move || {
        service.dispose();
        tx.send(()).unwrap();
    });
    disposed
        .recv_timeout(DEADLINE)
        .expect("staging callback held a service lock");
    disposing.join().unwrap();
    assert_version(queued.finish(), "retained");
    active.assert_waiting();
    gate.release.send(()).unwrap();
    assert_version(active.finish(), "retained");
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
    assert!(h.root.staged_files().is_empty());
    assert!(sentinel.exists());
    assert_eq!(h.events.lock().unwrap().len(), 1);
    h.assert_calls(2);
}

#[test]
fn publication_failure_preserves_prior_cache_and_files_without_success() {
    let gate = PublicationGate::after_seed();
    let h = Harness::with_now(Arc::clone(&gate.now));
    assert_version(
        h.step("seed", LoadStep::Loaded(h.root.loaded("retained"))),
        "retained",
    );
    let bytes = fs::read(h.root.cache_path()).unwrap();
    let sentinel = h.root.sentinel();
    let active = h.call("refresh");
    h.enter(2);
    h.loaded("must-not-publish");
    gate.wait();
    let staged = h.root.staged_files();
    assert_eq!(staged.len(), 1);
    fs::remove_file(&staged[0]).unwrap();
    gate.release.send(()).unwrap();
    assert!(active.finish().is_err());
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
    assert_version(Ok(h.service.current()), "retained");
    assert!(h.root.staged_files().is_empty());
    assert!(sentinel.exists());
    assert_eq!(h.events.lock().unwrap().len(), 1);
}

struct StaticLoader(LoadedPlugins);

impl PluginSkillsLoader for StaticLoader {
    fn load(&self) -> Result<LoadedPlugins, String> {
        Ok(self.0.clone())
    }
}

#[test]
fn successful_reporter_can_reenter_and_dispose_after_publication() {
    let root = TempRoot::new();
    let owner = Arc::new(Mutex::new(Weak::<SandPluginSkillsService>::new()));
    let reporter_owner = Arc::clone(&owner);
    let (tx, reported) = mpsc::channel();
    let service = Arc::new(
        SandPluginSkillsService::new(&root.0, Arc::new(StaticLoader(root.loaded("committed"))))
            .with_reporter(Arc::new(move |event| {
                let service = reporter_owner.lock().unwrap().upgrade().unwrap();
                let recursive = service.sync("reporter-recursive").unwrap_err();
                service.dispose();
                tx.send((event, recursive, service.current())).unwrap();
            })),
    );
    *owner.lock().unwrap() = Arc::downgrade(&service);
    assert_version(Call::start(&service, "startup").finish(), "committed");
    let (event, recursive, current) = reported.recv_timeout(DEADLINE).unwrap();
    assert_eq!(event.outcome, "ok");
    assert_eq!(
        recursive,
        "recursive plugin skills sync cannot wait on its own pass",
    );
    assert_version(Ok(current), "committed");
    assert_version(Ok(service.current()), "committed");
    assert!(service.state.lock().unwrap().disposed);
}

struct ReadyAuth;
impl super::super::extension::AuthRenewalPort for ReadyAuth {
    fn peek_access_token(&self) -> Option<String> {
        Some("fixture-ready".into())
    }

    fn subscribe_to_renewal(
        &self,
        _listener: Arc<dyn Fn(super::super::extension::AuthRenewalEvent) + Send + Sync>,
    ) -> Box<dyn FnOnce() + Send> {
        Box::new(|| {})
    }
}

fn start_real_poll(
    service: &Arc<SandPluginSkillsService>,
    sweeps: &Arc<AtomicUsize>,
) -> super::super::extension::PluginSkillsAuthenticatedStartup {
    let sweeps = Arc::clone(sweeps);
    super::super::extension::PluginSkillsAuthenticatedStartup::start(
        Arc::new(ReadyAuth),
        service.clone(),
        Arc::new(
            super::super::production::RealPluginSkillsPolling::with_interval(Duration::from_secs(
                60,
            )),
        ),
        Some(Arc::new(move || {
            sweeps.fetch_add(1, Ordering::SeqCst);
        })),
    )
}

#[test]
fn real_poll_shutdown_retires_a_queued_sync_before_joining_its_thread() {
    let h = Harness::new();
    let active = h.call("manual-refresh");
    h.enter(1);
    let sweeps = Arc::new(AtomicUsize::new(0));
    let startup = start_real_poll(&h.service, &sweeps);
    h.pending("startup");
    let (tx, stopped) = mpsc::channel();
    let stopping = thread::spawn(move || {
        startup.dispose();
        tx.send(()).unwrap();
    });
    // The queued poll must retire without waiting for the unrelated loader.
    stopped
        .recv_timeout(DEADLINE)
        .expect("queued poll blocked shutdown");
    stopping.join().unwrap();
    active.assert_waiting();
    h.assert_calls(1);
    h.loaded("must-not-write");
    assert!(active.finish().unwrap().is_empty());
    assert!(h.service.current().is_empty());
    assert_eq!(sweeps.load(Ordering::SeqCst), 0);
    h.assert_calls(1);
}

#[test]
fn real_poll_shutdown_fences_an_active_load_before_join_finishes() {
    let h = Harness::new();
    let sweeps = Arc::new(AtomicUsize::new(0));
    let startup = start_real_poll(&h.service, &sweeps);
    h.enter(1);
    let (tx, stopped) = mpsc::channel();
    let stopping = thread::spawn(move || {
        startup.dispose();
        tx.send(()).unwrap();
    });
    let until = Instant::now() + DEADLINE;
    let mut state = h.service.state.lock().unwrap();
    while !state.disposed {
        let remaining = until
            .checked_duration_since(Instant::now())
            .expect("skills not disposed");
        state = h.service.changed.wait_timeout(state, remaining).unwrap().0;
    }
    drop(state);
    assert!(matches!(stopped.try_recv(), Err(TryRecvError::Empty)));
    h.loaded("must-not-write");
    stopped
        .recv_timeout(DEADLINE)
        .expect("active poll blocked shutdown");
    stopping.join().unwrap();
    assert!(h.service.current().is_empty());
    assert_eq!(sweeps.load(Ordering::SeqCst), 0);
    h.assert_calls(1);
}

pub(super) fn two_plugins(root: &TempRoot, version: &str) -> LoadedPlugins {
    let mut loaded = root.loaded(version);
    let mut other = loaded.plugins[0].clone();
    other.identifier.plugin_db_id = Some("10".into());
    other.identifier.name = "other".into();
    other.install_path = root.0.join("plugins/cache/market/other");
    loaded.plugins.push(other);
    loaded.listed_plugin_ids.push("10".into());
    loaded.listed_cache_keys.push(ListedCacheKey {
        marketplace_slug: "market".into(),
        plugin_id: "other".into(),
    });
    for plugin in &loaded.plugins {
        let path = plugin.install_path.join("skills/test/SKILL.md");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        if !path.exists() {
            fs::write(path, format!("{} original skill content", plugin.identifier.name)).unwrap();
        }
    }
    loaded
}

pub(super) fn refuse_nine_with_invalid_stage(root: &TempRoot) -> LoadedPlugins {
    let mut loaded = two_plugins(root, "must-not-publish");
    loaded.auth_blocked.push(auth_block("9"));
    // This is a valid authoritative loader aggregate. Its generated record for
    // the unrelated plugin 10 fails the real cache parser before staging.
    loaded.plugins[1].install_path = PathBuf::from("relative/other");
    loaded
}

pub(super) fn skill_file_bytes(records: &[PluginSkillRecord]) -> Vec<(PathBuf, Vec<u8>)> {
    records.iter().map(|record| {
        let path = PathBuf::from(&record.file_path);
        let bytes = fs::read(&path).unwrap();
        (path, bytes)
    }).collect()
}

pub(super) fn assert_skill_file_bytes(files: &[(PathBuf, Vec<u8>)]) {
    for (path, bytes) in files {
        assert_eq!(&fs::read(path).unwrap(), bytes, "{}", path.display());
    }
}

pub(super) fn record_ids(records: &[PluginSkillRecord]) -> Vec<&str> {
    let mut ids = records.iter().map(|record| record.plugin_id.as_str()).collect::<Vec<_>>();
    ids.sort_unstable();
    ids
}

fn confirm_uninstall(service: &SandPluginSkillsService, plugin_id: &str) -> Result<(), String> {
    let intent = service.begin_uninstall(plugin_id)?;
    service.settle_uninstall(&intent, PluginUninstallSettlement::Removed)
}

fn assert_uninstall_custody(
    service: &SandPluginSkillsService,
    root: &TempRoot,
    confirmed: &[&str],
    pending: &[&PluginUninstallIntent],
) {
    assert_uninstall_custody_with_auth(service, root, confirmed, pending, &[], None);
}

fn assert_uninstall_custody_with_auth(
    service: &SandPluginSkillsService,
    root: &TempRoot,
    confirmed: &[&str],
    pending: &[&PluginUninstallIntent],
    refusals: &[PluginAuthBlock],
    refresh: Option<&str>,
) {
    let entries = pending.iter().map(|intent| json!({
        "pluginId": intent.plugin_id,
        "operationId": intent.operation_id,
    })).collect::<Vec<_>>();
    let durable: Value = serde_json::from_slice(&fs::read(root.uninstall_state_path()).unwrap()).unwrap();
    assert_eq!(durable, json!({
        "formatVersion": 2,
        "confirmedPluginIds": confirmed,
        "pending": entries,
        "authRefusals": refusals,
        "refreshIntent": refresh.map(|operation_id| json!({"operationId": operation_id})),
    }));
    assert_eq!(fs::read(root.uninstall_version_path()).unwrap(), b"1\n");
    let status = if confirmed.is_empty() && pending.is_empty() {
        None
    } else {
        Some(json!({"confirmedPluginIds": confirmed, "pending": entries}))
    };
    assert_eq!(service.uninstall_recovery_status(), status);
    let mut projected = json!({"authBlocked": service.current_auth_blocked()});
    if let Some(status) = status {
        projected["uninstallRecovery"] = status;
    }
    if !refusals.is_empty() {
        projected["authRecovery"] = json!({
            "refusals": refusals,
            "refreshOutcomeUnknown": false,
        });
    }
    assert_eq!(<SandPluginSkillsService as PluginSkillsPort>::status(service), projected);
}

pub(super) fn refresh_operation_id(root: &TempRoot) -> String {
    let image: Value = serde_json::from_slice(&fs::read(root.uninstall_state_path()).unwrap()).unwrap();
    let operation_id = image["refreshIntent"]["operationId"].as_str().unwrap();
    assert_eq!(uuid::Uuid::parse_str(operation_id).unwrap().to_string(), operation_id);
    operation_id.to_string()
}

#[test]
fn live_refusal_restoration_reports_changed_once_after_stage_or_rename_failure() {
    for failed_rename in [false, true] {
        for zero_skills in [false, true] {
            let gate = failed_rename.then(PublicationGate::after_seed);
            let h = match &gate {
                Some(gate) => Harness::with_now(Arc::clone(&gate.now)),
                None => Harness::new(),
            };
            let mut loaded = two_plugins(&h.root, "v1");
            let original_files = skill_file_bytes(&plugin_contents_to_skill_records(
                &loaded.plugins,
                &loaded.publisher_facts,
            ));
            if zero_skills {
                loaded.plugins[0].skills.clear();
            }
            let seed = h.step("seed", LoadStep::Loaded(loaded.clone())).unwrap();
            assert_eq!(
                record_ids(&seed),
                if zero_skills { vec!["10"] } else { vec!["10", "9"] },
            );
            let index = h.service.current_index().unwrap();
            assert_eq!(index.current_user_id, Some(12));
            assert!(index.auth_blocked.is_empty());
            let bytes = fs::read(h.root.cache_path()).unwrap();
            let sentinel = h.root.sentinel();
            let reader = h.service.admission_reader();
            assert!(!reader.is_denied("9"));

            if let Some(gate) = &gate {
                let mut denied = loaded.clone();
                denied.auth_blocked.push(auth_block("9"));
                let active = h.call("denied-rename-failure");
                h.enter(2);
                h.release.send(LoadStep::Loaded(denied)).unwrap();
                gate.wait();
                assert!(reader.is_denied("9"));
                let staged = h.root.staged_files();
                assert_eq!(staged.len(), 1);
                fs::remove_file(&staged[0]).unwrap();
                gate.release.send(()).unwrap();
                assert!(active.finish().is_err());
            } else {
                assert_eq!(
                    h.step(
                        "denied-stage-failure",
                        LoadStep::Loaded(refuse_nine_with_invalid_stage(&h.root)),
                    ).unwrap_err(),
                    "invalid plugin skills cache snapshot",
                );
            }
            assert!(reader.is_denied("9"));
            assert!(!reader.is_denied("10"));
            assert_eq!(record_ids(&h.service.current()), vec!["10"]);
            assert_eq!(h.service.current_index().unwrap(), index);
            assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
            assert_skill_file_bytes(&original_files);
            assert!(h.root.staged_files().is_empty());
            assert!(sentinel.exists());
            assert_eq!(h.events.lock().unwrap().len(), 1);

            for (trigger, changed) in [("restored", true), ("unchanged", false)] {
                assert_eq!(h.step(trigger, LoadStep::Loaded(loaded.clone())).unwrap(), seed);
                assert!(Arc::ptr_eq(&reader, &h.service.admission_reader()));
                assert!(!reader.is_denied("9"));
                assert!(!reader.is_denied("10"));
                assert_eq!(h.service.current(), seed);
                // Constant time and the exact same materialization keep every
                // disk fact equal. Only the first pass releases the live refusal,
                // including when plugin 9 successfully materializes no skills.
                assert_eq!(h.service.current_index().unwrap(), index);
                assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
                assert_skill_file_bytes(&original_files);
                let events = h.events.lock().unwrap();
                let event = events.last().unwrap();
                assert_eq!(event.trigger, trigger);
                assert_eq!(event.outcome, "ok");
                assert_eq!(
                    event.changed, changed,
                    "failed_rename={failed_rename}, zero_skills={zero_skills}, trigger={trigger}",
                );
                assert_eq!(event.skill_count, seed.len());
                assert_eq!(event.error_class, None);
            }
            assert!(!sentinel.exists());
            assert_eq!(h.events.lock().unwrap().len(), 3);
            h.assert_calls(4);
        }
    }
}

#[test]
fn live_refusal_survives_invalid_stage_failed_release_and_partial_until_valid_fresh_success() {
    // Seed takes three clock reads, invalid-stage denial takes two, and the
    // next fresh-success duration is call eight. Keep that real failed rename
    // distinguishable from a failure to materialize the plugin.
    let gate = PublicationGate::at_clock_call(8);
    let h = Harness::with_now(Arc::clone(&gate.now));
    let seed = h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
    assert_eq!(record_ids(&seed), vec!["10", "9"]);
    let original_files = skill_file_bytes(&seed);
    let bytes = fs::read(h.root.cache_path()).unwrap();
    let sentinel = h.root.sentinel();
    let reader = h.service.admission_reader();
    assert!(Arc::ptr_eq(&reader, &h.service.admission_reader()));
    assert!(!reader.is_denied("9"));

    assert_eq!(
        h.step("denied-stage-failure", LoadStep::Loaded(refuse_nine_with_invalid_stage(&h.root)))
            .unwrap_err(),
        "invalid plugin skills cache snapshot",
    );
    assert!(reader.is_denied("9"));
    assert!(!reader.is_denied("10"));
    assert_eq!(record_ids(&h.service.current()), vec!["10"]);
    assert_eq!(h.service.current_index().unwrap().skills, seed);
    assert!(blocked_ids(&h.service).is_empty());
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
    assert!(h.root.staged_files().is_empty());
    assert!(sentinel.exists());
    assert_skill_file_bytes(&original_files);
    assert_eq!(h.events.lock().unwrap().len(), 1);

    let failed_release = h.call("fresh-but-unpublished");
    h.enter(3);
    h.release.send(LoadStep::Loaded(two_plugins(&h.root, "fresh"))).unwrap();
    gate.wait();
    assert!(reader.is_denied("9"));
    assert_eq!(record_ids(&h.service.current()), vec!["10"]);
    let staged = h.root.staged_files();
    assert_eq!(staged.len(), 1);
    fs::remove_file(&staged[0]).unwrap();
    gate.release.send(()).unwrap();
    assert!(failed_release.finish().is_err());
    assert_skill_file_bytes(&original_files);
    assert!(reader.is_denied("9"));
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
    assert_eq!(h.events.lock().unwrap().len(), 1);
    assert!(sentinel.exists());
    assert!(h.step("offline", LoadStep::Failed).is_err());
    assert!(reader.is_denied("9"));
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);

    let mut partial = two_plugins(&h.root, "unmaterialized");
    partial.plugins.clear();
    let retained = h.step("partial", LoadStep::Loaded(partial)).unwrap();
    assert_eq!(record_ids(&retained), vec!["10"]);
    assert_eq!(retained[0].plugin_version, "retained");
    assert_eq!(blocked_ids(&h.service), vec!["9"]);
    assert!(reader.is_denied("9"));
    assert!(sentinel.exists());

    let mut conflicting = two_plugins(&h.root, "conflicting");
    conflicting.auth_blocked.push(auth_block("9"));
    assert_eq!(
        record_ids(&h.step("denial-wins", LoadStep::Loaded(conflicting)).unwrap()),
        vec!["10"],
    );
    assert!(reader.is_denied("9"));

    let mut zero_skills = two_plugins(&h.root, "zero-skills");
    zero_skills.plugins[0].skills.clear();
    assert_eq!(
        record_ids(&h.step("fresh-zero-skills", LoadStep::Loaded(zero_skills)).unwrap()),
        vec!["10"],
    );
    assert!(!reader.is_denied("9"));
    assert!(blocked_ids(&h.service).is_empty());
    assert!(!sentinel.exists());
    assert_eq!(
        record_ids(&h.step("fresh-records", LoadStep::Loaded(two_plugins(&h.root, "fresh"))).unwrap()),
        vec!["10", "9"],
    );
    assert!(!reader.is_denied("9"));
}

#[test]
fn new_refusal_is_live_before_rename_and_failed_publication_preserves_old_allowed_bytes() {
    let gate = PublicationGate::after_seed();
    let h = Harness::with_now(Arc::clone(&gate.now));
    let seed = h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
    let original_files = skill_file_bytes(&seed);
    let bytes = fs::read(h.root.cache_path()).unwrap();
    let sentinel = h.root.sentinel();
    let mut denied = two_plugins(&h.root, "must-not-publish");
    denied.auth_blocked.push(auth_block("9"));
    let active = h.call("denied-rename-failure");
    h.enter(2);
    h.release.send(LoadStep::Loaded(denied)).unwrap();
    gate.wait();
    let reader = h.service.admission_reader();
    assert!(reader.is_denied("9"));
    assert_eq!(record_ids(&h.service.current()), vec!["10"]);
    assert_eq!(h.service.current_index().unwrap().skills, seed);
    assert!(h.service.current_auth_blocked().is_empty());
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
    let staged = h.root.staged_files();
    assert_eq!(staged.len(), 1);
    fs::remove_file(&staged[0]).unwrap();
    gate.release.send(()).unwrap();
    assert!(active.finish().is_err());
    assert_skill_file_bytes(&original_files);
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
    assert_eq!(record_ids(&h.service.current()), vec!["10"]);
    assert!(sentinel.exists());
    assert_eq!(h.events.lock().unwrap().len(), 1);
    h.service.dispose();
    assert!(reader.is_denied("9"));
    assert_eq!(record_ids(&h.call("disposed").finish().unwrap()), vec!["10"]);
    assert_eq!(h.service.current_index().unwrap().skills, seed);
    h.assert_calls(2);
}

#[test]
fn malformed_aggregate_never_imports_its_otherwise_valid_refusal_or_unknown_identity() {
    let h = Harness::new();
    let seed = h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
    let bytes = fs::read(h.root.cache_path()).unwrap();
    let reader = h.service.admission_reader();
    let mut malformed = two_plugins(&h.root, "must-not-publish");
    malformed.auth_blocked = vec![auth_block("9"), auth_block("unlisted")];
    assert_eq!(
        h.step("invalid-aggregate", LoadStep::Loaded(malformed)).unwrap_err(),
        "plugin skills loader returned an unlisted blocked plugin ID",
    );
    assert!(!reader.is_denied("9"));
    assert_eq!(h.service.current(), seed);
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
    let mut unknown_identity = two_plugins(&h.root, "retained");
    unknown_identity.current_user_id = None;
    assert_eq!(h.step("unknown-publisher", LoadStep::Loaded(unknown_identity)).unwrap(), seed);
    assert!(!reader.is_denied("9"));
    assert_eq!(h.service.current_index().unwrap().current_user_id, None);
    assert!(h.events.lock().unwrap().last().unwrap().changed);
}

#[test]
fn missing_or_malformed_readback_never_releases_a_live_refusal() {
    for missing in [true, false] {
        let gate = BoundaryGate::at_visit(2);
        let h = Harness::with_hooks(
            Arc::new(|| 1_000),
            ServiceTestHooks {
                after_rename: Some(Arc::clone(&gate.pause)),
                ..ServiceTestHooks::default()
            },
        );
        let seed = h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
        let bytes = fs::read(h.root.cache_path()).unwrap();
        let sentinel = h.root.sentinel();
        assert!(h.step("refusal", LoadStep::Loaded(refuse_nine_with_invalid_stage(&h.root))).is_err());
        let reader = h.service.admission_reader();
        let release_attempt = h.call("fresh-readback-failure");
        h.enter(3);
        h.release.send(LoadStep::Loaded(two_plugins(&h.root, "fresh"))).unwrap();
        gate.wait();
        // Fault injection after the actual rename, before the production strict
        // readback. This is not a claim that arbitrary external writers are fenced.
        if missing {
            fs::remove_file(h.root.cache_path()).unwrap();
        } else {
            fs::write(h.root.cache_path(), b"{malformed").unwrap();
        }
        assert!(reader.is_denied("9"));
        assert!(h.service.state.try_lock().is_ok());
        assert!(h.service.admission.state.try_lock().is_ok());
        gate.release.send(()).unwrap();
        assert_eq!(
            release_attempt.finish().unwrap_err(),
            "could not read committed plugin skills snapshot",
        );
        assert!(reader.is_denied("9"));
        assert_eq!(h.events.lock().unwrap().len(), 1);
        assert!(sentinel.exists());
        // Restore the real old allowed cache to distinguish retained live denial
        // from the strict reader merely refusing the injected unreadable file.
        fs::write(h.root.cache_path(), &bytes).unwrap();
        assert_eq!(h.service.current_index().unwrap().skills, seed);
        assert_eq!(record_ids(&h.service.current()), vec!["10"]);
        assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
        assert_eq!(
            record_ids(&h.step("valid-readback", LoadStep::Loaded(two_plugins(&h.root, "fresh"))).unwrap()),
            vec!["10", "9"],
        );
        assert!(!reader.is_denied("9"));
    }
}

#[test]
fn completed_coalesced_return_rechecks_live_refusal_after_releasing_service_state() {
    let gate = BoundaryGate::at_visit(1);
    let h = Harness::with_hooks(
        Arc::new(|| 1_000),
        ServiceTestHooks {
            coalesced_result: Some(Arc::clone(&gate.pause)),
            ..ServiceTestHooks::default()
        },
    );
    let first = h.call("startup");
    h.enter(1);
    let (settled_tx, settled) = mpsc::channel();
    let second_service = Arc::clone(&h.service);
    let second_tx = settled_tx.clone();
    let second = thread::spawn(move || {
        second_tx.send(second_service.sync("followup")).unwrap();
    });
    h.pending("followup");
    let third_service = Arc::clone(&h.service);
    let third = thread::spawn(move || {
        settled_tx.send(third_service.sync("coalesced")).unwrap();
    });
    h.pending("coalesced");
    h.loaded("v1");
    assert_version(first.finish(), "v1");
    h.enter(2);
    h.loaded("v2");
    gate.wait();
    // The follow-up executor has returned. The other actual caller already
    // copied that completed pass, but is paused before its admission check.
    assert_version(settled.recv_timeout(DEADLINE).unwrap(), "v2");
    assert!(h.service.state.try_lock().is_ok());
    assert!(h.step("later-refusal", LoadStep::Loaded(refuse_nine_with_invalid_stage(&h.root))).is_err());
    assert_eq!(h.service.current_index().unwrap().skills[0].plugin_version, "v2");
    assert!(h.service.admission_reader().is_denied("9"));
    gate.release.send(()).unwrap();
    assert_empty(settled.recv_timeout(DEADLINE).unwrap());
    second.join().unwrap();
    third.join().unwrap();
    h.assert_calls(3);
}

#[test]
fn same_root_successor_keeps_reader_and_fences_late_imports_and_old_runtime_dispose() {
    let h = Harness::new();
    h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
    assert!(h.step("refusal", LoadStep::Loaded(refuse_nine_with_invalid_stage(&h.root))).is_err());
    let reader = h.service.admission_reader();
    let old_active = h.call("old-load");
    h.enter(3);
    let old_queued = h.call("old-queued");
    h.pending("old-queued");
    let mut successor_loaded = two_plugins(&h.root, "successor");
    successor_loaded.auth_blocked.push(auth_block("10"));
    let successor = h.service.successor_with_loader(Arc::new(StaticLoader(successor_loaded))).unwrap();
    assert_eq!(successor.sand_root_dir(), h.root.0.as_path());
    assert!(Arc::ptr_eq(&reader, &successor.admission_reader()));
    assert_eq!(record_ids(&old_queued.finish().unwrap()), vec!["10"]);
    assert!(reader.is_denied("9"));
    assert_eq!(record_ids(&successor.sync("startup").unwrap()), vec!["9"]);
    assert!(!reader.is_denied("9"));
    assert!(reader.is_denied("10"));
    let successor_bytes = fs::read(h.root.cache_path()).unwrap();
    old_active.assert_waiting();
    let mut stale = two_plugins(&h.root, "late-old-load");
    stale.auth_blocked.push(auth_block("9"));
    h.release.send(LoadStep::Loaded(stale)).unwrap();
    assert_eq!(record_ids(&old_active.finish().unwrap()), vec!["9"]);
    assert!(!reader.is_denied("9"));
    assert!(reader.is_denied("10"));
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), successor_bytes);
    assert_eq!(h.events.lock().unwrap().len(), 1);
    h.service.dispose();
    h.service.dispose();
    assert!(!successor.state.lock().unwrap().disposed);
    assert_eq!(record_ids(&successor.sync("after-old-stop").unwrap()), vec!["9"]);
    assert!(h.service.successor_with_loader(Arc::new(StaticLoader(h.root.loaded("fork")))).is_err());
    assert_eq!(h.service.retire_for_replacement().unwrap_err(), "plugin skills replacement already claimed");
    h.assert_calls(3);
}

#[test]
fn same_root_successor_fences_staged_predecessor_rename_and_positive_release() {
    let gate = PublicationGate::at_clock_call(8);
    let h = Harness::with_now(Arc::clone(&gate.now));
    h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
    assert!(h.step("refusal", LoadStep::Loaded(refuse_nine_with_invalid_stage(&h.root))).is_err());
    let old_fresh = h.call("old-fresh");
    h.enter(3);
    h.release.send(LoadStep::Loaded(two_plugins(&h.root, "old-fresh"))).unwrap();
    gate.wait();
    assert_eq!(h.root.staged_files().len(), 1);
    let mut successor_loaded = two_plugins(&h.root, "successor");
    successor_loaded.auth_blocked.push(auth_block("10"));
    let successor = h.service.successor_with_loader(Arc::new(StaticLoader(successor_loaded))).unwrap();
    assert_eq!(record_ids(&successor.sync("startup").unwrap()), vec!["9"]);
    let reader = successor.admission_reader();
    let bytes = fs::read(h.root.cache_path()).unwrap();
    assert!(reader.is_denied("10"));
    gate.release.send(()).unwrap();
    assert_eq!(record_ids(&old_fresh.finish().unwrap()), vec!["9"]);
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
    assert!(reader.is_denied("10"));
    assert!(!reader.is_denied("9"));
    assert!(h.root.staged_files().is_empty());
    assert_eq!(h.events.lock().unwrap().len(), 1);
}

#[test]
fn poisoned_admission_fails_closed_for_retained_and_successor_readers_without_resetting_truth() {
    let h = Harness::new();
    let seed = h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
    let bytes = fs::read(h.root.cache_path()).unwrap();
    let reader = h.service.admission_reader();
    let admission = Arc::clone(&h.service.admission);
    assert!(thread::spawn(move || {
        let _guard = admission.state.lock().unwrap();
        panic!("poison the private admission mutex");
    }).join().is_err());
    assert!(reader.is_denied("9"));
    assert!(reader.is_denied("10"));
    assert!(h.service.current().is_empty());
    assert_eq!(h.service.current_index().unwrap().skills, seed);
    assert_eq!(
        h.call("poisoned-refresh").finish().unwrap_err(),
        "plugin skills live admission poisoned",
    );
    h.assert_calls(1);
    assert_eq!(
        h.service.begin_uninstall("9").unwrap_err(),
        "plugin skills live admission poisoned",
    );
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
    h.service.dispose();
    assert_empty(h.call("disposed").finish());
    let successor = h.service.successor_with_loader(Arc::new(StaticLoader(two_plugins(&h.root, "fresh")))).unwrap();
    assert!(Arc::ptr_eq(&reader, &successor.admission_reader()));
    assert!(successor.current().is_empty());
    assert!(reader.is_denied("9") && reader.is_denied("10"));
    assert_eq!(successor.auth_recovery_status(), Some(json!({
        "refusals": [], "refreshOutcomeUnknown": true,
        "fault": "admission-state-unavailable",
    })));
    assert_eq!(h.service.retire_for_replacement().unwrap_err(), "plugin skills replacement already claimed");
    assert_eq!(successor.sync("startup").unwrap_err(), "plugin skills live admission poisoned");
    h.assert_calls(1);
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
}

#[test]
fn different_root_retirement_does_not_let_new_root_success_clear_an_old_reader() {
    let h = Harness::new();
    h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
    assert!(h.step("refusal", LoadStep::Loaded(refuse_nine_with_invalid_stage(&h.root))).is_err());
    let old_reader = h.service.admission_reader();
    h.service.retire_for_replacement().unwrap();
    let other_root = TempRoot::new();
    let independent = SandPluginSkillsService::new(
        &other_root.0,
        Arc::new(StaticLoader(other_root.loaded("different-root"))),
    );
    assert!(!Arc::ptr_eq(&old_reader, &independent.admission_reader()));
    assert_version(independent.sync("startup"), "different-root");
    assert!(old_reader.is_denied("9"));
    assert_eq!(record_ids(&h.service.current()), vec!["10"]);
    assert!(!independent.admission_reader().is_denied("9"));
    assert!(h.service.retire_for_replacement().is_err());
}

fn new_successor_plugin(root: &TempRoot) -> LoadedPlugins {
    let mut loaded = root.loaded("successor");
    loaded.plugins[0].identifier.plugin_db_id = Some("11".into());
    loaded.plugins[0].identifier.name = "new-successor".into();
    loaded.plugins[0].install_path = root.0.join("plugins/cache/market/new-successor");
    loaded.listed_plugin_ids = vec!["11".into()];
    loaded.listed_cache_keys[0].plugin_id = "new-successor".into();
    loaded
}

struct MaterializingLoader(LoadedPlugins);

impl PluginSkillsLoader for MaterializingLoader {
    fn load(&self) -> Result<LoadedPlugins, String> {
        for plugin in &self.0.plugins {
            for skill in &plugin.skills {
                let path = plugin.install_path.join(&skill.path);
                fs::create_dir_all(path.parent().unwrap()).map_err(|error| error.to_string())?;
                fs::write(&path, "successor materialized skill content").map_err(|error| error.to_string())?;
            }
        }
        Ok(self.0.clone())
    }
}

#[test]
fn successor_claim_before_old_cleanup_preserves_new_directory_and_old_report() {
    let gate = BoundaryGate::at_visit(1);
    let h = Harness::with_hooks(
        Arc::new(|| 1_000),
        ServiceTestHooks {
            after_commit: Some(Arc::clone(&gate.pause)),
            ..ServiceTestHooks::default()
        },
    );
    let old = h.call("old-committed");
    h.enter(1);
    h.loaded("old");
    gate.wait();
    assert_version(Ok(h.service.current()), "old");
    assert!(h.service.publication.try_lock().is_ok());
    let successor = h.service.successor_with_loader(Arc::new(MaterializingLoader(new_successor_plugin(&h.root)))).unwrap();
    let new_records = successor.sync("successor-startup").unwrap();
    assert_eq!(record_ids(&new_records), vec!["11"]);
    let new_file = PathBuf::from(&new_records[0].file_path);
    assert_eq!(fs::read_to_string(&new_file).unwrap(), "successor materialized skill content");
    let committed_bytes = fs::read(h.root.cache_path()).unwrap();
    gate.release.send(()).unwrap();
    assert_version(old.finish(), "old");
    assert_eq!(fs::read_to_string(&new_file).unwrap(), "successor materialized skill content");
    assert_eq!(successor.current(), new_records);
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), committed_bytes);
    let events = h.events.lock().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].trigger, "old-committed");
    assert_eq!(events[0].outcome, "ok");
    drop(events);

    // Causal negative control: the exact real old-prune inputs do delete an
    // equivalent new directory when the replacement barrier is omitted. Its
    // key and indexed path deliberately differ from the old pass's keep-set.
    let control = TempRoot::new();
    let old_loaded = control.loaded("old");
    let old_records = plugin_contents_to_skill_records(&old_loaded.plugins, &old_loaded.publisher_facts);
    let old_retained_paths = vec![
        old_records[0].file_path.clone(),
        old_loaded.plugins[0].install_path.to_string_lossy().into_owned(),
    ];
    let control_loaded = MaterializingLoader(new_successor_plugin(&control)).load().unwrap();
    let control_file = control_loaded.plugins[0].install_path.join("skills/test/SKILL.md");
    assert!(control_file.exists());
    prune_uninstalled_plugin_dirs(
        &get_plugins_root_dir(&control.0).join("cache"),
        &old_loaded.listed_cache_keys,
        &old_retained_paths,
    );
    assert!(!control_file.exists(), "the fixture must expose the actual stale-prune deletion");
}

#[test]
fn ordinary_dispose_after_commit_keeps_valid_pruning_and_reporting() {
    let gate = BoundaryGate::at_visit(1);
    let h = Harness::with_hooks(
        Arc::new(|| 1_000),
        ServiceTestHooks {
            after_commit: Some(Arc::clone(&gate.pause)),
            ..ServiceTestHooks::default()
        },
    );
    let sentinel = h.root.sentinel();
    let active = h.call("committed-before-dispose");
    h.enter(1);
    h.loaded("committed");
    gate.wait();
    assert!(sentinel.exists());
    h.service.dispose();
    assert!(h.service.state.lock().unwrap().disposed);
    assert!(!h.service.state.lock().unwrap().replacement_claimed);
    gate.release.send(()).unwrap();
    assert_version(active.finish(), "committed");
    assert!(!sentinel.exists());
    assert_version(Ok(h.service.current()), "committed");
    let events = h.events.lock().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].outcome, "ok");
    assert_eq!(events[0].trigger, "committed-before-dispose");
}

#[test]
fn old_cleanup_holds_only_publication_and_finishes_before_successor_materialization() {
    let gate = BoundaryGate::at_visit(1);
    let h = Harness::with_hooks(
        Arc::new(|| 1_000),
        ServiceTestHooks {
            prune_locked: Some(Arc::clone(&gate.pause)),
            ..ServiceTestHooks::default()
        },
    );
    let sentinel = h.root.sentinel();
    let old = h.call("old-cleanup-first");
    h.enter(1);
    h.loaded("old");
    gate.wait();
    assert!(matches!(h.service.publication.try_lock(), Err(std::sync::TryLockError::WouldBlock)));
    assert!(h.service.state.try_lock().is_ok());
    assert!(h.service.admission.state.try_lock().is_ok());
    assert!(sentinel.exists());
    let next_loaded = new_successor_plugin(&h.root);
    let new_file = next_loaded.plugins[0].install_path.join("skills/test/SKILL.md");
    assert!(!new_file.exists());
    let (enter_tx, entering) = mpsc::channel();
    let (ready_tx, ready) = mpsc::channel();
    let old_service = Arc::clone(&h.service);
    let handoff = thread::spawn(move || {
        enter_tx.send(()).unwrap();
        let successor = old_service.successor_with_loader(Arc::new(MaterializingLoader(next_loaded)));
        assert!(ready_tx.send(successor).is_ok());
    });
    entering.recv_timeout(DEADLINE).unwrap();
    // P is held by the actual cleanup path, so neither the one-shot claim nor
    // the subsequent loader can pass it. No wall-clock sleep is an oracle.
    assert!(matches!(ready.try_recv(), Err(TryRecvError::Empty)));
    assert!(!h.service.state.lock().unwrap().replacement_claimed);
    assert!(!new_file.exists());
    gate.release.send(()).unwrap();
    assert_version(old.finish(), "old");
    let successor = ready.recv_timeout(DEADLINE).unwrap().unwrap();
    handoff.join().unwrap();
    assert!(!sentinel.exists());
    assert!(!new_file.exists());
    let records = successor.sync("after-old-cleanup").unwrap();
    assert_eq!(record_ids(&records), vec!["11"]);
    assert_eq!(fs::read_to_string(&new_file).unwrap(), "successor materialized skill content");
    assert_eq!(successor.current(), records);
    assert_eq!(h.events.lock().unwrap().len(), 1);
}

#[test]
fn confirmed_uninstall_refuses_only_the_exact_id_in_retained_and_new_workflow_stores() {
    use crate::workflows::workflow_library::{WorkflowSpec, get_global_workflows_dir};
    use crate::workflows::workflow_store::FileWorkflowStore;

    let h = Harness::new();
    let mut loaded = two_plugins(&h.root, "retained");
    loaded.publisher_facts.insert("9".into(), PublisherFacts {
        publisher_user_id: Some(12), marketplace_team_id: Some(34),
    });
    let seed = h.step("seed", LoadStep::Loaded(loaded)).unwrap();
    let files = skill_file_bytes(&seed);
    let bytes = fs::read(h.root.cache_path()).unwrap();
    let sentinel = h.root.sentinel();
    let reader = h.service.admission_reader();
    let store = || FileWorkflowStore::new(
        h.root.0.join("agents/owner"), get_global_workflows_dir(&h.root.0),
    ).with_plugin_skills_admission(Arc::clone(&reader));
    let retained = store();
    let before = retained.get("plugin-9-test").unwrap();
    assert!(before.published_by_current_user);
    assert_eq!(before.body, "plugin original skill content");
    assert!(retained.get("plugin-10-test").is_some());
    let user = retained.create(&WorkflowSpec {
        name: "Unrelated user control".into(), description: "USER_CONTROL".into(),
        body: "USER_BODY".into(), trigger: None, source_ref: None,
    }).unwrap().unwrap();

    let clean_journal = fs::read(h.root.uninstall_state_path()).unwrap();
    assert_uninstall_custody(&h.service, &h.root, &[], &[]);
    for invalid in ["", " ", " 9", "9 ", "\t9", "\u{feff}9", "9\u{feff}", "9\n0", "09", "00"] {
        assert_eq!(
            h.service.begin_uninstall(invalid).unwrap_err(),
            "plugin uninstall requires a canonical plugin ID",
        );
        assert!(!reader.is_denied("9"));
        assert_eq!(h.service.current(), seed);
        assert!(h.service.uninstall_recovery_status().is_none());
        assert_eq!(fs::read(h.root.uninstall_state_path()).unwrap(), clean_journal);
        assert_uninstall_custody(&h.service, &h.root, &[], &[]);
    }
    confirm_uninstall(&h.service, "99").unwrap();
    assert!(reader.is_denied("99"));
    assert!(!reader.is_denied("9"));
    assert!(!reader.is_denied("10"));
    assert_eq!(h.service.current(), seed);

    confirm_uninstall(&h.service, "9").unwrap();
    assert!(reader.is_denied("9"));
    assert!(!reader.is_denied("10"));
    assert!(h.service.current_auth_blocked().is_empty());
    assert_eq!(h.service.current_index().unwrap().skills, seed);
    assert_eq!(record_ids(&h.service.current()), vec!["10"]);
    assert_eq!(h.step("offline-after-uninstall", LoadStep::Failed).unwrap_err(), "fixture loader failure");
    let reopened = store();
    for workflows in [&retained, &reopened] {
        assert!(workflows.get("plugin-9-test").is_none());
        assert!(workflows.list().iter().all(|record| record.plugin_id.as_deref() != Some("9")));
        assert_eq!(workflows.get("plugin-10-test").unwrap().body, "other original skill content");
        assert_eq!(workflows.get(&user.id).unwrap().body, "USER_BODY");
        assert!(workflows.update("plugin-9-test", &WorkflowSpec {
            name: "Refused edit".into(), description: "must not touch".into(),
            body: "MUST_NOT_WRITE".into(), trigger: None, source_ref: None,
        }).unwrap().is_none());
    }
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
    assert_skill_file_bytes(&files);
    assert!(sentinel.exists());
    assert!(h.root.staged_files().is_empty());
    let events = h.events.lock().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[1].outcome, "failed");
    assert_eq!(events[1].skill_count, 1);
    drop(events);
    h.assert_calls(2);

    h.service.dispose();
    assert_eq!(
        h.service.begin_uninstall("10").unwrap_err(),
        "plugin skills owner is disposed; uninstall state was not changed",
    );
    assert!(reader.is_denied("9"));
    assert!(!reader.is_denied("10"));
    assert_eq!(record_ids(&h.call("disposed").finish().unwrap()), vec!["10"]);
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
    h.assert_calls(2);
}

#[cfg(unix)]
#[test]
fn confirmed_uninstall_refuses_a_proven_warm_workflow_body_cache_hit() {
    use crate::workflows::workflow_library::get_global_workflows_dir;
    use crate::workflows::workflow_store::FileWorkflowStore;

    let h = Harness::new();
    let seed = h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
    let target = PathBuf::from(&seed.iter().find(|record| record.plugin_id == "9").unwrap().file_path);
    let old_time = std::time::UNIX_EPOCH + Duration::from_secs(1_000);
    for path in [h.root.cache_path(), target.clone(), target.parent().unwrap().to_path_buf()] {
        fs::File::open(&path).unwrap().set_modified(old_time).unwrap();
        assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), old_time);
    }
    let reader = h.service.admission_reader();
    let store = || FileWorkflowStore::new(
        h.root.0.join("agents/warm"), get_global_workflows_dir(&h.root.0),
    ).with_plugin_skills_admission(Arc::clone(&reader));
    let warm = store();
    assert_eq!(warm.get("plugin-9-test").unwrap().body, "plugin original skill content");
    let old_body = fs::read_to_string(&target).unwrap();
    let new_body = old_body.replace("original", "replaced");
    assert_ne!(new_body, old_body);
    assert_eq!(new_body.len(), old_body.len());
    fs::write(&target, &new_body).unwrap();
    fs::File::open(&target).unwrap().set_modified(old_time).unwrap();
    assert_eq!(warm.get("plugin-9-test").unwrap().body, old_body, "actual body cache hit");
    assert_eq!(store().get("plugin-9-test").unwrap().body, new_body, "cold-reader control");
    let bytes = fs::read(h.root.cache_path()).unwrap();

    let intent = h.service.begin_uninstall("9").unwrap();
    assert_uninstall_custody(&h.service, &h.root, &[], &[&intent]);
    assert!(warm.get("plugin-9-test").is_none());
    assert!(store().get("plugin-9-test").is_none());
    assert!(warm.get("plugin-10-test").is_some());
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
    assert_eq!(fs::read_to_string(&target).unwrap(), new_body);
    h.service.settle_uninstall(&intent, PluginUninstallSettlement::Removed).unwrap();
    assert_uninstall_custody(&h.service, &h.root, &["9"], &[]);
    assert!(warm.get("plugin-9-test").is_none());
    assert!(store().get("plugin-9-test").is_none());
    assert!(warm.get("plugin-10-test").is_some());
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
    assert_eq!(fs::read_to_string(&target).unwrap(), new_body);
    assert!(h.service.current_auth_blocked().is_empty());
}

#[test]
fn confirmed_uninstall_fences_an_old_load_and_queued_pass_captures_when_it_really_loads() {
    let h = Harness::new();
    let seed = h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
    let files = skill_file_bytes(&seed);
    let bytes = fs::read(h.root.cache_path()).unwrap();
    let sentinel = h.root.sentinel();
    let old = h.call("old-load");
    h.enter(2);
    let queued = h.call("queued-before-uninstall");
    h.pending("queued-before-uninstall");
    confirm_uninstall(&h.service, "9").unwrap();
    let reader = h.service.admission_reader();
    let mut stale = two_plugins(&h.root, "must-not-publish");
    stale.auth_blocked.push(auth_block("10"));
    h.release.send(LoadStep::Loaded(stale)).unwrap();
    assert_eq!(old.finish().unwrap_err(), "plugin skills sync superseded by an uninstall observation");
    h.enter(3);
    assert!(reader.is_denied("9"));
    assert!(!reader.is_denied("10"), "stale load cannot import its auth refusal");
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
    assert_skill_file_bytes(&files);
    assert!(h.root.staged_files().is_empty());
    assert!(sentinel.exists());
    assert_eq!(h.events.lock().unwrap().len(), 1, "stale pass did not report publication success");

    // This caller queued before the fact, but its actual load starts after it.
    // It is a legitimate authoritative restoration, not a stale generation.
    h.release.send(LoadStep::Loaded(two_plugins(&h.root, "fresh"))).unwrap();
    assert_eq!(record_ids(&queued.finish().unwrap()), vec!["10", "9"]);
    assert!(!reader.is_denied("9"));
    assert!(!reader.is_denied("10"));
    assert_eq!(h.events.lock().unwrap().len(), 2);
    h.assert_calls(3);
}

#[test]
fn confirmed_uninstall_fences_owned_staging_even_when_the_same_id_was_already_observed() {
    for already_observed in [false, true] {
        let gate = PublicationGate::after_seed();
        let h = Harness::with_now(Arc::clone(&gate.now));
        let seed = h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
        let files = skill_file_bytes(&seed);
        let bytes = fs::read(h.root.cache_path()).unwrap();
        let sentinel = h.root.sentinel();
        if already_observed {
            confirm_uninstall(&h.service, "9").unwrap();
        }
        let old = h.call("old-staged-success");
        h.enter(2);
        h.release.send(LoadStep::Loaded(two_plugins(&h.root, "must-not-publish"))).unwrap();
        gate.wait();
        assert_eq!(h.root.staged_files().len(), 1);
        assert!(h.service.state.try_lock().is_ok());
        assert!(h.service.admission.state.try_lock().is_ok());
        confirm_uninstall(&h.service, "9").unwrap();
        assert!(h.service.admission_reader().is_denied("9"));
        gate.release.send(()).unwrap();
        assert_eq!(old.finish().unwrap_err(), "plugin skills sync superseded by an uninstall observation");
        assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
        assert_skill_file_bytes(&files);
        assert!(h.root.staged_files().is_empty());
        assert!(sentinel.exists());
        assert_eq!(record_ids(&h.service.current()), vec!["10"]);
        assert!(h.service.current_auth_blocked().is_empty());
        assert_eq!(h.events.lock().unwrap().len(), 1);
        h.assert_calls(2);
    }
}

#[test]
fn confirmed_uninstall_survives_fresh_publication_failures_and_unresolved_fallback_until_fresh_success() {
    for failed_rename in [false, true] {
        let gate = failed_rename.then(PublicationGate::after_seed);
        let h = match &gate {
            Some(gate) => Harness::with_now(Arc::clone(&gate.now)),
            None => Harness::new(),
        };
        let seed = h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
        let files = skill_file_bytes(&seed);
        let bytes = fs::read(h.root.cache_path()).unwrap();
        let sentinel = h.root.sentinel();
        let reader = h.service.admission_reader();
        confirm_uninstall(&h.service, "9").unwrap();
        if let Some(gate) = &gate {
            let fresh = h.call("fresh-rename-failure");
            h.enter(2);
            h.release.send(LoadStep::Loaded(two_plugins(&h.root, "fresh"))).unwrap();
            gate.wait();
            let staged = h.root.staged_files();
            assert_eq!(staged.len(), 1);
            fs::remove_file(&staged[0]).unwrap();
            gate.release.send(()).unwrap();
            assert!(fresh.finish().is_err());
        } else {
            let mut invalid = two_plugins(&h.root, "fresh");
            invalid.plugins[1].install_path = PathBuf::from("relative/other");
            assert_eq!(
                h.step("fresh-stage-failure", LoadStep::Loaded(invalid)).unwrap_err(),
                "invalid plugin skills cache snapshot",
            );
        }
        assert!(reader.is_denied("9"));
        assert!(!reader.is_denied("10"));
        assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
        assert_skill_file_bytes(&files);
        assert!(sentinel.exists());
        assert_eq!(h.events.lock().unwrap().len(), 1);
        assert!(h.step("fresh-offline", LoadStep::Failed).is_err());
        assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
        assert!(h.service.current_auth_blocked().is_empty());

        let mut partial = two_plugins(&h.root, "unresolved");
        partial.plugins.clear();
        let retained = h.step("partial", LoadStep::Loaded(partial)).unwrap();
        assert_eq!(record_ids(&retained), vec!["10"]);
        assert_eq!(retained[0].plugin_version, "retained");
        assert_eq!(record_ids(&h.service.current_index().unwrap().skills), vec!["10"]);
        assert!(reader.is_denied("9"));
        assert!(h.service.current_auth_blocked().is_empty(), "uninstall is not an auth failure");
        assert!(sentinel.exists());
        let mut other_success = two_plugins(&h.root, "other-only");
        other_success.plugins.remove(0);
        assert_eq!(record_ids(&h.step("other-only", LoadStep::Loaded(other_success)).unwrap()), vec!["10"]);
        assert!(reader.is_denied("9"), "another ID cannot release the uninstall");

        let mut denied = two_plugins(&h.root, "current-denial");
        denied.auth_blocked.push(auth_block("9"));
        assert_eq!(record_ids(&h.step("auth-denial-wins", LoadStep::Loaded(denied)).unwrap()), vec!["10"]);
        assert!(reader.is_denied("9"));
        assert_eq!(blocked_ids(&h.service), vec!["9"]);
        let mut empty_success = two_plugins(&h.root, "zero-skills");
        empty_success.plugins[0].skills.clear();
        assert_eq!(record_ids(&h.step("fresh-zero-skills", LoadStep::Loaded(empty_success)).unwrap()), vec!["10"]);
        assert!(!reader.is_denied("9"));
        assert!(h.service.current_auth_blocked().is_empty());
        assert!(!sentinel.exists());
        assert_eq!(record_ids(&h.step("fresh-reinstall", LoadStep::Loaded(two_plugins(&h.root, "restored"))).unwrap()), vec!["10", "9"]);
        assert!(!reader.is_denied("9"));
        assert_skill_file_bytes(&files);
        h.assert_calls(8);
    }
}

#[test]
fn confirmed_uninstall_equal_identity_restoration_reports_changed_once_including_zero_skills() {
    for zero_skills in [false, true] {
        let h = Harness::new();
        let mut loaded = two_plugins(&h.root, "same-version");
        if zero_skills {
            loaded.plugins[0].skills.clear();
        }
        let seed = h.step("seed", LoadStep::Loaded(loaded.clone())).unwrap();
        let bytes = fs::read(h.root.cache_path()).unwrap();
        let index = h.service.current_index().unwrap();
        let reader = h.service.admission_reader();
        confirm_uninstall(&h.service, "9").unwrap();
        assert!(reader.is_denied("9"));
        assert!(h.service.current_auth_blocked().is_empty());
        for (trigger, changed) in [("fresh-restoration", true), ("unchanged", false)] {
            assert_eq!(h.step(trigger, LoadStep::Loaded(loaded.clone())).unwrap(), seed);
            assert!(!reader.is_denied("9"));
            assert_eq!(h.service.current_index().unwrap(), index);
            assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
            let events = h.events.lock().unwrap();
            let report = events.last().unwrap();
            assert_eq!(report.trigger, trigger);
            assert_eq!(report.outcome, "ok");
            assert_eq!(report.changed, changed);
            assert_eq!(report.skill_count, seed.len());
        }
        h.assert_calls(3);
    }
}

#[test]
fn confirmed_uninstall_is_not_released_by_missing_or_malformed_commit_readback() {
    for missing in [false, true] {
        let gate = BoundaryGate::at_visit(2);
        let h = Harness::with_hooks(
            Arc::new(|| 1_000),
            ServiceTestHooks { after_rename: Some(Arc::clone(&gate.pause)), ..ServiceTestHooks::default() },
        );
        let seed = h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
        let bytes = fs::read(h.root.cache_path()).unwrap();
        let files = skill_file_bytes(&seed);
        let sentinel = h.root.sentinel();
        let reader = h.service.admission_reader();
        confirm_uninstall(&h.service, "9").unwrap();
        let fresh = h.call("fresh-readback-failure");
        h.enter(2);
        h.release.send(LoadStep::Loaded(two_plugins(&h.root, "fresh"))).unwrap();
        gate.wait();
        if missing {
            fs::remove_file(h.root.cache_path()).unwrap();
        } else {
            fs::write(h.root.cache_path(), b"{malformed").unwrap();
        }
        assert!(reader.is_denied("9"));
        assert!(h.service.state.try_lock().is_ok());
        assert!(h.service.admission.state.try_lock().is_ok());
        gate.release.send(()).unwrap();
        assert_eq!(fresh.finish().unwrap_err(), "could not read committed plugin skills snapshot");
        // Restore the prior allowed disk image, proving refusal comes from the
        // real owner rather than only from an unreadable cache in this fixture.
        fs::write(h.root.cache_path(), &bytes).unwrap();
        assert_eq!(record_ids(&h.service.current()), vec!["10"]);
        assert!(reader.is_denied("9"));
        assert!(!reader.is_denied("10"));
        assert!(h.service.current_auth_blocked().is_empty());
        assert_skill_file_bytes(&files);
        assert!(sentinel.exists());
        assert_eq!(h.events.lock().unwrap().len(), 1);
        assert_eq!(record_ids(&h.step("fresh-valid-readback", LoadStep::Loaded(two_plugins(&h.root, "fresh"))).unwrap()), vec!["10", "9"]);
        assert!(!reader.is_denied("9"));
        h.assert_calls(3);
    }
}

#[test]
fn confirmed_uninstall_same_root_successor_keeps_fact_and_rejects_retired_observations() {
    let h = Harness::new();
    h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
    confirm_uninstall(&h.service, "9").unwrap();
    let reader = h.service.admission_reader();
    let old = h.call("old-load");
    h.enter(2);
    let successor = h.service.successor_with_loader(Arc::new(StaticLoader(
        two_plugins(&h.root, "successor"),
    ))).unwrap();
    assert!(Arc::ptr_eq(&reader, &successor.admission_reader()));
    assert!(reader.is_denied("9"));
    assert_eq!(record_ids(&successor.current()), vec!["10"]);
    assert_eq!(
        h.service.begin_uninstall("10").unwrap_err(),
        "plugin skills owner is disposed; uninstall state was not changed",
    );
    assert!(!reader.is_denied("10"));
    assert_eq!(record_ids(&successor.sync("fresh-successor").unwrap()), vec!["10", "9"]);
    assert!(!reader.is_denied("9"));
    let bytes = fs::read(h.root.cache_path()).unwrap();
    let mut stale = two_plugins(&h.root, "old-must-not-publish");
    stale.auth_blocked.push(auth_block("9"));
    h.release.send(LoadStep::Loaded(stale)).unwrap();
    assert_eq!(record_ids(&old.finish().unwrap()), vec!["10", "9"]);
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
    assert!(!reader.is_denied("9"));
    h.service.dispose();
    confirm_uninstall(&successor, "10").unwrap();
    assert!(reader.is_denied("10"));
    assert!(!reader.is_denied("9"));
    assert_eq!(record_ids(&successor.current()), vec!["9"]);
    assert_eq!(h.events.lock().unwrap().len(), 1);
    h.assert_calls(2);
}

#[test]
fn confirmed_uninstall_after_commit_skips_old_prune_and_filters_return_but_keeps_commit_report() {
    let gate = BoundaryGate::at_visit(2);
    let h = Harness::with_hooks(
        Arc::new(|| 1_000),
        ServiceTestHooks { after_commit: Some(Arc::clone(&gate.pause)), ..ServiceTestHooks::default() },
    );
    h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
    let sentinel = h.root.sentinel();
    let committed = h.call("commit-before-fact");
    h.enter(2);
    h.release.send(LoadStep::Loaded(two_plugins(&h.root, "committed"))).unwrap();
    gate.wait();
    let bytes = fs::read(h.root.cache_path()).unwrap();
    assert_eq!(record_ids(&h.service.current()), vec!["10", "9"]);
    assert!(sentinel.exists());
    confirm_uninstall(&h.service, "9").unwrap();
    assert_eq!(record_ids(&h.service.current()), vec!["10"]);
    gate.release.send(()).unwrap();
    assert_eq!(record_ids(&committed.finish().unwrap()), vec!["10"]);
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
    assert!(sentinel.exists(), "new observation fences old cleanup after the valid commit");
    let events = h.events.lock().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[1].outcome, "ok");
    assert_eq!(events[1].trigger, "commit-before-fact");
    assert_eq!(events[1].skill_count, 2, "report describes the already-completed commit");
    assert!(h.service.admission_reader().is_denied("9"));
}

#[test]
fn confirmed_uninstall_orders_after_an_entered_prune_without_holding_reader_or_state_locks() {
    let gate = BoundaryGate::at_visit(1);
    let h = Harness::with_hooks(
        Arc::new(|| 1_000),
        ServiceTestHooks { prune_locked: Some(Arc::clone(&gate.pause)), ..ServiceTestHooks::default() },
    );
    let sentinel = h.root.sentinel();
    let committed = h.call("prune-before-fact");
    h.enter(1);
    h.release.send(LoadStep::Loaded(two_plugins(&h.root, "committed"))).unwrap();
    gate.wait();
    assert!(matches!(h.service.publication.try_lock(), Err(std::sync::TryLockError::WouldBlock)));
    assert!(h.service.state.try_lock().is_ok());
    assert!(h.service.admission.state.try_lock().is_ok());
    let service = Arc::clone(&h.service);
    let (enter_tx, entering) = mpsc::channel();
    let (done_tx, done) = mpsc::channel();
    let observer = thread::spawn(move || {
        enter_tx.send(()).unwrap();
        done_tx.send(confirm_uninstall(&service, "9")).unwrap();
    });
    entering.recv_timeout(DEADLINE).unwrap();
    assert!(matches!(done.try_recv(), Err(TryRecvError::Empty)));
    assert!(!h.service.admission_reader().is_denied("9"));
    assert!(sentinel.exists());
    gate.release.send(()).unwrap();
    committed.finish().unwrap();
    done.recv_timeout(DEADLINE).unwrap().unwrap();
    observer.join().unwrap();
    assert!(!sentinel.exists(), "the prune entered before the observation linearized");
    assert!(h.service.admission_reader().is_denied("9"));
    assert_eq!(record_ids(&h.service.current()), vec!["10"]);
    assert_eq!(h.events.lock().unwrap().len(), 1);
}

#[test]
fn pending_uninstall_reports_only_admitted_changes_at_publication() {
    for with_control in [false, true] {
        let final_pass = if with_control { 4 } else { 3 };
        let gate = BoundaryGate::at_visit(final_pass);
        let h = Harness::with_hooks(
            Arc::new(|| 1_000),
            ServiceTestHooks { after_commit: Some(Arc::clone(&gate.pause)), ..ServiceTestHooks::default() },
        );
        let mut loaded = two_plugins(&h.root, "retained");
        if !with_control {
            loaded.plugins.truncate(1);
            loaded.listed_plugin_ids.truncate(1);
            loaded.listed_cache_keys.truncate(1);
        }
        let seed = h.step("seed", LoadStep::Loaded(loaded.clone())).unwrap();
        let files = skill_file_bytes(&seed);
        let seed_cache = fs::read(h.root.cache_path()).unwrap();
        let sentinel = h.root.sentinel();
        let intent = h.service.begin_uninstall("9").unwrap();
        let journal = fs::read(h.root.uninstall_state_path()).unwrap();
        let reader = h.service.admission_reader();
        let admitted_ids = if with_control { vec!["10"] } else { vec![] };
        let admitted_count = usize::from(with_control);
        let cases: &[(&'static str, &'static str, bool)] = if with_control {
            &[("only-pending-version-changed", "retained", false), ("allowed-version-changed", "allowed-v2", true)]
        } else {
            &[("only-pending-version-changed", "retained", false)]
        };
        for (index, &(trigger, control_version, changed)) in cases.iter().enumerate() {
            let mut fresh = loaded.clone();
            fresh.plugins[0].identifier.version = Some("pending-v2".into());
            if with_control {
                fresh.plugins[1].identifier.version = Some(control_version.into());
            }
            let result = h.step(trigger, LoadStep::Loaded(fresh)).unwrap();
            assert_eq!(record_ids(&result), admitted_ids);
            assert_eq!(h.service.current(), result);
            assert!(reader.is_denied("9"));
            assert!(!reader.is_denied("10"));
            let raw = h.service.current_index().unwrap();
            assert_eq!(record_ids(&raw.skills), record_ids(&seed));
            assert_eq!(raw.skills.iter().find(|record| record.plugin_id == "9").unwrap().plugin_version, "pending-v2");
            if with_control {
                assert_eq!(raw.skills.iter().find(|record| record.plugin_id == "10").unwrap().plugin_version, control_version);
                assert_eq!(result[0].plugin_version, control_version);
            }
            assert_eq!(raw.current_user_id, Some(12));
            assert!(raw.auth_blocked.is_empty());
            assert_ne!(fs::read(h.root.cache_path()).unwrap(), seed_cache, "pending materialization still reaches the raw cache");
            assert_uninstall_custody(&h.service, &h.root, &[], &[&intent]);
            assert_eq!(fs::read(h.root.uninstall_state_path()).unwrap(), journal);
            assert_skill_file_bytes(&files);
            assert!(sentinel.exists());
            assert!(h.root.staged_files().is_empty());
            let events = h.events.lock().unwrap();
            assert_eq!(events.len(), index + 2);
            let report = events.last().unwrap();
            assert_eq!(report.trigger, trigger);
            assert_eq!(report.outcome, "ok");
            assert_eq!(report.changed, changed);
            assert_eq!(report.skill_count, admitted_count);
            assert_eq!(report.error_class, None);
        }

        // A later terminal reply changes live admission, but the already
        // committed report must retain its publication-time pending projection.
        let mut fresh = loaded.clone();
        fresh.plugins[0].identifier.version = Some("pending-v3".into());
        if with_control {
            fresh.plugins[1].identifier.version = Some("allowed-v2".into());
        }
        let committed = h.call("pending-view-before-later-settlement");
        h.enter(final_pass);
        h.release.send(LoadStep::Loaded(fresh)).unwrap();
        gate.wait();
        assert_eq!(record_ids(&h.service.current()), admitted_ids);
        let raw = h.service.current_index().unwrap();
        assert_eq!(record_ids(&raw.skills), record_ids(&seed));
        assert_eq!(raw.skills.iter().find(|record| record.plugin_id == "9").unwrap().plugin_version, "pending-v3");
        let cache = fs::read(h.root.cache_path()).unwrap();
        assert_uninstall_custody(&h.service, &h.root, &[], &[&intent]);
        assert_eq!(fs::read(h.root.uninstall_state_path()).unwrap(), journal);
        assert_eq!(h.events.lock().unwrap().len(), final_pass - 1);
        assert!(sentinel.exists());
        assert!(h.service.publication.try_lock().is_ok());
        h.service.settle_uninstall(&intent, PluginUninstallSettlement::NotRemoved).unwrap();
        assert_uninstall_custody(&h.service, &h.root, &[], &[]);
        assert!(!reader.is_denied("9"));
        assert_eq!(record_ids(&h.service.current()), record_ids(&seed));
        gate.release.send(()).unwrap();
        assert_eq!(record_ids(&committed.finish().unwrap()), record_ids(&seed));
        assert_eq!(fs::read(h.root.cache_path()).unwrap(), cache);
        assert_skill_file_bytes(&files);
        assert!(sentinel.exists(), "the later settlement also fences the committed pass's prune");
        let events = h.events.lock().unwrap();
        assert_eq!(events.len(), final_pass);
        let report = events.last().unwrap();
        assert_eq!(report.trigger, "pending-view-before-later-settlement");
        assert_eq!(report.outcome, "ok");
        assert!(!report.changed, "only the pending plugin changed at publication");
        assert_eq!(report.skill_count, admitted_count, "a later reply cannot rewrite the committed admission count");
        assert_eq!(report.error_class, None);
        drop(events);
        h.assert_calls(final_pass);
    }
}

#[test]
fn pending_uninstall_survives_fresh_sync_without_release_or_prune() {
    use crate::workflows::workflow_library::{WorkflowSpec, get_global_workflows_dir};
    use crate::workflows::workflow_store::FileWorkflowStore;

    for (prior_confirmed, zero_skills) in [(false, false), (true, false), (true, true)] {
        let h = Harness::new();
        let seed = h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
        let files = skill_file_bytes(&seed);
        let bytes = fs::read(h.root.cache_path()).unwrap();
        let sentinel = h.root.sentinel();
        let reader = h.service.admission_reader();
        let store = || FileWorkflowStore::new(
            h.root.0.join("agents/pending"), get_global_workflows_dir(&h.root.0),
        ).with_plugin_skills_admission(Arc::clone(&reader));
        let retained = store();
        assert!(retained.get("plugin-9-test").is_some());
        let user = retained.create(&WorkflowSpec {
            name: "Pending control".into(), description: "USER_CONTROL".into(),
            body: "USER_BODY".into(), trigger: None, source_ref: None,
        }).unwrap().unwrap();
        if prior_confirmed {
            confirm_uninstall(&h.service, "9").unwrap();
        }
        let intent = h.service.begin_uninstall("9").unwrap();
        assert_eq!(intent.plugin_id, "9");
        assert_eq!(uuid::Uuid::parse_str(&intent.operation_id).unwrap().to_string(), intent.operation_id);
        let confirmed: &[&str] = if prior_confirmed { &["9"] } else { &[] };
        assert_uninstall_custody(&h.service, &h.root, confirmed, &[&intent]);
        let journal = fs::read(h.root.uninstall_state_path()).unwrap();
        assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
        assert_eq!(record_ids(&h.service.current()), vec!["10"]);
        assert!(h.service.current_auth_blocked().is_empty());
        h.assert_calls(1);

        let mut fresh = two_plugins(&h.root, "while-pending");
        if zero_skills {
            fresh.plugins[0].skills.clear();
        }
        assert_eq!(record_ids(&h.step("fresh-but-pending", LoadStep::Loaded(fresh)).unwrap()), vec!["10"]);
        assert_uninstall_custody(&h.service, &h.root, confirmed, &[&intent]);
        assert_eq!(fs::read(h.root.uninstall_state_path()).unwrap(), journal);
        assert!(reader.is_denied("9"));
        assert!(!reader.is_denied("10"));
        assert!(h.service.current_auth_blocked().is_empty());
        let reopened = store();
        for workflows in [&retained, &reopened] {
            assert!(workflows.get("plugin-9-test").is_none());
            assert!(workflows.list().iter().all(|record| record.plugin_id.as_deref() != Some("9")));
            assert_eq!(workflows.get("plugin-10-test").unwrap().body, "other original skill content");
            assert_eq!(workflows.get(&user.id).unwrap().body, "USER_BODY");
        }
        assert_skill_file_bytes(&files);
        assert!(sentinel.exists(), "even a complete refresh cannot prune during unresolved custody");

        let mut absent = two_plugins(&h.root, "absent-while-pending");
        absent.plugins.clear();
        absent.listed_plugin_ids.clear();
        absent.listed_cache_keys.clear();
        assert_empty(h.step("empty-but-pending", LoadStep::Loaded(absent)));
        assert_uninstall_custody(&h.service, &h.root, confirmed, &[&intent]);
        assert!(reader.is_denied("9"));
        assert_skill_file_bytes(&files);
        assert!(sentinel.exists(), "an empty listing cannot establish remote quiescence");

        h.service.settle_uninstall(&intent, PluginUninstallSettlement::Removed).unwrap();
        assert_uninstall_custody(&h.service, &h.root, &["9"], &[]);
        assert!(reader.is_denied("9"));
        let mut reinstall = two_plugins(&h.root, "reinstalled");
        if zero_skills {
            reinstall.plugins[0].skills.clear();
        }
        let restored = h.step("after-terminal", LoadStep::Loaded(reinstall)).unwrap();
        assert_eq!(record_ids(&restored), if zero_skills { vec!["10"] } else { vec!["10", "9"] });
        assert_uninstall_custody(&h.service, &h.root, &[], &[]);
        assert!(!reader.is_denied("9"));
        assert!(!sentinel.exists());
        assert_skill_file_bytes(&files);
        assert!(h.events.lock().unwrap().last().unwrap().changed);
        h.assert_calls(4);
    }
}

#[test]
fn uninstall_settlement_requires_the_exact_current_ticket_without_advancing_observation_on_refusal() {
    let h = Harness::new();
    let seed = h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
    let files = skill_file_bytes(&seed);
    let cache = fs::read(h.root.cache_path()).unwrap();
    let sentinel = h.root.sentinel();
    assert!(h.service.uninstall_recovery_status().is_none());
    let nine = h.service.begin_uninstall("9").unwrap();
    let ten = h.service.begin_uninstall("10").unwrap();
    assert_ne!(nine.operation_id, ten.operation_id);
    assert_uninstall_custody(&h.service, &h.root, &[], &[&ten, &nine]);
    let journal = fs::read(h.root.uninstall_state_path()).unwrap();
    let active = h.call("load-survives-refused-tickets");
    h.enter(2);
    // Actual loader entry has reserved its own refresh token. Refused uninstall
    // operations must preserve this active image as well as both exact tickets.
    let active_journal = fs::read(h.root.uninstall_state_path()).unwrap();
    let refresh = refresh_operation_id(&h.root);
    assert_eq!(h.service.begin_uninstall("9").unwrap_err(), "plugin uninstall already has an unresolved intent");
    let wrong_tickets = [
        PluginUninstallIntent { plugin_id: "9".into(), operation_id: ten.operation_id.clone() },
        PluginUninstallIntent { plugin_id: "10".into(), operation_id: nine.operation_id.clone() },
        PluginUninstallIntent { plugin_id: "99".into(), operation_id: nine.operation_id.clone() },
        PluginUninstallIntent { plugin_id: "9".into(), operation_id: uuid::Uuid::new_v4().to_string() },
        PluginUninstallIntent { plugin_id: "9".into(), operation_id: String::new() },
    ];
    for wrong in &wrong_tickets {
        for settlement in [
            PluginUninstallSettlement::NotDispatched,
            PluginUninstallSettlement::NotRemoved,
            PluginUninstallSettlement::Removed,
        ] {
            assert_eq!(
                h.service.settle_uninstall(wrong, settlement).unwrap_err(),
                "plugin uninstall settlement does not match the current intent",
            );
            assert_eq!(fs::read(h.root.uninstall_state_path()).unwrap(), active_journal);
            assert_uninstall_custody_with_auth(&h.service, &h.root, &[], &[&ten, &nine], &[], Some(&refresh));
        }
    }
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), cache);
    assert!(h.service.current().is_empty());
    assert!(h.service.current_auth_blocked().is_empty());
    h.release.send(LoadStep::Loaded(two_plugins(&h.root, "current-load"))).unwrap();
    assert_empty(active.finish());
    assert_eq!(fs::read(h.root.uninstall_state_path()).unwrap(), journal);
    assert!(sentinel.exists());
    assert_skill_file_bytes(&files);

    h.service.settle_uninstall(&nine, PluginUninstallSettlement::NotRemoved).unwrap();
    assert_uninstall_custody(&h.service, &h.root, &[], &[&ten]);
    assert_eq!(record_ids(&h.service.current()), vec!["9"]);
    let settled_journal = fs::read(h.root.uninstall_state_path()).unwrap();
    assert_eq!(
        h.service.settle_uninstall(&nine, PluginUninstallSettlement::Removed).unwrap_err(),
        "plugin uninstall settlement does not match the current intent",
    );
    assert_eq!(fs::read(h.root.uninstall_state_path()).unwrap(), settled_journal);
    assert_uninstall_custody(&h.service, &h.root, &[], &[&ten]);
    let next_nine = h.service.begin_uninstall("9").unwrap();
    assert_ne!(next_nine.operation_id, nine.operation_id);
    let next_journal = fs::read(h.root.uninstall_state_path()).unwrap();
    for settlement in [
        PluginUninstallSettlement::NotDispatched,
        PluginUninstallSettlement::NotRemoved,
        PluginUninstallSettlement::Removed,
    ] {
        assert_eq!(
            h.service.settle_uninstall(&nine, settlement).unwrap_err(),
            "plugin uninstall settlement does not match the current intent",
        );
        assert_eq!(fs::read(h.root.uninstall_state_path()).unwrap(), next_journal);
        assert_uninstall_custody(&h.service, &h.root, &[], &[&ten, &next_nine]);
    }
    h.service.settle_uninstall(&next_nine, PluginUninstallSettlement::NotDispatched).unwrap();
    h.service.settle_uninstall(&ten, PluginUninstallSettlement::Removed).unwrap();
    assert_uninstall_custody(&h.service, &h.root, &["10"], &[]);
    assert_eq!(record_ids(&h.service.current()), vec!["9"]);
    assert_eq!(record_ids(&h.step("restored", LoadStep::Loaded(two_plugins(&h.root, "restored"))).unwrap()), vec!["10", "9"]);
    assert_uninstall_custody(&h.service, &h.root, &[], &[]);
    assert!(!sentinel.exists());
    h.assert_calls(3);
}

#[test]
fn uninstall_non_removal_preserves_prior_confirmed_and_auth_refusals() {
    for settlement in [PluginUninstallSettlement::NotDispatched, PluginUninstallSettlement::NotRemoved] {
        for (confirmed_before, auth_before) in [(true, false), (false, true), (true, true)] {
            let h = Harness::new();
            let seed = h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
            let files = skill_file_bytes(&seed);
            let cache = fs::read(h.root.cache_path()).unwrap();
            let reader = h.service.admission_reader();
            if auth_before {
                assert!(h.step("auth-refusal", LoadStep::Loaded(refuse_nine_with_invalid_stage(&h.root))).is_err());
            }
            if confirmed_before {
                confirm_uninstall(&h.service, "9").unwrap();
            }
            let intent = h.service.begin_uninstall("9").unwrap();
            let confirmed: &[&str] = if confirmed_before { &["9"] } else { &[] };
            let refusals = if auth_before { vec![auth_block("9")] } else { vec![] };
            assert_uninstall_custody_with_auth(&h.service, &h.root, confirmed, &[&intent], &refusals, None);
            h.service.settle_uninstall(&intent, settlement).unwrap();
            assert_uninstall_custody_with_auth(&h.service, &h.root, confirmed, &[], &refusals, None);
            assert!(reader.is_denied("9"));
            assert!(!reader.is_denied("10"));
            assert_eq!(record_ids(&h.service.current()), vec!["10"]);
            assert_eq!(fs::read(h.root.cache_path()).unwrap(), cache);
            assert_skill_file_bytes(&files);

            let mut unresolved = two_plugins(&h.root, "other-fresh");
            unresolved.plugins.remove(0);
            assert_eq!(record_ids(&h.step("unresolved-after-reply", LoadStep::Loaded(unresolved)).unwrap()), vec!["10"]);
            assert_eq!(blocked_ids(&h.service), if auth_before { vec!["9"] } else { vec![] });
            assert!(reader.is_denied("9"));
            assert_uninstall_custody_with_auth(&h.service, &h.root, confirmed, &[], &refusals, None);
            assert_eq!(record_ids(&h.step("fresh-reinstall", LoadStep::Loaded(two_plugins(&h.root, "restored"))).unwrap()), vec!["10", "9"]);
            assert_uninstall_custody(&h.service, &h.root, &[], &[]);
            assert!(!reader.is_denied("9"));
            assert!(h.service.current_auth_blocked().is_empty());
            assert_skill_file_bytes(&files);
            h.assert_calls(if auth_before { 4 } else { 3 });
        }
    }
}

#[test]
fn uninstall_reservation_fences_an_old_load_or_staged_pass_but_not_its_queued_fresh_load() {
    for staged in [false, true] {
        let gate = staged.then(PublicationGate::after_seed);
        let h = match &gate {
            Some(gate) => Harness::with_now(Arc::clone(&gate.now)),
            None => Harness::new(),
        };
        let seed = h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
        let cache = fs::read(h.root.cache_path()).unwrap();
        let files = skill_file_bytes(&seed);
        let sentinel = h.root.sentinel();
        let old = h.call("before-reservation");
        h.enter(2);
        if let Some(gate) = &gate {
            h.release.send(LoadStep::Loaded(two_plugins(&h.root, "must-not-publish"))).unwrap();
            gate.wait();
            assert_eq!(h.root.staged_files().len(), 1);
        }
        let queued = h.call("queued-before-reservation");
        h.pending("queued-before-reservation");
        let refresh = if staged { None } else { Some(refresh_operation_id(&h.root)) };
        let intent = h.service.begin_uninstall("9").unwrap();
        assert_uninstall_custody_with_auth(&h.service, &h.root, &[], &[&intent], &[], refresh.as_deref());
        if let Some(gate) = &gate {
            gate.release.send(()).unwrap();
        } else {
            let mut stale = two_plugins(&h.root, "must-not-publish");
            stale.auth_blocked.push(auth_block("10"));
            h.release.send(LoadStep::Loaded(stale)).unwrap();
        }
        assert_eq!(old.finish().unwrap_err(), "plugin skills sync superseded by an uninstall observation");
        h.enter(3);
        assert_eq!(fs::read(h.root.cache_path()).unwrap(), cache);
        assert_skill_file_bytes(&files);
        assert!(h.root.staged_files().is_empty());
        assert!(sentinel.exists());
        assert!(!h.service.admission_reader().is_denied("10"));
        assert_eq!(h.events.lock().unwrap().len(), 1);
        h.release.send(LoadStep::Loaded(two_plugins(&h.root, "fresh-while-pending"))).unwrap();
        assert_eq!(record_ids(&queued.finish().unwrap()), vec!["10"]);
        assert_uninstall_custody(&h.service, &h.root, &[], &[&intent]);
        assert!(h.service.admission_reader().is_denied("9"));
        assert!(sentinel.exists());
        h.service.settle_uninstall(&intent, PluginUninstallSettlement::NotDispatched).unwrap();
        assert_eq!(record_ids(&h.service.current()), vec!["10", "9"]);
        h.assert_calls(3);
    }
}

#[test]
fn uninstall_terminal_settlements_fence_loads_started_before_the_reply() {
    for settlement in [
        PluginUninstallSettlement::NotDispatched,
        PluginUninstallSettlement::NotRemoved,
        PluginUninstallSettlement::Removed,
    ] {
        let h = Harness::new();
        let seed = h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
        let cache = fs::read(h.root.cache_path()).unwrap();
        let files = skill_file_bytes(&seed);
        let sentinel = h.root.sentinel();
        let intent = h.service.begin_uninstall("9").unwrap();
        let old = h.call("before-terminal-reply");
        h.enter(2);
        let refresh = refresh_operation_id(&h.root);
        h.service.settle_uninstall(&intent, settlement).unwrap();
        let confirmed: &[&str] = if settlement == PluginUninstallSettlement::Removed { &["9"] } else { &[] };
        assert_uninstall_custody_with_auth(&h.service, &h.root, confirmed, &[], &[], Some(&refresh));
        let mut stale = two_plugins(&h.root, "must-not-publish");
        stale.auth_blocked.push(auth_block("10"));
        h.release.send(LoadStep::Loaded(stale)).unwrap();
        assert_eq!(old.finish().unwrap_err(), "plugin skills sync superseded by an uninstall observation");
        assert_eq!(fs::read(h.root.cache_path()).unwrap(), cache);
        assert_skill_file_bytes(&files);
        assert!(sentinel.exists());
        assert!(h.root.staged_files().is_empty());
        assert!(!h.service.admission_reader().is_denied("10"));
        assert_eq!(h.service.admission_reader().is_denied("9"), settlement == PluginUninstallSettlement::Removed);
        assert_eq!(h.events.lock().unwrap().len(), 1);
        assert_eq!(record_ids(&h.step("after-terminal-reply", LoadStep::Loaded(two_plugins(&h.root, "fresh"))).unwrap()), vec!["10", "9"]);
        assert_uninstall_custody(&h.service, &h.root, &[], &[]);
        assert!(!sentinel.exists());
        h.assert_calls(3);
    }
}

#[test]
fn uninstall_recovery_loads_exact_pending_and_confirmed_facts_without_invoking_the_loader() {
    use crate::workflows::workflow_library::get_global_workflows_dir;
    use crate::workflows::workflow_store::FileWorkflowStore;

    let h = Harness::new();
    let seed = h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
    let files = skill_file_bytes(&seed);
    let cache = fs::read(h.root.cache_path()).unwrap();
    let sentinel = h.root.sentinel();
    confirm_uninstall(&h.service, "9").unwrap();
    let intent = h.service.begin_uninstall("9").unwrap();
    let old_reader = h.service.admission_reader();
    let journal = fs::read(h.root.uninstall_state_path()).unwrap();
    h.service.dispose();
    let recovered = Arc::new(SandPluginSkillsService::new(&h.root.0, h.loader.clone())
        .with_now(Arc::new(|| 1_000)));
    let reader = recovered.admission_reader();
    assert!(!Arc::ptr_eq(&old_reader, &reader), "fresh construction reads durable state, not the retained Arc");
    assert_uninstall_custody(&recovered, &h.root, &["9"], &[&intent]);
    assert_eq!(fs::read(h.root.uninstall_state_path()).unwrap(), journal);
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), cache);
    assert_eq!(recovered.current_index().unwrap().skills, seed);
    assert_eq!(record_ids(&recovered.current()), vec!["10"]);
    assert!(reader.is_denied("9"));
    assert!(!reader.is_denied("10"));
    assert!(recovered.current_auth_blocked().is_empty());
    h.assert_calls(1);
    let workflows = FileWorkflowStore::new(
        h.root.0.join("agents/recovered"), get_global_workflows_dir(&h.root.0),
    ).with_plugin_skills_admission(Arc::clone(&reader));
    assert!(workflows.get("plugin-9-test").is_none());
    assert_eq!(workflows.get("plugin-10-test").unwrap().body, "other original skill content");

    let fresh = Call::start(&recovered, "fresh-after-recovery");
    h.enter(2);
    h.release.send(LoadStep::Loaded(two_plugins(&h.root, "fresh"))).unwrap();
    assert_eq!(record_ids(&fresh.finish().unwrap()), vec!["10"]);
    assert_uninstall_custody(&recovered, &h.root, &["9"], &[&intent]);
    assert_eq!(fs::read(h.root.uninstall_state_path()).unwrap(), journal);
    assert!(reader.is_denied("9"));
    assert!(sentinel.exists());
    assert_skill_file_bytes(&files);
    assert_eq!(
        h.service.settle_uninstall(&intent, PluginUninstallSettlement::Removed).unwrap_err(),
        "plugin skills owner is disposed; uninstall state was not changed",
    );
    assert_eq!(fs::read(h.root.uninstall_state_path()).unwrap(), journal);
    h.assert_calls(2);
}

#[test]
fn uninstall_recovery_fault_requires_a_fresh_owner_after_repair() {
    use crate::extensions::managed_setup::managed_skills_cache::{
        ManagedSkill, get_managed_skills_dir, write_managed_skills_cache,
    };
    use crate::workflows::workflow_library::{WorkflowSpec, get_global_workflows_dir};
    use crate::workflows::workflow_store::FileWorkflowStore;

    for fault in ["invalid-image", "duplicate-id", "missing-initialized-image", "invalid-marker", "unreadable-image"] {
        let h = Harness::new();
        let seed = h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
        let files = skill_file_bytes(&seed);
        let cache = fs::read(h.root.cache_path()).unwrap();
        let sentinel = h.root.sentinel();
        write_managed_skills_cache(get_managed_skills_dir(&h.root.0), &[ManagedSkill {
            id: "managed-recovery-control".into(), name: "Managed".into(),
            description: "MANAGED_DESCRIPTION".into(), body: "MANAGED_BODY".into(),
        }], 1_000.0).unwrap();
        let before = FileWorkflowStore::new(
            h.root.0.join("agents/recovery-fault"), get_global_workflows_dir(&h.root.0),
        ).with_plugin_skills_admission(h.service.admission_reader());
        let user = before.create(&WorkflowSpec {
            name: "Recovery user control".into(), description: "USER_DESCRIPTION".into(),
            body: "USER_BODY".into(), trigger: None, source_ref: None,
        }).unwrap().unwrap();
        confirm_uninstall(&h.service, "9").unwrap();
        let journal = fs::read(h.root.uninstall_state_path()).unwrap();
        h.service.dispose();
        match fault {
            "invalid-image" => fs::write(h.root.uninstall_state_path(), b"{incomplete").unwrap(),
            "duplicate-id" => fs::write(h.root.uninstall_state_path(),
                br#"{"formatVersion":1,"confirmedPluginIds":["9","9"],"pending":[]}"#).unwrap(),
            "missing-initialized-image" => fs::remove_file(h.root.uninstall_state_path()).unwrap(),
            "invalid-marker" => fs::write(h.root.uninstall_version_path(), b"2\n").unwrap(),
            "unreadable-image" => {
                fs::remove_file(h.root.uninstall_state_path()).unwrap();
                fs::create_dir(h.root.uninstall_state_path()).unwrap();
                fs::write(h.root.uninstall_state_path().join("foreign.txt"), b"preserve unknown state").unwrap();
            }
            _ => unreachable!(),
        }
        let fault_bytes = fs::read(h.root.uninstall_state_path()).ok();
        let marker_bytes = fs::read(h.root.uninstall_version_path()).unwrap();
        let faulted = SandPluginSkillsService::new(&h.root.0, h.loader.clone());
        let reader = faulted.admission_reader();
        assert_eq!(faulted.uninstall_recovery_status(), Some(json!({
            "confirmedPluginIds": [], "pending": [], "fault": "uninstall-state-unavailable",
        })), "{fault}");
        assert!(reader.is_denied("9"));
        assert!(reader.is_denied("10"));
        assert!(faulted.current().is_empty());
        assert_eq!(faulted.current_index().unwrap().skills, seed);
        assert!(faulted.current_auth_blocked().is_empty());
        let workflows = FileWorkflowStore::new(
            h.root.0.join("agents/recovery-fault"), get_global_workflows_dir(&h.root.0),
        ).with_plugin_skills_admission(Arc::clone(&reader));
        assert!(workflows.list().iter().all(|record| record.plugin_id.is_none()));
        assert!(workflows.get("plugin-9-test").is_none());
        assert!(workflows.get("plugin-10-test").is_none());
        assert_eq!(workflows.get(&user.id).unwrap().body, "USER_BODY");
        assert_eq!(workflows.get("managed-recovery-control").unwrap().body, "MANAGED_BODY");
        assert_eq!(faulted.begin_uninstall("10").unwrap_err(), "plugin uninstall recovery state is unavailable");
        assert_eq!(faulted.sync("must-not-reset-fault").unwrap_err(), "plugin uninstall recovery state is unavailable");
        assert_eq!(fs::read(h.root.uninstall_state_path()).ok(), fault_bytes);
        assert_eq!(fs::read(h.root.uninstall_version_path()).unwrap(), marker_bytes);
        assert_eq!(fs::read(h.root.cache_path()).unwrap(), cache);
        assert_skill_file_bytes(&files);
        assert!(sentinel.exists());
        h.assert_calls(1);

        // Repair is an explicit fixture/operator action. The faulted owner may
        // neither erase unknown state nor silently treat repaired bytes as a grant.
        if h.root.uninstall_state_path().is_dir() {
            assert_eq!(fs::read(h.root.uninstall_state_path().join("foreign.txt")).unwrap(), b"preserve unknown state");
            fs::remove_dir_all(h.root.uninstall_state_path()).unwrap();
        }
        fs::write(h.root.uninstall_state_path(), &journal).unwrap();
        fs::write(h.root.uninstall_version_path(), b"1\n").unwrap();
        assert_eq!(faulted.sync("repair-does-not-reset-live-owner").unwrap_err(), "plugin uninstall recovery state is unavailable");
        assert!(reader.is_denied("10"));
        h.assert_calls(1);
        faulted.dispose();
        let repaired = Arc::new(SandPluginSkillsService::new(&h.root.0, h.loader.clone())
            .with_now(Arc::new(|| 1_000)));
        assert_uninstall_custody(&repaired, &h.root, &["9"], &[]);
        assert_eq!(record_ids(&repaired.current()), vec!["10"]);
        assert!(repaired.admission_reader().is_denied("9"));
        assert!(!repaired.admission_reader().is_denied("10"));
        let fresh = Call::start(&repaired, "fresh-after-repair");
        h.enter(2);
        h.release.send(LoadStep::Loaded(two_plugins(&h.root, "reinstalled"))).unwrap();
        assert_eq!(record_ids(&fresh.finish().unwrap()), vec!["10", "9"]);
        assert_uninstall_custody(&repaired, &h.root, &[], &[]);
        assert!(reader.is_denied("9"), "the retired faulted reader is not rewritten by a new owner");
        assert!(reader.is_denied("10"));
        assert_skill_file_bytes(&files);
        assert!(!sentinel.exists());
        h.assert_calls(2);
    }
}

#[test]
fn uninstall_recovery_preserves_a_valid_unmarked_image_until_an_explicit_transition() {
    let h = Harness::new();
    h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
    let nine = h.service.begin_uninstall("9").unwrap();
    let journal = fs::read(h.root.uninstall_state_path()).unwrap();
    fs::remove_file(h.root.uninstall_version_path()).unwrap();
    h.service.dispose();
    let recovered = Arc::new(SandPluginSkillsService::new(&h.root.0, h.loader.clone())
        .with_now(Arc::new(|| 1_000)));
    assert_eq!(recovered.uninstall_recovery_status(), Some(json!({
        "confirmedPluginIds": [],
        "pending": [{"pluginId": "9", "operationId": nine.operation_id}],
    })));
    assert!(recovered.admission_reader().is_denied("9"));
    assert!(!recovered.admission_reader().is_denied("10"));
    assert!(!h.root.uninstall_version_path().exists());
    assert_eq!(fs::read(h.root.uninstall_state_path()).unwrap(), journal);
    h.assert_calls(1);
    let fresh = Call::start(&recovered, "fresh-with-unmarked-intent");
    h.enter(2);
    h.release.send(LoadStep::Loaded(two_plugins(&h.root, "fresh"))).unwrap();
    assert_eq!(record_ids(&fresh.finish().unwrap()), vec!["10"]);
    // AR-01 makes an actual refresh an explicit durable transition. Loading
    // the unmarked image above remains read-only; reservation initializes it.
    assert_eq!(fs::read(h.root.uninstall_version_path()).unwrap(), b"1\n");
    assert_eq!(fs::read(h.root.uninstall_state_path()).unwrap(), journal);
    let ten = recovered.begin_uninstall("10").unwrap();
    assert_uninstall_custody(&recovered, &h.root, &[], &[&ten, &nine]);
    recovered.settle_uninstall(&ten, PluginUninstallSettlement::NotDispatched).unwrap();
    recovered.settle_uninstall(&nine, PluginUninstallSettlement::NotRemoved).unwrap();
    assert_uninstall_custody(&recovered, &h.root, &[], &[]);
    assert_eq!(record_ids(&recovered.current()), vec!["10", "9"]);
    h.assert_calls(2);
}

#[test]
fn uninstall_release_failure_after_valid_cache_publication_keeps_denial_and_files() {
    let gate = BoundaryGate::at_visit(2);
    let h = Harness::with_hooks(
        Arc::new(|| 1_000),
        ServiceTestHooks { after_rename: Some(Arc::clone(&gate.pause)), ..ServiceTestHooks::default() },
    );
    let seed = h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
    let files = skill_file_bytes(&seed);
    let sentinel = h.root.sentinel();
    confirm_uninstall(&h.service, "9").unwrap();
    let journal = fs::read(h.root.uninstall_state_path()).unwrap();
    let reader = h.service.admission_reader();
    let fresh = h.call("valid-cache-but-failed-durable-release");
    h.enter(2);
    h.release.send(LoadStep::Loaded(two_plugins(&h.root, "fresh"))).unwrap();
    gate.wait();
    let published = read_plugin_skills_snapshot(get_plugin_skills_dir(&h.root.0)).unwrap();
    assert_eq!(record_ids(&published.index.skills), vec!["10", "9"]);
    assert!(published.index.skills.iter().all(|record| record.plugin_version == "fresh"));
    let cache = fs::read(h.root.cache_path()).unwrap();
    assert!(reader.is_denied("9"));
    assert!(!reader.is_denied("10"));
    assert!(matches!(h.service.publication.try_lock(), Err(std::sync::TryLockError::WouldBlock)));
    assert!(h.service.state.try_lock().is_ok());
    assert!(h.service.admission.state.try_lock().is_ok());
    let preserved = get_plugin_skills_dir(&h.root.0).join("preserved-uninstall-state.fixture");
    fs::rename(h.root.uninstall_state_path(), &preserved).unwrap();
    fs::create_dir(h.root.uninstall_state_path()).unwrap();
    fs::write(h.root.uninstall_state_path().join("foreign.txt"), b"preserve release obstruction").unwrap();
    gate.release.send(()).unwrap();
    assert_eq!(fresh.finish().unwrap_err(), "plugin uninstall recovery state is unavailable");
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), cache);
    assert_eq!(fs::read(&preserved).unwrap(), journal);
    assert_eq!(fs::read(h.root.uninstall_state_path().join("foreign.txt")).unwrap(), b"preserve release obstruction");
    assert_eq!(h.service.uninstall_recovery_status(), Some(json!({
        "confirmedPluginIds": ["9"], "pending": [], "fault": "uninstall-state-unavailable",
    })));
    assert!(reader.is_denied("9"));
    assert!(reader.is_denied("10"), "the failed durable outcome is an explicit whole-plugin recovery fault");
    assert!(h.service.current().is_empty());
    assert!(h.service.current_auth_blocked().is_empty());
    assert_skill_file_bytes(&files);
    assert!(sentinel.exists());
    assert!(h.root.staged_files().is_empty());
    assert_eq!(h.events.lock().unwrap().len(), 1, "failed durable release is not a publication-success event");

    fs::remove_dir_all(h.root.uninstall_state_path()).unwrap();
    fs::rename(&preserved, h.root.uninstall_state_path()).unwrap();
    assert_eq!(h.call("still-faulted").finish().unwrap_err(), "plugin uninstall recovery state is unavailable");
    h.assert_calls(2);
    h.service.dispose();
    let recovered = SandPluginSkillsService::new(&h.root.0, Arc::new(StaticLoader(two_plugins(&h.root, "fresh"))))
        .with_now(Arc::new(|| 1_000));
    assert_uninstall_custody(&recovered, &h.root, &["9"], &[]);
    assert_eq!(record_ids(&recovered.current()), vec!["10"]);
    assert_eq!(record_ids(&recovered.sync("durable-release-repaired").unwrap()), vec!["10", "9"]);
    assert_uninstall_custody(&recovered, &h.root, &[], &[]);
    assert!(!recovered.admission_reader().is_denied("9"));
    assert!(!sentinel.exists());
    assert_skill_file_bytes(&files);
}

#[test]
fn pending_uninstall_same_root_successor_rejects_every_retired_settlement() {
    let h = Harness::new();
    let seed = h.step("seed", LoadStep::Loaded(two_plugins(&h.root, "retained"))).unwrap();
    let files = skill_file_bytes(&seed);
    let sentinel = h.root.sentinel();
    let intent = h.service.begin_uninstall("9").unwrap();
    let journal = fs::read(h.root.uninstall_state_path()).unwrap();
    let reader = h.service.admission_reader();
    let old = h.call("old-pending-load");
    h.enter(2);
    let active_journal = fs::read(h.root.uninstall_state_path()).unwrap();
    let refresh = refresh_operation_id(&h.root);
    let successor = h.service.successor_with_loader(Arc::new(StaticLoader(two_plugins(&h.root, "successor")))).unwrap();
    assert!(Arc::ptr_eq(&reader, &successor.admission_reader()));
    assert_uninstall_custody_with_auth(&successor, &h.root, &[], &[&intent], &[], Some(&refresh));
    for settlement in [
        PluginUninstallSettlement::NotDispatched,
        PluginUninstallSettlement::NotRemoved,
        PluginUninstallSettlement::Removed,
    ] {
        assert_eq!(
            h.service.settle_uninstall(&intent, settlement).unwrap_err(),
            "plugin skills owner is disposed; uninstall state was not changed",
        );
        assert_eq!(fs::read(h.root.uninstall_state_path()).unwrap(), active_journal);
        assert_uninstall_custody_with_auth(&successor, &h.root, &[], &[&intent], &[], Some(&refresh));
    }
    assert_eq!(h.service.begin_uninstall("10").unwrap_err(), "plugin skills owner is disposed; uninstall state was not changed");
    assert_eq!(record_ids(&successor.sync("fresh-successor-while-pending").unwrap()), vec!["10"]);
    assert!(reader.is_denied("9"));
    assert!(!reader.is_denied("10"));
    assert!(sentinel.exists());
    let cache = fs::read(h.root.cache_path()).unwrap();
    let mut stale = two_plugins(&h.root, "must-not-publish");
    stale.auth_blocked.push(auth_block("10"));
    h.release.send(LoadStep::Loaded(stale)).unwrap();
    assert_eq!(record_ids(&old.finish().unwrap()), vec!["10"]);
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), cache);
    assert_eq!(fs::read(h.root.uninstall_state_path()).unwrap(), journal);
    assert!(!reader.is_denied("10"));
    h.service.dispose();
    assert!(!successor.state.lock().unwrap().disposed);
    successor.settle_uninstall(&intent, PluginUninstallSettlement::Removed).unwrap();
    assert_uninstall_custody(&successor, &h.root, &["9"], &[]);
    assert!(reader.is_denied("9"));
    assert_eq!(record_ids(&successor.sync("fresh-after-known-successor-reply").unwrap()), vec!["10", "9"]);
    assert_uninstall_custody(&successor, &h.root, &[], &[]);
    assert!(!reader.is_denied("9"));
    assert!(!sentinel.exists());
    assert_skill_file_bytes(&files);
    assert_eq!(h.service.retire_for_replacement().unwrap_err(), "plugin skills replacement already claimed");
    assert_eq!(h.events.lock().unwrap().len(), 1);
    h.assert_calls(2);
}
