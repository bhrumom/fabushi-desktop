# Telegram source read: deterministic orders 4201-4300

Authority: accepted `telegramdesktop/tdesktop@22b352e866d0402505c07fa4ed21d75d7e4fb3db` / tree `94ae09469c816b350f60dc9ada1ff049323be8e7`; exact blob/size/type comes from Source authority run `37872749663`, job `113634587680`, artifact `11591390327` (sha256 `9e8d9d297b9f957c5c60e069a3ede35c517d56e18662bcf7573946663b971d65`).

Orders 4201-4207 finish Data Export UI: resolved settings/path, progress rows, delayed skip-file affordance, cancel/done/start streams and top-bar progress. 4208-4212 define bounded FFmpeg byte/file seek, frame decoding/timing/scaling, thread/memory limits and RAII/error handling. 4213-4227 define Gram/TON account/emulation/NFT/rates/request/stream/BOC contracts with fail-closed parsing. 4228-4282 are immutable API/fragment fixtures and parser/emulation/NFT/rates/stream tests; they are treated as test oracles/provenance, not as shipping UI/product owners. 4283-4291 enter Admin Log filters/event list/section lifecycle. 4292-4300 enter core History, drag/drop, inner transcript interaction/accessibility and message-item state.

The History rows directly deepen `TDRP-R9-HISTORY-STATE-CONTRACT-001` and `TDRP-R9-HISTORY-LIFECYCLE-001` source understanding, but do not promote them until Fabushi shipping entrypoints and exact-head tests/artifacts prove parity.

Accounting: `read_through=4300`, `unread=11820`, `unknown=15841`, `unknown_closed=279`, `omitted=0`.
