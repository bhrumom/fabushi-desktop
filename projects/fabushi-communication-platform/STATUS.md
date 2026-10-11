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
| Telegram capability graph | in progress | deterministic source review is 11,244/16,125; unread/unknown remain open |
| Capability → owner mapping | in progress | reviewed responsibilities map existing-owner-first; full source→shipping evidence graph remains open |
| Conversation/message model evolution | in progress | native/mahayana-messaging has canonical Conversation/protocol/service evolution; mapped-open product closure remains |
| Native messaging infrastructure | in progress | canonical messaging service plus source/host real API transport are implemented; broader server/platform/release evidence remains |
| Human messaging in existing workspace | in progress | existing Composer/submission/Host path carries human send including silent/scheduled semantics; full regression/release closure remains |
| Human + Agent unified flow | in progress | canonical ConversationWorkspace/Composer infrastructure is shared; complete Hybrid creation/policy/regression evidence remains open |
| Full feature absorption | in progress | source/product mapping is active; unread=4,881 and unknown=15,845 prevent closure |
| Packaged acceptance | blocked | no artifact |
| Release | blocked | ACs incomplete |

## 2026-10-11 range-v3 views/build provenance/action source accounting

Exact-head `af7183eb431ee41730ba0631f2306c186478890d` Rust desktop runtime `38151143307` completed/success; Host Runner, prompt attachment projection, Human Computer takeover and ConversationActor/CapabilityBroker contracts all executed successfully. Source-authority job `114504686124` materialized and exact-blob verified orders 11,181–11,244, artifact `11706761066` (`sha256:09b595119048168b7fb4f857d8ce3e642e25c46163de2ff93a2eea6a778c9d32`). Current deterministic accounting is **11,244/16,125 read; 4,881 unread; 15,845 unknown; 0 omitted**; first unread is 11,245 `Telegram/ThirdParty/range-v3::test/action/slice.cpp@ff01398e6d72622e3c105dcf3bc9a23380f8c7bc`.

The window remains generic range-v3 view/compiler-compatibility/benchmark/action responsibility. `install_libcxx.sh` additionally exposes mutable tip-of-trunk LLVM/libc++/libc++abi clones and unverified `curl -LO` downloads; this is recorded under existing Build / Quality / Dependency-Provenance / License authority and shipping remains fail-closed. Product/release acceptance remains open.

## 2026-10-11 range-v3 utility/view source accounting

Exact-head `e2f2ca67fad222cf5631b858a13719a7cbad29b3` Rust desktop runtime `38143537190` completed/success; source-authority job `114482083927` materialized and exact-blob verified orders 11,117–11,180, artifact `11704176427` (`sha256:aacf80e5efad0eda199b16cd5aa8d3112f69aca2d36d4ed77dc3324b134390d1`). Current deterministic accounting is **11,180/16,125 read; 4,945 unread; 15,845 unknown; 0 omitted**; first unread is 11,181 `Telegram/ThirdParty/range-v3::include/range/v3/view/interface.hpp@aee727d5ee7d9133bf6e269c0af2143cd09a9c41`. This is source accounting only; product/release acceptance remains open.

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
