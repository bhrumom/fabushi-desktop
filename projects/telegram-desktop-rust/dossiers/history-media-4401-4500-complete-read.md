# Telegram source read: deterministic orders 4401-4500

Authority: live accepted telegramdesktop/tdesktop@42f8a36d / tree 6ac9bbc1. Order 4413 history_view_message.cpp re-read and rebound to 5152d85e89b6da554095d85a1a16d96d4f969c3d; MarkdownArticleBubbleEdges is now explicit. Prior artifacts are historical-only.

This batch covers the History list widget and message projection; keyboard text selection; group-call/member surfaces; paid reaction feedback; pinned bar/section/tracker; read metrics; reply/reply button; requests/scheduled sections; self-forward tagging; send-action/service-message/sponsored/sticker feedback; subsection tabs and summary/top bar; recent/top-peer selector; transcription and translation lifecycles; message view/webpage preview/welcome state; and transcript media for birthday/call/community/contact/custom emoji/dice/document/ephemeral/file/game/GIF/giveaway/Gram transfer/invoice/large emoji.

Direct mapped intersections include History state/lifecycle and Recent Peers contract/lifecycle. These rows deepen source state-machine and resource-lifetime evidence only; no implementation or release promotion is taken.

Accounting: `read_through=4500`, `unread=11620`, `unknown=15841`, `unknown_closed=279`, `omitted=0`.
