use super::super::{list_openclaw_cron_jobs, trigger_openclaw_cron_job};

use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use mahayana_host_runtime::automations::automation::{AutomationRecord, AutomationRun};
use mahayana_host_runtime::automations::automation_id::stable_automation_id;
use mahayana_host_runtime::extensions::session::agent_session::SandAgentSessionStore;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::transcript::automation_run_path::{
    AutomationExecutionResult, AutomationRunObservation, AutomationRunTrigger,
    FireAutomationOutcome,
};
use mahayana_host_runtime::extensions::transcript::automation_runtime::{
    AutomationRuntime, dispatch_automation_command,
};
use mahayana_host_runtime::gateway_command_error::GatewayCommandError;
use mahayana_host_runtime::host_gateway_api::{
    HostGatewayOwner, OPENCLAW_GATEWAY_METHODS, openclaw_gateway_owner,
};
use serde_json::{Value, json};

struct Owners {
    workers: Arc<ProductionSessionWorkers>,
    runtime: AutomationRuntime,
    observations: Arc<Mutex<Vec<AutomationRunObservation>>>,
}

impl Owners {
    fn open(root: &Path) -> Self {
        let workers = Arc::new(ProductionSessionWorkers::with_agents_root(
            root.join("agents"),
            500,
        ));
        let runtime = AutomationRuntime::new(Arc::clone(&workers));
        let observations = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&observations);
        runtime.run_path().set_run_reporter(Some(Arc::new(move |observation| {
            sink.lock().expect("owned observation sink").push(observation.clone());
        })));
        Self { workers, runtime, observations }
    }

    fn sessions(&self) -> SandAgentSessionStore {
        SandAgentSessionStore::new(Arc::clone(&self.workers))
    }
}

impl Drop for Owners {
    fn drop(&mut self) {
        self.runtime.dispose();
        self.workers.shutdown();
    }
}

struct Fixture {
    root: PathBuf,
    owners: Option<Owners>,
}

#[derive(Clone, Debug)]
struct Routine {
    agent_id: String,
    local_id: String,
    canonical_id: String,
    name: String,
    prompt: String,
    config_path: PathBuf,
    history_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FileImage {
    path: PathBuf,
    bytes: Option<Vec<u8>>,
}

impl FileImage {
    fn read(path: &Path) -> Self {
        let bytes = match fs::read(path) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == ErrorKind::NotFound => None,
            Err(error) => panic!("cannot read owned fixture {}: {error}", path.display()),
        };
        Self { path: path.to_path_buf(), bytes }
    }

    fn assert_unchanged(&self) {
        assert_eq!(Self::read(&self.path), *self, "unexpected routine file mutation");
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RoutineImage {
    config: FileImage,
    history: FileImage,
}

impl RoutineImage {
    fn read(routine: &Routine) -> Self {
        Self {
            config: FileImage::read(&routine.config_path),
            history: FileImage::read(&routine.history_path),
        }
    }

    fn assert_unchanged(&self) {
        self.config.assert_unchanged();
        self.history.assert_unchanged();
    }
}

struct ExecutionTrace {
    running: AutomationRun,
    admitted_image: RoutineImage,
}

struct RunReceipt {
    response: Value,
    outcome: Option<FireAutomationOutcome>,
    history: Vec<AutomationRun>,
    trace: ExecutionTrace,
    observations: Vec<AutomationRunObservation>,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "fabushi-cron-job-identity-{label}-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4(),
        ));
        fs::create_dir(&root).expect("exclusively create owned cron identity fixture");
        let mut fixture = Self { root, owners: None };
        fixture.owners = Some(Owners::open(&fixture.root));
        fixture
    }

    fn owners(&self) -> &Owners {
        self.owners.as_ref().expect("live fixture owners")
    }

    fn reopen(&mut self) {
        drop(self.owners.take());
        self.owners = Some(Owners::open(&self.root));
    }

    fn agent(&self) -> String {
        self.owners().sessions().create_session(None, "user", None)
            .expect("create actual admitted Agent").id
    }

    fn native(&self, method: &str, args: Value) -> Value {
        dispatch_automation_command(&self.owners().runtime, method, &args)
            .expect("existing native automation dispatcher owns the method")
            .expect("native automation command succeeds")
    }

    fn routine(&self, agent_id: &str, name: &str, prompt: &str) -> Routine {
        let created = self.native("createAgentAutomation", json!({
            "id": agent_id,
            "spec": {
                "name": name,
                "prompt": prompt,
                "trigger": {"type": "cron", "schedule": "@daily"},
                "isEnabled": true,
            },
        }));
        let record = created.as_array().expect("native routine list").iter()
            .find(|record| record["name"] == name && record["prompt"] == prompt)
            .expect("the native producer persisted the requested routine");
        let local_id = record["id"].as_str().expect("native local folder ID").to_string();
        let store = self.owners().sessions().automation_store_for(agent_id)
            .expect("actual admitted routine store");
        let routine = Routine {
            agent_id: agent_id.to_string(),
            canonical_id: stable_automation_id(agent_id, &local_id),
            config_path: store.config_path(&local_id),
            history_path: store.runs_path(&local_id),
            local_id,
            name: name.to_string(),
            prompt: prompt.to_string(),
        };
        assert!(RoutineImage::read(&routine).config.bytes.is_some());
        routine
    }

    fn record(&self, routine: &Routine) -> AutomationRecord {
        self.owners().runtime.get_agent_automations(&routine.agent_id)
            .expect("read real Agent routines").into_iter()
            .find(|record| record.id == routine.local_id)
            .expect("exact local routine remains present")
    }

    fn runs(&self, routine: &Routine) -> Vec<AutomationRun> {
        self.owners().sessions().automation_store_for(&routine.agent_id)
            .expect("actual admitted history store").read_runs(&routine.local_id)
    }

    fn list(&self, args: Value) -> Value {
        list_openclaw_cron_jobs(&self.owners().sessions(), &args)
            .expect("the production list function reads the actual owners")
    }

    fn seed_history(&self, routine: &Routine) -> AutomationRun {
        let outcome = self.owners().runtime.run_agent_automation_now_with(
            &routine.agent_id,
            &routine.local_id,
            |prompt| {
                assert!(prompt.contains(&routine.prompt));
                Ok(AutomationExecutionResult::Completed)
            },
        ).expect("real native run establishes prior durable history");
        assert_eq!(outcome, Some(FireAutomationOutcome::Ok));
        let run = self.runs(routine).into_iter().next().expect("prior history row");
        assert_eq!(run.status, "ok");
        assert!(run.finished_at.is_some());
        assert!(RoutineImage::read(routine).history.bytes.is_some());
        run
    }

    fn trigger_real(
        &self,
        args: Value,
        selected: &Routine,
        leaf_result: Result<AutomationExecutionResult, String>,
    ) -> RunReceipt {
        let observation_start = self.owners().observations.lock()
            .expect("owned observations").len();
        let mut dispatches = 0;
        let mut trace = None;
        let mut outcome = None;
        let response = trigger_openclaw_cron_job(
            &self.owners().sessions(),
            &args,
            |run_args| {
                dispatches += 1;
                assert_dispatch_pair(&run_args, selected);
                // Execute the pair supplied by the real adapter, rather than
                // substituting an expected target in this test port.
                let agent_id = run_args["agentId"].as_str().expect("dispatched Agent");
                let local_id = run_args["automationId"].as_str().expect("dispatched local ID");
                let runtime_outcome = self.owners().runtime.run_agent_automation_now_with(
                    agent_id,
                    local_id,
                    |prompt| {
                        assert!(prompt.contains(&selected.prompt), "the selected prompt must reach execution");
                        let running = self.runs(selected).into_iter().next()
                            .expect("real Runtime admits history before its executor");
                        assert_eq!(running.status, "running");
                        assert_eq!(running.trigger, "manual");
                        assert!(running.finished_at.is_none());
                        trace = Some(ExecutionTrace {
                            running,
                            admitted_image: RoutineImage::read(selected),
                        });
                        leaf_result
                    },
                ).map_err(GatewayCommandError::Internal)?;
                outcome = Some(runtime_outcome);
                Ok(Value::Null)
            },
        ).expect("production trigger invokes the existing Runtime");
        assert_eq!(dispatches, 1);
        let trace = trace.expect("one actual Runtime executor entry");
        assert_eq!(response["run"]["jobId"].as_str(), Some(selected.canonical_id.as_str()));
        assert_eq!(response["runId"].as_str(), Some(trace.running.id.as_str()));
        assert_eq!(response["run"]["runId"], response["runId"]);
        assert_eq!(response["ok"], true);
        assert_eq!(response["enqueued"], true);
        assert!(response["run"]["ts"].is_number());
        assert!(!self.owners().runtime.run_path().is_in_flight(&selected.agent_id, &selected.local_id));
        let history = self.runs(selected);
        assert_eq!(history.first().expect("current history").id, trace.running.id);
        let observations = self.owners().observations.lock()
            .expect("owned observations")[observation_start..].to_vec();
        RunReceipt {
            response,
            outcome: outcome.expect("the actual Runtime returned an outcome"),
            history,
            trace,
            observations,
        }
    }

    fn refuse_without_effects(&self, args: Value, routines: &[&Routine]) {
        let images = routines.iter().map(|routine| RoutineImage::read(routine)).collect::<Vec<_>>();
        let observations_before = self.owners().observations.lock().expect("owned observations").len();
        let mut dispatches = 0;
        let result = trigger_openclaw_cron_job(&self.owners().sessions(), &args, |_| {
            dispatches += 1;
            Ok(Value::Null)
        });
        let error = result.expect_err("identity refusal must not return an accepted response");
        assert!(matches!(&error, GatewayCommandError::BadRequest(_)), "unexpected refusal: {error}");
        assert_eq!(dispatches, 0, "identity resolution must precede any execution port");
        assert_eq!(self.owners().observations.lock().expect("owned observations").len(), observations_before);
        for image in images {
            image.assert_unchanged();
        }
    }

    fn delete(&self, routine: &Routine) {
        let remaining = self.native("deleteAgentAutomation", json!({
            "id": routine.agent_id,
            "automationId": routine.local_id,
        }));
        assert!(remaining.as_array().expect("native routine list").iter()
            .all(|record| record["id"] != routine.local_id));
        assert!(!routine.config_path.exists());
        assert!(!routine.history_path.exists());
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        // Owners stop all their own workers before this exclusively claimed
        // fixture tree is removed. No sibling root or process is touched.
        drop(self.owners.take());
        let cleanup = fs::remove_dir_all(&self.root);
        if !std::thread::panicking() {
            cleanup.expect("remove only the owned cron identity fixture");
        }
    }
}

fn assert_dispatch_pair(args: &Value, selected: &Routine) {
    assert_eq!(args["agentId"].as_str(), Some(selected.agent_id.as_str()));
    assert_eq!(args["automationId"].as_str(), Some(selected.local_id.as_str()));
    assert_ne!(args["automationId"].as_str(), Some(selected.canonical_id.as_str()));
}

fn listed_job<'a>(page: &'a Value, routine: &Routine) -> &'a Value {
    let matches = page["jobs"].as_array().expect("public job list").iter()
        .filter(|job| job["agentId"] == routine.agent_id && job["id"] == routine.canonical_id)
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1, "each durable pair has exactly one public identity");
    matches[0]
}

fn identities(page: &Value) -> BTreeMap<String, String> {
    page["jobs"].as_array().expect("public job list").iter().map(|job| {
        (
            job["agentId"].as_str().expect("Agent ID").to_string(),
            job["id"].as_str().expect("canonical ID").to_string(),
        )
    }).collect()
}

fn assert_projection(fixture: &Fixture, page: &Value, routine: &Routine) {
    let job = listed_job(page, routine);
    let record = fixture.record(routine);
    assert_eq!(job["id"], routine.canonical_id);
    assert_eq!(job["jobId"], routine.canonical_id);
    assert_eq!(job["name"], routine.name);
    assert_eq!(job["description"], routine.prompt);
    assert_eq!(job["payload"], json!({"kind": "agentTurn", "message": routine.prompt}));
    assert_eq!(job["enabled"], record.is_enabled);
    assert_eq!(job["schedule"], json!({"kind": "cron", "expr": "@daily", "tz": "UTC"}));
    assert_eq!(job["createdAtMs"], json!(record.created_at as i64));
    assert_eq!(job["updatedAtMs"], json!(record.last_run_at.unwrap_or(record.created_at) as i64));
    assert_eq!(job["state"]["nextRunAtMs"], json!(record.next_run_at.map(|at| at as i64)));
    assert_eq!(job["state"]["lastRunAtMs"], json!(record.last_run_at.map(|at| at as i64)));
    assert_eq!(job["state"]["lastRunStatus"], json!(record.runs.first().map(|run| &run.status)));
    assert_eq!(job["state"]["lastRunError"], json!(record.runs.first().and_then(|run| run.detail.as_ref())));
}

fn assert_terminal(receipt: &RunReceipt, routine: &Routine, outcome: FireAutomationOutcome, detail: Option<&str>) {
    let expected_status = if outcome == FireAutomationOutcome::Ok { "ok" } else { "error" };
    let run = receipt.history.first().expect("terminal history");
    assert_eq!(receipt.outcome, Some(outcome));
    assert_eq!(run.status, expected_status);
    assert!(run.finished_at.is_some());
    assert_eq!(run.detail.as_deref(), detail);
    assert_eq!(receipt.response["finished"], true);
    assert_eq!(receipt.response["run"]["action"], "finished");
    assert_eq!(receipt.response["run"]["status"], expected_status);
    assert_eq!(receipt.response["run"]["detail"], json!(detail));
    assert_eq!(receipt.observations.len(), 1);
    let observation = &receipt.observations[0];
    assert_eq!(observation.agent_id, routine.agent_id);
    assert_eq!(observation.automation_id, routine.local_id);
    assert_eq!(observation.automation_run_id.as_deref(), Some(run.id.as_str()));
    assert_eq!(observation.trigger, AutomationRunTrigger::Manual);
    assert_eq!(observation.outcome, outcome);
}

fn assert_shipping_cron_wiring() {
    for method in ["cron.list", "cron.trigger"] {
        assert!(OPENCLAW_GATEWAY_METHODS.contains(&method));
        assert_eq!(openclaw_gateway_owner(method), Some(HostGatewayOwner::Automations));
    }
    // This is only a supplementary source anchor. The tests below execute the
    // actual production functions and owners; they do not run the full HTTP/App.
    let source = include_str!("../main.rs");
    let production = source.split_once("\n#[cfg(test)]\nmod tests {")
        .expect("shipping/test boundary").0;
    let dispatch = production.split_once("    fn call_openclaw_gateway_command(")
        .expect("registered App dispatcher").1
        .split_once("    fn call_public_settings_command(").expect("dispatcher boundary").0;
    assert!(dispatch.contains("\"cron.list\" => self.call_openclaw_cron_list(&args)"));
    assert!(dispatch.contains("\"cron.trigger\" => self.call_openclaw_cron_trigger(&args)"));
    let list = production.split_once("    fn call_openclaw_cron_list(")
        .expect("App list method").1
        .split_once("    fn call_openclaw_cron_trigger(").expect("list method boundary").0;
    assert!(list.contains("list_openclaw_cron_jobs(&store, args)"));
    assert!(list.contains("SandAgentSessionStore::new(Arc::clone(&self.session_workers))"));
    let trigger = production.split_once("    fn call_openclaw_cron_trigger(")
        .expect("App trigger method").1
        .split_once("    fn call_openclaw_node_register(").expect("trigger method boundary").0;
    assert!(trigger.contains("trigger_openclaw_cron_job(&store, args,"));
    assert!(trigger.contains("self.call(\"runAgentAutomationNow\","));
    assert!(trigger.contains("SandAgentSessionStore::new(Arc::clone(&self.session_workers))"));
}

#[test]
fn cron_job_identity_list_and_trigger_keep_same_name_agents_distinct() {
    assert_shipping_cron_wiring();
    let fixture = Fixture::new("same-name");
    let first = fixture.routine(&fixture.agent(), "Daily check", "Inspect only the first Agent's inbox.");
    let second = fixture.routine(&fixture.agent(), "Daily check", "Inspect only the second Agent's calendar.");
    assert_ne!(first.agent_id, second.agent_id);
    assert_eq!(first.local_id, "daily-check");
    assert_eq!(first.local_id, second.local_id);
    assert_ne!(first.canonical_id, second.canonical_id);
    let first_prior = fixture.seed_history(&first);
    let second_prior = fixture.seed_history(&second);
    let page = fixture.list(json!({}));
    assert_eq!(page["total"], 2);
    assert_eq!(page["hasMore"], false);
    assert!(page["nextOffset"].is_null());
    assert_projection(&fixture, &page, &first);
    assert_projection(&fixture, &page, &second);

    let jobs = page["jobs"].as_array().expect("two public jobs");
    let first_page = fixture.list(json!({"limit": 1, "offset": 0}));
    let second_page = fixture.list(json!({"limit": 1, "offset": 1}));
    assert_eq!(first_page["total"], 2);
    assert_eq!(first_page["jobs"], json!([jobs[0].clone()]));
    assert_eq!(first_page["hasMore"], true);
    assert_eq!(first_page["nextOffset"], 1);
    assert_eq!(second_page["jobs"], json!([jobs[1].clone()]));
    assert_eq!(second_page["hasMore"], false);
    assert!(second_page["nextOffset"].is_null());

    for (index, job) in jobs.iter().rev().enumerate() {
        let (selected, other, prior) = if job["agentId"] == first.agent_id {
            (&first, &second, &first_prior)
        } else {
            assert_eq!(job["agentId"], second.agent_id);
            (&second, &first, &second_prior)
        };
        let untouched = RoutineImage::read(other);
        let args = if index == 0 {
            json!({"id": job["id"], "agentId": other.agent_id})
        } else {
            json!({"jobId": job["jobId"], "agentId": other.agent_id})
        };
        let receipt = fixture.trigger_real(args, selected, Ok(AutomationExecutionResult::Completed));
        assert_terminal(&receipt, selected, FireAutomationOutcome::Ok, None);
        assert_eq!(receipt.history.len(), 2);
        assert_ne!(receipt.history[0].id, prior.id);
        assert!(receipt.history.iter().any(|run| run == prior), "previous history must survive unchanged");
        untouched.assert_unchanged();
        assert_projection(&fixture, &fixture.list(json!({})), selected);
        let native = fixture.native("getAgentAutomations", json!({"id": selected.agent_id}));
        assert_eq!(native[0]["id"], selected.local_id, "native IDs stay Agent-local");
    }
}

#[test]
fn cron_job_identity_legacy_aliases_refuse_ambiguity_without_effects() {
    let fixture = Fixture::new("legacy");
    let first = fixture.routine(&fixture.agent(), "Legacy brief", "Read only the legacy first Agent.");
    fixture.seed_history(&first);
    let legacy = fixture.trigger_real(
        json!({"jobId": first.local_id}),
        &first,
        Ok(AutomationExecutionResult::Completed),
    );
    assert_terminal(&legacy, &first, FireAutomationOutcome::Ok, None);

    let second = fixture.routine(&fixture.agent(), "Legacy brief", "Read only the legacy second Agent.");
    fixture.seed_history(&second);
    assert_eq!(first.local_id, second.local_id);
    for args in [
        json!({"id": first.local_id}),
        json!({"jobId": first.local_id, "agentId": first.agent_id}),
        json!({"id": first.local_id, "agentId": second.agent_id}),
        json!({}),
        json!({"id": ""}),
        json!({"jobId": " \n\t"}),
        json!({"id": 7}),
        json!({"jobId": "unknown-routine", "agentId": first.agent_id}),
        json!({"id": "", "jobId": first.canonical_id}),
        json!({"id": "unknown-routine", "jobId": first.canonical_id}),
    ] {
        fixture.refuse_without_effects(args, &[&first, &second]);
    }

    let untouched = RoutineImage::read(&second);
    let canonical = fixture.trigger_real(
        json!({"id": first.canonical_id, "jobId": second.canonical_id, "agentId": second.agent_id}),
        &first,
        Ok(AutomationExecutionResult::Completed),
    );
    assert_terminal(&canonical, &first, FireAutomationOutcome::Ok, None);
    untouched.assert_unchanged();
}

#[test]
fn cron_job_identity_canonical_legacy_collision_refuses_without_effects() {
    let fixture = Fixture::new("namespace-collision");
    let original = fixture.routine(&fixture.agent(), "Original job", "Run only the original canonical owner.");
    fixture.seed_history(&original);
    assert_eq!(uuid::Uuid::parse_str(&original.canonical_id).unwrap().get_version_num(), 5);
    let collider = fixture.routine(
        &fixture.agent(),
        &original.canonical_id,
        "Run only the deliberately colliding local-name owner.",
    );
    fixture.seed_history(&collider);
    assert_eq!(collider.local_id, original.canonical_id, "native creation must create the collision");
    assert_ne!(collider.canonical_id, original.canonical_id);
    let page = fixture.list(json!({}));
    assert_projection(&fixture, &page, &original);
    assert_projection(&fixture, &page, &collider);
    for args in [
        json!({"id": original.canonical_id}),
        json!({"jobId": collider.local_id}),
        json!({"id": original.canonical_id, "agentId": original.agent_id}),
        json!({"jobId": original.canonical_id, "agentId": collider.agent_id}),
    ] {
        fixture.refuse_without_effects(args, &[&original, &collider]);
    }

    let untouched = RoutineImage::read(&collider);
    fixture.delete(&original);
    untouched.assert_unchanged();
    // CJI-ID-NAMESPACE-01: losing the canonical owner must not reinterpret
    // its former UUID as the other routine's now-unique legacy alias.
    for args in [
        json!({"id": original.canonical_id}),
        json!({"jobId": original.canonical_id, "agentId": collider.agent_id}),
    ] {
        fixture.refuse_without_effects(args, &[&original, &collider]);
    }
    let page = fixture.list(json!({}));
    assert_eq!(page["total"], 1);
    let listed_id = listed_job(&page, &collider)["id"].clone();
    let deleted = RoutineImage::read(&original);
    let receipt = fixture.trigger_real(
        json!({"jobId": listed_id}),
        &collider,
        Ok(AutomationExecutionResult::Completed),
    );
    assert_terminal(&receipt, &collider, FireAutomationOutcome::Ok, None);
    deleted.assert_unchanged();
}

#[test]
fn cron_job_identity_reopen_and_delete_preserve_target_continuity() {
    let mut fixture = Fixture::new("reopen");
    let first = fixture.routine(&fixture.agent(), "Durable check", "Inspect the first durable Agent.");
    let second = fixture.routine(&fixture.agent(), "Durable check", "Inspect the second durable Agent.");
    fixture.seed_history(&first);
    fixture.seed_history(&second);
    let before = fixture.list(json!({}));
    let first_image = RoutineImage::read(&first);
    let second_image = RoutineImage::read(&second);
    fixture.reopen();
    let reopened = fixture.list(json!({}));
    assert_eq!(identities(&reopened), identities(&before));
    assert_projection(&fixture, &reopened, &first);
    assert_projection(&fixture, &reopened, &second);
    first_image.assert_unchanged();
    second_image.assert_unchanged();

    let receipt = fixture.trigger_real(
        json!({"id": second.canonical_id, "agentId": first.agent_id}),
        &second,
        Ok(AutomationExecutionResult::Completed),
    );
    assert_terminal(&receipt, &second, FireAutomationOutcome::Ok, None);
    first_image.assert_unchanged();
    fixture.delete(&second);
    first_image.assert_unchanged();
    fixture.refuse_without_effects(
        json!({"jobId": second.canonical_id, "agentId": first.agent_id}),
        &[&first, &second],
    );
    let deleted = RoutineImage::read(&second);
    let survivor = fixture.trigger_real(
        json!({"jobId": first.local_id}),
        &first,
        Ok(AutomationExecutionResult::Completed),
    );
    assert_terminal(&survivor, &first, FireAutomationOutcome::Ok, None);
    deleted.assert_unchanged();
    let page = fixture.list(json!({}));
    assert_eq!(page["total"], 1);
    assert_projection(&fixture, &page, &first);
}

#[test]
fn cron_job_identity_existing_execution_outcomes_remain_owned() {
    let fixture = Fixture::new("outcomes");
    let sentinel = fixture.routine(&fixture.agent(), "Untouched sentinel", "Never execute this unrelated sentinel.");
    fixture.seed_history(&sentinel);
    let sentinel_image = RoutineImage::read(&sentinel);
    let cases = [
        ("Completed", Ok(AutomationExecutionResult::Completed), Some(FireAutomationOutcome::Ok), None),
        ("Executor error", Err("controlled executor error".to_string()), Some(FireAutomationOutcome::Error), Some("controlled executor error")),
        ("Reported failure", Ok(AutomationExecutionResult::Failed { detail: "controlled terminal failure".into() }), Some(FireAutomationOutcome::Error), Some("controlled terminal failure")),
        ("Interrupted", Ok(AutomationExecutionResult::Interrupted {
            detail: "controlled cancellation".into(), quiesced_for_upgrade: false,
        }), Some(FireAutomationOutcome::Interrupted), Some("controlled cancellation")),
        ("Retained uncertainty", Ok(AutomationExecutionResult::RetainedUnsettled {
            terminal: Box::new(AutomationExecutionResult::Interrupted {
                detail: "external outcome remains unknown".into(), quiesced_for_upgrade: false,
            }),
        }), None, None),
    ];
    let mut settled_images = Vec::new();
    for (name, leaf_result, expected, detail) in cases {
        let routine = fixture.routine(&fixture.agent(), name, &format!("Execute only the {name} case."));
        let receipt = fixture.trigger_real(json!({"jobId": routine.canonical_id}), &routine, leaf_result);
        assert_eq!(receipt.history.len(), 1);
        match expected {
            Some(outcome) => assert_terminal(&receipt, &routine, outcome, detail),
            None => {
                assert_eq!(receipt.outcome, None);
                assert_eq!(receipt.history[0], receipt.trace.running);
                assert_eq!(receipt.history[0].status, "running");
                assert!(receipt.history[0].finished_at.is_none());
                assert!(receipt.history[0].detail.is_none());
                assert_eq!(receipt.response["finished"], false);
                assert_eq!(receipt.response["run"]["action"], "accepted");
                assert_eq!(receipt.response["run"]["status"], "running");
                assert!(receipt.response["run"]["detail"].is_null());
                assert!(receipt.observations.is_empty(), "uncertain effects cannot publish a terminal observation");
                receipt.trace.admitted_image.assert_unchanged();
            }
        }
        sentinel_image.assert_unchanged();
        for image in &settled_images {
            RoutineImage::assert_unchanged(image);
        }
        settled_images.push(RoutineImage::read(&routine));
        assert_projection(&fixture, &fixture.list(json!({})), &routine);
    }

    let refused = fixture.routine(&fixture.agent(), "Port refused", "Do not manufacture a run after port refusal.");
    fixture.seed_history(&refused);
    let before = RoutineImage::read(&refused);
    let observation_count = fixture.owners().observations.lock().expect("owned observations").len();
    let mut dispatches = 0;
    let result = trigger_openclaw_cron_job(
        &fixture.owners().sessions(),
        &json!({"id": refused.canonical_id}),
        |args| {
            dispatches += 1;
            assert_dispatch_pair(&args, &refused);
            Err(GatewayCommandError::Internal("controlled native run port refusal".into()))
        },
    );
    let error = result.expect_err("a refused dispatcher cannot become an accepted response");
    assert!(matches!(&error, GatewayCommandError::Internal(message) if message == "controlled native run port refusal"));
    assert_eq!(dispatches, 1, "the adapter must not retry a failed native dispatch");
    assert_eq!(fixture.owners().observations.lock().expect("owned observations").len(), observation_count);
    before.assert_unchanged();
    sentinel_image.assert_unchanged();
    for image in settled_images {
        image.assert_unchanged();
    }
}
