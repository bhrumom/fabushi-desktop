use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use mahayana_host_runtime::runner::auto_review_gate::{
    AutoReviewGate, AutoReviewGateController, AutoReviewGateDependencies,
    AutoReviewInstructions, AutoReviewModes, SandAutoReviewMode,
    ShellApprovalSurface,
};

#[derive(Default)]
struct Controller {
    expired: Mutex<Vec<BTreeSet<&'static str>>>,
    pending: Mutex<usize>,
}

impl AutoReviewGateController for Controller {
    fn expire_surfaces(&self, surfaces: &BTreeSet<&'static str>) {
        self.expired
            .lock()
            .expect("expired")
            .push(surfaces.clone());
    }

    fn pending_approval_count(&self) -> usize {
        *self.pending.lock().expect("pending")
    }
}

struct Deps {
    controller: Arc<Controller>,
}

impl AutoReviewGateDependencies for Deps {
    fn base_modes(&self) -> AutoReviewModes {
        AutoReviewModes {
            host_shell: SandAutoReviewMode::Enforce,
            box_shell: SandAutoReviewMode::Ask,
            mcp: SandAutoReviewMode::Off,
            computer: SandAutoReviewMode::Enforce,
            automation_write: SandAutoReviewMode::Ask,
            cloud_agent: SandAutoReviewMode::Enforce,
            subagent_launch: SandAutoReviewMode::Off,
        }
    }

    fn controller(&self) -> Option<Arc<dyn AutoReviewGateController>> {
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
    let controller = Arc::new(Controller::default());
    let gate = AutoReviewGate::new(Arc::new(Deps {
        controller: controller.clone(),
    }));

    let modes = gate.current_modes();
    assert_eq!(modes.host_shell, SandAutoReviewMode::Enforce);

    let expired = controller.expired.lock().expect("expired");
    assert_eq!(expired.len(), 1);
    assert_eq!(
        expired[0],
        BTreeSet::from([
            "automation_write",
            "box_shell",
            "mcp",
            "subagent",
        ])
    );
}

#[test]
fn auto_review_gate_blocks_new_side_effect_when_approval_is_pending() {
    let controller = Arc::new(Controller::default());
    *controller.pending.lock().expect("pending") = 1;
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
    let controller = Arc::new(Controller::default());
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
    let controller = Arc::new(Controller::default());
    let gate = AutoReviewGate::new(Arc::new(Deps { controller }));

    let instructions = gate.user_instructions().expect("instructions");
    assert_eq!(instructions.allow_instructions, vec!["allow signed releases"]);
    assert_eq!(
        instructions.block_instructions,
        vec!["block destructive reset"]
    );
}
