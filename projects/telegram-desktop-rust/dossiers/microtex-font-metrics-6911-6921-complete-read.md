# MicroTeX font metrics/build closure — deterministic read 6911–6921

Authority:
- Telegram: `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941`
- mounted component: `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`
- recursive orders: 6911–6921

## Exact read result

The batch covers `cmssbx10`, `cmssi10`, `cmsy10`, `cmti10`, the unchanged italic compatibility variant, `cmtt10`, `dsrom10`, Euler Fraktur bold/medium, `i10`, and the font `meson.build` closure.

Responsibilities observed across the exact blobs include glyph metrics, family/style fallback relations, larger-glyph chains, ligatures, kern/skew behavior where present, math-alphabet typography, and the exact build list that binds font-definition translation units. These are dependency typography/provenance responsibilities, not justification for a second MicroTeX runtime.

## Canonical ownership

Existing owner remains:
- `frontend/src/recovered/features/conversation/workspace/math-runtime.ts#renderKatexMarkup/loadShippedKatexRuntime`
- `frontend/src/recovered/features/conversation/workspace/math.tsx#AssistantMath`
- existing Build/Release dependency provenance owner

No row is promoted to implemented/verified by reading. Focused KaTeX typography/fallback/resource-provenance GitHub Actions evidence is still required before unknown can close.

## Accounting

After durable inclusion of this batch: read-through 6,921 / 16,125; unread 9,204; unknown 15,845; omitted 0. First unread is order 6,922 `Telegram/ThirdParty/MicroTeX::src/res/font/moustache.def.cpp` (blob `629e014ed715e4eb8731a76d9edb39e8de0b7bf6`).
