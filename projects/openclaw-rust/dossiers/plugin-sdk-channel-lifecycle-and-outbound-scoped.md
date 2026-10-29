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

## 6. Audit Status
- Mapped Files: 12 added (Total mapped: 805 / 58,750).
- Unclassified Remaining: 57,945.
- Global completion flags: all 8 remaining `false` (fail-closed).
