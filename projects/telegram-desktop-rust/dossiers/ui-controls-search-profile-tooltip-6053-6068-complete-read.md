# UI controls search/profile/tooltip 6053-6068 complete read

Exact accepted blobs 6,053-6,068 are read/decomposed in deterministic order; global unknown remains unchanged.

## Grouped search and picker

`tabbed_search.{cpp,h}` carries more than a search-box skin: trimmed live query and a separate 400 ms debounced query, immediate empty-query settlement, category selection that replaces typed text, Premium sentinel semantics, stable custom-emoji icon identity across group refreshes, RTL-aware wheel/drag scrolling, drag-threshold versus click arbitration, selected-group visibility, loading/cancel/back/search affordances, focus steal/return and responsive reserved-width/fade geometry. It maps to canonical SearchField/TextField/IconButton/Picker plus the existing emoji/media provider. A source-derived TabbedSearch owner is forbidden; full shipping composition remains mapped-open.

## Detail rows, title status and transient tooltip

`table_rows.{cpp,h}` composes reactive detail rows, inline/multiline action buttons, Avatar/profile navigation, premium/emoji badges, hidden-author fallback and transient anchored tooltips. `title_sub_widget.{cpp,h}` projects reactive title/status chrome with active-window and palette state. The existing canonical `SandTooltip` is extended source-neutrally rather than creating a Telegram tooltip: optional auto-dismiss, outside-press/Escape dismissal and focus-return policy are now expressible while existing passive hover defaults remain unchanged. `sand-tooltip.contract.test.ts` is wired into the Rust desktop runtime contract gate. Table/profile composition and real shipping consumers remain mapped-open.

## Localized amount input

`ton_common.{cpp,h}` defines fixed-precision amount parsing/formatting and IME behavior: 1e9 nanos, overflow fencing, locale/dot/comma separators, configurable fraction digits, full-width digit/punctuation normalization, cursor-preserving correction, canonical leading-zero behavior, signed/rounded rendering and grapheme-by-grapheme IME commit. Generic input semantics map to canonical TextField/numeric validation; product-domain monetary-unit applicability still requires an existing-owner audit and is not declared non-applicable by reading alone.

## Ephemeral media presentation

`ttl_media.{cpp,h,style}` is presentation over authoritative message/media expiry: server destroyAt is projected to a monotonic countdown, arc progress is clamped, particles pause for inactive windows, seconds round upward, view-once and fire-icon states are cached/rendered with DPR awareness, and tooltip layout is bounded. Canonical Status/Badge/Tooltip own presentation; authoritative expiration/deletion must remain in the existing message/media lifecycle.

## Avatar/profile media lifecycle

`userpic_button.{cpp,h,style}` spans Avatar open/change/choose/custom roles, peer/non-personal/custom sources, file/camera/clipboard/emoji-builder acquisition, set/suggest/reset policy, camera-versus-current-call arbitration, crop shape, profile-photo privacy navigation, reactive peer/download updates, upload progress/cancel/failure rollback, video-avatar streaming/loop/error fallback, hover change overlay and Saved/Replies/Notes/hidden-author projections. It maps to canonical Avatar/IconButton/Menu plus existing profile/privacy/media-picker/editor/Resource/streaming owners. No UserpicButton-derived parallel owner is accepted; the responsibility remains mapped-open.

## Warning tooltip lifecycle

`warning_tooltip.{cpp,h}` reinforces one transient Tooltip owner: default 2 s/configurable duration, anchored/custom placement, parent-resize reposition, replacement of the prior tooltip and deterministic animated/instant teardown. The canonical SandTooltip extension above is a partial production closure for that lifecycle, not a claim that all tooltip consumers are verified.

Accounting: **6,068 / 16,125** read; **10,057** unread; **15,846** unknown; **0** omitted. First unread: **6,069** `Telegram/SourceFiles/ui/controls/who_reacted_context_action.cpp@6851dee06d13ac8f1f771cb3384c9367b5a53c75`. Reading alone grants no closure credit.
