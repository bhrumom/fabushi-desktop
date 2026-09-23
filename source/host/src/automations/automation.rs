use std::path::PathBuf;
use serde_json::Value;
use uuid::Uuid;
use super::automation_trigger::{trigger_schedule,trigger_members};

pub const AUTOMATION_MAX_NAME_LENGTH:usize=80;
pub const AUTOMATION_MAX_PER_AGENT:usize=50;
pub const AUTOMATION_UI_LIMIT:usize=100;
pub const AUTOMATION_MAX_RUN_HISTORY:usize=20;
pub const AUTOMATION_MAX_RUN_DETAIL_LENGTH:usize=300;
pub const MAX_EVENTS_IN_AUTOMATION_WAKE:usize=25;

#[derive(Debug,Clone,PartialEq)]
pub struct AutomationRun{pub id:String,pub trigger:String,pub started_at:f64,pub finished_at:Option<f64>,pub status:String,pub detail:Option<String>,pub event:Option<String>,pub coalesced_run_ids:Option<Vec<String>>}
#[derive(Debug,Clone,PartialEq)]
pub struct AutomationConfig{pub name:String,pub prompt:String,pub trigger:Value,pub is_enabled:bool,pub created_at:f64,pub last_run_at:Option<f64>,pub raised_notices:Vec<String>}
#[derive(Debug,Clone,PartialEq)]
pub struct AutomationRecord{pub id:String,pub name:String,pub prompt:String,pub trigger:Value,pub is_enabled:bool,pub created_at:f64,pub last_run_at:Option<f64>,pub raised_notices:Vec<String>,pub schedule:String,pub trigger_description:String,pub next_run_at:Option<f64>,pub runs:Vec<AutomationRun>,pub file_path:PathBuf}
#[derive(Debug,Clone,PartialEq)]
pub struct AutomationSpec{pub name:String,pub prompt:String,pub trigger:Value,pub is_enabled:Option<bool>}

pub fn clamp_automation_name(name:&str)->String{name.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(AUTOMATION_MAX_NAME_LENGTH).collect()}
pub fn normalize_automation_prompt(prompt:&str)->String{prompt.trim().to_string()}
pub fn slugify_automation_name(name:&str)->String{let mut out=String::new();let mut dash=false;for c in name.trim().to_ascii_lowercase().chars(){if c.is_ascii_alphanumeric(){out.push(c);dash=false;}else if !dash&&!out.is_empty(){out.push('-');dash=true;}}while out.ends_with('-'){out.pop();}if out.is_empty(){"automation".into()}else{out}}
pub fn create_automation_run_id()->String{Uuid::new_v4().to_string()}
pub fn clamp_run_detail(detail:Option<&str>)->Option<String>{let value=detail?.trim();(!value.is_empty()).then(||value.chars().take(AUTOMATION_MAX_RUN_DETAIL_LENGTH).collect())}
pub fn clamp_coalesced_run_ids(values:Option<&[String]>)->Option<Vec<String>>{let out=values.unwrap_or_default().iter().filter(|id|!id.is_empty()).take(MAX_EVENTS_IN_AUTOMATION_WAKE).cloned().collect::<Vec<_>>();(!out.is_empty()).then_some(out)}
pub fn describe_trigger(trigger:&Value)->String{
    if let Some(schedule)=trigger_schedule(trigger){return format!("Scheduled: {schedule}");}
    let members=trigger_members(trigger); if members.len()>1{return format!("{} event listeners",members.len());}
    let Some(member)=members.first() else{return "Unknown trigger".into();};
    match member.get("type").and_then(Value::as_str).unwrap_or_default(){
        "slack"=>format!("Slack {}",member.get("channel").and_then(Value::as_str).unwrap_or_default()),
        "github"=>format!("GitHub {}",member.get("repo").and_then(Value::as_str).unwrap_or_default()),
        "microsoftTeams"=>"Microsoft Teams event".into(),"linear"=>"Linear event".into(),"sentry"=>"Sentry event".into(),"pagerduty"=>"PagerDuty event".into(),other=>other.to_string(),
    }
}
