# Device pairing revocation and retained-stream continuity

Status: specification recorded before implementation; production and contract cases authored, awaiting independent frozen-scope review and exact-H Actions. No execution or acceptance claim.  
Contract: DVR-v1, requirements DVR-01 through DVR-10.  
Task: OCRM-P0 / 647e0932-bea4-4113-9a61-65ae3db58955.  
Source: `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`.  
Starting target: `bhrumom/fabushi-desktop@b9953fced2714c45b70ef47b4ffb42a261c190db`, tree `9835448cfd6c24b87536c0d3aa945bb00abfd8f2`, existing migration branch `codex/openclaw-rust-fusion-chat-20261101-writer`. DPA owner9/HTTP8 and DPR20/DJC8 have direct passing evidence at recovery H `31cac7c95ec0297f7dee72d4bc40dad56aee4f01`; its overall campaign failed the three existing wiring oracles identified in DVR-006. The independent fixed-U reading-range correction and checkpoint writer are preserved from integration base `5f8c5ab76fee0cef28ebe43dc7250a21e5eec0b5`. These prior-H results do not verify DVR or replace the new complete subject checks.

## Problem, ownership and scope

The real native `device.pair.revoke` entry drops its authenticated principal before `DevicePairingOwner::handle_pair_revoke`. An ordinary operator/write pairing can remove another pairing or its own mixed-role pairing. The method deletes the entire paired row even when a `role` receipt label is supplied. Its existence read precedes a deferred transaction, hides read failure as absence and does not check the delete count. The established native whole-pairing meaning must be preserved.

The existing `/events`, `/local-exec/requests` and `/webauthn/requests` loops authenticate once. Deleting or replacing a credential does not retire a retained connection. On a secret-free listener an explicit invalid bearer can currently become ambient authority on entries other than the protected DPA/Memory/channel methods, including internal MCP event access.

The existing Rust pairing owner remains the only authority for credentials, approval, mutation and SQLite state. `GatewayApi` carries its sealed DPA proof; `UnifiedGatewayApi` delegates to that same durable owner. `GatewayServer` retains the current stream loops, event bus, bridge subscription and drop cleanup. No permission database, subscription registry, timer service, scheduler, Agent loop, session store, runtime or schema is added. The existing module-map responsibilities M03/M04/M13/M32 and RTM remain the project owners.

Closed loop: paired HTTP request → Gateway authentication → sealed owner proof → bound App hook → owner transaction → paired row/revision commit → native reply and node projection; existing stream/bridge consumer → same currentness authority → existing receiver/subscription drop. The final reply to a successful self-removal remains deliverable.

## Required behavior

### DVR-01 — Native semantics and atomic deletion

`handle_pair_revoke` and the new bound paired method share one private deletion operation. Keep `deviceId`, optional native `role` receipt label, `revokedAtMs` and `revision`, including native whole-row deletion when a role is supplied. Keep direct/native configured-secret callers and public request/status/join behavior.

Hold the existing owner mutex, begin `IMMEDIATE`, then read the caller/target, admit policy, delete exactly one paired row, increment the existing checked revision once and commit before success. An authorized missing target is 400; SQL/commit/corrupt-revision/exhaustion failure is 500 with redacted fixed diagnostics, no success and no partial deletion/revision update. Repeated absent deletion never bumps revision. Other paired rows, role credentials, pending rows and unrelated configuration remain unchanged. Reopen must observe the committed deletion or the complete rollback. Do not replay an unknown outcome.

Deleting the paired row removes all its role tokens and retained/pending node-surface columns. Native pending repair requests remain unchanged, as established by the native contract. Approval of a surviving repair may later create a new pairing; removal is not a permanent ban.

### DVR-02 — Current sealed authority and management policy

Reuse the private DPA issuer/principal/role/token-fingerprint/admitted-grant proof. Factor the common current-row/exact-token check inside the existing owner without changing DPA rotation policy or exposing a proof constructor. Do not trust mutable public projection fields, body caller/admin assertions or replacement credentials. An owner-instance mismatch, absent pairing/role/token, replaced/revoked credential or approval that no longer covers the live credential is 401. The selected token entry's missing or wrong-type identity/token field retains DPA's 401 classification. Other needed malformed stored authority, including decisive primary/approved/extra-entry role fields, is 500 with no raw JSON/token/fingerprint disclosure.

For paired removal, inside the same write transaction require a current operator credential with `operator.write` through the intersection of sealed admission, live token grant, current approval baseline and an optional singular `x-sand-scope` ceiling. Reuse DPA's fixed-U operator implications: admin implies write; write does not imply admin. A blank ceiling is 403. Current node/other roles and read/empty/unrelated grants are 403. A stale proof is checked before its role-policy denial. This includes checking the complete live grant against approval using the sealed role's own implication rules before the operator-only refusal: unchanged node/observer credentials whose approval narrowed are 401, malformed decisive grants/baselines are 500, and restored current nonoperators remain 403. The shared ordering also fulfills existing DPA-02/03; no role gains management authority or bypasses its caller/header ceilings.

Non-admin may remove only its own operator-only pairing. Deny cross-device before reading the target, including an unknown target. For self-removal inspect the stored primary role, approved roles and every token entry's role, including revoked entries; a non-operator role anywhere requires admin. Missing/malformed decisive role fields are corrupt authority, not permission. An optional paired `role` must be a nonempty string and is only a receipt label: `role=operator` never narrows the whole-row effect or hides mixed authority. Admin may remove any target without needing to parse that target's corrupt token payload; caller authority is still fully validated. A write ceiling removes admin from an admin credential.

### DVR-03 — Bound dispatch and credential classification

Add `GatewayApi::revoke_paired_device(args, authorization, scope_ceiling, context)` with a default 403 refusal. Recognized paired removal never falls back to generic native dispatch. `UnifiedGatewayApi` invokes the existing owner and updates desktop contact once for the command, retaining the common report/result flow and request context. Existing node binding and native method guards remain.

Classify all protected ingress consistently: configured native secret, recognized paired token, or credential-absent native mode on the actual loopback listener. An explicit malformed/blank/unknown/removed bearer must never become ambient native authority. A `Host` header cannot turn a wildcard listener into loopback. Public health/request/status/join exemptions remain public subject to their existing origin/Host/method constraints; public exemption does not create authenticated authority. Preserve configured-secret and credential-absent loopback positives, origin checks, replay fences, header ceilings and all native-only MCP/attachment controls.

The generic literal-write check exempts the bound revoke operation just as it exempts bound rotation; the mutation owner applies current implications atomically. This does not authorize unrelated pairing list/approve/reject/setup operations or claim their full parity.

### DVR-04 — Retained credential admission

Add owner `validate_current_device_authorization(&DeviceTokenAuthorization) -> Result<(), GatewayCommandError>` and fail-closed `GatewayApi::validate_paired_device_authorization` forwarding in the App. It validates the original sealed credential against the same owner and current database without modifying last-seen, revision, desktop activity or any data. It supports existing operator/read, operator/write/admin and non-operator/node stream users; it does not require management write permission.

The complete live grant must still fit current approval, and the sealed admitted grant must still fit both the live grant and current approval, using the role's existing implication rules. Narrowing a retained grant closes that stream; expansion does not promote its sealed admission. SQL NULL approved baseline retains the DPA legacy fallback to stored scopes; malformed present baseline fails. Original proof replacement is never silently refreshed. Foreign owner/reopen proofs refuse; freshly authenticated credentials on the reopened owner work.

### DVR-05 — Existing event-stream lifetime

Carry the original optional paired proof through `serve_events` and `serve_event_body`. Check it before subscribing, again after subscription immediately before HTTP headers, and after every existing `recv_timeout` wake before filtering, slimming, serialization or any new application frame/heartbeat. Also check stop after wake. This includes filtered events and internal MCP exclusions; a filtered flood cannot starve retirement. Check before the initial retry frame as well. A stale/error result ends the connection permanently. Before HTTP headers, return the normal redacted error; after HTTP 200, close instead of injecting a second HTTP response.

Use the existing 15-second receive heartbeat and five-second write timeout. Quiet retirement is detected at the next heartbeat plus database/scheduling and any already-admitted I/O delay. Do not add an unbounded busy loop, polling thread, per-device registry or SQL lock across network write/flush. Preserve native streams, channel selection, slimming, gzip and paired exclusion of both Host MCP and Runner request channels.

The linearization boundary is the currentness admission, not completion of a network write. A deletion committed before that admission refuses the next frame. A frame admitted before deletion may finish afterward. Buffered bytes and gzip footer may flush during close. No zero-bytes-after-COMMIT claim is made. Rotation retires the replaced role's proof while leaving other current roles/devices live.

### DVR-06 — Existing bridges and response admission

Apply the same optional proof and wake/currentness rules to both bridge request streams. Validate before real subscription/provider registration, again after registration immediately before HTTP headers, and before its initial retry/frame. A revocation committed while registration is blocked therefore returns 401 before stream headers and drops that registration exactly once. Dropping `GatewayBridgeSubscription` invokes its existing close callback once and the actual registered provider is removed; no new cleanup owner is added. Revalidate a paired response immediately after decoding and before `submit_responses`, with the same admission boundary. Native responses keep their established path.

No claim is made to cancel a process, WebAuthn ceremony or other effect already handed to an independent execution owner. Cleanup closes this connection's owned request registration. HTTP close/retirement cannot revive a stopped connection or substitute a successor token. Server close joins only its accept thread: tests must await their connection/API/provider cleanup before deleting a database or reopening an owned fixture.

### DVR-07 — Real App consumers and node projection

Wire both new hooks on the actual `UnifiedGatewayApi`. Reuse the production helper for node list/status to prove removal makes a stored remote node disappear/become unknown while the local `sand-local-node` remains. A copied JSON projection or a thin HTTP test adapter is not evidence of shipping App wiring. Preserve the established DPA contact count and command reporting; retained checks do not produce user activity.

The current production composition only connects the local node. Pairing-bound remote transport/worker sessions, wake/APNs cleanup, worker cancellation and companion clients need their own later existing-owner integration and real-device acceptance. Do not introduce a fictional remote connection or claim physical device teardown from these HTTP tests.

### DVR-08 — Direct failure, concurrency and recovery oracles

Owner tests use real file-backed SQLite and independent read-only observations of all paired/pending rows, token/surface data and raw revision type/value/timestamp. Cover self/operator-only and admin-only positives; cross-device and mixed-role self negatives; body/header forgery; foreign/altered/stale proofs; permission/approval narrowing; malformed data; authorized unknown target; native role-label compatibility; revision corruption/exhaustion rollback and reopen; unrelated device/pending preservation. Test a separate database connection where it falsifies transaction behavior.

Actual TCP tests call the bound real owner. Cover default-hook refusal, absence of paired generic fallback, context/reporting, listener/credential/header negatives and native/public positives. An after-authentication barrier must allow removal/replacement of the caller before owner admission: result 401 and no target/revision change relative to the post-retirement snapshot. Include live/unrelated-change controls and concurrent self-removal (one success, one stale result, one increment). Preserve DPA/DPR/DJC/DPS and affected Gateway regressions.

Stream tests use real TCP/event bus/bridge subscriptions, bounded readiness and controlled owner-admission barriers. Removal before a queued frame's check produces EOF without that payload; a frame admitted first may finish and the next refuses. Cover live/unrelated/native, operator/read and node, role replacement, narrowed approval, storage refusal, plain/gzip/filter/MCP controls, old/malformed reconnection with/without configured secret, and at least one actual quiet-heartbeat retirement. A timeout alone is never a passing closure oracle. Provider registration/drop must be observed exactly once. Preserve errors as evidence and close/join only owned resources.

### DVR-09 — Verification and review

Freeze the complete batch on its exact integrated H. All executable checks run only in GitHub Actions through the existing runtime/source workflows and caches. Select complete new contract targets, each required unique positive name, exact counts, zero command and capture exits, source H/tree/input hashes and original logs/artifacts. Keep the existing strict DPA 9/8, DPR20, DJC8, shipping DPS7 and other affected App/Memory/channel/MCP/Automation assertions. Do not borrow an old H verdict or mock real provider/device/platform acceptance.

The integrated App regression selection also retains the three existing routing/continuation/node wiring obligations whose old text assertions failed on `31cac7c95ec0297f7dee72d4bc40dad56aee4f01`, Actions `41550569321`, artifact `13192016490`. DVR-006 records TEST_FLAW before repair: inspect the actual abort delegate and its existing route-validation/run-filter helper, both continuation error and exact-stream cancelled-result cleanup with review-custody release before session-admission release, and the same LocalExec owner's timeout-aware shell call with `timeoutMs`. Preserve the remaining capability/generation/result/payload assertions and add falsifiable source-mutation controls for changed delegation/order/timeout wiring. These source checks supplement actual owner/App tests; they are not live provider/node acceptance. No production behavior is changed to satisfy stale text literals.

Independent security/lifetime and structure/reuse/performance reviews inspect immutable full scope plus adjacent callers. Adjudicate findings before fixes and obtain fresh focused review after fixes. Reconcile concurrent branch updates without overwriting uncommitted work. Source review, authored cases, executed cases and scoped source authority remain distinct; no unit is called verified without the required scoped authority/coverage and direct behavior evidence.

### DVR-10 — Release and remaining acceptance

No storage schema/dependency/UI migration is required. Existing native APIs continue using the same SQLite owner. Reverting restores the vulnerable admission/lifetime behavior, so a rollback requires explicit security assessment; it does not alter the persisted pairing schema. Observe normal typed command success/error reports without tokens, sealed fingerprints or raw authority data. Unknown effects remain unknown and are not automatically replayed.

The narrow batch exits only after production wiring, its exact-H affected checks and independent review have direct evidence. Whole OCRM remains incomplete until every applicable source/dependency/legal responsibility, G01–G12, AC01–AC24, real ecosystem/device/UI/platform/soak/upgrade/rollback and independent packaged ACCEPT is satisfied. All eight global flags remain false.

## Source provenance and deliberate compatibility boundaries

The fixed-U device-management handler/authz, token kernel/policy/identity/operator-scope sources were deeply read in DPA and remain unchanged. Additional DVR sources and actual reading extents are preserved in the task's `reviews/paired-revoke-upstream-source-pack.json`: worker remove dispatch and state transaction, client invalidation/deferred close, node runtime cleanup and worker reconciliation, with exact blob/content identities. `device-pairing-core.kernel.ts` removal and `worker-environments/device-provider.ts` reconciliation were read by ranges, not relabeled as full reads. APNs helper and full worker termination internals remain unreviewed for this batch's unimplemented remote/APNs responsibility.

U `device.pair.remove` also deletes pending repairs and APNs registrations; native `device.pair.revoke` retains its established pending rows. U `device.token.revoke` is a separate role-token operation. This batch does not claim those wire aliases, pending/APNs effects or every upstream client lifecycle are now equivalent. Its behavior adapts source management/currentness requirements into the existing native HTTP/SSE owners. These explicit gaps remain applicable future work, not out-of-scope exemptions for OCRM.

References: root `AGENTS.md`; OCRM SOT, main Spec §17, module map and P0 task; `device-pairing-authenticated-rotation.md`; `device-pairing-revision-admission.md`; `device-pairing-startup-durability.md`; `device-pairing-join-admission.md`; actual `source/host/src/{device_pairing_owner,gateway_server}.rs`, `source/host/app/src/main.rs`, event bus and local-exec/WebAuthn provider owners.

## Exact-H ordinary-route oracle recovery — DVR-CI-011

On product H `8bdcf1012c15c08a0320c85c9d4d42e1cc100949`, runtime
`41559093519`, Linux job `125665070447`, original artifact `13196816131`
retains 38 accepted portable groups out of 39, including complete DVR26/8/14,
DPA9/8, DPR20, DJC8, DJL11/7/2/1, generated-history9 and the PRC wiring
controls. The integration group has two failed ordinary-route controls:
`channel_gateway_registry_origin_and_memory_controls_remain_intact` and
`memory_gateway_preserves_projection_selectors_and_unrelated_routes`.
Both receive HTTP401 `unauthorized` where their helpers expect200.

Classify **TEST_FLAW / FIX before editing**: both source callers explicitly
send `Authorization: Basic ignored` to protected `getTranscript` under an
unconfigured loopback listener and expect native success. That expectation
contradicts DVR-03's already-adopted explicit-credential refusal. The original
log records helper panic lines (Channel306 and Memory338), not a full stack
identifying the source-diagnosed caller lines (Channel619 and Memory577).

Retain the real paired-node ordinary-route positive, add the existing native
configured-secret/credential-absent positive in both configuration modes, and
retain the explicit Basic request as a401 negative through each fixture's
existing denial helper. Preserve no-dispatch, device-binding, original bytes,
Memory/credential projections, private-content and valid-after-refusal checks.
Keep all downstream origin/Host/registry/config/native-only controls, selector
and100-row-cap assertions, test names, command selections and strict counts.
Do not weaken production authentication to restore the obsolete expectation.

The original failed artifact and its SHA-256
`365c7162b207d4814730925507829544b5cb641ca78ffef61bd3e802808e545b`
remain evidence for H. New execution uses a separately frozen exact H after
fresh focused review. The later native link-preview-step failure is separately
resolved as DVR-CI-012 below; skipped App/Automation and queued macOS receive no credit.
Source structural and renderer positives do not make this failed runtime pass.

## Exact-H wildcard-listener oracle recovery — DVR-CI-012

The same H/run/job's original final artifact `13196766905`, SHA-256
`697697eaa5cec007e39b7c4430382505056d5b3bb1dd0aeadaacc2170f60d15e`,
preserves Host source checks9passed and Gateway41passed/2failed. The failing
`native_box_mcp_nonloopback_configuration_requires_authentication` and
`native_link_preview_nonloopback_configuration_requires_authentication`
expect native-only403 where unauthenticated wildcard ingress now returns401.
Their captured helper panics are Gateway684 and1334. Both also contain later
unexecuted ordinary-route200 expectations under the same unauthenticated
wildcard configuration; those source-diagnosed follow-ons are not additional
captured failures.

Classify **TEST_FLAW / FIX before editing** under DVR-03. Require fixed401
`unauthorized` for protected native-only and ordinary methods on that listener,
preserving no-owner-access and unchanged real cache/file witnesses. Keep
configured-secret native positives and the authenticated paired/header-limited
native-only403 negatives elsewhere. Close the denied wildcard listener, then
start an actual trusted loopback listener around the same API/service/cache;
require native and ordinary positive results there without replacing the owner.
Keep all43 Gateway tests and their remaining assertions, existing commands,
source checks and strict receipt conditions. No production change is required.

The original Coordinator58/14/1 and filtered actual link-preview App3 positives
remain attached to H. The missing shared-MCP consumer receipt follows skipped
Automation; it is a dependency failure, not a separately demonstrated product
defect. Main App81/DPS7 and Automation require the corrected candidate's
execution. No local/htch reproduction or unchanged-subject rerun is authorized.
