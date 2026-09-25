use std::collections::HashSet;
use std::sync::Arc;

pub trait AttachmentRosterPort: Send + Sync {
    fn set_fallback_agent_id(&self, agent_id: Option<&str>);
}

pub trait TranscriptRosterPort: Send + Sync {
    fn live_running_agent_ids(&self) -> Vec<String>;
}

pub trait ForeverBoxRosterPort: Send + Sync {
    fn enroll_disk_pressure_reminder(&self, agent_ids: &HashSet<String>);
    fn set_busy(&self, is_busy: bool);
}

pub trait SnapshotBackstopPort: Send + Sync {
    fn is_enabled(&self) -> bool;
    fn schedule_snapshot(&self, agent_id: &str);
}

pub trait BoxStoreSnapshotPort: Send + Sync {
    fn is_enabled(&self) -> bool;
    fn schedule_store_db_snapshot(&self, agent_id: &str);
}

pub trait SourceMapRosterPort: Send + Sync {
    fn get_or_create(&self, agent_id: &str);
}

pub struct HostRosterPorts {
    pub attachments: Arc<dyn AttachmentRosterPort>,
    pub transcript: Arc<dyn TranscriptRosterPort>,
    pub forever_box: Arc<dyn ForeverBoxRosterPort>,
    pub state_backstop: Arc<dyn SnapshotBackstopPort>,
    pub box_store_sync: Arc<dyn BoxStoreSnapshotPort>,
    pub source_map: Arc<dyn SourceMapRosterPort>,
}

pub struct HostRosterBookkeeping {
    ports: HostRosterPorts,
    latest_active_agent_id: Option<String>,
    is_busy: bool,
    running_agent_ids: HashSet<String>,
}

impl HostRosterBookkeeping {
    pub fn new(ports: HostRosterPorts) -> Self {
        Self {
            ports,
            latest_active_agent_id: None,
            is_busy: false,
            running_agent_ids: HashSet::new(),
        }
    }

    pub fn latest_active_agent_id(&self) -> Option<&str> {
        self.latest_active_agent_id.as_deref()
    }

    pub fn is_busy(&self) -> bool {
        self.is_busy
    }

    pub fn running_agent_ids(&self) -> HashSet<String> {
        self.running_agent_ids.clone()
    }

    pub fn apply(&mut self, active_agent_id: Option<&str>) {
        let normalized = active_agent_id
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        self.latest_active_agent_id = normalized.clone();
        self.ports.attachments.set_fallback_agent_id(normalized.as_deref());

        let previously_running = self.running_agent_ids.clone();
        self.running_agent_ids = self
            .ports
            .transcript
            .live_running_agent_ids()
            .into_iter()
            .collect();
        self.is_busy = !self.running_agent_ids.is_empty();

        let started = self
            .running_agent_ids
            .difference(&previously_running)
            .cloned()
            .collect::<HashSet<_>>();
        if !started.is_empty() {
            self.ports
                .forever_box
                .enroll_disk_pressure_reminder(&started);
        }
        self.ports.forever_box.set_busy(self.is_busy);

        if self.ports.state_backstop.is_enabled()
            || self.ports.box_store_sync.is_enabled()
        {
            for agent_id in previously_running.difference(&self.running_agent_ids) {
                self.ports.state_backstop.schedule_snapshot(agent_id);
                self.ports
                    .box_store_sync
                    .schedule_store_db_snapshot(agent_id);
            }
        }

        if let Some(agent_id) = normalized.as_deref() {
            self.ports.source_map.get_or_create(agent_id);
        }
    }
}
