# TDRP Revision 9 — filter Tabs, color editor and slider responsibilities 6245–6251 complete read

Accepted authority: `telegramdesktop/tdesktop@65e23ba7137ea4129b6bc1b2616104a1f59495ef` / tree `6b616494f3465324e749a04dcd1c9d508657a998`.

Orders **6,245–6,251** are exact-blob read and responsibility-decomposed.

- **6245–6246** bind the canonical filter Tabs surface to authoritative filter/session state: premium locking, unread, active filter, edit/remove/mark-read ContextMenu actions, settings routing, saved reorder, drag/drop, scroll visibility and keyboard cycling.
- **6247–6249** define a canonical color Picker/editor: RGBA/HSL palette generation, hue/opacity/lightness controls, validated numeric/result fields, current/new preview, limits, and change/submit propagation.
- **6250–6251** define Continuous/Filled/Media Slider semantics: pointer/wheel/key input, progress-versus-finished convergence, a11y value changes, horizontal/vertical mapping, pseudo-discrete snapping, disabled/fade/hover state, receivedTill and divider/marker rendering.

All must converge on source-neutral canonical components and existing domain owners. Reading alone does not reduce unknown. Accounting: **6,251 / 16,125 read**, **9,874 unread**, **15,846 unknown**, **0 omitted**. First unread: **6,252** `Telegram/SourceFiles/ui/widgets/cross_fade_label.cpp@fb094439ceebc0ad1b45db68ebf08d31450464e3`.
