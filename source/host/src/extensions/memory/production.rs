use std::path::PathBuf;
use std::sync::Arc;

use crate::extensions::inference::production::ProductionInferenceExtension;
use crate::storage::agent_paths::get_sand_agents_root_dir;

use super::extension::{HostMemoryExtension, start_memory_extension};
use super::memory_service::MemoryService;
use super::memory_synthesis_service::{
    MemorySynthesisOptions, MemorySynthesisService, SynthesisTarget, parse_json_object,
    synthesis_request_json, synthesis_system_prompt, verification_request_json,
    verification_system_prompt,
};

pub fn start_production_memory_extension() -> HostMemoryExtension {
    start_memory_extension(get_sand_agents_root_dir(None))
}

pub fn create_production_memory_synthesis(
    service: Arc<MemoryService>,
    inference: Arc<ProductionInferenceExtension>,
    data_dir: PathBuf,
) -> Arc<MemorySynthesisService> {
    let list_service = Arc::clone(&service);
    let list_targets = Arc::new(move || {
        list_service
            .list_synthesis_targets()
            .into_iter()
            .map(|(agent_id, store)| {
                (
                    agent_id,
                    Arc::new(store) as Arc<dyn SynthesisTarget>,
                )
            })
            .collect::<Vec<_>>()
    });

    let target_service = Arc::clone(&service);
    let get_target = Arc::new(move |agent_id: &str| {
        target_service
            .synthesis_target_for_agent(agent_id)
            .map(|store| Arc::new(store) as Arc<dyn SynthesisTarget>)
    });

    let propose_inference = Arc::clone(&inference);
    let propose_data_dir = data_dir.clone();
    let propose = Arc::new(move |request, cancel| {
        let user = synthesis_request_json(&request);
        let output = propose_inference
            .run_summarization_prompt(
                &propose_data_dir,
                synthesis_system_prompt(),
                &user,
                &|| cancel.is_cancelled(),
            )
            .map_err(|error| format!("{error:?}"))?;
        parse_json_object(&output)
            .ok_or_else(|| "memory synthesis inference returned invalid JSON".to_string())
    });

    let verify_inference = Arc::clone(&inference);
    let verify = Arc::new(move |request, cancel| {
        let user = verification_request_json(&request);
        let output = verify_inference
            .run_summarization_prompt(
                &data_dir,
                verification_system_prompt(),
                &user,
                &|| cancel.is_cancelled(),
            )
            .map_err(|error| format!("{error:?}"))?;
        let verdict = parse_json_object(&output)
            .ok_or_else(|| "memory synthesis verification returned invalid JSON".to_string())?;
        Ok(verdict
            .get("approved")
            .and_then(serde_json::Value::as_bool)
            == Some(true))
    });

    let mut options = MemorySynthesisOptions::new(list_targets, get_target, propose);
    options.verify = Some(verify);
    Arc::new(MemorySynthesisService::new(options))
}
