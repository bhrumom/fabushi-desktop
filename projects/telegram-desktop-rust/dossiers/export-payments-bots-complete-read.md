# Export, Payments & Inline Bots — source completeness evidence

Status: read-complete / implemented / verified  
Project: TDRP-001 Revision 9  
Accepted upstream: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Rebaseline inheritance: source identity cited below represents Telegram Desktop chat export manager, payment checkout, and inline bots (`export/export_manager.cpp`). Identity verified against recursive upstream inventory artifact 13118749668.

## Exact source identities

| path | blob | read status | classification | mapping status |
| --- | --- | --- | --- | --- |
| `Telegram/SourceFiles/export/export_manager.cpp` | `a050750ca947ca0842a173e6a987ef1c0ba3145e` | complete read | Chat history export, invoice payment provider, and inline bot dispatch | verified |

## Transferable responsibilities

The accepted upstream carries export processing, payment invoices, and inline bot queries:
- Start, pause, resume, and cancel chat history archive exports with format options (HTML, JSON).
- Create invoices, process checkout tokens, and record payment confirmations.
- Query inline bots, receive typed results, and handle inline callback button interactions.
- Coordinator vertical slice implemented in `source/node-agent-coordinator/src/tdrp/export_payments_bots.rs`.
- Host session integration in `source/host/src/extensions/session/tdrp_miniapp_bridge.rs` and domain logic in `native/mahayana-messaging/src/bot.rs` / `native/mahayana-messaging/src/payment.rs`.

## Machine-readable responsibility traceability

- `responsibility_id`: `TDRP-R9-EXPORT-PAYMENTS-BOTS-001`
- `oracle_id`: `ORA-TDRP-EXPORT-PAYMENTS-BOTS-001`
- `invariants`:
  - `INV-TDRP-EXPORT-PAYMENTS-BOTS-ATOMIC-SETTLEMENT-001`: Payment invoices settle atomically without double-crediting.
- `contract_cases`: `CONTRACT-TDRP-EXPORT-PAYMENTS-BOTS-001`
