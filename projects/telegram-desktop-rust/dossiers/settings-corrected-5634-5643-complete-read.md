# Corrected deterministic source batch 5634-5643

Authority: `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e`, tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders are exact non-directory/blob ranks in the accepted recursive tree.

- 5634 `Telegram/SourceFiles/settings/settings_experimental.cpp@34c03579...`: Experimental option editor with search, guards, reset/import/export/deeplinks and restart settlement. Owner: Canonical feature-flag/experiment policy + controlled developer Settings.
- 5635 `Telegram/SourceFiles/settings/settings_experimental.h@c1b4c4bb...`: Experimental Settings lifecycle/search/menu/import-export interface. Owner: Canonical feature-flag Settings route.
- 5636 `Telegram/SourceFiles/settings/settings_faq_suggestions.cpp@d7a9b3ce...`: Cancellable FAQ suggestion loader/parser over localized FAQ TOC. Owner: Canonical Help/Support search suggestions.
- 5637 `Telegram/SourceFiles/settings/settings_faq_suggestions.h@dcd1c123...`: FAQ suggestion entry/list/loading lifecycle interface. Owner: Canonical Help/Support.
- 5638 `Telegram/SourceFiles/settings/settings_intro.cpp@e0f2ac53...`: Settings layer shell, top bar, responsive sizing, focus/a11y and quick-control composition. Owner: Canonical Settings shell + design system.
- 5639 `Telegram/SourceFiles/settings/settings_intro.h@3d2db92a...`: Settings LayerWidget lifecycle/sizing/focus/paint contract. Owner: Canonical Settings shell.
- 5640 `Telegram/SourceFiles/settings/settings_key_navigation.cpp@34c27c77...`: Keyboard navigation state machine over visible enabled Settings controls. Owner: Canonical keyboard/focus navigation + design system.
- 5641 `Telegram/SourceFiles/settings/settings_key_navigation.h@39b413c4...`: Keyboard navigation tracker/activation/highlight interface. Owner: Canonical keyboard/focus navigation.
- 5642 `Telegram/SourceFiles/settings/settings_notifications_common.h@a994906b...`: Shared notification Settings presentation contract. Owner: Canonical Notification Settings + design system.
- 5643 `Telegram/SourceFiles/settings/settings_power_saving.cpp@d026fb92...`: Power-saving policy editor combining OS battery-saving input with canonical feature suppression. Owner: Canonical Performance/Power policy + platform battery adapter.

All applicable responsibilities remain mapped-open; reading grants no unknown credit. Current contiguous read-through after the corrected batches is 5,653/16,123; unread 10,470; unknown 15,844; omitted 0. First unread is 5,654 `Telegram/SourceFiles/statistics/chart_lines_filter_controller.cpp@40c3b612...`.
