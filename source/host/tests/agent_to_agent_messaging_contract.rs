use std::fs;
use std::sync::{Arc,Mutex};
use std::sync::atomic::{AtomicUsize,Ordering};
use std::time::{SystemTime,UNIX_EPOCH};
use mahayana_host_runtime::agents::agent_messaging::AgentMessageImage;
use mahayana_host_runtime::agents::agent_profile::SandAgentProfile;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::transcript::agent_to_agent_messaging::{AgentWakeRequest,ProductionAgentToAgentMessaging};

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
