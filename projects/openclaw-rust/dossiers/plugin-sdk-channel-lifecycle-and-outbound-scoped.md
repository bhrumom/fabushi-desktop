# Plugin SDK Channel Lifecycle, Outbound Pipeline & Pairing Dossier

## 1. Scope & Source Attribution
- **Target Files (12 files in `src/plugin-sdk/`)**:
  - `src/plugin-sdk/channel-lifecycle.core.ts` (185 lines)
  - `src/plugin-sdk/channel-lifecycle.queue.test.ts` (146 lines)
  - `src/plugin-sdk/channel-lifecycle.test.ts` (206 lines)
  - `src/plugin-sdk/channel-mention-gating.ts` (25 lines)
  - `src/plugin-sdk/channel-message.test.ts` (94 lines)
  - `src/plugin-sdk/channel-outbound.reply-pipeline.test.ts` (53 lines)
  - `src/plugin-sdk/channel-outbound.stale-install.test.ts` (82 lines)
  - `src/plugin-sdk/channel-outbound.streaming.test.ts` (29 lines)
  - `src/plugin-sdk/channel-outbound.ts` (308 lines)
  - `src/plugin-sdk/channel-pairing.store.test.ts` (128 lines)
  - `src/plugin-sdk/channel-pairing.test.ts` (132 lines)
  - `src/plugin-sdk/channel-pairing.ts` (70 lines)
- **Upstream Commit**: `ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`).
- **Module ID**: `OCRM-M28` (plugin-sdk).
- **Reviewer**: `xuyao@ombhrum.com`.
- **Mapped Responsibility**: `OCRM-RESP-PLUGIN-SDK-CHANNEL-LIFECYCLE-OUTBOUND-001`.

## 2. In-Depth Source Behavior Decomposition

### 2.1 Channel Run Queue & Stall Watchdog
- **ChannelRunQueue**: Concurrency-controlled asynchronous task queue per channel identifier. Ensures sequential delivery and backpressure isolation across distinct channel instances.
- **ArmableStallWatchdog**: Watchdog timer for detecting unresponsive channel connections, heartbeat dropouts, and stalling network sockets with configurable timeout and trigger callbacks.

### 2.2 Channel Mention Gating
- **Mention Gating Invariants**: Evaluates whether incoming messages require explicit @-mentions to wake up the agent or if ambient listening is enabled. Respects direct message overrides, group allowlists, and broadcast exemptions.

### 2.3 Outbound Pipeline, Streaming & Echo Tracking
- **OutboundEchoTracker**: Tracks sent outbound message signatures to deduplicate loopback echoes from message brokers and chat platforms (e.g. self-sent webhook echoes).
- **ChannelDraftStreamingChunking**: Splits streaming output into bounded chunks based on channel formatting limits (e.g. Telegram 4096-char limit, Discord 2000-char limit) while preserving markdown entity boundaries and code blocks.
- **Payload Media Selection**: Preserves audio/video/image attachments and correctly resolves fallback reply text when rich embeds are unsupported.

### 2.4 Channel Device Pairing Store & Code Generation
- **ChannelPairingStore**: In-memory and persistent pairing request storage supporting pending, approved, and rejected states. Generates cryptographically secure alphanumeric pairing codes with expiration time-to-live.
- **Pairing Command Parsing**: Sanitizes and strips pairing prefixes (`/pair`, `pair:`) to extract verification tokens across multi-channel environments.

## 3. Rust Equivalents Implemented
- [`source/host/src/extensions/plugin_sdk/channel_lifecycle.rs`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/src/extensions/plugin_sdk/channel_lifecycle.rs)
- [`source/host/src/extensions/plugin_sdk/channel_outbound.rs`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/src/extensions/plugin_sdk/channel_outbound.rs)
- [`source/host/src/extensions/plugin_sdk/channel_pairing.rs`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/src/extensions/plugin_sdk/channel_pairing.rs)

## 4. Contract Test Verification
- [`source/host/tests/openclaw_plugin_sdk_contract.rs`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/tests/openclaw_plugin_sdk_contract.rs):
  - `test_ocrm_plugin_sdk_channel_lifecycle_outbound_pairing`

## 5. Pairing randomness boundary (2026-10-29)

`ChannelPairingStore::issue_challenge` now fails explicitly when the operating
system secure-random source is unavailable. It no longer emits the previous
predictable `123456` fallback, and the contract caller must handle the
`Result` before exposing a challenge. This is a local fail-closed security
correction; the upstream scoped pairing-store/controller and real channel
provider wiring remain separate pending responsibilities.

The channel run queue also now re-reads its lifecycle state at execution time.
If deactivation wins a dequeue race, the removed task is discarded instead of
running with the stale enqueue-time `is_active=true` context. This closes a
local lifecycle fence; it does not establish the upstream async queue's status
sink, abort signal, or real channel-provider consumer.

## 6. Audit Status
- Mapped Files: 12 added (Total mapped: 805 / 58,750).
- Unclassified Remaining: 57,945.
- Global completion flags: all 8 remaining `false` (fail-closed).

## 7. Outbound echo identity correction (2026-10-29)

The locked `src/channels/message/outbound-echo.ts` source (`7a4439d7618b8441ca284d38ba48c1dfb6928040`) defines a 30-second echo window, scopes identities by normalized channel/account/conversation, accepts either a platform `messageId` or a `sourceId`, and bounds retained identities to 10,000 entries. The previous Rust helper used an unbounded process-local set keyed only by channel/account/message ID.

The existing Rust plugin-SDK owner now uses the same scoped identity dimensions, independently expires entries after the 30-second window, supports both identity kinds, and evicts the oldest entries at the 10,000-entry bound. Its contract fixture covers source-ID matching, conversation isolation, and expiry. This is implementation evidence only: no channel provider registration, inbound consumer, or production delivery wiring is inferred, and the responsibility remains `wiring=not-wired`, `verification=not-run`, `acceptance=pending` until the exact-head Actions receipt and real channel path are obtained.

## 8. Draft-streaming resolver correction (2026-10-29)

The locked `src/channels/draft-streaming-chunking.ts` source (`ca99866dc4c2b40b7d735f3799e8fee80e24fb5b`) defines defaults of `minChars=200` and `maxChars=800`. It selects the complete account-level preview-chunk configuration when present, otherwise the channel-level configuration; clamps the requested maximum to the already-resolved text limit; clamps the minimum to the resulting maximum; and accepts only `paragraph`, `newline`, or `sentence` break preferences.

The Rust owner now exposes `ChannelDraftStreamingChunking::resolve` with an explicit override input and caller-supplied text limit, preserving the existing ownership boundary for channel configuration and text-limit lookup. The contract test covers account-over-channel precedence, text-limit clamping, default resolution, and the accepted break preference. This remains a local resolver contract: no provider registration, streaming consumer, or production channel wiring is inferred; `wiring=not-wired`, `verification=not-run`, and `acceptance=pending` remain unchanged until exact-head Actions and real channel-path evidence exist.
