# TDRP Revision 9 — UI boxes 5,860–5,871 complete read

Authority: `65e23ba7137ea4129b6bc1b2616104a1f59495ef` / `6b616494f3465324e749a04dcd1c9d508657a998`.

Exact blobs 5,860–5,871 were fully read and responsibility-decomposed. They cover an about/feature information dialog, message auto-delete TTL settings, boost/community capability projection, calendar/range selection, and scheduled date-time/repeat-period selection. These are product/UI responsibilities, not evidence-only rows.

The migration mapping is source-neutral: canonical Fabushi Dialog/AlertDialog/Picker/Button/IconButton/TextField/Menu primitives must be reused; Qt-specific paint/event mechanics are implementation details. Backend/domain truth must remain with the existing Fabushi owner. Cocoon/Boost applicability and backend replacement are still open and receive no product credit merely because their source was read.

Calendar and ChooseDateTime additionally require locale/a11y/bounds/lifetime behavior and are directly relevant to scheduled-message/Tasks/Automations surfaces, but no verified claim is made until exact production composition and same-head tests exist.

Accounting after this batch: **5,871/16,125 read; 10,254 unread; 15,846 unknown; 0 omitted**. First unread: **5,872** `Telegram/SourceFiles/ui/boxes/choose_font_box.cpp@56a7386cb666bac9412bdfafc9e9a2f02e20598e`.
