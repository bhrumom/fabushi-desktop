# IV Markdown article complete read — d346 baseline

Status: read-complete / mapped-open / not implemented / not verified  
Spec: TDRP-001 Revision 9 / FBCP-001 Revision 7  
Accepted upstream: `telegramdesktop/tdesktop@d346b42a1d30ef60dc989b6e5191bb8e571f6bd5`  
Root tree: `5db05afa460bb030ac36316beb6496df03742c59`

## Source identities and read method

- `Telegram/SourceFiles/iv/markdown/iv_markdown_article.h@ed6c7afd9f15330e4ed1f254c4bcaf8786344d6b`
- `Telegram/SourceFiles/iv/markdown/iv_markdown_article.cpp@15adf3076d04850c89b6683ab6e76626537d36d0`

The header was read completely. The implementation was read completely in bounded ranges 1–600, 601–1200, 1201–1800, 1801–2400, 2401–3000, 3001–3600, 3601–4200, 4201–4800, 4801–5400, 5401–6000, 6001–6600, 6601–7200, and 7201–EOF (7557). No truncated rendering is credited.

## Responsibilities recovered

### TDRP-R9-IV-ARTICLE-CONTRACT-001

The article contract is much more than a painter. It exposes one stateful rich-content projection over canonical content:

- stable selection/search segments, anchors and scroll anchors;
- typed hit-test results for text, prepared links, media, inline buttons, button rows, code-copy, edit controls and structural drop targets;
- editable leaf/block/list/table identities rather than pixel-only edit targets;
- details/blockquote expansion and toggle state;
- horizontal-scroll ownership and begin/update/end semantics;
- text/rich-page selection extraction, formula/media geometry, visible-range projection and formatted-date scheduling;
- palette/raster invalidation, heavy-resource unload, spoiler hiding and explicit before-destroy teardown.

### TDRP-R9-IV-ARTICLE-CONTENT-LIFECYCLE-001

The implementation preserves state across content/layout generations while failing closed on identity mismatch:

- text-leaf caches are keyed by stable edit identity or prepared path and source signatures include RTL, style, resize limits, media-runtime dependence and inline-button-column dependence;
- changing media runtime prunes only cached leaves that depend on that runtime; compatible media blocks may be reused by stable media identity;
- related-article images survive compatible relayout through explicit state capture/restore;
- `updatePreparedLeaf` patches only when edit identity, block kind and structural invariants still match; otherwise it falls back to whole `setContent`;
- code-highlight jobs are keyed by source text/language and pruned when content no longer owns them; laid-out block pointers are cleared before geometry invalidation;
- formula raster/color caches are generation-aware and reset for changed formula slots, renderer, palette or DPR;
- runtime maps for placeholders, button rows and task-marker ripple state are pruned against live laid-out identities;
- dead media blocks are detached from the host and recreated instead of being silently reused;
- missing media is counted, not faked, and heavy media can be unloaded without deleting canonical article content;
- layout invalidation captures horizontal scroll state before destroying segment/anchor geometry.

### TDRP-R9-IV-ARTICLE-INTERACTION-001

Interaction and navigation are identity- and scope-aware:

- visible-segment lookup is bounded by precomputed top/bottom indexes when the pointer is in the visible range;
- hit testing prioritizes code-copy, text links, block/media/button actions and only then fallback segment boundaries;
- media external URLs are converted to typed prepared links; button URLs and disabled controls keep their own semantics;
- structural drag/drop rejects insertion into the selected subtree or its own no-op interval;
- details/blockquote anchor expansion opens ancestors required to reveal the target;
- scroll anchors store segment identity plus fractional position for relayout restoration;
- horizontal scroll owner identity uses edit block path where available and prepared path otherwise; offsets are captured, clamped and restored after relayout;
- touch/scrollbar/media horizontal scrolling share explicit begin/update/end ownership;
- `revealSegment` scrolls only the owning horizontal viewport enough to expose the logical segment;
- selection extraction keeps direct/indirect segment semantics and can return both MIME text and a corresponding RichPage slice.

### TDRP-R9-IV-ARTICLE-BOUNDED-WORK-001

The file contains several bounded-work and recovery rules:

- reveal line-count cache is scoped to one layout generation;
- formatted-date refresh only rebuilds expired leaves/cells and recomputes the next deadline;
- placeholder loading/ripple and button/task feedback repaint the smallest known live rect when possible;
- runtime maps are pruned after each finalized relayout;
- final relayout rebuilds anchors/segments/visible lookup, restores scroll state, refreshes scroll-translated geometry and media rounding, and records the next formatted-date deadline;
- a retained-layout fast path is used only when compatible; failure falls back to a full relayout;
- release Linux GCC/LTO width miscompilation is explicitly fenced by keeping relayout width in object state rather than a parameter.

## Existing-owner audit

Current Fabushi already has source-neutral owners that this behavior must extend rather than replace:

- `frontend/src/recovered/features/conversation/workspace/transcript.tsx`: unique ConversationTranscript/rich-content projection.
- `frontend/src/recovered/features/conversation/workspace/find-in-chat-controller.ts`: object-scoped transcript search/navigation lifecycle.
- `frontend/src/recovered/features/conversation/workspace/math.tsx`: canonical math renderer/fallback.
- `frontend/src/recovered/features/conversation/workspace/media-viewer.tsx`: canonical media/resource projection and async lifetime fencing.
- `frontend/src/recovered/features/conversation/cards/transcript-card/url-card.ts`: canonical URL/external-open owner.
- `frontend/src/production/ProductionRenderer.tsx`: only shipping ConversationWorkspace composition.

Rejected: a Telegram/IV article runtime, second message/transcript store, second search owner, second media store, or second URL owner.

## Oracles and invariants

- **ORA-TDRP-IV-ARTICLE-001:** all rich-content projections must derive from the current canonical transcript/content identity; cached layout/media/action state may be reused only when its source identity and lifecycle remain compatible.
- **INV-TDRP-IV-ARTICLE-PATCH-001:** a partial leaf update that cannot prove stable structural/edit identity must fall back to whole-content reconciliation.
- **INV-TDRP-IV-ARTICLE-CACHE-001:** cached text/formula/media/highlight state cannot cross an incompatible media/runtime/content/layout generation.
- **INV-TDRP-IV-ARTICLE-SCROLL-001:** scroll state is restored by stable owner/segment identity and clamped to the new geometry, never by stale pixel ownership alone.
- **INV-TDRP-IV-ARTICLE-DROP-001:** structural drag/drop cannot target the selected subtree or a no-op insertion interval.
- **INV-TDRP-IV-ARTICLE-HIT-001:** typed button/link/media/edit hit semantics cannot collapse into a generic click capability.
- **INV-TDRP-IV-ARTICLE-VISIBLE-001:** visible-range acceleration may narrow work but cannot change hit/search/selection correctness.
- **INV-TDRP-IV-ARTICLE-RUNTIME-001:** placeholder/button/task/media/highlight runtimes are pruned or detached when their canonical source identity is no longer live.
- **INV-TDRP-IV-ARTICLE-HEAVY-001:** heavy-resource unload releases derived media resources without deleting canonical content or selection/search identity.

## Target composition

`ProductShell -> ConversationWorkspace -> ConversationTranscript -> canonical Search / Math / Resource / URL / typed action owners`

Qt layout, painter and widget implementation are not copied. State ownership, identity, recovery, cancellation, bounded work and interaction semantics remain applicable.

## Future evidence required

Before implementation/verification, GitHub Actions must cover at minimum:

- current-head Actions evidence for committed-generation assistant projection patch versus whole-remount fallback; arbitrary non-assistant prepared-leaf patch breadth remains open;
- media-runtime replacement and stale media/highlight callback fencing;
- formula renderer/palette/DPR invalidation;
- stable search/selection/anchor behavior through relayout;
- current-head Actions evidence for canonical-message/structural-owner horizontal-scroll capture, compatible restore/clamp and incompatible-owner reset; broader reveal behavior after layout changes remains open;
- structural drag/drop self/nested-selection rejection;
- current-head Actions evidence for thinking/tool-call hidden-detail search navigation and disclosure-commit re-highlighting; quote/details ancestor expansion beyond canonical outline rows remains open;
- current-head Actions evidence for shared near-viewport transcript image/video unload/reload, active-or-near filmstrip thumbnail resolution, stale-settlement fencing and eager compatibility fallback; active-viewer runtime pruning and broader heavy-resource policy remain open;
- visible-range optimization equivalence;
- shipping keyboard/a11y, temporal, reload/restart and packaged recovery where applicable.

Normative identifiers: `TDRP-MOD-01`, `TDRP-MOD-02`, `TDRP-OWN-01`, `TDRP-COMP-01`, `G-FILE`, `G-MODULE`, `G-PRODUCTION`, `G-COMPOSITION`, `G-SEARCH`, `G-UI-FUNCTIONAL`, `G-TEMPORAL`, `G-PERF-SOAK`, `G-SECURITY-PRIVACY`, `G-EVIDENCE`.

## Current verdict

Both changed source entries are read-complete and remain mapped-open. The current PR implements bounded production slices in existing canonical owners: assistant rich-content keys are reconciled against the last committed projection, retaining identity only for equal owner/path/kind/shape plus exact or monotonic streaming growth and remounting the whole projection on shortening, non-prefix rewrite, owner or structural mismatch. Code/table regions retain horizontal offset only across those compatible canonical-message plus structural identities, and both the ordinary ConversationTranscript path and the send-message:text lazy card leaf supply their canonical entry identity. Transcript attachment cards resolve image/video derived media only near the viewport, equal-margin subscriptions share one observer, and the open viewer filmstrip resolves only active or horizontally near-visible thumbnails; leaving those budgets cancels stale settlement and releases reconstructable derived state. IntersectionObserver absence preserves eager compatibility; audio and the explicitly opened MediaViewer remain live. Find-in-chat now indexes thinking/tool-call labels, full hidden detail text and the exact visible ToolResultCard projection; stepping to such a match resolves its stable entry row, expands the existing disclosure owner and reapplies highlights after the expansion commit, while the collapsed preview is removed during expansion to keep occurrence order aligned. `UNIT-TDRP-IV-ARTICLE-PROJECTION-RECONCILE-001`, `UNIT-TDRP-IV-ARTICLE-PROJECTION-SEQUENCE-001` and `CONTRACT-TDRP-IV-ARTICLE-PARTIAL-FALLBACK-001` trace the bounded projection slice; `UNIT-TDRP-IV-ARTICLE-SCROLL-CLAMP-001` and `CONTRACT-TDRP-IV-ARTICLE-SCROLL-CONTINUITY-001` remain the scroll trace points; `UNIT-TDRP-IV-ARTICLE-MEDIA-VISIBILITY-001` and `CONTRACT-TDRP-IV-ARTICLE-MEDIA-LIFECYCLE-001` trace the visibility slice. `UNIT-TDRP-IV-ARTICLE-HIDDEN-SEARCH-001` and `CONTRACT-TDRP-IV-ARTICLE-HIDDEN-REVEAL-001` trace the bounded hidden-detail search/reveal slice. None is promoted beyond mapped status. Exact-head GitHub Actions evidence is still required, and arbitrary non-assistant prepared-leaf patch breadth, quote/details ancestor expansion, general anchor reveal after relayout, highlight/runtime pruning, active-viewer heavy-resource policy, visible-range equivalence and packaged acceptance remain open; no verified claim is made.
