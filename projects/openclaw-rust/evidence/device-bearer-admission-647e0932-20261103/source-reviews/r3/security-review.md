# Device bearer admission R3 — independent security, general and lifetime review

Verdict: **SOURCE_REVIEW_APPROVED** for the immutable scope identified below. No actionable security, correctness or lifetime finding was identified in the focused delta or the complete frozen change scope. This is a source-review verdict only. It is not compilation, test execution, scoped G01/G02 closure, unit verification, packaged acceptance or permission to reinterpret earlier execution evidence as execution of this candidate.

## Reviewer settings and independence

- Reviewer: fresh `/root/device_bearer_security_r3` worker.
- Actual launch settings supplied by the parent: `gpt-6-astra`, `xhigh`, `fork_turns=none`. No model/effort fallback was used or requested by this worker.
- Effort rationale: shared pairing-mutex authority, sensitive stored grants, typed error precedence, sealed proof lifetime, and actual HTTP/Settings effect and cleanup boundaries require deep review.
- No prior-round reports, peer reports or root arbitration decisions were read. The manifest and source contracts were treated as scope/requirements, not as conclusions.
- Operations performed: read-only Git, text inspection and hashing; this report is the only write. No product/source mutation, GitHub write, build, formatter, linter, test, generator, benchmark, schema check or other executable validation occurred. No local or htch validation occurred.

## Immutable identity and exact scope

Repository: `/workspace/scratch/29e7135d8bfe/fabushi-desktop`.

| Identity | Value |
| --- | --- |
| Manifest | `/workspace/scratch/29e7135d8bfe/reviews/device-bearer-admission-r3/manifest.json` |
| Original base and starting HEAD | `fa75f9ccd66e43f3a4d72ab47acdefaa832212f6` |
| Reviewed R3 tree | `0c2fcb5d8c8d6c7d6d348543eed955541ca2cd2b` |
| Previous immutable tree | `b415b8a429b3e0ffe7a2f2ba0e86b2c67f62eab0` |
| Declared publication parent | `4f0639321c4ae03cc7dbef1d8e49dea0f042e180` |
| Fixed upstream U | `1440bb1771372334bea23d7642bb52deb7db1845` |
| Fixed upstream tree | `90c8aab39972da11cfad220361781d49e1bc6700` |
| Complete patch | 94,110 bytes; SHA-256 `5b39b47b0d09fe197179de7afb7bda2717493f5a15e4b23e4c4b23dd87afc684` |
| Focused patch | 10,259 bytes; SHA-256 `4d15729b783c84da476e730290515c9b32e6c0f161a610b09be2622d963ccd79` |
| Adjacent shipping App source | `source/host/app/src/main.rs`, blob `ab3cd46cfb869db19d12f2e1ee77a7c18e7b52df` |

I independently recomputed both stored patch byte counts and SHA-256 values. I also reproduced both patches with `git diff --binary --full-index`, restricted to the original twelve paths, against the original base and previous tree respectively; their hashes match the manifest. `git ls-tree` confirms all twelve member blob identities. The adjacent App main blob matches both the reviewed tree and declared publication parent. A later parent notification about disjoint metadata/evidence publication did not change this review authority or widen the scope.

The complete original twelve-path scope was retained and its entire changed content inspected:

| Path | R3 blob | Review disposition |
| --- | --- | --- |
| `projects/openclaw-rust/docs/specs/device-bearer-approval-admission.md` | `49ead89ce610835037bc67085dd1b44fe8f8503e` | Full DFA contract, including DFA-09 |
| `source/host/src/device_pairing_owner.rs` | `22ef5673ee42bed751a66ac1cf30e6a2e9ab79ad` | All changed code and adjacent proof, canonicalization, policy, stored-authority and caller-admission code |
| `source/host/tests/device_pairing_rotation_authority_contract.rs` | `a06e0593cf3be3673e91706e1636399dab90d0a8` | All changed tests/helpers and relevant surrounding state/effect assertions |
| `source/host/tests/device_pairing_revocation_authority_contract.rs` | `6c6e64cca6041fda69a22b7d327838ebafa4eb40` | All changed tests, complete added fresh-admission cases, fixture and independent state/error oracles |
| `source/host/tests/device_token_rotation_wire_authority_contract.rs` | `7c37e8093b8b7f2975eb031d76dae1b1419ed13c` | All changed tests and adjacent concurrent/rollback/repair assertions |
| `source/host/tests/device_token_rotation_wire_gateway_contract.rs` | `e7c42f88067d8d4695ff7ee356987e493b1f4224` | Changed primary-role withdrawal case and its real successor/retained/reopen chain |
| `source/host/app/src/tests/config_secret_boundary_tests.rs` | `4f4a8162d838fc80411f9cb2123f9b50bb0601ed` | Complete added HTTP fixture, tests and source-binding controls; original Settings fixture/consumer boundaries |
| `source/host/app/src/tests/local_media_file_url_tests.rs` | `57a0a3393f1bb9560eb036687688656b1f3fe6c8` | Narrow explicit-import and floating-point fixture-input prerequisite only |
| `.github/workflows/rust-desktop-runtime.yml` | `ede3b6b9586383a2d0245de933d6ff4417e6ff10` | Every changed hunk, affected name/input selection and surrounding acceptance predicates |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | `f5f3d5624568cb553441b8ca0c0f63aff76ebeae` | Every changed hunk, affected complete-target selection and strict receipt predicates |
| `projects/openclaw-rust/parity-ledger.json` | `2914b1a9aa1e07dac123af7fbcb10f88b1dee93c` | Every changed hunk; pending/evidence/scope language and incoming additions |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | `58a6a94c9cb19b67199b628561d4345a26656410` | Every changed hunk; references without verification-state promotion |

The R3 focused delta changes the DFA specification, the common currentness validator, and the owner/App large-grant controls. Incoming cron/LFA/evidence material remains contextual preservation, not a newly reviewed product implementation. Unrelated media-index/DB/oracle recovery was excluded; there was no silent scope expansion.

## Security and general correctness assessment

**One currentness implementation under the existing lock.** At `source/host/src/device_pairing_owner.rs:607-684`, fresh authentication retains the original credential comparison, active-marker check, canonical projection and private proof construction. Before the proof can escape, it calls the new private connection-taking validator with the already held connection guard. The public retained validator at lines 1558-1603 acquires its own guard and calls that same implementation. The helper neither locks recursively nor opens another connection. It contains reads and ephemeral derived collections, with no update, revision bump, activity write or added persisted/cache authority.

**Typed refusal and error order remain intact.** Only `GatewayCommandError::Unauthorized(_)` becomes `Ok(None)` at fresh lookup. Other errors propagate. Retained validation still returns its original typed error. `current_pairing_record` at lines 1608-1647 still checks owner allocation identity, stored row, strict role-array membership, selected token-map entry, entry role, active marker and exact fingerprint before grant policy. The selected live grant is strictly parsed before the decisive approval baseline; only after both reads succeed are membership sets built. Therefore a malformed selected scope array cannot escape via the fresh projection's permissive filtering, and unused fallback data remains ignored only when a present approved baseline is decisive. The existing role/token/currentness-before-policy ordering is unchanged.

**DFA-09 preserves policy semantics.** The only membership change at lines 1579-1595 is replacing repeated vector scans with two local `BTreeSet<&str>` collections and passing their membership operations into the existing `role_scope_policy_allows` function. The original vectors remain available for canonical projections. The sets borrow those local vectors, do not outlive the call and introduce no mutable alias or cross-request cache. Exact string membership, ECMAScript scope trimming/deduplication, literal NEL spelling, operator role prefix, nonoperator role prefix, fixed admin/write/read/session implications and complete live/admitted grant checks are preserved. There is no scope-count cap or wildcard interpretation. With live/approval/admitted cardinalities L/A/P, set construction and membership work is bounded by ordered-set logarithmic lookup, rather than a full grant-vector scan per checked scope. This is a source complexity conclusion, not a measured latency claim.

**Read-only and issuer boundaries remain explicit.** The owner controls at `source/host/tests/device_pairing_revocation_authority_contract.rs:1143-1329` use genuine owner-issued proofs and independent file-backed SQLite observations. The large node grant has 20,000 distinct literal scopes, preserves the full public vector, succeeds at fresh and retained admission, and becomes wholly invalid after one unapproved tail entry. The original proof is tested before reopen so failure cannot be satisfied solely by issuer mismatch. Repair restores the exact prior database image; reopened authentication issues a new current proof. Existing controls also distinguish withdrawn role, wrong entry role/map key, decisive corruption, valid SQL-NULL fallback, malformed unused fallback, unknown bearer and retired marker. Complete pairing/pending/machine images and unaffected credentials remain asserted.

**Late management oracles remain causal.** The changed DPA/DVR/DTR tests acquire valid proofs before the specific authority withdrawal. Reopened invalid credentials are directly tested as fresh refusals; an old-owner proof is not used as a substitute for the changed stored-policy decision. The DPA empty-baseline helper makes only one explicit `(TARGET, node)` exception and requires it to occur exactly once; the ordinary helper still requires every other nonselected credential to authenticate. Stored rows, selected replacement, revision, metadata, rollback, repair and previously decisive refusal assertions remain in place. Native direct management and the existing bound management transactions are not changed by the product patch.

## HTTP, Settings and lifetime assessment

The added fixture at `source/host/app/src/tests/config_secret_boundary_tests.rs:481-804` uses `start_gateway_server`, real TCP requests, a file-backed `DevicePairingOwner`, the actual App `dispatch_public_settings_command` helper and the real `SettingsService`. It does not copy the owner predicate. Its permissive operator request hook matches the relevant shipping hook branch, which returns immediately for non-node credentials (`source/host/app/src/main.rs:13507-13544`).

The refusal oracle captures the entire pairing/pending/machine image, actual Settings file bytes, directory entries and seven counters. Its known bearer tests exercise approval narrowing, partially excessive grants, role withdrawal and malformed decisive storage with and without a configured native secret. Refusal must produce zero request-hook dispatch, Settings dispatch, commit callback and command reports; malformed matched authority must return the fixed internal message. A valid read credential reaches fresh admission but is refused by the ordinary mutation permission guard. Invalid explicit credentials cannot use the listener's native absence mode. Separate writer and native positives commit distinct timezone values and a fresh Settings owner reads the persisted value.

The R3 HTTP addition at lines 771-802 admits 20,000 literal operator permissions plus `operator.write`, then appends one unapproved tail permission. That request must return 401 before dispatch while preserving the complete state oracle. An unaffected writer succeeds, repair removes only the injected invalid scope, and owner/server reopen preserves the repaired pairing image before the same bearer again commits through Settings. No timing assertion is used as authorization evidence.

Resource custody is adequate for this fixture. `GatewayServer::close` only joins the accept thread (`source/host/src/gateway_server.rs:676-697`); the fixture does not treat that as connection cleanup. `DfaGatewayApi::drop` explicitly closes its pairing owner and only then sends a bounded acknowledgement. `stop` waits for this final-Arc acknowledgement before reopen. On teardown failure it retains the owned Settings fixture instead of deleting files while a connection may still own them, and fails the normal test path. HTTP response reading requires EOF within a deadline; timeout is failure. No new production cleanup owner, poller or background task was introduced.

I directly read the shipping authentication/ordinary dispatch/Settings chain and relevant server ingress and reporting code. The source-binding controls at lines 806-847 target actual owner/authentication/configuration/composition strings and include ten mutations that must break the binding predicate. Existing App Cargo dependencies and test-module inclusion support the added fixture. The adjacent App main blob is unchanged at the publication boundary. The source-binding evidence is appropriately described separately from HTTP/helper execution: the fixture does not construct the complete `UnifiedGatewayApi`, perform full App startup or prove packaged/physical-device operation.

## CI/evidence preservation

The workflow and portable capture additions select the three new owner test names and two App names and bind the DFA specification as an input. R3 large-grant controls are inside those already required named cases. The complete DPA/DVR/DRT/DTR/retained target lists and existing App obligations remain required.

In the workflow at lines 2613-2669, acceptance still requires unique required positive names, complete exact-count summaries with zero failed/ignored/measured/filtered tests, command/capture acceptance and exact subject/run binding. The portable capture predicates at lines 1094-1104 retain the same complete-target strictness. The App runner at workflow lines 4354-4365 still checks exact test discovery and one-pass/zero-failure/zero-ignore completion for each required name, then requires every selected case. The patch does not relax these predicates or replace them with prose claims. Ledger/RTM additions describe implementation and required evidence while retaining pending verification/global acceptance boundaries.

## Findings and read limits

Findings: **none**. No severity-ranked fix request is made.

Read scope was all changed content in the twelve declared paths, the complete focused patch, applicable root instructions, the full DFA and DPA/DVR/DRT/DTR contracts, relevant OCRM Spec §17/SOT/module-map authority, and adjacent source necessary to judge proof, policy, Gateway, Settings, fixture cleanup, test inclusion and CI strictness. Large files were not represented as fully line-by-line read; unchanged unrelated test cases, ledger entries, workflow jobs and product domains were not exhaustively audited. Nearby retained-stream authentication/withdrawal call sites were located as context without upgrading that unchanged target to a new comprehensive review.

The fixed-U operator policy blob was not available in this repository's Git object database when directly requested. I did not fetch it or claim a fresh complete upstream read. This review establishes preservation against the existing native policy and the adopted contracts; it does not establish new upstream source completeness or full equivalence. No prior source-read state was promoted.

Required next evidence remains the affected exact-published-H Actions compilation and named behavior controls, scoped source authority/coverage, and the ordinary final acceptance gates. The report approves the inspected source within its declared boundary and leaves those execution and wider compatibility obligations open.
