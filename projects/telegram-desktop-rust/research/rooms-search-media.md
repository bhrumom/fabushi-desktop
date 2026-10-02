# Telegram rooms, search, and media-transfer research dossier

Status: researched and owner-resolved for this capability cluster; production implementation/acceptance remains open
Frozen upstream: `telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677`
Canonical architecture snapshot: PR #20 `de0f1749a675729fb97017085374b96aaa2ca7bb`

Snapshot refresh note: the exact `556f6308... → c2767eac...` PR #20 delta was inspected; it only moves Agent outline-stream identity/coalescing into shipping `RosterProjection`/`ProductionRosterEmit`, so the owner paths selected in this dossier remain valid at c276.

## Groups / members / admin
Frozen `Telegram/SourceFiles/data/data_chat.cpp:52-108` makes permissions explicit capabilities rather than UI guesses. Around 191-200, member state is versioned: an old update is ignored and a skipped version invalidates participants and requests authoritative refresh. Around 326-480, join/leave/admin/rank/default-right updates mutate the same chat/member state and publish member/rights changes.

Fabushi absorption: extend existing Shared Rooms, Human/Agent member model and permissions. `source/host/src/extensions/transcript/shared_rooms.rs` already keeps room materialization and transcript writes under Session/Transcript ownership; `group_chat_glue.rs` already owns group creation/member validation. Native sync must carry versioned room membership/role events and trigger bounded authoritative roster recovery on a gap.

## Channels / broadcast
Frozen `data_channel.cpp:74-114` attaches forum/monoforum behavior to the same channel/history owner; `data_channel.cpp:180+` evolves flags such as megagroup/forum/community instead of creating another navigation shell.

Fabushi absorption: channel is an evolved existing room/conversation with broadcast/post permission policy. It uses the same sidebar, workspace, transcript and member/permission owners. No Channel subsystem or alternate message store is permitted.

## Topics / forums / threads
Frozen `data_channel.cpp` binds Forum to the existing history, while `data_forum_topic.cpp` maintains topic-scoped state under that forum/history relationship.

Fabushi absorption: extend the existing transcript relation/thread model and bounded transcript windows. `source/host/src/extensions/session/agent_db_transcript_pages.rs` already returns bounded transcript windows and thread counts; native sync must add room/topic sequence and gap semantics without creating a second topic workspace.

## Search
Frozen `Telegram/SourceFiles/api/api_messages_search.cpp` performs bounded, history-aware message search/pagination rather than materializing a second complete history.

Fabushi absorption: use the existing Content Search owner plus transcript pagination. Edits/deletes/reactions that affect searchable projection must update the same search index. Remote/native sync may fill a missing bounded range, but search must not become a separate canonical message store.

## Files / upload / download
Frozen `storage/file_upload.cpp` models transfer as part/chunk progress with bounded concurrent sessions, cancellation and explicit failure/progress settlement. Frozen `storage/file_download.cpp:105-163` separates loaded bytes/local file settlement; `220-260` can resume from partial local state; `343-414` cancels cleanly and writes at explicit offsets.

Fabushi already has a strong product owner: `source/host/src/extensions/attachments/attachments_service.rs` performs content-addressed ingestion by SHA-256, enforces resource limits and safe paths, and supports bounded chunk reads. The missing responsibility is minimal Fabushi-native blob transfer below this owner: resumable offsets/chunk acknowledgements, content-hash verification, retry/backpressure/cancel and crash recovery.

## Required contracts
- skipped member-version update invalidates the roster projection and requests bounded authoritative recovery;
- duplicate join/leave/role events are idempotent and permissions fail closed;
- Human + Agent members coexist in one Shared Room and one transcript;
- broadcast policy cannot be bypassed by a stale client;
- topic history preserves room identity while maintaining topic/thread pagination and drafts;
- search sees Human/Agent messages from canonical transcript state and converges after edit/delete;
- upload/download resumes after restart without a second resource identity;
- duplicate content hash reuses safe resource identity where policy permits;
- hash/size mismatch, disk failure and cancellation settle explicitly;
- transfer progress is ephemeral UI state; durable attachment/resource state remains under existing attachment/artifact ownership.

Research does not mean implementation. Native room synchronization, remote search fill, blob service/network transfer and packaged acceptance remain open.
