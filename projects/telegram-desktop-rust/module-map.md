# Telegram Research → Existing Fabushi Owner Map

Status: active  
Project: TDRP-001 Revision 4  
Parent: FBCP-001  
Purpose: map capabilities to current Fabushi ownership, not to source-equivalent target modules.

## Decision rule

For every capability:

1. research Telegram behavior;
2. inspect the current Fabushi exact HEAD;
3. list plausible existing owners;
4. select `existing_owner`;
5. define `absorption_plan`;
6. if and only if none fits, create `new_owner_proposal` with ADR.

## Initial hypotheses

| Telegram capability | Source areas to research | Existing Fabushi owner candidates |
| --- | --- | --- |
| Dialog list/folders/archive | dialogs, history, data | sidebar / conversation list |
| Private chat/history | history, data, api | conversation workspace / transcript |
| Message variants | history, data, chat_helpers | transcript model/cards |
| Composer/draft | history/composer/ui | existing composer / draft owner |
| Reply/quote/forward | history, data | transcript relation/provenance |
| Reactions/polls | data, history, ui | existing reaction/transcript owners |
| Groups/members | info, profile, data, boxes | Shared Room / group members |
| Admin/permissions | info, api, settings | room controller / permissions |
| Topics/forums | history, data, api | conversation thread/topic |
| Channels | data, api, history | room/conversation + channel mode |
| Search | dialogs, history, api | existing search/command surfaces |
| Scheduled send | history, api | Automations + composer |
| Attachments/files | media, storage, api | attachments/artifacts/resources |
| Upload/download | media, storage, api | resource lifecycle + transfer infra |
| Calls/screen share | calls, lib_webrtc, tgcalls | existing calls/Computer/realtime owners |
| Stickers/GIF/emoji | chat_helpers, ui, media | composer/rich transcript |
| Stories | data, api, ui | product shell or minimal Story owner |
| Bot interactions | inline_bots, api, ui | Agent/composer interaction |
| Mini Apps | lib_webview, api, ui | Plugins/MCP/Web capability |
| Notifications | platform, tray, settings | desktop/platform lifecycle |
| Settings/privacy | settings, data | settings/permissions |
| Local storage/recovery | storage, data | durable state/storage owners |
| Multi-device sync | mtproto/api/data | durable state + native sync infrastructure |
| Presence/typing | data/ui | Shared Room/conversation projection |
| Export | export, storage | artifacts/data lifecycle |
| i18n/accessibility | lang, ui, platform | existing product shell |
| install/update | core, platform, build | existing platform/release owners |

P0 must replace hypotheses with exact-head evidence.

## Broad new owners are forbidden

Do not propose:

- TelegramCore
- TelegramRuntime
- TelegramProvider
- TelegramMessaging
- CommunicationCore
- MessengerRuntime

A new owner must be capability-specific and minimal.

## Native network implications

Telegram protocol code may produce requirements for Fabushi's own:

- ordering
- duplicate suppression
- offline queue
- reconnect
- gap recovery
- multi-device sync
- media resume
- message identity
- push
- call signaling

Record these as infrastructure requirements, not as Telegram protocol compatibility requirements.
