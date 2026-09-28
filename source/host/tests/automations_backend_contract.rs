use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use mahayana_host_runtime::extensions::automations::backend_relay_source::{
    BackendRelayRuntime, map_relay_wire_event, relay_subscription_request,
};
use mahayana_host_runtime::extensions::automations::backend_transport::{
    AutomationsBackendError, AutomationsBackendTransport,
};
use mahayana_host_runtime::extensions::automations::sand_automation_fire_consumer::{
    AutomationFireBackendRuntime, FireCompletion,
};
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
            Some(FireCompletion::succeeded())
        })
        .expect("fire tick");
    assert_eq!(count, 1);
    assert_eq!(runtime.pending_len(), 1);

    runtime.tick(101, true, true, |_| None).expect("ack fire tick");
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
fn fire_runtime_treats_completion_conflict_as_reported_and_honors_next_poll_delay() {
    let transport = MockTransport::with_responses(vec![
        Ok(json!({"events":[{
            "id":"run-1",
            "sandAgentId":"agent-1",
            "automationId":"cloud-1",
            "timestampMs":10
        }]})),
        Err(AutomationsBackendError::Status {
            path:"/sand/automation-runs/complete".into(),
            status:409,
        }),
        Ok(json!({"events":[]})),
        Ok(json!({"events":[],"nextPollAfterMs":5000})),
    ]);
    let mut runtime = AutomationFireBackendRuntime::new(transport.clone());
    runtime
        .tick(100, true, true, |_| Some(FireCompletion::failed("boom")))
        .expect("delivery tick");
    runtime.tick(101, true, true, |_| None).expect("ack tick");
    runtime.tick(102, true, true, |_| None).expect("empty tick");
    let before = transport.calls().len();
    runtime.tick(103, true, true, |_| None).expect("delayed tick");
    assert_eq!(transport.calls().len(), before);
}
