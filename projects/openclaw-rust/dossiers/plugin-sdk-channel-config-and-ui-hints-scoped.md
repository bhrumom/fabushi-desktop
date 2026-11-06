# Plugin SDK Channel Configuration & UI Schema Hints Dossier

## 1. Scope & Source Attribution
- **Target Files (11 files in `src/plugin-sdk/`)**:
  - `src/plugin-sdk/bundled-channel-config-schema.ts` (96 lines)
  - `src/plugin-sdk/channel-access-compat.ts` (4 lines)
  - `src/plugin-sdk/channel-actions.ts` (30 lines)
  - `src/plugin-sdk/channel-activity-runtime.ts` (3 lines)
  - `src/plugin-sdk/channel-config-helpers.test.ts` (356 lines)
  - `src/plugin-sdk/channel-config-helpers.ts` (443 lines)
  - `src/plugin-sdk/channel-config-primitives.ts` (24 lines)
  - `src/plugin-sdk/channel-config-schema.ts` (56 lines)
  - `src/plugin-sdk/channel-config-ui-hints.test.ts` (73 lines)
  - `src/plugin-sdk/channel-config-ui-hints.ts` (255 lines)
  - `src/plugin-sdk/channel-config-writes.ts` (1 lines)
- **Upstream Commit**: `ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`).
- **Module ID**: `OCRM-M28` (plugin-sdk).
- **Reviewer**: `xuyao@ombhrum.com`.
- **Mapped Responsibility**: `OCRM-RESP-PLUGIN-SDK-CHANNEL-CONFIG-001`.

## 2. In-Depth Source Behavior Decomposition

### 2.1 Channel Config Schemas & Schema Primitives
- **Schema Builders**: Generates Zod/TypeBox definitions for multi-account channels, catch-all channel schemas, nested DM config schemas, group schemas, markdown configuration, and tool policies.
- **Bundled vs External Schemas**: Encapsulates bundled provider schemas (iMessage, Telegram, Google Chat, WhatsApp) behind facade loaders while deprecating legacy external channel package exports (Slack, Discord, Signal, Teams).

### 2.2 UI Configuration Hints Generator
- **DM Policy Hints**: Emits label and help text for direct message access controls (`dmPolicy` and legacy `dm.policy`).
- **Mention Patterns**: Controls pattern policies, modes (`allow` vs `deny`), allowlists, and denylists for group chats.
- **Implicit Mentions**: Configures signals for replies to bot, quoted bot messages, and thread participation.
- **Progress Drafts**: Controls label pools, max compact lines, max characters per line, tool progress visibility, command text exposure (`status` vs `raw`), and commentary.
- **Streaming & Retry Groups**: Maps block streaming, native transport, chunk modes, and exponential retry parameters with jitter.

### 2.3 Account-Scoped Config Writes & Permissions
- **Config Write Permission Check**: `resolveChannelConfigWrites` checks whether config mutations from channel events are enabled at the channel level, allowing per-account overrides.
- **DM Access Resolution**: `resolveChannelDmAccess` resolves allowlists and policies with precedence for account-specific settings over root channel defaults.
- **Canonical Migration**: `set_canonical_dm_allow_from` relocates legacy `dm.allowFrom` entries to top-level `allowFrom` while emitting immutable change logs.
- **Open DM Policy Wildcard Guard**: When a channel's DM policy is `open`, enforces that `*` is present in `allowFrom`.

## 3. Rust Equivalents Implemented
- [`source/host/src/extensions/plugin_sdk/channel_config.rs`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/src/extensions/plugin_sdk/channel_config.rs)

## 4. Contract Test Verification
- [`source/host/tests/openclaw_plugin_sdk_contract.rs`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/tests/openclaw_plugin_sdk_contract.rs):
  - `test_ocrm_plugin_sdk_channel_config_and_ui_hints`

## 5. Audit Status
- Mapped Files: 11 added (Total mapped: 775 / 58,750).
- Unclassified Remaining: 57,975.
- Global completion flags: all 8 remaining `false` (fail-closed).
