# Plugin Contract Registry & Bundled Capability Metadata Dossier

## 1. Scope & Source Attribution
- **Target Files**:
  - `src/plugins/contracts/registry.ts` (356 lines, SHA: `d866a2cb1e3677ceb22100806495be13ee831cba`)
  - `src/plugins/contracts/registry.contract.test.ts` (201 lines, SHA: `fb8b4614dac4a4d580a373efeff1575a91db49b1`)
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
