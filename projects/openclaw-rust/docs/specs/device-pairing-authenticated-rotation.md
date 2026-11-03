# Authenticated paired HTTP token rotation — DPA-v1

Status: implementation contract; exact-H Actions and external acceptance pending.
Task: `647e0932-bea4-4113-9a61-65ae3db58955`, round 1.
Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`, tree
`90c8aab39972da11cfad220361781d49e1bc6700`.
Discovery target: tree `15911945f31b8a11b7d32e0072110ff3a75c18e4`;
current integration base `f71bb36a8757b4acf2356fa34de9d73721ccfbda`, tree
`094035e3c9121ad75f85b4dfb35594c3bbb2b2e8`. The pairing owner remains blob
`510d42d62acc1a0a0b95a651f343bbb9bcc5690d`; intervening App changes belong to
MCP/channel processing. Integrate after the reviewed DPR-v1 CI consolidation.

## Problem and existing ownership

The HTTP server authenticates a paired bearer, but drops its identity at the
generic command boundary. The rotation owner receives only target arguments.
An ordinary operator/write caller can therefore address another device and
obtain its replacement token by omitting `crossDevice`. An explicit failed
bearer on a listener with no configured secret can fall back to ambient native
authority. Separately, the authentication snapshot has no binding to the token
generation actually presented, so replacement/revocation before mutation is
not rechecked. These are source findings, not an executed exploit report.

The single production chain remains HTTP Gateway → `UnifiedGatewayApi` →
`DevicePairingOwner` → its existing SQLite pairing/token/revision tables →
commit → the existing Gateway response/report path. This batch extends that
owner and its typed Rust boundary. It introduces no store, permission service,
schema migration, scheduler, Agent loop or frontend policy.

## Shared Rust contract

`DeviceTokenAuthorization` keeps its public device/role/scopes projection and
adds a private, cloneable owner-issued proof. Only authentication's same locked
token comparison may issue it. It seals the issuer allocation, device ID, role,
accepted token SHA-256 and admitted scopes. Public projection mutations cannot
change the sealed principal/grant. No serde/public proof constructor or raw
bearer in context/debug/errors is allowed. Proof debug output is redacted;
issuer equality uses allocation identity, not unit-value equality. Reopening
an owner preserves persisted tokens but requires newly issued proofs.

The existing owner exposes
`rotate_token_for_authenticated_device(&self, args: &Value, authorization:
&DeviceTokenAuthorization, scope_ceiling: Option<&str>)` returning the existing
`Result<Value, GatewayCommandError>`. The optional string is the singular
`x-sand-scope` ceiling; it can only constrain authority.

`GatewayApi::rotate_paired_device_token(&self, args: Value, authorization:
&DeviceTokenAuthorization, scope_ceiling: Option<&str>, context:
&GatewayCommandContext)` has the same result type and a fail-closed default.
Recognized paired HTTP rotation invokes this hook instead of the generic
dispatch. The actual App override delegates to the same pairing owner and
performs the same single desktop-contact update as normal `call`. The hook
result joins the existing timing/report/error/response path. Native direct and
configured-secret calls keep their existing dispatcher. Other Gateway methods
and the generic context contract are unchanged.

## Required behavior and observable oracles

| ID | Contract |
| --- | --- |
| DPA-01 | Recognized paired rotation always carries its sealed credential into the bound hook. The default hook refuses instead of falling back to legacy `call`. An explicit invalid/blank/malformed/retired bearer cannot become native on a secret-free listener. Credential-absent native mode requires the actual listener to be loopback, not just a loopback Host header. Configured native-secret and intended public pairing request/status behavior remain. |
| DPA-02 | Begin an immediate SQLite write transaction while holding the existing owner mutex before reading caller or target facts. Re-admit the exact issuer/device/role/token fingerprint and current approved role/baseline. The entire live token grant must still fit the current approved baseline; a partly unapproved live token cannot retain a smaller permission subset. Missing/replaced/revoked or no-longer-approved credentials refuse with 401. A different owner or mutated public projection cannot manufacture proof. No unlocked precheck followed by a separate mutation is sufficient. |
| DPA-03 | The caller must currently be an operator with the existing rotation write permission. For each requested permission, all of the sealed admitted grant, live token grant, current approval ceiling and optional header ceiling must allow it. Admin implies operator permissions; write implies only read/talk/cron/sessions.read/sessions.write, and read or sessions.write implies sessions.read. Non-operator scopes require their own role prefix and literal membership. Unknown roles, forged headers/body caller fields and blank ceilings cannot grant authority. |
| DPA-04 | Non-admin callers may manage only their own operator token. Check cross-device/non-operator-role authority before revealing target state. Admin may rotate another approved device/role, bounded by that target's approval. No header can supply missing admin; a write ceiling on an admin bearer removes cross-device authority. |
| DPA-05 | Target role defaults to operator; paired request types must be valid. Desired scopes come from explicit scopes (including an explicit empty array), otherwise the existing role token scopes, otherwise the target's role-filtered baseline. Prefer approved_scopes_json, fall back to scopes_json only when SQL NULL; missing/malformed data refuses without a permissive default. Require desired scopes within the target baseline and, for operator tokens, every caller ceiling. Reject unauthorized requested scopes rather than silently discarding them. Omission must not expand a narrower existing token to the whole baseline. |
| DPA-06 | After all checks, generate one token, replace only the selected entry, clear the same retained/pending node surfaces, advance revision using DPR-v1, check one affected row, and commit before success. Any parse/policy/RNG/SQL/revision/commit error returns no successful receipt or secret. Preserve other role tokens and atomic rollback. |
| DPA-07 | Paired in-band delivery derives only from sealed caller ID == target ID. Body crossDevice/caller/admin fields cannot change permission or disclosure. Self receives the committed token even with crossDevice=true; authorized cross-device admin receives withheld-cross-device and no token. Native crossDevice semantics, method name, default role, token prefix and response fields remain compatible. |
| DPA-08 | If caller retirement commits before bound admission, a paused HTTP request refuses; if rotation commits first, later retirement does not retroactively undo it. Concurrent self-rotations with one original credential admit exactly one replacement. Unrelated device revision changes do not retire a current caller. Refusal/replacement and unaffected-role authority survive reopen. Never automatically replay a lost/unknown result. |

Malformed request arguments use 400, denied grants use 403, retired/untrusted
credentials use 401, and corrupt stored authorization or storage failure uses
the existing internal-error boundary. New diagnostics exclude token bytes,
fingerprints and raw persisted JSON. Authentication-only provenance remains
distinct from an always-current grant; the definitive checks are inside the
mutation transaction. Native rotation reuses the same transaction/update code
without being silently converted into a paired principal.

## Implementation and validation campaign

Root integrates the owner/proof, public trait, actual App composition, workflow
and RTM serially. Test workers may own disjoint new owner and HTTP contract
files against this DPA-v1 contract and an exact base; they do not modify the
shared public files. Replace the sole synthetic authorization struct literal
in the old Gateway fixture with an actually issued owner credential.

Use a real file-backed owner and independently opened read-only SQLite
observations of both devices, token maps, roles/scopes/baselines, surfaces,
pending rows and raw revision type/value/timestamp. Observe old/new token
authentication, unchanged nonselected roles and reopen behavior. Cover self,
cross-device, node/admin/write/read/unknown-role, header reduction/forgery,
explicit/omitted/empty/implied scopes, corrupt/missing approval, revision
corruption/exhaustion, foreign-owner proof and altered public projection.

HTTP controls use `start_gateway_server` and actual TCP/HTTP with a thin API
delegate to the production owner, no copied policy. A barrier in that delegate
pauses after real authentication, then production token replacement/removal
retires the caller before dispatch resumes. Compare the target/revision to the
state after that independent retirement. Include positive no-retirement and
two-concurrent-self controls with bounded waits, not scheduling sleeps. Test
the fail-closed trait default and native/public controls. Existing real App
compilation plus a bounded wiring check covers the shipping override; a thin
test adapter is not described as a full shipping-binary runtime test.

Freeze the complete candidate and obtain independent security/lifetime and
structure/reuse/performance review. Run affected Host/App compilation, both
new complete contract targets and preserved DPR/DPS/Gateway/Memory controls in
the existing GitHub Actions lanes. Receipts bind exact H/tree/U, source and log
hashes, zero command/capture exits and all named positive tests with no ignored
or failed cases. No local/htch executable validation. A source approval or old
H result cannot satisfy this candidate's behavior evidence.

## Source, release and remaining scope

Necessary fixed-U sources: devices.ts blob
`532afea705ba73a2ff38a99043eadee4f9f6af60`, device-management-authz.ts
`6d4e874d6a6cb97ce46e54103533a436ec93c471`, device-pairing-tokens.kernel.ts
`5c61ce1a12a816dd5c147a91f66a0f1f837051d4`, device-pairing-token-policy.ts
`36b7e7da9ee177b31fad17e379743418c558a18f`, operator-scope-compat.ts
`a265fb895a8535378e9bb9626c19a030df463e1f`, device-pairing-identity.ts
`cba1da2c85a9955b5793acefc70d86b5c3bc68cf`, and the DPR transaction/store
source. The assessment records actual dependency read extent separately;
historical OCRM reading facts and global flags are not promoted.

No database conversion, reset or new persisted permission field is needed.
Existing records retain their tokens; paired rotation now refuses unauthorized
or corrupt requests. Rolling back source restores the known authorization
defect, so rollback needs an explicit operational choice. Existing Gateway
diagnostics/reporting carry refusals, without logging secrets.

This closes only the declared paired HTTP rotation contract. U aliases,
shared-auth token-delivery parity, other pairing management authorization,
websocket invalidation, external token import/resurrection, issuer-generation
policy, worker/device teardown, arbitrary database replacement, full companion
devices/platform packages and actual release/upgrade/rollback/soak remain
required subsequent scope. Native whole-device revoke keeps its meaning. No
claim of isolation across every pairing method, physical-device acceptance,
complete source closure, G01–G12, AC01–AC24 or independent final ACCEPT follows.
