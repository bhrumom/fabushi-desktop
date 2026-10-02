# TDRP/FBCP P0 — Telegram Top-Level Capability Graph (Discovery Pass)

Status: discovery pass; recursive closure and behavior research incomplete  
Frozen upstream: `telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677`  
Target owner snapshot: PR #20 `d7592663043d809fbaf40af7aa006a4e351082ea`

This graph records product capability domains observed in the frozen Telegram Desktop source tree. It is intentionally not marked research-complete: the recursive submodule/download/generated/resource closure is still open, so P0 cannot yet claim that no unknown capability domain remains.

| Capability domain | Representative frozen source regions | Initial Fabushi owner candidates |
| --- | --- | --- |
| startup / app lifecycle | `SourceFiles/main*`, `core`, `window`, `platform` | existing product shell + Electron/platform lifecycle |
| account / auth / sessions / multi-account | `intro`, `api/api_authorizations.*`, `data/data_authorization.h`, `storage/storage_account.*` | existing account/auth/session owners; native Human identity contract still unresolved |
| contacts / identity / presence | `data/data_peer*`, `api/api_peer_search.*`, participant/status code | Shared Room/member + account identity + minimal presence infrastructure if required |
| dialog list / folders / archive / pinning | `dialogs/*`, `data/data_chat_filters.*`, `data/data_folder.*`, `storage/storage_folder_archive.*` | existing sidebar / conversation list |
| private messaging / history | `history/*`, `data/data_messages.*`, `data/data_history_messages.*`, `api/api_sending.*` | existing conversation workspace + transcript; native messaging infrastructure below |
| reply / quote / forward provenance | `history/history_item*`, `data/data_reply_preview.*`, replies/thread structures | existing transcript relation/thread owner |
| edit / delete lifecycle | `api/api_editing.*`, history/data message state | existing transcript/message lifecycle |
| reactions | `data/data_message_reactions.*`, `api/api_who_reacted.*`, info reaction list | existing reaction/transcript owner |
| polls / rich message variants | `data/data_poll*`, `api/api_polls.*`, `poll/*` | existing typed transcript/card system |
| drafts / composer / voice input | `data/data_drafts.*`, history composer/view code | existing composer/draft owner |
| scheduled / silent send / send progress | `api/api_sending.*`, `api/api_send_progress.*`, scheduled-history code | Automations + existing composer/send action |
| groups / members / admin | `data/data_chat.*`, `data/data_groups.*`, `api/api_chat_participants.*`, `info/members` | existing Shared Room/group/member + permissions |
| channels / broadcast | `data/data_channel.*`, history/dialog/api channel code | existing room/conversation model + broadcast policy |
| topics / forums / threads | `data/data_forum.*`, `data/data_forum_topic.*`, `data/data_thread.*` | existing conversation/thread model |
| search | `api/api_messages_search.*`, `data/data_search_controller.*`, dialog search code | existing search/command/find-in-chat owners |
| files / upload / download | `storage/file_upload.*`, `storage/file_download*`, `data/data_document*` | existing attachment/artifact/resource lifecycle + transfer infrastructure |
| images / video / audio / voice / streaming | `media/*`, `data/data_streaming.*`, media preload/rotation | existing resources/transcript + minimal media transfer/cache infra |
| stickers / GIF / emoji / custom emoji | `chat_helpers`, data/media/UI helpers | existing composer + rich transcript/cards |
| Stories | `data/data_stories.*`, `data/data_story.*`, `media/stories`, `info/stories` | product shell or minimal Story owner after owner analysis |
| notifications / tray / badge | platform notification manager, `tray*`, settings notifications | existing desktop notification/tray owners + push infrastructure |
| privacy / safety / blocking | settings privacy controllers, `api/api_user_privacy.*`, blocked-peers | existing settings/permissions plus native identity policy |
| local lock / credential / WebAuthn | cloud password, `webauthn`, platform WebAuthn | existing auth/settings/security boundaries |
| Bots / inline interactions | `inline_bots/*`, `api/api_bot.*`, peer bot commands | existing Agent/composer interaction primitives; no Telegram Bot runtime dependency |
| Mini Apps / WebView concepts | inline bot attach web view + `lib_webview` gitlink | existing Plugins/MCP/Web capability where semantically appropriate |
| Premium / subscriptions / credits / Stars | premium/credits data + API/settings | existing settings/product shell; domain owner requires business decision |
| gifts | `data/data_star_gift.*`, peer gifts | product shell; owner unresolved until full behavior research |
| business features | `data/business`, `settings/business` | existing product/workflow/settings owners by capability, not a single business subsystem by default |
| payments | `payments`, credits/earn code | explicit payments owner only after ADR/security research if accepted |
| Passport / security identity flows | `passport`, `webauthn`, auth code | existing auth/security boundaries; applicability still to be decided |
| export / data lifecycle | `export`, storage serialization | existing artifacts/data-lifecycle owner |
| local storage / migration / corruption recovery | `storage/*`, serialization/account/domain/facade, file locks | existing durable-state/storage/recovery owners |
| themes / language / RTL / IME / accessibility | `lang`, `ui`, platform, history/dialog accessibility sources | existing product shell/platform/accessibility owners |
| calls / video / screen sharing | `calls/*`, `calls/group`, `lib_webrtc`, `tgcalls` | Computer/platform/realtime candidates; call-session/signaling owner unresolved |
| reconnect / sleep-wake / proxy / network transition | `mtproto`, core/network/platform lifecycle | native sync/network infrastructure + existing platform lifecycle; no MTProto runtime dependency |
| multi-device sync / ordering / duplicate suppression / gap recovery | update/session/data/mtproto state machinery | existing durable state + minimal native sync/messaging infrastructure |
| large histories / pagination / stale updates | history/data sparse-id/search/update code | existing transcript pagination/storage + sync infrastructure |
| update / install / rollback | core/platform/build/update code | existing Electron/platform update owners |
| communities / managed communities | `api/api_communities.*`, peer/community boxes, `info/community*` | extend Shared Room/group/member/admin owners after behavior research |
| Compose AI / AI tone helpers | `api/api_compose_with_ai.*`, `boxes/compose_ai_box.*`, create/preview AI tone boxes | existing composer + Agent capability; preserve typed Agent flow rather than a Telegram-specific AI subsystem |
| rich tasks / todo lists | `api/api_rich_tasks.*`, `api/api_todo_lists.*`, todo-list editor boxes | existing task/automation/transcript typed-event owners |
| ringtones / notification sounds | `api/api_ringtones.*`, ringtone settings/boxes, `Resources/sounds` | existing notification/settings/resource owners |
| self-destruct / expiry | `api/api_self_destruct.*`, self-destruction UI, message lifecycle | existing transcript/message lifecycle + permissions/settings |
| sensitive-content policy | `api/api_sensitive_content.*`, privacy/settings surfaces | existing settings/permissions/safety policy |
| statistics / analytics surfaces | `api/api_statistics*.*`, `statistics/*`, channel statistics info | existing product analytics/room info surfaces; applicability requires product decision |
| usernames / websites / links | `api/api_user_names.*`, `api/api_websites.*`, deep-link/local-url handlers | existing identity/profile + product shell/navigation/security |
| media/photo/video editor | `editor/*`, media editor scene/controllers/video | existing attachment/resource composer flow; minimal editor capability only if applicable |
| Instant View / rich document rendering | `iv/*` | existing artifact/web/rich-content rendering owner |
| support / moderation / report flows | `support/*`, report/moderation boxes, blocked peers | existing settings/permissions/safety/support surfaces |
| TDE2E / end-to-end encryption research | `tde2e/*` | native identity/security/network infrastructure; protocol requirements only, no Telegram wire dependency |
| commerce/security long tail discovered outside list | data business/credits/gifts, passport/payments/webauthn and future recursively discovered regions | resolve one capability at a time to existing owners; no broad new subsystem |

## Discovery evidence

The frozen source root contains product areas including `api`, `calls`, `chat_helpers`, `core`, `data`, `dialogs`, `export`, `history`, `info`, `inline_bots`, `intro`, `lang`, `media`, `mtproto`, `passport`, `payments`, `platform`, `poll`, `profile`, `settings`, `storage`, `ui`, `webauthn`, and `window`.

Representative exact paths observed at the frozen commit include:

- `Telegram/SourceFiles/api/api_sending.cpp`
- `Telegram/SourceFiles/api/api_editing.cpp`
- `Telegram/SourceFiles/api/api_messages_search.cpp`
- `Telegram/SourceFiles/data/data_messages.cpp`
- `Telegram/SourceFiles/data/data_drafts.cpp`
- `Telegram/SourceFiles/data/data_message_reactions.cpp`
- `Telegram/SourceFiles/data/data_forum_topic.cpp`
- `Telegram/SourceFiles/data/data_stories.cpp`
- `Telegram/SourceFiles/dialogs/dialogs_main_list.cpp`
- `Telegram/SourceFiles/history/history.cpp`
- `Telegram/SourceFiles/storage/file_upload.cpp`
- `Telegram/SourceFiles/storage/file_download.cpp`
- `Telegram/SourceFiles/storage/localstorage.cpp`
- `Telegram/SourceFiles/calls/calls_call.cpp`
- `Telegram/SourceFiles/calls/calls_controller_webrtc.cpp`
- `Telegram/SourceFiles/inline_bots/bot_attach_web_view.cpp`
- `Telegram/SourceFiles/platform/platform_notifications_manager.h`

## Recursive-tree discovery update

The frozen root recursive tree was enumerated with `recursive=1` and returned `truncated=false`. It contains 118 SourceFiles directories at depth <= 4 and 35 direct gitlinks. All 35 direct gitlink trees were queried at their pinned commits. Three nested gitlinks were observed:

- `desktop-app/cmake_helpers/external/glib/cppgir@47cf94f83b54cda59018135601e19d7fb0c77776` (GitLab; commit-specific recursive tree still pending);
- `PJK/libcbor/doxygen-theme@46111c61a9f49b7a9886127e679d4317478fab1c` (recursive GitHub tree verified, no further gitlinks);
- `ericniebler/range-v3/doc/gh-pages@2dae74bb693e42d850fb0adcc9045c5b71fbdeae` (recursive GitHub tree verified, no further gitlinks).

This pass also exposed product-capability domains that were missing from the first discovery table, including communities, Compose AI, rich tasks/todo lists, ringtones, self-destruct, sensitive-content policy, statistics, usernames/websites, the built-in media editor, Instant View, support/moderation, and TDE2E. They are now explicitly in scope.

## What this does not prove

- the final external GitLab `cppgir@47cf94f…` recursive leaf (all GitHub-hosted gitlink recursion above it is verified);
- build-time downloads, generated source, patches, resources, shaders or platform packaging closure;
- behavior/state-machine details for each capability;
- C++ production-responsibility completeness;
- full owner-resolution coverage;
- implementation or acceptance.

Until those are closed, `research_inventory_status` remains partial and P0 remains open.
