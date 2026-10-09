# Corrected deterministic source batch 5644-5653

Authority: `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e`, tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders are exact non-directory/blob ranks in the accepted recursive tree.

- 5644 `Telegram/SourceFiles/settings/settings_power_saving.h@b618abe7...`: Power-saving box/label interface over canonical power flags. Owner: Canonical Performance/Power Settings route.
- 5645 `Telegram/SourceFiles/settings/settings_privacy_controllers.cpp@749395e8...`: Granular privacy controllers: blocked peers; phone/added-by-phone; last-seen/read-time; group invites; calls/P2P; forwarding; fallback profile photo; voices; About/Birthday; gift policy; saved music. Owner: Canonical Account Privacy + Blocked Peers + Calls + Profile + Gift policy.
- 5646 `Telegram/SourceFiles/settings/settings_privacy_controllers.h@57ad9a89...`: Typed granular privacy controller contracts and save hooks. Owner: Canonical Account Privacy.
- 5647 `Telegram/SourceFiles/settings/settings_recent_searches.cpp@546bfb80...`: Bounded recent Settings-search identity list with bump/remove/load persistence. Owner: Canonical Settings/UniversalSearch recent history.
- 5648 `Telegram/SourceFiles/settings/settings_recent_searches.h@6a3ab9a3...`: Recent Settings-search list interface. Owner: Canonical Settings/UniversalSearch.
- 5649 `Telegram/SourceFiles/settings/settings_scale_preview.cpp@0c3063e5...`: Responsive display-scale preview with sample avatar/reply/message and screen-geometry clamping. Owner: Canonical Appearance/Display scale preview + design system.
- 5650 `Telegram/SourceFiles/settings/settings_scale_preview.h@b7d0f326...`: Display-scale preview show/update/hide interface. Owner: Canonical Appearance/Display Settings.
- 5651 `Telegram/SourceFiles/settings/settings_search.cpp@c665664e...`: Unified Settings search over canonical registry, FAQ and recent entries with ranking, keyboard navigation, deeplink/highlight and query restoration. Owner: Canonical UniversalSearch + Settings.
- 5652 `Telegram/SourceFiles/settings/settings_search.h@ae4f3d58...`: Settings search state/index/result/navigation interface. Owner: Canonical UniversalSearch + Settings.
- 5653 `Telegram/SourceFiles/settings/settings_type.h@ee546091...`: Shared typed AbstractSectionFactory identity alias. Owner: Canonical Settings routing type system.

All applicable responsibilities remain mapped-open; reading grants no unknown credit. Current contiguous read-through after the corrected batches is 5,653/16,123; unread 10,470; unknown 15,844; omitted 0. First unread is 5,654 `Telegram/SourceFiles/statistics/chart_lines_filter_controller.cpp@40c3b612...`.
