# Plugin Compatibility Registry & Deprecation Framework Dossier

## 1. Scope & Source Attribution
- **Target Files (35 files in `src/plugins/compat/`)**:
  - `src/plugins/compat/activation-hint-records.ts` (54 lines)
  - `src/plugins/compat/agent-harness-records.ts` (84 lines)
  - `src/plugins/compat/agent-list-runtime-projection-records.ts` (17 lines)
  - `src/plugins/compat/assistant-text-phase-terminal-hint-record.ts` (14 lines)
  - `src/plugins/compat/async-sqlite-preparation-records.ts` (134 lines)
  - `src/plugins/compat/auth-async-record.ts` (70 lines)
  - `src/plugins/compat/binding-persistence-records.ts` (69 lines)
  - `src/plugins/compat/channel-pairing-record.ts` (17 lines)
  - `src/plugins/compat/channel-prepared-tool-policy-record.ts` (18 lines)
  - `src/plugins/compat/conversation-read-tools.ts` (42 lines)
  - `src/plugins/compat/cron-store-path-record.ts` (15 lines)
  - `src/plugins/compat/deprecation-marking.ts` (372 lines)
  - `src/plugins/compat/media-legacy-projection.ts` (30 lines)
  - `src/plugins/compat/memory-session-records.ts` (52 lines)
  - `src/plugins/compat/mention-inbox-deprecation.test.ts` (50 lines)
  - `src/plugins/compat/mention-inbox-deprecation.ts` (14 lines)
  - `src/plugins/compat/model-account-connect-deprecation.test.ts` (49 lines)
  - `src/plugins/compat/model-account-connect-deprecation.ts` (13 lines)
  - `src/plugins/compat/native-exec-approval-record.ts` (15 lines)
  - `src/plugins/compat/plugin-sdk-private-runtime-export-record.ts` (51 lines)
  - `src/plugins/compat/plugin-sdk-removed-export-records.ts` (166 lines)
  - `src/plugins/compat/plugin-sdk-subpath-records.ts` (228 lines)
  - `src/plugins/compat/progress-receipt-handoff-record.ts` (19 lines)
  - `src/plugins/compat/registry-records.ts` (653 lines)
  - `src/plugins/compat/registry.test.ts` (39 lines)
  - `src/plugins/compat/registry.ts` (10 lines)
  - `src/plugins/compat/reply-tool-authority-record.ts` (25 lines)
  - `src/plugins/compat/session-persistence-records.ts` (439 lines)
  - `src/plugins/compat/session-transcript-preparation-deprecation.ts` (14 lines)
  - `src/plugins/compat/session-transcript-view-record.ts` (15 lines)
  - `src/plugins/compat/sqlite-runtime-records.ts` (34 lines)
  - `src/plugins/compat/transcript-stats-record.ts` (21 lines)
  - `src/plugins/compat/types.ts` (29 lines)
  - `src/plugins/compat/web-search-provider-list-record.ts` (15 lines)
  - `src/plugins/compat/workspace-mutation-guard.ts` (18 lines)
- **Upstream Commit**: `ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`).
- **Module ID**: `OCRM-M28` (plugins).
- **Reviewer**: `xuyao@ombhrum.com`.
- **Mapped Responsibility**: `OCRM-RESP-PLUGINS-COMPAT-001`.

## 2. In-Depth Source Behavior Decomposition

### 2.1 Compatibility Records & Registry
- Defines canonical compatibility metadata across 146 unique codes (`PluginCompatRecord`).
- Fields:
  - `code`: Unique identifier (e.g., `plugin-sdk-tool-plugin-public-demotion`, `activation-agent-harness-hint`).
  - `status`: `"active" | "deprecated" | "removal-pending" | "removed"`.
  - `owner`: `"agent-runtime" | "channel" | "config" | "core" | "plugin-execution" | "provider" | "sdk" | "setup"`.
  - Dates: `introduced`, `deprecated`, `warningStarts`, `removeAfter` (formatted as `YYYY-MM-DD`).
  - Deprecation constraints:
    - If `status === "deprecated"`, requires valid `deprecated` and `warningStarts` date strings.
    - If `removalGate === "stable-deprecation-period"`, `removeAfter` is omitted.
    - If `code` is in `removalDatePendingCompatCodes` (`plugin-sdk-tool-plugin-public-demotion`, `agent-harness-sdk-alias`, `plugin-sdk-shipped-channel-setup-exports`), `removeAfter` is omitted.
    - Otherwise, `removeAfter` is required with a valid `YYYY-MM-DD` date.
  - Actionable diagnostics & surfaces: Non-empty string arrays for `surfaces` and `diagnostics`.

### 2.2 Host Restricted Conversation Read Tools
- Upstream policy restricts `feishu:feishu_chat` as host-only conversation read tool.
- Name normalization: Lowercased and trimmed.
- Registration gating: Examines actually produced tool names (`names`), falling back to `declaredNames` only when `names` is empty.
- Bundled verification: Validates that both registration and manifest entries have `origin === "bundled"`.

### 2.3 Deprecation Warning Framework
- Per-(pluginId, family, method) single-warning budget across plugin reload cycles.
- Plugin ID validation: Validates candidate string against regex `^(?:@[a-z0-9._-]+\/)?[a-z0-9][a-z0-9._-]*$`, length <= 128.
- Warning format:
  - Code: `DEP_SESSION_PERSISTENCE`.
  - Prefix: `[{pluginId}] ` if pluginId valid.
  - Removal info:
    - For SQLite removal families (55 specific families): `"Final compatibility day: 2026-12-31"`.
    - Otherwise: `"Removal requires an announced ordinary stable release window"`.

## 3. Native Rust Implementation & Equivalence
- **Implementation Module**: [`source/host/src/extensions/plugins/compat.rs`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/src/extensions/plugins/compat.rs).
- **Embedded Records**: [`source/host/src/extensions/plugins/plugin_compat_records.json`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/src/extensions/plugins/plugin_compat_records.json).
- **Test Oracle**: `source/host/tests/openclaw_plugins_contract.rs` (`test_plugin_compat_registry_validation_and_records`, `test_conversation_read_tools_host_restriction`, `test_plugin_sdk_deprecation_tracker`).
