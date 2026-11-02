# Native Human call credential lifetime

Spec ID: FBCP-CALL-CREDENTIAL-001
Status: bounded AC-CALL-CRED-01–03 passed at523c8ca4883e6eaa10d1d19a0e237f9d3964b3c6; shipping/full product acceptance pending
Task: 20bf9475-a885-4314-b509-bea53ef8849f, round 1
Date: 2026-11-02

## Observed production defect

Independent source inspection at shared checkpoint 2d03120fe107b94f389409ac4f6890947940af4e (production blob3a7b85fb2b9079d3bf917dc4bc630290fbe31162, native client blob990eeb5a9e4906e2eafbd4bf0bc86f6456b16406) found an account-lifetime gap. HumanCallControls sends transitionCallSession through the existing Session gateway. Production sync checks identity before a Native GET, but the call HTTP methods do not fence the complete credential after consuming its body. A held valid call GET can cross a credential replacement from Human1 to Human2. The subsequent event POST independently reloads credentials and can emit the original Human1 command with Human2's bearer/device. A later sync can return an error only after this effect.

The response-barrier contract exercised this source-derived path at exact523c8ca/run41525143801/job125550827148; six credential cases and retained5+3+1 passed, with separate bounded independent assessment. Both Humans may be legitimate participants in the same call, but no assumption about deployed backend acceptance is needed: zero event POST is the decisive negative oracle.

## Requirements and existing owners

- CALL-CRED-01: Every shipping Native call HTTP family (ICE credentials, discovery, creation, state/event retrieval, and event append) used by a fixed-Human Session must bind its credential user to that Session's authenticated Human before request dispatch. Keep identity normalization and existing method/service contracts compatible. An earlier identity() check alone cannot authorize a later independent credential load.
- CALL-CRED-02: Reuse the private Native credential owner and its existing complete six-field snapshot check (user, device, token, session, expiry, issuer). After a call response body has been consumed, reject a changed credential before returning call/ICE data for settlement. Do not expose raw tokens, session identifiers or new credential fingerprints to UI, commands, events, logs or durable call records.
- CALL-CRED-03: A valid call GET held across any of the six credential-field rotations must not authorize a later lifecycle-event POST. The actual gateway/Native HTTP/SQLite negative must retain the existing call record, event replay cursor and signals. Restore the original credential and prove normal retry/settlement through the same owners.
- CALL-CRED-04: Distinguish pre-dispatch rejection from a response rejected after an already-dispatched mutation. For an admitted create/event POST whose body crosses a credential change, reject stale local settlement without claiming the remote effect was undone. Restoring the original authorized account must use existing sync/discovery/replay to recover the authoritative effect. Preserve ordinary valid create/discovery/lifecycle/ICE behavior and the local no-Native path.
- CALL-CRED-05: Use bounded response barriers and the real public gateway, Native client and SQLite; a mere error result is insufficient. Cover wrong expected account before transport, complete response-body field changes, zero follow-on mutation, unchanged durable state and restore/retry. Preserve the frozen CALL-SCOPE-001 guard, its five tests and the retained 3+1 call regressions.
- CALL-CRED-06: For a signal event, validate payload.senderDeviceId against the same private credential snapshot that supplies the outgoing device header, after expected-user admission and before POST. Missing or mismatched sender identity must produce zero HTTP. Preserve matching-sender signaling and existing response-body fencing; an earlier Production identity sample alone is insufficient.

No new account, credential, call, participant, persistence, permission, UI or lifecycle owner is authorized. Existing native bound-user and six-field helpers should be reused. Compatibility wrappers may retain existing Rust callers while production uses explicit expected-Human variants; no wire schema changes. Root integrates production composition/call sites, canonical row/RTM/workflow and shared docs. Implementation must not touch messaging/forwarding/friends/upload code or concurrent owners.

## Batch boundary and verification

Base production batch is 8bcfd464691534acaa04b13852cd523ab49a3dcc on checkpoint/tdrp-native-call-scope-20bf9475-r1, PR191 targets the existing migration checkpoint. Freeze the new call credential delta and independently review before publication. Apply only the call-family/native and call-site differences; do not promote stale full files from the shared checkpoint.

All compilation, schema checks and behavioral execution occur only in GitHub Actions at exact H. The original call-scope batch remains a separate recorded execution subject while this independent repair is authored. Add precise selected cases/logs/artifacts to the existing native-service batch. Record implementation, authored tests, execution and acceptance separately in the existing CALL-SCOPE-001 account/peer responsibility and RTM; do not create another inventory or claim a full source read.

AC-CALL-CRED-01: wrong-account predispatch produces zero transport for each shipping family. AC-CALL-CRED-02: held-body rotations produce no follow-on event POST and no stale durable settlement; same-account recovery passes. AC-CALL-CRED-03: valid call behavior, prior scope negatives and existing call lifecycle gates pass at the containing H; source review and actual execution are independently identified.

Root scope decision, before implementation: **FIX** the independently identified signal device-snapshot gap within the same Native call-family boundary. Production checks senderDeviceId against an earlier identity read; Native append then rereads credentials. Rechecking payload.senderDeviceId against the exact credential used for POST closes that specific device mismatch without a new public argument, schema or authority. Add a zero-HTTP negative and matching-sender positive. This does not claim all call/device/transport-lease races resolved.

## Limits and rollback

This does not promise atomic revocation of an HTTP request already dispatched with an originally admitted credential, compensate remote effects, or close all media/device/network/concurrency acceptance. Preserve existing service authorization and normal replay recovery. No schema/data migration or deletion is allowed. Ordinary code revert is possible but reopens the credential-lifetime defect and does not authorize release. Whole-call, deployed real-service, platform/UI/accessibility/security/performance/soak/upgrade/Bot and independent migration ACCEPT remain open. Fixed U remains f985723b552b30fe1c17d377d57f44e61a15a8c4.

