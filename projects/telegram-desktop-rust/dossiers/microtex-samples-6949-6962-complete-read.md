# MicroTeX sample integration/lifecycle — deterministic read 6949–6962

Authority: `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941` -> `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`.

The exact sample blobs cover GTK, memory-check, Qt, Skia, QML and Win32 demos. Platform demo UI is not a Fabushi shipping surface. Applicable responsibilities are nevertheless preserved: single application-scoped math initialization/release, replacing and freeing stale render state before re-parse, local parse-failure containment, authoritative render dimensions, bounded paint/export, and deterministic teardown in memory-check and native event-loop paths.

These map to the existing AssistantMath/KaTeX runtime interface, focused math contract tests, and Build/Release provenance. No sample widget/runtime is introduced into Fabushi.

Accounting: read-through 6,962 / 16,125; unread 9,163; unknown 15,845; omitted 0. First unread: 6,963 `Telegram/ThirdParty/MicroTeX::src/utils/dict_tree.h@314b8eca99c896f44280228978d25b601bae10ea`.
