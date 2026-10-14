# fcitx5-qt Qt5 GUI-wrapper / IM probing deterministic source read — 7,506–7,515

Authority: `fcitx/fcitx5-qt@0285a5d18367d8f3af80dab7e4819d5555982340`, mounted by `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941`.

All ten non-directory entries from Qt5 guiwrapper and immodule-probing are exact-blob read and responsibility-decomposed. This is mapping evidence only; no unknown is closed.

## GUI wrapper replacement and retained semantics

The Qt helper executable, widgets, desktop entry, foreign-window APIs and Fcitx-branded presentation are replacement-only. Fabushi keeps one canonical Settings/ProductShell composition.

Applicable behavior maps to existing Settings/platform owners:
- normalize a supplied configuration path and probe capability existence before opening;
- fail explicitly when no config surface exists;
- dirty state enables Apply/OK/Reset, while Reset reloads authoritative state;
- asynchronous save disables editing until settlement and prevents duplicate mutation;
- after local save, notify the authoritative platform owner; a failed notification restores interaction, shows a visible error and cancels close-after-save;
- close-after-save happens only after successful settlement;
- parent/modal lifetime is preserved by the ProductShell rather than a source-shaped helper window.

## Platform input diagnostics

The IM-module probing executable is not retained. Its product-relevant contract is observability: distinguish the active platform, the requested input-module identity, the actual active input-context implementation, and an explicitly absent context. These diagnostics map to existing platform input adapters, support/Settings diagnostics and Build/Release evidence.

Next: order 7,516 `Telegram/ThirdParty/fcitx5-qt::qt5/platforminputcontext/CMakeLists.txt`. Accounting: 7,515/16,125 read; 8,610 unread; 15,845 unknown; 0 omitted.
