use std::path::PathBuf;

use mahayana_host_runtime::automations::automation::{AutomationRecord, AutomationRun};
use mahayana_host_runtime::automations::automation_id::stable_automation_id;
use mahayana_host_runtime::extensions::automations::fire_delivery::{
    PreparedBackendFire, admits_bare_fire, fire_event_matches_trigger, parse_fire_trigger_event,
    prepare_backend_fire,
};
use mahayana_host_runtime::extensions::automations::sand_automation_fire_consumer::{
    BackendAutomationFire, FireCompletion,
};
use mahayana_host_runtime::extensions::session::agent_session::AgentAutomationEntry;
use serde_json::{Value, json};

fn entry(trigger: Value) -> AgentAutomationEntry {
    AgentAutomationEntry {
        agent_id: "agent-1".into(),
        automation: AutomationRecord {
            id: "local-1".into(),
            name: "Routine".into(),
            prompt: "Do it".into(),
            trigger,
            is_enabled: true,
            created_at: 1.0,
            last_run_at: None,
            raised_notices: Vec::new(),
            schedule: String::new(),
            trigger_description: String::new(),
            next_run_at: None,
            runs: Vec::new(),
            file_path: PathBuf::from("/tmp/automation.md"),
        },
    }
}

fn fire(event: Option<Value>) -> BackendAutomationFire {
    BackendAutomationFire {
        id: "run-1".into(),
        sand_agent_id: "agent-1".into(),
        automation_id: stable_automation_id("agent-1", "local-1"),
        timestamp_ms: 100,
        definition_revision: None,
        scheduled_for_ms: None,
        event,
    }
}

// Pure delivery tests explicitly supply their in-memory authority. Shipping
// and owner integration tests instead use AutomationRuntime's checked read.
fn fixture_history(entry: &AgentAutomationEntry, run_id: &str) -> Result<Option<AutomationRun>, String> {
    Ok(entry.automation.runs.iter().find(|run| {
        run.id == run_id || run.coalesced_run_ids.as_ref()
            .is_some_and(|ids| ids.iter().any(|id| id == run_id))
    }).cloned())
}

#[test]
fn prepares_schedule_and_event_delivery_only_after_target_validation() {
    let schedule = entry(json!({"type":"cron","schedule":"0 * * * *"}));
    let mut scheduled_fire = fire(None);
    scheduled_fire.scheduled_for_ms = Some(90);
    assert!(matches!(
        prepare_backend_fire(&[schedule], &scheduled_fire, |_| None, fixture_history),
        PreparedBackendFire::Schedule {
            ref agent_id,
            ref automation_id,
            ref run_uuid,
            scheduled_for_ms: Some(90),
        } if agent_id == "agent-1" && automation_id == "local-1" && run_uuid == "run-1"
    ));

    let listener = entry(json!({
        "type":"github",
        "repo":"openai/repo",
        "events":["pr-opened"]
    }));
    let event_fire = fire(Some(json!({
        "source":"github",
        "repo":"openai/repo",
        "kind":"pr-opened",
        "actor":"octocat",
        "timestampMs":99
    })));
    assert!(matches!(
        prepare_backend_fire(&[listener], &event_fire, |_| None, fixture_history),
        PreparedBackendFire::Event {
            ref agent_id,
            ref automation_id,
            ref run_uuid,
            ..
        } if agent_id == "agent-1" && automation_id == "local-1" && run_uuid == "run-1"
    ));
}

#[test]
fn existing_completed_run_is_not_executed_twice() {
    let mut target = entry(json!({"type":"cron","schedule":"0 * * * *"}));
    target.automation.runs.push(AutomationRun {
        id: "other".into(),
        trigger: "schedule".into(),
        started_at: 1.0,
        finished_at: Some(2.0),
        status: "ok".into(),
        detail: None,
        event: None,
        coalesced_run_ids: Some(vec!["run-1".into()]),
    });
    let prepared = prepare_backend_fire(&[target], &fire(None), |_| None, fixture_history);
    assert!(matches!(
        prepared,
        PreparedBackendFire::Complete {
            ref completion,
            reason: "existing_run"
        } if completion.status == "succeeded"
    ));
}

#[test]
fn running_original_and_coalesced_ids_are_abandoned_before_fire_validation() {
    for stored_id in ["run-1", "coalesced-parent"] {
        for case in [
            "valid_event",
            "revisioned_schedule",
            "legacy_covered_slot",
            "disabled",
            "changed_definition",
            "unrecognized_event",
            "changed_trigger",
            "missing_event_context",
        ] {
            let mut target = entry(json!({"type":"group","listeners":[
                {"type":"cron","schedule":"0 * * * *"},
                {"type":"github","repo":"openai/repo","events":["pr-opened"]}
            ]}));
            let mut backend_fire = fire(None);
            backend_fire.scheduled_for_ms = Some(90);
            match case {
                "valid_event" => backend_fire.event = Some(json!({
                    "source":"github","repo":"openai/repo","kind":"pr-opened"
                })),
                "revisioned_schedule" => {
                    backend_fire.definition_revision = Some("current-revision".into());
                }
                "legacy_covered_slot" => target.automation.last_run_at = Some(101.0),
                "disabled" => target.automation.is_enabled = false,
                "changed_definition" => {
                    backend_fire.definition_revision = Some("old-revision".into());
                }
                "unrecognized_event" => {
                    backend_fire.event = Some(json!({"source":"unrecognized"}));
                }
                "changed_trigger" => backend_fire.event = Some(json!({
                    "source":"github","repo":"other/repo","kind":"pr-opened"
                })),
                "missing_event_context" => backend_fire.scheduled_for_ms = None,
                _ => unreachable!(),
            }
            target.automation.runs.push(AutomationRun {
                id: stored_id.into(),
                trigger: "event".into(),
                started_at: 1.0,
                finished_at: None,
                status: "running".into(),
                detail: None,
                event: None,
                coalesced_run_ids: (stored_id != "run-1").then(|| vec!["run-1".into()]),
            });
            let before = target.clone();
            assert_eq!(
                prepare_backend_fire(std::slice::from_ref(&target), &backend_fire, |_| {
                    panic!("known running identity must precede revision lookup: {case}")
                }, fixture_history),
                PreparedBackendFire::Abandon { reason: "existing_run_unsettled" },
                "{case}, stored_id={stored_id}",
            );
            assert_eq!(target, before, "preparation must not mutate history: {case}");
        }
    }
}

#[test]
fn terminal_original_and_coalesced_ids_replay_stored_success_and_failure() {
    for stored_id in ["run-1", "coalesced-parent"] {
        for (status, detail, completion) in [
            ("ok", None, FireCompletion::succeeded()),
            ("error", Some("stored failure"), FireCompletion::failed("stored failure")),
            ("error", None, FireCompletion::failed("Automation run failed on the Sand box")),
        ] {
            let mut target = entry(json!({"type":"cron","schedule":"0 * * * *"}));
            target.automation.is_enabled = false;
            target.automation.last_run_at = Some(101.0);
            target.automation.runs.push(AutomationRun {
                id: stored_id.into(),
                trigger: "schedule".into(),
                started_at: 1.0,
                finished_at: Some(2.0),
                status: status.into(),
                detail: detail.map(str::to_owned),
                event: None,
                coalesced_run_ids: (stored_id != "run-1").then(|| vec!["run-1".into()]),
            });
            let mut backend_fire = fire(Some(json!({"source":"unrecognized"})));
            backend_fire.definition_revision = Some("old-revision".into());
            assert_eq!(
                prepare_backend_fire(&[target], &backend_fire, |_| {
                    panic!("stored terminal replay must precede revision lookup")
                }, fixture_history),
                PreparedBackendFire::Complete { completion, reason: "existing_run" },
                "status={status}, stored_id={stored_id}",
            );
        }
    }
}

#[test]
fn stale_or_unverifiable_definition_never_executes() {
    let target = entry(json!({"type":"cron","schedule":"0 * * * *"}));
    let mut revisioned = fire(None);
    revisioned.scheduled_for_ms = Some(90);
    revisioned.definition_revision = Some("remote-hash".into());

    assert_eq!(
        prepare_backend_fire(&[target.clone()], &revisioned, |_| None, fixture_history),
        PreparedBackendFire::Abandon {
            reason: "definition_revision_unavailable"
        }
    );
    assert!(matches!(
        prepare_backend_fire(&[target], &revisioned, |_| Some("local-hash".into()), fixture_history),
        PreparedBackendFire::Complete {
            reason: "definition_changed",
            ..
        }
    ));
}

#[test]
fn legacy_schedule_slot_covered_by_last_run_is_rejected() {
    let mut target = entry(json!({"type":"cron","schedule":"0 * * * *"}));
    target.automation.last_run_at = Some(101.0);
    let mut backend_fire = fire(None);
    backend_fire.scheduled_for_ms = Some(100);
    assert!(matches!(
        prepare_backend_fire(&[target], &backend_fire, |_| None, fixture_history),
        PreparedBackendFire::Complete {
            reason: "slot_already_covered",
            ..
        }
    ));
}

#[test]
fn checked_history_result_governs_delivery_independently_of_display_projection() {
    let target = entry(json!({"type":"cron","schedule":"0 * * * *"}));
    let retained = AutomationRun {
        id: "hidden-parent".into(), trigger: "event".into(), started_at: 0.0,
        finished_at: None, status: "running".into(), detail: None, event: None,
        coalesced_run_ids: Some(vec!["run-1".into()]),
    };
    assert!(target.automation.runs.is_empty(), "old retained authority is outside display");
    assert_eq!(prepare_backend_fire(std::slice::from_ref(&target), &fire(None), |_| {
        panic!("known unfinished authority precedes revision")
    }, |selected, run_id| {
        assert_eq!(selected.agent_id, "agent-1");
        assert_eq!(selected.automation.id, "local-1");
        assert_eq!(run_id, "run-1");
        Ok(Some(retained.clone()))
    }), PreparedBackendFire::Abandon { reason: "existing_run_unsettled" });

    let mut stale_display = target;
    stale_display.automation.runs.push(retained);
    assert!(matches!(prepare_backend_fire(&[stale_display], &fire(None), |_| None, |_, _| Ok(None)),
        PreparedBackendFire::Schedule { .. }), "only the supplied checked authority establishes absence");
}

#[test]
fn unavailable_checked_history_never_falls_back_to_display_or_definition_rejection() {
    for case in ["enabled", "disabled", "covered_slot", "changed_revision", "unrecognized_event"] {
        let mut target = entry(json!({"type":"cron","schedule":"0 * * * *"}));
        target.automation.runs.push(AutomationRun {
            id: "run-1".into(), trigger: "schedule".into(), started_at: 1.0,
            finished_at: Some(2.0), status: "ok".into(), detail: None, event: None,
            coalesced_run_ids: None,
        });
        let mut backend_fire = fire(None);
        match case {
            "disabled" => target.automation.is_enabled = false,
            "covered_slot" => target.automation.last_run_at = Some(101.0),
            "changed_revision" => backend_fire.definition_revision = Some("old".into()),
            "unrecognized_event" => backend_fire.event = Some(json!({"source":"unrecognized"})),
            _ => {}
        }
        let before = target.clone();
        assert_eq!(prepare_backend_fire(std::slice::from_ref(&target), &backend_fire, |_| {
            panic!("unavailable history must precede revision: {case}")
        }, |_, run_id| {
            assert_eq!(run_id, "run-1");
            Err("checked input is unreadable, malformed or ambiguous".into())
        }), PreparedBackendFire::Abandon { reason: "run_history_unavailable" }, "{case}");
        assert_eq!(target, before);
    }
}

#[test]
fn slack_channel_id_alias_can_match_trigger() {
    let parsed = parse_fire_trigger_event(
        &json!({
            "source":"slack",
            "channel":"#alerts",
            "channelId":"C123",
            "sender":"@u",
            "text":"deploy",
            "timestampMs":1
        }),
        1,
    )
    .expect("slack event");
    assert!(fire_event_matches_trigger(
        &json!({
            "type":"slack",
            "channel":"C123",
            "match":{"kind":"message"}
        }),
        &parsed,
        &json!({
            "source":"slack",
            "channel":"#alerts",
            "channelId":"C123",
            "sender":"@u",
            "text":"deploy"
        }),
    ));
}

#[test]
fn bounded_platform_event_parsing_rejects_oversized_or_incomplete_payloads() {
    assert!(parse_fire_trigger_event(
        &json!({
            "source":"microsoftTeams",
            "tenantId":"t",
            "teamId":"team",
            "channelId":"c",
            "text":"hello"
        }),
        10,
    )
    .is_some());

    assert!(parse_fire_trigger_event(
        &json!({
            "source":"microsoftTeams",
            "tenantId":"x".repeat(201),
            "teamId":"team",
            "channelId":"c"
        }),
        10,
    )
    .is_none());

    assert!(parse_fire_trigger_event(
        &json!({
            "source":"pagerduty",
            "event":"incidentTriggered",
            "incidentId":"i",
            "title":"title",
            "status":"triggered"
        }),
        10,
    )
    .is_none());
}

#[test]
fn bare_fire_requires_schedule_and_disambiguates_mixed_triggers() {
    let mut backend_fire = fire(None);
    let schedule_only = json!({"type":"cron","schedule":"0 * * * *"});
    assert!(admits_bare_fire(&schedule_only, &backend_fire));

    let mixed = json!({
        "type":"group",
        "listeners":[
            {"type":"cron","schedule":"0 * * * *"},
            {"type":"github","repo":"openai/repo","events":["pr-opened"]}
        ]
    });
    assert!(!admits_bare_fire(&mixed, &backend_fire));
    backend_fire.scheduled_for_ms = Some(100);
    assert!(admits_bare_fire(&mixed, &backend_fire));
}
