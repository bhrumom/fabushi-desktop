# TDRP Revision 9 live rebaseline — 863cf10d wallet/editor/bot-menu delta

Accepted upstream: `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / root tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Previous `42f8a36d43b8c805bc821905bea4cfeb3af1d41d` is historical-only for changed blobs. Root non-directory count is 6,651; recursive total 16,123.

Three commits were read: `3157f7eb96dd5121470cf9fc6662ba5171dda6d9` (wallet outside click), `b7c199631074e9f131d0255da36905bc9eef0abf` (circle sticker rounding), and `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` (typing-responsive bot-menu shrink). All 15 changed paths were inspected against the live tree.

| new order | path | live blob | delta | read disposition |
| ---: | --- | --- | --- | --- |
| 92 | `Telegram/CMakeLists.txt` | `57d281e7f8004fda0e3c093b6739e4bd0909ce62` | modified | read-complete / re-decomposed |
| 95 | `Telegram/Resources/animations/bot_menu.tgs` | `d97bcdfac84e7439f3ebeb23005af43aff3e973a` | added | read-complete / re-decomposed |
| 3111 | `Telegram/Resources/langs/lang.strings` | `ffc8ec99465bae81e99f1ca6134bb8fa541fa844` | modified | read-complete / re-decomposed |
| 3132 | `Telegram/Resources/qrc/telegram/animations.qrc` | `6791a54a5e75d894bb7068d4596d80ba0af01f58` | modified | read-complete / re-decomposed |
| 4117 | `Telegram/SourceFiles/editor/photo_editor_common.cpp` | `706958639adbb566674c2567b46a3c04285d3548` | modified | read-complete / re-decomposed |
| 4118 | `Telegram/SourceFiles/editor/photo_editor_common.h` | `638547389d31ce248ea33a25a0cd05e51ddb1a55` | modified | read-complete / re-decomposed |
| 4121 | `Telegram/SourceFiles/editor/photo_editor_controls.cpp` | `4f06f9914877a71bbcf9814fbe3b69c0adf07490` | modified | read-complete / re-decomposed |
| 4330 | `Telegram/SourceFiles/history/history_widget.cpp` | `43b200334aa85c3febe47953c1ac5e277f9c48b6` | modified | read-complete / re-decomposed |
| 4331 | `Telegram/SourceFiles/history/history_widget.h` | `f7c8e2c51c2de32190088b5868d93c29c2c8b864` | modified | read-complete / re-decomposed |
| 4333 | `Telegram/SourceFiles/history/view/controls/history_view_bot_menu_button.cpp` | `ad488b26845891801b9eac85e0ff187fa8610bc2` | added | read-complete / re-decomposed |
| 4334 | `Telegram/SourceFiles/history/view/controls/history_view_bot_menu_button.h` | `16c99ba4f0666831a183588d4849c2418dfef732` | added | read-complete / re-decomposed |
| 4343 | `Telegram/SourceFiles/history/view/controls/history_view_compose_controls.cpp` | `f95a3511a8840bcd7c002bf52ec0bb71cc7e0071` | modified | read-complete / re-decomposed |
| 4344 | `Telegram/SourceFiles/history/view/controls/history_view_compose_controls.h` | `49b796d3f3332ae8f23b8b9064e1e43f23879779` | modified | read-complete / re-decomposed |
| 5941 | `Telegram/SourceFiles/ui/chat/chat.style` | `b67008036a5f969406aa2c6968eb5c33d2b15c08` | modified | delta-reviewed mapped-open; deterministic unread credit preserved |
| 6314 | `Telegram/SourceFiles/wallet/wallet_content.cpp` | `764aada8f033174de2dfd66a07e1d54ced0b66d4` | modified | delta-reviewed mapped-open; deterministic unread credit preserved |

## Responsibility decomposition

**Wallet modal outside click.** Background mouse press while the top information layer is shown is consumed; only the current box closes through a deferred callback, preserving the Transaction layer below. Canonical owner: Dialog/Popover layer-stack. Production/evidence remains open. `wallet_content.cpp` is order 6,314, so deterministic unread credit is preserved.

**Sticker circle rounding.** Circle is a first-class rounded-corners choice with multiplier 0.5 and localized control entry. Canonical owner: Media Editor state/mask/control. No Telegram editor root. Production/visual/state evidence remains open.

**Typing-responsive bot menu.** 150ms width/content transition compacts at the field-character threshold, uses first emoji or animated icon fallback, reflows composer geometry, reacts to command changes and fences recreation by peer identity. Canonical owners: Composer + Button/IconButton + animation/resource. TelegramButton/BotMenuButton is prohibited. Rapid-typing cancellation, conversation switch, keyboard/focus, a11y, reduced-motion, light/dark and responsive evidence are required.

Read-through is 5,583. First unread is order 5,584 `Telegram/SourceFiles/settings/sections/settings_business.cpp` blob `66f2e398dc6e41616c64b122d11c4a829c3cce7f`. Unknown stays 15,844; omitted stays 0.
