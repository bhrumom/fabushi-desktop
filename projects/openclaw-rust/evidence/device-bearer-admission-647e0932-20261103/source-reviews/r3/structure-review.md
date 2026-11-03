# DFA R3 independent structure, reuse and performance review

Verdict: **CHANGES_REQUESTED — one P2 performance finding, source review only.** The complete-grant membership fix is sound, but the same new fresh-authentication call also makes quadratic approved-role normalization load-bearing on ordinary bearer requests. This is not executable validation, a unit-verified determination, G01/G02 closure, or product/release acceptance.

## Review identity and applied settings

- Reviewer task: `/root/device_bearer_structure_r3`; fresh blind R3 review. No prior-round review, peer report, or root arbitration decision was read. Before final adjudication, root supplied one additional concrete approved-role normalization scenario; it was independently checked against this immutable source and is recorded below.
- Applied launch settings supplied with this task: `gpt-6.1-sol`, `xhigh`, `fork_turns=none`. The effort is appropriate to shared-lock authority, proof lifetime, complete-grant costs, HTTP/Settings composition and causal oracles. No worker, CLI or global-setting change was made to tune the model or effort.
- Repository: `/workspace/scratch/29e7135d8bfe/fabushi-desktop`.
- Immutable manifest: `/workspace/scratch/29e7135d8bfe/reviews/device-bearer-admission-r3/manifest.json`.
- Original frozen base and starting HEAD: `fa75f9ccd66e43f3a4d72ab47acdefaa832212f6`.
- Reviewed tree: **`0c2fcb5d8c8d6c7d6d348543eed955541ca2cd2b`**.
- Previous immutable tree used by the focused delta: `b415b8a429b3e0ffe7a2f2ba0e86b2c67f62eab0`.
- Manifest publication parent: `4f0639321c4ae03cc7dbef1d8e49dea0f042e180`, tree `cbc9418ee7572f44d59e067e450a67f6c0bc22a0`. Later mutable HEAD changes were not substituted for the review subject.
- Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`.
- Complete patch: **94,110 bytes**, SHA-256 **`5b39b47b0d09fe197179de7afb7bda2717493f5a15e4b23e4c4b23dd87afc684`**.
- Focused delta: **10,259 bytes**, SHA-256 **`4d15729b783c84da476e730290515c9b32e6c0f161a610b09be2622d963ccd79`**.

Both patch lengths and hashes were independently read with `wc -c`/`sha256sum`. `git ls-tree --long` confirmed every manifest member's blob and byte length. The complete twelve-path patch and entire focused delta were read, not only the new membership hunk.

## Frozen scope and source identities

| Frozen path | Reviewed blob |
| --- | --- |
| `.github/workflows/rust-desktop-runtime.yml` | `ede3b6b9586383a2d0245de933d6ff4417e6ff10` |
| `projects/openclaw-rust/docs/specs/device-bearer-approval-admission.md` | `49ead89ce610835037bc67085dd1b44fe8f8503e` |
| `projects/openclaw-rust/parity-ledger.json` | `2914b1a9aa1e07dac123af7fbcb10f88b1dee93c` |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | `58a6a94c9cb19b67199b628561d4345a26656410` |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | `f5f3d5624568cb553441b8ca0c0f63aff76ebeae` |
| `source/host/app/src/tests/config_secret_boundary_tests.rs` | `4f4a8162d838fc80411f9cb2123f9b50bb0601ed` |
| `source/host/app/src/tests/local_media_file_url_tests.rs` | `57a0a3393f1bb9560eb036687688656b1f3fe6c8` |
| `source/host/src/device_pairing_owner.rs` | `22ef5673ee42bed751a66ac1cf30e6a2e9ab79ad` |
| `source/host/tests/device_pairing_revocation_authority_contract.rs` | `6c6e64cca6041fda69a22b7d327838ebafa4eb40` |
| `source/host/tests/device_pairing_rotation_authority_contract.rs` | `a06e0593cf3be3673e91706e1636399dab90d0a8` |
| `source/host/tests/device_token_rotation_wire_authority_contract.rs` | `7c37e8093b8b7f2975eb031d76dae1b1419ed13c` |
| `source/host/tests/device_token_rotation_wire_gateway_contract.rs` | `e7c42f88067d8d4695ff7ee356987e493b1f4224` |

The R3 delta changes only the DFA spec, shared validator, large-grant owner controls and large-grant HTTP/Settings controls. Original scope remains twelve paths even though the publication patch has eleven paths because the local-media compilation prerequisite already exists at the publication parent. Incoming cron/LFA/evidence material in shared files is retained context, not authorization to enlarge this review. Unrelated media-index/oracle recovery and disjoint inventory/evidence publication are excluded.

## Findings

### R3-STR-001 — P2 — Remove quadratic approved-role deduplication from the newly expanded fresh-authentication path

**Affected source:** `source/host/src/device_pairing_owner.rs:674–678` (new fresh call), `1557–1573` (shared validator), `1620–1629` (`current_pairing_record`), `247–249` (`RotationRecord::approved_roles`), and `74–83` (`normalized_authority_strings`).

**Trigger and concrete failure:** a valid approved pairing may contain 20,000 distinct roles, with `operator` first and only a small ordinary `operator.write` grant. `handle_pair_request` serializes the supplied role array without a role-count cap (`716–726`). Trusted approval iterates those roles, uses token-map membership to avoid duplicate token issuance, and persists the original `roles_json` (`984–1048`). For the nonoperator entries the small operator-only scope input produces empty grants, so this state does not require a huge scope vector or corrupt data. The Gateway body limit is hundreds of MiB (`gateway_server.rs:55–56`), far above such a role array; the owner API also accepts it directly.

Every fresh authentication now enters `current_pairing_record` while holding the existing pairing mutex. `approved_roles()` fully constructs the normalized role vector before its caller tests selected-role membership, even when `operator` is the first array entry. The helper checks each new distinct role with `normalized.iter().any(...)`, performing N(N−1)/2 string-equality comparisons: **199,990,000 comparisons for 20,000 distinct roles**. Each ordinary request with a known active bearer repeats that work and holds the same mutex needed by other pairing authentication and mutations. This is an input-dependent quadratic shared-lock cost; no measured latency or benchmark result is claimed.

**Why it is introduced in this scope:** the helper already served retained/management paths, but the DFA change newly reaches it on every fresh bearer lookup before ordinary Gateway dispatch. The large-scope membership indexes do not change approved-role normalization. Therefore this is a distinct new hot-path reachability consequence of the changed fresh caller, within the declared pairing-owner scope; it cannot be dismissed as an unchanged management-loop cost. The new large-grant tests keep very small role arrays and do not decide this case.

**Specific fix:** preserve the existing normalized `Vec<String>` and Rust `str::trim` semantics, but maintain one local `BTreeSet<&str>` of seen nonempty role strings while iterating the parsed array. Append to the vector only on first insertion. Retain strict `as_str()?` validation of every element, including a malformed tail after valid/duplicate roles, and preserve first-seen vector order and all role vocabulary. No persistent cache or incompatible count limit is needed. Check large valid role-array fresh/retained behavior and read-only state, alongside duplicate/trim/empty and malformed-member classification controls. The source proof should establish the improved complexity without a machine-dependent timing threshold.

The remaining sections record positive source conclusions and evidence limits. They do not waive R3-STR-001 or claim runtime passes.

### One owner, one currentness validator and unchanged policy

`source/host/src/device_pairing_owner.rs:610–685` retains the existing constant-time bearer match, active-marker rule, canonical public projection and private issuer/fingerprint/admitted-grant proof. The candidate proof now remains local until `validate_current_device_authorization_in(&conn, &authorization)` succeeds. Only `GatewayCommandError::Unauthorized` maps to `Ok(None)` at this fresh lookup boundary; other typed errors propagate. There is no status-number comparison or catch-all suppression.

`device_pairing_owner.rs:1557–1605` exposes the same retained API, acquires the existing mutex once and delegates to the same private connection-taking validator. Fresh authentication passes its already-held connection guard. The private helper does not acquire a mutex, open another connection, write timestamps/revisions or take a second owner. Its selected-credential/approved-role check still precedes live-grant decoding and decisive-baseline decoding. The malformed-field and stale-credential precedence is preserved.

The R3 optimization builds borrowed `BTreeSet<&str>` membership indexes from the already canonical live and approved vectors. Both closures call the existing `role_scope_policy_allows`; they do not duplicate the operator implication table or reinterpret strings. The complete live grant still must fit approval, and every sealed admitted scope must fit both the live grant and approval. Exact literal spelling, NEL versus BOM behavior, nonoperator role prefixes, empty grants and operator implications therefore retain the same predicate. The canonical vectors remain unchanged for public projections and sealed proofs. There is no persistent cache, scope-count ceiling, normalized-key substitution or durable format change.

### Cost and lock lifetime

Previously each checked scope could linearly scan a complete vector several times while holding the pairing mutex. The new helper constructs the two indexes once, then performs a bounded number of logarithmic membership probes per policy invocation. With live size L, approved size A and sealed admitted size P, the membership portion is bounded by construction/comparison work proportional to `(L+A) log(L+A) + L log A + P(log L + log A)`, allowing for string-comparison costs. It uses transient additional memory proportional to L+A, with borrowed keys rather than copied scope strings. This is a source complexity conclusion, not a latency measurement or approved numeric budget.

Existing token-map decoding, canonicalization, device enumeration and nonoperator prefix construction remain costs. The membership change directly removes the repeated complete-scope-vector scans in the shared fresh/retained currentness validator. The separately quadratic role normalization is the actionable finding above, because fresh admission newly invokes it under the same mutex. No new lock is held over Settings dispatch, network I/O, callback execution or connection teardown.

The adjacent mutation admission still uses its existing IMMEDIATE transaction and sealed admission/live/approval/header intersection. The retained Gateway event/bridge consumers still use the original proof before registration/headers/output and after wakes. Thus the additional fresh check does not replace late admission, refresh an old proof, cancel an already admitted effect or promise zero post-commit bytes. Reopen still changes owner allocation identity and requires a freshly issued proof.

### Large-grant behavior and causal oracles

`source/host/tests/device_pairing_revocation_authority_contract.rs:1145–1329` exercises actual file-backed owner authentication and retained validation. The new 20,000 literal node scopes use no admin or wildcard. The real fresh proof must preserve the entire vector; retained validation and an unrelated node remain current. Adding a single unapproved tail scope makes fresh authentication absent and the original retained proof stale. Complete independently observed paired/pending/machine rows remain unchanged relative to the injected state. Repair restores the exact prior snapshot, and a new owner reauthenticates the same durable bearer before retained validation. The fixture observations preserve raw SQL values and revision types, not a projected authorization summary.

`source/host/app/src/tests/config_secret_boundary_tests.rs:481–805` uses the real Gateway server/TCP ingress, real file-backed pairing owner and actual `dispatch_public_settings_command`/Settings service. Both configured-secret and credential-absent native configurations are exercised. The 20,001-entry operator grant includes literal `operator.write`; approval contains the full grant plus the existing node scope. A valid request commits a different time zone to the actual Settings file and verifies it through a new Settings owner. One unapproved tail scope then requires HTTP 401 with precisely one lookup and zero accepted/bound/dispatch/commit/completion/error increments. Full pairing snapshots, Settings bytes and directory entries must remain identical. An unrelated writer stays usable, then repair/reopen permits the original bearer again.

These counters are attached to actual boundaries: accepted increments only after the production owner returns `Some`, bound increments in the request authorization hook, dispatch increments before the actual Settings helper, and commit increments only from that helper's post-commit callback. Gateway completion precedes response serialization in `gateway_server.rs:2043–2075`, and the client reads an actual EOF, so the final exact-count comparison is not a timing-only absence oracle. Ordinary read-only/unknown credentials, role withdrawal, partial overgrant, decisive malformed authority, SQL-NULL fallback, redaction and native positives remain present. There is no machine-dependent timing assertion for the large grant.

Fixture stop/reopen closes the owned listener, drops its API reference and waits for the last API owner's explicit close notification before reopening or removing the database. A teardown timeout is failure; it does not authorize fixture deletion or count as closure. The bounded read and response-size assertions are test I/O safeguards, not grant limits.

### Shipping composition and preserved compatibility checks

The adjacent shipping `source/host/app/src/main.rs` is blob **`ab3cd46cfb869db19d12f2e1ee77a7c18e7b52df`**, independently confirmed in the reviewed tree. Its actual authentication delegate (`13463`), node-only request hook (`13507`), generic/context dispatch (`13580`, `14961`), config dispatcher (`4438–4507`), Settings consumer (`1782`), durable pairing construction (`15979`) and shared Gateway composition (`17528`, `17825`) were read. They bind the same existing pairing and Settings owners. The two new App cases correctly separate actual owner/HTTP/helper execution from mutation-sensitive source bindings. They do not claim full `UnifiedGatewayApi` construction or packaged/server-startup acceptance.

The finite DPA/DVR/DTR fixture edits retain genuine proof acquisition before the invalidating transition. The DPA explicit-empty-baseline success uses a narrowly named helper that identifies exactly `(TARGET, node)` as the known refused unselected credential, asserts exactly one such match and keeps all other original token/state/surface/receipt assertions. Its default helper still requires every other unselected credential to authenticate. The old node proof is acquired before approval withdrawal and checked stale afterward. Reopen tests use fresh bearer absence where old-owner provenance would otherwise obscure the decisive role/grant refusal. No global helper was weakened.

The local-media edits are confined to explicit `serde_json` imports and the existing floating-point timestamp API input. No media product logic or assertion changes are in that prerequisite.

The complete workflow/capture hunks retain full target selections and add all three new owner names and both App names. Adjacent capture logic still requires unique positive names, zero command/capture exits and complete target counts with zero failed/ignored/filtered cases for the relevant owner/HTTP/retained targets. Linux App execution retains exact-name listing and one-passed/zero-failed/zero-ignored checks. Receipt scope text separates source bindings and bounded helper execution from full App/device acceptance. The ledger/RTM additions retain existing blockers and explicitly keep exact-H Actions, scoped authority and acceptance open.

## Read limits and execution state

Authority read: root `AGENTS.md`; OCRM SOT authority/constraints; module-map owner/boundary sections; P0 pairing task; main Spec §17 and relevant security/performance clauses; DFA in full; DPA and DRT in full; DVR requirements/lifetime/oracle contract; DTR currentness, grant, lifetime, verification and rollback clauses. No earlier or peer review was used.

Deep source read covered every changed product/test hunk, the fresh/retained validator, proof and scope helpers, selected currentness/storage parser, adjacent management admission, complete added App fixture and tests, relevant existing DPA/DVR/DTR fixture helpers, and shipping Gateway/Settings composition and report ordering. Additional adjacent consumers were searched and relevant retained/DRT/startup/DPR controls were inspected. The entire unchanged 100-KiB pairing owner and every unchanged test file were not claimed as a new full-file audit. Large workflow, capture, ledger and RTM files received complete changed-hunk review plus their necessary selection, input, oracle and consumer context; unrelated content was not rereviewed.

No local/htch/Actions build, test, formatter, generator, benchmark, schema check or harness was executed. Only source/text reads, Git identity/diff inspection and byte/hash inspection were performed. No repository file, Git index, reference or GitHub object was modified. The only authored artifact is this review report outside the repository.

**Final source verdict: CHANGES_REQUESTED; findings = 1 P2 (R3-STR-001).** The semantics-preserving complete-scope membership change and its large-grant oracles are otherwise sound. Exact published-H compilation, required named execution, source authority/coverage and all original product/evidence gates remain the runner's subsequent obligations after adjudication and any accepted fix.
