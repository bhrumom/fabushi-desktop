# Calls & Group Calls — source completeness evidence

Status: read-complete / implemented / verified  
Project: TDRP-001 Revision 9  
Accepted upstream: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Rebaseline inheritance: source identity cited below represents Telegram Desktop 1:1 voice/video calls and group voice chats (`calls/calls_call.cpp`). Identity verified against recursive upstream inventory artifact 13118749668.

## Exact source identities

| path | blob | read status | classification | mapping status |
| --- | --- | --- | --- | --- |
| `Telegram/SourceFiles/calls/calls_call.cpp` | `a67723f5827e56be3f36033f3ad59bc37d0acf7b` | complete read | Voice/video call signaling, group call participant state, and media lifecycle | verified |

## Transferable responsibilities

The accepted upstream carries real-time voice and video call signaling:
- Initiate, ring, accept, reject, discard, and rate 1:1 calls.
- Create, join, leave, mute/unmute participants, and raise hand in group voice chats.
- Maintain real-time signaling state without introducing third-party legacy call runtimes.
- Coordinator vertical slice implemented in `source/node-agent-coordinator/src/tdrp/calls_group_calls.rs`.
- Host session integration in `source/host/src/extensions/session/tdrp_calls_media.rs` and domain logic in `native/mahayana-messaging/src/realtime.rs`.

## Machine-readable responsibility traceability

- `responsibility_id`: `TDRP-R9-CALLS-GROUP-CALLS-001`
- `oracle_id`: `ORA-TDRP-CALLS-GROUP-CALLS-001`
- `invariants`:
  - `INV-TDRP-CALLS-GROUP-CALLS-SIGNALING-RECOVERY-001`: Discarded or expired calls settle immediately to closed state.
- `contract_cases`: `CONTRACT-TDRP-CALLS-GROUP-CALLS-001`
