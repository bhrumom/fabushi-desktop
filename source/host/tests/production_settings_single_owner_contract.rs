const SHIPPING_HOST: &str = include_str!("../app/src/main.rs");
const PRODUCTION_EXTENSIONS: &str = include_str!("../src/host_production_extensions.rs");

#[test]
fn shipping_host_reuses_the_production_settings_owner() {
    assert!(
        PRODUCTION_EXTENSIONS.contains("let settings = start_settings_extension();"),
        "production extension composition must create the canonical Settings owner"
    );
    assert!(
        SHIPPING_HOST.contains(
            "let settings_extension = Arc::clone(&production_extensions.settings);"
        ),
        "shipping Host must reuse the Settings owner created by production extension composition"
    );
    assert_eq!(
        SHIPPING_HOST.matches("start_settings_extension()").count(),
        0,
        "shipping Host must not create a second Settings runtime owner"
    );
}
