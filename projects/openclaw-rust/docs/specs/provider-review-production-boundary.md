# Provider-review production boundary

Status: typed Codex producer, durable write, Gateway continuation admission, and bounded terminal settlement wired; supported native continuation identity remains unavailable in the current Rust producer
Date: 2026-11-01
Responsibility: `OCRM-RESP-SESSION-WORK-ADMISSION-001`

## Context and goal

The fixed OpenClaw source records provider-authored refusal findings against
one admitted session/run and lifecycle generation, then requires a host-issued
acknowledgment before a continuation run can claim the review. Fabushi's
current Rust provider path has bounded producer → admission owner → durable
state → continuation-admission → terminal-settlement wiring. Full native
continuation identity, detached lifecycle recovery, and exact production
acceptance remain outside this bounded slice. This spec defines the remaining
lifecycle using existing Fabushi owners.

The fixed refusal envelope was re-read against the frozen upstream subject:
the direct Responses API shape supplies only the bounded explanation,
continuation message, and provider error type. Native `threadId`/`turnId` are
owned by the separate Codex app-server/native runtime path; a Responses
`response_id` is not equivalent evidence and must not be projected into either
native field.

## Non-goals

- Do not classify provider-review state from error strings, HTTP status alone,
  or `SandAutoReview` tool/cloud-agent approval state.
- Do not create a second review database, global acknowledgment registry, or
  provider-specific session owner.
- Do not permit request metadata to manufacture an acknowledgment capability.
- Do not claim equivalence from a fixture, static symbol, or generic error
  propagation.

## Functional requirements

- `PR-01`: A structured provider refusal must carry `sessionId`, `runId`,
  provider, model, runtime identity, optional API/native turn identity, and
  bounded provider-authored review details.
- `PR-02`: The producer must assert the admitted session identity and captured
  lifecycle revision before recording a refusal.
- `PR-03`: Durable review writes are compare-and-set/idempotent. Repeated
  terminal notifications retain the existing review identity; a changed
  session, run, or generation fails closed.
- `PR-04`: Only the existing session admission owner stores review identity,
  phase, and the admitted continuation run identity; no provider runtime owns
  durable review state. `pending` is actionable, `accepted` is in-flight,
  `completed` clears the review, and `cancelled`/`failed`/`outcome-unknown`
  retain it as non-actionable terminal evidence.
- `PR-05`: A continuation requires a host-issued, non-serializable
  acknowledgment bound to the stored review ID and next run ID. One
  acknowledgment may be claimed by at most one admitted attempt.
- `PR-06`: Runtime/provider/model/API identity and lifecycle must be rechecked
  at continuation start and native-turn acceptance; stale acknowledgments are
  retired and cannot be retried.
- `PR-06a`: A continuation run must carry the captured lifecycle revision and
  run identity to the existing Runner terminal owner. A known successful
  terminal outcome clears the matching review; cancellation, failure, and
  outcome-unknown outcomes retain a non-actionable terminal phase and release
  the matching admitted work marker. A dispatch failure before Runner
  ownership restores `pending`.
- `PR-07`: Incognito, unsupported runtime/API, missing continuation detail,
  duplicate/conflicting review, cancellation, and outcome-unknown paths fail
  closed with explicit outcomes.

## Current and target state

Current Rust state now has a migration-safe `SessionProviderReview` nested in
`SessionWorkAdmissionRecord` with generation-bound idempotent/conflict-safe
owner writes and reads. The Codex Responses decoder recognizes only the fixed
`misalignment_policy_violation` envelope, carries bounded typed refusal facts,
and the terminal Runner owner records those facts against the current
admission generation. The existing session owner also issues a
non-serializable, single-claim acknowledgment and rechecks provider/model/
runtime/API identity before marking the durable review accepted; issuance now
fails closed when continuation details or supported native/runtime identity is
missing. The authenticated `sessions.providerReview.continue` Gateway owner now
performs generation-bound admission, issues and consumes the capability once,
and dispatches the bounded continuation through the existing routed-send/run
owners; unsupported current Rust identities fail before dispatch. The existing
Runner terminal owner now settles the matching accepted continuation against
the captured generation and run identity.
`SandAutoReview` is a separate action-approval owner and is not a valid
substitute.

The current Rust `RoutedProvider::Codex` producer uses the direct ChatGPT
Responses transport and records runtime identity `codex`, but it has no native
app-server thread/turn pair. Its own reviews therefore remain inspectable and
durable while continuation capability issuance correctly fails closed.

Target state is one owner-local review record bound to the session admission
generation. The provider/turn owner emits a typed refusal; the session owner
records it; the authenticated Gateway continuation producer requests a
host-issued acknowledgment; the next run consumes that capability before
provider dispatch; and the same session owner settles or clears the review only
after the continuation has a known terminal outcome.

## Ownership and interfaces

- Provider decoding owns recognition of a documented structured refusal event;
  it must return unknown when the event shape is not recognized.
- `ProductionTurnAgentOwner` owns run identity and terminal timing.
- `ProductionSessionWorkers`/`SandAgentDb` own lifecycle CAS, durable review
  identity, phase, and idempotency.
- `UnifiedGatewayApi::call_openclaw_sessions_provider_review_continue` owns
  authenticated admission and capability issuance; serialized request fields
  are inputs only, never proof of issuance. It then dispatches through the
  existing `accept_routed_send` and `runner.startRoutedProvider` owners.
- `ProductionTranscriptRuntime` owns transcript publication and must not infer
  review acceptance from terminal text.
- The existing Runner/turn settlement owner must report a known continuation
  outcome back to the session owner; dispatch failure, cancellation, and
  outcome-unknown paths must retain or explicitly settle the review without
  making a stale `accepted` phase look actionable.

## Failure modes

Provider refusal with incomplete identity, lifecycle change, archived/deleted
session, duplicate review with different facts, lost durable write, stale
acknowledgment, unsupported continuation, cancellation, and uncertain
provider/session outcome all remain non-accepted and must not start a successor
run.

## Implementation strategy

1. Confirm the fixed provider event/schema and add a typed Rust refusal value
   with bounded fields and no string classifier.
2. Add migration-safe owner-local review CAS/read/settle methods and direct
   negative contracts.
3. Bind terminal producer and lifecycle recheck to the existing turn owner.
4. Add authenticated capability issuance and one-claim continuation admission.
5. Wire the accepted review into the existing provider start path, then add
   exact-head verification and runtime evidence.

## Verification and Definition of Done

`PR-01`–`PR-07` each require direct positive and negative evidence: malformed
refusal, stale generation, duplicate/conflicting review, forged serialized
acknowledgment, double claim, runtime mismatch, incognito rejection, and
successful accepted continuation. The bounded terminal-settlement contract
requires direct evidence for success, cancellation, dispatch failure, and
outcome-unknown handling, including review projection after settlement. Static
contracts are design evidence only; the parity row remains pending until the
full lifecycle wiring and exact-head verification are complete.

## References

- Fixed upstream `src/sessions/provider-review.ts`
- Fixed upstream `packages/llm-core/src/utils/provider-refusal.ts`
- `projects/openclaw-rust/docs/specs/session-work-admission.md`
- `source/host/src/extensions/inference/provider_session.rs`
- `source/host/src/runner/production_turn_agent_owner.rs`
- `source/host/src/extensions/session/agent_db.rs`
- `source/host/src/extensions/session/production.rs`

Fixed-source identity recheck against commit
`ad8d12f126d29b0db204187adf5092db80294e70` / tree
`04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`:

- `src/sessions/provider-review.ts` — blob
  `28a1a20adda8a961b9d17ec2d2139f71906cf134`
- `packages/llm-core/src/utils/provider-refusal.ts` — blob
  `64e061db48b6b5358a193dd097f740b3d2a9907e`
