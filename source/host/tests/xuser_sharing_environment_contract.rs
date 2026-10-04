use std::collections::BTreeMap;

use mahayana_host_runtime::extensions::cross_user_sharing::xuser_sharing_environment::{
    SAND_DEV_XUSER_SHARING_ENV, SAND_XUSER_SHARING_ALLOW_PROD_ENV,
    is_production_backend_url, resolve_xuser_sharing_environment,
};

fn env(values: &[(&str, &str)]) -> BTreeMap<String, String> {
    values
        .iter()
        .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
        .collect()
}

#[test]
fn packaged_host_allows_cross_user_sharing_without_dev_overrides() {
    let resolved = resolve_xuser_sharing_environment(
        "https://api2.cursor.sh",
        &env(&[("SAND_PACKAGED", "1")]),
    );
    assert!(resolved.is_allowed);
    assert!(resolved.reason.is_none());
}

#[test]
fn dev_host_requires_explicit_xuser_opt_in() {
    let resolved = resolve_xuser_sharing_environment("https://staging.example", &env(&[]));
    assert!(!resolved.is_allowed);
    assert!(resolved
        .reason
        .as_deref()
        .is_some_and(|value| value.contains(SAND_DEV_XUSER_SHARING_ENV)));
}

#[test]
fn opted_in_dev_host_allows_non_production_backend() {
    let resolved = resolve_xuser_sharing_environment(
        "https://staging.example",
        &env(&[(SAND_DEV_XUSER_SHARING_ENV, "1")]),
    );
    assert!(resolved.is_allowed);
}

#[test]
fn opted_in_dev_host_still_requires_explicit_production_backend_opt_in() {
    let resolved = resolve_xuser_sharing_environment(
        "https://api2.cursor.sh/path",
        &env(&[(SAND_DEV_XUSER_SHARING_ENV, "1")]),
    );
    assert!(!resolved.is_allowed);
    assert!(resolved
        .reason
        .as_deref()
        .is_some_and(|value| value.contains(SAND_XUSER_SHARING_ALLOW_PROD_ENV)));

    let allowed = resolve_xuser_sharing_environment(
        "https://api2.cursor.sh",
        &env(&[
            (SAND_DEV_XUSER_SHARING_ENV, "1"),
            (SAND_XUSER_SHARING_ALLOW_PROD_ENV, "1"),
        ]),
    );
    assert!(allowed.is_allowed);
}

#[test]
fn production_origin_comparison_is_fail_closed() {
    assert!(is_production_backend_url("https://api2.cursor.sh/foo"));
    assert!(is_production_backend_url("not a url"));
    assert!(!is_production_backend_url("https://api2.cursor.sh.evil.example"));
}
