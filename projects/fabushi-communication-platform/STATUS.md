# FBCP Native Communication Absorption — Status

Date: 2026-10-11  
Spec: FBCP-001 Revision 7  
Complete: false

| Area | Status | Note |
| --- | --- | --- |
| Existing architecture as sole target | specified | PR #20/canonical Fabushi is the skeleton |
| Telegram as research source only | specified | no Telegram Provider/network dependency |
| Existing-owner-first absorption | specified | new owners require ADR |
| Native Fabushi network | in progress | canonical Rust messaging service and real Host HTTP API transport exist; broader service/release closure remains open |
| Exact existing-owner inventory | in progress | canonical owners are audited continuously; full 16,125-entry absorption is not closed |
| Telegram capability graph | in progress | deterministic source review is 10,860/16,125; unread/unknown remain open |
| Capability → owner mapping | in progress | reviewed responsibilities map existing-owner-first; full source→shipping evidence graph remains open |
| Conversation/message model evolution | in progress | native/mahayana-messaging has canonical Conversation/protocol/service evolution; mapped-open product closure remains |
| Native messaging infrastructure | in progress | canonical messaging service plus source/host real API transport are implemented; broader server/platform/release evidence remains |
| Human messaging in existing workspace | in progress | existing Composer/submission/Host path carries human send including silent/scheduled semantics; full regression/release closure remains |
| Human + Agent unified flow | in progress | canonical ConversationWorkspace/Composer infrastructure is shared; complete Hybrid creation/policy/regression evidence remains open |
| Full feature absorption | in progress | source/product mapping is active; unread=5,265 and unknown=15,845 prevent closure |
| Packaged acceptance | blocked | no artifact |
| Release | blocked | ACs incomplete |

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


## 2026-10-11 live implementation correction

The previous "not implemented" rows for native messaging and Human messaging were stale. Current shipping code includes `native/mahayana-messaging` canonical Conversation/protocol/service owners and `source/host/src/extensions/session/native_messaging.rs` real Fabushi API transport. `frontend/src/production/human-silent-scheduled-message.contract.test.ts` exercises the existing Composer → submission queue → Host call path. This corrects status to **in progress**, not complete: full source absorption, service-wide E2E, packaged acceptance and release evidence remain open.
