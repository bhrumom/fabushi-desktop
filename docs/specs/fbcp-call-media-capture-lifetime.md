# Human call media capture lifetime

Spec ID: FBCP-CALL-MEDIA-CAPTURE-001
Status: implemented; exact8f4 renderer/14 component cases passed; canonical source membership and full acceptance remain gated
Task: 20bf9475-a885-4314-b509-bea53ef8849f, round 1
Date: 2026-11-02
Fixed upstream: f985723b552b30fe1c17d377d57f44e61a15a8c4
Target inspected: 523c8ca4883e6eaa10d1d19a0e237f9d3964b3c6

## Current production defect and scope

The mounted existing HumanCallControls in frontend/src/production/human-call-media.tsx (blob8e5c4cbe5b3c9fb7f0d51d43300ccdeb2d2ac99e) owns local/remote streams and the peer connection. Display capture is acquired using the current Electron desktop constraint through getUserMedia after callMedia.listDisplaySources; it is not getDisplayMedia. Its stream is only a local variable, so Stop sharing changes metadata without stopping the capture or restoring the retained camera. Failure, decline, hangup and effect cleanup do not own that display stream. Late permission/preference/capture completions can install media after cleanup or a conversation change.

The existing component is reused across conversations. Its boolean disposedRef is reset by the next effect setup and cannot identify the originating lifetime. configurePeer depends on cameraEnabled while the cleanup effect depends on configurePeer, so ordinary camera state changes can tear down the entire media owner. Fix these causes together inside the current component. No new call, permission, resource, UI, credential or Coordinator owner is authorized.

## Requirements

- CAPTURE-01: The current HumanCallControls lifetime must explicitly own each acquired local/display stream, remote stream, peer connection and display sender. Release tracks and handlers on replacement, decline/end/failure, unmount, client/conversation replacement and remote terminal/no-active-call reconciliation. Local release must not wait for a remote transition response. Do not stop media belonging to a newer lifetime.
- CAPTURE-02: Use a private monotonic lifetime/capture-operation identity, not a reusable disposed boolean. Across permission, preference/source selection, media acquisition, ICE and sender replacement awaits, a stale continuation may not adopt media, create/install a peer, publish into the new scope or send follow-on media/signaling commands. If a stale capture has already produced tracks, stop every track immediately. Preserve existing Coordinator APIs; this does not establish a Coordinator pending-dispatch account epoch.
- CAPTURE-03: Stop sharing must stop the owned display stream and restore the same retained camera track to the held video sender, or replace it with null when no usable camera exists. The device-ended path uses the same cleanup boundary. Missing video, capture denial, replacement failure and repeated stop must not leak media or leave unhandled promises; a stale old ended callback cannot stop a newer capture. Remote metadata failure does not keep physical capture running.
- CAPTURE-04: Stabilize the lifecycle/polling effect against ordinary camera, mute and sharing state updates. Starting/accepting video, toggling camera and starting/stopping sharing must preserve the active call and unaffected microphone/camera tracks. Keep existing UI, user-selected device preferences and the recoverable stale-preference fallback; no new selection policy or route.
- CAPTURE-05: Serialize or supersede overlapping acquisition through this same owner. End/decline/disposal must invalidate pending acquisition immediately. Restore/retry after denial or ended sharing must use the same production entrypoints and release replaced media exactly once where ownership changes. Do not claim recovery of a remote mutation that was already dispatched.
- CAPTURE-06: Exercise the actual mounted component in locked Chromium with the public button/bridge/Coordinator surfaces, controlled asynchronous boundaries, real browser MediaStream/MediaStreamTrack objects and inspectable sender behavior. Cover ordinary accept/camera/share/stop/re-share, voice-only restoration, ended event, capture/replacement/metadata failures, late permission or capture after end/unmount/scope switch, and camera state not resetting the call. Assert physical track states, sender identity, no stale follow-on commands and preserved newer-lifetime state; error-only assertions are insufficient.

## Source and owner responsibility

The fixed-U calls_call.cpp capture owner explicitly deactivates and resets screencast capture on stop; calls_instance.cpp binds the Call lifetime to its account session. Call-level successful camera/answer continuations use crl::guard(this), with an answer-state check; Instance permission denial acts on the current Call. These observations do not establish that every deferred callback remains bound to its originating operation or Call across reuse. Exact paths/blobs/ranges and dependency-interface limits belong in the existing call dossier. Automatically restoring Fabushi's retained camera is a target-specific continuity rule, not a claim about Telegram behavior.

The canonical ledger currently has a call proxy-consumer responsibility and the selected-peer/account CALL-SCOPE responsibility; neither owns acquisition/attachment/release. Add a distinct FBCP-CALL-MEDIA-CAPTURE-001 responsibility in the one existing ledger after the scoped source/owner review, retaining all current rows and source-review/acceptance state. This adds traceability responsibility under existing owners, not a new module or ADR. Root owns schema/workflows/ledger/RTM and integration.

## Batch and validation

Allowed production edit: frontend/src/production/human-call-media.tsx. New test harness files may be frontend/src/production/human-call-media.fixture.tsx, frontend/src/production/human-call-media.behavior.test.mjs, desktop/human-call-media-test.html and desktop/human-call-media-test.ts. Preserve production renderer mounting, Coordinator command/event contracts, device bridge contracts, Host call code, other UI owners and the current lockfile.

Freeze source/tests and obtain distinct independent review before publication. Root must record any FIX/REJECT/DEFER/COMPROMISE before fixes and request focused rereview. All executable validation occurs only in GitHub Actions at containing exact H. Add a minimal dedicated call-media Actions lane using existing locked desktop dependencies, actual renderer compilation and locked Chromium; preserve existing Native/source gates. Reuse immutable previous results only for their declared inputs and exact execution subjects, without attributing old runs to the new code.

AC-CAPTURE-01: Stop/end/dispose and ended events release physical display tracks and restore/clear the sender while preserving current unaffected media.
AC-CAPTURE-02: Late asynchronous completion cannot install a stream/peer or emit follow-on commands into a disposed/replaced lifetime; returned stale tracks are stopped and the newer call remains intact.
AC-CAPTURE-03: Current valid start/accept/camera/sharing/retry behavior succeeds through the actual component, with no camera-state-triggered owner reset and bounded failure cleanup.
AC-CAPTURE-04: Exact-H renderer build/browser cases, scoped upstream evidence, production wiring and independent review are recorded separately; source, product and final release acceptance remain unpromoted.

## Limits and rollback

No user data or schema migration/deletion. Ordinary revert restores old code but reopens the capture-lifetime defect and does not authorize release. These deterministic browser contracts do not prove physical device/OS permission behavior, real WebRTC/ICE/TURN or remote media quality, account-epoch dispatch cancellation, supported packaged platforms, visual/accessibility completeness, performance/soak, upgrade/rollback or full Bot/migration ACCEPT. Those original final gates remain required.
