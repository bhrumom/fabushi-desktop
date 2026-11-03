# DTR-v1 R3 independent general, security and lifetime source review

**Verdict: APPROVE_SOURCE_R3 — no actionable blocking or non-blocking defect found in the focused R3 delta and the affected source contracts inspected below.** This is a source-review verdict only. It supplies no compilation, test, scoped-source-authority, unit-verification, device, package or whole-project acceptance credit.

## Identity, independence and execution boundary

- Task: `647e0932-bea4-4113-9a61-65ae3db58955`, task round 1, review round 3.
- Repository: `/workspace/scratch/29e7135d8bfe/fabushi-desktop`.
- Reviewed immutable tree: `a66145d238e8dbfef934b9cba7891d3ee02f05ce`.
- Publication base in the frozen manifest: `fee4f179b640489774208fa3ce653a3868b641bf`.
- Previous review tree, used only as an immutable diff base: `7644b264bf72ef57f7333a7f4f8d7accd2dd7af8`.
- Original frozen base: `497c2c7a7bc5201aead83d3d36e44f52ba164d04`; behavior overlay: `f6aaaf556fe99d53b46060da46691e154bffdfac`.
- Fixed upstream U: `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`.
- Review lens/model/effort: fresh independent general/security/lifetime reviewer, `gpt-6-astra`, `xhigh`, as actually configured by the parent worker launch. Shared credential interpretation, issuance/currentness continuity, retained consumers and integrated evidence gates justify this effort. No model substitution or extra agent was used.
- Frozen scope: the 16 paths and identities in `reviews/dtr-r3/manifest.json`. Product and adjacent target-source reads used `git show a66145d...:path`; immutable comparisons used the stated Git bases. No mutable product file was used as review authority.
- I did not read peer reviews, previous review reports, runner decisions/arbitration, performer handoffs or unrelated evidence verdicts. The parent subsequently reported a publication identity; that report did not change the reviewed tree or this verdict and is not independent publication verification here.
- No builds, tests, lint, rustfmt, schema checks, generators, candidate harnesses, benchmarks, Actions/htch execution or GitHub mutation were run. The sole authored file is this review. Original runtime 113 failures are not used as R3 evidence; no CI outcome is needed for or imported into this code verdict.

The governing material inspected was the exact-tree root `AGENTS.md`, current OCRM SOT authority/constraints, OCRM main Spec §17, current `P0-device-pairing-authority.md`, relevant module-map M03/M04/M13/M32 boundaries, and the complete current `device-token-rotation-wire.md` (332 lines). Historical SOT discovery entries and unrelated App/runtime areas are context, not freshly accepted behavior.

## Findings

**None.** No concrete failure scenario attributable to the R3 changes survived source inspection. Accordingly there is no severity, repair request or speculative cleanup item to assign.

The following are the substantive checks behind that conclusion, with line numbers bound to the reviewed tree.

### Canonical scope membership and borrowing

`source/host/src/device_pairing_owner.rs:124-165` keeps ECMAScript scope trimming in the existing private helper. `canonical_scope_strings` now inserts the trimmed `&str` into a local `BTreeSet` and appends an owned string only on first insertion (`132-141`). Tree ordering affects membership lookup only: output order is still the order of first canonical occurrences. Content comparison preserves the previous exact-equality duplicate semantics. Empty canonical strings remain omitted. U+0085 remains significant and U+FEFF remains trim whitespace. The helper does not add permissions or implications.

The borrowed keys live only inside the function and borrow the iterator's input strings, not the growable output vector. No input is mutated while a key is retained, no borrowed key escapes, and the returned `Vec<String>` owns its elements. This introduces no new owner, global state, retained credential reference, unsafe code or cross-request lifetime. Membership is tree-based rather than an all-predecessor scan; the separate DTR implication and UTF-16 ordering logic at `155-165` remains intact. This is an algorithmic source conclusion, not a measured throughput claim.

I traced its direct and intervening consumers: strict scope decoding (`144-153`), DTR request normalization (`174-192`), approved baseline decoding (`246-257`), native role-token production (`303-330`, approval call at `1000`, native rotation call at `1789`), fresh authentication (`610-674`), selected-role revocation (`1358` and surrounding original-scope diff), DTR issuance (`1401-1546`), retained currentness (`1551-1574`), paired management admission (`1621-1665`), and omitted legacy DPA scopes (`1734-1777`). The explicit DPA request adapter still uses native Rust trimming at `1755-1760`; it is not substituted for credential grant semantics. Role/identity normalization remains separate. Pending scope-set comparison uses the same trim policy with its own set and retains native role-set normalization (`333-379`).

The relevant authored controls remain selected, including fixed-U implications/UTF-16 ordering, literal NEL/BOM baselines, native and omitted-DPA producer order, fresh authentication, pending-scope identity, and adjacent DPA/DRT/DVR management. In particular `device_token_rotation_wire_authority_contract.rs:1605-1638` expects the first-seen order `[literal NEL admin, operator.write, operator.pairing]` through approval, native reissue, omitted DPA and fresh decoding. Those are source-inspected controls, not passed tests.

### Actual large-input HTTP denial and durable continuation

`source/host/tests/device_token_rotation_wire_gateway_contract.rs:897-932` creates 100,000 distinct scope strings and asserts that the serialized request remains below the unchanged Gateway body limit. It uses a real owner-issued current `operator.read` bearer, sends no narrowing header, and performs an actual TCP POST through the normal Gateway listener.

The call is observable independently of the expected status: the fixture's hook increments its bound count and records paired identity/context before delegating to `rotate_token_with_authority` (`276-292`); `assert_bound_once` also requires exactly one recognized bearer and no changes to other dispatch counters (`647-650`). The test requires the exact owner-specific 403 text (`908-915`). The production path exempts this one bound operation from the old literal-write prefilter, retains the explicit read-header refusal, and dispatches it through the bound hook (`gateway_server.rs:1893-1938,1992-2015`). The owner parses scopes before entering the IMMEDIATE transaction and then refuses missing pairing authority before target lookup/mutation (`device_pairing_owner.rs:1407-1435`). Therefore an early prefilter, default adapter 403 or body-limit rejection cannot satisfy the authored oracle.

`Fixture::deny` compares independent complete paired, pending and configuration SQL snapshots, including revision bytes/types (`465-475,529-539`), while the common refusal oracle checks response shape and redaction (`639-644`). The new case requires teardown acknowledgement before reopening (`452-460`), exact unchanged state after reopen, recognition and currentness of the original read-only credential, and a subsequent real self-rotation by a separate pairing-authorized credential on that reopened owner (`916-931`). Its existing positive oracle checks exact response and selected token metadata, predecessor retirement, unchanged unrelated rows/surfaces/pending/config and one revision increment (`542-587`). The new positive additionally calls successor currentness. Socket deadlines are transport/cleanup bounds, with no fabricated benchmark threshold.

### 4xx versus 5xx reporting oracle

The R3 correction at `device_token_rotation_wire_gateway_contract.rs:1166-1197` agrees with the existing production reporting branch at `gateway_server.rs:2043-2071`: success invokes `on_command_complete`, and only status >= 500 invokes `on_command_error`. An ordinary 403 has neither report.

The corrected loop preserves all three actual methods (`device.token.revoke`, legacy `device.pair.rotateToken`, wire `device.token.rotate`) and all three target/device-role combinations. It retains exact 403, complete state equality, redaction, recognized-bearer count, and the appropriate method-hook count. It now also proves no success report was emitted; the wire branch verifies the actual bound context's request ID and sealed-caller projection. The genuine-admin positives, legacy DPA NEL success/BOM refusal, and ordinary pairing-authorized successor remain (`1203-1239`).

The existing storage-corruption positive reporting oracle is still required: five malformed decisive-storage variants return fixed redacted 500, add exactly one error report, and preserve method/status/request ID/error text (`812-867`, particularly `835-854`). The correction neither changes Gateway product behavior nor weakens 403, redaction, state-preservation or 500-report assertions.

### Shared authority, issuance and retained lifetime context

The R3 helper change leaves the sealed proof and its redacted Debug representation intact (`device_pairing_owner.rs:21-57`). Current caller/target reads for mutation follow the owner mutex and IMMEDIATE transaction. Paired permission remains the intersection of admitted, live, approved and optional header ceilings (`235-242,1621-1665`); public projection fields do not drive the decisive owner mutation. Native admission and identity-bound delivery remain separate, with only a sealed same-device caller receiving the committed bearer (`1401-1546`).

I freshly inspected the selected-primary role materialization and its dependent currentness code: only the selected already-approved primary role is appended, only when normalized array membership is absent (`1444-1459`), and this update is in the same target UPDATE as token/surface changes (`1520-1537`). Currentness still requires `roles_json` membership, never primary-label fallback (`1603-1613`). The decisive `COALESCE(approved_scopes_json, scopes_json)` read is shared with caller/currentness, while legacy DPA target parsing retains its separately needed raw scopes (`1490-1497,1595-1602,267-281`). Thus a malformed unused fallback is not newly made decisive, and later role-array withdrawal still retires the credential.

Gateway retained event and LocalExec/WebAuthn consumers retain the original proof and check currentness at existing admission/wake boundaries (`gateway_server.rs:1368-1470,1581-1638`). There is no new registry or proof refresh. Authored selected-role successor/reconnect and primary-only/withdrawal controls were inspected at `device_pairing_retained_stream_contract.rs:504-531,1706-1757,2090-2180`; the unedited queued-frame, quiet-heartbeat and bridge test inventory remains selected. I do not claim a new complete deep read of all 27 retained cases or physical/remote transport acceptance.

The actual App hook still notes desktop contact once and delegates to the same owner with both proof and ceiling (`app/src/main.rs:13378-13389`); generic native registration remains at `4150-4159`. Adjacent App DPA/DRT/currentness hooks and node target policy were read (`13298-13420`), as were existing node projections (`2706-2830`). The three DTR App cases were read in full (`app/src/tests/device_token_rotation_wire_tests.rs:1-173`): two use actual App projection helpers/durable owner, and one is a source-wiring mutation oracle. They do not run a full shipping App transport or prove physical node teardown.

### Shared workflow and portable capture integration

The publication-base diff for the two shared capture files adds the large-request name and updates DTR's HTTP selection count from 13 to 14; no pre-existing command, selected name, input, gate or failure-preservation step is removed by DTR. Relative to the previous review tree, the incoming PRID additions are preserved as integration context, not credited as newly reviewed PRID implementation.

- Portable selections still include complete DTR owner44, HTTP14 and retained27 targets, with the prior DPA/DVR/DRT targets (`capture_portable_contracts.py:493-582,794-803`). Whole-target acceptance still requires every exact positive name once, zero command/capture exit, exact count, zero failures/ignored/measured/filtered tests, and final acceptance of every case (`1040-1086`).
- The workflow mirrors the new exact name and count and retains its command/capture evidence and same-subject/run checks (`rust-desktop-runtime.yml:2545-2665`). The three actual App DTR names remain additive to DVR/DRT and other App controls (`4195-4207`).
- Incoming PRID image/process cases retain their commands and names (`capture_portable_contracts.py:942-952`) and exact selected-count gate (`1053-1054`). The historical capture step remains after successful App selection (`rust-desktop-runtime.yml:4286-4294`), with its failure-fixture condition, credential-file exclusion and evidence upload (`4442-4456`). These are preservation observations; the separate PRID script/product implementation is outside this fresh DTR security verdict.
- MSI and SGF selections/count policy are retained in the immutable publication diff. The inspected portable strict acceptance branch still includes MSI and SGF recorder cases and the SGF three-target count oracle (`1053-1062`). No runtime success is inferred from selection text.

## All 16 paths: review disposition and read depth

“Unchanged R3” means unchanged relative to the previous review tree within the declared path; it does not import a previous review's verdict. Fresh readings below are stated precisely. Original-scope context and immutable diffs were retained, while unrelated incoming changes are not a second acceptance claim.

| Path | R3 disposition | Fresh inspection and limits |
| --- | --- | --- |
| `projects/openclaw-rust/docs/specs/device-token-rotation-wire.md` | Changed contract; consistent | Complete current 332-line contract, including PERF-04 and reporting rule. |
| `projects/openclaw-rust/evidence/device-token-rotation-wire-source-647e0932-20261103.json` | Unchanged R3; source identity/range carrier | U identity, source-record/range declarations, acquisition corrections, remaining scope and false verification/acceptance flags. No promotion of historical read windows. |
| `source/host/src/device_pairing_owner.rs` | Changed production helper; approve | Entire R3 hunk plus deep canonicalizer, scope policy, direct consumers, issuance, admission/currentness, legacy DPA and original product diff. Unrelated owner operations are contextual. |
| `source/host/src/gateway_server.rs` | Unchanged R3; affected consumer context | DTR registration/classification/bound hook, ingress guards, reporting branch, retained event/bridge admission and original DTR diff. No complete reread of unrelated HTTP implementation. |
| `source/host/src/host_gateway_api.rs` | Unchanged R3; registration context | Exact DTR inventory/owner-map addition and adjacent method registration contract. |
| `source/host/app/src/main.rs` | Unchanged R3; production wiring context | DTR dispatch/hook, adjacent DPA/DRT/currentness/node authorization, existing node projections and test registration. No full unrelated App reread claimed. |
| `source/host/app/src/tests/device_pairing_revocation_tests.rs` | Unchanged R3; reusable fixture context | Original-scope visibility-only helper changes and their DTR call sites; no new assertion weakening in those hunks. Not a fresh full deep read of every old App test. |
| `source/host/app/src/tests/device_token_rotation_wire_tests.rs` | Unchanged R3; fresh full source read | All 173 lines; both actual projection-helper cases and source-binding mutation case; bounded claims retained. |
| `source/host/tests/device_token_rotation_wire_authority_contract.rs` | Unchanged R3; focused oracle context | Complete test-name inventory plus normalization, decisive-fallback, role-union, NEL/BOM, producer order, pending identity, primary materialization/withdrawal/rollback cases. Not a fresh full deep read of all 44 bodies. |
| `source/host/tests/device_token_rotation_wire_gateway_contract.rs` | Changed test/oracle; approve | Entire R3 delta, complete relevant fixture/TCP/snapshot/hook/report helpers, large-request case, NEL multi-method loop and positives, malformed-storage 500 reporting. Other unchanged cases retained as context and named selections. |
| `source/host/tests/device_pairing_retained_stream_contract.rs` | Unchanged R3; lifetime consumer context | Test-name inventory, bound rotation helper, selected-role successor/reconnect and primary-only reopen/withdrawal cases, plus production admission counterparts. Remaining unchanged bodies are not newly deep-read claims. |
| `source/host/tests/gateway_protocol_contract.rs` | Unchanged R3; protocol registration control | Original DTR count, exact owner and case/extra-suffix rejection additions. |
| `.github/workflows/rust-desktop-runtime.yml` | Shared integration changed; approve preservation | Complete relevant R3/publication hunks, exact DTR and App selections, count/exit/subject checks, incoming after-App historical step and failure capture/upload. Unrelated workflow lanes are not freshly accepted. |
| `projects/openclaw-rust/parity-ledger.json` | Unchanged R3; accounting context | DTR responsibility additions, token-utils range correction with historical retention, invariants and open-risk text. Unrelated media/Runner ledger edits outside DTR remain context. |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | Unchanged R3; traceability context | DTR spec/test/App evidence references in affected requirement rows; no execution or acceptance credit inferred. |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | Shared integration changed; approve preservation | Relevant R3/publication hunks; complete DTR/retained name selections and execution/receipt acceptance logic; retained PRID/MSI/SGF contracts. Not a new verdict on unrelated case implementations. |

## Fixed-U source and remaining acceptance limits

The frozen 47-record source carrier is an identity/range record, not evidence that this reviewer deeply read all 47 files. I additionally read exact exported U content for `src/shared/device-auth.ts:1-34`, `src/infra/device-pairing-token-utils.ts:1-62`, `src/gateway/server-methods/device-management-authz.ts:1-70`, and the relevant normalization-core string-list functions (`string-normalization.ts:24-118`, with surrounding context). Read-only Git object hashing of those exported files returned their immutable blob identities, including `0fe497d...`, `7bb56b7...`, `6d4e874...` and `9466f1b...`; no upstream program was run. These sources corroborate separate implication/sorting, metadata, exact admin membership and list normalization responsibilities. They do not close the carrier's other partial/unread windows or global dependency review.

Scoped source authority/coverage, exact published-H executable evidence, external WebSocket/device/SDK behavior, issuer-generation policy, browser credential cache replacement, exact upstream node-surface retention, wake/APNs/remote worker behavior, supported platforms, real services/devices, packaging and all final OCRM gates remain unaccepted as declared. Source approval of this narrow R3 fix does not erase any of those obligations or turn selected 44/14/27/3 case counts into observed passes.
