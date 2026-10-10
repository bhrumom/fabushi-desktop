# TDRP Revision 9 — Tabs/reorder responsibilities 6240–6244 complete read

Accepted authority: `telegramdesktop/tdesktop@65e23ba7137ea4129b6bc1b2616104a1f59495ef` / tree `6b616494f3465324e749a04dcd1c9d508657a998`.

Orders **6,240–6,244** are exact-blob read and responsibility-decomposed. They cover persisted horizontal/vertical Tabs mode defaults; Tabs text/icon/unread Badge measurement; locked-range click and ContextMenu routing; custom-emoji pause; and drag reorder with pinned intervals, start threshold, animated neighbor shifts, edge auto-scroll, cancel/apply convergence and stable index remapping.

All map to existing source-neutral `Tabs` / `Badge` / `ContextMenu` / `Status` and the single canonical ordering/persistence owner. Reading alone does not reduce unknown. Accounting: **6,244 / 16,125 read**, **9,881 unread**, **15,846 unknown**, **0 omitted**. First unread: **6,245** `Telegram/SourceFiles/ui/widgets/chat_filters_tabs_strip.cpp@750feab6cbf3b30a3b1e2bc946d85eaed4a4382c`.
