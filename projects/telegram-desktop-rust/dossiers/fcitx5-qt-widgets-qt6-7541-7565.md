# fcitx5-qt widgets + Qt6 input contracts — 7,541–7,565

Authority: `fcitx/fcitx5-qt@0285a5d18367d8f3af80dab7e4819d5555982340`, mounted by `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941`.

All 25 entries are exact-blob read and responsibility-decomposed. Reading closes no unknown and grants no implementation or release credit.

Applicable responsibilities:
- Qt5 widgets-addons build/export provenance and consumer dependency closure;
- configuration-plugin discovery with normalized addon paths, exact IID/metadata validation, deterministic key ownership and fail-closed creation;
- canonical Settings load/reset/save plus dirty state, asynchronous save settlement via `saveFinished` on all paths and sub-config saves;
- canonical localization rather than a Fcitx-specific localization truth;
- shortcut capture policy for multi-key/modifierless/modifier-only/keycode modes, rollback/clear, left/right modifiers, a 600 ms modifierless settlement window and platform-gated raw keycode mode;
- deterministic Unicode/keypad/modifier/media key normalization;
- Qt6 build/generated-input provenance without creating a toolkit-version-specific runtime owner;
- Qt6 controller/typed input contracts and input-session commands/events map to the same canonical Settings/TextField/Composer/platform owners already used for Qt5.

Qt/Fcitx widget/plugin/DBus mechanics are replacement-only. Production equivalence still requires shipping owners, focused tests and descendant exact-head evidence.

Accounting: 7,565/16,125 read; 8,560 unread; 15,845 unknown; 0 omitted. Next: 7,566 `Telegram/ThirdParty/fcitx5-qt::qt6/dbusaddons/fcitxqtinputcontextproxy_p.h@226d1917e943bc3801a53b4bb11b5cfecdb39c3d`.
