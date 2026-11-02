# Native channel credential HTTP authorization — CGWA-v1

Status: implementation contract; exact-H Actions and final acceptance pending.
Discovery base: `79920666e475920109690520cd913df40b28c0f7`.
Integration base: `a1990f8a295f50a1852a63d4f22f971beb400b36`.
Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`.
The relevant Gateway source remains blob
`579f21811cb01b32f838d2170f05d72e18a3f7dc` at both subjects.

## Problem, decision and owner

The registered native `connectChannel` and `disconnectChannel` methods reach
the existing HTTP Gateway, `UnifiedGatewayApi`, TranscriptManager and Session
credential/metadata writers. Neither method appears in the current HTTP role
and scope classifications. An authenticated paired read-only operator, node,
or other approved role can therefore reach credential mutation according to
the inspected production chain. In no-configured-token mode an explicit
unrecognized credential can also retain ambient access for these methods.
This is a source-confirmed admission omission, not an executed exploit claim.

Decision **CGWA-01 — FIX / IMPL_BUG**, before implementation: extend the same
private native read/write classification currently used by Memory in
`source/host/src/gateway_server.rs`. The existing `DevicePairingOwner` remains
the only paired role/scope authority. Request headers may narrow its grant,
never create one. Keep the real Session filesystem and lifecycle owners.
No new permission service, channel runtime, registration registry or state
store is needed. This is a native Fabushi credential-write boundary; it does
not implement OpenClaw's separately named channel-account lifecycle API.

## Behavior contract

| ID | Required behavior |
| --- | --- |
| CGWA-01 | Both exact native methods require paired role `operator`. Node, unknown/other approved roles, and an explicitly non-operator role header fail before JSON decoding or Session dispatch. Registry handling for unknown and suffixed methods is unchanged. |
| CGWA-02 | A paired operator requires `operator.write` or `operator.admin`. Read-only, empty, pairing-only, session-only and unrelated grants do not authorize either credential write. An admin-only grant must work. Do not add these aliases to the older write-only mutation classifier, which would contradict this implication. |
| CGWA-03 | The singular `x-sand-scope` header is only a ceiling. Absent headers retain actual caller authority; write/admin ceilings can admit an already authorized caller. Read, empty, unknown or combined scalar values refuse. Privileged-looking headers cannot promote a read-only/node/other token. |
| CGWA-04 | Preserve configured native shared-bearer access and trusted-loopback access with no Authorization header. In no-configured-token mode an explicit unknown, malformed or revoked Authorization must return 401 rather than regain ambient native authority. A recognized paired bearer still uses its actual role/scopes. Configured missing/wrong/revoked credentials remain unauthorized. Preserve common Origin and Host checks. |
| CGWA-05 | Every denied request invokes the Session business dispatcher zero times, reveals no credential and leaves exact secret/metadata bytes and owner projections unchanged. Exercise valid bodies and malformed bodies for each method, so argument refusal cannot masquerade as authorization. An admitted request following a refusal must still reach the same owner. |
| CGWA-06 | Admitted connect/disconnect call the actual TranscriptManager/Session handlers, retain their existing response shape, and alter only the addressed Agent/platform. Verify intended token/metadata persistence and deletion after reopening actual file-backed owners, with another platform and another Agent as unchanged sentinels. Responses must not include token material. |
| CGWA-07 | Preserve Memory's existing read/write/admin rules and exact error text, native/paired compatibility, other route policies and all existing assertions. This batch does not silently change `getAgentChannels`/`refreshChannel`, native-only MCP/attachment rules, or dotted channel methods. Their remaining scope is retained for subsequent work. |

For the two new guarded methods, use the existing 401/403 response mechanism
with fixed, secret-free channel credential role/scope messages. Keep Memory
messages unchanged when generalizing its private classification. Permission
refusal precedes body decoding, device request binding and business dispatch;
network receipt of body bytes is not claimed to be prevented.

The exact channel role message is
`ROLE_MISMATCH: Channel credential methods require operator role`.
The scope message is
`INSUFFICIENT_SCOPE: Channel credential access is not granted for method: {method}`,
where `method` is one of the two registered literal names above.

## Production chain and state

Native Coordinator discovery supplies the existing shared bearer, without
role/scope headers. Native or paired HTTP requests enter
`GatewayServer::handle_connection` → existing registration/authentication →
the generalized private read/write gate → `GatewayApi::call_with_context` →
`UnifiedGatewayApi` → `TranscriptManager::dispatch_session_gateway_call` →
`ProductionSessionWorkers` → `SandConnectorSecretStore` / `FileChannelStore`.
The shipping caller publishes its current timeline event only after success.

The new gate is synchronous and owns no accepted asynchronous work. A denied
request creates no Session operation, persistence record, retry or recovery
obligation. Accepted operations retain the existing per-Agent lifecycle and
filesystem admission. This change does not infer rollback after a lost HTTP
reply, introduce automatic replay, or claim atomicity across separate secret
and metadata files. Corrupt-store handling, cross-process exclusion, partial
two-file mutations and real provider registration remain explicit open work.
No schema conversion or credential rewrite is required for rollout. Rollback
must not knowingly restore this permission omission; there is no data migration
to reverse. Existing native UI and thin transport contracts remain intact.

## Verification and delivery

Add `source/host/tests/channel_gateway_authorization_contract.rs`, using an
actual `GatewayServer`, TCP requests, real approved/rotated/revoked credentials
from `DevicePairingOwner`, and a thin `GatewayApi` adapter forwarding to the
actual TranscriptManager and Session owners. Counters observe that boundary;
they do not replace the credential writer. Use fixture-owned paths and fake
test credentials only. Verify raw byte snapshots and reopen state directly.

Required causal cases:

- `channel_gateway_rejects_read_node_and_unknown_roles_before_owner`;
- `channel_gateway_headers_and_unrelated_grants_cannot_authorize_credential_writes`;
- `channel_gateway_write_and_admin_credentials_persist_through_reopen`;
- `channel_gateway_native_credentials_and_header_ceilings_preserve_compatibility`;
- `channel_gateway_explicit_bad_or_revoked_credentials_never_regain_ambient_authority`;
- `channel_gateway_registry_origin_and_memory_controls_remain_intact`.

Select the new target alongside the existing Memory authorization contract in
the existing Linux/macOS recovery workflows and caches. Require each exact
positive once, zero command/capture failures and bound H/tree/U/input/log
identities. Retain all inherited controls. Compile the actual shipping Host
and inspect the selected actual-owner results on the same frozen H. Independent
full-scope source review precedes integration; every recovery classifies the
failure and retains still-valid evidence. Execute only in GitHub Actions, never
locally or in htch-runtime. Controlled TCP/owners do not establish real external
device, service, channel account or packaged desktop compatibility.

Root serially owns this spec, workflow integration and any additive evidence
links in the existing OCRM ledger/RTM. Production changes are confined to the
existing Gateway file; the independent test target has no new product owner.
Source-structural receipts are not scoped semantic authority. Record actual
behavioral observations without promoting whole-unit verified/accepted or any
global flag before the corresponding scoped and final gates exist.

### Review recovery — scoped command and capture admission

Decision **CGWA-REVIEW-01 — FIX / TEST_FLAW**, before the repair. Independent
review of the complete four-path tree
`ecac6cd55edf74cf2604269af4558c1f242b0f2a` found that the shared macOS
Memory/channel guard could emit a passed local receipt when Cargo and its
named tests succeeded but that command's log projection/capture failed.
The whole job would still fail; the defect is the local evidence verdict.
The actual wrapper already records command and capture outcomes separately.

Require exactly one zero command row and exactly one zero capture row for
every command used by this shared guard. Missing, duplicate, malformed or
nonzero rows refuse the receipt. Use one predicate for actual evidence and
the owned negative controls; preserve named positives and exact subject/tree.
Record both row sets in the receipt. Apply the same predicate to the derived
Memory-append receipt's integration and synthesis inputs and to the incoming
dedicated pairing receipt, so none can inherit or claim unchecked capture.
Keep every existing product assertion, selected target and whole-job failure.

Decision **CGWA-REVIEW-02 — FIX / TEST_FLAW**: the adjacent published
`CFL-CHECKED-v1` guard uses the same command-only check for its integration and
App cases. Apply the same shared row predicate there and record each capture
row set. The predicate and its controls live in the existing diagnostics
step and artifact; they introduce no product owner, workflow or acceptance
ledger. This is a source-established evidence defect, not an executed
channel-recovery failure or a reason to discard its still-valid observations.

Exercise valid, missing, duplicate, malformed and nonzero command/capture
records in the same existing Actions step and preserve the control results
with its artifacts. These are evidence-instrument controls, not product
behavior tests or approval. The fixture repair already published in
`c4106e419e0fa7a45a6fd255ba1a2c8587f9a0b7` restores the existing production Todo
adapter and its assertions; retain it verbatim when integrating. Neither
that correction nor this review promotes the failed `f71bb36a8757b4acf2356fa34de9d73721ccfbda`
campaign, a later H, or any OCRM global flag.

## Fixed-U and native provenance

The native classification follows the existing MGA-v1 contract in
`memory-gateway-selector-boundary.md`, the native Coordinator HTTP producer,
the registered aliases in `host_gateway_api.rs`, and actual Session handlers.
Fixed-U role and scope references, retaining their recorded review extents:

- `src/gateway/methods/core-descriptors.ts`, blob
  `852432d87eecbd5d68bb73b5435f23daa8862377`, lines 1–60 only;
- `src/gateway/role-policy.ts`, blob
  `09247fd9cfeb26a2bf5642d325d41f3d397a709a`, full 1–24;
- `src/shared/operator-scope-compat.ts`, blob
  `a265fb895a8535378e9bb9626c19a030df463e1f`, full 1–119;
- `src/gateway/server-methods/method-authorization.ts`, blob
  `ac99d03f94f35de81ff15cfb70bb84bb5fbc911a`, full 1–125;
- `src/gateway/server-methods/channels.ts`, blob
  `8a498f4bbe853416124a858db5a0db59c2633ac3`, operation ranges 60–133 and
  566–615 only, not a full implementation review.

At U, `channels.status` is read and `channels.start`/`stop`/`logout` are admin
operations over registered channel plugins/accounts. These are not aliases for
the native Agent/platform credential CRUD. Their admin semantics, registration
and lifecycle responsibilities remain required; CGWA does not claim them done.
All historical read facts and the single canonical OCRM inventory/ledger/RTM
remain independent from this native behavior repair.
