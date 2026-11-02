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
New containing-H execution is not-run at preparation; Actions supplies the
exact target, run/job/artifact binding. Historical passing counts are not new-H evidence.

| Requirement / oracle | Existing production owner | Authored check / actual base evidence |
| --- | --- | --- |
| NMR-46 / NMR-AC-37 recovery | ForwardMessageDialog fixture using existing runtime theme installer | Base renderer/Transcript16/settlement5 pass; Chromium11/12, missing setTheme. Retain Chromium12, keyboard control and four actual light/dark viewport screenshots. |
| NMR-47a/b / NMR-AC-38 | Source SandAgentDb journal plus destination Session/Transcript | Five actual HTTP/SQLite/process cases now authored; parent kills and reaps child, reads committed intent independently, resumes untouched target, injects staging/progress SQL errors. Require Human38. |
| NMR-47c/d / NMR-AC-38 | Existing credential/source authorization and ordinary sync | Ambiguous first POST reuses nonce; immutable source snapshot; same/subset/concurrent completed replay; changed source/privacy/expanded-set negatives; source/account/issuer denial; no generic-outbox bypass. |
| NMR-47e/f / NMR-AC-38 | Existing SandAgentDb expected-value CAS and Session eligibility | Two-handle stale read regression plus typed source denial and unrelated actual history GET; I/O failures still stop. Select sharebox1 and journal3 separately. |
| Gateway / Human / Agent preservation | Existing authenticated Gateway and Session writers | Base Human33/sharebox1 pass, count assertion fails; Agent4/private3/Gateway1 are skipped after it. All remain required on new H. |
| Native package / Social / transport | Actual AppHost shipping root and resource-copy owner | Base shipping125481711613 compiled AppHost and passed composition12/Native7/FeatureHost7/Social9/HTTP-owner6/Host6/ABI7/CLI3/gateway5, then stale expected3 count failed. Later library/protocol/TCP/copy checks not run; new H must execute them. |
| Electron / Windows | Existing carrier/auth/update/publisher owners | Base Electron125481711452 and Windows125481711041 succeed; no formal publisher or signed-package acceptance. |

Source_reviews remains absent; source/product/independent acceptance stay
false. Backend #137 remains open/draft/unmerged and read-only. Remote
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
