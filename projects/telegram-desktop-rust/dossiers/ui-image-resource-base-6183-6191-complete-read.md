# UI image/resource base 6183-6191 complete read

Accepted authority: `telegramdesktop/tdesktop@65e23ba7137ea4129b6bc1b2616104a1f59495ef`, root tree `6b616494f3465324e749a04dcd1c9d508657a998`. Orders **6,183-6,191** were read from exact accepted blobs and responsibility-decomposed. Reading alone does not reduce `unknown`.

This shard defines the image/resource foundation rather than a new product-domain owner. `Image` owns bounded decode/prepare and DPR-aware transform caches; image/download locations own typed identity, backward-compatible serialization, deterministic cache keys and file-reference refresh; factories project photo/document/sticker/video/progressive/cached/in-memory/web inputs into that location contract; local image sources enforce the application byte limit; SVG previews sanitize first and cap both input bytes (1 MiB) and unconstrained default rendering (4096 px).

Fabushi disposition is source-neutral reuse/completion of the canonical `Resource` / attachment / media pipeline. Required closure evidence includes download/cache/fallback/cancellation/teardown/stale-result fencing, cache-key determinism, serialization compatibility, file-reference refresh, decode-size enforcement and SVG safety. No Telegram-named image subsystem or second cache/download owner is permitted.

All nine rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`. Accounting after this read: **6,191 / 16,125 read**, **9,934 unread**, **15,846 unknown**, **0 omitted**. First unread: **6,192** `Telegram/SourceFiles/ui/item_text_options.cpp@eec63d2be0577b6c18d1dae1699e8a8e65504fe9`.
