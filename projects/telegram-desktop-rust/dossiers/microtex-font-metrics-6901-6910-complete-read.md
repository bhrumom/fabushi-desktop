# MicroTeX math font metrics and fallback graph — exact read 6901-6910

Authority: `6fed91ffab9861771a75f29df65031b3c80b6941` / `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`.

This batch exact-reads `cmbsy10`, `cmbx10`, `cmbxti10`, `cmex10`, `cmmi10`, `cmmi10_unchanged`, `cmmib10`, `cmmib10_unchanged`, `cmr10`, and `cmss10` at recursive orders 6901-6910 with their exact component blobs recorded in the source-disposition shard.

## Responsibility decomposition

The files encode glyph width/height/depth/italic metrics; family/style fallback edges; math skew; larger-glyph transitions; extensible delimiter pieces; ligatures; and kern pairs. These are data contracts supporting deterministic mathematical typography, not justification for a new font subsystem.

## Existing owner

Fabushi keeps `ConversationWorkspace -> Transcript -> AssistantMath -> math-runtime.ts` as the only shipping math owner. `KATEX_ASSET`, `loadShippedKatexRuntime`, and `renderKatexMarkup` define the shipped renderer boundary. Font/dependency artifact identity belongs to the existing Build/Release provenance owner.

## Closure status

**mapped-open**. Reading these tables does not demonstrate KaTeX equivalence for every MicroTeX metric/fallback behavior, so `unknown` remains unchanged. Focused GitHub Actions evidence must prove applicable typography/fallback/resource behavior or record a source-neutral platform delta before any row can become verified.

Accounting: read-through **6,910/16,125**, unread **9,215**, unknown **15,845**, omitted **0**. Next: **6911 `Telegram/ThirdParty/MicroTeX::src/res/font/cmssbx10.def.cpp@8b1c030f0123c71809eda15a48314298475300e7`**.
