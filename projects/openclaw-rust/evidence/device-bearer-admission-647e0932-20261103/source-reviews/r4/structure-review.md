# DFA round 4 and MSU recovery round 1 — independent structure, reuse and performance review

Review date: 2026-11-03 UTC. Reviewer task: `/root/dfa4_media1_structure_review`.

**Verdict A: APPROVED_SOURCE_ONLY for the unchanged original twelve-path DFA scope.** No blocking or non-blocking concrete finding was established in this lens.

**Verdict B: APPROVED_SOURCE_ONLY for the separate complete three-path MSU recovery scope.** No blocking or non-blocking concrete finding was established in this lens.

These are two independent source verdicts. Neither establishes executable success, `verified`, global acceptance, or permission to transfer historical execution results to a new H. Required new-H GitHub Actions and all remaining acceptance gates are pending.

## Review authority, method and limits

All reviewed repository source and context came from immutable Git objects at tree `f3a5f945195bd82f32537a6740817b5bdbf45857`, using `git show`, `git grep`, `git ls-tree` and exact-revision diffs. The mutable worktree was not used as source authority. I read both supplied complete patches and both supplied narrower deltas, the manifests and required cases; directly recomputed the four patch hashes from Git with the declared path sets; and compared every declared candidate member's blob identity with `git ls-tree`.

The frozen root `AGENTS.md` was read, including the complete-scope/adjacent-consumer independent review contract, model policy and OCRM execution restriction. Applicable OCRM authority was read from the current authority/constraint portions of `projects/openclaw-rust/SOURCE_OF_TRUTH.md` and both §17 records in `docs/specs/openclaw-rust-equivalence-migration.md`, including the current OCRM-EXEC-02 amendment. Both scoped specs were read completely:

- `projects/openclaw-rust/docs/specs/device-bearer-approval-admission.md`, DFA-01 through DFA-10.
- `projects/openclaw-rust/docs/specs/media-settlement-unknown.md`, MSU-01 through MSU-10, with particular attention to the scoped MSU-06/MSU-09 recovery requirements.

Actual launch settings were **`gpt-6.1-sol`, `xhigh`, `fork_turns=none`**. Root confirmed the host-applied `spawn_agent` fields and successful tool response during this review; this is launch metadata confirmed by root, not runtime model self-inspection. Xhigh applies to the whole assignment because shared authority, lock-held complexity and transaction-oracle recovery require interacting source judgments. No model fallback was reported.

The review was fresh and blind: I did not read prior/peer reviews, arbitration decisions or implementation reports. Reading the declared manifests and immutable prior-tree fix deltas supplied the scope, not another reviewer's conclusion. No repository edit, parser execution, build, test, lint, generator, benchmark, local/htch validation, or Actions dispatch was performed. Hashing and read-only Git identity/diff operations were the only mechanical checks. The only authored file is this report.

Reading limits are explicit: this review covers every changed hunk in both complete declared scopes and the functions, callers, consumers and build/capture contracts required to judge them. It is not a claim to have read every unchanged line of the large owner/test/workflow/ledger files, every historical Source-of-Truth drift appendix, or the full pinned OpenClaw tree/dependency closure. The unchanged policy implementation was traced locally; no new upstream semantic-reading completion is claimed. Source-oracle assertions were inspected, not executed. In particular, compile validity, actual fault application, real HTTP outcomes, wall-clock performance and platform behavior remain executable claims for Actions.

## Frozen identities and evidence hashes

Common subject:

| Identity | Value |
| --- | --- |
| Repository | `/workspace/scratch/29e7135d8bfe/fabushi-desktop` |
| Candidate tree | `f3a5f945195bd82f32537a6740817b5bdbf45857` |
| Publication parent | `14faff5d9605c99df1494303f1aa0ce31bcfd420` |
| Publication-parent tree | `518e019acc61861af909fcfd48a5c103a7ac7fe3` |
| Fixed upstream U | `1440bb1771372334bea23d7642bb52deb7db1845` |
| Fixed upstream tree | `90c8aab39972da11cfad220361781d49e1bc6700` |

Supplied artifacts are under `/workspace/scratch/29e7135d8bfe/reviews/device-bearer-admission-r4/`. Their byte SHA-256 identities, directly read with `sha256sum`, are:

| Artifact | SHA-256 |
| --- | --- |
| `manifest.json` | `de1878296297e373a78a92fff634c4afb2dbcb051d0ecd7c1e35189ab8494e7f` |
| `complete.patch` | `916513e6f74129540cc79e5cae3dc637bc0424d0d1c63f6b9918d2eb547d2147` |
| `focused-delta.patch` | `1224d0201e52c2865adbe20a081bb618ba97f993ea60fe279b923e33bdbb7136` |
| `required-cases.json` | `fd1c10e2d310a6f0607f337848bc33c559022429e275e53ca088d0dcd12e2fc4` |
| `media-manifest.json` | `66326d8151048e121e2da05854b71c513d84800769a7f2ca7a7f28a15280895c` |
| `media-complete.patch` | `0499dc632ac0a9707dda1b4b466fe7d9cccebe72b463f2f3a34968aef152451f` |
| `media-integration.patch` | `e018b1cac3175d058d71bbfe5e38d95d7bd280ec9f107b87cdcc27c5fe08d009` |

The complete A Git diff against `fa75f9ccd66e43f3a4d72ab47acdefaa832212f6`, focused A Git diff against `0c2fcb5d8c8d6c7d6d348543eed955541ca2cd2b`, complete B Git diff against `37ffd42410d37013512e8efbf3bda5b549f3a4db`, and integration B Git diff against publication parent `14faff5d9605c99df1494303f1aa0ce31bcfd420` independently produced the corresponding four supplied patch hashes above. No scope was replaced with the publication delta alone.

## A. DFA round 4 — exact scope and source assessment

Base and starting HEAD are both `fa75f9ccd66e43f3a4d72ab47acdefaa832212f6`. Previous round tree is `0c2fcb5d8c8d6c7d6d348543eed955541ca2cd2b`. The complete patch is 100,754 bytes and focused delta is 12,714 bytes. The original twelve paths remain the review scope, including the App prerequisite file even where the publication parent already contains its content.

| Declared path | Candidate blob |
| --- | --- |
| `projects/openclaw-rust/docs/specs/device-bearer-approval-admission.md` | `f99fe2e81da2822ef7dd871bceba1215d3b0229e` |
| `source/host/src/device_pairing_owner.rs` | `9825003cf1f7047027aa1b8e496c40f0c1693a23` |
| `source/host/tests/device_pairing_rotation_authority_contract.rs` | `a06e0593cf3be3673e91706e1636399dab90d0a8` |
| `source/host/tests/device_pairing_revocation_authority_contract.rs` | `212b1067c4ffed3523aec5ebb2c3104eafe4c86f` |
| `source/host/tests/device_token_rotation_wire_authority_contract.rs` | `7c37e8093b8b7f2975eb031d76dae1b1419ed13c` |
| `source/host/tests/device_token_rotation_wire_gateway_contract.rs` | `e7c42f88067d8d4695ff7ee356987e493b1f4224` |
| `source/host/app/src/tests/config_secret_boundary_tests.rs` | `8f3405e6335dac35e6df87674241ad4d2df2dc07` |
| `source/host/app/src/tests/local_media_file_url_tests.rs` | `57a0a3393f1bb9560eb036687688656b1f3fe6c8` |
| `.github/workflows/rust-desktop-runtime.yml` | `ede3b6b9586383a2d0245de933d6ff4417e6ff10` |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | `f5f3d5624568cb553441b8ca0c0f63aff76ebeae` |
| `projects/openclaw-rust/parity-ledger.json` | `285f9ca672947d9ad054832557c0b4abf38a0d68` |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | `1bdbd1ca17b2ff334f365183c51bcd15c7ca0871` |

### Shared owner, policy and lock boundaries

`DevicePairingOwner::authenticate_device_token` constructs the same owner-issued sealed proof after the existing constant-time bearer comparison and active-marker rule, then calls `validate_current_device_authorization_in(&conn, &authorization)` before returning it. The public retained validator acquires its own connection lock and calls that same private function (`device_pairing_owner.rs:611`, `:1559`, `:1567`). There is one policy implementation and one existing connection; fresh admission does not recursively acquire the mutex, create a second authority store, write activity/revision state or hold the pairing lock across Settings dispatch.

The fresh boundary matches the typed `GatewayCommandError::Unauthorized(_)` variant and maps only that result to `Ok(None)`. Every other error propagates. Retained callers still receive the validator's original error. `current_pairing_record` checks the private issuer, exact current role entry, active marker and token fingerprint; it reads only `COALESCE(approved_scopes_json, scopes_json)`, preserving decisive-baseline versus unused-fallback semantics (`:1605`). The grant validator still parses the entire live grant and approval, checks all live scopes against approval, then checks all originally admitted scopes against both live and approval grants. The public mutable projection does not become policy authority.

### Scope and role complexity/compatibility

The scope change indexes the canonical live and approved vectors once as local `BTreeSet<&str>` values and passes membership closures into the existing `role_scope_policy_allows` function (`:94`, `:105`, `:1579`). It does not duplicate or alter operator admin/write/read implications, literal NEL/BOM scope spelling, exact nonoperator role prefixes, projection order or the short-circuit check order. For fixed string-comparison costs, the repeated membership work is logarithmic per checked grant rather than scanning the complete grant vector for each scope; construction and checking are on the order of `(L + A + P) log(L + A)`, where L/A/P are live/approved/admitted scope counts. Existing normalization and the pre-existing bearer-row scan remain separate costs. No persistent cache, count cap or benchmark claim is introduced.

The focused role fix in `normalized_authority_strings` (`:74`) uses a local borrowed-string membership set only to decide whether to append a normalized value. It still calls `as_str()?` and Rust `str::trim` for every element, filters empties, and appends owned strings on first encounter in source order. A selected role encountered early cannot hide a later non-string. The set is dropped after parsing, without changing stored bytes or role vocabulary. I traced its callers through stored approved-role parsing, selected-role revocation, wire rotation and legacy DPA desired-scope parsing (`:87`, `:249`, `:1342`, `:1453`, `:1782`); the former vector-dedup semantics are preserved at each seam. This removes the role-array duplicate-search quadratic behavior without changing those callers' adapters.

### Behavioral oracles and regression preservation

The three new owner cases in `device_pairing_revocation_authority_contract.rs` remain actual file-backed owner controls. They cover approved-role withdrawal, map/entry-role mismatch, complete-grant implications and literal spelling, corrupt/missing/wrong SQL types, legacy approval fallback, issuer separation, repair and reopen. The 20,000-scope/20,000-role fixture admits a real proof, validates it through the retained API, rejects a final non-string role with the fixed internal error, rejects a single unapproved tail scope, preserves raw state and an unrelated credential, then admits the repaired state after reopen (`:1151` onward). State helpers take independent complete-row observations; they do not substitute the production validator.

`fresh_bearer_http_config_mutation_requires_current_approval` uses the real TCP `GatewayServer`, real file-backed `DevicePairingOwner`, and the actual App `dispatch_public_settings_command`/`SettingsService` consumer. The wrapper delegates authentication and Settings dispatch rather than supplying a second policy. It compares dispatch/commit/completion counters, all pairing/pending/machine rows, Settings bytes and directory entries on refusal, and reads committed settings through a new Settings owner on positives. Role/grant withdrawals refuse at 401; decisive malformed authority refuses at 500; a valid read credential is separately admitted and refused at the command permission boundary at 403. Native configured-secret and credential-absent loopback positives are retained. Large valid authority, malformed role tail, unapproved scope tail, unrelated writer and repaired reopen use the same ordinary HTTP path (`config_secret_boundary_tests.rs:477` onward).

The fixture closes its server, drops its own API Arc and waits for the API Drop acknowledgment after the pairing owner has been dropped before reopen/removal. `GatewayServer::close` joins the listener, while handler-owned dependency Arcs keep the API alive until handlers exit (`gateway_server.rs:676`, `:789`). The fixture acknowledgment therefore supplies the needed additional lifetime boundary; a timeout prevents fixture deletion. This does not claim physical-device or packaged lifecycle acceptance.

Shipping bindings were traced beyond the test wrapper: `UnifiedGatewayApi` authentication delegates to its `device_pairing`; config dispatch reaches `call_public_settings_command`, which invokes the same Settings helper; composition supplies the existing device owner and shared Settings extension; the running Gateway uses that API (`app/src/main.rs:1782`, `:4436`, `:13463`, `:17528`, `:17825`). The separate source-binding case mutates each actual binding and asserts the mutation changes the source before expecting refusal. It remains explicitly source evidence, with affected App compilation still required.

The compatibility changes retain genuine proof acquisition before the invalidating transitions. The DPA empty-approval case uses one explicitly named unselected invalid credential, checks it was encountered exactly once, preserves all other global all-current assertions and also checks the earlier node proof is stale. Reopen cases now check fresh refusal directly instead of using an old-owner proof as a proxy. Valid retained implication cases acquire the intended live grant before obtaining their proof. I found no removed decisive management/retained assertion in the declared deltas.

The workflow and portable capture retain the existing targets/selections and add the three exact required owner names. Both exact App names in `required-cases.json` are selected and required in the App receipt. Inputs include the new spec and existing owners; receipts distinguish runtime/helper, shipping source binding and incomplete acceptance. The separate local-media prerequisite changes only explicit serde imports and the existing floating-point timestamp argument. The ledger/RTM additions retain incomplete states; reference/counter reconciliation in the complete scope does not promote execution or acceptance.

**A findings:** none. **A source verdict:** approved for this structure/reuse/performance lens. Required owner, HTTP, App, shipping feature/compilation and management/retained Actions remain pending at the publication H.

## B. MSU recovery round 1 — exact scope and source assessment

Base and starting HEAD are both `37ffd42410d37013512e8efbf3bda5b549f3a4db`. The complete patch is 28,565 bytes. The integration delta against `14faff5d9605c99df1494303f1aa0ce31bcfd420` is 23,639 bytes. The complete three-path recovery, including the DB fixture change already present in the publication parent, was reviewed independently of A.

| Declared path | Candidate blob |
| --- | --- |
| `projects/openclaw-rust/docs/specs/media-settlement-unknown.md` | `8aee21c69bab72824025e4304eec38388ba1e9f8` |
| `source/host/src/extensions/session/agent_db.rs` | `87186bc4b246d88a5ce51ebda0623b76f4425d3d` |
| `source/host/tests/openclaw_sessions_and_config_contract.rs` | `88a80214aa08a1d61b3bd290063069d5aa4d417e` |

### Complete DB image without a projection change

The `agent_db.rs` changes are inside `#[cfg(test)] mod media_settlement_tests`. `Fixture::expected_committed_entries` independently constructs the ordered send-plus-receipt vector from fixture constants/identity/path; it does not call the production receipt builder. `assert_committed_storage` still compares the independent file-backed raw SQL row image and released admission record, and still checks original media bytes (`agent_db.rs:9189`). The two corrected getter expectations reuse that same full expected vector in the lost-ACK two-reopen loop and exact-other-owner fault isolation control (`:9364`, `:9583`). Retirement counts, one-shot fault observations, no-publication checks, admission/fence assertions and retained-file checks remain present.

This expected image matches the actual ownership boundary: `SandAgentDb::get_transcript_entries` reads `read_transcript_entries_from_db`, whose owner parser includes valid settlement receipts in sequence order (`:5843`, `:7777`). The receipt is inserted after the actual send and before the fence CAS in the same transaction (`:4990` onward); its production fields are defined by `media_settlement_receipt_entry` (`:6671`). Filtering belongs to Session's `hide_session_work_receipts` and its presentation reads (`extensions/session/production.rs:40`, `:1289`). No getter, filter, receipt format, transaction or shipping outcome is changed by this scope.

### Inherent-owner selection with a complete global scan

`media_commit_inherent_impl` now excludes trait implementations from the candidate set before demanding a single unconditional, exact unqualified inherent owner. Duplicate/foreign/conditional inherent implementations still fail. The token-boundary parser in `media_commit_owner_is_handled` likewise skips parsed trait items before manually parsing an inherent `impl` header (`openclaw_sessions_and_config_contract.rs:1693`, `:1740`). This resolves the actual `impl Drop for SandAgentDb` incompatibility while retaining the same one-owner/method/signature proof.

Crucially, neither change deletes trait source. `media_commit_checked_method_source` still removes only the exact whole COMMIT method after reparsing proves it belonged to the checked inherent helper. The media rollback method stays in the remaining source. The only call-site normalization is the previously proven exact `commands.commit(db)` expression, with one occurrence required. The remaining complete source then enters the existing channel command/caller oracle (`:1871`, `:1889`). The channel `Some(1)` caller-count rule remains unchanged. Its recursive token scan still covers trait bodies, aliases, raw identifiers, associated/method references and unresolved macro input; the original exact raw-COMMIT prohibition runs on the final remaining source (`:2286`, `:2371`, `:2448`). The bounded exception is not widened to an entire impl, trait or source region.

The positive case requires the actual complete Drop baseline and checks it remains in the global scope. Harmless traits for both owner types are retained byte-for-byte in the normalized source remainder. Trait-only substitutions for the helper and DB owner refuse. Added direct/aliased/raw-identifier trait callers must refuse for both owner types; added raw-COMMIT trait bodies must remain as exactly one forbidden statement in the final source. These test structures distinguish candidate selection from global safety scanning rather than suppressing traits from both.

### LF/CRLF and non-vacuous controls

`media_commit_source_image` normalizes CRLF to LF only in the oracle-local source image (`:1865`). Actual-owner mutation tests build their substitutions from that same normalized image before invoking the proof, so a Windows checkout cannot make an LF literal substitution silently do nothing. The actual-owner positive requires identical normalized remainders for LF and CRLF; trait coexistence, changed settlement-tail branches, direct/aliased trait callers and hidden raw-COMMIT trait owners have explicit LF/CRLF controls.

Every newly touched destructive media substitution asserts the changed fragment and/or complete changed source differs from its input before asserting refusal. The channel recovery edits add the same non-vacuity checks at the changed actual-source/fixture substitutions. The full retained negative arrays still cover missing, duplicate, foreign, conditional and malformed helper/owner boundaries; changed COMMIT or rollback commands; changed/forgotten authorizer cleanup; discarded/conditional settlement tails; changed rollback/autocommit/retirement/outcome branches; wrong closure entry; poisoned/prior/unsettled-BEGIN handling; extra direct/aliased/associated/macro calls; and raw COMMIT before, after, inside sibling methods, inside write closure and in traits (`:1952` through `:2191`, `:2499` through `:2802`). I inspected these controls and their parser flow; they were not run.

Raw evidence custody is separate from the oracle-local newline image. The adjacent unchanged `capture_media_cleanup.py::inputs` hashes original `read_bytes()` data and records raw SHA-256, committed blob and normalized checkout blob (`capture_media_cleanup.py:341`). Its blob is `d5376d94f569b112c5778efb529eb64d2b863766` and is unchanged from B's base. `TRANSACTION_COMMIT_NAMES` remains the same 27-name list, and `transaction-commit-owners` still invokes the same `commit` filter. `assess` requires nonempty unique required names, exact observed names/count, zero failures/ignored and successful command/capture exits (`:228`, `:358`, `:518` vicinity). No selected name or result-count requirement is dropped. The separate thirteen DB settlement controls and App/feature compilation requirements also remain separate evidence obligations.

This recovery adds no product abstraction, dependency, persistent state or new runtime owner. It changes test source recognition and test expectations at the owning boundaries. Full-source image allocation/parsing remains test-only work; no shipping latency or memory path is added. No timing, performance or successful platform-execution result is inferred from the source review.

**B findings:** none. **B source verdict:** approved for this structure/reuse/performance lens. Exact-H LF/CRLF oracle execution, all unchanged 27 selected transaction cases, thirteen DB controls, affected App cases and shipping-feature evidence remain pending, together with the original full acceptance obligations.

## Remaining acceptance boundary

Both manifests explicitly retain `unitVerified=false` and `globalAcceptance=false`; this review does not change them. Publication must obtain its own H-bound Actions artifacts and named positive/negative outcomes, including the actual owner/HTTP/Settings behavior for A and receipt/fault/oracle behavior for B. Original OCRM G01–G12/AC01–AC24 and real device/provider/channel/platform/package/upgrade/rollback/independent acceptance requirements remain open. No preference-only cleanup is proposed.
