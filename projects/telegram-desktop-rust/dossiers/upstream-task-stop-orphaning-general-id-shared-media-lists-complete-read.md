# strict130 complete read — Non-forum GeneralId shared-media list lifetime

Status: mapped-open
Project: TDRP-001 Revision 9
Source repository: `telegramdesktop/tdesktop`
Accepted upstream commit: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`
Accepted upstream tree: `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`
Exact source path: `tasks/2026/08/20/stop-orphaning-general-id-shared-media-lists.md`
Exact source blob: `c30808ee6b4ffb02561d243c24ec32eef5d5e909`
Parent Fabushi exact HEAD: `46ca9cb537310f5be0c9e131360b8c5ee00497de`

## Complete read

The exact 3,150-byte task was read in full. Its responsibility is a producer/lifetime bug, not a presentation bug.

Historically, pinning a message called `SharedMediaAddExisting` with `HistoryItem::topicRootId()` even outside a forum. `topicRootId()` falls back to `Data::ForumTopic::kGeneralId`, whose exact value is `1`. In a non-forum peer that integer is not a real thread identity.

That distinction matters because `SharedMediaAddExisting` calls `enforceLists` and therefore creates the keyed collection. By contrast, normal `SharedMediaAddNew` always maintains the whole-peer list and only *finds* an already-existing topic key. One incorrect pin could therefore create a dead `(peer, 1, 0)` collection and cause every later regular message to be mirrored into it for the rest of the session.

The accepted source fixes only that producer: the topic-key add is guarded by `_history->asForum()`. The discriminator is intentionally the peer's forum identity, not `item->topic()`. A real forum message can have a valid topic root while its `ForumTopic` object is not loaded yet, so gating on `topic()` would lose valid thread indexing. The real General forum topic remains valid root `1`; monoforum/Saved Messages sublists retain their separate scope.

The unpin arm is intentionally asymmetric. `SharedMediaRemoveOne` uses `find` for optional topic/sublist collections, so removing from a key that was never created is a no-op. The correct invariant is therefore **do not create an optional scoped collection without an authoritative scope**, not “force add/remove symmetry.”

## Exact accepted-source anchors

The following accepted upstream blobs were read at the locked commit:

- `Telegram/SourceFiles/history/history_item.cpp@b753d3ea7bd5fde15cdad206d17ab3eee6d68cab`: `topicRootId()` falls back to `ForumTopic::kGeneralId`; `setIsPinned(true)` always adds the whole-peer pinned key, adds the topic key only when `_history->asForum()`, and keeps the Saved-sublist key separate; unpin delegates to find-only removal.
- `Telegram/SourceFiles/storage/storage_shared_media.cpp@b52ec6c64d3c8d05e211ae4abe92b5dc5c19d642`: `SharedMediaAddExisting` / `SharedMediaAddSlice` call `enforceLists`, while `SharedMediaAddNew` and `SharedMediaRemoveOne` only find optional topic/sublist keys.
- `Telegram/SourceFiles/history/history.cpp@27c0d3c3f9f8668a497c5ab638135972380bfd6b`: ordinary history population creates the whole-peer list; real topic state is added only through an actual `item->topic()`, and sublists use their independent sublist peer key.
- `Telegram/SourceFiles/data/data_forum_topic.h@383fa26754334435fecbf4bca3b0003034189593`: `ForumTopic::kGeneralId = 1`.

The task directory contains only `tasks/2026/08/20/stop-orphaning-general-id-shared-media-lists.md`; its referenced rationale/measurements/follow-ups are not immutable accepted-tree evidence and are not promoted to Fabushi acceptance evidence.

## Existing-owner-first audit

The parent Fabushi tree is non-truncated. Existing source-neutral owners already cover the durable message/resource parts of the responsibility:

- `native/mahayana-messaging/src/message.rs@43befb0734a6d1d6d17aeb11f6aa5608a16a6f5a` owns canonical `Message`, `MediaRef`, durable `pinned`, and separate reply/thread identities.
- `source/host/src/extensions/attachments/attachments_service.rs@783a78e21ad4c87db1930cadf3ba46dd6bbb806e` owns attachment persistence/read boundaries, safe owner-directory resolution, agent scoping, box staging and media access.
- `source/electron-main/attachments/attachments.ts@764654b8f97b94215a015168c6ba82ae5e80aa45` owns the Electron attachment edge, bounded byte reads, staging/commit, preview resolution and download behavior.
- `frontend/src/recovered/features/conversation/cards/transcript-card/attachment-data.ts@52c8606066fe778b297759434c0f8a28d95fc696` owns typed transcript attachment projection.
- Canonical Transcript/Conversation owners from strict129 remain the only message collection/navigation truth.

These owners do **not** prove a thread-scoped shared-media index. The shipping `frontend/src/production/ProductionRenderer.tsx@af8a83a8dd7cf1444474889ce3e64fe796a7d458` still has `threadRootId: null` and `getThreadSummary: () => null`, and no exact-head production entrypoint was found that creates a pinned/media collection keyed by canonical conversation + explicit thread scope.

Therefore the existing owners must be extended when that capability is implemented; a TelegramSharedMediaStore, second message store, source-named index, or integer-sentinel thread namespace is forbidden.

## Applicability and fail-closed contract

UI applicability is source-neutral: a future pinned/media collection or filter belongs inside the canonical ConversationWorkspace/Transcript capability surface. Telegram's pinned bar, Info page, Qt widgets and thirteen `SparseIdsList` implementation are not copied.

The non-UI responsibility is applicable now and is stronger than the upstream data structure:

- whole-conversation collection scope and explicit thread scope must be typed separately;
- a General thread identity is valid only when the canonical conversation actually carries that thread capability;
- missing/unloaded presentation objects cannot erase a valid durable thread identity;
- a fallback/sentinel numeric value must never allocate a durable collection by itself;
- optional scoped collection removal is idempotent/no-op when the collection never existed;
- resource bytes remain owned by the canonical Resource/Attachment services while message/resource membership is owned by canonical Message/Transcript composition;
- stale account/conversation/thread generations must not populate another scope.

Because the parent exact head has durable `pinned` and resource owners but no shipping thread-scoped pinned/shared-media collection, strict130 remains **mapped-open**. It does not claim message-pin UI, collection persistence, thread-scoped media E2E, or release acceptance.

## Traceability anchors

Requirement / oracle / invariant IDs for this mapped-open responsibility:
- `TDRP-R9-SHARED-MEDIA-GENERAL-ID-LIFETIME-001`
- `ORA-TDRP-SHARED-MEDIA-GENERAL-ID-EXACT-TASK-SOURCE-001`
- `ORA-TDRP-SHARED-MEDIA-GENERAL-ID-TOPIC-ROOT-FALLBACK-001`
- `ORA-TDRP-SHARED-MEDIA-GENERAL-ID-ENFORCE-VS-FIND-001`
- `ORA-TDRP-SHARED-MEDIA-GENERAL-ID-AS-FORUM-DISCRIMINATOR-001`
- `ORA-TDRP-SHARED-MEDIA-GENERAL-ID-CANONICAL-MESSAGE-RESOURCE-OWNERS-001`
- `ORA-TDRP-SHARED-MEDIA-GENERAL-ID-MISSING-SHIPPING-COLLECTION-001`
- `INV-TDRP-SHARED-MEDIA-GENERAL-ID-NO-SENTINEL-THREAD-SCOPE-001`
- `INV-TDRP-SHARED-MEDIA-GENERAL-ID-NO-TOPIC-LOAD-RACE-GATE-001`
- `INV-TDRP-SHARED-MEDIA-GENERAL-ID-NO-SECOND-SHARED-MEDIA-STORE-001`
- `INV-TDRP-SHARED-MEDIA-GENERAL-ID-NO-ORPHAN-COLLECTION-001`
- `INV-TDRP-SHARED-MEDIA-GENERAL-ID-NO-CROSS-SCOPE-MEDIA-001`
- `INV-TDRP-SHARED-MEDIA-GENERAL-ID-EXACT-HEAD-ACTIONS-ONLY-001`

## Parent exact-head evidence

Parent `46ca9cb537310f5be0c9e131360b8c5ee00497de` completed:
- TDRP `39980505627`: **61/61 success**
  - authority `12514140088` `sha256:188355fe7ceb02ed5aa046da4b2be44dd1ac3df4ef13a0c753bf1b2df7ef6b79`
  - closure `12512474733` `sha256:c9aabe9893b50132e6700cf9c93b7deab4c92938f89ff74e78b25a71cd09128b`
  - reachability `12512983501` `sha256:65422f858cb3fb172b8db04826bad58db39dcc8b7165ccf1589d54a9b151345d`
- Rust desktop runtime `39980505846`: **6/6 success**
  - artifact `12513905803` `sha256:5f641229b759119053ce7528ef6030310c4bd8f91b2ee5cb2cfb127a6d1cabd7`

No parent exact-head job failed, so no production/CI repair was justified for strict130.

## Disposition

Requirement: `TDRP-R9-SHARED-MEDIA-GENERAL-ID-LIFETIME-001`
Disposition: `shared-media-general-id-lifetime-mapped-open`

Counters after strict130: unknown=16,246; unread=16,221; omitted=0; responsibilities=148; unmapped_responsibilities=0.

Next deterministic root entry: `tasks/2026/08/21/preserve-info-thread-identity-and-lifetime.md@2ae8fa2751b7a1572d72b5a416b6c3cca3ba0e6b` (strict131).

Global source closure remains open; `baseline_ready=false`, `accepted=false`, and independent release acceptance is not complete.
