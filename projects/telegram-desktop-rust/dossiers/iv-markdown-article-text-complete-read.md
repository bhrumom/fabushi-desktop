# IV Markdown article-text complete read — d346 baseline

Status: read-complete / mapped-open / bounded External+Email prepared-link slice implemented / not verified  
Spec: TDRP-001 Revision 9 / FBCP-001 Revision 7  
Accepted upstream: `telegramdesktop/tdesktop@d346b42a1d30ef60dc989b6e5191bb8e571f6bd5`  
Root tree: `5db05afa460bb030ac36316beb6496df03742c59`

## Source identities and read method

The accepted header and implementation were read completely. The implementation was read in bounded ranges 1–520, 521–1040, 1041–1560, 1561–2080, and 2081–2502; no truncated output is credited.

- `Telegram/SourceFiles/iv/markdown/iv_markdown_article_text.h@82da919256c01a0fcd09765d9e22b0c2d0ded1f3`
- `Telegram/SourceFiles/iv/markdown/iv_markdown_article_text.cpp@2fcab1a3e549d320ca38266373d562238c13fe29`

## Responsibilities recovered

### TDRP-R9-IV-ARTICLE-TEXT-CONTRACT-001

The public contract exposes product semantics that must survive the UI rewrite:

- prepared links are bound to text leaves with typed click/copy/tooltip/entity behavior;
- spoiler links use an explicit click filter rather than becoming ordinary links;
- inline formulas have a shared cache with renderer swap, palette invalidation, raster invalidation and explicit fallback;
- text leaves can project formula, inline image and rich-button objects without creating a separate transcript truth;
- inline rich buttons expose actionable/disabled, URL/tooltip and loading state through the same text object contract;
- media/runtime repaint hooks and minimum resize/RTL semantics are part of the projection contract.

### TDRP-R9-IV-ARTICLE-TEXT-LIFECYCLE-001

The implementation defines additional behavior and lifecycle:

**Prepared links**
- Bounded Fabushi implementation: canonical transcript links now project HTTP(S) and explicit `mailto:` through the existing URL/message-action owners. HTTP(S) remains behind the existing metadata/open normalizer; mailto strips query data, decodes the address, opens only through the existing desktop external-open boundary, and exposes exact `Copy Email` text through the existing message context menu. Partial/custom rendered labels disclose the canonical target as a tooltip only when the visible label is not target-equivalent; bare targets avoid redundant tooltip text. Unsupported script/file/relative targets remain fail-closed. `CONTRACT-TDRP-IV-PREPARED-LINK-EXTERNAL-COPY-001` locks this slice; exact-head execution is pending.
- External and InstantView links normalize a `TextEntity` and distinguish URL, CustomUrl and Email.
- Copy text/context labels differ for email/link and for anchor/footnote/local-file cases.
- Rejected-relative/toggle-details/toggle-blockquote/rich-page-button links do not accidentally gain external-copy behavior.
- Only left/middle click is delegated to the integration link handler.
- Rich-page buttons deliberately bypass the generic prepared-link extraction route and use the typed inline-button action path.

**Inline rich buttons**
- Disabled types are removed from the actionable set.
- Activation reconstructs one typed button record and delegates to the existing rich-button action boundary.
- Tooltip/copy-link text are derived from the current typed button data.
- Label parsing is deliberately bounded. Very long custom-emoji spans are shortened only under conservative all-or-nothing conditions, with bidi direction, entity overlap, parser trimming, surrogate pairs and emoji-crossing seams protected.
- The bounded-prefix search prevents sender-controlled labels from causing unbounded renderer work while preserving visible output around the pill width.
- Button objects unload persistent label animation and derive pending/loading feedback from the authoritative button request key.

**Inline formulas**
- Formula identity includes TeX, kind, text size and render caps.
- Measurement can be shared across objects; raster output is cached per device pixel ratio and colorized output per color+DPR.
- Palette/raster invalidation is explicit; renderer replacement invalidates raster state.
- Render failure falls back to measured/source text rather than losing semantic content.
- Exact ascent/descent/inset metrics are normalized before projection.

**Inline images / resources**
- Dynamic image updates subscribe only after first paint, repaint the last painted rect when possible, and unsubscribe on unload.
- Replacement text remains available as a non-image semantic fallback.

## Existing-owner audit

Current Fabushi already has suitable canonical owners:

- `frontend/src/recovered/features/conversation/workspace/transcript.tsx`: one `ConversationTranscript`, canonical assistant Markdown rendering, link projection, attachment projection and `AssistantMessageContent`.
- `frontend/src/recovered/features/conversation/workspace/math.tsx`: canonical `AssistantMath`/KaTeX load-render-fallback owner.
- `frontend/src/recovered/features/conversation/cards/transcript-card/url-card.ts`: canonical URL normalization, metadata cache and native external-open boundary.
- `frontend/src/recovered/features/conversation/workspace/media-viewer.tsx`: canonical attachment/media resolver/view projection with stale async fencing.
- `frontend/src/recovered/features/conversation/cards/transcript-card/widget-interactions.ts`: existing typed action pending/settlement/scope-generation owner.

Rejected: a Telegram/IV Markdown runtime, second URL owner, second formula cache product owner, second media store, or second Transcript root.

## Behavior oracle / invariants

- **ORA-TDRP-IV-ARTICLE-TEXT-001:** canonical rich text preserves typed link/button/formula/media semantics while all asynchronous/render caches remain derived from the same transcript/resource/action truth.
- **INV-TDRP-IV-LINK-TYPED-001:** external/email/anchor/local/toggle/button link kinds cannot silently broaden into one generic external-open capability.
- **INV-TDRP-IV-BUTTON-BOUNDED-001:** untrusted inline-button labels must have bounded parse/layout work without changing the visible/action semantics of accepted content.
- **INV-TDRP-IV-FORMULA-FALLBACK-001:** formula render/cache failure preserves semantic fallback text and cannot corrupt a later renderer/palette/DPR generation.
- **INV-TDRP-IV-MEDIA-LIFECYCLE-001:** inline media update subscriptions and async resolution cannot mutate an unloaded/replaced transcript surface.
- **INV-TDRP-IV-TEXT-DISABLED-001:** disabled inline buttons have no click/action capability.

## Target composition

`ConversationTranscript -> canonical rich-text/URL/Math/Resource/TranscriptCard owners -> typed desktop/native/service boundaries`

Qt text engine/painter details are not copied. The mature behavior above is retained through canonical Fabushi owners.

## Required future evidence

Before full verification:
- bounded External/Email projection now has `CONTRACT-TDRP-IV-PREPARED-LINK-EXTERNAL-COPY-001`; complete the remaining link-kind table for Anchor, footnote/backlink, LocalFile, rejected-relative, toggle kinds and RichPageButton, plus broader CustomUrl/InstantView entity-shape coverage;
- external-open/auth negative cases at the existing desktop/native boundary;
- long/overlapping/bidi/surrogate/custom-emoji label property cases with bounded work;
- formula measurement/raster/palette/DPR replacement and failure fallback cases;
- inline media late-update/unload/replacement fencing;
- disabled/pending inline-button action tests;
- shipping transcript keyboard/a11y/visual and temporal/reload/restart evidence where applicable.

Normative identifiers: `TDRP-MOD-01`, `TDRP-MOD-02`, `TDRP-OWN-01`, `TDRP-COMP-01`, `G-FILE`, `G-MODULE`, `G-PRODUCTION`, `G-COMPOSITION`, `G-UI-FUNCTIONAL`, `G-TEMPORAL`, `G-PERF-SOAK`, `G-SECURITY-PRIVACY`, `G-EVIDENCE`.

## Current verdict

Read-complete and mapped-open. The bounded External/Email prepared-link slice is implemented through the existing `ConversationTranscript`, `url-card` and canonical message-action menu: HTTP(S) and explicit mailto open remain behind the shipping external-open boundary, mailto copy semantics expose the decoded address with `Copy Email`, and script/file/relative targets fail closed. `CONTRACT-TDRP-IV-PREPARED-LINK-EXTERNAL-COPY-001` traces this slice. Exact-head execution is still pending, and Anchor/footnote/backlink/LocalFile/toggle/RichPageButton, broader CustomUrl/InstantView entity-shape coverage, bounded rich-button label work, full formula/media lifecycle, packaged keyboard/a11y/visual and independent review remain open; no verified or complete article-text claim is made.
