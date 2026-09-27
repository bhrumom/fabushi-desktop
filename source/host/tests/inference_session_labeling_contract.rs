use std::sync::{Arc, Mutex};

use mahayana_host_runtime::extensions::inference::cursor_session::{
    RequestedModel, ResolveRequestedModelInputs, SandSessionOptions,
    parse_sand_mock_script, resolve_sand_requested_model, sand_default_model_selection,
};
use mahayana_host_runtime::extensions::inference::inference_service::{
    HostInferenceService, InferenceRoute, InferenceSettings, InferenceUsage,
    resolve_inference_route, usage_from_extended_fields,
};
use mahayana_host_runtime::extensions::inference::provider_session::RoutedProvider;
use mahayana_host_runtime::extensions::inference::sand_labeling::{
    FollowupLabelingTracker, LabelMessage, SAND_AGENT_MODE, SAND_SUMMARIZATION_REASON,
    prepare_post_turn_labeling,
};
use mahayana_host_runtime::extensions::inference::sand_model_experiment::{
    SandAgentModelParameter, SandAgentModelSelection,
};

fn selection(model_id: &str, max_mode: bool) -> SandAgentModelSelection {
    SandAgentModelSelection {
        model_id: model_id.into(),
        max_mode,
        parameters: vec![SandAgentModelParameter {
            id: "effort".into(),
            value: "medium".into(),
        }],
    }
}

fn requested(
    session_options: Option<&SandSessionOptions>,
    env_model_override: Option<&str>,
    stored_default_model: Option<&SandAgentModelSelection>,
    stored_computer_use_model: Option<&SandAgentModelSelection>,
    stored_browser_use_model: Option<&SandAgentModelSelection>,
    experiment_model_override: Option<&SandAgentModelSelection>,
) -> RequestedModel {
    resolve_sand_requested_model(ResolveRequestedModelInputs {
        session_options,
        env_model_override,
        stored_default_model,
        stored_computer_use_model,
        stored_browser_use_model,
        experiment_model_override,
    })
}

#[test]
fn cursor_model_resolution_preserves_frozen_precedence() {
    let stored = selection("stored", false);
    let experiment = selection("experiment", true);
    assert_eq!(
        requested(None, None, Some(&stored), None, None, Some(&experiment)).model_id,
        "experiment"
    );

    let env = requested(None, Some("env-model"), Some(&stored), None, None, Some(&experiment));
    assert_eq!(env.model_id, "env-model");
    assert_eq!(env.max_mode, None);

    let summarization = SandSessionOptions {
        model_id: Some("summary-model".into()),
        is_summarization_session: true,
        ..Default::default()
    };
    let resolved = requested(Some(&summarization), None, Some(&stored), None, None, None);
    assert_eq!(resolved.model_id, "summary-model");
    assert_eq!(resolved.max_mode, Some(true));

    let computer = SandSessionOptions {
        is_computer_use_subagent: true,
        ..Default::default()
    };
    let resolved = requested(Some(&computer), None, None, None, None, None);
    assert_eq!(resolved.model_id, "claude-opus-4-8");
    assert_eq!(resolved.max_mode, Some(false));

    let browser_model = selection("browser", false);
    let browser = SandSessionOptions {
        is_browser_use_subagent: true,
        ..Default::default()
    };
    assert_eq!(
        requested(Some(&browser), None, None, None, Some(&browser_model), None).model_id,
        "browser"
    );

    assert_eq!(
        requested(None, None, None, None, None, None),
        RequestedModel {
            model_id: sand_default_model_selection().model_id,
            max_mode: Some(true),
            parameters: sand_default_model_selection().parameters,
        }
    );
}

#[test]
fn cursor_mock_script_accepts_send_message_and_strict_tool_calls() {
    let send = parse_sand_mock_script(r#"{"sendMessage":"hello"}"#).expect("send message");
    assert_eq!(send.tool_calls.len(), 1);
    assert_eq!(send.tool_calls[0].tool_name, "SendMessage");
    assert_eq!(send.tool_calls[0].args["content"], "hello");

    let tools = parse_sand_mock_script(
        r#"{"toolCalls":[{"toolCallId":"call-1","toolName":"Read","args":{"path":"a"}}]}"#,
    )
    .expect("tool calls");
    assert_eq!(tools.tool_calls[0].tool_call_id.as_deref(), Some("call-1"));
    assert_eq!(tools.tool_calls[0].tool_name, "Read");
    assert_eq!(tools.tool_calls[0].args["path"], "a");

    assert!(parse_sand_mock_script("{}").is_none());
    assert!(parse_sand_mock_script(r#"{"toolCalls":[]}"#).is_none());
    assert!(parse_sand_mock_script(r#"{"toolCalls":[{"toolName":"","args":{}}]}"#).is_none());
}

#[derive(Clone, Default)]
struct RecordingSettings {
    provider: Arc<Mutex<Option<RoutedProvider>>>,
    usage: Arc<Mutex<Vec<(RoutedProvider, InferenceUsage)>>>,
}

impl InferenceSettings for RecordingSettings {
    fn inference_provider(&self) -> RoutedProvider {
        self.provider.lock().unwrap().unwrap_or(RoutedProvider::Cursor)
    }

    fn record_inference_usage(&self, provider: RoutedProvider, usage: InferenceUsage) {
        self.usage.lock().unwrap().push((provider, usage));
    }
}

#[test]
fn inference_service_keeps_cursor_out_of_local_provider_router_and_records_usage() {
    assert_eq!(resolve_inference_route(RoutedProvider::Cursor), InferenceRoute::Cursor);
    assert_eq!(
        resolve_inference_route(RoutedProvider::Codex),
        InferenceRoute::Routed(RoutedProvider::Codex)
    );

    let settings = RecordingSettings::default();
    *settings.provider.lock().unwrap() = Some(RoutedProvider::OpenRouter);
    let service = HostInferenceService::new(settings.clone());
    assert_eq!(
        service.route(),
        InferenceRoute::Routed(RoutedProvider::OpenRouter)
    );

    let usage = usage_from_extended_fields(Some(10), Some(20), Some(3), Some(-1));
    assert_eq!(usage.cache_write_tokens, None);
    service.record_usage(RoutedProvider::OpenRouter, usage);
    assert_eq!(settings.usage.lock().unwrap().as_slice(), &[(RoutedProvider::OpenRouter, usage)]);
}

#[test]
fn labeling_tracks_previous_request_once_and_skips_summarization() {
    let mut tracker = FollowupLabelingTracker::default();
    let normal = [LabelMessage::default()];
    assert!(tracker.observe_stream("conv", "req-1", "model", &normal).is_none());
    let followup = tracker
        .observe_stream("conv", "req-2", "model", &normal)
        .expect("followup");
    assert_eq!(followup.replying_to_request_id, "req-1");
    assert_eq!(followup.request_id, "req-2");
    assert_eq!(followup.agent_mode, SAND_AGENT_MODE);
    assert!(tracker.observe_stream("conv", "req-2", "model", &normal).is_none());

    let summarization = [LabelMessage {
        inference_reason: Some(SAND_SUMMARIZATION_REASON.into()),
    }];
    assert!(
        tracker
            .observe_stream("conv", "req-3", "model", &summarization)
            .is_none()
    );
    assert_eq!(tracker.last_request_id("conv"), Some("req-2"));

    assert!(prepare_post_turn_labeling("", "req", "model", &normal).is_none());
    let post = prepare_post_turn_labeling("conv", "req-4", "model", &normal).expect("post");
    assert_eq!(post.agent_mode, SAND_AGENT_MODE);
}
