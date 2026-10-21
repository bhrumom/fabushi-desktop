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

#[test]
fn responses_bad_arguments_are_feedback_and_corrected_call_executes_once() {
    for raw in ["{broken", "[]", "null", "42", ""] {
        let call = |id: &str, args: &str| vec![json!({"type":"response.completed","response":{
            "id": id, "output":[{"type":"function_call","name":"write","call_id":id,"arguments":args}]
        }})];
        let mut transport = Transport { responses: VecDeque::from([
            call("bad", raw), call("corrected", r#"{"value":1}"#),
            vec![json!({"type":"response.completed","response":{"id":"done","output":[]}})],
        ]), requests:0 };
        let mut options = CodexDirectOptions::new("fixture", "", vec![]);
        options.tools = vec![CodexDirectTool { name:"write".into(), description:None,
            parameters:json!({"type":"object"}), source:Value::Null }];
        let mut effects = 0;
        let mut saw_error = false;
        run_codex_direct_responses_with_lifecycle(&mut transport, &options, None,
            &mut |_, args, id| { assert_eq!(id,"corrected");assert_eq!(args,json!({"value":1}));effects+=1;Ok(Value::Null) },
            &mut |_,_| {}, &mut |checkpoint| {
                if checkpoint.completed_steps == 1 && checkpoint.pending_tool_calls.is_empty() {
                    saw_error = checkpoint.input.iter().any(|item| item["call_id"] == "bad"
                        && item["output"].as_str().is_some_and(|s| s.contains("Invalid tool arguments")));
                }
                Ok(())
            }, &||false).unwrap();
        assert_eq!(effects,1);assert!(saw_error);assert_eq!(transport.requests,3);
    }
}

#[test]
fn openrouter_bad_arguments_are_feedback_and_corrected_call_executes_once() {
    for raw in ["{broken", "[]", "null", "42", ""] {
        let call = |id: &str, args: &str| vec![json!({"choices":[{"delta":{"tool_calls":[{
            "index":0,"id":id,"function":{"name":"write","arguments":args}
        }]}}]})];
        let mut transport = Transport { responses:VecDeque::from([
            call("bad",raw), call("corrected",r#"{"value":1}"#),
            vec![json!({"choices":[{"delta":{"content":"done"},"finish_reason":"stop"}]})],
        ]), requests:0 };
        let mut effects=0;
        let mut saw_error=false;
        run_openrouter_with_transport(&mut transport,"fixture",&[],&tools(),
            &mut |_,args,id| {assert_eq!(id,"corrected");assert_eq!(args,json!({"value":1}));effects+=1;Ok(Value::Null)},
            &mut |_,_|{},&||false,None,&mut |checkpoint| {
                if checkpoint.completed_steps==1 && checkpoint.pending_tool_calls.is_empty() {
                    saw_error=checkpoint.conversation.iter().any(|item|item["tool_call_id"]=="bad"
                        && item["content"].as_str().is_some_and(|s|s.contains("Invalid tool arguments")));
                }
                Ok(())
            }).unwrap();
        assert_eq!(effects,1);assert!(saw_error);assert_eq!(transport.requests,3);
    }
}

struct CursorSequenceTransport { rounds:Mutex<VecDeque<Vec<CursorInferenceEvent>>> }
impl CursorInferenceStreamTransport for CursorSequenceTransport {
    fn stream_conversation(&self,_:&[CursorCheckpointMessage],_:&[RoutedToolDefinition],_:&dyn Fn()->bool,
        event:&mut dyn FnMut(CursorInferenceEvent)->Result<(),ProviderSessionError>)->Result<(),ProviderSessionError> {
        let round=self.rounds.lock().unwrap().pop_front().expect("unexpected model call");
        for value in round {event(value)?;}
        Ok(())
    }
}
#[test]
fn cursor_bad_arguments_are_feedback_and_corrected_call_executes_once() {
    for raw in ["{broken", "[]", "null", "42", ""] {
        let call=|id:&str,args:&str|vec![CursorInferenceEvent::ToolCall {
            tool_call_id:id.into(),tool_name:"write".into(),args:args.into(),complete:true,tool_index:Some(0),
        }];
        let transport=CursorSequenceTransport {rounds:Mutex::new(VecDeque::from([
            call("bad",raw),call("corrected",r#"{"value":1}"#),vec![],
        ]))};
        let mut effects=0;
        let mut saw_error=false;
        run_cursor_with_transport_reporting_usage(&transport,&[],&tools(),
            &mut |_,args,id| {assert_eq!(id,"corrected");assert_eq!(args,json!({"value":1}));effects+=1;Ok(Value::Null)},
            &mut |_,_|{},&||false,None,&mut |checkpoint| {
                if checkpoint.completed_steps==1 && checkpoint.pending_tool_calls.is_empty() {
                    saw_error=checkpoint.conversation.iter().any(|item| matches!(item,
                        CursorCheckpointMessage::ToolResult{tool_call_id,is_error:true,result,..}
                        if tool_call_id=="bad" && result.to_string().contains("Invalid tool arguments")));
                }
                Ok(())
            },&mut |_|{}).unwrap();
        assert_eq!(effects,1);assert!(saw_error);assert!(transport.rounds.lock().unwrap().is_empty());
    }
}

#[test]
fn responses_usage_survives_later_failure_and_resume_reports_only_new_spend() {
    use mahayana_host_runtime::extensions::inference::codex_direct_responses::{
        CodexDirectUsage, run_codex_direct_responses_with_lifecycle_reporting_usage,
    };
    let mut options=CodexDirectOptions::new("fixture","",vec![]);
    options.tools=vec![CodexDirectTool { name:"write".into(),description:None,
        parameters:json!({"type":"object"}),source:Value::Null }];
    let response=json!({"type":"response.completed","response":{
        "id":"paid","usage":{"input_tokens":11,"output_tokens":7},
        "output":[{"type":"function_call","name":"write","call_id":"effect","arguments":"{}"}]
    }});
    let mut transport=Transport {responses:VecDeque::from([vec![response]]),requests:0};
    let mut spend=Vec::new();
    let error=run_codex_direct_responses_with_lifecycle_reporting_usage(
        &mut transport,&options,None,&mut |_,_,_|panic!("rejected intent cannot dispatch"),
        &mut |_,_|{},&mut |_|Err(CodexDirectError::Transport("disk rejected".into())),
        &||false,None,&mut |usage|spend.push(usage),
    ).unwrap_err();
    assert!(error.to_string().contains("disk rejected"));
    assert_eq!(spend.len(),1);assert_eq!(spend[0].input_tokens,11);assert_eq!(spend[0].output_tokens,7);

    let previous=CodexDirectCheckpoint {input:vec![],text:String::new(),response_id:"old".into(),
        usage:CodexDirectUsage {input_tokens:100,output_tokens:50,..Default::default()},
        completed_steps:1,tool_calls_completed:1,pending_tool_calls:vec![]};
    let response=json!({"type":"response.completed","response":{
        "id":"new","usage":{"input_tokens":13,"output_tokens":3},"output":[]
    }});
    // Duplicate terminal frames must not double-report a single request's usage.
    let mut transport=Transport {responses:VecDeque::from([vec![response.clone(),response]]),requests:0};
    spend.clear();
    let result=run_codex_direct_responses_with_lifecycle_reporting_usage(
        &mut transport,&options,Some(&previous),&mut |_,_,_|panic!("no effect"),
        &mut |_,_|{},&mut |_|Ok(()),&||false,None,&mut |usage|spend.push(usage),
    ).unwrap();
    assert_eq!(spend.len(),1);assert_eq!(spend[0].input_tokens,13);
    assert_eq!(result.usage.input_tokens,113,"checkpoint aggregates remain historical totals");

    let mut transport=Transport {responses:VecDeque::from([vec![json!({"type":"response.failed","response":{
        "usage":{"input_tokens":5,"output_tokens":2},"error":{"message":"failed after billing"}
    }})]]),requests:0};
    spend.clear();
    assert!(run_codex_direct_responses_with_lifecycle_reporting_usage(
        &mut transport,&options,None,&mut |_,_,_|panic!("failed response has no effects"),
        &mut |_,_|{},&mut |_|Ok(()),&||false,None,&mut |usage|spend.push(usage),
    ).is_err());
    assert_eq!(spend.len(),1);assert_eq!(spend[0].input_tokens,5);
}

#[test]
fn openrouter_missing_call_id_is_assigned_before_durable_intent() {
    let mut transport=Transport {responses:VecDeque::from([
        vec![json!({"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"name":"write","arguments":"{}"}}]}}]})],
        vec![json!({"choices":[{"delta":{"content":"done"}}]})],
    ]),requests:0};
    let recorded=RefCell::new(String::new());
    let dispatched=Cell::new(false);
    run_openrouter_with_transport(&mut transport,"fixture",&[],&tools(),
        &mut |_,_,id| {assert!(!id.is_empty());assert_eq!(id,recorded.borrow().as_str());dispatched.set(true);Ok(Value::Null)},
        &mut |_,_|{},&||false,None,&mut |checkpoint| {
            if let Some(call)=checkpoint.pending_tool_calls.first() {
                *recorded.borrow_mut()=call["id"].as_str().unwrap().to_string();
                assert!(!recorded.borrow().is_empty());
            }
            Ok(())
        }).unwrap();
    assert!(dispatched.get());
}

#[test]
fn provider_repeat_observation_is_once_only_and_never_dispatches_extra_calls() {
    let call=|id:&str,args:Value|vec![json!({"choices":[{"delta":{"tool_calls":[{
        "index":0,"id":id,"function":{"name":"write","arguments":args.to_string()}
    }]}}]})];
    let same=json!({"nested":{"b":2,"a":1}});
    let changed=json!({"nested":{"a":9,"b":2}});
    let mut rounds=VecDeque::new();
    for (index,args) in [same.clone(),same.clone(),same.clone(),changed,same.clone(),same.clone(),same.clone()].into_iter().enumerate() {
        rounds.push_back(call(&format!("call-{index}"),args));
    }
    rounds.push_back(vec![json!({"choices":[{"delta":{"content":"done"}}]})]);
    let mut transport=Transport{responses:rounds,requests:0};
    let mut dispatched=0;
    let mut feedback=Vec::new();
    run_openrouter_with_transport(&mut transport,"fixture",&[],&tools(),
        &mut |_,_,_| {
            dispatched+=1;
            if dispatched==5 {Ok(json!({"error":null,"failed":0}))}
            else {Ok(json!({"isError":true,"error":"daemon refused"}))}
        },&mut |_,_|{},&||false,None,&mut |checkpoint| {
            if checkpoint.pending_tool_calls.is_empty() {
                let result=checkpoint.conversation.last().unwrap()["content"].as_str().unwrap();
                feedback.push(serde_json::from_str::<Value>(result).unwrap());
            }
            Ok(())
        }).unwrap();
    assert_eq!(dispatched,7,"only model-proposed calls execute");
    assert_eq!(transport.requests,8,"observations cannot trigger another model request");
    assert_eq!(feedback.iter().map(|v|v.get("toolObservation").is_some()).collect::<Vec<_>>(),
        vec![false,true,false,false,false,false,true]);
    assert_eq!(feedback[1]["error"],"daemon refused","original diagnostic is retained");
}

#[test]
fn cursor_embedded_failure_is_error_but_empty_error_and_refusal_are_not() {
    let results=[json!({"error":null,"failed":0}),json!({"error":false}),
        json!({"rejected":"policy","error":"not allowed"}),json!({"isError":true,"error":"refused"})];
    let mut rounds=VecDeque::new();
    for index in 0..results.len() {
        rounds.push_back(vec![CursorInferenceEvent::ToolCall {
            tool_call_id:format!("call-{index}"),tool_name:"write".into(),args:"{}".into(),complete:true,tool_index:Some(0),
        }]);
    }
    rounds.push_back(vec![]);
    let transport=CursorSequenceTransport{rounds:Mutex::new(rounds)};
    let mut outputs=results.into_iter();
    let mut flags=Vec::new();
    run_cursor_with_transport_reporting_usage(&transport,&[],&tools(),
        &mut |_,_,_|Ok(outputs.next().unwrap()),&mut |_,_|{},&||false,None,&mut |checkpoint| {
            if checkpoint.pending_tool_calls.is_empty() {
                if let Some(CursorCheckpointMessage::ToolResult{is_error,..})=checkpoint.conversation.last() {
                    flags.push(*is_error);
                }
            }
            Ok(())
        },&mut |_|{}).unwrap();
    assert_eq!(flags,vec![false,false,false,true]);
}
