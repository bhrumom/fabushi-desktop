# Orders 505-511 — TTL, ephemeral and managed-Bot chat resources

Accepted source authority: `telegramdesktop/tdesktop@63a1831728147e941ad2dc8939879cf5afe419c7`.

This closes exact source attribution for seven resource blobs only. It does **not** claim TTL/once-media expiry, replay prevention, durable persistence, service settlement, managed-Bot behavior, Bot regression, visual parity, accessibility, packaged E2E, or release acceptance as implemented/verified.

## 505-507 audio_once

Exact accepted-tree consumers:

- `Telegram/SourceFiles/ui/controls/ttl_media.style@1d8cce39c28e436ea6b88ea3d52cd4d64038c002::ttlMediaOnceIcon`
- `Telegram/SourceFiles/history/view/controls/history_view_voice_record_bar.style@17089e2bd00902465851734fb6e3ddfe2ba328c1::historyRecordVoiceOnceInactive`
- `Telegram/SourceFiles/ui/chat/chat_style.style@1bc3cf58aee61bc79304973c5d5d2c3736ffac2d::historyVideoMessageTtlIcon/historyVideoMessageTtlIconSelected`

These prove once/TTL voice/video presentation responsibility. Canonical owner mapping is Message/Resource media lifecycle plus Composer/Transcript projection; no Telegram media store is introduced.

## 508 code_tags

Exact accepted-tree consumers:

- `Telegram/SourceFiles/history/view/history_view_about_view.style@d298dd4a1a3bd2d370f9d6ab5d4ea96ba5cf3ce3::managedBotCodeIcon`
- `Telegram/SourceFiles/boxes/boxes.style@3f19208ddc0be0bf4faf6042a137be00a114bfec::managedBotIconEmoji`

This proves managed-Bot presentation responsibility only. Canonical Agent/Bot/permissions/command owners remain authoritative.

## 509-511 ephemeral_badge

Exact accepted-tree consumers:

- `Telegram/SourceFiles/ui/chat/chat_style.style@1bc3cf58aee61bc79304973c5d5d2c3736ffac2d::historyEphemeralIconInSelected/historyEphemeralIconOut/historyEphemeralIconOutSelected/historyEphemeralInvertedIcon`
- `Telegram/SourceFiles/chat_helpers/chat_helpers.style@1fff0edd34ecef94aa441e9e05205ddb5f6e6d86::mentionEphemeralIcon/mentionEphemeralIconOver`

These prove transcript/mention ephemeral-state presentation. The canonical Message lifecycle must still independently prove expiry, persistence, anti-replay, multi-device semantics and temporal E2E.
