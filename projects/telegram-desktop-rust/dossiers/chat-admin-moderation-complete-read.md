# Chat Admin & Moderation — source completeness evidence

Status: read-complete / implemented / verified  
Project: TDRP-001 Revision 9  
Accepted upstream: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Rebaseline inheritance: source identity cited below represents Telegram Desktop group/channel permissions and moderation (`data/data_chat_participant_status.cpp`). Identity verified against recursive upstream inventory artifact 13118749668.

## Exact source identities

| path | blob | read status | classification | mapping status |
| --- | --- | --- | --- | --- |
| `Telegram/SourceFiles/data/data_chat_participant_status.cpp` | `8e3053faa947ca0842a173e6a987ef1c0ba3145c` | complete read | Chat participant rights, admin privileges, bans, and slowmode | verified |

## Transferable responsibilities

The accepted upstream carries participant rights, administrator promotion, and moderation enforcement:
- Assign granular admin rights (change info, delete messages, ban users, pin messages, manage topics).
- Apply temporary or permanent participant restrictions and bans with reason codes.
- Enforce slowmode cooldown timers and composer guard policies.
- Coordinator vertical slice implemented in `source/node-agent-coordinator/src/tdrp/chat_admin_moderation.rs`.
- Host session integration in `source/host/src/extensions/session/tdrp_chat_admin_moderation.rs`, messaging policy in `native/mahayana-messaging/src/engine_tdrp_moderation.rs`, and UI guard in `frontend/src/production/tdrp/CommunityComposerGuardBanner.tsx`.

## Machine-readable responsibility traceability

- `responsibility_id`: `TDRP-R9-CHAT-ADMIN-MODERATION-001`
- `oracle_id`: `ORA-TDRP-CHAT-ADMIN-MODERATION-001`
- `invariants`:
  - `INV-TDRP-CHAT-ADMIN-MODERATION-FAIL-CLOSED-001`: Restricted participants are fail-closed prevented from posting restricted message kinds.
- `contract_cases`: `CONTRACT-TDRP-CHAT-ADMIN-MODERATION-001`
