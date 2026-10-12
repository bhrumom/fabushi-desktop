# Story sibling source-completeness dossier

Authority: TDRP-001 Revision 9  
Accepted upstream: `telegramdesktop/tdesktop@72b3b71c3d6e450e5ef94a3112dd750a0168aa0b`  
Status: read-complete / responsibility mapping open

## Exact source entries

| path | blob | read status | classification | mapping status |
| --- | --- | --- | --- | --- |
| `Telegram/SourceFiles/media/stories/media_stories_sibling.h` | `24f334cae788ee6a1dc0b07cd313e81b77ca1ae2` | complete | story-adjacent preview lifecycle/loader/view contract | open |
| `Telegram/SourceFiles/media/stories/media_stories_sibling.cpp` | `a9d875c9e44a94909bb84c4aeb6e70c7f47403d3` | complete | source selection, async story/media resolution, thumbnail/stream fallback and cached projection | open |

Both files were read in full against the accepted commit. No truncated rendering is credited as complete.

## Responsibilities recovered

- **Source identity / stale suggestion fencing.** `LookupShownId` uses a suggested Story id only when it is still present in the current `StoriesSource`; otherwise it falls back to the source's current open id. A stale suggestion must not silently bind to a foreign story.
- **Story-resolution lifecycle.** Missing/unknown story data resolves through the existing Story owner and a weak lifetime guard. Destruction must prevent delayed resolution callbacks from mutating a dead surface; unresolved/unsupported media falls back safely instead of inventing content.
- **Photo resource lifecycle.** Inline thumbnail is the immediate low-fidelity fallback, large media is requested through the existing downloader/cache lifecycle, and completion invalidates only the derived preview.
- **Video thumbnail / streaming fallback.** Good-thumbnail cache state is checked before creating a streaming instance; generation/download callbacks are fenced, the fallback player is locked and paused, and a streaming failure settles to a terminal fallback rather than spinning a retry loop.
- **Derived projection cache.** Userpic and display-name images are invalidated by stable participant/userpic identity, font/available-width, and device-pixel-ratio inputs. These are projection caches, not a second Participant or Story truth.
- **Pure presentation delta.** Hover fade/scale, image radius, text opacity and painter details are presentation behavior to adapt through the live Fabushi design system; they do not justify porting Qt drawing code.

## Existing-owner-first mapping

The canonical Rust `native/mahayana-messaging/src/story.rs` already owns Story identity, visibility, expiry, media reference and viewer state. Media bytes/cache/streaming must stay in the existing Fabushi Resource/attachment/media owners, and peer name/avatar projection stays in canonical Participant/Profile owners. Any story viewer/card UI must consume those owners through the live Fabushi component/design-system contracts. No `TelegramStorySibling`, second Story store, second media cache, or retained C++ runtime is justified.

The current Fabushi Story model does not yet prove the full sibling-source ordering/resolution/thumbnail fallback composition in shipping UI, so these responsibilities are **understood/open**, not implemented or verified. Future implementation must retain stale-result fencing, bounded async lifetimes, terminal media failure behavior, and one canonical Story/Resource truth.

Coverage after these two additional complete reads: `unread=15,761`, `unknown=15,788`, `omitted=0`, `baseline_ready=false`, `acceptance.accepted=false`.
