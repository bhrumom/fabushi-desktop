# Plugin SDK Channel Feedback, Inbound Debouncing & Ingress Coordination Dossier

## 1. Scope & Source Attribution
- **Target Files (8 files in `src/plugin-sdk/`)**:
  - `src/plugin-sdk/channel-feedback.ts` (71 lines)
  - `src/plugin-sdk/channel-inbound-debounce.ts` (5 lines)
  - `src/plugin-sdk/channel-inbound.test.ts` (300 lines)
  - `src/plugin-sdk/channel-inbound.ts` (335 lines)
  - `src/plugin-sdk/channel-ingress-runtime.test.ts` (90 lines)
  - `src/plugin-sdk/channel-ingress-runtime.ts` (61 lines)
  - `src/plugin-sdk/channel-ingress-test-runtime.ts` (7 lines)
  - `src/plugin-sdk/channel-join-intro-runtime.ts` (8 lines)
- **Upstream Commit**: `ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`).
- **Module ID**: `OCRM-M28` (plugin-sdk).
- **Reviewer**: `xuyao@ombhrum.com`.
- **Mapped Responsibility**: `OCRM-RESP-PLUGIN-SDK-CHANNEL-INBOUND-INGRESS-001`.

## 2. In-Depth Source Behavior Decomposition

### 2.1 WhatsApp ACK Reactions & Tool Emojis
- **WhatsApp ACK Decision Gate**: Verifies direct vs group mode (`always`, `never`, `mentions`), checking mention status and group activation flags.
- **Contextual Tool Status Emojis**: Maps tool classification tokens (`BUILD_TOOL_TOKENS`, `CODING_TOOL_TOKENS`, `CONCIERGE_TOOL_TOKENS`, `DEPLOY_TOOL_TOKENS`, `WEB_TOOL_TOKENS`) to distinct UI emojis (`💻`, `🔨`, `🚀`, `🔍`, `🌐`, `⚡`).

### 2.2 Ingress Lifecycle Fan-in & Coordination
- **Multi-Source Claims**: `fan_in_channel_ingress_lifecycles` coordinates multiple durable message ingress claims into a single execution lifecycle.
- **Cascade Error Release**: If an adoption throws midway through a chain of lifecycles, all remaining unadopted claims are cleanly notified via `on_failed` with the active error.
- **Heartbeat Interval Resolution**: Selects the minimum positive finite interval from all contributing lifecycles.
- **Claim Finalization**: Exposes unified `settle`, `abandon`, and `cancel` semantics with once-only idempotency guards.

### 2.3 Identifier Authentication Levels
- **Hierarchy Comparison**: Strict ordinal comparison across `Mutable < Asserted < Authenticated < Verified < Signed`.

### 2.4 Inbound Debouncing, Envelopes & Room Joins
- **Inbound Debouncer**: Prevents duplicate processing by checking elapsed duration since the last message from a given sender key.
- **Envelope Serialization**: Standardizes format `[{channel}] {from} ({timestamp_ms}): {body}`.
- **Room Join Outcome**: Emits join confirmation text including inviter metadata.

## 3. Rust Equivalents Implemented
- [`source/host/src/extensions/plugin_sdk/channel_feedback.rs`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/src/extensions/plugin_sdk/channel_feedback.rs)
- [`source/host/src/extensions/plugin_sdk/channel_ingress.rs`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/src/extensions/plugin_sdk/channel_ingress.rs)
- [`source/host/src/extensions/plugin_sdk/channel_inbound.rs`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/src/extensions/plugin_sdk/channel_inbound.rs)

## 4. Contract Test Verification
- [`source/host/tests/openclaw_plugin_sdk_contract.rs`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/tests/openclaw_plugin_sdk_contract.rs):
  - `test_ocrm_plugin_sdk_channel_feedback_inbound_ingress`

## 5. Audit Status
- Mapped Files: 8 added (Total mapped: 793 / 58,750).
- Unclassified Remaining: 57,957.
- Global completion flags: all 8 remaining `false` (fail-closed).
