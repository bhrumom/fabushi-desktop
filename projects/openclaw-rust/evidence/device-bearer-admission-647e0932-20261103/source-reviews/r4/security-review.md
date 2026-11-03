# Frozen source review — DFA round 4 and MSU recovery round 1

Date: 2026-11-03. Reviewer: `/root/dfa4_media1_security_review`. Lenses: security, general correctness, ownership and lifetime. This is a fresh blind source review, with two explicitly separate scopes and verdicts.

**Scope A verdict: APPROVE, source-only.** No concrete blocking defect found in the original twelve-path DFA scope at the candidate tree below, including its complete original delta and round-4 focused repair.

**Scope B verdict: APPROVE, source-only.** No concrete blocking defect found in the separate three-path MSU recovery scope at the same candidate tree.

These verdicts concern the inspected source changes and their declared causal controls. They are not runtime results, unit verification, scoped G01/G02 approval, publication approval, or global acceptance. **Actions execution, runtime verification, and global acceptance all remain pending.**

## Identity and review discipline

Repository: `/workspace/scratch/29e7135d8bfe/fabushi-desktop`.

| Identity | Exact value |
| --- | --- |
| Candidate tree, both scopes | `f3a5f945195bd82f32537a6740817b5bdbf45857` |
| Publication/integration parent | `14faff5d9605c99df1494303f1aa0ce31bcfd420` |
| Publication parent tree | `518e019acc61861af909fcfd48a5c103a7ac7fe3` |
| Scope A original base and starting HEAD | `fa75f9ccd66e43f3a4d72ab47acdefaa832212f6` |
| Scope A original base tree | `ba707272d259b7da5f0def5f31eb3380ce6fd3e8` |
| Scope A previous round tree | `0c2fcb5d8c8d6c7d6d348543eed955541ca2cd2b` |
| Scope B original base and starting HEAD | `37ffd42410d37013512e8efbf3bda5b549f3a4db` |
| Scope B original base tree | `dc9af82ac4a14937d92e4c230c1cbb1e7585afab` |
| Declared fixed upstream U | `1440bb1771372334bea23d7642bb52deb7db1845` |
| Declared fixed upstream tree | `90c8aab39972da11cfad220361781d49e1bc6700` |

Actual host-applied launch settings were confirmed by root from the accepted spawn call: **model `gpt-6-astra`, reasoning effort `xhigh`, `fork_turns=none`**. This is host-confirmed launch metadata, not child runtime self-inspection. Xhigh is appropriate because this assignment judges bearer authority, sealed-proof lifetime, transaction settlement and oracle recovery.

All reviewed repository content was obtained from immutable `git show`, `git diff`, `git grep` or `git ls-tree` against the declared identities. The mutable checkout was not used as implementation authority. I read neither prior/peer review reports nor root decision ledgers nor implementation reports. Report filenames appearing in tree listings or canonical evidence references were not followed. No repository file was edited. No local or htch build, test, lint, parser execution, generator, benchmark or harness validation was run. Shell operations were limited to source/text/identity reads, hashing and writing this report.

The following review-input SHA-256 values were read directly with `sha256sum`; the patches match the supplied manifest hash declarations:

| Input under `reviews/device-bearer-admission-r4/` | SHA-256 |
| --- | --- |
| `manifest.json` | `de1878296297e373a78a92fff634c4afb2dbcb051d0ecd7c1e35189ab8494e7f` |
| `complete.patch` | `916513e6f74129540cc79e5cae3dc637bc0424d0d1c63f6b9918d2eb547d2147` |
| `focused-delta.patch` | `1224d0201e52c2865adbe20a081bb618ba97f993ea60fe279b923e33bdbb7136` |
| `required-cases.json` | `fd1c10e2d310a6f0607f337848bc33c559022429e275e53ca088d0dcd12e2fc4` |
| `media-manifest.json` | `66326d8151048e121e2da05854b71c513d84800769a7f2ca7a7f28a15280895c` |
| `media-complete.patch` | `0499dc632ac0a9707dda1b4b466fe7d9cccebe72b463f2f3a34968aef152451f` |
| `media-integration.patch` | `e018b1cac3175d058d71bbfe5e38d95d7bd280ec9f107b87cdcc27c5fe08d009` |

## Scope A: DFA authority and lifetime assessment

The complete twelve-path diff was reviewed, with direct immutable rereads of the modified implementation, new tests, changed existing assertions and relevant consumers. The focused round-4 delta was read independently of the complete delta.

`DevicePairingOwner::authenticate_device_token` (611–683) constructs a candidate proof only after the existing bearer comparison and active-marker checks. It calls `validate_current_device_authorization_in` while retaining the same connection guard. The public retained validator (1559–1565) acquires its guard and delegates to that private function. There is no recursive lock acquisition, alternate connection, second validator policy, extra issuer, cache, persistent revision or write in this change.

The shared validator (1567–1601) first calls `current_pairing_record` (1605–1642). That preserves owner-local `Arc` identity, decisive stored-row decoding, strict approved-role membership, selected map-key/entry-role agreement, active marker and exact fingerprint currentness. It then strictly decodes the full live scope list and decisive approved baseline. Both the complete live grant and every originally admitted scope must satisfy the existing role policy. Only `GatewayCommandError::Unauthorized(_)` is translated to `Ok(None)` at fresh lookup. Other errors propagate; retained validation retains its error return. The ordering of owner/current-role/current-token checks before scope policy is unchanged.

The two local `BTreeSet<&str>` membership indexes preserve the canonical vectors and the existing `role_scope_policy_allows` implementation, including enumerated operator implications and nonoperator prefix checks. They do not reinterpret literal scope spelling, truncate a grant or impose a new count limit. `normalized_authority_strings` (74–85) now deduplicates with a local set while pushing values into the first-seen vector. Every array element still passes `as_str()?` and Rust `trim`; a selected role encountered early cannot hide a later non-string. The other parser consumers in target revocation, wire rotation and the legacy DPA request adapter retain their existing contracts (1342, 1453, 1782).

The owner tests provide concrete fresh and retained cases for role withdrawal, map-key/entry-role mismatch, entire-grant refusal, implication rules, empty grants, BOM/NEL spelling, strict decisive-storage errors, SQL-NULL fallback, repair and owner reopen. The large-authority case (1184–1229 in the revocation contract) includes 20,000 distinct scopes and 20,000 distinct role strings, trimmed/duplicate/empty roles, then a malformed final role and a single unapproved final scope. The observations preserve complete raw pairing/pending/machine state, keep the bystander credential usable and distinguish old-owner proofs from newly issued proofs. These are behavioral source controls; I make no measured timing or executed-test claim.

The App test uses the real file-backed pairing owner, actual TCP Gateway server, `dispatch_public_settings_command` and actual Settings service. Its denial oracle checks lookup/admission/dispatch/commit/report counters, exact stored pairing state, Settings bytes, directory entries and redacted responses. Its positive control changes and reopens the real Settings file. Native configured and native loopback-absence cases remain separate from explicit unknown-bearer refusal. The large HTTP case has the same malformed-tail and unapproved-tail controls, an unrelated valid writer, repair and reopen.

The HTTP fixture's lifetime boundary is explicit: server close and last API-owner release precede a bounded close acknowledgment; the API drops the pairing owner before acknowledging. Reopen requires that acknowledgment. Failed teardown retains the installation instead of removing files under an unacknowledged owner (`DfaHttpFixture::stop`, `reopen`, `Drop`, and `DfaGatewayApi::drop`, 549–623, 710–718). No fixture callback keeps the pairing lock across Settings.

I separately inspected shipping `UnifiedGatewayApi` authentication/currentness delegates, ordinary operator request hook, config routing, Settings dispatch, durable pairing initialization and shared-owner Gateway composition. The source-binding test's claims match those actual sources. The test remains a bounded HTTP/helper test with mutation-sensitive shipping source checks, not construction or startup of the complete App. The incoming main blob is `ab3cd46cfb869db19d12f2e1ee77a7c18e7b52df`.

The compatibility edits preserve the decisive earlier management oracles. In particular, the DPA empty-baseline case singles out exactly `(TARGET, node)` as the unselected now-invalid credential and asserts exactly one match; all other calls to the helper retain the all-current default. Genuine proofs are acquired before role/grant invalidation. Same-owner late-admission checks remain, and reopen checks directly test fresh refusal rather than using an old issuer proof as a substitute for grant evidence. Wire-rotation fault/revision/commit, role-materialization, exact state and response-redaction assertions remain present. The local-media prerequisite changes only the explicit JSON imports and the existing floating-point fixture timestamp.

CI retains the three required fresh owner names and both App names. The portable owner capture requires unique positives, zero command/capture exits and exact full target summaries with no failures, ignored or filtered tests. The workflow's rotation receipt preserves exact head/tree/run/attempt binding and required existing owner/HTTP/retained controls. App execution performs exact name listing and exact single-test runs, with non-passing outcome on absence or failed summary. Source/input hashing remains explicit. Ledger/RTM additions describe DFA behavior and evidence references while preserving the incomplete acceptance state; incoming shared metadata edits are not treated as authority for promoting this scope.

**Scope A findings:** none. No falsifiable security, correctness or lifetime defect requiring a fix was established in the changed scope. The source supports proceeding to the required exact-published-H Actions checks.

## Scope B: MSU oracle and fixture recovery assessment

The complete three-path diff from the original MSU base and the integration delta to publication parent were reviewed. The database changes are confined to the private test fixture and its expectations; the integration parent already contains those same fixture corrections. There is no production settlement algorithm change in this recovery scope.

`Fixture::expected_committed_entries` (agent_db.rs 9189–9198) specifies the ordered send plus complete receipt image independently using fixture identity, session, revision, work ID, media path and send ID. It does not call the production receipt factory or derive the expected receipt by reading the returned rows. `assert_committed_storage` retains exact full-row equality, released fence comparison and original-byte retention. Both the lost-ack two-reopen check and the exact-other-owner check now use that same complete expected image. Retirement count, closed owner, no publication/write-generation advance, retained bytes and unaffected other-owner checks remain.

This expectation matches the actual owner boundary: `get_transcript_entries` calls `read_transcript_entries_from_db`, which uses `parse_owner_transcript_entry`; the receipt is deliberately present in that owner view. `production.rs::hide_session_work_receipts` filters it only for the Session presentation view. The fixture change therefore repairs an incorrect projection expectation without weakening storage or lifetime assertions. I also read the actual media append transaction, receipt insertion/fence compare-and-set, commit/rollback classification and exact connection retirement to verify that the tested image and lifetime controls match the real owner.

The oracle's `media_commit_inherent_impl` now excludes trait implementations only from choosing the unique inherent owner (1693–1712). It still rejects missing, duplicate, foreign-path, generic, conditional or otherwise unsupported inherent owners. The matching source parser also skips trait items when locating the one inherent media method (1793); the complete original source continues into the global command scanner. This resolves coexistence with the actual `Drop for SandAgentDb` without granting traits the adapter exception.

The complete media COMMIT method, rollback method, public signature, entry prefix and returned handling tail remain pinned. The intervening synchronous write closure must retain its three admission checks. Only the one checked COMMIT method and the one proved handled call are removed/replaced in the oracle image. The original channel scanner still requires `Some(1)` for its sole argument-taking commit caller and checks its complete returned outcome tail. Trait bodies remain visible to its recursive token traversal and to the final direct raw-COMMIT prohibition.

`media_commit_source_image` normalizes only CRLF to LF in a source comparison image. It does not alter files, product code, runtime data or raw receipt hashes. The actual-owner positive explicitly includes the real Drop body and checks that body survives the exception processing. LF/CRLF actual-owner and harmless-trait coexistence positives compare the complete remaining source. Direct, alias and raw-identifier trait calls are negative cases for both owner types and both newline forms. Raw COMMIT injected before/after the owner, into helper siblings, the actual write closure or trait implementations remains exactly once in the unexempted source. Tail changes are rejected in both forms. Destructive substitutions now assert that their input changed, preventing a failed literal replacement from masquerading as a negative control.

The unchanged capture consumer `capture_media_cleanup.py` selects the exact **27** `TRANSACTION_COMMIT_NAMES`, including all eight media oracle cases and the existing channel/global/raw-COMMIT controls. It requires unique expected names, no unexpected positives, exact count and zero failed/ignored results, together with zero command/capture exits. Its raw-byte hashes, Git blob identity checks and before/after input equality remain separate from oracle normalization. The capture file has no diff from the original MSU base to the candidate. The existing 13 real DB fault cases, five custody cases, Tool consumers and separately required App/Session/Runner cases remain selected. Cargo still declares default-off `test-support` and enables it through the App dev-dependency; this review does not substitute for the required feature-off/feature-on compiler artifact evidence.

**Scope B findings:** none. No falsifiable weakening of the bounded adapter proof, global trait/raw-COMMIT scan, ordered receipt oracle or selected-check contract was established. The source supports proceeding to exact-published-H Actions recovery execution.

## Exact scope membership

All blob identities below were independently read from the candidate tree, rather than inferred from the mutable checkout.

| Scope | Path | Candidate blob |
| --- | --- | --- |
| A | `projects/openclaw-rust/docs/specs/device-bearer-approval-admission.md` | `f99fe2e81da2822ef7dd871bceba1215d3b0229e` |
| A | `source/host/src/device_pairing_owner.rs` | `9825003cf1f7047027aa1b8e496c40f0c1693a23` |
| A | `source/host/tests/device_pairing_rotation_authority_contract.rs` | `a06e0593cf3be3673e91706e1636399dab90d0a8` |
| A | `source/host/tests/device_pairing_revocation_authority_contract.rs` | `212b1067c4ffed3523aec5ebb2c3104eafe4c86f` |
| A | `source/host/tests/device_token_rotation_wire_authority_contract.rs` | `7c37e8093b8b7f2975eb031d76dae1b1419ed13c` |
| A | `source/host/tests/device_token_rotation_wire_gateway_contract.rs` | `e7c42f88067d8d4695ff7ee356987e493b1f4224` |
| A | `source/host/app/src/tests/config_secret_boundary_tests.rs` | `8f3405e6335dac35e6df87674241ad4d2df2dc07` |
| A | `source/host/app/src/tests/local_media_file_url_tests.rs` | `57a0a3393f1bb9560eb036687688656b1f3fe6c8` |
| A | `.github/workflows/rust-desktop-runtime.yml` | `ede3b6b9586383a2d0245de933d6ff4417e6ff10` |
| A | `projects/openclaw-rust/scripts/capture_portable_contracts.py` | `f5f3d5624568cb553441b8ca0c0f63aff76ebeae` |
| A | `projects/openclaw-rust/parity-ledger.json` | `285f9ca672947d9ad054832557c0b4abf38a0d68` |
| A | `projects/openclaw-rust/requirements-traceability-matrix.json` | `1bdbd1ca17b2ff334f365183c51bcd15c7ca0871` |
| B | `projects/openclaw-rust/docs/specs/media-settlement-unknown.md` | `8aee21c69bab72824025e4304eec38388ba1e9f8` |
| B | `source/host/src/extensions/session/agent_db.rs` | `87186bc4b246d88a5ce51ebda0623b76f4425d3d` |
| B | `source/host/tests/openclaw_sessions_and_config_contract.rs` | `88a80214aa08a1d61b3bd290063069d5aa4d417e` |

Independently hashed principal implementation/oracle inputs: device owner SHA-256 `719a84fe502691e1a14008167f3652174b9a7fe7bd3975ed34067b5351885073`; App config test SHA-256 `1ce6049924c9f1bcfbe305a1a94994232bffe6c29bc8848b688623c1c83f52e0`; media DB SHA-256 `1d1632094c80d6ba204f7c3a75db5eab0619628b25898449a1880e5ccca81900`; session contract SHA-256 `d93a5301f5cfac83d647a759785c259d9b78e87e7e2e4038e8aca7dbaac38121`. These agree with their manifest declarations.

## Reading limits and remaining gates

I read the frozen root AGENTS, the relevant current-authority sections of OCRM SOURCE_OF_TRUTH (1–85), main OCRM spec §17/OCRM-EXEC-02 (671–end), and both complete scoped specs. Governance blobs are respectively `e4682748087759b2d4bb2b3a47916b6148e8a6f2`, `3748d734b10c9ed5b2d948467fbaa9483f1dd306`, and `69c9667af2328e20a1fba09ebf80c5aab4d5c336`.

The complete declared diffs were covered; whole-file reading is not claimed for the large pre-existing owner, tests, workflow or canonical ledger. Deep adjacent reads focused on the symbols and ranges cited above: DFA parser/proof/currentness and management consumers; App fixture and shipping entry/composition/Settings mutation; media transaction/commands/retirement/getter/receipt projection; media and channel oracle scanners; changed DB tests; and required-name/input/exit capture consumers. Search located additional references but does not count as deep reading. Large historical SOURCE_OF_TRUTH chronology, prior review/evidence reports, unrelated product families and the complete upstream tree were not read. No historical source-reading status is promoted by this report.

The first broad text reads were too large for tool output limits; the decisive spec/code/diff areas were reread in bounded ranges. No conclusion depends on text hidden by truncation. Underlying Rust parsing, compilation, actual socket scheduling/teardown, database faults, LF/CRLF executable positives/negatives and all required named checks remain unexecuted by this reviewer.

Scope A and B remain separate approvals for the exact frozen tree. Publication with changed relevant inputs requires the applicable rereview. Required Actions checks, current-H artifact custody, scoped authority/coverage, real providers/devices/platforms, packaging/upgrade/rollback and independent final acceptance remain pending. Neither verdict changes any global completion flag.
