# Hime GTAB runtime, input data and settings — orders 8,096–8,105

Exact pinned-source reading at `hime-ime/hime@9b3e6f9ab59d1fe4d9de73d3bf0fed7789f921c5` verified every Git blob identity for orders 8,096–8,105.

The range adds several applicable responsibility families without adopting Hime/XIM/GTK as architecture: derived phrase-data rebuild and provenance; durable candidate usage learning; alignment-safe key decoding; the large GTAB composition/candidate/preedit/commit lifecycle including exact/partial/wildcard matching, paging, selection, reset and auto-send; typed input-method state/flags; deterministic CIN→GTAB input-table compilation and validation; authoritative platform keyboard/input signals such as caps-lock/input identity; and backward-compatible Settings/resource persistence.

Order 8,101 is a GTK compatibility no-op shim. It is recorded as mapped-open rather than used to claim unknown closure; any necessary workaround belongs only in a thin source-neutral platform adapter.

All ten rows remain `mapped-open`, `unknown_closed=false`, and `omitted=false`. The production target remains existing canonical platform-input, PromptRichTextEditor/Composer, Settings, Resource/persistence and Build/Release owners. Native OS method/language/width/scriptVariant/candidate/preedit/focus lifecycle remains open and must use truthful native signals rather than browser-locale inference.

Accounting after this read window: **8,105/16,125 read; 8,020 unread; 15,845 unknown; 0 omitted**. Next source: **8,106 `Telegram/ThirdParty/hime::src/hime-crypt.c@f1beec82b6099ea79ce0abb27228a3a520e67b7a`**.
