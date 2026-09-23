use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::settings::settings_service::{
    SettingsService, is_valid_iana_time_zone,
};

fn temp_path(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-settings-{label}-{}-{suffix}.json",
        std::process::id()
    ))
}

#[test]
fn timezone_settings_validate_persist_preserve_unknown_fields_and_notify_effective_changes() {
    let path = temp_path("timezone");
    fs::write(
        &path,
        r#"{"version":1,"unrelated":{"keep":true},"userTimeZone":"UTC"}"#,
    )
    .expect("seed settings");
    let service = SettingsService::new(&path);
    assert!(is_valid_iana_time_zone("America/Los_Angeles"));
    assert!(!is_valid_iana_time_zone("Not/AZone"));
    assert_eq!(service.get_user_time_zone().as_deref(), Some("UTC"));

    let observed = Arc::new(Mutex::new(Vec::<Option<String>>::new()));
    let observed_for_listener = Arc::clone(&observed);
    let _subscription = service.subscribe_to_user_time_zone(Arc::new(move |zone| {
        observed_for_listener.lock().expect("observed lock").push(zone);
    }));

    assert!(!service
        .set_user_time_zone_override(Some("Not/AZone"))
        .expect("invalid zone ignored"));
    assert!(service
        .set_user_time_zone_override(Some(" America/Los_Angeles "))
        .expect("set override"));
    assert_eq!(
        service.get_user_time_zone().as_deref(),
        Some("America/Los_Angeles")
    );

    assert!(service
        .set_user_time_zone(Some("Europe/Berlin"))
        .expect("set detected zone"));
    assert_eq!(
        service.get_user_time_zone().as_deref(),
        Some("America/Los_Angeles")
    );

    assert!(service
        .set_user_time_zone_override(Some(""))
        .expect("clear override"));
    assert_eq!(service.get_user_time_zone().as_deref(), Some("Europe/Berlin"));

    let persisted: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("settings text"))
            .expect("settings json");
    assert_eq!(persisted["unrelated"]["keep"], true);
    assert_eq!(
        observed.lock().expect("observed lock").as_slice(),
        &[
            Some("America/Los_Angeles".to_string()),
            Some("Europe/Berlin".to_string())
        ]
    );
    let _ = fs::remove_file(path);
}
