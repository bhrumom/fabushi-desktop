# TDRP Revision 9 — UI boxes 5,872–5,909 complete read

Authority: `65e23ba7137ea4129b6bc1b2616104a1f59495ef` / tree `6b616494f3465324e749a04dcd1c9d508657a998`.

All 38 exact blobs were read and responsibility-decomposed. This closes the current `ui/boxes/**` read prefix but grants **no unknown/product/release credit by itself**.

Responsibilities covered: font search/preview; language selection; IME-safe duration input; collectible info/copy/link; generic confirm/inform semantics; phone-code confirmation; accessible country search; birthday picker; fact-check editing; invite-link limits/approval/subscription; stake/payment UI; profile QR generation; call rating; report reason/details; privacy reveal versus premium; generic radio choice; time drum picker.

Canonical mapping is source-neutral and must reuse Fabushi Dialog/AlertDialog/Picker/SearchField/TextField/Radio/Checkbox/Button/IconButton/ListRow/Avatar/Toast/Menu. Server/auth/payment/report/entitlement truth stays with the existing canonical domain owner. Telegram-specific product components/runtimes are not authorized.

A same-head Revision 9 run on predecessor `d52b8c5` proved the earlier unread mismatch was fixed but exposed a stricter real contract mismatch: `upstream.lock.json.source_disposition_evidence.read_through` remained 5,848 while ledger/index/root were 5,871. This batch fixes that persisted store rather than weakening `validate-tdrp-revision9-authority.mjs`.

Accounting after this batch: **5,909/16,125 read; 10,216 unread; 15,846 unknown; 0 omitted**. First unread: **5,910** `Telegram/SourceFiles/ui/cached_round_corners.cpp@7da69a2299a1fc5d67f8392fe45ee82672b33440`.
