# Complete read — .agents/skills/code-health/scripts/cpp_text.py

Status: independently understood and mapped; exact-head verification pending
Project: TDRP-001 Revision 9
Upstream: `telegramdesktop/tdesktop@351204f3695b4a39bc3f4d2f230c061b9a01fe2a`
Source: `.agents/skills/code-health/scripts/cpp_text.py@366f173248add43c34317c8fe17d9ca31d46b1d4`

## Inventory identity

The write lease for this closure is Fabushi `main@6550d08c6f73d6c75dc9651a3d06de84f956212f`, upstream `dev@351204f3695b4a39bc3f4d2f230c061b9a01fe2a`, tree `5151db79560ed40bf3802482fcfb74b950be6c40`. Fresh authority artifact `12235538548` from the immediately preceding source-authority campaign is the candidate recursive authority input. The recursive authority contains 16,202 non-directory entries and identifies actual 1-based recursive array entry 27 as this exact path/blob. Immediately before this write the merged parity ledger has zero rows for this exact path/blob and coverage is unknown=16,176, unread=16,149, omitted=0. This closure accounts only entry 27.

## Complete source responsibility

`cpp_text.py` is a small read-only lexical evidence primitive tuned to tdesktop source layout. It has no CLI, filesystem I/O, network I/O, durable state, finding store or task lifecycle.

- `EXTENSIONS_TU` recognizes `.cpp/.mm/.c/.cc/.cxx`; `EXTENSIONS_HEADER` recognizes `.h/.hpp`. `is_tu` and `is_header` are suffix classifiers only.
- `includes(text)` enumerates physical lines from one and returns `(line, path)` only for quoted `#include` or `#import` directives accepted by its regex. It does not resolve paths and does not claim angle-bracket or macro include coverage.
- `identifiers(text)` returns the lexical identifier regex matches; it provides occurrence evidence, not C++ name/type/overload resolution.
- `code_lines(text)` preserves one output line for every `text.split("\n")` input line while blanking/removing `/* ... */` block-comment segments. A cross-line block comment emits empty lines until the first closing marker and keeps the tail after that marker. It is intentionally lexical and does not constitute a full literal/raw-string-aware C++ lexer.
- `normalize(line)` strips leading/trailing whitespace, drops blank/comment/preprocessor/namespace/using/declared trivial lines, trims a trailing ` //` comment, drops very short non-word payloads and collapses remaining whitespace. It does not call `code_lines`; clone consumers intentionally apply the two stages in order.
- `definition_name(header)` returns the first callable-looking regex match not ending in a control-word name. Qualified names, destructors and operator spellings are lexical matches; successful recognition is not a compiler-semantic claim.
- `functions(text)` works on the line-preserving comment-blanked view, considers only column-zero candidate headers, bounds opening-brace search to 80 lines, rejects declaration-like/signature-interrupting forms, and searches for a column-zero closing brace. It returns `Function(name,start,end)` with inclusive physical line numbers and a derived `lines` count. Unsupported layout or an unmatched body is not evidence that the real C++ program lacks a function.
- `classes(text)` recognizes column-zero class/struct starts, excludes ordinary forward declarations, finds a column-zero `};`, and roughly counts one-tab direct member lines as methods or fields while skipping comments/Q_OBJECT/friend/using. It returns `Class(name,start,end,methods,fields)`; the counts are maintainability heuristics, not an AST.
- `Function` and `Class` dataclasses are transient outputs only.

The module catches no operational exception and emits no user-facing error. Any dependent command/result semantics belong to its callers. A malformed/unsupported source shape, stale/partial source authority or scanner failure therefore cannot be translated into an authoritative clean result.

## Real imports and callers

Exact upstream imports/callers at this snapshot were read rather than inferred:

- `scripts/metrics.py@ebab29607cb52782899d5424636f0bd5e0c126cd` uses extension classification, `functions`, `classes` and `identifiers` for file metrics, long-function/large-class observations and unused-function candidates. Those are candidates; deletion still requires the class/build/review proof.
- `scripts/report.py@a1177613670e933694f545f387ca7e7ed8cf0ee0` uses header classification while composing the canonical snapshot/findings alongside metrics, include graph, clones, history and build-model evidence. Report owns thresholds, aggregation and stable findings.
- `scripts/review.py@070b55553052267e8fbc12d70985b447b24bbb02` uses source-kind classification, quoted includes, functions/classes, the line-preserving comment view and definition-name recognition for diff ratchets, definition/use checks, stranded/orphaned warnings, class-growth checks and copy/include checks. `review.check` owns `(errors,warnings)`, strictness and message semantics.
- `scripts/clones.py@b74e6e939a3ccaae894d1afa5978f422aec8ee65` enumerates `code_lines` with original physical line numbers, applies `normalize`, then owns clone windows, maximal matching, overlap/common-bucket/declaration exclusions and stable clone identity. Review-time copied-hunk detection uses the same normalization boundary. Clone ownership was independently closed in entry 25.
- `scripts/includes.py@2c02b4e7eb821f123588e0a80056edef86fc084c` uses source-kind classification and `includes` to build its project include graph. It owns include resolution/unresolved accounting, closure, translation-unit reach, build-cost proxy and layering semantics.
- `scripts/atlas.py@6680fa747a1ddc520f4396949f3c84ab25cd8c43` uses the declared source extensions to scope candidate responsibility-atlas measurements. Atlas classification/approval remains a separate owner.
- `scripts/health.py@f08675ef7a03c016d85f91c60b8f29e99454abe9` imports the module at this snapshot but has no direct `cpp_text.*` dereference; it is the command owner that drives report/review/atlas consumers, exact-ref publication and failure routing. There is no `.agents/skills/code-health/scripts/scan.py` at this commit: "scan" is the `health.py scan` command, not a separate parser owner.
- `scripts/health_test.py@2176aea672fd9ed16c01342939fbffb41490ae73` directly locks the lexical contract: multiline function spans, constructor initializer lists, block-comment/license handling, aggregate/forward/object-construction negatives, class-member counts and line-preserving block-comment normalization. Its adjacent graph/clone/unused/review/move tests prove downstream use.
- `scripts/sources.py@688aa05c4223d0d45b8572cff89fbc5148254844` is the exact-ref/worktree source acquisition owner and does not import `cpp_text`; `history.py@7c12f5b091936723c6ad005dda4bb8e2b1e549df` likewise has no text-scanner ownership.

The resulting evidence chain is: exact source snapshot -> bounded line-preserving lexical/text facts -> metrics/include/clone/report/review/atlas consumers -> `health.py` scan/check/area/verify/atlas surfaces -> stable findings/proposals -> existing class/trust/routing lifecycle.

## References, perform-task and scheduled routing

`code-health/SKILL.md@6885938b3d186b55f5d69cb8b9b9a96bbca0e097` owns the daily/ad-hoc workflow. A durable scan runs `health.py scan --ref origin/dev --fetch --state ...`, must use the exact fetched revision, names skipped submodules and fails rather than presenting an older ref as fresh. It publishes the scan, audits/surveys, writes report/proposals, and routes only what its trust rules allow.

`references/atlas.md@7f43dd75fcfb670c327bc3e71c1de0589e7506f0`, `maps.md@89dd78f48283b15d6ad959b2d450895258a0980f`, `classes.md@27847cae96838be7ddf68d3759dba89279f7cf0e` and `survey.md@d900e7a42b768679c69cd4c584559c783b59d47a` retain responsibility/placement, task-class and source-review authority. Scanner observations select investigation; they do not approve ownership, a map or a change.

`perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c` sends Q0-Q2 through `mechanical-lane.md@e5e3cc88ba80ece6a20df7c4dd6002491d66a4ef` and Q3-Q5 through the full `pipeline.md@8a3dd7428be2f0976ef94863f801263ee330a9a7` / `phase-prompts.md@9e9eac194ffbcf9dd7125876e71dad651a7cabba`. The scanner can contribute candidate/ratchet evidence but cannot lower the task's proportional build/test/review proof.

`process-inbox/SKILL.md@0ce70794ac743d99e6d5c80894cde970b52842db` owns conversion of routed proposals into durable tasks; `continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5` owns selected-batch scheduling, one-task delegation and continuing independent work after task-local blocks. Text normalization creates, claims, starts, approves or completes none of those records.

## Existing-owner-first resolution

Current Fabushi already has the applicable source-neutral owners:

1. root `AGENTS.md` repository quality-health governance owns exact-source maintainability evidence and caller/task boundaries;
2. the existing repository duplicate/clone evidence governance already owns the normalization/provenance rules for clone use;
3. `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` owns positive/negative qualification;
4. `projects/fabushi-communication-platform/quality/evidence-contract.md` owns durable exact-source evidence/provenance;
5. the ordinary Fabushi task/review/evidence lifecycle owns findings, placement, routing, builds/tests and release acceptance.

The missing durable detail was the general source-text boundary shared by metrics/includes/review/clones/atlas: line-preserving lexical preparation, bounded recognition, explicit non-compiler limitations, and caller-owned result semantics. This closure extends only those existing owners.

Rejected alternatives: shipping Agent/Coordinator/Host/Runner and product-domain owners do not own repository lexical analysis; copying Telegram `cpp_text.py`/quality runtime would create a source-specific parallel parser; a new parser/findings/ownership/task store would duplicate canonical governance; treating lexical recognition as C++ semantic or build authority would weaken stronger existing evidence owners.

No novel shipping capability, new owner or ADR is required.

## Fabushi-equivalent contract

`fresh exact source authority -> declared source-kind scope -> deterministic line-provenance-preserving lexical/text normalization and bounded recognition -> caller-owned metrics/include/clone/report/review/atlas evidence -> health command exact-ref publication/check -> existing finding/class/placement/task lifecycle; unsupported/ambiguous lexical semantics -> explicit limitation/unknown, never proof of absence`.

## Failure and authority boundaries

- stale, partial or mutable source cannot back durable scanner evidence;
- block/inline comment handling is a documented lexical approximation and cannot be promoted to full C++ lexical correctness;
- physical line cardinality/provenance must not drift while comments are blanked/removed;
- unsupported macros, literals/raw strings, generated layout, nested/indented definitions or other unrecognized syntax cannot be treated as proof of absence;
- identifier/include/function/class/member observations are candidates/quality evidence, not compiler resolution;
- parser/scanner exception or malformed/incomplete input cannot become a zero finding or clean review;
- metrics/report/review/includes/clones/atlas/health keep their own thresholds, identities, warnings/errors, resolution, publication and routing semantics;
- no Telegram Python code-health runtime, second parser/findings ledger, ownership graph or task queue may be introduced.

## Traceability

- Requirement: `TDRP-R9-CODE-HEALTH-SOURCE-TEXT-NORMALIZATION-001`
- Oracles: `ORA-TDRP-CODE-HEALTH-TEXT-LINE-PRESERVATION-001`, `ORA-TDRP-CODE-HEALTH-TEXT-LEXICAL-NORMALIZATION-001`, `ORA-TDRP-CODE-HEALTH-TEXT-CONSUMERS-001`
- Invariants: `INV-TDRP-CODE-HEALTH-TEXT-NONCOMPILER-001`, `INV-TDRP-CODE-HEALTH-TEXT-PROVENANCE-001`, `INV-TDRP-CODE-HEALTH-TEXT-NO-SECOND-OWNER-001`
- Contract: `CONTRACT-TDRP-CODE-HEALTH-SOURCE-TEXT-001`
- Integration: `INT-TDRP-CODE-HEALTH-SOURCE-TEXT-CONSUMERS-001`
- Fault controls: `FAULT-TDRP-CODE-HEALTH-TEXT-MALFORMED-001`, `FAULT-TDRP-CODE-HEALTH-TEXT-UNSUPPORTED-001`, `FAULT-TDRP-CODE-HEALTH-STALE-SOURCE-001`
- Security control: `SEC-TDRP-CODE-HEALTH-TEXT-AUTHORITY-BYPASS-001`

## Verification disposition

This source entry is repository quality/evidence tooling and adds no shipping runtime. Exact-head verification remains the project-required GitHub Actions pair on one immutable PR HEAD.

TDRP Revision 9 source authority must remain fail-closed in Accepted mode while `upstream.accepted=false`, preserve the complete recursive authority proof (12 root shards, 24 child shards, 16 next shards, 202->203 fixed point with zero unresolved gitlinks, reachability/root-candidate accounting, root source archives 2/94,443/582 and Boost 582/582/open=0), and emit a fresh authority artifact bound to the final PR head.

Rust desktop runtime on that same head must retain renderer, platform-worker and rust-host terminal success and continue to execute the native messaging manifest at `native/mahayana-messaging/Cargo.toml`.

Only after both campaigns are terminal-success on the same exact head, PR/main/upstream/entry27 are reread unchanged and GitHub reports that exact head mergeable may this one source entry merge. Full TDRP source closure remains open after this entry at unknown=16,175, unread=16,148, omitted=0.
