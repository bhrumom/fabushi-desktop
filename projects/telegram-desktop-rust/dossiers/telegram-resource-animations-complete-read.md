# Telegram animation resources — rolling complete-read dossier

Accepted upstream: `telegramdesktop/tdesktop@aac515c5408015231a273c80ab4c4b33815e63ab`

## Entry 89 — ban.tgs

Path: `Telegram/Resources/animations/ban.tgs`  
Blob: `1ac1aaa8a493a99160d772a7739224a08435cc68`  
Read status: complete after gzip decompression.

The exact asset is a TGS/Lottie JSON document (v5.7.1), 512x512, 60 fps, frames 0–120. Its primary layer is named `Hand` and the vector group is named `Block-user`; it is included by `Telegram/Resources/qrc/telegram/animations.qrc` under alias `ban.tgs`.

Disposition is intentionally still open: the asset clearly carries an applicable block/ban visual meaning, but exact runtime consumer/state ownership has not yet been established from production source. It therefore decreases unread but does **not** decrease unknown. No Telegram-specific Fabushi component is created; once the consumer is identified it must map to an existing canonical Fabushi surface where possible and receive visual/a11y/light-dark/responsive evidence.

Accounting after entry 89:
- full-read: `149`
- unread: `15,663`
- unknown: `15,730`
- omitted: `0`
