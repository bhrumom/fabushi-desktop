# OpenClaw Meeting Bot Parity Dossier

## 1. Overview & Responsibility Mapping
- **Module ID**: `OCRM-RESP-MEETING-BOT-AUDIO-001`, `OCRM-RESP-MEETING-BOT-BROWSER-001`, `OCRM-RESP-MEETING-BOT-LIFECYCLE-001`
- **Scope**: 105 upstream files in `src/meeting-bot/` (100% of directory).
- **Owner**: `mahayana_host_runtime::extensions::meeting_bot` (`audio_bridge.rs`, `browser_controller.rs`, `session_lifecycle.rs`).

## 2. Audited Files & Upstream Anchors
- **Audio & Media Bridge** (25 files):
  - `src/meeting-bot/audio-backend.ts`, `src/meeting-bot/audio-backend.test-support.ts`
  - `src/meeting-bot/audio-base64.ts`, `src/meeting-bot/audio-base64.test.ts`
  - `src/meeting-bot/audio-bridge-source.ts`, `src/meeting-bot/bridge-process.ts`
  - `src/meeting-bot/node-audio-config.ts`
  - `src/meeting-bot/sox-audio-command.ts`, `src/meeting-bot/voice-call-gateway.ts`
  - `src/meeting-bot/output-loopback-verifier.ts`, `src/meeting-bot/output-loopback-verifier.test.ts`
  - `src/meeting-bot/transcripts-bridge.ts`, `src/meeting-bot/transcripts-bridge.provenance.test.ts`
  - And all related node audio configurations and loopback verifiers.
- **Browser & Chrome Node Automation** (28 files):
  - `src/meeting-bot/browser-act-lock.ts`, `src/meeting-bot/browser-act-lock.test.ts`
  - `src/meeting-bot/browser-controller.ts`, `src/meeting-bot/browser-controller.test.ts`
  - `src/meeting-bot/browser-navigation-errors.ts`, `src/meeting-bot/browser-node.ts`
  - `src/meeting-bot/browser-plugin-cli.ts`, `src/meeting-bot/browser-adapter-options.ts`
  - `src/meeting-bot/chrome-node-result.ts`, `src/meeting-bot/page-script-source.ts`
  - `src/meeting-bot/platform-adapter-contract.ts`
  - And all related browser actions and Chrome node runner files.
- **Realtime Meeting Lifecycle, Participation, & Consult** (52 files):
  - `src/meeting-bot/agent-consult.ts`, `src/meeting-bot/agent-consult.test.ts`
  - `src/meeting-bot/meeting-modes.ts`
  - `src/meeting-bot/participation-browser.ts`, `src/meeting-bot/participation-browser.test.ts`
  - `src/meeting-bot/realtime-agent-engine.ts`
  - `src/meeting-bot/session-cleanup-tracker.ts`
  - `src/meeting-bot/status-call-ownership-source.ts`
  - And all related meeting session runner and observation provenance suites.

## 3. Rust Native Equivalence Architecture
1. **Audio & Media Bridge** (`audio_bridge.rs`):
   - Strict base64 canonical validation with deterministic error messaging matching upstream.
   - 16-bit PCM chunk duration and sample rate conversions (`16kHz`, `24kHz`, `48kHz`).
2. **Browser Controller & Tab Action Lock** (`browser_controller.rs`):
   - Process-serialized per-target action queue preventing tab evaluate interleaving.
   - Enforced deadline timeout yielding `Meeting browser operation timed out waiting for browser tab control.`
3. **Meeting Session Lifecycle & Cleanup** (`session_lifecycle.rs`):
   - Talkback mode gate (`agent` or `bidi`).
   - Comprehensive realtime route readiness check (`in_call`, unmuted mic, input/output routed, no manual action).
   - Atomic cleanup tracker tracking `browser_settled`, `stop_settled`, and `unpublished`.

## 4. Contract Tests & Verification
- Test harness: `source/host/tests/openclaw_meeting_bot_contract.rs`
  - `test_ocrm_meeting_bot_audio_bridge`
  - `test_ocrm_meeting_bot_browser_controller`
  - `test_ocrm_meeting_bot_session_lifecycle`
- Evidence: 3/3 passing contract tests with full positive and negative assertions.
