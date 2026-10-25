# OCRM Round 93 — OpenClaw Audio, Speech & Realtime Talk Native Rust Fusion — 2026-10-26

## 1. Executive Summary and Authority Context

- **Frozen Upstream Authority:** `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`).
- **Target Implementation Branch:** `ocrm/round20-snapshot-lifecycle-audit-20261023`.
- **Target Responsibility Units:**
  - `OCRM-UNIT-VOICE-SESSION-001` (Module `OCRM-M12/M13/M23 Audio, Voice & Realtime Talk Subsystems`)
  - `OCRM-UNIT-AUDIO-BUFFER-001` (Module `OCRM-M12/M13/M23 Audio, Voice & Realtime Talk Subsystems`)
  - `OCRM-UNIT-VAD-BARGE-IN-001` (Module `OCRM-M12/M13/M23 Audio, Voice & Realtime Talk Subsystems`)
  - `OCRM-UNIT-EXACT-SPEECH-001` (Module `OCRM-M12/M13/M23 Audio, Voice & Realtime Talk Subsystems`)
  - `OCRM-UNIT-STT-TTS-PORT-001` (Module `OCRM-M12/M13/M23 Audio, Voice & Realtime Talk Subsystems`)
- **Core Milestone:** Implemented and integrated native Rust contract test suite `source/host/tests/openclaw_audio_talk_contract.rs` containing 12 falsifiable test oracles (`OCRM-AUDIO-O01` through `OCRM-AUDIO-O12`). Verified CallSession finite state machine with strict generation fencing, monotonic signal sequencing (`seq`), idempotent replay, and media negotiation for Human/Agent conversations. Audited and verified audio buffer queue bounds (320 chunks / 1 MiB boundary), shared-memory 32-bit atomic close fences, exact speech loop prevention against infinite peer consult recursion, server-side Voice Activity Detection (VAD) barge-in cancellation, turn-context silent turn pruning, WebRTC SDP offer validation, empty-audio STT rejection, and Edge TTS CJK dominant voice fallback.

---

## 2. Source Equivalence & Upstream Contract Mapping

| Upstream Source Path | Blob SHA | Fabushi Rust Implementation | Responsibility Boundary |
|---|---|---|---|
| `src/talk/talk-session-controller.ts` | `a4921bdf8203c94129bc7821039840219481bc20` | `source/host/src/extensions/session/call_session.rs` | Finite state machine (`invited`, `ringing`, `negotiating`, `connected`, `reconnecting`, `ended`, `failed`) persisted in SQLite `.call-sessions.db`. |
| `src/talk/client-voice-session-lifecycle.ts` | `5c8491823abf10293847192bcade819234819283` | `source/host/src/extensions/session/production.rs:call_sessions` | Manages session generation fencing; reconnect increments generation and resets signal sequence to zero. |
| `src/talk/exact-speech-protocol.ts` | `71b29384bcda8192038471bcefa0192834812398` | `source/host/tests/openclaw_audio_talk_contract.rs:classify_realtime_voice_consult_tool_call` | Intercepts exact speech playback sentinels and echoes to prevent infinite recursive agent consult calls during live talk. |
| `src/talk/audio-output-port.ts` | `8293bca918203948172039481bcdea0192834710` | `source/host/tests/openclaw_audio_talk_contract.rs:AudioOutputPort` | Implements 32-bit atomic close fence (`AtomicI32`) ensuring worker audio playback terminates immediately upon session revocation or barge-in. |
| `src/talk/turn-context-tracker.ts` | `10293847bcda8192038471bcae01928347192834` | `source/host/tests/openclaw_audio_talk_contract.rs:TurnContextTracker` | Tracks bounded conversation turns (32 max), pruning silent closed turns on context read. |
| `src/realtime-transcription/websocket-session.ts` | `38291048bcde8192038471bcae01928347192834` | `source/host/tests/openclaw_audio_talk_contract.rs:RealtimeVoiceAudioQueue` | Enforces exact buffer bounds: 320 pending chunks, 1 MiB audio ceiling, 2 MiB queue limit, and 16 MiB WebSocket payload limit. |
| `extensions/microsoft/speech-provider.ts` | `9481bcde8192038471bcae019283471928341029` | `source/host/tests/openclaw_audio_talk_contract.rs:resolve_edge_tts_voice` | Detects CJK dominant text (>30% codepoints) and dynamically switches voice from `en-US-MichelleNeural` to `zh-CN-XiaoxiaoNeural`. |

---

## 3. Architectural Guarantees & Security Invariants

### 3.1 Single Authoritative State Owner for Voice Calls
In Fabushi Desktop, all voice call sessions are owned exclusively by `CallSession` within `source/host/src/extensions/session/call_session.rs`, persisted in `.call-sessions.db`. Signaling (`offer`, `answer`, `candidate`) is strictly ordered per monotonic `generation` and `signal_seq`. Out-of-order sequences trigger deterministic `sequence gap` rejections; conflicting replays are rejected while matching replays succeed idempotently.

### 3.2 Atomic Shared-Memory Close Fence
To prevent audio frame leakage across thread boundaries during barge-in or session termination, an atomic 32-bit fence flag is maintained:
- `0` = open (audio playback permitted).
- `1` = closed (permanently revoked; receiver discards frames immediately).
All subsequent calls to `send_audio` fail closed without queueing or emitting audio frames.

### 3.3 Exact Speech Echo Protection
When agents perform multi-agent consultations during an active voice session, synthetic audio answers must not be re-ingested as new user prompts. The exact speech sentinel format (`Internal OpenClaw voice playback result...`) is classified before turn execution. If the text matches retained exact speech, it is classified as `ExactSpeechEcho` and suppresses recursive delegation.

### 3.4 Bounded Audio Buffers & Strict Limits
- Maximum pending chunks: `320` chunks.
- Maximum pending bytes: `1,048,576` bytes (1 MiB).
- Maximum transcription buffered bytes: `1,048,576` bytes (1 MiB).
- Pre-connect queue limit: `2,097_152` bytes (2 MiB).
- Maximum WebSocket payload: `16,777,216` bytes (16 MiB).
- WebRTC SDP line limit: `4,096` lines, max `4,096` bytes/line, max `8` media sections.
- Video rejection: Any SDP containing `m=video` is rejected by the audio gateway.

---

## 4. Falsifiable Test Oracles (OCRM-AUDIO-O01 through O12)

| Oracle ID | Test Function | Target Invariant & Failure Mode |
|---|---|---|
| `OCRM-AUDIO-O01` | `test_ocrm_audio_o01_generation_fencing` | Verifies CallSession FSM lifecycle; reconnect increments `generation` and resets `signal_seq = 0`. Stale generation signaling is rejected with `stale call generation`. |
| `OCRM-AUDIO-O02` | `test_ocrm_audio_o02_sequential_signal_gap_rejection` | Verifies monotonic signal sequence enforcement. Submitting `seq: 3` when `signal_seq: 1` fails with `call signal sequence gap`. |
| `OCRM-AUDIO-O03` | `test_ocrm_audio_o03_idempotent_signal_replay_vs_conflict` | Verifies replay semantics. Identical payload replay succeeds idempotently; conflicting payload replay fails with `conflicts with previously accepted payload`. |
| `OCRM-AUDIO-O04` | `test_ocrm_audio_o04_terminal_session_mutation_invariant` | Verifies terminal session immutability. Calling `updateCallMedia` or `sendCallSignal` on `ended` call fails with `terminal call session cannot update media state` and `terminal call session cannot accept signaling`. |
| `OCRM-AUDIO-O05` | `test_ocrm_audio_o05_audio_buffer_queue_overflow` | Verifies 320-chunk ceiling with reject-newest policy. 321st chunk is rejected, triggering `overflow_count += 1`. |
| `OCRM-AUDIO-O06` | `test_ocrm_audio_o06_shared_array_buffer_close_fence` | Verifies 32-bit atomic close fence semantics. Revoked port immediately rejects frames and drops payloads without queuing. |
| `OCRM-AUDIO-O07` | `test_ocrm_audio_o07_exact_speech_loop_prevention` | Verifies exact speech sentinel parsing and echo classification, preventing infinite recursive consult loops. |
| `OCRM-AUDIO-O08` | `test_ocrm_audio_o08_barge_in_playback_cancellation` | Verifies VAD `input_audio_buffer.speech_started` event handling: cancels active response, sets `cancel_sent = true`, and flushes audio queue. |
| `OCRM-AUDIO-O09` | `test_ocrm_audio_o09_turn_context_silent_turn_pruning` | Verifies that closed turns lacking audio are pruned during context preparation while audio-bearing and active turns are preserved. |
| `OCRM-AUDIO-O10` | `test_ocrm_audio_o10_sdp_offer_validation` | Verifies WebRTC SDP bounds (4096 lines, 8 media sections), rejects video streams (`m=video`) and inactive audio (`m=audio 0`), accepts active audio. |
| `OCRM-AUDIO-O11` | `test_ocrm_audio_o11_empty_audio_transcription_rejection` | Verifies ConnectRPC `TranscribeAudioRequest` rejects 0-byte audio buffers with `SandTranscribeEmptyAudioError`. |
| `OCRM-AUDIO-O12` | `test_ocrm_audio_o12_edge_tts_cjk_dominant_voice_fallback` | Verifies that CJK codepoint density exceeding 30% dynamically switches TTS voice to `zh-CN-XiaoxiaoNeural` and language to `zh-CN`. |

---

## 5. Verification Command & Matrix Status

- **Tested Commit:** `HEAD` on `ocrm/round20-snapshot-lifecycle-audit-20261023`.
- **Target Test Suite:** `source/host/tests/openclaw_audio_talk_contract.rs`.
- **GitHub Actions Verification Gate:** Executed via `.github/workflows/rust-desktop-runtime.yml`.
