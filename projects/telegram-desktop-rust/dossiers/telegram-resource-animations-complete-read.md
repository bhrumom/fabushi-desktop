# Telegram animation resources — rolling complete-read dossier

Accepted upstream: `telegramdesktop/tdesktop@aac515c5408015231a273c80ab4c4b33815e63ab`

## Entry 89 — ban.tgs

Path: `Telegram/Resources/animations/ban.tgs`  
Blob: `1ac1aaa8a493a99160d772a7739224a08435cc68`  
Read status: complete after gzip decompression.

The exact asset is a TGS/Lottie JSON document (v5.7.1), 512x512, 60 fps, frames 0–120. Its primary layer is named `Hand` and the vector group is named `Block-user`; it is included by `Telegram/Resources/qrc/telegram/animations.qrc` under alias `ban.tgs`.

Disposition is intentionally still open: the asset clearly carries an applicable block/ban visual meaning, but exact runtime consumer/state ownership has not yet been established from production source. It therefore decreases unread but does **not** decrease unknown. No Telegram-specific Fabushi component is created; once the consumer is identified it must map to an existing canonical Fabushi surface where possible and receive visual/a11y/light-dark/responsive evidence.

Accounting after entry 89:
- full-read: `149`
- unread: `15,663`
- unknown: `15,730`
- omitted: `0`

## Entries 90–98 — blocked/cake/call/camera/change-number and chat animation resources

All nine exact TGS blobs were gzip-decompressed in a streaming read (no persistent build/checkout artifact) and their Lottie metadata was inspected. Consumer searches identified the production references listed in `inventory/source-dispositions.json`.

- 90 `blocked_peers_empty.tgs`: blocked-peers/report empty-state illustration.
- 91 `cake.tgs`: birthday suggestion local TGS sticker.
- 92 `call_rate.tgs`: call-rating star/burst feedback.
- 93 `camera_outline.tgs`: edit-contact photo/avatar affordance.
- 94 `change_number.tgs`: Settings change-number animation.
- 95 `chat/sparkles_emoji.tgs`: Compose AI decorative custom emoji.
- 96 `chat/video_to_voice.tgs`: record-mode video-to-voice transition.
- 97 `chat/voice_to_video.tgs`: record-mode voice-to-video transition.
- 98 `chat/white_flag_emoji.tgs`: Compose AI neutral-style custom emoji.

These entries are read/decomposed but intentionally remain implementation-open until the referenced production consumers reach their own deterministic full-read turn. Thus unread falls while unknown does not.

Accounting: full-read `158`, unread `15,654`, unknown `15,730`, omitted `0`.

## Entries 99–106 — chat link, storage/filter, and cloud-password animations

These eight TGS resources were streamed and gzip-decompressed without persistent checkout/build artifacts; exact Lottie metadata and QRC inclusion were inspected. Production consumers were identified for cache clearing, cloud filters, and all cloud-password assets. `chat_link.tgs` has clear link-in-bubble semantics but no exact asset-load consumer beyond QRC was found in the current search, so it remains especially explicit as consumer-open.

No entry receives implementation credit before its referenced production source is fully read. Accounting: full-read `166`, unread `15,646`, unknown `15,730`, omitted `0`.



## Entries 107–126 — cocoon, collectibles, craft, dice, discussion and peer-edit animations

All 20 exact TGS blobs were streamed from telegramdesktop/tdesktop@aac515c5408015231a273c80ab4c4b33815e63ab, gzip-decompressed and structurally inspected without a persistent checkout or build artifact. QRC inclusion was confirmed for each. Exact production consumers were traced for cocoon translation UI, collectible phone/username info and internal routes, star-gift craft progress/failure, diamond currency/stake UI, dice/slot local sticker generation, discussion-link management, direct-message management, and topics/forum management.

No resource receives Fabushi implementation credit merely because its asset was read: each row remains implementation-open until the referenced production source reaches deterministic full-read order and its canonical Fabushi owner, UI/UX, failure states and focused tests are bound. Accounting: full-read `186`, unread `15,626`, unknown `15,730`, omitted `0`.


## Entries 127–146 — topics layouts, business/security empties and photo-editor tools

Twenty more exact TGS blobs were streamed and gzip-decompressed without persistent checkout/build artifacts. Exact consumer references were proven for topics list/tabs, filters, business greeting/working-hours/location, hello status, local passcode, forbidden media, gift/profile empty state, dialogs/search empty states, passkeys and phone visuals. Palette plus the five photo-editor tool assets remain explicitly consumer-open where focused exact-name search did not prove the shipping loader; they are not closed by inference.

Accounting: full-read `206`, unread `15,606`, unknown `15,730`, omitted `0`. No new resource row receives production implementation credit before its referenced consumer is fully read and mapped to Fabushi.


## Entries 147–166 — profile transitions, RTMP/search/privacy, star reactions and statistics

Twenty exact TGS blobs were streamed and gzip-decompressed without persistent checkout/build artifacts. Exact consumers were proven for profile mute/unmute, RTMP, search empty state, privacy/premium last-seen/read-time, business away-message sleep, star-referral link and statistics visuals. The photo-editor text-align asset, photo suggestion, robot, chat-automation and star-reaction segment loaders remain explicitly consumer-open because focused basename searches did not prove their shipping composition sites.

Accounting: full-read `226`, unread `15,586`, unknown `15,730`, omitted `0`. No applicable resource is marked implemented or unknown-closed solely from asset semantics.


## Entries 167–186 — statistics, swipe actions and toast feedback

Twenty exact TGS blobs were streamed/decompressed. Statistics boosts/earn, stop, and toast resources were tied to exact production consumers. Swipe-action archive/delete/disabled/mute/pin/read/unarchive/ungroup/unmute/unpin/unread assets were fully read, but a focused exact-source scan did not yet prove the runtime registry/path builder, so those rows remain explicitly consumer-open; generic words such as `mute`, `read` or `delete` are not accepted as asset evidence.

Accounting: full-read `246`, unread `15,566`, unknown `15,730`, omitted `0`.
