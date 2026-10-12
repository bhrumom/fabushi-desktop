# Telegram source read: deterministic orders 4001-4100

Authority: accepted `telegramdesktop/tdesktop@22b352e866d0402505c07fa4ed21d75d7e4fb3db` / tree `94ae09469c816b350f60dc9ada1ff049323be8e7`. Exact object/size/type is bound to Source authority run `37872749663`, job `113634587680`, artifact `11591390327` (sha256 `9e8d9d297b9f957c5c60e069a3ede35c517d56e18662bcf7573946663b971d65`).

Orders 4001-4077 cover the Dialogs product surface: community rows, typed row keys including SavedSublist/Topic, indexed/name/date lists, pinned ordering and re-entrant cache-index protection, unread/badge state, fixed-on-top sorting, drag/filter/cache behavior, accessibility names/subitems, row quick actions, scoped search/from-user/tags/public-post pagination and flood/premium state, top-bar suggestions for auth/premium/credits/birthday/userpic, story list projection, topic/saved-sublist row projection, animated userpics, restore-window prompts and recent/top-peer keyboard/menu/drag/scroll interaction.

Orders 4078-4100 enter Media Editor: audio track attach/remove/trim/seek/playback/waveform/volume, brush/color/tool state, sticker/media panel controller composition, undo/redo event streams, crop geometry and bounds, keyboard legend/accessibility, modal layer key forwarding/background cache/fade timers and link-preview resolver/loading state.

Direct responsibility intersections include Recent Peers contract/lifecycle, Saved Messages recent/pinned/sublist UI behavior, Universal Search, Story UI lifecycle and canonical accessibility/performance obligations. These are source-understanding evidence only; no row is promoted to implemented or verified.

Accounting after this batch: `read_through=4100`, `unread=12020`, `unknown=15841`, `unknown_closed=279`, `omitted=0`.
