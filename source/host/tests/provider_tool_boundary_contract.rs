use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::sync::Mutex;
use mahayana_host_runtime::extensions::inference::codex_direct_responses::{
    CodexDirectCheckpoint, CodexDirectError, CodexDirectOptions, CodexDirectTool,
    CodexDirectTransport, run_codex_direct_responses_with_lifecycle,
};
use mahayana_host_runtime::extensions::inference::provider_session::{
    OpenRouterCheckpoint, OpenRouterTransport, ProviderMessage, ProviderSessionError,
    RoutedToolDefinition, run_openrouter_with_transport,
};
use mahayana_host_runtime::extensions::inference::cursor_inference_transport::{
    CursorCheckpoint, CursorCheckpointMessage, CursorInferenceEvent,
    CursorInferenceStreamTransport, run_cursor_with_transport_reporting_usage,
};
use serde_json::{Value, json};

#[derive(Default)]
struct Transport { responses: VecDeque<Vec<Value>>, requests: usize }
impl CodexDirectTransport for Transport {
    fn stream_response(&mut self, _: &Value, event: &mut dyn FnMut(Value)->Result<(),CodexDirectError>, _: &dyn Fn()->bool)->Result<(),CodexDirectError> {
        self.requests += 1;
        for value in self.responses.pop_front().expect("unexpected model request") { event(value)?; }
        Ok(())
    }
}
impl OpenRouterTransport for Transport {
    fn stream_response(&mut self, _: &Value, event: &mut dyn FnMut(Value)->Result<(),ProviderSessionError>, _: &dyn Fn()->bool)->Result<(),ProviderSessionError> {
        self.requests += 1;
        for value in self.responses.pop_front().expect("unexpected model request") { event(value)?; }
        Ok(())
    }
}
fn tools() -> Vec<RoutedToolDefinition> {
    vec![RoutedToolDefinition { name:"write".into(),tool_name:"write".into(),
        provider_identifier:"fixture".into(),description:None,input_schema:json!({"type":"object"}) }]
}

#[test]
fn responses_intent_is_durable_before_effect_and_uncertain_recovery_never_replays() {
    for fail_before in [true,false] {
        let mut transport = Transport { responses:VecDeque::from([vec![json!({"type":"response.completed","response":{
            "id":"response","output":[{"type":"function_call","name":"write","call_id":"effect-1","arguments":"{}"}]
        }})]]), requests:0 };
        let mut options = CodexDirectOptions::new("fixture","",vec![]);
        options.tools = vec![CodexDirectTool {name:"write".into(),description:None,parameters:json!({"type":"object"}),source:Value::Null}];
        let effects = Cell::new(0);
        let accepted = RefCell::new(None::<CodexDirectCheckpoint>);
        let error = run_codex_direct_responses_with_lifecycle(&mut transport,&options,None,
            &mut |_,_,_| { assert!(accepted.borrow().as_ref().is_some_and(|c| !c.pending_tool_calls.is_empty())); effects.set(effects.get()+1); Ok(Value::Null) },
            &mut |_,_| {}, &mut |checkpoint| {
                if !checkpoint.pending_tool_calls.is_empty() {
                    assert_eq!(checkpoint.pending_tool_calls[0]["call_id"],"effect-1");
                    assert_eq!(effects.get(),0);
                    if !fail_before { *accepted.borrow_mut()=Some(checkpoint.clone()); return Ok(()); }
                }
                Err(CodexDirectError::Transport("checkpoint rejected".into()))
            }, &|| false).unwrap_err();
        assert!(error.to_string().contains("checkpoint rejected"));
        assert_eq!(effects.get(),usize::from(!fail_before));
        if let Some(checkpoint) = accepted.into_inner() {
            let mut resumed = Transport::default();
            let error = run_codex_direct_responses_with_lifecycle(&mut resumed,&options,Some(&checkpoint),
                &mut |_,_,_| panic!("uncertain effect replayed"), &mut |_,_| {}, &mut |_| panic!("unexpected checkpoint"), &|| false).unwrap_err();
            assert!(error.to_string().contains("uncertain execution"));
            assert_eq!(resumed.requests,0);
        }
    }
}

#[test]
fn openrouter_intent_failure_prevents_effect_and_uncertain_recovery_is_blocked() {
    for fail_before in [true,false] {
        let mut transport = Transport { responses:VecDeque::from([vec![json!({"choices":[{"delta":{"tool_calls":[{
            "index":0,"id":"effect-1","function":{"name":"write","arguments":"{}"}
        }]}}]})]]), requests:0 };
        let effects = Cell::new(0);
        let accepted = RefCell::new(None::<OpenRouterCheckpoint>);
        let error = run_openrouter_with_transport(&mut transport,"fixture",&[],&tools(),
            &mut |_,_,_| {assert!(accepted.borrow().is_some()); effects.set(effects.get()+1); Ok(Value::Null)},
            &mut |_,_| {}, &|| false,None,&mut |checkpoint| {
                if !checkpoint.pending_tool_calls.is_empty() {
                    assert_eq!(checkpoint.pending_tool_calls[0]["id"],"effect-1");
                    assert_eq!(effects.get(),0);
                    if !fail_before { *accepted.borrow_mut()=Some(checkpoint.clone()); return Ok(()); }
                }
                Err(ProviderSessionError::Transport("checkpoint rejected".into()))
            }).unwrap_err();
        assert!(error.to_string().contains("checkpoint rejected"));
        assert_eq!(effects.get(),usize::from(!fail_before));
        if let Some(checkpoint) = accepted.into_inner() {
            let mut resumed=Transport::default();
            let error=run_openrouter_with_transport(&mut resumed,"fixture",&[],&tools(),
                &mut |_,_,_| panic!("uncertain effect replayed"),&mut |_,_| {},&||false,Some(&checkpoint),&mut |_|Ok(())).unwrap_err();
            assert!(error.to_string().contains("uncertain execution"));
            assert_eq!(resumed.requests,0);
        }
    }
}

struct CursorTransport { events:Mutex<Option<Vec<CursorInferenceEvent>>>, requests:std::sync::atomic::AtomicUsize }
impl CursorInferenceStreamTransport for CursorTransport {
    fn stream_conversation(&self,_:&[CursorCheckpointMessage],_:&[RoutedToolDefinition],_:&dyn Fn()->bool,
        event:&mut dyn FnMut(CursorInferenceEvent)->Result<(),ProviderSessionError>)->Result<(),ProviderSessionError> {
        self.requests.fetch_add(1,std::sync::atomic::Ordering::SeqCst);
        for value in self.events.lock().unwrap().take().expect("unexpected model request") {event(value)?;}
        Ok(())
    }
}
#[test]
fn cursor_intent_failure_prevents_effect_and_uncertain_recovery_is_blocked() {
    for fail_before in [true,false] {
        let transport=CursorTransport {events:Mutex::new(Some(vec![CursorInferenceEvent::ToolCall {
            tool_call_id:"effect-1".into(),tool_name:"write".into(),args:"{}".into(),complete:true,tool_index:Some(0)
        }])), requests:0.into()};
        let effects=Cell::new(0);
        let accepted=RefCell::new(None::<CursorCheckpoint>);
        let error=run_cursor_with_transport_reporting_usage(&transport,&[] as &[ProviderMessage],&tools(),
            &mut |_,_,_| {assert!(accepted.borrow().is_some());effects.set(effects.get()+1);Ok(Value::Null)},
            &mut |_,_| {},&||false,None,&mut |checkpoint| {
                if !checkpoint.pending_tool_calls.is_empty() {
                    assert_eq!(checkpoint.pending_tool_calls[0]["id"],"effect-1");assert_eq!(effects.get(),0);
                    if !fail_before {*accepted.borrow_mut()=Some(checkpoint.clone());return Ok(());}
                }
                Err(ProviderSessionError::Transport("checkpoint rejected".into()))
            },&mut |_|{}).unwrap_err();
        assert!(error.to_string().contains("checkpoint rejected"));assert_eq!(effects.get(),usize::from(!fail_before));
        if let Some(checkpoint)=accepted.into_inner() {
            let resumed=CursorTransport{events:Mutex::new(None),requests:0.into()};
            let error=run_cursor_with_transport_reporting_usage(&resumed,&[],&tools(),
                &mut |_,_,_|panic!("uncertain effect replayed"),&mut |_,_|{},&||false,Some(&checkpoint),&mut |_|Ok(()),&mut |_|{}).unwrap_err();
            assert!(error.to_string().contains("uncertain execution"));assert_eq!(resumed.requests.load(std::sync::atomic::Ordering::SeqCst),0);
        }
    }
}

#[test]
fn historical_completed_checkpoints_default_to_no_pending_effects() {
    let value=json!({"input":[],"text":"","response_id":"","usage":{"input_tokens":0,"output_tokens":0,"cache_read_tokens":0,"cache_write_tokens":0},"completed_steps":1,"tool_calls_completed":1});
    let checkpoint:CodexDirectCheckpoint=serde_json::from_value(value).unwrap();
    assert!(checkpoint.pending_tool_calls.is_empty());
    for provider in ["openrouter","cursor"] {
        let value=json!({"conversation":[],"text":"","completed_steps":1,"tool_calls_completed":1});
        if provider=="openrouter" {assert!(serde_json::from_value::<OpenRouterCheckpoint>(value).unwrap().pending_tool_calls.is_empty());}
        else {assert!(serde_json::from_value::<CursorCheckpoint>(value).unwrap().pending_tool_calls.is_empty());}
    }
}
