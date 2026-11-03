# Requirements Traceability Matrix

Status: active / normative schema  
Matrix ID: FQT-RTM-001  
Parent: FBCP-001 Revision 8 / TDRP-001 Revision 10 / FMEP-001 Revision 2  
Updated: 2026-11-02


## Peer durable mutation boundary — NMR-PEER-DURABILITY-01

Spec: `docs/specs/fbcp-gateway-peer-durability.md`.
Evidence: `projects/telegram-desktop-rust/evidence/round20-peer-durable-mutations.json`.
Unique peer IDs preserve the concurrent NMR48 forwarding-attachment contract.
Execution is scoped to exact9bb/run41528207637 below. Evidence: `projects/telegram-desktop-rust/evidence/round20-peer-durable-execution.json`.
No whole native-workflow or independent acceptance is granted. The separate peer Spec retains non-atomic peer-resolution to Host handoff, possible historical replay before peer issue refusal, replay merge/epoch, power-loss, mixed-version and rollback gaps.

| Requirement | Existing production owner and observation | Required execution / remaining boundary |
| --- | --- | --- |
| NMR-PEER-DURABILITY-01a / NMR-PEER-DURABILITY-01b | `PersistentServerRequestRegistry::mutate` and `issue_approval_request`: **Peer mutation commits before live snapshot or success/frame publication**; **Latest-disk approval identity admission under the same shared lock**. Temporary import/mutation persists before replacing live state. | Exact9bb Linux23 and Windows23 passed. Exact808/run41543732585 jobs125613993935/125613994291 also passed23 each, zero failed/ignored/filtered, including `failed_issue_retains_live_state_then_repairs_and_reopens`, `approval_issue_and_response_lookup_use_latest_transaction` and stale-instance non-resurrection. |
| NMR-PEER-DURABILITY-01c | Existing peer/replay `acquire_state_lock`: **Stable advisory lock inode without timestamp stealing**; fs2 0.4.3 handles, owned temporary cleanup and only-backup retention remain. | Exact9bb and exact808 Linux/Windows peer23 passed, including `held_old_lock_cannot_be_stolen_and_killed_child_releases_it` and nonregular-input refusal. The historical9bb shipping125561310442 was queued; current808 shipping125613994273 is also unqualified at this observation. Mixed-version and power-loss acceptance remain open. |
| NMR-PEER-DURABILITY-01d | Actual `publish_projected_events`, `resolve_peer_response` and `PeerGuardedRuntime::resolve_approval`: **Storage errors cannot become healthy absence or reach Host-facing approval dispatch**. Canonical parser and Native Host features remain. | Existing gateway executable9/library13 include `failed_peer_commit_publishes_no_approval_event_or_frame_and_recovers`, `response_storage_failure_preserves_pending_and_never_calls_host` and compatibility healthy-absence/error controls. They remain unexecuted at current808 while shipping125613994273 is queued; peer-library23 is not their execution. |
| NMR-AC-PEER-DURABILITY-01 | Containing exact-H focused peer suites plus every existing Human/Agent/CLI/shipping/UI/Electron/Windows gate | Exact9bb peer/platform, Social, retained-C and package checks have scoped evidence, while UI and creation failures have separate containing-H repairs. Exact5c source158/schema37/candidate passed without source acceptance. Peer/Host crash handoff, replay transaction/epoch semantics and full migration ACCEPT remain open. |

The canonical ledger is changed only by an append to its existing NMR row;
all current source_reviews, source/legal identities, other rows and acceptance
fields are preserved. Backend137 current52fb fixture CI is distinct from its
skipped production readiness; no merge, deployment or live pagination acceptance.


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

Existing Chromium12 cases are retained and extended to16, alongside settlement5,
the original Transcript16 plus two reply-projection cases (Transcript18), and
current Host/Agent/shipping/Electron/Windows gates. Current-H
execution is pending at authoring; no local executable verification or independent
ACCEPT. Evidence: `projects/telegram-desktop-rust/evidence/task20bf-round1-forward-intent-ui.json`.

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
| NMR-47a / NMR-47b / NMR-AC-38 | Source SandAgentDb journal plus destination Session/Transcript | Five actual HTTP/SQLite/process cases passed in H59 Human38; parent kills and reaps child, reads committed intent independently, resumes untouched target, injects staging/progress SQL errors. Require Human38. |
| NMR-47c / NMR-47d / NMR-AC-38 | Existing credential/source authorization and ordinary sync | Ambiguous first POST reuses nonce; immutable source snapshot; same/subset/concurrent completed replay; changed source/privacy/expanded-set negatives; source/account/issuer denial; no generic-outbox bypass. |
| NMR-47e / NMR-47f / NMR-AC-38 | Existing SandAgentDb expected-value CAS and Session eligibility | Two-handle stale read regression plus typed source denial and unrelated actual history GET; I/O failures still stop. Select sharebox1 and journal3 separately. |
| NMR-48a / NMR-48b / NMR-48c / NMR-AC-39 | Existing `forward_batch_journal` snapshot/read/publication and Session complete-buffer preflight/upload: account/issuer/request-scoped digest+size bytes precede journal admission, and the same validated buffers are uploaded. | Exact808 Human72 includes all five `shipping_human_forward_` attachment-byte cases: process-kill/source replacement; missing/corrupt snapshot repair; invalid input/staging refusal; legacy unpinned refusal without blocking history; source-policy/privacy. Journal12 retains exact-limit, nonregular/symlink/path, publication and legacy-record controls. This freezes admitted local bytes; original remote provenance, power-loss, actual service and signed-platform acceptance remain open. |
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
| NMR-42 / NMR-AC-33 | Direct MahayanaHost and actual approval calls without ordinary dispatch | CLI3, executable5, full lib9;8 positive/9 negative approval inputs | CLI3 passed; executable4/5 TEST_FLAW; lib9 not run |
| NMR-43 / NMR-AC-34 | Final package hashes/sizes plus complete decoded blockmap equality; receipt binds actual sidecar | Metadata18 and unchanged digest-pinned artifact13093546572 repair/immutable-byte experiment | Old15 passed, historical experiment failed compressed-byte comparison; fix authored; signed candidate acceptance open |
| NMR-44 / NMR-AC-35 | Runtime lifecycle owner; v1 cannot revive interrupted runs/duplicate terminals |3 lifecycle cases in full lib9 | Compiled at base; lib9 execution blocked by executable fixture failure |
| NMR-45 / NMR-AC-36 | Real persistent request registry with UUID-owned fixture, lastError/restoredRequests and stable srq-ID | Keep executable5; first1/duplicate0, disk and memory unchanged, reopen retains request, independent store issues1 | TEST_FLAW repaired in tests; containing-H execution required |

Current observation at exact808/run41543732585: metadata18 actually passed in Electron job125613994124, zero failed/skipped. The CLI/gateway oracles `approval_respond_routes_all_legacy_choices_to_canonical_host_decisions`, `approval_respond_unknown_choice_fails_before_host_dispatch`, the three `lifecycle_projection_` cases, `approval_event_emits_peer_request_with_same_runtime_request_id` and `repeated_approval_event_does_not_duplicate_peer_lock` remain selected in the current executable9/library13 shipping gate, job125613994273, which was queued. The preceding table preserves historical failures/counts; peer23 and metadata18 do not qualify those CLI cases or a newly signed/installed package.

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
| NMR-37 | Generic `mahayana-host/Cargo.toml` direct dependency and its two existing lock edges, defined by frozen implementation commit `adf15caadac5775fdd8582227cad3bbf9c93cf0f` and `current-owner-inventory.md` Host handoff record; no explicit NMR-37 definition exists in the main Native Messaging Spec at808. | Existing shipping step runs `cargo tree --locked -p mahayana-host --edges normal --depth 1` and rejects a direct `mahayana-telegram` edge. Current808 shipping125613994273 was queued, so execution remains unqualified. Retained direct C APIs, external callers, full dependency/package/source qualification and rollback remain open. |

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

Scope and authority: `docs/specs/fbcp-native-message-reconciliation.md` NMR-01, NMR-02, NMR-03, NMR-04, NMR-05, NMR-06, NMR-09, NMR-18, NMR-19, NMR-20 and NMR-AC-01, NMR-AC-02, NMR-AC-03, NMR-AC-04, NMR-AC-05, NMR-AC-12. The bounded responsibility is recorded in the existing TDRP ledger as `FBCP-NMR-001`, with existing owner anchors `MSG-SEND-SETTLEMENT`, `MSG-REACTION` and `NETWORK-SYNC-REALTIME`; no new message owner or parallel ledger is introduced.

The actual shipping route is `ProductionRenderer::sendComposerPrompt` / reconnect-open -> `session/gateway.rs` Human commands -> `ProductionSessionWorkers` -> issuer/account-bound `FabushiNativeMessagingClient` and `/api/social/messages` -> existing `SandAgentDb` transcript/dispatch state -> existing transcript mutation and renderer projection. The complete `MessagingService` engine is not asserted to be the native Human send/sync path. Existing forwarding and recipient authorization reuse their shared policy functions independently.

For NMR-06, all case symbols below execute the real Host/SQLite/loopback boundary in `source/host/tests/session_gateway_contract.rs`. Exact808/run41543732585/job125613993664 executed the current72 Human cases plus the separately selected Transcript3/resource3/creation14/storage3/sharebox1/journal12/Agent4/credential3/HTTP1, total116 passed with zero failed/ignored. These are scoped execution records, not source or full product acceptance:

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
- NMR-02 / NMR-04 / NMR-AC-02: the strengthened peer-nonce case first creates a real failed-dispatch pending intent, admits a peer row using the same nonce without consuming that intent, reopens the DB, then allows only the matching local acknowledgement to settle the local slot. The legacy-peer case separately preserves historical content and durable nonce provenance.
- NMR-03 / NMR-AC-03: the strengthened acknowledgement case isolates text, reply identity, resource identity, resource order and missing-resource differences, asserts the specific rejection and unchanged pending/anchor state, then reopens and proves matching-ack recovery with the expected resource bytes. Canonical-ID nonce rejection has an independent correct-nonce control.
- NMR-05 / NMR-AC-04 / NMR-09: retain real attachment/reply dispatch, ambiguous-send restart without reupload, reaction-toggle, bounded history-gap and resource-gap-before-later-row cases. Existing `source/host/tests/send_pipeline_nonce_boundary_contract.rs` contributes four Agent admission/nonce regressions; it is not complete Bot acceptance.
- NMR-REG-TRM-01 (retained Agent/Transcript regression, `TEST_FLAW`): `source/host/src/transcript_mirror/conversation_state_binary.rs#tests::conversation_compaction_epoch_counts_external_and_self_summaries`, `source/host/src/transcript_mirror/conversation_state_binary.rs#tests::recovery_fields_decode_token_details_and_turn_timing`, `source/host/src/transcript_mirror/conversation_state_binary.rs#tests::stale_token_details_can_be_removed_without_reencoding_other_fields`. Move the misplaced test-function closing brace only; keep production bytes, fixtures and assertions unchanged. The existing checkpoint decoder remains the owner. The containing-H Ubuntu reconciliation must actually run all three cases (3 passed/0 failed/0 ignored) and report both formerly nested recovery names as `ok`; actually executed at containing H `f544e9b3161d758141e1b419bcdab22cc522952e`, run `41526852400`, job `125556686433`: all 3 passed, including both formerly nested names. That job later failed the independent creation preservation test; complete-batch acceptance stays blocked. This is additional existing Bot regression evidence, not a product feature or full Bot acceptance.
- NMR-AC-05: the existing RTM, three owner anchors and the bounded canonical responsibility record the production route; implementation, authored tests, execution and independent acceptance remain distinct.

- NMR-18 / NMR-19 / NMR-20 / NMR-AC-12: all four allowed reaction tuples preserve exact strings/projection across reopen, including NFC-equivalent distinct strings. Five malformed summaries are rejected before resources and known-ID overlap while valid independent pages persist. Invalid reaction POST preserves the entire row and emits no target mutation; a later valid POST recovers after reopen and publishes exactly one durable update.

Authorship/evidence boundary: the original strengthened ten-case blob `127700079e9ae9d5c2155cd1d8a218a47f2f8075` actually passed together with all required message gates at H=`68d4f07688741e8f54b35f3746a087f7fdd12108`, run `41402071662`, attempt1. The three new reaction cases and unchanged original ten are frozen at blob `0728038de6b3fa6df656604a352283715d2cdfff`; the new production blob is `c9a486af86b252ca4c7a025452928caf66439563`. Both were independently statically reviewed and then actually executed at exact H=`95b29df688f718a35574fa36ef4ca75df437ed30`, run `41403643662`, attempt1: all thirteen Human cases and the complete preserved message gates passed. The existing Human fixture's 15-second accept/read deadline and socket write timeout remain. Full run/job/artifact/digest details stay in the existing round3 receipt.

The bounded Host gate passed at exact95/run41403643662. Every subsequent message change remains gated by NMR-AC-06/07/08/09/10/11/12: both named message jobs on one exact H, actual thirteen Human and four Agent nonce cases, locked AppHost compilation, response normalization, both bare future-salts library fixtures, full protocol/TCP including its live case, proxy rejection and credential-schema cases. A skipped step, diagnostic lock generation, test authorship or old-H result is not a pass. Use the existing round3 evidence receipt for execution records; no second result ledger.

Fixed-U mapping uses only the explicitly read peer-scoped dispatch, existing-message application and identity-transition sections recorded in `research/native-message-known-overlap-fixed-u.md`. It does not close whole upstream files or change inventory/source_reviews/unknown/unread/source acceptance. Known-message refresh, durable history-range coverage, read-state/visible receipt behavior, distinct-emoji/whole-payload bounds and stale mutation/refresh ordering remain separate open work. Full native-service, UI/package/platform, performance/soak, complete Bot regression and independent ACCEPT remain open.







### NMR-51: Native Human resource upload service boundary

NMR-51 / NMR-AC-42 extends the existing `FBCP-NMR-001` and `MEDIA-TRANSFER` responsibility; the production path remains Human composer -> `sendHumanMessage` -> `ProductionSessionWorkers` -> `FabushiNativeMessagingClient` -> existing resource/message service and Session/Transcript SQLite. Backend `bhrumom/fabushi-backend@687718ce9e0083aa124d29005bcce4051d357919:fabushi/web/src/handlers/friends.js` (blob `6179f3ce8e223f4e0bd9664885871e134f4488a8`) requires a nonempty resource at most 25 MiB. Native preflight and actual limit+1 reading enforce this service boundary while preserving shared Agent video and download budgets.

- `source/host/tests/session_gateway_contract.rs#shipping_human_oversized_upload_stays_pending_across_restart_and_recovers_same_nonce`: one oversized video is rejected before resource/message HTTP; original pending/error/nonce survives pure DB reopen and runtime restart; corrected bytes recover with the same nonce, exactly one upload/message/download and one canonical row.
- `source/host/src/extensions/session/native_messaging.rs#native_message_resource_upload_tests::exact_service_limit_is_readable_but_empty_and_read_errors_are_rejected`: exact 25 MiB succeeds; empty input and partial-read failure reject.
- `source/host/src/extensions/session/native_messaging.rs#native_message_resource_upload_tests::growth_after_metadata_observation_consumes_at_most_limit_plus_one`: actual consumed bytes stop at limit+1 and oversize rejects.

Authored at the original H76 freeze; that containing-H selection required all 48 Human gateway cases, three focused resource-bound cases and preserved required message gates. Later actual execution and current selections are recorded separately below. Prerequisite H `59f5fa24827812f78336c464995d161f403dedfb`, run `41508537631`, passed all five applicable jobs including Human38, protocol122, TCP9/live and actual AppHost package-copy4; it does not execute this extension. Evidence: `projects/telegram-desktop-rust/evidence/round3-native-resource-upload-20261102.json`. Fixed-U review is limited to the recorded media preparation/file-part sections; source completeness, multi-attachment atomicity, full UI/service/platform/package/Bot acceptance and independent ACCEPT stay open.

The current byte-buffer upload and forward snapshot readers share the literal25MiB service bound. Their digest/path/atomic snapshot/complete-target prevalidation rules remain; existing forward exact-limit/oversize tests use service25MiB literals and all twelve journal cases are retained. `native_message_resource_upload_tests::byte_upload_rejects_empty_and_oversize_before_credentials_or_http` covers the added buffer entrypoint. Existing32MiB download and200MiB Agent video boundaries are unchanged. The original pending/same-nonce oracle here is the Host gateway. H76's fresh-nonce/reply/committed-path/reopen UI gaps are historical; the later d1b7 existing-owner static review records identity-only Human retry, retained pre-frame commit progress, final-frame account/selection guards and durable failure projection. Its actual containing-H execution and complete qualification remain required; see `round3-human-ui-retry-gap-20261102.json`.


NMR51 execution followup (round 3): exact `63c59566aae7fa9b424a364df1429176c792d249`, run `41520507652`, attempt1, reconciliation job `125535006727` completed/success. Full actual logs show Human54/resource3/sharebox1/journal12/AgentNonce4/credential3/forwardHTTP1, all zero failed/ignored, including the oversized/restart/same-nonce case and each resource reader/buffer case. Artifact `13177247633`, digest `sha256:fa8b48faf174824b4b7a1fdd709955cb139b0c159571abb0c7499c9abfc450b6`. The same H also passed the actual UI58, Electron89 and native Windows11 selected cases and renderer/Electron compilation. Its macOS shipping job was still queued; these results do not qualify the complete batch, prove H76 execution, or execute the later TRM registration repair. Earlier authored-at-freeze text remains history. Four fixed-U primary resource texts and the configured TL generator/input/consumer binding were subsequently read as recorded in `projects/telegram-desktop-rust/evidence/round3-resource-source-review-20261102.json`; complete source understanding/acceptance is still false. All run/job/artifact and scope details remain in the existing `projects/telegram-desktop-rust/evidence/round3-native-resource-upload-20261102.json`; no source/release/independent acceptance promotion.

### NMR-CREATE-01b / NMR-CREATE-01c: existing owner reuse and empty-store admission

`ProductionSessionWorkers::create_human_conversation` and `human_creation_summary` preserve the winning cached Arc; `SandAgentDb::initialize_human_metadata_if_empty` admits identity only when KV, transcript and blob tables are empty in one SQLite statement. Refusal never recursively deletes or adopts retained data. Exact808/run41543732585/job125613993664 executed creation14 and storage3, including `reader_published_after_initialization_keeps_its_arc_and_durable_state`, `first_initializer_refuses_identityless_kv_transcript_or_blob_without_adoption`, `cached_owner_with_missing_identity_never_relabels_retained_data`, and the three distinct-handle/held-transaction initialization cases. Custom VFS, arbitrary hot/corrupt sidecar recovery, power-loss, broad platform and independent acceptance remain open.

### NMR-CREATE-01d / NMR-AC-CREATE-01: retained store format and lock preservation

Exact `f544e9b3161d758141e1b419bcdab22cc522952e`, run `41526852400`,
job `125556686433`, passed Transcript3/Human63/resource3, then failed creation
11/1 because NOTADB refusal removed existing sidecars. Later selected steps
were not run. This is a production preservation defect; the preceding db6
`usize:FromSql` compile failure is a separate fixture defect and executed no
selected tests. Full logs, hashes and artifacts remain in
`projects/telegram-desktop-rust/evidence/round3-human-store-preservation-20261102.json`.

Repair `d1b7ced237f108a22c1b166877a6d1c1bd62d301` retains the existing
`probe_human_conversation_store` / `SandAgentDb` / `ProductionSessionWorkers`
owner. Missing/empty main plus nonempty retained WAL/SHM/journal refuses
without SQLite writes; truly empty stores stay creatable. A default-VFS,
private-cache, immutable SQLite `schema_version` format read propagates every
error and owns its descriptors; no raw database open/read/close releases
another SQLite connection's POSIX locks. Valid retained snapshots continue
through the existing WAL-aware owner; no-sidecar stores retain `quick_check`.

- `human_creation_tests::retained_wal_creation_keeps_committed_identity_and_transcript_after_reopen`: actual committed identity/transcript exist only in retained WAL after last writer closes; main-only stale data is proved, then existing create/read/reopen preserves the WAL rows.
- `human_creation_tests::human_format_probe_preserves_posix_locks_against_an_external_writer`: independent process UPDATE is busy before and after the real probe, then succeeds after parent rollback. The exact child case/marker proves execution without inflating the outer count.
- Preserve all original 12 cases, ten corruption variants, three original-file diagnostics and empty-directory/zero-file/schema-only positives; add missing/empty main with sidecars to the existing refusal matrix. The containing Ubuntu gate requires creation14 plus unchanged storage3 and both new names; all Human67/TRM3/resource/UI/Agent/shipping gates remain.

Run `41530841386` attempt1 binds this repair. At initial API observations it
was queued with no runner or executed steps; authored tests and independent
static reviews are not passes. All applicable jobs must terminate successfully
with actual behavior/protocol/TCP evidence before dependent qualification.
Custom VFS, metadata/open races, arbitrary hot/corrupt sidecar recovery,
complete source/legal/service/package/Bot and independent ACCEPT stay open.

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
| NMR-PAGING-01a / NMR-PAGING-01b | Same-time and late inserts, settlement and timestamp changes cannot skip/repeat a saved walk; corrupted/expired/scoped indices fail; restart/retry is exact | `shipping_human_tail_keeps_physical_anchors_stable_during_backfill_and_equal_time_inserts`; `shipping_human_snapshot_cursor_survives_pending_settlement_and_authoritative_anchor_refresh` |
| NMR-PAGING-01c | Opaque token survives the existing cache/reopen chain; failed pagination keeps committed updates; Human nonce correlation respects author; Agent shape/merge unchanged | Existing `transcript-projection.contract.test.ts` plus `recovered-conversation-infra.contract.test.ts` and shared RPC behavior cases |
| NMR-ACTIVITY-01 | History and Agent handoff serialize at the existing lane; older timestamps cannot regress activity; failed metadata commits report failure and replay repairs the durable row/metadata gap | `shipping_human_agent_handoff_and_history_download_keep_activity_monotonic`; `shipping_human_activity_write_failure_keeps_committed_rows_and_repairs_after_restart` |

For NMR-AC-PAGING-01, the existing exact-H Actions job retains every current message, credential, reply/forward, upload, AppHost/shipping, UI/Chromium/Electron and Windows gate. At the integrated freeze the authored selected counts are Human52, projection20 and cursor/RPC17; execution remains pending until the containing-H logs show the actual results. Two concurrent NMR49 cases retain their complete physical-row/atomic/permission assertions and separately assert the new public canonical ordering. Existing prerequisite results are provenance only; see `projects/telegram-desktop-rust/evidence/task20bf-round6-resumed-stable-pagination.json`. Full source/legal, deployed Worker, supported signed packages, UI/a11y/temporal/Bot/soak and independent ACCEPT remain open.

Current808 Host72 actually executed both SQLite walk cases above and the activity/write-repair cases. UI qualification is separate: current `transcript-projection` tests preserve opaque tokens through root-cache reopen, failed-read overlays and account ABA; `recovered-conversation-infra` and shared Coordinator request/reply parsers retain exact cursor scope. At this observation the current UI job125613994076 remained in progress. No older-remote-mutation feed, active Worker compatibility or complete pagination acceptance is inferred.

### Round7 resumed NMR-22 / NMR-24 / NMR-26 boundary evidence

Current additions retain the existing FBCP-NMR-001 row and production route. NMR-24 uses the same conversation lane in `sync_human_conversation_history`, `toggle_human_message_reaction` and `replay_human_reaction_intent`; independent conversations remain independent. Exact808 Human72 executed `shipping_human_concurrent_reactions_serialize_one_conversation_without_blocking_another`, `shipping_human_reaction_waits_for_inflight_history_before_deriving_desired_state`, and durable-intent failure/reopen/already-applied replay cases. Confirmation precedes row persistence and intent clearing; deployed history/older mutation discovery remain unqualified.
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
| FMEP-SOURCE-01 / existing desktop package builders | Existing Coordinator/box Cargo.lock and worker go.mod/go.sum are pinned; desktop, CI and macOS package commands retain locked/readonly guards | Capture565a/run41514329200/artifact13175461007 remains provenance. Pin e13c34a5/run41519264825 verified tracked inputs and actual Coordinator/box Mach-O arm64 binaries. Original prebuilt generation was recovered from run35281869572 to tailscale6608b9a387513e69ed260e323376a17aa4029084; effective post-install locks, recursive dependency/legal disposition and full source ACCEPT remain open. |
| NMR-AC-30 / same Coordinator, Box and network-worker owners | Existing staged-runtime source/binary gates retain actual executables, normal/build dependency and symbol/library refusals; Coordinator1+1+4, Box13+Host3, Go18 | Exact e13c34a5 passed Coordinator6 and Box harness13+Host3, but the Box disconnect oracle was invalid (illegal PID probe). Corrected d2771e953a5b3978b3624518e7f07eaf692ee411/run41526069466/job125554615301 actually passed all13 on Ubuntu, including live leader/child/group preconditions and pre-cleanup ESRCH. Corrected macOS Box/Host execution and actual packaged worker remain pending; no signed-package or independent ACCEPT. |
| FDN-WORKER-LISTEN-01 / FDN-WORKER-LISTEN-AC-01 | Existing Go streamListen -> acceptPeer -> streams: cancellation closes the listener and every unadmitted connection; matching successful admission transfers ownership | Exact e711505dbc44bde087595c10a060f5ae6deb5847/run41523198228: Linux18, macOS18, Windows18 and Host supervisor3 actually passed, zero failed/ignored/skipped. Actual macOS packaged-worker source check, live network and independent acceptance remain required. |
| FDN-HOST-FRAME-01 / FDN-HOST-FRAME-AC-01 | Existing RemoteComputerTarget service -> private-frame reader -> worker stream_read: one total deadline, shrinking read budgets and rejection after expiry; existing stream-close path retained | Exact0291685eb5f7a5a7b70a11fba8a70a45793b2ae9/run41524923463/job125550076691: six controlled-clock production-reader behaviors and existing enrollment9/IPC3/target10/wiring5 all ran and passed. Full real-network/shutdown and unknown-open-result process retirement are not qualified. |
| NMR-39 compatibility retention / existing legacy C exports | `providers/telegram-runtime/tests/runtime_ffi.rs`: five direct create/persistent-create/execute/close/free cases, persistent close/reopen, owned response frees, null/UTF-8/key-length/closed-handle refusal | Fifteen non-live tests plus the original one explicitly ignored live-DC test are required by `retained-direct-ffi-compatibility`; authored, not run at preparation. Invalid-length buffers are allocated; all stores/keys are synthetic. This does not qualify a final shared/static library or installed external consumer. |

Concurrent commit `e13c34a5bfc9eddfc5306102e7ad68d4e864bd4e` already
integrated the identical four inputs, desktop locked builders, capture=false and
release readonly build. This batch preserves that delivery, adds the remaining
CI/release input guards and the retained C-wrapper target; it does not replay
or claim authorship of the concurrent pin commit.

The authoritative receipt remains
`projects/telegram-desktop-rust/evidence/round16-apphost-package-boundary.json`.
Later exact-H executions, original-generation bindings and the Box TEST_FLAW
correction are recorded in the existing
`projects/telegram-desktop-rust/evidence/task20bf-round2-auxiliary-input-pin.json`;
source/ledger acceptance and independent ACCEPT are unchanged.
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
| NMR-SOCIAL-OWNER-02; NMR-SOCIAL-AC-01 | FeatureHost real execute: exact early refusal before absent runtime, poisoned lock, file or event side effects; explicit Test/non-desktop/unrelated controls | Existing desktop_messaging_boundary_tests: 10 passed/0 failed/0 ignored at exact f2903f53ded5b07faea73e7f07d780eda0423283, run41527613857/job125559290571 |
| NMR-SOCIAL-OWNER-03; NMR-SOCIAL-AC-02 | Runtime real SQLite recovery preserves Social records and continues ordinary Agent work | retained_social_recovery_tests: 9 passed/0 failed/0 ignored in the same exact-f290 Ubuntu job; source-comment ID correction did not change its executable body |
| NMR-SOCIAL-OWNER-04; NMR-SOCIAL-AC-02 | Existing session-owned platform client validates actual URL/origin/path before credentials and disallows desktop redirects | session_owned_platform_: 6 passed/0 failed/0 ignored in the same exact-f290 job, including expanded UUID resource, alias, origin and redirect assertions; production implementation unchanged |

See `docs/specs/fbcp-desktop-social-owner-boundary.md` and
`projects/telegram-desktop-rust/evidence/round23-owner-boundary-recovery.json`.
H0 Windows hook timeout and later Windows11 success retain their actual run
identities. Formal publisher, real package install/upgrade/rollback, authenticated
production pagination, source/legal/full Bot and independent ACCEPT stay open.

The existing workflow now also runs these same Feature10/Runtime9/HTTP6 behavioral
harnesses on Ubuntu while retaining the queued macOS shipping requirement. This
focused job is not a signed-package, live-service or independent acceptance gate.

## Recovery round 3 — retired protocol ownership correction (2026-11-02)

`TDRP-R9-MTPROTO-BAD-MESSAGE-RECOVERY-001` and
`TDRP-R9-MTPROTO-SINGLE-RPC-MESSAGE-CONTAINER-001` are reopened as
`understood / pending / blocked`. The actual desktop Native Session does not
use their former Telegram lease/runtime chain. Current owner, lifecycle,
service, composition, capability and UI mappings remain unresolved; original
values and implementation evidence are retained as historical compatibility.
The unique ledger records two known unmapped responsibilities. Source identities,
six scoped source reviews, global null unknown/unread and acceptance are preserved.
Exact94/run41513767498 protocol122/TCP9 successes qualify compatibility only.
The corrected containing H must pass existing source/schema/candidate Actions;
later Native shipping and every selected product gate remain independently required.
Evidence: `projects/telegram-desktop-rust/evidence/round3-native-owner-recovery-20261102.json`.

### Round29 SQLite CAS qualification — NMR-47e / NMR-AC-38

The existing `FBCP-NMR-001` / Session forward journal remains the owner. Commit `beafc6fcb4ce59ccdd0e5130c3892172cb9dc20a` strengthens only `source/host/src/extensions/session/forward_batch_journal.rs#tests::concurrent_forward_metadata_writers_have_one_durable_winner_and_recover`: prepare independent owners and valid batches before the barrier; require zero busy callbacks because busy/locked returns `Ok(false)`; independently read raw SQLite metadata; reject the stale loser without changing the row; fresh-merge and reopen with both immutable intents and unrelated metadata intact. This proves concurrent API-call behavior, not every internal scheduler interleaving or multi-AppHost cross-source uniqueness.

Beaf run41522952938/job125543328873 failed inherited reaction-fixture `E0382` before the Human/journal cases executed. The containing one-line fixture repair `106bb95ba97ef7722e9523285783824393039bc0`, run41523018920/job125543545349, actually passed Human62/resource3/sharebox1/journal12/Agent4/credential3/HTTP1, including all five original process/HTTP/SQLite forward cases. Artifact13178302134 digest `sha256:02acce01cf84e609998936473557833958237099b6b1b395402966e2cbe377d6` binds the full execution. The later production observed-write helper at `d8789216b6d3278d64cfec69ecafd56e130747f5` also passed these groups in run41524395411/job125548279795; the enhanced concurrent test remains byte-for-byte identical across journal blobs0eb499f and0a5d192. Its separate stale-helper case deterministically rejects old production-write observations.

The canonical evidence append is `558b57bf9730446b08cf40c120d17c988c56ed16`; all other ledger rows, source reviews, execution scalar fields and acceptance remain unchanged. Exact source contract job125557439692 passed158 Node and37 Python schema cases with zero ledger shape errors; candidate125557839138 also passed consistency for16381 source identities/184 responsibilities/six reviews (two complete), retained unknown/unread=null and correctly rejected accepted mode. Full identities and bounded failure/recovery records are in `projects/telegram-desktop-rust/evidence/round29-forward-journal-concurrency.json`.

Later shared SQLite initialization, Session permissions/history and actual renderer changes require their own exactf544/run41526852400 results (Human63, creation12+3, retained Transcript3, journal12, actual ProductionRenderer11+8 and Social). Historical59/106/d878 results do not qualify those changed inputs. Current shipping, real services, complete source/legal closure, signed platform install/upgrade/rollback, full Bot regression and independent ACCEPT remain open. Backend PR137 stays read-only.

## Human message retry — NMR-HUMAN-RETRY-01

| Requirement | Actual owner/call path | Authored evidence / remaining gate |
| --- | --- | --- |
| HRR-01 | Shared parser + explicit Session/gateway allowlist; exactly two identities | Shared2 new + actual port4 + existing authenticated HTTP gate; containing-H Actions required |
| HRR-02/03 | Session unique existing row, direct-pair identity, original nonce/reply/resources, Native settlement | Real attachment/disconnect/reopen/concurrent-click test and64 admission variants; Human67 required |
| HRR-04 | Existing source batch authorization → only original destination | Actual process-kill/protected-source/provenance-loss/repair test; no generic resend bypass |
| HRR-05/06 | Existing Retry button/controller/projection/account/selection/acknowledgement owners | UI28 + port4 + pre-Host8; ABA, reconnect-ready frame, deferred updater and event-obsolete cases; actual renderer build |
| HRR-AC-01–04 | Same source-bound batch; existing Agent/credential/journal/shipping gates retained | `projects/telegram-desktop-rust/evidence/task20bf-round1-human-message-retry.json`; authored only until exact containing-H run, packaged visual/a11y and independent ACCEPT remain open |

The message POST releases the conversation mutation lane; existing resource staging and automatic source-batch recovery serialization are not claimed as new nonblocking behavior. No source inventory, source-review completion or release-acceptance counter changes.

HRR-07 integration: failed pre-frame Human submissions resume the original existing Composer queue input with per-attachment commit progress; possibly posted frames switch to identity-only retry. No projected payload, fresh nonce or current-draft mutation is used. Actual authored gates at the current source freeze: Human67; Transcript34; retry helper28 + port4 + pre-Host queue8 =40; shared cursor/RPC19; retained actual Renderer Chromium11. These totals retain concurrent work and remain unexecuted at preparation. Full containing-H evidence is recorded in the same task receipt.

## Native authenticated response completion

Requirement: `NMR-NATIVE-RESPONSE-01` / `AC-NATIVE-RESPONSE-01`.
Existing ProductClient keeps the actual NativeCredential from dispatch through bounded
response reading, then compares the existing full same_binding authority before returning
success or error. Platform, JSON GET/POST and multipart publication share this boundary;
anonymous/legacy behavior and Session/Transcript ownership remain unchanged. There is no
network retry or reversal of a server operation already dispatched.

Five real parent tests with 69 subprocess/HTTP scenarios are authored in
`native_desktop_session.rs` (`native_response_completion_` filter). They exercise account,
issuer, same-token session/device/expiry and secret binding changes, logout/invalid files,
anonymous/current positives, all platform methods, multipart and body/error barriers.
The exact containing-H Actions gate requires5/5 plus69 successful scenario markers,
HTTP owner6, native handoff7 and actual AppHost compilation; authoring is not execution.
The existing Ubuntu lane also runs the same five completion tests and69 markers,
with its own exact-H log; the macOS shipping copy and actual AppHost compilation
remain independently required. See `projects/telegram-desktop-rust/evidence/round23-qualification-wiring.json`.
Receipt: `projects/telegram-desktop-rust/evidence/round25-native-response-binding.json`.
All independent/source/service/publisher/signed-package/full product ACCEPT gates remain open.

## Recovery round15 — retained forwarding source discovery

Bindings: NMR-DATA-ADMISSION-01 / NMR-AC-DATA-ADMISSION-01 in the native-message
Spec; existing responsibility FBCP-NMR-001. Receipt:
`projects/telegram-desktop-rust/evidence/round15-desktop-messaging-boundary.json`.

| Requirement | Existing production owner | Authored oracle and required execution |
| --- | --- | --- |
| NMR-DATA-ADMISSION-01 | Session forward/sync → ProductionSessionWorkers → source SandAgentDb journal | Enumerate retained sources before dispatch; unavailable source cannot free requestId or stage another source. Own revoked-source denial and other-account isolation remain. |
| FBCP-HUMAN-CREATE-04 follow-up | Existing Human creation/cold probe before SQLite open | Preserve the current peer-owned immutable negative check, PRIVATE_CACHE and propagation of every probe error. Missing/zero main with any known sidecar refuses before open; zero/no-sidecar remains valid. Existing creation14/storage3 and all corruption/retry/restart oracles remain; no independent main-file descriptor, FFI or readonly_shm. |
| NMR-AC-DATA-ADMISSION-01 | Existing native-service-contracts workflow | Human68 = current67 + one strict process/HTTP/SQLite recovery case. Preserve creation14/storage3, resource3/sharebox1/journal12/Agent4/private3/HTTP1 and every current UI, retry, FeatureHost, Host, ABI, protocol/TCP and shipping gate. Containing-H execution is required. |

The broader unpublished seven-case admission prototype is withdrawn, not executed
or accepted. Pinned-source review qualifies reuse of the existing probe only as
a negative main-format check, not WAL integrity, identity or arbitrary crash
preservation. Existing concurrent CREATE01..04 and Human retry implementation
are preserved, not re-attributed. Source completeness and independent ACCEPT
are unchanged; historical counts remain scoped to their immutable H and attempt.

## NMR-50 forwarding recovery and admission

The existing `FBCP-NMR-001` and `TDRP-R9-SHARE-FORWARD-IDEMPOTENCY-001` rows retain Session/Transcript, source SandAgentDb and Native HTTP ownership. This bounded batch changes no journal schema, Backend PR137, source-review status or acceptance value.

| Requirement | Production boundary | Authored execution oracle |
| --- | --- | --- |
| NMR-50a | Automatic recovery holds one successful batch `try_lock` guard through existing delivery; explicit forward/retry remains serialized | Held POST response; both unrelated and destination history finish first; journal unchanged and POST1; canonical settlement and reopen |
| NMR-50b | Direct history entry recovers once; top-level roster recovers once and calls the shared history-only body for each friend | Three-friend roster retries failed target once per invocation; original payload/nonce survives shutdown/reopen |
| NMR-50c | Host-bound friends request checks active user before GET and the full credential snapshot after the complete body | Wrong-user zero HTTP; six streaming credential rotations preserve roster/metadata/transcript and issue no downstream HTTP; authorized retry and reopen |
| NMR-50d | Dialog lifetime admission passes through ProductionRenderer to the existing Coordinator request-frame boundary | Actual client/controlled port in the existing Chromium picker fixture; six pre-ready retirement paths post zero frames; live positive sends exactly one; after-post lifetime retirement preserves Host intent |

The four added Host functions require Human72, retaining every prior Human/Agent/journal gate. The existing16 Chromium picker functions include the new real-port scenarios. These are authored and statically reviewed at preparation; only the containing exact-H Actions result recorded in `projects/telegram-desktop-rust/evidence/round27-forward-pending-ownership.json` can qualify execution. Complete source/legal, real service, supported signed packages, full Bot/UI/accessibility/performance/soak/upgrade/rollback, independent ACCEPT and formal release remain blocked.

## Round30 — named Human forwarding feedback

| Requirement | Existing production owner and wiring | Authored evidence and remaining gate |
| --- | --- | --- |
| NMR-FORWARD-FEEDBACK-01 | ForwardMessageDialog lifetime and immutable request snapshot -> validated Host settlement -> ProductionRenderer.settleForward -> settleForwardProjection -> ConversationComposer notice | Names are presentation-only, frozen for retries and scoped to each completed target; service request and durable journal remain unchanged. |
| UI-NMR-NAMED-FORWARD-001 | Actual root More/Forward/recipient selection/Coordinator RPC/settlement notice | Successful named notice survives same-account roster failure and is cleared by account switch; new containing-H Chromium execution required. |
| UI-NMR-RETIRED-FORWARD-NAMES-001 | Existing account generation and dialog lifetime | Late old-account reply after A-B-A cannot disclose names or close a current dialog. |
| NMR-AC-FORWARD-FEEDBACK-01 | Existing native-service-contracts UI job; current Host and other gates retained | settlement9/dialog17/renderer13 authored; zero skipped/flaky/retry and actual renderer compilation required. |

The unique ledger owner is `TDRP-R9-SHARE-FORWARD-PRIVACY-001`; both exact
phrase-factory source reviews remain pending for Saved/self/premium/navigation
and complete responsibility closure. No new message authority or data migration.
The earlier pointer-bridge production fix is `7d57aa4393c4f6c13bc6129255e2bec8173ed5ed`,
run41531186027; jobs were queued with no steps at preparation, not passed.
Bounded changes, exact inputs, causal old failures and subsequent execution:
`projects/telegram-desktop-rust/evidence/round30-forward-recovery-qualification.json`.
Real services, full source/legal/platform/Bot/quality and independent ACCEPT/release
remain open; Backend PR137 is read-only.


## Round8 NMR-UI-01 / NMR-DRAFT-01 / NMR-ELECTRON-01

| Requirement | Existing production owner and wiring | Required batch evidence |
| --- | --- | --- |
| NMR-UI-01 / NMR-AC-UI-01 | ProductionRenderer, current Transcript projection/cache, original Human retry and Agent queue/approval owners | Preserve current opaque-page, attachment, reaction pointer-corridor and named-forwarding work. Exact808 executed pure42/draft14/persistence5/reply5/cursor19/retry40/Forward9+17/history13/Transcript8/author21, total193 with compiled Renderer. The subsequent background-navigation repair requires author22 and all retained applicable gates at its own frozen H. |
| NMR-UI-01a | Existing `projectTranscriptEntry`, author-aware nonce correlation, acknowledgement and queue/cancel/retry paths | Projection contract same-author positive and peer/assistant/unscoped Human negatives; actual author/recovery case retaining colliding peer nonce without consuming a local send. Preserve Agent attachment and tool-turn behavior. |
| NMR-UI-01b | Existing account/conversation/lifetime generations, per-read/receipt overlays and original draft identity settlement | Actual delayed send/reaction receipts, semantic refusal, inactive conversation, account ABA, clear/disposal and later same-text draft cases; draft14/reply5/persistence5 qualify the bounded recovery owner. No Host cancellation is inferred. |
| NMR-UI-01c | Canonical Human empty text/resources/rich text/reactions through current cache, page and mutation reconciliation | Projection contract empty-value and attachment-set cases plus actual light/dark paged edits and old-row reaction/empty-message cases; preserve opaque continuation, failed-page retry overlays and viewport anchoring. |
| NMR-DRAFT-01 | Existing V1 composer-drafts account owner -> awaited identity binding -> existing send/retry RPC -> local-author canonical observer -> ID-specific persisted retirement | Draft14, scoped reply5 and existing Sand persistence5, including binding failure zero RPC, malformed binding/legacy preservation, field merge, canonical-before-restore, same-text new ID, ABA/disposal and I/O refusal/order. Full Composer reply/rich-text/fork recovery must be consumed at actual send, not just stored. |
| NMR-ELECTRON-01 | Actual Electron/main/preload/utility/Rust Coordinator/AppHost Session/Native HTTP/SQLite | Existing two process cases: complete canonical row survives offline same-directory reopen; held-receipt outbox replay retires original A before any new edit and preserves same-text B/new ID through the third lifetime. Retain original Plugins2 and Agent Network case. |

The current input is checkpoint `6e2f1c3728ac29cae3b460e2a4ec7deb7719d01b`;
root integrates only the reviewed delta and preserves concurrent contracts.
The original requested H5d534898/run41413368942 passed Human20/Agent4 and
shipping at that H. Review H375936fb/run41531580905 compiled and passed
pure41/cursor17/Forward5+16, but historical browser11 was 10 passed/1 failed
(pointer path); later browser groups were skipped. Concurrent 7d57aa fixes
the real hover corridor and current13 cases. Electron H608f0d49/run41530756868
failed Renderer TS2698 before any Plugins/Human case; the broad type guard is
corrected in this batch. No failed or skipped execution is promoted to success.
Full containing-H evidence belongs to the existing round8 renderer/Electron
receipts; preparation and blob transport are not product verification.
Source/legal, active afterId=0 deployment, signed platforms, full Bot/UI/a11y,
security/performance/soak/upgrade/rollback and independent ACCEPT remain open.

Round8 follow-up after exact808: `openAgent` keeps the existing selection/Transcript owners and distinguishes background roster/restore/clear loads from explicit navigation. Background work cannot dismiss Plugins, settings, workspace route or command palette, or reselect a superseded conversation; the synchronous selection store is authoritative. `NMR-UI-01 background transcript recovery preserves Plugins until explicit navigation` holds the initial Agent roster and later clear reload, exercises real Plugins Close twice, and reselects the pending Agent through the normal keyboard shortcut. The 22nd browser case and the subsequent exact-H execution are pending at preparation. Exact808 Mac run41543732546 actually failed Plugins0/2 after the dialog vanished, and Human0/2 at trace startup before any message behavior; the frozen follow-up retains those failures and adds the locked Playwright video dependency plus fail-closed capture initialization. The complete migration remains unaccepted.

### NMR-FEED-01 — Existing Human durable update projection

| Requirement | Existing production owner and invariant | Behavioral oracle and retained gaps |
|---|---|---|
| NMR-FEED-01a | `TranscriptExtension` shared constructor installs one Human `entries-upserted` observer; existing `ProductionRosterEmit::publish_transcript_updated` sends the existing `transcript/updated` contract to Gateway, Coordinator, Electron and Renderer. | NMR-AC-FEED-01: `shipping_human_committed_refresh_reaches_production_transcript_bus` uses real native HTTP/SQLite refresh and the started extension/Host bus. Existing Renderer older-page mutation consumers remain required. |
| NMR-FEED-01b | Internal `originDbPath` is produced from a trusted writer path, compared with this Session's trusted Human owner and never serialized to the public feed; only current durable IDs/rows may be projected. | `shipping_human_transcript_feed_reads_current_owner_and_deduplicates_ids` and `shipping_human_transcript_feed_rejects_foreign_roots_agents_and_unauthorized_origins`: forged bodies, duplicate IDs, same-ID foreign roots, Agent owners, nonparticipants and bad origins cannot leak notification content. |
| NMR-FEED-01c | Session performs indexed per-ID reads without a recursive Human mutation lock, full-history replay, HTTP, creation or writes; commit-before-publication remains authoritative. | The existing complete6400/32-page/reopen test keeps its original budget, deadline, ordering and durable checkpoints while exercising the same production projection; no extra logical message owner. |
| NMR-FEED-01d | The extension's admission fence covers read/publication; Drop closes it and explicitly unsubscribes before manager disposal. | `shipping_human_transcript_feed_disposes_copied_callbacks_before_replacement` holds a real registry dispatch snapshot, retires the old observer, then proves exactly one replacement publication. |
| NMR-AC-FEED-01 | Four additional `shipping_human_` cases join the original72 (authored total76); containing-H Actions must actually execute them with the retained consumers. | Static audited chain44792fe02191a56c7a729d67436423ed252ae081 /1698bf8c3273ea092dfbc1133da7769af6f1c2b0 is provenance only. Remote revisions/tombstones, arbitrary-old changes, physical Human deletion/clear, reaction409 recovery, deployed afterId=0 and full migration/independent ACCEPT remain open. |


### NMR-TRANSPORT-RECOVERY-01 — Existing runtime availability and roster Retry

| Requirement | Current owner and production connection | Required bounded oracle |
|---|---|---|
| NMR-TRANSPORT-RECOVERY-01a / NMR-TRANSPORT-RECOVERY-01b | Rust Coordinator runtime lifecycle -> existing Electron carrier -> production Coordinator client -> existing transport subscriptions. Classify before generation/sequence; Gateway SSE establishes availability; terminal Host and old-port events cannot resurrect an obsolete episode. | Six additional actual client/controller port cases, including same-generation down/up, missed down/new generation, informational/malformed watermark poisoning, discovery failure and unchanged pending RPC settlement. |
| NMR-TRANSPORT-RECOVERY-01c | Existing connection controller keeps one account-scoped Retry obligation, retires old request identity and starts its replacement only after listener fanout. ProductionRenderer advances synchronous transport generation/state first. | Held old success/failure/finally cannot clear replacement Retrying or suppress current listAgents -> Human sync -> active Transcript. Account ABA/dispose do not replay across owners. |
| NMR-TRANSPORT-RECOVERY-01d | Current transport is replayed by the same client to new subscribers; controller and Renderer treat initial ready only as an unobserved-state fallback. Existing Transcript, selection and draft authorities remain. | Actual browser case `NMR-TRANSPORT-RECOVERY-01 runtime Host and SSE recovery preserve the active Human draft after a held Retry`: real runtime envelopes, visible Retry, retained paged cache/selection/draft and service-only new rows; no reload/focus/chat-switch recovery trigger. |
| NMR-TRANSPORT-RECOVERY-01e | Existing actual Electron fixture isolates AppHost with FABUSHI_APP_DATA and retains explicit unavailable404 auxiliary behavior. No production success is fabricated. | Original Plugins2/Human2, strict unknown method/path/query oracle, no added retry/timeouts/reload. Port1337 conflict and actual process recovery require their own execution evidence. |
| NMR-AC-TRANSPORT-RECOVERY-01 | Frozen contract at f5499f671c5be9aa824416c0832f855d3647d786; same canonical FBCP-NMR-001 ledger row. | Authored retry46 and author23 preserve all previous cases; exact containing-H Actions only. Prior H1bc reconciliation120/120 and H98 UI194/194 remain attached to those exact inputs; neither is new runtime recovery evidence. |

No source/legal coverage or release/independent acceptance changes. Active
afterId=0 deployment, remote revisions/tombstones/old-message discovery,
reaction409 recovery, signed platforms and full Bot/UI/a11y/security/performance/
soak/upgrade/rollback remain open. The existing round8 renderer/Electron receipts
retain old failures, fixture causes, full-log endpoints, actual counts and
frozen artifact identities.


### NMR-OFFLINE-ORIGIN-01 — Explicit Human queue origin

| Requirement | Existing production owner and invariant | Bounded behavioral oracle |
|---|---|---|
| NMR-OFFLINE-ORIGIN-01a | Existing TranscriptSubmissionScope records observed transport queuing before possible Human admission; original nonce/time/input remain in the current queue. Online busy queue and held receipts do not prove offline. | `Human Composer offline origin follows observed transport queuing and survives retries without marking online busy sends` uses actual queue/observer/Coordinator frames; original8 plus1 =9, combined retry47. Agent dispatch and admission semantics remain. |
| NMR-OFFLINE-ORIGIN-01b | Existing sendHumanMessage accepts optional strict boolean queuedOffline; true requires finite composedAtMs. Existing identity-only retry cannot carry or replace origin. | Gateway rejects malformed/null marker and true without finite time before staging/HTTP; ordinary legacy arguments remain compatible. |
| NMR-OFFLINE-ORIGIN-01c | Existing Session/SQLite first intent fixes origin; same-author/nonce canonical remap preserves explicit evidence and original composed time while server timestamp stays authoritative. | `shipping_human_offline_origin_is_explicit_and_survives_settlement_and_restart` adds1 Human case (77/total121), including reopen, refresh, immutable nonce and damaged-origin retention. Ordinary bounded GET remains allowed; no added global history preflight. |
| NMR-OFFLINE-ORIGIN-01d | Current projection carries explicit origin; actual Transcript calls shouldShowSentWhileOfflineNotice. Human canonical sent/true/time are required; old timestamp/side-map/cache values cannot infer origin. | `NMR-OFFLINE-ORIGIN-01 Human offline labels require an explicit origin and canonical sent state` adds1 (43). Existing browser13 offline case adds actual queued/pending/sent and canonical-refresh/reopen badge controls. Agent rules remain. |
| NMR-OFFLINE-ORIGIN-01e | Actual Electron reads one existing prose body and independently checks canonical/delivery/time/no-placeholder and online zero-badge invariants. | Original Human2/Plugins2 counts, real process/draft checks and retries0 remain. The H98 failure receipt1b9bd861f08915691b70abe851b31af83b468fef distinguishes oracle scope from the real presentation defect. |
| NMR-AC-OFFLINE-ORIGIN-01 | Contract frozen at36ae38b3791c7c07bf6e4e4907a96265881d0c2f; no new owner or external service field. | Complete UI203 and reconciliation121 are authored targets pending exact containing-H Actions; Mac2+2/shipping remain required. |

The older browser13 harness retains its existing unlisted-RPC null fallback;
this origin extension validates only its explicit Human-send boundary. The
author23 harness and actual Mac gate retain their own strict unknown-call
oracles. This bounded evidence is not a whole-renderer RPC contract acceptance.
All source/service/package/Bot/quality/independent acceptance gaps remain.


### NMR-PERSIST-MIGRATION-01 — Existing one-shot marker recovery

| Requirement | Existing owner and production wiring | Required evidence |
|---|---|---|
| NMR-PERSIST-MIGRATION-01a | Existing Electron main SandClientPersistenceStore -> trusted secrets IPC -> preload migration API. Only ENOENT is absent; other stat errors and null/nonregular observations reject before any import. Regular historical markers retain their existing completed semantics. | `NMR-DRAFT-01 one-shot migration rejects marker I/O errors without resurrecting retired drafts`: real owner, same/reopened instance, retired key stays absent, original error and file bytes retained, zero mutating calls, error recovery and ENOENT compatibility. |
| NMR-PERSIST-MIGRATION-01b / NMR-PERSIST-MIGRATION-01c | Existing owner writes its unchanged marker through same-directory .tmp then rename, after eligible blob processing. Existing keys win, namespace/caps and supplied originals remain, no new startup invocation. | `NMR-DRAFT-01 one-shot migration commits its marker atomically and recovers after write or rename failure`: partial write and rename faults, same/reopened owner, no false completion, newer B survives stale A retry, missing-entry continuation, cleanup and repeated no-write completion. |
| NMR-AC-PERSIST-MIGRATION-01 | Spec frozen in4b69d61c93332500197bab14daaf653027468c3d; same canonical FBCP-NMR-001 responsibility and actual persistence owner. | Preserve original5 tests and add2 (persistence7, full UI205). Execute containing-H Actions. Unknown old marker repair, automatic browser startup migration and physical power-loss remain unqualified; all previous message, actual Electron and final ACCEPT gates remain. |


### NMR-RETAINED-TELEGRAM-01 — Desktop historical RuntimeStore preservation

| Requirement | Existing production owner | Required behavior and evidence |
|---|---|---|
| NMR-RETAINED-TELEGRAM-01a / NMR-RETAINED-TELEGRAM-01b | Existing Session/Transcript Host constructor selects the new in-memory Runtime retention policy; both existing recovery loops use the same earliest predicate. | Preserve nonempty original Social and telegram:chat: rows, questions, nonce/input, generation and handoff payloads before mutations or provider calls. Old None and Social-only policies remain. `retained_telegram_turns_and_handoffs_survive_desktop_sqlite_restart_without_dispatch` and `legacy_telegram_recovery_keeps_default_and_social_only_policies` use real SQLite, registered recorders, repeated reopen and ordinary Agent controls; Runtime9→11. |
| NMR-RETAINED-TELEGRAM-01c | Existing FeatureHost Production+Electron execute boundary. | Explicit trimmed Telegram open/send, including empty suffix and plugin-target variants, reject before auth, locks, plugin/context work or events. Existing wire error remains. Expand the three existing boundary cases; retain all Social/Test/non-Electron/near-prefix, poisoned-state, file and event controls; Feature10 remains10. |
| NMR-RETAINED-TELEGRAM-01d | Existing Production AppHost/RuntimeStore and native account-handoff child-process harness. | `native_telegram_owner_preserves_sqlite_turns_and_refuses_bypass` retains exact durable tables/payloads through separate process restarts and account/logout transitions. Independently prove actual provider absence, unchanged credentials, no new operations/events or transport, and ordinary Agent behavior. All previous encrypted Telegram/original-reader controls remain; AppHost7→8. |
| NMR-AC-RETAINED-TELEGRAM-01 | Contract frozen at1de845141d5e30ae9ff4965caf2915f68df2f650, same FBCP-NMR-001 responsibility and existing owners. | Both existing Linux Social-owner and macOS shipping jobs require10+11+6+8+5=40 selected top-level cases and exact three new names. Child/parameter variants are not added to that count. All message121, UI205, actual Mac2+2 and shipping requirements remain. Containing-H Actions and independent ACCEPT remain separate. |

This addresses a conditional current-source startup/refusal defect. Actual user
store presence, old-data import/mapping, active service, signed installed packages,
full source/legal/Bot/quality/upgrade/rollback and independent ACCEPT are unproved.
No retained data, provider reader or credential is removed or reinterpreted.

### NMR-REACTION-RECOVERY-01 — Qualified reaction refusal and bounded independent recovery

| Requirement | Existing production owner | Required behavior and evidence |
|---|---|---|
| NMR-REACTION-RECOVERY-01a | Native reaction response classification and Session's original intent/target admission. | Only stable actual404/409 with explicit failure and no contradictory success payload may defer its unchanged public error. Recheck exact original intent, account, owner, participants and canonical target after HTTP. Other local, credential, protocol and ACK failures remain hard; no tombstone or acknowledgement is inferred. |
| NMR-REACTION-RECOVERY-01b / NMR-REACTION-RECOVERY-01d | Existing Session history/roster, SandAgentDb and post-commit Transcript feed; existing Renderer error/local-read fallback. | Retain exact intent, continue bounded history and later friends/Calls, then return first refusal. Carry strict continuation into later friends without their own intent; pending and row validation errors stop immediately. Real restart/HTTP progress and roster/call cases preserve original error, precise requests and durable coverage. |
| NMR-REACTION-RECOVERY-01c | Original full Native credential fingerprint, existing root/store/participant checks and CallSession owner. | Capture before forward recovery and never rebind. Check before downstream requests/create/write, after responses and before coverage. Native Calls reads check headers/body; Session validates original call scope and event identity before applying. Held body and committed-row rotation controls retain earlier valid state, with no later unauthorized effects; existing Calls DB is observed read-only. |
| NMR-AC-REACTION-RECOVERY-01 | Spec frozen at4d05c96a8485d67425d04140f85833ae19c57729, same FBCP-NMR-001 responsibility. | Four exact shipping_human_ cases in session_gateway_contract.rs extend77→81; reconciliation125 and zero failed/ignored, original30 bad-intent variants retained. Preserve UI205, owner40, actual Mac2+2 and all shipping gates. Require containing-H Actions; source audits and authored assertions do not qualify deployment or ACCEPT. |

Permanent-intent termination, arbitrary old-message mutation discovery, canonical
revision/tombstone semantics and active backend afterId=0 remain separate open
contracts. This batch does not clear a refused intent, migrate user data or grant
full source/product/independent acceptance.


### NMR-CONVERSATION-IDENTITY-01 — Canonical Human pair admission

| Requirement | Existing production owner / behavior | Test and evidence status |
|---|---|---|
| NMR-CONVERSATION-IDENTITY-01a | `ProductionSessionWorkers::read_human_conversation_metadata` checks Human kind, requested/persisted ID, two distinct canonical participant strings and `human_direct_conversation_id` equality for cached and cold owner access. | Implemented in `source/host/src/extensions/session/production.rs`; reversed participant order and title independence remain compatible. |
| NMR-CONVERSATION-IDENTITY-01b | Existing Session/Transcript/SandAgentDb authority; identity refusal does not repair, rewrite or delete metadata, transcript, resources or journals. | Rebound peer is refused before ordinary owner use; original metadata and all rows remain intact. |
| NMR-CONVERSATION-IDENTITY-01c | Existing owner reopens only after the original participant pair is restored; no automatic repair or migration. | Existing retry admission matrix and pure reopen path preserve the original conversation identity. |
| NMR-AC-CONVERSATION-IDENTITY-01 | Existing `shipping_human_retry_rejects_invalid_identity_and_payload_without_http_or_mutation` Host test and `native-message-reconciliation` Actions job. | Test is authored and pushed at `bbcee06bc82333e10b8fbea331fbd0f31453b32c`; run41564603392 is nonterminal at this receipt. No pass is claimed. |


Oracle and invariants for FBCP-NMR-001:

- ORA-NMR-CONVERSATION-CANONICAL-PAIR-001: a Human conversation owner is admitted only when the exact persisted participant pair deterministically derives the requested canonical conversation ID.
- INV-NMR-HUMAN-PAIR-MATCHES-DIRECTORY-ID-001: changing either participant without changing the canonical directory identity is rejected before the owner is returned to an ordinary consumer.
- INV-NMR-HUMAN-PAIR-ORDER-INDEPENDENT-001: the sorted participant pair produces the same canonical ID regardless of stored participant order; display title is not part of identity.



## NMR-UI-ENTRY-REFRESH-01 — Existing Human dependent UI lifetime

Frozen boundary: `docs/specs/fbcp-native-message-reconciliation.md#nmr-ui-entry-refresh-01--retire-dependent-ui-state-when-an-existing-human-row-changes`.
Same Host/Transcript/Renderer owners; no revision/tombstone or remote deletion inference.
Source tuple is shared by the existing workspace model, gallery and root spreadsheet
mount. Future source changes in both viewer owners and their PDF/media CSS remain in
the actual Native/UI and Mac workflow path filters.

| Requirement | Production entry | Required actual behavior evidence | Execution |
| --- | --- | --- | --- |
| NMR-UI-ENTRY-REFRESH-01a | ProductionRenderer Human find-in-chat request and navigation lifetime | human_entry_refresh_requeries_search_and_discards_older_results | Pending containing-H Actions |
| NMR-UI-ENTRY-REFRESH-01b | Existing MessageActionAnchor hooks, eligibility and listeners | human_entry_refresh_retires_action_menus_without_hook_order_errors | Pending containing-H Actions |
| NMR-UI-ENTRY-REFRESH-01c | TranscriptAttachmentGallery and shared transcriptAttachmentIdentity | human_entry_refresh_retires_removed_pdf_and_late_resource_reads | Pending containing-H Actions |
| NMR-UI-ENTRY-REFRESH-01d | ProductionRenderer original row/attachment-bound spreadsheet mount and existing provider | human_entry_refresh_retires_spreadsheet_mount_when_source_changes | Pending containing-H Actions |
| NMR-AC-UI-ENTRY-REFRESH-01 | Existing Native/UI Actions job and real ProductionRenderer/Coordinator/feed/viewer harness | Original8 plus four named cases above; UI209 = Node173 + projection13 + author23, with zero failures/skips/retries and retained Host/shipping gates | Authored; containing-H execution required |

All four cases extend the existing ProductionRenderer.transcript.behavior.test.mjs
from8 to12 without changing original8 bodies or strict diagnostics. Required
UI209 is173 Node +13 projection +23 author, zero skipped/retries; Human81/
reconciliation125, owner40, actualMac2+2 and all current gates stay mandatory.
Legal bounded-read viewer lifetime controls do not establish parser/rendering
acceptance. Fixed-U source, canonical mutation discovery, service deployment,
signed platforms, all Bot/quality and independent ACCEPT remain open.

Repair evidence: exact a0e6cd2ded523f59add3a7ade6bd43890b448300/run41565477479
executed UI209 with206 passed/3 failed and reconciliation84 with80 passed/4
failed; the remaining41 reconciliation cases did not execute. Source43529823/
41565715729 passed161 Node/37 Python/schema0 and unaccepted candidate. Later
6f7 source41566271637 rejected four redundant fields; current source3a4af377
removed those fields, while14 exact review-to-responsibility relations still
need the containing repair's source gate. No earlier result certifies this H.

The repair keeps invalid participant metadata through retry and cold-reopen
negative oracles; retained-forward journal access remains private and cannot
admit ordinary source messages. The actual viewer pointer test also checks
real window-control hit testing. Strict empty prompt shape and one external
pending read per fresh coalesced provider activation replace the invalid test
assumptions; no extra reads on updates/reorder and all late-result controls
remain. Promise-identity settlement retires shared pending-read slots without
clearing another generation. These source changes require new containing-H
Actions; schema, counts and every existing gate stay intact. Compact evidence:
projects/telegram-desktop-rust/evidence/round8-entry-refresh-repair-20261103.json.

Latest concurrent3a4af377451d915795466362b084247e05975101/run41566993215
executed reconciliation84 (80 passed/4 failed) and UI209 (207 passed/2 failed).
Source41566993101 passed161/37/schema0, then candidate125690859096 rejected
the first missing exact review relation. These are retained terminal failures,
not qualification of the repair.

Exact39d0e7555676c8987f6686a55bc6ae0be31c8c8d/run41567924833 now has
reconciliation125/125, including Human81 and creation14; UI209 executed208
passed/1 failed. The remaining PDF case fails actual Minimize hit testing
because the production Human handoff SELECT overlaps its center. Search,
action-menu and sheet lifetime cases passed; the original PDF pointer click
is retained after the failing hit assertion. Source41567924925 passed161
Node/37 Python/schema0 and unaccepted-candidate-consistent; no ACCEPT follows.

Concurrent85a5e9990f7d7cc040dd916d2adf577da95c873e/run41568637079
executed reconciliation101:100 passed/1 failed; later24 cases did not run.
Its added raw database-file read/close lets an external writer through a held
POSIX lock, caught by the unchanged human_format_probe_preserves_posix_locks_against_an_external_writer
case. The same corrupted-main/sidecar case still passed. Concurrente3945a01
removed raw reads and changed the immutable SQLite probe; keep both exact
oracles and verify retained-WAL compatibility on the containing head. The header
repair reserves the existing WindowChrome inset outside its hitbox and requires
both actual Handoff and all three window-control targets to remain hittable.
New containing-H125/209, actualMac2+2, shipping and source results are required.
Compact cause, changed blobs and independent review:
projects/telegram-desktop-rust/evidence/round8-header-controls-repair-20261103.json.

## TDRP-UI-SEARCH / FBCP-NMR-001 — Canonical Human+Agent Search Consolidation & Older-Page Navigation

| Requirement | Existing production owner | Required behavior and evidence | Execution |
| --- | --- | --- | --- |
| TDRP-UI-SEARCH (Host unified message/media/conversation search + domain-backed multi-term & forward-origin search) | `native/mahayana-messaging/src/search.rs` + `source/host/src/extensions/transcript/roster_search.rs` + `source/host/src/extensions/session/gateway.rs` + `ProductionSessionWorkers` + `ContentSearchExtension` | `search_agents`, `search_agents_linear`, and `search_media` merge Agent results with authenticated Human conversation message/attachment matches (`collect_human_conversation_search_results`, `collect_human_conversation_media_results`), enforcing account/participant isolation (`list_human_conversations` + `local_human_id`), deterministic `timestamp_ms` ordering, `role` mapping (`user` vs `assistant`), and `limit` truncation; `searchHumanMessages` delegates through `roster_search::search_human_conversation_messages`; `fabushi_messaging_core::query_matches_fields`, `visible_forward_origin_author`, and `tokenize` power recipient search, conversation/global/media search, and non-contiguous multi-term snippets while suppressing anonymous forward origin (`drop_sender_names || drop_captions`); covered by `roster_search_contract.rs::canonical_search_merges_authenticated_human_and_agent_conversations_and_enforces_account_isolation` (4/4) and `native/mahayana-messaging/src/search.rs#recipient_policy_tests::forward_origin_visibility_and_multi_term_field_matching_enforce_domain_policy`. | Verified green at `2ac90b3115a280c9a915f3b248b828b20f5ffab3` (`run 41686626329` job `126093969085`, artifact `13254024956`) |
| TDRP-UI-SEARCH (Renderer Command Palette Human search, older-page navigation & Find-in-Chat multi-term/forward-origin projection) | `frontend/src/production/ProductionRenderer.tsx` + `CommandPalette.tsx` + `transcriptPaginationController` + `frontend/src/production/model.ts` + `find-in-chat-controller.ts` | `CommandPalette` receives `agents={allConversations}` (`[...humanConversations, ...agents]`); `openPaletteMessage` / `openPaletteFile` and `paletteMessageTarget` navigate to the target conversation and paginate older Human transcript pages (`transcriptPaginationController.loadOlder()`, bounded to 32 pages and fenced by account/conversation scope); `projectTranscriptEntry` projects `authorName` and privacy-gated `forwardedFromAuthorName` (`visibleForwardOriginAuthorName`), and `findInChatSearchText` / `matchesFor` support multi-term token matching across body, attachments, author, and visible forward origin; covered by `ProductionRenderer.transcript.behavior.test.mjs::command_palette_searches_human_conversations_messages_and_files_and_paginates_older_targets` and `human_entry_refresh_requeries_search_and_discards_older_results` (13/13), plus `recovered-conversation-infra.contract.test.ts` (19/19). | Verified green at `2ac90b3115a280c9a915f3b248b828b20f5ffab3` (`run 41686626329` job `126093969100`, artifact `13256170163`) |
| NMR-CLI-HOST-01 (`mahayana-gateway` macOS mutex starvation fix) | `source/mahayana/mahayana-rs/mahayana-cli/src/bin/mahayana-gateway.rs` (`event_pump`) | Polls `runtime.receive(0)` under `pump_runtime.lock()`, drops the mutex guard immediately, and sleeps `10ms` outside the lock when `Ok(None)` is returned so `main`'s command thread is not starved on macOS unfair `std::sync::Mutex`; verified by `native-message-shipping-compile` (`mahayana-gateway` 9/9). | Verified green at `2ac90b3115a280c9a915f3b248b828b20f5ffab3` (`run 41686626329` job `126093968967`, artifact `13255394553`) |
| FBCP-NMR-001 (Multi-window Human history coverage continuation across 32-page sync windows) | `source/host/src/extensions/session/production.rs` + `source/host/src/extensions/session/gateway.rs` + `frontend/src/production/ProductionRenderer.tsx` | `metadata_history_coverage_incomplete` and `human_conversation_coverage_incomplete` expose `"historyCoverageComplete": false` on `listHumanConversations` summaries and `getHumanConversationTranscriptTail` responses whenever `humanHistoryCoverage.complete == false` (omitting the field when complete); `ProductionRenderer::continueHumanHistoryCoverage` (`humanCoverageContinuationInFlightRef`) drives bounded multi-window `syncHumanConversation` -> `listHumanConversations` -> `getHumanConversationTranscript` continuation for the active conversation, fenced by account, transport, selection, and conversation generations; covered by `session_gateway_contract.rs::shipping_human_history_coverage_incomplete_signal_is_exposed_until_sync_reaches_end` and `ProductionRenderer.transcript.behavior.test.mjs::human_history_coverage_continuation_advances_multi_window_sync_and_fences_account_switches`. | Verified green at `1ba6c3680af2237cd6bbfb4b4a08070d03e6644d` (`run 41691497048` job `126110954651` artifact `13257940723` & job `126110954631` artifact `13255819231`) |
| FBCP-NMR-001 / NMR-READ-RECEIPT-01 (Human read-receipt monotonic projection & privacy-gated forwarded provenance UI) | `frontend/src/recovered/features/conversation/workspace/model.ts` + `frontend/src/production/model.ts` + `frontend/src/production/transcript-projection.ts` + `frontend/src/recovered/features/conversation/workspace/transcript.tsx` | `projectTranscriptEntry` projects validated RFC3339 `readAt` (`isValidHumanReadAtTimestamp`) onto canonical settled Human rows (`id === "human-server-message:" + remoteMessageId`); `mergeAuthoritativeTranscriptMessage` enforces monotonic `readAt` (`monotonicHumanReadAt`) across out-of-order refreshes and mutations on the same canonical Human row (`id` + `authorId`); `ConversationTranscript` renders privacy-gated `ForwardedFromNotice` (`role="note"`, `"Forwarded from {authorName}"`, `shouldShowHumanForwardedNotice`) and outgoing settled `HumanReadReceiptNotice` (`role="status"`, `data-read-at={readAt}`, `"Read · {label}"`, `shouldShowHumanReadReceipt`); covered by `transcript-projection.contract.test.ts::NMR-READ-RECEIPT-01 Human read receipts and forwarded provenance require valid canonical state and preserve monotonic readAt across refreshes` (44/44) and `ProductionRenderer.transcript.behavior.test.mjs::human_entry_refresh_renders_monotonic_read_receipts_and_privacy_gated_forwarded_provenance` (15/15). | Verified green at `9385fad973fcd1a751af48100e5b410d33f34022` (`run 41695632873` job `126125274269` artifact `13262305740`) |
| FBCP-NMR-001 / TDRP-MSG-FWD (Chained forward-origin attribution & peer display title fallback) | `native/mahayana-messaging/src/message.rs` + `source/host/src/extensions/session/production.rs` + `source/host/tests/roster_search_contract.rs` | `ForwardPrivacy::resolve_forward_origin_attribution` (`ForwardOriginAttribution`) and `ProductionSessionWorkers::resolve_human_forward_origin` preserve visible upstream `forwardContext.origin` across chained forwards (`dropSenderNames: false`), prevent anonymous forwards (`dropSenderNames: true` or `dropCaptions: true`) from leaking hidden upstream origins when re-forwarded, and fall back to the source conversation's peer display `title` when forwarding a peer message whose row lacks `authorName`; covered by `native/mahayana-messaging/src/search.rs#recipient_policy_tests::forward_origin_visibility_and_multi_term_field_matching_enforce_domain_policy` and `roster_search_contract.rs::canonical_search_merges_authenticated_human_and_agent_conversations_and_enforces_account_isolation`. | Verified green at `c1882a8c8f2af0019d3d1e4abbfaee21a690536b` (`run 41700052529` job `126140642522` artifact `13262543561` & job `126140643192` artifact `13261494815`) |
| TDRP-R9-SHARE-RECIPIENT-ELIGIBILITY-001 & TDRP-R9-SHARE-DESTINATION-POLICY-001 (Typed share/forward recipient & destination policy enforcement) | `native/mahayana-messaging/src/conversation.rs` + `native/mahayana-messaging/src/community.rs` + `native/mahayana-messaging/src/service.rs` + `native/mahayana-messaging/src/engine.rs` + `source/host/src/extensions/session/production.rs` | `ConversationPermissions` (`can_send_other`, `can_send_inline`, `can_send_games`), `MemberRestrictions` (`send_other`, `send_inline`, `send_games`), `MessagingService::actor_can_send_to_recipient`, `MessagingEngine` (`OtherSendPermissionDenied`, `CommunityOtherRestricted`, `message_content_uses_other`), and `ProductionSessionWorkers` (`human_recipient_requirements_from_entry`, `human_recipient_authorization_from_metadata`) enforce typed destination eligibility both before recipient exposure (`search_human_recipients` / `SearchRecipients`) and at forward/send settlement (`deliver_forward_batch_while_locked` / `ForwardMessage` / `QueueMessage`); covered by `m5_contacts_groups_contract.rs::share_recipient_search_uses_canonical_typed_permission_axes_before_exposure` & `typed_share_permission_fields_keep_legacy_persistence_fail_closed` (6/6) and `production.rs::sharebox_shipping_tests::shipping_human_forward_reuses_session_owner_across_retry_and_restart`. | Verified green at `0fc16eb80ffd421dc2cc65a7f0e0cea06b6de895` (`run 41712794667` job `126185100349` artifact `13269395432`) and `6a7a730ea2bb77bb5d2587332584e8b8f44cb5a2` (`run 41719169070` job `126207063253` artifact `13270999524`) |
| NMR-UI-ENTRY-REFRESH-01 & TDRP-R9-SHARE-FORWARD-PRIVACY-001 / SB-04 (Forward dialog live source refresh & ineligible/cleared source retirement) | `frontend/src/production/transcript-projection.ts` + `frontend/src/production/ProductionRenderer.tsx` + `frontend/src/production/ForwardMessageDialog.tsx` | `isForwardableTranscriptMessage` and `reconcileForwardTargetMessage` reconcile live transcript updates (`onBaseline`, `onUpdated`, `onAppended`) against the open `forwardTarget`, preserving `ForwardMessageDialog` scope on in-place content/read-receipt updates and synchronously retiring `forwardTarget` to `null` (and fencing in-flight `searchHumanRecipients` / pre-ready `forwardHumanMessage`) when the source message becomes empty/non-actionable or the conversation is cleared (`onCleared`); covered by `transcript-projection.contract.test.ts::NMR-UI-ENTRY-REFRESH-01` (45/45) and `ProductionRenderer.transcript.behavior.test.mjs::human_entry_refresh_retires_action_menus_without_hook_order_errors` (15/15). | Verified green at `0fc16eb80ffd421dc2cc65a7f0e0cea06b6de895` (`run 41712794667` job `126185100516` artifact `13269447448`) and `6a7a730ea2bb77bb5d2587332584e8b8f44cb5a2` (`run 41719169070` job `126207063156` artifact `13271148419`) |
| TDRP-UI-SEARCH (Scoped Media/Files/Links SearchIndex parity, attachment metadata persistence & Rich Agent transcript search) | `native/mahayana-messaging/src/search.rs` + `source/host/src/extensions/session/production.rs` + `source/host/src/extensions/transcript/roster_search.rs` | `SearchIndex` enforces scoped `Media` (`Photo`/`Video`/`MediaGroup`), `Files` (`Document`/`File`/`Voice`), and `Links` (`LinkPreview` or embedded `http://`/`https://` URLs) collection filtering (`entry_matches_scope`), indexes `file_name` and `url`, and applies `visible_forward_origin_author` (`drop_sender_names || drop_captions`); `ProductionSessionWorkers::append_human_message_from_intent` preserves normalized attachment `"mimeType"`, `"width"`, and `"height"`; `roster_search` surfaces all matching attachments per Human message (`human:{conversation_id}:{message_id}:attachment:{idx}`) and indexes structured Agent transcript entries (`Rich`, `Code`, `Thinking`, `ToolCall`, `ToolResult`, `Approval`, `Artifact`); covered by `native/mahayana-messaging/src/search.rs#recipient_policy_tests::search_index_supports_scoped_media_files_links_collections_and_filename_matching` and `roster_search_contract.rs::canonical_search_merges_authenticated_human_and_agent_conversations_and_enforces_account_isolation` (4/4). | Verified green at `6a7a730ea2bb77bb5d2587332584e8b8f44cb5a2` (`run 41719169070` job `126207063253` artifact `13270999524`) |
| FBCP-NMR-001 / TDRP-R9-SHARE-MODAL-UI-001 (Live reply-target invalidation & ForwardMessageDialog keyboard navigation and backdrop protection) | `frontend/src/recovered/features/conversation/workspace/reply-thread-controller.ts` + `frontend/src/production/ProductionRenderer.tsx` + `frontend/src/production/ForwardMessageDialog.tsx` | `isActionableReplyTargetEntry` and `ReplyThreadController.syncEntries` automatically clear `activeReplyTarget` and notify `onReplyTargetInvalidated` when a live transcript refresh clears the conversation or renders the reply target non-actionable (`pending`, `failed`, or empty text+attachments), wired in `ProductionRenderer` to clear `pendingReplyToMessageId` without discarding typed composer text; `ForwardMessageDialog` search input handles `ArrowDown`/`ArrowUp`/`Home`/`End`/`Enter`/`Space` with `aria-activedescendant` and `aria-controls="forward-recipient-results"`, and disables backdrop dismissal (`closeOnBackdrop={selected.size === 0 && !snapshotLocked && !submitting}`) while recipients are selected; covered by `reply-thread-controller.contract.test.ts` (5/5), `ForwardMessageDialog.behavior.test.mjs` (17/17), and `ProductionRenderer.transcript.behavior.test.mjs` (15/15). | Verified green at `6a7a730ea2bb77bb5d2587332584e8b8f44cb5a2` (`run 41719169070` job `126207063156` artifact `13271148419`) |
| TDRP-R9-SHARE-THREAD-DESTINATION-001 & TDRP-R9-SHARE-SEND-OPTIONS-001 (Domain reply-draft & child-window deletion reconciliation + self / Saved Messages forward feedback) | `native/mahayana-messaging/src/conversation.rs` + `native/mahayana-messaging/src/engine.rs` + `frontend/src/production/forward-settlement.ts` + `frontend/src/production/ProductionRenderer.tsx` | `ConversationChildPaginationState::remove_messages`, `ConversationChildRuntimeState::clear_draft_reply_to`, `ConversationChildRuntimeState::remove_deleted_messages`, and `MessagingEngine::apply(Event::MessagesDeleted)` clear deleted `reply_to_message_id` from conversation, topic, and child destination drafts while preserving non-empty draft text (and removing empty reply-only drafts) and reconciling child pagination windows (`note_locally_empty`); `settleForwardProjection` and `ProductionRenderer::settleForward`bind `selfConversationId` (`human:${selfAuthorId}`) for single self / Saved Messages forwards, supporting `"Saved Messages"` default label, custom self label, `onSelfForwarded` callback, and optional `suppressSelfNotice`; covered by `child_destination_tests::child_runtime_removes_deleted_messages_and_clears_matching_draft_reply_without_losing_text` (7/7), `m6_channels_topics_contract.rs::deleting_messages_clears_reply_drafts_without_losing_text_and_reconciles_child_windows` (5/5), and `forward-settlement.contract.test.ts::single self/Saved Messages destination uses Saved Messages default, custom self label, or optional notice suppression` (10/10). | Authored on top of `6a7a730ea2bb77bb5d2587332584e8b8f44cb5a2`; requires containing-H GitHub Actions execution |
