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
