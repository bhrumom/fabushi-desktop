# MicroTeX builtin resources and font provenance — exact read 6893-6900

Authority: `6fed91ffab9861771a75f29df65031b3c80b6941` / `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`.

Exact source identities:
- 6893 `src/res/builtin/formula_mappings.res.cpp@51c4d0a62987eb97590bc2ef151b9b69b2f69127`
- 6894 `src/res/builtin/meson.build@358b7c814f247b00628013000202fad5b31a642d`
- 6895 `src/res/builtin/symbol_mapping.res.cpp@debc0618ff4dcfaf5484b6e5c9559c1063c14394`
- 6896 `src/res/builtin/tex_param.res.cpp@c7b5b993d377db79a0e67dca4369f695fd82790c`
- 6897 `src/res/builtin/tex_symbols.res.cpp@8ce76d17a0b995cf745601a82e569dc96ab31a30`
- 6898 `src/res/font/README@f0603ff0ee08ec518f340bf2d78edefdf6f0a9fb`
- 6899 `src/res/font/bi10.def.cpp@3fc80cfdca144eae4b3463cf473259a8aaf9dbc0`
- 6900 `src/res/font/bx10.def.cpp@2c77e6e636a6bf6f4d603b06b6267eb1258df282`

## Responsibilities

The builtin tables carry deterministic Unicode/codepoint-to-formula normalization, ASCII/Unicode symbol aliases, text-mode mappings, TeX layout constants, symbol atom taxonomy and delimiter/operator classification. `meson.build` is the exact resource/build closure tying the builtin translation units together. The font README supplies provenance to CTAN Computer Modern TFM inputs; `bi10` and `bx10` bind family/style fallbacks and glyph metrics.

## Existing-owner resolution

Fabushi keeps the existing `ConversationWorkspace -> Transcript -> AssistantMath/math-runtime` owner and the existing Build/Release provenance owner. MicroTeX C++ resource tables, parser, graphics stack and font runtime are not target architecture. Applicable behavior must be expressed through source-neutral normalization, KaTeX/render fallback, deterministic dependency/version/resource closure and release provenance.

## Open production contract

Status: **mapped-open**. Implementation status remains open: reading/decomposition creates no production credit and does not reduce `unknown`.

Required closure includes focused canonical tests for Unicode/formula normalization and alias consistency, symbol/fallback behavior relevant to Fabushi's shipped KaTeX surface, bounded/fail-closed unsupported input, and release checks proving the exact shipped math dependency/resource provenance. No fake local response, second math owner, or imported Telegram/MicroTeX runtime is acceptable.

Accounting after this dossier: read-through **6,900/16,125**, unread **9,225**, unknown **15,845**, omitted **0**. Next deterministic source: **6901 `Telegram/ThirdParty/MicroTeX::src/res/font/cmbsy10.def.cpp@6e72926a0156f2b06033d9995c7653b5d76b2a10`**.
