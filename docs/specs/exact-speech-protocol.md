# Exact Speech Protocol — Specification

Status: active  
Owner: OCRM session/talk runtime owner  
Last updated: 2026-10-31  
Related project: `projects/openclaw-rust`  
Related task / issue / PR: `OCRM-RESP-TALK-EXACT-SPEECH-001`

## 1. Context / problem

The current Rust target has no production exact-speech classifier. The only
available behavior is a fixture helper in
`source/host/tests/openclaw_audio_talk_contract.rs`; fixture behavior cannot
prevent recursive voice-consult re-entry in the Runner.

## 2. Goal

Provide a deterministic Rust classifier for a realtime voice-consult payload
that distinguishes a retained exact-speech playback echo from a new peer
consult, and rejects malformed sentinel payloads.

## 3. Non-goals / out of scope

- This slice does not invent a realtime voice-consult producer or playback
  transport.
- It does not claim Runner or packaged Talk wiring until a real producer,
  retained-speech state owner, and consumer are identified.
- It does not mutate transcript or session state.

## 4. Requirements

- R1: A payload without the exact-speech prefix is classified as a peer
  consult, preserving the original text.
- R2: A payload with the prefix must also contain the required no-tool
  directive, exact-speech instruction, and `Answer: "…"` delimiter; otherwise
  it is malformed.
- R3: A valid extracted answer matching retained exact-speech text is
  classified as an echo; a non-matching answer is classified as a peer
  consult.
- R4: Classification is pure and does not alter retained state or global
  agent configuration.

## 5. Current state

The fixture helper implements R1–R3 privately, but no declared Rust owner or
production caller exists. The ledger remains `wiring=not-wired`,
`verification=not-run`, and `acceptance=pending`.

## 6. Target state

`source/host/src/runner/mod.rs` exports the typed classifier and sentinel
constants. It is available for a future verified Runner consumer without
duplicating classification logic in a fixture.

## 7. Architecture and ownership boundaries

- `runner/mod.rs` owns only pure classification and typed outcomes.
- A future Talk producer owns payload creation and a future Runner/turn owner
  owns retained exact-speech state and suppression side effects.
- The classifier must not own audio playback, transcript persistence, or tool
  dispatch.

## 8. Interfaces / contracts / schemas / data flow

`classify_realtime_voice_consult_tool_call(text, retained_exact_speech)` takes
one payload and an immutable retained-answer slice and returns:

- `ExactSpeechEcho(answer)`;
- `PeerConsult(text)` or `PeerConsult(answer)`; or
- `Malformed(reason)`.

The exact sentinel strings are constants and answer extraction uses the final
closing quote after the `Answer: "` delimiter, matching the audited fixture
contract.

## 9. Constraints and non-functional requirements

- No I/O, network access, locks, or mutation in the classifier.
- Preserve the original payload for non-sentinel peer consults.
- Do not infer production acceptance from fixture-only coverage.

## 10. Failure modes and edge cases

- Missing required directive/instruction is malformed.
- Missing `Answer: "` delimiter is malformed.
- Unterminated answer is malformed.
- Retained-answer mismatch is a new peer consult, not an echo.
- Empty retained state never converts a valid new answer into an echo.

## 11. Implementation strategy

Port the audited fixture classification into the existing Runner module as a
public typed pure function, preserving sentinel text and failure distinctions.
Update the OCRM dossier and parity ledger to record implementation progress
without upgrading wiring or acceptance.

## 12. Verification / test strategy

- Static: `git diff --check` and source review against R1–R4.
- Contract: exercise sentinel, malformed, retained-match, and retained-miss
  cases in GitHub Actions after the development freeze.
- Integration: identify and test the real producer/consumer before changing
  `wiring` or `acceptance`.

## 13. Acceptance criteria / Definition of Done

- AC-1: Typed Rust outcomes and constants implement R1–R4.
- AC-2: No fixture-only result is recorded as production wiring.
- AC-3: Exact-head Actions contract evidence and a real Runner consumer are
  required before final acceptance.

## 14. Release / migration / rollback

No release or migration in this slice. Rollback is removal of the pure
classifier and its ledger/spec progress without changing runtime state.

## 15. Observability / evidence

Static source diff and future exact-head contract logs are the required
evidence. No runtime telemetry is introduced by this pure function.

## 16. References / provenance

- `projects/openclaw-rust/SOURCE_OF_TRUTH.md`
- `projects/openclaw-rust/dossiers/audio-speech-talk-scoped.md`
- `source/host/tests/openclaw_audio_talk_contract.rs` fixture oracle
- Upstream source record `src/talk/exact-speech-protocol.ts`, blob
  `de7fe86c4709e4dd09ace653f43b0b5802126b18`

## 17. Spec compliance record

| Requirement / AC | Status | Evidence / reason |
| --- | --- | --- |
| R1–R4 | pending | Implementation follows after this spec is added. |
| AC-1–AC-3 | pending | Exact-head verification and production consumer remain pending. |
