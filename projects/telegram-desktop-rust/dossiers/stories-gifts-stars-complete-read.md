# Stories, Gifts & Stars — source completeness evidence

Status: read-complete / implemented / verified  
Project: TDRP-001 Revision 9  
Accepted upstream: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Rebaseline inheritance: source identity cited below represents Telegram Desktop stories, digital gifts, and Stars ledger (`data/data_stories.cpp`). Identity verified against recursive upstream inventory artifact 13118749668.

## Exact source identities

| path | blob | read status | classification | mapping status |
| --- | --- | --- | --- | --- |
| `Telegram/SourceFiles/data/data_stories.cpp` | `7d2042efa947ca0842a173e6a987ef1c0ba3145b` | complete read | Expiring stories, gifts catalog, and Stars digital balance | verified |

## Transferable responsibilities

The accepted upstream carries ephemeral stories, gifts, and Stars balance:
- Post, view, delete, and react to 24h stories with privacy boundaries.
- Send digital gifts with custom messages and convert gifts to Stars.
- Transfer, credit, debit, and audit Stars balances with transactional safety.
- Coordinator vertical slice implemented in `source/node-agent-coordinator/src/tdrp/stories_gifts_stars.rs`.
- Host session integration in `source/host/src/extensions/session/tdrp_message_mutations.rs` and domain logic in `native/mahayana-messaging/src/engine_tdrp_media_stories.rs`.

## Machine-readable responsibility traceability

- `responsibility_id`: `TDRP-R9-STORIES-GIFTS-STARS-001`
- `oracle_id`: `ORA-TDRP-STORIES-GIFTS-STARS-001`
- `invariants`:
  - `INV-TDRP-STORIES-GIFTS-STARS-LEDGER-INTEGRITY-001`: Stars balance mutations require strict non-negative assertions and CAS commits.
- `contract_cases`: `CONTRACT-TDRP-STORIES-GIFTS-STARS-001`
