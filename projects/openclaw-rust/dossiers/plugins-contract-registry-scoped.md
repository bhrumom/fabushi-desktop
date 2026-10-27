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



