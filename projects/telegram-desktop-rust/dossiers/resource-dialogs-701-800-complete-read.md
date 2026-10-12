# Telegram dialog resources 701–800 — complete source read

Upstream: `telegramdesktop/tdesktop@22b352e866d0402505c07fa4ed21d75d7e4fb3db` / tree `94ae09469c816b350f60dc9ada1ff049323be8e7`.

Evidence: Fabushi Desktop `f847e1e194541667f85fd5228d4328aefb0e8738`; GitHub Actions run `37852738180`; source-authority job `113569832774`; artifact `11582408341`; digest `sha256:ec0e29a0994932273502f03b5c495e61811001a23b7906183ee8bbc026629ab1`.

## Evidence contract
`source-binary-evidence-701-800.txt` re-hashes every order 701–800 file from the accepted upstream checkout and records the exact blob SHA, byte size and file type. `source-consumer-reachability-701-800.txt` records accepted-tree named-consumer probes and `source-consumer-trace-701-800.txt` records shipping/style consumer traces. Each source-disposition row stores the exact blob, byte size, file type, consumer evidence, responsibility, upstream owner, Fabushi canonical owner and fail-closed disposition.

## Responsibility groups
- 701–703: transcript voice-to-text collapse/expand -> existing TranscriptEntry/transcription-state owner.
- 704–709: color-editor slider orientation/handles -> canonical Settings appearance controls; production parity remains open.
- 710–711: community request hidden/member state -> ParticipantRow/ProfileSection community-management owner.
- 712–729: connecting frame/body pieces -> one canonical transport-status projection via Status/Banner; Coordinator/Host remain transport owners.
- 730–735: contacts alphabet/online sort modes -> canonical participant collection sorting.
- 736–741: hidden-author / personal-notes avatars -> Avatar + ConversationRow identity projection.
- 742–756: Bot/calendar/clear/channel/chat row semantics -> existing Bot owner plus canonical SearchField/IconButton/Popover/ConversationRow components.
- 757–763: call/video-call/mention/poll/reaction row previews -> typed ConversationRow preview projection over canonical event truth.
- 764–775: downloads, forum/topic, lock/unlock -> canonical file/topic/security surfaces; no Telegram-specific roots.
- 776–778: `dialogs_mention` has no accepted-tree named consumer and remains reachability-open. No mention capability is inferred from the filename.
- 779–799: menu unread badge/dot, mini media play, mute/silent and pinned state -> Menu/Badge, ConversationRow, TranscriptEntry and existing media/notification/pin owners.
- 800: premium marker -> canonical Badge/Status over entitlement state. Its scale family continues at deterministic orders 801–802, so this read credit does not claim family closure at 800.

## Accounting
Read-through advances 700 -> 800. Unread falls 15,420 -> 15,320. Unknown stays 16,033 and omitted stays 0. Source-read evidence does not fabricate production verification, and all product/visual/a11y/release gates remain fail-closed until exact-head evidence closes them.
