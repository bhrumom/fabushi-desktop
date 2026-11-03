# Exact speech protocol

Status: architecture/specification required before implementation  
Project: OCRM-001  
Responsibility: `OCRM-RESP-TALK-EXACT-SPEECH-001`  
Owner boundary: existing Runner voice-consult owner, once identified

## Context and problem

OpenClaw classifies a realtime voice consult request before normal delegation.
A complete exact-playback message may bypass a new consultation only when its
text is present in session-owned retained speech state. The marker text alone
is untrusted and must not authorize the bypass; malformed requests fall back to
the normal consult error path.

The current Fabushi source has no `classify_realtime_voice_consult_tool_call`
implementation, no retained exact-speech state owner, and no production voice
consult producer/consumer pair. The existing `openclaw_audio_talk_contract.rs`
contains a private test helper with similar logic; it is not production
evidence and cannot be promoted to the runtime owner.

## Goal

Implement exact-speech classification in the existing Runner voice-consult
boundary only after the real consult tool producer, session-owned retained
speech state, and production consumer are identified and bound together.

## Non-goals

- Do not add a second Agent loop, voice session store, or notification owner.
- Do not authorize replay from marker text without retained session state.
- Do not treat the test helper or static sentinel strings as runtime wiring.
- Do not claim voice/provider/device acceptance from a pure classifier.

## Required design decisions

- ESP-D01: identify the production tool-call argument producer and the exact
  Runner consumer that currently delegates voice consults.
- ESP-D02: select the existing session/voice owner for retained exact-speech
  texts, including append/retire lifecycle and generation fencing.
- ESP-D03: define malformed-argument and late-echo outcomes without silently
  converting an unknown result into a replay.
- ESP-D04: preserve the existing provider and channel notification owners;
  the classifier may only return a typed decision to those owners.

## Required behavior once unblocked

- Build exact-playback messages with JSON-string-safe answer text.
- Extract only the supported consult argument fields.
- Return `exact-speech-echo` only for a retained session text.
- Return normal `consult` for ordinary requests and late unretained echoes.
- Return `malformed` for invalid consult arguments through the existing error
  formatting owner.

## Verification and Definition of Done

- Source review proves producer → retained session state → Runner classifier →
  existing consult/notification consumer.
- Focused oracle covers authorized replay, marker-only rejection, escaped JSON,
  late echo fallback, ordinary consult, and malformed input.
- Exact-head Actions and real voice runtime evidence remain required before
  verification or acceptance is promoted.
- Until ESP-D01 through ESP-D04 are resolved, ledger status remains
  `implementation=not-started`, `wiring=not-wired`, `verification=not-run`,
  `acceptance=pending`.

## References

- `projects/openclaw-rust/SOURCE_OF_TRUTH.md`
- `projects/openclaw-rust/dossiers/audio-speech-talk-scoped.md`
- `source/host/src/runner/mod.rs`
- `source/host/src/runner/production_turn_agent_owner.rs`
- `source/host/tests/openclaw_audio_talk_contract.rs`
- OpenClaw `src/talk/exact-speech-protocol.ts` at the current discovery subject
