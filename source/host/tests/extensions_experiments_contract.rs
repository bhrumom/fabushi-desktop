use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use mahayana_host_runtime::extensions::browser_ua::BrowserUaExperimentsApi;
use mahayana_host_runtime::extensions::experiments::{
    EXPERIMENTS_DEPENDENCIES, HostExperimentsExtension, HostExperimentsOptions,
    experiments_extension_id,
};
use mahayana_host_runtime::extensions::extension_ids_generated::HostExtensionId;

#[test]
fn experiments_extension_preserves_grok_gate_defaults_and_dev_overrides() {
    assert_eq!(experiments_extension_id(), HostExtensionId::Experiments);
    assert_eq!(
        EXPERIMENTS_DEPENDENCIES,
        &[HostExtensionId::Auth, HostExtensionId::Settings]
    );

    let default_service = HostExperimentsExtension::new(HostExperimentsOptions {
        is_dev_build: true,
        env_gate_overrides: None,
    });
    assert!(!default_service.is_agent_network_enabled());
    assert!(!default_service.is_ua_token_kill_switch_enabled());
    assert!(default_service.check_feature_gate("sand_multitask"));
    assert!(default_service.check_feature_gate("sand_spotlight"));
    assert!(default_service.check_feature_gate("enable_sparse_plugin_clones"));

    let dev_override = HostExperimentsExtension::new(HostExperimentsOptions {
        is_dev_build: true,
        env_gate_overrides: Some(
            "sand_agent_network=1,sand_browser_ua_token_kill_switch=on".into(),
        ),
    });
    assert!(dev_override.is_agent_network_enabled());
    assert!(BrowserUaExperimentsApi::is_ua_token_kill_switch_enabled(
        &dev_override
    ));

    let packaged = HostExperimentsExtension::new(HostExperimentsOptions {
        is_dev_build: false,
        env_gate_overrides: Some("sand_agent_network=1".into()),
    });
    assert!(!packaged.is_agent_network_enabled());
}

#[test]
fn settings_overrides_win_and_experiment_subscribers_are_disposable() {
    let service = HostExperimentsExtension::new(HostExperimentsOptions {
        is_dev_build: true,
        env_gate_overrides: Some("sand_agent_network=1".into()),
    });
    let notifications = Arc::new(AtomicUsize::new(0));
    let sink = Arc::clone(&notifications);
    let stop = service.subscribe(Arc::new(move || {
        sink.fetch_add(1, Ordering::SeqCst);
    }));

    service.replace_feature_flag_overrides(BTreeMap::from([
        ("sand_agent_network".into(), false),
    ]));
    assert!(!service.is_agent_network_enabled());
    assert_eq!(notifications.load(Ordering::SeqCst), 1);

    stop();
    service.replace_feature_flag_overrides(BTreeMap::from([
        ("sand_agent_network".into(), true),
    ]));
    assert!(service.is_agent_network_enabled());
    assert_eq!(notifications.load(Ordering::SeqCst), 1);
}
