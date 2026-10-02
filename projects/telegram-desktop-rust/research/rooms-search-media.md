# Telegram rooms, search, and media-transfer research dossier

Status: researched and owner-resolved for this capability cluster; production implementation/acceptance remains open
Frozen upstream: `telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677`
Canonical architecture snapshot: PR #20 `010ef77aa2502b6301b92b542255cff7a5285f81`

Snapshot refresh note: the exact `dcb19a94383833fc1ec5074f10c4bbbd28c09036 → 010ef77aa2502b6301b92b542255cff7a5285f81` PR #20 delta was inspected. It restores generated SendMessage shaping/threading ownership through the shipping transcript pipeline (`main.rs`, `production_runtime.rs`, `send_pipeline.rs` and its production-composition contract). It does not introduce a competing room/search/media owner, so the selected existing owners below remain valid at `010ef77a...`.

## Groups / members / admin
Frozen `Telegram/SourceFiles/data/data_chat.cpp:52-108` makes permissions explicit capabilities rather than UI guesses. Around 191-200, member state is versioned: an old update is ignored and a skipped version invalidates participants and requests authoritative refresh. Around 326-480, join/leave/admin/rank/default-right updates mutate the same chat/member state and publish member/rights changes.

Plausible existing owners: Shared Room, Human/Agent member model, permissions/room controller, conversation workspace, or a new room/membership core.

`existing_owner = existing Shared Room + Human/Agent member model + permissions/room controller`.

Selected reason: membership and role state already belongs to the canonical room/member/permission surfaces; creating another room truth would violate the absorption law.

Fabushi absorption: extend existing Shared Rooms, Human/Agent member model and permissions. `source/host/src/extensions/transcript/shared_rooms.rs` already keeps room materialization and transcript writes under Session/Transcript ownership; `group_chat_glue.rs` already owns group creation/member validation. Native sync must carry versioned room membership/role events and trigger bounded authoritative roster recovery on a gap.

## Channels / broadcast
Frozen `data_channel.cpp:74-114` attaches forum/monoforum behavior to the same channel/history owner; `data_channel.cpp:180+` evolves flags such as megagroup/forum/community instead of creating another navigation shell.

Plausible existing owners: Shared Room, conversation/sidebar model, permissions/room controller, transcript, or a new Channel subsystem.

`existing_owner = existing Shared Room/conversation model + permissions/room controller`.

Selected reason: broadcast is a room permission/mode over the existing conversation identity, not a second navigation or message-store domain.

Fabushi absorption: channel is an evolved existing room/conversation with broadcast/post permission policy. It uses the same sidebar, workspace, transcript and member/permission owners. No Channel subsystem or alternate message store is permitted.

## Topics / forums / threads
Frozen `data_channel.cpp` binds Forum to the existing history, while `data_forum_topic.cpp` maintains topic-scoped state under that forum/history relationship.

Plausible existing owners: conversation workspace, transcript relation/thread model, Session transcript pagination, Shared Room, or a new Topic subsystem.

`existing_owner = existing transcript relation/thread model + Session bounded transcript pagination`.

Selected reason: topic identity scopes history/thread projection under one room/conversation; it does not require another workspace or canonical history owner.

Fabushi absorption: extend the existing transcript relation/thread model and bounded transcript windows. `source/host/src/extensions/session/agent_db_transcript_pages.rs` already returns bounded transcript windows and thread counts; native sync must add room/topic sequence and gap semantics without creating a second topic workspace.

## Search
Frozen `Telegram/SourceFiles/api/api_messages_search.cpp` performs bounded, history-aware message search/pagination rather than materializing a second complete history.

Plausible existing owners: existing Content Search, transcript pagination, sidebar search affordance, Host session storage, or a new search store.

`existing_owner = existing Content Search + transcript pagination`.

Selected reason: search is a projection/query over canonical transcript state. A separate canonical message store is explicitly rejected.

Fabushi absorption: use the existing Content Search owner plus transcript pagination. Edits/deletes/reactions that affect searchable projection must update the same search index. Remote/native sync may fill a missing bounded range, but search must not become a separate canonical message store.

## Files / upload / download
Frozen `storage/file_upload.cpp` models transfer as part/chunk progress with bounded concurrent sessions, cancellation and explicit failure/progress settlement. Frozen `storage/file_download.cpp:105-163` separates loaded bytes/local file settlement; `220-260` can resume from partial local state; `343-414` cancels cleanly and writes at explicit offsets.

Plausible existing owners: attachments/artifacts/resource lifecycle, transcript attachment projection, Host storage, or a new media-transfer product owner.

`existing_owner = existing attachments/artifacts/resource lifecycle`, with a minimal native blob-transfer infrastructure layer below it for transport mechanics.

Selected reason: identity, safety limits and durable resource state already belong to the attachment/resource owner; resumable network transfer is infrastructure, not a parallel product model.

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
