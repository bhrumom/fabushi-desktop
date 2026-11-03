# DTR-v1 round 1 — independent general, security and lifetime source review

## Verdict

**REQUEST_CHANGES — two P2 findings.** The new issuer can commit a credential whose authority is interpreted differently by another existing management operation, and it can report successful issuance for a role that the existing retained/currentness consumer immediately rejects. Both findings concern newly produced DTR credentials and their existing native consumers. Neither finding depends on executing the authored tests or on claiming broader upstream compatibility.

This is a source-review verdict for the frozen candidate. It is not a runtime result, unit verification, publication approval or global acceptance.

## Identity, settings and independence

- Repository: `bhrumom/fabushi-desktop`.
- Task: `647e0932-bea4-4113-9a61-65ae3db58955`, task round 1, review round 1.
- Committed base: `497c2c7a7bc5201aead83d3d36e44f52ba164d04`.
- Explicit preceding DRT behavior overlay: tree `f6aaaf556fe99d53b46060da46691e154bffdfac`.
- Reviewed candidate: immutable tree `5d8dd303c69999f1f8d68d687ab74b779d675464`.
- Current DTR contract: blob `df77dd07d19680ea27793cbd2f1f71442b39d7f7`.
- Fixed upstream U: `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`.
- Frozen scope: `reviews/dtr-r1/manifest.json`; complete 15-path DTR batch, with the preceding DRT overlay explicitly included as described in the manifest. The complete patch is against the committed base; the introduced patch is against that overlay. A later publication of identical overlay bytes does not by itself supply execution evidence.
- Actual reviewer settings supplied at spawn: model `gpt-6-astra`, reasoning effort `xhigh`, fresh blind fork (`fork_turns=none`). The depth is appropriate for credential issuance, identity-bound secret delivery, transaction ordering, retained lifetimes and recovery. No substitute reviewer model was used.
- No other reviewer report, reviewer conclusion or root decision ledger was read. No sub-reviewer was spawned. Reviewed source was read from immutable Git objects; the fixed-U source pack was read as source evidence. The only file written by this review is this report.

## Findings

### DTR-SEC-01 — P2: a newly issued NEL-suffixed scope becomes admin at the adjacent management boundary

**Primary location:** `source/host/src/device_pairing_owner.rs`, `rotate_token_with_authority`, especially lines 1480–1496; related new `wire_scope_trim`, `normalized_wire_scopes` and `wire_role_scope_allows`, lines 119–152.

**Existing consuming locations:** `normalized_authority_strings` and `role_scope_allows`, lines 74–97; `PairedOperatorAuthority::allows`, lines 221–229; `admit_paired_operator`, lines 1592–1636; `revoke_token_with_authority`, lines 1286–1307. HTTP reaches that consumer through `source/host/src/gateway_server.rs` lines 1901–1909 and 2017–2023. The actual App delegates to the same owner at `source/host/app/src/main.rs` lines 13391–13401.

**Concrete triggering scenario:** Start with an ordinary, current paired operator admin whose approved baseline is `operator.admin`. Use that genuine sealed credential to self-rotate through `device.token.rotate` with:

```json
{"deviceId":"the-current-device","role":"operator","scopes":["operator.admin\u0085"]}
```

The DTR parser intentionally retains U+0085. This resulting scope starts with `operator.`, so the caller's canonical admin and the target's canonical admin baseline allow issuance. `normalized_wire_scopes` does not add admin/read/write implications because the string is not exactly `operator.admin`. The transaction persists the literal suffixed scope and the self response delivers the replacement bearer with that same literal scope.

Authenticate that replacement and invoke `device.token.revoke` against another device's valid node token, with no scope header. Authentication preserves the raw scope in the sealed proof. The existing shared admission uses Rust `str::trim` on stored grants and in `role_scope_allows`; U+0085 is removed there. Both required pairing authority and the cross-device/nonoperator admin test consequently treat this newly issued bearer as canonical `operator.admin`. The target can be revoked. The same fresh bearer is denied the corresponding DTR management authority by DTR's raw-proof wire check, which demonstrates the inconsistent interpretation across adjacent endpoints.

This is not a claim that an unprivileged caller can mint the example without an approving issuer/baseline. The defect is that a successfully attenuated/newly issued credential can exercise authority absent from its specified and returned literal grant. The issuer's existing authority does not justify the replacement acquiring a different interpretation after issuance. DTR-01 expressly makes U+0085 significant, and DTR-03 makes the entire resulting grant the issued grant. The literal-write HTTP prefilter does not prevent the example: bound DPA/DVR/DRT/DTR methods are explicitly exempt and rely on their owner admission.

**Why the existing authored coverage misses it:** `wire_rotation_normalization_matches_fixed_u_implications_and_utf16_order` covers NEL retention and rejection under a narrow target baseline, but it does not present a newly issued privileged-name-plus-NEL token to an adjacent management consumer. `Fixture::rotated`, lines 377–381, verifies raw successor lookup and the copied raw scope array only. The legacy DPA Unicode regression checks DPA request normalization; preserving that request behavior is distinct from silently reinterpreting the output of this new issuer.

**Required fix:** Reconcile persisted-credential grant interpretation across issuance and management/currentness consumers. Preserve legacy DPA request normalization separately if required, rather than allowing it to reinterpret a newly issued DTR grant. If a coherent consumer policy cannot be introduced in this batch, fail before mutation for newly issued cross-policy-ambiguous grants and explicitly reconcile that restriction with the DTR contract. Add successor-use owner and real HTTP cases that issue a trailing-NEL privileged scope, then attempt cross-device/nonoperator DRT and DPA management. Assert the actual authorization result, unchanged target/revision on refusal and unchanged literal receipt, rather than only token lookup.

**Compatibility boundary:** DTR-08 reserves a disposition for legacy stored-data normalization. This finding begins with valid ordinary current state and a fresh DTR request; the inconsistency is produced by this batch. It therefore needs an explicit security disposition before this issuer is accepted, even if broader legacy migration remains separate.

### DTR-LIFE-02 — P2: primary-role-only issuance succeeds but its successor is immediately noncurrent

**Primary location:** `source/host/src/device_pairing_owner.rs`, `rotate_token_with_authority`, lines 1427–1447 and 1493–1528.

**Existing consuming locations:** `RotationRecord::approved_roles`, lines 232–235; `rotation_record`, lines 253–267; `current_pairing_record`, lines 1573–1575; `validate_current_device_authorization`, lines 1533–1556. The actual App's retained validation delegates to that owner at `source/host/app/src/main.rs` lines 13331–13335.

**Concrete triggering scenario:** A target has valid `roles_json = []`, primary `role = "node"`, a valid `node.invoke` approval baseline, and no selected token entry. A separate current admin issues a node token through DTR with explicit `scopes: ["node.invoke"]`. Alternatively, use valid `roles_json = ["node"]`, primary `role = "operator"`, and an explicit approved operator scope.

These are explicitly supported inputs: DTR-03 defines target approval as the role-array/primary-role union, and the new `wire_rotation_role_union_allows_missing_entry_but_never_stray_approval` test uses both forms at lines 641–655. The issuer commits a new active token, increments revision, and returns success. In the node case it also clears the selected node surfaces.

Fresh bearer lookup finds that token because `authenticate_device_token` selects the stored token entry and copies its role and scopes without checking approved roles. However, the very next `validate_current_device_authorization` call rejects it as stale: `rotation_record` never selects the primary role, and `current_pairing_record` requires membership in `roles_json` alone. A newly authenticated successor consequently cannot pass the existing `/events`, LocalExec or WebAuthn retained admission; the operator-primary-only successor also cannot use the bound owner management operations, including self rotation, because they share `current_pairing_record`. Reopening the owner does not change that persisted policy mismatch.

This is not a predecessor-fencing problem. The new credential itself has been issued as an approved role under one predicate and rejected as unapproved under another. The nominal success and revision/surface changes are durable even though the intended repair did not create a usable current credential for these existing consumers.

**Why the existing authored coverage misses it:** The role-union case's `Fixture::rotated` assertion ends at `authenticate_device_token` plus raw role/scope equality, lines 377–381. It never asks the currentness consumer to admit the successor. The new retained successor-continuity cases use ordinary role-array-approved fixtures, so they do not cover the new union-only issuance forms.

**Required fix:** Reconcile the approval predicate for issuance and currentness, including the primary-role-only forms this batch intentionally supports. Preserve the existing role-withdrawal and stale-proof guarantees; merely adding primary-role fallback to currentness without reconciling how approval is withdrawn could weaken those guarantees. If the adapter cannot safely support primary-role-only successors, the contract and issuer must refuse before committing instead of reporting successful repair. Add owner currentness and actual retained reconnect cases for both union-only forms, including after reopen, alongside negative role-withdrawal controls and exact revision/surface assertions.

## Coverage and observations

The review covered the complete declared DTR change, its adjacent DRT overlay behavior, and the relevant existing consumers; it was not limited to isolated new hunks.

- **Authority and contracts:** Root AGENTS/SOT/current main Spec §17 were consulted. The complete immutable DTR contract and adjacent DPA/DVR/DRT contracts were read, together with module-map owner boundaries and the current pairing task. The explicit overlay, source-review/runtime distinction, spec-first status and remaining-scope boundaries were kept separate.
- **Owner implementation:** Deep-read the full new request/normalization/issuance path and the existing proof construction, exact-credential currentness, paired admission, scope implication, DPA rotation, DRT revoke, revision and token generation paths. Traced sealed versus mutable proof data, caller/target ordering, full live grants, header ceilings, selected metadata, node-only effects, checked revision, immediate transaction acquisition, commit-before-delivery and redacted errors.
- **HTTP and actual App:** Read exact method registration/classification and bound/default hook changes, the ingress/dispatch sequence, explicit-credential/native split, replay/Origin/Host/role guards, command reports and response dispatch. Read the actual App hook/delegation, direct trusted dispatch, existing node target binding, retained validation and node list/status projections. No generic-dispatch fallback or native/self-identity substitution was found in the new HTTP path.
- **Lifetime and concurrency:** Read the retained admission points before registration, headers, wake/output and response effects; existing socket/heartbeat bounds and ownership cleanup; the shared test helper changes and all original 20 plus 6 new retained cases. Read the owner and HTTP race/lock/rollback/reopen/lost-reply scenarios and their independent state observations. The lost-reply test correctly does not prove an OS write failure or infer rollback from disconnect. The source uses one immediate transaction and commits before emitting the in-band self receipt; it does not recheck the retired original proof after commit and discard a successful replacement.
- **Authored contracts:** Read all 37 new owner cases, all 11 new HTTP cases, all 26 retained cases and all 3 new actual App cases, including the existing App fixture/delegate controls they use. Reviewed the protocol registration test and affected old admission/native behavior. The new App coverage includes actual projection helpers plus source binding checks; it is not a substitute for full application/transport execution.
- **Evidence and workflow:** Reviewed the candidate source-evidence JSON, workflow additions and required-name/count/whole-target summary controls, and parity-ledger/RTM deltas. The evidence retains pending/unverified/global-false status. Generated output identity/acquisition and selected binding inspection are not a complete semantic read or generator execution. Workflow command presence and textual count/exit gates are authored verification mechanisms, not proof that a runner accepted this tree.
- **Fixed U:** Read the selected authoritative handler, management authorization, token kernel/utilities, normalization/scope policy, identity/token-policy and schema material from the exact-U pack; examined actual UI result classification/cache ownership and relevant validator/binding locations. Used the candidate's honest full/partial/generated boundaries as boundaries, not as a claim that this reviewer independently semantically read every line of every acquired upstream file. External WebSocket, worker, APNs/wake, browser cache and final compatibility obligations remain distinct from the local owner/SSE source claims.

No additional must-fix finding is asserted for secret response gating, redacted new errors/Debug fixtures, commit rollback design, independent-writer linearization, retained predecessor retirement, node-only column invalidation or the exact method/App wiring beyond the two integration findings above. This statement records source-review coverage, not successful execution or a guarantee that those paths pass.

## Verification limits

No build, test, lint, schema check, generator, behavioral reproduction, local runtime or htch runtime was run. No GitHub mutation, workflow dispatch or external message was performed. No production source was edited. All newly authored cases and prior regressions remain **unexecuted in this review**, and no observed pass/fail result is claimed for them. The triggering scenarios above are source-derived traces; the follow-up cases are requested oracles, not purported reproductions.

The preceding DRT overlay's review/publication remains an independent boundary. This report neither accepts that overlay's runtime behavior nor resolves its eventual committed publication identity. Wider retained/cache/issuer/remote-transport/platform/recovery and full compatibility obligations remain subject to their stated evidence and acceptance gates.
