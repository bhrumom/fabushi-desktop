use serde_json::{Map, Value, json};

pub const TRIGGER_ANY_SCOPE: &str = "*";
pub const TRIGGER_MAX_GROUP_LISTENERS: usize = 8;
pub const TRIGGER_MAX_REACTION_EMOJI: usize = 8;
pub const TRIGGER_MAX_CHANNEL_LENGTH: usize = 80;
pub const TRIGGER_MAX_KEYWORD_LENGTH: usize = 120;
pub const TRIGGER_MAX_REPO_LENGTH: usize = 140;
pub const TRIGGER_MAX_BRANCH_LENGTH: usize = 200;
pub const TRIGGER_MAX_ALLOWLIST_LOGINS: usize = 50;
pub const TRIGGER_MAX_ALLOWLIST_LOGIN_LENGTH: usize = 80;
pub const TRIGGER_MAX_FILTER_IDS: usize = 50;
pub const TRIGGER_MAX_ID_LENGTH: usize = 200;

pub const GITHUB_EVENT_KINDS: &[&str] = &[
    "pr-opened", "pr-pushed", "pr-merged", "review-requested", "review-approved",
    "review-changes-requested", "review-commented", "pr-comment",
    "inline-review-comment", "review-thread-resolved", "review-thread-unresolved",
    "issue-assigned", "ci-passed", "ci-failed",
];

fn record(value: &Value) -> Option<&Map<String, Value>> { value.as_object() }

fn token(raw: Option<&Value>, max: usize) -> Option<String> {
    let raw = raw?.as_str()?;
    let value = raw.replace(['\r', '\n'], " ").trim().chars().take(max).collect::<String>();
    (!value.is_empty()).then_some(value)
}

fn list(raw: Option<&Value>) -> Vec<String> {
    let Some(values)=raw.and_then(Value::as_array) else { return Vec::new(); };
    let mut result=Vec::new();
    for entry in values {
        let Some(value)=token(Some(entry), TRIGGER_MAX_ID_LENGTH) else { continue; };
        if !result.contains(&value) { result.push(value); }
        if result.len()>=TRIGGER_MAX_FILTER_IDS { break; }
    }
    result
}

fn is_github_ci_event_kind(kind:&str)->bool { matches!(kind,"ci-passed"|"ci-failed") }
fn valid_github_repo(repo:&str)->bool {
    let mut parts=repo.split('/');
    matches!((parts.next(),parts.next(),parts.next()),(Some(a),Some(b),None) if !a.is_empty()&&!b.is_empty()&&!a.chars().any(char::is_whitespace)&&!b.chars().any(char::is_whitespace))
}
fn valid_git_branch(branch:&str)->bool {
    !branch.is_empty() && !branch.chars().any(char::is_whitespace) && !branch.contains("..")
        && !branch.contains("@{") && !branch.starts_with('-') && !branch.starts_with('/')
        && !branch.ends_with('/') && !branch.chars().any(|c| matches!(c,'~'|'^'|':'|'?'|'*'|'['|'\\'|'|'))
}
fn normalize_reaction_emoji(raw:&str)->String {
    let bare=raw.trim().trim_matches(':');
    bare.split("::").next().unwrap_or(bare).trim().to_ascii_lowercase()
}
fn valid_reaction_emoji(value:&str)->bool {
    !value.is_empty() && value.chars().all(|c| c.is_ascii_lowercase()||c.is_ascii_digit()||matches!(c,'_'|'+'|'-'))
}

fn parse_slack(value:&Map<String,Value>)->Option<Value>{
    let channel=token(value.get("channel"),TRIGGER_MAX_CHANNEL_LENGTH)?;
    let match_obj=record(value.get("match")?)?;
    let kind=match_obj.get("kind")?.as_str()?;
    let parsed=match kind {
        "mention"|"message"=>json!({"kind":kind}),
        "keyword"=>{
            let keyword=token(match_obj.get("keyword"),TRIGGER_MAX_KEYWORD_LENGTH)?;
            json!({"kind":"keyword","keyword":keyword})
        }
        "reaction"=>{
            let mut emoji=Vec::<String>::new();
            if let Some(raw)=match_obj.get("emoji").and_then(Value::as_array){
                for entry in raw {
                    let Some(raw)=entry.as_str() else {continue};
                    let value=normalize_reaction_emoji(raw);
                    if valid_reaction_emoji(&value)&&!emoji.contains(&value){emoji.push(value);}
                    if emoji.len()>=TRIGGER_MAX_REACTION_EMOJI{break;}
                }
            }
            let mut object=Map::new();
            object.insert("kind".into(),Value::String("reaction".into()));
            if !emoji.is_empty(){object.insert("emoji".into(),json!(emoji));}
            if match_obj.get("bySelf").and_then(Value::as_bool)==Some(true){object.insert("bySelf".into(),Value::Bool(true));}
            Value::Object(object)
        }
        _=>return None,
    };
    Some(json!({"type":"slack","channel":channel,"match":parsed}))
}

fn parse_github(value:&Map<String,Value>)->Option<Value>{
    let repo=token(value.get("repo"),TRIGGER_MAX_REPO_LENGTH)?;
    if !valid_github_repo(&repo){return None;}
    let mut events=Vec::<String>::new();
    if let Some(raw)=value.get("events").and_then(Value::as_array){
        for event in raw {
            let Some(kind)=event.as_str() else {continue};
            if GITHUB_EVENT_KINDS.contains(&kind)&&!events.iter().any(|v|v==kind){events.push(kind.to_string());}
        }
    }
    let branch=token(value.get("ciBranch"),TRIGGER_MAX_BRANCH_LENGTH).filter(|b|valid_git_branch(b));
    if branch.is_none(){events.retain(|kind|!is_github_ci_event_kind(kind));}
    if events.is_empty(){return None;}
    let mut users=Vec::<String>::new();
    if let Some(raw)=value.get("userAllowlist").and_then(Value::as_array){
        for entry in raw {
            let Some(login)=token(Some(entry),TRIGGER_MAX_ALLOWLIST_LOGIN_LENGTH) else {continue};
            let login=login.trim_start_matches('@').to_string();
            if !login.is_empty()&&!users.iter().any(|v|v.eq_ignore_ascii_case(&login)){users.push(login);}
            if users.len()>=TRIGGER_MAX_ALLOWLIST_LOGINS{break;}
        }
    }
    let watches_ci=events.iter().any(|kind|is_github_ci_event_kind(kind));
    let mut out=json!({"type":"github","repo":repo,"events":events});
    let object=out.as_object_mut().unwrap();
    if !users.is_empty(){object.insert("userAllowlist".into(),json!(users));}
    if watches_ci { if let Some(branch)=branch { object.insert("ciBranch".into(),Value::String(branch)); } }
    Some(out)
}

fn parse_member(value:&Value)->Option<Value>{
    let object=record(value)?;
    match object.get("type")?.as_str()? {
        "cron"=>Some(json!({"type":"cron","schedule":token(object.get("schedule"),120)?})),
        "slack"=>parse_slack(object),
        "github"=>parse_github(object),
        "microsoftTeams"=>{
            let tenant=token(object.get("tenantId"),TRIGGER_MAX_ID_LENGTH)?;
            let team_id=token(object.get("teamId"),TRIGGER_MAX_ID_LENGTH).unwrap_or_default();
            let team_ids=list(object.get("teamIds"));
            if team_id.is_empty()&&team_ids.is_empty(){return None;}
            let contains=object.get("messageContains").and_then(Value::as_str).unwrap_or_default()
                .replace(['\r','\n']," ").trim().chars().take(TRIGGER_MAX_KEYWORD_LENGTH).collect::<String>();
            Some(json!({"type":"microsoftTeams","tenantId":tenant,"teamId":team_id,"teamIds":team_ids,
                "channelIds":list(object.get("channelIds")),"messageContains":contains,
                "messageContainsIsRegex":object.get("messageContainsIsRegex").and_then(Value::as_bool)==Some(true),
                "blockUnauthenticatedTeamsUsers":object.get("blockUnauthenticatedTeamsUsers").and_then(Value::as_bool)==Some(true)}))
        }
        "linear"=>{
            let event=record(object.get("event")?)?; let case=event.get("case")?.as_str()?;
            let event_value=match case {
                "issueCreated"=>json!({"case":"issueCreated"}),
                "statusChanged"=>json!({"case":"statusChanged","statusIds":list(event.get("statusIds"))}),
                "endOfCycle"=>json!({"case":"endOfCycle","cycleIds":list(event.get("cycleIds"))}),
                _=>return None,
            };
            Some(json!({"type":"linear","event":event_value,"projectIds":list(object.get("projectIds")),"teamIds":list(object.get("teamIds"))}))
        }
        "sentry"|"pagerduty"=>{
            let ty=object.get("type")?.as_str()?; let event=record(object.get("event")?)?; let case=event.get("case")?.as_str()?;
            let allowed=if ty=="sentry" {["issueCreated","issueResolved","issueAssigned","issueArchived","issueUnresolved","issueAny"].as_slice()} else {["incidentTriggered","incidentAcknowledged","incidentResolved","incidentEscalated","incidentAny"].as_slice()};
            if !allowed.contains(&case){return None;}
            if ty=="sentry"{Some(json!({"type":"sentry","event":{"case":case},"projectIds":list(object.get("projectIds"))}))}
            else{Some(json!({"type":"pagerduty","event":{"case":case},"serviceIds":list(object.get("serviceIds"))}))}
        }
        _=>None,
    }
}

pub fn trigger_members(trigger:&Value)->Vec<Value>{
    if trigger.get("type").and_then(Value::as_str)==Some("group"){
        trigger.get("listeners").and_then(Value::as_array).cloned().unwrap_or_default()
    } else { vec![trigger.clone()] }
}
pub fn parse_stored_trigger(value:&Value)->Option<Value>{
    let entries=if let Some(array)=value.as_array(){array.clone()}
    else if value.get("type").and_then(Value::as_str)==Some("group"){value.get("listeners").and_then(Value::as_array).cloned().unwrap_or_default()}
    else{return parse_member(value);};
    let mut members=Vec::new();
    for entry in entries { if let Some(member)=parse_member(&entry){members.push(member);} if members.len()>=TRIGGER_MAX_GROUP_LISTENERS{break;} }
    match members.len(){0=>None,1=>members.into_iter().next(),_=>Some(json!({"type":"group","listeners":members}))}
}
pub fn trigger_schedule(trigger:&Value)->Option<String>{
    trigger_members(trigger).into_iter().find_map(|member|{
        (member.get("type").and_then(Value::as_str)==Some("cron")).then(||member.get("schedule").and_then(Value::as_str).map(ToOwned::to_owned)).flatten()
    })
}
pub fn trigger_listener_platforms(trigger:&Value)->Vec<String>{
    let mut out=Vec::new();
    for member in trigger_members(trigger){
        let Some(ty)=member.get("type").and_then(Value::as_str) else {continue};
        if matches!(ty,"github"|"slack")&&!out.iter().any(|v|v==ty){out.push(ty.to_string());}
    }
    out
}
pub fn trigger_identity(trigger:&Value)->String{
    if trigger.get("type").and_then(Value::as_str)==Some("cron"){
        format!("cron:{}",trigger.get("schedule").and_then(Value::as_str).unwrap_or_default())
    }else{serde_json::to_string(trigger).unwrap_or_default()}
}
pub fn slack_scope_matches(scope:&str,actual:&str)->bool{
    if scope==TRIGGER_ANY_SCOPE{return true;}
    fn split(value:&str)->(Option<char>,String){
        let first=value.chars().next().filter(|c|matches!(c,'#'|'@'));
        let name=value.trim_start_matches(['#','@']).to_ascii_lowercase(); (first,name)
    }
    let (a_sigil,a)=split(scope); let (b_sigil,b)=split(actual);
    !(a_sigil.is_some()&&b_sigil.is_some()&&a_sigil!=b_sigil)&&a==b
}
