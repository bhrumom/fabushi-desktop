use std::collections::BTreeMap;
use std::sync::Arc;

use mahayana_host_runtime::runner::sand_auto_review::{
    SandAutoReviewController, SandAutoReviewEvent, SandAutoReviewExpiryPolicy,
    SandAutoReviewMode, SandAutoReviewResolution,
};
use mahayana_host_runtime::runner::sand_auto_review_classifier_run::{
    AutoReviewClassifierDecision, AutoReviewClassifierError,
};
use mahayana_host_runtime::runner::sand_auto_review_tool_escalations::{
    McpApprovalRequest, ShellApprovalRequest, ShellApprovalTarget,
    request_sand_mcp_approval, request_sand_shell_approval,
};
use mahayana_host_runtime::runner::sand_automation_auto_review::{
    AutomationReference, AutomationReviewOutcome, AutomationWriteTarget,
    build_sand_automation_write_risk_target, review_sand_automation_write,
};
use mahayana_host_runtime::runner::sand_browser_auto_review::{
    SandBrowserReviewState, build_sand_browser_auto_review_canonical_target,
    is_sand_browser_auto_review_mutating_action, run_sand_browser_auto_review_preflight,
};
use mahayana_host_runtime::runner::sand_cloud_agent_auto_review::{
    CloudAgentReviewImage, CloudAgentReviewOutcome,
    build_sand_cloud_agent_lifecycle_review_target, build_sand_cloud_agent_review_target,
    describe_sand_cloud_agent_review_images, review_sand_cloud_agent_lifecycle_action,
};
use mahayana_host_runtime::runner::sand_computer_auto_review::{
    BoxIdentity, compute_sand_computer_page_state_identity,
    run_sand_computer_auto_review_preflight,
};
use mahayana_host_runtime::runner::sand_shell_auto_review_enrichment::{
    SandShellEnrichmentCandidate, SandShellReadAccessor, SandShellTargetEnrichment,
    ShellReadResult, build_sand_shell_auto_review_target_enrichment,
    parse_sand_shell_enrichment_candidate,
};
use mahayana_host_runtime::runner::sand_subagent_auto_review::{
    build_sand_subagent_launch_review_target, build_sand_subagent_risk_target,
};
use mahayana_host_runtime::runner::sand_auto_review_summaries::CloudLifecycleAction;
use serde_json::{Value, json};

fn approving_controller() -> SandAutoReviewController {
    let controller = SandAutoReviewController::new("agent-a", "generation-a");
    let resolver = controller.clone();
    controller.subscribe(Arc::new(move |event| {
        if let SandAutoReviewEvent::Created(approval) = event {
            let _ = resolver.resolve_approval(
                &approval.id,
                SandAutoReviewResolution::Approved,
            );
        }
    }));
    controller
}

#[test]
fn automation_review_preserves_off_shadow_enforce_and_approval_contract() {
    let target = AutomationWriteTarget {
        operation: "create".into(),
        name: "Morning brief".into(),
        trigger_description: "Every weekday at 8 AM".into(),
        prompt: "Summarize inbox".into(),
        is_enabled: Some(false),
        referencing_routines: vec![AutomationReference {
            id: "routine-a".into(),
            name: "Daily".into(),
        }],
    };
    let risk = build_sand_automation_write_risk_target(&target);
    assert_eq!(risk["action"], "sand_automation_write");
    assert_eq!(risk["arguments"]["name"], "Morning brief");

    let mut shadow_calls = 0;
    assert_eq!(
        review_sand_automation_write(
            SandAutoReviewMode::Shadow,
            &target,
            None,
            "turn",
            |_, mode| {
                assert_eq!(mode, "shadow");
                shadow_calls += 1;
                Ok(AutoReviewClassifierDecision::Block {
                    reason: "blocked".into(),
                    proposed_rule: None,
                })
            },
        )
        .expect("shadow review"),
        AutomationReviewOutcome::Allowed
    );
    assert_eq!(shadow_calls, 1);

    let controller = approving_controller();
    assert_eq!(
        review_sand_automation_write(
            SandAutoReviewMode::Enforce,
            &target,
            Some(&controller),
            "turn",
            |_, mode| {
                assert_eq!(mode, "enforce");
                Ok(AutoReviewClassifierDecision::Block {
                    reason: "Needs approval".into(),
                    proposed_rule: Some("Allow scheduled inbox summaries".into()),
                })
            },
        )
        .expect("approved automation"),
        AutomationReviewOutcome::Allowed
    );
}

#[test]
fn computer_and_browser_preflight_bind_approval_to_display_state() {
    let box_identity = BoxIdentity {
        box_id: "box-a".into(),
        window_generation: "window-7".into(),
    };
    assert_eq!(
        compute_sand_computer_page_state_identity("b\na\n"),
        compute_sand_computer_page_state_identity("a\nb\n")
    );
    let controller = approving_controller();
    run_sand_computer_auto_review_preflight(
        SandAutoReviewMode::Enforce,
        &json!({"action":"click","x":20,"y":30}),
        Some("Open settings"),
        &box_identity,
        "agent-a",
        "turn",
        Some(&controller),
        || Ok("screen-1".into()),
        |target, mode| {
            assert_eq!(mode, "enforce");
            assert_eq!(target["action"], "sand_computer");
            Ok(AutoReviewClassifierDecision::Block {
                reason: "Review click".into(),
                proposed_rule: None,
            })
        },
    )
    .expect("approved computer click");

    assert!(!is_sand_browser_auto_review_mutating_action(
        &json!({"op":"screenshot"})
    ));
    assert!(is_sand_browser_auto_review_mutating_action(
        &json!({"op":"click","element":"Settings"})
    ));
    let state = SandBrowserReviewState {
        display_state_identity: "dom-1".into(),
        target_page_url: Some("https://example.test".into()),
    };
    let canonical = build_sand_browser_auto_review_canonical_target(
        &json!({"op":"click","element":" Settings "}),
        &box_identity,
        &state,
    )
    .expect("canonical browser target");
    assert_eq!(canonical["exact_action"]["element"], "Settings");
    run_sand_browser_auto_review_preflight(
        SandAutoReviewMode::Enforce,
        &json!({"op":"click","element":"Settings"}),
        &box_identity,
        "agent-a",
        "turn",
        Some(&controller),
        || Ok(state.clone()),
        |target, _| {
            assert_eq!(target["action"], "sand_computer");
            Ok(AutoReviewClassifierDecision::Allow)
        },
    )
    .expect("allowed browser click");
}

#[test]
fn cloud_and_subagent_targets_preserve_hash_identity_and_lifecycle_guard() {
    let images = vec![CloudAgentReviewImage {
        name: "screen.png".into(),
        data: b"abc".to_vec(),
    }];
    let described = describe_sand_cloud_agent_review_images(&images);
    assert_eq!(
        described[0]["sha256"],
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    let target = build_sand_cloud_agent_review_target(
        "reply",
        "Inspect the trace",
        Some("cloud-7"),
        &images,
        true,
        None,
        None,
    )
    .expect("cloud target");
    assert_eq!(target.action, "reply");
    let lifecycle = build_sand_cloud_agent_lifecycle_review_target(
        "delete",
        "cloud-7",
        None,
    )
    .expect("lifecycle");
    assert!(matches!(
        review_sand_cloud_agent_lifecycle_action(
            SandAutoReviewMode::Enforce,
            CloudLifecycleAction::Delete,
            &lifecycle,
            None,
            "turn",
            || false,
        )
        .expect("lifecycle review"),
        CloudAgentReviewOutcome::Blocked(_)
    ));

    let subagent =
        build_sand_subagent_launch_review_target("Inspect the trace", Some("browserUse"))
            .expect("subagent");
    let risk = build_sand_subagent_risk_target(&subagent, None, None, None);
    assert_eq!(risk["action"], "sand_subagent");
    assert_eq!(risk["arguments"]["subagent_type"], "browserUse");
}

#[test]
fn shell_and_mcp_escalations_share_controller_and_summary_contract() {
    let controller = approving_controller();
    let shell = request_sand_shell_approval(
        &controller,
        &ShellApprovalRequest {
            target: ShellApprovalTarget {
                surface: mahayana_host_runtime::runner::sand_auto_review::SandAutoReviewSurface::HostShell,
                description: Some("List release artifacts.".into()),
                working_directory: Some("/tmp/build".into()),
            },
            fingerprint: "shell-fingerprint".into(),
            reason: "Review local command".into(),
            command: "ls -la".into(),
            proposed_rule: None,
            expiry_policy: SandAutoReviewExpiryPolicy::Park,
        },
        |_| Ok(()),
        |_| Ok(()),
    )
    .expect("shell approval");
    assert_eq!(
        shell,
        mahayana_host_runtime::runner::sand_auto_review::SandAutoReviewDecision::Approved
    );

    let mcp = request_sand_mcp_approval(
        &controller,
        &McpApprovalRequest {
            fingerprint: "mcp-fingerprint".into(),
            reason: "Review external post".into(),
            server_display_name: "Slack".into(),
            tool_name: "sendMessage".into(),
            mcp_arguments: Some(json!({"channel_id":"release-room"})),
            description: Some("Post the release notice.".into()),
            proposed_rule: None,
            expiry_policy: SandAutoReviewExpiryPolicy::Park,
        },
    )
    .expect("mcp approval");
    assert_eq!(
        mcp,
        mahayana_host_runtime::runner::sand_auto_review::SandAutoReviewDecision::Approved
    );
}

struct MemoryReadAccessor {
    files: BTreeMap<String, String>,
}

impl SandShellReadAccessor for MemoryReadAccessor {
    fn read(
        &self,
        path: &str,
        _tool_call_id: &str,
        offset: usize,
        limit: usize,
    ) -> Result<Option<ShellReadResult>, String> {
        let Some(raw) = self.files.get(path) else {
            return Ok(None);
        };
        let lines = raw.lines().collect::<Vec<_>>();
        let start = offset.saturating_sub(1).min(lines.len());
        let end = start.saturating_add(limit).min(lines.len());
        let content = if start == end {
            String::new()
        } else {
            let mut value = lines[start..end].join("\n");
            if raw.ends_with('\n') {
                value.push('\n');
            }
            value
        };
        Ok(Some(ShellReadResult {
            content,
            total_lines: lines.len().max(1),
            truncated: false,
        }))
    }
}

#[test]
fn shell_enrichment_resolves_package_scripts_and_binds_definition_hash() {
    assert_eq!(
        parse_sand_shell_enrichment_candidate("cd tools && npm run build", Some("/repo")),
        Some(SandShellEnrichmentCandidate::PackageScript {
            package_json_path: "/repo/tools/package.json".into(),
            script_name: "build".into(),
        })
    );
    let accessor = MemoryReadAccessor {
        files: BTreeMap::from([(
            "/repo/tools/package.json".into(),
            "{\"scripts\":{\"build\":\"vite build\"}}\n".into(),
        )]),
    };
    let enrichment = build_sand_shell_auto_review_target_enrichment(
        "cd tools && npm run build",
        Some("/repo"),
        &accessor,
        "tool-1",
    )
    .expect("enrichment");
    match enrichment {
        SandShellTargetEnrichment::PackageScript {
            path,
            name,
            definition,
            definition_hash,
        } => {
            assert_eq!(path, "/repo/tools/package.json");
            assert_eq!(name, "build");
            assert_eq!(definition, "vite build");
            assert_eq!(definition_hash.len(), 64);
        }
        other => panic!("unexpected enrichment: {other:?}"),
    }
}

#[test]
fn specialized_classifiers_surface_transport_errors_without_guessing() {
    let target = AutomationWriteTarget {
        operation: "update".into(),
        name: "Daily".into(),
        trigger_description: "every day".into(),
        prompt: "Summarize".into(),
        is_enabled: Some(true),
        referencing_routines: vec![],
    };
    let error = review_sand_automation_write(
        SandAutoReviewMode::Enforce,
        &target,
        None,
        "turn",
        |_, _| Err(AutoReviewClassifierError::Aborted("cancelled".into())),
    )
    .expect_err("abort must propagate");
    assert_eq!(
        error,
        AutoReviewClassifierError::Aborted("cancelled".into())
    );
}
