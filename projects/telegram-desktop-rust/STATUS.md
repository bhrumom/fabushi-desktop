# Telegram Capability Research — Status

Date: 2026-10-02
Spec: TDRP-001 Revision 4
Parent: FBCP-001
Complete: false

| Area | Status | Note |
| --- | --- | --- |
| Telegram role as research source | specified | no runtime provider |
| Fixed upstream baseline | recorded | 33261535a0e747f125e0ed25486f01e556330677 |
| Recursive source closure | partial | root + 35 direct gitlinks + all nested gitlinks including `cppgir`→`expected-lite` verified; tracked generator/packaging roots mapped; non-gitlink build acquisitions and resource/license closure remain |
| C++ responsibility inventory | blocked | P0 pending |
| Full capability graph | in-progress | recursive root discovery added missing communities/AI/todo/ringtone/self-destruct/statistics/editor/IV/support/TDE2E domains; behavior dossiers still pending |
| Current Fabushi owner inventory | recorded | refreshed against exact PR #20 `de0f1749a675729fb97017085374b96aaa2ca7bb`; must refresh if PR #20 moves |
| Capability → existing owner mapping | in-progress | message/history lifecycle now resolves to existing transcript/session/composer/pagination owners; remaining capabilities pending |
| Absorption plans | in-progress | message/history lifecycle dossier records concrete owner/model/persistence/native-sync changes; remaining capabilities pending |
| Minimal new-owner proposals | blocked | only if required |
| Native network implications | in-progress | message/history dossier defines idempotent durable intent, ordered events, gap recovery, bounded retry and reconnect requirements; broader domains pending |
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
