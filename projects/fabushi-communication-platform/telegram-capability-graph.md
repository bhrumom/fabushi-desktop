# TDRP/FBCP P0 — Telegram Top-Level Capability Graph (Discovery Pass)

Status: discovery pass; recursive gitlink closure complete; behavior/resource/license closure still incomplete  
Frozen upstream: `telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677`  
Target owner snapshot: PR #20 `c2767eac1383fd8db7be9b536e5acaf4a4d2b7f0`

This graph records product capability domains observed in the frozen Telegram Desktop source tree. It is intentionally not marked research-complete: recursive gitlink closure is now complete, but generated/resource/platform-packaging/license provenance and behavior-level resolution for remaining long-tail domains are still open, so P0 cannot yet claim that no unknown capability domain remains.

| Capability domain | Representative frozen source regions | Initial Fabushi owner candidates |
| --- | --- | --- |
| startup / app lifecycle | `main.cpp`, `main/main_domain.cpp`, `core`, `window`, `platform`; dossier `research/foundations-lifecycle-identity-dialogs.md` | owner-resolved as `APP-LIFECYCLE` → existing Electron main/platform lifecycle; native communication resume remains blocker |
| account / auth / sessions / multi-account | `main/main_account.cpp`, `main/main_domain.cpp`, `api/api_authorizations.cpp`, storage/intro; foundation dossier | owner-resolved as `ACCOUNT-AUTH-MULTI` → existing Electron account/auth + Host Session; local Human identity exists, native multi-device session/revocation remains blocker |
| contacts / identity / presence | `data/data_lastseen_status.h`, `data/data_peer_values.cpp`, peer/member/search areas; foundation dossier | owner-resolved as `CONTACT-ID-PRESENCE` → Human identity/account + Shared Room/member, with minimal ephemeral presence infra; no dedicated current presence owner |
| dialog list / folders / archive / pinning | `dialogs/dialogs_main_list.cpp`, `data/data_chat_filters.cpp`, `data/data_folder.cpp`; foundation dossier | owner-resolved as `DIALOG-FOLDER-ARCHIVE` → existing sidebar/conversation-list owner; Agent-only schema generalization + native list sync remain blockers |
| private messaging / history | `history/*`, `data/data_messages.*`, `data/data_history_messages.*`, `api/api_sending.*`, `api/api_updates.*` | researched in `projects/telegram-desktop-rust/research/message-history-lifecycle.md` and `message-settlement-and-history-gap.md`; owner resolution split into `MSG-SEND-SETTLEMENT` and `MSG-HISTORY-ORDER-GAP`; existing Session/Transcript + Composer/pagination remain product owners, with minimal native transport/sync below |
| reply / quote / forward provenance | `history/history_item*`, `data/data_reply_preview.*`, replies/thread structures | researched in `projects/telegram-desktop-rust/research/message-relations-and-settlement.md`; existing transcript relation/provenance owner selected |
| edit / delete lifecycle | `api/api_editing.*`, history/data message state | researched in `projects/telegram-desktop-rust/research/message-relations-and-settlement.md`; existing transcript/message lifecycle selected |
| reactions | `data/data_message_reactions.*`, `api/api_who_reacted.*`, info reaction list | researched in `projects/telegram-desktop-rust/research/message-relations-and-settlement.md`; existing reaction/transcript owner selected |
| polls / rich message variants | `data/data_poll.cpp`, `api/api_polls.*`, `poll/*`; dossier `research/rich-messages-media-expression.md` | owner-resolved as `MSG-POLL-RICH` → existing typed transcript/message + card registry; native poll operation/sync remains blocker |
| drafts / composer / voice input | `data/data_drafts.*`, history composer/view code | researched in `projects/telegram-desktop-rust/research/message-relations-and-settlement.md`; existing composer/draft owner selected |
| scheduled / silent send / send progress | `api/api_sending.*`, `api/api_send_progress.*`, scheduled-history code | researched in `projects/telegram-desktop-rust/research/message-relations-and-settlement.md`; Automations + composer/send + ephemeral presence/progress selected |
| groups / members / admin | `data/data_chat.*`, `data/data_groups.*`, `api/api_chat_participants.*`, `info/members` | researched in `projects/telegram-desktop-rust/research/rooms-search-media.md`; existing Shared Room/member/permissions selected |
| channels / broadcast | `data/data_channel.*`, history/dialog/api channel code | researched in `projects/telegram-desktop-rust/research/rooms-search-media.md`; existing room/conversation + broadcast policy selected |
| topics / forums / threads | `data/data_forum.*`, `data/data_forum_topic.*`, `data/data_thread.*` | researched in `projects/telegram-desktop-rust/research/rooms-search-media.md`; existing conversation/thread/transcript selected |
| search | `api/api_messages_search.*`, `data/data_search_controller.*`, dialog search code | researched in `projects/telegram-desktop-rust/research/rooms-search-media.md`; existing Content Search + transcript pagination selected |
| files / upload / download | `storage/file_upload.*`, `storage/file_download*`, `data/data_document*` | researched in `projects/telegram-desktop-rust/research/rooms-search-media.md`; existing attachments/resources + minimal blob transfer selected |
| images / video / audio / voice / streaming | `data/data_streaming.cpp`, `media/*`, preload/rotation; rich-media dossier | owner-resolved as `MEDIA-STREAM-CACHE` → existing attachments/resources + media viewer/cards; minimal transfer/cache infra below |
| stickers / GIF / emoji / custom emoji | `chat_helpers`, data/media/UI expression areas; rich-media dossier | owner-resolved as `EXPRESSION-STICKER-GIF-EMOJI` → existing Composer + rich transcript/reaction surfaces + resource lifecycle |
| Stories | `data/data_stories.*`, `data/data_story.*`, `media/stories`, `info/stories`; dossier `research/stories-notifications-privacy-security-bots-webview.md` | owner-resolved as `STORY-LIFECYCLE` → existing product shell + Session durable state + attachments/resources + identity/permissions; native publication/sync remains blocker |
| notifications / tray / badge | platform notification manager, `tray*`, settings notifications; dossier `research/stories-notifications-privacy-security-bots-webview.md` | owner-resolved as `NOTIFICATION-TRAY` → existing Electron notifications + Host notifications/trays; native push remains blocker |
| privacy / safety / blocking | settings privacy controllers, `api/api_user_privacy.*`, blocked-peers; dossier `research/stories-notifications-privacy-security-bots-webview.md` | owner-resolved as `PRIVACY-BLOCKING` → existing settings + account/Human identity + permissions/member policy |
| local lock / credential / WebAuthn | cloud password, `webauthn`, platform WebAuthn; dossier `research/stories-notifications-privacy-security-bots-webview.md` | owner-resolved as `LOCAL-CREDENTIAL-WEBAUTHN` → existing account/auth + secrets + Host WebAuthn proxy |
| Bots / inline interactions | `inline_bots/*`, `api/api_bot.*`, peer bot commands; dossier `research/stories-notifications-privacy-security-bots-webview.md` | owner-resolved as `BOT-INLINE` → existing Agent + Composer + typed transcript cards + Coordinator/Host/Runner + Plugins/MCP |
| Mini Apps / WebView concepts | inline bot attach web view + `lib_webview` gitlink; dossier `research/stories-notifications-privacy-security-bots-webview.md` | owner-resolved as `MINIAPP-WEBVIEW` → existing Plugins/MCP + plugin browser/Web surface + Electron security boundary |
| Premium / subscriptions / credits / Stars | premium/credits data + API/settings; dossier `research/commerce-data-calls-realtime.md` | owner-resolved as `PREMIUM-ENTITLEMENT` for entitlement projection and `CREDITS-GIFTS`/`PAYMENT-SETTLEMENT` for value; real settlement requires ADR-002 and product/provider decision |
| gifts | `data/data_star_gift.*`, peer gifts; commerce dossier | owner-resolved as `CREDITS-GIFTS` → existing typed transcript/resource/shell presentation plus ADR-002 for real value settlement |
| business features | `data/business`, `settings/business`; commerce dossier | owner-resolved as `BUSINESS-WORKFLOWS` → existing Automations + Agent + settings/permissions + Composer/Transcript |
| payments | `payments`, credits/earn code; commerce dossier | `PAYMENT-SETTLEMENT`: existing_owner=none; minimal PaymentSettlement infrastructure fully specified in ADR-002, implementation blocked on product/provider applicability |
| Passport / security identity flows | `passport`, `webauthn`, auth code; commerce dossier | owner-resolved as `IDENTITY-DOCUMENT-SECURITY` → existing account/auth + secrets + resource lifecycle; regulated verification remains conditional/not-configured |
| export / data lifecycle | `export`, storage serialization; commerce dossier | owner-resolved as `EXPORT-DATA` → existing settings + Session/data lifecycle + attachments/artifacts |
| local storage / migration / corruption recovery | `storage/*`, serialization/account/domain/facade, file locks; commerce dossier | owner-resolved as `STORAGE-MIGRATION-RECOVERY` → existing Host Session recovery + Electron startup migration + attachment/cache owners |
| themes / language / RTL / IME / accessibility | `lang`, `ui`, platform, history/dialog accessibility sources; commerce dossier | owner-resolved as `THEME-I18N-A11Y` → existing root shell/theme + settings + platform/accessibility composition |
| calls / video / screen sharing | `calls/*`, `calls/group`, `lib_webrtc`, `tgcalls`; commerce dossier | `CALL-REALTIME`: existing_owner=none for session/signaling, minimal owner in ADR-003; existing Computer retains screen-share/control integration |
| reconnect / sleep-wake / proxy / network transition | `mtproto`, core/network/platform lifecycle; commerce dossier | owner-resolved as `NETWORK-SYNC-REALTIME` → existing product owners plus minimal native sync/network infrastructure below them; no MTProto runtime |
| multi-device sync / ordering / duplicate suppression / gap recovery | update/session/data/mtproto state machinery; commerce dossier | owner-resolved under `NETWORK-SYNC-REALTIME` + message/history rows; durable state remains existing owner |
| large histories / pagination / stale updates | history/data sparse-id/search/update code | existing transcript pagination/storage + sync infrastructure |
| update / install / rollback | core/platform/build/update code; commerce dossier | owner-resolved as `UPDATE-INSTALL-ROLLBACK` → existing Electron update/platform owner |
| communities / managed communities | `api/api_communities.*`, peer/community boxes, `info/community*`; long-tail dossier | owner-resolved as `COMMUNITY-MANAGED` → existing Shared Room/member/admin + sidebar/conversation projection |
| Compose AI / AI tone helpers | `api/api_compose_with_ai.*`, `boxes/compose_ai_box.*`, create/preview AI tone boxes; long-tail dossier | owner-resolved as `COMPOSE-AI` → existing Composer + Agent/Coordinator/Host/Runner |
| rich tasks / todo lists | `api/api_rich_tasks.*`, `api/api_todo_lists.*`, todo-list editor boxes; long-tail dossier | owner-resolved as `RICH-TASKS-TODO` → existing Task/Automations + typed Transcript/cards + Composer |
| ringtones / notification sounds | `api/api_ringtones.*`, ringtone settings/boxes, `Resources/sounds`; long-tail dossier | owner-resolved as `NOTIFICATION-RINGTONE` → existing notifications/settings + resource lifecycle |
| self-destruct / expiry | `api/api_self_destruct.*`, self-destruction UI, message lifecycle; long-tail dossier | owner-resolved as `SELF-DESTRUCT-POLICY` → existing account lifecycle + settings/permissions + Transcript lifecycle |
| sensitive-content policy | `api/api_sensitive_content.*`, privacy/settings surfaces; long-tail dossier | owner-resolved as `SENSITIVE-CONTENT` → existing settings + permissions/safety policy |
| statistics / analytics surfaces | `api/api_statistics*.*`, `statistics/*`, channel statistics info; long-tail dossier | owner-resolved as `ROOM-STATISTICS` → existing Shared Room/room-info read-only projection |
| usernames / websites / links | `api/api_user_names.*`, `api/api_websites.*`, deep-link/local-url handlers; long-tail dossier | owner-resolved as `IDENTITY-USERNAME-WEBSITE` → existing identity/profile + account/auth + navigation security |
| media/photo/video editor | `editor/*`, media editor scene/controllers/video; long-tail dossier | owner-resolved as `MEDIA-EDITOR` → existing Composer + attachments/resources + media viewer |
| Instant View / rich document rendering | `iv/*`; long-tail dossier | owner-resolved as `RICH-DOCUMENT-VIEW` → existing Artifacts + safe Web/rich-content + transcript/navigation owners |
| support / moderation / report flows | `support/*`, report/moderation boxes, blocked peers; long-tail dossier | owner-resolved as `SUPPORT-MODERATION` → existing feedback/support + permissions/Shared Room admin + Transcript actions |
| TDE2E / end-to-end encryption research | `tde2e/*`; long-tail dossier | owner-resolved as `TDE2E-SECURITY` → existing identity/device + native sync security + CallSession + Shared Room permissions; Telegram protocol not reused |
| commerce/security long tail discovered outside list | recursively discovered business/credits/gifts/passport/payments/webauthn areas | resolved into named commerce/security rows; any future newly discovered domain must repeat the same owner law |

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


## P0 behavior-resolution checkpoint — 2026-10-02

The message/history cluster now has behavior dossiers and explicit owner-resolution rows for send settlement, ordered history/gap recovery, reply/quote/forward, edit/delete, reactions, drafts, scheduled/silent send, and transient send progress. This is research/ownership closure for those rows only. It does not close the global capability graph, recursive source closure, native transport/sync implementation, PR #20 strict parity, CI, or packaged acceptance.


## Foundation owner-resolution checkpoint — 2026-10-02

`APP-LIFECYCLE`, `ACCOUNT-AUTH-MULTI`, `CONTACT-ID-PRESENCE`, and `DIALOG-FOLDER-ARCHIVE` now have fixed-upstream behavior evidence, plausible-owner analysis, selected existing owners, exact c276 owner paths, persistence/native-network requirements and blockers in the owner-resolution matrix and foundation dossier. This closes ownership research for those rows only; it does not satisfy global P0 because many capability domains and recursive source/provenance leaves remain unresolved.


## Stories through Mini Apps owner-resolution checkpoint — 2026-10-02

`STORY-LIFECYCLE`, `NOTIFICATION-TRAY`, `PRIVACY-BLOCKING`, `LOCAL-CREDENTIAL-WEBAUTHN`, `BOT-INLINE`, and `MINIAPP-WEBVIEW` now have fixed-upstream behavior evidence, plausible-owner analysis, selected existing owners, exact c276 owner paths, persistence/native-network requirements, focused tests and explicit blockers in the formal matrix and dossier. This is P0 research/ownership progress only; none of these rows is promoted to implemented or accepted by documentation.


## Commerce, data lifecycle, shell quality and realtime checkpoint — 2026-10-02

Formal owner-resolution now covers Premium/entitlements, credits/gifts, business workflows, payment settlement, sensitive identity-document flows, export, storage/migration/recovery, themes/language/RTL/IME/accessibility, calls/realtime signaling, network/sync/reconnect semantics, and update/install/rollback. Payment and call signaling are the two rows in this batch where no existing canonical owner safely owns the authoritative state; ADR-002 and ADR-003 define minimal infrastructure owners without creating a parallel product architecture. These rows remain research/ownership only, not implementation claims.


## Recursive long-tail owner-resolution checkpoint — 2026-10-02

Every product-capability domain currently present in this recursively discovered graph now has a formal owner-resolution row or is explicitly covered by a named aggregate row. This closes the current known graph's owner-resolution unknowns, not global P0: generated/resource/platform-packaging and third-party/license provenance are still incomplete, so source closure cannot yet prove that no additional applicable product capability is hidden outside the current inventory. Production implementation and packaged acceptance remain separate.
