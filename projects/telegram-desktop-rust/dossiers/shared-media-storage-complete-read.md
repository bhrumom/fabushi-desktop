# Shared-media storage complete-read dossier

Authority: `telegramdesktop/tdesktop@d346b42a1d30ef60dc989b6e5191bb8e571f6bd5`.

| source | blob | read |
| --- | --- | --- |
| `Telegram/SourceFiles/storage/storage_shared_media.h` | `cd0d29d594e012d5fabb0e53e148fe98e4e86d44` | complete, 316 lines |
| `Telegram/SourceFiles/storage/storage_shared_media.cpp` | `b52ec6c64d3c8d05e211ae4abe92b5dc5c19d642` | complete, 242 lines |
| `Telegram/SourceFiles/storage/storage_facade.h` | `8d7ddcde8efd4dd82c08670cb1ab303eb9c928d6` | complete, 84 lines |
| `Telegram/SourceFiles/storage/storage_facade.cpp` | `ffb252a7edf89b0e86b4a98e09835c9b2ab19930` | complete, 238 lines |

## TDRP-R9-SHARED-MEDIA-STORAGE-CONTRACT-001

`SharedMediaKey` and related commands make the projection identity explicit: parent peer + topic root + monoforum/SavedSublist discriminator + type/message. The projection is derived product state, not ownership truth. Exact thread unload must invalidate only that typed projection. Fabushi replacement is existing `MessageContent/MediaRef` canonical truth projected by `SearchIndex` Media/Files/Links scopes, with `MediaCache` owning physical materialization independently.

Traceability: `TDRP-R9-SHARED-MEDIA-STORAGE-CONTRACT-001`, `ORA-TDRP-R9-SHARED-MEDIA-STORAGE-CONTRACT-001`, `INV-TDRP-R9-SHARED-MEDIA-STORAGE-CONTRACT-001-CANONICAL`.

Status: mapped/open. SavedSublist exact message membership is still service-blocked and therefore child-owned destructive cleanup cannot be inferred from a parent message or generic relation.

## TDRP-R9-SHARED-MEDIA-STORAGE-LIFECYCLE-001

The cpp establishes projection fan-out and invalidation behavior. Parent lists are always maintained; typed topic/monoforum lists are updated only when materialized. Missing query projections return empty/done. `unload(SharedMediaUnloadThread)` erases exactly one composite key. `unload(SharedMediaUnloadAllTopics)` erases topic-root projections only. This is cache/projection lifecycle, not physical-media deletion.

Fabushi must preserve the product effect through canonical message deletion + `SearchIndex::remove_message`/projection rebuild and must not delete `MediaCache` bytes simply because one child projection disappears.

Traceability: `TDRP-R9-SHARED-MEDIA-STORAGE-LIFECYCLE-001`, `ORA-TDRP-R9-SHARED-MEDIA-STORAGE-LIFECYCLE-001`, `INV-TDRP-R9-SHARED-MEDIA-STORAGE-LIFECYCLE-001-CANONICAL`.

Status: mapped/open. Full SavedMessages child-message destruction/search invalidation remains open.

## TDRP-R9-STORAGE-FACADE-CONTRACT-001

The header exposes one storage delegation boundary and does not define independent state. Fabushi must not copy this into a second `StorageFacade` owner; existing typed engine/search/media owner APIs are the replacement.

Traceability: `TDRP-R9-STORAGE-FACADE-CONTRACT-001`, `ORA-TDRP-R9-STORAGE-FACADE-CONTRACT-001`, `INV-TDRP-R9-STORAGE-FACADE-CONTRACT-001-CANONICAL`.

Status: mapped/open.

## TDRP-R9-STORAGE-FACADE-DELEGATION-001

The cpp is transparent composition: shared-media methods forward exactly to the single `SharedMedia` instance, photo methods to `UserPhotos`, and query/update streams are passed through. There is no hidden second cache or policy layer.

Fabushi replacement is direct composition of canonical `MessagingEngine`, `SearchIndex`, and `MediaCache` owners. The source-shaped facade class is intentionally not reproduced.

Traceability: `TDRP-R9-STORAGE-FACADE-DELEGATION-001`, `ORA-TDRP-R9-STORAGE-FACADE-DELEGATION-001`, `INV-TDRP-R9-STORAGE-FACADE-DELEGATION-001-CANONICAL`.

Status: mapped/open.

## SavedMessages deletion implication

Accepted `data_saved_messages.cpp` calls `SharedMediaUnloadThread(parentPeer, 0, sublistPeer)` after child destruction. Under Fabushi architecture this means: authoritative child-owned message deletion must remove those messages from canonical search/media projections, while resource bytes remain under their normal reference/cache lifecycle. The current child runtime destroy alone is insufficient evidence of this cross-owner composition.

Accounting after these four full reads: unknown `15,788`, unread `15,740`, omitted `0`.
