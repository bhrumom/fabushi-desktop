//! Causal consumer oracles for OCRM-AUTOMATION-TERMINAL-CUSTODY-v1.
//! The actual guarded effect, finalizer, waiter, Automation/Session stores and
//! production backend callback remain real; only external transport is controlled.

use super::{
    RoutedProviderTerminalFixture, UnusedCompactionProviderPorts,
    completed_guard_custody_observation, completed_guard_custody_report,
};
use crate::{
    AutomationExecutionResult, ProductionSessionWorkers, TranscriptRunnerRegistry,
    RUNNER_INFERENCE_EVENT_CHANNEL, automation_fire_completion, automation_terminal_from_event,
};
use std::fs;

#[test]
fn automation_terminal_custody_requires_bound_complete_evidence() {
    use mahayana_host_runtime::runner::{
        ToolEffectCustodyObservation, ToolEffectCustodyReport,
        ToolEffectCustodyUnavailableReason, ToolEffectUnknownResult,
    };
    use std::sync::mpsc;
    use std::time::{Duration, Instant};
    use serde_json::json;

    let registry = TranscriptRunnerRegistry::default();
    let stream_id = "automation-custody-matrix";
    let clear = completed_guard_custody_observation(1);
    let mut reports = vec![
        ("missing", None),
        ("null", Some(serde_json::Value::Null)),
        ("wrong-type", Some(json!("not custody"))),
        ("missing-invocations", Some(json!({"streamId":stream_id}))),
        ("empty", Some(json!({"streamId":stream_id,"invocations":[]}))),
        ("unknown-state", Some(json!({"streamId":stream_id,"invocations":[{"state":"futureState"}]}))),
        ("incomplete-snapshot", Some(json!({"streamId":stream_id,"invocations":[{"state":"known","runGeneration":1}]}))),
    ];
    for (name, binding) in [("foreign", "other-stream"), ("blank", "")] {
        let mut report = ToolEffectCustodyReport::new(binding);
        report.record(Some(&clear));
        reports.push((name, Some(serde_json::to_value(report).unwrap())));
    }
    for (name, reason) in [
        ("missing-observation", ToolEffectCustodyUnavailableReason::MissingObservation),
        ("uninstrumented", ToolEffectCustodyUnavailableReason::UninstrumentedExecution),
        ("interrupted-observation", ToolEffectCustodyUnavailableReason::ObservationInterrupted),
        ("binding-mismatch", ToolEffectCustodyUnavailableReason::StreamBindingMismatch),
    ] {
        let mut report = ToolEffectCustodyReport::new(stream_id);
        report.record(Some(&ToolEffectCustodyObservation::Unavailable { reason }));
        report.record(Some(&clear));
        reports.push((name, Some(serde_json::to_value(report).unwrap())));
    }
    let ToolEffectCustodyObservation::Known { snapshot, .. } = &clear else {
        panic!("guard must supply a known observation");
    };
    for (name, generation, pending, unknown) in [
        ("zero-generation", 0, 0, None),
        ("pending", 1, 1, None),
        ("timeout", 1, 0, Some(ToolEffectUnknownResult::TimedOut)),
        ("worker-lost", 1, 0, Some(ToolEffectUnknownResult::WorkerLost)),
        ("discarded", 1, 0, Some(ToolEffectUnknownResult::DiscardedResult)),
        ("downstream-unknown", 1, 0, Some(ToolEffectUnknownResult::DownstreamOutcomeUnknown)),
    ] {
        let mut snapshot = snapshot.clone();
        snapshot.pending_effects = pending;
        snapshot.unknown_result = unknown;
        let mut report = ToolEffectCustodyReport::new(stream_id);
        report.record(Some(&ToolEffectCustodyObservation::Known { run_generation: generation, snapshot }));
        report.record(Some(&clear));
        reports.push((name, Some(serde_json::to_value(report).unwrap())));
    }
    for kind in ["completed", "failed", "cancelled"] {
        for (name, report) in &reports {
            let mut event = json!({"channel":RUNNER_INFERENCE_EVENT_CHANNEL,
                "payload":{"streamId":stream_id,"type":kind,"message":"local outcome"}});
            if let Some(report) = report {
                event["payload"]["toolEffectCustody"] = report.clone();
            }
            // The shared local-terminal mapper remains unchanged for other
            // consumers; custody is applied by the Automation waiter only.
            let local = automation_terminal_from_event(&event, stream_id).unwrap().unwrap();
            let (sender, receiver) = mpsc::channel();
            sender.send(event).unwrap();
            let result = crate::wait_for_automation_terminal(
                &receiver, &registry, stream_id, Instant::now() + Duration::from_secs(5), Instant::now,
            );
            assert_eq!(result, Ok(AutomationExecutionResult::RetainedUnsettled {
                terminal: Box::new(local),
            }), "{kind}/{name}: later clear evidence must not erase uncertainty");
        }
    }
}

#[test]
fn automation_terminal_custody_preserves_safe_outcomes_and_upgrade_annotation() {
    use mahayana_host_runtime::runner::production_turn_agent_owner::ProductionTurnAgentOwner;
    use mahayana_host_runtime::runner::routed_provider_runtime::RunnerRequestContextSnapshot;
    use mahayana_host_runtime::runner::turn_agent_composition::TurnAgentComposition;
    use mahayana_host_runtime::runner::ToolEffectCustodyObservation;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    for (kind, upgrade, retain) in [
        ("completed", false, false), ("failed", false, false),
        ("cancelled", false, false), ("cancelled", true, false),
        ("cancelled", true, true),
    ] {
        let fixture = RoutedProviderTerminalFixture::new();
        let stream_id = "safe-custody-terminal";
        let receiver = fixture.events.subscribe();
        let (cancellation, mut lease) = fixture.admit_with_source("safe-agent", stream_id, "automation");
        assert!(!cancellation.is_cancelled(), "admission must supply a fresh handle");
        assert_eq!(cancellation.reason(), None);
        assert!(!fixture.registry.is_quiescing_for_upgrade());
        if upgrade {
            fixture.registry.request_quiesce_for_upgrade();
            assert!(cancellation.is_cancelled(), "upgrade owns the first cancellation");
            assert_eq!(cancellation.reason().as_deref(), Some("quiescing for forced host upgrade"));
            assert!(!fixture.registry.cancel_stream(stream_id, "safe terminal cancellation"),
                "a repeat cancellation cannot claim a new transition");
            assert_eq!(cancellation.reason().as_deref(), Some("quiescing for forced host upgrade"));
        } else if kind == "cancelled" {
            assert!(fixture.registry.cancel_stream(stream_id, "safe terminal cancellation"));
            assert!(cancellation.is_cancelled());
            assert_eq!(cancellation.reason().as_deref(), Some("safe terminal cancellation"));
        }
        assert_eq!(fixture.registry.is_quiescing_for_upgrade(), upgrade);
        assert_eq!(fixture.registry.active_stream_ids_for_agent("safe-agent"), vec![stream_id.to_string()]);
        assert!(fixture.runtime.is_agent_running("safe-agent"));
        fixture.runtime.require_routed_turn_lease("safe-agent", stream_id)
            .expect("cancellation alone must not retire the admitted turn");
        let provider_result = if kind == "failed" {
            Err(crate::ProviderSessionError::Tool("known failure".into()))
        } else { Ok("local result".into()) };
        let report = if retain { crate::ToolEffectCustodyReport::new(stream_id) }
            else { completed_guard_custody_report(stream_id) };
        let terminal = crate::RoutedProviderTerminal::capture(
            stream_id, false, &cancellation, provider_result, 2, true, report,
        );
        assert_eq!(terminal.event["payload"]["type"], kind);
        assert_eq!(terminal.event["payload"]["sentMessageCount"], 2);
        assert_eq!(terminal.event["payload"]["reacted"], true);
        fixture.runtime.end_provider_run_with_kind("safe-agent", false);
        fixture.registry.finish_routed_provider(stream_id);
        lease.settle();
        terminal.publish(&fixture.events);
        let local = match kind {
            "completed" => AutomationExecutionResult::Completed,
            "failed" => AutomationExecutionResult::Failed { detail: "known failure".into() },
            _ => AutomationExecutionResult::Interrupted {
                detail: "Interrupted before it finished.".into(), quiesced_for_upgrade: upgrade,
            },
        };
        let expected = if retain { AutomationExecutionResult::RetainedUnsettled { terminal: Box::new(local) } }
            else { local };
        assert_eq!(crate::wait_for_automation_terminal(
            &receiver, &fixture.registry, stream_id, Instant::now() + Duration::from_secs(5), Instant::now,
        ), Ok(expected));
        drop(lease);
        fixture.close();
    }

    // Obtain NotStarted from the real production owner refusing before its
    // build input, provider transport, tools or checkpoint store are used.
    let fixture = RoutedProviderTerminalFixture::new();
    let composition = TurnAgentComposition::new(
        crate::RoutedProvider::OpenRouter, Arc::new(UnusedCompactionProviderPorts),
        RunnerRequestContextSnapshot {
            context: mahayana_host_runtime::host_request_context::HostRequestContext {
                os_version: "automation-custody".into(), shell: None, time_zone: Some("UTC".into()),
                transcripts_folder: fixture.root.join("transcripts").to_string_lossy().into_owned(),
                user_full_name: None,
            }, rules: None,
        }, crate::RoutedProviderCancellation::default(), Arc::new(UnusedCompactionProviderPorts),
    );
    let mut owner = ProductionTurnAgentOwner::new(composition);
    let refusal = owner.run_routed_provider(&fixture.root, &[
        crate::ProviderMessage { role: "user".into(), content: "never admitted".into() },
    ], &mut |_, _| panic!("pre-entry refusal cannot emit provider output"));
    assert!(matches!(&refusal, Err(crate::ProviderSessionError::Configuration(detail))
        if detail == "production turn build input is not bound"));
    assert_eq!(owner.last_tool_effect_custody(), Some(&ToolEffectCustodyObservation::NotStarted));
    let mut report = crate::ToolEffectCustodyReport::new("not-started");
    report.record(owner.last_tool_effect_custody());
    let terminal = crate::RoutedProviderTerminal::capture(
        "not-started", false, &crate::RoutedProviderCancellation::default(), refusal, 0, false, report,
    );
    let expected = automation_terminal_from_event(&terminal.event, "not-started").unwrap().unwrap();
    assert!(matches!(&expected, AutomationExecutionResult::Failed { .. }));
    let receiver = fixture.events.subscribe();
    terminal.publish(&fixture.events);
    assert_eq!(crate::wait_for_automation_terminal(
        &receiver, &fixture.registry, "not-started", Instant::now() + Duration::from_secs(5), Instant::now,
    ), Ok(expected));
    drop(owner);
    fixture.close();
}

#[test]
fn automation_unknown_tool_terminal_retains_durable_run_and_backend_delivery() {
    use mahayana_host_runtime::automations::automation::AutomationSpec;
    use mahayana_host_runtime::automations::automation_id::stable_automation_id;
    use mahayana_host_runtime::extensions::automations::backend_transport::{
        AutomationsBackendError, AutomationsBackendTransport,
    };
    use mahayana_host_runtime::extensions::automations::fire_delivery::{
        PreparedBackendFire, prepare_backend_fire,
    };
    use mahayana_host_runtime::extensions::automations::production::ProductionAutomationsBackendRuntime;
    use mahayana_host_runtime::extensions::browser_ua::extension::StopSubscription;
    use mahayana_host_runtime::extensions::notify_bus::extension::{
        HostNotifyBusExtension, NotifyBusExperimentsApi, NotifyBusExtensionOptions,
        start_notify_bus_extension_with_options,
    };
    use mahayana_host_runtime::extensions::notify_bus::notify_bus_client::NotifyBusTiming;
    use mahayana_host_runtime::extensions::session::agent_session::SandAgentSessionStore;
    use mahayana_host_runtime::extensions::transcript::automation_run_path::AutomationRunObservation;
    use mahayana_host_runtime::extensions::transcript::automation_runtime::AutomationRuntime;
    use mahayana_host_runtime::runner::production_turn_run_shell_adapter::RoutedProviderAttemptControl;
    use mahayana_host_runtime::runner::tools::mcp_meta_tools::execute_routed_tool_with_timeout_ms;
    use mahayana_host_runtime::runner::{
        StreamAttemptPolicy, StreamAttemptRuntime, ToolEffectCustodyObservation,
        ToolEffectCustodyReport, ToolEffectUnknownResult,
    };
    use std::io::Write;
    use std::sync::{Arc, Mutex, mpsc, atomic::{AtomicUsize, Ordering}};
    use std::time::{Duration, Instant};
    use serde_json::{Value, json};

    struct PollTransport { fire: Value, calls: Mutex<Vec<(String, Value)>> }
    impl AutomationsBackendTransport for PollTransport {
        fn post_json(&self, path: &str, body: &Value) -> Result<Value, AutomationsBackendError> {
            self.calls.lock().unwrap().push((path.into(), body.clone()));
            match path {
                "/sand/listener-subscriptions" => Ok(json!({"slack":{"status":"ok"},"github":{"status":"ok"}})),
                "/sand/listener-events/poll" => Ok(json!({"events":[]})),
                "/sand/automation-events/poll" => Ok(json!({"events":[self.fire.clone()]})),
                _ => Err(AutomationsBackendError::Transport(format!("unexpected request {path}"))),
            }
        }
    }
    struct NoNotifyConnection;
    impl NotifyBusExperimentsApi for NoNotifyConnection {
        fn check_feature_gate(&self, _name: &str) -> bool { false }
        fn subscribe(&self, _listener: Arc<dyn Fn() + Send + Sync>) -> StopSubscription {
            Box::new(|| {})
        }
    }
    struct OwnedBackend {
        backend: Arc<ProductionAutomationsBackendRuntime>,
        notify: HostNotifyBusExtension,
    }
    impl Drop for OwnedBackend {
        fn drop(&mut self) { self.backend.stop(); self.notify.stop(); }
    }
    type Delivery = (PreparedBackendFire, Arc<dyn Fn(Option<crate::FireCompletion>) + Send + Sync>);
    fn start_backend(
        transport: Arc<PollTransport>, automations: Arc<AutomationRuntime>,
        errors: Arc<Mutex<Vec<String>>>,
    ) -> (OwnedBackend, mpsc::Receiver<Delivery>) {
        let notify = start_notify_bus_extension_with_options(Arc::new(NoNotifyConnection),
            NotifyBusExtensionOptions {
                get_backend_url: Arc::new(|| panic!("this fixture does not mark the notify owner ready")),
                get_access_token: Arc::new(|_| panic!("the controlled poll must not read credentials")),
                now_ms: Arc::new(|| 0), timing: NotifyBusTiming::default(), log: Arc::new(|_| {}),
            }).expect("real notify owner without an external connection");
        let (sender, receiver) = mpsc::channel();
        let poll_errors = Arc::clone(&errors);
        let backend = ProductionAutomationsBackendRuntime::start(
            transport, notify.clone(), Arc::new(|| (Vec::new(), Vec::new())), Arc::new(|_| false),
            Arc::new(move |fire, completion| {
                let entries = match automations.list_all_automations() {
                    Ok(entries) => entries,
                    Err(error) => { errors.lock().unwrap().push(error); return false; }
                };
                let prepared = prepare_backend_fire(&entries, &fire, |_| None, |entry, run_uuid| {
                    automations.read_run_for_delivery(&entry.agent_id, &entry.automation.id, run_uuid)
                });
                let accepted = matches!(&prepared, PreparedBackendFire::Schedule { .. });
                sender.send((prepared, completion)).expect("bounded receiver owns this dispatch");
                accepted
            }),
            Arc::new(|| (true, true)),
            Arc::new(move |error| poll_errors.lock().unwrap().push(error.to_string())),
            Arc::new(|_| {}),
        );
        (OwnedBackend { backend, notify }, receiver)
    }
    fn assert_no_completion_or_ack(transport: &PollTransport) {
        let calls = transport.calls.lock().unwrap();
        assert!(calls.iter().filter(|(path, _)| path == "/sand/automation-events/poll").count() >= 2);
        for (path, body) in calls.iter() {
            assert!(!path.ends_with("/complete"), "unknown effects cannot be completed: {path}");
            if path == "/sand/automation-events/poll" {
                assert_eq!(body["ackRunUuids"], json!([]));
            }
        }
    }

    for held_after_return in [true, false] {
        let fixture = RoutedProviderTerminalFixture::new();
        let agents_root = fixture.root.join("agents");
        let workers = Arc::new(ProductionSessionWorkers::with_agents_root(agents_root.clone(), 500));
        let sessions = SandAgentSessionStore::new(Arc::clone(&workers));
        let agent_id = sessions.create_session(None, "user", None).expect("real agent").id;
        let automations = Arc::new(AutomationRuntime::new(Arc::clone(&workers)));
        let (definitions, _) = automations.create_agent_automation(&agent_id, &AutomationSpec {
            name: "Unsettled custody".into(), prompt: "Inspect the real tool result".into(),
            trigger: json!({"type":"cron","schedule":"@daily"}), is_enabled: Some(true),
        }).expect("real definition");
        let automation_id = definitions[0].id.clone();
        let store = sessions.automation_store_for(&agent_id).expect("admitted store");
        let run_id = "custody-run";
        let stream_id = "custody-stream";
        let transport = Arc::new(PollTransport {
            fire: json!({"id":run_id,"sandAgentId":agent_id,
                "automationId":stable_automation_id(&agent_id, &automation_id),"timestampMs":1000}),
            calls: Mutex::new(Vec::new()),
        });
        let backend_errors = Arc::new(Mutex::new(Vec::new()));
        let (backend, deliveries) = start_backend(
            Arc::clone(&transport), Arc::clone(&automations), Arc::clone(&backend_errors),
        );
        let (prepared, completion) = deliveries.recv_timeout(Duration::from_secs(10)).expect("real backend dispatch");
        let PreparedBackendFire::Schedule { agent_id: target_agent, automation_id: target_automation,
            run_uuid, scheduled_for_ms } = prepared else { panic!("initial backend fire must be admitted"); };
        assert_eq!((&target_agent, &target_automation, run_uuid.as_str()), (&agent_id, &automation_id, run_id));
        let observations = Arc::new(Mutex::new(Vec::<AutomationRunObservation>::new()));
        let observed = Arc::clone(&observations);
        automations.run_path().set_run_reporter(Some(Arc::new(move |report| observed.lock().unwrap().push(report.clone()))));
        let effect_runtime = Arc::new(StreamAttemptRuntime::new(StreamAttemptPolicy::default()));
        let effects = Arc::new(AtomicUsize::new(0));
        let receipt = fixture.root.join("counted-external-effect.txt");
        let (release, held) = mpsc::channel();
        let (late_done, late_return) = mpsc::channel();
        let mut baseline = None;
        let mut frozen_terminal = None;
        let outcome = automations.run_server_scheduled_automation_with(
            &target_agent, &target_automation, run_uuid, scheduled_for_ms.map(|value| value as f64), |_| {
                let history = store.read_runs(&automation_id);
                assert_eq!(history.len(), 1);
                assert_eq!(history[0].id, run_id);
                assert_eq!(history[0].status, "running");
                assert!(history[0].finished_at.is_none());
                let rows = workers.list_cron_session_runs(&agent_id).unwrap();
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].run_id, run_id);
                assert_eq!(rows[0].state, "running");
                assert!(!rows[0].fence_state_known);
                baseline = Some((fs::read(store.config_path(&automation_id)).unwrap(),
                    fs::read(store.runs_path(&automation_id)).unwrap(), rows));
                assert!(automations.run_path().is_in_flight(&agent_id, &automation_id));
                let receiver = fixture.events.subscribe();
                let (cancellation, mut lease) = fixture.admit_with_source(&agent_id, stream_id, "automation");
                let attempt = RoutedProviderAttemptControl::new(Arc::clone(&effect_runtime),
                    effect_runtime.begin_attempt(), cancellation.clone());
                let counted = Arc::clone(&effects);
                let worker_receipt = receipt.clone();
                let (entered, entry) = mpsc::channel();
                let (returned, guard_return) = mpsc::channel();
                let caller = std::thread::spawn(move || {
                    let result = execute_routed_tool_with_timeout_ms(&attempt, "automation-custody".into(), 500, move || {
                        let mut file = fs::OpenOptions::new().create_new(true).write(true)
                            .open(worker_receipt).expect("one actual effect");
                        writeln!(file, "one admitted effect").expect("effect bytes");
                        file.sync_all().expect("durable effect receipt");
                        counted.fetch_add(1, Ordering::AcqRel);
                        entered.send(()).expect("effect entered");
                        if held_after_return {
                            held.recv_timeout(Duration::from_secs(10)).expect("release exact held operation");
                            late_done.send(()).expect("observe late return");
                            Ok(json!({"content":[]}))
                        } else {
                            Err(crate::ProviderSessionError::ToolOutcomeUnknown("downstream result unavailable".into()))
                        }
                    });
                    returned.send(result).expect("guard result reaches the actual finalizer");
                });
                entry.recv_timeout(Duration::from_secs(10)).expect("entered before terminal");
                let result = guard_return.recv_timeout(Duration::from_secs(10)).expect("guard returns");
                caller.join().expect("owned guard caller exits");
                assert!(result.is_err());
                let snapshot = effect_runtime.effect_custody();
                assert_eq!(snapshot.pending_effects, usize::from(held_after_return));
                assert_eq!(snapshot.unknown_result, Some(if held_after_return { ToolEffectUnknownResult::TimedOut }
                    else { ToolEffectUnknownResult::DownstreamOutcomeUnknown }));
                let mut report = ToolEffectCustodyReport::new(stream_id);
                report.record(Some(&ToolEffectCustodyObservation::Known { run_generation: 1, snapshot }));
                report.record(Some(&completed_guard_custody_observation(2)));
                assert!(report.requires_retention());
                assert!(fixture.registry.cancel_stream(stream_id, "retire local work with unknown external outcome"));
                let terminal = crate::RoutedProviderTerminal::capture(
                    stream_id, false, &cancellation, result.map(|_| "returned".into()), 1, true, report,
                );
                assert_eq!(terminal.event["payload"]["type"], "cancelled");
                assert_eq!(terminal.event["payload"]["sentMessageCount"], 1);
                assert_eq!(terminal.event["payload"]["reacted"], true);
                frozen_terminal = Some(terminal.event.clone());
                fixture.runtime.end_provider_run_with_kind(&agent_id, false);
                fixture.registry.finish_routed_provider(stream_id);
                lease.settle();
                assert!(fixture.registry.active_stream_ids_for_agent(&agent_id).is_empty());
                assert!(!fixture.runtime.has_routed_turn_lease(&agent_id, stream_id));
                let structured = Mutex::new(fixture.telemetry.logs.start_turn(&agent_id, "automation", None));
                terminal.finalize_observations(&structured, None);
                terminal.publish(&fixture.events);
                let result = crate::wait_for_automation_terminal(
                    &receiver, &fixture.registry, stream_id, Instant::now() + Duration::from_secs(5), Instant::now,
                ).expect("real Automation terminal waiter");
                assert_eq!(result, AutomationExecutionResult::RetainedUnsettled {
                    terminal: Box::new(AutomationExecutionResult::Interrupted {
                        detail: "Interrupted before it finished.".into(), quiesced_for_upgrade: false,
                    }),
                });
                assert!(matches!(receiver.try_recv(), Err(mpsc::TryRecvError::Empty)));
                Ok(result)
            },
        );
        assert_eq!(outcome, Ok(None), "the real run path must retain, not synthesize a local failure");
        assert_eq!(automation_fire_completion(outcome.clone()), None);
        completion(automation_fire_completion(outcome));
        drop(completion);
        assert!(!automations.run_path().is_in_flight(&agent_id, &automation_id));
        let (config_bytes, history_bytes, rows) = baseline.expect("baseline after actual admission");
        assert_eq!(fs::read(store.config_path(&automation_id)).unwrap(), config_bytes);
        assert_eq!(fs::read(store.runs_path(&automation_id)).unwrap(), history_bytes);
        assert_eq!(workers.list_cron_session_runs(&agent_id).unwrap(), rows);
        assert!(observations.lock().unwrap().is_empty());
        let frozen = frozen_terminal.expect("actual terminal frozen before local retirement");
        let published: ToolEffectCustodyReport = serde_json::from_value(frozen["payload"]["toolEffectCustody"].clone()).unwrap();
        assert!(published.requires_retention());
        if held_after_return {
            release.send(()).expect("release only the admitted effect");
            late_return.recv_timeout(Duration::from_secs(5)).expect("late operation returned");
            let deadline = Instant::now() + Duration::from_secs(5);
            while effect_runtime.effect_custody().pending_effects != 0 {
                assert!(Instant::now() < deadline, "late guard receipt did not return");
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        assert!(effect_runtime.effect_custody().unknown_result.is_some());
        assert_eq!(serde_json::to_value(&published).unwrap(), frozen["payload"]["toolEffectCustody"]);
        assert_eq!(effects.load(Ordering::Acquire), 1);
        assert_eq!(fs::read_to_string(&receipt).unwrap().lines().count(), 1);
        backend.backend.request_reconcile();
        let (prepared, _) = deliveries.recv_timeout(Duration::from_secs(10)).expect("volatile None release permits redelivery");
        assert_eq!(prepared, PreparedBackendFire::Abandon { reason: "existing_run_unsettled" });
        assert_no_completion_or_ack(&transport);
        assert!(backend_errors.lock().unwrap().is_empty());
        drop(backend);
        drop(deliveries);
        assert_eq!(fs::read(store.config_path(&automation_id)).unwrap(), config_bytes);
        assert_eq!(fs::read(store.runs_path(&automation_id)).unwrap(), history_bytes);
        assert_eq!(workers.list_cron_session_runs(&agent_id).unwrap(), rows);
        assert!(observations.lock().unwrap().is_empty());
        automations.dispose();
        workers.shutdown();
        drop(store);
        drop(automations);
        drop(sessions);
        drop(workers);

        // Reconstruct the actual owners from the same files. No terminal
        // row, synthetic history or volatile backend state is seeded.
        let reopened_workers = Arc::new(ProductionSessionWorkers::with_agents_root(agents_root, 500));
        let reopened_sessions = SandAgentSessionStore::new(Arc::clone(&reopened_workers));
        let reopened = Arc::new(AutomationRuntime::new(Arc::clone(&reopened_workers)));
        let reopened_store = reopened_sessions.automation_store_for(&agent_id).unwrap();
        assert_eq!(fs::read(reopened_store.config_path(&automation_id)).unwrap(), config_bytes);
        assert_eq!(fs::read(reopened_store.runs_path(&automation_id)).unwrap(), history_bytes);
        assert_eq!(reopened_workers.list_cron_session_runs(&agent_id).unwrap(), rows);
        let (reopened_backend, reopened_deliveries) = start_backend(
            Arc::clone(&transport), Arc::clone(&reopened), Arc::clone(&backend_errors),
        );
        let (prepared, _) = reopened_deliveries.recv_timeout(Duration::from_secs(10)).expect("reopened real backend dispatch");
        assert_eq!(prepared, PreparedBackendFire::Abandon { reason: "existing_run_unsettled" });
        assert_no_completion_or_ack(&transport);
        assert!(backend_errors.lock().unwrap().is_empty());
        drop(reopened_backend);
        drop(reopened_deliveries);
        assert_eq!(fs::read(reopened_store.runs_path(&automation_id)).unwrap(), history_bytes);
        assert_eq!(reopened_workers.list_cron_session_runs(&agent_id).unwrap(), rows);
        assert_eq!(effects.load(Ordering::Acquire), 1);
        reopened.dispose();
        reopened_workers.shutdown();
        drop(reopened_store);
        drop(reopened);
        drop(reopened_sessions);
        drop(reopened_workers);
        fixture.close();
    }
}

// BH-05/06: exercise the same discovery/preparation/immediate-callback boundary
// that UnifiedGatewayApi passes to the real production backend. The backend,
// Runtime, Session admission and file/cron stores remain the actual owners.
mod checked_backend_delivery {
    use crate::{AutomationExecutionResult, ProductionSessionWorkers, automation_fire_completion};
    use mahayana_host_runtime::automations::automation::{AutomationSpec, AUTOMATION_MAX_RUN_HISTORY};
    use mahayana_host_runtime::automations::automation_id::stable_automation_id;
    use mahayana_host_runtime::extensions::automations::backend_transport::{
        AutomationsBackendError, AutomationsBackendTransport,
    };
    use mahayana_host_runtime::extensions::automations::fire_delivery::PreparedBackendFire;
    use mahayana_host_runtime::extensions::automations::production::ProductionAutomationsBackendRuntime;
    use mahayana_host_runtime::extensions::automations::sand_automation_fire_consumer::FireCompletion;
    use mahayana_host_runtime::extensions::browser_ua::extension::StopSubscription;
    use mahayana_host_runtime::extensions::notify_bus::extension::{
        HostNotifyBusExtension, NotifyBusExperimentsApi, NotifyBusExtensionOptions,
        start_notify_bus_extension_with_options,
    };
    use mahayana_host_runtime::extensions::notify_bus::notify_bus_client::NotifyBusTiming;
    use mahayana_host_runtime::extensions::session::agent_db::CronSessionRunRecord;
    use mahayana_host_runtime::extensions::session::agent_session::SandAgentSessionStore;
    use mahayana_host_runtime::extensions::transcript::automation_run_path::{
        AutomationRunObservation, AutomationRunTrigger,
    };
    use mahayana_host_runtime::extensions::transcript::automation_runtime::AutomationRuntime;
    use serde_json::{Value, json};
    use std::collections::BTreeSet;
    use std::ffi::OsString;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::{Child, Command};
    use std::sync::{Arc, Mutex, mpsc, atomic::{AtomicUsize, Ordering}};
    use std::time::{Duration, Instant};

    const ORIGINAL: &str = "checked-shipping-original";
    const ALIAS: &str = "checked-shipping-alias";
    const WAIT: Duration = Duration::from_secs(10);
    const CHILD_CASE: &str = "FABUSHI_CHECKED_BACKEND_CHILD_CASE";
    const CHILD_ROOT: &str = "FABUSHI_CHECKED_BACKEND_CHILD_ROOT";
    const CHILD_NONCE: &str = "FABUSHI_CHECKED_BACKEND_CHILD_NONCE";
    const ARTIFACT_ROOT: &str = "FABUSHI_AUTOMATION_CONTRACT_ARTIFACT_ROOT";

    // Keep the existing Automation bounded-child custody contract: exclusive
    // fixture leaf, exact case/PID/nonce receipt, 45-second bound and retained
    // failure artifacts. A stuck owner mutex is a failure, never a skipped case.
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
                eprintln!("[checked-backend] retained child={} root={}", self.child.id(), self.root.display());
            }
        }
    }

    fn bounded(case: &str, run: impl FnOnce(&Path)) {
        if std::env::var(CHILD_CASE).ok().as_deref() == Some(case) {
            let root = PathBuf::from(std::env::var_os(CHILD_ROOT).expect("owned fixture root"));
            let nonce = std::env::var(CHILD_NONCE).expect("owned fixture nonce");
            let allocation: Value = serde_json::from_slice(&fs::read(root.join("allocation.json")).unwrap()).unwrap();
            assert_eq!(allocation["case"], case);
            assert_eq!(allocation["nonce"], nonce);
            eprintln!("[checked-backend] entered pid={} case={} root={}", std::process::id(), case, root.display());
            run(&root);
            fs::write(root.join("completed.json"), json!({
                "case":case, "nonce":nonce, "pid":std::process::id(),
            }).to_string()).expect("completed assertions receipt");
            return;
        }
        let base = std::env::var_os(ARTIFACT_ROOT).map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
        let root = base.join(format!("fabushi-checked-backend-{}-{}", std::process::id(), uuid::Uuid::new_v4()));
        fs::create_dir(&root).expect("claim exclusive checked-backend fixture");
        let nonce = uuid::Uuid::new_v4().to_string();
        fs::write(root.join("allocation.json"), json!({
            "case":case, "nonce":nonce, "parentPid":std::process::id(), "root":root,
        }).to_string()).unwrap();
        let child = Command::new(std::env::current_exe().expect("test executable"))
            .args(["--exact", case, "--nocapture", "--test-threads=1"])
            .env(CHILD_CASE, case).env(CHILD_ROOT, &root).env(CHILD_NONCE, &nonce)
            .spawn().expect("start bounded shipping discovery case");
        eprintln!("[checked-backend] launched parent={} child={} case={} root={}",
            std::process::id(), child.id(), case, root.display());
        let mut owned = OwnedChild { child, root, completed: false };
        let deadline = Instant::now() + Duration::from_secs(45);
        let status = loop {
            if let Some(status) = owned.child.try_wait().expect("observe exact owned child") { break status; }
            assert!(Instant::now() < deadline, "checked-backend case timed out: {case}; root={}", owned.root.display());
            std::thread::sleep(Duration::from_millis(10));
        };
        assert!(status.success(), "checked-backend child failed: {case}: {status}; root={}", owned.root.display());
        let receipt: Value = serde_json::from_slice(&fs::read(owned.root.join("completed.json"))
            .expect("nonempty exact-case completion receipt")).unwrap();
        assert_eq!(receipt, json!({"case":case, "nonce":nonce, "pid":owned.child.id()}));
        owned.completed = true;
    }

    struct Owners {
        workers: Arc<ProductionSessionWorkers>,
        runtime: AutomationRuntime,
        observations: Arc<Mutex<Vec<AutomationRunObservation>>>,
    }
    impl Owners {
        fn open(root: &Path) -> Self {
            let workers = Arc::new(ProductionSessionWorkers::with_agents_root(root.join("agents"), 500));
            let runtime = AutomationRuntime::new(Arc::clone(&workers));
            let observations = Arc::new(Mutex::new(Vec::new()));
            let observed = Arc::clone(&observations);
            runtime.run_path().set_run_reporter(Some(Arc::new(move |report| {
                observed.lock().unwrap().push(report.clone());
            })));
            Self { workers, runtime, observations }
        }
        fn sessions(&self) -> SandAgentSessionStore { SandAgentSessionStore::new(Arc::clone(&self.workers)) }
    }
    impl Drop for Owners {
        fn drop(&mut self) { self.runtime.dispose(); self.workers.shutdown(); }
    }

    struct Fixture {
        root: PathBuf,
        agent_id: String,
        automation_id: String,
        unrelated_id: String,
        database: PathBuf,
        automations: PathBuf,
        config: PathBuf,
        history: PathBuf,
        config_bytes: Vec<u8>,
        history_bytes: Vec<u8>,
        cron: Vec<CronSessionRunRecord>,
    }
    impl Fixture {
        fn create(parent: &Path, name: &str) -> Self {
            let root = parent.join(name);
            fs::create_dir(&root).expect("exclusive scenario directory");
            fs::write(root.join("scenario.txt"), name).unwrap();
            let owners = Owners::open(&root);
            let sessions = owners.sessions();
            let agent_id = sessions.create_session(None, "user", None).unwrap_or_else(|error| {
                let metadata = [&root, &root.join("agents")].map(|path| {
                    (path.to_path_buf(), fs::symlink_metadata(path).map(|m| (m.is_dir(), m.file_type().is_symlink()))
                        .map_err(|e| (e.kind(), e.raw_os_error())))
                });
                panic!("real Session setup failed for {name}: {error}; owned paths={metadata:?}");
            }).id;
            let (definitions, _) = owners.runtime.create_agent_automation(&agent_id, &AutomationSpec {
                name: "Checked shipping target".into(), prompt: "Inspect the controlled execution".into(),
                trigger: json!({"type":"cron","schedule":"@daily"}), is_enabled: Some(true),
            }).unwrap();
            let automation_id = definitions[0].id.clone();
            let (definitions, _) = owners.runtime.create_agent_automation(&agent_id, &AutomationSpec {
                name: "Unrelated valid target".into(), prompt: "Independent controlled execution".into(),
                trigger: json!({"type":"cron","schedule":"@daily"}), is_enabled: Some(true),
            }).unwrap();
            let unrelated_id = definitions.iter().find(|definition| definition.id != automation_id).unwrap().id.clone();
            let store = sessions.automation_store_for(&agent_id).unwrap();
            let mut seed_executions = 0;
            let result = owners.runtime.run_background_automation_with(
                &agent_id, &automation_id, AutomationRunTrigger::Schedule, Vec::new(),
                Some(ORIGINAL.into()), vec![ALIAS.into()], 100.0, Some(90.0), |_| {
                    seed_executions += 1;
                    assert_eq!(store.read_runs(&automation_id)[0].status, "running");
                    assert_eq!(owners.workers.list_cron_session_runs(&agent_id).unwrap()[0].state, "running");
                    Ok(AutomationExecutionResult::RetainedUnsettled {
                        terminal: Box::new(AutomationExecutionResult::Completed),
                    })
                },
            );
            assert_eq!(result, Ok(None), "seed actual retained Runtime output, not a fabricated history row");
            assert_eq!(seed_executions, 1);
            assert!(owners.observations.lock().unwrap().is_empty());
            assert!(!owners.runtime.run_path().is_in_flight(&agent_id, &automation_id));
            sessions.set_agent_automation_enabled(&agent_id, &automation_id, false).unwrap();
            let config = store.config_path(&automation_id);
            let history = store.runs_path(&automation_id);
            let config_bytes = fs::read(&config).unwrap();
            let history_bytes = fs::read(&history).unwrap();
            let cron = owners.workers.list_cron_session_runs(&agent_id).unwrap();
            assert_eq!(cron.len(), 1);
            assert_eq!(cron[0].run_id, ORIGINAL);
            assert_eq!(cron[0].state, "running");
            assert!(!cron[0].fence_state_known);
            let running = owners.runtime.read_run_for_delivery(&agent_id, &automation_id, ALIAS).unwrap().unwrap();
            assert_eq!(running.id, ORIGINAL);
            assert_eq!(running.coalesced_run_ids, Some(vec![ALIAS.into()]));
            assert_eq!(running.status, "running");
            assert_eq!(running.finished_at, None);
            fs::write(root.join("baseline-history.bin"), &history_bytes).unwrap();
            fs::write(root.join("baseline-definition.bin"), &config_bytes).unwrap();
            fs::write(root.join("baseline-cron.txt"), format!("{cron:#?}")).unwrap();
            let fixture = Self {
                root, agent_id: agent_id.clone(), automation_id, unrelated_id,
                database: owners.workers.session_db_path(&agent_id).unwrap(),
                automations: store.get_location().to_path_buf(), config, history,
                config_bytes, history_bytes, cron,
            };
            drop(store);
            drop(sessions);
            drop(owners);
            fixture
        }
        fn open(&self) -> Owners { Owners::open(&self.root) }
        fn fire(&self, run: &str, automation: &str) -> Value {
            json!({"id":run, "sandAgentId":self.agent_id,
                "automationId":stable_automation_id(&self.agent_id, automation),
                "timestampMs":1000, "scheduledForMs":900})
        }
        fn retained_fires(&self) -> Vec<Value> {
            [ORIGINAL, ALIAS].map(|id| {
                let mut fire = self.fire(id, &self.automation_id);
                fire["definitionRevision"] = json!("obsolete-backend-definition");
                fire
            }).into_iter().collect()
        }
        fn assert_retained_cron(&self, owners: &Owners) {
            let rows = owners.workers.list_cron_session_runs(&self.agent_id).unwrap().into_iter()
                .filter(|row| row.automation_id == self.automation_id).collect::<Vec<_>>();
            assert_eq!(rows, self.cron, "every cron identity, state, timestamp and fence stays unchanged");
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    enum PathImage { Missing, File(Vec<u8>), Directory(Vec<(OsString, PathImage)>), Link(PathBuf) }
    impl PathImage {
        fn read(path: &Path) -> Self {
            let metadata = match fs::symlink_metadata(path) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Self::Missing,
                Err(error) => panic!("capture owned path {}: {error}", path.display()),
            };
            if metadata.file_type().is_symlink() { return Self::Link(fs::read_link(path).unwrap()); }
            if metadata.is_dir() {
                let mut entries = fs::read_dir(path).unwrap().map(|entry| {
                    let entry = entry.unwrap();
                    (entry.file_name(), Self::read(&entry.path()))
                }).collect::<Vec<_>>();
                entries.sort_by(|left, right| left.0.cmp(&right.0));
                Self::Directory(entries)
            } else {
                assert!(metadata.is_file(), "fixture requires a regular file at {}", path.display());
                Self::File(fs::read(path).unwrap())
            }
        }
    }
    struct FileSnapshot(Vec<(PathBuf, PathImage)>);
    impl FileSnapshot {
        fn capture(paths: Vec<PathBuf>) -> Self {
            Self(paths.into_iter().map(|path| { let image = PathImage::read(&path); (path, image) }).collect())
        }
        fn assert_unchanged(&self) {
            for (path, image) in &self.0 {
                assert_eq!(&PathImage::read(path), image, "checked delivery changed {}", path.display());
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    enum Fault {
        InvalidJson, InvalidUtf8, MissingConfig, ConfigDirectory, AutomationRootFile,
        UnsafeAgent, AgentDirectoryFile, AgentsRootFile, MissingDatabase, DeletionPending,
        UnknownHistoryStatus, CrossRowAlias, InvalidHistoryTail, InvalidHistoryUtf8,
    }
    struct InjectedFault {
        fault: Fault,
        displaced: Option<(PathBuf, PathBuf)>,
        snapshot: FileSnapshot,
    }
    impl InjectedFault {
        fn install(fixture: &Fixture, fault: Fault) -> Self {
            let mut displaced = None;
            let mut paths = Vec::new();
            match fault {
                Fault::InvalidJson => fs::write(&fixture.config, b"{invalid-definition").unwrap(),
                Fault::InvalidUtf8 => fs::write(&fixture.config, [0xff, 0xfe, 0x80]).unwrap(),
                Fault::MissingConfig => fs::remove_file(&fixture.config).unwrap(),
                Fault::ConfigDirectory => {
                    fs::remove_file(&fixture.config).unwrap();
                    fs::create_dir(&fixture.config).unwrap();
                    fs::write(fixture.config.join("read-failure-sentinel"), b"preserve directory contents").unwrap();
                }
                Fault::AutomationRootFile | Fault::AgentDirectoryFile | Fault::AgentsRootFile | Fault::MissingDatabase => {
                    let from = match fault {
                        Fault::AutomationRootFile => fixture.automations.clone(),
                        Fault::AgentDirectoryFile => fixture.database.parent().unwrap().to_path_buf(),
                        Fault::AgentsRootFile => fixture.root.join("agents"),
                        _ => fixture.database.clone(),
                    };
                    let to = fixture.root.join("displaced-authority");
                    fs::rename(&from, &to).unwrap();
                    if !matches!(fault, Fault::MissingDatabase) {
                        fs::write(&from, b"existing non-directory is not checked absence").unwrap();
                    }
                    paths.push(from.clone());
                    displaced = Some((from, to));
                }
                Fault::DeletionPending => {
                    let owners = fixture.open();
                    owners.workers.begin_agent_delete(&fixture.agent_id).unwrap();
                }
                Fault::UnsafeAgent => {}
                Fault::UnknownHistoryStatus | Fault::CrossRowAlias | Fault::InvalidHistoryTail => {
                    let mut rows: Vec<Value> = serde_json::from_slice(&fixture.history_bytes).unwrap();
                    match fault {
                        Fault::UnknownHistoryStatus => rows[0]["status"] = json!("future_outcome_unknown"),
                        Fault::CrossRowAlias => {
                            // The conflict is unrelated to ORIGINAL/ALIAS. A new
                            // queried UUID must not turn global ambiguity into absence.
                            rows.push(json!({"id":"other-owner-a","startedAt":200,"status":"running",
                                "coalescedRunIds":["shared-foreign-alias"]}));
                            rows.push(json!({"id":"other-owner-b","startedAt":201,"status":"running",
                                "coalescedRunIds":["shared-foreign-alias"]}));
                        }
                        _ => {
                            for index in 0..=AUTOMATION_MAX_RUN_HISTORY {
                                rows.push(json!({"id":format!("valid-terminal-{index}"),
                                    "startedAt":1000 + index, "status":"ok", "finishedAt":2000 + index}));
                            }
                            rows.push(json!({"id":"bad-tail","startedAt":1,"status":"ok"}));
                        }
                    }
                    fs::write(&fixture.history, serde_json::to_vec_pretty(&rows).unwrap()).unwrap();
                }
                Fault::InvalidHistoryUtf8 => fs::write(&fixture.history, [0xff, 0x80]).unwrap(),
            }
            let retained_path = |path: &Path| {
                if let Some((from, to)) = &displaced {
                    if let Ok(relative) = path.strip_prefix(from) {
                        return if relative.as_os_str().is_empty() { to.clone() } else { to.join(relative) };
                    }
                }
                path.to_path_buf()
            };
            paths.extend([retained_path(&fixture.config), retained_path(&fixture.history)]);
            if matches!(fault, Fault::AgentDirectoryFile | Fault::AgentsRootFile | Fault::MissingDatabase) {
                // These owners are closed and the actual DB is deliberately
                // displaced. Preserve its exact bytes as well as checking cron
                // rows again after repair. Normal owner opening may change DB
                // internals and is intentionally not a zero-open oracle.
                paths.push(retained_path(&fixture.database));
            }
            let snapshot = FileSnapshot::capture(paths);
            Self { fault, displaced, snapshot }
        }
        fn cron_readable(&self) -> bool {
            !matches!(self.fault, Fault::AgentDirectoryFile | Fault::AgentsRootFile | Fault::MissingDatabase)
        }
        fn fires(&self, fixture: &Fixture) -> Vec<Value> {
            let mut fires = fixture.retained_fires();
            if matches!(self.fault, Fault::UnsafeAgent) {
                for fire in &mut fires { fire["sandAgentId"] = json!("../invalid-agent"); }
            }
            if matches!(self.fault, Fault::UnknownHistoryStatus | Fault::CrossRowAlias | Fault::InvalidHistoryTail | Fault::InvalidHistoryUtf8) {
                fires.push(fixture.fire("checked-absent-history-query", &fixture.automation_id));
            }
            fires
        }
        fn repair(self, fixture: &Fixture) {
            if let Some((from, to)) = self.displaced {
                if !matches!(self.fault, Fault::MissingDatabase) { fs::remove_file(&from).unwrap(); }
                fs::rename(to, from).unwrap();
            } else {
                match self.fault {
                    Fault::InvalidJson | Fault::InvalidUtf8 | Fault::MissingConfig => fs::write(&fixture.config, &fixture.config_bytes).unwrap(),
                    Fault::ConfigDirectory => {
                        fs::remove_dir_all(&fixture.config).unwrap();
                        fs::write(&fixture.config, &fixture.config_bytes).unwrap();
                    }
                    Fault::DeletionPending => {
                        let owners = fixture.open();
                        owners.workers.end_agent_delete(&fixture.agent_id).unwrap();
                    }
                    Fault::UnknownHistoryStatus | Fault::CrossRowAlias | Fault::InvalidHistoryTail | Fault::InvalidHistoryUtf8 => {
                        fs::write(&fixture.history, &fixture.history_bytes).unwrap();
                    }
                    _ => {}
                }
            }
        }
    }

    struct NoNotifyConnection;
    impl NotifyBusExperimentsApi for NoNotifyConnection {
        fn check_feature_gate(&self, _: &str) -> bool { false }
        fn subscribe(&self, _: Arc<dyn Fn() + Send + Sync>) -> StopSubscription { Box::new(|| {}) }
    }
    struct Transport {
        events: Vec<Value>,
        calls: Mutex<Vec<(String, Value)>>,
        poll_count: AtomicUsize,
        polls: mpsc::Sender<(usize, Value)>,
        evidence: PathBuf,
    }
    impl AutomationsBackendTransport for Transport {
        fn post_json(&self, path: &str, body: &Value) -> Result<Value, AutomationsBackendError> {
            {
                let mut calls = self.calls.lock().unwrap();
                calls.push((path.into(), body.clone()));
                fs::write(&self.evidence, serde_json::to_vec_pretty(&*calls).unwrap()).unwrap();
            }
            match path {
                "/sand/listener-subscriptions" => Ok(json!({"slack":{"status":"ok"},"github":{"status":"ok"}})),
                "/sand/listener-events/poll" => Ok(json!({"events":[]})),
                "/sand/automation-events/poll" => {
                    let poll = self.poll_count.fetch_add(1, Ordering::SeqCst) + 1;
                    self.polls.send((poll, body.clone())).expect("owned poll observer");
                    Ok(json!({"events":self.events}))
                }
                "/sand/automation-runs/complete" => Ok(json!({})),
                _ => Err(AutomationsBackendError::Transport(format!("unexpected checked-backend request {path}"))),
            }
        }
    }
    #[derive(Debug)]
    struct DispatchReceipt { poll: usize, id: String, accepted: bool }
    #[derive(Default)]
    struct Counts {
        launches: Mutex<Vec<(String, String, String)>>,
        executions: Mutex<Vec<String>>,
        completions: Mutex<Vec<(String, Option<FireCompletion>)>>,
        dropped: Mutex<Vec<(String, &'static str)>>,
        errors: Mutex<Vec<String>>,
    }
    struct Backend {
        runtime: Arc<ProductionAutomationsBackendRuntime>,
        notify: HostNotifyBusExtension,
        transport: Arc<Transport>,
        counts: Arc<Counts>,
        dispatches: mpsc::Receiver<DispatchReceipt>,
        polls: mpsc::Receiver<(usize, Value)>,
    }
    impl Backend {
        fn start(fixture: &Fixture, owners: &Owners, phase: &str, events: Vec<Value>) -> Self {
            let notify = start_notify_bus_extension_with_options(Arc::new(NoNotifyConnection),
                NotifyBusExtensionOptions {
                    get_backend_url: Arc::new(|| panic!("controlled poll cannot open notify transport")),
                    get_access_token: Arc::new(|_| panic!("controlled poll cannot read credentials")),
                    now_ms: Arc::new(|| 0), timing: NotifyBusTiming::default(), log: Arc::new(|_| {}),
                }).expect("actual notify owner without external connection readiness");
            let (poll_tx, polls) = mpsc::channel();
            let transport = Arc::new(Transport { events, calls: Mutex::new(Vec::new()),
                poll_count: AtomicUsize::new(0), polls: poll_tx,
                evidence: fixture.root.join(format!("{phase}-requests.json")),
            });
            let counts = Arc::new(Counts::default());
            let (dispatch_tx, dispatches) = mpsc::channel();
            let runtime = owners.runtime.clone();
            let dispatch_counts = Arc::clone(&counts);
            let poll_errors = Arc::clone(&counts);
            let dispatch_transport = Arc::clone(&transport);
            let trace_path = fixture.root.join(format!("{phase}-dispatches.txt"));
            let trace = Mutex::new(Vec::<String>::new());
            let backend = ProductionAutomationsBackendRuntime::start(
                transport.clone(), notify.clone(), Arc::new(|| (Vec::new(), Vec::new())), Arc::new(|_| false),
                Arc::new(move |fire, completion| {
                    let callback_counts = Arc::clone(&dispatch_counts);
                    let callback_id = fire.id.clone();
                    let callback: Arc<dyn Fn(Option<FireCompletion>) + Send + Sync> = Arc::new(move |terminal| {
                        callback_counts.completions.lock().unwrap().push((callback_id.clone(), terminal.clone()));
                        // Forward the actual production callback. Tests never
                        // implement complete/abandon/ACK transitions themselves.
                        completion(terminal);
                    });
                    let launch_callback = Arc::clone(&callback);
                    let accepted = crate::dispatch_checked_production_backend_fire(
                        &runtime, &fire, None, callback.as_ref(),
                        |reason| dispatch_counts.dropped.lock().unwrap().push((fire.id.clone(), reason)),
                        |prepared, target| {
                            dispatch_counts.launches.lock().unwrap().push((
                                fire.id.clone(), target.agent_id.clone(), target.automation.id.clone(),
                            ));
                            let PreparedBackendFire::Schedule { agent_id, automation_id, run_uuid, scheduled_for_ms } = prepared else {
                                dispatch_counts.errors.lock().unwrap().push("non-schedule reached controlled schedule executor".into());
                                return false;
                            };
                            let result = runtime.run_server_scheduled_automation_with(
                                &agent_id, &automation_id, run_uuid.clone(), scheduled_for_ms.map(|value| value as f64), |_| {
                                    dispatch_counts.executions.lock().unwrap().push(run_uuid.clone());
                                    Ok(AutomationExecutionResult::Completed)
                                },
                            );
                            if let Err(error) = &result { dispatch_counts.errors.lock().unwrap().push(error.clone()); }
                            launch_callback(automation_fire_completion(result));
                            true
                        },
                    );
                    let receipt = DispatchReceipt {
                        poll: dispatch_transport.poll_count.load(Ordering::SeqCst), id: fire.id, accepted,
                    };
                    {
                        let mut trace = trace.lock().unwrap();
                        trace.push(format!("{receipt:?}"));
                        fs::write(&trace_path, trace.join("\n")).unwrap();
                    }
                    dispatch_tx.send(receipt).expect("owned shipping-dispatch observer");
                    accepted
                }),
                // External readiness is controlled so a broken display list
                // cannot prevent the backend from exercising target discovery.
                Arc::new(|| (true, true)),
                Arc::new(move |error| poll_errors.errors.lock().unwrap().push(error.to_string())),
                Arc::new(|_| {}),
            );
            Self { runtime: backend, notify, transport, counts, dispatches, polls }
        }
        fn wait_dispatches(&self, ids: &[&str], after: usize, accepted: bool) {
            let deadline = Instant::now() + WAIT;
            let mut seen = BTreeSet::new();
            while seen.len() < ids.len() {
                let receipt = self.dispatches.recv_timeout(deadline.saturating_duration_since(Instant::now()))
                    .expect("actual shipping dispatch receipt before bounded deadline");
                assert!(ids.contains(&receipt.id.as_str()), "unexpected dispatch: {receipt:?}");
                assert_eq!(receipt.accepted, accepted, "shipping dispatch disposition: {receipt:?}");
                if receipt.poll > after { seen.insert(receipt.id); }
            }
        }
        fn repeated_refusal(&self, ids: &[&str]) {
            self.wait_dispatches(ids, 0, false);
            for _ in 0..2 {
                let before = self.transport.poll_count.load(Ordering::SeqCst);
                self.runtime.request_reconcile();
                self.wait_dispatches(ids, before, false);
            }
            self.runtime.stop();
            assert!(self.counts.launches.lock().unwrap().is_empty(), "refusal cannot launch an execution worker");
            assert!(self.counts.executions.lock().unwrap().is_empty(), "refusal cannot enter the executor");
            assert!(self.counts.completions.lock().unwrap().is_empty(), "refusal cannot invoke even a None completion callback");
            assert!(self.counts.dropped.lock().unwrap().is_empty(), "unknown discovery is not a known dropped fire");
            assert!(self.counts.errors.lock().unwrap().is_empty());
            let calls = self.transport.calls.lock().unwrap();
            assert!(calls.iter().filter(|(path, _)| path == "/sand/automation-events/poll").count() >= 3);
            for (path, body) in calls.iter() {
                assert_ne!(path, "/sand/automation-runs/complete", "refusal cannot POST completion");
                if path == "/sand/automation-events/poll" { assert_eq!(body, &json!({"ackRunUuids":[]})); }
            }
        }
        fn completed_and_acked(&self, ids: &[&str], terminal: &FireCompletion, executions: usize) {
            self.wait_dispatches(ids, 0, true);
            // Poll twice after actual callbacks. This reaches the unchanged
            // Reported -> ACK consumer and proves reported UUIDs do not launch again.
            for _ in 0..2 {
                let before = self.transport.poll_count.load(Ordering::SeqCst);
                self.runtime.request_reconcile();
                let deadline = Instant::now() + WAIT;
                loop {
                    let (poll, body) = self.polls.recv_timeout(deadline.saturating_duration_since(Instant::now()))
                        .expect("actual post-completion ACK poll");
                    if poll <= before { continue; }
                    let ack = body["ackRunUuids"].as_array().unwrap().iter()
                        .map(|value| value.as_str().unwrap()).collect::<BTreeSet<_>>();
                    assert_eq!(ack, ids.iter().copied().collect::<BTreeSet<_>>());
                    break;
                }
            }
            self.runtime.stop();
            assert_eq!(self.counts.launches.lock().unwrap().len(), executions);
            assert_eq!(self.counts.executions.lock().unwrap().len(), executions);
            assert!(self.counts.errors.lock().unwrap().is_empty());
            let callbacks = self.counts.completions.lock().unwrap();
            assert_eq!(callbacks.len(), ids.len(), "one actual completion callback per backend UUID");
            for id in ids {
                assert_eq!(callbacks.iter().filter(|(run, value)| run.as_str() == *id && value.as_ref() == Some(terminal)).count(), 1);
            }
            let calls = self.transport.calls.lock().unwrap();
            let posts = calls.iter().filter(|(path, _)| path == "/sand/automation-runs/complete")
                .map(|(_, body)| body).collect::<Vec<_>>();
            assert_eq!(posts.len(), ids.len(), "one completion POST per accepted UUID");
            for id in ids {
                assert_eq!(posts.iter().filter(|body| **body == &json!({
                    "runUuid":id, "status":terminal.status, "errorMessage":terminal.error_message,
                })).count(), 1);
            }
        }
    }
    impl Drop for Backend {
        fn drop(&mut self) { self.runtime.stop(); self.notify.stop(); }
    }

    fn refusal_across_reopen(fixture: &Fixture, injected: &InjectedFault) {
        let events = injected.fires(fixture);
        let ids = events.iter().map(|event| event["id"].as_str().unwrap()).collect::<Vec<_>>();
        for phase in ["fresh", "reopened"] {
            let owners = fixture.open();
            if matches!(injected.fault, Fault::DeletionPending) {
                if phase == "fresh" { owners.workers.begin_agent_delete(&fixture.agent_id).unwrap(); }
                assert_eq!(owners.workers.is_agent_being_deleted(&fixture.agent_id), phase == "fresh",
                    "reopened refusal must reach the durable deletion fence");
            }
            let backend = Backend::start(fixture, &owners, phase, events.clone());
            backend.repeated_refusal(&ids);
            drop(backend);
            assert!(owners.observations.lock().unwrap().is_empty());
            assert!(!owners.runtime.run_path().is_in_flight(&fixture.agent_id, &fixture.automation_id));
            injected.snapshot.assert_unchanged();
            if injected.cron_readable() { fixture.assert_retained_cron(&owners); }
            drop(owners);
            injected.snapshot.assert_unchanged();
        }
    }

    #[test]
    fn backend_fire_checked_discovery_refuses_invalid_definitions_across_reopen() {
        bounded("tests::automation_terminal_custody_tests::checked_backend_delivery::backend_fire_checked_discovery_refuses_invalid_definitions_across_reopen", |root| {
            for fault in [Fault::InvalidJson, Fault::InvalidUtf8, Fault::MissingConfig, Fault::ConfigDirectory, Fault::AutomationRootFile] {
                let fixture = Fixture::create(root, &format!("{fault:?}"));
                let injected = InjectedFault::install(&fixture, fault);
                refusal_across_reopen(&fixture, &injected);
                injected.repair(&fixture);
                let owners = fixture.open();
                fixture.assert_retained_cron(&owners);
                assert_eq!(fs::read(&fixture.config).unwrap(), fixture.config_bytes);
                assert_eq!(fs::read(&fixture.history).unwrap(), fixture.history_bytes);
            }
        });
    }

    #[test]
    fn backend_fire_checked_agent_admission_refuses_unknown_state_across_reopen() {
        bounded("tests::automation_terminal_custody_tests::checked_backend_delivery::backend_fire_checked_agent_admission_refuses_unknown_state_across_reopen", |root| {
            for fault in [Fault::UnsafeAgent, Fault::AgentDirectoryFile, Fault::AgentsRootFile, Fault::MissingDatabase, Fault::DeletionPending] {
                let fixture = Fixture::create(root, &format!("{fault:?}"));
                let injected = InjectedFault::install(&fixture, fault);
                refusal_across_reopen(&fixture, &injected);
                injected.repair(&fixture);
                let owners = fixture.open();
                fixture.assert_retained_cron(&owners);
                assert_eq!(fs::read(&fixture.config).unwrap(), fixture.config_bytes);
                assert_eq!(fs::read(&fixture.history).unwrap(), fixture.history_bytes);
                let backend = Backend::start(&fixture, &owners, "agent-repaired-running", fixture.retained_fires());
                backend.repeated_refusal(&[ORIGINAL, ALIAS]);
                assert!(owners.observations.lock().unwrap().is_empty());
            }
        });
    }

    #[test]
    fn backend_fire_checked_history_refuses_malformed_custody_across_reopen() {
        bounded("tests::automation_terminal_custody_tests::checked_backend_delivery::backend_fire_checked_history_refuses_malformed_custody_across_reopen", |root| {
            for fault in [Fault::UnknownHistoryStatus, Fault::CrossRowAlias, Fault::InvalidHistoryTail, Fault::InvalidHistoryUtf8] {
                let fixture = Fixture::create(root, &format!("{fault:?}"));
                let injected = InjectedFault::install(&fixture, fault);
                refusal_across_reopen(&fixture, &injected);
                injected.repair(&fixture);
                let owners = fixture.open();
                let backend = Backend::start(&fixture, &owners, "history-repaired-running", fixture.retained_fires());
                backend.repeated_refusal(&[ORIGINAL, ALIAS]);
                fixture.assert_retained_cron(&owners);
                assert_eq!(fs::read(&fixture.history).unwrap(), fixture.history_bytes);
                assert_eq!(fs::read(&fixture.config).unwrap(), fixture.config_bytes);
                assert!(owners.observations.lock().unwrap().is_empty());
            }
        });
    }

    #[test]
    fn backend_fire_checked_discovery_repairs_running_terminal_and_new_fire() {
        bounded("tests::automation_terminal_custody_tests::checked_backend_delivery::backend_fire_checked_discovery_repairs_running_terminal_and_new_fire", |root| {
            let fixture = Fixture::create(root, "repair-and-positive-controls");
            let injected = InjectedFault::install(&fixture, Fault::InvalidJson);
            refusal_across_reopen(&fixture, &injected);

            // A malformed unrelated definition must not poison another exact
            // target in the same automation directory. This runs the real
            // Runtime from the helper's launch closure and settles its own row.
            let owners = fixture.open();
            let other = Backend::start(&fixture, &owners, "unrelated-valid", vec![fixture.fire("unrelated-new-fire", &fixture.unrelated_id)]);
            other.completed_and_acked(&["unrelated-new-fire"], &FireCompletion::succeeded(), 1);
            assert_eq!(*other.counts.launches.lock().unwrap(), vec![("unrelated-new-fire".into(), fixture.agent_id.clone(), fixture.unrelated_id.clone())]);
            assert!(other.counts.dropped.lock().unwrap().is_empty());
            drop(other);
            assert_eq!(owners.observations.lock().unwrap().len(), 1);
            injected.snapshot.assert_unchanged();
            fixture.assert_retained_cron(&owners);
            drop(owners);

            injected.repair(&fixture);
            let owners = fixture.open();
            let repaired = Backend::start(&fixture, &owners, "definition-repaired-running", fixture.retained_fires());
            repaired.repeated_refusal(&[ORIGINAL, ALIAS]);
            drop(repaired);
            assert!(owners.observations.lock().unwrap().is_empty());
            fixture.assert_retained_cron(&owners);
            assert_eq!(fs::read(&fixture.history).unwrap(), fixture.history_bytes);
            assert_eq!(fs::read(&fixture.config).unwrap(), fixture.config_bytes);

            // Explicit fixture settlement through the existing owners is the
            // only authority change. It is not automatic unknown-effect recovery.
            let sessions = owners.sessions();
            let store = sessions.automation_store_for(&fixture.agent_id).unwrap();
            let detail = "explicit checked-delivery fixture terminal";
            assert!(owners.workers.finish_cron_session_run(
                &fixture.agent_id, &fixture.automation_id, ORIGINAL, "error", 200.0, Some(detail),
            ).unwrap());
            sessions.with_admitted_filesystem_mutation(&fixture.agent_id, || {
                store.finish_run(&fixture.automation_id, ORIGINAL, "error", 200.0, Some(detail))
                    .map_err(|error| error.to_string())?.ok_or_else(|| "explicit original settlement missing".to_string())?;
                Ok(())
            }).unwrap();
            let terminal_files = FileSnapshot::capture(vec![fixture.config.clone(), fixture.history.clone()]);
            let terminal_cron = owners.workers.list_cron_session_runs(&fixture.agent_id).unwrap();
            let terminal = Backend::start(&fixture, &owners, "terminal-error-replay", fixture.retained_fires());
            terminal.completed_and_acked(&[ORIGINAL, ALIAS], &FireCompletion::failed(detail), 0);
            assert!(terminal.counts.dropped.lock().unwrap().is_empty(), "existing terminal is not dropped-fire telemetry");
            drop(terminal);
            terminal_files.assert_unchanged();
            assert_eq!(owners.workers.list_cron_session_runs(&fixture.agent_id).unwrap(), terminal_cron);
            assert!(owners.observations.lock().unwrap().is_empty());

            sessions.set_agent_automation_enabled(&fixture.agent_id, &fixture.automation_id, true).unwrap();
            let new = Backend::start(&fixture, &owners, "distinct-new-fire", vec![fixture.fire("distinct-valid-fire", &fixture.automation_id)]);
            new.completed_and_acked(&["distinct-valid-fire"], &FireCompletion::succeeded(), 1);
            assert_eq!(*new.counts.executions.lock().unwrap(), vec!["distinct-valid-fire".to_string()]);
            assert_eq!(*new.counts.launches.lock().unwrap(), vec![("distinct-valid-fire".into(), fixture.agent_id.clone(), fixture.automation_id.clone())]);
            assert!(new.counts.dropped.lock().unwrap().is_empty());
            drop(new);
            assert_eq!(owners.observations.lock().unwrap().len(), 1);
            let new_history = owners.runtime.read_run_for_delivery(&fixture.agent_id, &fixture.automation_id, "distinct-valid-fire").unwrap().unwrap();
            assert_eq!(new_history.status, "ok");
            assert!(new_history.finished_at.is_some());
            let settled_files = FileSnapshot::capture(vec![fixture.config.clone(), fixture.history.clone()]);
            let settled_cron = owners.workers.list_cron_session_runs(&fixture.agent_id).unwrap();
            drop(store);
            drop(sessions);
            drop(owners);

            let reopened = fixture.open();
            let success = Backend::start(&fixture, &reopened, "reopened-success-replay", vec![fixture.fire("distinct-valid-fire", &fixture.automation_id)]);
            success.completed_and_acked(&["distinct-valid-fire"], &FireCompletion::succeeded(), 0);
            assert!(success.counts.dropped.lock().unwrap().is_empty());
            drop(success);
            settled_files.assert_unchanged();
            assert_eq!(reopened.workers.list_cron_session_runs(&fixture.agent_id).unwrap(), settled_cron);
            assert!(reopened.observations.lock().unwrap().is_empty());

            // Only verified absence retains the documented missing-target
            // callback. Check an absent local folder and a genuinely absent
            // safe agent while the unrelated valid automation remains present.
            let absent_agent = uuid::Uuid::new_v4().to_string();
            let mut missing_agent = fixture.fire("genuine-missing-agent", &fixture.automation_id);
            missing_agent["sandAgentId"] = json!(absent_agent);
            missing_agent["automationId"] = json!(stable_automation_id(&absent_agent, &fixture.automation_id));
            let missing = Backend::start(&fixture, &reopened, "genuine-missing-targets", vec![
                fixture.fire("genuine-missing-target", "never-created-automation"), missing_agent,
            ]);
            missing.completed_and_acked(&["genuine-missing-target", "genuine-missing-agent"],
                &FireCompletion::failed("Automation not found on the Sand box"), 0);
            let dropped = missing.counts.dropped.lock().unwrap().clone();
            assert_eq!(dropped.len(), 2);
            for id in ["genuine-missing-target", "genuine-missing-agent"] {
                assert_eq!(dropped.iter().filter(|(run, reason)| run.as_str() == id && *reason == "automation_missing").count(), 1);
            }
            drop(missing);
            settled_files.assert_unchanged();
            assert_eq!(reopened.workers.list_cron_session_runs(&fixture.agent_id).unwrap(), settled_cron);
            assert!(reopened.observations.lock().unwrap().is_empty());
            assert!(!fixture.root.join("agents").join(absent_agent).exists(), "checked absence cannot materialize an agent");
            assert!(!fixture.automations.join("never-created-automation").exists());
            assert!(reopened.runtime.find_automation_for_backend_delivery(
                &fixture.agent_id, &stable_automation_id(&fixture.agent_id, &fixture.unrelated_id),
            ).unwrap().is_some());
        });
    }
}

