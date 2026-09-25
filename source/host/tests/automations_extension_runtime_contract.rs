use mahayana_host_runtime::automations::automation_trigger::parse_stored_trigger;
use mahayana_host_runtime::extensions::automations::backend_relay_source::BackendRelaySource;
use mahayana_host_runtime::extensions::automations::extension::AutomationExtensionRuntime;
use mahayana_host_runtime::extensions::automations::sand_automation_fire_consumer::{
    AutomationFireEnvelope, SandAutomationFireConsumer,
};
use mahayana_host_runtime::extensions::automations::sand_trigger_hub::ScheduledAutomation;
use serde_json::json;

#[test]
fn backend_relay_source_only_emits_started_matching_kind() {
    let listener=parse_stored_trigger(&json!({
        "type":"slack","channel":"#eng","match":{"kind":"keyword","keyword":"deploy"}
    })).unwrap();
    let mut source=BackendRelaySource::new("slack");
    source.set_listeners(vec![listener]);
    assert!(!source.accepts(&json!({"source":"slack","channel":"eng","text":"deploy"}),false,false));
    source.start();
    assert!(source.accepts(&json!({"source":"slack","channel":"eng","text":"DEPLOY"}),false,false));
    assert!(!source.accepts(&json!({"source":"github","channel":"eng","text":"deploy"}),false,false));
    source.stop();
    assert!(!source.accepts(&json!({"source":"slack","channel":"eng","text":"deploy"}),false,false));
}

#[test]
fn fire_consumer_serializes_queue_and_isolates_failures() {
    let mut consumer=SandAutomationFireConsumer::default();
    consumer.enqueue(AutomationFireEnvelope{
        agent_id:"a".into(),automation_id:"one".into(),event:json!({"source":"slack"})
    });
    consumer.enqueue(AutomationFireEnvelope{
        agent_id:"b".into(),automation_id:"two".into(),event:json!({"source":"slack"})
    });
    let mut seen=Vec::new();
    let failures=consumer.drain(|fire|{
        seen.push(fire.automation_id.clone());
        if fire.automation_id=="one"{Err("boom".into())}else{Ok(())}
    });
    assert_eq!(seen,vec!["one","two"]);
    assert_eq!(failures.len(),1);
    assert_eq!(failures[0].automation_id,"one");
    assert!(!consumer.is_firing());
    assert_eq!(consumer.pending_len(),0);
}

#[test]
fn extension_reconciles_sources_and_queues_real_fire_intents() {
    let scheduled=vec![ScheduledAutomation{
        agent_id:"agent".into(),
        automation_id:"routine".into(),
        is_enabled:true,
        trigger:parse_stored_trigger(&json!({
            "type":"slack","channel":"#eng","match":{"kind":"keyword","keyword":"deploy"}
        })).unwrap(),
    }];
    let mut runtime=AutomationExtensionRuntime::with_source_kinds(vec!["slack".into(),"github".into()]);
    runtime.reconcile(&scheduled,true,|_,_|true);
    assert!(runtime.source("slack").unwrap().is_started());
    assert!(!runtime.source("github").unwrap().is_started());

    assert_eq!(runtime.ingest_event(
        &scheduled,
        json!({"source":"slack","channel":"eng","text":"deploy prod"}),
        true,false,|_,_|true
    ),1);
    let mut fired=Vec::new();
    assert!(runtime.drain_fires(|envelope|{
        fired.push((envelope.agent_id.clone(),envelope.automation_id.clone()));
        Ok(())
    }).is_empty());
    assert_eq!(fired,vec![("agent".into(),"routine".into())]);

    runtime.stop();
    assert_eq!(runtime.ingest_event(
        &scheduled,
        json!({"source":"slack","channel":"eng","text":"deploy"}),
        true,false,|_,_|true
    ),0);
}
