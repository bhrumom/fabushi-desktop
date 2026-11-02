# Human call explicit same-device media rejoin

Spec ID: FBCP-CALL-MEDIA-REJOIN-001
Status: active; implementation and execution pending
Parent responsibility: FBCP-CALL-MEDIA-CAPTURE-001
Task: 20bf9475-a885-4314-b509-bea53ef8849f, round 1
Date: 2026-11-02
Fixed upstream: f985723b552b30fe1c17d377d57f44e61a15a8c4
Inspected target: 8f4b1943b4043e6a9da7051d30ff1cefc04b39b2

## Existing production gap and owners

ProductionRenderer mounts the existing HumanCallControls in the Human conversation header. The capture-lifetime implementation is blob45641d86e7c2c5adcd0fa7b70609f3a0e28bff78. A fresh instance can reconcile an already negotiating/connected/reconnecting call and an owned lease, but localStreamRef and pcRef are null. Poll only configures an existing local stream; mute/camera/share return early. Start is hidden/refuses an active call, Accept is only invited/ringing, and automatic reconnect requires a peer event. The user sees an active call without an action to restore its media.

Reuse HumanCallControls for the explicit capture action, DesktopBridge/Electron for permissions/preferences, and Host Session/CallSessionStore for existing durable call state, current Native lease and lifecycle. No new module, store, public command, schema, automatic device takeover or display-capture restoration.

Pinned Host productionb2daaab48b10f11580534b11b5b423e0a5016f4d reads authenticated lease state from Native and checks local participation/remote identity/cursor. CallSessionStoref97f628ec14618ab2788bdf5b5c301f7a00c81f6 allows reconnect from negotiating/connected/reconnecting, increments generation and resets signal sequence; resume requires reconnecting and retains generation. Existing local restart and Native ringing/reopen tests are useful prerequisite evidence, but do not prove this renderer rejoin path.

## Requirements

- REJOIN-01: Offer explicit voice/video rejoin in the existing active-call controls only for negotiating/connected/reconnecting with no local media/peer and no competing acquisition. Merely mounting, polling or switching conversations must not capture microphone/camera. Preserve all original capture controls and fourteen regression cases.
- REJOIN-02: Read current authoritative call and require a fresh strict same-device owner lease before permission/capture. An unclaimed, foreign-device, malformed, unavailable, terminal or mismatched call/generation lease cannot authorize capture or lifecycle/signaling/media mutations. Do not treat claimAvailable as permission to take over. Recheck current lease/identity/generation after delayed capture and before remote mutation; existing Host/service admission remains authoritative.
- REJOIN-03: Acquire fresh audio/video through existing permission/preferences/fallback policy. Do not infer local camera or display intent from global media metadata. Use canonical reconnect then resume to establish a fresh negotiation generation; reset local signal cursor and configure the current peer through existing role-specific offer/answer paths. Preserve the same call ID and conversation; never create a duplicate call.
- REJOIN-04: Give rejoin a current lifetime/call/attempt boundary. Serialize double activation and polling so they cannot build competing peers or publish an older snapshot. End, remote terminal, scope/client/generation replacement, lease loss and stale success/rejection invalidate pending work. Stop all newly returned/unadopted tracks without affecting a newer owner and emit no stale follow-on command. Already admitted remote work is not rolled back.
- REJOIN-05: Permission/capture/ICE/service failure releases this operation's physical media and leaves a retryable current call surface instead of failing an otherwise active remote call solely because local acquisition failed. A retry reads canonical state/lease again and accounts for a reconnect/resume already accepted before a lost response. End remains usable while acquisition is pending. Do not hide current errors or revive a terminal call.
- REJOIN-06: Verify the actual mounted component with the existing Chromium harness and native synthetic tracks, plus a narrow actual gateway/HTTP/SQLite rejoin-and-reopen contract. Keep test authorship, execution and independent assessment separate. All executable checks occur only in GitHub Actions at frozen containing H; no local or htch-runtime validation.

## Acceptance cases and boundaries

AC-REJOIN-01: Fresh instance over preserved connected/negotiating/reconnecting state performs no automatic capture; explicit voice/video rejoin for the owning device obtains fresh tracks, the same call ID and the current new generation. Creator sends an offer; peer consumes an offer/produces an answer. Old-generation signals are not reused.

AC-REJOIN-02: Foreign/unclaimed/malformed/unavailable or mismatched lease/call denies before capture. A changed lease after held capture releases returned tracks and prevents follow-on lifecycle/signaling/media work. No takeover or cross-scope/new-call effect.

AC-REJOIN-03: End/terminal/unmount/client/scope/generation replacement and double/poll overlap preserve one current media owner. Late successful or rejected old operations cannot mutate or fail a newer owner. Permission/capture/ICE and admitted-remote-response failures leave bounded retry that rereads durable state and uses no duplicate create.

AC-REJOIN-04: Actual Session gateway and Native HTTP fixture reopen the existing CallSessionStore across rejoin, preserving call ID/scope/participants and applied generation/event cursor, with stale-generation rejection and accepted-effect replay/retry. Browser remount, in-process Host reopen and synthetic peer behavior are separate bounded evidence, not a packaged process-crash/hardware/real WebRTC claim.

## Scope, source and validation

UI author may change only human-call-media.tsx and its existing fixture/behavior tests in an isolated copy of exact8f4b1943. Host test author may add only source/host/tests/native_call_rejoin_contract.rs; no Host production or shared schema change is pre-authorized. Root owns workflows/spec/dossier/owner/RTM/canonical ledger and remote integration. Final code/tests freeze and distinct independent review precede publication; root records FIX/REJECT/DEFER/COMPROMISE before finding-driven edits.

This extends the existing capture responsibility's recovery path, using its seven bounded fixed-U source identities and existing Host lifecycle contract. It does not assert that Telegram restores an active call across process restart or complete any whole-file source review. Record the target-specific recovery adaptation and actual shipping caller in the existing dossier/RTM, with no second inventory/ledger.

Use the current locked renderer/Chromium lane for all retained and new component cases and a minimal Ubuntu Host lane using existing locked Rust/dependencies and test harness. Only affected inputs require new execution; prior exact523 Native credential/scope evidence remains explicitly historical. Preserve all current source/schema/accepted-entrypoint gates and original Bot/product capabilities.

## Remaining final gates and rollback

No user data or public schema migration/deletion. An ordinary revert removes this recovery entrypoint and reopens the gap. Deployed service lease arbitration, physical devices/OS permissions, real ICE/TURN/WebRTC/media quality, signed supported packages, full account-epoch dispatch, hard process-crash/power-loss, complete UI/visual/accessibility/temporal/security/performance/soak/upgrade/rollback/Bot and independent migration ACCEPT remain required.
