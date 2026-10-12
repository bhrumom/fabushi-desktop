# Telegram source read: deterministic orders 3901-4000

Authority: accepted `telegramdesktop/tdesktop@22b352e866d0402505c07fa4ed21d75d7e4fb3db` / tree `94ae09469c816b350f60dc9ada1ff049323be8e7`; exact object/size/type is bound to Source authority run `37872749663`, job `113634587680`, artifact `11591390327` (sha256 `9e8d9d297b9f957c5c60e069a3ede35c517d56e18662bcf7573946663b971d65`), member `upstream-recursive-inventory.json`.

Exact source reading covers Peer identity/bot commands/colors/values; Photo/media and Poll state; Premium limits/subscriptions and PTS update ordering; Replies/reply preview/report; Saved Messages, saved sublists and saved music; Search calendar/controller and send actions; the Data Session composition root; Shared Media sparse storage; Star Gifts and statistics; Stories, Story IDs/content and streaming; subscriptions, Thread/History and Todo state; typed data values, message cursors and reactive unread values; User/profile names/photos; wallpaper; WebPage/IV article content; notification defaults/per-peer/thread/forum/community mute/sound/volume lifecycle; country bounds; custom emoji/sticker set async cache/load/repaint lifecycle; and Community dialog-list/reactive requestable-list UI state.

Direct mapped-responsibility intersections in this batch include Saved Messages parent/request/pagination/pin-empty/active-subsection/cleanup/recent-order/unread/membership, Shared Media Storage contract/lifecycle, Data Types value contracts, History lifecycle/state, Notification Manager contract/lifecycle, and IV Article/WebPage content responsibilities. Those intersections are source-understanding evidence only: each row remains mapped/open until current Fabushi shipping entrypoints, production behavior, tests and exact-head artifacts prove implementation.

State/lifecycle observations include request batching and stale-page fencing; sparse slice/range updates; PTS ordering; Story polling/read/pin/expiry; Thread unread/mentions/reactions/polls and notification queues; notification mute timers/default inheritance/sound caching; Custom Emoji resolve→cache lookup→load/cancel→repaint; Stickers recent/favorite/update timers/install events; and Community linked-peer reactive rebuild/selection/ripple/count/open lifecycles.

No generic “Data” owner is introduced. Each row is bound to the current canonical product-domain owner, with existing canonical Fabushi UI components reused where applicable. Telegram C++/MTProto structure remains reference behavior only.

Accounting: `read_through=4000`, `unread=12120`, `unknown=15841`, `omitted=0`, `unknown_closed=279`.

Read-complete does not promote `implementation_status`, `coverage_matrix_status`, `release_gate_status`, or independent release acceptance.
