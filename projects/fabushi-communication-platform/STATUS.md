# FBCP Native Communication Absorption — Status

Date: 2026-10-03
Spec: FBCP-001 Revision 2
Complete: false

| Area | Status | Note |
| --- | --- | --- |
| Existing architecture as sole target | specified | PR #20/canonical Fabushi is the skeleton |
| Telegram as research source only | specified | no Telegram Provider/network dependency |
| Existing-owner-first absorption | specified | new owners require ADR |
| Native Fabushi network | specified | protocol/services not yet designed |
| Exact existing-owner inventory | recorded | refreshed against PR #20 `a538679392bc09531041030a98fa0880107e7d40`; refresh on head change |
| Telegram capability graph | recorded / behavior-closure open | 50 capability/aggregate rows; frozen reverse citation audit covers 40/40 SourceFiles top-level areas and 130/130 paths through relative depth two; deeper behavior exhaustiveness and legal/provenance review remain open |
| Capability → owner mapping | recorded | all 50 current matrix rows have selected owner resolution and complete absorption/persistence/native-network/test fields; ADR-002/003 cover the only conditional minimal infrastructure exceptions |
| Conversation/message model evolution | partial | Human private conversation and Human/Agent transcript projection are production-wired for the first slice; broader capability mapping remains P0-open |
| Native messaging infrastructure | partial | existing Session/Transcript SQLite owner durably accepts Human messages with clientNonce idempotency; native network/sync/reconnect/multi-device transport remains open |
| Human messaging in existing workspace | production-wired / unaccepted | same sidebar/workspace/composer/transcript path is wired; exact-head CI and packaged acceptance remain required |
| Human + Agent unified flow | production-wired / unaccepted | explicit Human -> Agent handoff reuses Coordinator -> Host -> Runner and persists the Agent result into the same Human transcript; verification still open |
| Full feature absorption | blocked | not implemented |
| Packaged acceptance | blocked | no artifact |
| Release | blocked | ACs incomplete |

## P0 evidence

- `projects/fabushi-communication-platform/current-owner-inventory.md`
- `projects/fabushi-communication-platform/telegram-capability-graph.md`
- `projects/telegram-desktop-rust/upstream.lock.json`
- `projects/telegram-desktop-rust/contracts/parity-ledger.schema.json`

The owner inventory is exact-head scoped and must be invalidated when PR #20 moves. The capability graph is a discovery artifact only; it does not satisfy recursive research closure.

## Do not report

- Telegram migration percentage as product progress
- Telegram Provider progress
- a new Communication Core as progress
- file-count parity
- source reading as implementation

## Progress dimensions

1. existing-owner inventory
2. Telegram research coverage
3. absorption mapping
4. model evolution designed
5. native network designed
6. native network implemented
7. existing owners production-wired
8. Human + Agent unified UX
9. full capability acceptance
10. packaged/release acceptance
