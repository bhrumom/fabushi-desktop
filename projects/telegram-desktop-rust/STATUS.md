# Telegram Capability Research — Status

Date: 2026-10-02
Spec: TDRP-001 Revision 4
Parent: FBCP-001
Complete: false

| Area | Status | Note |
| --- | --- | --- |
| Telegram role as research source | specified | no runtime provider |
| Fixed upstream baseline | recorded | 33261535a0e747f125e0ed25486f01e556330677 |
| Recursive source closure | partial | root + 35 direct gitlinks + all nested gitlinks including `cppgir`→`expected-lite` verified; tracked generator/packaging roots and external-acquisition entrypoints mapped; immutable external resolution and resource/license closure remain |
| C++ responsibility inventory | recorded | all 40 frozen `SourceFiles` top-level areas classified by product/state, protocol/security, UI, platform, persistence/media, tooling/test responsibility; third-party/build roots separately classified; implementation remains open |
| Full capability graph | recorded / behavior-closure open | current graph has 50 capability/aggregate rows; reverse frozen-tree audit covers 40/40 SourceFiles top-level areas and 130/130 paths through relative depth two; deeper behavior exhaustiveness still requires source-to-behavior closure |
| Current Fabushi owner inventory | recorded | refreshed against exact PR #20 `dcb19a94383833fc1ec5074f10c4bbbd28c09036`; must refresh if PR #20 moves |
| Capability → existing owner mapping | recorded | all 50 current graph/matrix rows have nonempty selected owner resolution; conditional PaymentSettlement and CallSession exceptions are ADR-backed |
| Absorption plans | recorded | all 50 matrix rows contain source behavior, owner candidates, selected owner, exact evidence, absorption/model/UI, persistence/native-network and focused-test/blocker fields |
| Minimal new-owner proposals | recorded / conditional | ADR-002 PaymentSettlement and ADR-003 CallSession/Signaling are minimal conditional infrastructure owners; no broad Communication Core exists |
| Native network implications | recorded / implementation open | every matrix row records persistence/native-network implications; identity/messaging/sync/presence/media/call/push remain infrastructure beneath existing product owners, with production transport/sync still incomplete |
| Implemented absorbed behavior | partial / unaccepted | first Human private-message local durable slice is production-wired on the FBCP branch; native transport/sync and exact-head packaged acceptance remain open |
| Packaged acceptance | blocked | no evidence |

## Metrics

Report separately:

- research coverage
- owner resolution coverage
- absorption-plan coverage
- C++ responsibility implementation coverage
- exact-head behavior verification
- packaged product acceptance

Do not report Telegram-provider or Telegram-network progress.
