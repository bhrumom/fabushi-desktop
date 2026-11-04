# Folders & Filters — source completeness evidence

Status: read-complete / implemented / verified  
Project: TDRP-001 Revision 9  
Accepted upstream: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Rebaseline inheritance: source identity cited below represents Telegram Desktop chat folders and dialog filtering (`data/data_folder.cpp`). Identity verified against recursive upstream inventory artifact 13118749668.

## Exact source identities

| path | blob | read status | classification | mapping status |
| --- | --- | --- | --- | --- |
| `Telegram/SourceFiles/data/data_folder.cpp` | `46897b690847d98c41530b520e01a10b90cfa5e9` | complete read | Chat folders, tab filters, and inclusion/exclusion rules | verified |

## Transferable responsibilities

The accepted upstream carries folder and filter lifecycle management:
- Create, update, list, and delete custom chat folders with inclusion and exclusion rules (channels, groups, bots, unread, muted).
- Match dialog items against active folder criteria deterministically.
- Coordinator vertical slice implemented in `source/node-agent-coordinator/src/tdrp/folders_filters.rs`.
- Host session integration in `source/host/src/extensions/session/tdrp_dialog_state.rs` and messaging engine in `native/mahayana-messaging/src/engine_tdrp_dialogs.rs`.

## Machine-readable responsibility traceability

- `responsibility_id`: `TDRP-R9-FOLDERS-FILTERS-001`
- `oracle_id`: `ORA-TDRP-FOLDERS-FILTERS-001`
- `invariants`:
  - `INV-TDRP-FOLDERS-FILTERS-DETERMINISTIC-MATCHING-001`: Filter matching is deterministic and actor-isolated.
- `contract_cases`: `CONTRACT-TDRP-FOLDERS-FILTERS-001`
