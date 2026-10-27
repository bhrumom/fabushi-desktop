# System Agent, Talk Voice Session, Transcripts Capture & State Persistence Dossier

## 1. Scope and Responsibilities
- **Scope**: Native Rust host runtime extensions and contract tests for system agent lifecycle, voice interaction controllers, transcript capture store, and monotonic state persistence engine.
- **Target Extensions**:
  - `source/host/src/extensions/system_agent/system_agent_runner.rs`
  - `source/host/src/extensions/talk/talk_controller.rs`
  - `source/host/src/extensions/transcripts/capture_summary.rs`
  - `source/host/src/extensions/state/state_persistence_engine.rs`
- **Upstream Commit**: `ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`).
- **Mapped Responsibilities**:
  1. `OCRM-RESP-SYSTEM-AGENT-RUNNER-001`: System Agent Onboarding Proposals, Diagnostic Audit Runner, and Overview Command Parser.
  2. `OCRM-RESP-TALK-VOICE-SESSION-001`: Talk Voice Session Phase Controller, Talkback Mode State Machine, and Audio Energy RMS Speech Detection.
  3. `OCRM-RESP-TRANSCRIPTS-CAPTURE-SUMMARY-001`: Transcript Capture Store, Query Limit Budgeting, and Markdown Conversation Summary Generator.
  4. `OCRM-RESP-STATE-PERSISTENCE-ENGINE-001`: Monotonic Versioning State Persistence Engine, Partition KV Store, and Sequential Schema Migrations.

## 2. Producer -> Owner -> Storage -> Consumer Traces
1. **System Agent Lifecycle Pipeline**:
   - `Producer`: User onboarding CLI commands and workspace initialization requests.
   - `Owner`: `source/host/src/extensions/system_agent/system_agent_runner.rs` (`SystemAgentRunner`, `SystemAgentOperationKind`).
   - `Storage`: Workspace system agent configuration and onboarding proposal.
   - `Consumer`: System agent setup wizard and diagnostic reporting endpoints.
2. **Talk Voice Session Pipeline**:
   - `Producer`: Inbound audio PCM capture buffers from microphone stream.
   - `Owner`: `source/host/src/extensions/talk/talk_controller.rs` (`TalkSessionController`, `VoiceSessionPhase`, `compute_audio_rms`).
   - `Storage`: Talk session phase state and talkback audio mode flags.
   - `Consumer`: Real-time voice interaction router and TTS playback pipeline.
3. **Transcripts Capture & Summary Pipeline**:
   - `Producer`: Host conversation events, assistant streaming tokens, and execution trajectory steps.
   - `Owner`: `source/host/src/extensions/transcripts/capture_summary.rs` (`TranscriptCaptureStore`, `TranscriptEntryRecord`).
   - `Storage`: Monotonically incremented transcript entries in in-memory capture store.
   - `Consumer`: Transcript summary generators, search endpoints, and conversation export.
4. **State Persistence & Monotonic Versioning Pipeline**:
   - `Producer`: Host extension state writes and partition data mutations.
   - `Owner`: `source/host/src/extensions/state/state_persistence_engine.rs` (`StatePersistenceEngine`, `PartitionStoreEntry`).
   - `Storage`: Partitioned key-value cell store with sequential migration version tracking.
   - `Consumer`: Database migration runners, crash recovery loaders, and host state synchronization.

## 3. Verification & Oracle Evidence
- **Contract Tests**: `source/host/tests/openclaw_system_talk_transcripts_contract.rs`:
  - `test_ocrm_system_agent_runner` (PASSED)
  - `test_ocrm_talk_voice_session` (PASSED)
  - `test_ocrm_tts_directives_and_auto_apply_contract` (PASSED)
  - `test_ocrm_transcripts_and_state` (PASSED)
