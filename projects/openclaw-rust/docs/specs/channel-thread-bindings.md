# Channel Thread Bindings — Development Specification

## 1. Context / problem

`OCRM-RESP-CHANNEL-THREAD-BINDINGS-001` requires channel messages to retain
thread affinity while entering the existing Fabushi transcript/session and
background-turn owners. The current Host has a generic `wakeForInbound` path,
but its envelope is opaque to the session owner: the path persists channel
address, sender, text, timestamp, and session key, then starts the existing
background turn. It does not carry a provider-owned thread identity or persist
the resulting conversation scope.

The connector owner already contains `ChannelThreadRouter`, but
`resolve_or_create` has no production caller. A direct call at the generic wake
boundary would guess the provider contract and would create a second,
unreviewed routing authority.

This specification is revalidated against the current implementation subject
at `fabushi-desktop` commit `217d60ea9e38f5cf2f303ba615db682acb3ed261`, tree
`8f2e889961ee93b2ea292b3f1766dbf801040b6e`. The OpenClaw source identity
remains the authority recorded by `projects/openclaw-rust/SOURCE_OF_TRUTH.md`;
the current implementation subject does not promote the discovery-only
upstream candidate or alter any acceptance flag.

## 2. Goal

Provide one authenticated inbound-channel ingress contract that resolves a
provider-owned thread to one canonical conversation scope and delivers the
message through the existing transcript/session and background-turn owners,
with durable retry and lifecycle semantics.

## 3. Non-goals / out of scope

- Do not add a second session, transcript, conversation, or thread-binding
  store.
- Do not infer thread identity from arbitrary text, sender, channel address,
  agent ID, or an untrusted request body.
- Do not make `wakeForInbound` itself a provider adapter or transport receiver.
- Do not promote the in-memory `ChannelThreadRouter` map to durable state
  without an approved persistence owner and lifecycle contract.
- Do not claim provider, device, packaged, runtime, OCRM, or acceptance success
  from unit contracts or static reachability alone.

## 4. Functional requirements

### R1 — Authenticated ingress identity

The production ingress must supply a validated provider/channel identity,
account scope, message identity, and optional provider-owned thread identity.
Missing or malformed identity fails closed before transcript mutation.

### R2 — Canonical scope resolution

The existing connector owner resolves `(normalized channel, provider thread)`
to one canonical conversation scope. Repeated messages for the same binding
must select the same scope; root messages from distinct channel scopes must not
share a scope.

### R3 — Existing owner handoff

The resolved scope must be handed to the existing session/transcript owner and
background-turn owner. The ingress must not write a parallel transcript or
maintain a parallel session registry.

### R4 — Idempotency and retry

Message identity must fence duplicate delivery. A retry after an ambiguous
failure must either observe the already-settled transcript admission or safely
re-attempt it without duplicating the user message or starting a second turn.

### R5 — Failure ordering

Transcript/session admission must settle before a transport acknowledgement.
Admission failure must not dispatch a turn. Worker, provider, or upgrade
failure must preserve a retryable pending state owned by the existing wake or
ingress lifecycle.

### R6 — Binding lifecycle

Idle timeout, maximum age, isolated/fork spawn policy, replacement, and cleanup
must have one durable owner. A stale lifecycle operation must not remove a
replacement binding.

## 5. Current state

- `source/host/app/src/main.rs::wakeForInbound` accepts an agent ID and opaque
  envelope, queues it, and starts the existing revival worker.
- `append_channel_inbound_entries` persists channel address, sender, text,
  timestamp, and the admitted session key through the existing session worker.
- `run_channel_inbound_revival_worker` builds the inbound prompt and invokes the
  existing background revival turn.
- `source/host/src/connectors/channel_delivery.rs::ChannelThreadRouter` is a
  pure in-memory contract owner with no production caller or durable binding
  lifecycle.
- Therefore the responsibility remains `implementation=implemented`,
  `wiring=not-wired`, `verification=not-run`, and `acceptance=pending`.

## 6. Target architecture / ownership

The provider-specific authenticated ingress owns transport validation and
constructs a typed inbound event. `ChannelThreadRouter` owns only normalized
thread-affinity resolution. The existing session/transcript owner owns
conversation admission, durable transcript mutation, and message-id fencing.
The existing background-turn owner owns dispatch and redrive. No layer may
silently take another layer's state ownership.

Required flow:

`authenticated provider ingress → typed channel event → thread router →
existing session/transcript admission → existing background turn`

## 7. Interfaces and data contract

Before runtime implementation, the owning ingress must define a typed event
equivalent to:

```text
InboundChannelEvent {
  provider_id,
  account_id,
  channel_id,
  thread_id: optional provider-owned identity,
  message_id,
  sender,
  content,
  received_at,
  authentication/provenance receipt
}
```

The exact Rust type and persistence schema require review against the selected
provider adapter. `serde_json::Value` at `wakeForInbound` is not sufficient as
the canonical contract. The event must preserve the provider identity and
message identity through scope resolution and transcript admission.

## 8. Failure modes and edge cases

- Missing provider/account/channel/message identity: reject without mutation.
- Empty thread identity: use the explicitly defined root scope only after the
  provider adapter has authenticated the channel; never use sender text as a
  substitute.
- Duplicate message ID: return the existing settlement and do not append or
  dispatch twice.
- Binding replacement racing with cleanup: compare the binding generation or
  revision before removal.
- Transcript admission failure: do not dispatch; retain retry ownership.
- Turn dispatch failure after durable admission: preserve the durable message
  and use the existing redrive semantics without duplicating it.
- Provider/account mismatch: fail closed; never merge scopes across accounts.

## 9. Implementation strategy

1. Select or establish the authenticated provider ingress owner and its durable
   message-receipt owner.
2. Define the typed event and exact provider/account/thread/message identity
   contract.
3. Extend the existing session/transcript owner with an idempotent admission
   seam that accepts the resolved canonical scope.
4. Call `ChannelThreadRouter` from that ingress seam, not from an opaque generic
   gateway command.
5. Preserve the existing background-turn owner and redrive behavior.
6. Add focused contracts for identity rejection, scope affinity, duplicate
   fencing, failure ordering, and replacement-safe lifecycle cleanup.
7. Update the ledger and provenance only after source and wiring are directly
   evidenced. Keep verification and acceptance pending until their required
   exact-head checks run.

## 10. Verification strategy

Static review may establish ownership, call-site, and contract facts only.
Focused tests may establish pure router and admission invariants. Production
acceptance additionally requires a real authenticated provider ingress,
durable restart/retry evidence, and the repository's applicable OCRM gates.
This worktree's current task constraints prohibit running those commands, so
no verification or acceptance status may be advanced in this phase.

## 11. Acceptance criteria / Definition of Done

- AC1: One authenticated provider ingress produces the typed event with stable
  provider/account/channel/thread/message identity.
- AC2: The existing `ChannelThreadRouter` is called by that ingress and no
  second routing/session/transcript owner is introduced.
- AC3: Same-thread messages share one canonical scope; distinct root/channel
  scopes remain isolated.
- AC4: Duplicate and retry delivery is idempotent across the durable receipt
  boundary.
- AC5: Admission-before-ack and admission-before-dispatch ordering is directly
  evidenced, including failure and redrive paths.
- AC6: Binding lifecycle replacement and cleanup are revision-safe.
- AC7: Focused contracts pass, required repository verification runs on the
  exact accepted head, and only then may ledger wiring/verification/acceptance
  be reconsidered.

## 12. References / provenance

- `projects/openclaw-rust/parity-ledger.json` —
  `OCRM-RESP-CHANNEL-THREAD-BINDINGS-001`.
- `projects/openclaw-rust/dossiers/channels-outbound-delivery-scoped.md` —
  current inbound wake versus thread-binding reachability audit.
- `source/host/app/src/main.rs` — `wakeForInbound`,
  `append_channel_inbound_entries`, and
  `run_channel_inbound_revival_worker`.
- `source/host/src/connectors/channel_delivery.rs` —
  `ChannelThreadRouter`.
- `projects/openclaw-rust/dossiers/ocrm-closure-state-audit-20261031.md` —
  unresolved producer/consumer frontier.
