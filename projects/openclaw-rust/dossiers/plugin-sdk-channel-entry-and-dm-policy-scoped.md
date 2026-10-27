# Plugin SDK Channel Entry Contract & DM Setup Policy Dossier

## 1. Scope & Source Attribution
- **Target Files (10 files in `src/plugin-sdk/`)**:
  - `src/plugin-sdk/channel-contract-testing.ts` (25 lines)
  - `src/plugin-sdk/channel-contract.ts` (46 lines)
  - `src/plugin-sdk/channel-core.ts` (27 lines)
  - `src/plugin-sdk/channel-dm-policy.test.ts` (76 lines)
  - `src/plugin-sdk/channel-dm-policy.ts` (120 lines)
  - `src/plugin-sdk/channel-entry-contract.lifecycle.test.ts` (95 lines)
  - `src/plugin-sdk/channel-entry-contract.test.ts` (486 lines)
  - `src/plugin-sdk/channel-entry-contract.ts` (613 lines)
  - `src/plugin-sdk/channel-entry-contract.types.ts` (33 lines)
  - `src/plugin-sdk/channel-entry.test-support.ts` (38 lines)
- **Upstream Commit**: `ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`).
- **Module ID**: `OCRM-M28` (plugin-sdk).
- **Reviewer**: `xuyao@ombhrum.com`.
- **Mapped Responsibility**: `OCRM-RESP-PLUGIN-SDK-CHANNEL-ENTRY-001`.

## 2. In-Depth Source Behavior Decomposition

### 2.1 Bundled Channel Entry Contract & Lifecycle
- **Registration Modes**: Segregates entrypoint plugin registration into four distinct modes:
  - `Full`: Full runtime capabilities and tools.
  - `CliMetadata`: Fast CLI help/inspection without heavy module imports.
  - `Capabilities`: Surface capability declarations for planner and catalog discovery.
  - `SetupOnly`: Minimal setup-wizard surface.
- **Lazy Module Loaders**: Provides lazy loading boundaries (`loadChannelPlugin`, `loadChannelOutbound`, `loadChannelSecrets`, `loadChannelAccountInspector`) to prevent eager activation overhead.

### 2.2 Account-Aware DM Policy Setup Descriptor
- **Descriptor Mechanics**: Encapsulates `ChannelDmPolicyDescriptor` with root (`channels.<channel>.dmPolicy`) and account-scoped (`channels.<channel>.accounts.<id>.dmPolicy`) keys.
- **Wildcard Open Injection**: Automatically adds `*` to `allowFrom` when the policy changes to `"open"`.
- **Custom Policy Paths**: Supports nested legacy configurations (such as Matrix `dm.policy` and `dm.allowFrom`).

### 2.3 Outbound Session Routing & Normalization Helpers
- **Route Synthesis**: `build_channel_outbound_session_route` generates canonical session keys (with optional thread IDs) and strips redundant channel prefixes (`channel:<target>` -> `<target>`).
- **Target Prefix Stripping**: Normalizes target handles by removing channel-specific and kind-specific prefixes (`user:`, `channel:`).
- **Thread Recovery**: Recovers thread IDs from formatted session keys (`:thread:<id>`).

### 2.4 Testkit & Delivery Trace Harness
- **Wire Recording**: `WireRecorder` captures chronological delivery events (inbound/outbound) for verifying protocol fidelity against golden fixtures.

## 3. Rust Equivalents Implemented
- [`source/host/src/extensions/plugin_sdk/channel_dm_policy.rs`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/src/extensions/plugin_sdk/channel_dm_policy.rs)
- [`source/host/src/extensions/plugin_sdk/channel_entry.rs`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/src/extensions/plugin_sdk/channel_entry.rs)

## 4. Contract Test Verification
- [`source/host/tests/openclaw_plugin_sdk_contract.rs`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/tests/openclaw_plugin_sdk_contract.rs):
  - `test_ocrm_plugin_sdk_channel_dm_policy_and_entry_contract`

## 5. Audit Status
- Mapped Files: 10 added (Total mapped: 785 / 58,750).
- Unclassified Remaining: 57,965.
- Global completion flags: all 8 remaining `false` (fail-closed).
