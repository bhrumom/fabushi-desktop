# Telegram source read: deterministic orders 4401-4500

Authority: accepted `telegramdesktop/tdesktop@22b352e866d0402505c07fa4ed21d75d7e4fb3db` / tree `94ae09469c816b350f60dc9ada1ff049323be8e7`; exact blob/size/type comes from Source authority run `37872749663`, job `113634587680`, artifact `11591390327` (sha256 `9e8d9d297b9f957c5c60e069a3ede35c517d56e18662bcf7573946663b971d65`).

This batch covers the History list widget and message projection; keyboard text selection; group-call/member surfaces; paid reaction feedback; pinned bar/section/tracker; read metrics; reply/reply button; requests/scheduled sections; self-forward tagging; send-action/service-message/sponsored/sticker feedback; subsection tabs and summary/top bar; recent/top-peer selector; transcription and translation lifecycles; message view/webpage preview/welcome state; and transcript media for birthday/call/community/contact/custom emoji/dice/document/ephemeral/file/game/GIF/giveaway/Gram transfer/invoice/large emoji.

Direct mapped intersections include History state/lifecycle and Recent Peers contract/lifecycle. These rows deepen source state-machine and resource-lifetime evidence only; no implementation or release promotion is taken.

Accounting: `read_through=4500`, `unread=11620`, `unknown=15841`, `unknown_closed=279`, `omitted=0`.
