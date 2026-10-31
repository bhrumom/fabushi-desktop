use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const CLOUD_WORKERS_STATUS_METHOD: &str = "cloudWorkers.status";
pub const CLOUD_WORKERS_DOCTOR_METHOD: &str = "cloudWorkers.doctor";
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
#[serde(rename_all = "kebab-case")]
pub enum CloudWorkerCoordinatorStatus {
    Disabled,
    Enabled,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudWorkerStatusProjection {
    pub owner: String,
    pub status: CloudWorkerCoordinatorStatus,
    #[serde(rename = "providerReady")]
    pub provider_ready: bool,
    #[serde(rename = "productionMutation")]
    pub production_mutation: bool,
    #[serde(default, rename = "statePersisted")]
    pub state_persisted: bool,
    #[serde(default, rename = "leaseCount")]
    pub lease_count: usize,
    #[serde(default, rename = "warmImageCount")]
    pub warm_image_count: usize,
    #[serde(default, rename = "bootstrapCount")]
    pub bootstrap_count: usize,
    #[serde(default, rename = "artifactCount")]
    pub artifact_count: usize,
    #[serde(default, rename = "migrationReceiptCount")]
    pub migration_receipt_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudWorkerDoctorProjectionStatus {
    InspectionOnly,
    Pending,
    Completed,
    Failed,
    OutcomeUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudWorkerDoctorProjection {
    pub status: CloudWorkerDoctorProjectionStatus,
    #[serde(rename = "mutationStarted")]
    pub mutation_started: bool,
    #[serde(default, rename = "diagnosticCode")]
    pub diagnostic_code: Option<String>,
    #[serde(default, rename = "statePersisted")]
    pub state_persisted: bool,
    #[serde(default, rename = "leaseCount")]
    pub lease_count: usize,
    #[serde(default, rename = "bootstrapCount")]
    pub bootstrap_count: usize,
    #[serde(default, rename = "artifactCount")]
    pub artifact_count: usize,
    #[serde(default, rename = "warmImageCount")]
    pub warm_image_count: usize,
}

impl CloudWorkerStatusProjection {
    pub fn is_disabled(&self) -> bool {
        self.status == CloudWorkerCoordinatorStatus::Disabled
    }

    pub fn is_unavailable(&self) -> bool {
        self.status == CloudWorkerCoordinatorStatus::Unavailable
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

    /// Decode only the registered cloud-worker event family. Unknown families
    /// and malformed payloads stay outside this projection boundary.
    pub fn from_event(family: &str, payload: Value) -> Option<Self> {
        (family == CLOUD_WORKERS_STATUS_EVENT_FAMILY)
            .then(|| serde_json::from_value(payload).ok())
            .flatten()
    }

    pub fn payload(&self) -> Value {
        serde_json::to_value(self).expect("cloud-worker event is serializable")
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
            status: CloudWorkerCoordinatorStatus::Disabled,
            provider_ready: false,
            production_mutation: false,
            state_persisted: false,
            lease_count: 0,
            warm_image_count: 0,
            bootstrap_count: 0,
            artifact_count: 0,
            migration_receipt_count: 0,
        };
        assert!(projection.is_disabled());
        let event = CloudWorkerCoordinatorEvent::StatusChanged { projection };
        assert_eq!(event.family(), "cloud-workers.status");
    }

    #[test]
    fn status_projection_preserves_unavailable_as_a_closed_state() {
        let projection: CloudWorkerStatusProjection = serde_json::from_value(serde_json::json!({
            "owner": "cloud-workers",
            "status": "unavailable",
            "providerReady": false,
            "productionMutation": false
        }))
        .unwrap();
        assert!(projection.is_unavailable());
        assert!(!projection.is_disabled());
        assert!(serde_json::from_value::<CloudWorkerStatusProjection>(serde_json::json!({
            "owner": "cloud-workers",
            "status": "future-status",
            "providerReady": false,
            "productionMutation": false
        }))
        .is_err());
    }

    #[test]
    fn event_round_trip_is_scoped_to_the_registered_family() {
        let event = CloudWorkerCoordinatorEvent::StatusChanged {
            projection: CloudWorkerStatusProjection {
                owner: "cloud-workers".into(),
                status: CloudWorkerCoordinatorStatus::Disabled,
                provider_ready: false,
                production_mutation: false,
                state_persisted: false,
                lease_count: 0,
                warm_image_count: 0,
                bootstrap_count: 0,
                artifact_count: 0,
                migration_receipt_count: 0,
            },
        };
        let decoded = CloudWorkerCoordinatorEvent::from_event(event.family(), event.payload());
        assert_eq!(decoded, Some(event));
        assert_eq!(
            CloudWorkerCoordinatorEvent::from_event("other.status", serde_json::json!({})),
            None
        );
    }

    #[test]
    fn preserves_read_only_owner_snapshot_facts_from_host() {
        let projection: CloudWorkerStatusProjection = serde_json::from_value(serde_json::json!({
            "owner": "cloud-workers",
            "status": "enabled",
            "providerReady": false,
            "productionMutation": false,
            "statePersisted": true,
            "leaseCount": 2,
            "warmImageCount": 1,
            "migrationReceiptCount": 3
        }))
        .expect("host status projection should decode");
        assert!(projection.state_persisted);
        assert_eq!(projection.lease_count, 2);
        assert_eq!(projection.warm_image_count, 1);
        assert_eq!(projection.migration_receipt_count, 3);
        assert!(!projection.provider_ready);
        assert!(!projection.production_mutation);
    }
}
