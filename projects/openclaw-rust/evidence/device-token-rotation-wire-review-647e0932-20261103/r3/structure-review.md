# DTR-v1 R3 — independent structure, reuse and performance source review

Verdict: **APPROVED for the assigned focused source-review lenses; no blocking findings.** This verdict concerns the frozen DTR source and authored verification contracts. It supplies no runtime pass, scoped-authority acceptance, unit-verified status, upstream closure, packaged acceptance or global acceptance.

## Identity and independence

- Task: `647e0932-bea4-4113-9a61-65ae3db58955`, task round 1, review round 3.
- Repository: `/workspace/scratch/29e7135d8bfe/fabushi-desktop`.
- Reviewed immutable tree: `a66145d238e8dbfef934b9cba7891d3ee02f05ce`.
- Publication comparison base: `fee4f179b640489774208fa3ce653a3868b641bf`.
- Focused change comparison: `7644b264bf72ef57f7333a7f4f8d7accd2dd7af8` to the reviewed tree.
- Complete-scope comparison: original frozen base `497c2c7a7bc5201aead83d3d36e44f52ba164d04` to the reviewed tree; original behavior overlay `f6aaaf556fe99d53b46060da46691e154bffdfac` was supplied as lineage, not execution evidence.
- Scope manifest: `/workspace/scratch/29e7135d8bfe/reviews/dtr-r3/manifest.json`; all 16 declared identities are dispositioned below. The complete, publication and fix-delta patches remain available in that directory.
- Fixed upstream: `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`.
- Actual configured review worker: **`gpt-6.1-sol`, `xhigh`**, as applied at worker launch. The scope requires shared grant normalization, request authority, persistent state, listener lifetime and shared CI integration analysis. This is the root AGENTS reuse/structure/performance model pin; no model fallback was used.
- Fresh independent worker. No peer/prior review reports, root arbitration/decision records, performer handoffs or unrelated evidence verdicts were consulted. No subagents were used.
- Product source reads used `git show a66145d238e8dbfef934b9cba7891d3ee02f05ce:<path>` and immutable Git diffs. The exported upstream source cache was used only as the supplied exact-source reference, not treated as a Git checkout or a new source authority.

The parent separately reported publication at `0c7da50fcf457184f38d6a61b73f8e32810cff49`, tree `77aee57690a8d09cdf794f6459412ae26f604f1a`, after four unrelated plugin commits. That notice does not change this review subject or establish runtime evidence. This reviewer did not assess those plugin changes as DTR acceptance.

## Findings

**None.** No concrete failure scenario in the focused R3 change or its examined affected callers warranted a blocking or nonblocking structure/reuse/performance finding. There is no requested product fix from this review.

## Focused reasoning and evidence

### Shared normalization and work bound

`source/host/src/device_pairing_owner.rs:121-166` retains one private canonicalizer in the existing pairing owner. The R3 change at lines 132-141 introduces a local `BTreeSet<&str>` for membership while the existing `Vec<String>` retains first-seen canonical output. The code never emits the tree's iteration order. Each accepted unique trimmed input is copied once into the returned vector; duplicate tree membership does not rescan every prior output string.

For `n` input strings and `u` unique canonical values, membership performs `O(n log u)` ordered-set operations, with string comparisons dependent on string length, plus trimming over the input text. The set holds borrowed slices into the caller's input for this invocation only. Neither the set nor its references escape; the result owns its strings. The explicit lifetime on the iterator makes this the same underlying input lifetime, rather than a reference into the output vector that could move during growth. No shared cache, retained registration, state owner or dependency is added.

The semantic layers remain separate:

- `wire_scope_trim` still explicitly implements the ECMAScript whitespace set, including FEFF and excluding NEL.
- `canonical_scope_strings` only trims, drops blank canonical values and deduplicates in first-seen order.
- `normalized_wire_scopes` alone adds the specified admin/write implications and then compares `encode_utf16()` sequences for final DTR ordering.
- Native identity and role arrays retain the existing Rust trimming path through `normalized_authority_strings`; that helper was not redirected to the scope adapter.
- Explicit legacy DPA request scopes continue to use `normalized_authority_strings` at lines 1755-1765. Omitted legacy scopes and persisted credentials use the credential canonicalizer. Exact grant comparison in `role_scope_allows` does not trim a newly issued NEL literal into admin authority.

Affected existing callers were traced through `scopes_for_role` (303-331), native approval issuance (975-1004), fresh authentication (610-675), stored baseline reads (246-257), currentness and paired admission (1551-1666), legacy DPA issuance/defaulting (1684-1789), and pending request comparison (333-381, 741-790). These callers receive the same canonical first-seen values as before R3, with improved deduplication membership work. The existing role and scope policy ownership remains in this file.

This is a proof of the changed normalization bound, not a claim that every authorization operation is globally `O(n log n)`. Existing policy comparisons over grant vectors have their own costs. No new broad performance acceptance is inferred.

The authored owner controls still exercise FEFF/NEL, implications and UTF-16 ordering at `device_token_rotation_wire_authority_contract.rs:522-546`; literal baselines and DPA compatibility at 1579-1602; first-seen approval/native/DPA/authentication production at 1605-1639; and pending NEL/BOM identity separation with unchanged native role treatment at 1642-1673. No fresh execution is claimed for them.

### Real large-input HTTP authority oracle

`source/host/tests/device_token_rotation_wire_gateway_contract.rs:897-932` sends 100,000 distinct scope strings through real TCP to `/api/device.token.rotate`, after asserting that the serialized body fits the unchanged `MAX_BODY_BYTES`. It uses the real current CALLER read-only credential and supplies no narrowing scope header, avoiding the existing explicit-read header prefilter.

The adapter at 276-293 increments the bound hook witness and delegates the same arguments, optional proof and ceiling directly to `DevicePairingOwner::rotate_token_with_authority`. The case requires exactly one recognized credential and bound call, no generic/native fallback or other management call, a paired authenticated operator context with no ceiling, and the exact owner-specific pairing-authority 403. In the production owner, parsing precedes the transaction and admission; denial occurs before target mutation at 1407-1427. The Gateway keeps the method exemption from the old literal-write prefilter and reaches the bound hook at `gateway_server.rs:1893-1917,1992-2024`.

The refusal oracle is independent of the response: `observe` at 465-475 opens a separate read-only SQLite connection and captures every column of paired, pending and configuration rows with revision SQL value/type witnesses. `deny` at 529-539 compares that complete snapshot and checks call routing. The new test then tears down the actual API/listener, reopens the same owner, compares the entire snapshot, freshly authenticates the original credential and validates its currentness. Its positive control uses the already issued TARGET pairing credential on that reopened owner, observes a real selected replacement and checked revision through the existing `assert_rotation` oracle, and validates the resulting proof as current.

The fixture's existing socket/deadline bounds are cleanup and transport bounds. The test does not assert an elapsed-time performance threshold, sleep for acceptance, replace the normalizer with a stub, or accept a timeout as a policy refusal.

### Corrected ordinary-refusal reporting oracle

The R3 correction at `device_token_rotation_wire_gateway_contract.rs:1166-1198` matches the production reporting boundary at `gateway_server.rs:2043-2072`: successful commands report completion; only statuses at least 500 report command errors. Ordinary 403 refusals report neither.

The change removes an impossible expectation of a 403 error callback and replaces it with unchanged error-report and success-report counts. It retains actual credential recognition, method-specific bound-hook counts for DPA/DRT/DTR, exact 403 status, complete SQL equality, response and report secret/fingerprint redaction, and the DTR authenticated/request-ID context witness. Before the refusal loop, the exact issued NEL successor is authenticated, validated as current, and admitted to the ordinary probe. After the loop, it remains current and storage remains at the committed snapshot. Genuine admin positives still traverse all three management hooks; DPA's explicit NEL-positive/FEFF-negative request behavior and ordinary pairing rotation remain present at 1203-1239.

The separate 500 path at 835-854 still requires one real report with method, status, request ID and fixed redacted error. It is followed by repair/reopen/authorized-positive controls. The oracle correction therefore does not weaken the production reporting policy or let a generic refusal substitute for reaching the intended owner.

### Existing owner and lifetime boundaries

The wire method continues through the existing registry, Gateway optional-proof hook, actual App delegation and `DevicePairingOwner`. The default Gateway hook fails closed. Trusted native admission has no paired self identity. The wire issuance implementation at owner lines 1401-1547 reuses the existing mutex, IMMEDIATE transaction, secure token generator, sealed caller policy, checked revision and commit boundary. Identity-bound delivery is decided from the sealed paired identity after that commit. Native/DPA behavior remains explicit rather than being forced through a generic alias with incompatible delivery and metadata semantics.

Currentness uses exact owner issuer/fingerprint, approved role-array membership and the decisive approved baseline. Selected primary-only role materialization commits with the selected token; currentness does not acquire a primary-label fallback that could revive a withdrawn old credential. The R3 membership optimization does not change these boundaries.

The retained target still uses the existing API/currentness hooks and owned bridge registrations. Targeted unchanged context included real bound rotation and reopen helpers (504-597), selected/unselected reconnect continuity (1706-1759), actual quiet heartbeat retirement (1829-1857), and primary-only reconnect/reopen/withdrawal behavior (2090 onward). Those tests distinguish timeout from EOF and preserve unaffected native/other-role controls. This review does not reclassify all 27 retained tests as newly deep-read or executed.

### Shared CI integration

Against publication base `fee4f179...`, the workflow and portable-capture diffs each add the new required HTTP test name and update the descriptive scope text. Their execution, capture and acceptance machinery is preserved.

The portable script retains exact HEAD admission, workflow/script/source input hashing, the real `cargo test --locked` complete-target selections, required positive name uniqueness, command/capture exits, ignored/filtered/measured safeguards and final all-cases receipt aggregation. Its DTR lists are owner44, HTTP14 and retained27 (`capture_portable_contracts.py:493-583,795-803,1034-1086`). Whole-target acceptance requires the complete summary exactly equal to `(len(required), 0, 0, 0, 0)`. Synthetic result-oracle controls at 976-1009 remain assertions on the existing parser and cover false-positive forms including failed/ignored expected panic, duplicate names, decoration/trailing text, zero/missing summary and command/capture refusal.

The incoming PRID source inputs remain at 172-177; its two selected groups remain at 942-952, including the historical exporter/consumer skips for the bounded process selection. Both groups remain in the exact-count branch at 1053-1054. The workflow's separate historical prior-root capture remains after successful actual App work at 4286-4294 and retains failure-artifact handling at 4443 onward. This establishes integration preservation, not independent semantic acceptance of PRID.

The original shared workflow also retains the DPA/DVR/DRT/DTR complete-name/count/exit oracles at 2590-2670, strict actual App selection and execution at 4195-4207 and 4239-4252, and the required three DTR App cases alongside all six DVR/DRT App cases. Existing MSI/SGF/shared Runner selections and source inputs were treated as integration context; the DTR publication diff does not remove or rewrite their commands or result oracles. No unrelated plugin-loader acceptance claim is made.

## All 16 paths: disposition and read limits

Every row is bound to the blob identity in the frozen manifest. “Targeted unchanged context” means source inspection for the affected caller/contract, not a newly completed full-file semantic review.

| Declared path | Disposition and actual review depth |
| --- | --- |
| `projects/openclaw-rust/docs/specs/device-token-rotation-wire.md` | Full current contract read, including R3 performance and reporting amendments, invariants, verification and remaining scope. |
| `projects/openclaw-rust/evidence/device-token-rotation-wire-source-647e0932-20261103.json` | Source-carrier disposition: inspected identity/provenance and read-limit declarations, acquisition correction, dependency and remaining-scope/verification fields. Historical ranges were not promoted into this reviewer's new reads. Not all 47 exported source files were deep-read. |
| `source/host/src/device_pairing_owner.rs` | R3 canonicalizer deep-read with callers; DTR parsing/normalization/issuance/currentness and affected producers/legacy DPA/pending comparison deeply traced. Relevant lines 59-381, 610-675, 685-862, 930-1062, 1370-1825. Unrelated owner tests and operations are targeted context only. |
| `source/host/src/gateway_server.rs` | Full declared DTR/DRT diff inspected; bound trait hooks, classification, role/scope admission, parsing/dispatch and 4xx/5xx reporting traced at relevant unchanged callers. Not a new full-file review of unrelated routes. |
| `source/host/src/host_gateway_api.rs` | Declared registry/owner mapping changes and adjacent method-membership context inspected. Existing DevicePairing ownership retained. |
| `source/host/app/src/main.rs` | Declared native dispatch, bound rotation/revocation hooks and test-module inclusion inspected at frozen identity. Actual shipping source is approximately 990 KB; unrelated App body was not claimed as deep-read. |
| `source/host/app/src/tests/device_pairing_revocation_tests.rs` | Declared fixture reuse/visibility and added shared helper/DRT context inspected; existing projection consumer boundaries retained. Unchanged in R3. |
| `source/host/app/src/tests/device_token_rotation_wire_tests.rs` | Full file read (1-173): reuse of existing durable projection fixture, two owner/projection cases and mutation-qualified source-binding supplement. These are not full-App HTTP execution claims. |
| `source/host/tests/device_token_rotation_wire_authority_contract.rs` | Targeted unchanged context: independent SQL/credential fixture and complete authored selection names; deep inspection of normalization, literal baseline, first-seen producer, legacy adapter and pending identity controls (not all 44 test bodies). |
| `source/host/tests/device_token_rotation_wire_gateway_contract.rs` | R3 100,000-scope case and reporting correction deeply reviewed with real transport/owner/state/lifetime helpers, 500 report controls, existing positives and relevant delivery/race context. Other unchanged cases were targeted context. |
| `source/host/tests/device_pairing_retained_stream_contract.rs` | Declared fixture/bound-hook changes and targeted DTR rotation/currentness/reopen/quiet/primary-only context reviewed. Full set of 27 selections retained; not all 27 bodies newly deep-read. |
| `source/host/tests/gateway_protocol_contract.rs` | Declared exact membership/count/owner and negative-name additions inspected; frozen inventory remains a separate existing contract. |
| `.github/workflows/rust-desktop-runtime.yml` | R3 and publication diffs inspected; DTR complete-target/name/count/exit logic, actual App source inputs and selections, portable entry, and incoming after-App historical capture deeply traced. Unrelated platform/media/plugin jobs are context, not newly accepted. |
| `projects/openclaw-rust/parity-ledger.json` | Targeted DTR existing responsibility and additions inspected. Owner, risks, source-range correction, invariants, tests and unaccepted scope remain explicit. Unchanged from previous R3 comparison/publication base; unrelated ledger additions are dispositioned as integration context. |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | Targeted DTR evidence/requirement additions inspected. No R3 publication diff and no new verified/acceptance promotion inferred. Unrelated RTM rows not newly semantically reviewed. |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | R3/publication changes plus relevant full input catalogue, DTR lists/commands and result/exit aggregation reviewed; incoming PRID groups and parser controls preserved. Other domains' test bodies were not reviewed by reading their names. |

Applicable governance read: root AGENTS (including independent immutable review, model/effort and execution rules); OCRM SOT current authority/scope sections; main spec §17; current pairing task; module-map ownership/status and M03/M04/M13/M32; full DTR contract. Historical discovery appendices were treated as historical provenance, not current authority.

Selected fixed-U source independently inspected for this lens: `src/shared/device-auth.ts` (full), `packages/normalization-core/src/string-normalization.ts` (full), `src/infra/device-pairing-token-policy.ts` (full), `src/gateway/server-methods/device-management-authz.ts` (full), and targeted rotation/kernel/handler windows in `src/infra/device-pairing-tokens.kernel.ts` and `src/gateway/server-methods/devices.ts`. Some large search/diff outputs were truncated; no content beyond the actual inspected windows is credited as newly read. The other exported upstream files remain carrier/catalogue context for this review.

## Verification boundary

No local or `htch-runtime` build, test, lint, rustfmt, schema check, generator, harness or benchmark was run. No GitHub mutation was performed by this reviewer. The sole written output is this review file. Original 113-case runtime failure history was not used as R3 evidence. Exact-published-H Actions evidence and independent acceptance remain required under the existing contract; this source verdict must not be used to claim their completion.
