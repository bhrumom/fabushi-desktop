# TDRP Revision 9 — rebaseline UI radius complete read

Accepted upstream: `telegramdesktop/tdesktop@72b3b71c3d6e450e5ef94a3112dd750a0168aa0b`

This dossier records full-file semantic reads only. It does **not** mark either source responsibility verified, so global `unknown` is unchanged.

## Fully read entries

| path | blob SHA | status |
| --- | --- | --- |
| `Telegram/SourceFiles/ui/chat/chat_style_radius.cpp` | `c3976270c7f841d553c261286c1d7fd1dfc6ef7b` | complete read |
| `Telegram/SourceFiles/ui/chat/chat_style_radius.h` | `408f3a6d7d4582f0ed6369b587c2ec1bba0d4a26` | complete read |

## Responsibilities understood

The files mostly own presentation geometry for message bubbles and file thumbnails, but they also contain an applicable product preference: `use-small-msg-bubble-radius`. That preference selects the canonical radius family and is declared restart-bound. Corner behavior itself is typed (`None`, `Tail`, `Small`, `Large`) rather than allowing arbitrary per-surface radii.

Fabushi disposition is deliberately still open:

- Qt painting/style implementation is not ported mechanically;
- any retained radius preference belongs to the existing Fabushi Settings + design-system token owners, not a Telegram-specific UI owner;
- a Fabushi implementation must keep one canonical radius token family, preserve any applicable grouping/tail semantics, persist the preference if retained, and truthfully model whether applying it is live or restart-bound;
- implementation, visual/interaction/a11y evidence, current-head Actions and design-system acceptance are still pending.

These two files therefore reduce `unread` by 2 but do not reduce `unknown`.
