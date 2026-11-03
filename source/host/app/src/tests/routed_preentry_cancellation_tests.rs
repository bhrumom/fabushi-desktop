//! PRC-v1 exercises the shipping cancellation helpers, actual start refusal,
//! model-admission boundary and late registry attachment with real owners.
//! The bounded child isolates credentials/environment. It does not execute an
//! external provider, tool or restart and is not compatibility acceptance.

use super::super as shipping;
use super::channel_host_lane_tests::{IdleRunnerOwners, idle_runner_owners};
use mahayana_host_runtime::extensions::session::gateway::persist_accepted_send_prompt_context;
use mahayana_host_runtime::extensions::transcript::production_runtime::{
    ProductionSendError, ProductionTranscriptRuntime, RoutedTurnStartState,
};
use mahayana_host_runtime::extensions::transcript::send_pipeline::PersistedSendContext;
use mahayana_host_runtime::storage::store_db::get_sand_agent_db_write_generation;
use serde_json::{Value, json};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

const CHILD_CASE: &str = "FABUSHI_PRC_CHILD_CASE";
const CHILD_ROOT: &str = "FABUSHI_PRC_CHILD_ROOT";
const CHILD_NONCE: &str = "FABUSHI_PRC_CHILD_NONCE";
const WAIT: Duration = Duration::from_secs(5);

struct OwnedChild { child: Child, root: PathBuf, completed: bool }

impl Drop for OwnedChild {
    fn drop(&mut self) {
        if !matches!(self.child.try_wait(), Ok(Some(_))) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        if self.completed {
            let _ = fs::remove_dir_all(&self.root);
        } else {
            eprintln!("[prc] retained child={} root={}", self.child.id(), self.root.display());
        }
    }
}

fn bounded(case: &str, run: impl FnOnce(&Path)) {
    if std::env::var(CHILD_CASE).ok().as_deref() == Some(case) {
        let root = PathBuf::from(std::env::var_os(CHILD_ROOT).expect("owned PRC root"));
        let nonce = std::env::var(CHILD_NONCE).expect("owned PRC nonce");
        let allocation: Value = serde_json::from_slice(&fs::read(root.join("allocation.json")).unwrap()).unwrap();
        assert_eq!(allocation["case"], case);
        assert_eq!(allocation["nonce"], nonce);
        let mut options = fs::OpenOptions::new();
        options.create_new(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        options.open(root.join("credential.json")).unwrap().write_all(json!({
            "version":1, "issuer":"http://127.0.0.1:9",
            "accessToken":"synthetic-prc-child-private-credential",
            "expiresAtMs":4_102_444_800_000_u64,
            "userId":"prc-user", "deviceId":"prc-device", "sessionId":"prc-account-session",
        }).to_string().as_bytes()).unwrap();
        run(&root);
        fs::write(root.join("completed.json"), json!({
            "case":case, "nonce":nonce, "pid":std::process::id(), "phase":"owners-retired",
        }).to_string()).expect("actual PRC assertion completion");
        return;
    }
    let base = std::env::var_os("FABUSHI_AUTOMATION_CONTRACT_ARTIFACT_ROOT")
        .map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    let root = base.join(format!("fabushi-prc-{}-{}", std::process::id(), uuid::Uuid::new_v4()));
    fs::create_dir(&root).expect("exclusive PRC child root");
    let nonce = uuid::Uuid::new_v4().to_string();
    fs::write(root.join("allocation.json"), json!({
        "case":case, "nonce":nonce, "parentPid":std::process::id(),
    }).to_string()).unwrap();
    let child = Command::new(std::env::current_exe().expect("App test executable"))
        .args(["--exact", case, "--nocapture", "--test-threads=1"])
        .env(CHILD_CASE, case).env(CHILD_ROOT, &root).env(CHILD_NONCE, &nonce)
        .env("FABUSHI_RESPONSES_URL", "http://127.0.0.1:9/v1/responses")
        .env("FABUSHI_HOST_ACCESS_CREDENTIAL_FILE", root.join("credential.json"))
        .env_remove("SAND_AGENT_MOCK_RESPONSE").env_remove("SAND_PACKAGED_RUNTIME")
        .spawn().expect("start exact PRC child");
    let mut owned = OwnedChild { child, root, completed:false };
    let deadline = Instant::now() + Duration::from_secs(45);
    let status = loop {
        if let Some(status) = owned.child.try_wait().expect("observe owned PRC child") { break status; }
        assert!(Instant::now() < deadline, "PRC child timed out: {case}; root={}", owned.root.display());
        std::thread::sleep(Duration::from_millis(10));
    };
    assert!(status.success(), "PRC child failed: {case}: {status}; root={}", owned.root.display());
    let receipt: Value = serde_json::from_slice(&fs::read(owned.root.join("completed.json"))
        .expect("exact test completion receipt")).unwrap();
    assert_eq!(receipt, json!({
        "case":case, "nonce":nonce, "pid":owned.child.id(), "phase":"owners-retired",
    }));
    owned.completed = true;
}

#[derive(Clone, Copy)]
enum CancelRoute { Stream, Agent, Chat }

struct Fixture {
    owners: IdleRunnerOwners,
    agent: String,
    session_key: String,
    lifecycle: Arc<Mutex<Vec<(String, String)>>>,
    host_rx: mpsc::Receiver<shipping::HostLaneRequest>,
}

impl Fixture {
    fn open(root: &Path) -> Self {
        fs::create_dir(root).unwrap();
        fs::write(root.join("settings.json"), br#"{"version":1,"inferenceProvider":"fabushi"}"#).unwrap();
        let sessions = Arc::new(shipping::ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
        let agent = sessions.materialize_new_session(None, "user", None).unwrap().id;
        let session_key = sessions.read_session_work_admission(&agent).unwrap().unwrap().session_key;
        let manager = Arc::new(shipping::TranscriptManager::new(root, Arc::clone(&sessions)));
        let (host_tx, host_rx) = mpsc::channel();
        let owners = idle_runner_owners(root, host_tx, shipping::GatewayEventHub::default(), sessions, manager);
        assert!(owners.deps.settings.set_agent_default_model(Some(&json!({
            "provider":"fabushi", "modelId":"Vendor/PRC@Original", "maxMode":false, "parameters":[],
        }))).unwrap());
        let lifecycle = Arc::new(Mutex::new(Vec::new()));
        let observed = Arc::clone(&lifecycle);
        owners.deps.transcript_runtime.set_agent_run_lifecycle_observer(Some(Arc::new(move |kind, stream| {
            observed.lock().unwrap().push((kind.to_string(), stream.to_string()));
        })));
        Self { owners, agent, session_key, lifecycle, host_rx }
    }

    fn args(&self, stream: &str, nonce: &str) -> Value {
        json!({
            "agentId":self.agent, "sessionKey":self.session_key,
            "provider":"fabushi", "streamId":stream, "clientNonce":nonce,
            "prompt":format!("original accepted input {nonce}"),
            "messages":[{"role":"user","content":format!("original accepted input {nonce}")}],
        })
    }

    fn accept(&self, stream: &str, nonce: &str) -> Value {
        let args = self.args(stream, nonce);
        let sessions = &self.owners.deps.session_workers;
        let accepted = self.owners.deps.transcript_runtime.accept_routed_send(&args, |ack| {
            persist_accepted_send_prompt_context(sessions, &args, ack).map_err(shipping::map_session_send_error)
        }).expect("actual durable Session acceptance");
        assert!(accepted.context.accepted_durably);
        assert_eq!(accepted.context.user_message_id, accepted.context.echo_entry_id);
        assert!(accepted.context.user_message_id.is_some());
        self.owners.deps.ack_obligations.record_send(&self.agent, 1.0).unwrap();
        assert!(self.owners.deps.transcript_runtime.has_routed_turn_lease(&self.agent, stream));
        args
    }

    fn cancel(&self, route: CancelRoute, stream: &str) -> Value {
        let deps = &self.owners.deps;
        match route {
            CancelRoute::Stream => shipping::request_routed_provider_cancellation(
                &json!({"streamId":stream,"reason":"explicit PRC user stop"}),
                &deps.runner_registry, &deps.transcript_runtime,
                &deps.transcript_manager.background_wakes(), &deps.telemetry_logs,
            ),
            CancelRoute::Agent => shipping::request_agent_cancellation(
                &json!({"id":self.agent}), &deps.runner_registry, &deps.transcript_runtime,
                &deps.ack_obligations, &deps.telemetry_logs,
            ),
            CancelRoute::Chat => shipping::request_openclaw_chat_abort(
                &json!({"sessionKey":self.session_key,"runId":stream}),
                &deps.runner_registry, &deps.transcript_runtime,
            ),
        }.expect("actual shipping cancellation helper")
    }

    fn changed(reply: &Value, route: CancelRoute) -> bool {
        reply[match route { CancelRoute::Stream => "cancelled", CancelRoute::Agent => "interrupted", CancelRoute::Chat => "aborted" }]
            .as_bool().expect("typed cancellation disposition")
    }

    fn ended(&self, stream: &str) -> usize {
        self.lifecycle.lock().unwrap().iter().filter(|(kind, id)| kind == "ended" && id == stream).count()
    }

    fn launch(&self, args: Value) -> Result<shipping::RoutedProviderStartOutcome, shipping::GatewayCommandError> {
        let d = &self.owners.deps;
        shipping::start_routed_provider_task(
            Arc::clone(&d.routed_tool_relay), Arc::clone(&d.mcp_service), d.events.clone(),
            d.host_tx.clone(), d.data_dir.clone(), Arc::clone(&d.request_context),
            Arc::clone(&d.auth), Arc::clone(&d.auto_review), Arc::clone(&d.experiments),
            Arc::clone(&d.settings), Arc::clone(&d.inference), Arc::clone(&d.session_workers),
            Arc::clone(&d.runner_registry), Arc::clone(&d.ack_obligations),
            Arc::clone(&d.transcript_runtime), Arc::clone(&d.transcript_manager),
            Arc::clone(&d.generated_agent_runtime), Arc::clone(&d.completion_revivals),
            Arc::clone(&d.forever_box), Arc::clone(&d.local_exec), Arc::clone(&d.local_tool_permission),
            d.session_handoff.clone(), Arc::clone(&d.trays), d.telemetry_logs.clone(),
            d.product_analytics.clone(), d.production_action_auditor.clone(),
            Arc::clone(&d.cloud_agents), Arc::clone(&d.cloud_agent_watches),
            Arc::clone(&d.background_shell_watches), Arc::clone(&d.host_runner_composition),
            d.box_store_sync.clone(), Arc::clone(&d.automations_lifecycle),
            Arc::clone(&d.generated_subagent_cleanup), None, args,
        )
    }

    fn prepare(&self, stream: &str, observe: impl FnMut(shipping::RoutedTurnAdmissionPoint))
        -> Result<shipping::RoutedTurnModelAdmission, shipping::RoutedTurnAdmissionError>
    {
        let d = &self.owners.deps;
        shipping::prepare_routed_turn_model_admission_observed(
            &d.transcript_runtime, &d.events, &d.session_workers, &d.inference,
            d.local_tool_permission.controller().as_ref(), shipping::RoutedProvider::Fabushi,
            &self.agent, stream, None, true, observe,
        )
    }

    fn assert_no_prepare_write(&self, before: &[Value], generation: u64) {
        let sessions = &self.owners.deps.session_workers;
        let db = sessions.open_agent_db_owner(&self.agent).unwrap();
        assert_eq!(sessions.read_agent_transcript_entries(&self.agent).unwrap(), before,
            "the original user input remains; no generated Transcript append");
        assert_eq!(get_sand_agent_db_write_generation(db.db_path()), generation,
            "no Session/profile/Memory preparation write");
        assert!(db.get_agent_profile_prompt_snapshot().unwrap().is_none());
        assert!(db.get_memory_prompt_snapshot().unwrap().is_none());
        assert!(self.host_rx.try_recv().is_err(), "no Host-lane tool or preparation request");
        assert_eq!(self.owners.deps.runner_registry.active_count(), 0,
            "no worker was registered");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.owners.dispose(&self.agent);
        self.owners.deps.session_workers.shutdown();
    }
}

#[test]
fn routed_preentry_public_cancellation_stops_actual_start_before_model_and_preserves_input() {
    bounded("tests::routed_preentry_cancellation_tests::routed_preentry_public_cancellation_stops_actual_start_before_model_and_preserves_input", |root| {
        for (index, route) in [CancelRoute::Stream, CancelRoute::Agent, CancelRoute::Chat].into_iter().enumerate() {
            let f = Fixture::open(&root.join(format!("route-{index}")));
            let d = &f.owners.deps;
            let stream = format!("pre-model-{index}");
            let args = f.accept(&stream, &stream);
            let foreign_agent = d.session_workers.materialize_new_session(None, "user", None).unwrap().id;
            let foreign_key = d.session_workers.read_session_work_admission(&foreign_agent).unwrap().unwrap().session_key;
            let foreign_args = json!({
                "agentId":foreign_agent, "sessionKey":foreign_key, "streamId":"foreign-stream",
                "clientNonce":"foreign-stream", "prompt":"unrelated accepted input",
            });
            d.transcript_runtime.accept_routed_send(&foreign_args, |ack| {
                persist_accepted_send_prompt_context(&d.session_workers, &foreign_args, ack)
                    .map_err(shipping::map_session_send_error)
            }).unwrap();
            let foreign_guard = shipping::RoutedTurnLeaseGuard::new(
                Arc::clone(&d.transcript_runtime), foreign_agent.clone(), "foreign-stream".into(), d.events.clone(),
            ).unwrap();
            let db = d.session_workers.open_agent_db_owner(&f.agent).unwrap();
            assert!(db.write_kv("sessionModelSelection", "{").unwrap(),
                "a reached model read would be a real refusal, not cancellation");
            let rows = d.session_workers.read_agent_transcript_entries(&f.agent).unwrap();
            let generation = get_sand_agent_db_write_generation(db.db_path());
            let direction = d.local_tool_permission.controller().direction_epoch(&f.agent);
            if matches!(route, CancelRoute::Agent) {
                let missing = shipping::request_agent_cancellation(
                    &json!({"id":"missing-agent"}), &d.runner_registry, &d.transcript_runtime,
                    &d.ack_obligations, &d.telemetry_logs,
                ).unwrap();
                assert!(!Fixture::changed(&missing, route));
            } else {
                assert!(!Fixture::changed(&f.cancel(route, "missing-stream"), route));
            }
            let foreign_reply = shipping::request_openclaw_chat_abort(
                &json!({"sessionKey":foreign_key,"runId":stream}), &d.runner_registry, &d.transcript_runtime,
            ).unwrap();
            assert!(!foreign_reply["aborted"].as_bool().unwrap(), "a foreign route cannot cancel the named stream");
            let first = f.cancel(route, &stream);
            assert!(Fixture::changed(&first, route));
            assert!(!Fixture::changed(&f.cancel(route, &stream), route), "repeat is not a new request");
            assert!(d.transcript_runtime.has_routed_turn_lease(&f.agent, &stream));
            assert!(!d.transcript_runtime.is_turn_dispatch_idle(&f.agent));
            assert_eq!(f.ended(&stream), 0, "cancel response does not retire the lease");
            let outcome = f.launch(args).expect("cancelled admission is a typed start outcome");
            assert!(matches!(&outcome, shipping::RoutedProviderStartOutcome::CancelledBeforeStart { stream_id, .. } if stream_id == &stream));
            assert_eq!(outcome.into_gateway_value(), json!({"accepted":false,"cancelled":true,"streamId":stream}));
            assert_eq!(d.local_tool_permission.controller().direction_epoch(&f.agent), direction);
            f.assert_no_prepare_write(&rows, generation);
            assert!(!d.transcript_runtime.has_routed_turn_lease(&f.agent, &stream));
            assert_eq!(d.transcript_runtime.in_flight_run_count(&f.agent), 0);
            assert!(d.transcript_runtime.is_turn_dispatch_idle(&f.agent));
            assert_eq!(f.ended(&stream), 1, "actual guard retirement emits ended once");
            assert_eq!(d.transcript_runtime.check_routed_turn_start(&foreign_guard.control).unwrap(), RoutedTurnStartState::Ready);
            assert!(!foreign_guard.control.cancellation().is_cancelled());
            assert_eq!(f.ended("foreign-stream"), 0);
            drop(foreign_guard);
            assert_eq!(f.ended("foreign-stream"), 1);
            assert!(db.delete_kv("sessionModelSelection").unwrap());
            f.accept("later-valid", "later-valid");
            let later = f.prepare("later-valid", |_| {}).expect("a later turn is admissible");
            assert_eq!(later.fabushi_model.as_deref(), Some("Vendor/PRC@Original"));
            assert_eq!(later.direction_epoch, direction + 1);
            drop(later);
            assert_eq!(f.ended("later-valid"), 1);
        }
    });
}

#[test]
fn routed_preentry_cancel_after_model_capture_refuses_permission_and_allows_next_turn() {
    bounded("tests::routed_preentry_cancellation_tests::routed_preentry_cancel_after_model_capture_refuses_permission_and_allows_next_turn", |root| {
        let f = Fixture::open(&root.join("after-model"));
        let d = &f.owners.deps;
        f.accept("captured", "captured");
        let rows = d.session_workers.read_agent_transcript_entries(&f.agent).unwrap();
        let db = d.session_workers.open_agent_db_owner(&f.agent).unwrap();
        let generation = get_sand_agent_db_write_generation(db.db_path());
        let direction = d.local_tool_permission.controller().direction_epoch(&f.agent);
        let mut observed = Vec::new();
        let result = f.prepare("captured", |point| {
            observed.push(point);
            if point == shipping::RoutedTurnAdmissionPoint::ModelCaptured {
                assert!(Fixture::changed(&f.cancel(CancelRoute::Chat, "captured"), CancelRoute::Chat));
                assert_eq!(d.local_tool_permission.controller().direction_epoch(&f.agent), direction);
                assert!(d.transcript_runtime.has_routed_turn_lease(&f.agent, "captured"));
                assert_eq!(f.ended("captured"), 0);
            }
        });
        assert!(matches!(result, Err(shipping::RoutedTurnAdmissionError::Cancelled)));
        assert_eq!(observed, vec![shipping::RoutedTurnAdmissionPoint::BeforeModel, shipping::RoutedTurnAdmissionPoint::ModelCaptured]);
        assert_eq!(d.local_tool_permission.controller().direction_epoch(&f.agent), direction);
        f.assert_no_prepare_write(&rows, generation);
        assert_eq!(f.ended("captured"), 1);
        assert!(d.transcript_runtime.is_turn_dispatch_idle(&f.agent));
        f.accept("after-cancel", "after-cancel");
        let admitted = f.prepare("after-cancel", |_| {}).expect("later valid model admission");
        assert!(admitted.fabushi_request_binding.is_some());
        assert!(admitted.original_turn.is_some(), "real durable original input remains bindable");
        assert_eq!(admitted.direction_epoch, direction + 1);
        drop(admitted);
    });
}

#[test]
fn routed_preentry_late_registration_shares_cancellation_and_retains_custody_until_guard() {
    bounded("tests::routed_preentry_cancellation_tests::routed_preentry_late_registration_shares_cancellation_and_retains_custody_until_guard", |root| {
        let f = Fixture::open(&root.join("late-attach"));
        let d = &f.owners.deps;
        f.accept("late-attach", "late-attach");
        let admitted = f.prepare("late-attach", |_| {}).expect("preparation admission wins first");
        let token = admitted.lease_guard.control.cancellation();
        assert!(Fixture::changed(&f.cancel(CancelRoute::Stream, "late-attach"), CancelRoute::Stream));
        let attached = d.runner_registry.register_routed_provider_with_cancellation(
            &f.agent, "late-attach", false, token.clone(),
        ).expect("late attachment uses the existing cancelled token");
        assert!(attached.is_cancelled());
        assert_eq!(attached.reason().as_deref(), Some("explicit PRC user stop"));
        assert!(attached.check().is_err());
        assert!(!d.runner_registry.cancel_stream("late-attach", "must not replace first reason"));
        assert_eq!(token.reason(), attached.reason());
        assert_eq!(d.runner_registry.active_count(), 1);
        assert!(d.transcript_runtime.has_routed_turn_lease(&f.agent, "late-attach"));
        assert_eq!(f.ended("late-attach"), 0);

        let next_args = f.args("queued-successor", "queued-successor");
        let runtime = Arc::clone(&d.transcript_runtime);
        let sessions = Arc::clone(&d.session_workers);
        let (queued_tx, queued_rx) = mpsc::channel();
        let (accepted_tx, accepted_rx) = mpsc::channel();
        let successor = std::thread::spawn(move || {
            let result = runtime.accept_routed_send_with_queue_observers(
                &next_args,
                |ack| persist_accepted_send_prompt_context(&sessions, &next_args, ack).map_err(shipping::map_session_send_error),
                |_| false, |_| { queued_tx.send(()).unwrap(); }, |_| {},
            );
            accepted_tx.send(result.is_ok()).unwrap();
            result.expect("successor obtains the existing serial lane after retirement");
        });
        queued_rx.recv_timeout(WAIT).expect("real successor queue admission");
        assert!(accepted_rx.try_recv().is_err(), "cancel acknowledgement cannot open the lane");
        assert!(!d.transcript_runtime.has_routed_turn_lease(&f.agent, "queued-successor"));
        assert_eq!(f.ended("late-attach"), 0);
        d.runner_registry.finish_routed_provider("late-attach");
        assert!(d.transcript_runtime.has_routed_turn_lease(&f.agent, "late-attach"),
            "registry retirement alone does not own the Host lease");
        assert!(accepted_rx.try_recv().is_err());
        drop(admitted);
        assert!(accepted_rx.recv_timeout(WAIT).expect("successor after actual guard retirement"));
        successor.join().expect("owned queue worker joined");
        assert_eq!(f.ended("late-attach"), 1);
        let next = f.prepare("queued-successor", |_| {}).expect("next generation executes admission normally");
        assert!(!next.lease_guard.control.cancellation().is_cancelled());
        drop(next);
        assert_eq!(f.ended("queued-successor"), 1);
    });
}

#[test]
fn routed_preentry_stale_and_foreign_controls_cannot_retire_reused_stream() {
    bounded("tests::routed_preentry_cancellation_tests::routed_preentry_stale_and_foreign_controls_cannot_retire_reused_stream", |root| {
        let f = Fixture::open(&root.join("guard-identity"));
        let d = &f.owners.deps;
        f.accept("reused", "first-input");
        assert!(d.transcript_runtime.capture_routed_turn_start("foreign-agent", "reused").is_err());
        assert!(!d.transcript_runtime.cancel_routed_turn_lease("reused", Some("foreign-agent"), "foreign stop"));
        let old = shipping::RoutedTurnLeaseGuard::new(
            Arc::clone(&d.transcript_runtime), f.agent.clone(), "reused".into(), d.events.clone(),
        ).expect("one start claimant");
        assert!(d.transcript_runtime.capture_routed_turn_start(&f.agent, "reused").is_err());
        assert_eq!(d.transcript_runtime.check_routed_turn_start(&old.control).unwrap(), RoutedTurnStartState::Ready);

        // A different owner can have the same text IDs and first ticket number.
        // Its token identity still cannot authorize this runtime's retirement.
        let foreign_runtime = ProductionTranscriptRuntime::new(None);
        foreign_runtime.accept_routed_send(&f.args("reused", "foreign-input"), |_| {
            Ok::<_, ProductionSendError>(PersistedSendContext::default())
        }).unwrap();
        let foreign = foreign_runtime.capture_routed_turn_start(&f.agent, "reused").unwrap();
        assert!(d.transcript_runtime.check_routed_turn_start(&foreign).is_err());
        assert!(d.transcript_runtime.settle_routed_turn_control(&foreign, 10).unwrap().is_none());
        assert!(d.transcript_runtime.has_routed_turn_lease(&f.agent, "reused"));
        assert_eq!(f.ended("reused"), 0);
        foreign_runtime.settle_routed_turn_control(&foreign, 11).unwrap();

        assert!(d.transcript_runtime.settle_routed_turn_control(&old.control, 12).unwrap().is_some());
        assert_eq!(f.ended("reused"), 1);
        f.accept("reused", "replacement-input");
        let replacement = shipping::RoutedTurnLeaseGuard::new(
            Arc::clone(&d.transcript_runtime), f.agent.clone(), "reused".into(), d.events.clone(),
        ).expect("replacement lease has a new captured generation");
        assert!(d.transcript_runtime.check_routed_turn_start(&old.control).is_err());
        assert!(d.transcript_runtime.settle_routed_turn_control(&old.control, 13).unwrap().is_none());
        drop(old);
        assert!(d.transcript_runtime.has_routed_turn_lease(&f.agent, "reused"));
        assert_eq!(f.ended("reused"), 1, "old guard did not end the replacement");
        assert_eq!(d.transcript_runtime.check_routed_turn_start(&replacement.control).unwrap(), RoutedTurnStartState::Ready);
        assert!(!replacement.control.cancellation().is_cancelled());
        drop(replacement);
        assert_eq!(f.ended("reused"), 2);
        assert!(d.transcript_runtime.is_turn_dispatch_idle(&f.agent));
        assert_eq!(d.transcript_runtime.in_flight_run_count(&f.agent), 0);
    });
}

