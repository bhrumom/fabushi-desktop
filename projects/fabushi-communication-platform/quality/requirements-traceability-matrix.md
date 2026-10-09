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
| TDRP-R9-COMPOSER-STASH-EXCHANGE-001 | critical | ORA-TDRP-R9-COMPOSER-STASH-EXCHANGE-001 | INV-TDRP-R9-COMPOSER-STASH-ATOMIC-001; INV-TDRP-R9-COMPOSER-STASH-SCOPE-001 | CONTRACT-TDRP-COMPOSER-STASH-EXCHANGE-001; TEMP-TDRP-COMPOSER-STASH-RESTORE-001; FAULT-TDRP-COMPOSER-STASH-INVALID-RESTORE-001 | GitHub Actions frontend contract suite | UI-TDRP-COMPOSER-STASH-SWAP-001; A11Y-TDRP-COMPOSER-STASH-SHORTCUT-001 | SEC-TDRP-COMPOSER-STASH-SCOPE-001 | ComposerDraftStateStore keyed exchange + canonical ConversationComposer | REG-TDRP-COMPOSER-STASH-DRAFT-LOSS-001 | current-head run pending | pending-independent-review | IMPLEMENTED-PENDING-CI |
| TDRP-R9-LIVE-SOURCE-AUTHORITY-001 | critical | ORA-TDRP-R9-LIVE-TREE-001 | INV-TDRP-R9-ROOT-CENSUS-001; INV-TDRP-R9-RECURSIVE-CENSUS-001; INV-TDRP-R9-ACQUISITION-FAIL-CLOSED-001 | static machine-authority arithmetic | current-head source-authority workflow | pending exact-head recursive/acquisition Actions | — | source/build provenance | REG-TDRP-R9-STALE-BASELINE-001 | workflow:pending-current-head | pending-independent-review | MAPPED |
| TDRP-R9-SEARCH-ROW-REPLACEMENT-001 | high | ORA-TDRP-SEARCH-ROW-REPLACEMENT-001 | INV-TDRP-SEARCH-CANONICAL-ID-001; INV-TDRP-SEARCH-LATEST-ROW-001 | PROP-TDRP-SEARCH-ROW-REPLACEMENT-001 | CONTRACT-TDRP-CANONICAL-SEARCH-001 | E2E-TDRP-CANONICAL-SEARCH-001 | UI-TDRP-CANONICAL-SEARCH-RESULT-001 | SEC-TDRP-SEARCH-AUTHORIZED-INPUT-001 | REG-TDRP-SEARCH-DUPLICATE-PARTICIPANT-001 | commit:674d60bbc1e9059e6160cbfd321fbfff47612fff; workflow:37597016700@d99de586173377e4618fc1956bb2f30be8f1db11 | pending-independent-review | IMPLEMENTED |
| TDRP-R9-EXTERNAL-URL-AUTH-CONTEXT-001 | critical | ORA-TDRP-URL-AUTH-ACCEPTED-CONTEXT-001 | INV-TDRP-URL-AUTH-STRIP-FOREIGN-001; INV-TDRP-URL-AUTH-EXACT-ORIGIN-001; INV-TDRP-URL-AUTH-NO-RENDERER-UPGRADE-001 | PROP-TDRP-URL-AUTH-ENCODING-001 | CONTRACT-TDRP-URL-AUTH-BOUNDARY-001; INT-TDRP-FABUSHI-ACCOUNT-EXTERNAL-OPEN-001 | E2E-TDRP-EXTERNAL-URL-AUTH-001; TEMP-TDRP-URL-AUTH-EPHEMERAL-CONTEXT-001 | — | SEC-TDRP-URL-AUTH-STRIP-001; SEC-TDRP-URL-AUTH-EXACT-ORIGIN-001; SEC-TDRP-URL-AUTH-PRIVILEGE-001 | REG-TDRP-URL-AUTH-TOKEN-SMUGGLING-001 | commit:aa9df7fef3454a81aacf489fecae984099cdeab4; workflow:pending-current-head | pending-independent-review | IMPLEMENTED |
| TDRP-R9-SHARE-FORWARD-PRIVACY-001 | critical | ORA-TDRP-SHARE-FORWARD-PRIVACY-001 | INV-TDRP-FORWARD-CAPTION-IMPLIES-NO-SENDER-001; INV-TDRP-FORWARD-PRIVACY-NATIVE-001; INV-TDRP-FORWARD-PRIVACY-IDEMPOTENT-001 | PROP-TDRP-SHARE-FORWARD-PRIVACY-001 | CONTRACT-TDRP-SHARE-FORWARD-PRIVACY-001; INT-TDRP-SHARE-FORWARD-PRIVACY-001 | TEMP-TDRP-SHARE-FORWARD-PRIVACY-001; FAULT-TDRP-SHARE-FORWARD-PRIVACY-001 | pending shipping Forward E2E | — | REG-TDRP-FORWARD-PRIVACY-001 | workflow:pending-current-head | pending-independent-review | IMPLEMENTED |
| TDRP-R9-SHARE-RECIPIENT-ELIGIBILITY-001 | critical | ORA-TDRP-SHARE-RECIPIENT-ELIGIBILITY-001 | INV-TDRP-RECIPIENT-AUTH-BEFORE-EXPOSURE-001; INV-TDRP-RECIPIENT-MEDIA-POLL-ELIGIBILITY-001; INV-TDRP-RECIPIENT-CHANNEL-POSTING-001 | PROP-TDRP-SHARE-RECIPIENT-ELIGIBILITY-001 | CONTRACT-TDRP-SHARE-RECIPIENT-ELIGIBILITY-001; INT-TDRP-SHARE-RECIPIENT-ELIGIBILITY-001 | TEMP-TDRP-SHARE-RECIPIENT-POLICY-REFRESH-001; FAULT-TDRP-SHARE-RECIPIENT-ELIGIBILITY-001 | pending shipping recipient-picker E2E | — | REG-TDRP-RECIPIENT-PREEXPOSURE-001 | workflow:pending-current-head | pending-independent-review | IMPLEMENTED |
| TDRP-R9-HISTORY-STATE-CONTRACT-001 | high | ORA-TDRP-R9-HISTORY-STATE-CONTRACT-001 | INV-TDRP-R9-HISTORY-STATE-CONTRACT-001-CANONICAL | pending | pending | pending child/message lifecycle integration coverage | pending Conversation/Transcript/Composer projection coverage | pending Search/resource cleanup coverage | pending Bot + messaging regression | evidence:pending-current-head | pending-independent-review | MAPPED |
| TDRP-R9-HISTORY-LIFECYCLE-001 | critical | ORA-TDRP-R9-HISTORY-LIFECYCLE-001 | INV-TDRP-R9-HISTORY-LIFECYCLE-001-CANONICAL | pending | pending | pending deletion/destroyed ordering + fault/temporal coverage | pending exact child cleanup E2E | pending authorization/Search/resource cleanup coverage | pending Bot + messaging regression | evidence:pending-current-head | pending-independent-review | MAPPED |
| TDRP-R9-DATA-CHANGES-CONTRACT-001 | high | ORA-TDRP-R9-DATA-CHANGES-CONTRACT-001 | INV-TDRP-R9-DATA-CHANGES-CONTRACT-001-CANONICAL | pending | pending | pending destruction/cleanup integration and temporal coverage | pending canonical Conversation/Composer projection coverage | pending resource/security cleanup coverage | pending Bot + messaging regression | evidence:pending-current-head | pending-independent-review | MAPPED |
| TDRP-R9-DATA-CHANGES-LIFECYCLE-001 | high | ORA-TDRP-R9-DATA-CHANGES-LIFECYCLE-001 | INV-TDRP-R9-DATA-CHANGES-LIFECYCLE-001-CANONICAL | pending | pending | pending realtime/coalesced/destroyed sequencing coverage | pending child lifecycle projection coverage | pending stale-event/resource cleanup coverage | pending Bot + messaging regression | evidence:pending-current-head | pending-independent-review | MAPPED |
| TDRP-R9-DATA-TYPES-VALUE-CONTRACT-001 | high | ORA-TDRP-R9-DATA-TYPES-VALUE-CONTRACT-001 | INV-TDRP-R9-DATA-TYPES-VALUE-CONTRACT-001-CANONICAL | pending | pending | pending forward/message value-contract integration coverage | pending Composer/Conversation projection coverage | pending resource/cache/security applicability coverage | pending Bot + messaging regression | evidence:pending-current-head | pending-independent-review | MAPPED |
| TDRP-R9-IV-VIEW-WIDGET-LIFECYCLE-001 | high | ORA-TDRP-IV-VIEW-WIDGET-001 | INV-TDRP-IV-VIEW-REPLACE-001; INV-TDRP-IV-SELECTION-ACTION-001; INV-TDRP-IV-COPY-TYPED-001; INV-TDRP-IV-SCROLL-LIFECYCLE-001; INV-TDRP-IV-VISIBLE-WORK-001; INV-TDRP-IV-MEDIA-ARBITRATION-001 | UNIT-TDRP-IV-VIEW-HIGHLIGHT-RUNTIME-001 | CONTRACT-TDRP-IV-VIEW-HIGHLIGHT-DISPOSAL-001; CONTRACT-TDRP-IV-VIEW-POINTER-ACTIVATION-FENCE-001; CONTRACT-TDRP-IV-VIEW-MEDIA-POINTER-RELEASE-001; CONTRACT-TDRP-IV-VIEW-CODE-COPY-SANITIZED-001 | pending broader drag/gesture/hidden/touch/scroll temporal matrix | pending packaged keyboard/a11y/visual matrix | pending UI performance/soak and security review | pending Bot + transcript regression breadth | workflow:pending-current-head | pending-independent-review | MAPPED |
| TDRP-R9-IV-ARTICLE-TEXT-LINK-001 | high | ORA-TDRP-IV-ARTICLE-TEXT-001 | INV-TDRP-IV-LINK-TYPED-001 | — | CONTRACT-TDRP-IV-PREPARED-LINK-EXTERNAL-COPY-001; INT-TDRP-FABUSHI-ACCOUNT-EXTERNAL-OPEN-001 | pending remaining link-kind/replacement temporal matrix | pending packaged context-menu keyboard/a11y/visual matrix | pending full external-link policy negative matrix | pending Bot + transcript typed-link regression breadth | workflow:pending-current-head | pending-independent-review | MAPPED |
| TDRP-R9-IV-ARTICLE-CONTRACT-001 | high | ORA-TDRP-IV-ARTICLE-001 | INV-TDRP-IV-ARTICLE-PATCH-001; INV-TDRP-IV-ARTICLE-SCROLL-001; INV-TDRP-IV-ARTICLE-RUNTIME-001; INV-TDRP-IV-ARTICLE-HEAVY-001 | UNIT-TDRP-IV-ARTICLE-SCROLL-CLAMP-001; UNIT-TDRP-IV-ARTICLE-MEDIA-VISIBILITY-001; UNIT-TDRP-IV-ARTICLE-PROJECTION-RECONCILE-001; UNIT-TDRP-IV-ARTICLE-PROJECTION-SEQUENCE-001 | CONTRACT-TDRP-IV-ARTICLE-SCROLL-CONTINUITY-001; CONTRACT-TDRP-IV-ARTICLE-MEDIA-LIFECYCLE-001; CONTRACT-TDRP-IV-ARTICLE-PARTIAL-FALLBACK-001 | pending full article replacement/reload temporal coverage | pending packaged keyboard/a11y/visual matrix | pending UI performance/soak and security review | pending Bot + transcript regression breadth | workflow:pending-current-head | pending-independent-review | MAPPED |
| TDRP-R9-IV-ARTICLE-CONTENT-LIFECYCLE-001 | high | ORA-TDRP-IV-ARTICLE-001 | INV-TDRP-IV-ARTICLE-PATCH-001; INV-TDRP-IV-ARTICLE-SCROLL-001; INV-TDRP-IV-ARTICLE-RUNTIME-001; INV-TDRP-IV-ARTICLE-HEAVY-001 | UNIT-TDRP-IV-ARTICLE-SCROLL-CLAMP-001; UNIT-TDRP-IV-ARTICLE-MEDIA-VISIBILITY-001; UNIT-TDRP-IV-ARTICLE-PROJECTION-RECONCILE-001; UNIT-TDRP-IV-ARTICLE-PROJECTION-SEQUENCE-001 | CONTRACT-TDRP-IV-ARTICLE-SCROLL-CONTINUITY-001; CONTRACT-TDRP-IV-ARTICLE-MEDIA-LIFECYCLE-001; CONTRACT-TDRP-IV-ARTICLE-PARTIAL-FALLBACK-001 | pending arbitrary replacement/reload temporal coverage | pending packaged keyboard/a11y/visual matrix | pending UI performance/soak and security review | pending Bot + transcript regression breadth | workflow:pending-current-head | pending-independent-review | MAPPED |
| TDRP-R9-IV-ARTICLE-INTERACTION-001 | high | ORA-TDRP-IV-ARTICLE-001 | INV-TDRP-IV-ARTICLE-SCROLL-001; INV-TDRP-IV-ARTICLE-HIT-001; INV-TDRP-IV-ARTICLE-REVEAL-001; INV-TDRP-IV-ARTICLE-RUNTIME-001 | UNIT-TDRP-IV-ARTICLE-SCROLL-CLAMP-001; UNIT-TDRP-IV-ARTICLE-HIDDEN-SEARCH-001; UNIT-TDRP-IV-ARTICLE-HORIZONTAL-POINTER-LIFECYCLE-001; UNIT-TDRP-IV-ARTICLE-WHEEL-DIRECTION-LOCK-001; UNIT-TDRP-IV-ARTICLE-WHEEL-DELTA-NORMALIZATION-001 | CONTRACT-TDRP-IV-ARTICLE-SCROLL-CONTINUITY-001; CONTRACT-TDRP-IV-ARTICLE-HIDDEN-REVEAL-001; CONTRACT-TDRP-IV-ARTICLE-HORIZONTAL-POINTER-BALANCE-001; CONTRACT-TDRP-IV-ARTICLE-WHEEL-DIRECTION-LOCK-001 | pending quote/details ancestor, relayout reveal and mouse-scrollbar packaged temporal coverage | pending packaged keyboard/a11y/visual matrix | pending UI performance/soak and security review | pending Bot + transcript regression breadth | workflow:pending-current-head | pending-independent-review | MAPPED |

### TDRP-R9-IV-VIEW-WIDGET-LIFECYCLE-001 oracle

- `ORA-TDRP-IV-VIEW-WIDGET-001`: derived pointer, search-highlight, media and repaint work must resolve only against the current canonical conversation surface and cannot settle into a replaced or disposed owner.
- `INV-TDRP-IV-VISIBLE-WORK-001`: Find-in-Chat owns at most one pending highlight refresh; superseding navigation cancels the previous frame, transcript-container replacement invalidates it, settlement reads the latest controller/container references, and real unmount disposal prevents late CSS Highlight ranges from retaining detached DOM.
- `INV-TDRP-IV-VIEW-REPLACE-001`: transcript pointer intent for canonical links and buttons is bound to the current entries generation plus exact action element; entries replacement, pointer/touch cancellation, pointer leave, movement beyond the bounded drag threshold, focus-chain exit, window blur, or page hiding retires that intent, and a later pointer click is rejected in capture unless both identities still match. Pointer-originated canonical transcript links/buttons are also rejected while the browser exposes a non-empty text selection, preventing selection settlement from becoming an action; keyboard activation remains independent. This includes transcript attachment/media buttons and disclosure toggles.
- `INV-TDRP-IV-MEDIA-ARBITRATION-001`: canonical MediaViewer replacement and unmount release any active pointer capture before retiring pointer ownership; implicit `lostpointercapture` clears the matching owner; zoomed pan capture ignores interactive descendants so close/navigation/filmstrip controls keep their own activation lifecycle.
- Current cases: `UNIT-TDRP-IV-VIEW-HIGHLIGHT-RUNTIME-001`, `UNIT-TDRP-IV-VIEW-CONTROL-WHEEL-ZOOM-001`, `UNIT-TDRP-IV-ARTICLE-HORIZONTAL-POINTER-LIFECYCLE-001`, `UNIT-TDRP-IV-ARTICLE-WHEEL-DIRECTION-LOCK-001`, `UNIT-TDRP-IV-ARTICLE-WHEEL-DELTA-NORMALIZATION-001`, `CONTRACT-TDRP-IV-VIEW-HIGHLIGHT-DISPOSAL-001`, `CONTRACT-TDRP-IV-VIEW-POINTER-ACTIVATION-FENCE-001`, `CONTRACT-TDRP-IV-VIEW-MEDIA-POINTER-RELEASE-001`, `CONTRACT-TDRP-IV-VIEW-CODE-COPY-SANITIZED-001`, and `CONTRACT-TDRP-IV-ARTICLE-HORIZONTAL-POINTER-BALANCE-001`, `CONTRACT-TDRP-IV-ARTICLE-WHEEL-DIRECTION-LOCK-001`, and `CONTRACT-TDRP-IV-VIEW-STATIC-DATE-WORK-001`.
- Dossier: `projects/telegram-desktop-rust/dossiers/iv-markdown-view-widget-complete-read.md`.
- Current verdict: `MAPPED`; bounded Highlight refresh/disposal, transcript link/button replacement fencing, focus/window/visibility/touch-pointer cancellation, selection-versus-pointer-action arbitration, capability-minimal code-copy sanitization, and MediaViewer pointer-capture retirement, and deterministic Control-wheel zoom accumulation, touch-like horizontal pointer direction/capture balancing, browser wheel-burst direction locking with line/page delta normalization, and the static-timestamp no-background-timer replacement are present, while exact-head execution and mouse-scrollbar packaged temporal parity, broader MediaViewer pan/zoom gesture breadth and packaged lifecycle evidence remain pending.

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


### TDRP Revision 9 resource-read requirements 279-338

| requirement_id | risk | oracle_ids | invariant_ids | unit/property | contract/integration | e2e/temporal | ui/visual/a11y | perf/security | regression_ids | evidence_ids | reviewer | verdict |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| TDRP-R9-EMOJI-SET-RESOURCES-001 | high | ORA-TDRP-R9-EMOJI-SET-RESOURCES-001 | INV-TDRP-R9-EMOJI-SINGLE-OWNER-001; INV-TDRP-R9-EMOJI-MESSAGE-IDENTITY-001 | pending | pending canonical emoji owner binding | pending | pending Settings/Appearance + Composer/Transcript evidence | provenance/security pending | pending | run 37842440135 job 113534924423 artifact 11578775634 + dossier resource-emoji-export-279-338-complete-read.md | pending-independent-review | MAPPED-SOURCE-READ-PRODUCTION-OPEN |
| TDRP-R9-DATA-EXPORT-HTML-RESOURCES-001 | high | ORA-TDRP-R9-DATA-EXPORT-HTML-RESOURCES-001 | INV-TDRP-R9-EXPORT-REPRESENTATION-NOT-TRUTH-001 | pending | pending canonical data-export owner binding | pending | pending exported HTML accessibility/visual evidence | offline-script/link security pending | pending | run 37842440135 job 113534924423 artifact 11578775634 + exact full reads 291/336 + dossier | pending-independent-review | MAPPED-SOURCE-READ-PRODUCTION-OPEN |
| TDRP-R9-RICH-EXPORT-RESOURCES-001 | high | ORA-TDRP-R9-RICH-EXPORT-RESOURCES-001 | INV-TDRP-R9-RICH-EXPORT-NO-SECOND-RENDERER-001 | pending | pending canonical rich export owner binding | pending | pending rich export/clipboard evidence | escaping/script security pending | pending | exact full reads 337/338 + qrc/iv_rich_message_html_export consumer trace + dossier | pending-independent-review | MAPPED-SOURCE-READ-PRODUCTION-OPEN |

- `ORA-TDRP-R9-EMOJI-SET-RESOURCES-001`: accepted resource bytes 279-290 map to one emoji rendering/set-selection responsibility; set selection must preserve one canonical emoji identity and may not change canonical message text/entity identity.
- `INV-TDRP-R9-EMOJI-SINGLE-OWNER-001`: Fabushi may not add a Telegram-shaped emoji picker/set root when canonical Composer/Transcript and Settings/Appearance owners can express the responsibility.
- `INV-TDRP-R9-EMOJI-MESSAGE-IDENTITY-001`: visual-set switching never rewrites persisted message/entity identity.
- `ORA-TDRP-R9-DATA-EXPORT-HTML-RESOURCES-001`: 291-336 are consumed by export.qrc/export_output_html.cpp to create self-contained offline representation; the export is never canonical live conversation state.
- `INV-TDRP-R9-EXPORT-REPRESENTATION-NOT-TRUTH-001`: HTML/CSS/JS/image export artifacts cannot become Conversation/Message truth or a second message store.
- `ORA-TDRP-R9-RICH-EXPORT-RESOURCES-001`: 337-338 are consumed by export_rich.qrc/iv_rich_message_html_export.cpp as escaped, offline rich-content presentation with conditional slideshow JS.
- `INV-TDRP-R9-RICH-EXPORT-NO-SECOND-RENDERER-001`: rich export may reuse canonical content semantics but cannot become a second live TranscriptEntry renderer/state owner.
- Dossier: `projects/telegram-desktop-rust/dossiers/resource-emoji-export-279-338-complete-read.md`.
- These rows grant source-read/traceability credit only. They do not grant Fabushi production, test, release, or independent-acceptance credit.

### TDRP Revision 9 resource-read requirements 339-368

| requirement_id | risk | oracle_ids | invariant_ids | unit/property | contract/integration | e2e/temporal | ui/visual/a11y | perf/security | regression_ids | evidence_ids | reviewer | verdict |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| TDRP-R9-ARCHIVE-AVATAR-ASSET-001 | medium | ORA-TDRP-R9-ARCHIVE-AVATAR-ASSET-001 | INV-TDRP-R9-ARCHIVE-STATE-NOT-PIXELS-001 | pending | pending canonical ConversationRow/Avatar evidence | pending | pending archive-row visual/a11y | provenance pending | pending | run 37846454160 job 113548896289 artifact 11579099694 + dossier resource-icons-339-368-complete-read.md | pending-independent-review | MAPPED-SOURCE-READ-PRODUCTION-OPEN |
| TDRP-R9-SELECT-ARROW-ASSET-001 | medium | ORA-TDRP-R9-SELECT-ARROW-ASSET-001 | INV-TDRP-R9-SELECTION-STATE-NOT-ICON-001 | pending | pending canonical Menu/Popover/Picker evidence | pending | pending schedule/select visual/a11y | provenance pending | pending | same exact artifact + dossier | pending-independent-review | MAPPED-SOURCE-READ-PRODUCTION-OPEN |
| TDRP-R9-BOOST-GIFT-ASSETS-001 | high | ORA-TDRP-R9-BOOST-GIFT-ASSETS-001 | INV-TDRP-R9-ENTITLEMENT-TRUTH-SINGLE-OWNER-001 | pending | pending canonical commerce/entitlement owner binding | pending | pending Badge/Status/ListRow evidence | rights/provenance + claimability security pending | pending | same exact artifact + consumer traces + dossier | pending-independent-review | MAPPED-SOURCE-READ-PRODUCTION-OPEN |
| TDRP-R9-BUBBLE-TAIL-ASSET-001 | medium | ORA-TDRP-R9-BUBBLE-TAIL-ASSET-001 | INV-TDRP-R9-TRANSCRIPT-STATE-NOT-DECORATION-001 | pending | pending canonical TranscriptEntry visual contract | pending | pending direction/selection visual/a11y | provenance pending | pending | same exact artifact + chat_style consumer + dossier | pending-independent-review | MAPPED-SOURCE-READ-PRODUCTION-OPEN |
| TDRP-R9-CALENDAR-DIRECTION-ASSET-001 | medium | ORA-TDRP-R9-CALENDAR-DIRECTION-ASSET-001 | INV-TDRP-R9-DATE-STATE-NOT-ICON-001 | pending | pending canonical Picker/Popover/Menu navigation evidence | pending | pending previous/next/disabled/RTL/a11y | provenance pending | pending | same exact artifact + boxes/iv/calls style consumers + dossier | pending-independent-review | MAPPED-SOURCE-READ-PRODUCTION-OPEN |

- `ORA-TDRP-R9-ARCHIVE-AVATAR-ASSET-001`: orders 339-341 are scale variants consumed by `dialogsArchiveUserpic`; archive membership remains canonical conversation state.
- `INV-TDRP-R9-ARCHIVE-STATE-NOT-PIXELS-001`: placeholder pixels cannot become archive membership or conversation identity.
- `ORA-TDRP-R9-SELECT-ARROW-ASSET-001`: orders 342-344 are shared dropdown/selector affordances used by schedule-repeat and AI selector styles.
- `INV-TDRP-R9-SELECTION-STATE-NOT-ICON-001`: selection/open/schedule truth remains with canonical control owners.
- `ORA-TDRP-R9-BOOST-GIFT-ASSETS-001`: orders 345-362 render boost/gift/giveaway status across statistics/chat/giveaway styles.
- `INV-TDRP-R9-ENTITLEMENT-TRUTH-SINGLE-OWNER-001`: visual status cannot own entitlement, claimability or purchase state.
- `ORA-TDRP-R9-BUBBLE-TAIL-ASSET-001`: orders 363-365 are derived incoming/outgoing/selected message-bubble decoration.
- `INV-TDRP-R9-TRANSCRIPT-STATE-NOT-DECORATION-001`: `TranscriptEntry`/message state remains authoritative; decoration is derived.
- `ORA-TDRP-R9-CALENDAR-DIRECTION-ASSET-001`: orders 366-368 are directional date/search/calendar affordances used by boxes/IV/call styles.
- `INV-TDRP-R9-DATE-STATE-NOT-ICON-001`: selected date, search position and call/message schedule remain canonical state, not icon state.
- Dossier: `projects/telegram-desktop-rust/dossiers/resource-icons-339-368-complete-read.md`.
- These rows grant source-read/traceability credit only. They do not grant Fabushi production, test, release, or independent-acceptance credit.

### TDRP Revision 9 call-resource requirements 369-430

| requirement_id | risk | oracle_ids | invariant_ids | contract/integration | e2e/temporal | ui/visual/a11y | evidence_ids | verdict |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| TDRP-R9-CALL-CORE-ASSETS-001 | high | ORA-TDRP-R9-CALL-CORE-ASSETS-001 | INV-TDRP-R9-CALL-STATE-NOT-ASSET-001 | existing HumanCallControls/CallSession subset; exact-head evidence required | pending | pending | run 37848024391 job 113553686091 artifact 11580506869 + dossier | MAPPED-SOURCE-READ-PRODUCTION-PARTIAL |
| TDRP-R9-GROUP-CALL-ASSETS-001 | critical | ORA-TDRP-R9-GROUP-CALL-ASSETS-001 | INV-TDRP-R9-ONE-CALL-DOMAIN-OWNER-001 | participant/message/recording/rating gaps open | pending | pending | same artifact + calls.style/group consumers + dossier | MAPPED-SOURCE-READ-PRODUCTION-GAP |

- `INV-TDRP-R9-CALL-STATE-NOT-ASSET-001`: raster resources are derived call presentation and never own call/session/media state.
- `INV-TDRP-R9-ONE-CALL-DOMAIN-OWNER-001`: group-call additions must extend canonical HumanCallControls/CallSession/server call domain rather than introduce Telegram-shaped duplicate owners.
- Current two-party call support is not accepted as evidence for group-call completion.
- Dossier: `projects/telegram-desktop-rust/dossiers/resource-calls-369-430-complete-read.md`.

### TDRP Revision 9 source-read requirements 431-500

| requirement_id | risk | invariant | evidence | production status |
| --- | --- | --- | --- | --- |
| TDRP-R9-CALL-GROUP-ASSETS-431-497 | critical | all participant/media/group-call UI derives from the single canonical Human call domain | run 37849072863 / job 113557467926 / artifact 11580952146 + dossier | PARTIAL; canonical server remains 1:1, group responsibilities open |
| TDRP-R9-CALL-DIALOGS-USER-432-434 | medium | do not infer product responsibility from an unproven asset filename | same artifact + negative named-consumer search | SOURCE READ; CONSUMER REACHABILITY OPEN |
| TDRP-R9-PAID-CROWN-435 | high | visual crown never owns entitlement/payment truth | same artifact + premium/paid-reaction consumer mapping | MAPPED; production evidence open |
| TDRP-R9-AI-COMPOSE-ASSETS-498-500 | high | AI capability remains with canonical Composer/AI runtime owners | same artifact + chat/settings/editor consumer mapping | MAPPED; visual/a11y evidence open |

Dossier: `projects/telegram-desktop-rust/dossiers/resource-call-ai-icons-431-500-complete-read.md`.

### TDRP Revision 9 source-read requirements 501-600

| requirement | risk | invariant | exact source evidence | production status |
| --- | --- | --- | --- | --- |
| TDRP-R9-COMPOSER-501-575 | high | all draft/edit/reply/Bot/media actions reuse canonical owners | run 37849904823 / job 113560126953 / artifact 11580638761 + dossier | mixed mapped/partial; reachability-open rows explicit |
| TDRP-R9-SCHEDULE-SILENT-576-596 | critical | one Composer/submission/Host/server scheduler path; no recipient delivery before due; idempotency includes silent/scheduled time | same artifact + server PR #2746 + desktop Host/RPC/Composer contracts | IMPLEMENTED; exact-head/release acceptance still pending |
| TDRP-R9-CONSUMER-REACHABILITY-551-553-558-563-600 | medium | filenames never substitute for consumer proof | source-consumer-reachability-501-600.txt | OPEN |

Dossier: `projects/telegram-desktop-rust/dossiers/resource-composer-chat-501-600-complete-read.md`.

### TDRP Revision 9 build provenance snapshot resolution

| requirement | evidence | verdict |
| --- | --- | --- |
| TDRP-R9-PROVENANCE-CCACHE-001 | signed v4.13.6 tag -> c6f36725… + release asset sha256:508b2a… | SNAPSHOT RESOLVED; SHIPPING URL/DIGEST VERIFICATION OPEN |
| TDRP-R9-PROVENANCE-XZ-001 | verified v5.8.4 tag -> d3e650e6… | SNAPSHOT RESOLVED; TAG-BASED ACQUISITION OPEN |
| TDRP-R9-PROVENANCE-OPENAL-001 | coreaudio_device_uid -> c2eab43d… | SNAPSHOT RESOLVED; MOVABLE BRANCH ACQUISITION OPEN |

Dossier: `projects/telegram-desktop-rust/dossiers/build-provenance-resolution-ccache-xz-openal.md`.


### TDRP Revision 9 platform source-read requirements 5411-5475

| requirement | risk | invariant | exact source evidence | production status |
| --- | --- | --- | --- | --- |
| TDRP-R9-PLATFORM-FILE-SECURITY-5411-5414-5465-5467 | high | bookmark/file-dialog/Open-With/external-launch state stays behind one canonical file/security owner; nested native event-loop behavior stays fenced | accepted blobs in corrected source-dispositions 5411-5475 + attestation manifest | MAPPED-SOURCE-READ-PRODUCTION-OPEN |
| TDRP-R9-PLATFORM-LOCATION-5464 | high | exact location/reverse-geocode remains permission-aware and owned by the canonical location capability | same corrected shard + dossier | MAPPED-SOURCE-READ-PRODUCTION-OPEN |
| TDRP-R9-PLATFORM-INTEGRATION-WINDOW-5415-5435-5463-5471-5473-5475 | high | native menus/windows/integration/lifecycle project through canonical app-menu/window/theme/settings owners; no second UI/runtime root | same corrected shard + dossier | MAPPED-SOURCE-READ-PRODUCTION-OPEN |
| TDRP-R9-NATIVE-NOTIFICATION-POLICY-5426-5429-5472 | critical | native notification policy/actions must authorize exact conversation/message scope and cleanup exact native notifications | same corrected shard + dossier | MAPPED-SOURCE-READ-PRODUCTION-GAP |
| TDRP-R9-MAC-TOUCHBAR-OCR-TRANSLATION-TRAY-5436-5459 | medium | platform projections cannot own Composer/Conversation/Media/translation truth | same corrected shard + dossier | MAPPED-SOURCE-READ-PRODUCTION-OPEN |
| TDRP-R9-WALLET-PASSKEY-5460-5462 | critical | device auth/Secure Enclave/passkey operations remain fail-closed under canonical wallet/account/auth security owners | same corrected shard + dossier | MAPPED-SOURCE-READ-PRODUCTION-GAP |
| TDRP-R9-UPDATER-LAUNCHER-5419-5420-5470 | critical | updater/launcher handoff may proceed only after authenticated package/trusted staging/preflight/privilege/post-install verification and recoverable failure semantics | corrected macOS/shared-launcher source evidence; canonical updater hardening exists but Windows source-read credit begins only at order 5481+ | MAPPED-SOURCE-READ-PRODUCTION-PARTIAL |
| TDRP-R9-MEDIAVIEW-OVERLAY-5473-5474 | medium | overlay title controls/hit-testing/pointer routing remain canonical MediaViewer/window-chrome interactions with a11y/responsive state | same corrected shard + dossier | MAPPED-SOURCE-READ-PRODUCTION-OPEN |

- INV-TDRP-R9-NATIVE-NOTIFICATION-SCOPE-001: native reply/action callbacks must bind to the exact authorized conversation/message identity and cannot become a second message owner.
- INV-TDRP-R9-PROTECTED-UPDATER-001: an updater matrix that only proves update gating or parent handoff does not close trusted-path, authenticated-package, private-staging, privilege, post-install-digest, or failure-recovery responsibilities.
- INV-TDRP-R9-PASSKEY-SINGLE-AUTH-OWNER-001: native passkey/security-key APIs are adapters to canonical account/auth state, never a second credential truth store.
- Dossier: `projects/telegram-desktop-rust/dossiers/platform-mac-windows-5411-5475-complete-read.md` (filename retained for provenance; corrected content records Windows start at order 5481).
- These rows grant source-read/traceability credit only. They do not grant production, test, release, or independent-acceptance credit.


### TDRP Revision 9 platform source-read requirements 5476-5500

| requirement | risk | invariant | exact source evidence | production status |
| --- | --- | --- | --- | --- |
| TDRP-R9-SHARED-CAPABILITIES-5476-5480 | high | OCR/translation/tray/WebAuthn/window-title adapters report truthful capability and derive all product state from canonical owners | accepted blobs in source-dispositions 5476-5500 + attestation manifest | MAPPED-SOURCE-READ-PRODUCTION-OPEN |
| TDRP-R9-WIN-LOCATION-5481-5482 | high | exact location is permission/failure aware; unavailable reverse-geocode is never fabricated | same shard + dossier | MAPPED-SOURCE-READ-PRODUCTION-OPEN |
| TDRP-R9-WIN-FILE-DIALOG-5483-5484 | high | Open-With/external launch/Zone.Identifier/dialog persistence remain behind canonical file/security policy and native resources are bounded | same shard + dossier | MAPPED-SOURCE-READ-PRODUCTION-OPEN |
| TDRP-R9-WIN-LIFECYCLE-5485-5486 | high | native events stay fenced; sleep/resume is debounced and screen-lock/time/settings changes update one canonical lifecycle state | same shard + dossier | MAPPED-SOURCE-READ-PRODUCTION-OPEN |
| TDRP-R9-WIN-UPDATER-HANDOFF-5487-5488 | critical | privilege and parent-process handoff may only follow authenticated staging and must resist PID reuse/modal test deadlock | same shard + current canonical updater sources | MAPPED-SOURCE-READ-PRODUCTION-PARTIAL |
| TDRP-R9-WIN-WINDOW-PRIVACY-5489-5490 | critical | passcode/protected content cannot leak through DWM preview/capture and native window state remains a projection | same shard + canonical window-chrome source comparison | MAPPED-SOURCE-READ-PRODUCTION-GAP |
| TDRP-R9-WIN-NOTIFICATION-ACTIONS-5491-5492 | critical | reply/mark/open activation requires exact active session/peer/topic-or-sublist/message scope and exact cleanup | same shard + canonical SandOsNotificationManager comparison | MAPPED-SOURCE-READ-PRODUCTION-GAP |
| TDRP-R9-WIN-PLATFORM-SECURITY-5493-5499 | high | overlay/tray/autostart/permission/capture/theme/capability state remains source-neutral and unsupported native OCR/translation is reported unavailable | same shard + dossier | MAPPED-SOURCE-READ-PRODUCTION-OPEN |
| TDRP-R9-WIN-HELLO-WALLET-5500 | critical | only hardware-attested TPM-backed credential flow may derive wrap keys; secret buffers and error classes fail closed | same shard + dossier | MAPPED-SOURCE-READ-PRODUCTION-GAP |

- `INV-TDRP-R9-WIN-NOTIFICATION-ACTION-SCOPE-001`: a native toast action is not authorization; the exact active canonical conversation/message scope must still exist before reply/mark/open.
- `INV-TDRP-R9-WIN-LOCKED-PREVIEW-PRIVACY-001`: minimized/taskbar/DWM/capture surfaces must not reveal protected conversation content while the canonical lock/privacy state requires redaction.
- `INV-TDRP-R9-WIN-POWER-DEBOUNCE-001`: duplicate/out-of-order suspend/resume broadcasts cannot create duplicate lifecycle transitions or stale online state.
- `INV-TDRP-R9-WIN-HELLO-HARDWARE-001`: Windows Hello support alone is insufficient; TPM presence, hardware attestation, exact credential payload semantics and fail-closed unwrap classification are required.
- `INV-TDRP-R9-WIN-CAPABILITY-ABSENCE-001`: Windows-native OCR/translation absence must remain observable; a source-neutral replacement may satisfy product capability but cannot masquerade as the unavailable native adapter.
- Dossier: `projects/telegram-desktop-rust/dossiers/platform-shared-windows-5476-5500-complete-read.md`.
- These rows grant source-read/traceability credit only. They do not grant production, test, release, or independent-acceptance credit.


### TDRP Revision 9 source-read requirements 5501-5529

| requirement | risk | invariant | exact source evidence | production status |
| --- | --- | --- | --- | --- |
| TDRP-R9-WIN-AUTH-IDENTITY-5501-5504 | critical | passkey/wallet/shell identity is exact, fail-closed and owned by canonical auth/packaging owners | source-dispositions 5501-5529 + manifest + dossier | MAPPED-SOURCE-READ-PRODUCTION-GAP |
| TDRP-R9-WIN-STARTUP-NATIVE-5505-5509 | high | OS policy/capability absence is observable and native APIs remain bounded adapters | same shard + dossier | MAPPED-SOURCE-READ-PRODUCTION-OPEN |
| TDRP-R9-WIN-TASKBAR-TOAST-5510-5514 | critical | player/notification state remains canonical; native taskbar/toast input is derived, fenced and authorized | same shard + dossier | MAPPED-SOURCE-READ-PRODUCTION-GAP |
| TDRP-R9-POLL-LINK-MEDIA-5515-5520 | critical | one canonical poll/survey owner controls option links/media; upload callbacks are token-fenced and cancel-safe | same shard + dossier | MAPPED-SOURCE-READ-PRODUCTION-GAP |
| TDRP-R9-PROFILE-PRIMITIVES-5521-5527 | medium | profile presentation reuses canonical ProfileSection/design-system and never owns profile truth | same shard + dossier | MAPPED-SOURCE-READ-PRODUCTION-OPEN |
| TDRP-R9-SETTINGS-STATE-5528-5529 | high | settings/persistence/security/lifecycle truth is typed and single-owner; passcode backoff and filesystem permissions fail safe | same shard + dossier | MAPPED-SOURCE-READ-PRODUCTION-OPEN |

- `INV-TDRP-R9-POLL-UPLOAD-TOKEN-001`: stale preparation/upload completion cannot replace media selected after its token was issued.
- `INV-TDRP-R9-WIN-TASKBAR-DERIVED-001`: taskbar controls and icons derive from the canonical media player and theme; Explorer-specific settlement delay cannot become player truth.
- `INV-TDRP-R9-TOAST-ACTIVATION-AUTH-001`: COM activation/user input is untrusted transport until exact canonical notification/conversation authorization succeeds.
- Dossier: `projects/telegram-desktop-rust/dossiers/windows-poll-profile-settings-5501-5529-complete-read.md`.
- These rows grant source-read/traceability credit only.


### TDRP Revision 9 business settings source-read requirements 5530-5540

| requirement | risk | invariant | exact source evidence | production status |
| --- | --- | --- | --- | --- |
| TDRP-R9-BUSINESS-AWAY-GREETING-5530-5531-5538-5539 | critical | scheduled/inactivity automation uses one canonical conversation-policy owner; recipient/shortcut limits and persistence failures are fail-closed | shard/manifest/dossier | MAPPED-SOURCE-READ-PRODUCTION-GAP |
| TDRP-R9-BUSINESS-CHAT-INTRO-5532-5533 | high | intro title/description/sticker are canonical onboarding/profile state; preview is derived | same | MAPPED-SOURCE-READ-PRODUCTION-GAP |
| TDRP-R9-BUSINESS-CHAT-LINKS-5534-5535 | critical | create/share/copy/rename/delete resolves through canonical conversation link/deep-link owner and preserves exact recipient/message state | same | MAPPED-SOURCE-READ-PRODUCTION-GAP |
| TDRP-R9-BUSINESS-CHATBOT-DELEGATION-5536-5537 | critical | bot delegation extends canonical Agent/Bot + CapabilityBroker permissions; elevated transfers/username changes require explicit user-visible warning | same | MAPPED-SOURCE-READ-PRODUCTION-GAP |
| TDRP-R9-BUSINESS-LOCATION-5540 | high | address/map point remain canonical profile/location state and map unavailability has an explicit fallback | same | MAPPED-SOURCE-READ-PRODUCTION-GAP |

- `INV-TDRP-R9-BUSINESS-AUTOMATION-SINGLE-OWNER-001`: settings surfaces may edit automation but cannot own delivery truth or create a second scheduler.
- `INV-TDRP-R9-DELEGATED-BOT-CAPABILITY-001`: high-impact delegated permissions are explicit capability grants, never inferred from bot selection.
- Dossier: `projects/telegram-desktop-rust/dossiers/business-settings-5530-5540-complete-read.md`.


### TDRP Revision 9 source-read requirements 5541-5571

| requirement | risk | invariant | exact source evidence | production status |
| --- | --- | --- | --- | --- |
| TDRP-R9-BUSINESS-QUICK-REPLIES-5541-5547 | critical | templates and recipient scopes have one canonical owner; premium/count/name/message limits and include/exclude invariants fail closed | shard/manifest/dossier | MAPPED-SOURCE-READ-PRODUCTION-GAP |
| TDRP-R9-BUSINESS-WORKING-HOURS-5548-5549 | critical | normalized non-overlapping day/next-day intervals and timezone identity remain canonical availability state | same | MAPPED-SOURCE-READ-PRODUCTION-GAP |
| TDRP-R9-ACCOUNT-2SV-5550-5569 | critical | secret/code state is transient and bounded; recovery/reset/cancel-reset/error/invalidation semantics fail closed; other-device password changes revoke current flow | same | MAPPED-SOURCE-READ-PRODUCTION-GAP |
| TDRP-R9-ACCOUNT-2SV-PRESENTATION-5570-5571 | medium | success visuals are derived design-system presentation and never security truth | same | MAPPED-SOURCE-READ-PRODUCTION-OPEN |

- `INV-TDRP-R9-2SV-TRANSIENT-SECRET-001`: current/new passwords, recovery codes and pending email state must be cleared on invalidation, exit or idle expiry and never appear in logs/evidence.
- `INV-TDRP-R9-2SV-REMOTE-CHANGE-001`: PASSWORD_HASH_INVALID/SRP_PASSWORD_CHANGED invalidates the entire in-progress security flow rather than permitting retries with stale authority.
- `INV-TDRP-R9-2SV-RESET-STATE-001`: Recover, pending Reset, ready Reset and CancelReset are distinct observable states and cannot collapse into a single destructive action.
- `INV-TDRP-R9-BUSINESS-RECIPIENT-SCOPE-001`: all-except and selected-only maintain exact include/exclude/type invariants; settings UI never becomes delivery truth.
- Dossier: `projects/telegram-desktop-rust/dossiers/business-cloud-password-5541-5571-complete-read.md`.
- These rows grant source-read/traceability credit only.


### TDRP Revision 9 settings source-read requirements 5572-5580

| requirement | risk | invariant | exact source evidence | production status |
| --- | --- | --- | --- | --- |
| TDRP-R9-SETTINGS-SWITCH-A11Y-5572-5573 | medium | settings toggle exposes checkable/Switch semantics and lock/disabled state cannot mutate canonical value | shard/manifest/dossier | MAPPED-SOURCE-READ-PRODUCTION-OPEN |
| TDRP-R9-ACTIVE-SESSIONS-5574-5576 | critical | session identity/hash, current/incomplete classification, terminate scope and TTL remain server-backed and fail closed | same | MAPPED-SOURCE-READ-PRODUCTION-OPEN |
| TDRP-R9-ADVANCED-SETTINGS-5577-5578 | critical | network/storage/window/tray/autostart/update/archive controls mutate only canonical owners and obey platform capability/security constraints | same | MAPPED-SOURCE-READ-PRODUCTION-PARTIAL |
| TDRP-R9-BLOCKED-PEERS-5579-5580 | critical | block state is canonical privacy state; counts/list/empty UI are reactive projections only | same | MAPPED-SOURCE-READ-PRODUCTION-OPEN |

- `INV-TDRP-R9-ACTIVE-SESSION-HASH-001`: terminate-one must target the exact active authorization hash and never the current session by display inference.
- `INV-TDRP-R9-AUTOSTART-PASSCODE-001`: start-minimized remains disallowed when a local passcode requires an interactive verified launch prompt.
- `INV-TDRP-R9-SETTINGS-A11Y-001`: a custom toggle must preserve semantic checkable role/state, keyboard behavior and locked/disabled non-mutation.
- Dossier: `projects/telegram-desktop-rust/dossiers/settings-sessions-advanced-blocked-5572-5580-complete-read.md`.

## TDRP 863cf10d upstream delta — mapped-open requirements

| requirement_id | oracle_ids | invariant_ids | production owner | required evidence | verdict |
| --- | --- | --- | --- | --- | --- |
| TDRP-R9-WALLET-OUTSIDE-CLICK-001 | ORA-TDRP-WALLET-OUTSIDE-CLICK-001 | INV-WALLET-CLOSE-TOP-ONLY-001; INV-WALLET-EVENT-SWALLOWED-001; INV-WALLET-DEFERRED-DESTROY-001 | Canonical Dialog/Popover layer-stack owner | unit + interaction integration + keyboard/focus + cancellation + a11y + light/dark + responsive E2E | mapped-open |
| TDRP-R9-EDITOR-CIRCLE-ROUNDING-001 | ORA-TDRP-EDITOR-CIRCLE-ROUNDING-001 | INV-EDITOR-CIRCLE-RADIUS-001; INV-EDITOR-NONDESTRUCTIVE-STATE-001 | Canonical Media Editor owner | unit + editor state + visual + light/dark + keyboard/a11y + persistence regression | mapped-open |
| TDRP-R9-COMPOSER-BOT-MENU-COMPACT-001 | ORA-TDRP-COMPOSER-BOT-MENU-COMPACT-001 | INV-COMPOSER-PEER-FENCE-001; INV-COMPOSER-ANIMATION-CANCEL-001; INV-COMPOSER-CANONICAL-COMPONENT-001 | Canonical Composer + Button/IconButton + animation owner | unit + contract + rapid-typing cancellation + focus/keyboard + a11y + reduced-motion + light/dark + responsive + interaction E2E | mapped-open |

Wallet background `MouseButtonPress` is swallowed while the top information box is visible, closes only that box through deferred destruction, and preserves the lower Transaction layer. Circle sticker rounding maps to the canonical Media Editor (`Circle`, multiplier 0.5). Bot-menu compact/full behavior maps to canonical Composer/Button/IconButton/animation owners; TelegramButton/BotMenuButton is prohibited. None reduce unknown until production + exact-head evidence close.

| TDRP-R9-SETTINGS-BUSINESS-ROUTER-001 | ORA-TDRP-SETTINGS-BUSINESS-5584-5585 | INV-BUSINESS-READINESS-FENCE-001; INV-BUSINESS-PREMIUM-GATE-001; INV-BUSINESS-ACCOUNT-SCOPE-001 | Canonical Settings shell + existing Business capability owners | unit + server contract + readiness/account-switch fault + sponsored mutation reconciliation + keyboard/focus/a11y/light-dark/responsive E2E | mapped-open |


| TDRP-R9-SETTINGS-CALLS-001 | ORA-TDRP-SETTINGS-CALLS-5586-5587 | INV-CALL-DEVICE-FALLBACK-001; INV-CALL-PERMISSION-001; INV-CALL-PREVIEW-FENCE-001; INV-CALL-AUTHORIZATION-001 | Canonical Settings + call media/device + account authorization owners | unit + media-device contract + permission fault + current-call/group-call preview cancellation + authorization server reconciliation + keyboard/a11y/light-dark/responsive E2E | mapped-open |


| TDRP-R9-SETTINGS-CHAT-001 | ORA-TDRP-SETTINGS-CHAT-5588-5589 | INV-CHAT-THEME-SINGLE-OWNER-001; INV-CHAT-STORAGE-ROUTING-001; INV-CHAT-PRIVACY-ARCHIVE-001; INV-CHAT-SUPPORT-ACCOUNT-SCOPE-001 | Canonical Settings + Theme + Storage/Download + Privacy + Composer/Reaction + Support owners | unit + persistence/server contract + theme/file fault + account-switch cancellation + keyboard/focus/a11y/light-dark/responsive E2E | mapped-open |
 
| TDRP-R9-SETTINGS-CREDITS-001 | ORA-TDRP-SETTINGS-CREDITS-5590-5591 | INV-CREDITS-ACCOUNT-SCOPE-001; INV-CREDITS-TOPUP-SETTLEMENT-001; INV-CREDITS-HISTORY-RECONCILE-001; INV-CREDITS-WITHDRAWAL-001 | Canonical Wallet/Payments/Credits + subscription/history/earnings owners | unit + server contract + topup idempotency/cancel/account-switch + history pagination/rebuild + withdrawal fault/security + keyboard/a11y/light-dark/responsive E2E | mapped-open |
| TDRP-R9-SETTINGS-FOLDERS-001 | ORA-TDRP-SETTINGS-FOLDERS-5592-5593 | INV-FOLDER-DIFF-ORDER-001; INV-FOLDER-SHARED-LEAVE-001; INV-FOLDER-TAGS-DEBOUNCE-001; INV-FOLDER-PREMIUM-GATE-001 | Canonical Conversation collection/filter + Settings + account messaging service owners | unit + ordered server contract + optimistic/reconnect/account-switch reconciliation + shared-list peer-removal fault + tags debounce/teardown + premium/keyboard/a11y/light-dark/responsive E2E | mapped-open |
| TDRP-R9-SETTINGS-GLOBAL-TTL-001 | ORA-TDRP-SETTINGS-GLOBAL-TTL-5594-5595 | INV-TTL-DEFAULT-SERVER-001; INV-TTL-ELIGIBILITY-001; INV-TTL-BULK-PARTIAL-001; INV-TTL-ACCOUNT-FENCE-001 | Canonical Privacy/Retention + Conversation policy + account messaging service owners | unit + policy/server contract + property eligibility + bulk partial-failure/retry + account/peer stale fencing + keyboard/a11y/light-dark/responsive E2E | mapped-open |
| TDRP-R9-SETTINGS-INFORMATION-001 | ORA-TDRP-SETTINGS-INFORMATION-5596-5597 | INV-PROFILE-ASYNC-FENCE-001; INV-BIO-DEBOUNCE-FLUSH-001; INV-MULTIACCOUNT-SWITCH-001; INV-ACCOUNT-ORDER-001; INV-PRIVACY-BIRTHDAY-001 | Canonical Identity/Profile + Account/Session + Business/Privacy owners | unit + identity/server contract + debounce/teardown + account-switch/new-window/reorder/logout fault + privacy/security + keyboard/a11y/light-dark/responsive E2E | mapped-open |
| TDRP-R9-SETTINGS-LOCAL-PASSCODE-001 | ORA-TDRP-SETTINGS-LOCAL-PASSCODE-5598-5599 | INV-PASSCODE-60S-EXPIRY-001; INV-PASSCODE-STALE-INVALIDATE-001; INV-PASSCODE-VAULT-VERIFY-001; INV-PASSCODE-APPLOCK-001; INV-PASSCODE-SYSTEM-UNLOCK-001 | Canonical Local Security/App Lock + Wallet Key Protection + platform unlock owners | unit + crypto/storage contract + stale/external-change + retry/lockout + vault migration/fault + WinHello/TouchID/AppleWatch + keyboard/a11y/security E2E | mapped-open |


## TDRP settings local storage + main 5600-5604 — mapped-open requirements

| requirement_id | oracle_ids | invariant_ids | production owner | required evidence | verdict |
| --- | --- | --- | --- | --- | --- |
| TDRP-R9-SETTINGS-LOCAL-STORAGE-5600-5602 | ORA-TDRP-LOCAL-STORAGE-5600 | INV-STORAGE-DUALDB-001; INV-STORAGE-CLEAR-IDEMPOTENT-001; INV-STORAGE-QUOTA-COUPLING-001; INV-STORAGE-ACCOUNT-FENCE-001 | Canonical Settings/Storage + Cache/MediaCache policy + filesystem-capacity adapter | unit + storage contract + quota property + partial-clear/restart/account-switch fault + keyboard/focus/a11y/light-dark/responsive/reduced-motion E2E | mapped-open |
| TDRP-R9-SETTINGS-MAIN-5603-5604 | ORA-TDRP-SETTINGS-MAIN-5603 | INV-SETTINGS-ROUTE-OWNER-001; INV-SETTINGS-ACCOUNT-FENCE-001; INV-SETTINGS-SCALE-RESTART-001; INV-SETTINGS-ASYNC-REFRESH-001 | Canonical Settings shell + Identity/Account + Privacy/Security + Wallet/Business + preferences/platform adapters | unit + route/capability matrix + account-switch/stale-result + profile-upload fault + scale confirm/cancel/restart + support/promo idempotency + keyboard/focus/a11y/light-dark/responsive E2E | mapped-open |

- `INV-STORAGE-DUALDB-001`: normal and big-media cache stats/clearing reconcile as one storage surface without a second policy owner.
- `INV-STORAGE-CLEAR-IDEMPOTENT-001`: duplicate clears cannot overlap; completion requires actual clearing settlement, not only animation time.
- `INV-STORAGE-QUOTA-COUPLING-001`: total cache minus media cache remains at least 100 MB whenever policy is accepted.
- `INV-SETTINGS-ACCOUNT-FENCE-001`: profile, balances, business capability, suggestions and async reload results cannot cross active-account/session lifetime.
- `INV-SETTINGS-SCALE-RESTART-001`: cancelled preview restores configured state; confirmed restart persists one canonical preference mutation.
- Dossier: `projects/telegram-desktop-rust/dossiers/settings-local-storage-main-5600-5604-complete-read.md`.


## TDRP Notifications + Passkeys 5605-5613 — mapped-open requirements

| requirement_id | oracle_ids | invariant_ids | production owner | required evidence | verdict |
| --- | --- | --- | --- | --- | --- |
| TDRP-R9-NOTIFICATIONS-5605-5611 | ORA-TDRP-NOTIFY-5605; ORA-TDRP-NOTIFY-REACTIONS-5608; ORA-TDRP-NOTIFY-TYPE-5610 | INV-NOTIFY-POLICY-SINGLE-OWNER-001; INV-NOTIFY-ACCOUNT-CLEANUP-001; INV-NOTIFY-NATIVE-ADAPTER-001; INV-NOTIFY-PRIVACY-PREVIEW-001; INV-NOTIFY-ACTION-SCOPE-001; INV-NOTIFY-EXCEPTION-SCOPE-001 | Canonical Notification Policy + Account/Session scope + Privacy + Call authorization + Conversation override + Platform Notification adapter | unit + server contract/fault + account-switch/reload + reaction source-scope + type default/exception precedence + native/custom recreation + exact inactive-session cleanup + native reply/mark-read/open exact-scope + keyboard/a11y/light-dark/responsive packaged E2E | mapped-open |
| TDRP-R9-PASSKEYS-5612-5613 | ORA-TDRP-PASSKEYS-5612 | INV-PASSKEY-CHALLENGE-FENCE-001; INV-PASSKEY-SIGNED-BUILD-001; INV-PASSKEY-EXACT-ID-001; INV-PASSKEY-FINALIZE-RECONCILE-001 | Canonical Account Auth/Passkeys + Platform WebAuthn adapter | unit + server/WebAuthn contract + stale challenge/cancel/unsupported/unsigned + platform-success/server-fail + idempotency/account-switch/restart + signed packaged Windows/macOS WebAuthn acceptance | mapped-open |

- `INV-NOTIFY-ACCOUNT-CLEANUP-001`: disabling all-account notifications clears only inactive-session native/custom notifications and preserves active-session state.
- `INV-NOTIFY-NATIVE-ADAPTER-001`: native/custom notification managers are bounded projections; message/read truth remains canonical.
- `INV-NOTIFY-ACTION-SCOPE-001`: native reply/mark-read/open must re-authorize exact account/conversation/topic-or-sublist/message scope before mutation and clean only matching notifications.
- `INV-NOTIFY-EXCEPTION-SCOPE-001`: private/group/broadcast defaults and exact-peer exceptions preserve owner/type identity; add/remove/clear cannot leak across account or peer scope.
- `INV-PASSKEY-CHALLENGE-FENCE-001`: registration challenge and platform callback cannot outlive their account/session/request identity.
- `INV-PASSKEY-SIGNED-BUILD-001`: unsupported or unsigned platform credential creation fails closed; no software fallback may masquerade as platform WebAuthn.
- `INV-PASSKEY-EXACT-ID-001`: delete targets the exact canonical server credential id behind explicit confirmation.
- Existing remote Agent WebAuthn proxy/signer/runtime is not accepted as evidence of Account Passkeys product list/create/delete lifecycle.
- Dossiers: `projects/telegram-desktop-rust/dossiers/settings-notifications-passkeys-5605-5609-complete-read.md`; `projects/telegram-desktop-rust/dossiers/settings-notification-type-passkeys-5610-5613-complete-read.md`.

## TDRP Premium entitlement / commerce 5614–5616 — mapped-open requirements

| requirement_id | oracle_ids | invariant_ids | production owner | required evidence | verdict |
| --- | --- | --- | --- | --- | --- |
| TDRP-R9-PREMIUM-ENTITLEMENT-COMMERCE-5614-5616 | ORA-TDRP-PREMIUM-5614 | INV-PREMIUM-ENTITLEMENT-ACCOUNT-001; INV-PREMIUM-PURCHASE-ROUTE-001; INV-PREMIUM-REF-ATTRIBUTION-001; INV-PREMIUM-SETTLEMENT-RECONCILE-001; INV-PREMIUM-UI-DERIVED-001 | Canonical Entitlement/Subscription + Wallet/Payments/Commerce + Settings + design system | unit + service/commerce contract + invalid-route fail-closed + duplicate/idempotency + cancel/failure/settlement/account-switch/reload/restart + currency/option refresh + keyboard/focus/a11y/light-dark/responsive/reduced-motion + signed packaged acceptance | mapped-open |

Reading grants source/traceability credit only. Dossier: `projects/telegram-desktop-rust/dossiers/settings-premium-5614-5616-complete-read.md`.

## TDRP Privacy/Security + Shortcuts + Websites 5617–5622

| requirement_id | risk | invariant | production owner | verdict |
| --- | --- | --- | --- | --- |
| TDRP-R9-PRIVACY-SECURITY-5617-5618 | critical | Settings only routes exact-account canonical security/privacy state; destructive clear/session/TTL actions require fresh scope and fail closed | Account Security/Privacy + Authorization + Retention + Payments | mapped-open |
| TDRP-R9-SHORTCUTS-5619-5620 | high | one canonical command registry owns bindings; recording/collision/reset cannot leak focus or create duplicate command truth | Command/Shortcut Registry + Settings | mapped-open |
| TDRP-R9-WEB-AUTH-5621-5622 | critical | terminate-one targets exact server authorization hash; terminate-all/block coupling reconciles once under exact account | Account Authorization + Bot permissions + Blocked Peers | mapped-open |

Dossier: `projects/telegram-desktop-rust/dossiers/settings-privacy-shortcuts-websites-5617-5622-complete-read.md`.

## TDRP Settings infrastructure + Credits/Gift 5623–5632

| requirement_id | risk | invariant | production owner | verdict |
| --- | --- | --- | --- | --- |
| TDRP-R9-SETTINGS-BUILDER-5623-5630 | high | one typed Settings definition drives rendered controls, search/index and highlights; no duplicated setting truth; hidden diagnostics are environment/security bounded | Settings + UniversalSearch + design system + support diagnostics | mapped-open |
| TDRP-R9-CREDITS-GIFT-5631-5632 | critical | wallet/gift/payment actions use exact account/owner/id and reconcile duplicate/cancel/failure/settlement/restart once | Wallet/Payments/Credits + Gift/Commerce | mapped-open |

Dossier: `projects/telegram-desktop-rust/dossiers/settings-builder-credits-5623-5632-complete-read.md`.

## TDRP Settings experimental/support/navigation/power 5633–5642

| requirement_id | risk | invariant | production owner | verdict |
| --- | --- | --- | --- | --- |
| TDRP-R9-EXPERIMENTAL-5633-5634 | high | flags/import/export/restart are environment/security bounded and never bypass canonical policy | Feature Flag/Experiment policy + Settings | mapped-open |
| TDRP-R9-FAQ-5635-5636 | medium | cancellable locale/account FAQ fetch cannot publish stale suggestions | Help/Support | mapped-open |
| TDRP-R9-SETTINGS-NAV-5637-5641 | high | responsive Settings layer and keyboard/focus/highlight/search reuse canonical controls and never activate hidden/disabled state | Settings + design system + Notification Settings | mapped-open |
| TDRP-R9-POWER-5642 | high | OS battery state is an adapter input; one canonical effective power policy controls expensive features and save/cancel | Performance/Power + platform battery adapter | mapped-open |

Dossier: `projects/telegram-desktop-rust/dossiers/settings-experimental-power-5633-5642-complete-read.md`.
