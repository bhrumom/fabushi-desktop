# Plugin Contract Registry & Bundled Capability Metadata Dossier

## 1. Scope & Source Attribution
- **Target Files**:
  - `src/plugins/contracts/registry.ts` (356 lines, SHA: `d866a2cb1e3677ceb22100806495be13ee831cba`)
  - `src/plugins/contracts/registry.contract.test.ts` (201 lines, SHA: `fb8b4614dac4a4d580a373efeff1575a91db49b1`)
  - `src/plugins/contracts/capability-catalog.contract.test.ts` (109 lines, SHA: `67ac28b12ca8a15ade41dbfea4c18488bc051f86`)
  - `src/plugins/contracts/providers.contract.test.ts` (31 lines, SHA: `51ec10ea2a99edec388a6f0887594d3e22c0efe2`)
  - `src/plugins/contracts/web-fetch-provider.contract.test.ts` (22 lines, SHA: `b9d637526ba86496e95bc7c7ea565d4632b99261`)
  - `src/plugins/contracts/registry.retry.test.ts` (82 lines, SHA: `cf3c45ec802b14969e8e3bf7218e8bfc0da05a3c`)
  - `src/plugins/contracts/shared-upstream-model.contract.test.ts` (141 lines, SHA: `9dc26ecc0f3241f30af9f6f65af997a29c05af0b`)
  - `src/plugins/contracts/provider-catalog-deprecation.contract.test.ts` (134 lines, SHA: `a3d72e37dae910ac9885cffacc519745ca12a312`)
  - `src/plugins/contracts/auth-choice.contract.test.ts` (81 lines, SHA: `e88e0234fe3772b07fedee0776bd400b93b490c2`)
- **Upstream Commit**: `ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`).
- **Module ID**: `OCRM-M28` (plugins).
- **Reviewer**: `xuyao@ombhrum.com`.
- **Mapped Responsibility**: `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001`.

## 2. In-Depth Source Behavior Decomposition

### 2.1 Inputs, Outputs & Semantics
- `resolveBundledManifestContracts()`:
  - Inputs: Bundled capability snapshots and plugin manifests (`loadBundledCapabilityRuntimeRegistry`).
  - Outputs: Array of `PluginRegistrationContractEntry` containing plugin ID, CLI backends, provider IDs, worker provider IDs, storage provider IDs, and normalized environment variable mappings.
- `normalizeProviderEnvVars(providerEnvVars)`:
  - Inputs: Record of provider IDs to environment variable string lists.
  - Outputs: Canonical lowercase normalized environment variable name dictionaries.
- `registeredPluginIds(key)`:
  - Inputs: Filter key for capability families (`webFetchProviderIds` or `webSearchProviderIds`).
  - Outputs: Sorted list of deduplicated plugin IDs offering the capability.
- `resolveWebSearchProviders(bundled, activationScoped)`:
  - Inputs: Bundled search providers and activation-scoped overrides (`codex`, `qa-lab`).
  - Outputs: Sorted unique union of web search providers.

### 2.2 State Mutations & Side Effects
- Upstream TypeScript maintains in-memory module-level arrays of contract entries (`pluginRegistrationContractRegistry`) populated during startup.
- In Fabushi native Rust, `PluginCapabilityCatalog` maintains concurrent, thread-safe access via `Arc<Mutex<HashMap>>` for capability provider entries, channel entries, canonical aliases, and onboarding auth choices.

### 2.3 Callers & Consumers
- Callers: Plugin onboarding wizard, CLI flag parsers (`--alibaba-model-studio-api-key`, `--github-copilot-token`), tool registration router, and provider execution bridge.
- Consumers: Channel gateways, LLM search/fetch executors, and assistant device authentication handlers.

### 2.4 Error Handling & Permissions
- Provider ID uniqueness: Duplicate provider IDs across bundled plugins or within a plugin trigger explicit validation errors.
- Onboarding Isolation: Video/image-only providers (`alibaba`, `runway`) are filtered out from text-chat onboarding, preventing invalid credential prompts.
- GitHub Copilot App-Guided Auth: Non-interactive token onboarding requires device-code flow (`appGuidedAuth: "device-code"`) with assistant priority 1 for public GitHub vs priority 2 for Enterprise tenants.

## 3. Producer -> Owner -> Storage -> Consumer Trace
1. `Producer`: Plugin manifest loader and contract registry scanner (`src/plugins/contracts/registry.ts`).
2. `Owner`: `source/host/src/extensions/plugins/capability_catalog.rs` (`PluginCapabilityCatalog`).
3. `Storage`: Thread-safe catalog maps for capabilities, channels, aliases, and auth choices.
4. `Consumer`: Onboarding wizard CLI, activation planner, and capability dispatch runtime.

## 4. Verification & Oracle Evidence
- **Contract Test Suite**: `source/host/tests/openclaw_plugins_contract.rs`
  - `test_ocrm_plugins_contract_registry_oracle` (PASSED)
    - Verifies canonical-first aliases (`azure-speech`, `volcengine`, `xai`).
    - Verifies duplicate provider ID rejection (`validate_unique_provider_ids`).
    - Verifies onboarding auth choice filtering (`filter_text_onboarding_auth_choices`).
    - Verifies web search provider resolution with activation-scoped overrides.
  - `test_ocrm_plugins_registry_public_artifacts_retry_precedence` (PASSED)
    - Verifies public artifacts precedence before falling back to bundled runtime registry for providers and web search.
  - `test_ocrm_plugins_shared_upstream_model_contract` (PASSED)
    - Verifies upstream model reference integrity and code-mode tier consistency across shared model sibling rows.
  - `test_ocrm_plugins_provider_catalog_deprecation_guard` (PASSED)
    - Verifies absence of deprecated `discovery:` hooks in provider plugins, enforcing static descriptor isolation.
  - `test_ocrm_plugins_auth_choice_contract` (PASSED)
    - Verifies method choice string building and resolution to preferred provider ID.
  - `test_ocrm_plugins_embedding_provider_registration` (PASSED)
    - Verifies public SDK helpers are read-only (`registerEmbeddingProvider`, `clearEmbeddingProviders` forbidden from public exposure).
    - Verifies plugins must declare `contracts.embeddingProviders` before registering adapter.
    - Verifies duplicate provider IDs and reserved core provider IDs (`openai-compatible`) are rejected.
  - `test_ocrm_plugins_wizard_choice_resolution_and_model_picker` (PASSED)
    - Verifies exact provider, method, and wizard resolution across colon, hyphen, and primary default formats.
    - Verifies wizard model picker entries exposed across shared wizard layer.
  - `test_ocrm_plugins_bundled_capability_metadata_and_subpath_bundle` (PASSED)
    - Verifies bundled plugin contract snapshot models across all 21 capability families and setup provider env vars.
  - `test_ocrm_plugins_bundled_channel_metadata_compat_and_dir_resolver` (PASSED)
    - Verifies bundled channel metadata validation (history limits, media limits, empty ID check).
    - Verifies bundled compat ID normalization (`legacy-telegram` -> `telegram`, etc.).
    - Verifies bundled directory relative path resolution and path traversal rejection.

## 5. Audited Bundled Capability Runtime, Channel Metadata, Compat & Dir Files (9 files)
- **Responsibility**: `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001`
- **Reviewer**: `xuyao@ombhrum.com`
- **Native Implementation**: `source/host/src/extensions/plugins/capability_catalog.rs` (`BundledChannelMetadataValidator`, `BundledCompatResolver`, `BundledDirResolver`)

| Path | Lines | Size (bytes) | Git Blob SHA |
| --- | --- | --- | --- |
| `src/plugins/bundled-capability-runtime.test.ts` | 185 | 7310 | `3e30f14376483c66f50c05877114b73e51240ce4` |
| `src/plugins/bundled-capability-runtime.ts` | 114 | 4518 | `369a19d2a6ff4685ff421a169b1778c1872146f4` |
| `src/plugins/bundled-channel-config-metadata.ts` | 148 | 6364 | `c2cb9848529329007f30cb7ef087ee46ca9f4a5f` |
| `src/plugins/bundled-channel-runtime.test.ts` | 68 | 2687 | `4a3170e5b8d2c2c0199d255152a5ca386b453a2b` |
| `src/plugins/bundled-channel-runtime.ts` | 74 | 2704 | `7a8848d61eafe4736fdf3e2b26002f23ef77efd5` |
| `src/plugins/bundled-compat.test.ts` | 64 | 2467 | `eb341d7cbdd9b25f5a89fb3f338d330eb9cf058d` |
| `src/plugins/bundled-compat.ts` | 69 | 2468 | `fc59c5d01217e94f061214e21a71966db99d2fef` |
| `src/plugins/bundled-dir.test.ts` | 468 | 21011 | `fef80f5509a25b3a62888c3a9ec2b10a247ec6ba` |
| `src/plugins/bundled-dir.ts` | 315 | 12267 | `41f7e0fc21da0c961e6c3be5b078e223ca87efd7` |





## 6. Audited Bundled Metadata, Scanning, Startup, Compat, Overlays & Naming Files (10 files)
- **Primary Responsibilities**: `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001`, `OCRM-RESP-PLUGINS-BUNDLE-MANIFEST-001`
- **Reviewer**: `xuyao@ombhrum.com`
- **Native Implementation**: `source/host/src/extensions/plugins/capability_catalog.rs` (`BundledPluginScanner`, `BundledProviderCompatValidator`, `BundledSourceOverlayResolver`, `BundledPluginNamingAndArtworkValidator`)
- **Contract Tests**: `source/host/tests/openclaw_plugins_contract.rs::test_ocrm_plugins_bundled_metadata_scan_compat_and_naming_artwork`

| Path | Lines | Size (bytes) | Git Blob SHA | Mapped Responsibility |
| --- | --- | --- | --- | --- |
| `src/plugins/bundled-plugin-categories.test.ts` | 65 | 3185 | `1876fce7cf9fae0a56525185c61a2a8d554b5ec9` | `OCRM-RESP-PLUGINS-BUNDLE-MANIFEST-001` |
| `src/plugins/bundled-plugin-icons.test.ts` | 46 | 2052 | `5204b88ea5f6f8b1c2cf81862ff16f6e8245a71a` | `OCRM-RESP-PLUGINS-BUNDLE-MANIFEST-001` |
| `src/plugins/bundled-plugin-metadata.public-surfaces.test.ts` | 181 | 6319 | `46d059bcee7bfd05bcf134527cf1c7fb305be524` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/bundled-plugin-metadata.test.ts` | 312 | 12251 | `24b98ca4a2652f9ca601bdae6728ea6fdab8f4fe` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/bundled-plugin-metadata.ts` | 155 | 5739 | `9c40d4733fcd99083b1cf6004513a212fa8e9b69` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/bundled-plugin-naming.test.ts` | 154 | 4985 | `fb47d17be4aeedb7b8d7d1967f097d8d6cd4863b` | `OCRM-RESP-PLUGINS-BUNDLE-MANIFEST-001` |
| `src/plugins/bundled-plugin-scan.ts` | 224 | 7402 | `3939d808f00dac1fb934ff24d8978b1a1e45a27e` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/bundled-plugin-startup-metadata.ts` | 18 | 762 | `c11dab1b7e4654bfb4c2f564a6faca1546918b20` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/bundled-provider-compat.ts` | 43 | 1410 | `3a8752d31f277134d16182083fab67736ee11645` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/bundled-source-overlays.ts` | 115 | 3678 | `94b6b8004b54340a71ed92bc1277f8b9622e0306` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |

### 6.1 Invariant Validation Summary
- **Category Slugs**: All bundled plugins require exactly 1 purpose category chosen strictly from `PLUGIN_CATEGORY_SLUGS`.
- **Artwork Guardrails**: Bundled plugin identity PNG icon must start with `[137, 80, 78, 71, 13, 10, 26, 10]` and have exactly 512x512 resolution.
- **Naming Conventions**: Directory names correspond directly to manifest ID (with explicit exception `kimi-coding` -> `kimi`), and package names must start with `@openclaw/` plus an allowed suffix (`-provider`, `-plugin`, `-speech`, etc.).
- **Fast Startup Resolution**: Plugin startup manifest can be checked via fast-path JSON probing without materializing full catalog metadata.
- **Overlay Security**: Mount points parsed from `/proc/self/mountinfo` are matched against legacy extension paths, with complete disable support via `OPENCLAW_DISABLE_BUNDLED_SOURCE_OVERLAYS`.

## 7. Audited Bundled Sources, Candidate Install Owner, Capability Catalog Context & Consent (10 files)
- **Primary Responsibilities**: `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001`, `OCRM-RESP-PLUGINS-BUNDLE-MANIFEST-001`, `OCRM-RESP-PLUGINS-INSTALL-MANAGEMENT-001`
- **Reviewer**: `xuyao@ombhrum.com`
- **Native Implementation**: `source/host/src/extensions/plugins/capability_catalog.rs` (`BundledPluginSourceResolver`, `CandidateInstallOwnerResolver`, `CapabilityCatalogValidator`, `CapabilityConsentManager`)
- **Contract Tests**: `source/host/tests/openclaw_plugins_contract.rs::test_ocrm_plugins_bundled_sources_catalog_and_capability_consent`

| Path | Lines | Size (bytes) | Git Blob SHA | Mapped Responsibility |
| --- | --- | --- | --- | --- |
| `src/plugins/bundled-sources.test.ts` | 168 | 5587 | `4c9a4fecfbd60fcb7308c21990029167d42ba3aa` | `OCRM-RESP-PLUGINS-BUNDLE-MANIFEST-001` |
| `src/plugins/bundled-sources.ts` | 182 | 6038 | `9b5c9cea33a0cff056cd640807ff0e1965d90a88` | `OCRM-RESP-PLUGINS-BUNDLE-MANIFEST-001` |
| `src/plugins/candidate-install-owner.ts` | 54 | 1914 | `fea3646375b475cb3d330f779b30f3c0f9e5e4a3` | `OCRM-RESP-PLUGINS-INSTALL-MANAGEMENT-001` |
| `src/plugins/capability-artifact.ts` | 156 | 6106 | `0e0f3d3648033ea724f2439afe2785b67fa5f891` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/capability-catalog-context.ts` | 55 | 2238 | `38abda596f14b92784a8f220400357bcd150c560` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/capability-catalog-context.types.ts` | 42 | 3441 | `0ce2cc58f7b45d31ae305b73e4a44ba9a51b61b0` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/capability-catalog.ts` | 84 | 3709 | `00a3c1bc8f8a151fd42d6317b38d65a471a7ddd4` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/capability-catalog.types.ts` | 12 | 495 | `8b1febb6db7b871ab16c28a47ba65292dd467fc8` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/capability-consent.test.ts` | 467 | 17156 | `9e3d50bc6be829173d2b495fa810f0ddd6e61bd0` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/capability-consent.ts` | 457 | 18642 | `98ba3e6afc5d51bb156bf9d39efce21fa03aae9d` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |

### 7.1 Invariant Validation Summary
- **Bundled Source Precedence & Dormancy**: `resolveBundledPluginSources` projects bundled sources keyed by plugin ID, preserving explicit path overrides. First registration wins, subsequent duplicates are discarded.
- **Config Requirements Detection**: Checks `configSchema.required` array for non-empty string entries to determine if plugin requires configuration before activation.
- **Immutable Install Identity**: Global NPM installs produce deterministic JSON array identity `[owner, installPath, spec, integrity, installedAt]`. Editable/path installs yield None, as replacement invalidates identity.
- **Capability Catalog Shape**: Exactly 3 valid families (`speechProviders`, `realtimeTranscriptionProviders`, `realtimeVoiceProviders`) mapped to required methods (`synthesize`, `createSession`, `createBridge`). All providers require non-empty ID, label, isConfigured check, and family method.
- **Capability Consent & Declared Surface Widening**: Computes canonical SHA-256 surface hash. Compares declared permissions across 10 dimensions (tools, channels, providers, contracts, hooks, mcpServers, cliCommands, cliBackends, skills, dangerousConfigFlags). Detects widening upon updates.
- **Integrity Selection Precedence**: SSRI `integrity` -> SSRI `npmIntegrity` -> SHA256 `clawpackSha256` -> Git commit `gitCommit`.
- **Credential Scrubbing**: Sanitizes credentials and sensitive query parameters (`token`, `key`, `secret`) from `git:https://...` specs before review presentation.

## 8. Audited Capability Lease, Provider Acquisition, Runtime Generation, Summary & Captured Registration (11 files)
- **Primary Responsibility**: `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001`
- **Reviewer**: `xuyao@ombhrum.com`
- **Native Implementation**: `source/host/src/extensions/plugins/capability_catalog.rs` (`PluginRuntimeCapabilityLease`, `CapabilitySummaryBuilder`, `CapturedPluginRegistration`, `PluginCapabilityAcquisitionScope`)
- **Contract Tests**: `source/host/tests/openclaw_plugins_contract.rs::test_ocrm_plugins_capability_lease_summary_and_acquisition`

| Path | Lines | Size (bytes) | Git Blob SHA | Mapped Responsibility |
| --- | --- | --- | --- | --- |
| `src/plugins/capability-lease.ts` | 38 | 913 | `786f0ec7074715e623a4b46c392cc55f0b7ef781` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/capability-provider-acquisition.test.ts` | 92 | 3448 | `49c0f4ee3f8355c44b157cdcfe356f1c3f082fdc` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/capability-provider-acquisition.ts` | 244 | 8517 | `93afbefca47dc0f90a3c5c5b326e5038f7c15fdd` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/capability-provider-runtime.generation.test.ts` | 641 | 24361 | `20409885191c2b70e11054f5243f47790f87be47` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/capability-provider-runtime.test.ts` | 747 | 27159 | `377a937472ec863af8d25819355465a2341327b0` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/capability-provider-runtime.ts` | 660 | 22485 | `4c773ec0ec6a0fb8188a3c2f5ed9c2afcd81ce0d` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/capability-provider.types.ts` | 559 | 20566 | `eba4c36640e3cd6584d79d258ba6712b104d1ffb` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/capability-summary.test.ts` | 138 | 5104 | `7debc9aeb2da89e063cf4a99d0e25b8b0f68476c` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/capability-summary.ts` | 313 | 10815 | `001257ccaee0124a1e7c5c9364a66fb1d172aa74` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/captured-registration.test.ts` | 359 | 13840 | `8976e5c191b38e6189a00333b10e87caf6be21b4` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/captured-registration.ts` | 287 | 10419 | `a2064948ff85bd9eab4ff7bce1253cb9895f8217` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |

### 8.1 Invariant Validation Summary
- **Capability Lease Lifecycle**: `PluginRuntimeCapabilityLease` guards active registrations with atomic lifecycle leases. Cleanups run on revocation. Retaining on an inactive lease triggers immediate cleanup execution and throws.
- **Surface Merge Semantics**: Multiple declared surfaces are merged across all 10 canonical categories, deduplicating elements and maintaining deterministic lexicographical sort order.
- **Default Operator Grants Policy**: Bundled plugins default to allowing conversation access (`effective = true`), whereas non-bundled plugins require explicit operator opt-in (`effective = false` by default). Prompt injection always defaults to disabled.
- **Capability Acquisition & Reload Fence**: `PluginCapabilityAcquisitionScope` tracks generation counters. If provider setup changes during request preparation, `assert_open` fails closed with a retry directive.
- **Complete Capability Capture**: `CapturedPluginRegistration` collects registrations across all 21 capability families and rejects unregistered families.

## 9. Audited Catalog Discovery, Search, Icon Registry & Channel Catalog Registry (9 files)
- **Primary Responsibility**: `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001`
- **Reviewer**: `xuyao@ombhrum.com`
- **Native Implementation**: `source/host/src/extensions/plugins/capability_catalog.rs` (`CatalogDiscoveryCodec`, `CatalogIconRegistry`, `CatalogSearchRanking`, `ChannelCatalogResolver`)
- **Contract Tests**: `source/host/tests/openclaw_plugins_contract.rs::test_ocrm_plugins_catalog_discovery_icon_ranking_and_channel_registry`

| Path | Lines | Size (bytes) | Git Blob SHA | Mapped Responsibility |
| --- | --- | --- | --- | --- |
| `src/plugins/catalog-discovery-runtime.ts` | 249 | 8452 | `40518ddab7c2b75cc13cd23f154732680baf7c22` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/catalog-discovery.test.ts` | 575 | 22442 | `1928c3092342182e0baaeb0db8f7deb4e41027d0` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/catalog-discovery.ts` | 445 | 16348 | `c295c5c1fcedcf54c807a44e01875cb2b28f0853` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/catalog-icon-registry.test.ts` | 42 | 1488 | `a6aa0a38d55ca910bdfa96ae83f53b9b466dfcf4` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/catalog-icon-registry.ts` | 34 | 1083 | `155e191a4094cae973bcabd6737602303ea0725e` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/catalog-search.test.ts` | 80 | 2776 | `45dff5df3386a3393ef98b05583e979c9352b362` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/catalog-search.ts` | 40 | 1292 | `f5016e2753dc549a1e651ef3288194fcc1fd1cfb` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/channel-catalog-registry.test.ts` | 288 | 10186 | `c8cf9def2801207dc08872723dec48487d04e25f` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/channel-catalog-registry.ts` | 128 | 4459 | `af8b005acfbde1e7af6c19a71d5215f1766555ec` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |

### 9.1 Invariant Validation Summary
- **Catalog Discovery Route Identities**: Base64URL encoding with `ch_` (ClawHub published package) and `local_` (local/bundled unindexed plugin) prefixes. Bidirectional encoding roundtrip verified. Non-base64url payload characters fail closed.
- **Catalog Icon Normalization Guardrails**: Validates HTTPS scheme, host presence, max 2048 chars, and forbids embedded credentials (`user:pass@`) or URL fragments (`#`).
- **Icon LRU Eviction**: FIFO/LRU eviction at capacity (1024 max), retaining only verifiable secure icon URLs.
- **Search Ranking & Deduplication**: Filter restricted to `code-plugin` and `bundle-plugin` families. Multi-group duplicate packages deduplicate in favor of higher score, with stable tie-breaking on score equality. Search limit clamped between 1 and 100 (default 20).
- **Channel Catalog Resolution Precedence**: Resolves channel plugins using fallback chain: `bundledManifest.id` -> `bundledManifestId` -> `packageManifest.plugin.id` -> `idHint`. Skips candidate entries missing channel declarations or empty channel IDs.

## 10. Audited Channel Policies, Validation, Registry Adoption & Scoped IDs (8 files)
- **Primary Responsibility**: `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001`
- **Reviewer**: `xuyao@ombhrum.com`
- **Native Implementation**: `source/host/src/extensions/plugins/capability_catalog.rs` (`ChannelPluginValidator`, `ChannelRegistryAdoption`, `ChannelStartupPolicy`, `ChannelPresencePolicy`)
- **Contract Tests**: `source/host/tests/openclaw_plugins_contract.rs::test_ocrm_plugins_channel_policy_validation_and_adoption`

| Path | Lines | Size (bytes) | Git Blob SHA | Mapped Responsibility |
| --- | --- | --- | --- | --- |
| `src/plugins/channel-catalog-registry.workspace.test.ts` | 162 | 7025 | `7fe18a1b1bbf3e9aa2bd43c9e866b4014d2cd188` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/channel-plugin-ids.test.ts` | 1847 | 59889 | `811b75ca0bba76b002d518354fe3ee2bb1a986de` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/channel-plugin-ids.ts` | 27 | 1058 | `520597843dee0d61989c61cb5c25cdb3acedb75e` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/channel-presence-policy.ts` | 696 | 24865 | `a2f4737d617b981a6d02ca8f810ddcdd9a10a1ac` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/channel-registry-adoption.ts` | 125 | 4878 | `c69791e8b1395b4c709f8e60d2a7afc8e69831f2` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/channel-registry-state.types.ts` | 19 | 866 | `e5b58f8424de02920ca6bdb5bf8f7733b11a4623` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/channel-startup-policy.ts` | 50 | 1814 | `2eaf2e8a65bc6ac13128b2a144591acbe1bff416` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/channel-validation.test.ts` | 111 | 3782 | `feba6bae38e3c93804160bc29ff538c17c5584ed` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |

### 10.1 Invariant Validation Summary
- **Channel Registration Validation & Diagnostics**: Enforces presence of required account configuration helpers (`list_account_ids` and `resolve_account`); registrations lacking them are rejected with error diagnostic.
- **Metadata Repair & Normalization**: Mismatched `meta.id` emits a warning and is aligned to the registered channel ID. Omitted fields (`label`, `selectionLabel`, `docsPath`, `blurb`) are backfilled with standard fallbacks and emit warning diagnostics. Official external channel aliases (e.g. `openclaw-weixin`) are populated with canonical aliases (`["weixin", "wechat", "微信"]`).
- **Channel Registry Adoption & Donor Verification**: During prepared registry admission, ensures donor registry is current and verified. Target and donor environment digests and gateway context resolvers must match. If lifecycle authority changed, fails closed with `"Channel runtime owner changed during prepared registry admission"`. Successfully adopted channels are wrapped with `borrowed_runtime_record`.
- **Channel Startup Policy & Restrictive Allowlist Exceptions**: Evaluates manifest owner policy against deny lists and allowlists. Bundled channel owners gain an explicit bypass to restrictive allowlists if corresponding channel configuration is explicitly provided in root configuration. Non-bundled channel plugins require explicit enablement in authored plugins configuration entries.
- **Channel Presence Signals & Announce Filtering**: Aggregates presence signals from explicit configuration, environment variables, manifest contracts, and persisted auth. Explicitly disabled channels (`enabled: false`) are suppressed case-insensitively and excluded from announce lists. Ambient-only presence signals are suppressed when requested.

## 11. Audited ClawHub Package Distribution, Archive Verification & Security Assessment (11 files)
- **Primary Responsibility**: `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001`
- **Reviewer**: `xuyao@ombhrum.com`
- **Native Implementation**: `source/host/src/extensions/plugins/clawhub.rs` (`ClawHubInstallErrorCode`, `ClawHubInstallFailure`, `ClawHubArchiveVerification`, `ClawHubPluginInstallRecordFields`, `ClawHubSecurityAssessment`, `ClawHubInstallPipeline`)
- **Contract Tests**: `source/host/tests/openclaw_plugins_contract.rs::test_ocrm_plugins_clawhub_package_archive_verification_and_install_policy`

| Path | Lines | Size (bytes) | Git Blob SHA | Mapped Responsibility |
| --- | --- | --- | --- | --- |
| `src/plugins/claw-managed-capability-consent.integration.test.ts` | 329 | 12487 | `778cc9bfaa740d05f463155f5d67874f8913100d` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/clawhub-archive-verification.ts` | 152 | 6590 | `51c03f0dd9e123d751f83ca3cc438d12813c2d67` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/clawhub-error-codes.ts` | 59 | 2103 | `e3e2e97193b8ac5f457cd56375be34df62bfc46b` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/clawhub-install-records.ts` | 144 | 5901 | `456b0d79758585478c75121e4aef87f079b0d04e` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/clawhub-official-source.ts` | 14 | 459 | `431bb5746302f084ff6e257f144a91c8fc996628` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/clawhub-presentation.ts` | 65 | 2432 | `c07fef4a9e7c0123487dbf67cc687974dd83fbe1` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/clawhub.cleanup.test.ts` | 77 | 3115 | `206a146f068a60edb72e8e2d1dd6e03684fbc555` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/clawhub.test-fixtures.ts` | 38 | 974 | `8f180a29ef4f92cbaacc78c0a1d9b5c9a62c2161` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/clawhub.test-support.ts` | 176 | 5730 | `b302a5a5d2832fc10de6fb4eff8fc51f9b7bda23` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/clawhub.test.ts` | 1012 | 40316 | `ddd4d8e6a346b121c4a991d458a7cd4a93102278` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |
| `src/plugins/clawhub.ts` | 911 | 35524 | `286f795de513c1116a43fbb0d804f0b3a04ec62f` | `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001` |

### 11.1 Invariant Validation Summary
- **ClawHub Stable Error Taxonomy**: Standard error enum covering 17 failure modes (`invalid_spec`, `package_not_found`, `version_not_found`, `no_installable_version`, `skill_package`, `unsupported_family`, `private_package`, `incompatible_plugin_api`, `invalid_gateway_version`, `unknown_gateway_version`, `incompatible_gateway`, `artifact_unavailable`, `missing_archive_integrity`, `artifact_download_unavailable`, `archive_integrity_mismatch`, `clawhub_security_unavailable`, `clawhub_download_blocked`, `plugin_id_mismatch`). Unavailable targets (`package_not_found` and `version_not_found`) are distinguished for selector widening.
- **Official Package Attribution**: Distinguishes default official registry packages (`https://clawhub.ai` with `channel == "official"` or `isOfficial == true`) from third-party registries.
- **Archive Verification & Metadata Guard**: Extracted ZIP archives validate presence and integrity of all declared files. Generated `_meta.json` is validated for JSON validity and exact match of slug and version against target package. Missing files, SHA256 mismatches, or unexpected extra files trigger `archive_integrity_mismatch` with lexicographically first unexpected file reported.
- **Family & Security Assessment Guardrails**: Enforces family constraints (`code-plugin` and `bundle-plugin` only; skills reject with install guidance; unsupported families fail closed). Malicious or blocked security verdicts halt install immediately with `clawhub_download_blocked`. Risky verdicts emit operator review recommendations. Temporary archive removal failures log warnings without invalidating successful installation.





