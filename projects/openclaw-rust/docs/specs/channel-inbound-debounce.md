# Channel inbound debounce policy — Specification

Status: active  
Owner: Host connector owner  
Last updated: 2026-10-31  
Related project: `projects/openclaw-rust`  
Related responsibility: `OCRM-RESP-CHANNEL-INBOUND-DEBOUNCE-001`

## 1. Context / problem

The existing Rust channel connector owns inbound message identity
deduplication, but it does not yet represent the upstream debounce policy.
Rapid text messages can therefore be admitted without a channel-scoped quiet
window. Control commands and media-bearing messages must remain immediately
actionable and must not be delayed behind conversational text.

## 2. Goal

Add a deterministic, bounded debounce policy to the existing
`channel_delivery` owner. The policy must resolve a delay per normalized
channel, bypass control commands and media messages immediately, and suppress
text arrivals inside the active channel window.

## 3. Non-goals / out of scope

- No inbound transport, channel adapter, timer thread, session, or turn owner.
- No replacement for `InboundChannelDeduplicator` or message cursor storage.
- No production wiring claim while a real inbound producer and turn consumer
  remain absent.
- No use of wall-clock time inside the policy; callers provide the observation
  instant.

## 4. Requirements

- R1: Normalize channel identifiers and resolve a configured delay for that
  channel, falling back to the default delay.
- R2: Control commands (`abort`, `stop`, and `status`) bypass debounce.
- R3: Messages carrying media bypass debounce.
- R4: Text messages are admitted immediately when no active channel window
  exists and are suppressed while the window is active.
- R5: Invalid channels and out-of-range delays fail closed; tracked channel
  state is bounded and stale entries are removable.

## 5. Current state

`InboundChannelDeduplicator` is a pure connector helper for message IDs and
cursors. No debounce state or channel-scoped timing contract exists, and no
production inbound producer has been identified.

## 6. Target state

`InboundChannelDebounceGate` is a pure state owner under
`source/host/src/connectors/channel_delivery.rs`. It consumes a normalized
channel, message kind, policy, and caller-supplied `Instant`, then returns
either `Immediate` or `Debounced` without performing I/O or dispatch.

## 7. Architecture and ownership boundaries

The connector owns policy state only. An eventual inbound channel adapter owns
transport receipt and an eventual Host turn owner owns dispatch. The debounce
gate must not create sessions, start timers, call providers, or mutate
transcripts.

## 8. Interfaces / contracts / schemas / data flow

`ChannelInboundMessageKind` classifies text, control commands, and media.
`ChannelInboundDebouncePolicy` stores a bounded default and per-channel delay
map. `InboundChannelDebounceGate::admit` returns
`ChannelInboundDebounceDecision` and accepts `Instant` explicitly for
deterministic callers and tests.

## 9. Constraints and non-functional requirements

- Per-channel state is bounded to a fixed maximum entry count.
- Policy values use millisecond precision and a finite upper bound.
- No secrets or message bodies are retained; only normalized channel keys and
  last admitted text instants are stored.

## 10. Failure modes and edge cases

- Empty or whitespace-only channel identifiers are rejected.
- A delay exceeding the configured maximum is rejected.
- Control commands are matched after trimming and ASCII case normalization.
- Media bypass does not update the text debounce timestamp.
- A stale timestamp outside the window admits the next text message.

## 11. Implementation strategy

Add the policy value types and bounded gate beside the existing
`InboundChannelDeduplicator`, reusing the connector's channel normalization
rule. Keep the gate independent of transport and preserve the current
deduplicator API. Add source-level contract tests for bypass, per-channel
resolution, suppression, expiry, and invalid input.

## 12. Verification / test strategy

Static review must show the policy remains a pure connector owner. Contract
tests must cover R1–R5. Tests are present as source evidence during the current
development freeze and must not be described as executed until the permitted
verification campaign runs.

## 13. Acceptance criteria / Definition of Done

- AC1: The gate exposes deterministic immediate/busy decisions for all three
  message kinds.
- AC2: Channel-specific delays override the default and do not affect another
  channel.
- AC3: Invalid input is rejected without changing gate state.
- AC4: The responsibility remains `wiring=not-wired`,
  `verification=not-run`, and `acceptance=pending` until a real producer,
  consumer, and required evidence exist.

## 14. Release / migration / rollback

No storage migration or release change. Rollback removes the pure policy types
and source-level tests; existing deduplication behavior remains unchanged.

## 15. Observability / evidence

The policy emits no logs. Future production wiring must provide channel-safe
admission/settlement evidence without recording message bodies.

## 16. References / provenance

- `projects/openclaw-rust/dossiers/channels-outbound-delivery-scoped.md`
- `projects/openclaw-rust/parity-ledger.json`
- Historical upstream source: `src/channels/inbound-debounce-policy.ts`,
  blob `edc265f0c352c44dcf816f48200c1d65a33d6a14`
- Root repository instructions in `AGENTS.md`

## 17. Spec compliance record

| Requirement / AC | Status | Evidence / reason |
| --- | --- | --- |
| R1–R5 | pending | Implementation and source tests are being added; verification is frozen. |
| AC1–AC4 | pending | Production producer/consumer and verification evidence remain open. |
