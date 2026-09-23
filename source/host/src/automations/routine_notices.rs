use serde_json::Value;
use super::automation_trigger::trigger_members;
pub const GITHUB_LISTENER_SCOPE:&str="github-listener-scope";
pub const GITHUB_LISTENER_SCOPE_CREATED_BEFORE_MS:f64=1_775_001_600_000.0;
pub fn is_routine_notice_id(value:&str)->bool{value==GITHUB_LISTENER_SCOPE}
pub fn routine_notice_ids_to_raise(created_at:f64,trigger:&Value,raised:&[String])->Vec<String>{
    if created_at>=GITHUB_LISTENER_SCOPE_CREATED_BEFORE_MS||raised.iter().any(|id|id==GITHUB_LISTENER_SCOPE){return Vec::new();}
    let applies=trigger_members(trigger).iter().any(|listener|listener.get("type").and_then(Value::as_str)==Some("github")&&listener.get("userAllowlist").and_then(Value::as_array).is_none_or(|v|v.is_empty()));
    applies.then(||vec![GITHUB_LISTENER_SCOPE.to_string()]).unwrap_or_default()
}
