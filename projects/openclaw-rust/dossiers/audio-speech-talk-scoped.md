# OCRM Technical Dossier — Unit 8: Voice, Speech & Realtime Talk (`OCRM-UNIT-TALK-001`)

**Project ID**: OCRM-001  
**Module**: OCRM-M12/M13/M23 (Audio, Voice & Realtime Talk Subsystems)  
**Upstream Target Authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `ocrm/round20-snapshot-lifecycle-audit-20261023`  
**Reviewer**: `xuyao@ombhrum.com`  

---

## 1. Upstream Source Audit & Line-by-Line Reading

The following 6 core talk and voice source files (totaling 1,380 lines) were line-by-line audited and mapped into canonical shards:

| Shard | Source File | Blob SHA | Lines | Scope | Mapped Responsibility |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `part-0047` | `src/talk/talk-session-controller.ts` | `ddd39516d5697de19af870ee654b3f807e5864ac` | 222 | 1..222 (full-file) | `OCRM-RESP-TALK-SESSION-CONTROLLER-001` |
| `part-0047` | `src/talk/client-voice-session-lifecycle.ts` | `9b31de718e10bd5fb748a30126889bea85533c91` | 208 | 1..208 (full-file) | `OCRM-RESP-TALK-CLIENT-LIFECYCLE-001` |
| `part-0047` | `src/talk/exact-speech-protocol.ts` | `de7fe86c4709e4dd09ace653f43b0b5802126b18` | 96 | 1..96 (full-file) | `OCRM-RESP-TALK-EXACT-SPEECH-001` |
| `part-0047` | `src/talk/audio-output-port.ts` | `d708e9b70db100bf16e24d1ea6f397cadf8ec3ee` | 98 | 1..98 (full-file) | `OCRM-RESP-TALK-AUDIO-PORT-FENCE-001` |
| `part-0047` | `src/talk/turn-context-tracker.ts` | `f31643dca1189f96264a02bd1c26b802ef4105b8` | 192 | 1..192 (full-file) | `OCRM-RESP-TALK-TURN-CONTEXT-001` |
| `part-0045` | `src/realtime-transcription/websocket-session.ts` | `01934ce2d4c638f646e7d2b05fd18c964a1712ef` | 564 | 1..564 (full-file) | `OCRM-RESP-TALK-WS-TRANSCRIPTION-001` |

---

## 2. Decomposed Runtime Responsibilities

The audited source code was decomposed into 6 discrete Rust runtime responsibilities registered in `parity-ledger.json`:

1. **`OCRM-RESP-TALK-SESSION-CONTROLLER-001`**:
   - **Behavior**: Voice call session finite state machine (`invited`, `ringing`, `connected`, `ended`, `failed`), generation fencing, and immutable terminal states.
   - **Rust Owner**: `source/host/src/extensions/session/call_session.rs:CallSession`.
   - **Oracles**: `OCRM-AUDIO-O01`, `OCRM-AUDIO-O04`.

2. **`OCRM-RESP-TALK-CLIENT-LIFECYCLE-001`**:
   - **Behavior**: Monotonic signaling sequence (`signal_seq`), out-of-order gap detection, and idempotent signal replay vs conflict detection.
   - **Rust Owner**: `source/host/src/extensions/session/production.rs:call_sessions`.
   - **Oracles**: `OCRM-AUDIO-O02`, `OCRM-AUDIO-O03`.

3. **`OCRM-RESP-TALK-EXACT-SPEECH-001`**:
   - **Behavior**: Exact speech protocol sentinel parsing and echo suppression, preventing infinite recursive agent consultation loops.
   - **Rust Owner**: `source/host/src/runner/mod.rs:classify_realtime_voice_consult_tool_call`.
   - **Oracle**: `OCRM-AUDIO-O07`.

4. **`OCRM-RESP-TALK-AUDIO-PORT-FENCE-001`**:
   - **Behavior**: Realtime PCM audio queue bounds (320 chunks / 1 MiB) and 32-bit atomic close fence (`AtomicI32`) ensuring worker audio terminates cleanly upon revocation.
   - **Rust Owner**: `source/host/src/ports/mod.rs:AudioOutputPort`.
   - **Oracles**: `OCRM-AUDIO-O05`, `OCRM-AUDIO-O06`.

5. **`OCRM-RESP-TALK-TURN-CONTEXT-001`**:
   - **Behavior**: Bounded turn context tracking (32 turns max), VAD barge-in response cancellation, and silent closed turn pruning.
   - **Rust Owner**: `source/host/src/runner/mod.rs:TurnContextTracker`.
   - **Oracles**: `OCRM-AUDIO-O08`, `OCRM-AUDIO-O09`.

6. **`OCRM-RESP-TALK-WS-TRANSCRIPTION-001`**:
   - **Behavior**: Realtime transcription WebSocket buffer queue, WebRTC SDP offer validation, 0-byte empty audio rejection, and CJK dominant voice fallback.
   - **Rust Owner**: `source/host/src/runner/mod.rs:RealtimeVoiceAudioQueue`.
   - **Oracles**: `OCRM-AUDIO-O10`, `OCRM-AUDIO-O11`, `OCRM-AUDIO-O12`.

---

## 3. Falsifiable Test Oracles (`source/host/tests/openclaw_audio_talk_contract.rs`)

1. **`OCRM-AUDIO-O01`**: `test_ocrm_audio_o01_generation_fencing`
2. **`OCRM-AUDIO-O02`**: `test_ocrm_audio_o02_sequential_signal_gap_rejection`
3. **`OCRM-AUDIO-O03`**: `test_ocrm_audio_o03_idempotent_signal_replay_vs_conflict`
4. **`OCRM-AUDIO-O04`**: `test_ocrm_audio_o04_terminal_session_mutation_invariant`
5. **`OCRM-AUDIO-O05`**: `test_ocrm_audio_o05_audio_buffer_queue_overflow`
6. **`OCRM-AUDIO-O06`**: `test_ocrm_audio_o06_shared_array_buffer_close_fence`
7. **`OCRM-AUDIO-O07`**: `test_ocrm_audio_o07_exact_speech_loop_prevention`
8. **`OCRM-AUDIO-O08`**: `test_ocrm_audio_o08_barge_in_playback_cancellation`
9. **`OCRM-AUDIO-O09`**: `test_ocrm_audio_o09_turn_context_silent_turn_pruning`
10. **`OCRM-AUDIO-O10`**: `test_ocrm_audio_o10_sdp_offer_validation`
11. **`OCRM-AUDIO-O11`**: `test_ocrm_audio_o11_empty_audio_transcription_rejection`
12. **`OCRM-AUDIO-O12`**: `test_ocrm_audio_o12_edge_tts_cjk_dominant_voice_fallback`

---

## 4. Conservation & Invariant Metrics

- `tracked_entries_total`: **58,750**
- `mapped_paths`: **106** (100 prior + 6 Unit 8)
- `unread`: **58,644**
- `unclassified`: **58,644**
- `registered_responsibilities`: **48**
- All 58 inventory shards checksums verified in `manifest.json`.
- All global completion flags strictly maintained as **false**.
