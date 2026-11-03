# MSU R6 shared-oracle integration proposal

## Status and scope

Task `647e0932-bea4-4113-9a61-65ae3db58955`, round 1; repository
`bhrumom/fabushi-desktop`; branch
`codex/openclaw-rust-fusion-chat-20261101-writer`; fixed OpenClaw U
`1440bb1771372334bea23d7642bb52deb7db1845`.

This is an implementation proposal for the root to adjudicate, not an independent
review or an executable-verification result. The sole proposed repository change
is a complete replacement for
`source/host/tests/openclaw_sessions_and_config_contract.rs`, supplied at
`/workspace/scratch/7d27ea239237/msu-r6-oracle-candidate.rs`.

The assignment was classified `xhigh` because preservation of two independently
changed assertion sets and the exact source-ownership boundary requires judgment.
The configured model and parent session settings were inherited; no model or
effort override, alternate worker process, or runtime-setting change was made.
This child has no separate introspection of the inherited runtime effort value.

Read the complete repository instruction at
`media-settlement-authority/AGENTS.md` and the applicable MSU-v1 specification at
`media-settlement-r5/projects/openclaw-rust/docs/specs/media-settlement-unknown.md`
before editing. No nested `AGENTS.md` was present in the supplied R5, incoming-905,
or incoming-fc snapshots. No review reports were read.

## Exact identities

The incoming publication binding supplied by the parent is commit
`9059d0c4493505851339ab228ec0d58a6023c904`, tree
`63385ed61d0ef7074dab86baea7fab548224416d`. This proposal does not claim a new API
verification of that commit/tree binding. Local file contents were independently
bound to the following Git blob identities with `git hash-object` without `-w`.

| Input or output | Exact Git blob | Local source |
| --- | --- | --- |
| Common Hfc oracle | `35719fc77edb1fea8f7acf0b22524c6eff7b29aa` | `media-settlement-incoming-fc/source/host/tests/openclaw_sessions_and_config_contract.rs` |
| Published incoming-905 oracle | `88a80214aa08a1d61b3bd290063069d5aa4d417e` | `media-settlement-incoming-905/source/host/tests/openclaw_sessions_and_config_contract.rs` |
| Unpushed R5 oracle | `c0162f99b896c00d0cd100ba59ad700eb01ad802` | `media-settlement-r5/source/host/tests/openclaw_sessions_and_config_contract.rs` |
| Production DB subject | `87186bc4b246d88a5ce51ebda0623b76f4425d3d` | `media-settlement-r5/source/host/src/extensions/session/agent_db.rs` |
| Applicable MSU-v1 specification | `4caeb8ca80252351acc8b59b2528d1c75b9e8d44` | `media-settlement-r5/projects/openclaw-rust/docs/specs/media-settlement-unknown.md` |
| Repository instruction | `e4682748087759b2d4bb2b3a47916b6148e8a6f2` | `media-settlement-authority/AGENTS.md` |
| Proposed complete oracle | `32a7218874c4b03e27a9872adf339186fab50f28` | `/workspace/scratch/7d27ea239237/msu-r6-oracle-candidate.rs` |

Candidate SHA-256:
`d88e69128092c78915fbc17463e3bf98f325cb2e10f408cce58558be619204df`.
Candidate size is 3,531 lines and 170,525 bytes.

Both complete deltas, Hfc to incoming-905 and Hfc to R5, were read before editing.
The complete candidate deltas against incoming-905 and against R5 were read after
editing. The relevant shared media and channel oracle implementations, their
constants and callers, and the production media method, command methods, and
actual `Drop` implementation were also read. This is not a claim of a complete
10,224-line production DB read or a broader migration review.

## Merge decisions

The proposal starts with the complete incoming-905 oracle, retaining its channel
section verbatim, then integrates R5's additional media controls. The two changes
have compatible behavior requirements.

1. Keep incoming-905's `media_commit_source_image` and its oracle-only CRLF-to-LF
   normalization. Use that helper in R5's `media_commit_oracle_fixture`, so every
   media counterfactual starts from the normalized actual source and first
   requires successful composition of the two bounded adapter proofs. Original
   production/source receipts remain untouched.
2. Keep R5's `replace_media_commit_site`. Every production-source replacement in
   the media counterfactuals requires exactly one intended source site and a
   differing replacement. Keep the incoming explicit before/after no-op guards
   as well, including separate guards for changed command methods, tail, prefix,
   closure fragment, full source, nested helper caller, and appended callers.
3. Route all media rejection controls through R5's
   `assert_media_commit_rejected_for_both_line_endings`. The original LF rejection
   assertions are preserved semantically, with their reasons, and paired with
   CRLF rejection. Existing syntax assertions remain present for the intended
   well-formed fixtures; malformed fixtures remain deliberate rejection inputs.
4. Retain the actual-source positives for LF and CRLF, including owner-local
   admission checks, exact media-only residual equality, composed residual
   equality, retention of the complete actual `Drop`, and all three ordinary
   zero-argument RAII positives under both line endings.
5. Keep incoming-905's exact harmless-trait residual equality against the
   unmodified baseline residual, as well as R5's explicit whole-trait retention
   assertion. Keep the incoming direct trait-caller case for each of the two
   owners, alongside ordinary and raw-identified aliases.
6. Keep all eight R5 raw-COMMIT placement fixtures, including both
   `HiddenRawCommit` trait implementations and both `RawMediaOwner` receiver
   trait implementations. Each LF and CRLF residual still must contain the raw
   statement, and incoming-905's stronger exact count of one is also required.

There are two spelling-only overlaps with more than one reasonable merge
interpretation: incoming-905 uses `MediaOracleTrait` when replacing the helper or
owner inherent impl, whereas R5 uses `MediaCommandsTrait` and `MediaOwnerTrait`.
They exercise the same kind of ownership failure. This proposal retains both
spellings as separate counterfactuals rather than silently dropping either
branch's fixture. That accounts for two of the additional candidate variants.
Likewise, the weaker residual `contains` assertions coexist with the stronger
incoming exact-equality/count assertions; neither assertion set is silently
discarded. No unresolved behavioral interpretation conflict was identified in
the supplied oracle deltas.

## Assertion preservation

| Contract | Candidate disposition |
| --- | --- |
| Exact inherent media helper and owner; no whole-impl exemption | Existing `media_commit_inherent_impl`, checked-method removal proof, signature/prefix/tail proof, three-guard closure requirement, and sole-call normalization are unchanged from the supplied inputs. |
| Complete command methods and settlement classification | Pinned media COMMIT, ROLLBACK, owner signature, prefix, and returned tail are unchanged; every pre-existing command, retirement, rollback, autocommit, publication, closure-shape, and classification mutation remains. |
| Actual production source baseline | Incoming normalized source image and R5's successful actual-source fixture precondition compose before all seven media counterfactual tests. The positive test continues to check the real `Drop` and admission guards directly. |
| Mutation causality | Incoming no-op assertions remain. R5's exact-one-site requirement applies to all media source replacements; owner method duplication additionally retains the unique signature, unique adjacent boundary, and complete-method suffix assertions. |
| LF/CRLF | Every media rejection variant and every raw residual placement is paired across both line endings; actual-source, harmless-trait, and RAII positive coverage remains. |
| Trait implementations | Harmless complete trait bodies remain exactly in the residual; direct and both aliased trait callers reject; both raw-trait fixture families remain visible to the original prohibition. |
| Channel command ownership | The complete channel method, unique caller count `Some(1)`, returned error tail, cleanup checks, hidden-owner controls, alias/macro negatives, and positive RAII controls are unchanged from incoming-905. |
| Incoming channel no-op safeguards | All ten added assertion sites remain verbatim: production cleanup mutation, missing helper, unsafe-owner fixture, changed error arm, discarded tail, conditional caller, wrong owner, malformed/ambiguous helper fixture, rewritten command, and hidden owner insertion. |
| Global raw-COMMIT prohibition | The final prohibition and individually proven method-sized exceptions remain; no count was raised and no additional source was exempted. Each media raw-owner counterfactual leaves exactly one forbidden direct statement after both known adapters are removed. |

The entire suffix beginning at `// CA-07:` through EOF is byte-identical between
incoming-905 and the candidate. Its independently computed SHA-256 in each file
is `0981a09d4344a8fda14d0f4a62df22327f5a2c53ae0b2fe7b1b1ae6396dc647d`.
This includes all incoming channel enhancements and every later unrelated test.
The full candidate-to-incoming diff is confined to the media fixture/positive/
counterfactual section.

## Source counts and diff size

The common Hfc file, incoming-905 file, R5 file, and candidate each contain 63
top-level `#[test]` declarations, including the same eight media oracle tests.
No test declaration was added, removed, renamed, or ignored. These are textual
source counts, not a harness listing or executed/passing-test count.

The following candidate fixture counts were enumerated by source reading of the
arrays and bounded loops. Counts exclude the actual-source and RAII positives.

| Media control family | Counterfactual variants | Intended line-ending evaluations |
| --- | ---: | ---: |
| Missing, duplicate, conditional, foreign, or trait-only helper: 12 well-formed plus 2 malformed | 14 | 28 rejection checks |
| Missing, duplicate, conditional, foreign, or trait-only owner | 11 | 22 rejection checks |
| Changed COMMIT: 4; changed ROLLBACK: 3 | 7 | 14 rejection checks |
| Extra/alias/macro callers: 15; nested helper sibling: 1; two trait owners times three calls: 6 | 22 | 44 rejection checks |
| Changed/discarded tail: 10; changed closure entry: 2 | 12 | 24 rejection checks |
| Changed unknown-entry classification or retirement | 6 | 12 rejection checks |
| Raw-COMMIT residual placements | 8 | 16 accepted-adapter/residual checks |
| **Total** | **80** | **144 rejection checks plus 16 residual checks** |

These are intended evaluations if execution reaches each loop. They do not show
that any fixture parsed, assertion passed, or test completed. The spelling-only
trait fixtures deliberately contribute two variants. The union also retains
incoming-905's two direct trait-caller combinations and R5's two additional
`HiddenRawCommit` combinations.

Read-only `git diff --no-index --numstat` reports the following full-file deltas:

| Comparison to candidate | Added lines | Removed lines |
| --- | ---: | ---: |
| Incoming-905 | 123 | 81 |
| R5 | 57 | 17 |
| Common Hfc | 171 | 92 |

## Verification limits and root handoff

Only source reads/searches, complete textual diffs, designated-file edits/copy,
textual counts, and content hashes were performed. No local or htch build,
`cargo`, `rustc`, test execution, test listing, Rust parser, generator, validator,
lint, or capture was run. The `syn` calls in the candidate are authored test code;
they were not executed here. No push, dispatch, branch mutation, repository
staging, workflow edit, ledger edit, specification edit, or snapshot edit was
performed. This worker wrote only the candidate and this integration note.

MSU-06's requested source design is represented in the proposal; its executable
acceptance remains unverified. MSU-01 through MSU-05 and MSU-07 through MSU-10 are
not reimplemented or independently accepted by this narrow test integration.
Production fault controls, real transaction outcomes, compiled feature isolation,
App behavior, and migration acceptance retain their existing evidence obligations.

The root must adjudicate this complete candidate against the current exact
integration authority before applying it. Subsequent permitted GitHub Actions
execution must bind the newly frozen source and retain the original selected
positive/negative controls and counts. None of the source-only observations in
this note promotes ledger state, current-H evidence, or product acceptance.
