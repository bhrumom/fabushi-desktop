# Exact-source dossier: orders 4501-4600

- Accepted upstream: `telegramdesktop/tdesktop@36a0c87ca096c48ccf6193aa2c31707fdcafcc7c`
- Root tree: `94e009f981d886ee55cd0450f0a5cc9b38305ba8`
- Read method: direct full-file/source-symbol reading, including class/function/RPC/state/lifecycle/UI/service/failure boundaries; rows are not promoted to implementation/release by this read.
- Accounting after this batch: read-through 4,600; unread 11,520; unknown 15,841; unknown-closed 279; omitted 0.

## Responsibility decomposition

### 4501-4501: Large/custom emoji media rendering
- Canonical owner: Canonical TranscriptEntry Rich Media owner
- State machine: custom-emoji/image resolve -> layout/paint -> selection/open semantics -> unload/teardown
- Side effects: custom emoji/image cache and repaint are scoped to the transcript entry
- Failure/lifecycle boundary: stale custom-emoji document, unloaded image/player, invalid selection/open target, offscreen resource leak
- Fabushi closure target: render large/custom emoji through canonical TranscriptEntry/Media primitives with bounded resource lifetime

### 4502-4503: Live/static location media lifecycle
- Canonical owner: Canonical Location Media owner
- State machine: location snapshot/live tracker -> remaining/status timers -> image/userpic resolve -> draw/link state -> finish/unload
- Side effects: map/image fetch, live countdown timers, userpic updates and location-open actions stay with the location media owner
- Failure/lifecycle boundary: expired live location, stale timer after destroy, image fetch failure, missing peer/userpic, invalid coordinates, offscreen leak
- Fabushi closure target: close canonical location card/map preview, live countdown, open action and teardown semantics

### 4504-4517: History media base/generic/grouped/spoiler lifecycle
- Canonical owner: Canonical TranscriptEntry Media owner
- State machine: message media input -> type/group/spoiler/selection/bubble state -> layout/paint/open/save -> child async settlement -> unload/destroy
- Side effects: media layout, spoiler reveal, grouped child actions, TTL/payment/open/save and repaint remain within canonical TranscriptEntry media ownership
- Failure/lifecycle boundary: stale child/media pointer, malformed group, spoiler race, TTL expiry, async download/payment failure, inaccessible action, heavy-resource leak
- Fabushi closure target: close shared media projection/selection/spoiler/grouping/open/save lifecycle in canonical TranscriptEntry and Media Viewer

### 4518-4519: Photo streaming/autoplay/media lifecycle
- Canonical owner: Canonical Photo Media owner
- State machine: photo metadata -> thumbnail/full media/stream state -> autoplay/frame/error -> open/grouped paint -> stop/unload
- Side effects: photo download/stream frames, story polling, spoiler and Media Viewer open side effects stay with one photo owner
- Failure/lifecycle boundary: download/stream error, stale frame callback, autoplay policy change, destroyed message, spoiler/open race, heavy-part leak
- Fabushi closure target: close canonical photo thumbnail/full/stream/autoplay/error/open/unload behavior

### 4520-4521: Poll answer/vote/media lifecycle
- Canonical owner: Canonical Poll TranscriptEntry owner
- State machine: poll data/version -> answer/media/solution state -> choose/send animation -> vote settlement -> result/recent-voter refresh
- Side effects: vote RPC state, answer animations, attached media, solution/webpage and recent-voter projection stay with canonical poll owner
- Failure/lifecycle boundary: closed/quiz/revote restrictions, duplicate send, stale result/version, attached media failure, destroyed entry during vote
- Fabushi closure target: close canonical poll option/media/vote/result state machine and accessible interaction

### 4522-4523: Premium/star gift service-card lifecycle
- Canonical owner: Canonical Commerce TranscriptEntry owner
- State machine: gift/service payload -> title/subtitle/button/sticker state -> open/redeem/settings action -> reactive settlement/unload
- Side effects: gift-code/star-gift/Premium navigation and sticker resources stay with canonical commerce service entry
- Failure/lifecycle boundary: expired/redeemed/invalid gift, entitlement mismatch, stale button state, destroyed sticker player, network/service failure
- Fabushi closure target: close gift service card, redemption/open action and resource lifecycle in canonical TranscriptEntry commerce owner

### 4524-4525: Save-document action lifecycle
- Canonical owner: Canonical Attachment Save owner
- State machine: document action -> permission/type destination -> save-and-track / Saved Music / Saved Messages / file -> toast settlement
- Side effects: file persistence and Saved Music/Saved Messages mutations stay with canonical attachment owner
- Failure/lifecycle boundary: invalid path/document, duplicate save, disk failure, unavailable destination, stale toast/navigation
- Fabushi closure target: close canonical attachment save destinations, durable persistence and feedback

### 4526-4527: Reactive service-box content lifecycle
- Canonical owner: Canonical TranscriptEntry Service owner
- State machine: service content producer -> title/button/change state -> layout/ripple/link -> resize/repaint -> sticker unload
- Side effects: service action links, reactive resize/repaint and sticker player lifetime stay with canonical service entry
- Failure/lifecycle boundary: content invalidation during click, stale reactive producer, destroyed entry, sticker resource leak
- Fabushi closure target: close reusable canonical service-entry content/button/reactive lifecycle

### 4528-4529: Similar-channel discovery lifecycle
- Canonical owner: Canonical Peer Discovery owner
- State machine: peer/session -> similar API/default-vs-premium limit -> expanded state -> thumbnails/list -> view-all navigation
- Side effects: similar-peer query, ChannelData expansion flag, thumbnail subscriptions and navigation stay with peer discovery
- Failure/lifecycle boundary: premium limit mismatch, stale channel list, request failure, destroyed view, thumbnail leak
- Fabushi closure target: close canonical similar-peer discovery/list expansion and navigation

### 4530-4531: Slot-machine rich animation lifecycle
- Canonical owner: Canonical Rich Media Animation owner
- State machine: dice value -> sticker-pack reel start/end assets -> play/completion timing -> final state -> unload
- Side effects: dice sticker asset resolution/playback and completion repaint stay with canonical rich-media animation owner
- Failure/lifecycle boundary: missing sticker-pack asset, stale dice value, interrupted playback, destroyed entry, player leak
- Fabushi closure target: close canonical dice/slot rich-media deterministic playback and unload

### 4532-4536: Sticker player/rendering lifecycle
- Canonical owner: Canonical Sticker Media owner
- State machine: document/format/power policy -> lottie/webm/static player -> frame/effect/loop state -> open-set action -> unload
- Side effects: sticker decode/player callbacks, premium effects, repaint and set navigation stay within canonical sticker media owner
- Failure/lifecycle boundary: unsupported/corrupt asset, power-saving transition, stale frame callback, player teardown race, missing set
- Fabushi closure target: close canonical static/vector/video sticker playback, effects, navigation and bounded resource lifetime

### 4537-4538: Story-mention service media lifecycle
- Canonical owner: Canonical Story TranscriptEntry owner
- State machine: story mention -> unread/poll registration -> dynamic thumbnail -> open story/photo/document -> unload/unregister
- Side effects: story polling and thumbnail subscriptions stay with canonical story/message owner
- Failure/lifecycle boundary: deleted/expired story, polling after destroy, missing media, stale unread state, thumbnail leak
- Fabushi closure target: close canonical story mention unread/open/polling/resource lifecycle

### 4539-4540: Message suggestion/decision service state
- Canonical owner: Canonical TranscriptEntry Decision owner
- State machine: suggestion/service payload -> pending/accepted/rejected/expired/balance state -> price/time/reply projection -> action settlement
- Side effects: decision state and payment/balance presentation stay with canonical transcript service owner
- Failure/lifecycle boundary: expired/rejected race, insufficient balance, stale reply target, duplicate decision, malformed price/time
- Fabushi closure target: close canonical suggestion decision status/action/payment projection

### 4541-4542: Theme/wallpaper/gift service lifecycle
- Canonical owner: Canonical Theme and Commerce Service owner
- State machine: theme/document/gift payload -> thumbnail/preview/button state -> wallpaper revert/theme choose/gift action -> apply updates -> unload
- Side effects: MTP wallpaper mutation, theme navigation and gift sticker resources stay in canonical theme/commerce service owner
- Failure/lifecycle boundary: theme document load failure, wallpaper revert failure, stale peer theme, expired gift, destroyed preview/player
- Fabushi closure target: close canonical theme/wallpaper preview/revert and gift-service card lifecycle

### 4543-4544: Todo-list task completion lifecycle
- Canonical owner: Canonical Todo TranscriptEntry owner
- State machine: TodoList version/items -> task permission/completion state -> optimistic dependent-message update -> toggleCompletion RPC -> settle/animate/fireworks
- Side effects: todo completion RPC, optimistic projection, userpics and task animations stay with canonical todo message owner
- Failure/lifecycle boundary: forwarded/restricted/premium-gated task, stale list version, RPC failure, duplicate toggle, destroyed view during animation
- Fabushi closure target: close canonical todo task permissions, optimistic mutation, reconciliation and accessible task UI

### 4545-4546: Unique/star gift media projection lifecycle
- Canonical owner: Canonical Commerce TranscriptEntry owner
- State machine: gift model/pattern/backdrop/burned/released-by -> generated parts/badge/image -> link/action -> dynamic update -> unload
- Side effects: gift image subscriptions, released-by/profile navigation and commerce links stay with canonical gift owner
- Failure/lifecycle boundary: missing gift document, stale image subscription, burned/expired transition, hidden author mismatch, destroyed view
- Fabushi closure target: close canonical unique-gift media composition, badge/link and resource lifecycle

### 4547-4548: Unsupported-media notice contract
- Canonical owner: Canonical Unsupported Media ErrorState owner
- State machine: unsupported payload -> safe notice/action projection -> selection/layout -> stable fallback lifecycle
- Side effects: unsupported content is fail-closed into canonical ErrorState/TranscriptEntry without executing unknown actions
- Failure/lifecycle boundary: unknown media type, malformed metadata, unsafe action, missing compatible client path
- Fabushi closure target: close safe unsupported-content notice and upgrade/fallback behavior

### 4549-4550: Userpic suggestion service lifecycle
- Canonical owner: Canonical Profile Suggestion owner
- State machine: suggested userpic media -> preview/action state -> accept/open/update -> service settlement -> unload
- Side effects: profile-photo suggestion mutation/navigation and media resources stay with canonical profile owner
- Failure/lifecycle boundary: stale peer/photo, permission loss, duplicate acceptance, update failure, destroyed preview
- Fabushi closure target: close canonical profile-photo suggestion preview/action/reconciliation

### 4551-4554: Video-message seek/status interaction
- Canonical owner: Canonical Video Media owner
- State machine: video playback/download state -> seek/timestamp/corner-status geometry -> pointer/drag action -> playback/download settlement
- Side effects: seek, timestamp and download status UI stay bound to canonical video player state
- Failure/lifecycle boundary: invalid duration/position, stale playback instance, drag after destroy, download failure, inaccessible seek control
- Fabushi closure target: close canonical video seek/status/download affordances and playback synchronization

### 4555-4556: Web-page/link-preview lifecycle
- Canonical owner: Canonical Link Preview owner
- State machine: webpage metadata/type -> attachment/factcheck/sponsored/IV/button state -> safe link/open-app/media action -> animation/unload
- Side effects: URL safety checks, sponsored/factcheck hints, IV/app/theme/story actions and attached media stay with canonical link-preview owner
- Failure/lifecycle boundary: suspicious/hidden URL, stale webpage data, bot-app confirmation mismatch, missing attachment, factcheck transition, animation leak
- Fabushi closure target: close canonical link-preview rendering, safe navigation, rich attachment and lifecycle semantics

### 4557-4558: Poll context actions/statistics/media editor
- Canonical owner: Canonical Poll Action owner
- State machine: poll menu state -> submit/retract/stats/media-preview action -> RPC/download/editor settlement -> refresh
- Side effects: sendVotes/requestStats plus poll media preview/edit/download side effects stay with canonical poll action owner
- Failure/lifecycle boundary: closed/quiz/revote restriction, request cancellation/failure, media download failure, stale poll state
- Fabushi closure target: close canonical poll context menu, vote actions, stats and media-preview/editor behavior

### 4559-4570: Message reactions selection/list/pagination lifecycle
- Canonical owner: Canonical Reaction owner
- State machine: possible/message reactions -> inline/button/selector/tab state -> choose/expand/list request -> pagination/moderation settlement -> animation/custom-emoji unload
- Side effects: reaction choose/send UI, MTP reactions-list pagination/cancel, moderation, userpics/custom emoji and fly/ripple animations stay with one reaction owner
- Failure/lifecycle boundary: stale reaction list, duplicate choose, paginated request switch/cancel race, removed reaction, moderation permission loss, custom-emoji leak
- Fabushi closure target: close canonical reaction selector/list/pagination/moderation/animation lifecycle

### 4571-4574: Bot earnings statistics lifecycle
- Canonical owner: Canonical Bot Monetization owner
- State machine: bot earn statistics -> load/content state -> transaction/list/show requests -> saved memento/scroll/focus -> refresh
- Side effects: earnings API/statistics and section navigation state stay with canonical bot monetization owner
- Failure/lifecycle boundary: request failure, stale statistics, pagination/refresh race, destroyed widget, incorrect saved-state restore
- Fabushi closure target: close canonical bot earnings overview/list, loading/error and restore state

### 4575-4580: Bot StarRef program/connect/setup lifecycle
- Canonical owner: Canonical Bot Referral owner
- State machine: program/config -> suggested/connected pagination -> commission/duration setup -> connect/update/revoke RPC -> link/recipient state -> restore
- Side effects: MTPpayments StarRef connect/list/update, recipient selection, referral link and commission configuration stay with canonical bot referral owner
- Failure/lifecycle boundary: request cancellation, duplicate connect, invalid commission/duration, revoked program, stale recipient/bot, pagination race
- Fabushi closure target: close canonical bot referral discovery/connect/setup/revoke/link lifecycle with durable state and error handling

### 4581-4595: Channel boosts/giveaway lifecycle
- Canonical owner: Canonical Channel Boost and Giveaway owner
- State machine: boost status/options -> giveaway type/recipients/countries/channels/quantity/date/prize state -> validate/purchase/start -> boost/gift lists -> reload/restore
- Side effects: boost API, giveaway purchase/start, member/channel selection, invite-link copy/share and gift resolution stay with canonical channel monetization owner
- Failure/lifecycle boundary: invalid quantity/country/member/channel, purchase/payment failure, request race, prepaid mismatch, stale boost status, permission loss
- Fabushi closure target: close canonical boost dashboard and giveaway creation/selection/purchase/reload lifecycle

### 4596-4600: Channel earnings presentation contract
- Canonical owner: Canonical Earnings Presentation owner
- State machine: earn amount/model -> major/minor formatting and currency icon/style projection -> responsive render
- Side effects: formatting/icon resource generation remains pure presentation under canonical earnings owner
- Failure/lifecycle boundary: overflow/precision mismatch, missing icon resource, theme/scale mismatch, inaccessible amount semantics
- Fabushi closure target: close canonical earnings amount formatting/icons/styles across themes/scales

## Exact objects

| Order | Path | Blob | Bytes | Type | Responsibility |
|---:|---|---|---:|---|---|
| 4501 | `Telegram/SourceFiles/history/view/media/history_view_large_emoji.h` | `85852e96c06fe7b8a29bca2da4c915fcb7a61d4c` | 1479 | blob | Large/custom emoji media rendering |
| 4502 | `Telegram/SourceFiles/history/view/media/history_view_location.cpp` | `e5958126addb28860f01f16a1186de26dd64c80f` | 24131 | blob | Live/static location media lifecycle |
| 4503 | `Telegram/SourceFiles/history/view/media/history_view_location.h` | `b4ecc4b61f91b0f582505ebb1ed58905e17f4b60` | 3512 | blob | Live/static location media lifecycle |
| 4504 | `Telegram/SourceFiles/history/view/media/history_view_media.cpp` | `485adea6f956a2cb250afeb41fc02dae10cfa9f1` | 18881 | blob | History media base/generic/grouped/spoiler lifecycle |
| 4505 | `Telegram/SourceFiles/history/view/media/history_view_media.h` | `a58d7e36bc4392db3069813cd2b83ee3c86ed9fc` | 12471 | blob | History media base/generic/grouped/spoiler lifecycle |
| 4506 | `Telegram/SourceFiles/history/view/media/history_view_media_common.cpp` | `8477f4581e458f67a6fe3042d925c95e584b5c75` | 19621 | blob | History media base/generic/grouped/spoiler lifecycle |
| 4507 | `Telegram/SourceFiles/history/view/media/history_view_media_common.h` | `e2cd40c34956b2977250dfbba8e6792e61fc3c77` | 2951 | blob | History media base/generic/grouped/spoiler lifecycle |
| 4508 | `Telegram/SourceFiles/history/view/media/history_view_media_generic.cpp` | `d91c5d259987293e1238fdf90f2d9c604aadb207` | 30181 | blob | History media base/generic/grouped/spoiler lifecycle |
| 4509 | `Telegram/SourceFiles/history/view/media/history_view_media_generic.h` | `deeb70b387dd005c33b8a6dfcb016ae819896ed9` | 11245 | blob | History media base/generic/grouped/spoiler lifecycle |
| 4510 | `Telegram/SourceFiles/history/view/media/history_view_media_grouped.cpp` | `248e8e51eb3ac8f3312e99b7b93479498947e8f6` | 26683 | blob | History media base/generic/grouped/spoiler lifecycle |
| 4511 | `Telegram/SourceFiles/history/view/media/history_view_media_grouped.h` | `6706c5f5f047336f927eae2288eb7b1593295716` | 4577 | blob | History media base/generic/grouped/spoiler lifecycle |
| 4512 | `Telegram/SourceFiles/history/view/media/history_view_media_spoiler.cpp` | `cdf0ff3dc0485427c39242b9fa35ec8c75bbad75` | 300 | blob | History media base/generic/grouped/spoiler lifecycle |
| 4513 | `Telegram/SourceFiles/history/view/media/history_view_media_spoiler.h` | `f668a2fd9037dc8b2f15fbbeab8f6ee0dc40cfe7` | 856 | blob | History media base/generic/grouped/spoiler lifecycle |
| 4514 | `Telegram/SourceFiles/history/view/media/history_view_media_unwrapped.cpp` | `269853a89e3dfb6404362ecf6797a7a165d14707` | 25204 | blob | History media base/generic/grouped/spoiler lifecycle |
| 4515 | `Telegram/SourceFiles/history/view/media/history_view_media_unwrapped.h` | `c2d1b020086b1be91693a86c754190bcc89cb8b2` | 4458 | blob | History media base/generic/grouped/spoiler lifecycle |
| 4516 | `Telegram/SourceFiles/history/view/media/history_view_no_forwards_request.cpp` | `5c3c5ea6a500ce690dfcc86a19ee08824482dc4f` | 3507 | blob | History media base/generic/grouped/spoiler lifecycle |
| 4517 | `Telegram/SourceFiles/history/view/media/history_view_no_forwards_request.h` | `7ac6d49a64a608c00f8b79a9b60301b035b020b4` | 619 | blob | History media base/generic/grouped/spoiler lifecycle |
| 4518 | `Telegram/SourceFiles/history/view/media/history_view_photo.cpp` | `44beb4ffa87e94f18184f0e3b7fb5d46b2dd9396` | 36217 | blob | Photo streaming/autoplay/media lifecycle |
| 4519 | `Telegram/SourceFiles/history/view/media/history_view_photo.h` | `8bb536d269f75a00a04f40828ba8da7cc10ed364` | 4862 | blob | Photo streaming/autoplay/media lifecycle |
| 4520 | `Telegram/SourceFiles/history/view/media/history_view_poll.cpp` | `8e2f8e2d3bd820d9635ec0efd16b0ed91adcde9e` | 130346 | blob | Poll answer/vote/media lifecycle |
| 4521 | `Telegram/SourceFiles/history/view/media/history_view_poll.h` | `fc544177ff900233a091d9b04729597168acebdc` | 3713 | blob | Poll answer/vote/media lifecycle |
| 4522 | `Telegram/SourceFiles/history/view/media/history_view_premium_gift.cpp` | `ef0edea70b7cf2790594dd7149783b7aa2fc4206` | 18201 | blob | Premium/star gift service-card lifecycle |
| 4523 | `Telegram/SourceFiles/history/view/media/history_view_premium_gift.h` | `0dae6ab65b34c9fea31ceb4d728c099a17349a99` | 2313 | blob | Premium/star gift service-card lifecycle |
| 4524 | `Telegram/SourceFiles/history/view/media/history_view_save_document_action.cpp` | `6bb6eeb4e64f968206237e2d0f97706872ab5126` | 4222 | blob | Save-document action lifecycle |
| 4525 | `Telegram/SourceFiles/history/view/media/history_view_save_document_action.h` | `d4893fc5717245610cc60b6ec5331a05060a6ee8` | 883 | blob | Save-document action lifecycle |
| 4526 | `Telegram/SourceFiles/history/view/media/history_view_service_box.cpp` | `03f15eec961021fe0a848a3340bd8801e21236e7` | 14000 | blob | Reactive service-box content lifecycle |
| 4527 | `Telegram/SourceFiles/history/view/media/history_view_service_box.h` | `235ed2a4ca587eb87f41f4f5f296b2e7427f98a4` | 3869 | blob | Reactive service-box content lifecycle |
| 4528 | `Telegram/SourceFiles/history/view/media/history_view_similar_channels.cpp` | `3fd08432906b412ff71d3544acf95f8a27a12b66` | 19247 | blob | Similar-channel discovery lifecycle |
| 4529 | `Telegram/SourceFiles/history/view/media/history_view_similar_channels.h` | `138373914ed0c99707f7ceb2a57d9cf6d40b4715` | 2774 | blob | Similar-channel discovery lifecycle |
| 4530 | `Telegram/SourceFiles/history/view/media/history_view_slot_machine.cpp` | `95a52dee3c49612e722a3007b8be75878056893a` | 5454 | blob | Slot-machine rich animation lifecycle |
| 4531 | `Telegram/SourceFiles/history/view/media/history_view_slot_machine.h` | `68dc7e4fa4def08450defac8d1d7fc5621de7a91` | 1864 | blob | Slot-machine rich animation lifecycle |
| 4532 | `Telegram/SourceFiles/history/view/media/history_view_sticker.cpp` | `4ef4cc7be1288442c9d70ab6388e8287076e2f3e` | 18474 | blob | Sticker player/rendering lifecycle |
| 4533 | `Telegram/SourceFiles/history/view/media/history_view_sticker.h` | `536e12dd4c0c71de125b0ff5df200ef66a121a44` | 4636 | blob | Sticker player/rendering lifecycle |
| 4534 | `Telegram/SourceFiles/history/view/media/history_view_sticker_player.cpp` | `947681e16a971e8424fed0acc372eac2637e8daa` | 3856 | blob | Sticker player/rendering lifecycle |
| 4535 | `Telegram/SourceFiles/history/view/media/history_view_sticker_player.h` | `9d5ef7ce1ee5c3784b972fde6e825378a333422e` | 2028 | blob | Sticker player/rendering lifecycle |
| 4536 | `Telegram/SourceFiles/history/view/media/history_view_sticker_player_abstract.h` | `491c655f8ab373bde28a4b0624e6a9d7ee70bf9c` | 752 | blob | Sticker player/rendering lifecycle |
| 4537 | `Telegram/SourceFiles/history/view/media/history_view_story_mention.cpp` | `8bbeabf840950a71f443c80c4348b9f56dce7f5c` | 5386 | blob | Story-mention service media lifecycle |
| 4538 | `Telegram/SourceFiles/history/view/media/history_view_story_mention.h` | `18d42847775d0a5476adeb2c07d73e819eaf6a49` | 1718 | blob | Story-mention service media lifecycle |
| 4539 | `Telegram/SourceFiles/history/view/media/history_view_suggest_decision.cpp` | `12955599746e865dc05ddc6f73d8d6767f93bf7f` | 10538 | blob | Message suggestion/decision service state |
| 4540 | `Telegram/SourceFiles/history/view/media/history_view_suggest_decision.h` | `6bbf5fef6eaec377cf299377757e7f57d6857394` | 849 | blob | Message suggestion/decision service state |
| 4541 | `Telegram/SourceFiles/history/view/media/history_view_theme_document.cpp` | `65f8f230bd012106f43ee04809105233969aa321` | 25197 | blob | Theme/wallpaper/gift service lifecycle |
| 4542 | `Telegram/SourceFiles/history/view/media/history_view_theme_document.h` | `23006073fb8dd5b2d0d29fe5a883406fd3898873` | 4859 | blob | Theme/wallpaper/gift service lifecycle |
| 4543 | `Telegram/SourceFiles/history/view/media/history_view_todo_list.cpp` | `3490f92e9d6226bb3684e149dcaa8ff9432edbe5` | 24621 | blob | Todo-list task completion lifecycle |
| 4544 | `Telegram/SourceFiles/history/view/media/history_view_todo_list.h` | `b17e7853ccec2d2f97682023d2ef5c8173d716c0` | 3562 | blob | Todo-list task completion lifecycle |
| 4545 | `Telegram/SourceFiles/history/view/media/history_view_unique_gift.cpp` | `bb95362263fa58c656d7e6de55ab844d3ac4477a` | 28722 | blob | Unique/star gift media projection lifecycle |
| 4546 | `Telegram/SourceFiles/history/view/media/history_view_unique_gift.h` | `4bdb28d6e6160ee3b61ac8b554ba0108a1d510bf` | 3834 | blob | Unique/star gift media projection lifecycle |
| 4547 | `Telegram/SourceFiles/history/view/media/history_view_unsupported_notice.cpp` | `f252f9dd2a4a054fda5de254393868cdee2921f0` | 3238 | blob | Unsupported-media notice contract |
| 4548 | `Telegram/SourceFiles/history/view/media/history_view_unsupported_notice.h` | `3fb93eb01226ce82cdcc3b4c62613bd7c2c21edd` | 1307 | blob | Unsupported-media notice contract |
| 4549 | `Telegram/SourceFiles/history/view/media/history_view_userpic_suggestion.cpp` | `7eb6dd055ccdd574d17a38b86979971f48db0ad7` | 7977 | blob | Userpic suggestion service lifecycle |
| 4550 | `Telegram/SourceFiles/history/view/media/history_view_userpic_suggestion.h` | `f9a19581f9913bd48572a619e05f193077901a29` | 1430 | blob | Userpic suggestion service lifecycle |
| 4551 | `Telegram/SourceFiles/history/view/media/history_view_video_message_seek.cpp` | `9d17e89eb34116960c7145d2302cf847877b4169` | 5689 | blob | Video-message seek/status interaction |
| 4552 | `Telegram/SourceFiles/history/view/media/history_view_video_message_seek.h` | `0167453cc5e9fbb161a487c55ae5023523686f1c` | 1304 | blob | Video-message seek/status interaction |
| 4553 | `Telegram/SourceFiles/history/view/media/history_view_video_status.cpp` | `1a9d5fdb0404c7efe6c120fcecfa661e8386dcc1` | 5098 | blob | Video-message seek/status interaction |
| 4554 | `Telegram/SourceFiles/history/view/media/history_view_video_status.h` | `89c793c42efbeb0240d12dbcb276a27d39577db3` | 982 | blob | Video-message seek/status interaction |
| 4555 | `Telegram/SourceFiles/history/view/media/history_view_web_page.cpp` | `e3b4e274d526d4ccdfd8de0358d475dc7cfccc73` | 57289 | blob | Web-page/link-preview lifecycle |
| 4556 | `Telegram/SourceFiles/history/view/media/history_view_web_page.h` | `0d828a5fd9d041c16f96cedf4000425d28a30ec5` | 5933 | blob | Web-page/link-preview lifecycle |
| 4557 | `Telegram/SourceFiles/history/view/media/menu/history_view_poll_menu.cpp` | `eb7f3757d45b9452af76ea00ceda6b67b8961a99` | 21743 | blob | Poll context actions/statistics/media editor |
| 4558 | `Telegram/SourceFiles/history/view/media/menu/history_view_poll_menu.h` | `32709bb61f646269643e688705d7df5cf775d299` | 2366 | blob | Poll context actions/statistics/media editor |
| 4559 | `Telegram/SourceFiles/history/view/reactions/history_view_reactions.cpp` | `ae17555034b4a280a58ee38ffadae567cb2a6a33` | 28176 | blob | Message reactions selection/list/pagination lifecycle |
| 4560 | `Telegram/SourceFiles/history/view/reactions/history_view_reactions.h` | `44625784116898b8c51f89d8963044a268084642` | 4469 | blob | Message reactions selection/list/pagination lifecycle |
| 4561 | `Telegram/SourceFiles/history/view/reactions/history_view_reactions_button.cpp` | `eb2bd2c58e01a9e323ad064706727e254e0d17f4` | 26183 | blob | Message reactions selection/list/pagination lifecycle |
| 4562 | `Telegram/SourceFiles/history/view/reactions/history_view_reactions_button.h` | `8cb9c11a9bcb744b074b16887526dc1abc5b9680` | 6740 | blob | Message reactions selection/list/pagination lifecycle |
| 4563 | `Telegram/SourceFiles/history/view/reactions/history_view_reactions_list.cpp` | `c997ed96bddfc023b2b8454fe49ae87819f74f7b` | 19218 | blob | Message reactions selection/list/pagination lifecycle |
| 4564 | `Telegram/SourceFiles/history/view/reactions/history_view_reactions_list.h` | `b1cbd347d3b80d696a6a338a45dc8fb7b220be67` | 2175 | blob | Message reactions selection/list/pagination lifecycle |
| 4565 | `Telegram/SourceFiles/history/view/reactions/history_view_reactions_selector.cpp` | `e82903a5952260280c5eb03b11284def2daabaa0` | 43846 | blob | Message reactions selection/list/pagination lifecycle |
| 4566 | `Telegram/SourceFiles/history/view/reactions/history_view_reactions_selector.h` | `47e8c59b8ab8334303f88f2a92454c68d44abdc8` | 8272 | blob | Message reactions selection/list/pagination lifecycle |
| 4567 | `Telegram/SourceFiles/history/view/reactions/history_view_reactions_strip.cpp` | `721eb55a34bd37b9ef3115dee2cec6ecc5c87562` | 15949 | blob | Message reactions selection/list/pagination lifecycle |
| 4568 | `Telegram/SourceFiles/history/view/reactions/history_view_reactions_strip.h` | `2e539f4ad791be7c23b89739cbbf09072f2a8942` | 4583 | blob | Message reactions selection/list/pagination lifecycle |
| 4569 | `Telegram/SourceFiles/history/view/reactions/history_view_reactions_tabs.cpp` | `acaad5ba4b92c8117f424931e8e35d39c023582d` | 6243 | blob | Message reactions selection/list/pagination lifecycle |
| 4570 | `Telegram/SourceFiles/history/view/reactions/history_view_reactions_tabs.h` | `8e3566552263468c17552e049d63a43aea9690ec` | 947 | blob | Message reactions selection/list/pagination lifecycle |
| 4571 | `Telegram/SourceFiles/info/bot/earn/info_bot_earn_list.cpp` | `7e81adc13f43b2d7169912491d4288cdebfdcf82` | 13152 | blob | Bot earnings statistics lifecycle |
| 4572 | `Telegram/SourceFiles/info/bot/earn/info_bot_earn_list.h` | `1d89d4aa98e0f1f810dc2464a84592f119064adf` | 1540 | blob | Bot earnings statistics lifecycle |
| 4573 | `Telegram/SourceFiles/info/bot/earn/info_bot_earn_widget.cpp` | `ed4ef00f83f5d8962abe5a53c288c0a7f73464ca` | 2917 | blob | Bot earnings statistics lifecycle |
| 4574 | `Telegram/SourceFiles/info/bot/earn/info_bot_earn_widget.h` | `37ab50e941d4f96ad63fc312334164131ff49a3b` | 1639 | blob | Bot earnings statistics lifecycle |
| 4575 | `Telegram/SourceFiles/info/bot/starref/info_bot_starref_common.cpp` | `d5ceb9075aa8e0f8948c6652957d52ba6fdd182a` | 30295 | blob | Bot StarRef program/connect/setup lifecycle |
| 4576 | `Telegram/SourceFiles/info/bot/starref/info_bot_starref_common.h` | `e38bdd1c5a1250077169a7f47577ebc8994ac19d` | 2993 | blob | Bot StarRef program/connect/setup lifecycle |
| 4577 | `Telegram/SourceFiles/info/bot/starref/info_bot_starref_join_widget.cpp` | `e73c9b39dd09c2d7caa014e419999c7f791c2f0c` | 30668 | blob | Bot StarRef program/connect/setup lifecycle |
| 4578 | `Telegram/SourceFiles/info/bot/starref/info_bot_starref_join_widget.h` | `ef9afee14f534aefadf3fbb259e8f4abba9611c6` | 2250 | blob | Bot StarRef program/connect/setup lifecycle |
| 4579 | `Telegram/SourceFiles/info/bot/starref/info_bot_starref_setup_widget.cpp` | `f75c08f5f2928e6508aac0647a6bec4ae29e6c89` | 29426 | blob | Bot StarRef program/connect/setup lifecycle |
| 4580 | `Telegram/SourceFiles/info/bot/starref/info_bot_starref_setup_widget.h` | `6fd66c8cafc069ab354918feef4af33332cdc718` | 2235 | blob | Bot StarRef program/connect/setup lifecycle |
| 4581 | `Telegram/SourceFiles/info/channel_statistics/boosts/create_giveaway_box.cpp` | `ddbb9059156aa1218b20c92e5a2c689dc2d2416e` | 48429 | blob | Channel boosts/giveaway lifecycle |
| 4582 | `Telegram/SourceFiles/info/channel_statistics/boosts/create_giveaway_box.h` | `95db05c6f04c4295e282cac31359726b32b9a728` | 677 | blob | Channel boosts/giveaway lifecycle |
| 4583 | `Telegram/SourceFiles/info/channel_statistics/boosts/giveaway/boost_badge.cpp` | `625fe22142e0a941a96d2ea52c5eba420e3fc0b5` | 5045 | blob | Channel boosts/giveaway lifecycle |
| 4584 | `Telegram/SourceFiles/info/channel_statistics/boosts/giveaway/boost_badge.h` | `bc42a9d1caa6d6e6f511798ed8afeab65bc6704b` | 1153 | blob | Channel boosts/giveaway lifecycle |
| 4585 | `Telegram/SourceFiles/info/channel_statistics/boosts/giveaway/giveaway.style` | `ba4e32f69ac2f9c34ec4acf0c321da23e131aa72` | 8942 | blob | Channel boosts/giveaway lifecycle |
| 4586 | `Telegram/SourceFiles/info/channel_statistics/boosts/giveaway/giveaway_list_controllers.cpp` | `15aa552e02c6d10cd5adb7df8e45cd491fc7404b` | 10626 | blob | Channel boosts/giveaway lifecycle |
| 4587 | `Telegram/SourceFiles/info/channel_statistics/boosts/giveaway/giveaway_list_controllers.h` | `6459d6d8fdeea6ee8d74c0ee813d662da33ba066` | 2882 | blob | Channel boosts/giveaway lifecycle |
| 4588 | `Telegram/SourceFiles/info/channel_statistics/boosts/giveaway/giveaway_type_row.cpp` | `7aa3ade27aa3a7b4cb900b641f36686fe59da3ed` | 5435 | blob | Channel boosts/giveaway lifecycle |
| 4589 | `Telegram/SourceFiles/info/channel_statistics/boosts/giveaway/giveaway_type_row.h` | `ee53a40767715afccbba4fb0abba51edde6266d6` | 1268 | blob | Channel boosts/giveaway lifecycle |
| 4590 | `Telegram/SourceFiles/info/channel_statistics/boosts/giveaway/select_countries_box.cpp` | `4128ef8de39a0330f1e420e59a9b54cbaad68c7d` | 6570 | blob | Channel boosts/giveaway lifecycle |
| 4591 | `Telegram/SourceFiles/info/channel_statistics/boosts/giveaway/select_countries_box.h` | `a1714f8fb76cdcb174e252efc78c8bda8a6776db` | 588 | blob | Channel boosts/giveaway lifecycle |
| 4592 | `Telegram/SourceFiles/info/channel_statistics/boosts/info_boosts_inner_widget.cpp` | `45206c7456b6472b77ad84e196de7e64706e70a3` | 16538 | blob | Channel boosts/giveaway lifecycle |
| 4593 | `Telegram/SourceFiles/info/channel_statistics/boosts/info_boosts_inner_widget.h` | `3161f81c0453f19011b08d907a8dad647ba509c3` | 1378 | blob | Channel boosts/giveaway lifecycle |
| 4594 | `Telegram/SourceFiles/info/channel_statistics/boosts/info_boosts_widget.cpp` | `89d517bf2144c3785d88f10d8a381fa4d9839655` | 2933 | blob | Channel boosts/giveaway lifecycle |
| 4595 | `Telegram/SourceFiles/info/channel_statistics/boosts/info_boosts_widget.h` | `6da483431acd6f0cb340663581d1cd68f7331000` | 1589 | blob | Channel boosts/giveaway lifecycle |
| 4596 | `Telegram/SourceFiles/info/channel_statistics/earn/channel_earn.style` | `393f0661beb32b1bd909c3dffa40b80f1b25b8a9` | 4354 | blob | Channel earnings presentation contract |
| 4597 | `Telegram/SourceFiles/info/channel_statistics/earn/earn_format.cpp` | `69ece36f327de47cc91b137e56386b1fc9fa024a` | 2423 | blob | Channel earnings presentation contract |
| 4598 | `Telegram/SourceFiles/info/channel_statistics/earn/earn_format.h` | `02ac4cee32f7e342d96b5215e2ed1acb09e0f29d` | 740 | blob | Channel earnings presentation contract |
| 4599 | `Telegram/SourceFiles/info/channel_statistics/earn/earn_icons.cpp` | `5cdadef97fee01fa77ddf0413037b366bcbc69fd` | 7888 | blob | Channel earnings presentation contract |
| 4600 | `Telegram/SourceFiles/info/channel_statistics/earn/earn_icons.h` | `3cec5c0f17abc0538daf0f801ae418e85edda25d` | 1620 | blob | Channel earnings presentation contract |

## Status discipline

- All 100 entries are `read_complete=true` and responsibility-decomposed.
- All 100 remain open for shipping implementation/verification unless independently proven elsewhere; this batch does not modify the 55-row implementation/coverage/release status.
- `unknown` remains 15,841 and `omitted` remains 0. `read_complete` is not an automatic unknown-closure mechanism.
- Current-head GitHub Actions must attest this batch and the rebased historical manifests before any exact-head authority claim.
