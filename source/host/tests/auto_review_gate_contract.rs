use std::sync::Arc;

use mahayana_host_runtime::runner::auto_review_gate::{
    AutoReviewGate, AutoReviewGateDependencies, AutoReviewInstructions,
    ShellApprovalSurface,
};
use mahayana_host_runtime::runner::sand_auto_review::{
    SandAutoReviewController, SandAutoReviewMode, SandAutoReviewModes,
    SandAutoReviewRequest, SandAutoReviewRequestOutcome, SandAutoReviewSurface,
};

struct Deps {
    controller: Arc<SandAutoReviewController>,
}

impl AutoReviewGateDependencies for Deps {
    fn base_modes(&self) -> SandAutoReviewModes {
        SandAutoReviewModes {
            host_shell: SandAutoReviewMode::Enforce,
            box_shell: SandAutoReviewMode::Shadow,
            mcp: SandAutoReviewMode::Off,
            computer: SandAutoReviewMode::Enforce,
            automation_write: SandAutoReviewMode::Off,
            cloud_agent: SandAutoReviewMode::Enforce,
            subagent_launch: SandAutoReviewMode::Off,
        }
    }

    fn controller(&self) -> Option<Arc<SandAutoReviewController>> {
        Some(self.controller.clone())
    }

    fn resolve_box_id(&self) -> String {
        "box-7".into()
    }

    fn instructions(&self) -> Option<AutoReviewInstructions> {
        Some(AutoReviewInstructions {
            allow_instructions: vec!["allow signed releases".into()],
            block_instructions: vec!["block destructive reset".into()],
        })
    }
}

#[test]
fn auto_review_gate_expires_every_non_enforcing_surface() {
    let controller = Arc::new(SandAutoReviewController::new("agent-1", "host-1"));
    let gate = AutoReviewGate::new(Arc::new(Deps {
        controller: controller.clone(),
    }));

    let modes = gate.current_modes();
    assert_eq!(modes.host_shell, SandAutoReviewMode::Enforce);

    assert!(controller.get_pending_approvals().is_empty());
}

#[test]
fn auto_review_gate_blocks_new_side_effect_when_approval_is_pending() {
    let controller = Arc::new(SandAutoReviewController::new("agent-1", "host-1"));
    let pending = controller.request_approval(SandAutoReviewRequest {
        agent_id: None,
        surface: SandAutoReviewSurface::HostShell,
        fingerprint: "fingerprint".into(),
        reason: "needs review".into(),
        summary: "sensitive action".into(),
        command: None,
        proposed_rule: None,
        expiry_policy: None,
    });
    assert!(matches!(pending, SandAutoReviewRequestOutcome::Pending(_)));
    let gate = AutoReviewGate::new(Arc::new(Deps { controller }));

    let error = gate
        .assert_no_pending_approval()
        .expect_err("pending approval blocks the next side effect");
    assert_eq!(
        error.to_string(),
        "Another action is waiting for Auto-review approval; no new side effect may start yet."
    );
}

#[test]
fn shell_approval_identity_changes_only_after_side_effect_start() {
    let controller = Arc::new(SandAutoReviewController::new("agent-1", "host-1"));
    let gate = AutoReviewGate::new(Arc::new(Deps { controller }));

    assert_eq!(
        gate.shell_approval_identity(ShellApprovalSurface::HostShell),
        "host_shell:0"
    );
    assert_eq!(
        gate.shell_approval_identity(ShellApprovalSurface::BoxShell),
        "box_shell:box-7:0"
    );

    gate.mark_shell_side_effect_start(ShellApprovalSurface::BoxShell);

    assert_eq!(
        gate.shell_approval_identity(ShellApprovalSurface::HostShell),
        "host_shell:0"
    );
    assert_eq!(
        gate.shell_approval_identity(ShellApprovalSurface::BoxShell),
        "box_shell:box-7:1"
    );
}

#[test]
fn user_instructions_are_cloned_from_live_dependencies() {
    let controller = Arc::new(SandAutoReviewController::new("agent-1", "host-1"));
    let gate = AutoReviewGate::new(Arc::new(Deps { controller }));

    let instructions = gate.user_instructions().expect("instructions");
    assert_eq!(instructions.allow_instructions, vec!["allow signed releases"]);
    assert_eq!(
        instructions.block_instructions,
        vec!["block destructive reset"]
    );
}
