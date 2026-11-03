# Heartbeat response policy

Status: active
Owner: OCRM Runner owner
Last updated: 2026-11-01
Related responsibility: `OCRM-RESP-AUTO-REPLY-HEARTBEAT-001`

## Context / problem

The fixed OpenClaw source has a typed `heartbeat_respond` response contract,
but Fabushi currently owns only heartbeat prompt and acknowledgement policy.
The response contract can be implemented as a pure Runner policy primitive,
provided it does not claim ownership of scheduling, sessions, transcripts,
notifications, or durable outcome state.

Fixed source authority:
`openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`, tree
`04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`,
`src/auto-reply/heartbeat-tool-response.ts`.

## Goal

Provide a deterministic Rust normalizer for the structured
`heartbeat_respond` result so a future existing-owner consumer can distinguish
valid visible/silent outcomes without inventing missing fields or native
session identities.

## Non-goals

- No heartbeat timer, scheduler, wake admission, or retry loop.
- No session/transcript publication, filtering, notification transport, or
  durable outcome store.
- No response-tool registration or production call-chain claim.
- No truncation beyond the fixed source's scratch byte limit.

## Requirements

- HPR-01: Reject non-object values, missing/blank `outcome`, missing/non-boolean
  `notify`, and missing/blank `summary`.
- HPR-02: Accept only `no_change`, `progress`, `done`, `blocked`, and
  `needs_attention` outcomes.
- HPR-03: Preserve optional non-blank `notificationText` (including the
  `notification_text` alias), `reason`, and `nextCheck` strings; accept only
  `low`, `normal`, or `high` for priority, otherwise omit the priority.
- HPR-04: Preserve an explicitly supplied string `scratch` only when its UTF-8
  size is at most 256 KiB; reject oversized scratch. Non-string scratch is
  treated as absent, matching the fixed source's type guard.
- HPR-05: Keep this contract pure and owned by the Runner policy module; no
  production producer, consumer, or acceptance status changes follow from this
  slice.

## Failure modes and edge cases

Blank required strings reject the result. Blank optional strings are omitted.
Unknown priority is omitted rather than rejecting the whole result. Both
camelCase and snake_case aliases are recognized for the two fixed-source alias
fields. Native thread/turn/session identifiers are outside this contract and
must never be synthesized.

## Implementation strategy

Add an owned response struct, enums, and a `serde_json::Value` normalizer to
`source/host/src/runner/heartbeat.rs`, with focused in-module contract tests.
Do not wire it into a runtime path until a separate spec identifies the real
heartbeat producer, session admission, result consumer, and notification owner.

## Verification / Definition of Done

- Static review maps HPR-01..HPR-05 to the Rust owner and fixed source.
- Focused tests cover valid visible/silent outcomes, required-field rejection,
  aliases, unknown priority omission, scratch size rejection, and no invented
  identities.
- Local build/test/formatter/lint and OCRM CI remain intentionally unrun under
  the current development freeze.
- The parity row remains `wiring=not-wired`, `verification=not-run`, and
  `acceptance=pending` for the full heartbeat responsibility.

## References

- `AGENTS.md`
- `projects/openclaw-rust/SOURCE_OF_TRUTH.md`
- `projects/openclaw-rust/dossiers/cron-automations-heartbeat-scoped.md`
- `source/host/src/runner/heartbeat.rs`
- Fixed source:
  `/Users/bhrum/.gemini/antigravity/scratch/openclaw/src/auto-reply/heartbeat-tool-response.ts`
