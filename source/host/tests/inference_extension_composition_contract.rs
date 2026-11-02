use std::fs;
use std::collections::BTreeMap;
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::experiments::extension::{
    HostExperimentsExtension, HostExperimentsOptions,
};
use mahayana_host_runtime::extensions::inference::cursor_web_tools::{
    CursorWebBackend, WebDocument, WebFetchResponse, WebSearchRequest, WebSearchResponse,
};
use mahayana_host_runtime::extensions::inference::extension::{
    AgentInferenceOwner, InferenceExtensionRuntime,
};
use mahayana_host_runtime::extensions::inference::inference_service::{
    InferenceRoute, InferenceSettings, InferenceUsage, authorize_routed_provider_request,
};
use mahayana_host_runtime::extensions::inference::production::{
    CursorWebBackendFactory, InferenceAuth, InferencePortFactory, InferenceProductionExtras,
    ModelExperimentApplied, ProductionInferenceSettings, RequestIdObserver,
    summarization_session_options,
};
use mahayana_host_runtime::extensions::inference::provider_session::RoutedProvider;
use mahayana_host_runtime::extensions::inference::cursor_session::SAND_SUMMARIZATION_MODEL_ID;
use mahayana_host_runtime::extensions::settings::settings_service::{
    HostSettingsUpdateError, SettingsService,
};

struct FakeAuth;
impl InferenceAuth for FakeAuth {
    fn access_token(&self) -> Result<String, String> { Ok("token-a".into()) }
    fn machine_id(&self) -> String { "machine-a".into() }
}

struct FakeOwner;
impl AgentInferenceOwner for FakeOwner {}

#[derive(Default)]
struct FakePortFactory {
    auth_seen: Mutex<Vec<(String, String)>>,
}
impl InferencePortFactory for FakePortFactory {
    fn create_host_inference(
        &self,
        auth: Arc<dyn InferenceAuth>,
        on_model_experiment_applied: ModelExperimentApplied,
    ) -> Arc<dyn AgentInferenceOwner> {
        self.auth_seen.lock().unwrap().push((auth.access_token().unwrap(), auth.machine_id()));
        on_model_experiment_applied();
        Arc::new(FakeOwner)
    }
}

#[derive(Default)]
struct FakeBackendFactory {
    auth_seen: Mutex<Vec<(String, String)>>,
}
impl CursorWebBackendFactory for FakeBackendFactory {
    fn create_backend(
        &self,
        auth: Arc<dyn InferenceAuth>,
        on_request_id: Option<RequestIdObserver>,
    ) -> Arc<dyn CursorWebBackend> {
        self.auth_seen.lock().unwrap().push((auth.access_token().unwrap(), auth.machine_id()));
        if let Some(observer) = on_request_id { observer("req-42"); }
        Arc::new(FakeBackend)
    }
}

struct FakeBackend;
impl CursorWebBackend for FakeBackend {
    fn run_web_search(&self, request: WebSearchRequest) -> Result<WebSearchResponse, String> {
        assert_eq!(request.search_term, "Grok architecture");
        assert_eq!(request.explanation.as_deref(), Some("parity"));
        assert_eq!(request.model_id, "grok-4.5");
        Ok(WebSearchResponse {
            answer: Some("answer".into()),
            documents: vec![WebDocument {
                url: "https://example.invalid".into(),
                title: "doc".into(),
                text: "body".into(),
            }],
        })
    }

    fn run_web_fetch(&self, url: &str) -> Result<WebFetchResponse, String> {
        match url {
            "https://ok.invalid" => Ok(WebFetchResponse::Success { content: "page".into() }),
            "https://timeout.invalid" => Ok(WebFetchResponse::Error {
                error: "timeout".into(),
                is_timeout: Some(true),
            }),
            _ => Ok(WebFetchResponse::MissingResult),
        }
    }
}

#[test]
fn inference_extension_readiness_and_experiment_notifications_match_frozen_boundary() {
    let runtime = InferenceExtensionRuntime::new(FakeOwner, Arc::new(|| Some("token".to_string())));
    assert!(runtime.is_ready(false));

    let offline = InferenceExtensionRuntime::new(FakeOwner, Arc::new(|| None));
    assert!(!offline.is_ready(false));
    assert!(offline.is_ready(true));

    let applied = Arc::new(Mutex::new(0usize));
    let counter = Arc::clone(&applied);
    runtime.on_model_experiment_applied(Arc::new(move || *counter.lock().unwrap() += 1));
    runtime.notify_model_experiment_applied();
    assert_eq!(*applied.lock().unwrap(), 1);
}

#[test]
fn production_extras_share_auth_and_forward_request_identity_to_web_services() {
    let port_factory = Arc::new(FakePortFactory::default());
    let backend_factory = Arc::new(FakeBackendFactory::default());
    let extras = InferenceProductionExtras::new(
        Arc::new(FakeAuth),
        port_factory.clone(),
        backend_factory.clone(),
    );

    let applied = Arc::new(Mutex::new(0usize));
    let applied_counter = Arc::clone(&applied);
    let _port = extras.create_port(Arc::new(move || *applied_counter.lock().unwrap() += 1));
    assert_eq!(*applied.lock().unwrap(), 1);

    let request_ids = Arc::new(Mutex::new(Vec::<String>::new()));
    let request_ids_sink = Arc::clone(&request_ids);
    let search = extras.create_web_search(
        "grok-4.5",
        Some(Arc::new(move |request_id| {
            request_ids_sink.lock().unwrap().push(request_id.to_string())
        })),
    );
    let response = search.search("Grok architecture", Some("parity".into())).expect("search");
    assert_eq!(response.answer.as_deref(), Some("answer"));
    assert_eq!(response.documents.len(), 1);

    let fetch = extras.create_web_fetch(None);
    assert_eq!(fetch.fetch("https://ok.invalid").unwrap().content.as_deref(), Some("page"));
    let timeout = fetch.fetch("https://timeout.invalid").unwrap();
    assert_eq!(timeout.error.as_deref(), Some("timeout"));
    assert_eq!(timeout.is_timeout, Some(true));
    assert_eq!(
        fetch.fetch("https://missing.invalid").unwrap().error.as_deref(),
        Some("Web fetch returned no result.")
    );

    assert_eq!(request_ids.lock().unwrap().as_slice(), ["req-42"]);
    assert_eq!(port_factory.auth_seen.lock().unwrap().len(), 1);
    assert_eq!(backend_factory.auth_seen.lock().unwrap().len(), 2);
}


#[test]
fn production_inference_settings_owns_live_route_and_usage_persistence() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "fabushi-production-inference-settings-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("create temp settings dir");
    let settings = Arc::new(SettingsService::new(root.join("settings.json")));
    let adapter = ProductionInferenceSettings::new(Arc::clone(&settings));
    let initial = settings.get_host_settings_snapshot();
    let committed = settings
        .set_host_settings_if_current(
            Some(&initial.hash),
            serde_json::json!({"inferenceProvider": "openrouter"}),
        )
        .expect("persist provider through the conditional owner");

    assert_eq!(adapter.inference_provider(), RoutedProvider::OpenRouter);
    assert!(matches!(
        settings.set_host_settings_if_current(
            Some(&initial.hash),
            serde_json::json!({"inferenceProvider": "codex"}),
        ),
        Err(HostSettingsUpdateError::Conflict { .. })
    ));
    assert_eq!(settings.get_host_settings_snapshot(), committed);
    assert_eq!(adapter.inference_provider(), RoutedProvider::OpenRouter);
    adapter.record_inference_usage(
        RoutedProvider::OpenRouter,
        InferenceUsage {
            input_tokens: Some(11),
            output_tokens: Some(7),
            cache_read_tokens: Some(3),
            cache_write_tokens: Some(2),
        },
    );

    let usage = settings.get_inference_router_usage();
    assert_eq!(usage["providers"]["openrouter"]["requests"].as_u64(), Some(1));
    assert_eq!(usage["providers"]["openrouter"]["inputTokens"].as_u64(), Some(11));
    assert_eq!(usage["providers"]["openrouter"]["outputTokens"].as_u64(), Some(7));
    assert_eq!(usage["providers"]["openrouter"]["cacheReadTokens"].as_u64(), Some(3));
    assert_eq!(usage["providers"]["openrouter"]["cacheWriteTokens"].as_u64(), Some(2));
    let persisted = settings.get_host_settings_snapshot();
    drop(adapter);
    drop(settings);
    let reopened = Arc::new(SettingsService::new(root.join("settings.json")));
    let reopened_adapter = ProductionInferenceSettings::new(Arc::clone(&reopened));
    assert_eq!(reopened.get_host_settings_snapshot(), persisted);
    assert_eq!(reopened_adapter.inference_provider(), RoutedProvider::OpenRouter);
    assert_eq!(reopened.get_inference_router_usage(), usage);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn production_experiments_apply_reentrant_settings_notifications_in_commit_order() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "fabushi-settings-notification-order-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("create settings directory");
    let path = root.join("settings.json");
    let settings = Arc::new(SettingsService::new(&path));
    let experiments = Arc::new(HostExperimentsExtension::new(HostExperimentsOptions {
        is_dev_build: false,
        env_gate_overrides: None,
    }));
    experiments.replace_feature_flag_overrides(BTreeMap::from([
        ("settings_commit_order_gate".into(), false),
    ]));
    let applied = Arc::new(Mutex::new(Vec::new()));
    let applied_sink = Arc::clone(&applied);
    let experiments_for_settings = Arc::clone(&experiments);
    let _feature_subscription = settings.subscribe_to_feature_flag_overrides(Arc::new(
        move |overrides| {
            // This is the same real consumer called by production composition.
            experiments_for_settings.replace_feature_flag_overrides(overrides);
            applied_sink.lock().expect("applied gates").push(
                experiments_for_settings.check_feature_gate("settings_commit_order_gate"),
            );
        },
    ));
    let nested_receipt = Arc::new(Mutex::new(None));
    let receipt_sink = Arc::clone(&nested_receipt);
    let applied_during_callback = Arc::clone(&applied);
    let weak_owner = Arc::downgrade(&settings);
    let _timezone_subscription = settings.subscribe_to_user_time_zone(Arc::new(move |zone| {
        assert_eq!(zone.as_deref(), Some("Asia/Tokyo"));
        let owner = weak_owner.upgrade().expect("settings owner");
        let current = owner.get_host_settings_snapshot();
        let receipt = owner
            .set_host_settings_if_current(
                Some(&current.hash),
                serde_json::json!({
                    "inferenceProvider": "codex",
                    "featureFlagOverrides": {"settings_commit_order_gate": false}
                }),
            )
            .expect("B commits from A's timezone callback");
        assert_eq!(receipt.settings["inferenceProvider"], "codex");
        assert_eq!(owner.get_host_settings_snapshot(), receipt);
        assert!(applied_during_callback.lock().expect("applied gates").is_empty(),
            "B's receipt precedes notification completion while A is draining");
        *receipt_sink.lock().expect("nested receipt") = Some(receipt);
    }));

    let owner = Arc::clone(&settings);
    let (sender, receiver) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        sender.send(owner.set_host_settings_if_current(
            None,
            serde_json::json!({
                "userTimeZone": "Asia/Tokyo",
                "inferenceProvider": "openrouter",
                "featureFlagOverrides": {"settings_commit_order_gate": true}
            }),
        )).expect("deliver A receipt");
    });
    let first = receiver.recv_timeout(Duration::from_secs(10))
        .expect("reentrant settings notifications must not deadlock")
        .expect("A commits");
    worker.join().expect("settings drainer thread");
    let second = nested_receipt.lock().expect("nested receipt")
        .take().expect("B receipt");
    assert_eq!(first.settings["inferenceProvider"], "openrouter");
    assert_ne!(first.hash, second.hash);
    assert_ne!(first.config_source["rawHash"], second.config_source["rawHash"]);
    assert_eq!(settings.get_host_settings_snapshot(), second);
    assert_eq!(applied.lock().expect("applied gates").as_slice(), &[true, false]);
    assert!(!experiments.check_feature_gate("settings_commit_order_gate"));
    let stored: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("latest source bytes"))
            .expect("persisted settings JSON");
    assert_eq!(stored["inferenceProvider"], "codex");
    assert!(stored.get("featureFlagOverrides").is_none(),
        "the existing feature override contract remains runtime-only");
    let _ = fs::remove_dir_all(root);
}


#[test]
fn cursor_route_is_authorized_through_the_same_shipping_runner_boundary() {
    assert_eq!(
        authorize_routed_provider_request(InferenceRoute::Cursor, RoutedProvider::Cursor),
        Ok(RoutedProvider::Cursor)
    );
    assert!(authorize_routed_provider_request(
        InferenceRoute::Cursor,
        RoutedProvider::ClaudeCode,
    )
    .is_err());

    let shipping_main = include_str!("../app/src/main.rs");
    assert!(shipping_main.contains(
        "let requested_provider = RoutedProvider::parse(provider_name)"
    ));
    assert!(!shipping_main.contains(
        ".filter(|provider| *provider != RoutedProvider::Cursor)"
    ));
    assert!(shipping_main.contains(
        "let cursor_auth = inference.cursor_auth_for_session(Some(&worker_session_options));"
    ));
    assert!(shipping_main.contains(
        "cursor_auth: Some(cursor_auth)"
    ));
    assert!(shipping_main.contains(
        "run_local_background_revival_turn("
    ));
    assert!(!shipping_main.contains(
        "Cursor background revival remains on the compatibility path"
    ));
    assert!(shipping_main.contains(
        "configured_routed_provider(&gateway.data_dir.join(\"settings.json\"))"
    ));
}


#[test]
fn summarization_session_matches_frozen_model_and_labeling_fence() {
    let options = summarization_session_options();
    assert_eq!(
        options.model_id.as_deref(),
        Some(SAND_SUMMARIZATION_MODEL_ID)
    );
    assert_eq!(SAND_SUMMARIZATION_MODEL_ID, "gemini-2.5-flash");
    assert!(options.is_summarization_session);
    assert!(options.skip_labeling);
    assert!(!options.is_computer_use_subagent);
    assert!(!options.is_browser_use_subagent);
}
