# Plugin SDK Allow-From, API Baseline & Workspace Runtime Dossier

## 1. Scope & Source Attribution
- **Target Files (11 files in `src/plugin-sdk/`)**:
  - `src/plugin-sdk/agent-workspace-runtime.ts` (17 lines)
  - `src/plugin-sdk/allow-from.test.ts` (94 lines)
  - `src/plugin-sdk/allow-from.ts` (144 lines)
  - `src/plugin-sdk/allowlist-config-edit.test.ts` (191 lines)
  - `src/plugin-sdk/allowlist-config-edit.ts` (447 lines)
  - `src/plugin-sdk/api-baseline-declaration-closure.ts` (628 lines)
  - `src/plugin-sdk/api-baseline-declaration-print.ts` (92 lines)
  - `src/plugin-sdk/api-baseline-normalization.ts` (33 lines)
  - `src/plugin-sdk/api-baseline.test.ts` (461 lines)
  - `src/plugin-sdk/api-baseline.ts` (479 lines)
  - `src/plugin-sdk/api-diff.ts` (484 lines)
- **Upstream Commit**: `ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`).
- **Module ID**: `OCRM-M28` (plugin-sdk).
- **Reviewer**: `xuyao@ombhrum.com`.
- **Mapped Responsibility**: `OCRM-RESP-PLUGIN-SDK-ALLOW-FROM-API-BASELINE-001`.

## 2. In-Depth Source Behavior Decomposition

### 2.1 Allow-From Normalization & Sender Matching
- **Entry Parsing & Trimming**:
  - Delimiters: newline, comma, semicolon (`[\n,;]+`).
  - Wildcard `*`: allows any sender explicitly.
  - Fail-closed: Empty allowlists deny by default.
  - Prefix stripping: Case-insensitive prefix stripping (e.g., `tg:`, `zalo:`) followed by lowercase normalization.
- **Account-Scoped Config Edits**:
  - Channel config paths: `channels.<channelId>.allowFrom` (unscoped/default) vs `channels.<channelId>.accounts.<accountId>.allowFrom` (scoped).
  - Legacy cleanup paths: `channels.<channelId>.dm.allowFrom` read and cleaned up on modern writes.
  - Flat and nested override collectors: `collect_allowlist_overrides_from_value` preserves labeled group overrides for UI/management.

### 2.2 Plugin SDK API Baseline & Diffing Engine
- **Declaration Normalization**:
  - Strips repo-root machine-specific paths from compiler output.
  - Preserves node_modules-relative canonical specifiers.
- **Export Evolution & Diff Analysis**:
  - Tracks public export kinds: `function`, `type`, `class`, `const`, `interface`.
  - Breaking changes: Kind mutation (e.g. function -> const) and export removal.
  - Change reporting: Formats markdown reports with `### Breaking Changes`, `### Added Exports`, `### Removed Exports`, `### Changed Exports`.

### 2.3 Agent Workspace Runtime
- **Access Policies**:
  - Modes: `ReadOnly`, `ReadWrite`.
  - Hidden file protection: Blocks dot-files (`.git`, `.env`) unless explicitly permitted.
  - Attachment limits: Enforces max attachment count and cumulative byte size limits.

## 3. Native Rust Implementation & Equivalence
- **Implementation Modules**:
  - [`source/host/src/extensions/plugin_sdk/allow_from.rs`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/src/extensions/plugin_sdk/allow_from.rs).
  - [`source/host/src/extensions/plugin_sdk/api_baseline.rs`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/src/extensions/plugin_sdk/api_baseline.rs).
  - [`source/host/src/extensions/plugin_sdk/agent_workspace_runtime.rs`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/src/extensions/plugin_sdk/agent_workspace_runtime.rs).
- **Test Oracles**:
  - `source/host/tests/openclaw_plugin_sdk_contract.rs`:
    - `test_ocrm_plugin_sdk_allow_from_and_config_edit`
    - `test_ocrm_plugin_sdk_api_baseline_and_diff`
    - `test_ocrm_plugin_sdk_agent_workspace_runtime`
