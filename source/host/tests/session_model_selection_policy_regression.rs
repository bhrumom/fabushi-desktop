//! Scoped production SQLite mutation contract for trusted model selection policy.
//! Executed only by exact-H GitHub Actions (not the local migration environment).

use std::fs;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::inference::model_picker_service::{
    AdmittedRunOperatorAuthority, ApplySessionModelSelectionParams,
    ApplySessionModelSelectionResult, ModelCatalogEntry, ModelPickerRejectReason,
    SessionModelSelectionRequest, SessionModelSelectionRuntime, WorkerPlacementContext,
};
use mahayana_host_runtime::extensions::session::agent_session::SandAgentSessionStore;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::storage::store_db::get_sand_agent_db_write_generation;

#[test]
fn selection_store_policy_refusal_preserves_sqlite_generation_and_default() {
    let suffix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let root = std::env::temp_dir().join(format!(
        "fabushi-model-policy-store-{}-{suffix}", std::process::id()
    ));
    let agents = root.join("agents");
    let production = Arc::new(ProductionSessionWorkers::with_agents_root(&agents, 500));
    let store = SandAgentSessionStore::new(Arc::clone(&production));
    let record = store.create_session(None, "user", None).unwrap();
    let admission = production.read_session_work_admission(&record.id).unwrap().unwrap();
    let session_key = format!("agent:{}:main", record.id);
    let catalog = [ModelCatalogEntry {
        provider: "fabushi".into(), id: "ModelCase".into(), name: None,
        context_window: None, context_tokens: Some(8192),
        api: None, base_url: None, reasoning: None,
    }];
    let read_only = AdmittedRunOperatorAuthority {
        profile_id: "read-only".into(), scopes: vec!["operator.read".into()],
        allowed_models: None,
    };
    let writer = AdmittedRunOperatorAuthority {
        profile_id: "native-writer".into(), scopes: vec!["operator.write".into()],
        allowed_models: Some(vec!["fabushi/ModelCase".into()]),
    };
    let foreign_placement = WorkerPlacementContext {
        session_id: "other-agent".into(), state: "active".into(),
        execution_mode: Some("local".into()), generation: 8,
    };
    let owner = production.open_agent_db_owner(&record.id).unwrap();
    let initial_bytes = owner.read_kv("sessionModelSelection").unwrap();
    let initial_generation = get_sand_agent_db_write_generation(owner.db_path());

    let apply = |model: &str, authority: Option<&AdmittedRunOperatorAuthority>,
                 placement: Option<&WorkerPlacementContext>,
                 initial: Option<u64>, current: Option<u64>| {
        store.apply_session_model_selection(
            &record.id, &record.id, &admission.lifecycle_revision,
            ApplySessionModelSelectionParams {
                agent_id: &record.id, session_key: &session_key,
                default_provider: "fabushi", default_model: "modelcase",
                current_provider: "fabushi", current_model: "modelcase",
                model_catalog: &catalog, visibility_policy: None,
                operator_authority: authority, placement_context: placement,
                candidate_execution_mode: Some("local"),
                initial_generation: initial, current_generation: current,
                request: SessionModelSelectionRequest {
                    provider: "fabushi".into(), model: model.into(),
                    is_default: false, reset_to_default: false,
                    alias: None, profile_override: None,
                    runtime: SessionModelSelectionRuntime::Unchanged,
                },
            }, 100,
        ).unwrap()
    };

    assert!(matches!(
        apply("ModelCase", Some(&read_only), None, Some(7), Some(7)),
        ApplySessionModelSelectionResult::Rejected { reason: ModelPickerRejectReason::NotAllowed, .. }
    ));
    assert!(matches!(
        apply("modelcase", Some(&writer), None, Some(7), Some(7)),
        ApplySessionModelSelectionResult::Rejected { reason: ModelPickerRejectReason::UnknownProvider, .. }
    ));
    assert!(matches!(
        apply("ModelCase", Some(&writer), None, Some(7), None),
        ApplySessionModelSelectionResult::Conflict { .. }
    ));
    assert!(matches!(
        apply("ModelCase", Some(&writer), Some(&foreign_placement), Some(7), Some(7)),
        ApplySessionModelSelectionResult::Conflict { .. }
    ));
    assert_eq!(owner.read_kv("sessionModelSelection").unwrap(), initial_bytes);
    assert_eq!(get_sand_agent_db_write_generation(owner.db_path()), initial_generation);

    assert!(matches!(
        apply("ModelCase", Some(&writer), None, Some(7), Some(7)),
        ApplySessionModelSelectionResult::Applied(_)
    ));
    let selected = store.read_session_model_entry(&record.id).unwrap();
    assert_eq!(selected.provider_override.as_deref(), Some("fabushi"));
    assert_eq!(selected.model_override.as_deref(), Some("ModelCase"));
    let reopened = SandAgentSessionStore::new(Arc::new(
        ProductionSessionWorkers::with_agents_root(&agents, 500)
    ));
    assert_eq!(
        reopened.read_session_model_entry(&record.id).unwrap().model_override.as_deref(),
        Some("ModelCase")
    );
    store.close_worker_pool();
    let _ = fs::remove_dir_all(root);
}
