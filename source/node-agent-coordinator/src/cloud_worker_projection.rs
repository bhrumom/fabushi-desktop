use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const CLOUD_WORKERS_STATUS_METHOD: &str = "cloudWorkers.status";
pub const CLOUD_WORKERS_STATUS_EVENT_FAMILY: &str = "cloud-workers.status";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CloudWorkerCoordinatorCommand {
    Status,
}

impl CloudWorkerCoordinatorCommand {
    pub const fn method(self) -> &'static str {
        match self {
            Self::Status => CLOUD_WORKERS_STATUS_METHOD,
        }
    }

    pub fn from_method(method: &str) -> Option<Self> {
        match method {
            CLOUD_WORKERS_STATUS_METHOD => Some(Self::Status),
            _ => None,
        }
    }

    pub fn args(self) -> Value {
        match self {
            Self::Status => Value::Object(Default::default()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudWorkerStatusProjection {
    pub owner: String,
    pub status: String,
    #[serde(rename = "providerReady")]
    pub provider_ready: bool,
    #[serde(rename = "productionMutation")]
    pub production_mutation: bool,
}

impl CloudWorkerStatusProjection {
    pub fn is_disabled(&self) -> bool {
        self.status == "disabled"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum CloudWorkerCoordinatorEvent {
    StatusChanged {
        projection: CloudWorkerStatusProjection,
    },
}

impl CloudWorkerCoordinatorEvent {
    pub const fn family(&self) -> &'static str {
        CLOUD_WORKERS_STATUS_EVENT_FAMILY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_gateway_method_and_coordinator_command_identical() {
        assert_eq!(
            CloudWorkerCoordinatorCommand::Status.method(),
            "cloudWorkers.status"
        );
        assert_eq!(
            CloudWorkerCoordinatorCommand::from_method("cloudWorkers.status"),
            Some(CloudWorkerCoordinatorCommand::Status)
        );
        assert_eq!(
            CloudWorkerCoordinatorCommand::Status.args(),
            serde_json::json!({})
        );
    }

    #[test]
    fn status_projection_preserves_disabled_boundary() {
        let projection = CloudWorkerStatusProjection {
            owner: "cloud-workers".into(),
            status: "disabled".into(),
            provider_ready: false,
            production_mutation: false,
        };
        assert!(projection.is_disabled());
        let event = CloudWorkerCoordinatorEvent::StatusChanged { projection };
        assert_eq!(event.family(), "cloud-workers.status");
    }
}
