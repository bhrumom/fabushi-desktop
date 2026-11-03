# DTR-v1 — Bound protocol device-token rotation

Status: implementation contract, authored before code; runtime verification and acceptance pending.
Task: `647e0932-bea4-4113-9a61-65ae3db58955`, round 1. Fixed U:
`1440bb1771372334bea23d7642bb52deb7db1845`, tree
`90c8aab39972da11cfad220361781d49e1bc6700`.

## Problem, current state and owner

An authorized client needs to replace one approved role credential through
`device.token.rotate`, optionally narrow its grant, and distinguish delivery of a
replacement from a successful rotation that withholds it. The existing native
`device.pair.rotateToken` already performs rotation, but its write permission,
default arguments, native delivery flag, metadata replacement and response shape
are a different contract. A name alias would expose a bearer to native/shared
credentials and would omit the required result scopes.

Use the existing `DevicePairingOwner`, SQLite tables/revision, sealed proof,
Gateway, actual App and retained-connection admission. The preceding DRT source
tree `6b6eef057b3eff3d7990642b4e1314a16bc79ed9` supplies the same-owner optional
proof/native-ceiling hook pattern and selected-role retirement. This tree is a
source reference, not a runtime acceptance result. Implementation may start from
this explicitly frozen overlay under §17.3. The root must reconcile the latest
publication boundary and affected shared-gate evidence before publishing; an
actual failed shared contract is repaired before dependent work continues.

There is no new permission store, token registry, event loop, scheduler, runtime,
dependency or database schema. Preserve native DPA, DVR, DRT, join/startup,
existing Fabushi Bot and local-node behavior. React/Electron remain presentation
and thin bridges. External WebSocket/SDK/device and packaging obligations remain
applicable beyond this HTTP/owner batch.

## DTR-01 — Entry, request and normalization

Register exact `device.token.rotate` under the current DevicePairing owner and
mutating/operator-only classifications. Generic trusted-native dispatch calls
`handle_token_rotate(&Value)`. All HTTP callers use the fail-closed
`GatewayApi::rotate_device_token(Value, Option<&DeviceTokenAuthorization>,
Option<&str>, &GatewayCommandContext)` hook; its default returns a fixed 403 and
does not fall back to generic dispatch. The actual App records desktop contact
once and delegates to `DevicePairingOwner::rotate_token_with_authority`.

Accept a closed object containing required string `deviceId` and `role`, and
optionally `scopes`. Trim identities using the native identity convention and
require nonblank, preserving case. Reject missing identities, wrong types,
unknown members (including caller/admin/crossDevice/revision/delivery claims),
null/nonarray scopes, nonstring members and literal empty scope strings with a
fixed 400. Preserve the distinction between omission and explicit `[]`.

Normalize scope strings using the fixed source's ECMAScript trimming, discard
whitespace-only strings, deduplicate, add admin→read/write or write→read, and sort
in UTF-16 code-unit order. Use existing Rust primitives inside this owner; do not
introduce a normalization service. In particular U+FEFF trims and U+0085 does not.
Stored authority arrays remain strictly typed native state; request validation
and storage-corruption errors are distinct.

**Credential consumer invariant (DTR-SEC-01):** a committed scope has the same
meaning in fresh authentication, currentness, retained admission and every
existing pairing management consumer. A literal `operator.admin\u0085` is not
canonical admin or pairing authority, including when a genuine admin issued it
under an admin baseline. Persisted/live/admitted grants must not be reinterpreted
through a different whitespace policy after issuance. Keep the legacy DPA
request adapter's normalization and response behavior separate from credential
grant interpretation; its existing NEL-positive/BOM-negative request controls
remain required. This fixes a newly produced credential, not a deferred legacy
compatibility issue, and does not add an encoding marker or permission store.

Use the same private ECMAScript scope canonicalization for stored-scope
producers, authentication's admitted/public scope snapshots, live/approved
credential reads and pending-request scope-set comparison. That operation trims
and deduplicates without adding implied permissions; DTR's requested-result
normalizer still adds its specified read/write implications. Keep role/identity
normalization separate. Authorization compares an already adapted requested
scope exactly against canonical grants. Explicit legacy DPA request scopes
retain their Rust-trim adapter before that comparison, including their existing
NEL-success/BOM-refusal behavior; omitted stored scopes use credential semantics.

## DTR-02 — Current caller and target addressing

Lock the existing owner and begin an IMMEDIATE transaction before decisive
stored caller or target reads. Reuse sealed issuer/device/role/token fingerprint
and admitted grant. Re-admit the entire current live caller grant against its
approval before management policy; intersect admitted/live/approved grants and
the optional singular header ceiling. Mutable public proof projections and
body/header identity assertions never become permission truth.

Require `operator.pairing`; admin implies pairing and write alone does not.
Current nonoperator callers are forbidden. A missing/replaced/retired/unapproved
or foreign-owner proof remains 401; malformed decisive storage is redacted 500;
known policy refusal is 403. Existing ingress authentication, replay, Origin,
Host, role mismatch, node/operator-only and explicit read-header refusals keep
their ordering. Only this bound method is exempted from the old literal-write
prefilter. Invalid explicit credentials never fall back to native admission.

Already admitted native callers have admin authority only absent a narrowing
header. A supplied blank, combined or unrelated singular ceiling cannot grant
pairing. Native role headers must be absent or exactly operator. Native admission
has no paired self identity. A nonadmin paired caller may manage only its own
operator role; cross-device paired management and every nonoperator target need
effective admin. Native pairing-only authority may issue a sufficiently narrow
operator grant. Addressing refusal precedes target lookup and is independent of
whether an inaccessible target exists.

## DTR-03 — Approved role, resulting grant and repair

Read the target's strictly typed role array, optional primary role, token map and
needed scope columns in the transaction. Approved target roles are the trimmed
deduplicated union of roles_json and primary role; token entries cannot establish
approval. Missing target or unapproved role is fixed 400. SQL/type/JSON corruption
of decisive storage is fixed 500, with no automatic repair.

**Successor consumer invariant (DTR-LIFE-02):** successful issuance for the
primary-role/role-array union must produce a credential that passes the existing
currentness consumer immediately and after owner reopen. Raw token lookup alone
is insufficient. Reconcile the issuer at its existing atomic mutation boundary;
do not broaden shared currentness to resurrect an old credential after its role
has been withdrawn from `roles_json`, even if the primary label remains. No
unrelated role or caller gains approval through this repair.

When the selected approved role exists only in the primary label, append that
normalized selected role to the existing `roles_json` array in the same UPDATE
and transaction as its new token. Preserve every original array string and its
order; do not add other roles or rewrite the primary label. The changed array
may use compact JSON serialization. When normalized membership already exists,
preserve the complete original `roles_json` bytes. Shared caller currentness
continues to require role-array membership, so later withdrawal still retires
the credential. Revision or known commit refusal rolls back this materialization
together with the token and node surfaces; it is not a second repair transaction.

An approved role may lack a token entry. An existing selected entry must be an
object, but its old active marker, token text or role value are not independent
issuance prerequisites: an otherwise authorized manager may replace an obsolete
entry. Caller self-authority still requires the genuine current token. This
keeps the fixed-U repair behavior instead of importing DRT's old-target-grant
retirement rule into issuance.

Desired scopes are explicit input, otherwise non-null existing selected scopes,
otherwise stored device scopes. A missing/null existing scopes member uses that
fallback. When a stored array is used it must contain only strings; do not
coerce mixed/nonarray corruption into an empty grant. A missing device-scope
fallback is treated as an empty requested grant, while a present malformed
fallback is a storage error. Never substitute the broader approved baseline or
silently remove scopes belonging to another role.

**Decisive approval continuity (DTR-LIFE-03):** successor currentness and bound
caller admission read the approved column, with the device-scope column only as
its SQL-NULL fallback. A malformed unused fallback must not invalidate a freshly
issued successor whose explicit result and approved baseline are valid. The
shared caller read selects that decisive baseline before decoding it; malformed
decisive approval/fallback still fails closed. Preserve the legacy DPA target
read, which separately needs raw device scopes for its own request/defaulting
contract. No column is repaired or discarded by this read.

The issuance ceiling is approved_scopes_json, falling back to scopes_json only
when the approved column is SQL NULL. A valid baseline array is required even
for explicit `[]`; an empty approved array is a real empty ceiling. The entire
normalized resulting grant must fit the selected role's baseline. Operator
grants must also fit all caller ceilings. Nonoperator scopes require the exact
`${role}.` prefix and literal baseline membership. Excess is fixed 403. Explicit
empty/narrow grants may repair an old excessive or malformed selected scopes
member because that member is not read as the requested grant. Other roles,
scope approval, pending pairing rows and unrelated configuration remain unchanged.
The only role-array change is the selected-primary materialization above; its
effective preexisting role-array/primary approval union does not expand.

## DTR-04 — Credential metadata and one commit

After admission, generate one fresh bearer with the existing secure RNG/native
opaque token format. Preserve the selected object's unknown metadata, supported
issuer JSON, creation time and last-use time; replace token/role/scopes, stamp
rotatedAtMs and remove revokedAtMs. Missing or null createdAtMs uses now, as in U;
a new entry also uses now. Present non-null createdAtMs/lastUsedAtMs must be
nonnegative safe integers by numeric value, otherwise refuse with fixed 500.
Integral JSON numbers such as `1.0` and `-0.0` remain valid and are preserved;
strings, fractional values and values above 2^53−1 are invalid. This lifecycle
rule does not change DRT's integer-zero-only active-marker rule. Missing/null
lastUsedAtMs remains unchanged. Issuer metadata is preserved as opaque JSON and
confers no permission; issuer-generation enforcement remains a named native gap.

Use one positive signed-millisecond timestamp no greater than 2^53−1. Token
identity, not timestamp ordering, decides generations; same-millisecond rotations
are valid. Do not put a token or fingerprint in errors, Debug, logs, revision or
events. The selected node role alone clears both existing native retained and
pending node-surface columns atomically; other roles preserve their exact bytes.
Keep old DPA's existing any-role surface behavior unchanged.

Require one updated target row and one existing checked revision increment;
reuse `bump_revision_with_existing_requirement(..., true)`. Missing, malformed or
exhausted revision refuses. Commit once before any success or replacement is
returned. Known SQL/row-count/revision/commit failure rolls back the entire
mutation. Do not start a second mutation to repair an unknown result.

Reuse existing admission, secure RNG, scope policy, target parsing and durable
commit mechanisms where their contracts coincide. Keep old native/DPA shape,
permission, metadata, surface, error and response behavior explicit if a shared
private rotation plan/helper is introduced. Do not copy another state owner or
route this operation through an unbound caller-ID helper.

## DTR-05 — Exact response and identity-bound delivery

Return exactly normalized deviceId, normalized role, committed normalized
scopes, rotatedAtMs, tokenDelivery, and conditionally token. Do not return native
revision, success flag, stored row, issuer or lifecycle metadata.

Only a real sealed paired caller whose device equals the normalized target gets
`tokenDelivery: "in-band"` and the committed nonempty replacement. A self admin
may receive another approved role's token after the nonoperator admin check.
All native/shared-auth and paired cross-device managers get
`"withheld-cross-device"`; token must be absent, not null or empty. No claimed
body/header identity or admin privilege changes that delivery decision. A
postmutation currentness check must not suppress a committed self response.

## DTR-06 — Retained lifetime, races and uncertain outcomes

The old selected fingerprint becomes stale for fresh authentication and the
existing event/LocalExec/WebAuthn admission boundaries. Reuse their checks before
subscription/registration/headers/output, after wakes and before response
effects. Retire each owned registration once. Other roles/devices and native
streams remain usable. Frames admitted before commit may finish; quiet expiry
uses the existing heartbeat/write bounds. No second registry or polling loop.

Two self/operator rotations admitted with the same original credential have one
replacement winner; the other sees stale authority. A separate current admin
may rotate the same other target twice serially: both can commit and the final
credential wins. Caller retirement, narrowed approval, selected-role revoke and
rotation linearize under the same transaction. Test refusal state against the
state after a competing independent commit, preserving that commit's effects.

Disconnect/cancellation/response-write loss after commit does not prove rollback
or delivery. Do not retry automatically, compensate by issuance, expose a
withheld token through readback or infer request identity from a timestamp.
Current list/status cannot resolve an exact lost rotation request; explicit
authorized repair/re-pair is the current recovery path. Remote worker, APNs,
wake, synchronous upstream WebSocket fencing and client credential-cache
replacement are separate consumers, not claims made by native SSE EOF.

## DTR-07 — Verification and acceptance criteria

Freeze complete targets, exact names/counts, inputs and all actual App selections
before executing only in GitHub Actions against published H/tree/U. Obtain the
appropriate scoped source authority/coverage and behavior evidence before any
unit-verified claim. Independent source reviews are not execution evidence.

Required causal oracles are: real issued credentials and separate SQLite complete
snapshots; self/native/cross-device/same-device-other-role delivery; pairing versus
write/admin/node/header/currentness; inaccessible-target non-disclosure; role
union/missing entry/retired replacement; explicit/omitted/empty/implied/Unicode
scopes and legacy-node grant repair; exact response and metadata; all unchanged
rows/surfaces/pending/config; reopen/revision corruption/exhaustion/row-count and
known rollback; second-connection lock order and rotation/revoke races; real TCP
hook/context/default refusal and listener guards; selected predecessor EOF and
unaffected continuity including quiet/gzip/bridge paths; actual App delegation,
node projection and sand-local-node preservation. Timeout is not successful EOF.

For DTR-SEC-01, issue a privileged-looking NEL literal through the real owner and
HTTP hook, then use that exact successor against existing DPA/DRT and same-wire
management consumers. Require literal grant preservation, refusal without any
target/revision mutation and ordinary pairing/admin positive controls. For
DTR-LIFE-02, cover both operator-primary-only and node-primary-only successors
through actual currentness and retained reconnect, including reopen, exact
revision/surface effects, known rollback and later role-withdrawal refusal.

Retain DPA/DVR/DRT/DPR/DJC/DPS and affected protocol/Gateway/App/Automation
regressions. A real failed shared contract is repaired before dependent work.
Preserve original failing evidence; classify implementation, oracle or environment
and rerun only invalidated/failed checks with controls. Missing physical devices,
accounts, services or platform runners is blocked, never mock compatibility.

## DTR-08 — Adaptations, remaining scope and rollback

This native adapter deliberately keeps native trimmed identity, prefixed opaque
tokens, strict decisive SQL/authority shape, optional singular native ceiling,
unknown selected metadata preservation and node-only full-column invalidation.
U instead echoes raw input ID, uses unprefixed tokens, reconstructs known token
fields, and preserves node surfaces except generation-owned bins/sessionHost.
These differences and legacy stored-data normalization need their own complete
compatibility disposition; this batch does not exempt them from final scope.

No schema migration is required. Reverting method registration prevents new wire
rotations; do not restore a predecessor bearer or remove the active-marker/
fingerprint check. Keep committed rotations and existing native operations
consistent through upgrades/rollback. No release or independent ACCEPT is implied.
Full CLI list/handshake negotiation, browser credential cache, issuer generation,
remote node transport/wake/cache/worker recovery, supported platforms, real
ecosystem, UI/temporal/security/performance/soak and G01–G12/AC01–AC24 remain
applicable. All eight global completion flags remain false.

## Fixed-source and existing-owner references

Load-bearing U identities: devices schema `40aa94ce88d4f64bdc9604f13d491eb4bf4df0f6`;
descriptor `852432d87eecbd5d68bb73b5435f23daa8862377`; device handler
`532afea705ba73a2ff38a99043eadee4f9f6af60`; management authz
`6d4e874d6a6cb97ce46e54103533a436ec93c471`; token kernel
`5c61ce1a12a816dd5c147a91f66a0f1f837051d4`; token utilities
`7bb56b799c5945489f60520329d67ea18c06ce75`; scope normalization
`0fe497d4367619e897bd126e06b91eb42f89f1d8`; token generator
`6ae16b04337cbfb9d56cad0ce50bfa512c713d6f`; UI result/cache operation
`9e678b5d00d6aafc49467a037b632526fcf6ab80`; CLI runtime
`042d560beb26ec5a8cd7bfcdf4b16dd7a9486622`. Generator inputs/bindings,
transaction/publication, identity/token-policy and actual consumers are recorded
with their honest full/partial read ranges in the batch source evidence. Acquired
generated output is contract-reviewed, not mislabeled as a complete semantic read.

Existing project contracts: `device-pairing-authenticated-rotation.md`,
`device-pairing-revocation-continuity.md`, `device-role-token-revocation.md`,
`device-pairing-startup-durability.md`; root AGENTS, OCRM SOT/main Spec §17,
module-map M03/M04/M13/M32 and the current pairing task remain authoritative.
