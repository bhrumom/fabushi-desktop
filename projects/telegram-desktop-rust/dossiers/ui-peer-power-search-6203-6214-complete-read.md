# TDRP Revision 9 — peer/power/search helpers 6203–6214 complete read

Accepted authority: `telegramdesktop/tdesktop@65e23ba7137ea4129b6bc1b2616104a1f59495ef` / tree `6b616494f3465324e749a04dcd1c9d508657a998`.

Orders **6,203–6,214** are exact-blob read and responsibility-decomposed. Reading alone grants no implementation/verification or unknown-closure credit.

- **6203–6204**: responsive selectable profile/chat color samples map to canonical Picker/Radio/profile appearance presentation.
- **6205–6206**: video Avatar streaming maps to canonical Avatar + Resource/media playback, including peer/photo identity fencing, loop/start-position, pause, failure marking, masks/DPR and explicit clear/error teardown.
- **6207–6208**: power-saving flags/force-all and animation suppression map to the single canonical reduced-motion/power policy.
- **6209**: horizontal drag/finish lifecycle maps to the canonical resizable panel/splitter interaction.
- **6210–6211**: row scroll rendering cache is explicitly ephemeral and bounded (256 entries / 32 MiB), physical-size/DPR keyed, invalidatable and cleared after 120 ms scroll settlement.
- **6212–6213**: query/view lifecycle, clear a11y and responsive SearchField layout map to canonical SearchField/Search ownership.
- **6214**: style imports only; consume the canonical design system.

Accounting: **6,214 / 16,125 read**, **9,911 unread**, **15,846 unknown**, **0 omitted**. First unread: **6,215** `Telegram/SourceFiles/ui/text/format_song_document_name.cpp@d5a77025b392b8385bff3268be1f7465d91d48a2`.
