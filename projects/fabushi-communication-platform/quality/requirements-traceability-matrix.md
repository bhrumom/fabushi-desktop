# Requirements Traceability Matrix

Status: active / normative schema  
Matrix ID: FQT-RTM-001  
Parent: FBCP-001 Revision 8 / TDRP-001 Revision 10 / FMEP-001 Revision 2  
Updated: 2026-11-02


## Round 18 — native CLI composition and installer members

Production changes are frozen at `1b0e674b3e39292ae715a3eb50435d83e2cb8577`; ledger binding
is `1a2b864aee290db21548262c983033d184cab356`. Fixed U and all concurrent numeric message
requirements are retained. These named IDs avoid redefining those requirements.

| Requirement / acceptance | Existing production chain | Observed execution | Remaining gate |
| --- | --- | --- | --- |
| NMR-CLI-HOST-01 / NMR-AC-CLI-HOST-01 | CLI/gateway RuntimeHandle -> MahayanaHost with desktop-full/local-only -> NativeEngine/NativeAgentBackend and real local MCP | Three source files and the workflow were API-read back at exact1b0; prior59 CLI3/gateway5/library11 are prerequisite only | Exact1b0 queued shipping must execute new MCP assertions, single-root production graph/build, both binary hash/symbol/library inventories and process4; complete external model/Bot and external FFI remain open |
| NMR-PACKAGE-DELIVERY-01 / NMR-AC-PACKAGE-DELIVERY-01 | Existing release artifact uploader -> unchanged installer promotion member allowlist | Exact1b0 Electron job125524808067 passed promotion6 (5 behavioral +1 source-shape), metadata18 and immutable historical ZIP/DMG control; three current contract artifacts were digest/target verified | Current source signed archive membership, complete installed runtime and immutable promotion acceptance remain unqualified; historical package bytes are not a new-H installer |

Original exact99 run41456371872 completed with composition12/Native6,125 auth
parameter requests (not125 Rust test cases),17 typed Settings calls/15 variants
plus real restart, proxy30, actual AppHost copy4 and2 required rejecting controls.
The complete logs and artifact byte identities are retained in the round18
receipt. Later exact59 proxy48/48/all6 permutations are separate evidence.
At exact1b0, UI18/settlement5/Chromium16, Electron10/8/3/16/28/18/6 and Windows11
passed; the two macOS Rust jobs had no runner/steps at the recorded observation.
Source-contract41518863601 at exact ledger1a2 passed Node157/Python37 and
candidate consistency; two actual ZIPs matched their digest and target SHA.
Accepted-mode rejected the candidate, with no source/product/release promotion. Required full service,
supported-platform, Bot, visual/a11y, security, performance, soak, upgrade/rollback
and independent ACCEPT gates remain open.

Evidence: `projects/telegram-desktop-rust/evidence/round18-current-host-features-and-delivery.json`;
`projects/telegram-desktop-rust/evidence/round18-ffi-caller-reference-index.json`.

## NMR-47g pending forward ownership

Exact source/oracle/old-failure/execution boundary: `projects/telegram-desktop-rust/evidence/round27-forward-pending-ownership.json`.

| Requirement | Existing owner and behavior | Case / qualification |
| --- | --- | --- |
| NMR-47g / NMR-AC-38 | Session strict source journal plus existing destination Transcript; persistent marker before upload/POST; actual request/target nonce proves ownership even after context loss | One actual process/SQLite/HTTP case with revoked source, six damaged-marker/context variants, malformed/unavailable store and readable prefixed legacy positive; containing-H execution pending |
| Existing reply/resource behavior | NMR-49 current-row reply recovery and NMR-48 pinned-byte preflight retained; arbitrary ordinary reply metadata is not a forwarding identity | Retain all existing Human/Agent/credential/resource/journal gates; add exactly one selected Human case |
| Evidence/acceptance | Original2a failed;59 message jobs passed;936 UI18/5/16 and source157/schema37 passed while two Native jobs queued | Source counters/reviews/ACCEPT unchanged; temporary ledger transport is not product validation |

## Round31 — canonical reply remote identity and exact execution

NMR-49c / NMR-AC-40: existing Session reply admission now uses the strict
canonical positive safe-integer message-ID parser for both direct and historical
parents. Valid local/peer and local-only pending semantics remain. Two existing
tests cover28 historical/numeric scenarios and17 direct-target scenarios; readable
notice fixtures exercise the kind guard. Physical rows, independent chronology,
HTTP refusal/idempotency, sync and reopen are checked without a new owner.

Exact H `4cbd7ddbe58ae789db9d2c674e8c9fa73146af40`, run41524748572,
reconciliation125549482711 passed Human62/resource3/sharebox1/journal12/
Agent4/credential3/forwardHTTP1. The named reply, mixed-stale, dual-SQLite CAS
and corrupt-journal recovery cases actually passed. UI passed Transcript29,
cursor17, forward5 and Chromium16; Electron, Windows11 and retained C wrappers
15-pass/1-live-DC-ignore also passed at this H. Shipping125549483258 remains
queued at the recorded observation; old ff6 shipping is not a replacement for
this affected-H result. Complete log hashes and artifact identities are in
`projects/telegram-desktop-rust/evidence/round31-reply-identity-recovery.json`.

Original06 reconciliation was cancelled without a runner. Originalff6
reconciliation remains unexecuted; containing106bb and this4cbd run qualify the
requested later recovery assertions separately. The afb E0382 fixture failure
was TEST_FLAW, corrected in concurrent106bb and preserved. Backend PR137 stays
read-only; full source/legal, real services, signed supported-platform packages,
full Bot/quality/upgrade/rollback and independent ACCEPT remain open.

## NMR-49 / NMR-AC-40 — pending-parent reply recovery

Fixed U unchanged; integration base `b25c2003a8e76779f2535fc3dfa90e4fe5bde149`.
Evidence: `projects/telegram-desktop-rust/evidence/round12-pending-parent-reply-recovery.json`.
Prerequisite runtime H59f5fa/run41508537631 passed Human38/Agent4/credential3,
forward sharebox1/journal3/HTTP1, UI16/forward5/Chromium12 and selected shipping,
Electron and Windows jobs. New tests below are authored; execution at preparation
is not-run and must bind to the containing code H, not that earlier run.

| Requirement / invariant | Existing production owner / real consumer | Authored behavior / required gate |
| --- | --- | --- |
| NMR-49a: no reply action before canonical settlement | production/model.ts → existing TranscriptMessage/actionable guard | Actual projectTranscriptEntry across message/attachment shapes and normal Human/Agent controls; Transcript18 and renderer build |
| NMR-49b: parent and unsent local reply edges commit together | SandAgentDb::settle_transcript_entry_identity → existing deletion/upsert consumers | Real SQLite parent/child failure injection, unchanged immutable fields and atomic event stream, reopen and actual HTTP recovery; Human47 |
| NMR-49c: current durable reference and scoped historical alias | Session pending replay and append_human_message_with_context → native direct-message POST | A→B→C restart ordering, unique local canonical parent; peer same nonce, foreign conversation, conflicting/ambiguous identity fail closed; Human47 |
| Retained NMR-47e and NMR-48 attachment snapshots | Exact94 journal/native service/forwarding owners | Preserve twelve journal cases, strict metadata/source recovery and five attachment byte cases; this batch adds no journal implementation |
| NMR-AC-40: preserve existing owners and regressions | Same Session, DB, HTTP client and projection; no public/schema owner added | Retain Agent4/credential3/sharebox1/journal12/forward-HTTP1, forward projection5/Chromium16 and selected shipping/Electron/Windows gates |

Source reviews, acceptance counters and all full-migration/service/package/Bot
and independent ACCEPT gates remain unpromoted. Backend PR137 is Draft and
production readiness has no authorized live execution.

## Retained concurrent batch — task20bf round1 resumed, immutable forward intent

Current input `94dcf7a2213808a8ded6baddaf069562d365c6d7` supersedes the interrupted
`bb0013e1302c69cbd125bf0c4e64c070688de542` local snapshot. Existing Host journal,
account fences, source acquisition and schema improvements are retained. The
earlier production input `59f5fa24827812f78336c464995d161f403dedfb` passed
run41508537631 attempt1; that is predecessor evidence, not execution of the new
picker/source-review extension. New Host snapshot and journal guards are
retained with Human44 and journal12 in the concurrent base; the combined reply
batch requires Human47 and retains journal12.

| Requirement / oracle | Existing production owner | Authored check / acceptance boundary |
| --- | --- | --- |
| FWD-UI-01 / ORA-TDRP-FORWARD-INTENT-UI-001 | ForwardMessageDialog → Session HumanForwardBatch | Same-ID refresh, partial subset retry, stable nonce/options, duplicate-submit and keyboard target-change fences |
| FWD-UI-02 | Existing picker and settleForwardProjection | Missing/duplicate/conflicting/unexpected destinations cannot confirm success; search cannot erase send error |
| FWD-UI-03 | Existing recipient selection; Host bound remains authoritative | 32 unique targets across searches; 33rd refused while selection remains editable |
| FWD-UI-04 | Existing privacy controls plus shared ForwardPrivacy | Hiding captions hides names; showing names restores captions; first dispatch locks snapshot |
| FWD-UI-05 | Existing account/dialog lifetime and Session durable recovery | Close while pending retires callback only; late result cannot settle/close reopened presentation |
| FMEP-SOURCE-01 | Unique source_reviews and existing source candidate checker | Two exact option-source records complete, four fully read panel/phrase records pending decomposition; pending records never counted complete and full acceptance still requires the entire source set |
| NMR-48d / NMR-AC-39b | HumanForwardBatch::validate on existing Session journal load/store | `forward_journal_privacy_rejects_contradictions_without_replacing_valid_intent` and `shipping_human_forward_privacy_corruption_blocks_restart_until_exact_intent_repair`; three semantic contradictions, cold/cached owners, exact damaged-byte retention, no target I/O, repair/same-nonce continuation and text-only positive control |

Existing Chromium12 cases are retained and extended to16, alongside settlement5,
the original Transcript16 plus two concurrent reply-projection cases (Transcript18),
and current Host/Agent/shipping/Electron/Windows gates. At H
`73cc6e1cc9fa871e5f80931979606ac9ee810551`, source157/schema37/candidate,
Chromium16/settlement5/Transcript16, Electron83, Windows11 and reconciliation
Human44/sharebox1/journal12/Agent4/credential3/Gateway1 passed. Six actual UI
screenshots were viewed. Shipping was still queued at this observation.
NMR-48d's containing H preserves the concurrent reply cases and requires
Human49/journal13; new execution is not-run at authoring. No local executable
verification or independent ACCEPT.
Evidence: `projects/telegram-desktop-rust/evidence/task20bf-round1-forward-intent-ui.json`.

## Historical round 28 — actual forwarding crash tests and persistence repair

Evidence: `projects/telegram-desktop-rust/evidence/round28-forward-crash-qualification.json`.
Inspected base H=`65253740c033508ab08a4b500449a5b56727e940`,
run41504939510 attempt1. The prior Round27 description of five authored
integration cases/owner5 and its absent evidence file are withdrawn; the base
actually has Human33, sharebox1 and journal2. This round authors the five cases.
Exact H59f5fa24827812f78336c464995d161f403dedfb/run41508537631 is now
completed/success: Human38/sharebox1/journal3/Agent4/private3/Gateway1,
renderer/Transcript16/settlement5/Chromium12, actual AppHost/gateway5/library11/
protocol122/TCP9/copy4 with three refusal controls, Electron and Windows11.
All five complete logs and four CI theme/viewport images were inspected.
The same journal test is strengthened on the later e8b48cb owner through
the actual valid-batch merge/write path, preserving12 cases and all concurrent
attachment/completion/concurrency changes. That oracle requires containing-H
Actions; old-H execution is not promoted to later code.

| Requirement / oracle | Existing production owner | Authored check / actual base evidence |
| --- | --- | --- |
| NMR-46 / NMR-AC-37 recovery | ForwardMessageDialog fixture using existing runtime theme installer | Base renderer/Transcript16/settlement5 pass; Chromium11/12, missing setTheme. Retain Chromium12, keyboard control and four actual light/dark viewport screenshots. |
| NMR-47a/b / NMR-AC-38 | Source SandAgentDb journal plus destination Session/Transcript | Five actual HTTP/SQLite/process cases passed in H59 Human38; parent kills and reaps child, reads committed intent independently, resumes untouched target, injects staging/progress SQL errors. Require Human38. |
| NMR-47c/d / NMR-AC-38 | Existing credential/source authorization and ordinary sync | Ambiguous first POST reuses nonce; immutable source snapshot; same/subset/concurrent completed replay; changed source/privacy/expanded-set negatives; source/account/issuer denial; no generic-outbox bypass. |
| NMR-47e/f / NMR-AC-38 | Existing SandAgentDb expected-value CAS and Session eligibility | Two-handle stale read regression plus typed source denial and unrelated actual history GET; I/O failures still stop. Select sharebox1 and journal3 separately. |
| Gateway / Human / Agent preservation | Existing authenticated Gateway and Session writers | Base Human33/sharebox1 pass, count assertion fails; Agent4/private3/Gateway1 are skipped after it. All remain required on new H. |
| Native package / Social / transport | Actual AppHost shipping root and resource-copy owner | Base shipping125481711613 compiled AppHost and passed composition12/Native7/FeatureHost7/Social9/HTTP-owner6/Host6/ABI7/CLI3/gateway5, then stale expected3 count failed. Later library/protocol/TCP/copy checks not run; new H must execute them. |
| Electron / Windows | Existing carrier/auth/update/publisher owners | Base Electron125481711452 and Windows125481711041 succeed; no formal publisher or signed-package acceptance. |

Source_reviews was absent at65253740; later concurrent records are retained.
Source/product/independent acceptance remain ungranted. Backend #137 remains open/draft/unmerged and read-only. Remote
provenance/policy, resource retention, multi-AppHost uniqueness, real service,
packaged visual/a11y, signed release, Bot and performance/soak remain open.

## Round 21 explicit update source and exact-H evidence

Authority: `docs/specs/electron-update-source-authority.md` EUS-01–06 and
CSI-R1/3/5/6. Evidence: `projects/telegram-desktop-rust/evidence/round21-update-source-authority.json`.
The reviewed base is `8ce0a9a43b7fe94df2ec2085399fe65a57343577`; current
implementation execution is pending at preparation. Independent frozen-source
review is separate from execution and whole-product ACCEPT. The source audit is
partial; the canonical source inventory, source_reviews and acceptance counters
are not promoted by this existing-owner safety correction. The same canonical
ledger now maps the narrow product responsibility as
`TDRP-UPDATER-DISCOVERY-AUTHORITY-001` under existing `UPDATE-INSTALL-ROLLBACK`;
the Windows workflow row retains its tooling scope and pending release state.

| Requirement | Actual production owner / behavior oracle | Authored Actions gate | Acceptance boundary |
| --- | --- | --- | --- |
| EUS-01 / CSI-R1/3 | Main-process config → production binding → wiring → SandUpdateService; no default; empty/invalid primary never falls back | Real wiring plus direct service source-precedence cases | Approved origin configuration is not proof of an operational feed |
| EUS-02 / CSI-R5/6 | Existing shared Fabushi origin policy; reject before I/O; no rejected raw configuration in status; manifest redirect refused | Invalid/foreign matrix and real loopback HTTP302 target-hit counter | Asset download provenance, signer policy and real package authenticity remain open |
| EUS-03 | Same updater state/timer/AbortController owner; runtime gates cannot bypass source; disposal stops new checks | Startup/manual/track/backend/minimum-version/live-gate/abort/dispose cases | Controlled ports; no OS installation claim |
| EUS-04 | Existing settings and apply-marker persistence; settle prior apply outcome while source-disabled; retain user-data sentinel | Real marker files, controlled download/hash/signature/staging/apply chain, reconstructed service | Settings reconstruction is a controlled store; real signed install, disk settings recovery and rollback remain open |
| EUS-05 | Existing typed update status IPC → Settings update message; non-retryable telemetry | Actual Settings helper and telemetry functions | Full visual/accessibility and packaged UI acceptance remain open |
| EUS-06 | Existing native-service-contracts Electron job plus actual Electron and renderer builds | Focused updater suite16; carrier10/auth8/copy3/metadata18 and UI16 retained | Execute only GitHub Actions at the containing exact H; no local validation |

Base run41471743677 has actual reconciliation33+4+3, UI16 and Electron
carrier10/auth8/copy3/metadata18 passes. The immutable historical ZIP/DMG
experiment passed without changing package bytes. At this preparation observation,
shipping125366805190 was cancelled before runner assignment (steps empty): gateway library11, protocol122, TCP9,
actual package-copy suite4 and its two negatives are not recorded as executed.
The suite4 comprises three fixture cases and one actual compiled AppHost case.
The run is completed/failure. Its missing shipping log (404 BlobNotFound) and
unreadable check-run annotations leave the cancellation cause unverified; no
implementation or assertion failure is inferred. The previous lib9 expectation
was an oracle defect; all future gates require11.
Historical signed package acceptance failed and is not repaired by metadata success.

## Round 20 bounded oracle/execution refresh

Evidence: projects/telegram-desktop-rust/evidence/round20-blockmap-gateway-recovery.json.
Base151d3ffe6382219dab98fb317e59fe5faa8cea01/run41465224038 completed failure.
The four complete job logs separate passed, failed and unexecuted gates below.
Containing-H execution of the two repairs and independent ACCEPT remain pending
at preparation; historical execution does not qualify later changed code.

| Requirement | Existing owner and oracle | Authored execution gate | Verdict |
| --- | --- | --- | --- |
| NMR-41 / AC-32 | ProductSettingsOwner endpoint identity/order, selected/mode, repeat stability |16 ordered subsets/48 deletions/48 repeats; all6 permutations within AppHost12 | Passed exact151/job125344195252; not full Settings acceptance |
| NMR-42 / AC-33 | Direct MahayanaHost and actual approval calls without ordinary dispatch | CLI3, executable5, full lib9;8 positive/9 negative approval inputs | CLI3 passed; executable4/5 TEST_FLAW; lib9 not run |
| NMR-43 / AC-34 | Final package hashes/sizes plus complete decoded blockmap equality; receipt binds actual sidecar | Metadata18 and unchanged digest-pinned artifact13093546572 repair/immutable-byte experiment | Old15 passed, historical experiment failed compressed-byte comparison; fix authored; signed candidate acceptance open |
| NMR-44 / AC-35 | Runtime lifecycle owner; v1 cannot revive interrupted runs/duplicate terminals |3 lifecycle cases in full lib9 | Compiled at base; lib9 execution blocked by executable fixture failure |
| NMR-45 / AC-36 | Real persistent request registry with UUID-owned fixture, lastError/restoredRequests and stable srq-ID | Keep executable5; first1/duplicate0, disk and memory unchanged, reopen retains request, independent store issues1 | TEST_FLAW repaired in tests; containing-H execution required |

## Round 30 — exact recovery gate classification

Original06fb/run41512635210 has four passing execution jobs; reconciliation
was cancelled before runner assignment, so Human39/journal5 were not executed.
Later d261afe/run41521138485 passed Human54/journal12 with mixed stale snapshots
and26 cold/cached corruption controls. Preserve the newer beafc6fc concurrent
CAS valid-intent, zero-busy-callback, raw-byte and fresh-retry/reopen oracle.
Exact afb34b49/run41522324008 failed before Human62 execution with Rust E0382 in
the reaction-restart fixture: it reused `runtime` after drop. Concurrent
`106bb95ba97ef7722e9523285783824393039bc0` closes the live `restarted` owner;
the exact remote test blob matches the independently reviewed one-line fix and
retains all lifecycle, durability, event and HTTP assertions. This is TEST_FLAW,
not evidence of a product failure. Exact106bb95/run41523018920 subsequently
passed Human62/resource3/sharebox1/journal12/Agent4/credential3/HTTP1, including
the newer CAS and corruption controls; UI/Electron/Windows also pass. Shipping
125543545357 is queued at2026-11-02T20:54Z; overall pass, independent ACCEPT and
release remain blocked. Evidence:
`projects/telegram-desktop-rust/evidence/round30-forward-recovery-qualification.json`.

## Rule

每个 applicable requirement/AC 必须追到 oracle/invariant、test case IDs、execution evidence 与 verdict；test 也必须反查 requirement。`TBD` 仅可 planning，不能 verified/release。

## Required columns

`requirement_id | risk | oracle_ids | invariant_ids | unit/property | contract/integration | e2e/temporal | ui/visual/a11y | perf/security | regression_ids | evidence_ids | reviewer | verdict`

## AC coverage plan

| AC | Primary suites/gates |
| --- | --- |
| AC-01 | ARCH-CANON, UI-ROOT |
| AC-02 | FUNC-NATIVE-SVC, SEC-NET |
| AC-03 | MAP-OWNER, ARCH-WIRING |
| AC-04 | ARCH-NO-PARALLEL |
| AC-05 | FUNC-CAPABILITY, RTM-COVERAGE |
| AC-06 | UI-CONVERSATION, FUNC-MIXED |
| AC-07 | BOT-REGRESSION |
| AC-08 | ARCH-ADR |
| AC-09 | PROP-PROTOCOL, TEMP-RECOVERY |
| AC-10 | ARCH-LANGUAGE |
| AC-11 | PROP-CANONICAL-TRUTH, INT-PROJECTION |
| AC-12 | BOT-REGRESSION |
| AC-13 | EVIDENCE-IDENTITY |
| AC-14 | PKG-ACCEPTANCE |
| AC-15 | INVENTORY-TRACE |
| AC-16 | UI-IA, UI-A11Y |
| AC-17 | BRAND-SCAN |
| AC-18 | FUNC-SERVICE-E2E |
| AC-19 | SEC-PRIVACY |
| AC-20 | PERF-SOAK |
| AC-21 | FUNC-MIGRATION-ROLLBACK |
| AC-22 | STUB-BLOCKER-SCAN |
| AC-23 | BASELINE-DIFF |
| AC-24 | RELEASE-INDEPENDENT |
| AC-25 | ARCH-COMPOSITION |
| AC-26 | ARCH-NOVEL-CAPABILITY |
| AC-27 | UI-ENTRY-ROUTE |
| AC-28 | UI-CREATION-CONVERSATION |
| AC-29 | FUNC-SEARCH, ARCH-SEARCH |
| AC-30 | FUNC-SEARCH-NEG, PROP-SEARCH |
| AC-31 | UI-INTERACTION, UI-STATE-CONTINUITY |
| AC-32 | UI-TOKEN-LINT |
| AC-33 | UI-COMPONENT-OWNER |
| AC-34 | UI-SCREEN-STATE |
| AC-35 | VISUAL-SYSTEM |
| AC-36 | VISUAL-MATRIX, UI-A11Y |
| AC-37 | VISUAL-REGRESSION |
| AC-38 | UI-LARGE-DATA, PERF-UI |
| AC-39 | BRAND-ASSET-COPY |
| AC-40 | UI-DESIGN-EXCEPTION |
| AC-41 | RTM-COVERAGE |
| AC-42 | UNIT-PROP-CONTRACT-INT-PKG |
| AC-43 | ORA-TURN-001, TEMP-TURN |
| AC-44 | ORA-ORDER-001, ORA-RECON-001, PROP-RECON |
| AC-45 | UI-FUNCTIONAL, VISUAL-TIMELINE, EXP-REVIEW |
| AC-46 | DEFECT-REGRESSION |
| AC-47 | FAULT-RECOVERY, FLAKE-GATE |
| AC-48 | PERF-SOAK |
| AC-49 | SEC-PRIVACY, UI-A11Y |
| AC-50 | RELEASE-ENTRY-EXIT, RELEASE-REPORT |

实际执行时每行展开 concrete case IDs + run/artifact；本表目前定义 coverage plan，不声称任何 AC 已通过。


## Revision 9 concrete responsibility rows

These rows are additive to the AC coverage plan. A row at `IMPLEMENTED` is not a release verdict.

| requirement_id | risk | oracle_ids | invariant_ids | unit/property | contract/integration | e2e/temporal | ui/visual/a11y | perf/security | regression_ids | evidence_ids | reviewer | verdict |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| TDRP-R9-SEARCH-ROW-REPLACEMENT-001 | high | ORA-TDRP-SEARCH-ROW-REPLACEMENT-001 | INV-TDRP-SEARCH-CANONICAL-ID-001; INV-TDRP-SEARCH-LATEST-ROW-001 | PROP-TDRP-SEARCH-ROW-REPLACEMENT-001 | CONTRACT-TDRP-CANONICAL-SEARCH-001 | E2E-TDRP-CANONICAL-SEARCH-001 | UI-TDRP-CANONICAL-SEARCH-RESULT-001 | SEC-TDRP-SEARCH-AUTHORIZED-INPUT-001 | REG-TDRP-SEARCH-DUPLICATE-PARTICIPANT-001 | commit:674d60bbc1e9059e6160cbfd321fbfff47612fff; workflow:37597016700@d99de586173377e4618fc1956bb2f30be8f1db11 | pending-independent-review | IMPLEMENTED |
| TDRP-R9-EXTERNAL-URL-AUTH-CONTEXT-001 | critical | ORA-TDRP-URL-AUTH-ACCEPTED-CONTEXT-001 | INV-TDRP-URL-AUTH-STRIP-FOREIGN-001; INV-TDRP-URL-AUTH-EXACT-ORIGIN-001; INV-TDRP-URL-AUTH-NO-RENDERER-UPGRADE-001 | PROP-TDRP-URL-AUTH-ENCODING-001 | CONTRACT-TDRP-URL-AUTH-BOUNDARY-001; INT-TDRP-FABUSHI-ACCOUNT-EXTERNAL-OPEN-001 | E2E-TDRP-EXTERNAL-URL-AUTH-001; TEMP-TDRP-URL-AUTH-EPHEMERAL-CONTEXT-001 | — | SEC-TDRP-URL-AUTH-STRIP-001; SEC-TDRP-URL-AUTH-EXACT-ORIGIN-001; SEC-TDRP-URL-AUTH-PRIVILEGE-001 | REG-TDRP-URL-AUTH-TOKEN-SMUGGLING-001 | commit:aa9df7fef3454a81aacf489fecae984099cdeab4; workflow:pending-current-head | pending-independent-review | IMPLEMENTED |
| TDRP-R9-SHARE-FORWARD-PRIVACY-001 | critical | ORA-TDRP-SHARE-FORWARD-PRIVACY-001 | INV-TDRP-FORWARD-CAPTION-IMPLIES-NO-SENDER-001; INV-TDRP-FORWARD-PRIVACY-NATIVE-001; INV-TDRP-FORWARD-PRIVACY-IDEMPOTENT-001 | PROP-TDRP-SHARE-FORWARD-PRIVACY-001 | CONTRACT-TDRP-SHARE-FORWARD-PRIVACY-001; INT-TDRP-SHARE-FORWARD-PRIVACY-001 | TEMP-TDRP-SHARE-FORWARD-PRIVACY-001; FAULT-TDRP-SHARE-FORWARD-PRIVACY-001 | pending shipping Forward E2E | — | REG-TDRP-FORWARD-PRIVACY-001 | workflow:pending-current-head | pending-independent-review | IMPLEMENTED |
| TDRP-R9-SHARE-RECIPIENT-ELIGIBILITY-001 | critical | ORA-TDRP-SHARE-RECIPIENT-ELIGIBILITY-001 | INV-TDRP-RECIPIENT-AUTH-BEFORE-EXPOSURE-001; INV-TDRP-RECIPIENT-MEDIA-POLL-ELIGIBILITY-001; INV-TDRP-RECIPIENT-CHANNEL-POSTING-001 | PROP-TDRP-SHARE-RECIPIENT-ELIGIBILITY-001 | CONTRACT-TDRP-SHARE-RECIPIENT-ELIGIBILITY-001; INT-TDRP-SHARE-RECIPIENT-ELIGIBILITY-001 | TEMP-TDRP-SHARE-RECIPIENT-POLICY-REFRESH-001; FAULT-TDRP-SHARE-RECIPIENT-ELIGIBILITY-001 | pending shipping recipient-picker E2E | — | REG-TDRP-RECIPIENT-PREEXPOSURE-001 | workflow:pending-current-head | pending-independent-review | IMPLEMENTED |
| FBCP-NMR-001 | critical | NMR-AC-01–05,12,22–40 | NMR-01–06,09,18–26,31–49c | Existing Session/Transcript and SandAgentDb; whole-batch CAS, source denial, private attachment snapshots and atomic pending-reply identity repair | Exact59f5fa/run41508537631 Human38/sharebox1/journal3/Agent4/private3/Gateway1 all pass | Retain NMR-48 five byte cases, source-corruption recovery and journal12; NMR-49 adds three Host/SQLite/HTTP and two projection cases. Containing-H Human47/journal12/Transcript18 authored, not-run at preparation | Exact59 renderer16/settlement5/Chromium12, Electron and Windows gates pass; require combined Transcript18/settlement5/Chromium16 and retained shipping jobs at new H | Preserve source/credential and complete-buffer fences; pending-parent aliases require same conversation, local Human author and unique canonical nonce; failed identity/child-edge persistence rolls back without identity events | REG-NMR-FORWARD-ACCOUNT-001; NMR-47 CAS/process; NMR-48 byte retention/repair; NMR-49 A→B→C, scoped aliases and parent/child ABORT/IGNORE recovery | fbcp-native-message-round4-20261102.json binds exact59 evidence; task20bf-round1-forward-intent-ui.json and round12-pending-parent-reply-recovery.json record authored extensions pending containing-H execution; historical round27 claims remain withdrawn | Focused source and independent static implementation review only; independent ACCEPT pending | IMPLEMENTED; NMR-48/NMR-49 and retained forward UI require containing-H execution; full acceptance BLOCKED |
| TDRP-UPDATER-DISCOVERY-AUTHORITY-001 | critical | EUS-01–06 / EUS-WIN-01–06 | Explicit source and exact publisher/bytes/one-use invariants | Existing updater/service/Windows installer and Settings | At a119: source16, controlled publisher28, native Windows11; zero failures/skips | Controlled ports and native CI signer; authorized real signed install/upgrade/rollback open | Existing Settings helper; full visual/a11y pending | Source/redirect/abort/marker and signature/publisher/tamper/one-use negatives; performance/soak pending | Existing valid source scheduling and marker recovery | Round26 receipt: run41496846317 Electron125453934276/Windows125453934371 success | Bounded frozen review; release ACCEPT pending | IMPLEMENTED / BOUNDED TESTS EXECUTED; source/product/release BLOCKED |

### TDRP-R9-SEARCH-ROW-REPLACEMENT-001 oracle

- `ORA-TDRP-SEARCH-ROW-REPLACEMENT-001`: when an authoritative participant/dialog row is replaced or a collection scope is rebuilt, the current Search projection contains at most one result for the stable participant identity and projects the replacement row.
- `INV-TDRP-SEARCH-CANONICAL-ID-001`: one stable participant id may produce at most one canonical Command Palette participant result.
- `INV-TDRP-SEARCH-LATEST-ROW-001`: if an old and replacement row coexist at projection input, replacement metadata/visibility wins.
- `PROP-TDRP-SEARCH-ROW-REPLACEMENT-001`: replacement is idempotent under repeated duplicate rows and preserves the first canonical list position while updating the row value.
- `CONTRACT-TDRP-CANONICAL-SEARCH-001`: the behavior is implemented in the existing Command Palette Search owner; no Telegram-specific or second Search root is permitted.
- `E2E-TDRP-CANONICAL-SEARCH-001`: `desktop/e2e/tdrp-canonical-search-contract.spec.ts` plus shipping renderer typecheck in `.github/workflows/tdrp-search-responsibility.yml`.
- `SEC-TDRP-SEARCH-AUTHORIZED-INPUT-001`: dedupe may only project identities already supplied by the authorized canonical roster/provider; it cannot manufacture or broaden result visibility.
- Dossier: `projects/telegram-desktop-rust/dossiers/search-share-box-row-replacement.md`.

The row is intentionally not `VERIFIED`: final current-head evidence, artifact-bound execution evidence, and independent reviewer acceptance are still open.


### TDRP-R9-EXTERNAL-URL-AUTH-CONTEXT-001 oracle

- `ORA-TDRP-URL-AUTH-ACCEPTED-CONTEXT-001`: ordinary external URLs cannot preserve foreign login credentials, while a server-returned URL may retain server-issued auth parameters only after the existing Fabushi account owner validates the exact first-party origin.
- `INV-TDRP-URL-AUTH-STRIP-FOREIGN-001`: ordinary HTTP/HTTPS external opens remove reserved foreign web-auth parameter names from query and fragment forms before native dispatch.
- `INV-TDRP-URL-AUTH-EXACT-ORIGIN-001`: accepted-auth preservation requires exact normalized first-party origin and rejects embedded URL credentials.
- `INV-TDRP-URL-AUTH-NO-RENDERER-UPGRADE-001`: generic renderer/MainEdge input has no accepted-auth capability.
- `PROP-TDRP-URL-AUTH-ENCODING-001`: casing and bounded nested percent encoding cannot bypass reserved-name detection.
- `CONTRACT-TDRP-URL-AUTH-BOUNDARY-001`: `parseServerAcceptedAuthExternalUrl` is composed through the existing Fabushi account owner, while generic open remains on `parseAllowedExternalUrl`.
- `INT-TDRP-FABUSHI-ACCOUNT-EXTERNAL-OPEN-001`: authenticated browser login validates the server URL origin immediately before native external open.
- `E2E-TDRP-EXTERNAL-URL-AUTH-001`: `desktop/e2e/tdrp-external-url-auth-policy.spec.ts` plus shipping typecheck in `.github/workflows/tdrp-external-url-auth-responsibility.yml`.
- `SEC-TDRP-URL-AUTH-STRIP-001`, `SEC-TDRP-URL-AUTH-EXACT-ORIGIN-001`, `SEC-TDRP-URL-AUTH-PRIVILEGE-001`: token stripping, exact-origin acceptance, and no-renderer-upgrade security cases.
- Dossier: `projects/telegram-desktop-rust/dossiers/external-url-auth-context.md`.

This row is intentionally not `VERIFIED`: current-head artifact-bound execution evidence and independent reviewer acceptance remain open.


### TDRP-R9-SHARE-FORWARD-PRIVACY-001 oracle

- `ORA-TDRP-SHARE-FORWARD-PRIVACY-001`: forwarding applies one canonical privacy snapshot before destination Message persistence; hiding a media caption also hides sender provenance, and replay cannot broaden or change that snapshot.
- `INV-TDRP-FORWARD-CAPTION-IMPLIES-NO-SENDER-001`: `drop_captions=true` normalizes to `drop_sender_names=true`.
- `INV-TDRP-FORWARD-PRIVACY-NATIVE-001`: privacy is executed by canonical native `ForwardPrivacy::normalized()` and destination Message mutation, not renderer-only hiding; shipping Host reuses the same executable policy.
- `INV-TDRP-FORWARD-PRIVACY-IDEMPOTENT-001`: identical request identity may replay only the identical normalized privacy snapshot; a changed snapshot fails closed.
- Required evidence IDs: `PROP-TDRP-SHARE-FORWARD-PRIVACY-001`, `CONTRACT-TDRP-SHARE-FORWARD-PRIVACY-001`, `INT-TDRP-SHARE-FORWARD-PRIVACY-001`, `TEMP-TDRP-SHARE-FORWARD-PRIVACY-001`, `FAULT-TDRP-SHARE-FORWARD-PRIVACY-001`, `REG-TDRP-FORWARD-PRIVACY-001`.
- Dossier: `projects/telegram-desktop-rust/dossiers/share-box-complete-read.md`.
- Oracle detail: `projects/fabushi-communication-platform/quality/oracles/share-forward-recipient-oracle.md`.

This row remains `IMPLEMENTED`, not `VERIFIED`, until current-head native/Host integration, Electron E2E, temporal/recovery and independent evidence are terminal-success.

### TDRP-R9-SHARE-RECIPIENT-ELIGIBILITY-001 oracle

- `ORA-TDRP-SHARE-RECIPIENT-ELIGIBILITY-001`: canonical recipient Search authorizes a candidate before SearchIndex/UI exposure using current Conversation/Community policy; shipping Host discovery passes the actual source message identity and executes the same source-neutral authorization function before exposure.
- `INV-TDRP-RECIPIENT-AUTH-BEFORE-EXPOSURE-001`: an unauthorized destination never crosses the Search provider boundary. This includes protected source content, incompatible standard/secret destinations, and typed send-other/inline/game requests; permission axes not yet represented by the canonical Conversation owner fail closed instead of default-allowing.
- `INV-TDRP-RECIPIENT-MEDIA-POLL-ELIGIBILITY-001`: media/poll requirements are evaluated before indexing/exposure.
- `INV-TDRP-RECIPIENT-CHANNEL-POSTING-001`: channel recipients require current owner/admin posting authority and membership state.
- Required evidence IDs: `PROP-TDRP-SHARE-RECIPIENT-ELIGIBILITY-001`, `CONTRACT-TDRP-SHARE-RECIPIENT-ELIGIBILITY-001`, `INT-TDRP-SHARE-RECIPIENT-ELIGIBILITY-001`, `TEMP-TDRP-SHARE-RECIPIENT-POLICY-REFRESH-001`, `FAULT-TDRP-SHARE-RECIPIENT-ELIGIBILITY-001`, `REG-TDRP-RECIPIENT-PREEXPOSURE-001`.
- Dossier: `projects/telegram-desktop-rust/dossiers/share-box-complete-read.md`.
- Oracle detail: `projects/fabushi-communication-platform/quality/oracles/share-forward-recipient-oracle.md`.

This row remains `IMPLEMENTED`, not `VERIFIED`, because shipping recipient-picker composition and exact-head Electron evidence remain open.

### FBCP-NMR-001 native Human admission and settlement

#### Round17 current extension — auth parameters, Settings recovery and actual AppHost copy

Base `23f737faba69b4f60eecb6ec1a031602f87f60dc` / run41451598911 is terminal
success. Full logs qualify Human33/Agent4/private3, composition11/Native6,
FeatureHost7/Host6/C ABI7, protocol122/TCP9 (including live), UI16 and Electron
10/8/3 at that base. The former round16 parameter-matrix claim was authored
intent, not the file contents: this batch supplies those missing assertions.

| Requirement / oracle | Actual owner and case | Evidence boundary |
| --- | --- | --- |
| NMR-38 / NMR-AC-28 | `native_desktop_account_handoff.rs::assert_legacy_auth_matrix`: 12 routes × 9 parameter forms, 11 extra registration cases, 6 unknown-method controls; exact ID/result/error and immediate event refusal | 125 requests per invocation through actual Production AppHost; same six subprocess scenarios and Native/Agent/retained-data positives. |
| NMR-38 / NMR-AC-28 | `assert_settings_restart_matrix`: 29 typed rows cover all15 mutations; 6 compatibility retries, 8 invalid and 6 malformed mutations, 2 persistence failures and 2 recoveries; complete snapshots/disk/restart | Four publication variants require unavailable; others require null. No Native transport or full Settings UI acceptance is claimed. |
| NMR-38 / NMR-AC-29 | Actual shipping AppHost normal/build graph, locked executable, SHA256, defined C symbols and dynamic libraries | Base23 artifact13145291819 already inspected; repeat affected gates at containing H. |
| NMR-39 / NMR-AC-30 | Installed locked app-builder-lib FileSet/copy implementation, Unix/Windows-named fixture bytes and modes, sentinel/same-name directory/unrestricted controls | Three original copy cases passed base23; original staging and rollback readers remain intact. |
| NMR-40 / NMR-AC-31 | `native-package-resources.contract.test.mjs` actual-built-AppHost case; source/build/destination hash, executable mode, exact inventory; Actions missing-input/hash expected failures and copied Mach-O/symbol/library checks | New fourth macOS case uses real built bytes and uploads them under resources/bin. This is a bounded package resource, not a signed/installed full application. |
| NMR-41 / NMR-AC-32 | `product_proxy_remove_preserves_surviving_preference_identity_and_order`: 10 preference orders × 3 removal positions, plus absent no-op; dispatch matrix removes first/middle/inserted/last and reopens Settings | REG-NMR-PROXY-REMOVE-ADJACENT-001. Deliberate correction of fixed-U inherited defect. Composition11 becomes12; expanded Native top-level count stays6. |

The same canonical ledger rows `FBCP-NMR-001` and
`TDRP-R9-PROXY-SETTINGS-STATE-PERSISTENCE-001` carry current wiring, base-H
execution and new authored scope separately. `evidence/round17-auth-settings-actual-package.json`
binds full-log hashes, actual artifacts, source identities, backend read-only
findings and the defect. Source review counters and final acceptance stay
unchanged. Containing-commit execution is pending at preparation.

#### Round 15 extension — desktop messaging boundary and retained data

Base H is `c15dfbd9631f8b9fb6920cb5f80b9aea35210d07`. Run 41438907013
ended failure with verified Electron carrier10/auth8 and UI16. Both Rust jobs
were cancelled without runner assignment/steps; logs return BlobNotFound.
No Rust count is accepted and the exact scheduling cause remains unverified.

Recovery binds NMR-34/35 to `98e51b31b5a296065190f665696b2ffe11a3efe1`:
run `41443460672` completed success with every selected actual count below;
source checker run `41443460908` also passed while source/product acceptance
remained false. NMR-36 is a subsequent production extension, not covered by
those earlier executions.

| Requirement / oracle | Existing owner and executable case | Execution / acceptance boundary |
| --- | --- | --- |
| NMR-34 / NMR-AC-25 | `FeatureHostController` guard at sync execution, blob read and access issuance; `desktop_messaging_boundary_tests` covers direct and typed routes, malformed inner payloads, absent roots, poisoned state and immediate event non-publication | 7 authored owner cases; actual Production AppHost separately exercised. Test/non-Electron positives prevent a blanket disable. |
| NMR-35 / NMR-AC-25 | Original snapshot/blob/access writer-reader compatibility; nested `_messaging` automation discovery/override/write/cleanup exclusion and normal Agent create/update/delete | Real data/byte and negative authorization oracles; no import, cleanup or mobile acceptance claim. |
| NMR-AC-25 | Existing `native_desktop_account_handoff` target: 3 previous cases plus missing/retained/corrupt standalone store cases; explicit routes and generic wrapped/unwrapped dispatch in actual Production subprocesses | 6 authored tests; production/isolated issuers, A/B/logout/relogin/restart, immediate real event drain, Native and Agent positives. |
| NMR-AC-26 | Existing `native-service-contracts.yml`, plus unchanged `tdrp-source-review-contract.yml` for the canonical ledger | Exact containing H must execute owner7/AppHost6 and all prior Human33/Agent4/private3/composition11/UI16/Electron10+8/protocol/TCP gates. No queue or authored count is a pass. |
| NMR-36 / NMR-AC-27 | `MahayanaHost::create` and test-support constructor; `mahayana-host/src/legacy_handoff_tests.rs` | Six authored public-constructor cases: value/serde matrix, absent and existing data, restart, forbidden backend use, and native Agent completion. Subsequent exact-H execution required. |
| NMR-36a / NMR-AC-27 | `mahayana_runtime_create` and actual exported status/receive/last-error/free/close; `mahayana-ffi/src/legacy_handoff_tests.rs` | Four new C ABI behaviors plus three existing lifecycle/error controls; original data/ABI/readers retained. Default dependencies and direct old-client exports are not retired. |

`evidence/round15-desktop-messaging-boundary.json` records exact read identities,
backend read-only audit, predecessor execution, design finding R15-D1 and its
FIX disposition. The existing ledger row is reconciled in this batch; its old
exact95 evidence remains historical. No inventory/source-review counter or
independent ACCEPT is promoted.

#### Round 14 extension — Native startup isolation and retained legacy data

The following extends the same responsibility; the older exact95 evidence
below retains its historical scope and does not qualify this candidate.

| Requirement / oracle | Existing owner and executable case | Execution / acceptance boundary |
| --- | --- | --- |
| NMR-31 / NMR-AC-22 | `FabushiAuthService.writeHostCredential`; `source/product/fabushi/fabushi-account-service.behavior.test.mjs`: real Native descriptor, legacy-secret no-access/retention and login/refresh/restart/logout/account-isolation behavior | Actions `native-message-electron`; authored, execution pending at commit preparation. |
| NMR-32 / NMR-AC-23 | `AppHost::new_with_feature_mode_and_storage_passphrase` → `create_feature_host`; `native_desktop_account_handoff` with old v1 optional legacy fields, actual provider rejection and Native account/Agent positive controls | Actions `native-message-shipping-compile`; Production mode and subprocess restart, not Test-mode substitution. |
| NMR-33 / NMR-AC-23 | `native_startup_preserves_encrypted_legacy_storage_for_rollback`; `native_startup_ignores_corrupt_legacy_database_and_preserves_sidecars` in the existing account-handoff test target | Compare retained files after each process; decrypt retained metadata/snapshot with the original storage API after shutdown. No data migration or package rollback claim. |
| NMR-AC-24 | Existing workflow compiles shipping `source/host/app`, runs the three AppHost cases and Electron behaviors; preserves Human33, Agent4, private settlement3, UI16 and Electron carrier10 | Complete logs/counts/artifacts must bind `github.sha`; final FBCP/TDRP ACCEPT remains false. |

The round-15 reread found that the canonical FBCP-NMR-001 ledger row still held
the older exact95/round-4 scope despite this section's previous update claim.
This batch adds the missing target/test/preparation anchors to that same row,
preserving its historical execution records and source counters. FMEP-SOURCE-01
is already represented by `source_reviews` in the current schema and
`scripts/tdrp-source-review.mjs`; the existing Actions source-review contract
also validates the current ledger shape. No invented read-complete or new
schema state is needed for this composition correction. Global single-state
acceptance remains open for non-desktop FeatureHost authority, old data import,
legacy FFI/direct Host injection/default links; see the current owner inventory.

Scope and authority: `docs/specs/fbcp-native-message-reconciliation.md` NMR-01/02/03/04/05/06/09/18/19/20 and NMR-AC-01/02/03/04/05/12. The bounded responsibility is recorded in the existing TDRP ledger as `FBCP-NMR-001`, with existing owner anchors `MSG-SEND-SETTLEMENT`, `MSG-REACTION` and `NETWORK-SYNC-REALTIME`; no new message owner or parallel ledger is introduced.

The actual shipping route is `ProductionRenderer::sendComposerPrompt` / reconnect-open -> `session/gateway.rs` Human commands -> `ProductionSessionWorkers` -> issuer/account-bound `FabushiNativeMessagingClient` and `/api/social/messages` -> existing `SandAgentDb` transcript/dispatch state -> existing transcript mutation and renderer projection. The complete `MessagingService` engine is not asserted to be the native Human send/sync path. Existing forwarding and recipient authorization reuse their shared policy functions independently.

All case symbols below are in `source/host/tests/session_gateway_contract.rs`. They are executable Host/SQLite/loopback cases, not policy-only IDs:

- `shipping_human_send_uploads_attachment_and_binds_reply_to_server_identity`
- `shipping_human_send_recovers_ambiguous_dispatch_after_restart_without_reupload`
- `shipping_human_reaction_toggles_backend_truth_and_updates_canonical_transcript`
- `shipping_human_sync_recovers_multi_device_gap_until_known_remote_overlap`
- `shipping_human_sync_rejects_foreign_participants_before_resources_or_overlap`
- `shipping_human_sync_keeps_peer_nonce_separate_through_local_settlement_and_restart`
- `shipping_human_sync_rejects_conflicting_local_acknowledgements_before_resource_access`
- `shipping_human_sync_rejects_canonical_ack_duplicates_with_missing_or_changed_nonce`
- `shipping_human_legacy_peer_nonce_is_safe_in_reactions_and_local_send`
- `shipping_human_sync_retries_resource_gap_after_restart_before_later_overlap`
- `shipping_human_reaction_summaries_preserve_allowed_states_and_exact_unicode_after_restart`
- `shipping_human_sync_rejects_invalid_reactions_before_resources_and_known_overlap`
- `shipping_human_reaction_toggle_rejects_without_mutation_and_recovers_after_restart`

Requirement bindings:

- NMR-AC-01: foreign-participant rejection preserves the stored row and issues no attachment GET; valid independent progress remains observable.
- NMR-AC-02: the strengthened peer-nonce case first creates a real failed-dispatch pending intent, admits a peer row using the same nonce without consuming that intent, reopens the DB, then allows only the matching local acknowledgement to settle the local slot. The legacy-peer case separately preserves historical content and durable nonce provenance.
- NMR-AC-03: the strengthened acknowledgement case isolates text, reply identity, resource identity, resource order and missing-resource differences, asserts the specific rejection and unchanged pending/anchor state, then reopens and proves matching-ack recovery with the expected resource bytes. Canonical-ID nonce rejection has an independent correct-nonce control.
- NMR-AC-04 / NMR-09: retain real attachment/reply dispatch, ambiguous-send restart without reupload, reaction-toggle, bounded history-gap and resource-gap-before-later-row cases. Existing `source/host/tests/send_pipeline_nonce_boundary_contract.rs` contributes four Agent admission/nonce regressions; it is not complete Bot acceptance.
- NMR-AC-05: the existing RTM, three owner anchors and the bounded canonical responsibility record the production route; implementation, authored tests, execution and independent acceptance remain distinct.

- NMR-AC-12: all four allowed reaction tuples preserve exact strings/projection across reopen, including NFC-equivalent distinct strings. Five malformed summaries are rejected before resources and known-ID overlap while valid independent pages persist. Invalid reaction POST preserves the entire row and emits no target mutation; a later valid POST recovers after reopen and publishes exactly one durable update.

Authorship/evidence boundary: the original strengthened ten-case blob `127700079e9ae9d5c2155cd1d8a218a47f2f8075` actually passed together with all required message gates at H=`68d4f07688741e8f54b35f3746a087f7fdd12108`, run `41402071662`, attempt1. The three new reaction cases and unchanged original ten are frozen at blob `0728038de6b3fa6df656604a352283715d2cdfff`; the new production blob is `c9a486af86b252ca4c7a025452928caf66439563`. Both were independently statically reviewed and then actually executed at exact H=`95b29df688f718a35574fa36ef4ca75df437ed30`, run `41403643662`, attempt1: all thirteen Human cases and the complete preserved message gates passed. The existing Human fixture's 15-second accept/read deadline and socket write timeout remain. Full run/job/artifact/digest details stay in the existing round3 receipt.

The bounded Host gate passed at exact95/run41403643662. Every subsequent message change remains gated by NMR-AC-06/07/08/09/10/11/12: both named message jobs on one exact H, actual thirteen Human and four Agent nonce cases, locked AppHost compilation, response normalization, both bare future-salts library fixtures, full protocol/TCP including its live case, proxy rejection and credential-schema cases. A skipped step, diagnostic lock generation, test authorship or old-H result is not a pass. Use the existing round3 evidence receipt for execution records; no second result ledger.

Fixed-U mapping uses only the explicitly read peer-scoped dispatch, existing-message application and identity-transition sections recorded in `research/native-message-known-overlap-fixed-u.md`. It does not close whole upstream files or change inventory/source_reviews/unknown/unread/source acceptance. Known-message refresh, durable history-range coverage, read-state/visible receipt behavior, distinct-emoji/whole-payload bounds and stale mutation/refresh ordering remain separate open work. Full native-service, UI/package/platform, performance/soak, complete Bot regression and independent ACCEPT remain open.







### NMR-51: Native Human resource upload service boundary

NMR-51 / NMR-AC-42 extends the existing `FBCP-NMR-001` and `MEDIA-TRANSFER` responsibility; the production path remains Human composer -> `sendHumanMessage` -> `ProductionSessionWorkers` -> `FabushiNativeMessagingClient` -> existing resource/message service and Session/Transcript SQLite. Backend `bhrumom/fabushi-backend@687718ce9e0083aa124d29005bcce4051d357919:fabushi/web/src/handlers/friends.js` (blob `6179f3ce8e223f4e0bd9664885871e134f4488a8`) requires a nonempty resource at most 25 MiB. Native preflight and actual limit+1 reading enforce this service boundary while preserving shared Agent video and download budgets.

- `source/host/tests/session_gateway_contract.rs#shipping_human_oversized_upload_stays_pending_across_restart_and_recovers_same_nonce`: one oversized video is rejected before resource/message HTTP; original pending/error/nonce survives pure DB reopen and runtime restart; corrected bytes recover with the same nonce, exactly one upload/message/download and one canonical row.
- `source/host/src/extensions/session/native_messaging.rs#native_message_resource_upload_tests::exact_service_limit_is_readable_but_empty_and_read_errors_are_rejected`: exact 25 MiB succeeds; empty input and partial-read failure reject.
- `source/host/src/extensions/session/native_messaging.rs#native_message_resource_upload_tests::growth_after_metadata_observation_consumes_at_most_limit_plus_one`: actual consumed bytes stop at limit+1 and oversize rejects.

Authored at freeze; containing-H Actions must execute all 48 Human gateway cases, three focused resource-bound cases and preserved required message gates. Prerequisite H `59f5fa24827812f78336c464995d161f403dedfb`, run `41508537631`, passed all five applicable jobs including Human38, protocol122, TCP9/live and actual AppHost package-copy4; it does not execute this extension. Evidence: `projects/telegram-desktop-rust/evidence/round3-native-resource-upload-20261102.json`. Fixed-U review is limited to the recorded media preparation/file-part sections; source completeness, multi-attachment atomicity, full UI/service/platform/package/Bot acceptance and independent ACCEPT stay open.

The current byte-buffer upload and forward snapshot readers share the literal25MiB service bound. Their digest/path/atomic snapshot/complete-target prevalidation rules remain; existing forward exact-limit/oversize tests use service25MiB literals and all twelve journal cases are retained. `native_message_resource_upload_tests::byte_upload_rejects_empty_and_oversize_before_credentials_or_http` covers the added buffer entrypoint. Existing32MiB download and200MiB Agent video boundaries are unchanged. The pending/same-nonce oracle here is the Host gateway, not a claim that the current UI retry is already correct; fresh-nonce/reply/committed-path/reopen projection remains a separate consumer gap.


## Round21 updater source and exact-run reconciliation

`TDRP-UPDATER-DISCOVERY-AUTHORITY-001` remains the existing Electron updater responsibility. Its primary source is corrected to fixed-U `Telegram/SourceFiles/core/update_checker.cpp@73eee14f179d769854237640b246858959808736`, with exact supporting tuples and actual selected deep-read ranges in `projects/telegram-desktop-rust/evidence/round21-update-source-authority.json`. The previous `mac_packaged.yml` tuple remains contextual build provenance. This is partial product-source understanding, not full-file/updater/legal acceptance. Existing six source reviews (two complete, four pending), all other responsibility rows, acquisition guards and acceptance states remain unchanged.

The original code `52ba898107642db27c87a354687c2c6eb0456054` removes the implicit historical third-party feed through the existing production wiring/service/Settings owners. Primary configuration has precedence; only an absent primary permits explicit legacy configuration. Missing/invalid selected authority stays disabled before network I/O, and startup still settles the existing apply marker. Later Windows publisher authorization and recovery remain intact.

Exact `73cc6e1cc9fa871e5f80931979606ac9ee810551`, run `41514576570`: Electron job `125514709195` passed source16/publisher28/carrier10/auth8/copy-fixture3/metadata18 and the real Electron build; Windows job `125514709410` passed11 actual native Authenticode/Unicode cases with inert CI-only fixtures; UI job `125514709141` passed Transcript16/settlement5/Chromium16 and the actual renderer build. These are component proofs at that H, not formal publisher, installed-update or whole-migration acceptance. Full artifact identities and log hashes are in the same receipt.

Exact `94dcf7a2213808a8ded6baddaf069562d365c6d7`, run `41513767498`, completed all five selected jobs successfully. Reconciliation passed Human44/sharebox1/journal12/Agent4/private3/HTTP1. Shipping job `125511939405` passed gateway executable5/library11, protocol122, TCP9 and all declared supporting suites. Copy4 means3 fixtures plus1 actual160158128-byte AppHost, mode0755, matching source/destination SHA-256 `708a20213b1363a2ac72153273b917d28a9b03cf1e19c5c4619291cfc93c956b`. Missing-input, invalid-hash and unequal-hash negatives all refused; inner TAP counts for the first two were not printed and are not invented. Original `8ce0a9a` shipping remains cancelled/unexecuted. Later Human recovery inputs need their own CI and do not inherit94dc as execution.

Source run `41514576605` at73cc passed157 source contracts and37 schema cases with zero ledger-shape errors; candidate remained unaccepted with16381 inventory identities/184 responsibilities, six review records, unknown/unread null. The final containing ledger commit requires its own source gates. Backend PR137 remains read-only, open/draft/unmerged at audited52fb; actual online version/authenticated pagination, formal feed/publisher approval, signed platform install/upgrade/rollback, full product/source/legal closure and independent ACCEPT remain open.

### NMR-PAGING-01 / NMR-ACTIVITY-01 — resumed round 6

The actual production path is the existing Human workspace -> Coordinator `getHumanConversationTranscriptTail` -> Session gateway -> `ProductionSessionWorkers` -> `SandAgentDb` / `agent_db_transcript_pages`, returning through the existing pagination/projection and root cache. The SQLite owner stores a bounded sequence index; it does not copy message payloads into another store. Human full reads and new walks use the canonical timestamp/remote-ID/sequence order; continuations preserve membership while returning current payloads.

| Requirement | Behavioral oracle | Authored test / execution boundary |
|---|---|---|
| NMR-PAGING-01a/b | Same-time and late inserts, settlement and timestamp changes cannot skip/repeat a saved walk; corrupted/expired/scoped indices fail; restart/retry is exact | `shipping_human_tail_keeps_physical_anchors_stable_during_backfill_and_equal_time_inserts`; `shipping_human_snapshot_cursor_survives_pending_settlement_and_authoritative_anchor_refresh` |
| NMR-PAGING-01c | Opaque token survives the existing cache/reopen chain; failed pagination keeps committed updates; Human nonce correlation respects author; Agent shape/merge unchanged | Existing `transcript-projection.contract.test.ts` plus `recovered-conversation-infra.contract.test.ts` and shared RPC behavior cases |
| NMR-ACTIVITY-01 | History and Agent handoff serialize at the existing lane; older timestamps cannot regress activity; failed metadata commits report failure and replay repairs the durable row/metadata gap | `shipping_human_agent_handoff_and_history_download_keep_activity_monotonic`; `shipping_human_activity_write_failure_keeps_committed_rows_and_repairs_after_restart` |

The existing exact-H Actions job retains every current message, credential, reply/forward, upload, AppHost/shipping, UI/Chromium/Electron and Windows gate. At the integrated freeze the authored selected counts are Human52, projection20 and cursor/RPC17; execution remains pending until the containing-H logs show the actual results. Two concurrent NMR49 cases retain their complete physical-row/atomic/permission assertions and separately assert the new public canonical ordering. Existing prerequisite results are provenance only; see `projects/telegram-desktop-rust/evidence/task20bf-round6-resumed-stable-pagination.json`. Full source/legal, deployed Worker, supported signed packages, UI/a11y/temporal/Bot/soak and independent ACCEPT remain open.

## PS-UI-01..05 canonical proxy preference settlement

Spec: `docs/specs/fabushi-product-settings-owner.md` §14. Responsibility remains
`TDRP-R9-PROXY-SETTINGS-STATE-PERSISTENCE-001` in the unique TDRP ledger.

| Requirement | Actual production owner/path | Authored evidence and remaining gate |
| --- | --- | --- |
| PS-UI-01 / PS-UI-02 | Existing SettingsDesktopSurface → GeneralSettingsPanel → networkProxy adapter → DesktopBridge → Main RPC → Coordinator MainData → Native ProductSettingsOwner; full result preserves canonical settings and separate publication | Actual-surface timeout same-value normalization and five publication cases; unavailable/unconfirmed never claims connected routing |
| PS-UI-03 / PS-UI-04 | Surface lifetime/write tokens plus adapter in-flight promise barrier; accepted Host write continues after close, reopened read waits then gets canonical state | Duplicate/pending controls with independent Theme, failed endpoint/timeout drafts, close/unmount/bridge ABA, pending success/error reopening and fixed read-error Retry; 18 authored Chromium cases total |
| PS-UI-05 | Existing native-message-ui Actions job, production Settings fixture and unchanged Native Settings persistence/restart contracts | Exact2ac00f98ad7831f351ad2de986abca91bceee2b0/run41524706583/job125549341413 Settings18 and actual renderer build passed. Exactdb27/run41526294942/job125554766420 repaired renderer132 plus both strict architecture gates passed; new integrated-H checks and shipping remain open, together with Native proxy transport, supported packages, visual/a11y/security/upgrade/rollback and independent ACCEPT |

Test: `frontend/src/recovered/features/settings/overlay/network-proxy.behavior.test.mjs`.
Current-H results stay in `projects/telegram-desktop-rust/evidence/task20bf-round1-forward-intent-ui.json`;
no source-review count, historical Rust execution or independent acceptance is promoted.
### Round7 resumed NMR-22/24/26 boundary evidence

Current additions retain the existing FBCP-NMR-001 row and production route.
Eight new `shipping_human_` cases in `session_gateway_contract.rs` cover short/empty
32nd pages, durable partial-page replay, stable equal-timestamp beforeSeq,
applied-row/uncleared-intention restart, malformed persisted target admission,
post-HTTP target changes, and actual gateway pre-staging refusal with a numeric-ID
positive control, plus canonical message-ID and strict page-order rejection with
checkpoint-preserving restart recovery. All prior Human/forward/Agent/credential/shipping cases remain.
Nine new cases in `transcript-projection.contract.test.ts` exercise the actual
completion fence and intermediate-render viewport anchor. Existing tail merge,
read buffers, cursor cache, immutable forward UI and one Transcript remain.

The round7 evidence receipt separates original failed63da, passed fixture30b4,
qualified later predecessor94dc, unpublished superseded preparation and the new
containing-H candidate. Authored source counts must include all concurrent Human
and Transcript additions; actual results require the final exact-H Actions logs.
No mounted reaction/visual/a11y, source or full independent acceptance is claimed.

### Resumed round16: exact delivery correction, auxiliary inputs and retained C wrappers

This annotation preserves the dated snapshots above. Actual round16 delivery
was `23f737faba69b4f60eecb6ec1a031602f87f60dc`: its run `41451598911`
passed the built AppHost dependency/symbol/library checks and three synthetic
package-copy cases, but did **not** contain the locally prepared twelve-route
account matrix or the original ledger/owner-matrix edits. The later verified
`59f5fa24827812f78336c464995d161f403dedfb` run `41508537631` includes
the actual AppHost resource copy, Human38 and the separately recorded later
composition/CLI/gateway checks. Neither result qualifies subsequent Session,
UI or source-review changes. The old thirteen-Human paragraph above is the
exact95 historical gate; containing-H workflow requirements remain authoritative.

| Requirement / owner | Actual input or executable case | Evidence and remaining gate |
| --- | --- | --- |
| FMEP-SOURCE-01 / existing desktop package builders | Coordinator and box Cargo.lock; network worker go.mod/go.sum; existing npm, Rust CI and macOS release commands require locked/readonly inputs | Capture run41514329200 at565a4639f1b7efbd15937aff93f4e14168322389, job125513870853, artifact13175461007 SHA25685488acba970ec85e92327d8b5e1726061a69ffd1caf6b70c19cc83b8813c18b. Four input files match the original artifact bytes. Capture Mac jobs skipped; pinned-H actual binary/owner verification remains required. |
| NMR-AC-30 / same Coordinator, Box and network-worker owners | `staged-runtime-source-contracts.yml`: required tracked inputs; actual macOS executables; normal/build dependencies and symbol/library refusals; Coordinator1+1+4, Box13+Host supervisor3, Go13 | Authored containing-H gates, not executed at preparation. No fabricated substitute binaries; actual Box path is supplied to the Host lifecycle fixture. Full signed package, other platforms and dependency source/legal acceptance remain open. |
| NMR-39 compatibility retention / existing legacy C exports | `providers/telegram-runtime/tests/runtime_ffi.rs`: five direct create/persistent-create/execute/close/free cases, persistent close/reopen, owned response frees, null/UTF-8/key-length/closed-handle refusal | Fifteen non-live tests plus the original one explicitly ignored live-DC test are required by `retained-direct-ffi-compatibility`; authored, not run at preparation. Invalid-length buffers are allocated; all stores/keys are synthetic. This does not qualify a final shared/static library or installed external consumer. |

Concurrent commit `e13c34a5bfc9eddfc5306102e7ad68d4e864bd4e` already
integrated the identical four inputs, desktop locked builders, capture=false and
release readonly build. This batch preserves that delivery, adds the remaining
CI/release input guards and the retained C-wrapper target; it does not replay
or claim authorship of the concurrent pin commit.

The authoritative receipt remains
`projects/telegram-desktop-rust/evidence/round16-apphost-package-boundary.json`.
The unique ledger's two existing responsibility rows retain source identities,
review depths, all concurrent source reviews and unaccepted release states.
Original SQLite/sidecars/keys/readers are preserved. Backend PR137 remains
open/draft and read-only; its skipped production readiness job is not service
acceptance. External mobile/installer producer-consumer mismatches remain
separately recorded gaps, with no external repository mutation.

### Round 19 execution binding — journal, direct Host and Plugins Close

Fixed upstream remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. The code batch below is
`682aea58b4fbac7b1b509c30fd565a12c4af5a2f`; later concurrent checkpoint changes require
their own exact-H evidence. This annotation preserves the historical failures and all existing rows.

| Existing requirement / owner | Actual execution and production binding | Remaining qualification |
| --- | --- | --- |
| NMR-47e / Session forward source journal | Expanded stale-snapshot oracle uses two SQLite handles, raw metadata preservation, terminal replay and reopen; exact682 reconciliation job125536673830 passed Human54/resource3/sharebox1/journal12/Agent4/credential3/forward HTTP1, including the expanded stale and concurrent-writer cases | Twelve journal cases are the top-level count; full forwarding service, migration and rollback acceptance remain open |
| NMR-41 and NMR-CLI-HOST-01 / existing AppHost preferences and direct MahayanaHost consumers | Exact94dcf7a2213808a8ded6baddaf069562d365c6d7 shipping job125511939405 executed all six proxy permutations:16 ordered subsets,48 removals,48 repeated-removal no-ops,18 full-permutation deletions inside one Rust matrix; CLI3/gateway5/library11 passed, including eight valid approval aliases and nine invalid zero-dispatch controls | Original5281 shipping failed E0004 before approval tests; historical94 cannot qualify682 shipping125536674079, still queued without runner or steps at20:56:52 UTC. Current process controls4 and actual AppHost copy remain unexecuted |
| NMR-PAGING-01c / existing Coordinator reply converter and Transcript | Preserve exact29 UI TS2741 failure after20 Transcript/17 cursor tests; concurrent977 repaired the missing human-transcript-page converter. Exact682 UI125536673715 passed20/17/settlement5/Chromium16 and renderer compilation | This preserves one message owner and projection; it is not full real-service, visual or accessibility acceptance |
| FPM-UI-CLOSE-01 / ProductionRenderer overlay state and PluginsDesktopSurface onClose | Exact682 run41520998019/job125536673032 passed two real Electron pointer cases at1024x684 and1280x900, with four normal Close clicks and successful hides across initial/reopened states. Downloaded artifact13177399337 matches347040 bytes/SHA256c1eb66fd578f8321ca2f776d0566e2476f5990eaaf63aae435514efd52ae8f10; frame y52/z5002, hit-tests and no-drag confirmed | Controlled Host/loopback auth; native window button actions0. Reopened PNGs capture catalog loading. Current signed installer, full visual/a11y and real-service acceptance remain open |

The bounded FFI review traces Electron carrier -> Coordinator -> AppHost -> FeatureHost -> MahayanaHost,
and direct CLI/gateway Host construction. The retained FFI exports and five legacy link anchors stay in place:
repository-external consumers remain unknown. The inspected production graph and source manifests do not
constitute current-H binary or external-consumer acceptance.

Evidence, full-log identities, actual artifact/geometry/trace records, bounded four-image review,
source manifest and the latest authority refresh are recorded in
`projects/telegram-desktop-rust/evidence/round19-resumed-terminal-progress.json`.
The unique ledger already links that receipt; no source-review completion counters are promoted.
Backend PR137 remains read-only/open/draft and unmerged. Complete source/legal accounting, current signed
packages and stapling metadata, supported platforms, real services, Bot regressions, security/performance,
soak, upgrade/rollback and independent ACCEPT remain required and unaccepted.

### Request round 9: NMR-23a / NMR-26a / NMR-26b

| Existing requirement | Actual owner / authored behavioral evidence | Execution boundary |
|---|---|---|
| NMR-23a | Session coverage version1; legacy metadata preserved until matching historyCursor page; invalid versions refuse before same-conversation replay; new legacy-upgrade/reopen case plus retained6400/32 and short/empty32 | Human63 in containing-H Actions; no production Worker claim |
| NMR-26a | Existing projection/root/older-page caches compose explicit empty fields and identity remaps while preserving Agent semantics | Projection33 plus actual ProductionRenderer11; existing cursor17 retained |
| NMR-26b | Existing account/queue/feed owners fence delayed acknowledgements and drain offline sends once after connected state commits | Actual renderer switch/logout/in-flight-or-queued account reset/reconnect scenarios; Electron/carrier and shipping gates retained |
| FMEP-SOURCE-01 | Existing authority CLI subprocess rejects real open acquisition after candidate markers are removed; source review schema/helpers preserved | Actions-only regression; no source or product acceptance promotion |

Exact evidence is recorded in the containing request-round9 receipt; authored
cases, prerequisite results and actual execution remain distinct. All other
responsibility rows and current acceptance requirements remain unchanged.

### Round23 desktop Social owner-boundary controls

| Requirement / acceptance | Existing production owner and direct oracle | Execution scope |
| --- | --- | --- |
| NMR-SOCIAL-OWNER-01; NMR-SOCIAL-AC-02 | AppHost/Host Production Electron Session policy; ordinary/Codex registry selection and native retained-data handoff | Current composition/native suites; historical59f success is scoped evidence, not all-current approval |
| NMR-SOCIAL-OWNER-02; NMR-SOCIAL-AC-01 | FeatureHost real execute: exact early refusal before absent runtime, poisoned lock, file or event side effects; explicit Test/non-desktop/unrelated controls | Existing desktop_messaging_boundary_tests: seven retained plus three added, ten required at containing H |
| NMR-SOCIAL-OWNER-03; NMR-SOCIAL-AC-02 | Runtime real SQLite recovery preserves Social records and continues ordinary Agent work | retained_social_recovery_tests: nine; source-comment ID correction only in this batch |
| NMR-SOCIAL-OWNER-04; NMR-SOCIAL-AC-02 | Existing session-owned platform client validates actual URL/origin/path before credentials and disallows desktop redirects | session_owned_platform_: six; unchanged production implementation |

See `docs/specs/fbcp-desktop-social-owner-boundary.md` and
`projects/telegram-desktop-rust/evidence/round23-owner-boundary-recovery.json`.
H0 Windows hook timeout and later Windows11 success retain their actual run
identities. Formal publisher, real package install/upgrade/rollback, authenticated
production pagination, source/legal/full Bot and independent ACCEPT stay open.

The existing workflow now also runs these same Feature10/Runtime9/HTTP6 behavioral
harnesses on Ubuntu while retaining the queued macOS shipping requirement. This
focused job is not a signed-package, live-service or independent acceptance gate.

### Task20bf prerequisite recovery in the frozen217 integration

| Existing requirement | Production change | Required exact-H evidence |
| --- | --- | --- |
| NMR-50d | Original Forward dialog/account predicate reaches the existing Coordinator call after ready and before pending registration/post; accepted Host work is retained after presentation retirement | Actual client/controlled-port13 plus real dialog/client7 new cases; retain original Forward16 and all settlement/Agent gates |
| HRR-04 / HRR-AC-02 partial prerequisite | Existing ordinary Human send staging lane refuses to reuse source-owned or unresolved provenance without its original context before upload/POST; original batch recovery remains authoritative | New `shipping_human_forward_ordinary_send_rejects_lost_provenance_before_dispatch`: cold/cached,12 corruption scenarios, retained bytes, source denial, exact recovery and ordinary-send positive controls; Human union65 |
| NMR-26 pagination/refresh | Same pagination/projection owner coalesces repeated failed-read events, retains authored alias candidates, Human clears and later partial fields, and bounds replay by identities | Five new actual controller cases, including no-nonce cross-author alias ordering, first post-failure update and measured superseded-payload replay; projection union38 |

These are prerequisite fixes, not completion of the new `retryHumanMessage` command
or its reload/retry UI. All newly combined input requires containing-H Actions;
previous results keep their original SHA. The existing task receipt records the
reviewed production paths, test selections, failures and later execution separately.
Full source/service/package/quality/Bot/upgrade/rollback and independent ACCEPT remain open.

### Task20bf current recovery after exact7f8 failures

| Requirement | Existing production chain | Authored oracle and current evidence boundary |
| --- | --- | --- |
| NMR-50a/b/c | Session sync → nonblocking automatic batch admission / one held delivery guard → current source/target policy and native send; roster → one scan/private history helper → native bound friends/full completed-body credential fence | Four real Host/HTTP/SQLite/barrier/reopen cases; expected Human69 at repair H. Explicit sends retain serialization, corrupt/storage/poison errors fail, wrong user gives zero HTTP, six credential rotations preserve persisted state and recover. Static review passed within scope; execution pending. |
| UI-NMR-CURSOR-ACTIVATION-001 | ProductionRenderer committed initial/cache/feed rows → existing Transcript near-top effect → unchanged pagination cursor/single-flight/anchor owner | Original Renderer11 retained, original Transcript8 plus1 no-scroll A/B activation/pending/failure/keyboard Retry case. H7f actual first114 UI cases passed but Renderer1/10 failed and Transcript8 did not execute; repair-H expected UI134, not a current pass. |
| Preserved test/oracle gates | Existing native-service workflow and structured gateway error | H7f Human compile E0599/E0277 repaired in test only; Social Feature10 actually passed and its expected caught-panic line capture is repaired without removing three named gates. Runtime9/HTTP6, Human69 and same-H shipping must execute successfully. |

Exact run/job/artifact identities and independently reviewed blobs remain in the
existing `task20bf-round1-forward-intent-ui.json` receipt. Fixed U, unique inventory,
source review counters, final service/package/Bot/quality and independent ACCEPT
are unchanged; no partial suite or authored case is promoted to full acceptance.


## Round11 — run290 native-message reconciliation and transcript UI evidence

At exact H `740f2eee1759874bc97686cddfa29250130ea7db`, run `41691957848`
completed the requested jobs: reconciliation `126112539527` success (Human73,
0 failures), UI `126112539660` success (Transcript10, 10 passed/0 failed),
and shipping compile `126112539665` success. Reconciliation artifact
`13257813493` and UI artifact `13257063906` were downloaded and inspected.
The Human73 log includes independent SQLite duplicate clientNonce identity
and duplicate HTTP canonical identity after restart negatives. UI artifact
shows UI-NMR-RESELECT-ACTIVE-PAGE-001 passed; browser evidence records one
in-flight older-page request (id23), settled once, rows m21..m64, no
diagnostics and no unexpected RPCs.

Run285's nine ProductionRenderer transcript scenarios all failed at fixture
mount with `ProductionRenderer fixture failed to mount: []`; the screenshot
showed "No chats yet". Coordinator roster calls succeeded without unexpected
RPCs, but the fixture's peer-only participantIds were filtered by the
existing `projectHumanConversation` contract requiring exactly two distinct
nonempty string identities. The fixture now asserts the Host contract and
the source contract documents the local-account/peer pair.

At new exact H `50bc3d275dff24be1bf60a13810fce93ae568930`, the foreign
retry test now prints and compares the actual `SessionGatewayError`; the
fixture enforces two distinct nonempty string participant identities; the
NMR-26 specification records the root cause and active-page reselection
oracle. This is authored evidence only. Run291 `41694984873` and source
run9649 `41694984872` are the required current-H gates; no current-H pass
is inferred from run290. PR #189 remains draft.

## Round12 — exact-H qualification and retry admission follow-up

Run291 `41694984873` at exact `50bc3d275dff24be1bf60a13810fce93ae568930` passed: Human73/0 failures/0 ignored, including independent SQLite duplicate clientNonce before HTTP, HTTP duplicate canonical identity after restart, concurrent retry idempotency and attachment receipt reuse/settled replay. Foreign retry actual was `Internal("local Human identity is not a conversation participant")`, matching the explicit oracle. UI passed actual ProductionRenderer13 and Transcript10/10 including active-page reselection; shipping compile succeeded. Run91 source contract171/Python schema37/schema errors0; candidate remains unaccepted. Run9650 renderer build/45-asset boundary/strict architecture modules2002 passed; conditional matrix jobs skipped.

Exact H `08f408ac0de6223fb0e7637974bb7cadc8db4927` passes captured current-scope predicate into the existing Coordinator request boundary, and adds four actual-root renderer cases for persisted failure after reload, identity-only single-flight retry, selection ABA and account ABA. Transcript gate14. Run295 `41704725299` and Rust run9658 `41704725388` were queued at evidence freeze; this follow-up is not yet executed. PR #189 remains open/draft.


## Round13 — exact-H Human retry and restored transcript ownership

At exact H `81e022997c943cdb8f0b74f2ff5c0be5f210f96e`, Native service run `41724037792` (attempt 1) is terminal
success. Jobs: native-message-ui `126224000145`, native-message-reconciliation
`126224000021`, native-message-shipping-compile `126224000304`, and
desktop-social-owner-behaviors `126224000268` all completed successfully on
the same exact commit. Actual ProductionRenderer13 and Transcript14 passed;
Human73 passed with zero failures/ignored cases. The Human log printed the
foreign retry error `Internal("local Human identity is not a conversation participant")`
and matched the negative oracle. SQLite duplicate clientNonce before HTTP,
duplicate HTTP canonical identity after restart, concurrent retry idempotency,
attachment receipt reuse after restart and settled replay passed.

The actual ProductionRenderer browser cases passed for persisted
`lastDispatchError` after real reload, identity-only single-flight retry with
failure then retry, selection A-B-A refusal before Coordinator frame, and
account A-B-A refusal before Coordinator frame. The account A-B-A case also
proved the replacement account's Human transcript tail is requested after
roster resolution and `openAgentTail` is not called. The prior test-oracle
message prefix is retained as an explicit assertion; no negative was removed.

Shipping compile built the actual AppHost, CLI and Gateway and ran the selected
native composition, startup, Host/FFI handoff, DNS, future-salts, Protocol122,
TCP9, proxy and credential checks. Actual package resource copy passed 4/4;
built and copied AppHost SHA-256 is
`3eb1a5fa5d374e6400e46830615970ebe9a265e39c667fefb8c264bc0fbedf00`
(size 160462688, mode 0755). The one package-hash mismatch assertion failure
is an expected negative-control result inside a successful harness, not a
workflow failure. The artifact scope does not qualify a signed application or
installed package.

Artifacts: UI `13273695080` /
`sha256:8c1b2f118472889a14632c9b7110355631cdec7ae4f6790c6950263982a17d2c`;
reconciliation `13273975441` /
`sha256:f42d7b3b5b69066319ad6f7f47cc569d30813f07a7fd4e022c6e949a14e128af`;
shipping `13274136085` /
`sha256:48e8ce9cd0f0d59671f825bab0718a0edc014093b6e33e9c5fe595d4813ad964`;
Social `13273508513` /
`sha256:4c03fc4986e23eebfbf0d33b9245c09234616086f02c692cee62905261375993`.
All four ZIPs were downloaded and local SHA-256 matched the API digest.

The fixed upstream remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`,
unaccepted. Recursive inventory remains 16,381 entries; six source reviews
remain 2 complete / 4 pending; unknown/unread remain null. Product/source
closure, live authenticated service, signed supported-platform package,
visual/a11y/security/performance/soak/upgrade/rollback/Bot regression and
independent ACCEPT remain open. PR #189 remains open/draft.
\n\n## Round16 — forward settlement fail-closed negative\n\nThe existing ProductionRenderer settlement projection now treats only explicit sent as success. Unknown runtime statuses remain attention-required and cannot expose an unconfirmed recipient name. The added negative test is authored and awaits current-H GitHub Actions execution. Fixed U remains f985723b552b30fe1c17d377d57f44e61a15a8c4; the 16,381-entry recursive Git inventory and 76 build-input unresolved references remain open. Six source reviews remain two complete/four pending. No source, product or independent acceptance is promoted.\n