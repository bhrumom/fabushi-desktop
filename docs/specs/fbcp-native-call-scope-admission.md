# Native Human call creation: canonical conversation admission

Spec ID: FBCP-CALL-SCOPE-001  
Status: active, implementation/verification pending  
Task: `20bf9475-a885-4314-b509-bea53ef8849f`, round 1  
Date: 2026-11-02

## Current problem and scope

At checkpoint `e2d64c2c53d9a91f3188b99203bce65e5d68f894` (rechecked against subsequent unchanged call-creation code), the mounted `HumanCallControls` sends `createCallSession` with the selected conversation ID and member IDs. The Session gateway forwards those fields to the existing `ProductionSessionWorkers::create_call_session`. That method verifies the conversation is readable, but independently selects its outbound Native call peer from the caller's participant array. For a valid A↔B conversation, `[A,C]` can therefore create a remote call to C and persist it under the A↔B scope. Later ordinary call discovery derives A↔C and rejects the mismatched durable scope.

Repair this existing production creation boundary. No new call, conversation, identity, permission, storage or UI owner is introduced. Preserve concurrent NMR-49 reply recovery, NMR-50 forward/roster/request-frame work, NMR-51 upload limits and all existing Agent/Bot behavior. This bounded admission repair does not claim complete call/media, remote-service or migration acceptance.

## Required behavior and ownership

- CALL-SCOPE-01: In the shipping Native call path, derive the authorized direct participant pair from the selected canonical Human conversation's current metadata. Require exactly two valid distinct stored Human IDs containing the authenticated local identity. The requested normalized pair must equal that pair. The request may not substitute a different peer even if both conversations are independently readable.
- CALL-SCOPE-02: Reject mismatched peer, missing local identity, self-only, extra distinct participants and malformed/ambiguous stored pair before generating a remote call effect or writing a CallSession record. Keep prior conversation metadata, transcript and existing call rows intact. Preserve the native credential-user check and backend new-call validation.
- CALL-SCOPE-03: A matching pair retains the existing call ID, native POST, durable CallSession, state machine, gateway response and ordinary discovery/reopen behavior. Existing normalized participant order semantics remain compatible. Local non-Native CallSession operation keeps its current behavior; this change does not redefine future group-call semantics.
- CALL-SCOPE-04: Use the actual Session gateway, native HTTP client and SQLite owner in bounded loopback contracts. Check no HTTP and unchanged durable state for negatives, a real positive create followed by existing lifecycle/reopen behavior, and a service refusal with no local call creation. Do not substitute source-string tests. Reuse the existing Actions/native-service harness and preserve its current Human/Agent/source/UI/platform gates.

The production mutation is confined to the call-creation function and, if necessary, a private same-owner pair-admission helper. Existing `open_human_conversation_db_owner` supplies kind/ID checking; conversation metadata owns participant identity. CallSessionStore owns call persistence; the backend owns the remote call; UI remains a projection/command caller. This batch does not alter public command/event/service schemas.

## Failures, recovery and verification

Test a legitimate A↔B scope with a substituted C, a second readable A↔C scope with B requested, invalid request sets, and malformed stored participants. Assert no request reaches the loopback service and no call is added. Reopen the original SQLite stores and verify metadata/transcript and existing call rows remain. A positive A↔B request must reach the service with B, return/persist the same call ID and scope, and remain accessible through the existing store after reopen. A backend refusal must return the existing error path without a local phantom call. Preserve the existing shipping call lifecycle/discovery regression cases and the local CallSession lifecycle/restart contract.

All executable validation is GitHub Actions only, at the exact published commit. Record selected command, complete log, job/artifact, actual test count and independent source verdict. Queued, unrun, failing or wrong-head evidence is not pass. No visual layout change is planned; the current mounted caller is checked by source, and broader real-service/UI/media/device/credential-rotation/platform acceptance remains open.

## Acceptance and rollback

AC-CALL-SCOPE-01: wrong-conversation participants cannot create remote or local call state; negatives prove unchanged durable state and zero HTTP. AC-CALL-SCOPE-02: valid creation/refusal/lifecycle/reopen retains the existing owner and no phantom call. AC-CALL-SCOPE-03: current callers, responsibility linkage, tests, exact-H Actions and independent review are recorded separately in the existing owner matrix, RTM and ledger.

Deliver through a non-force expected-head update on the existing migration lineage. The shared checkpoint advanced during three CAS publication attempts, so use its derived isolated branch `checkpoint/tdrp-native-call-scope-20bf9475-r1` and a PR targeting `checkpoint/tdrp-stage-a-msg-mutations-7457eb1c-r1`. Bind native/source Actions to this exact feature head, preserve concurrent target work, and reconcile all overlapping files and the canonical ledger before integration. Never publish stale full-file proposals. A normal revert removes the new admission guard without deleting call/session/user data; it reopens the wrong-scope defect and is not a shipping approval. Fixed upstream U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; no new source deep-review credit or full product ACCEPT is implied.

References: root `AGENTS.md`; FBCP/TDRP source-of-truth and main specs; FMEP-001 Revision 2 §§12–13; `docs/adr/FBCP-ADR-003-call-session-signaling.md`; existing `production.rs`, `native_messaging.rs`, Session gateway and `human-call-media.tsx` consumers.
