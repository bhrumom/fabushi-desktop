# Polls & Todo Checklists — source completeness evidence

Status: read-complete / implemented / verified  
Project: TDRP-001 Revision 9  
Accepted upstream: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Rebaseline inheritance: source identity cited below represents Telegram Desktop polls and quiz checklists (`data/data_poll.cpp`). Identity verified against recursive upstream inventory artifact 13118749668.

## Exact source identities

| path | blob | read status | classification | mapping status |
| --- | --- | --- | --- | --- |
| `Telegram/SourceFiles/data/data_poll.cpp` | `6c1032dfa947ca0842a173e6a987ef1c0ba3145a` | complete read | Poll voting, anonymous polls, quiz mode, and todo checklists | verified |

## Transferable responsibilities

The accepted upstream carries interactive poll and checklist management:
- Create multiple-choice polls, anonymous voting, quizzes with explanation, and collaborative todo items.
- Retract vote, close poll, and aggregate voter distributions.
- Coordinator vertical slice implemented in `source/node-agent-coordinator/src/tdrp/polls_todo.rs`.
- Host session integration in `source/host/src/extensions/session/tdrp_message_mutations.rs` and domain logic in `native/mahayana-messaging/src/poll.rs`.

## Machine-readable responsibility traceability

- `responsibility_id`: `TDRP-R9-POLLS-TODO-001`
- `oracle_id`: `ORA-TDRP-POLLS-TODO-001`
- `invariants`:
  - `INV-TDRP-POLLS-TODO-ANONYMOUS-VOTE-001`: Anonymous voter identity is never revealed in aggregated counts.
- `contract_cases`: `CONTRACT-TDRP-POLLS-TODO-001`
