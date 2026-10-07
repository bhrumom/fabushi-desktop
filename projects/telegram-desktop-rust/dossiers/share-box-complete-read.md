# ShareBox complete source read — source completeness evidence

Status: read-complete / responsibility closure open  
Project: TDRP-001 Revision 9  
Accepted upstream: `telegramdesktop/tdesktop@f23c37857220eb84f8559f0901ea26fb304b564b`  
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

Disposition: map to canonical Conversation/Topic typed destination and canonical Participant/Search picker. No second Group/Channel/Topic app. `18289a68a55a3166f42013b9ea8cb84b7565faf1` adds `thread_root_message_id` to the canonical forward protocol/service/engine path and preserves it on the destination Message, with a serde default for legacy payloads. Machine-readable responsibility: `TDRP-R9-SHARE-THREAD-DESTINATION-001`; oracle `ORA-TDRP-SHARE-THREAD-DESTINATION-001`; invariants `INV-TDRP-FORWARD-THREAD-NOT-FLATTENED-001` and `INV-TDRP-FORWARD-LEGACY-NONE-001`. This slice is implemented, not verified. Exact shipping picker lifecycle/destruction, saved-message sublist/community-child mapping, topic eligibility and E2E remain open.

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

Disposition: existing messaging domain owns the state. `824daf3aefe47c747d8abe648108265b17d8255f` now preserves `scheduled_at_ms` and `silent` through the canonical forward protocol/service/engine/Message path. Machine-readable responsibility: `TDRP-R9-SHARE-SEND-OPTIONS-001`; oracle `ORA-TDRP-SHARE-SEND-OPTIONS-001`; invariants `INV-TDRP-SHARE-SCHEDULE-PRESERVE-001` and `INV-TDRP-SHARE-SILENT-PRESERVE-001`. This slice is implemented, not verified. Open gaps still include reminder/schedule-until-online semantics and exact UI menu/picker eligibility/evidence.

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
- `native/mahayana-messaging/src/message.rs` — `Message.forward_origin`, reply relation, stable/client identities.
- Host transcript send pipeline under `source/host/src/extensions/transcript/` owns existing Bot send acceptance/fanout and must not regress.

Disposition: reuse canonical Message/Conversation/Host send owners. Open gaps: explicit forward-option privacy model (drop names/captions), multi-destination settlement oracle, repeat schedule, suggested/effect/video/ephemeral equivalence where applicable, and current-head tests.

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

Disposition: extend canonical Conversation permissions plus Search/provider authorization; never “search everything then hide in UI.” `470171e326134da2d3ab5753de331aa85552744d` now makes the native `ForwardMessage` path fail closed on destination participant/message/media/poll policy, community/channel restrictions, slow mode, and secret-chat content constraints before queueing a destination message. Machine-readable responsibility: `TDRP-R9-SHARE-DESTINATION-POLICY-001`; oracle `ORA-TDRP-SHARE-DESTINATION-POLICY-001`; invariants `INV-TDRP-FORWARD-NO-PERMISSION-BYPASS-001` and `INV-TDRP-FORWARD-NO-QUEUE-ON-DENY-001`. This slice is implemented, not verified. Recipient pre-exposure parity for inline/game/send-other rights remains open.

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

## Lifecycle, failure and concurrency summary

The source does not treat Share as a stateless modal. Important transferable responsibilities include request→query correlation, debounce/cache, stale replacement handling, nested thread lifetimes, asynchronous restriction refresh, approval revalidation, duplicate-submit fencing, multi-request settlement, source-item disappearance, destination authorization, and explicit send errors. These must be represented by existing Fabushi owners before this source file can become mapped/verified.

## UI replacement boundary

Pure Qt paint/style mechanics (`Painter`, exact columns/pixels, Telegram style records) are presentation-replaced. Their product effects are not omitted: active-row keyboard semantics, responsive target layout, loading/empty/not-found behavior, selection indication, focus/close rules, restriction badge meaning, and feedback/toast semantics stay in scope under FDS-001/FUI-001/FSP-001/FVA-001.

## Coverage accounting

This dossier supports the following narrow counter movement only:
- `unread: 15788 -> 15786`
- `unknown: 15788` unchanged
- `omitted: 0` unchanged
- `baseline_ready: false`
- `acceptance.accepted: false`

No production responsibility is declared verified by this read alone. Four narrow responsibilities are now machine-readable and implemented (`TDRP-R9-SHARE-SEND-OPTIONS-001`, `TDRP-R9-SHARE-DESTINATION-POLICY-001`, `TDRP-R9-SHARE-LINK-ENTITY-OFFSET-001`, `TDRP-R9-SHARE-THREAD-DESTINATION-001`), but current-head GitHub Actions and independent evidence remain pending and the other SB-01…SB-08 slices remain open. Therefore `unknown` stays 15,788 and ShareBox is not mapped/verified as a whole.
