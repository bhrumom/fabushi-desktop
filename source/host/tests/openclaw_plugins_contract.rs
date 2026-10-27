use sha2::Digest;
use mahayana_host_runtime::extensions::plugins::activation_planner::{
    ActivationPlanner, ActivationTrigger, PluginManifestRecord,
};
use mahayana_host_runtime::extensions::plugins::bundle_manifest::{
    encode_hex, parse_clawhub_plugin_spec, verify_archive_limits, verify_sha256_integrity,
    ClawHubPluginSpec, PluginBundleFormat, AGENT_BUNDLE_MANIFEST_RELATIVE_PATH,
    CLAUDE_BUNDLE_MANIFEST_RELATIVE_PATH, CODEX_BUNDLE_MANIFEST_RELATIVE_PATH,
    CURSOR_BUNDLE_MANIFEST_RELATIVE_PATH, DEFAULT_MAX_ARCHIVE_BYTES_ZIP,
    DEFAULT_MAX_ENTRIES, DEFAULT_MAX_EXTRACTED_BYTES,
};
use mahayana_host_runtime::extensions::plugins::capability_catalog::{
    CapabilityProviderEntry, ChannelCatalogEntry, PluginCapabilityCatalog, PluginOrigin,
    CAPABILITY_FAMILY_SPEECH_PROVIDERS, CAPABILITY_FAMILY_TRANSCRIPTION_PROVIDERS,
};

#[test]
fn test_ocrm_plugins_bundle_manifest() {
    // 1. Bundle manifest paths
    assert_eq!(
        PluginBundleFormat::Codex.manifest_relative_path(),
        CODEX_BUNDLE_MANIFEST_RELATIVE_PATH
    );
    assert_eq!(
        PluginBundleFormat::Claude.manifest_relative_path(),
        CLAUDE_BUNDLE_MANIFEST_RELATIVE_PATH
    );
    assert_eq!(
        PluginBundleFormat::Cursor.manifest_relative_path(),
        CURSOR_BUNDLE_MANIFEST_RELATIVE_PATH
    );
    assert_eq!(
        PluginBundleFormat::Agent.manifest_relative_path(),
        AGENT_BUNDLE_MANIFEST_RELATIVE_PATH
    );

    // 2. ClawHub plugin spec parsing
    let scoped = parse_clawhub_plugin_spec("@openclaw/browser-tools@1.4.2").expect("Parse scoped spec");
    assert_eq!(
        scoped,
        ClawHubPluginSpec {
            scope: Some("openclaw".to_string()),
            name: "browser-tools".to_string(),
            version: "1.4.2".to_string(),
        }
    );

    let unscoped = parse_clawhub_plugin_spec("discord-bot@latest").expect("Parse unscoped spec");
    assert_eq!(
        unscoped,
        ClawHubPluginSpec {
            scope: None,
            name: "discord-bot".to_string(),
            version: "latest".to_string(),
        }
    );

    let default_latest = parse_clawhub_plugin_spec("@vendor/my-pack").expect("Parse default latest");
    assert_eq!(default_latest.version, "latest");
    assert_eq!(default_latest.scope, Some("vendor".to_string()));

    // 3. Archive limits
    assert!(verify_archive_limits(1024, 2048, 10).is_ok());
    assert!(verify_archive_limits(DEFAULT_MAX_ARCHIVE_BYTES_ZIP + 1, 1024, 10).is_err());
    assert!(verify_archive_limits(1024, DEFAULT_MAX_EXTRACTED_BYTES + 1, 10).is_err());
    assert!(verify_archive_limits(1024, 1024, DEFAULT_MAX_ENTRIES + 1).is_err());

    // 4. SHA-256 integrity
    let test_data = b"openclaw-plugin-bundle-data";
    // verify with hex
    let hex_digest = encode_hex(&sha2::Sha256::digest(test_data));
    assert!(verify_sha256_integrity(test_data, &hex_digest).is_ok());
    assert!(verify_sha256_integrity(test_data, "bad-digest").is_err());
}

#[test]
fn test_ocrm_plugins_activation_planner() {
    let mut planner = ActivationPlanner::new();

    // Bundled plugin
    planner.register_manifest(PluginManifestRecord {
        plugin_id: "bundled-git".to_string(),
        commands: vec!["git:commit".to_string(), "git:status".to_string()],
        providers: vec![],
        channels: vec![],
        capabilities: vec!["vcs".to_string()],
        is_bundled: true,
        is_trusted: true,
    });

    // Trusted external plugin
    planner.register_manifest(PluginManifestRecord {
        plugin_id: "slack-channel".to_string(),
        commands: vec!["slack:send".to_string()],
        providers: vec![],
        channels: vec!["slack".to_string()],
        capabilities: vec![],
        is_bundled: false,
        is_trusted: true,
    });

    // Untrusted external plugin
    planner.register_manifest(PluginManifestRecord {
        plugin_id: "untrusted-addon".to_string(),
        commands: vec!["exploit:run".to_string()],
        providers: vec![],
        channels: vec![],
        capabilities: vec!["dangerous".to_string()],
        is_bundled: false,
        is_trusted: false,
    });

    // Plan with commands
    let plan1 = planner.plan_activation(&[ActivationTrigger::Command("git:commit".to_string())]);
    assert_eq!(plan1, vec!["bundled-git"]);

    // Plan with channel
    let plan2 = planner.plan_activation(&[ActivationTrigger::Channel("slack".to_string())]);
    assert_eq!(plan2, vec!["slack-channel"]);

    // Untrusted plugin must NOT be activated
    let plan3 = planner.plan_activation(&[ActivationTrigger::Command("exploit:run".to_string())]);
    assert!(plan3.is_empty());
}

#[test]
fn test_ocrm_plugins_capability_catalog() {
    let catalog = PluginCapabilityCatalog::new();

    // Register capability
    catalog.register_capability(CapabilityProviderEntry {
        family: CAPABILITY_FAMILY_SPEECH_PROVIDERS.to_string(),
        provider_id: "elevenlabs".to_string(),
        plugin_id: "elevenlabs-voice-plugin".to_string(),
    });
    catalog.register_capability(CapabilityProviderEntry {
        family: CAPABILITY_FAMILY_TRANSCRIPTION_PROVIDERS.to_string(),
        provider_id: "whisper-local".to_string(),
        plugin_id: "whisper-plugin".to_string(),
    });

    let entry = catalog
        .get_capability(CAPABILITY_FAMILY_SPEECH_PROVIDERS, "ElevenLabs")
        .expect("Case-insensitive lookup");
    assert_eq!(entry.plugin_id, "elevenlabs-voice-plugin");

    let speech_list = catalog.list_capabilities_for_family(CAPABILITY_FAMILY_SPEECH_PROVIDERS);
    assert_eq!(speech_list.len(), 1);

    // Channel catalog
    catalog.register_channel(ChannelCatalogEntry {
        plugin_id: "discord-plugin".to_string(),
        origin: PluginOrigin::Bundled,
        package_name: Some("@openclaw/discord".to_string()),
        channel_name: "discord".to_string(),
        trusted_official_install: true,
    });

    let channel = catalog.get_channel("DISCORD").expect("Channel lookup");
    assert_eq!(channel.plugin_id, "discord-plugin");
    assert!(channel.trusted_official_install);
}
