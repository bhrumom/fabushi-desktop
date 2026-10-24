# strict129 complete read — Replies / thread / shared-media attribution

Status: mapped-open
Project: TDRP-001 Revision 9
Source repository: `telegramdesktop/tdesktop`
Accepted upstream commit: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`
Accepted upstream tree: `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`
Exact source path: `tasks/2026/08/19/align-replies-thread-shared-media-attribution.md`
Exact source blob: `ced812fb2b3a26d44a5e972fdc57d7a1dff4414c`
Parent Fabushi exact HEAD: `cd71c1709bd5a7f8830c553611a0b15f2d7a6b27`

## Complete read

The exact 3,354-byte task was read in full. Its correction is not “use the reply root everywhere.” The task proves three different attribution concepts must stay distinct:

- a message reply target / replies root,
- the active topic or canonical thread root,
- the shared-media storage scope key.

In upstream `TopControls`, building the pinned shared-media key from `_repliesRootId` could name `(discussionGroup, repliesRootId, 0)`, but no such per-thread shared-media list can exist for a non-forum discussion group. The corrected code takes the topic root from `activeThread()->topicRootId()`, matching the existing `PinnedTracker` ownership.

The task also records two negative facts that matter for migration: the alleged replies-root shared-media list is absent rather than stale, and a channel-comments replies view without a topic has no pinned bar in the first place. Therefore no Fabushi implementation may fabricate a per-reply-root shared-media scope to “repair” a state that should not exist.

The rejected alternative is equally important: `HistoryItem::topicRootId()` cannot be redefined as the reply root because it also feeds persisted draft identity and outgoing protocol fields. Upstream already has the separate `replyToTop()` concept for reply-root attribution.

## Exact accepted-source anchors

The following accepted upstream blobs were read at the locked commit:

- `Telegram/SourceFiles/history/view/controls/history_view_top_controls.cpp@ddd3324c45574586daf56dc8bf0462c4c9e79107`: `activeThread()` selects sublist/topic/history; pinned shared-media construction uses `thread->topicRootId()` and the dedicated monoforum field.
- `Telegram/SourceFiles/history/view/history_view_pinned_tracker.cpp@be2420005132800cb019860c072beb5fd67a00b2`: the tracker keys pinned media from the thread's topic root and monoforum peer.
- `Telegram/SourceFiles/data/data_thread.cpp@c1367fdcdb86c22e2b484e648f08edf8854a9478`: `Thread::topicRootId()` is the forum-topic root or zero; `monoforumPeerId()` is independently derived from a sublist.
- `Telegram/SourceFiles/data/data_shared_media.cpp@990c48363dc32791f02ec492d9292d45ad97fe99`: storage/view updates are scoped by peer, topic root, monoforum peer and media type.
- `Telegram/SourceFiles/apiwrap.cpp@342e501177996be3c083ba7985d3720a53100b51`: invalid nonzero topic/sublist scopes are rejected before `SharedMediaAddSlice`; direct-link export explicitly chooses topic root for forums and `replyToTop()` otherwise.
- `Telegram/SourceFiles/history/history_item.cpp@b753d3ea7bd5fde15cdad206d17ab3eee6d68cab`: `replyToTop()` and `topicRootId()` are separate; pinned/shared-media writers only create topic-scoped state when a real forum topic exists.

These are product-state attribution and persistence responsibilities. Qt widgets and Telegram-specific protocol/storage keys are source provenance, not the target architecture.

## Existing-owner-first audit

The parent Fabushi tree is non-truncated. Existing canonical owners already separate reply and thread identities:

- `native/mahayana-messaging/src/message.rs@43befb0734a6d1d6d17aeb11f6aa5608a16a6f5a` has one canonical `Message` with distinct `reply_to_message_id` and `thread_root_message_id`, plus canonical pinned state.
- `source/shared/transcript.ts@37d025c238b84828546920d046ba9883fa181c5a` owns main/thread transcript projection and handles missing/cyclic reply ancestry fail-closed.
- `source/shared/transcript-threads.ts@156e959d2cb26fdd8ea29ddbab3d5c168a9c7499` owns branch reply counts/descendant projection without introducing another message store.
- `source/host/src/extensions/transcript/transcript_manager.rs@3d86d37255aebeff06707ed5888fb722b9760a73` and `transcript_store.rs@140976b7cbb8bce9650f41bd18a76e206a8cf18d` remain the single Host transcript manager/store.
- `source/host/src/extensions/transcript/send_thread_stamping.rs@e0c37baf3c70eda870515e55edd980bf4eb82aa1` resolves reply targets only when a live entry exists and strips invalid/in-flight AI reply targets.
- `frontend/src/recovered/features/conversation/workspace/model.ts@e8bf075b7ed1fe6d8a8493c6fba585d73640303a` keeps canonical `TranscriptMessage.replyToId`.
- `frontend/src/recovered/features/conversation/workspace/reply-thread-controller.ts@99ea6e8a4b88ac21b4d3eddcfbacb73dde835365` is the canonical reply selection/navigation/projection owner.
- `frontend/src/recovered/features/conversation/cards/transcript-card/thread-summary-controller.ts@233fe27c5b9f9765a96975da902d0a860c05171d` keeps thread counts keyed by explicit root IDs and drops unknown/malformed/non-positive summaries.
- `frontend/src/recovered/features/conversation/cards/transcript-card/thread-loader.ts@458f57d1df313f0dd478ba162ef8646e4c293c89` is the existing thread-loading owner with account/agent/root scope and generation fencing.

No TelegramMessage, TelegramThread, TelegramSharedMediaStore, second Transcript store, or second Conversation root is justified.

## Shipping composition and fail-closed gap

The shipping `frontend/src/production/ProductionRenderer.tsx@af8a83a8dd7cf1444474889ce3e64fe796a7d458` directly composes `createReplyThreadController`, refreshes it from the canonical entries, projects `replyToId` into drafts/submissions, and passes reply preview/jump/reply actions into the one `ConversationTranscript`.

That proves reply attribution is shipping-wired.

The same production root currently sets `threadRootId: null` and `getThreadSummary: () => null`. It does not compose `createThreadSummaryController`, `createThreadLoader`, or a `getAgentThread` transport path. Consequently the full thread-summary/thread-loader/shared-media attribution path is **not** shipping-wired and must remain mapped-open. The absence is fail-closed: production does not invent a thread count or a per-reply-root media list.

A future implementation must extend the same Message/Transcript/Conversation/Resource owners so any pinned/shared-media collection key uses the canonical conversation/thread scope, never a reply target merely because the UI is displaying replies. Missing or invalid thread scope must not expose cross-thread media or fabricate per-thread state.

## UI / non-UI applicability

Telegram's pinned bar, Qt controls and storage types are not copied. Applicable UI responsibility is expressed through the existing Fabushi `ConversationTranscript`, reply preview, Composer reply target, and typed thread affordance slots.

Applicable non-UI responsibility is stronger: reply relation, thread root and shared-media scope remain independent typed concepts through persistence, navigation, loading, retry/recovery and future indexing. Rebinding `thread_root_message_id` from `reply_to_message_id` without an explicit thread transition is forbidden.

Because thread summary/loader and a canonical thread-scoped shared-media/pinned projection are not yet shipping-composed, strict129 does not claim complete thread/pinned-media implementation, packaged E2E, or release acceptance.

## Traceability anchors

Requirement / oracle / invariant IDs for this mapped-open responsibility:
- `TDRP-R9-REPLY-THREAD-SHARED-MEDIA-ATTRIBUTION-001`
- `ORA-TDRP-REPLY-THREAD-ATTRIBUTION-EXACT-TASK-SOURCE-001`
- `ORA-TDRP-REPLY-THREAD-ATTRIBUTION-UPSTREAM-ROOT-SEPARATION-001`
- `ORA-TDRP-REPLY-THREAD-ATTRIBUTION-CANONICAL-MESSAGE-SEPARATION-001`
- `ORA-TDRP-REPLY-THREAD-ATTRIBUTION-SHIPPING-REPLY-WIRED-001`
- `ORA-TDRP-REPLY-THREAD-ATTRIBUTION-THREAD-SUMMARY-FAIL-CLOSED-001`
- `ORA-TDRP-REPLY-THREAD-ATTRIBUTION-SHIPPING-THREAD-GAP-001`
- `INV-TDRP-REPLY-THREAD-ATTRIBUTION-NO-REPLY-ROOT-AS-THREAD-ROOT-001`
- `INV-TDRP-REPLY-THREAD-ATTRIBUTION-NO-INVALID-SHARED-MEDIA-KEY-001`
- `INV-TDRP-REPLY-THREAD-ATTRIBUTION-NO-SECOND-MESSAGE-TRANSCRIPT-OWNER-001`
- `INV-TDRP-REPLY-THREAD-ATTRIBUTION-NO-FAKE-THREAD-SUMMARY-001`
- `INV-TDRP-REPLY-THREAD-ATTRIBUTION-EXACT-HEAD-ACTIONS-ONLY-001`

## Parent exact-head gate and repair provenance

The originally requested parent `9f3430593562f2045cf6e2233d5bed9f2ba54213` exposed a real authority-validator defect in the strict128 row. The first minimal repair `3b9525454762cd3a789cdd709bb0b918fcec314e` replaced the nonexistent explanatory target symbol with the real `--fabushi-surface-raised`. Its authority validator then reported missing strict128 traceability IDs, so `cd71c1709bd5a7f8830c553611a0b15f2d7a6b27` added only those dossier anchors.

The same `3b952...` TDRP run also had a recursive shard fail while cloning GitLab libyuv with HTTP 520. No repository change was made for that transient network failure; the exact `cd71c1709bd5a7f8830c553611a0b15f2d7a6b27` rerun completed the same recursive authority successfully.

Parent `cd71c1709bd5a7f8830c553611a0b15f2d7a6b27` completed:
- TDRP `39977768466`: **61/61 success**
  - source authority job `120499220688`
  - authority `12512480894` `sha256:8f7e9ae81f58f70d42a1cc700910176a9299c115119c08f7a0f6288a05e326a5`
  - closure `12512497271` `sha256:8e9382df91474202a084342003c078fd1c7fb91277564e0da4f5173fef5fcf46`
  - reachability `12513071225` `sha256:2d427d12219ad9a53eae723226878ea379e54889bc82eed06591fd35f9f3264a`
- Rust desktop runtime `39977768469`: **6/6 success**
  - artifact `12512815138` `sha256:05c2ebb951a4c1e7e7b17b2d6f1c5aa25737a5e359ee4c2c1b9198c3ac45455f`

## Disposition

Requirement: `TDRP-R9-REPLY-THREAD-SHARED-MEDIA-ATTRIBUTION-001`
Disposition: `reply-thread-shared-media-attribution-mapped-open`

Counters after strict129: unknown=16,247; unread=16,222; omitted=0; responsibilities=147; unmapped_responsibilities=0.

Next deterministic root entry: `tasks/2026/08/20/stop-orphaning-general-id-shared-media-lists.md@c30808ee6b4ffb02561d243c24ec32eef5d5e909` (strict130).

Global source closure remains open; `baseline_ready=false`, `accepted=false`, and independent release acceptance is not complete.
