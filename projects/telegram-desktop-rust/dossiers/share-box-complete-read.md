# ShareBox complete source read — source completeness evidence

Status: read-complete / responsibility closure open  
Project: TDRP-001 Revision 9  
Accepted upstream: `telegramdesktop/tdesktop@0a11537c220df35fe59a96aebe413b1e52bc6e47`  
Rebaseline inheritance: source entries cited below retain the same blob/content hashes on `0a11537c220df35fe59a96aebe413b1e52bc6e47`; read status is inherited by blob identity, while execution evidence must be reacquired on the current Fabushi HEAD.
Fabushi audit base: `e62b1166e8779b1aa5ecbbec3b14fb5f80a3dd42`

## Exact source identity

| path | blob | read status | mapping status |
| --- | --- | --- | --- |
| `Telegram/SourceFiles/boxes/share_box.cpp` | `b7bf9f0d1f253d8e1a333ec1daa61d224fb188be` | complete, 2,326 lines | open |
| `Telegram/SourceFiles/boxes/share_box.h` | `ec31a835b502ef28c4786840ff23275a36d58b46` | complete, 214 lines | open |

These two entries are credited only against `unread`. They are **not** credited against `unknown`, because every responsibility below still needs a complete machine-readable ledger row and exact-head production/test evidence before the file can leave unknown.

## Responsibility decomposition

### SB-01 canonical recipient search and stale-result fencing

Source symbols: `ShareBox::prepare`, `searchByUsername`, `needSearchByUsername`, `peopleDone`, `peopleFail`, `Inner::updateFilter`, `Inner::applyChatFilter`, `Inner::peopleReceived`, `dialogsRowReplacements`.

Behavior:
- normalizes query words, debounces remote lookup, caches by query, and associates each request id with the originating query;
- ignores results that are no longer the current request;
- merges local rows with remote people results while deduplicating peers already represented by the active indexed collection;
- rebuilds local projection when the chat filter changes so stale rows from a destroyed collection cannot survive;
- replacement rows update the filtered projection rather than creating duplicate recipient identities.

Existing Fabushi owners inspected:
- `frontend/src/production/command-palette-model.ts` — existing canonical Search projection; the already-landed `TDRP-R9-SEARCH-ROW-REPLACEMENT-001` row owns stable-id/latest-row dedupe.
- `native/mahayana-messaging/src/search.rs` — canonical Rust `SearchQuery`, `SearchScope`, `SearchIndex`, `SearchResult`.
- `native/mahayana-messaging/src/service.rs` — permission-filtered search envelope over visible conversations.

Disposition: extend these canonical Search owners; no ShareBox/Telegram-specific search owner. The existing row closes only the row-replacement slice; request freshness, local/remote composition, participant/thread picker reuse and permission-before-exposure still need explicit rows/evidence.

### SB-02 selected destination identity is a Thread, not merely a peer

Source symbols: `Inner::Chat`, `chatThread`, `selected`, `changeCheckState`, `changePeerCheckState`, `chooseForumTopic`, `chooseMonoforumSublist`, `chooseCommunityChat`.

Behavior:
- a selected visual peer may resolve to its base history, a forum topic, a saved-message sublist, or a community child thread;
- nested target destruction automatically clears selection;
- selecting a forum/community first enters a typed child picker instead of silently collapsing the destination to the parent;
- deselection tears down topic/sublist lifetimes and returns to the base peer identity;
- selection callbacks carry the resolved `Data::Thread`.

Existing Fabushi owners inspected:
- `native/mahayana-messaging/src/conversation.rs` — canonical `Conversation`, `ConversationKind`, `Topic`, `ConversationDraft`, `TopicDraft`.
- canonical ProductShell/Conversation creation and picker contracts defined by FBCP IA/component contracts.

Disposition: map to canonical Conversation/Topic typed destination and canonical Participant/Search picker. No second Group/Channel/Topic app. `18289a68a55a3166f42013b9ea8cb84b7565faf1` adds `thread_root_message_id` to the canonical forward protocol/service/engine path and preserves it on the destination Message, with a serde default for legacy payloads. Machine-readable responsibility: `TDRP-R9-SHARE-THREAD-DESTINATION-001`; oracle `ORA-TDRP-SHARE-THREAD-DESTINATION-001`; invariants `INV-TDRP-FORWARD-THREAD-NOT-FLATTENED-001` and `INV-TDRP-FORWARD-LEGACY-NONE-001`. `df02b3bd1146bac38fcf41a37c819d0662851890` adds native revalidation that the selected topic exists and is neither closed nor hidden; `c13a85a2a193660f95cfa9cefe29808cdb368fd4` adds explicit missing/closed/hidden regression coverage. That policy is tracked separately as `TDRP-R9-SHARE-TOPIC-ELIGIBILITY-001` so typed-route preservation is not conflated with eligibility. Both slices remain implemented, not verified until current-head GitHub Actions evidence exists. Topic eligibility traceability is explicit: oracle `ORA-TDRP-SHARE-TOPIC-ELIGIBILITY-001`; invariants `INV-TDRP-FORWARD-TOPIC-EXISTS-001`, `INV-TDRP-FORWARD-TOPIC-OPEN-VISIBLE-001`, and `INV-TDRP-FORWARD-TOPIC-NO-QUEUE-ON-DENY-001`. The accepted `data/data_saved_sublist.{h,cpp}` authority has now also been read completely. It confirms that a saved child is a typed `Data::Thread` with parent/sublist identity, an explicit destroyed signal, child-scoped pagination/read state/drafts, unread reconciliation, client-side merge, list existence/pinning and a per-child no-paid-messages exception. Current Fabushi topic/read/draft owners cover only part of that contract; exact shipping picker lifecycle/destruction and saved-message sublist/community-child identity/state mapping remain open. The exact dependency identities and full read disposition are recorded here; no separate SavedMessages subsystem is accepted.

#### Saved-sublist dependency authority

| path | blob | read status | classification | mapping status |
| --- | --- | --- | --- | --- |
| `Telegram/SourceFiles/data/data_saved_sublist.h` | `c62b6186445814e75d7a021affe38908bf47b4c8` | complete, 6,250 bytes | typed child-thread state/lifecycle contract | open |
| `Telegram/SourceFiles/data/data_saved_sublist.cpp` | `68290f41b6edf80bcb04657cabc33e4393cd75f3` | complete, 35,862 bytes | child history pagination/read/draft/list/payment lifecycle | open |

The implementation file was reread in bounded line ranges after the initial connector rendering truncated; no truncated rendering is credited as complete. Beyond ShareBox selection, these files carry stable parent + child identity, destroyed lifecycle, around/before/after pagination with skipped-gap accounting and request cancellation/retry, client-side-message merge, last/list-message reconciliation, inbox/outbox read cursors, unread/mark/reaction state, delayed read settlement, child-scoped draft behavior, list/pin restoration, preload, notification clearing, and a child-level no-paid-messages exception. `bc3252c40684f4420ee0f2f51dc7b9237cc58038` starts the source-neutral canonical mapping inside the existing Conversation owner: `ConversationChildIdentity` distinguishes Topic root-message identity, SavedSublist participant identity, and nested/community Conversation identity; `ConversationDestinationSelection::clear_destroyed_child` clears only the exact destroyed child instead of silently collapsing to the parent; and `ConversationDestination::can_toggle_unread` preserves the Thread-level Topic/community/self-sublist/monoforum-admin unread-toggle distinctions. The current SB-02 branch then extends the same owner with `ConversationChildRuntimeState` and `ConversationChildPaginationState`: actor-scoped monotonic inbox/outbox cursors, child draft state, skipped-before/after/full-count accounting, pin restoration across temporary emptiness, active-child continuity, exact-child destroy cleanup and per-child no-paid-messages policy are now source-neutral serializable state under `MessagingState::conversation_child_states`. `TopicReadChanged` and `TopicDraftChanged` project existing topic traffic into this typed state, and topic unread projection prefers the typed cursor while retaining the legacy topic cursor only as a snapshot/protocol compatibility fallback. The same branch now also exposes source-neutral `MarkConversationChildRead` and `SetConversationChildDraft` commands through the canonical protocol/service/engine path. SavedSublist/community child callers therefore no longer need a Topic-only command surface for these two state transitions; authorization is evaluated against the existing parent Conversation/community membership before the canonical child state is mutated. Focused contracts cover non-regressing read cursors, bad/duplicate pagination accounting, pin restoration/destroy cleanup, SavedSublist read+draft persistence, and outsider denial. This remains only an implemented SB-02 slice. The current branch now exposes typed child pagination-window, pin, active-child, marked-unread, SavedSublist no-paid-messages and exact-child destroy commands through the same canonical protocol/service/engine owner, and projects actor-scoped child runtime state through SyncBatch. Non-empty Nested/Conversation child pagination is accepted only when each message is provably owned by that child; non-empty SavedSublist pagination fails closed because the canonical Message shape does not yet carry a source-neutral message-to-SavedSublist relation. SavedSublist lifecycle validation now accepts only an explicit canonical `ConversationKind::SavedMessages` parent owned by the authenticated actor; arbitrary Direct/Group/Channel parents fail closed. Telegram's separate monoforum-parent variant remains open until Fabushi has a source-neutral monoforum parent relation and is not inferred from generic Channel/admin membership. Load-time migration that removes the legacy topic fallback, a canonical SavedSublist message-membership relation, request cancellation/retry, notification/reaction reconciliation, actual child-list pagination composition and shipping picker lifecycle remain open. Therefore these source entries remain unknown and SB-02 is not verified.

#### SavedMessages parent-owner dependency authority

| path | blob | read status | classification | mapping status |
| --- | --- | --- | --- | --- |
| `Telegram/SourceFiles/data/data_saved_messages.h` | `82b9aed5cb2e1250fc0f4c5c2013bd58c689a344` | complete, 4,056 bytes | saved-child collection/cache/lifecycle owner | open |
| `Telegram/SourceFiles/data/data_saved_messages.cpp` | `2be216670b644eb012e7a241af78b3adda639619` | complete, 17,635 bytes | child loading, stale refresh, pin/list, destroy cleanup and active-child lifecycle | open |

This parent owner adds responsibilities beyond the individual SavedSublist: support/unsupported state, exact child cache identity, stale batched refresh, request callback coalescing, pinned and paginated child-list loading, active-child continuity, child deletion cleanup, shared-media unload, forward-draft clearing, recent-child projection, unread-count reconciliation and parent-chat/monoforum behavior. These responsibilities must extend canonical Conversation/thread/list/read/draft/resource owners rather than introduce a SavedMessages product subsystem.

### SB-03 send-mode derivation and schedule/silent/reminder semantics

Source symbols: `sendMenuDetails`, `showMenu`, `submit`.

Behavior:
- paid destinations constrain the send menu to silent-only;
- otherwise eligibility can produce scheduled-to-user, self-reminder, or generic scheduled mode;
- scheduling flows through one `SendOptions` object;
- comment send and forwarded payload share the selected thread/send options, while effect availability is explicitly constrained by the surface.

Existing Fabushi owners inspected:
- `native/mahayana-messaging/src/protocol.rs` — canonical `ClientCommand::SendMessage` includes `scheduled_at_ms`, `silent`, reply and client-message identity.
- `native/mahayana-messaging/src/message.rs` — canonical `Message` persists `scheduled_at_ms`, `silent`, reply relation and forward origin.
- `native/mahayana-messaging/src/service.rs` — visibility and idempotent send replay honor scheduled/silent fields.
- `native/mahayana-messaging/src/engine.rs` — canonical send/forward state transitions.

Disposition: existing messaging domain owns the state. `824daf3aefe47c747d8abe648108265b17d8255f` now preserves `scheduled_at_ms` and `silent` through the canonical forward protocol/service/engine/Message path. Machine-readable responsibility: `TDRP-R9-SHARE-SEND-OPTIONS-001`; oracle `ORA-TDRP-SHARE-SEND-OPTIONS-001`; invariants `INV-TDRP-SHARE-SCHEDULE-PRESERVE-001` and `INV-TDRP-SHARE-SILENT-PRESERVE-001`. This slice is implemented, not verified. The accepted `history_view_schedule_box.{h,cpp}` + `api_common.h` source has now also been read completely: "send when online" is a distinct eligibility-gated send type, while Telegram's `0x7FFFFFFE` timestamp is only a transport sentinel. Fabushi already has canonical Presence, LastSeen privacy, Conversation and Message owners, but its current `scheduled_at_ms` model does not yet express a typed presence trigger; reminder/send-when-online persistence, recovery, server execution, UI eligibility and exact E2E therefore remain open. See `scheduled-send-complete-read.md`.

### SB-04 forward provenance, options, fan-out and idempotency

Source symbols: `DefaultForwardCountMessages`, `DefaultForwardCallback`, `CreateForwardedMessagePhraseArgs`, `ShowForwardedMessageToast`, `FastShareMessage`, `FastShareMessageToSelf`.

Behavior:
- resolves only still-existing source messages before send;
- prevents duplicate submit while request set is non-empty;
- validates each destination before payment/send;
- preserves or intentionally drops sender names/captions;
- normalizes rich-page forward options;
- carries topic/sublist reply routing, scheduled/repeat, silent, quick-reply, paid approval, suggested-post, effect, video timestamp and ephemeral provenance flags;
- creates independent random ids for forwarded message range sends;
- tracks each request to terminal success/failure and only reports global success when all outstanding sends settle;
- surfaces changed payment requirements and forbidden voice-message failures;
- self-forward has a specialized recent-forward update and success feedback.

Existing Fabushi owners inspected:
- `native/mahayana-messaging/src/protocol.rs` — `ClientCommand::ForwardMessage`.
- `native/mahayana-messaging/src/engine.rs` — `Command::ForwardMessage`, canonical `forward_origin`, and message state transitions.
- `native/mahayana-messaging/src/service.rs` — actor-scoped `stable_message_id` plus compatible replay/conflict handling.

Current production slices: `b598bd9f98903f245842ac81547bd6d5bfdf0acc` moves forward creation onto the same actor-scoped stable message identity used by canonical sends and extends service replay handling to forwards; `e4494def67715379e619ba5882078347c00c08e8` proves same-payload retry replays one existing message while changed options using the same id fail closed. Machine-readable responsibility: `TDRP-R9-SHARE-FORWARD-IDEMPOTENCY-001`; oracle `ORA-TDRP-SHARE-FORWARD-IDEMPOTENCY-001`; invariants `INV-TDRP-FORWARD-RETRY-ONE-MESSAGE-001` and `INV-TDRP-FORWARD-ID-CONFLICT-FAIL-CLOSED-001`. The canonical privacy slice then adds `ForwardPrivacy` through message/protocol/service/engine (`633d54b0a393b9ee042266d85e1ccbd9fc661300`, `7b52b7218a0d34aa64d5203af7a8dbfced304e48`, `08b7a6f6cc14606d04f2e1c242179948a12e384e`, `1430809599ab2f5a5e20c993ed84541b1d9d895c`), normalizes caption hiding to also hide sender provenance, strips the destination canonical Message rather than merely hiding UI, and binds the privacy snapshot to idempotent replay. Focused tests land in `24523945ecf607e9546cebabb1b98bfb651494ec` and `21ab7870a3ba3129172942d722befb83c321f413`. Machine-readable responsibility: `TDRP-R9-SHARE-FORWARD-PRIVACY-001`; oracle `ORA-TDRP-SHARE-FORWARD-PRIVACY-001`. Both slices are implemented, not verified. Required invariants are `INV-TDRP-FORWARD-CAPTION-IMPLIES-NO-SENDER-001`, `INV-TDRP-FORWARD-PRIVACY-NATIVE-001`, and `INV-TDRP-FORWARD-PRIVACY-IDEMPOTENT-001`. Remaining SB-04 gaps include renderer action-level reverse normalization, multi-recipient fan-out settlement, repeat/suggested/effect/video/ephemeral applicability, paid-send revalidation and UI duplicate-submit fencing.
- `native/mahayana-messaging/src/message.rs` — `Message.forward_origin`, reply relation, stable/client identities.
- Host transcript send pipeline under `source/host/src/extensions/transcript/` owns existing Bot send acceptance/fanout and must not regress.

Disposition: reuse canonical Message/Conversation/Host send owners. The native forward privacy model is now implemented; open gaps are the exact shipping UI action contract, multi-destination settlement oracle, repeat schedule, suggested/effect/video/ephemeral equivalence where applicable, paid-send revalidation, duplicate-submit fencing, and current-head execution evidence.

### SB-05 paid-send restriction and approval revalidation

Source symbols: `initChatRestriction`, `refreshRestrictedRows`, `preloadUserpic`, `showLockedError`, `computeStarsCount`, `submit/checkPaid`.

Behavior:
- restrictions may initially be unknown and are asynchronously resolved;
- premium/account changes refresh already-rendered recipients;
- unresolved recipient information fences submit until full info/credit state arrives;
- aggregate cost is recomputed from selected recipients × outgoing message count;
- a prior approval is reused only up to the approved amount; increased cost requires a new confirmation;
- per-range forwarding consumes approved value and sends the exact paid allowance;
- restriction changes during send fail visibly rather than silently accepting stale authorization.

Existing Fabushi owners inspected:
- `native/mahayana-messaging/src/payment.rs` — canonical `Money`, `PaymentOrder`, `PaymentStatus`, wallet/entitlement structures.
- `native/mahayana-messaging/src/service.rs` — canonical payment/settlement boundary.

Disposition: payment domain is the existing owner, not a Telegram stars subsystem. A per-message paid-send authorization/revalidation contract is not yet demonstrated by the inspected owners and remains an implementation/evidence gap.

### SB-06 permission-before-recipient-exposure and send

Source symbols: `FastShareMessage/filterCallback`, `FastShareLink/filterCallback`, recipient population/filter paths.

Behavior:
- message forwarding filters recipients using required send right, inline-send right and broadcast/game constraints;
- link sharing requires the destination’s send-other permission;
- special users that explicitly bypass money restrictions are handled as a typed policy case;
- invalid destinations fail before network send.

Existing Fabushi owners inspected:
- `native/mahayana-messaging/src/conversation.rs` — `ConversationPermissions`.
- `native/mahayana-messaging/src/service.rs` — membership/role authorization and permission-filtered search exposure.

Disposition: extend canonical Conversation permissions plus Search/provider authorization; never “search everything then hide in UI.” `470171e326134da2d3ab5753de331aa85552744d` makes the native `ForwardMessage` path fail closed on destination participant/message/media/poll policy, community/channel restrictions, slow mode, and secret-chat content constraints before queueing a destination message. Machine-readable responsibility: `TDRP-R9-SHARE-DESTINATION-POLICY-001`; oracle `ORA-TDRP-SHARE-DESTINATION-POLICY-001`; invariants `INV-TDRP-FORWARD-NO-PERMISSION-BYPASS-001` and `INV-TDRP-FORWARD-NO-QUEUE-ON-DENY-001`. This slice is implemented, not verified. `34d19cc79a6821698a45017ebe89ce3b0722d073` adds the canonical `SearchRecipients` path with typed media/poll requirements and service-side filtering before SearchIndex result generation; `5445d8ab5da92c9abdf0809cb2745d8baca176f8` adds focused blocked-send/media-disabled exposure tests. `a50fd7cf28b6dcebda7240d94208e524b18f145e` extends that same source-neutral authorization contract with typed send-other/inline/game requirements, protected-source denial and standard/secret destination compatibility, without creating a second Search owner. The existing Conversation model does not yet expose send-other/inline/game permission axes, so those typed requests fail closed rather than being default-allowed. The shipping Human forward picker now sends the actual `sourceEntryId`; Host validates the settled source message, derives media/protected requirements, and calls the same `fabushi_messaging_core::recipient_search_authorized` function before returning any Human-direct candidate. Machine-readable responsibility: `TDRP-R9-SHARE-RECIPIENT-ELIGIBILITY-001`. It remains implemented, not verified. Oracle `ORA-TDRP-SHARE-RECIPIENT-ELIGIBILITY-001` and the existing invariants remain authoritative. Open composition gaps are widening the shipping candidate universe beyond Human-direct to canonical Group/Channel/typed child destinations, representing authorized send-other/inline/game policy in the existing Conversation owner, and obtaining exact-head Electron/temporal/recovery evidence.

### SB-07 interaction lifecycle, keyboard navigation and responsive list behavior

Source symbols: `keyPressEvent`, `Inner::activateSkipColumn`, `activateSkipPage`, `setActive`, `resizeEvent`, selection callbacks, close-by-outside-click behavior.

Behavior:
- Up/Down/PageUp/PageDown move a stable active recipient and request scroll-to-visible;
- Ctrl/Meta submit differs from plain selection;
- selection/comment content controls whether outside-click may close the surface;
- column geometry recomputes on resize;
- only visible-range profile data is preloaded.

Disposition: Qt painting/layout is presentation-replaced, but keyboard/focus/selection/state-continuity/responsive/preload responsibilities must map to canonical Fabushi Picker/ListRow/CreationFlow behavior. Exact current shipping component symbols and visual/a11y/performance evidence remain open.

### SB-08 fast-share-link and copy-link semantics

Source symbols: `FastShareLink`, `FastShareMessage/copyCallback`, `copyLink`.

Behavior:
- one operation either copies a canonical deep/direct link or sends it through selected conversations;
- comments prepend the URL while shifting entity offsets so formatting metadata remains aligned;
- duplicate share submission is fenced by an in-flight boolean;
- successful multi-recipient share closes the layer only after the local dispatch loop is created.

Disposition: canonical Message formatted-text/entity owner plus OS clipboard adapter and canonical Conversation picker. `cf60178cab2142e75e9b0f5b7d24145b0b997b15` adds `FormattedText::prepend_plain_text`, which shifts existing entity offsets by the prefix length in UTF-16 code units and fails atomically on offset overflow. Machine-readable responsibility: `TDRP-R9-SHARE-LINK-ENTITY-OFFSET-001`; oracle `ORA-TDRP-SHARE-LINK-ENTITY-OFFSET-001`; invariants `INV-TDRP-SHARE-LINK-UTF16-OFFSET-001` and `INV-TDRP-SHARE-LINK-ATOMIC-OVERFLOW-001`. This narrow slice is implemented, not verified. Clipboard ownership, duplicate-submit fencing, multi-recipient settlement, SendOther pre-exposure, and exact shipping UI evidence remain open.

## Shipping composition audit

The current shipping Human Conversation path now has an implemented Forward surface, but the responsibility is still **not verified**:

- `frontend/src/production/ProductionRenderer.tsx` remains the real shipping Conversation root and injects `onForward` only for Human conversations into the existing `ConversationTranscript`; no second conversation route or source-specific app was added.
- `frontend/src/production/ForwardMessageDialog.tsx` is mounted inside the existing Conversation route. It performs debounced/stale-fenced recipient queries, multi-selection, sender/caption privacy controls, duplicate-submit fencing, and partial-settlement retry while retaining one request identity.
- recipient query flows `ProductionRenderer -> ForwardMessageDialog(sourceEntryId) -> Host gateway searchHumanRecipients -> ProductionSessionWorkers::search_human_recipients -> list_human_conversations`. Host validates the actual settled source message and executes `fabushi_messaging_core::recipient_search_authorized` before exposure. This reads the existing Host Session Human Conversation truth and creates no persisted Search/recipient owner.
- forward submission flows `ProductionRenderer -> Host gateway forwardHumanMessage -> ProductionSessionWorkers::forward_human_message -> append_human_message_with_context`. It therefore reuses the existing Human Message owner for clientNonce durability, pending/dispatching lifecycle, attachment handling, remote settlement and restart recovery rather than introducing another Message store.
- the focused Host regression `sharebox_shipping_tests::shipping_human_forward_reuses_session_owner_across_retry_and_restart` exercises recipient lookup, duplicate-destination collapse, partial fan-out settlement, privacy coupling, changed-policy replay conflict and restart replay.
- `source/host/Cargo.toml` depends directly on the existing source-neutral `fabushi-messaging-core`. Shipping `forward_human_message` executes that crate's `ForwardPrivacy::normalized()`, and shipping recipient discovery executes its `recipient_search_authorized`; both policies therefore have one executable source. The Host candidate universe remains Human-direct scoped, so Group/Channel/topic/saved-sublist composition is still open even though each returned Human candidate crosses the canonical authorization boundary.

Therefore the previous “not shipping-wired” blocker is closed and both ForwardPrivacy plus recipient pre-exposure filtering now reuse source-neutral executable policy from `fabushi-messaging-core`. **Exact-head Forward evidence and full recipient composition remain open**. `TDRP-R9-SHARE-FORWARD-PRIVACY-001` cannot become verified until current-head integration/temporal/recovery/E2E evidence is green. `TDRP-R9-SHARE-RECIPIENT-ELIGIBILITY-001` cannot become verified until the shipping picker can enumerate and authorize the complete applicable destination universe (including Group/Channel and typed child destinations) and exact-head Electron/temporal/recovery evidence is green. Send-other/inline/game requests currently fail closed because their permission axes are not yet represented by the canonical Conversation owner. There is still no second persisted Message or Search owner.

## Exact-head evidence routing

The Rust desktop runtime now executes the complete `native/mahayana-messaging` contract suite and uploads a machine-readable `rust-messaging-responsibilities-<sha>` artifact. Both that workflow and the macOS exact-source packaging workflow include `projects/telegram-desktop-rust/**` in their push path filters, so a traceability change cannot become the live `main` HEAD without generating fresh runtime and packaging evidence for that same HEAD. Until those current-head runs are terminal-success, all ShareBox rows remain `implemented`, not `verified`.

## Lifecycle, failure and concurrency summary

The source does not treat Share as a stateless modal. Important transferable responsibilities include request→query correlation, debounce/cache, stale replacement handling, nested thread lifetimes, asynchronous restriction refresh, approval revalidation, duplicate-submit fencing, multi-request settlement, source-item disappearance, destination authorization, and explicit send errors. These must be represented by existing Fabushi owners before this source file can become mapped/verified.

## UI replacement boundary

Pure Qt paint/style mechanics (`Painter`, exact columns/pixels, Telegram style records) are presentation-replaced. Their product effects are not omitted: active-row keyboard semantics, responsive target layout, loading/empty/not-found behavior, selection indication, focus/close rules, restriction badge meaning, and feedback/toast semantics stay in scope under FDS-001/FUI-001/FSP-001/FVA-001.

## Coverage accounting

This dossier supports the following narrow counter movement only:
- `unread: 15788 -> 15779` (the original ShareBox `.cpp/.h` pair plus the separately recorded `share_box.style`, `Telegram/cmake/td_ui.cmake`, `ui/chat/forward_options_box.{h,cpp}`, `history_view_schedule_box.{h,cpp}`, and `api/api_common.h` reads)
- `unknown: 15788` unchanged
- `omitted: 0` unchanged
- `baseline_ready: false`
- `acceptance.accepted: false`

No production responsibility is declared verified by this read alone. Seven narrow ShareBox responsibilities are now machine-readable and implemented (`TDRP-R9-SHARE-SEND-OPTIONS-001`, `TDRP-R9-SHARE-DESTINATION-POLICY-001`, `TDRP-R9-SHARE-LINK-ENTITY-OFFSET-001`, `TDRP-R9-SHARE-THREAD-DESTINATION-001`, `TDRP-R9-SHARE-TOPIC-ELIGIBILITY-001`, `TDRP-R9-SHARE-FORWARD-IDEMPOTENCY-001`, `TDRP-R9-SHARE-FORWARD-PRIVACY-001`), but current-head GitHub Actions and independent evidence remain pending and the other SB-01…SB-08 slices remain open. Therefore `unknown` stays 15,788 and ShareBox is not mapped/verified as a whole.


### Send-menu dependency authority

| path | blob | read status | classification | mapping status |
| --- | --- | --- | --- | --- |
| `Telegram/SourceFiles/menu/menu_send.h` | `8f6c908d85fd448441f5b451617d502a6fa3c568` | complete, 2,203 bytes | typed send-menu/action contract | open |
| `Telegram/SourceFiles/menu/menu_send.cpp` | `1bbd5494928f7fa08192cb56bd40146dc0ec4d5f` | complete, 30,851 bytes | send-mode eligibility, shortcuts, effects and unread-thread actions | open |

These files make SB-03/SB-07 stricter than a generic schedule/silent pair. `SendMenu::Type` distinguishes disabled, reminder, silent-only and scheduled-to-user eligibility; send-when-online is emitted only for the scheduled-to-user case, reminder suppresses silent send, and silent-only suppresses scheduling. Keyboard shortcuts call the same typed policy rather than bypassing it. The same menu also carries effect, spoiler, caption placement, photo-quality, cover and paid-price applicability, while its thread-level unread actions preserve topic/sublist identity. Fabushi must therefore model these as typed applicability/trigger responsibilities in existing Message/Presence/Conversation/Composer owners; the two files are read-complete but remain unknown because those responsibilities are not fully mapped, shipping-composed and evidenced.

Coverage after this read only: `unread=15,769`, `unknown=15,788`, `omitted=0`, `baseline_ready=false`, `acceptance.accepted=false`.


### Thread/send-detail dependency authority

| path | blob | read status | classification | mapping status |
| --- | --- | --- | --- | --- |
| `Telegram/SourceFiles/menu/menu_send_details.h` | `cf57a6cd8c6b90aece73443572b84c464f016a30` | complete, 1,274 bytes | typed send eligibility/applicability contract | open |
| `Telegram/SourceFiles/api/api_common.cpp` | `9e86c94abff19adeacf898dd33d2206e9c33913d` | complete, 1,287 bytes | send-when-online/thread routing transport projection | open |
| `Telegram/SourceFiles/data/data_thread.h` | `9756e25020f83f91a89fbedd1810205df7178a27` | complete, 4,262 bytes | canonical upstream Thread lifecycle contract | open |
| `Telegram/SourceFiles/data/data_thread.cpp` | `c1367fdcdb86c22e2b484e648f08edf8854a9478` | complete, 5,986 bytes | typed topic/sublist identity, unread/notification/pin/active-child behavior | open |

The send details make applicability typed rather than boolean: Disabled, SilentOnly, Scheduled, ScheduledToUser, Reminder and EditCommentPrice are distinct modes, while spoiler/caption/photo-quality/cover/effect/price are independent axes. `DefaultSendWhenOnlineOptions` confirms that Telegram's special timestamp is a transport encoding of a presence-triggered send contract and may inherit Ctrl-silent state; Fabushi must model the trigger semantically rather than copy the sentinel.

`Data::Thread` is the common upstream abstraction for history, forum topic and saved-message sublist. Its identity is not reducible to one topic root: topic uses `topicRootId()`, saved sublist uses `monoforumPeerId()/maybeSublistPeer()`, and all share notification queues, unread mention/reaction/poll-vote state, mute/pin state and active-subsection continuity. Its `canToggleUnread` rules also differ by topic/forum/community/self-saved-sublist/monoforum-admin. This confirms SB-02 must introduce a source-neutral typed child destination/state representation inside existing Conversation owners rather than coercing saved/community child identity into `thread_root_message_id = topic:<id>`. No new SavedMessages product owner is justified.

Coverage after these complete reads only: `unread=15,765`, `unknown=15,788`, `omitted=0`, `baseline_ready=false`, `acceptance.accepted=false`.

### Story-share caller authority

| path | blob | read status | classification | mapping status |
| --- | --- | --- | --- | --- |
| `Telegram/SourceFiles/media/stories/media_stories_share.cpp` | `cac84bc4f383fdc260e046ead9ee1fa96738a2f8` | complete, 320 lines | ShareBox caller policy, paid approval, fan-out settlement and send-option projection | open |
| `Telegram/SourceFiles/media/stories/media_stories_share.h` | `c7149b0ca980a85c8592676f1cbc91abd1c6bafa` | complete, 35 lines | public story-share entrypoints and share-at-time context contract | open |

The header and caller were read in full at accepted upstream `0a11537c220df35fe59a96aebe413b1e52bc6e47`. The header fixes the public entrypoint contract (`PrepareShareBox`, `PrepareShareAtTimeBox`, `FormatShareAtTime`), including the item + video timestamp context that must survive navigation/share projection rather than being treated as incidental presentation. The implementation adds transferable responsibilities beyond the ShareBox root itself: source disappearance revalidation; pre-exposure destination filtering using the exact required send right plus inline/media rights; a user exception that may ignore money restrictions without bypassing other canonical policy; duplicate-submit fencing; a second error check at submit time; paid approval revalidation before any send; single- and multi-message counting when an optional comment accompanies the story; deterministic link-plus-comment entity offset shifting; typed schedule/repeat/silent/quick-reply/effect/suggest/caption-invert options; per-destination Stars consumption bounded by both destination price and remaining approval; independent per-destination random request identity; partial success/failure settlement; and closing the surface only when the entire fan-out request set settles.

The Fabushi mapping must stay in existing Search/Conversation/Message/Payment/Composer owners. In particular, recipient authorization must occur before exposure and again at submission, approved value must be consumed per destination rather than treated as a global boolean, and fan-out completion must not be inferred from the first successful destination. No StoryShare, TelegramPayments, or second Message owner is justified by this source.

This read only advances source completeness. It does not verify any responsibility because the paid approval, full destination universe, and packaged shipping evidence remain incomplete.

Coverage after the story-share header + implementation complete reads only: `unread=15,763`, `unknown=15,788`, `omitted=0`, `baseline_ready=false`, `acceptance.accepted=false`.



## Production wiring updates and ledger symbol synchronization (Wave 1~3: W1-A, W1-B, W2-C, W2-E, W2-G, W3-A, W3-C)

The machine-readable ledger records seven product responsibilities mapped to this dossier and upstream ShareBox authority (`f4787dbb34cdb5fb2af78a69ae7537e91e06395a`). Below documents the authoritative production wiring, traceability IDs, and exact literal code symbols across Wave 1 to Wave 3 slices.

### 1. TDRP-R9-SHARE-SEND-OPTIONS-001 (Send Options, Modes and Send Menu Derivation)
- **Traceability**:
  - `requirement_ids`: `TDRP-R9-SHARE-SEND-OPTIONS-001`
  - `oracle_ids`: `ORA-TDRP-SHARE-SEND-OPTIONS-001`
  - `invariant_ids`: `INV-TDRP-SHARE-SCHEDULE-PRESERVE-001`, `INV-TDRP-SHARE-SILENT-PRESERVE-001`
- **Fabushi Target Paths**:
  - `native/mahayana-messaging/src/protocol.rs`
  - `native/mahayana-messaging/src/service.rs`
  - `native/mahayana-messaging/src/engine.rs`
  - `native/mahayana-messaging/src/message.rs`
  - `frontend/src/production/forward-settlement.ts`
  - `frontend/src/production/forward-settlement.contract.test.ts`
- **Authoritative Production Symbols**:
  - `scheduled_at_ms`: timestamp field on canonical message protocol/struct.
  - `silent`: boolean flag on canonical message protocol/struct.
  - `SendMenuMode`: enum representing standard, silent, or scheduled send modes.
  - `SendMenuDetails`: typed structure containing send options and mode constraints.
  - `resolve_send_menu_details`: pure function deriving menu modes from destination attributes.
  - `resolve_destination_send_menu_details`: helper binding recipient capabilities to menu details.
  - `settleForwardProjection`: frontend helper reconciling forward completion state.
  - `resolve_send_menu_details_from_inputs`: pure function resolving send details from user inputs.

### 2. TDRP-R9-SHARE-DESTINATION-POLICY-001 (Destination Authorization and Send Policies)
- **Traceability**:
  - `requirement_ids`: `TDRP-R9-SHARE-DESTINATION-POLICY-001`
  - `oracle_ids`: `ORA-TDRP-SHARE-DESTINATION-POLICY-001`
  - `invariant_ids`: `INV-TDRP-FORWARD-NO-PERMISSION-BYPASS-001`, `INV-TDRP-FORWARD-NO-QUEUE-ON-DENY-001`
- **Fabushi Target Paths**:
  - `native/mahayana-messaging/src/search.rs`
  - `native/mahayana-messaging/src/engine.rs`
  - `source/host/src/extensions/session/production.rs`
- **Authoritative Production Symbols**:
  - `DestinationAuthorizationDenial`: typed error enum for forward destination authorization rejections.
  - `authorize_destination`: pure policy verifying destination conversation and topic permissions before forwarding.
  - `MessageSendPermissionDenied`: denial when user lacks standard send permissions.
  - `CommunitySendRestricted`: denial when community member restrictions forbid posting.
  - `OtherSendPermissionDenied`: denial when user lacks send_other permission.
  - `CommunityOtherRestricted`: denial when community restrictions forbid sending other/links.
  - `message_content_uses_other`: helper inspecting message payload for links or rich entities.
  - `deliver_forward_batch_while_locked`: Session method atomically executing destination forward batch while holding session lock.

### 3. TDRP-R9-SHARE-LINK-ENTITY-OFFSET-001 (ShareLink Entity Offsets, Paid Send & Forward With Comment)
- **Traceability**:
  - `requirement_ids`: `TDRP-R9-SHARE-LINK-ENTITY-OFFSET-001`
  - `oracle_ids`: `ORA-TDRP-SHARE-LINK-ENTITY-OFFSET-001`
  - `invariant_ids`: `INV-TDRP-SHARE-LINK-UTF16-OFFSET-001`, `INV-TDRP-SHARE-LINK-ATOMIC-OVERFLOW-001`
- **Fabushi Target Paths**:
  - `native/mahayana-messaging/src/message.rs`
  - `native/mahayana-messaging/src/payment.rs`
  - `native/mahayana-messaging/src/story.rs`
  - `native/mahayana-messaging/src/engine.rs`
  - `source/host/src/extensions/session/production.rs`
  - `source/host/src/extensions/session/gateway.rs`
  - `frontend/src/production/ForwardMessageDialog.tsx`
- **Authoritative Production Symbols**:
  - `prepend_plain_text`: pure function shifting entity offsets in UTF-16 code units upon prepending link text.
  - `format_share_at_time`: story share helper appending video timestamp parameters.
  - `append_share_at_time_query`: URL query builder preserving timestamp anchors.
  - `StoryShareContext`: context structure holding story item metadata and video timestamp.
  - `PaidSendPlan`: structured calculation of required payment for paid destination messages.
  - `PaidSendApprovalConsumer`: approval tracking and revalidation boundary.
  - `compute_paid_send_plan`: pure function computing aggregate cost across recipient set.
  - `revalidate_and_consume_paid_send`: policy ensuring approved credit covers actual send.
  - `resolve_paid_restriction_from_capabilities`: pure evaluation of paid send requirements.
  - `compose_share_comment`: pure comment composer in `fabushi_messaging_core` (`message.rs`), exposed on `ProductionSessionWorkers`.
  - `validate_formatted_text`: pure UTF-16 bounds and overflow validator called at gateway and Session boundaries.
  - `forward_human_message_with_comment`: Session method dispatching comment text with deterministic nonce `{clientNonce}:comment` prior to forwarded payload.
  - `forwardHumanMessage`: Host gateway IPC route processing forward requests.
  - `ForwardMessageDialog`: frontend dialog component managing recipient selection and forward submission.
  - `commentText`: controlled React state binding user input in `ForwardMessageDialog.tsx`.

### 4. TDRP-R9-SHARE-THREAD-DESTINATION-001 (Thread and Child Destination Routing & Lifecycle)
- **Traceability**:
  - `requirement_ids`: `TDRP-R9-SHARE-THREAD-DESTINATION-001`
  - `oracle_ids`: `ORA-TDRP-SHARE-THREAD-DESTINATION-001`
  - `invariant_ids`: `INV-TDRP-FORWARD-THREAD-NOT-FLATTENED-001`, `INV-TDRP-FORWARD-LEGACY-NONE-001`
- **Fabushi Target Paths**:
  - `native/mahayana-messaging/src/conversation.rs`
  - `native/mahayana-messaging/src/protocol.rs`
  - `native/mahayana-messaging/src/service.rs`
  - `native/mahayana-messaging/src/engine.rs`
  - `native/mahayana-messaging/src/message.rs`
  - `native/mahayana-messaging/tests/m6_channels_topics_contract.rs`
- **Authoritative Production Symbols**:
  - `ConversationChildIdentity`: typed identity distinguishing Topic root message, SavedSublist participant, and nested conversation.
  - `ConversationDestination`: target specification routing to base chat or typed child thread.
  - `ConversationDestinationSelection`: selection state tracking active target and handling child destruction.
  - `pub fn remove_messages(`: method removing deleted messages while preserving child cursors.
  - `pub fn clear_draft_reply_to(`: method clearing draft replies upon thread deletion.
  - `pub fn remove_deleted_messages(`: batch deletion helper maintaining child thread consistency.
  - `pub fn can_toggle_unread(`: permission check verifying thread unread state toggle eligibility.
  - `saved_sublist_participant_for_message`: helper resolving participant binding for Saved Messages.
  - `migrate_legacy_topic_child_states`: migration adapter upgrading legacy topic cursors to typed child states.
  - `thread_root_message_id`: metadata field on canonical Message preserving destination thread identity.

### 5. TDRP-R9-SHARE-TOPIC-ELIGIBILITY-001 (Forum Topic Validation and Forward Eligibility)
- **Traceability**:
  - `requirement_ids`: `TDRP-R9-SHARE-TOPIC-ELIGIBILITY-001`
  - `oracle_ids`: `ORA-TDRP-SHARE-TOPIC-ELIGIBILITY-001`
  - `invariant_ids`: `INV-TDRP-FORWARD-TOPIC-EXISTS-001`, `INV-TDRP-FORWARD-TOPIC-OPEN-VISIBLE-001`, `INV-TDRP-FORWARD-TOPIC-NO-QUEUE-ON-DENY-001`
- **Fabushi Target Paths**:
  - `native/mahayana-messaging/src/engine.rs`
  - `native/mahayana-messaging/tests/m6_channels_topics_contract.rs`
- **Authoritative Production Symbols**:
  - `Command::ForwardMessage`: canonical engine command executing forward dispatch.
  - `topic_id_from_root`: helper deriving topic identifier from thread root message id.
  - `EngineError::ForumTopicNotFound`: fail-closed rejection when destination topic does not exist.
  - `EngineError::ForumTopicClosed`: fail-closed rejection when destination topic is closed to posting.

### 6. TDRP-R9-SHARE-FORWARD-IDEMPOTENCY-001 (Forward Batch Idempotency, Journaling & Replay)
- **Traceability**:
  - `requirement_ids`: `TDRP-R9-SHARE-FORWARD-IDEMPOTENCY-001`, `NMR-47`, `NMR-47g`, `NMR-AC-38`
  - `oracle_ids`: `ORA-TDRP-SHARE-FORWARD-IDEMPOTENCY-001`
  - `invariant_ids`: `INV-TDRP-FORWARD-RETRY-ONE-MESSAGE-001`, `INV-TDRP-FORWARD-ID-CONFLICT-FAIL-CLOSED-001`
- **Fabushi Target Paths**:
  - `native/mahayana-messaging/src/service.rs`
  - `native/mahayana-messaging/tests/service_contract.rs`
  - `source/host/src/extensions/session/production.rs`
  - `source/host/src/extensions/session/forward_batch_journal.rs`
  - `source/host/src/extensions/session/agent_db.rs`
  - `source/host/src/extensions/session/native_messaging.rs`
  - `source/host/tests/session_gateway_contract.rs`
  - `frontend/src/production/ForwardMessageDialog.tsx`
- **Authoritative Production Symbols**:
  - `stable_message_id`: actor-scoped client nonce providing deterministic message identity.
  - `fn idempotent_send_replay(`: service replay logic ensuring duplicate requests return existing message.
  - `fn project_command(`: engine projection handler mapping commands to events.
  - `pub fn forward_human_message(`: Session worker method initiating forward batch delivery.
  - `fn resume_human_forward_batches(`: Session startup recovery resuming pending batches from SQLite journal.
  - `fn pending_human_forward_is_batch_owned(`: helper isolating batch-owned pending messages during recovery.
  - `store_forward_batch`: journal persistence writing batch state and destination completion progress.

### 7. TDRP-R9-SHARE-RECIPIENT-ELIGIBILITY-001 (Pre-Exposure Recipient Discovery and Authorization)
- **Traceability**:
  - `requirement_ids`: `TDRP-R9-SHARE-RECIPIENT-ELIGIBILITY-001`
  - `oracle_ids`: `ORA-TDRP-SHARE-RECIPIENT-ELIGIBILITY-001`
  - `invariant_ids`:
    - `INV-TDRP-RECIPIENT-AUTH-BEFORE-EXPOSURE-001`: Destinations are filtered before exposure; unpermitted peers are never exposed.
    - `INV-TDRP-RECIPIENT-MEDIA-POLL-ELIGIBILITY-001`: Destinations enforce media and poll policy constraints prior to display.
    - `INV-TDRP-RECIPIENT-CHANNEL-POSTING-001`: Broadcast channels require administrative posting rights.
- **Fabushi Target Paths**:
  - `native/mahayana-messaging/src/search.rs`
  - `native/mahayana-messaging/src/protocol.rs`
  - `native/mahayana-messaging/src/service.rs`
  - `source/host/src/extensions/session/production.rs`
  - `source/host/src/extensions/session/gateway.rs`
  - `frontend/src/production/ForwardMessageDialog.tsx`
  - `native/mahayana-messaging/src/conversation.rs`
  - `native/mahayana-messaging/src/community.rs`
  - `native/mahayana-messaging/tests/m5_contacts_groups_contract.rs`
- **Authoritative Production Symbols**:
  - `RecipientSearchRequirements`: specification of required permissions (media, poll, other) for candidate discovery.
  - `RecipientAuthorizationInput`: actor and conversation permission snapshot evaluated before exposure.
  - `recipient_search_authorized`: pure policy function evaluating whether candidate destination may be displayed.
  - `ClientCommand::SearchRecipients`: protocol command for recipient querying.
  - `fn recipient_search_envelope(`: service boundary enforcing authorization envelope around search index.
  - `fn actor_can_send_to_recipient(`: helper evaluating send eligibility for recipient.
  - `pub fn search_human_recipients(`: Session worker method returning filtered candidate recipients.
  - `pub can_send_other: bool`: permission field on `ConversationPermissions`.
  - `pub send_other: bool`: restriction field on `MemberRestrictions`.
  - `human_recipient_requirements_from_entry`: helper deriving requirements from source entry metadata.
  - `human_recipient_authorization_from_metadata`: helper constructing authorization input from session metadata.

### 8. Cross-Slice Production Wiring Synchronization (Wave 1~3)
- **W1-A (Forward with Comment, S1a)**:
  - `compose_share_comment` and `validate_formatted_text` in `fabushi_messaging_core` (`message.rs`).
  - `forward_human_message_with_comment` in `ProductionSessionWorkers` (`production.rs`).
  - Deterministic comment nonce `{clientNonce}:comment` and per-destination `commentCompleted` tracking in `forward_batch_journal.rs`.
- **W1-B (Mini App Bridge Isolation, S5)**:
  - `forwardContext: null` explicit isolation in Mini App messaging preventing conflict with forward batch journals.
- **W2-C (Session Group / Channel / Topic Lifecycle, ADR-005 Steps 2-5)**:
  - `ProductionSessionWorkers::create_group_conversation`, `create_channel_conversation`, `upsert_conversation_topic`, `list_conversation_topics`, `upsert_conversation_member`, `get_conversation_member`, `list_conversation_members`.
  - `ProductionSessionWorkers::forward_human_message_to_destinations`, `append_human_message_with_topic`, `append_human_message_with_topic_and_context`.
  - `SandAgentDb::upsert_conversation_topic`, `SandAgentDb::get_conversation_topic`, `SandAgentDb::list_conversation_topics`.
  - `SandAgentDb::upsert_conversation_member`, `SandAgentDb::get_conversation_member`, `SandAgentDb::list_conversation_members`.
  - `dispatch_production_session_gateway_call_with_content_search_and_group_chat`.
- **W2-E (Frontend Topic Selector & Transcript Projection)**:
  - `ForwardRecipient` interface extended with `topicId?: string`, `topicTitle?: string`, `topicClosed?: boolean`, `topicDeleted?: boolean`.
  - `destinationTopics` mapping in `ForwardMessageDialog.tsx` and `ProductionRenderer.tsx`.
  - `filterTranscriptEntriesByTopic` in `frontend/src/production/transcript-projection.ts`.
  - `TranscriptMessage` interface extended with optional readonly `topicId?: string`.
  - `mergeAuthoritativeTranscriptMessage` preserving topic identity during transcript merge.
- **W2-G (S4 Host Wiring: Pin Lifecycle, Topic Mute, Typing Throttle & Media Preflight)**:
  - `ProductionSessionWorkers::pin_conversation_message`, `unpin_conversation_message`, `list_conversation_pins`.
  - `ProductionSessionWorkers::set_conversation_topic_mute`, `evaluate_message_notification`.
  - `ProductionSessionWorkers::publish_conversation_typing`.
  - Gateway IPC registrations: `pinConversationMessage`, `unpinConversationMessage`, `listConversationPins`, `setConversationTopicMute`, `evaluateMessageNotification`, `publishConversationTyping`, `validateMedia`.
- **W3-A (ShareLink & S1 Closure)**:
  - `prepend_plain_text` entity offset shifts and `ForwardMessageDialog.tsx` comment submission wired end-to-end.
- **W3-C (S4 Media Cache & S5 Mini App Extension)**:
  - Scoped media cache quota management (`prune_media_cache`, `resolve_media_cache_path`).
  - Attachment preflight validation with 6-batch count limit (`validate_media_attachments`, `validateMedia`).
