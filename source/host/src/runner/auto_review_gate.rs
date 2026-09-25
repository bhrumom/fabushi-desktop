use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandAutoReviewMode {
    Off,
    Ask,
    Enforce,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutoReviewModes {
    pub host_shell: SandAutoReviewMode,
    pub box_shell: SandAutoReviewMode,
    pub mcp: SandAutoReviewMode,
    pub computer: SandAutoReviewMode,
    pub automation_write: SandAutoReviewMode,
    pub cloud_agent: SandAutoReviewMode,
    pub subagent_launch: SandAutoReviewMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AutoReviewInstructions {
    pub allow_instructions: Vec<String>,
    pub block_instructions: Vec<String>,
}

pub trait AutoReviewGateController: Send + Sync {
    fn expire_surfaces(&self, non_enforcing_surfaces: &BTreeSet<&'static str>);
    fn pending_approval_count(&self) -> usize;
}

pub trait AutoReviewGateDependencies: Send + Sync {
    fn base_modes(&self) -> AutoReviewModes;
    fn current_modes(&self) -> Option<AutoReviewModes> {
        None
    }
    fn controller(&self) -> Option<Arc<dyn AutoReviewGateController>> {
        None
    }
    fn resolve_box_id(&self) -> String;
    fn instructions(&self) -> Option<AutoReviewInstructions> {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Another action is waiting for Auto-review approval; no new side effect may start yet.")]
pub struct SandAutoReviewPendingApprovalError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellApprovalSurface {
    HostShell,
    BoxShell,
}

impl ShellApprovalSurface {
    fn as_str(self) -> &'static str {
        match self {
            Self::HostShell => "host_shell",
            Self::BoxShell => "box_shell",
        }
    }
}

#[derive(Debug, Default)]
struct ShellApprovalGeneration {
    host_shell: u64,
    box_shell: u64,
}

pub struct AutoReviewGate {
    deps: Arc<dyn AutoReviewGateDependencies>,
    shell_generation: Mutex<ShellApprovalGeneration>,
}

impl AutoReviewGate {
    pub fn new(deps: Arc<dyn AutoReviewGateDependencies>) -> Self {
        Self {
            deps,
            shell_generation: Mutex::new(ShellApprovalGeneration::default()),
        }
    }

    pub fn current_modes(&self) -> AutoReviewModes {
        let modes = self
            .deps
            .current_modes()
            .unwrap_or_else(|| self.deps.base_modes());
        let mut non_enforcing = BTreeSet::new();
        if modes.host_shell != SandAutoReviewMode::Enforce {
            non_enforcing.insert("host_shell");
        }
        if modes.box_shell != SandAutoReviewMode::Enforce {
            non_enforcing.insert("box_shell");
        }
        if modes.mcp != SandAutoReviewMode::Enforce {
            non_enforcing.insert("mcp");
        }
        if modes.computer != SandAutoReviewMode::Enforce {
            non_enforcing.insert("computer");
        }
        if modes.automation_write != SandAutoReviewMode::Enforce {
            non_enforcing.insert("automation_write");
        }
        if modes.cloud_agent != SandAutoReviewMode::Enforce {
            non_enforcing.insert("cloud_agent");
        }
        if modes.subagent_launch != SandAutoReviewMode::Enforce {
            non_enforcing.insert("subagent");
        }
        if let Some(controller) = self.deps.controller() {
            controller.expire_surfaces(&non_enforcing);
        }
        modes
    }

    pub fn assert_no_pending_approval(
        &self,
    ) -> Result<(), SandAutoReviewPendingApprovalError> {
        if self
            .deps
            .controller()
            .is_some_and(|controller| controller.pending_approval_count() > 0)
        {
            return Err(SandAutoReviewPendingApprovalError);
        }
        Ok(())
    }

    pub fn user_instructions(&self) -> Option<AutoReviewInstructions> {
        self.deps.instructions().filter(|instructions| {
            !instructions.allow_instructions.is_empty()
                || !instructions.block_instructions.is_empty()
        })
    }

    pub fn shell_approval_identity(&self, surface: ShellApprovalSurface) -> String {
        let generation = self
            .shell_generation
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let value = match surface {
            ShellApprovalSurface::HostShell => generation.host_shell,
            ShellApprovalSurface::BoxShell => generation.box_shell,
        };
        match surface {
            ShellApprovalSurface::HostShell => {
                format!("{}:{value}", surface.as_str())
            }
            ShellApprovalSurface::BoxShell => format!(
                "{}:{}:{value}",
                surface.as_str(),
                self.deps.resolve_box_id()
            ),
        }
    }

    pub fn mark_shell_side_effect_start(&self, surface: ShellApprovalSurface) {
        let mut generation = self
            .shell_generation
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match surface {
            ShellApprovalSurface::HostShell => {
                generation.host_shell = generation.host_shell.saturating_add(1)
            }
            ShellApprovalSurface::BoxShell => {
                generation.box_shell = generation.box_shell.saturating_add(1)
            }
        }
    }
}
