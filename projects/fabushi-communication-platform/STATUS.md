# FBCP Native Communication Absorption — Status

Date: 2026-10-02  
Spec: FBCP-001 Revision 2  
Complete: false

| Area | Status | Note |
| --- | --- | --- |
| Existing architecture as sole target | specified | PR #20/canonical Fabushi is the skeleton |
| Telegram as research source only | specified | no Telegram Provider/network dependency |
| Existing-owner-first absorption | specified | new owners require ADR |
| Native Fabushi network | specified | protocol/services not yet designed |
| Exact existing-owner inventory | recorded | evidence snapshot at PR #20 `f9546f9a77e22d8ce7dabf978eeeffa079b22915`; refresh on head change |
| Telegram capability graph | in-progress | top-level discovery pass recorded; recursive source closure and behavior research incomplete |
| Capability → owner mapping | blocked | full per-capability resolution still pending |
| Conversation/message model evolution | blocked | P0 pending |
| Native messaging infrastructure | blocked | not implemented |
| Human messaging in existing workspace | blocked | not implemented |
| Human + Agent unified flow | blocked | not implemented |
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
