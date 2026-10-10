# fcitx5-qt Qt6 closure — 7,566–7,642

Authority: `fcitx/fcitx5-qt@0285a5d18367d8f3af80dab7e4819d5555982340`, mounted by `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941`.

This closes the remaining **77** non-directory entries of the pinned fcitx5-qt component. All are exact-object read and responsibility-decomposed. Reading closes no unknown and grants no production/release credit.

Retained source-neutral responsibilities:
- exact service/portal availability and owner fencing, exactly-one input-context creation, pending create/introspection/query cleanup, focus/key/preedit/surrounding/candidate/virtual-keyboard lifecycle and teardown;
- executable DBus generator provenance, while the `interfaces` row is a symlink alias to the already-read Qt5 XML authority;
- Qt6 helper/platform-input compatibility aliases are composition/provenance only and do not duplicate target behavior;
- requested-vs-active input-module observability;
- quick-phrase persistence with UTF-8/line-format validation, escaped values, dirty state, safe async save, stale-save/file-switch fencing, Save/Discard/Cancel, filename validation, system-file protection, destructive confirmation and visible file-operation failure;
- Qt6 Settings plugin/factory/save and shortcut-capture semantics remain one canonical Settings/Input owner, not a toolkit-version-specific root;
- focused key-translation tests are behavior oracles for canonical keyboard normalization.

The pinned component contains **215** non-directory entries total and is now exact-read through its final row 7,642 `test/testkeytrans.cpp`.

Accounting: 7,642/16,125 read; 8,483 unread; 15,845 unknown; 0 omitted. Next: 7,643 `Telegram/ThirdParty/hime::.clang-format@8dfce611a060025a35ab44967831b2a8984d5d69`.
