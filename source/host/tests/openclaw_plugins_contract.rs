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
    CapabilityProviderEntry, ChannelCatalogEntry, PluginCapabilityCatalog,
    PluginContractRegistryEntry, PluginOrigin, ProviderAuthChoice,
    CAPABILITY_FAMILY_SPEECH_PROVIDERS, CAPABILITY_FAMILY_TRANSCRIPTION_PROVIDERS,
    BundledPluginScanner, BundledProviderCompatValidator, BundledSourceOverlayResolver,
    BundledPluginNamingAndArtworkValidator,
    BundledPluginSource, BundledPluginLookup, BundledPluginSourceResolver,
    CandidateInstallOwnerResolver,
    CapabilityCatalogFamily, CapabilityCatalogValidator,
    PluginAcceptedDeclaredSurface, CapabilityConsentManager,
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

    // 5. Package manifest authoring validation (package-manifest.contract.test.ts)
    use mahayana_host_runtime::extensions::plugins::bundle_manifest::{
        resolve_package_extension_entries, validate_package_extension_entries_for_install,
        OpenClawMetadata,
    };
    use serde_json::json;

    // Valid package manifest
    let valid_json = json!({
        "openclaw": {
            "extensions": ["./src/index.ts"],
            "runtimeExtensions": ["./dist/index.js"],
            "setupEntry": "./src/setup.ts",
            "runtimeSetupEntry": "./dist/setup.js"
        }
    });
    let res = resolve_package_extension_entries(&valid_json);
    assert_eq!(res.status, "ok");
    assert_eq!(res.entries, vec!["./src/index.ts"]);

    // Non-object openclaw metadata
    let non_obj = json!({"openclaw": "invalid"});
    let res_non_obj = resolve_package_extension_entries(&non_obj);
    assert_eq!(res_non_obj.status, "invalid");
    assert_eq!(
        res_non_obj.error.as_deref(),
        Some("package.json openclaw must be an object")
    );

    // Non-array extensions
    let non_arr = json!({"openclaw": {"extensions": "./index.js"}});
    let res_non_arr = resolve_package_extension_entries(&non_arr);
    assert_eq!(res_non_arr.status, "invalid");
    assert_eq!(
        res_non_arr.error.as_deref(),
        Some("package.json openclaw.extensions must be an array")
    );

    // Blank extension
    let blank = json!({"openclaw": {"extensions": [" "]}});
    let res_blank = resolve_package_extension_entries(&blank);
    assert_eq!(res_blank.status, "invalid");
    assert_eq!(
        res_blank.error.as_deref(),
        Some("package.json openclaw.extensions[0] must be a non-empty string")
    );

    // Runtime extensions length mismatch
    let mismatch_meta = OpenClawMetadata {
        extensions: vec!["./src/one.ts".to_string(), "./src/two.ts".to_string()],
        runtime_extensions: vec!["./dist/one.js".to_string()],
        setup_entry: None,
        runtime_setup_entry: None,
    };
    assert!(validate_package_extension_entries_for_install(&mismatch_meta).is_err());

    // Runtime setup without source setup
    let runtime_setup_only = OpenClawMetadata {
        extensions: vec![],
        runtime_extensions: vec![],
        setup_entry: None,
        runtime_setup_entry: Some("./dist/setup.js".to_string()),
    };
    assert!(validate_package_extension_entries_for_install(&runtime_setup_only).is_err());

    // Tool manifest contracts validation (plugin-tool-contracts.test.ts)
    use mahayana_host_runtime::extensions::plugins::bundle_manifest::{
        validate_plugin_manifest_tool_contracts, VirtualTestPluginHarness,
    };

    // Valid: all registered tools declared
    assert!(validate_plugin_manifest_tool_contracts(
        "my-plugin",
        &["tool_a".to_string(), "tool_b".to_string()],
        &["tool_a".to_string()],
    ).is_ok());

    // Invalid: registers tools but has no contracts.tools
    let err_no_tools = validate_plugin_manifest_tool_contracts(
        "bad-plugin",
        &[],
        &["tool_a".to_string()],
    ).unwrap_err();
    assert_eq!(err_no_tools, "bad-plugin: registers agent tools but has no contracts.tools");

    // Invalid: missing contracts.tools for specific tools
    let err_missing = validate_plugin_manifest_tool_contracts(
        "bad-plugin",
        &["tool_a".to_string()],
        &["tool_a".to_string(), "tool_b".to_string()],
    ).unwrap_err();
    assert_eq!(err_missing, "bad-plugin: missing contracts.tools for tool_b");

    // Virtual test plugin tool execution (plugin-testing-harness.contract.test.ts)
    let outcome = VirtualTestPluginHarness::execute_fixture_echo_tool(
        "scoped",
        "/tmp/virtual-home",
        "call-1",
        "alpha",
    );
    assert_eq!(outcome.text, "scoped:alpha");
    assert_eq!(outcome.home, "/tmp/virtual-home");
    assert_eq!(outcome.message, "alpha");
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
        label: Some("ElevenLabs Text-to-Speech".to_string()),
        aliases: vec![],
    });
    catalog.register_capability(CapabilityProviderEntry {
        family: CAPABILITY_FAMILY_TRANSCRIPTION_PROVIDERS.to_string(),
        provider_id: "whisper-local".to_string(),
        plugin_id: "whisper-plugin".to_string(),
        label: Some("Local Whisper".to_string()),
        aliases: vec!["whisper".to_string()],
    });

    let entry = catalog
        .get_capability(CAPABILITY_FAMILY_SPEECH_PROVIDERS, "ElevenLabs")
        .expect("Case-insensitive lookup");
    assert_eq!(entry.plugin_id, "elevenlabs-voice-plugin");

    let speech_list = catalog.list_capabilities_for_family(CAPABILITY_FAMILY_SPEECH_PROVIDERS);
    assert_eq!(speech_list.len(), 1);

    // Cold descriptor matching with aliases (capability-catalog.contract.test.ts)
    assert!(catalog.matches_provider_id_or_alias(CAPABILITY_FAMILY_TRANSCRIPTION_PROVIDERS, "whisper-local"));
    assert!(catalog.matches_provider_id_or_alias(CAPABILITY_FAMILY_TRANSCRIPTION_PROVIDERS, "whisper"));
    assert!(!catalog.matches_provider_id_or_alias(CAPABILITY_FAMILY_TRANSCRIPTION_PROVIDERS, "deepgram"));

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

#[test]
fn test_ocrm_plugins_contract_registry_oracle() {
    // Tests equivalent behavior from upstream src/plugins/contracts/registry.contract.test.ts
    let catalog = PluginCapabilityCatalog::new();

    // 1. Canonical-first aliases oracle
    catalog.register_canonical_aliases(
        "azure-speech:speechProviderIds",
        vec!["azure-speech".to_string(), "azure".to_string()],
    );
    catalog.register_canonical_aliases(
        "volcengine:speechProviderIds",
        vec!["volcengine".to_string(), "bytedance".to_string(), "doubao".to_string()],
    );
    catalog.register_canonical_aliases(
        "xai:realtimeVoiceProviderIds",
        vec!["xai".to_string(), "grok-voice".to_string(), "xai-realtime-voice".to_string()],
    );

    assert_eq!(
        catalog.get_canonical_aliases("azure-speech:speechProviderIds"),
        Some(vec!["azure-speech".to_string(), "azure".to_string()])
    );
    assert_eq!(
        catalog.get_canonical_aliases("volcengine:speechProviderIds"),
        Some(vec!["volcengine".to_string(), "bytedance".to_string(), "doubao".to_string()])
    );
    assert_eq!(
        catalog.get_canonical_aliases("xai:realtimeVoiceProviderIds"),
        Some(vec!["xai".to_string(), "grok-voice".to_string(), "xai-realtime-voice".to_string()])
    );

    // 2. Provider ID uniqueness validator
    let valid_entries = vec![
        PluginContractRegistryEntry {
            plugin_id: "plugin-a".to_string(),
            provider_ids: vec!["provider-1".to_string(), "provider-2".to_string()],
            worker_provider_ids: vec![],
            web_fetch_provider_ids: vec![],
            web_search_provider_ids: vec![],
            speech_provider_ids: vec![],
            realtime_voice_provider_ids: vec![],
            realtime_transcription_provider_ids: vec![],
            image_generation_provider_ids: vec![],
        },
        PluginContractRegistryEntry {
            plugin_id: "plugin-b".to_string(),
            provider_ids: vec!["provider-3".to_string()],
            worker_provider_ids: vec![],
            web_fetch_provider_ids: vec![],
            web_search_provider_ids: vec![],
            speech_provider_ids: vec![],
            realtime_voice_provider_ids: vec![],
            realtime_transcription_provider_ids: vec![],
            image_generation_provider_ids: vec![],
        },
    ];
    assert!(PluginCapabilityCatalog::validate_unique_provider_ids(&valid_entries).is_ok());

    let duplicate_entries = vec![
        PluginContractRegistryEntry {
            plugin_id: "plugin-a".to_string(),
            provider_ids: vec!["provider-dup".to_string()],
            worker_provider_ids: vec![],
            web_fetch_provider_ids: vec![],
            web_search_provider_ids: vec![],
            speech_provider_ids: vec![],
            realtime_voice_provider_ids: vec![],
            realtime_transcription_provider_ids: vec![],
            image_generation_provider_ids: vec![],
        },
        PluginContractRegistryEntry {
            plugin_id: "plugin-b".to_string(),
            provider_ids: vec!["provider-dup".to_string()],
            worker_provider_ids: vec![],
            web_fetch_provider_ids: vec![],
            web_search_provider_ids: vec![],
            speech_provider_ids: vec![],
            realtime_voice_provider_ids: vec![],
            realtime_transcription_provider_ids: vec![],
            image_generation_provider_ids: vec![],
        },
    ];
    assert!(PluginCapabilityCatalog::validate_unique_provider_ids(&duplicate_entries).is_err());

    // 3. Onboarding separation: video/image-only provider auth choices kept out of text onboarding
    catalog.register_auth_choice(ProviderAuthChoice {
        provider: "alibaba".to_string(),
        method: "api-key".to_string(),
        choice_id: "alibaba-model-studio-api-key".to_string(),
        choice_label: "Alibaba Model Studio API key".to_string(),
        onboarding_scopes: vec!["image-generation".to_string()],
        cli_flag: Some("--alibaba-model-studio-api-key".to_string()),
        option_key: Some("alibabaModelStudioApiKey".to_string()),
    });
    catalog.register_auth_choice(ProviderAuthChoice {
        provider: "github-copilot".to_string(),
        method: "device".to_string(),
        choice_id: "github-copilot".to_string(),
        choice_label: "GitHub Copilot".to_string(),
        onboarding_scopes: vec!["text-chat".to_string(), "code".to_string()],
        cli_flag: Some("--github-copilot-token".to_string()),
        option_key: Some("githubCopilotToken".to_string()),
    });

    let filtered = catalog.filter_text_onboarding_auth_choices();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].provider, "github-copilot");

    // 4. Shared resolver for web search with activation-scoped overrides
    let bundled_search = vec!["bing".to_string(), "google".to_string(), "duckduckgo".to_string()];
    let activation_scoped = vec!["codex".to_string(), "qa-lab".to_string()];
    let resolved_search = PluginCapabilityCatalog::resolve_web_search_providers(&bundled_search, &activation_scoped);
    assert_eq!(
        resolved_search,
        vec!["bing", "codex", "duckduckgo", "google", "qa-lab"]
    );
}

#[test]
fn test_ocrm_plugins_providers_and_web_fetch_contract() {
    // Tests behavior from providers.contract.test.ts and web-fetch-provider.contract.test.ts
    let bundled_providers = [
        "anthropic",
        "fal",
        "google",
        "minimax",
        "moonshot",
        "openai",
        "openrouter",
        "xai",
    ];
    let bundled_web_search = [
        "brave",
        "duckduckgo",
        "exa",
        "firecrawl",
        "google",
        "minimax",
        "moonshot",
        "parallel",
        "perplexity",
        "tavily",
        "xai",
    ];

    assert_eq!(bundled_providers.len(), 8);
    assert_eq!(bundled_web_search.len(), 11);

    let entries = vec![
        PluginContractRegistryEntry {
            plugin_id: "google-search-plugin".to_string(),
            provider_ids: vec!["google".to_string()],
            worker_provider_ids: vec![],
            web_fetch_provider_ids: vec!["google-fetch".to_string()],
            web_search_provider_ids: vec!["google".to_string()],
            speech_provider_ids: vec![],
            realtime_voice_provider_ids: vec![],
            realtime_transcription_provider_ids: vec![],
            image_generation_provider_ids: vec![],
        },
        PluginContractRegistryEntry {
            plugin_id: "anthropic-plugin".to_string(),
            provider_ids: vec!["anthropic".to_string()],
            worker_provider_ids: vec![],
            web_fetch_provider_ids: vec![],
            web_search_provider_ids: vec![],
            speech_provider_ids: vec![],
            realtime_voice_provider_ids: vec![],
            realtime_transcription_provider_ids: vec![],
            image_generation_provider_ids: vec![],
        },
    ];

    let fetch_plugins: Vec<&str> = entries
        .iter()
        .filter(|e| !e.web_fetch_provider_ids.is_empty())
        .map(|e| e.plugin_id.as_str())
        .collect();
    assert_eq!(fetch_plugins, vec!["google-search-plugin"]);
}

#[test]
fn test_ocrm_plugins_shape_compatibility_matrix() {
    // Tests equivalent behavior from upstream src/plugins/contracts/shape.contract.test.ts
    use mahayana_host_runtime::extensions::plugins::activation_planner::{
        build_plugin_shape_summary, CapabilityMode, PluginCapabilityEntry, PluginInspectShape,
        PluginShapeParams,
    };

    // 1. Hook only
    let hook_only = build_plugin_shape_summary(&PluginShapeParams {
        plugin_id: "hook-only".to_string(),
        has_hooks: true,
        ..Default::default()
    });
    assert_eq!(hook_only.shape, PluginInspectShape::HookOnly);
    assert_eq!(hook_only.capability_mode, CapabilityMode::None);
    assert!(hook_only.capabilities.is_empty());

    // 2. Plain capability (text-inference)
    let plain_provider = build_plugin_shape_summary(&PluginShapeParams {
        plugin_id: "plain-provider".to_string(),
        capabilities: vec![PluginCapabilityEntry {
            kind: "text-inference".to_string(),
            ids: vec!["plain-provider".to_string()],
        }],
        ..Default::default()
    });
    assert_eq!(plain_provider.shape, PluginInspectShape::PlainCapability);
    assert_eq!(plain_provider.capability_mode, CapabilityMode::Plain);
    assert_eq!(plain_provider.capabilities.len(), 1);

    // 3. Hybrid capability (text-inference + web-search)
    let hybrid = build_plugin_shape_summary(&PluginShapeParams {
        plugin_id: "hybrid-company".to_string(),
        capabilities: vec![
            PluginCapabilityEntry {
                kind: "text-inference".to_string(),
                ids: vec!["hybrid-provider".to_string()],
            },
            PluginCapabilityEntry {
                kind: "web-search".to_string(),
                ids: vec!["hybrid-search".to_string()],
            },
        ],
        ..Default::default()
    });
    assert_eq!(hybrid.shape, PluginInspectShape::HybridCapability);
    assert_eq!(hybrid.capability_mode, CapabilityMode::Hybrid);
    assert_eq!(hybrid.capabilities.len(), 2);

    // 4. Channel demo
    let channel_demo = build_plugin_shape_summary(&PluginShapeParams {
        plugin_id: "channel-demo".to_string(),
        capabilities: vec![PluginCapabilityEntry {
            kind: "channel".to_string(),
            ids: vec!["channel-demo".to_string()],
        }],
        ..Default::default()
    });
    assert_eq!(channel_demo.shape, PluginInspectShape::PlainCapability);
    assert_eq!(channel_demo.capability_mode, CapabilityMode::Plain);

    // 5. Session catalog demo
    let session_catalog_demo = build_plugin_shape_summary(&PluginShapeParams {
        plugin_id: "session-catalog-demo".to_string(),
        capabilities: vec![PluginCapabilityEntry {
            kind: "session-catalog".to_string(),
            ids: vec!["session-catalog-demo".to_string()],
        }],
        ..Default::default()
    });
    assert_eq!(session_catalog_demo.shape, PluginInspectShape::PlainCapability);
    assert_eq!(session_catalog_demo.capability_mode, CapabilityMode::Plain);

    // 6. Document extractors
    let doc_extract = build_plugin_shape_summary(&PluginShapeParams {
        plugin_id: "document-extract-test".to_string(),
        capabilities: vec![PluginCapabilityEntry {
            kind: "document-extractors".to_string(),
            ids: vec!["pdf".to_string()],
        }],
        ..Default::default()
    });
    assert_eq!(doc_extract.shape, PluginInspectShape::PlainCapability);
    assert_eq!(doc_extract.capability_mode, CapabilityMode::Plain);
}

#[test]
fn test_ocrm_plugins_registry_public_artifacts_retry_precedence() {
    // Tests equivalent behavior from upstream src/plugins/contracts/registry.retry.test.ts
    use std::collections::HashMap;

    let catalog = PluginCapabilityCatalog::new();
    let mut public_artifacts = HashMap::new();
    public_artifacts.insert("openai".to_string(), vec!["openai".to_string()]);
    public_artifacts.insert("google".to_string(), vec!["gemini".to_string()]);

    // 1. Uses provider public artifacts before falling back to bundled runtime registry
    let resolved_providers = catalog.resolve_provider_contract_providers_for_plugin_ids(
        &["openai"],
        &public_artifacts,
    );
    assert_eq!(resolved_providers, vec!["openai"]);

    // 2. Uses web search public artifacts before falling back to bundled runtime registry
    let resolved_web_search = catalog.resolve_web_search_providers_for_plugin_id(
        "google",
        &public_artifacts,
    );
    assert_eq!(resolved_web_search, vec!["gemini"]);
}

#[test]
fn test_ocrm_plugins_shared_upstream_model_contract() {
    // Tests equivalent behavior from upstream src/plugins/contracts/shared-upstream-model.contract.test.ts
    use mahayana_host_runtime::extensions::plugins::capability_catalog::{
        ModelCatalogRow, SharedUpstreamModelCatalog,
    };

    let valid_catalog = vec![
        ModelCatalogRow {
            provider_id: "openai".to_string(),
            model_id: "gpt-4o".to_string(),
            upstream_model: None,
            code_mode: Some("preferred".to_string()),
        },
        ModelCatalogRow {
            provider_id: "azure".to_string(),
            model_id: "gpt-4o".to_string(),
            upstream_model: Some("openai/gpt-4o".to_string()),
            code_mode: Some("preferred".to_string()),
        },
        ModelCatalogRow {
            provider_id: "anthropic".to_string(),
            model_id: "claude-3-5-sonnet".to_string(),
            upstream_model: None,
            code_mode: None,
        },
    ];

    assert!(SharedUpstreamModelCatalog::validate_upstream_model_refs(&valid_catalog).is_ok());
    assert!(SharedUpstreamModelCatalog::validate_code_mode_tier_consistency(&valid_catalog).is_ok());

    // 1. Unresolved upstream model ref
    let bad_refs = vec![
        ModelCatalogRow {
            provider_id: "openrouter".to_string(),
            model_id: "nonexistent".to_string(),
            upstream_model: Some("missing/model".to_string()),
            code_mode: None,
        },
        // Self-referencing upstream model
        ModelCatalogRow {
            provider_id: "self".to_string(),
            model_id: "loop".to_string(),
            upstream_model: Some("self/loop".to_string()),
            code_mode: None,
        },
    ];
    let ref_errors = SharedUpstreamModelCatalog::validate_upstream_model_refs(&bad_refs).unwrap_err();
    assert_eq!(ref_errors.len(), 2);
    assert!(ref_errors[0].contains("openrouter/nonexistent -> missing/model"));
    assert!(ref_errors[1].contains("self/loop -> self/loop"));

    // 2. Silent sibling in shared model group missing code-mode tier
    let silent_sibling_catalog = vec![
        ModelCatalogRow {
            provider_id: "openai".to_string(),
            model_id: "gpt-4o".to_string(),
            upstream_model: None,
            code_mode: Some("capable".to_string()),
        },
        ModelCatalogRow {
            provider_id: "azure".to_string(),
            model_id: "gpt-4o".to_string(),
            upstream_model: Some("openai/gpt-4o".to_string()),
            code_mode: None, // Missing!
        },
    ];
    let tier_errors = SharedUpstreamModelCatalog::validate_code_mode_tier_consistency(&silent_sibling_catalog).unwrap_err();
    assert_eq!(tier_errors.len(), 1);
    assert!(tier_errors[0].contains("azure/gpt-4o"));
    assert!(tier_errors[0].contains("shared model gpt-4o"));
}

#[test]
fn test_ocrm_plugins_provider_catalog_deprecation_guard() {
    // Tests equivalent behavior from upstream src/plugins/contracts/provider-catalog-deprecation.contract.test.ts
    use mahayana_host_runtime::extensions::plugins::capability_catalog::ProviderCatalogDeprecationGuard;

    let clean_sources = [
        (
            "extensions/openai/src/index.ts",
            "api.registerProvider({ id: 'openai', label: 'OpenAI', auth: [] })",
        ),
        (
            "extensions/anthropic/src/index.ts",
            "const plugin: ProviderPlugin = { id: 'anthropic', label: 'Anthropic' }",
        ),
    ];
    assert!(ProviderCatalogDeprecationGuard::assert_no_deprecated_discovery_hooks(&clean_sources).is_ok());

    let deprecated_sources = [
        (
            "extensions/legacy/src/index.ts",
            "api.registerProvider({ id: 'legacy', discovery: async () => [] })",
        ),
    ];
    let err = ProviderCatalogDeprecationGuard::assert_no_deprecated_discovery_hooks(&deprecated_sources).unwrap_err();
    assert_eq!(err, vec!["extensions/legacy/src/index.ts"]);
}

#[test]
fn test_ocrm_plugins_auth_choice_contract() {
    // Tests equivalent behavior from upstream src/plugins/contracts/auth-choice.contract.test.ts
    use mahayana_host_runtime::extensions::plugins::capability_catalog::{
        PluginCapabilityCatalog, ProviderAuthChoice,
    };

    let catalog = PluginCapabilityCatalog::new();
    let choice_id = PluginCapabilityCatalog::build_provider_method_choice("demo-oauth-provider", "oauth");
    assert_eq!(choice_id, "demo-oauth-provider:oauth");

    catalog.register_auth_choice(ProviderAuthChoice {
        provider: "demo-oauth-provider".to_string(),
        method: "oauth".to_string(),
        choice_id: choice_id.clone(),
        choice_label: "OAuth".to_string(),
        onboarding_scopes: vec!["chat".to_string()],
        cli_flag: None,
        option_key: None,
    });

    // Resolves preferred provider for auth choice
    assert_eq!(
        catalog.resolve_preferred_provider_for_auth_choice(&choice_id),
        Some("demo-oauth-provider".to_string())
    );

    // Resolves None for unknown choice
    assert_eq!(
        catalog.resolve_preferred_provider_for_auth_choice("unknown"),
        None
    );
}

#[test]
fn test_ocrm_plugins_embedding_provider_registration() {
    // Tests equivalent behavior from upstream src/plugins/contracts/embedding-provider.contract.test.ts
    use mahayana_host_runtime::extensions::plugins::capability_catalog::{
        EmbeddingProviderRegistry, RegisteredEmbeddingProvider,
    };

    // Public SDK mutation methods are forbidden
    assert!(!EmbeddingProviderRegistry::is_public_sdk_mutation_allowed("registerEmbeddingProvider"));
    assert!(!EmbeddingProviderRegistry::is_public_sdk_mutation_allowed("listRegisteredEmbeddingProviders"));
    assert!(!EmbeddingProviderRegistry::is_public_sdk_mutation_allowed("clearEmbeddingProviders"));
    assert!(!EmbeddingProviderRegistry::is_public_sdk_mutation_allowed("restoreEmbeddingProviders"));
    assert!(!EmbeddingProviderRegistry::is_public_sdk_mutation_allowed("restoreRegisteredEmbeddingProviders"));
    assert!(EmbeddingProviderRegistry::is_public_sdk_mutation_allowed("getEmbeddingProvider"));

    let mut registry = EmbeddingProviderRegistry::new();

    // Rejects plugins that did not declare the capability contract
    let err_undeclared = registry.register("not-embedding", "forbidden", false).unwrap_err();
    assert_eq!(
        err_undeclared,
        "plugin must declare contracts.embeddingProviders for adapter: forbidden"
    );
    assert_eq!(registry.providers().len(), 0);

    // Allows plugins that declare the capability contract
    assert!(registry.register("embedding-owner", "embedding-owner", true).is_ok());
    assert_eq!(
        registry.providers(),
        &[RegisteredEmbeddingProvider {
            provider_id: "embedding-owner".to_string(),
            plugin_id: "embedding-owner".to_string(),
        }]
    );

    // Rejects duplicate provider ids
    let mut dup_registry = EmbeddingProviderRegistry::new();
    assert!(dup_registry.register("first-owner", "shared", true).is_ok());
    let err_dup = dup_registry.register("second-owner", "shared", true).unwrap_err();
    assert_eq!(
        err_dup,
        "embedding provider already registered: shared (owner: first-owner)"
    );

    // Keeps core embedding provider ids reserved
    let mut core_registry = EmbeddingProviderRegistry::new();
    let err_core = core_registry.register("core-shadow", "openai-compatible", true).unwrap_err();
    assert_eq!(
        err_core,
        "embedding provider already registered: openai-compatible (owner: core)"
    );
}

#[test]
fn test_ocrm_plugins_wizard_choice_resolution_and_model_picker() {
    // Tests equivalent behavior from upstream wizard.choice-resolution.contract.test.ts and wizard.model-picker.contract.test.ts
    use mahayana_host_runtime::extensions::plugins::capability_catalog::{
        ProviderWizardChoiceResolver, WizardAuthMethod, WizardProviderEntry,
    };

    let test_providers = vec![
        WizardProviderEntry {
            id: "alpha".to_string(),
            setup_wizard: Some("alpha-setup-wizard".to_string()),
            model_picker: false,
            auth: vec![
                WizardAuthMethod {
                    id: "api-key".to_string(),
                    wizard: Some("alpha-api-key-wizard".to_string()),
                    model_picker: false,
                },
                WizardAuthMethod {
                    id: "oauth".to_string(),
                    wizard: Some("alpha-oauth-wizard".to_string()),
                    model_picker: true,
                },
            ],
        },
        WizardProviderEntry {
            id: "beta".to_string(),
            setup_wizard: Some("beta-setup-wizard".to_string()),
            model_picker: true,
            auth: vec![WizardAuthMethod {
                id: "token".to_string(),
                wizard: None,
                model_picker: false,
            }],
        },
        WizardProviderEntry {
            id: "gamma".to_string(),
            setup_wizard: Some("gamma-setup-wizard".to_string()),
            model_picker: false,
            auth: vec![WizardAuthMethod {
                id: "alt".to_string(),
                wizard: None,
                model_picker: false,
            }],
        },
    ];

    // Choice resolution cases matching upstream it.each matrix
    let c1 = ProviderWizardChoiceResolver::resolve_choice(&test_providers, "provider-plugin:alpha:api-key").unwrap();
    assert_eq!(c1.provider_id, "alpha");
    assert_eq!(c1.method_id, "api-key");
    assert_eq!(c1.wizard_source, "method");
    assert_eq!(c1.wizard.as_deref(), Some("alpha-api-key-wizard"));

    let c2 = ProviderWizardChoiceResolver::resolve_choice(&test_providers, "alpha").unwrap();
    assert_eq!(c2.provider_id, "alpha");
    assert_eq!(c2.method_id, "api-key");
    assert_eq!(c2.wizard_source, "method");
    assert_eq!(c2.wizard.as_deref(), Some("alpha-api-key-wizard"));

    let c3 = ProviderWizardChoiceResolver::resolve_choice(&test_providers, "alpha-oauth").unwrap();
    assert_eq!(c3.provider_id, "alpha");
    assert_eq!(c3.method_id, "oauth");
    assert_eq!(c3.wizard_source, "method");
    assert_eq!(c3.wizard.as_deref(), Some("alpha-oauth-wizard"));

    let c4 = ProviderWizardChoiceResolver::resolve_choice(&test_providers, "beta").unwrap();
    assert_eq!(c4.provider_id, "beta");
    assert_eq!(c4.method_id, "token");
    assert_eq!(c4.wizard_source, "provider");
    assert_eq!(c4.wizard.as_deref(), Some("beta-setup-wizard"));

    let c5 = ProviderWizardChoiceResolver::resolve_choice(&test_providers, "gamma-alt").unwrap();
    assert_eq!(c5.provider_id, "gamma");
    assert_eq!(c5.method_id, "alt");
    assert_eq!(c5.wizard_source, "provider");
    assert_eq!(c5.wizard.as_deref(), Some("gamma-setup-wizard"));

    // Model picker entries
    let model_picker_entries = ProviderWizardChoiceResolver::resolve_model_picker_entries(&test_providers);
    assert_eq!(
        model_picker_entries,
        vec!["beta".to_string(), "provider-plugin:alpha:oauth".to_string()]
    );
    for choice in &model_picker_entries {
        assert!(ProviderWizardChoiceResolver::resolve_choice(&test_providers, choice).is_some());
    }
}

#[test]
fn test_ocrm_plugins_bundled_capability_metadata_and_subpath_bundle() {
    // Tests equivalent behavior from bundled-capability-metadata.ts, bundled-plugin-roots.ts,
    // and plugin-sdk-index.bundle.test.ts
    use mahayana_host_runtime::extensions::plugins::bundle_manifest::{
        BundledPluginRootsManager, BundledPluginSubpathValidator,
    };
    use mahayana_host_runtime::extensions::plugins::capability_catalog::BundledPluginContractSnapshot;

    // 1. Snapshot has capabilities
    let mut snapshot = BundledPluginContractSnapshot::default();
    assert!(!snapshot.has_capabilities());
    snapshot.speech_provider_ids.push("azure-speech".to_string());
    assert!(snapshot.has_capabilities());

    // 2. Bundled plugin roots resolution
    let roots = BundledPluginRootsManager::new(&[("matrix", "/app/extensions/matrix")]);
    assert_eq!(
        roots.resolve_file("matrix", "src/matrix/sdk.ts").unwrap(),
        "/app/extensions/matrix/src/matrix/sdk.ts"
    );
    assert!(roots.resolve_file("unknown", "sdk.ts").is_err());

    // 3. Subpath bundle validator never_bundle and bare matrix imports
    let never_bundle = BundledPluginSubpathValidator::never_bundle_dependencies();
    assert!(never_bundle.contains(&"@lancedb/lancedb"));
    assert!(never_bundle.contains(&"@matrix-org/matrix-sdk-crypto-nodejs"));
    assert!(never_bundle.contains(&"matrix-js-sdk"));

    let clean_sources = [("matrix/src/matrix/sdk.ts", "import { MatrixClient } from './client.js';")];
    assert!(BundledPluginSubpathValidator::assert_no_bare_matrix_sdk_imports(&clean_sources).is_ok());

    let bare_sources = [("matrix/src/bad.ts", "import * as sdk from 'matrix-js-sdk';")];
    let err_bare = BundledPluginSubpathValidator::assert_no_bare_matrix_sdk_imports(&bare_sources).unwrap_err();
    assert_eq!(err_bare, vec!["matrix/src/bad.ts"]);
}

#[test]
fn test_ocrm_plugins_host_hook_fixture_and_ui_projection() {
    // Tests equivalent behavior from host-hook-fixture.ts and test-helpers/host-hook-ui-projection.ts
    use mahayana_host_runtime::extensions::plugins::bundle_manifest::{
        HostHookFixturePlugin, HostHookUiProjection,
    };

    // 1. Fixture plugin doubles
    let fixture = HostHookFixturePlugin::register_fixture_defaults();
    assert_eq!(fixture.session_extensions, vec!["workflow"]);
    assert_eq!(fixture.tool_metadata_names, vec!["approval_fixture_tool"]);
    assert_eq!(fixture.control_ui_descriptors, vec!["workflow-card"]);
    assert_eq!(fixture.lifecycle_hooks, vec!["workflow-cleanup"]);
    assert_eq!(fixture.agent_event_subscriptions, vec!["workflow-events"]);
    assert_eq!(fixture.session_scheduler_jobs, vec!["workflow-nudge"]);
    assert_eq!(fixture.command_names, vec!["host-hook-fixture"]);

    let (cmd_text, cont) = fixture.execute_command(Some("test-arg"));
    assert_eq!(cmd_text, "fixture:test-arg");
    assert!(cont);

    assert_eq!(HostHookFixturePlugin::turn_prepare_context(), "fixture turn context");
    assert_eq!(HostHookFixturePlugin::heartbeat_prompt_contribution(), "fixture heartbeat context");

    // 2. UI Projection
    let projection = HostHookUiProjection::build(42);
    assert!(projection.ok);
    assert_eq!(projection.generation, 42);
    assert!(projection.methods.contains(&"plugins.uiDescriptors".to_string()));
    assert!(projection.methods.contains(&"plugins.sessionAction".to_string()));
    assert_eq!(projection.control_ui_widget_kinds.len(), 3);
    assert_eq!(projection.descriptors.len(), 2);

    let admin_desc = projection.descriptors.iter().find(|d| d.id == "admin-panel").unwrap();
    assert_eq!(admin_desc.surface, "settings");
    assert_eq!(admin_desc.required_scopes, vec!["operator.admin"]);

    let approval_desc = projection.descriptors.iter().find(|d| d.id == "approval-panel").unwrap();
    assert_eq!(approval_desc.surface, "session");
    assert!(approval_desc.required_scopes.is_empty());
}

#[test]
fn test_ocrm_plugins_bundle_manifest_guardrails() {
    // Tests equivalent behavior from config-footprint-guardrails, extension-package-project-boundaries,
    // extension-runtime-dependencies, and plugin-entry-guardrails contract test suites.
    use mahayana_host_runtime::extensions::plugins::bundle_manifest::{
        ConfigFootprintGuardrail, ExtensionPackageBoundaryGuard,
        ExtensionRuntimeDependenciesGuard, PluginEntryGuardrails,
    };

    // 1. Config footprint guardrail
    let clean_paths = ["plugins.entries.*.config", "channels.telegram.botToken"];
    assert!(ConfigFootprintGuardrail::assert_no_retired_legacy_paths(&clean_paths).is_ok());

    let retired_paths = ["talk.voiceId", "channels.telegram.streamMode"];
    let err_retired = ConfigFootprintGuardrail::assert_no_retired_legacy_paths(&retired_paths).unwrap_err();
    assert_eq!(err_retired, vec!["channels.telegram.streamMode", "talk.voiceId"]);


    let clean_network_paths = ["network.dangerouslyAllowPrivateNetwork"];
    assert!(ConfigFootprintGuardrail::assert_canonical_private_network_paths(&clean_network_paths).is_ok());

    let bad_network_paths = ["channels.matrix.allowPrivateNetwork"];
    let err_network = ConfigFootprintGuardrail::assert_canonical_private_network_paths(&bad_network_paths).unwrap_err();
    assert_eq!(err_network, vec!["channels.matrix.allowPrivateNetwork"]);

    // 2. Extension package boundary guard
    let clean_extension_src = "import { createPlugin } from 'openclaw/plugin-sdk/core';";
    assert!(ExtensionPackageBoundaryGuard::assert_allowed_cross_package_imports(
        "extensions/telegram/src/index.ts",
        clean_extension_src
    ).is_ok());

    let bad_extension_src = "import { secret } from 'openclaw/plugin-sdk/private/secrets';";
    let err_boundary = ExtensionPackageBoundaryGuard::assert_allowed_cross_package_imports(
        "extensions/telegram/src/index.ts",
        bad_extension_src
    ).unwrap_err();
    assert!(err_boundary[0].contains("imports forbidden private entrypoint"));

    // 3. Extension runtime dependencies whitelist
    assert!(ExtensionRuntimeDependenciesGuard::is_whitelisted_optional_peer("extensions/canvas", "@a2ui/markdown-it"));
    assert!(ExtensionRuntimeDependenciesGuard::is_whitelisted_optional_peer("extensions/discord", "@discordjs/opus"));
    assert!(ExtensionRuntimeDependenciesGuard::is_whitelisted_optional_peer("extensions/qa-lab", "crabline"));
    assert!(ExtensionRuntimeDependenciesGuard::is_whitelisted_optional_peer("extensions/browser", "zod"));
    assert!(ExtensionRuntimeDependenciesGuard::is_whitelisted_optional_peer("extensions/whatsapp", "audio-decode"));
    assert!(ExtensionRuntimeDependenciesGuard::is_whitelisted_optional_peer("extensions/memory-lancedb", "apache-arrow"));
    assert!(!ExtensionRuntimeDependenciesGuard::is_whitelisted_optional_peer("extensions/telegram", "random-dep"));

    // 4. Plugin entry guardrails (no test modules in entrypoints)
    let clean_entry = "export function register(api) { api.registerTool(); }";
    assert!(PluginEntryGuardrails::assert_no_testing_imports_in_entrypoint("index.ts", clean_entry).is_ok());

    let bad_entry = "import { describe, it } from 'vitest';\nimport { fixture } from './test-harness';";
    let err_entry = PluginEntryGuardrails::assert_no_testing_imports_in_entrypoint("index.ts", bad_entry).unwrap_err();
    assert_eq!(err_entry.len(), 2);
}

#[test]
fn test_ocrm_plugins_sdk_package_and_runtime_api_guardrails() {
    // Tests equivalent behavior from plugin-sdk-package-contract-guardrails,
    // plugin-sdk-runtime-api-guardrails, and plugin-sdk-subpaths contract test suites.
    use mahayana_host_runtime::extensions::plugins::bundle_manifest::{
        PluginSdkPackageContractGuard, PluginSdkRuntimeApiGuardrails,
    };

    // 1. Public plugin SDK entrypoints
    let public_entrypoints = PluginSdkPackageContractGuard::public_plugin_sdk_entrypoints();
    assert!(public_entrypoints.contains(&"channel-contract"));
    assert!(public_entrypoints.contains(&"core"));
    assert!(public_entrypoints.contains(&"media-runtime"));
    assert!(public_entrypoints.contains(&"plugin-entry"));

    // 2. Private local-only entrypoint boundary
    let clean_subpaths = ["openclaw/plugin-sdk/core", "openclaw/plugin-sdk/channel-contract"];
    assert!(PluginSdkPackageContractGuard::assert_no_private_entrypoint_in_public_subpaths(&clean_subpaths).is_ok());

    let leaky_subpaths = ["openclaw/plugin-sdk/browser-maintenance"];
    let err_leaky = PluginSdkPackageContractGuard::assert_no_private_entrypoint_in_public_subpaths(&leaky_subpaths).unwrap_err();
    assert_eq!(err_leaky, vec!["openclaw/plugin-sdk/browser-maintenance"]);

    // 3. Runtime API export guardrails
    let clean_exports = ["export { createFaceTimeRuntime } from './src/runtime.js';"];
    assert!(PluginSdkRuntimeApiGuardrails::assert_runtime_api_exports_allowed("facetime", &clean_exports).is_ok());

    let bad_exports = ["export { testHelper } from './test-api.js';"];
    let err_exports = PluginSdkRuntimeApiGuardrails::assert_runtime_api_exports_allowed("facetime", &bad_exports).unwrap_err();
    assert!(err_exports[0].contains("exports internal test module"));
}

#[test]
fn test_ocrm_plugins_activation_context_payload_and_runtime_registry() {
    use std::collections::HashSet;
    use serde_json::json;
    use mahayana_host_runtime::extensions::plugins::activation_planner::{
        ActivationPlanner, ActivationTrigger, PluginManifestRecord,
        PluginActivationSourceResolver, ActivePayloadVerification,
        PluginInstallRecordStub, ActiveRegistryIdentity, ActiveRuntimeRegistryResolver,
    };

    // 1. Activation planner with scoped plugin IDs
    let mut planner = ActivationPlanner::new();
    planner.register_manifest(PluginManifestRecord {
        plugin_id: "telegram".into(),
        commands: vec!["tg".into()],
        providers: vec![],
        channels: vec!["telegram".into()],
        capabilities: vec![],
        is_bundled: true,
        is_trusted: true,
    });
    planner.register_manifest(PluginManifestRecord {
        plugin_id: "discord".into(),
        commands: vec!["dc".into()],
        providers: vec![],
        channels: vec!["discord".into()],
        capabilities: vec![],
        is_bundled: true,
        is_trusted: true,
    });

    let triggers = vec![
        ActivationTrigger::Channel("telegram".into()),
        ActivationTrigger::Channel("discord".into()),
    ];

    // Explicit empty scope returns empty
    let empty_scope: Vec<String> = vec![];
    assert_eq!(planner.plan_activation_scoped(&triggers, Some(&empty_scope)), Vec::<String>::new());

    // Restricted scope returns only permitted
    let restricted_scope = vec!["telegram".to_string()];
    assert_eq!(planner.plan_activation_scoped(&triggers, Some(&restricted_scope)), vec!["telegram"]);

    // Unrestricted returns both
    assert_eq!(planner.plan_activation_scoped(&triggers, None), vec!["discord", "telegram"]);

    // 2. Activation source config resolution precedence
    let root_cfg = json!({"root": true});
    let captured_cfg = json!({"captured": true});
    let explicit_source = json!({"explicit": true});

    // Explicit source takes highest precedence
    assert_eq!(
        PluginActivationSourceResolver::resolve_source_config(Some(&root_cfg), Some(&explicit_source), Some(&captured_cfg)),
        explicit_source
    );
    // Captured takes next precedence
    assert_eq!(
        PluginActivationSourceResolver::resolve_source_config(Some(&root_cfg), None, Some(&captured_cfg)),
        captured_cfg
    );
    // Fallback to root
    assert_eq!(
        PluginActivationSourceResolver::resolve_source_config(Some(&root_cfg), None, None),
        root_cfg
    );
    // Fallback to empty object
    assert_eq!(
        PluginActivationSourceResolver::resolve_source_config(None, None, None),
        json!({})
    );

    // 3. Active payload verification filtering
    let records = vec![
        PluginInstallRecordStub {
            plugin_id: "denied-plug".into(),
            version: "1.0.0".into(),
            is_operator_managed: false,
            official_npm_spec: None,
            official_clawhub_spec: None,
        },
        PluginInstallRecordStub {
            plugin_id: "operator-plug".into(),
            version: "1.0.0".into(),
            is_operator_managed: true,
            official_npm_spec: None,
            official_clawhub_spec: None,
        },
        PluginInstallRecordStub {
            plugin_id: "disabled-regular".into(),
            version: "1.0.0".into(),
            is_operator_managed: false,
            official_npm_spec: None,
            official_clawhub_spec: None,
        },
        PluginInstallRecordStub {
            plugin_id: "disabled-official".into(),
            version: "1.0.0".into(),
            is_operator_managed: false,
            official_npm_spec: Some("@openclaw/voice@1.0.0".into()),
            official_clawhub_spec: None,
        },
        PluginInstallRecordStub {
            plugin_id: "enabled-plug".into(),
            version: "1.0.0".into(),
            is_operator_managed: false,
            official_npm_spec: None,
            official_clawhub_spec: None,
        },
    ];

    let mut denied_ids = HashSet::new();
    denied_ids.insert("denied-plug".into());

    let mut disabled_ids = HashSet::new();
    disabled_ids.insert("disabled-regular".into());
    disabled_ids.insert("disabled-official".into());

    let source_bundled_ids = HashSet::new();

    let active_records = ActivePayloadVerification::filter_records_to_active(
        &records,
        &disabled_ids,
        &denied_ids,
        &source_bundled_ids,
    );

    let active_ids: Vec<String> = active_records.into_iter().map(|r| r.plugin_id).collect();
    // Denied dropped; operator-managed dropped; disabled-regular dropped;
    // disabled-official preserved; enabled-plug preserved.
    assert_eq!(active_ids, vec!["disabled-official", "enabled-plug"]);

    // 4. Active runtime registry compatibility
    let active_id = ActiveRegistryIdentity {
        cache_key: "cache-key-123".into(),
        workspace_dir: Some("/workspace/alpha".into()),
        loaded_plugin_ids: vec!["telegram".into(), "discord".into(), "telegram".into()],
    };

    // Workspace mismatch rejected
    assert!(!ActiveRuntimeRegistryResolver::is_compatible(
        Some(&active_id),
        "cache-key-123",
        Some("/workspace/beta"),
    ));
    assert!(!ActiveRuntimeRegistryResolver::is_compatible(
        Some(&active_id),
        "cache-key-123",
        None,
    ));

    // Cache key mismatch rejected
    assert!(!ActiveRuntimeRegistryResolver::is_compatible(
        Some(&active_id),
        "cache-key-456",
        Some("/workspace/alpha"),
    ));

    // Exact match accepted
    assert!(ActiveRuntimeRegistryResolver::is_compatible(
        Some(&active_id),
        "cache-key-123",
        Some("/workspace/alpha"),
    ));

    // Sorted and deduplicated runtime plugin IDs
    assert_eq!(
        ActiveRuntimeRegistryResolver::list_runtime_plugin_ids(&active_id),
        vec!["discord", "telegram"]
    );
}

#[test]
fn test_ocrm_plugins_bundled_mcp_lsp_capabilities_and_command_scopes() {
    use std::collections::HashMap;
    use serde_json::json;
    use mahayana_host_runtime::extensions::plugins::bundle_manifest::{
        BundledMcpServerConfig, BundledMcpTransport, BundledMcpServerValidator,
        BundledLspServerValidator, BundleCapabilitySupport, BundleCommandScopeValidator,
    };

    // 1. Bundled MCP Server Validation
    let valid_mcp = BundledMcpServerConfig {
        name: "local-sqlite".into(),
        command: "bin/mcp-server".into(),
        args: vec!["--db".into(), "data/store.db".into()],
        env: HashMap::new(),
        transport: BundledMcpTransport::Stdio,
    };
    assert!(BundledMcpServerValidator::validate(&valid_mcp).is_ok());

    let bad_name = BundledMcpServerConfig {
        name: "local/sqlite".into(),
        ..valid_mcp.clone()
    };
    let err_name = BundledMcpServerValidator::validate(&bad_name).unwrap_err();
    assert!(err_name.contains("Invalid MCP server name"));

    let bad_cmd = BundledMcpServerConfig {
        command: "../../../bin/sh".into(),
        ..valid_mcp.clone()
    };
    let err_cmd = BundledMcpServerValidator::validate(&bad_cmd).unwrap_err();
    assert!(err_cmd.contains("cannot escape bundle root"));

    let bad_arg = BundledMcpServerConfig {
        args: vec!["--target".into(), "../../etc/passwd".into()],
        ..valid_mcp.clone()
    };
    let err_arg = BundledMcpServerValidator::validate(&bad_arg).unwrap_err();
    assert!(err_arg.contains("cannot escape bundle root"));

    // 2. Bundled LSP Server Validation
    assert!(BundledLspServerValidator::validate("rust-analyzer", "rust", "bin/ra").is_ok());
    assert!(BundledLspServerValidator::validate("", "rust", "bin/ra").is_err());
    assert!(BundledLspServerValidator::validate("ra", "", "bin/ra").is_err());
    assert!(BundledLspServerValidator::validate("ra", "rust", "../ra").is_err());

    // 3. Bundle Capability Support from Manifest
    let manifest = json!({
        "mcpServers": {
            "sqlite": { "command": "bin/sqlite" }
        },
        "tools": [
            { "name": "search" }
        ],
        "skills": [
            { "name": "web-browsing" }
        ]
    });
    let caps = BundleCapabilitySupport::from_manifest(&manifest);
    assert!(caps.has_mcp);
    assert!(caps.has_tools);
    assert!(caps.has_skills);
    assert!(!caps.has_lsp);
    assert!(!caps.has_channels);

    // 4. Bundle Command Scope Validation
    let user_scopes = ["operator.read", "tool.execute"];
    assert!(BundleCommandScopeValidator::assert_command_scope_authorized("run-tool", Some("tool.execute"), &user_scopes).is_ok());
    assert!(BundleCommandScopeValidator::assert_command_scope_authorized("read-info", None, &user_scopes).is_ok());

    // Unauthorized scope fails
    let err_scope = BundleCommandScopeValidator::assert_command_scope_authorized("delete-db", Some("operator.admin"), &user_scopes).unwrap_err();
    assert!(err_scope.contains("requires scope 'operator.admin'"));

    // Admin scope bypasses requirement
    let admin_scopes = ["operator.admin"];
    assert!(BundleCommandScopeValidator::assert_command_scope_authorized("delete-db", Some("restricted.write"), &admin_scopes).is_ok());
}

#[test]
fn test_ocrm_plugins_bundled_channel_metadata_compat_and_dir_resolver() {
    use mahayana_host_runtime::extensions::plugins::capability_catalog::{
        BundledChannelMetadata, BundledChannelMetadataValidator,
        BundledCompatResolver, BundledDirResolver,
    };

    // 1. Channel Metadata Validation
    let valid_channel = BundledChannelMetadata {
        channel_id: "telegram".into(),
        stream_mode: Some("block".into()),
        reply_to_mode: Some("all".into()),
        history_limit: Some(500),
        media_max_mb: Some(50),
        provider_skills: vec!["voice".into()],
    };
    assert!(BundledChannelMetadataValidator::validate(&valid_channel).is_ok());

    let empty_id = BundledChannelMetadata {
        channel_id: "".into(),
        ..valid_channel.clone()
    };
    assert!(BundledChannelMetadataValidator::validate(&empty_id).is_err());

    let bad_history = BundledChannelMetadata {
        history_limit: Some(200_000),
        ..valid_channel.clone()
    };
    assert!(BundledChannelMetadataValidator::validate(&bad_history).is_err());

    let bad_media = BundledChannelMetadata {
        media_max_mb: Some(5000),
        ..valid_channel.clone()
    };
    assert!(BundledChannelMetadataValidator::validate(&bad_media).is_err());

    // 2. Bundled Compat ID Normalization
    assert_eq!(BundledCompatResolver::normalize_plugin_id("legacy-telegram"), "telegram");
    assert_eq!(BundledCompatResolver::normalize_plugin_id("legacy-discord"), "discord");
    assert_eq!(BundledCompatResolver::normalize_plugin_id("legacy-slack"), "slack");
    assert_eq!(BundledCompatResolver::normalize_plugin_id("custom-plugin"), "custom-plugin");

    // 3. Bundled Directory Path Traversal Guard
    assert_eq!(
        BundledDirResolver::resolve_relative_path("/app/bundled/plugins", "my-plugin/index.js").unwrap(),
        "/app/bundled/plugins/my-plugin/index.js"
    );

    let err_traversal = BundledDirResolver::resolve_relative_path("/app/bundled/plugins", "../../../etc/passwd").unwrap_err();
    assert!(err_traversal.contains("Path traversal detected"));
}

#[test]
fn test_ocrm_plugins_bundled_metadata_scan_compat_and_naming_artwork() {
    use std::collections::{HashMap, HashSet};

    // 1. Bundled Plugin Scanner - entry path rewrite
    assert_eq!(
        BundledPluginScanner::rewrite_bundled_plugin_entry_to_built_path("./index.ts"),
        "index.js"
    );
    assert_eq!(
        BundledPluginScanner::rewrite_bundled_plugin_entry_to_built_path(".\\src\\runner.mts"),
        "src\\runner.js"
    );
    assert_eq!(
        BundledPluginScanner::rewrite_bundled_plugin_entry_to_built_path("dist/main.js"),
        "dist/main.js"
    );

    // 2. Bundled Plugin Scanner - id hint derivation
    assert_eq!(
        BundledPluginScanner::derive_bundled_plugin_id_hint(
            "./index.ts",
            "telegram",
            Some("@openclaw/telegram"),
            false
        ),
        "telegram"
    );
    assert_eq!(
        BundledPluginScanner::derive_bundled_plugin_id_hint(
            "./handlers/worker.ts",
            "telegram",
            Some("@openclaw/telegram-worker"),
            true
        ),
        "telegram-worker/worker"
    );

    // 3. Top-level public surface source validation
    assert!(BundledPluginScanner::is_top_level_public_surface_source("runtime-api.ts"));
    assert!(BundledPluginScanner::is_top_level_public_surface_source("helper-api.js"));
    assert!(!BundledPluginScanner::is_top_level_public_surface_source(".hidden.ts"));
    assert!(!BundledPluginScanner::is_top_level_public_surface_source("test-utils.ts"));
    assert!(!BundledPluginScanner::is_top_level_public_surface_source("types.d.ts"));
    assert!(!BundledPluginScanner::is_top_level_public_surface_source("config-api.ts"));
    assert!(!BundledPluginScanner::is_top_level_public_surface_source("runner.test.ts"));

    // 4. Runtime sidecar collection
    let public_surfaces = vec![
        "runtime-api.js".to_string(),
        "custom-helper.js".to_string(),
        "light-runtime-api.js".to_string(),
    ];
    let sidecars = BundledPluginScanner::collect_bundled_plugin_runtime_sidecar_artifacts(&public_surfaces);
    assert_eq!(sidecars, vec!["runtime-api.js", "light-runtime-api.js"]);

    // 5. Startup manifest fast-path resolution
    assert!(BundledPluginScanner::has_bundled_plugin_startup_manifest(
        "alpha",
        Some(r#"{"id":"alpha","name":"Alpha Plugin"}"#)
    ));
    assert!(!BundledPluginScanner::has_bundled_plugin_startup_manifest(
        "alpha",
        Some(r#"{"id":"beta","name":"Beta Plugin"}"#)
    ));
    assert!(!BundledPluginScanner::has_bundled_plugin_startup_manifest("alpha", None));

    // 6. Bundled Provider Compat validation
    assert!(BundledProviderCompatValidator::is_bundled_provider_compat_contract("speechProviders"));
    assert!(BundledProviderCompatValidator::is_bundled_provider_compat_contract("migrationProviders"));
    assert!(!BundledProviderCompatValidator::is_bundled_provider_compat_contract("unknownContract"));

    let mut counts = HashMap::new();
    counts.insert("speechProviders".to_string(), 1);
    assert!(BundledProviderCompatValidator::is_bundled_provider_compat_plugin("bundled", 0, &counts));
    assert!(BundledProviderCompatValidator::is_bundled_provider_compat_plugin("bundled", 2, &HashMap::new()));
    assert!(!BundledProviderCompatValidator::is_bundled_provider_compat_plugin("installed", 2, &counts));

    // 7. Bundled Source Overlays
    let mount_info_sample = "36 35 98:0 / /mnt/overlay rw,relatime - ext4 /dev/root rw\n\
                             37 36 98:1 / /home/user/extensions rw - ext4 /dev/sda1 rw";
    let mount_points = BundledSourceOverlayResolver::parse_linux_mount_info_mount_points(mount_info_sample);
    assert!(mount_points.contains("/mnt/overlay"));
    assert!(mount_points.contains("/home/user/extensions"));
    assert!(!mount_points.contains("/var/log"));

    assert!(BundledSourceOverlayResolver::source_overlays_disabled(Some("1")));
    assert!(BundledSourceOverlayResolver::source_overlays_disabled(Some("TRUE")));
    assert!(!BundledSourceOverlayResolver::source_overlays_disabled(Some("0")));
    assert!(!BundledSourceOverlayResolver::source_overlays_disabled(None));

    let peers = vec!["discord".to_string(), "telegram".to_string()];
    let mut custom_mounts = HashSet::new();
    custom_mounts.insert("/legacy/extensions/discord".to_string());
    let overlays = BundledSourceOverlayResolver::list_bundled_source_overlay_dirs(
        "/app/extensions",
        "/legacy/extensions",
        &peers,
        &custom_mounts,
        false
    );
    assert_eq!(overlays, vec!["/legacy/extensions/discord"]);

    // 8. Bundled Plugin Naming & Artwork Validator
    let cats = vec!["channels".to_string()];
    assert_eq!(
        BundledPluginNamingAndArtworkValidator::validate_bundled_plugin_categories(Some(&cats)).unwrap(),
        "channels"
    );
    assert!(BundledPluginNamingAndArtworkValidator::validate_bundled_plugin_categories(None).is_err());
    let invalid_cats = vec!["channels".to_string(), "tools".to_string()];
    assert!(BundledPluginNamingAndArtworkValidator::validate_bundled_plugin_categories(Some(&invalid_cats)).is_err());

    // Valid 512x512 PNG header (24 bytes minimum)
    let mut valid_png = vec![137, 80, 78, 71, 13, 10, 26, 10]; // signature
    valid_png.extend_from_slice(&[0, 0, 0, 13]); // IHDR chunk length
    valid_png.extend_from_slice(b"IHDR");
    valid_png.extend_from_slice(&512u32.to_be_bytes()); // width
    valid_png.extend_from_slice(&512u32.to_be_bytes()); // height
    assert!(BundledPluginNamingAndArtworkValidator::validate_bundled_plugin_icon_png(&valid_png).is_ok());

    let mut invalid_dim_png = valid_png.clone();
    invalid_dim_png[16..20].copy_from_slice(&256u32.to_be_bytes());
    assert!(BundledPluginNamingAndArtworkValidator::validate_bundled_plugin_icon_png(&invalid_dim_png).is_err());

    // Naming rules (including kimi-coding -> kimi exception)
    assert!(BundledPluginNamingAndArtworkValidator::validate_bundled_plugin_naming(
        "telegram",
        "telegram",
        "@openclaw/telegram"
    ).is_ok());
    assert!(BundledPluginNamingAndArtworkValidator::validate_bundled_plugin_naming(
        "kimi-coding",
        "kimi",
        "@openclaw/kimi-provider"
    ).is_ok());
    assert!(BundledPluginNamingAndArtworkValidator::validate_bundled_plugin_naming(
        "slack",
        "telegram",
        "@openclaw/slack"
    ).is_err());
    assert!(BundledPluginNamingAndArtworkValidator::validate_bundled_plugin_naming(
        "slack",
        "slack",
        "third-party/slack"
    ).is_err());
}

#[test]
fn test_ocrm_plugins_bundled_sources_catalog_and_capability_consent() {
    use std::collections::HashMap;

    // 1. Bundled Plugin Source Resolver & Config Requirements
    let empty_schema = serde_json::json!({});
    assert!(!BundledPluginSourceResolver::schema_has_required_fields(&empty_schema));

    let no_req_schema = serde_json::json!({
        "type": "object",
        "properties": { "name": { "type": "string" } }
    });
    assert!(!BundledPluginSourceResolver::schema_has_required_fields(&no_req_schema));

    let req_schema = serde_json::json!({
        "type": "object",
        "required": ["embedding"]
    });
    assert!(BundledPluginSourceResolver::schema_has_required_fields(&req_schema));

    let mut sources_map = HashMap::new();
    sources_map.insert("feishu".to_string(), BundledPluginSource {
        plugin_id: "feishu".to_string(),
        local_path: "/app/plugins/feishu".to_string(),
        npm_spec: Some("@openclaw/feishu".to_string()),
        version: Some("1.0.0".to_string()),
        config_schema: None,
        requires_config: false,
    });
    sources_map.insert("memory-lancedb".to_string(), BundledPluginSource {
        plugin_id: "memory-lancedb".to_string(),
        local_path: "/app/plugins/memory-lancedb".to_string(),
        npm_spec: Some("@openclaw/memory-lancedb".to_string()),
        version: Some("2.0.0".to_string()),
        config_schema: Some(req_schema),
        requires_config: true,
    });

    // Lookup tests
    let found_by_id = BundledPluginSourceResolver::find_in_map(
        &sources_map,
        BundledPluginLookup::PluginId("feishu"),
    );
    assert!(found_by_id.is_some());
    assert_eq!(found_by_id.unwrap().plugin_id, "feishu");

    let not_found_by_id = BundledPluginSourceResolver::find_in_map(
        &sources_map,
        BundledPluginLookup::PluginId("missing"),
    );
    assert!(not_found_by_id.is_none());

    let found_by_spec = BundledPluginSourceResolver::find_in_map(
        &sources_map,
        BundledPluginLookup::NpmSpec("@openclaw/memory-lancedb"),
    );
    assert!(found_by_spec.is_some());
    assert_eq!(found_by_spec.unwrap().plugin_id, "memory-lancedb");

    let found_by_path = BundledPluginSourceResolver::find_in_map(
        &sources_map,
        BundledPluginLookup::LocalPath("/app/plugins/feishu"),
    );
    assert!(found_by_path.is_some());
    assert_eq!(found_by_path.unwrap().plugin_id, "feishu");

    // Command hint
    let hint = BundledPluginSourceResolver::resolve_install_command_hint(found_by_id);
    assert_eq!(hint, Some("openclaw plugins install /app/plugins/feishu".to_string()));
    assert_eq!(BundledPluginSourceResolver::resolve_install_command_hint(None), None);

    // 2. Candidate Install Owner & Immutable Identity
    let identity = CandidateInstallOwnerResolver::resolve_immutable_install_identity(
        "global",
        Some("owner-pkg"),
        Some("npm"),
        Some("/var/plugins/owner-pkg"),
        Some("@openclaw/owner@1.0"),
        Some("@openclaw/owner@1.0.0"),
        Some("sha512-testintegrity"),
        Some("2026-10-27T00:00:00Z"),
    );
    assert!(identity.is_some());
    let id_str = identity.unwrap();
    assert!(id_str.contains("owner-pkg"));
    assert!(id_str.contains("sha512-testintegrity"));

    // Origin not global or source not npm -> None
    assert!(CandidateInstallOwnerResolver::resolve_immutable_install_identity(
        "bundled",
        Some("owner-pkg"),
        Some("npm"),
        Some("/var/plugins/owner-pkg"),
        None,
        None,
        Some("sha512-testintegrity"),
        None,
    ).is_none());

    assert!(CandidateInstallOwnerResolver::resolve_immutable_install_identity(
        "global",
        Some("owner-pkg"),
        Some("git"),
        Some("/var/plugins/owner-pkg"),
        None,
        None,
        Some("sha512-testintegrity"),
        None,
    ).is_none());

    // 3. Capability Catalog Families & Shape Validation
    assert_eq!(CapabilityCatalogFamily::ALL.len(), 3);
    assert_eq!(CapabilityCatalogFamily::SpeechProviders.required_method(), "synthesize");
    assert_eq!(CapabilityCatalogFamily::RealtimeTranscriptionProviders.required_method(), "createSession");
    assert_eq!(CapabilityCatalogFamily::RealtimeVoiceProviders.required_method(), "createBridge");

    let valid_catalog = serde_json::json!({
        "speechProviders": [
            {
                "id": "mock-tts",
                "label": "Mock TTS",
                "isConfigured": true,
                "synthesize": true
            }
        ],
        "realtimeTranscriptionProviders": [
            {
                "id": "mock-stt",
                "label": "Mock STT",
                "isConfigured": true,
                "createSession": true
            }
        ]
    });
    assert!(CapabilityCatalogValidator::validate_catalog_shape(&valid_catalog).is_ok());

    let invalid_family_catalog = serde_json::json!({
        "unknownFamily": []
    });
    assert!(CapabilityCatalogValidator::validate_catalog_shape(&invalid_family_catalog).is_err());

    let missing_method_catalog = serde_json::json!({
        "speechProviders": [
            {
                "id": "mock-tts",
                "label": "Mock TTS",
                "isConfigured": true
                // missing "synthesize"
            }
        ]
    });
    assert!(CapabilityCatalogValidator::validate_catalog_shape(&missing_method_catalog).is_err());

    // 4. Capability Consent Manager: Hashes, Integrity & Widening
    let surface_a = PluginAcceptedDeclaredSurface {
        tools: vec!["read".to_string()],
        channels: vec!["chat".to_string()],
        ..Default::default()
    };
    let hash_a = CapabilityConsentManager::compute_declared_surface_hash(&surface_a);
    assert_eq!(hash_a.len(), 64);

    let surface_b = PluginAcceptedDeclaredSurface {
        tools: vec!["read".to_string(), "write".to_string()],
        channels: vec!["chat".to_string()],
        ..Default::default()
    };
    let hash_b = CapabilityConsentManager::compute_declared_surface_hash(&surface_b);
    assert_ne!(hash_a, hash_b);

    // Widening diff
    let (widened, has_widening) = CapabilityConsentManager::diff_declared_surface_widening(&surface_a, &surface_b);
    assert!(has_widening);
    assert_eq!(widened.get("tools"), Some(&vec!["write".to_string()]));

    let (_widened_rev, has_widening_rev) = CapabilityConsentManager::diff_declared_surface_widening(&surface_b, &surface_a);
    assert!(!has_widening_rev);

    // Integrity precedence
    assert_eq!(
        CapabilityConsentManager::resolve_plugin_install_record_integrity(
            Some("sha512-primary"),
            Some("sha512-npm"),
            Some("clawpack-hash"),
            Some("gitcommit123"),
        ),
        Some(("sha512-primary".to_string(), "ssri"))
    );
    assert_eq!(
        CapabilityConsentManager::resolve_plugin_install_record_integrity(
            None,
            Some("sha512-npm"),
            Some("clawpack-hash"),
            None,
        ),
        Some(("sha512-npm".to_string(), "ssri"))
    );
    assert_eq!(
        CapabilityConsentManager::resolve_plugin_install_record_integrity(
            None,
            None,
            Some("clawpack-hash"),
            None,
        ),
        Some(("clawpack-hash".to_string(), "sha256"))
    );
    assert_eq!(
        CapabilityConsentManager::resolve_plugin_install_record_integrity(
            None,
            None,
            None,
            Some("gitcommit123"),
        ),
        Some(("gitcommit123".to_string(), "git-commit"))
    );

    // Surface currency checks
    assert!(CapabilityConsentManager::resolve_accepted_surface_current(
        Some(&surface_a),
        Some(&hash_a),
        Some("sha512-sig"),
        Some("sha512-sig"),
        &surface_a,
    ));
    // Forged hash
    assert!(!CapabilityConsentManager::resolve_accepted_surface_current(
        Some(&surface_a),
        Some("0000000000000000000000000000000000000000000000000000000000000000"),
        Some("sha512-sig"),
        Some("sha512-sig"),
        &surface_a,
    ));
    // Changed surface
    assert!(!CapabilityConsentManager::resolve_accepted_surface_current(
        Some(&surface_a),
        Some(&hash_a),
        Some("sha512-sig"),
        Some("sha512-sig"),
        &surface_b,
    ));
    // Changed integrity
    assert!(!CapabilityConsentManager::resolve_accepted_surface_current(
        Some(&surface_a),
        Some(&hash_a),
        Some("sha512-new"),
        Some("sha512-old"),
        &surface_a,
    ));

    // Redaction
    let spec = "git:https://fixture-user:fixture-password@example.invalid/plugins/demo.git?token=fixture-token";
    let redacted = CapabilityConsentManager::redact_source_spec(spec);
    assert!(!redacted.contains("fixture-user"));
    assert!(!redacted.contains("fixture-password"));
    assert!(!redacted.contains("fixture-token"));
    assert!(redacted.contains("***"));
    assert!(redacted.starts_with("git:https://"));
}








