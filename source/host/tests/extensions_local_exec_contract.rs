use mahayana_host_runtime::extensions::local_exec::{
    local_exec_error::SandLocalExecError,
    local_exec_failure_classifier::{
        classify_local_exec_failure, LocalExecFailureClass, LocalExecFailureClassification,
    },
};

#[test]
fn local_exec_error_preserves_provider_message() {
    let error = SandLocalExecError::new("computer disconnected");
    assert_eq!(error.to_string(), "computer disconnected");
}

#[test]
fn failure_classifier_matches_grok_spawn_errno_contract() {
    assert_eq!(
        classify_local_exec_failure("Error: spawn rg ENOENT"),
        LocalExecFailureClassification {
            error_class: LocalExecFailureClass::SpawnEnoent,
            errno: Some("ENOENT".into()),
        }
    );
    assert_eq!(
        classify_local_exec_failure("spawnSync tool EACCES"),
        LocalExecFailureClassification {
            error_class: LocalExecFailureClass::SpawnPermissions,
            errno: Some("EACCES".into()),
        }
    );
    assert_eq!(
        classify_local_exec_failure("spawn helper ETIMEDOUT"),
        LocalExecFailureClassification {
            error_class: LocalExecFailureClass::SpawnOther,
            errno: Some("ETIMEDOUT".into()),
        }
    );
    assert_eq!(
        classify_local_exec_failure("request failed EPERM"),
        LocalExecFailureClassification {
            error_class: LocalExecFailureClass::Other,
            errno: Some("EPERM".into()),
        }
    );
}
