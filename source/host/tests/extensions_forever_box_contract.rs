use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use mahayana_host_runtime::extensions::box_lifecycle::RecreateSandBoxResponse;
use mahayana_host_runtime::extensions::forever_box::{
    ForeverBoxLifecycle, ForeverBoxService, HostBox,
    is_host_bundle_auto_update_enabled, is_image_auto_update_enabled,
};
use mahayana_host_runtime::r#box::production::ProductionBoxEnvironment;

#[derive(Default)]
struct FakeLifecycle {
    image_update_available: bool,
    recreates: Mutex<Vec<(bool, Option<bool>)>>,
}

impl ForeverBoxLifecycle for FakeLifecycle {
    fn fetch_image_update_available(&self) -> Result<bool, String> {
        Ok(self.image_update_available)
    }

    fn recreate_in_box(
        &self,
        preserve_data: bool,
        force: Option<bool>,
    ) -> Result<RecreateSandBoxResponse, String> {
        self.recreates
            .lock()
            .expect("recreate mutex")
            .push((preserve_data, force));
        Ok(RecreateSandBoxResponse {
            started: true,
            reason: None,
        })
    }
}

#[test]
fn forever_box_options_match_frozen_store_and_auto_update_gates() {
    let mut environment = BTreeMap::new();
    assert!(!is_image_auto_update_enabled(&environment));
    assert!(is_host_bundle_auto_update_enabled(&environment));

    environment.insert("SAND_BOX_STORE_SYNC".into(), "true".into());
    environment.insert("SAND_BOX_STORE_COPY_IN".into(), "1".into());
    assert!(is_image_auto_update_enabled(&environment));

    environment.insert("SAND_BOX_AUTO_UPDATE".into(), "false".into());
    assert!(!is_image_auto_update_enabled(&environment));
    assert!(!is_host_bundle_auto_update_enabled(&environment));
}

#[test]
fn forever_box_service_composes_host_box_and_box_lifecycle_without_parallel_runtime() {
    let lifecycle = Arc::new(FakeLifecycle {
        image_update_available: true,
        recreates: Mutex::new(Vec::new()),
    });
    let box_ = HostBox::new(ProductionBoxEnvironment::new(
        "127.0.0.1",
        9,
        "test-token",
    ));
    let observed = Arc::new(Mutex::new(Vec::new()));
    let observed_for_listener = Arc::clone(&observed);
    box_.subscribe(Arc::new(move |status| {
        observed_for_listener
            .lock()
            .expect("status mutex")
            .push(status.clone());
    }));

    let initial = box_.get_status("agent-a");
    assert_eq!(initial.state, "absent");
    assert_eq!(initial.image_update_available, None);

    let service = ForeverBoxService::new(
        box_.clone(),
        lifecycle.clone(),
        true,
        false,
        true,
    );
    assert!(service.refresh_image_update_available().expect("image state"));
    assert_eq!(service.get_status("agent-a").image_update_available, Some(true));

    let reset = service.reset("agent-a").expect("reset request");
    assert_eq!(reset.state, "running");
    assert_eq!(reset.pull_percent, Some(0));

    let update = service.update("agent-a", Some(true)).expect("update request");
    assert_eq!(update.state, "running");
    assert_eq!(
        lifecycle.recreates.lock().expect("recreate mutex").as_slice(),
        &[(false, None), (true, Some(true))]
    );
    assert!(
        observed
            .lock()
            .expect("status mutex")
            .iter()
            .any(|status| status.image_update_available == Some(true))
    );
}

#[test]
fn forever_box_auto_update_preserves_reference_skip_reasons() {
    let lifecycle = Arc::new(FakeLifecycle {
        image_update_available: true,
        recreates: Mutex::new(Vec::new()),
    });
    let outside = ForeverBoxService::new(
        HostBox::new(ProductionBoxEnvironment::new("127.0.0.1", 9, "token")),
        lifecycle.clone(),
        true,
        false,
        false,
    );
    assert_eq!(
        outside.auto_update_now().expect("outside result").reason.as_deref(),
        Some("not-in-box")
    );

    let disabled = ForeverBoxService::new(
        HostBox::new(ProductionBoxEnvironment::new("127.0.0.1", 9, "token")),
        lifecycle,
        false,
        false,
        true,
    );
    assert_eq!(
        disabled
            .auto_update_now()
            .expect("disabled result")
            .reason
            .as_deref(),
        Some("auto-update-disabled")
    );
}
