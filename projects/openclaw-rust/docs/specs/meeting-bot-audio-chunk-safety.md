# Meeting bot audio chunk safety — specification

Status: active
Owner: existing Host meeting-bot audio owner
Last updated: 2026-11-01
Related project: `projects/openclaw-rust`
Related responsibility: `OCRM-RESP-MEETING-BOT-AUDIO-001`

## 1. Context / problem

`MeetingAudioChunk::duration_ms` derives PCM duration from a public Rust
constructor. The constructor currently accepts malformed channel metadata,
including `channels == 0`; duration calculation must not panic on untrusted or
partially decoded audio metadata.

## 2. Goal

Make duration calculation fail closed for zero-channel chunks while preserving
the existing valid 16-bit PCM calculation and the existing meeting-bot owner.

## 3. Non-goals

- No meeting-bot scheduler, browser transport, realtime audio producer, or
  notification consumer is added.
- No change to base64 decoding, sample-rate enum values, storage, or wire
  formats.
- This narrow safety fix does not advance production wiring, verification, or
  acceptance for the full meeting-bot responsibility.

## 4. Requirements

- R1: `duration_ms` must return `0` when `channels == 0` and must not divide by
  zero or panic.
- R2: For positive channel counts, duration remains
  `floor((data_bytes / (channels * 2)) * 1000 / sample_rate)`.
- R3: The change remains inside `meeting_bot::audio_bridge`; no second audio
  or lifecycle owner is introduced.

## 5. Current and target state

Current: `MeetingAudioChunk::new` accepts any `u8` channel count and
`duration_ms` divides by `channels * 2` unconditionally.

Target: malformed zero-channel metadata produces a bounded zero-duration
result; valid metadata is unchanged.

## 6. Failure modes and edge cases

Zero channels are treated as invalid metadata and produce `0`. Empty data with
positive channels remains `0`. The existing fixed sample-rate enum prevents a
zero sample-rate divisor.

## 7. Implementation strategy

Add a fail-closed guard before the existing PCM arithmetic. Keep all ownership
and API shapes unchanged.

## 8. Verification strategy

During the development freeze, perform static review and `git diff --check`
only. A future exact-head contract must cover zero channels, empty data, and a
valid mono duration; no local OCRM test/build command is run in this phase.

## 9. Acceptance criteria

- AC1: The source contains an explicit zero-channel guard before division.
- AC2: The valid mono calculation is unchanged.
- AC3: Full meeting-bot wiring and acceptance remain pending.

## 10. Release / rollback / observability

No migration or release action applies. Rollback is a one-commit revert. No
new telemetry is required for this pure malformed-input guard.

## 11. References / provenance

- `source/host/src/extensions/meeting_bot/audio_bridge.rs`
- `source/host/tests/openclaw_meeting_bot_contract.rs`
- `projects/openclaw-rust/dossiers/meeting-bot-scoped.md`
- `AGENTS.md`

## 12. Spec compliance record

| Requirement / AC | Status | Evidence / reason |
| --- | --- | --- |
| R1 / AC1 | passed | `audio_bridge.rs::duration_ms` returns before division when `channels == 0`; `git diff --check` passed |
| R2 / AC2 | passed | Existing positive-channel arithmetic is unchanged; static source review completed |
| R3 / AC3 | passed | Only the existing meeting-bot audio owner changed; no wiring/acceptance status was promoted |
