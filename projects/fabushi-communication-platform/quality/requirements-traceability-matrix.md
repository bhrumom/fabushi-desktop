# Requirements Traceability Matrix

Status: active / normative schema  
Matrix ID: FQT-RTM-001  
Parent: FBCP-001 Revision 7  
Updated: 2026-10-07

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
| TDRP-R9-HISTORY-STATE-CONTRACT-001 | high | ORA-TDRP-R9-HISTORY-STATE-CONTRACT-001 | INV-TDRP-R9-HISTORY-STATE-CONTRACT-001-CANONICAL | pending | pending | pending child/message lifecycle integration coverage | pending Conversation/Transcript/Composer projection coverage | pending Search/resource cleanup coverage | pending Bot + messaging regression | evidence:pending-current-head | pending-independent-review | MAPPED |
| TDRP-R9-HISTORY-LIFECYCLE-001 | critical | ORA-TDRP-R9-HISTORY-LIFECYCLE-001 | INV-TDRP-R9-HISTORY-LIFECYCLE-001-CANONICAL | pending | pending | pending deletion/destroyed ordering + fault/temporal coverage | pending exact child cleanup E2E | pending authorization/Search/resource cleanup coverage | pending Bot + messaging regression | evidence:pending-current-head | pending-independent-review | MAPPED |
| TDRP-R9-DATA-CHANGES-CONTRACT-001 | high | ORA-TDRP-R9-DATA-CHANGES-CONTRACT-001 | INV-TDRP-R9-DATA-CHANGES-CONTRACT-001-CANONICAL | pending | pending | pending destruction/cleanup integration and temporal coverage | pending canonical Conversation/Composer projection coverage | pending resource/security cleanup coverage | pending Bot + messaging regression | evidence:pending-current-head | pending-independent-review | MAPPED |
| TDRP-R9-DATA-CHANGES-LIFECYCLE-001 | high | ORA-TDRP-R9-DATA-CHANGES-LIFECYCLE-001 | INV-TDRP-R9-DATA-CHANGES-LIFECYCLE-001-CANONICAL | pending | pending | pending realtime/coalesced/destroyed sequencing coverage | pending child lifecycle projection coverage | pending stale-event/resource cleanup coverage | pending Bot + messaging regression | evidence:pending-current-head | pending-independent-review | MAPPED |
| TDRP-R9-DATA-TYPES-VALUE-CONTRACT-001 | high | ORA-TDRP-R9-DATA-TYPES-VALUE-CONTRACT-001 | INV-TDRP-R9-DATA-TYPES-VALUE-CONTRACT-001-CANONICAL | pending | pending | pending forward/message value-contract integration coverage | pending Composer/Conversation projection coverage | pending resource/cache/security applicability coverage | pending Bot + messaging regression | evidence:pending-current-head | pending-independent-review | MAPPED |
| TDRP-R9-IV-VIEW-WIDGET-LIFECYCLE-001 | high | ORA-TDRP-IV-VIEW-WIDGET-001 | INV-TDRP-IV-VIEW-REPLACE-001; INV-TDRP-IV-SELECTION-ACTION-001; INV-TDRP-IV-COPY-TYPED-001; INV-TDRP-IV-SCROLL-LIFECYCLE-001; INV-TDRP-IV-VISIBLE-WORK-001; INV-TDRP-IV-MEDIA-ARBITRATION-001 | UNIT-TDRP-IV-VIEW-HIGHLIGHT-RUNTIME-001 | CONTRACT-TDRP-IV-VIEW-HIGHLIGHT-DISPOSAL-001; CONTRACT-TDRP-IV-VIEW-POINTER-ACTIVATION-FENCE-001; CONTRACT-TDRP-IV-VIEW-MEDIA-POINTER-RELEASE-001 | pending broader drag/gesture/hidden/touch/scroll temporal matrix | pending packaged keyboard/a11y/visual matrix | pending UI performance/soak and security review | pending Bot + transcript regression breadth | workflow:pending-current-head | pending-independent-review | MAPPED |
| TDRP-R9-IV-ARTICLE-TEXT-LINK-001 | high | ORA-TDRP-IV-ARTICLE-TEXT-001 | INV-TDRP-IV-LINK-TYPED-001 | — | CONTRACT-TDRP-IV-PREPARED-LINK-EXTERNAL-COPY-001; INT-TDRP-FABUSHI-ACCOUNT-EXTERNAL-OPEN-001 | pending remaining link-kind/replacement temporal matrix | pending packaged context-menu keyboard/a11y/visual matrix | pending full external-link policy negative matrix | pending Bot + transcript typed-link regression breadth | workflow:pending-current-head | pending-independent-review | MAPPED |
| TDRP-R9-IV-ARTICLE-CONTRACT-001 | high | ORA-TDRP-IV-ARTICLE-001 | INV-TDRP-IV-ARTICLE-PATCH-001; INV-TDRP-IV-ARTICLE-SCROLL-001; INV-TDRP-IV-ARTICLE-RUNTIME-001; INV-TDRP-IV-ARTICLE-HEAVY-001 | UNIT-TDRP-IV-ARTICLE-SCROLL-CLAMP-001; UNIT-TDRP-IV-ARTICLE-MEDIA-VISIBILITY-001; UNIT-TDRP-IV-ARTICLE-PROJECTION-RECONCILE-001; UNIT-TDRP-IV-ARTICLE-PROJECTION-SEQUENCE-001 | CONTRACT-TDRP-IV-ARTICLE-SCROLL-CONTINUITY-001; CONTRACT-TDRP-IV-ARTICLE-MEDIA-LIFECYCLE-001; CONTRACT-TDRP-IV-ARTICLE-PARTIAL-FALLBACK-001 | pending full article replacement/reload temporal coverage | pending packaged keyboard/a11y/visual matrix | pending UI performance/soak and security review | pending Bot + transcript regression breadth | workflow:pending-current-head | pending-independent-review | MAPPED |
| TDRP-R9-IV-ARTICLE-CONTENT-LIFECYCLE-001 | high | ORA-TDRP-IV-ARTICLE-001 | INV-TDRP-IV-ARTICLE-PATCH-001; INV-TDRP-IV-ARTICLE-SCROLL-001; INV-TDRP-IV-ARTICLE-RUNTIME-001; INV-TDRP-IV-ARTICLE-HEAVY-001 | UNIT-TDRP-IV-ARTICLE-SCROLL-CLAMP-001; UNIT-TDRP-IV-ARTICLE-MEDIA-VISIBILITY-001; UNIT-TDRP-IV-ARTICLE-PROJECTION-RECONCILE-001; UNIT-TDRP-IV-ARTICLE-PROJECTION-SEQUENCE-001 | CONTRACT-TDRP-IV-ARTICLE-SCROLL-CONTINUITY-001; CONTRACT-TDRP-IV-ARTICLE-MEDIA-LIFECYCLE-001; CONTRACT-TDRP-IV-ARTICLE-PARTIAL-FALLBACK-001 | pending arbitrary replacement/reload temporal coverage | pending packaged keyboard/a11y/visual matrix | pending UI performance/soak and security review | pending Bot + transcript regression breadth | workflow:pending-current-head | pending-independent-review | MAPPED |
| TDRP-R9-IV-ARTICLE-INTERACTION-001 | high | ORA-TDRP-IV-ARTICLE-001 | INV-TDRP-IV-ARTICLE-SCROLL-001; INV-TDRP-IV-ARTICLE-HIT-001; INV-TDRP-IV-ARTICLE-REVEAL-001; INV-TDRP-IV-ARTICLE-RUNTIME-001 | UNIT-TDRP-IV-ARTICLE-SCROLL-CLAMP-001; UNIT-TDRP-IV-ARTICLE-HIDDEN-SEARCH-001 | CONTRACT-TDRP-IV-ARTICLE-SCROLL-CONTINUITY-001; CONTRACT-TDRP-IV-ARTICLE-HIDDEN-REVEAL-001 | pending quote/details ancestor and relayout reveal temporal coverage | pending packaged keyboard/a11y/visual matrix | pending UI performance/soak and security review | pending Bot + transcript regression breadth | workflow:pending-current-head | pending-independent-review | MAPPED |

### TDRP-R9-IV-VIEW-WIDGET-LIFECYCLE-001 oracle

- `ORA-TDRP-IV-VIEW-WIDGET-001`: derived pointer, search-highlight, media and repaint work must resolve only against the current canonical conversation surface and cannot settle into a replaced or disposed owner.
- `INV-TDRP-IV-VISIBLE-WORK-001`: Find-in-Chat owns at most one pending highlight refresh; superseding navigation cancels the previous frame, transcript-container replacement invalidates it, settlement reads the latest controller/container references, and real unmount disposal prevents late CSS Highlight ranges from retaining detached DOM.
- `INV-TDRP-IV-VIEW-REPLACE-001`: transcript pointer intent for canonical links and buttons is bound to the current entries generation plus exact action element; entries replacement, pointer cancellation, pointer leave, or movement beyond the bounded drag threshold retires that intent, and a later pointer click is rejected in capture unless both identities still match. This includes transcript attachment/media buttons and disclosure toggles; keyboard activation remains independent.
- `INV-TDRP-IV-MEDIA-ARBITRATION-001`: canonical MediaViewer replacement and unmount release any active pointer capture before retiring pointer ownership; implicit `lostpointercapture` clears the matching owner; zoomed pan capture ignores interactive descendants so close/navigation/filmstrip controls keep their own activation lifecycle.
- Current cases: `UNIT-TDRP-IV-VIEW-HIGHLIGHT-RUNTIME-001`, `CONTRACT-TDRP-IV-VIEW-HIGHLIGHT-DISPOSAL-001`, `CONTRACT-TDRP-IV-VIEW-POINTER-ACTIVATION-FENCE-001`, and `CONTRACT-TDRP-IV-VIEW-MEDIA-POINTER-RELEASE-001`.
- Dossier: `projects/telegram-desktop-rust/dossiers/iv-markdown-view-widget-complete-read.md`.
- Current verdict: `MAPPED`; bounded Highlight refresh/disposal, transcript link/button replacement fencing, and MediaViewer pointer-capture retirement are present, while exact-head execution and MediaViewer pan/zoom gesture breadth, touch/scroll/zoom, hidden-window timer and packaged lifecycle evidence remain pending.

### TDRP-R9-IV-ARTICLE-TEXT-LINK-001 oracle

- `ORA-TDRP-IV-ARTICLE-TEXT-001`: typed rich-text links keep open target, copy payload and contextual label distinct while remaining derived from canonical transcript content and the shipping desktop external-open boundary.
- `INV-TDRP-IV-LINK-TYPED-001`: HTTP(S) and Email may project as external actions, but unsupported script/file/relative targets cannot silently acquire external-open authority. Mailto query data is not copied as the email address; decoded address text is exposed through the existing message-menu `Copy Email` affordance. Partial/custom rendered labels disclose the canonical target only when label and target are not equivalent, while bare targets avoid redundant tooltip text.
- Current bounded case: `CONTRACT-TDRP-IV-PREPARED-LINK-EXTERNAL-COPY-001`, with the shipping external-open path continuing through `INT-TDRP-FABUSHI-ACCOUNT-EXTERNAL-OPEN-001`.
- Dossier: `projects/telegram-desktop-rust/dossiers/iv-markdown-article-text-complete-read.md`.
- Current verdict: `MAPPED`; External/Email projection plus partial/custom-label target disclosure is implemented, while exact-head execution, broader CustomUrl/InstantView entity-shape coverage, Anchor, footnote/backlink, LocalFile, rejected-relative/toggle/RichPageButton semantics and packaged interaction evidence remain pending.

### TDRP-R9-IV-ARTICLE-CONTRACT-001 oracle

- `ORA-TDRP-IV-ARTICLE-001`: rich-content projection remains under the canonical transcript/message identity; derived horizontal-scroll state never becomes a second article or message owner. Both the ordinary transcript and send-message:text lazy leaf must pass the canonical entry identity.
- `INV-TDRP-IV-ARTICLE-PATCH-001`: assistant projection keys persist only for the same canonical owner/path/kind/shape and exact or monotonic committed-stream growth; shortening, non-prefix rewrite, owner change or structural mismatch increments generation and remounts the whole projection.
- `INV-TDRP-IV-ARTICLE-SCROLL-001`: code/table horizontal scroll restores only for a compatible canonical message plus structural owner and clamps to current geometry.
- `INV-TDRP-IV-ARTICLE-HEAVY-001`: transcript-card image/video derived state is resolved only inside a bounded near-viewport margin, filmstrip thumbnails resolve only while active or horizontally near-visible, equal-margin subscriptions share one observer, and off-budget state is released with stale-settlement fencing before reconstruction from canonical attachment metadata; audio and the explicitly opened `MediaViewer` remain outside this unload policy.
- Current cases: `UNIT-TDRP-IV-ARTICLE-SCROLL-CLAMP-001`, `UNIT-TDRP-IV-ARTICLE-MEDIA-VISIBILITY-001`, `UNIT-TDRP-IV-ARTICLE-PROJECTION-RECONCILE-001`, `UNIT-TDRP-IV-ARTICLE-PROJECTION-SEQUENCE-001`, `CONTRACT-TDRP-IV-ARTICLE-SCROLL-CONTINUITY-001`, `CONTRACT-TDRP-IV-ARTICLE-MEDIA-LIFECYCLE-001`, and `CONTRACT-TDRP-IV-ARTICLE-PARTIAL-FALLBACK-001`.
- Dossier: `projects/telegram-desktop-rust/dossiers/iv-markdown-article-complete-read.md`.
- Current verdict: `MAPPED`; bounded projection patch/fallback, scroll and shared derived-media lifecycle slices are present, while exact-head execution and the full article contract remain pending.

### TDRP-R9-IV-ARTICLE-CONTENT-LIFECYCLE-001 oracle

- `ORA-TDRP-IV-ARTICLE-001`: compatible derived state may be reused only while its canonical message plus structural owner remains compatible through content revision.
- `INV-TDRP-IV-ARTICLE-PATCH-001`: the last committed assistant projection is the only reuse baseline; exact or monotonic streaming growth patches compatible entries, while shortening, non-prefix rewrite, owner change or structural mismatch increments generation and remounts all assistant blocks.
- `INV-TDRP-IV-ARTICLE-SCROLL-001`: code/table horizontal scroll is captured before content revision, restored only for the same owner identity, clamped to current geometry, and reset on owner mismatch.
- `INV-TDRP-IV-ARTICLE-RUNTIME-001`: scroll geometry remains derived component state rather than canonical message truth and is released with the owning transcript region.
- `INV-TDRP-IV-ARTICLE-HEAVY-001`: visibility changes cancel stale image/video and filmstrip-thumbnail settlement, retire only reconstructable derived state, and release a shared observer after its final subscriber; canonical attachment identity, audio playback policy, and an explicitly opened viewer remain owned by their existing boundaries.
- Current cases: `UNIT-TDRP-IV-ARTICLE-SCROLL-CLAMP-001`, `UNIT-TDRP-IV-ARTICLE-MEDIA-VISIBILITY-001`, `UNIT-TDRP-IV-ARTICLE-PROJECTION-RECONCILE-001`, `UNIT-TDRP-IV-ARTICLE-PROJECTION-SEQUENCE-001`, `CONTRACT-TDRP-IV-ARTICLE-SCROLL-CONTINUITY-001`, `CONTRACT-TDRP-IV-ARTICLE-MEDIA-LIFECYCLE-001`, and `CONTRACT-TDRP-IV-ARTICLE-PARTIAL-FALLBACK-001`.
- Dossier: `projects/telegram-desktop-rust/dossiers/iv-markdown-article-complete-read.md`.
- Current verdict: `MAPPED`; bounded committed-generation projection patch/fallback, code/table scroll and shared derived-media lifecycle slices are present, while exact-head execution and the broader article lifecycle remain pending.

### TDRP-R9-IV-ARTICLE-INTERACTION-001 oracle

- `ORA-TDRP-IV-ARTICLE-001`: code/table overflow and hidden-detail search interaction remain in the canonical `ConversationTranscript` plus `FindInChatController`; no IV-specific search, disclosure or message owner is introduced.
- `INV-TDRP-IV-ARTICLE-HIT-001`: scroll and find regions do not acquire message, URL, media or service capability; those actions stay delegated to existing typed owners.
- `INV-TDRP-IV-ARTICLE-REVEAL-001`: thinking/tool-call search text mirrors the expanded UI order, including only the visible ToolResultCard projection; navigation resolves a stable `data-entry-id`, expands the existing disclosure set, and reapplies highlights from the existing view-commit subscription after the detail mounts.
- Current cases: `UNIT-TDRP-IV-ARTICLE-SCROLL-CLAMP-001`, `UNIT-TDRP-IV-ARTICLE-HIDDEN-SEARCH-001`, `CONTRACT-TDRP-IV-ARTICLE-SCROLL-CONTINUITY-001`, and `CONTRACT-TDRP-IV-ARTICLE-HIDDEN-REVEAL-001`; packaged keyboard/a11y evidence remains pending.
- Dossier: `projects/telegram-desktop-rust/dossiers/iv-markdown-article-complete-read.md`.
- Current verdict: `MAPPED`; bounded scroll and collapsed thinking/tool-call reveal slices are present, while quote/details ancestor expansion, relayout reveal, pointer/touch breadth, packaged evidence and independent acceptance remain pending.

### TDRP-R9-HISTORY-STATE-CONTRACT-001 oracle

- `ORA-TDRP-R9-HISTORY-STATE-CONTRACT-001`: root, Topic and SavedSublist history state stays in canonical Conversation/Message owners with exact child identity; drafts/forward drafts, read cursors and deletion commands never fall back to parent or renderer-inferred identity.
- `INV-TDRP-R9-HISTORY-STATE-CONTRACT-001-CANONICAL`: only existing exact-head `MessagingEngine`, `ConversationChildRuntimeState` and `Message` symbols are registered as mapped targets. Search/resource cleanup remains explicitly open until a real shipping binding and evidence exists.
- Dossier: `projects/telegram-desktop-rust/dossiers/history-core-complete-read.md`.
- Current verdict: `MAPPED`; no implementation/verification evidence is claimed.

### TDRP-R9-HISTORY-LIFECYCLE-001 oracle

- `ORA-TDRP-R9-HISTORY-LIFECYCLE-001`: deletion/clear publishes the exact message/child lifecycle while identity is still available, clears notification/draft/unread/index projections for that exact scope, and prevents stale work from resurrecting retired state.
- `INV-TDRP-R9-HISTORY-LIFECYCLE-001-CANONICAL`: deletion lifecycle remains owned by canonical MessagingEngine/Conversation/Message boundaries; SearchIndex/resource cleanup is an open downstream responsibility and SavedSublist authority remains fail-closed on the server relation/membership feed.
- Dossier: `projects/telegram-desktop-rust/dossiers/history-core-complete-read.md`.
- Current verdict: `MAPPED`; production/tests/evidence remain pending.

### TDRP-R9-DATA-CHANGES-CONTRACT-001 oracle

- `ORA-TDRP-R9-DATA-CHANGES-CONTRACT-001`: canonical Fabushi owners preserve the typed Topic/SavedSublist/Message/Entry change identity and destination scope needed by draft, unread, notification, Search and resource observers; no source-shaped event bus or renderer inference becomes authoritative.
- `INV-TDRP-R9-DATA-CHANGES-CONTRACT-001-CANONICAL`: mapped state is owned by existing canonical MessagingEngine/Conversation/Message/Composer boundaries. Search/resource cleanup remain explicit downstream open responsibilities until their shipping owner bindings and evidence are added; absence of those bindings is not treated as implementation.
- Dossier: `projects/telegram-desktop-rust/dossiers/data-changes-types-complete-read.md`.
- Current verdict: `MAPPED`; production/test evidence intentionally pending.

### TDRP-R9-DATA-CHANGES-LIFECYCLE-001 oracle

- `ORA-TDRP-R9-DATA-CHANGES-LIFECYCLE-001`: ordinary typed changes may coalesce, but destruction merges pending flags, publishes the exact identity before retirement, and prevents a later scheduled callback from reviving stale child/message/entry state.
- `INV-TDRP-R9-DATA-CHANGES-LIFECYCLE-001-CANONICAL`: destruction sequencing is mapped to existing MessagingEngine lifecycle ownership; notification clear, draft clear, unread reconciliation, Search projection removal and resource cleanup remain open until wired and proven in shipping composition.
- Dossier: `projects/telegram-desktop-rust/dossiers/data-changes-types-complete-read.md`.
- Current verdict: `MAPPED`; no current-head implementation or verification claim.

### TDRP-R9-DATA-TYPES-VALUE-CONTRACT-001 oracle

- `ORA-TDRP-R9-DATA-TYPES-VALUE-CONTRACT-001`: source value semantics such as ForwardOptions/ForwardDraft, message identity/flags, cursors and resource/cache classifications are decomposed into the corresponding existing Fabushi owners without introducing a monolithic Telegram-shaped DataTypes owner.
- `INV-TDRP-R9-DATA-TYPES-VALUE-CONTRACT-001-CANONICAL`: only target symbols that actually exist in the current exact-head canonical owners may be registered; future Search/resource/cache bindings remain open rather than being represented by invented target symbols or fake evidence.
- Dossier: `projects/telegram-desktop-rust/dossiers/data-changes-types-complete-read.md`.
- Current verdict: `MAPPED`; applicability implementation/tests remain pending.

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
