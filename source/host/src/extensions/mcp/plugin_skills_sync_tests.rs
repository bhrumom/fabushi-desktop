use super::*;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Weak;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde_json::json;

use super::super::plugin_skills_cache::get_plugin_skills_cache_path;

const DEADLINE: Duration = Duration::from_secs(5);

// These private channel checkpoints pause actual service boundaries. Shipping
// builds contain neither the hooks nor a callback under the publication mutex.
#[derive(Default)]
pub(super) struct ServiceTestHooks {
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

struct BoundaryGate {
    pause: Arc<BoundaryPause>,
    entered: Receiver<()>,
    release: Sender<()>,
}

impl BoundaryGate {
    fn at_visit(visit: usize) -> Self {
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

    fn wait(&self) {
        self.entered.recv_timeout(DEADLINE).expect("service boundary not reached");
    }
}

enum LoadStep {
    Loaded(LoadedPlugins),
    Failed,
    Panicked,
}

struct GateLoader {
    entered: Sender<usize>,
    release: Mutex<Receiver<LoadStep>>,
    calls: AtomicUsize,
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

struct TempRoot(PathBuf);
impl TempRoot {
    fn new() -> Self {
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

    fn cache_path(&self) -> PathBuf {
        get_plugin_skills_cache_path(get_plugin_skills_dir(&self.0))
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

    fn sentinel(&self) -> PathBuf {
        let path = self.0.join("plugins/cache/market/unlisted/keep.txt");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "must survive an incomplete pass").unwrap();
        path
    }

    fn staged_files(&self) -> Vec<PathBuf> {
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

struct Call {
    result: Receiver<Result<SyncResult, ()>>,
    thread: JoinHandle<()>,
}
impl Call {
    fn start(service: &Arc<SandPluginSkillsService>, trigger: &'static str) -> Self {
        let service = Arc::clone(service);
        let (tx, result) = mpsc::channel();
        let thread = thread::spawn(move || {
            let outcome = catch_unwind(AssertUnwindSafe(|| service.sync(trigger))).map_err(|_| ());
            let _ = tx.send(outcome);
        });
        Self { result, thread }
    }

    fn assert_waiting(&self) {
        assert!(matches!(self.result.try_recv(), Err(TryRecvError::Empty)));
    }

    fn outcome(self) -> Result<SyncResult, ()> {
        let result = self
            .result
            .recv_timeout(DEADLINE)
            .expect("sync did not settle");
        self.thread.join().unwrap();
        result
    }

    fn finish(self) -> SyncResult {
        self.outcome().expect("unexpected sync panic")
    }
}

struct Harness {
    root: TempRoot,
    service: Arc<SandPluginSkillsService>,
    loader: Arc<GateLoader>,
    entered: Receiver<usize>,
    release: Sender<LoadStep>,
    events: Arc<Mutex<Vec<PluginSkillsSyncEvent>>>,
}
impl Harness {
    fn new() -> Self {
        Self::with_now(Arc::new(|| 1_000))
    }

    fn with_now(now: Arc<dyn Fn() -> i64 + Send + Sync>) -> Self {
        Self::with_hooks(now, ServiceTestHooks::default())
    }

    fn with_hooks(now: Arc<dyn Fn() -> i64 + Send + Sync>, hooks: ServiceTestHooks) -> Self {
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

    fn call(&self, trigger: &'static str) -> Call {
        Call::start(&self.service, trigger)
    }

    fn enter(&self, pass: usize) {
        assert_eq!(
            self.entered
                .recv_timeout(DEADLINE)
                .expect("loader did not start"),
            pass
        );
    }

    fn pending(&self, trigger: &str) {
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

    fn step(&self, trigger: &'static str, step: LoadStep) -> SyncResult {
        let next = self.loader.calls.load(Ordering::SeqCst) + 1;
        let call = self.call(trigger);
        self.enter(next);
        self.release.send(step).unwrap();
        call.finish()
    }

    fn assert_calls(&self, calls: usize) {
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

fn auth_block(id: &str) -> PluginAuthBlock {
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
    let h = Harness::new();
    let cache_dir = get_plugin_skills_dir(&h.root.0);
    fs::write(&cache_dir, "existing non-directory cache path").unwrap();
    let bytes = fs::read(&cache_dir).unwrap();
    let sentinel = h.root.sentinel();
    assert!(
        h.step("stage-failure", LoadStep::Loaded(h.root.loaded("v1")))
            .is_err(),
    );
    assert_eq!(fs::read(&cache_dir).unwrap(), bytes);
    assert!(sentinel.exists());
    assert!(h.root.0.join("plugins/cache/market/plugin").exists());
    assert!(h.events.lock().unwrap().is_empty());
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

fn two_plugins(root: &TempRoot, version: &str) -> LoadedPlugins {
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

fn refuse_nine_with_invalid_stage(root: &TempRoot) -> LoadedPlugins {
    let mut loaded = two_plugins(root, "must-not-publish");
    loaded.auth_blocked.push(auth_block("9"));
    // This is a valid authoritative loader aggregate. Its generated record for
    // the unrelated plugin 10 fails the real cache parser before staging.
    loaded.plugins[1].install_path = PathBuf::from("relative/other");
    loaded
}

fn skill_file_bytes(records: &[PluginSkillRecord]) -> Vec<(PathBuf, Vec<u8>)> {
    records.iter().map(|record| {
        let path = PathBuf::from(&record.file_path);
        let bytes = fs::read(&path).unwrap();
        (path, bytes)
    }).collect()
}

fn assert_skill_file_bytes(files: &[(PathBuf, Vec<u8>)]) {
    for (path, bytes) in files {
        assert_eq!(&fs::read(path).unwrap(), bytes, "{}", path.display());
    }
}

fn record_ids(records: &[PluginSkillRecord]) -> Vec<&str> {
    let mut ids = records.iter().map(|record| record.plugin_id.as_str()).collect::<Vec<_>>();
    ids.sort_unstable();
    ids
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
        assert!(h.service.admission.denied.try_lock().is_ok());
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
        let _guard = admission.denied.lock().unwrap();
        panic!("poison the private admission mutex");
    }).join().is_err());
    assert!(reader.is_denied("9"));
    assert!(reader.is_denied("10"));
    assert!(h.service.current().is_empty());
    assert_eq!(h.service.current_index().unwrap().skills, seed);
    assert_eq!(
        h.step("poisoned-refresh", LoadStep::Loaded(two_plugins(&h.root, "fresh"))).unwrap_err(),
        "plugin skills live admission poisoned",
    );
    assert_eq!(fs::read(h.root.cache_path()).unwrap(), bytes);
    h.service.dispose();
    assert_empty(h.call("disposed").finish());
    let successor = h.service.successor_with_loader(Arc::new(StaticLoader(two_plugins(&h.root, "fresh")))).unwrap();
    assert!(Arc::ptr_eq(&reader, &successor.admission_reader()));
    assert!(successor.current().is_empty());
    assert_eq!(successor.sync("startup").unwrap_err(), "plugin skills live admission poisoned");
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
    assert!(h.service.admission.denied.try_lock().is_ok());
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
