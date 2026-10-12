# TDRP Revision 9 — UI color/button controls 5,981–5,991 complete read

Authority: `65e23ba7137ea4129b6bc1b2616104a1f59495ef` / tree `6b616494f3465324e749a04dcd1c9d508657a998`.

Orders 5,981–5,991 are exact-blob read-complete and responsibility-decomposed. The range covers contrast/brightness and packed-color semantics plus canonical button pending, ContextMenu and two-label composition responsibilities.

The most important behavioral invariant is not cosmetic: upstream explicitly documents that a dimmed/disabled busy button is **not** the authoritative duplicate-operation refusal. Keyboard or connected handlers can still reach the operation, so Fabushi must pair canonical Button pending presentation with an independent in-flight/reentrancy fence in the owning command/state machine. Context menus likewise remain single-instance, lifetime-bound and empty-menu fail-closed.

Reading alone closes no unknown. Accounting after this batch: **5,991/16,125 read; 10,134 unread; 15,846 unknown; 0 omitted**. First unread: **5,992** `Telegram/SourceFiles/ui/controls/call_button.cpp@af74333fde39b33ca6a40a8bc8efd8bef0da3a9b`.
