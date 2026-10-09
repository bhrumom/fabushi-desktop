# Telegram Desktop source orders 1001–1016 — complete read

- Accepted upstream: `telegramdesktop/tdesktop@22b352e866d0402505c07fa4ed21d75d7e4fb3db`
- Upstream tree: `94ae09469c816b350f60dc9ada1ff049323be8e7`
- Exact-head evidence: run `37856138232`, job `113580961853`, artifact `11584172080`, digest `sha256:e6a309a9bce1f758ad1ba11b9408ccfb6d9d1142885a756c408e58f4b9db5dc2`, tested HEAD `48dd7f5eedb4838bf9a3bb7164cdd6b37c6dceb0`.
- Evidence files: `source-binary-evidence-1001-1100.txt`, `source-consumer-reachability-1001-1100.txt`, `source-consumer-trace-1001-1100.txt`.

## Responsibility

Orders 1001–1016 finish the built-in conversation-folder type marker family. The assets are presentation inputs for the canonical filter predicates (channels, contacts, groups, exclude-muted, non-contacts, exclude-read). They do not own filter state. Fabushi must preserve the predicate semantics through its canonical conversation collection/filter owner and reuse `Tabs`, `Menu`, and `ConversationRow`; no Telegram-named component or Telegram artwork is introduced.

All 16 entries have a tightened exact accepted-tree consumer in `Telegram/SourceFiles/window/window.style`; there are no reachability-open entries in this slice. The next distinct responsibility starts at order 1017 (`folders_unmuted`).

## Exact source evidence

| order | source | blob | bytes | file type | exact consumer |
|---:|---|---|---:|---|---|
| 1001 | `Telegram/Resources/icons/folders/folders_type_channels@3x.png` | `3e8d01368cce006e7934a98718479bc44ef20d98` | 1402 | PNG image data, 108 x 108, 8-bit/color RGBA, non-interlaced | `Telegram/SourceFiles/window/window.style:309:windowFilterTypeChannels: icon {{ "folders/folders_type_channels", historyPeerUserpicFg }};` |
| 1002 | `Telegram/Resources/icons/folders/folders_type_contacts.png` | `1cfb2a57518a49122229b5b16b35ddef4f0299e6` | 372 | PNG image data, 36 x 36, 8-bit/color RGBA, non-interlaced | `Telegram/SourceFiles/window/window.style:306:windowFilterTypeContacts: icon {{ "folders/folders_type_contacts", historyPeerUserpicFg }};` |
| 1003 | `Telegram/Resources/icons/folders/folders_type_contacts@2x.png` | `6c81c14ea88ba5cb172f80797e8bb5c9057401d6` | 699 | PNG image data, 72 x 72, 8-bit/color RGBA, non-interlaced | `Telegram/SourceFiles/window/window.style:306:windowFilterTypeContacts: icon {{ "folders/folders_type_contacts", historyPeerUserpicFg }};` |
| 1004 | `Telegram/Resources/icons/folders/folders_type_contacts@3x.png` | `f3ac2f12a2609356cf61050b4109bbc14bf19008` | 1223 | PNG image data, 108 x 108, 8-bit/color RGBA, non-interlaced | `Telegram/SourceFiles/window/window.style:306:windowFilterTypeContacts: icon {{ "folders/folders_type_contacts", historyPeerUserpicFg }};` |
| 1005 | `Telegram/Resources/icons/folders/folders_type_groups.png` | `c66addebf32d10ee2586aeaa1c674a9d941cd68c` | 508 | PNG image data, 36 x 36, 8-bit/color RGBA, non-interlaced | `Telegram/SourceFiles/window/window.style:308:windowFilterTypeGroups: icon {{ "folders/folders_type_groups", historyPeerUserpicFg }};` |
| 1006 | `Telegram/Resources/icons/folders/folders_type_groups@2x.png` | `114156e547ff75bb975e716bbc744f601c7ba27b` | 984 | PNG image data, 72 x 72, 8-bit/color RGBA, non-interlaced | `Telegram/SourceFiles/window/window.style:308:windowFilterTypeGroups: icon {{ "folders/folders_type_groups", historyPeerUserpicFg }};` |
| 1007 | `Telegram/Resources/icons/folders/folders_type_groups@3x.png` | `fcaffe853fceb48b581b6904ce80f7bf5d66baf3` | 1712 | PNG image data, 108 x 108, 8-bit/color RGBA, non-interlaced | `Telegram/SourceFiles/window/window.style:308:windowFilterTypeGroups: icon {{ "folders/folders_type_groups", historyPeerUserpicFg }};` |
| 1008 | `Telegram/Resources/icons/folders/folders_type_muted.png` | `92e3b1adc70b7559fb98d11171307b0e7050f54e` | 576 | PNG image data, 36 x 36, 8-bit/color RGBA, non-interlaced | `Telegram/SourceFiles/window/window.style:311:windowFilterTypeNoMuted: icon {{ "folders/folders_type_muted", historyPeerUserpicFg }};` |
| 1009 | `Telegram/Resources/icons/folders/folders_type_muted@2x.png` | `1dd51e26ca14bbec1a864b48acdee59f205f6ccc` | 1318 | PNG image data, 72 x 72, 8-bit/color RGBA, non-interlaced | `Telegram/SourceFiles/window/window.style:311:windowFilterTypeNoMuted: icon {{ "folders/folders_type_muted", historyPeerUserpicFg }};` |
| 1010 | `Telegram/Resources/icons/folders/folders_type_muted@3x.png` | `0f0f81dbb4e36ed1580fcb9f10f1134d49996e96` | 2101 | PNG image data, 108 x 108, 8-bit/color RGBA, non-interlaced | `Telegram/SourceFiles/window/window.style:311:windowFilterTypeNoMuted: icon {{ "folders/folders_type_muted", historyPeerUserpicFg }};` |
| 1011 | `Telegram/Resources/icons/folders/folders_type_noncontacts.png` | `213633ac0ba338618801089897a2c0a455aa2701` | 573 | PNG image data, 36 x 36, 8-bit/color RGBA, non-interlaced | `Telegram/SourceFiles/window/window.style:307:windowFilterTypeNonContacts: icon {{ "folders/folders_type_noncontacts", historyPeerUserpicFg }};` |
| 1012 | `Telegram/Resources/icons/folders/folders_type_noncontacts@2x.png` | `f1f7695b67c5230298febe9ab8fccec2f3e29622` | 1252 | PNG image data, 72 x 72, 8-bit/color RGBA, non-interlaced | `Telegram/SourceFiles/window/window.style:307:windowFilterTypeNonContacts: icon {{ "folders/folders_type_noncontacts", historyPeerUserpicFg }};` |
| 1013 | `Telegram/Resources/icons/folders/folders_type_noncontacts@3x.png` | `f5a58c2693a711478fc616b384a1dd93ba44c12f` | 1951 | PNG image data, 108 x 108, 8-bit/color RGBA, non-interlaced | `Telegram/SourceFiles/window/window.style:307:windowFilterTypeNonContacts: icon {{ "folders/folders_type_noncontacts", historyPeerUserpicFg }};` |
| 1014 | `Telegram/Resources/icons/folders/folders_type_read.png` | `56dee358a15890aea03716106df798cdb9375b8d` | 768 | PNG image data, 36 x 36, 8-bit/color RGB, non-interlaced | `Telegram/SourceFiles/window/window.style:313:windowFilterTypeNoRead: icon {{ "folders/folders_type_read", historyPeerUserpicFg }};` |
| 1015 | `Telegram/Resources/icons/folders/folders_type_read@2x.png` | `d88d4e919e80bdb867e36275d61c4c753636f388` | 1434 | PNG image data, 72 x 72, 8-bit/color RGB, non-interlaced | `Telegram/SourceFiles/window/window.style:313:windowFilterTypeNoRead: icon {{ "folders/folders_type_read", historyPeerUserpicFg }};` |
| 1016 | `Telegram/Resources/icons/folders/folders_type_read@3x.png` | `3cddeb6a6a0f5aa53db76645f13067d92bb0a6f3` | 2093 | PNG image data, 108 x 108, 8-bit/color RGB, non-interlaced | `Telegram/SourceFiles/window/window.style:313:windowFilterTypeNoRead: icon {{ "folders/folders_type_read", historyPeerUserpicFg }};` |

## Accounting after this slice

- deterministic read-through: **1,016 / 16,120**
- unread: **15,104**
- unknown: **16,033** (unchanged; source-read credit does not falsely close production responsibility evidence)
- omitted: **0**
- reachability-open in 1001–1016: **0**
