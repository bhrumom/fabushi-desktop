use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use mahayana_host_runtime::extensions::transcript::completion_revivals::{
    CompletionRevivalRuntimePort, CompletionRevivals, RevivalExecution, RevivalReport,
    ShellCompletion, SubagentCompletion, build_shell_revival_prompt,
    build_subagent_revival_prompt, describe_shell_outcome,
};
use mahayana_host_runtime::extensions::transcript::sand_pending_wake_store::{
    AutomationQuietOrigin, PendingWakeKind, QuietWakeOrigin,
};

#[derive(Default)]
struct FakeRuntime {
    can_execute: Mutex<bool>,
    deleted: Mutex<HashSet<String>>,
    gone: Mutex<HashSet<String>>,
    cleared: Mutex<Vec<(String, PendingWakeKind, String)>>,
    runs: Mutex<Vec<(String, String, String)>>,
    next: Mutex<Option<Result<RevivalExecution, String>>>,
    resume: Mutex<Vec<String>>,
    reports: Mutex<Vec<RevivalReport>>,
    errors: Mutex<Vec<String>>,
}
impl CompletionRevivalRuntimePort for FakeRuntime {
    fn can_execute(&self)->bool{*self.can_execute.lock().unwrap()}
    fn is_agent_deleted(&self,id:&str)->bool{self.deleted.lock().unwrap().contains(id)}
    fn is_agent_gone(&self,id:&str)->bool{self.gone.lock().unwrap().contains(id)}
    fn clear_pending_wake(&self,a:&str,k:PendingWakeKind,w:&str){self.cleared.lock().unwrap().push((a.into(),k,w.into()));}
    fn run_background_revival(&self,a:&str,s:&str,p:&str,_:bool,_:&str)->Result<RevivalExecution,String>{
        self.runs.lock().unwrap().push((a.into(),s.into(),p.into()));
        self.next.lock().unwrap().take().unwrap_or(Ok(RevivalExecution{aborted:false,quiesced_for_upgrade:false,sent_message_count:1}))
    }
    fn mark_resume_pending_for_quiesced_revival(&self,a:&str){self.resume.lock().unwrap().push(a.into());}
    fn report_revival(&self,r:RevivalReport){self.reports.lock().unwrap().push(r);}
    fn report_revival_error(&self,_:&str,t:&str,e:&str){self.errors.lock().unwrap().push(format!("{t}:{e}"));}
}
fn quiet()->QuietWakeOrigin{
    QuietWakeOrigin{automation:Some(AutomationQuietOrigin{id:"r1".into(),name:"Daily".into()})}
}
fn sub(id:&str)->SubagentCompletion{
    SubagentCompletion{parent_agent_id:"p".into(),subagent_agent_id:id.into(),title:"Research".into(),subagent_type:"task".into(),status:"success".into(),result:"done".into(),quiet_origin:None}
}
fn shell(id:&str)->ShellCompletion{
    ShellCompletion{agent_id:"p".into(),shell_id:id.into(),title:"Build".into(),status:"success".into(),detail:Some("ok".into()),output_path:Some("/tmp/out".into()),quiet_origin:None}
}

#[test]
fn frozen_prompt_shaping_preserves_quiet_and_shell_semantics(){
    let mut q=sub("s1"); q.quiet_origin=Some(quiet());
    let prompt=build_subagent_revival_prompt(&[q]);
    assert!(prompt.starts_with("[A background task just completed] A background task you started has finished."));
    assert!(prompt.contains("your routine \"Daily\" (folder r1)"));
    assert!(prompt.contains("quiet standing order(s)"));

    let mut sh=shell("x"); sh.status="aborted".into(); sh.quiet_origin=Some(quiet());
    let prompt=build_shell_revival_prompt(&[sh]);
    assert!(prompt.contains("Background command \"Build\" was stopped."));
    assert!(prompt.contains("Full output: /tmp/out"));
    assert_eq!(describe_shell_outcome("success"),"finished");
    assert_eq!(describe_shell_outcome("aborted"),"was stopped");
    assert_eq!(describe_shell_outcome("error"),"failed");
}

#[test]
fn subagent_dedupes_clears_pending_and_reports_delivery(){
    let rt=Arc::new(FakeRuntime::default()); *rt.can_execute.lock().unwrap()=true;
    let service=CompletionRevivals::new(rt.clone());
    service.handle_background_subagent_completion(sub("s1"));
    assert_eq!(rt.runs.lock().unwrap().len(),1);
    assert_eq!(rt.cleared.lock().unwrap().as_slice(), &[("p".into(),PendingWakeKind::Subagent,"s1".into())]);
    let reports=rt.reports.lock().unwrap();
    assert_eq!(reports.len(),1);
    assert_eq!(reports[0].outcome,"delivered");
    assert_eq!(reports[0].completion_count,1);
    assert_eq!(reports[0].sent_message_count,Some(1));
}

#[test]
fn deleted_and_gone_completions_drop_without_running(){
    let rt=Arc::new(FakeRuntime::default()); *rt.can_execute.lock().unwrap()=true;
    rt.deleted.lock().unwrap().insert("p".into());
    let service=CompletionRevivals::new(rt.clone());
    service.handle_background_subagent_completion(sub("s1"));
    assert!(rt.runs.lock().unwrap().is_empty());
    assert_eq!(rt.reports.lock().unwrap()[0].reason.as_deref(),Some("agent_deleted"));

    rt.deleted.lock().unwrap().clear();
    rt.gone.lock().unwrap().insert("p".into());
    service.handle_background_shell_completion(shell("sh1"));
    assert_eq!(rt.reports.lock().unwrap().last().unwrap().reason.as_deref(),Some("agent_gone"));
}

#[test]
fn quiesced_shell_marks_upgrade_resume_and_clears_durable_wake(){
    let rt=Arc::new(FakeRuntime::default()); *rt.can_execute.lock().unwrap()=true;
    *rt.next.lock().unwrap()=Some(Ok(RevivalExecution{aborted:false,quiesced_for_upgrade:true,sent_message_count:0}));
    let service=CompletionRevivals::new(rt.clone());
    service.handle_background_shell_completion(shell("sh1"));
    assert_eq!(rt.cleared.lock().unwrap().as_slice(), &[("p".into(),PendingWakeKind::Shell,"sh1".into())]);
    assert_eq!(rt.resume.lock().unwrap().as_slice(), &["p".to_string()]);
    let report=rt.reports.lock().unwrap().last().unwrap().clone();
    assert_eq!(report.outcome,"dropped");
    assert_eq!(report.reason.as_deref(),Some("quiesced"));
}

#[test]
fn cursor_agent_completion_clears_cloud_marker_and_errors_are_reported(){
    let rt=Arc::new(FakeRuntime::default()); *rt.can_execute.lock().unwrap()=true;
    *rt.next.lock().unwrap()=Some(Err("runner failed".into()));
    let service=CompletionRevivals::new(rt.clone());
    let mut c=sub("bc1"); c.subagent_type="cursor-agent".into(); c.status="error".into();
    service.handle_background_subagent_completion(c);
    assert_eq!(rt.cleared.lock().unwrap()[0].1,PendingWakeKind::CloudAgent);
    assert!(rt.errors.lock().unwrap()[0].contains("Background task follow-up failed:runner failed"));
    assert_eq!(rt.reports.lock().unwrap()[0].reason.as_deref(),Some("error"));
}
