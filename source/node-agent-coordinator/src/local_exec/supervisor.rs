use super::daemon_files::LocalExecDaemonDiscovery;

pub const LOCAL_EXEC_DAEMON_REFRESH_INTERVAL_MS: u64 = 30_000;
pub const LOCAL_EXEC_DAEMON_LIVENESS_INTERVAL_MS: u64 = 1_000;
pub const LOCAL_EXEC_DAEMON_READINESS_TIMEOUT_MS: u64 = 5_000;
pub const LOCAL_EXEC_DAEMON_READINESS_POLL_MS: u64 = 50;
pub const LOCAL_EXEC_DAEMON_DISCOVERY_MISS_LIMIT: u32 = 2;
pub const LOCAL_EXEC_DAEMON_RESPAWN_LIMIT: u32 = 10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalExecDaemonAction {
    Spawn,
    Adopt { pid: u32 },
    Replace { pid: u32 },
}

pub fn decide_local_exec_daemon_action(
    existing: Option<&LocalExecDaemonDiscovery>,
) -> LocalExecDaemonAction {
    match existing {
        None => LocalExecDaemonAction::Spawn,
        Some(existing) if existing.inflight_count.unwrap_or(0) > 0 => {
            LocalExecDaemonAction::Adopt { pid: existing.pid }
        }
        Some(existing) => LocalExecDaemonAction::Replace { pid: existing.pid },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalExecDaemonOrigin {
    Spawned,
    Adopted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalExecDaemonState {
    Absent,
    Adopting { pid: u32 },
    Replacing { pid: u32 },
    Active {
        origin: LocalExecDaemonOrigin,
        pid: u32,
        process_start_epoch_ms: u64,
        command: String,
        entry_realpath: String,
        generation_token: String,
    },
    Failed { reason: String },
}

impl Default for LocalExecDaemonState {
    fn default() -> Self {
        Self::Absent
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct LocalExecSupervisor {
    generation: u64,
    running: bool,
    restart_count: u32,
    state: LocalExecDaemonState,
}

impl LocalExecSupervisor {
    pub fn start(&mut self) -> u64 {
        self.generation = self.generation.saturating_add(1);
        self.running = true;
        self.generation
    }

    pub fn crash_and_restart(&mut self) -> u64 {
        self.running = false;
        self.restart_count = self.restart_count.saturating_add(1);
        if self.restart_count > LOCAL_EXEC_DAEMON_RESPAWN_LIMIT {
            self.state = LocalExecDaemonState::Failed {
                reason: "respawn limit reached".to_string(),
            };
            return self.generation;
        }
        self.start()
    }

    pub fn stop(&mut self) {
        self.running = false;
        self.state = LocalExecDaemonState::Absent;
    }

    pub fn reconcile_discovery(
        &mut self,
        existing: Option<&LocalExecDaemonDiscovery>,
    ) -> LocalExecDaemonAction {
        let action = decide_local_exec_daemon_action(existing);
        self.state = match action {
            LocalExecDaemonAction::Spawn => LocalExecDaemonState::Absent,
            LocalExecDaemonAction::Adopt { pid } => LocalExecDaemonState::Adopting { pid },
            LocalExecDaemonAction::Replace { pid } => LocalExecDaemonState::Replacing { pid },
        };
        action
    }

    pub fn mark_active(
        &mut self,
        origin: LocalExecDaemonOrigin,
        pid: u32,
        process_start_epoch_ms: u64,
        command: impl Into<String>,
        entry_realpath: impl Into<String>,
        generation_token: impl Into<String>,
    ) -> Result<(), &'static str> {
        let command = command.into();
        let entry_realpath = entry_realpath.into();
        let generation_token = generation_token.into();
        if pid == 0
            || process_start_epoch_ms == 0
            || command.trim().is_empty()
            || entry_realpath.trim().is_empty()
            || generation_token.trim().is_empty()
        {
            return Err("active local-exec daemon identity is incomplete");
        }
        self.running = true;
        self.state = LocalExecDaemonState::Active {
            origin,
            pid,
            process_start_epoch_ms,
            command,
            entry_realpath,
            generation_token,
        };
        Ok(())
    }

    pub fn mark_failed(&mut self, reason: impl Into<String>) {
        self.running = false;
        self.state = LocalExecDaemonState::Failed {
            reason: reason.into(),
        };
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn restart_count(&self) -> u32 {
        self.restart_count
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn state(&self) -> &LocalExecDaemonState {
        &self.state
    }
}
