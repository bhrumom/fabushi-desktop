# fcitx5-qt Qt5 canonical platform input — 7,526–7,540

Authority: `fcitx/fcitx5-qt@0285a5d18367d8f3af80dab7e4819d5555982340`, mounted by `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941`.

All fifteen entries are exact-blob read and responsibility-decomposed. Reading closes no unknown and grants no implementation/release credit.

Applicable semantics map to existing Fabushi owners: non-focus-stealing candidate/preedit selection and paging; validated theme/resource paths with deterministic fallback and hot reload; exactly-one active platform input owner with failover/stale-adapter fencing; focus/preedit/reset/surrounding/cursor/virtual-keyboard lifecycle; async key-event window/object liveness fencing; compose fallback; and deterministic XKB keypad/navigation/modifier/function/international/media/text key mapping. Orders 7,539/7,540 are the formally read targets for Qt4 symlink aliases 7,483/7,484.

Fcitx/Qt/QPA/DBus/XKB mechanisms and Fcitx visual identity remain replacement-only. Production equivalence requires shipping owners, focused tests and descendant exact-head evidence.

Accounting: 7,540/16,125 read; 8,585 unread; 15,845 unknown; 0 omitted. Next: 7,541 `Telegram/ThirdParty/fcitx5-qt::qt5/widgetsaddons/CMakeLists.txt@939af6dc8dc1f698aa2ff0867c5ce2143c280699`.
