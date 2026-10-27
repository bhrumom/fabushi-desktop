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
