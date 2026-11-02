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
        if upgrade { fixture.registry.request_quiesce_for_upgrade(); }
        if kind == "cancelled" {
            assert!(fixture.registry.cancel_stream(stream_id, "safe terminal cancellation"));
        }
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
                let prepared = prepare_backend_fire(&entries, &fire, |_| None);
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

