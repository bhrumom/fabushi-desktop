# Plugin Package Manifest Resolution & Contract Validation Dossier

## 1. Scope & Source Attribution
- **Target Files**:
  - `src/plugins/package-manifest.ts` (68 lines, SHA: `7bfe7039a65c52040c22013333087a0494b9542e`)
  - `src/plugins/contracts/package-manifest.contract.test.ts` (195 lines, SHA: `c39502b811e4f1db3c56f9abd6402f039b393476`)
  - `src/plugins/contracts/plugin-testing-harness.contract.test.ts` (73 lines, SHA: `19eba480e72768750aea34cb62b426a42ea50d81`)
  - `src/plugins/contracts/plugin-tool-contracts.test.ts` (164 lines, SHA: `e286cf4545c3caaf9525acfc5d143f6b741a0d93`)
- **Upstream Commit**: `ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`).
- **Module ID**: `OCRM-M28` (plugins).
- **Reviewer**: `xuyao@ombhrum.com`.
- **Mapped Responsibility**: `OCRM-RESP-PLUGINS-BUNDLE-MANIFEST-001`.

## 2. In-Depth Source Behavior Decomposition

### 2.1 Inputs, Outputs & Semantics
- `resolvePackageExtensionEntries(manifest)`:
  - Inputs: Parsed `package.json` object.
  - Return Status:
    - `"missing"`: `openclaw` object or `openclaw.extensions` is undefined/null.
    - `"invalid"`: `openclaw` is not an object, `extensions` is not an array, or an extension element is empty/whitespace.
    - `"empty"`: `extensions` array is empty.
    - `"ok"`: returns string array of non-empty extension relative paths.
- `validatePackageExtensionEntriesForInstall(openclaw)`:
  - Inputs: `OpenClawMetadata` struct with `extensions`, `runtime_extensions`, `setup_entry`, `runtime_setup_entry`.
  - Rules:
    - Runtime extensions length must match extensions length if specified.
    - `runtimeSetupEntry` requires `setupEntry` to be present.

### 2.2 Error Handling & Boundaries
- Non-object openclaw error: `"package.json openclaw must be an object"`.
- Non-array extensions error: `"package.json openclaw.extensions must be an array"`.
- Blank extension error: `"package.json openclaw.extensions[<index>] must be a non-empty string"`.
- Mismatched runtime extensions error: `"package.json openclaw.runtimeExtensions length (<M>) must match openclaw.extensions length (<N>)"`.
- Orphan runtime setup entry error: `"package.json openclaw.runtimeSetupEntry requires openclaw.setupEntry"`.

## 3. Producer -> Owner -> Storage -> Consumer Trace
1. `Producer`: Plugin package discovery (`package.json` reader, npm/ClawHub installer).
2. `Owner`: `source/host/src/extensions/plugins/bundle_manifest.rs` (`resolve_package_extension_entries`, `validate_package_extension_entries_for_install`).
3. `Storage`: In-memory plugin bundle registration entries.
4. `Consumer`: Plugin activation planner and runtime loader.

## 4. Verification & Oracle Evidence
- **Contract Tests**: `source/host/tests/openclaw_plugins_contract.rs`:
  - `test_ocrm_plugins_bundle_manifest` (PASSED)
    - Verifies `status: "ok"` on valid extensions.
    - Verifies `"package.json openclaw must be an object"` rejection.
    - Verifies `"package.json openclaw.extensions must be an array"` rejection.
    - Verifies non-empty string validation per element.
    - Verifies runtime extensions length match requirement.
    - Verifies runtime setup requires source setup entry.
    - Verifies `validate_plugin_manifest_tool_contracts` enforces that all registered tools are declared in `contracts.tools`.
    - Verifies virtual test plugin tool fixture execution with configured prefix and environment state.
  - `test_ocrm_plugins_bundled_capability_metadata_and_subpath_bundle` (PASSED)
    - Verifies bundled plugin roots resolution across relative subpaths.
    - Verifies never-bundle dependency boundaries (`@lancedb/lancedb`, `@matrix-org/matrix-sdk-crypto-nodejs`, `matrix-js-sdk`).
    - Verifies elimination of bare matrix-js-sdk import crashes in bundled subpath entries.
  - `test_ocrm_plugins_bundle_manifest_guardrails` (PASSED)
    - Verifies base config schema footprint excludes retired paths (`talk.*`, `channels.*.streamMode`, `agents.defaults.sandbox.perSession`).
    - Verifies canonical private-network schema property (`network.dangerouslyAllowPrivateNetwork`).
    - Verifies cross-package import boundaries forbid private internal plugin-sdk subpaths.
    - Verifies extension runtime dependencies whitelist optional peers (`@a2ui/markdown-it`, `@discordjs/opus`, `crabline`, `zod`, `audio-decode`, `apache-arrow`).
    - Verifies runtime plugin entrypoint files contain no test harness imports (`vitest`, `openclaw/plugin-sdk/testing`, `test-harness`, `test-api`).
  - `test_ocrm_plugins_sdk_package_and_runtime_api_guardrails` (PASSED)
    - Verifies public SDK package exports and subpath structure.
    - Verifies private local-only entrypoint boundaries (`browser-maintenance`, `channel-secret-owner-runtime`, `channel-secret-tts-runtime`).
    - Verifies runtime API exports across bundled extensions contain no internal test harnesses.
  - `test_ocrm_plugins_bundled_mcp_lsp_capabilities_and_command_scopes` (PASSED)
    - Verifies bundled MCP server config validation (name syntax, path separators, traversal escape prevention).
    - Verifies bundled LSP server definition validation (language ID, command).
    - Verifies manifest capability inspection (`mcpServers`, `lspServers`, `tools`, `skills`, `channels`).
    - Verifies bundle command scope authorization (`operator.admin` root bypass and exact scope matching).

## 5. Audited Bundle Manifest, Commands, MCP & LSP Files (10 files)
- **Responsibility**: `OCRM-RESP-PLUGINS-BUNDLE-MANIFEST-001`
- **Reviewer**: `xuyao@ombhrum.com`
- **Native Implementation**: `source/host/src/extensions/plugins/bundle_manifest.rs` (`BundledMcpServerValidator`, `BundledLspServerValidator`, `BundleCapabilitySupport`, `BundleCommandScopeValidator`)

| Path | Lines | Size (bytes) | Git Blob SHA |
| --- | --- | --- | --- |
| `src/plugins/bundle-capability-support.ts` | 24 | 785 | `2f9d54e4df5e236cbca8ebec886d5fffa8d10332` |
| `src/plugins/bundle-commands.test.ts` | 185 | 7321 | `30a618335dbf57e8fa134d1bc9ea11c1dfbf251b` |
| `src/plugins/bundle-config-shared.ts` | 129 | 4863 | `611590409a80e454b525893a72cb3eb8e034e34e` |
| `src/plugins/bundle-lsp.ts` | 89 | 3111 | `aa4490fa96d4ce03f8373b8893d96924b10da7fc` |
| `src/plugins/bundle-manifest.test.ts` | 338 | 11902 | `0032aa34ca8a946271a2fc3818e69dbf5c1d6ea5` |
| `src/plugins/bundle-manifest.ts` | 425 | 15570 | `fba0e300959fbeaa7f603cff9d714e3e3b97b1ca` |
| `src/plugins/bundle-mcp.test-support.ts` | 74 | 2602 | `83ef59d1872df0d9fc8f6df74681648a0ddcb99f` |
| `src/plugins/bundle-mcp.test.ts` | 426 | 17248 | `71b4bb6d0c2ee146ba8bbec0707c72f7c222ffda` |
| `src/plugins/bundle-mcp.ts` | 468 | 19157 | `8ab6f481a539bc2eb6368d5045b35d03c61b2d07` |
| `src/plugins/bundle-mcp.types.ts` | 21 | 543 | `4ae60bf318eb8ea072aa8ef4c5c703b0c0f99ae9` |




