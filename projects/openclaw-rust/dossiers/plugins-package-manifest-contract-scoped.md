# Plugin Package Manifest Resolution & Contract Validation Dossier

## 1. Scope & Source Attribution
- **Target Files**:
  - `src/plugins/package-manifest.ts` (68 lines, SHA: `7bfe7039a65c52040c22013333087a0494b9542e`)
  - `src/plugins/contracts/package-manifest.contract.test.ts` (195 lines, SHA: `c39502b811e4f1db3c56f9abd6402f039b393476`)
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
