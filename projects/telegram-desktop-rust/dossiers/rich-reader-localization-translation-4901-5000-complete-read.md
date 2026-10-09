# Exact-source dossier: orders 4901-5000

- Accepted upstream: `telegramdesktop/tdesktop@36a0c87ca096c48ccf6193aa2c31707fdcafcc7c`
- Root tree: `94e009f981d886ee55cd0450f0a5cc9b38305ba8`
- Read method: direct exact-blob semantic reading of reader/parser/prepare/resource/localization/translation state machines, service boundaries, security limits and teardown behavior. Read-complete does not imply implementation or release closure.
- Accounting after batch: read-through 5,000; unread 11,120; unknown 15,841; unknown-closed 279; omitted 0.

## Responsibility decomposition

### 4901-4901: Rich content model contract and validation limits
- Canonical owner: Canonical Rich Content model owner
- State machine: rich blocks/text/lists/tables/media/buttons -> bounded normalized model -> validation/link encoding
- Side effects/ownership: keeps one source-neutral rich content model with explicit block/media limits and structural metadata
- Failure/lifecycle boundary: invalid/deep/oversized model, malformed table/list/media/link metadata
- Disposition: `mapped-open-rich-content-model`; implementation and release verification remain open.

### 4902-4907: Rich reader search and zoom interaction lifecycle
- Canonical owner: Canonical Rich Content Reader Search/Toolbar owner
- State machine: query/focus -> snapshot/cache/coalesced scan -> current match navigation; zoom +/-/reset -> persisted reader scale
- Side effects/ownership: reuses canonical SearchField/Toolbar/Menu/Tooltip and reader settings without source-derived controls
- Failure/lifecycle boundary: stale search generation, hidden-text mismatch, focus return failure, zoom persistence/platform shortcut regression
- Disposition: `mapped-open-rich-reader-controls`; implementation and release verification remain open.

### 4908-4921: Rich article layout/paint/selection/text interaction lifecycle
- Canonical owner: Canonical Rich Content Reader layout/selection owner
- State machine: prepared document -> responsive layout/paint/search highlights/selection/hit test -> copy/link/media activation -> relayout
- Side effects/ownership: keeps tables/lists/code/details/formulas/media/text selection, anchors and horizontal scroll arbitration in one canonical reader
- Failure/lifecycle boundary: selection/copy offset drift, touch/wheel arbitration leak, relayout/cache stale, unsafe link activation, a11y/theme/DPI regression
- Disposition: `mapped-open-rich-reader-layout`; implementation and release verification remain open.

### 4922-4923: Rich action-button row runtime lifecycle
- Canonical owner: Canonical Rich Content Reader Button/Resource owner
- State machine: prepared buttons -> weighted responsive layout/elision -> permission-aware activation/ripple/loading -> settlement
- Side effects/ownership: reuses canonical Button/Menu/Tooltip semantics while preserving loading and bot/app action contracts
- Failure/lifecycle boundary: stale weak runtime, disabled/loading timer leak, wrong copy target, action permission failure
- Disposition: `mapped-open-rich-reader-actions`; implementation and release verification remain open.

### 4924-4941: Rich reader media/runtime/controller/document lifecycle
- Canonical owner: Canonical Rich Content Reader + Media/Resource owner
- State machine: document/media runtime -> controller/viewer/embed/history-media -> open/share/download/channel/media actions -> close/reuse/teardown
- Side effects/ownership: keeps photo/video/document/map/channel/embed media activation and stable resource reuse behind canonical reader/media gateways
- Failure/lifecycle boundary: unsupported mime, stale reused block, failed download/open/share, overlay/window teardown leak, spoiler/selection state loss
- Disposition: `mapped-open-rich-reader-media`; implementation and release verification remain open.

### 4942-4943: Math rendering backend and bounded formula lifecycle
- Canonical owner: Canonical Rich Content Math/Resource owner
- State machine: TeX input -> validate/cap/init/measure/render -> bounded image/cache/fallback -> paint
- Side effects/ownership: keeps formula parsing/rendering resource caps and fallback independent of Telegram naming
- Failure/lifecycle boundary: oversized TeX/image, init/parse/allocation failure, DPR/metric overflow, cache pressure
- Disposition: `mapped-open-rich-content-math`; implementation and release verification remain open.

### 4944-4951: Rich source parse/validate/finalize lifecycle
- Canonical owner: Canonical Rich Content Parser/Security owner
- State machine: source bytes -> size/memory/capability validation -> cmark conversion/math extraction -> anchors/footnotes/final document or explicit failure
- Side effects/ownership: keeps parser memory/formula/depth limits, unsupported preservation and warnings source-neutral
- Failure/lifecycle boundary: memory/limit overflow, malformed source/ranges/table/footnotes, parser abort, capability mismatch
- Disposition: `mapped-open-rich-content-parser`; implementation and release verification remain open.

### 4952-4969: Rich document prepare/native/serialize pipeline lifecycle
- Canonical owner: Canonical Rich Content Prepare/Serializer owner
- State machine: parsed or native rich model -> blocks/inline/formulas/links/media/table normalization -> measurements -> stable prepared document/inline objects
- Side effects/ownership: keeps one canonical prepare pipeline for markdown/native rich content with safe relative/local/external links and bounded table/formula fallbacks
- Failure/lifecycle boundary: unsafe relative path, malformed inline object, table flattening/error, formula cache mismatch, media/link/entity normalization loss
- Disposition: `mapped-open-rich-content-prepare`; implementation and release verification remain open.

### 4970-4977: Rich reader slideshow/theme/view/widget lifecycle
- Canonical owner: Canonical Rich Content Reader UI owner
- State machine: prepared article -> slideshow/theme/window/widget -> search/selection/media/link/touch/keyboard/scroll/zoom -> close/relayout
- Side effects/ownership: reuses canonical Toolbar/Search/Menu/ContextMenu/Tooltip/Dialog reader surfaces with light/dark/responsive/a11y behavior
- Failure/lifecycle boundary: focus/IME/touch/drag/scroll race, media recreation failure, stale retained article, theme/cache/geometry regression
- Disposition: `mapped-open-rich-content-reader-ui`; implementation and release verification remain open.

### 4978-4979: Localization cloud pack/switch lifecycle
- Canonical owner: Canonical Localization service owner
- State machine: account/domain -> language list/difference/version/suggestion -> confirm/switch/base-pack/restart -> reactive updated locale
- Side effects/ownership: keeps language-pack lifecycle behind canonical localization service; Telegram transport is reference-only
- Failure/lifecycle boundary: stale request/version, unofficial pack readiness, switch/restart race, base pack mismatch, network failure
- Disposition: `mapped-open-localization-service`; implementation and release verification remain open.

### 4980-4981: Custom language file parser lifecycle
- Canonical owner: Canonical Localization file/import owner
- State machine: bounded local file/content -> encoding/comment/key/value/tag parse -> requested values/warnings/errors
- Side effects/ownership: keeps custom localization import bounded and validated
- Failure/lifecycle boundary: oversize file, invalid BOM/encoding/syntax/tag/key, empty content, read error
- Disposition: `mapped-open-localization-import`; implementation and release verification remain open.

### 4982-4982: Hardcoded fallback error copy contract
- Canonical owner: Canonical Localization fallback owner
- State machine: unavailable localized key -> bounded product fallback copy
- Side effects/ownership: keeps last-resort user-visible errors source-neutral and localizable
- Failure/lifecycle boundary: stale brand/source-specific fallback, untranslated critical error
- Disposition: `mapped-open-localization-fallback`; implementation and release verification remain open.

### 4983-4986: Localization runtime/plural/date/name/key lifecycle
- Canonical owner: Canonical Localization runtime owner
- State machine: default/base/custom/cloud values -> tag/plural rules + date/name/weekday/month formatting -> reactive value producers
- Side effects/ownership: keeps one locale runtime with base fallback, custom serialization and platform-aware formatting
- Failure/lifecycle boundary: invalid serialized pack, unknown/repeated tag, plural/base drift, locale/date/name ordering regression
- Disposition: `mapped-open-localization-runtime`; implementation and release verification remain open.

### 4987-4988: Localized numeric animation replacement lifecycle
- Canonical owner: Canonical localized numeric presentation owner
- State machine: localized text + numeric tag replacement -> offset/length-aware animated string
- Side effects/ownership: preserves number-animation replacement positions after localization
- Failure/lifecycle boundary: tag offset/length drift, bidi/locale formatting regression
- Disposition: `mapped-open-localized-number-animation`; implementation and release verification remain open.

### 4989-4989: Localization build dependency boundary
- Canonical owner: Canonical desktop build boundary
- State machine: compile dependency surface -> localization module compilation
- Side effects/ownership: build-only dependency aggregation; no shipping source-derived public API
- Failure/lifecycle boundary: dependency drift or build break
- Disposition: `mapped-open-build-boundary`; implementation and release verification remain open.

### 4990-4996: Locale plural/tag/rich-entity translation contract
- Canonical owner: Canonical Localization formatting owner
- State machine: locale plural rule + tags/replacements/entities/projections -> plain/rich/reactive localized output
- Side effects/ownership: keeps CLDR-like plural selection and rich entity offset preservation in canonical localization formatting
- Failure/lifecycle boundary: plural category drift, entity-offset corruption, bidi/case/link projection failure
- Disposition: `mapped-open-localization-formatting`; implementation and release verification remain open.

### 4997-5000: Translation provider/service abstraction lifecycle
- Canonical owner: Canonical Translation service gateway
- State machine: message/text + source/target language -> provider selection/batch request -> translated TextWithEntities or typed error
- Side effects/ownership: keeps provider abstraction and message-id optimization; MTProto/platform/custom URL remain replaceable adapters outside canonical product contract
- Failure/lifecycle boundary: provider/network failure, batch cardinality mismatch, stale message text, entity conversion loss, unsupported provider
- Disposition: `mapped-open-translation-service`; implementation and release verification remain open.

## Exact-source attestation

Every order 4901-5000 is bound to its accepted-tree blob in `inventory/source-attestation-manifests/4901-5000.json` and the matching `source_binary_evidence` row. Current-head GitHub Actions Source authority must regenerate exact blob, consumer trace, and reachability evidence before this batch is accepted as current-head provenance.

No entry in this batch is promoted to implemented/verified solely by this read.
