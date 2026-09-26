use super::sand_automation_cloud_trigger::{BackendCloudTrigger, backend_cloud_trigger};
use super::sand_trigger_hub::ScheduledAutomation;

#[derive(Debug, Clone, PartialEq)]
pub struct DesiredCloudTrigger {
    pub agent_id: String,
    pub automation_id: String,
    pub trigger: BackendCloudTrigger,
}

pub fn desired_cloud_triggers(
    scheduled: &[ScheduledAutomation],
    should_sync: impl Fn(&str, &ScheduledAutomation) -> bool,
) -> Vec<DesiredCloudTrigger> {
    let mut desired = scheduled.iter()
        .filter(|entry| entry.is_enabled)
        .filter(|entry| should_sync(&entry.agent_id, entry))
        .filter_map(|entry| {
            backend_cloud_trigger(&entry.trigger).map(|trigger| DesiredCloudTrigger {
                agent_id: entry.agent_id.clone(),
                automation_id: entry.automation_id.clone(),
                trigger,
            })
        })
        .collect::<Vec<_>>();
    desired.sort_by(|left, right| {
        left.agent_id.cmp(&right.agent_id)
            .then_with(|| left.automation_id.cmp(&right.automation_id))
    });
    desired
}
