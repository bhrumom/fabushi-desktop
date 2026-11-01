# Coordinator renderer reply settlement — Specification

Status: active
Owner: Fabushi Desktop renderer transport owner
Last updated: 2026-11-01
Related project: `projects/openclaw-rust`
Related task / issue / PR: N/A

## 1. Context / problem

`frontend/src/production/coordinator-client.ts` removes a pending request as
soon as a matching reply frame is found, before validating the reply outcome.
For an invalid outcome status, or a failed outcome without a failure object,
the Promise is then neither resolved nor rejected and cannot be reclaimed by a
later transport close or replacement.

## 2. Goal

Every matching reply for an owned pending request must settle that request
exactly once: valid success resolves, valid failure rejects, and malformed
outcomes reject explicitly while preserving no indefinitely pending Promise.

## 3. Non-goals / out of scope

- No automatic replay of a request after transport replacement.
- No claim about whether a remote side effect occurred before a malformed
  reply.
- No new RPC owner, durable business-result store, or account/session owner.
- No change to Coordinator wire schemas beyond local malformed-reply handling.

## 4. Requirements

- R1: A matching `requestId` remains owned until its reply is classified and
  the pending Promise is settled.
- R2: `outcome.status === "ok"` resolves after existing method validation.
- R3: `outcome.status === "failed"` rejects when a failure object is present;
  a missing/invalid failure object rejects with a protocol error.
- R4: Any other outcome status rejects with a protocol error and does not
  leave the request pending.
- R5: Unmatched request IDs retain the existing ignore behavior.
- R6: A malformed frame's protocol-error shutdown send cannot prevent the
  existing disconnect cleanup from reclaiming pending requests and the port
  claim.
- R7: `dispose()` must reclaim the client claim, reject pending calls, and
  release the endpoint even when its shutdown send fails synchronously.
- R8: A newly claimed port whose hello send fails must enter the existing
  disconnect/reclaim path rather than escaping from the port-claim callback.
- R9: A request send failure after pending registration must close the failed
  endpoint and settle the request through the existing disconnect/reclaim path.
- R10: A reply frame with a missing or malformed outcome must trigger the
  existing protocol-breach disconnect path instead of being ignored.

## 5. Current state

The production renderer client validates only that `outcome` is an object,
deletes the pending record, and handles only two recognized combinations.
Invalid combinations therefore lose the only Promise settlement handle.

## 6. Target state

The existing renderer transport client owns a bounded state transition from
pending to exactly one terminal local result for every matching reply. The
existing disconnect, replacement, and disposal cleanup paths remain the
fallback for requests that have not received a matching reply.

## 7. Architecture and ownership boundaries

The renderer `createCoordinatorClient` owns ephemeral request/Promise
settlement. The transferred port remains the transport owner; Coordinator and
Host remain business owners. No reply parser or second pending registry is
introduced.

## 8. Interfaces / contracts / schemas / data flow

Input is the existing reply frame `{ kind: "reply", requestId, outcome }`.
The client classifies `outcome.status` and uses the existing `CoordinatorCallError`
shape for known failures; malformed outcomes use a local protocol-error
exception. There is no storage or migration.

## 9. Constraints and non-functional requirements

- Settlement must be bounded and exactly once.
- Malformed remote input must fail closed without throwing out of the message
  handler before pending cleanup.
- Existing success/failure and reconnect compatibility must remain unchanged.

## 10. Failure modes and edge cases

Cover invalid status, failed status without an object, failed object with
missing/non-string fields, unmatched request ID, disconnect after malformed
reply, malformed-frame shutdown send failure, and disposal after malformed
reply, disposal shutdown-send failure, and endpoint close failure. A remote
side effect remains outcome-unknown and must not be replayed by this client.
Also cover a synchronous request `postMessage` failure after pending
registration, and a reply frame whose outcome is absent or malformed.

## 11. Implementation strategy

Classify the outcome before removing the pending entry, then remove it exactly
once and settle it through the existing resolve/reject callbacks. Keep method
response validation on successful outcomes and preserve the current handling
for valid failure objects. Wrap only the malformed-frame shutdown send so its
failure still reaches the existing disconnect owner. Make explicit disposal's
local cleanup unconditional after the best-effort shutdown send, and route a
hello-send exception through the same disconnect owner. Treat a request-send
exception as transport failure, close that endpoint best-effort, and use the
same disconnect owner to settle all pending calls.
Treat a reply-shaped frame that fails the local outcome shape gate as a
protocol breach and use the same disconnect owner.

## 12. Verification / test strategy

Static review must show every matching-reply branch settles or explicitly
rejects the pending Promise. The focused renderer contract should exercise
valid success, valid failure, invalid status, missing failure object,
unmatched request ID, and cleanup after malformed input. Under the current
development freeze, local tests/builds and OCRM CI remain not run.

## 13. Acceptance criteria / Definition of Done

- AC-1: R1–R5 are implemented in the existing renderer client owner.
- AC-2: No malformed matching reply can delete a pending request without
  settling it.
- AC-3: `git diff --check` and source-level review pass; runtime/test and
  packaged acceptance remain pending until their authorized lanes run.

## 14. Release / migration / rollback

No migration. Rollback is reverting the single renderer-client change.

## 15. Observability / evidence

The rejection error includes a protocol-error message identifying the invalid
reply shape. Source diff and focused contract evidence are the required local
artifacts; no runtime acceptance is claimed here.

## 16. References / provenance

- `AGENTS.md`
- `projects/openclaw-rust/SOURCE_OF_TRUTH.md`
- `projects/openclaw-rust/dossiers/transcript-round67-renderer-malformed-reply-and-onboarding-connect-20261024.md`
- `frontend/src/production/coordinator-client.ts`

## 17. Spec compliance record

| Requirement / AC | Status | Evidence / reason |
| --- | --- | --- |
| R1–R10 | passed | `coordinator-client.ts` settles valid replies and routes malformed reply frames plus all covered transport failures through bounded disconnect/reclaim cleanup. |
| AC-1 | passed | Existing renderer client remains the sole pending/Promise owner; no new transport or business owner was added. |
| AC-2 | passed | Every matching reply branch removes the pending entry only immediately before resolving or rejecting it. |
| AC-3 | passed | `git diff --check` and source review passed; runtime, focused test, and packaged acceptance remain pending. |
