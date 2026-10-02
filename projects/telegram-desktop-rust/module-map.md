# Telegram Source Research → FBCP Capability Map

Status: active  
Project: TDRP-001  
Parent product: FBCP-001  
Purpose: research navigation and product-domain placement, not file-to-file migration.

## Rules

For every Telegram capability, answer:

1. which upstream areas implement it;
2. what user/network behavior must be preserved;
3. which FBCP product domain owns the generic behavior;
4. which Telegram Provider state owns network-specific truth;
5. which semantics require typed Telegram extensions;
6. what C++ production logic must become Rust;
7. whether any data crosses the InteractionGateway.

## Initial map

| Telegram capability | Main source research areas | FBCP destination | Provider-specific responsibility |
| --- | --- | --- | --- |
| App/account lifecycle | main, core, intro, settings | Identity / Accounts / Product Shell | Telegram auth/session |
| MTProto/TL | mtproto, api, lib_tl, codegen | Communication Provider | wire/session/DC |
| Updates/consistency | api, data, history, mtproto | Conversation/Messaging projection | pts/qts/seq/difference |
| Contacts/peers | data, api, profile | Identity / Contacts | Telegram peer truth |
| Dialogs | dialogs, history, data | Inbox / Conversations | folder/archive/read semantics |
| Messages | history, data, api, chat_helpers | Messaging / Conversation | Telegram message/update identity |
| Groups/channels/topics | info, profile, api, data, boxes | Conversations / Participants / Permissions | Telegram roles/channel/topic semantics |
| Search | dialogs, history, api | Search | provider search/index semantics |
| Media transfer | media, storage, api | Media | CDN/file references/transfer |
| Playback/recording | media, platform, ffmpeg | Media / Product Shell | codec/device/provider metadata |
| Calls | calls, lib_webrtc, tgcalls, platform | Calls | Telegram signaling/call semantics |
| Stickers/GIF/emoji | chat_helpers, ui, media, data | Messaging / Media / Composer | Telegram asset/custom emoji semantics |
| Stories | data, api, history, ui | Conversation extensions / Media | Telegram story lifecycle |
| Bots/Mini Apps | inline_bots, iv, api, lib_webview | Conversation extensions / WebView | Bot/Mini App contracts |
| Premium/Stars/Business | api, data, payments, boxes | Product extensions / Payments | Telegram entitlement/payment semantics |
| Settings/privacy | settings, data, platform | Settings / Permissions | Telegram account/privacy flags |
| Notifications/tray | platform, window, tray | Notifications / Product Shell | Telegram mute/read/provider data |
| Local storage | storage, lib_storage, data | Product Storage + Provider Storage | Telegram compatibility/recovery |
| i18n/accessibility | lang, ui, platform | Product Shell | provider text/assets where required |
| Build/update/platform | core, platform, cmake, tools | Release / Platform | Telegram-specific integration needs |

P0 must expand this until every fixed-source product area is classified.

## C++ classification

Each C++ source area is classified as:

- product logic
- protocol
- state machine
- UI behavior
- platform policy
- build tool
- third-party
- test/reference
- unreachable-with-evidence

The first six categories require Rust replacement when they carry Telegram/desktop-app product responsibility.

## Target ADRs

Telegram research feeds, but does not own, these FBCP ADRs:

- FBCP domain model
- Identity
- Conversation
- Provider interface
- Telegram binding
- InteractionGateway
- Permissions
- Data ownership
- Unified UX
- Storage
- Search
- Media/Calls
- Terms/Privacy

Provider-internal ADRs may additionally cover protocol runtime, sync, native dependency replacement and provider storage.

## Research dossier additions

Every dossier must now include:

- FBCP destination
- Telegram provider owner
- typed extension requirements
- InteractionGateway implications
- data/terms classification
