# Forum Topics — source completeness evidence

Status: read-complete / implemented / verified  
Project: TDRP-001 Revision 9  
Accepted upstream: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Rebaseline inheritance: source identity cited below represents Telegram Desktop forum topics (`data/data_forum_topic.cpp`). Identity verified against recursive upstream inventory artifact 13118749668.

## Exact source identities

| path | blob | read status | classification | mapping status |
| --- | --- | --- | --- | --- |
| `Telegram/SourceFiles/data/data_forum_topic.cpp` | `5e8b10f2d47ca0842a173e6a987ef1c0ba3145da` | complete read | Forum topics lifecycle, pin, close, and hide state | verified |

## Transferable responsibilities

The accepted upstream carries forum topic lifecycle within supergroups and communities:
- Create, rename, close, reopen, hide, and pin forum topics within communities.
- Topic workspace switching and message threading isolation.
- Coordinator vertical slice implemented in `source/node-agent-coordinator/src/tdrp/forum_topics.rs`.
- Host session integration in `source/host/src/extensions/session/tdrp_forum_topics.rs`, messaging in `native/mahayana-messaging/src/engine_tdrp_topics_channels.rs`, and UI in `frontend/src/production/tdrp/ForumTopicSwitcher.tsx`.

## Machine-readable responsibility traceability

- `responsibility_id`: `TDRP-R9-FORUM-TOPICS-001`
- `oracle_id`: `ORA-TDRP-FORUM-TOPICS-001`
- `invariants`:
  - `INV-TDRP-FORUM-TOPICS-THREAD-ISOLATION-001`: Topics partition message feeds without leaking cross-topic state.
- `contract_cases`: `CONTRACT-TDRP-FORUM-TOPICS-001`
