use std::fs;
use std::sync::{Arc,Mutex};
use std::sync::atomic::{AtomicUsize,Ordering};
use std::time::{SystemTime,UNIX_EPOCH};
use mahayana_host_runtime::agents::agent_messaging::AgentMessageImage;
use mahayana_host_runtime::agents::agent_profile::SandAgentProfile;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::telemetry::agent_error_telemetry::agent_error_telemetry;
use mahayana_host_runtime::extensions::transcript::agent_to_agent_messaging::{
    AgentWakeRequest, ProductionAgentToAgentMessaging, agent_inbound_failure_report,
    agent_inbound_failure_tray, should_interrupt_priority_peer,
};
use mahayana_host_runtime::extensions::transcript::run_scheduler::RunLane;

fn temp_root()->std::path::PathBuf{
    let suffix=SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
    std::env::temp_dir().join(format!("fabushi-agent-to-agent-{}-{suffix}",std::process::id()))
}
fn profile(name:&str)->SandAgentProfile{SandAgentProfile{name:name.into(),description:String::new(),title:String::new(),avatar_shape:String::new(),avatar_color:String::new()}}

#[test]
fn direct_delivery_writes_both_transcripts_partners_activity_and_priority_interrupt(){
    let root=temp_root();let sessions=Arc::new(ProductionSessionWorkers::with_agents_root(&root,500));
    let alpha=sessions.materialize_new_session(Some(&profile("Alpha")),"user",None).expect("alpha");
    let beta=sessions.materialize_new_session(Some(&profile("Beta")),"user",None).expect("beta");
    let wakes=Arc::new(Mutex::new(Vec::<AgentWakeRequest>::new()));let interrupts=Arc::new(AtomicUsize::new(0));
    let service=ProductionAgentToAgentMessaging::new(Arc::clone(&sessions),{
        let wakes=Arc::clone(&wakes);Arc::new(move|wake|wakes.lock().expect("wakes").push(wake.clone()))
    },Some({let interrupts=Arc::clone(&interrupts);Arc::new(move|_,_|{interrupts.fetch_add(1,Ordering::SeqCst);1})}));
    let ack=service.send_to_agent(&alpha.id,&beta.id,"  please review  ",&[AgentMessageImage{url:"https://example.com/chart.png".into(),alt:Some("chart".into())}],true).expect("send");
    assert!(ack.contains("priority"));
    let a=sessions.read_agent_transcript_entries(&alpha.id).expect("alpha transcript");
    let b=sessions.read_agent_transcript_entries(&beta.id).expect("beta transcript");
    assert_eq!(a.len(),1);assert_eq!(a[0]["toAgent"]["id"],beta.id);assert_eq!(a[0]["content"],"please review");
    assert_eq!(b.len(),1);assert_eq!(b[0]["fromAgent"]["id"],alpha.id);assert_eq!(b[0]["images"][0]["url"],"https://example.com/chart.png");
    assert_eq!(sessions.get_agent_conversation_partner_ids(&alpha.id).expect("a partners"),vec![beta.id.clone()]);
    assert_eq!(sessions.get_agent_conversation_partner_ids(&beta.id).expect("b partners"),vec![alpha.id.clone()]);
    assert_eq!(interrupts.load(Ordering::SeqCst),1);
    let wake=wakes.lock().expect("wakes").last().cloned().expect("wake");assert_eq!(wake.agent_id,beta.id);assert!(wake.priority);assert!(wake.prompt.starts_with("[agent]"));
    sessions.shutdown();let _=fs::remove_dir_all(root);
}

#[test]
fn priority_peer_preemption_never_interrupts_an_active_user_lane() {
    assert!(!should_interrupt_priority_peer(Some(RunLane::User)));
    assert!(should_interrupt_priority_peer(Some(RunLane::Agent)));
    assert!(should_interrupt_priority_peer(Some(RunLane::Background)));
    assert!(should_interrupt_priority_peer(None));
}



#[test]
fn direct_peer_wake_materializes_file_url_images_for_runner_selected_input() {
    let root = temp_root();
    fs::create_dir_all(&root).expect("root");
    let image_path = root.join("peer.png");
    fs::write(&image_path, [1_u8, 2, 3, 4]).expect("image");
    let image_url = url::Url::from_file_path(&image_path)
        .expect("file url")
        .to_string();

    let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(&root, 500));
    let alpha = sessions
        .materialize_new_session(Some(&profile("Alpha")), "user", None)
        .expect("alpha");
    let beta = sessions
        .materialize_new_session(Some(&profile("Beta")), "user", None)
        .expect("beta");
    let wakes = Arc::new(Mutex::new(Vec::<AgentWakeRequest>::new()));
    let service = ProductionAgentToAgentMessaging::new(
        Arc::clone(&sessions),
        {
            let wakes = Arc::clone(&wakes);
            Arc::new(move |wake| wakes.lock().expect("wakes").push(wake.clone()))
        },
        None,
    );

    service
        .send_to_agent(
            &alpha.id,
            &beta.id,
            "inspect this",
            &[AgentMessageImage {
                url: image_url,
                alt: Some("peer image".into()),
            }],
            false,
        )
        .expect("send");

    let wake = wakes.lock().expect("wakes").last().cloned().expect("wake");
    assert_eq!(wake.selected_images.len(), 1);
    assert_eq!(
        wake.selected_images[0]["path"],
        image_path.to_string_lossy().as_ref()
    );
    assert_eq!(wake.selected_images[0]["mimeType"], "image/png");
    assert_eq!(wake.selected_images[0]["data"], serde_json::json!([1, 2, 3, 4]));

    sessions.shutdown();
    let _ = fs::remove_dir_all(root);
}


#[test]
fn priority_wake_is_enqueued_before_peer_steering_interrupt() {
    let root = temp_root();
    let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(&root, 500));
    let alpha = sessions
        .materialize_new_session(Some(&profile("Alpha")), "user", None)
        .expect("alpha");
    let beta = sessions
        .materialize_new_session(Some(&profile("Beta")), "user", None)
        .expect("beta");
    let order = Arc::new(Mutex::new(Vec::<&'static str>::new()));

    let service = ProductionAgentToAgentMessaging::new(
        Arc::clone(&sessions),
        {
            let order = Arc::clone(&order);
            Arc::new(move |_| order.lock().expect("order").push("wake"))
        },
        Some({
            let order = Arc::clone(&order);
            Arc::new(move |_, _| {
                order.lock().expect("order").push("interrupt");
                1
            })
        }),
    );

    service
        .send_to_agent(&alpha.id, &beta.id, "urgent", &[], true)
        .expect("send");
    assert_eq!(*order.lock().expect("order"), vec!["wake", "interrupt"]);

    sessions.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shipping_priority_peer_steering_cancels_direct_and_group_member_runners() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let main = fs::read_to_string(manifest_dir.join("app/src/main.rs")).expect("shipping main");
    assert!(main.contains("priority_registry.cancel_agent(target_agent_id, reason)"));
    assert!(main.contains(
        "priority_registry.preempt_group_member_agent(target_agent_id, reason)"
    ));
    assert!(main.contains("priority_runtime.is_agent_running(target_agent_id)"));
    assert!(main.contains("reason: \"agent_steer\".into()"));
    assert!(main.contains("priority_telemetry.report_turn_interrupt(&TurnInterruptFields"));
}


#[test]
fn direct_delivery_records_frozen_product_analytics_fields() {
    let root = temp_root();
    let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(&root, 500));
    let alpha = sessions
        .materialize_new_session(Some(&profile("Alpha")), "user", None)
        .expect("alpha");
    let beta = sessions
        .materialize_new_session(Some(&profile("Beta")), "user", None)
        .expect("beta");
    let events = Arc::new(Mutex::new(Vec::<(String, String, bool)>::new()));
    let captured = Arc::clone(&events);
    let service = ProductionAgentToAgentMessaging::new(
        Arc::clone(&sessions),
        Arc::new(|_| {}),
        None,
    )
    .with_analytics(Arc::new(move |from, to, priority| {
        captured
            .lock()
            .expect("analytics")
            .push((from.to_string(), to.to_string(), priority));
    }));

    service
        .send_to_agent(&alpha.id, &beta.id, "hello", &[], true)
        .expect("send");
    assert_eq!(
        *events.lock().expect("analytics"),
        vec![(alpha.id.clone(), beta.id.clone(), true)]
    );

    sessions.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shipping_agent_message_analytics_uses_canonical_product_analytics_owner() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let main = fs::read_to_string(manifest_dir.join("app/src/main.rs")).expect("shipping main");
    assert!(main.contains(".with_analytics(Arc::new(move |from_agent_id, to_agent_id, is_priority|"));
    assert!(main.contains("\"sand.agent_message.sent\""));
    assert!(main.contains("\"from_agent_id\": from_agent_id"));
    assert!(main.contains("\"to_agent_id\": to_agent_id"));
    assert!(main.contains("\"is_group_target\": false"));
    assert!(main.contains("\"is_priority\": is_priority"));
}


#[test]
fn agent_inbound_failure_projects_frozen_telemetry_and_tray() {
    let report = agent_inbound_failure_report(
        "agent-b",
        Some("request-1"),
        "INFERENCE_PROVIDER_FAILED",
        "provider exploded",
    );
    assert_eq!(report.source, "agent");
    assert_eq!(report.conversation_id, "agent-b");
    assert_eq!(report.request_id.as_deref(), Some("request-1"));
    assert_eq!(report.error.code, "SAND-E0406");
    assert_eq!(
        report.detail.as_ref().map(|detail| detail.message.as_str()),
        Some("provider exploded")
    );
    let telemetry = agent_error_telemetry(&report);
    assert_eq!(telemetry.metadata["error_code"], "SAND-E0406");
    assert_eq!(telemetry.metadata["connect_code"], "INFERENCE_PROVIDER_FAILED");

    let tray = agent_inbound_failure_tray(
        "agent-b",
        Some("request-1"),
        "INFERENCE_PROVIDER_FAILED",
        "provider exploded",
    );
    assert_eq!(tray.agent_id.as_deref(), Some("agent-b"));
    assert_eq!(tray.title, "Message from another agent failed");
    assert_eq!(tray.detail, "provider exploded");
    assert_eq!(tray.request_id.as_deref(), Some("request-1"));
    assert_eq!(tray.error_kind.as_deref(), Some("INFERENCE_PROVIDER_FAILED"));
}

#[test]
fn shipping_host_owns_agent_inbound_failure_telemetry_and_tray() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let main = fs::read_to_string(manifest_dir.join("app/src/main.rs")).expect("shipping main");
    assert!(main.contains("method == \"reportAgentInboundFailure\""));
    assert!(main.contains("self.telemetry_logs.report_agent_error(&agent_inbound_failure_report("));
    assert!(main.contains("self.trays.push_error(agent_inbound_failure_tray("));
}


#[test]
fn deletion_fenced_target_uses_frozen_agent_gone_response() {
    let root = temp_root();
    let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(&root, 500));
    let alpha = sessions
        .materialize_new_session(Some(&profile("Alpha")), "user", None)
        .expect("alpha");
    let beta = sessions
        .materialize_new_session(Some(&profile("Beta")), "user", None)
        .expect("beta");
    sessions.begin_agent_delete(&beta.id);

    let service = ProductionAgentToAgentMessaging::new(
        Arc::clone(&sessions),
        Arc::new(|_| panic!("deleting target must not wake")),
        Some(Arc::new(|_, _| panic!("deleting target must not interrupt"))),
    );
    let ack = service
        .send_to_agent(&alpha.id, &beta.id, "hello", &[], false)
        .expect("send result");
    assert_eq!(ack, "That agent no longer exists.");

    sessions.end_agent_delete(&beta.id);
    sessions.shutdown();
    let _ = fs::remove_dir_all(root);
}
