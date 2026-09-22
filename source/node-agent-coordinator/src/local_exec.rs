#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct LocalExecSupervisor {
    generation: u64,
    running: bool,
    restart_count: u32,
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
        self.start()
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn is_running(&self) -> bool { self.running }
    pub fn restart_count(&self) -> u32 { self.restart_count }
}
