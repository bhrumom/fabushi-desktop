use std::collections::BTreeMap;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::extension_ids_generated::HostExtensionId;
use mahayana_host_runtime::extensions::secrets::extension::{
    SECRETS_DEPENDENCIES, secrets_extension_id,
};
use mahayana_host_runtime::extensions::secrets::secrets_service::{
    BOX_SECRET_REDACTION_NAMES_ENV_VAR, BoxSecretsApplier, BoxSecretsApplierOptions,
    BoxSecretsApplyError, BoxSecretsSetError, build_box_secrets_env,
    validate_box_secret_key, validate_box_secrets,
};

fn temp_path(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-secrets-{label}-{}-{suffix}.json",
        std::process::id()
    ))
}

#[test]
fn secret_validation_and_environment_projection_match_frozen_grok() {
    assert_eq!(
        validate_box_secret_key("PATH").as_deref(),
        Some("PATH is reserved by the box runtime")
    );
    assert_eq!(
        validate_box_secret_key("SAND_TOKEN").as_deref(),
        Some("Names starting with SAND_ are reserved by the box runtime")
    );
    assert_eq!(
        validate_box_secret_key("my_CURSOR_SANDBOX_key").as_deref(),
        Some("my_CURSOR_SANDBOX_key is reserved by the box runtime")
    );
    assert_eq!(
        validate_box_secret_key("9TOKEN").as_deref(),
        Some("\"9TOKEN\" is not a valid environment variable name")
    );
    assert_eq!(validate_box_secret_key("API_TOKEN"), None);

    let secrets = BTreeMap::from([
        ("Z_TOKEN".to_string(), "z".to_string()),
        ("API_TOKEN".to_string(), "a".to_string()),
    ]);
    assert_eq!(validate_box_secrets(&secrets), None);
    let projected = build_box_secrets_env(&secrets);
    assert_eq!(projected["API_TOKEN"], "a");
    assert_eq!(projected["Z_TOKEN"], "z");
    assert_eq!(
        projected[BOX_SECRET_REDACTION_NAMES_ENV_VAR],
        "API_TOKEN,Z_TOKEN"
    );
    assert!(build_box_secrets_env(&BTreeMap::new()).is_empty());
    assert_eq!(secrets_extension_id(), HostExtensionId::Secrets);
    assert_eq!(SECRETS_DEPENDENCIES, &[HostExtensionId::ForeverBox]);
}

#[test]
fn secrets_persist_retry_apply_and_reload_without_losing_generation() {
    let path = temp_path("retry");
    let attempts = Arc::new(AtomicUsize::new(0));
    let applied = Arc::new(Mutex::new(Vec::new()));
    let attempts_for_apply = Arc::clone(&attempts);
    let applied_for_apply = Arc::clone(&applied);
    let apply = Arc::new(move |update: &mahayana_host_runtime::r#box::box_env::BoxEnvironmentUpdate| {
        let attempt = attempts_for_apply.fetch_add(1, Ordering::SeqCst);
        if attempt == 0 {
            return Err(BoxSecretsApplyError::Retryable("temporary".into()));
        }
        applied_for_apply
            .lock()
            .expect("applied updates")
            .push(update.clone());
        Ok(())
    });
    let mut options = BoxSecretsApplierOptions::new(apply);
    options.store_path = path.clone();
    options.retry_initial = Duration::from_millis(2);
    options.retry_max = Duration::from_millis(4);
    options.apply_wait = Duration::from_millis(500);
    options.now_ms = Arc::new(|| 1234);
    options.log = Arc::new(|_| {});
    let service = BoxSecretsApplier::new(options);

    let status = service
        .set_secrets(BTreeMap::from([(
            "API_TOKEN".to_string(),
            "secret".to_string(),
        )]))
        .expect("set secrets");
    assert!(status.is_applied);
    assert_eq!(status.keys, vec!["API_TOKEN"]);
    assert_eq!(status.last_applied_at_ms, Some(1234));
    assert!(attempts.load(Ordering::SeqCst) >= 2);
    let update = applied.lock().expect("applied").last().cloned().expect("update");
    assert!(update.replace);
    assert_eq!(update.env["API_TOKEN"], "secret");
    assert_eq!(
        update.env[BOX_SECRET_REDACTION_NAMES_ENV_VAR],
        "API_TOKEN"
    );
    service.stop();

    let persisted: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("persisted secrets"))
            .expect("persisted json");
    assert_eq!(persisted["version"], 1);
    assert_eq!(persisted["secrets"]["API_TOKEN"], "secret");

    let reloaded = Arc::new(Mutex::new(Vec::new()));
    let reloaded_for_apply = Arc::clone(&reloaded);
    let apply = Arc::new(move |update: &mahayana_host_runtime::r#box::box_env::BoxEnvironmentUpdate| {
        reloaded_for_apply
            .lock()
            .expect("reloaded updates")
            .push(update.clone());
        Ok(())
    });
    let mut options = BoxSecretsApplierOptions::new(apply);
    options.store_path = path.clone();
    options.apply_wait = Duration::from_millis(500);
    options.log = Arc::new(|_| {});
    let service = BoxSecretsApplier::new(options);
    let status = service.apply_persisted().expect("apply persisted");
    assert!(status.is_applied);
    assert_eq!(status.keys, vec!["API_TOKEN"]);
    assert_eq!(
        reloaded.lock().expect("reloaded").last().expect("update").env["API_TOKEN"],
        "secret"
    );
    service.stop();
    let _ = fs::remove_file(path);
}

#[test]
fn invalid_secret_never_persists_or_applies() {
    let path = temp_path("invalid");
    let apply = Arc::new(|_: &mahayana_host_runtime::r#box::box_env::BoxEnvironmentUpdate| Ok(()));
    let mut options = BoxSecretsApplierOptions::new(apply);
    options.store_path = path.clone();
    options.log = Arc::new(|_| {});
    let service = BoxSecretsApplier::new(options);
    let error = service
        .set_secrets(BTreeMap::from([("PATH".to_string(), "x".to_string())]))
        .expect_err("reserved secret rejected");
    assert!(matches!(error, BoxSecretsSetError::Validation(_)));
    assert!(!path.exists());
    service.stop();
}
