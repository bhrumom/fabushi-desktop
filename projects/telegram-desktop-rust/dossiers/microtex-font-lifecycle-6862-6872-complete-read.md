# MicroTeX font lifecycle — exact read 6862-6872

Authority: `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941` → `Telegram/ThirdParty/MicroTeX` → `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`.

## Responsibility decomposition

- 6862-6863: Unicode block classification/alphabet registration with explicit unknown fallback.
- 6864-6865: glyph/font identity, immutable metrics and optional delimiter-extension ownership.
- 6866-6867: registration, lazy load, style variants and safe metrics/extension/next-larger/ligature/kern misses.
- 6868: deterministic font-set registration closure/order.
- 6869-6870: style/symbol/glyph resolution, safe misses, invalid/non-finite size and magnification rejection, reset/init/free lifecycle.
- 6871: font implementation/header build closure.
- 6872: deterministic symbol-set registration closure/order.

## Canonical mapping

Existing-owner-first keeps `frontend/src/recovered/features/conversation/workspace/math-runtime.ts` as the sole Transcript math wrapper, the shipped KaTeX asset as dependency implementation, and Build/Release as provenance owner. No MicroTeX/Telegram font owner, second parser, table store or process is introduced.

Applicable invariants are failure isolation and deterministic fallback: unsupported/missing glyphs or malformed input must not crash or poison later transcript renders; mutable per-render state must not leak between entries; dependency/build closure must not silently omit required runtime assets. MicroTeX table formats and C++ ownership mechanics are evidence, not target architecture.

This batch is read/responsibility-decomposed only and does not lower global unknown.
