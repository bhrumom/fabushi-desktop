use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use mahayana_host_runtime::automations::automation::{AutomationSpec, AUTOMATION_MAX_RUN_HISTORY};
use mahayana_host_runtime::automations::automation_id::stable_automation_id;
use mahayana_host_runtime::automations::automation_store::FileAutomationStore;
use mahayana_host_runtime::extensions::automations::backend_relay_source::{
    BackendRelayRuntime, map_relay_wire_event, relay_subscription_request,
};
use mahayana_host_runtime::extensions::automations::backend_transport::{
    AutomationsBackendError, AutomationsBackendTransport,
};
use mahayana_host_runtime::extensions::automations::fire_delivery::{
    PreparedBackendFire, prepare_backend_fire,
};
use mahayana_host_runtime::extensions::automations::sand_automation_cloud_sync::{
    ScheduledCloudAutomation, sand_cloud_definition,
};
use mahayana_host_runtime::extensions::automations::sand_automation_fire_consumer::{
    AutomationFireBackendRuntime, BackendAutomationFire, FireCompletion, NOTIFY_DRAIN_FLOOR_MS,
    NOTIFY_SAFETY_POLL_MS,
};
use mahayana_host_runtime::extensions::session::agent_db::CronSessionRunRecord;
use mahayana_host_runtime::extensions::session::agent_session::{
    AgentAutomationEntry, SandAgentSessionStore,
};
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::transcript::automation_run_path::{
    AutomationExecutionResult, AutomationRunObservation, FireAutomationOutcome,
};
use mahayana_host_runtime::extensions::transcript::automation_runtime::AutomationRuntime;
use serde_json::{Value, json};

#[derive(Default)]
struct MockTransport {
    responses: Mutex<VecDeque<Result<Value, AutomationsBackendError>>>,
    calls: Mutex<Vec<(String, Value)>>,
}

impl MockTransport {
    fn with_responses(responses: Vec<Result<Value, AutomationsBackendError>>) -> Arc<Self> {
        Arc::new(Self {
            responses: Mutex::new(responses.into()),
            calls: Mutex::new(Vec::new()),
        })
    }

    fn calls(&self) -> Vec<(String, Value)> {
        self.calls.lock().expect("calls mutex").clone()
    }
}

impl AutomationsBackendTransport for MockTransport {
    fn post_json(&self, path: &str, body: &Value) -> Result<Value, AutomationsBackendError> {
        self.calls
            .lock()
            .expect("calls mutex")
            .push((path.to_string(), body.clone()));
        self.responses
            .lock()
            .expect("responses mutex")
            .pop_front()
            .expect("mock response")
    }
}

#[test]
fn relay_subscription_request_is_stable_and_deduplicated() {
    let request = relay_subscription_request(
        &[
            json!({"type":"slack","channel":"#b"}),
            json!({"type":"slack","channel":"#a"}),
            json!({"type":"slack","channel":"#b"}),
        ],
        &[
            json!({"type":"github","repo":"OpenAI/Repo","events":["pr-opened","pr-merged"]}),
            json!({"type":"github","repo":"openai/repo","events":["pr-opened"]}),
        ],
    );
    assert_eq!(
        request,
        json!({
            "slackChannels":["#a","#b"],
            "githubRepos":["openai/repo"],
            "githubKinds":["pr-merged","pr-opened"],
        })
    );
}

#[test]
fn relay_wire_mapping_matches_grok_slack_and_github_shapes() {
    assert_eq!(
        map_relay_wire_event(
            &json!({
                "source":"slack",
                "kind":"reaction",
                "channelName":"alerts",
                "channelId":"C1",
                "senderSlackUserId":"U1",
                "reactionEmoji":":eyes:",
                "text":"ship it"
            }),
            42,
        ),
        Some(json!({
            "source":"slack",
            "channel":"#alerts",
            "sender":"@U1",
            "text":"ship it",
            "isMention":false,
            "isSelf":false,
            "reactionEmoji":":eyes:",
            "timestampMs":42
        }))
    );
    assert_eq!(
        map_relay_wire_event(
            &json!({
                "source":"github",
                "repo":"openai/repo",
                "kind":"pr-opened",
                "actor":"octocat",
                "prOwner":"owner"
            }),
            43,
        ),
        Some(json!({
            "source":"github",
            "repo":"openai/repo",
            "kind":"pr-opened",
            "title":"",
            "actor":"octocat",
            "prOwner":"owner",
            "timestampMs":43
        }))
    );
}

#[test]
fn relay_runtime_registers_polls_and_acks_only_accepted_or_unmappable_events() {
    let transport = MockTransport::with_responses(vec![
        Ok(json!({"slack":{"status":"ok"},"github":{"status":"ok"}})),
        Ok(json!({"events":[
            {"id":"accepted","source":"slack","channelName":"alerts","text":"one"},
            {"id":"retry","source":"slack","channelName":"alerts","text":"two"},
            {"id":"bad","source":"unknown"}
        ]})),
        Ok(json!({"events":[
            {"id":"accepted","source":"slack","channelName":"alerts","text":"one"},
            {"id":"bad","source":"unknown"}
        ]})),
    ]);
    let mut runtime = BackendRelayRuntime::new(transport.clone());
    runtime.set_listeners(vec![json!({"type":"slack","channel":"#alerts","match":{"kind":"message"}})], vec![]);

    let delivered = runtime
        .tick(1_000, true, true, |event| event.get("text").and_then(Value::as_str) != Some("two"))
        .expect("first relay tick");
    assert_eq!(delivered, 1);
    assert_eq!(runtime.pending_ack_ids(), &["accepted".to_string(), "bad".to_string()]);

    runtime.tick(1_001, true, true, |_| true).expect("ack relay tick");
    let calls = transport.calls();
    assert_eq!(calls[0].0, "/sand/listener-subscriptions");
    assert_eq!(calls[1].0, "/sand/listener-events/poll");
    assert_eq!(calls[2].1, json!({"ackIds":["accepted","bad"]}));
}

#[test]
fn relay_runtime_backoff_suppresses_immediate_retry() {
    let transport = MockTransport::with_responses(vec![
        Err(AutomationsBackendError::Status { path:"/sand/listener-subscriptions".into(), status:503 }),
    ]);
    let mut runtime = BackendRelayRuntime::new(transport.clone());
    runtime.set_listeners(vec![json!({"type":"slack","channel":"#alerts"})], vec![]);

    assert!(runtime.tick(100, true, true, |_| true).is_err());
    assert_eq!(runtime.tick(101, true, true, |_| true).expect("backoff tick"), 0);
    assert_eq!(transport.calls().len(), 1);
}

#[test]
fn fire_runtime_polls_delivers_reports_and_acks_on_next_poll() {
    let transport = MockTransport::with_responses(vec![
        Ok(json!({"events":[{
            "id":"run-1",
            "sandAgentId":"agent-1",
            "automationId":"cloud-1",
            "timestampMs":10,
            "scheduledForMs":9
        }]})),
        Ok(json!({})),
        Ok(json!({"events":[]})),
    ]);
    let mut runtime = AutomationFireBackendRuntime::new(transport.clone());

    let count = runtime
        .tick(100, true, true, |event| {
            assert_eq!(event.id, "run-1");
            assert_eq!(event.sand_agent_id, "agent-1");
            true
        })
        .expect("fire tick");
    assert_eq!(count, 1);
    assert!(runtime.is_running("run-1"));
    runtime
        .complete("run-1", FireCompletion::succeeded())
        .expect("completion report");
    assert_eq!(runtime.pending_len(), 1);

    runtime.tick(101, true, true, |_| false).expect("ack fire tick");
    assert_eq!(runtime.pending_len(), 0);

    let calls = transport.calls();
    assert_eq!(calls[0], ("/sand/automation-events/poll".into(), json!({"ackRunUuids":[]})));
    assert_eq!(
        calls[1],
        (
            "/sand/automation-runs/complete".into(),
            json!({"runUuid":"run-1","status":"succeeded","errorMessage":null})
        )
    );
    assert_eq!(
        calls[2],
        ("/sand/automation-events/poll".into(), json!({"ackRunUuids":["run-1"]}))
    );
}

#[test]
fn fire_runtime_does_not_poll_until_run_execution_is_ready() {
    let transport = MockTransport::with_responses(vec![
        Ok(json!({"events":[]})),
    ]);
    let mut runtime = AutomationFireBackendRuntime::new(transport.clone());

    let count = runtime
        .tick_with_state(100, true, true, false, true, |_| true)
        .expect("not-ready tick");
    assert_eq!(count, 0);
    assert!(transport.calls().is_empty());

    runtime
        .tick_with_state(101, true, true, true, true, |_| true)
        .expect("ready tick");
    assert_eq!(transport.calls().len(), 1);
}

#[test]
fn fire_runtime_drains_once_while_no_server_schedulable_routines_exist() {
    let transport = MockTransport::with_responses(vec![
        Ok(json!({"events":[]})),
        Ok(json!({"events":[]})),
    ]);
    let mut runtime = AutomationFireBackendRuntime::new(transport.clone());

    runtime
        .tick_with_state(100, true, true, true, false, |_| true)
        .expect("initial drain");
    assert_eq!(transport.calls().len(), 1);

    runtime
        .tick_with_state(101, true, true, true, false, |_| true)
        .expect("suppressed empty drain");
    assert_eq!(transport.calls().len(), 1);

    runtime.start();
    runtime
        .tick_with_state(102, true, true, true, false, |_| true)
        .expect("config-reset drain");
    assert_eq!(transport.calls().len(), 2);
}

#[test]
fn fire_runtime_matches_frozen_notify_floor_and_safety_poll_gate() {
    let transport = MockTransport::with_responses(vec![
        Ok(json!({"events":[]})),
        Ok(json!({"events":[]})),
        Ok(json!({"events":[]})),
    ]);
    let mut runtime = AutomationFireBackendRuntime::new(transport.clone());

    runtime
        .tick_with_state(100, true, true, true, true, |_| true)
        .expect("initial connected poll");
    assert_eq!(transport.calls().len(), 1);

    runtime.request_drain();
    runtime
        .tick_with_state(
            100 + NOTIFY_DRAIN_FLOOR_MS - 1,
            true,
            true,
            true,
            true,
            |_| true,
        )
        .expect("notify floor");
    assert_eq!(transport.calls().len(), 1);

    runtime
        .tick_with_state(
            100 + NOTIFY_DRAIN_FLOOR_MS,
            true,
            true,
            true,
            true,
            |_| true,
        )
        .expect("notify drain");
    assert_eq!(transport.calls().len(), 2);

    runtime
        .tick_with_state(
            100 + NOTIFY_DRAIN_FLOOR_MS + NOTIFY_SAFETY_POLL_MS - 1,
            true,
            true,
            true,
            true,
            |_| true,
        )
        .expect("before safety poll");
    assert_eq!(transport.calls().len(), 2);

    runtime
        .tick_with_state(
            100 + NOTIFY_DRAIN_FLOOR_MS + NOTIFY_SAFETY_POLL_MS,
            true,
            true,
            true,
            true,
            |_| true,
        )
        .expect("safety poll");
    assert_eq!(transport.calls().len(), 3);
}

#[test]
fn fire_runtime_disconnected_and_owed_work_bypass_notify_gate() {
    let transport = MockTransport::with_responses(vec![
        Ok(json!({"events":[]})),
        Ok(json!({"events":[{
            "id":"run-1",
            "sandAgentId":"agent-1",
            "automationId":"cloud-1",
            "timestampMs":10
        }]})),
        Ok(json!({"events":[{"id":"run-1","sandAgentId":"agent-1","automationId":"cloud-1","timestampMs":10}]})),
    ]);
    let mut runtime = AutomationFireBackendRuntime::new(transport.clone());

    runtime
        .tick_with_state(100, true, false, true, true, |_| true)
        .expect("initial poll");
    runtime
        .tick_with_state(101, false, false, true, true, |_| true)
        .expect("disconnected poll");
    assert_eq!(transport.calls().len(), 2);
    assert!(runtime.is_running("run-1"));

    runtime
        .tick_with_state(102, true, false, true, true, |_| true)
        .expect("owed work poll");
    assert_eq!(transport.calls().len(), 3);
}

#[test]
fn fire_runtime_config_reset_does_not_impersonate_notify() {
    let transport = MockTransport::with_responses(vec![
        Ok(json!({"events":[],"nextPollAfterMs":60000})),
        Ok(json!({"events":[]})),
    ]);
    let mut runtime = AutomationFireBackendRuntime::new(transport.clone());

    runtime
        .tick_with_state(100, true, true, true, true, |_| true)
        .expect("initial poll");
    runtime.reset_poll_delay();
    runtime
        .tick_with_state(101, true, true, true, true, |_| true)
        .expect("config reset remains notify-gated");
    assert_eq!(transport.calls().len(), 1);

    runtime
        .tick_with_state(
            100 + NOTIFY_SAFETY_POLL_MS,
            true,
            true,
            true,
            true,
            |_| true,
        )
        .expect("safety poll after config reset");
    assert_eq!(transport.calls().len(), 2);
}

#[test]
fn fire_runtime_successful_completion_honors_next_poll_delay() {
    let transport = MockTransport::with_responses(vec![
        Ok(json!({"events":[{
            "id":"run-1",
            "sandAgentId":"agent-1",
            "automationId":"cloud-1",
            "timestampMs":10
        }]})),
        Ok(json!({})),
        Ok(json!({"events":[]})),
        Ok(json!({"events":[],"nextPollAfterMs":5000})),
    ]);
    let mut runtime = AutomationFireBackendRuntime::new(transport.clone());
    runtime
        .tick(100, true, true, |_| true)
        .expect("delivery tick");
    runtime
        .complete("run-1", FireCompletion::failed("boom"))
        .expect("successful completion");
    runtime.tick(101, true, true, |_| false).expect("ack tick");
    runtime.tick(102, true, true, |_| false).expect("empty tick");
    let before = transport.calls().len();
    runtime.tick(103, true, true, |_| false).expect("delayed tick");
    assert_eq!(transport.calls().len(), before);
}


#[test]
fn relay_runtime_preserves_scope_and_repo_degradation_status() {
    let transport = MockTransport::with_responses(vec![
        Ok(json!({
            "slack": {
                "status": "ok",
                "teams": [{
                    "channels": [{"input":"#private","isBotMember":false}],
                    "unresolvedChannels": ["#missing"]
                }]
            },
            "github": {
                "status": "ok",
                "repos": [
                    {"repo":"org/good","isSubscribed":true},
                    {"repo":"org/bad","isSubscribed":false,"detail":"Missing repository access."}
                ]
            }
        })),
        Ok(json!({"events":[]}))
    ]);
    let mut runtime = BackendRelayRuntime::new(transport);
    runtime.set_listeners(
        vec![json!({"type":"slack","channel":"#private","match":{"kind":"message"}})],
        vec![json!({"type":"github","repo":"org/bad","events":["pr-opened"]})],
    );
    runtime.tick(1_000, true, true, |_| true).expect("relay status tick");

    assert_eq!(runtime.slack_status.state, "listening");
    assert_eq!(runtime.slack_status.scope_issues.len(), 2);
    assert!(runtime.slack_status.detail.as_deref().unwrap_or_default().contains("Invite @Cursor"));
    assert!(runtime.slack_status.detail.as_deref().unwrap_or_default().contains("#missing"));
    assert_eq!(runtime.github_status.state, "listening");
    assert_eq!(
        runtime.github_status.detail.as_deref(),
        Some("Missing repository access.")
    );
}


#[test]
fn fire_runtime_never_acks_unconfirmed_completion_or_redispatches_redelivery() {
    for error in [
        AutomationsBackendError::Status { path: "/sand/automation-runs/complete".into(), status: 401 },
        AutomationsBackendError::Status { path: "/sand/automation-runs/complete".into(), status: 404 },
        AutomationsBackendError::Status { path: "/sand/automation-runs/complete".into(), status: 409 },
        AutomationsBackendError::Transport("completion response lost".into()),
    ] {
        let delivery = json!({"events":[{
            "id":"run-1", "sandAgentId":"agent-1", "automationId":"cloud-1", "timestampMs":10
        }]});
        let transport = MockTransport::with_responses(vec![
            Ok(delivery.clone()), Err(error.clone()),
            Ok(delivery.clone()), Err(error),
            Ok(delivery), Ok(json!({})),
            Ok(json!({"events":[]})),
        ]);
        let mut runtime = AutomationFireBackendRuntime::new(transport.clone());
        let mut dispatched = 0;
        runtime.tick(100, false, false, |_| { dispatched += 1; true }).unwrap();
        let completion = FireCompletion::succeeded();
        assert!(runtime.complete("run-1", completion).is_err());
        assert_eq!(runtime.pending_len(), 1);
        runtime.tick(101, false, false, |_| { dispatched += 1; true }).unwrap();
        assert_eq!(runtime.pending_len(), 1);
        runtime.tick(102, false, false, |_| { dispatched += 1; true }).unwrap();
        runtime.tick(103, false, false, |_| { dispatched += 1; true }).unwrap();
        assert_eq!(dispatched, 1);
        assert_eq!(runtime.pending_len(), 0);
        let calls = transport.calls();
        let polls = calls.iter().filter(|(path, _)| path == "/sand/automation-events/poll")
            .map(|(_, body)| body.clone()).collect::<Vec<_>>();
        assert_eq!(polls, vec![
            json!({"ackRunUuids":[]}), json!({"ackRunUuids":[]}),
            json!({"ackRunUuids":[]}), json!({"ackRunUuids":["run-1"]}),
        ]);
        let reports = calls.iter().filter(|(path, _)| path == "/sand/automation-runs/complete")
            .map(|(_, body)| body.clone()).collect::<Vec<_>>();
        assert_eq!(reports, vec![json!({"runUuid":"run-1", "status":"succeeded", "errorMessage":null}); 3]);
    }
}

const ORIGINAL_RUN_UUID: &str = "00000000-0000-4000-8000-000000000001";
const COALESCED_RUN_UUID: &str = "00000000-0000-4000-8000-000000000002";
const DISTINCT_RUN_UUID: &str = "00000000-0000-4000-8000-000000000003";

#[derive(Debug, Clone, Copy)]
enum RedeliveryCase {
    Event,
    RevisionedSchedule,
    LegacyCoveredSlot,
    Disabled,
    ChangedDefinition,
    UnavailableDefinition,
}

fn cloud_revision(entry: &AgentAutomationEntry) -> Option<String> {
    sand_cloud_definition(
        &entry.agent_id,
        &ScheduledCloudAutomation {
            id: entry.automation.id.clone(),
            name: entry.automation.name.clone(),
            prompt: entry.automation.prompt.clone(),
            is_enabled: entry.automation.is_enabled,
            trigger: entry.automation.trigger.clone(),
        },
        None,
    )
    .map(|definition| definition.hash)
}

#[derive(Debug, PartialEq)]
struct PersistedFireState {
    config: Vec<u8>,
    history: Vec<u8>,
    cron: Vec<CronSessionRunRecord>,
}

struct PersistedFireFixture {
    root: PathBuf,
    automation_id: String,
    fire: BackendAutomationFire,
    baseline: PersistedFireState,
}

impl PersistedFireFixture {
    fn new(case: RedeliveryCase) -> Self {
        let root = std::env::temp_dir().join(format!(
            "fabushi-known-run-redelivery-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4(),
        ));
        fs::create_dir(&root).expect("claim exclusive redelivery fixture");
        let owners = FireDeliveryOwners::open(&root);
        let sessions = SandAgentSessionStore::new(Arc::clone(&owners.workers));
        let agent = sessions.create_session(None, "user", None).expect("create real session");
        let mut spec = AutomationSpec {
            name: "Known run redelivery".into(),
            prompt: "Inspect the queue".into(),
            trigger: json!({"type":"group","listeners":[
                {"type":"cron","schedule":"0 * * * *"},
                {"type":"github","repo":"org/repo","events":["pr-opened"]}
            ]}),
            is_enabled: Some(true),
        };
        let created = sessions.create_agent_automation(&agent.id, &spec).expect("create definition");
        assert_eq!(created.len(), 1);
        let automation_id = created[0].id.clone();
        let revision = cloud_revision(&AgentAutomationEntry {
            agent_id: agent.id.clone(),
            automation: created[0].clone(),
        }).expect("original cloud definition");
        let store = sessions.automation_store_for(&agent.id).expect("admitted file owner");
        // Use the actual admitted persistence APIs. No serialized history or
        // SQLite row is fabricated, and no fixture copies their write logic.
        sessions.with_admitted_filesystem_mutation(&agent.id, || {
            assert!(store.record_run_definition(&automation_id, 100.0)
                .map_err(|error| error.to_string())?.is_some());
            let run = store.begin_run(
                &automation_id,
                if matches!(case, RedeliveryCase::Event) { "event" } else { "schedule" },
                100.0,
                None,
                Some(ORIGINAL_RUN_UUID),
                Some(&[COALESCED_RUN_UUID.to_owned()]),
            ).map_err(|error| error.to_string())?.expect("persist running history");
            assert_eq!(run.status, "running");
            assert_eq!(run.finished_at, None);
            assert_eq!(run.coalesced_run_ids, Some(vec![COALESCED_RUN_UUID.to_owned()]));
            Ok(())
        }).expect("admitted history creation");
        assert!(owners.workers.begin_cron_session_run(
            &agent.id, &automation_id, ORIGINAL_RUN_UUID, 100.0,
        ).expect("persist running cron row"));

        match case {
            RedeliveryCase::Disabled => {
                sessions.set_agent_automation_enabled(&agent.id, &automation_id, false)
                    .expect("disable persisted definition");
            }
            RedeliveryCase::ChangedDefinition | RedeliveryCase::UnavailableDefinition => {
                spec.prompt = "Updated definition after admission".into();
                if matches!(case, RedeliveryCase::UnavailableDefinition) {
                    // The real cloud mapper cannot revision a Slack DM listener.
                    spec.trigger = json!({"type":"group","listeners":[
                        {"type":"cron","schedule":"0 * * * *"},
                        {"type":"slack","channel":"@alice","match":{"kind":"message"}}
                    ]});
                }
                sessions.update_agent_automation(&agent.id, &automation_id, &spec)
                    .expect("change persisted definition");
            }
            _ => {}
        }
        let fire = BackendAutomationFire {
            id: ORIGINAL_RUN_UUID.into(),
            sand_agent_id: agent.id.clone(),
            automation_id: stable_automation_id(&agent.id, &automation_id),
            timestamp_ms: 100,
            definition_revision: (!matches!(case,
                RedeliveryCase::Event | RedeliveryCase::LegacyCoveredSlot
            )).then_some(revision),
            scheduled_for_ms: Some(90),
            event: matches!(case, RedeliveryCase::Event).then(|| json!({
                "source":"github","repo":"org/repo","kind":"pr-opened",
                "actor":"reviewer","timestampMs":100
            })),
        };
        let baseline = owners.snapshot(&agent.id, &automation_id);
        assert_eq!(baseline.cron.len(), 1);
        assert_eq!(baseline.cron[0].state, "running");
        assert_eq!(baseline.cron[0].updated_at_ms, 100.0);
        assert!(!baseline.cron[0].fence_state_known);
        assert!(owners.observed.lock().unwrap().is_empty());
        // Dropping these owners closes the real Session database before any
        // redelivery is prepared; each caller opens fresh runtime/file owners.
        drop(store);
        drop(sessions);
        drop(owners);
        Self { root, automation_id, fire, baseline }
    }

    fn fire_for(&self, run_uuid: &str) -> BackendAutomationFire {
        BackendAutomationFire { id: run_uuid.into(), ..self.fire.clone() }
    }

    fn delivery(&self, ids: &[&str]) -> Value {
        let events = ids.iter().map(|id| {
            let fire = self.fire_for(id);
            json!({
                "id": fire.id,
                "sandAgentId": fire.sand_agent_id,
                "automationId": fire.automation_id,
                "timestampMs": fire.timestamp_ms,
                "definitionRevision": fire.definition_revision,
                "scheduledForMs": fire.scheduled_for_ms,
                "event": fire.event,
            })
        }).collect::<Vec<_>>();
        json!({"events": events})
    }
}

impl Drop for PersistedFireFixture {
    fn drop(&mut self) {
        if std::thread::panicking() {
            eprintln!("retained failed known-run redelivery fixture: {}", self.root.display());
        } else {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

struct FireDeliveryOwners {
    workers: Arc<ProductionSessionWorkers>,
    runtime: AutomationRuntime,
    executions: Arc<AtomicUsize>,
    observed: Arc<Mutex<Vec<AutomationRunObservation>>>,
}

impl FireDeliveryOwners {
    fn open(root: &Path) -> Self {
        let workers = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
        let runtime = AutomationRuntime::new(Arc::clone(&workers));
        let observed = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&observed);
        runtime.run_path().set_run_reporter(Some(Arc::new(move |report| {
            sink.lock().unwrap().push(report.clone());
        })));
        Self { workers, runtime, executions: Arc::new(AtomicUsize::new(0)), observed }
    }

    fn store(&self, agent_id: &str) -> FileAutomationStore {
        SandAgentSessionStore::new(Arc::clone(&self.workers))
            .automation_store_for(agent_id).expect("reopened file owner")
    }

    fn snapshot(&self, agent_id: &str, automation_id: &str) -> PersistedFireState {
        let store = self.store(agent_id);
        PersistedFireState {
            config: fs::read(store.config_path(automation_id)).expect("definition bytes"),
            history: fs::read(store.runs_path(automation_id)).expect("history bytes"),
            cron: self.workers.list_cron_session_runs(agent_id).expect("real cron projection"),
        }
    }

    fn prepare(&self, fire: &BackendAutomationFire) -> PreparedBackendFire {
        prepare_backend_fire(
            &self.runtime.list_all_automations().expect("reopened automation history"),
            fire,
            cloud_revision,
            |entry, run_id| self.runtime.read_run_for_delivery(&entry.agent_id, &entry.automation.id, run_id),
        )
    }

    fn poll(&self, backend: &mut AutomationFireBackendRuntime, now_ms: u64) -> Vec<PreparedBackendFire> {
        let mut pending = Vec::new();
        backend.tick(now_ms, false, false, |event| {
            pending.push(event.clone());
            true
        }).expect("actual backend poll");
        // Production collects pending fires under the backend mutex, then the
        // Host prepares each fire and completes or abandons it after that lock.
        // Exercise those real owners here without launching a Host process.
        pending.into_iter().map(|fire| {
            let prepared = self.prepare(&fire);
            let completion = match &prepared {
                PreparedBackendFire::Abandon { .. } => {
                    backend.abandon(&fire.id);
                    None
                }
                PreparedBackendFire::Complete { completion, .. } => Some(completion.clone()),
                PreparedBackendFire::Schedule {
                    agent_id, automation_id, run_uuid, scheduled_for_ms,
                } => {
                    let executions = Arc::clone(&self.executions);
                    let outcome = self.runtime.run_server_scheduled_automation_with(
                        agent_id, automation_id, run_uuid.clone(),
                        scheduled_for_ms.map(|value| value as f64),
                        move |_| {
                            executions.fetch_add(1, Ordering::SeqCst);
                            Ok(AutomationExecutionResult::Completed)
                        },
                    ).expect("actual scheduled execution");
                    assert_eq!(outcome, Some(FireAutomationOutcome::Ok));
                    Some(FireCompletion::succeeded())
                }
                PreparedBackendFire::Event { agent_id, automation_id, run_uuid, event } => {
                    let executions = Arc::clone(&self.executions);
                    let outcome = self.runtime.enqueue_event_automation_fire_with(
                        agent_id, automation_id, event.clone(), Some(run_uuid.clone()),
                        Arc::new(move |_, _, _, _| {
                            executions.fetch_add(1, Ordering::SeqCst);
                            Ok(AutomationExecutionResult::Completed)
                        }),
                    ).expect("actual event execution");
                    assert_eq!(outcome, Some(FireAutomationOutcome::Ok));
                    Some(FireCompletion::succeeded())
                }
            };
            if let Some(completion) = completion {
                backend.complete(&fire.id, completion).expect("actual completion report");
            }
            prepared
        }).collect()
    }
}

impl Drop for FireDeliveryOwners {
    fn drop(&mut self) {
        self.runtime.dispose();
        self.workers.shutdown();
    }
}

#[test]
fn reopened_running_original_and_alias_never_execute_complete_or_ack() {
    for case in [
        RedeliveryCase::Event,
        RedeliveryCase::RevisionedSchedule,
        RedeliveryCase::LegacyCoveredSlot,
        RedeliveryCase::Disabled,
        RedeliveryCase::ChangedDefinition,
        RedeliveryCase::UnavailableDefinition,
    ] {
        let fixture = PersistedFireFixture::new(case);
        for generation in 0..2 {
            let owners = FireDeliveryOwners::open(&fixture.root);
            assert_eq!(owners.snapshot(&fixture.fire.sand_agent_id, &fixture.automation_id), fixture.baseline);
            // An absent ID still takes the normal validation path. This also
            // proves the persisted disabled/changed/unavailable cases are real.
            let distinct = owners.prepare(&fixture.fire_for(DISTINCT_RUN_UUID));
            assert!(match case {
                RedeliveryCase::Event => matches!(distinct, PreparedBackendFire::Event { .. }),
                RedeliveryCase::RevisionedSchedule => matches!(distinct, PreparedBackendFire::Schedule { .. }),
                RedeliveryCase::LegacyCoveredSlot => matches!(distinct,
                    PreparedBackendFire::Complete { reason: "slot_already_covered", .. }),
                RedeliveryCase::Disabled => matches!(distinct,
                    PreparedBackendFire::Complete { reason: "automation_disabled", .. }),
                RedeliveryCase::ChangedDefinition => matches!(distinct,
                    PreparedBackendFire::Complete { reason: "definition_changed", .. }),
                RedeliveryCase::UnavailableDefinition => matches!(distinct,
                    PreparedBackendFire::Abandon { reason: "definition_revision_unavailable" }),
            }, "distinct-ID validation control: {case:?}");
            let delivery = fixture.delivery(&[ORIGINAL_RUN_UUID, COALESCED_RUN_UUID]);
            let transport = MockTransport::with_responses(vec![Ok(delivery); 3]);
            let mut backend = AutomationFireBackendRuntime::new(transport.clone());
            for poll in 0..3 {
                assert_eq!(owners.poll(&mut backend, 1_000 + poll), vec![
                    PreparedBackendFire::Abandon { reason: "existing_run_unsettled" }; 2
                ], "case={case:?}, generation={generation}, poll={poll}");
                assert_eq!(backend.pending_len(), 0, "only volatile entries are abandoned");
                assert!(!backend.is_running(ORIGINAL_RUN_UUID));
                assert!(!backend.is_running(COALESCED_RUN_UUID));
                assert_eq!(owners.executions.load(Ordering::SeqCst), 0);
                assert!(owners.observed.lock().unwrap().is_empty());
                assert!(!owners.runtime.run_path().is_in_flight(
                    &fixture.fire.sand_agent_id, &fixture.automation_id,
                ));
                assert_eq!(owners.snapshot(&fixture.fire.sand_agent_id, &fixture.automation_id), fixture.baseline);
            }
            assert_eq!(transport.calls(), vec![
                ("/sand/automation-events/poll".into(), json!({"ackRunUuids":[]})); 3
            ], "no completion requests or ACK for {case:?}, generation={generation}");
        }
    }
}

#[test]
fn reopened_backend_reads_hidden_legacy_running_authority_beyond_display_history() {
    let fixture = PersistedFireFixture::new(RedeliveryCase::RevisionedSchedule);
    let legacy = {
        let owners = FireDeliveryOwners::open(&fixture.root);
        let store = owners.store(&fixture.fire.sand_agent_id);
        let mut rows: Vec<Value> = serde_json::from_slice(&fixture.baseline.history).unwrap();
        rows.extend((0..AUTOMATION_MAX_RUN_HISTORY).map(|index| json!({
            "id":format!("newer-terminal-{index}"),"startedAt":1_000 + index,
            "finishedAt":2_000 + index,"status":"ok","trigger":"manual",
        })));
        // Represent a valid file left by an older writer. Current admission
        // must not fabricate such overflow to exercise authoritative reads.
        SandAgentSessionStore::new(Arc::clone(&owners.workers))
            .with_admitted_filesystem_mutation(&fixture.fire.sand_agent_id, || {
                fs::write(store.runs_path(&fixture.automation_id), serde_json::to_vec(&rows).unwrap())
                    .map_err(|error| error.to_string())
            }).unwrap();
        let state = owners.snapshot(&fixture.fire.sand_agent_id, &fixture.automation_id);
        assert_eq!(state.config, fixture.baseline.config);
        assert_eq!(state.cron, fixture.baseline.cron);
        state
    };
    for generation in 0..2 {
        let owners = FireDeliveryOwners::open(&fixture.root);
        let display = owners.store(&fixture.fire.sand_agent_id).read_runs(&fixture.automation_id);
        assert_eq!(display.len(), AUTOMATION_MAX_RUN_HISTORY);
        assert!(!display.iter().any(|run| run.id == ORIGINAL_RUN_UUID));
        for run_id in [ORIGINAL_RUN_UUID, COALESCED_RUN_UUID] {
            let found = owners.runtime.read_run_for_delivery(
                &fixture.fire.sand_agent_id, &fixture.automation_id, run_id,
            ).unwrap().expect("complete legacy authority");
            assert_eq!(found.id, ORIGINAL_RUN_UUID);
            assert_eq!(found.status, "running");
        }
        let transport = MockTransport::with_responses(vec![
            Ok(fixture.delivery(&[ORIGINAL_RUN_UUID, COALESCED_RUN_UUID])); 3
        ]);
        let mut backend = AutomationFireBackendRuntime::new(transport.clone());
        for poll in 0..3 {
            assert_eq!(owners.poll(&mut backend, 1_000 + poll), vec![
                PreparedBackendFire::Abandon { reason: "existing_run_unsettled" }; 2
            ], "generation={generation}, poll={poll}");
            assert_eq!(backend.pending_len(), 0);
            assert_eq!(owners.executions.load(Ordering::SeqCst), 0);
            assert!(owners.observed.lock().unwrap().is_empty());
            assert_eq!(owners.snapshot(&fixture.fire.sand_agent_id, &fixture.automation_id), legacy);
        }
        assert_eq!(transport.calls(), vec![
            ("/sand/automation-events/poll".into(), json!({"ackRunUuids":[]})); 3
        ]);
    }
}

#[test]
fn reopened_backend_abandons_unreadable_malformed_and_ambiguous_authority_without_ack() {
    for fault in ["unreadable", "invalid-json", "malformed-tail", "duplicate-primary", "ambiguous-alias"] {
        let fixture = PersistedFireFixture::new(RedeliveryCase::RevisionedSchedule);
        let history_path = {
            let owners = FireDeliveryOwners::open(&fixture.root);
            let store = owners.store(&fixture.fire.sand_agent_id);
            let path = store.runs_path(&fixture.automation_id);
            SandAgentSessionStore::new(Arc::clone(&owners.workers))
                .with_admitted_filesystem_mutation(&fixture.fire.sand_agent_id, || {
                    if fault == "unreadable" {
                        fs::remove_file(&path).map_err(|error| error.to_string())?;
                        fs::create_dir(&path).map_err(|error| error.to_string())?;
                        fs::write(path.join("sentinel"), b"owned unreadable history")
                            .map_err(|error| error.to_string())?;
                    } else if fault == "invalid-json" {
                        fs::write(&path, b"[ incomplete").map_err(|error| error.to_string())?;
                    } else {
                        let mut rows: Vec<Value> = serde_json::from_slice(&fixture.baseline.history).unwrap();
                        rows.extend((0..AUTOMATION_MAX_RUN_HISTORY + 1).map(|index| json!({
                            "id":format!("newer-{index}"),"startedAt":1_000 + index,"status":"ok",
                        })));
                        rows.push(match fault {
                            "malformed-tail" => json!({"id":"invalid","startedAt":"unknown"}),
                            "duplicate-primary" => rows[0].clone(),
                            "ambiguous-alias" => json!({"id":"other-owner","startedAt":0,"status":"ok",
                                "coalescedRunIds":[ORIGINAL_RUN_UUID,COALESCED_RUN_UUID]}),
                            _ => unreachable!(),
                        });
                        fs::write(&path, serde_json::to_vec(&rows).unwrap()).map_err(|error| error.to_string())?;
                    }
                    Ok(())
                }).unwrap();
            path
        };
        let before = if fault == "unreadable" { None } else { Some(fs::read(&history_path).unwrap()) };
        for generation in 0..2 {
            let owners = FireDeliveryOwners::open(&fixture.root);
            let transport = MockTransport::with_responses(vec![
                Ok(fixture.delivery(&[ORIGINAL_RUN_UUID, COALESCED_RUN_UUID])); 3
            ]);
            let mut backend = AutomationFireBackendRuntime::new(transport.clone());
            for poll in 0..3 {
                assert_eq!(owners.poll(&mut backend, 1_000 + poll), vec![
                    PreparedBackendFire::Abandon { reason: "run_history_unavailable" }; 2
                ], "fault={fault}, generation={generation}, poll={poll}");
                assert_eq!(backend.pending_len(), 0);
                assert_eq!(owners.executions.load(Ordering::SeqCst), 0);
                assert!(owners.observed.lock().unwrap().is_empty());
                assert_eq!(owners.workers.list_cron_session_runs(&fixture.fire.sand_agent_id).unwrap(), fixture.baseline.cron);
                let store = owners.store(&fixture.fire.sand_agent_id);
                assert_eq!(fs::read(store.config_path(&fixture.automation_id)).unwrap(), fixture.baseline.config);
                if let Some(before) = &before {
                    assert_eq!(&fs::read(&history_path).unwrap(), before);
                } else {
                    assert!(history_path.is_dir());
                    assert_eq!(fs::read(history_path.join("sentinel")).unwrap(), b"owned unreadable history");
                }
            }
            assert_eq!(transport.calls(), vec![
                ("/sand/automation-events/poll".into(), json!({"ackRunUuids":[]})); 3
            ]);
        }
    }
}

#[test]
fn reopened_running_history_does_not_fence_a_distinct_event_or_schedule() {
    for case in [RedeliveryCase::Event, RedeliveryCase::RevisionedSchedule] {
        let fixture = PersistedFireFixture::new(case);
        let owners = FireDeliveryOwners::open(&fixture.root);
        let original_history = owners.store(&fixture.fire.sand_agent_id).read_runs(&fixture.automation_id);
        let transport = MockTransport::with_responses(vec![
            Ok(fixture.delivery(&[ORIGINAL_RUN_UUID, COALESCED_RUN_UUID, DISTINCT_RUN_UUID])),
            Ok(json!({})),
            Ok(json!({"events":[]})),
        ]);
        let mut backend = AutomationFireBackendRuntime::new(transport.clone());
        let prepared = owners.poll(&mut backend, 1_000);
        assert_eq!(prepared.len(), 3);
        assert_eq!(&prepared[..2], vec![
            PreparedBackendFire::Abandon { reason: "existing_run_unsettled" }; 2
        ].as_slice());
        assert!(matches!(&prepared[2],
            PreparedBackendFire::Schedule { run_uuid, .. } | PreparedBackendFire::Event { run_uuid, .. }
                if run_uuid == DISTINCT_RUN_UUID));
        assert_eq!(owners.executions.load(Ordering::SeqCst), 1, "real executor control: {case:?}");
        assert_eq!(backend.pending_len(), 1);
        assert!(owners.poll(&mut backend, 1_001).is_empty());
        assert_eq!(backend.pending_len(), 0);
        let history = owners.store(&fixture.fire.sand_agent_id).read_runs(&fixture.automation_id);
        assert_eq!(history.len(), 2);
        assert_eq!(history.iter().find(|run| run.id == ORIGINAL_RUN_UUID), original_history.first());
        assert_eq!(history.iter().find(|run| run.id == DISTINCT_RUN_UUID).unwrap().status, "ok");
        assert!(!history.iter().any(|run| run.id == COALESCED_RUN_UUID));
        let cron = owners.workers.list_cron_session_runs(&fixture.fire.sand_agent_id).unwrap();
        assert_eq!(cron.len(), 2);
        assert_eq!(cron.iter().find(|row| row.run_id == ORIGINAL_RUN_UUID), fixture.baseline.cron.first());
        assert_eq!(cron.iter().find(|row| row.run_id == DISTINCT_RUN_UUID).unwrap().state, "terminal");
        let reports = owners.observed.lock().unwrap();
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].automation_run_id.as_deref(), Some(DISTINCT_RUN_UUID));
        assert_eq!(transport.calls(), vec![
            ("/sand/automation-events/poll".into(), json!({"ackRunUuids":[]})),
            ("/sand/automation-runs/complete".into(), json!({
                "runUuid":DISTINCT_RUN_UUID,"status":"succeeded","errorMessage":null
            })),
            ("/sand/automation-events/poll".into(), json!({"ackRunUuids":[DISTINCT_RUN_UUID]})),
        ]);
    }
}

#[test]
fn explicitly_settled_history_replays_terminal_success_or_failure_after_reopen() {
    for (status, detail, expected) in [
        ("ok", None, FireCompletion::succeeded()),
        ("error", Some("stored execution failure"), FireCompletion::failed("stored execution failure")),
    ] {
        let fixture = PersistedFireFixture::new(RedeliveryCase::ChangedDefinition);
        let settled = {
            let owners = FireDeliveryOwners::open(&fixture.root);
            let transport = MockTransport::with_responses(vec![
                Ok(fixture.delivery(&[ORIGINAL_RUN_UUID, COALESCED_RUN_UUID])),
                Ok(json!({})), Ok(json!({})),
            ]);
            let mut backend = AutomationFireBackendRuntime::new(transport.clone());
            assert_eq!(owners.poll(&mut backend, 1_000), vec![
                PreparedBackendFire::Abandon { reason: "existing_run_unsettled" }; 2
            ]);
            assert_eq!(backend.pending_len(), 0);
            assert_eq!(owners.executions.load(Ordering::SeqCst), 0);
            assert_eq!(transport.calls(), vec![
                ("/sand/automation-events/poll".into(), json!({"ackRunUuids":[]})),
            ]);
            assert_eq!(owners.snapshot(&fixture.fire.sand_agent_id, &fixture.automation_id), fixture.baseline);
            assert!(owners.workers.finish_cron_session_run(
                &fixture.fire.sand_agent_id, &fixture.automation_id, ORIGINAL_RUN_UUID,
                status, 200.0, detail,
            ).expect("explicit cron terminal settlement"));
            let store = owners.store(&fixture.fire.sand_agent_id);
            SandAgentSessionStore::new(Arc::clone(&owners.workers))
                .with_admitted_filesystem_mutation(&fixture.fire.sand_agent_id, || {
                    assert!(store.finish_run(
                        &fixture.automation_id, ORIGINAL_RUN_UUID, status, 200.0, detail,
                    ).map_err(|error| error.to_string())?.is_some());
                    Ok(())
                }).expect("explicit history terminal settlement");
            assert!(owners.observed.lock().unwrap().is_empty());
            let settled = owners.snapshot(&fixture.fire.sand_agent_id, &fixture.automation_id);
            assert_ne!(settled.history, fixture.baseline.history);
            assert_eq!(settled.config, fixture.baseline.config);
            assert_eq!(settled.cron.len(), 1);
            assert_eq!(settled.cron[0].state, "terminal");
            assert_eq!(settled.cron[0].terminal_outcome.as_deref(), Some(status));
            assert_eq!(settled.cron[0].updated_at_ms, 200.0);
            settled
        };
        let owners = FireDeliveryOwners::open(&fixture.root);
        assert_eq!(owners.snapshot(&fixture.fire.sand_agent_id, &fixture.automation_id), settled);
        let delivery = fixture.delivery(&[ORIGINAL_RUN_UUID, COALESCED_RUN_UUID]);
        let transport = MockTransport::with_responses(vec![
            Ok(delivery.clone()), Ok(json!({})), Ok(json!({})),
            Ok(delivery), Ok(json!({"events":[]})),
        ]);
        let mut backend = AutomationFireBackendRuntime::new(transport.clone());
        assert_eq!(owners.poll(&mut backend, 2_000), vec![
            PreparedBackendFire::Complete { completion: expected.clone(), reason: "existing_run" }; 2
        ]);
        assert_eq!(backend.pending_len(), 2);
        assert!(owners.poll(&mut backend, 2_001).is_empty());
        assert!(owners.poll(&mut backend, 2_002).is_empty());
        assert_eq!(backend.pending_len(), 0);
        assert_eq!(owners.executions.load(Ordering::SeqCst), 0);
        assert!(owners.observed.lock().unwrap().is_empty());
        assert_eq!(owners.snapshot(&fixture.fire.sand_agent_id, &fixture.automation_id), settled);
        assert_eq!(transport.calls(), vec![
            ("/sand/automation-events/poll".into(), json!({"ackRunUuids":[]})),
            ("/sand/automation-runs/complete".into(), json!({
                "runUuid":ORIGINAL_RUN_UUID,"status":expected.status,"errorMessage":expected.error_message
            })),
            ("/sand/automation-runs/complete".into(), json!({
                "runUuid":COALESCED_RUN_UUID,"status":expected.status,"errorMessage":expected.error_message
            })),
            ("/sand/automation-events/poll".into(), json!({"ackRunUuids":[ORIGINAL_RUN_UUID,COALESCED_RUN_UUID]})),
            ("/sand/automation-events/poll".into(), json!({"ackRunUuids":[ORIGINAL_RUN_UUID,COALESCED_RUN_UUID]})),
        ]);
    }
}
