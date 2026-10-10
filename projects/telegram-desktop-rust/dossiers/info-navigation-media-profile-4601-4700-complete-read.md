# Exact-source dossier: orders 4601-4700

- Accepted upstream: `telegramdesktop/tdesktop@36a0c87ca096c48ccf6193aa2c31707fdcafcc7c`
- Root tree: `94e009f981d886ee55cd0450f0a5cc9b38305ba8`
- Read method: direct source/symbol/RPC/state/lifecycle reading; read-complete does not imply implementation or release closure.
- Accounting after batch: read-through 4,700; unread 11,420; unknown 15,841; unknown-closed 279; omitted 0.

## Responsibility decomposition

### 4601-4604: Channel earnings statistics/withdrawal lifecycle
- Canonical owner: Canonical Channel Monetization owner
- State machine: earn/revenue state -> load/history/update subscription -> withdraw/learn-more action -> memento/scroll restore
- Side effects: revenue/credits APIs, service updates, history and withdrawal navigation stay in one monetization owner
- Failure/lifecycle boundary: request/update race, withdrawal eligibility, stale revenue, destroyed view, restore mismatch
- Fabushi closure target: close earnings overview/history/withdrawal/update/restore behavior

### 4605-4608: Common-groups discovery pagination lifecycle
- Canonical owner: Canonical Peer Discovery owner
- State machine: peer -> common chats request -> pagination/preload/search -> row navigation -> cancel/save/restore
- Side effects: common-chat query and forum/history navigation stay with peer discovery
- Failure/lifecycle boundary: request cancel/failure, duplicate page, destroyed peer, stale restored loading state
- Fabushi closure target: close common-groups pagination/search/navigation/memento behavior

### 4609-4610: Community management lifecycle
- Canonical owner: Canonical Community Management owner
- State machine: peer/community state -> linked peer/content controls -> management action -> reactive settlement/navigation
- Side effects: community management state/actions stay in existing peer/profile owner
- Failure/lifecycle boundary: permission loss, stale linked peer, request failure, destroyed section
- Fabushi closure target: close community management controls/state/service integration

### 4611-4613: Community join/request moderation lifecycle
- Canonical owner: Canonical Community Requests owner
- State machine: requester/request state -> visible row/actions -> accept/decline -> pending settlement -> refresh
- Side effects: join-request moderation actions and requester identity stay in community request owner
- Failure/lifecycle boundary: duplicate decision, permission loss, stale requester, RPC failure, pending action after destroy
- Fabushi closure target: close request list, pending accept/decline and permission-aware feedback

### 4614-4619: Downloads library/provider lifecycle
- Canonical owner: Canonical Downloads Library owner
- State machine: DownloadManager loading/loaded/external state -> index/search/select -> save/show/cancel/clear -> memento/session restore
- Side effects: download manager mutations, file actions and session tracking stay with downloads owner
- Failure/lifecycle boundary: external cancellation race, file missing, save failure, session switch, stale search/layout, clear failure
- Fabushi closure target: close downloads collection/search/selection/file actions/recovery

### 4620-4625: Global media search pagination lifecycle
- Canonical owner: Canonical Universal Media Search owner
- State machine: query/generation/cursor -> request slice -> preload/load-more -> cancellation/stale generation fence -> selection/memento
- Side effects: global-media request/cursor/preload and selection stay behind canonical media search provider
- Failure/lifecycle boundary: stale generation response, cancel race, cursor loop, session switch, destroyed view
- Fabushi closure target: close global media query/cursor/cancellation/preload/restore state machine

### 4626-4626: Info shell presentation contract
- Canonical owner: Canonical DetailPanel presentation contract
- State machine: semantic style tokens -> search/topbar/buttons/shared-media affordances -> responsive render
- Side effects: presentation only; no alternate business owner
- Failure/lifecycle boundary: theme/scale/locale/a11y contrast or hit-target regression
- Fabushi closure target: close canonical DetailPanel style/light-dark/responsive/a11y evidence

### 4627-4642: Info detail navigation/memento/layer lifecycle
- Canonical owner: Canonical DetailPanel Navigation owner
- State machine: section key/memento stack -> content/controller/layer/topbar/wrap -> search/selection/back/float-player -> move/restore/close
- Side effects: navigation stack, active section, floating media delegate and layer lifecycle stay in canonical DetailPanel owner
- Failure/lifecycle boundary: invalid memento, section destroyed, float-player delegate leak, gesture/scroll race, search/selection state loss
- Fabushi closure target: close DetailPanel routing/memento/layer/topbar/scroll lifecycle

### 4643-4660: Shared media browser/provider lifecycle
- Canonical owner: Canonical Shared Media Browser owner
- State machine: media type/provider/slice -> layout/search/zoom/selection/reorder -> open/save/story actions -> preload/restore
- Side effects: shared media/download/global provider composition and selection/open actions stay in one media browser owner
- Failure/lifecycle boundary: slice/provider race, stale layout, zoom anchor loss, selection invalidation, download/open failure, story action permission loss
- Fabushi closure target: close shared-media provider/list/grid/zoom/selection/open/restore lifecycle

### 4661-4662: Members section state/navigation lifecycle
- Canonical owner: Canonical Members and ParticipantRow owner
- State machine: member count/list -> search/group state -> open/add-member -> scroll/memento restore
- Side effects: participant visibility/add/search/navigation remain in canonical member owner
- Failure/lifecycle boundary: permission loss, stale count, add-member failure, restored state mismatch
- Fabushi closure target: close member list/search/add/navigation/restore behavior

### 4663-4668: Peer gifts collections/commerce lifecycle
- Canonical owner: Canonical Peer Gifts Commerce owner
- State machine: gift descriptor/collections -> page/load/filter/select -> create/rename/reorder/add-remove/share -> settle/memento
- Side effects: star gift collection APIs, sticker media, transfer/resale state and list pagination stay in commerce owner
- Failure/lifecycle boundary: collection limit, RPC failure, duplicate mutation, stale page, locked/expired gift, media download failure
- Fabushi closure target: close peer gifts collections/pagination/mutations/resource lifecycle

### 4669-4674: Poll list/results pagination lifecycle
- Canonical owner: Canonical Poll Results owner
- State machine: poll messages/vote option -> sparse list or voters offset -> preload/collapse/load more -> peer navigation -> save/restore
- Side effects: poll create permission, MTPmessages_GetPollVotes pagination and voter state stay with poll owner
- Failure/lifecycle boundary: permission restriction, request cancel/failure, stale vote count/offset, destroyed poll, restore mismatch
- Fabushi closure target: close poll collection/create permission/results pagination and memento behavior

### 4675-4675: Profile presentation contract
- Canonical owner: Canonical ProfileSection presentation contract
- State machine: semantic style definitions -> profile hierarchy/actions/levels -> responsive render
- Side effects: presentation only; no parallel product owner
- Failure/lifecycle boundary: theme/locale/scale/a11y regression
- Fabushi closure target: close profile presentation visual/a11y evidence

### 4676-4677: Profile details/actions/business-hours lifecycle
- Canonical owner: Canonical ProfileSection owner
- State machine: peer/topic/sublist updates -> details/actions/business-hours timer/timezone -> navigation/service actions -> reactive settlement
- Side effects: business details, usernames/about, member/manage, gift/app/profile actions and periodic hours state stay in profile owner
- Failure/lifecycle boundary: permission loss, timezone/day rollover, stale peer data, service action failure, timer after destroy
- Fabushi closure target: close permission-aware profile actions/business hours/reactive lifecycle

### 4678-4678: Profile presentation contract
- Canonical owner: Canonical ProfileSection presentation contract
- State machine: semantic style definitions -> profile hierarchy/actions/levels -> responsive render
- Side effects: presentation only; no parallel product owner
- Failure/lifecycle boundary: theme/locale/scale/a11y regression
- Fabushi closure target: close profile presentation visual/a11y evidence

### 4679-4680: Profile badge/emoji-status rendering lifecycle
- Canonical owner: Canonical Profile Badge owner
- State machine: peer badge/status -> allowed/premium policy -> custom emoji player -> paint/click/update -> unload
- Side effects: verified/premium/scam/fake/direct badges and custom emoji lifetime stay in profile badge owner
- Failure/lifecycle boundary: premium visibility change, missing emoji asset, power-saving change, stale verify info, animation leak
- Fabushi closure target: close badge policy/custom-emoji/resource/a11y lifecycle

### 4681-4682: Profile birthday effect lifecycle
- Canonical owner: Canonical Profile Effects owner
- State machine: birthday/age -> sticker-set request/download -> confetti/digits player -> fallback/fireworks -> timed destroy
- Side effects: sticker-set fetch/download and animation resources stay scoped to profile effect
- Failure/lifecycle boundary: sticker-set/download failure, stale player callback, reduced/power state, effect survives cover destruction
- Fabushi closure target: close bounded birthday effect loading/fallback/teardown

### 4683-4684: Profile cover/topic-icon media lifecycle
- Canonical owner: Canonical Profile Cover owner
- State machine: topic icon/document -> download/player/image fallback -> repaint -> resize/unload
- Side effects: topic cover media resources stay with profile cover owner
- Failure/lifecycle boundary: missing/corrupt document, download failure, stale topic icon, player leak
- Fabushi closure target: close profile/topic cover media loading/render/teardown

### 4685-4686: Emoji-status chooser and entitlement lifecycle
- Canonical owner: Canonical Profile Status owner
- State machine: status lists/mode/filter -> selector show/focus -> choose/until -> premium gate/set -> fly animation/hide
- Side effects: emoji status persistence, channel/background mode, premium gate and recent/default lists stay in profile status owner
- Failure/lifecycle boundary: premium ineligible status, destroyed panel callback, stale list, invalid collectible, animation/focus leak
- Fabushi closure target: close emoji-status selection/persistence/entitlement/focus/resource lifecycle

### 4687-4688: Profile icon presentation lifecycle
- Canonical owner: Canonical ProfileSection icon owner
- State machine: icon/tag data -> responsive paint/update
- Side effects: presentation stays within profile semantic component
- Failure/lifecycle boundary: theme/scale/update regression
- Fabushi closure target: close canonical profile icon presentation

### 4689-4690: Profile composition/tabs/section lifecycle
- Canonical owner: Canonical ProfileSection owner
- State machine: peer/topic/saved scope -> details/members/media/saved/polls/stories tabs -> visibility/scroll/topbar -> save/restore
- Side effects: profile section composition reuses canonical tabs/media/members/saved owners
- Failure/lifecycle boundary: scope switch, stale full-info, tab visibility race, member lazy-load after destroy, restore mismatch
- Fabushi closure target: close single-composition profile tabs/sections/scroll/memento behavior

### 4691-4694: Profile members/roles/permissions lifecycle
- Canonical owner: Canonical Members and ParticipantRow owner
- State machine: participant controller -> role/status rows/search/group -> add/open/manage/remove affordances -> save/restore
- Side effects: participant permissions, role tags, add-member flow and list state stay in canonical members owner
- Failure/lifecycle boundary: cannot-view/add permission, participant role change, stale controller, add/remove failure, search state loss
- Fabushi closure target: close permission-aware members rows/actions/search/restore behavior

### 4695-4696: Profile music action presentation
- Canonical owner: Canonical Profile Music owner
- State machine: music metadata -> button/player status presentation -> click/open action
- Side effects: saved/profile music navigation stays with canonical music owner
- Failure/lifecycle boundary: missing track, stale playback state, inaccessible button/theme mismatch
- Fabushi closure target: close profile music button/playback-aware presentation/action

### 4697-4698: Profile phone privacy/menu lifecycle
- Canonical owner: Canonical Profile Privacy owner
- State machine: phone metadata/settings -> collectible/spoiler menu -> open/copy/toggle hidden -> delayed settings save
- Side effects: phone privacy preference persistence and safe menu actions stay in profile privacy owner
- Failure/lifecycle boundary: self/noncollectible mismatch, unsafe link, settings persistence failure, stale popup
- Fabushi closure target: close phone privacy toggle/collectible action/menu persistence

### 4699-4700: Profile section visibility-stack lifecycle
- Canonical owner: Canonical ProfileSection layout owner
- State machine: section rows/shown producers -> separator visibility computation -> animated toggle/layout resize -> finalize
- Side effects: section composition/visibility belongs to canonical profile layout owner
- Failure/lifecycle boundary: reactive toggle during resize, orphan separator, destroyed producer, animation state leak
- Fabushi closure target: close profile section visibility/separator/reactive layout semantics

## Exact objects

| Order | Path | Blob | Bytes | Type | Responsibility |
|---:|---|---|---:|---|---|
| 4601 | `Telegram/SourceFiles/info/channel_statistics/earn/info_channel_earn_list.cpp` | `51a49b7cacf980995b045a5a85a5214b29d57a69` | 46760 | blob | Channel earnings statistics/withdrawal lifecycle |
| 4602 | `Telegram/SourceFiles/info/channel_statistics/earn/info_channel_earn_list.h` | `0fa417ec724eaf3457de3cb4e6513b9dc78943c0` | 1696 | blob | Channel earnings statistics/withdrawal lifecycle |
| 4603 | `Telegram/SourceFiles/info/channel_statistics/earn/info_channel_earn_widget.cpp` | `20bf8b01812007a6fd892a268c4624619b6f8930` | 3014 | blob | Channel earnings statistics/withdrawal lifecycle |
| 4604 | `Telegram/SourceFiles/info/channel_statistics/earn/info_channel_earn_widget.h` | `031b404d9146fbc72b0af29c2730394f15e664ed` | 1902 | blob | Channel earnings statistics/withdrawal lifecycle |
| 4605 | `Telegram/SourceFiles/info/common_groups/info_common_groups_inner_widget.cpp` | `10f6c9549e4ebe78a3a0a6af47e07d1b83c67806` | 8805 | blob | Common-groups discovery pagination lifecycle |
| 4606 | `Telegram/SourceFiles/info/common_groups/info_common_groups_inner_widget.h` | `30c42b50f6d410282b416464e5fac9b2d211dc49` | 2143 | blob | Common-groups discovery pagination lifecycle |
| 4607 | `Telegram/SourceFiles/info/common_groups/info_common_groups_widget.cpp` | `66205f30de61220e626f1046f33dc56abb6ed8d6` | 2694 | blob | Common-groups discovery pagination lifecycle |
| 4608 | `Telegram/SourceFiles/info/common_groups/info_common_groups_widget.h` | `78a8dc23a33f6f8aa2599637cfbc986bb5725da6` | 1573 | blob | Common-groups discovery pagination lifecycle |
| 4609 | `Telegram/SourceFiles/info/community/info_community_widget.cpp` | `4f18b4e6a7aba7b5925664ba7d29fac048e9a822` | 8435 | blob | Community management lifecycle |
| 4610 | `Telegram/SourceFiles/info/community/info_community_widget.h` | `81dbbbd8968d93e462ae5bd08b3c147abd373ed0` | 1590 | blob | Community management lifecycle |
| 4611 | `Telegram/SourceFiles/info/community_requests/info_community_requests_widget.cpp` | `189088542b624abb279d2255e98736d81f4a4c84` | 40085 | blob | Community join/request moderation lifecycle |
| 4612 | `Telegram/SourceFiles/info/community_requests/info_community_requests_widget.h` | `77d03717932f1db303e8c89006dbf92d354d9c2b` | 1610 | blob | Community join/request moderation lifecycle |
| 4613 | `Telegram/SourceFiles/info/community_requests/info_community_requests_widget.style` | `90efa2630f4c34c2202e72720dbc94265a613664` | 1350 | blob | Community join/request moderation lifecycle |
| 4614 | `Telegram/SourceFiles/info/downloads/info_downloads_inner_widget.cpp` | `6320be4c788b5cb540bd495094d9e8eb5681187c` | 5108 | blob | Downloads library/provider lifecycle |
| 4615 | `Telegram/SourceFiles/info/downloads/info_downloads_inner_widget.h` | `65b55f4d0de827ef2b2974822dac5ac50b707d75` | 1761 | blob | Downloads library/provider lifecycle |
| 4616 | `Telegram/SourceFiles/info/downloads/info_downloads_provider.cpp` | `78cb2b1469af748857575b7411fc47ef4948d83a` | 15060 | blob | Downloads library/provider lifecycle |
| 4617 | `Telegram/SourceFiles/info/downloads/info_downloads_provider.h` | `1a7840b10dba683258c23ac6f55d282d0822de78` | 4646 | blob | Downloads library/provider lifecycle |
| 4618 | `Telegram/SourceFiles/info/downloads/info_downloads_widget.cpp` | `a3c3db60c3f9ba2d876fe658168b0246dabf519a` | 3918 | blob | Downloads library/provider lifecycle |
| 4619 | `Telegram/SourceFiles/info/downloads/info_downloads_widget.h` | `f79a3146606dc54af4605b8cea115d96cfab20dd` | 1748 | blob | Downloads library/provider lifecycle |
| 4620 | `Telegram/SourceFiles/info/global_media/info_global_media_inner_widget.cpp` | `d75d2b0dda8adc3468a70493afeef8e46f559b47` | 3674 | blob | Global media search pagination lifecycle |
| 4621 | `Telegram/SourceFiles/info/global_media/info_global_media_inner_widget.h` | `004647cb1a121d81da51f3b2ec54f46968722840` | 1941 | blob | Global media search pagination lifecycle |
| 4622 | `Telegram/SourceFiles/info/global_media/info_global_media_provider.cpp` | `55954b6c70eae52e09b40139a5a314da3ab060b2` | 21467 | blob | Global media search pagination lifecycle |
| 4623 | `Telegram/SourceFiles/info/global_media/info_global_media_provider.h` | `a056ef42b521bd23b7d6367a3f5f67253ce59241` | 7188 | blob | Global media search pagination lifecycle |
| 4624 | `Telegram/SourceFiles/info/global_media/info_global_media_widget.cpp` | `761e0a818fabfd1786132f1dbf8ea1c6aa0cf792` | 4080 | blob | Global media search pagination lifecycle |
| 4625 | `Telegram/SourceFiles/info/global_media/info_global_media_widget.h` | `9a47cbe9c206f382b0b65e9c71cb9df8382f7b1c` | 1839 | blob | Global media search pagination lifecycle |
| 4626 | `Telegram/SourceFiles/info/info.style` | `b58295bc6dbef65c564a582410bee03567eba0ff` | 39553 | blob | Info shell presentation contract |
| 4627 | `Telegram/SourceFiles/info/info_content_widget.cpp` | `a0a9211d1f78996ff353758d608ae46365b96a6d` | 19333 | blob | Info detail navigation/memento/layer lifecycle |
| 4628 | `Telegram/SourceFiles/info/info_content_widget.h` | `57592a6ddbce2f7f71c3f023c9beb434a752d40f` | 11872 | blob | Info detail navigation/memento/layer lifecycle |
| 4629 | `Telegram/SourceFiles/info/info_controller.cpp` | `1d6fb54c40746b668db9e9fb3f1c52d707ef33e0` | 15795 | blob | Info detail navigation/memento/layer lifecycle |
| 4630 | `Telegram/SourceFiles/info/info_controller.h` | `dba20870bc11f68c5b74edb179fd4f2d06b6aaef` | 10749 | blob | Info detail navigation/memento/layer lifecycle |
| 4631 | `Telegram/SourceFiles/info/info_flexible_scroll.cpp` | `326fe228135d2f5544aff1b6798088db3559946e` | 10568 | blob | Info detail navigation/memento/layer lifecycle |
| 4632 | `Telegram/SourceFiles/info/info_flexible_scroll.h` | `b101cfc3a293b5d554b29f85da4d1971c33d8989` | 1997 | blob | Info detail navigation/memento/layer lifecycle |
| 4633 | `Telegram/SourceFiles/info/info_layer_widget.cpp` | `4ad4fab661b3d10bbbfbd182f024ef9d8f43ab44` | 12315 | blob | Info detail navigation/memento/layer lifecycle |
| 4634 | `Telegram/SourceFiles/info/info_layer_widget.h` | `07c3cab8e489b52979ebc2a0fdb8bbc92a5b7bdc` | 2566 | blob | Info detail navigation/memento/layer lifecycle |
| 4635 | `Telegram/SourceFiles/info/info_memento.cpp` | `452b761b80691e76226cbc47be146432ea72e2d1` | 11024 | blob | Info detail navigation/memento/layer lifecycle |
| 4636 | `Telegram/SourceFiles/info/info_memento.h` | `2c990aefaa921e2830c6e1afffdc4e85f3d7487e` | 4404 | blob | Info detail navigation/memento/layer lifecycle |
| 4637 | `Telegram/SourceFiles/info/info_section_widget.cpp` | `0cb471757ac66082e2b3e4de1a9a83851fff861f` | 3884 | blob | Info detail navigation/memento/layer lifecycle |
| 4638 | `Telegram/SourceFiles/info/info_section_widget.h` | `40b6f7189f17a73f2a7855130c5551c09ce00139` | 1870 | blob | Info detail navigation/memento/layer lifecycle |
| 4639 | `Telegram/SourceFiles/info/info_top_bar.cpp` | `cbe12fc24a836657f3f7818177c7abdaf1581565` | 24581 | blob | Info detail navigation/memento/layer lifecycle |
| 4640 | `Telegram/SourceFiles/info/info_top_bar.h` | `d7907c367b443790078bef52db30abd3675573a6` | 5694 | blob | Info detail navigation/memento/layer lifecycle |
| 4641 | `Telegram/SourceFiles/info/info_wrap_widget.cpp` | `8895f2c1857f18a005bec0b78085f36b8c48b591` | 36047 | blob | Info detail navigation/memento/layer lifecycle |
| 4642 | `Telegram/SourceFiles/info/info_wrap_widget.h` | `17ff3633bfb2c20d4e08d41e50b812e48daea4f3` | 7365 | blob | Info detail navigation/memento/layer lifecycle |
| 4643 | `Telegram/SourceFiles/info/media/info_media_buttons.cpp` | `192c9b7af6d017068304c2c8482cf17b5229606a` | 7925 | blob | Shared media browser/provider lifecycle |
| 4644 | `Telegram/SourceFiles/info/media/info_media_buttons.h` | `a5e394a17f15f8ddc25eda122d6a21bb00c06b0e` | 1786 | blob | Shared media browser/provider lifecycle |
| 4645 | `Telegram/SourceFiles/info/media/info_media_common.cpp` | `a26cdf558560c7ff0864120ce2a271276bdd480d` | 2594 | blob | Shared media browser/provider lifecycle |
| 4646 | `Telegram/SourceFiles/info/media/info_media_common.h` | `b41567d9ca32ff9af224208b56baa4ff1dd85687` | 5501 | blob | Shared media browser/provider lifecycle |
| 4647 | `Telegram/SourceFiles/info/media/info_media_empty_widget.cpp` | `fa3edb6f171634d12fd4b3cf478f3ddfb640e375` | 3416 | blob | Shared media browser/provider lifecycle |
| 4648 | `Telegram/SourceFiles/info/media/info_media_empty_widget.h` | `9b60a38ef723ff42e90ec50ffa234b93ab16c926` | 986 | blob | Shared media browser/provider lifecycle |
| 4649 | `Telegram/SourceFiles/info/media/info_media_grid_zoom.cpp` | `2c30b18d93d7f941620fafbbb18f26c2006572be` | 6471 | blob | Shared media browser/provider lifecycle |
| 4650 | `Telegram/SourceFiles/info/media/info_media_grid_zoom.h` | `227f2c24100c1c54aa3c8ab3bf25b435803e6b67` | 1427 | blob | Shared media browser/provider lifecycle |
| 4651 | `Telegram/SourceFiles/info/media/info_media_inner_widget.cpp` | `77152f54eaf73a5c1cd26687e2db5fd0fa5bf490` | 7176 | blob | Shared media browser/provider lifecycle |
| 4652 | `Telegram/SourceFiles/info/media/info_media_inner_widget.h` | `e1c4ca28e9d2c32fc44b6d3bbf0ebbfe38bf66f7` | 2413 | blob | Shared media browser/provider lifecycle |
| 4653 | `Telegram/SourceFiles/info/media/info_media_list_section.cpp` | `2e4376df2e8fae1d92d72a83591d72ec9fc17de5` | 12668 | blob | Shared media browser/provider lifecycle |
| 4654 | `Telegram/SourceFiles/info/media/info_media_list_section.h` | `06c64e5cc5baff91939f5a2107d7f336e3d28f63` | 2907 | blob | Shared media browser/provider lifecycle |
| 4655 | `Telegram/SourceFiles/info/media/info_media_list_widget.cpp` | `c35b2843aaa4d4117026d73e4a28a6e9af069079` | 85779 | blob | Shared media browser/provider lifecycle |
| 4656 | `Telegram/SourceFiles/info/media/info_media_list_widget.h` | `8518f5ff2cd7cb6ae592de3be18cd06572468246` | 14200 | blob | Shared media browser/provider lifecycle |
| 4657 | `Telegram/SourceFiles/info/media/info_media_provider.cpp` | `3d8acd456b951c78201cd059b45cd6988202cbca` | 17959 | blob | Shared media browser/provider lifecycle |
| 4658 | `Telegram/SourceFiles/info/media/info_media_provider.h` | `cb2e14ea39cf1187eeb680128b2e140cab7f351a` | 4055 | blob | Shared media browser/provider lifecycle |
| 4659 | `Telegram/SourceFiles/info/media/info_media_widget.cpp` | `2e01ff312a0afa424344905d130a07694413cb11` | 7126 | blob | Shared media browser/provider lifecycle |
| 4660 | `Telegram/SourceFiles/info/media/info_media_widget.h` | `490539d1ef175d03787e74ca6e5d89fbd2a5b35d` | 3453 | blob | Shared media browser/provider lifecycle |
| 4661 | `Telegram/SourceFiles/info/members/info_members_widget.cpp` | `c9dbc1f677c647a57bed89ec6a2770d3f8d41087` | 2741 | blob | Members section state/navigation lifecycle |
| 4662 | `Telegram/SourceFiles/info/members/info_members_widget.h` | `f91af76b709555f71e771da594ce7ab717d25c9e` | 1519 | blob | Members section state/navigation lifecycle |
| 4663 | `Telegram/SourceFiles/info/peer_gifts/info_peer_gifts_collections.cpp` | `bc377f5e0fcb7992113ee8ed3662a99d1b69eec9` | 4279 | blob | Peer gifts collections/commerce lifecycle |
| 4664 | `Telegram/SourceFiles/info/peer_gifts/info_peer_gifts_collections.h` | `c60c32e312deceeb581883e541dd890b82bfe12c` | 889 | blob | Peer gifts collections/commerce lifecycle |
| 4665 | `Telegram/SourceFiles/info/peer_gifts/info_peer_gifts_common.cpp` | `d761b3194da03d0d497ede874ca336ae8e1dd663` | 46046 | blob | Peer gifts collections/commerce lifecycle |
| 4666 | `Telegram/SourceFiles/info/peer_gifts/info_peer_gifts_common.h` | `0b31fd321e67a9daad0d85870ebccb9d5ae95f7e` | 9225 | blob | Peer gifts collections/commerce lifecycle |
| 4667 | `Telegram/SourceFiles/info/peer_gifts/info_peer_gifts_widget.cpp` | `fa2a473e233310c6d769027c3d3436aef0106c4c` | 75557 | blob | Peer gifts collections/commerce lifecycle |
| 4668 | `Telegram/SourceFiles/info/peer_gifts/info_peer_gifts_widget.h` | `adbc73d9250c725b7bf54cbdc50f2d89b8d69bad` | 3399 | blob | Peer gifts collections/commerce lifecycle |
| 4669 | `Telegram/SourceFiles/info/polls/info_polls_list_widget.cpp` | `fc875783b315bd09fc47b6715765c93598610ee4` | 30563 | blob | Poll list/results pagination lifecycle |
| 4670 | `Telegram/SourceFiles/info/polls/info_polls_list_widget.h` | `78a14d3aeea19ab652dbca80da7adf9b599a1c88` | 2779 | blob | Poll list/results pagination lifecycle |
| 4671 | `Telegram/SourceFiles/info/polls/info_polls_results_inner_widget.cpp` | `2a0ab20e4af6fd819c97117a809cdd5ba9af8296` | 21766 | blob | Poll list/results pagination lifecycle |
| 4672 | `Telegram/SourceFiles/info/polls/info_polls_results_inner_widget.h` | `c84e10ef97c8dadab638d03604c827448dffe4e8` | 1664 | blob | Poll list/results pagination lifecycle |
| 4673 | `Telegram/SourceFiles/info/polls/info_polls_results_widget.cpp` | `2dbc18025b49bc2f1eeb1c51283dab5270ffa0cb` | 2912 | blob | Poll list/results pagination lifecycle |
| 4674 | `Telegram/SourceFiles/info/polls/info_polls_results_widget.h` | `a3c4ac752c1aeca84fcd151c71bc79ff904f51d4` | 1585 | blob | Poll list/results pagination lifecycle |
| 4675 | `Telegram/SourceFiles/info/profile/info_levels.style` | `861b31e7fd9196946d8f71571a2717427dd8a91f` | 2480 | blob | Profile presentation contract |
| 4676 | `Telegram/SourceFiles/info/profile/info_profile_actions.cpp` | `cd5c93019d00bc38ac251788310ac4b6dc7a2f71` | 103261 | blob | Profile details/actions/business-hours lifecycle |
| 4677 | `Telegram/SourceFiles/info/profile/info_profile_actions.h` | `e94e90b172ba8130d160b9a9c68cdd4e5929d885` | 1395 | blob | Profile details/actions/business-hours lifecycle |
| 4678 | `Telegram/SourceFiles/info/profile/info_profile_actions.style` | `b70c39c9fefd3b22db87bb9f13d3db49bd150c50` | 2562 | blob | Profile presentation contract |
| 4679 | `Telegram/SourceFiles/info/profile/info_profile_badge.cpp` | `1b058ae69f4a956c8b6d48a10771c7fb039e4676` | 8898 | blob | Profile badge/emoji-status rendering lifecycle |
| 4680 | `Telegram/SourceFiles/info/profile/info_profile_badge.h` | `20771e06b2749759685ea38bfb988db2032de6a9` | 2670 | blob | Profile badge/emoji-status rendering lifecycle |
| 4681 | `Telegram/SourceFiles/info/profile/info_profile_birthday_effect.cpp` | `491795e2d5eb3467d2837df5eb2a613e59bd3da6` | 12803 | blob | Profile birthday effect lifecycle |
| 4682 | `Telegram/SourceFiles/info/profile/info_profile_birthday_effect.h` | `46f67902cf5b67dda87657fe6f2003ef6375bea4` | 521 | blob | Profile birthday effect lifecycle |
| 4683 | `Telegram/SourceFiles/info/profile/info_profile_cover.cpp` | `d4450ce93c16db03ec43cb77e54437de9708403d` | 5654 | blob | Profile cover/topic-icon media lifecycle |
| 4684 | `Telegram/SourceFiles/info/profile/info_profile_cover.h` | `3499809b041b57d465589a385ffcc516459e8172` | 1735 | blob | Profile cover/topic-icon media lifecycle |
| 4685 | `Telegram/SourceFiles/info/profile/info_profile_emoji_status_panel.cpp` | `085d4fff2f3bc9d36b985e8bb210c3b3521fadbc` | 10997 | blob | Emoji-status chooser and entitlement lifecycle |
| 4686 | `Telegram/SourceFiles/info/profile/info_profile_emoji_status_panel.h` | `f9a6252b3864b34b6959faad7abfc6ca24f3135c` | 2499 | blob | Emoji-status chooser and entitlement lifecycle |
| 4687 | `Telegram/SourceFiles/info/profile/info_profile_icon.cpp` | `174b144cf854669a256d3364a7a5466f9b89ef11` | 938 | blob | Profile icon presentation lifecycle |
| 4688 | `Telegram/SourceFiles/info/profile/info_profile_icon.h` | `abf0e686588dff7acfb9c2768d9d9f921d32dec5` | 734 | blob | Profile icon presentation lifecycle |
| 4689 | `Telegram/SourceFiles/info/profile/info_profile_inner_widget.cpp` | `4b167bea2beb2a1e1ec5cbb4eae9b72765f13e7c` | 16878 | blob | Profile composition/tabs/section lifecycle |
| 4690 | `Telegram/SourceFiles/info/profile/info_profile_inner_widget.h` | `9797071d46a8111a9de222816565821adaf98269` | 3267 | blob | Profile composition/tabs/section lifecycle |
| 4691 | `Telegram/SourceFiles/info/profile/info_profile_members.cpp` | `6b81250c8f6e2660828dfefb1e618e13b865d51b` | 13904 | blob | Profile members/roles/permissions lifecycle |
| 4692 | `Telegram/SourceFiles/info/profile/info_profile_members.h` | `8eeb4d6faee01ea5dac9f7bc5415d7949bedcdf5` | 4193 | blob | Profile members/roles/permissions lifecycle |
| 4693 | `Telegram/SourceFiles/info/profile/info_profile_members_controllers.cpp` | `643a6d13967b7ff77dfacc8a020e179883c5a20c` | 13614 | blob | Profile members/roles/permissions lifecycle |
| 4694 | `Telegram/SourceFiles/info/profile/info_profile_members_controllers.h` | `16e9b8448c5d00aad5269a7e710c0a1e915cff6f` | 3372 | blob | Profile members/roles/permissions lifecycle |
| 4695 | `Telegram/SourceFiles/info/profile/info_profile_music_button.cpp` | `38bf9d586dcbe7829925c6477182d9d9d855b9c7` | 4516 | blob | Profile music action presentation |
| 4696 | `Telegram/SourceFiles/info/profile/info_profile_music_button.h` | `21999605ce7f0e1b2a3a082e9cb803c48fb560aa` | 957 | blob | Profile music action presentation |
| 4697 | `Telegram/SourceFiles/info/profile/info_profile_phone_menu.cpp` | `0e0599b25abe213587fae89604884fbe33d31a40` | 4495 | blob | Profile phone privacy/menu lifecycle |
| 4698 | `Telegram/SourceFiles/info/profile/info_profile_phone_menu.h` | `c6ce038fe685760c8f58bd298eb65f02da88d927` | 632 | blob | Profile phone privacy/menu lifecycle |
| 4699 | `Telegram/SourceFiles/info/profile/info_profile_section_stack.cpp` | `3710660622af069049ef385b2ec1c8c3a25ea887` | 7017 | blob | Profile section visibility-stack lifecycle |
| 4700 | `Telegram/SourceFiles/info/profile/info_profile_section_stack.h` | `dbfc4e93725b5d0e709cf1bb2b8986bbb7b13614` | 1443 | blob | Profile section visibility-stack lifecycle |

## Status discipline

- 100/100 entries are read-complete and responsibility-decomposed.
- unknown remains 15,841; omitted remains 0; no new unknown-closure credit.
- 55-row implementation/coverage/release status is unchanged; same-head GitHub Actions evidence is still required.
