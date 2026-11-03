# Device role-token revocation

Contract **DRT-v1**. Fixed U is `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`. Implementation base is `fc4ce17977cde16e10d471287389ca24a5fd78d6`, tree `e926c102533da83a5b26b159aa408ae690e44151`; the first preimplementation spec was adopted on `b84a6d982ce88c5a01e63913763e6d2b4e847369`. Predecessor product H is `5593e6d96e849ce5bf93de299aedeffca70b9568`, tree `17ddcd3b19126ece3cadb722ea7b9bbdc5dcb561`; incoming media/Codex workspace implementation and evidence are retained. This specification precedes implementation. The predecessor's failed/skipped shared Linux gates (portable contracts, full Gateway, actual App/startup and Automation) recovered on Actions run `41560916684`, attempt 1: original artifacts `13196814478` and `13197111861` bind the 39 portable groups, 43 Gateway cases, 81 actual App cases, 7 startup cases and typed Automation oracles to that exact H. This evidence admits dependent implementation; it does not validate the incoming Codex changes or this new batch. Historical queued macOS and missing external-device acceptance remain non-passing final evidence without preventing later implementation under main Spec §17.

## Problem and user behavior

The existing Rust owner can rotate a role credential and remove an entire pairing. It cannot retire just one role credential while preserving that device's pairing and other roles. Fresh bearer lookup also ignores a stored `revokedAtMs`, although retained-proof validation already refuses inactive markers. Exposing a role revocation RPC without repairing fresh lookup would leave the retired bearer usable on ordinary commands and reconnects.

Implement the fixed-U public method `device.token.revoke` through the existing Gateway, App and `DevicePairingOwner`. A permitted operator or native administrator can retire one existing role credential. Its fresh and retained requests lose authority; unrelated roles/devices and the local node continue to work. No second permission store, runtime, session registry, scheduler or cancellation owner is introduced.

This batch does not complete all device equivalence. Remote pairing-bound transport, worker/placement teardown, wake/APNs cleanup, physical cancellation, Gateway v4/N-1 compatibility and real-device/platform acceptance remain applicable work. Existing native `device.pair.revoke` keeps its whole-row semantics and pending-row behavior. `device.pair.remove`, lifecycle status reconciliation and token-rotation wire aliases are separate responsibilities.

## Production ownership and shared interfaces

`POST /api/device.token.revoke` uses the existing public-method registry and authentication/Origin/Host/replay fences, then a fail-closed Gateway hook, the actual `UnifiedGatewayApi`, and one `DevicePairingOwner` SQLite transaction. Existing fresh authentication and event/LocalExec/WebAuthn retained consumers observe the same selected token marker.

Shared interfaces for this batch:

- `DevicePairingOwner::handle_token_revoke(&Value)`: established trusted-native direct dispatch with no header ceiling.
- `DevicePairingOwner::revoke_token_with_authority(&Value, Option<&DeviceTokenAuthorization>, Option<&str>)`: the same mutation with an optional real sealed paired proof and optional singular header ceiling. `None` is reserved for callers already admitted as trusted native; it is not a fallback after bearer failure.
- `GatewayApi::revoke_device_token(Value, Option<&DeviceTokenAuthorization>, Option<&str>, &GatewayCommandContext)`: default fixed 403; every dispatched HTTP command for the new method uses this hook, including native commands. No generic-dispatch fallback is allowed.

The actual App hook notes desktop contact exactly once and delegates to its existing pairing owner. Read-only retained validation records no activity. Add the method to `OPENCLAW_GATEWAY_METHODS`, `openclaw_gateway_owner`, operator/mutation classification, owner command dispatch and the App pairing branch. Native role headers must be absent or exactly `operator`. Paired identity and authority come from the sealed proof, never public projection fields or request assertions. The singular scope header must reach the mutation owner even for native callers.

## Requirements and acceptance oracles

### DRT-01 — Closed request and non-secret committed receipt

Accept exactly `{ "deviceId": string, "role": string }`. Both strings are trimmed and must remain nonempty; roles are case-sensitive. Missing/wrong-type fields, non-object bodies and any extra field are 400. In particular, `scopes`, `crossDevice` and caller/admin assertions cannot influence authority. This is a closed request, not a whole-pair revoke label.

After commit return exactly `deviceId`, `role`, `revokedAtMs`: normalized identities and the positive signed-integer timestamp actually stored. No token, fingerprint, scopes, raw row, delivery claim or new response revision is exposed. A nonpositive/unrepresentable timestamp cannot commit. The existing revision changes internally.

### DRT-02 — Current paired proof and method authority

Begin the existing owner's IMMEDIATE transaction before all decisive caller/target reads. Reuse the existing sealed issuer/device/role/exact-token/admitted-grant proof. Currentness includes the selected token, role approval and complete live grant against current approval, using the sealed role's implication rules before operator-only policy. Foreign, replaced, retired or withdrawn caller authority is 401; malformed decisive stored authority is redacted 500; a current nonoperator is 403. Preserve the existing SQL-NULL approved-scope fallback and DPA/DVR identity classifications.

This ordering governs atomic owner admission after existing HTTP guards. Preserve earlier role-mismatch, paired/node operator-only and explicit read-header refusals, including their 403 results and no-dispatch behavior. Retired raw bearers fail fresh authentication before those guards. Exempt the new bound method from the separate literal `operator.write` prefilter so pairing-only/admin-only operators reach the owner's pairing decision; no other method changes classification. Owner and HTTP oracles must distinguish those boundaries.

Generalize only the required scope parameter of the existing paired management admission. Existing DPA/DVR operations still require `operator.write` and preserve their established error strings; DRT requires `operator.pairing`. Admin implies pairing; write does not. Effective permission intersects original admission, live grant, current approval and the optional singular header ceiling. Blank is an explicit empty ceiling. No public projection or combined header can promote authority.

### DRT-03 — Target management and native ceiling

A nonadmin paired operator may retire its own operator token even on a mixed-role pairing. A different device or any nonoperator target role requires effective admin. Refuse unauthorized addressing before target lookup, so an inaccessible existing/missing target has the same refusal. Do not apply DVR's whole-pair operator-only-row rule to this selected-role mutation.

Native authority starts with the existing configured-secret or genuinely credential-absent loopback admission, intersected with its optional header ceiling in the owner transaction. It has no fabricated paired self identity. An absent/admin ceiling allows management; a pairing ceiling can retire an operator token whose entire grant fits that ceiling; nonoperator targets require admin. Write/read/blank/combined/unrelated ceilings fail the required pairing permission. Explicit invalid authorization never becomes native absence.

For an operator target, every normalized stored token scope must fit every caller ceiling, even on an authorized repeat. A valid excessive grant is 403. The selected entry's `scopes` must be a present array containing only strings; missing/null/scalar/object/mixed arrays are redacted 500, not an empty grant or device-scope fallback. An empty array is valid. Do not require target approval scopes to authorize retirement; withdrawal of that target's issuance baseline must not prevent a separately current administrator from retiring it. Nonoperator target scopes are not an additional issuance prerequisite after admin admission.

### DRT-04 — Existing target role and token only

Read only the target fields needed for selection and mutation. Require `roles_json` to be SQL TEXT decoding to a string array; SQL NULL, wrong storage type, invalid JSON/non-array or wrong-type members are 500. Normalize strings with existing trim/deduplication rules. Valid empty arrays contribute no roles.

Union that array with optional primary SQL TEXT `role`: SQL NULL/trimmed-empty contributes nothing, nonempty text contributes its role, non-text is 500. The selected role must belong to that union and have an existing token entry. Missing authorized device/role/entry is 400. Never infer approval from a stray token, default a missing role to operator, or issue a replacement token. This target-only union does not weaken current caller admission, DPA rotation or DVR whole-row checks.

Require a token-map object and a selected object whose role matches and token is a nonempty string; malformed required target storage is fixed redacted 500. Other entries need not be reinterpreted to retire this selected token. Preserve unknown selected-entry properties and all unselected values.

### DRT-05 — Atomic selected mutation and recovery

Set only the selected entry's `revokedAtMs`, preserving its token/role/grant/other metadata and every other entry. Preserve all other paired rows, pending requests and unrelated configuration. The changed JSON column may be canonically serialized. For selected `node` only, set both `node_surface_json` and `pending_node_surface_json` to SQL NULL atomically, including an authorized repeat; other roles preserve those columns byte-for-byte.

Require one updated target row, one checked existing revision increment and a successful commit before success. An authorized repeat rewrites the timestamp and increments revision again; two timestamps may coincide within a millisecond. A self-retired proof cannot repeat, whereas a separately current administrator can. Missing/corrupt/overflow revision, SQL/update-count errors and a known failed commit preserve the complete prior state. Reopen preserves either committed retirement or proven complete rollback; an old owner-issued proof cannot become current in a new owner. Existing explicit rotation/reapproval may issue a new credential without reviving the old bearer.

Client disconnect or response-write failure after the owner commits is not rollback evidence. Do not automatically replay, compensate by issuance, or claim confirmed delivery. Existing list/status does not expose exact role retirement receipts; uncertain outcomes remain uncertain until an explicit administrative action or future lifecycle readback resolves them.

### DRT-06 — One active-marker rule for fresh and retained admission

Reuse one predicate for fresh token lookup and existing retained/current proof validation. Absent, JSON null and integer zero markers are active subject to existing checks. Every other non-null marker is inactive, including nonzero/negative integers, floats (including `0.0`), strings (including `"0"`), booleans, arrays and objects. Preserve constant-time credential comparison and proof sealing. Do not substitute another role or successor credential, refresh the original proof, or write timestamps during validation.

A retired raw bearer must fail on new ordinary commands and reconnects with and without a configured native secret. Another role of the same device, another device and genuine native requests remain usable. This deliberately preserves the native fail-closed marker rule rather than JavaScript truthiness.

### DRT-07 — Retained effects and role isolation

Reuse existing event/LocalExec/WebAuthn admission boundaries: before subscription/registration/headers/initial output, on each existing wake before filters/frames/heartbeat and before bridge response submission. Only the selected role's original proof becomes stale. Drop each owned registration once. A frame admitted before commit may finish; quiet retirement uses existing heartbeat/write bounds. Do not promise zero bytes after commit or add polling/another registry. The initiating self-revoke command may deliver its final committed receipt.

### DRT-08 — Actual App and node preservation

The shipping App's Gateway hook and generic native dispatch both reach its same durable pairing owner. Actual node list/status consumers retain the paired remote row's existing disconnected/inactive projection; operator-role revocation does not erase node state. Preserve `sand-local-node` and its existing runtime authority. Do not create a synthetic connected remote session or claim APNs/worker/device cancellation as a result of SQLite retirement or TCP EOF.

### DRT-09 — Meaningful affected validation

Author and select complete named targets with independent behavioral oracles: real owner-issued credentials; separate SQLite complete-row snapshots; explicit current/admin/self/mixed-role/scope controls; malformed request and decisive-storage negatives; target existence non-disclosure; selected-entry/surface/pending/config preservation; repeat/reopen/revision failure/known rollback; causal second-connection and rotate/revoke races; fresh marker variants; real bound HTTP dispatch/context/reporting and fail-closed default hook; selected-role retained EOF versus unaffected-role continuity; actual App delegates and node helpers. Timeouts or missing output cannot satisfy refusal/cleanup controls. Preserve original DPA/DVR/DPR/DJC/DPS and affected Gateway/App assertions.

Freeze complete new target/name/count/input selections before Actions. Use the existing workflow and evidence harness; no local/htch compilation, tests, format/lint, generators or validation. Same-input positive evidence may be reused with explicit identity limits, never relabeled as execution of another H. Source review, exact bytes, structural G01/G02, runtime tests, real-device compatibility and global acceptance remain distinct.

The implementation selects the complete new `device_role_token_revocation_authority_contract` target (27 named cases), complete new `device_role_token_revocation_gateway_contract` target (10), and complete existing `device_pairing_retained_stream_contract` target (20: the original 14 plus six selected-role cases). Three added cases in the actual App's existing `device_pairing_revocation_tests` module cover operator retirement, node retirement and the bound shipping hook; its original three cases remain required. The existing workflow freezes every exact name and input identity on both applicable macOS and Linux paths. These are authored selections, not execution results.

Integration preserves the incoming canonical-root admission checks. At incoming H `7ba34edf041f0a7e0f67af76fd3a7584b8a49e1a`, runtime run `41564137858` attempt 1, all six selected Rust targets reported 91 passes with command/capture exits 0/0, but the receipt parser counted only 88 names because it omitted three successful libtest `- should panic` records. This is a receipt-oracle failure, not permission to remove the negative tests or the six-target/count/unique-name gates. The integrated parser must recognize that exact successful grammar, reject failed/incomplete/ignored forms, and preserve duplicate names for the existing nonunique-result refusal. Original failed step, skipped App/Automation and downstream unavailable MCP reuse remain historical nonpassing evidence; recovery requires the new exact-H Actions result. The existing canonical-root owner supplied that repair in `ff0d456f322094ca82f063942f548f8d7fe90f00`, including fourteen CI-only predicate controls. This batch preserves that implementation and the incoming media selections through `947e980bd3c9dcb06535127f5cedfd193b83f0fc`; its duplicate local parser proposal is withdrawn.

### DRT-10 — Scope, release and rollback

No schema migration is needed: the existing token JSON and surface columns hold the mutation. No new owner requires an ADR. Current launch/config/native behavior and whole-pair removal remain preserved. Fresh independent security/lifetime and structure/reuse reviews cover the integrated immutable scope, then the affected exact-H Actions gates and scoped authority/coverage must be obtained before calling the unit verified.

Do not roll back the active-marker check to a version that accepts tombstoned credentials without an explicit credential migration/retirement plan. No shipping release or independent ACCEPT is claimed by this batch. All OCRM G01–G12/AC01–AC24 and eight global flags retain their existing state; missing accounts/devices/platforms remain blocked, not not-applicable.

## Source decisions and limits

The fixed-U schema (`packages/gateway-protocol/src/schema/devices.ts`, blob `40aa94ce88d4f64bdc9604f13d491eb4bf4df0f6`), descriptor (`852432d87eecbd5d68bb73b5435f23daa8862377`), device handler (`532afea705ba73a2ff38a99043eadee4f9f6af60`), management authz (`6d4e874d6a6cb97ce46e54103533a436ec93c471`), token kernel (`5c61ce1a12a816dd5c147a91f66a0f1f837051d4`), identity (`cba1da2c85a9955b5793acefc70d86b5c3bc68cf`) and token policy (`36b7e7da9ee177b31fad17e379743418c558a18f`) establish the selected-role operation, pairing permission, cross-device/nonoperator admin rule, entire target-grant cap, existing-entry mutation and three-field receipt. The existing OCRM source carrier preserves the exact source/read records; historic partial reads are not upgraded.

The adopted preimplementation decisions refine the earlier preparation draft: native pairing ceilings can authorize sufficiently narrow operator targets and must reach the owner; malformed operator-target grants fail closed; node-only native surface columns are cleared; valid role arrays plus optional primary roles form target selection, with SQL-NULL arrays rejected. Native stricter normalization/status codes/whole-column invalidation are explicit adaptations. U instead permits malformed-scope fallback and deletes specific node-surface fields on generation change. U remote session/worker/wake effects remain required later integration, not silently exempted.

References: root `AGENTS.md`; OCRM SOT and main Spec §17; existing module map and pairing-authority task; `device-pairing-authenticated-rotation.md`; `device-pairing-revocation-continuity.md`; `device-pairing-revision-admission.md`; `device-pairing-startup-durability.md`; source/recovery carriers in `projects/openclaw-rust/evidence/`. No full-universe or physical-device acceptance follows from this contract.
