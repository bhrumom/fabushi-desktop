# Transcript Ordering Oracle

Status: active / normative  
Oracle ID: ORA-ORDER-001  
Updated: 2026-10-07

- ORD-I01: persisted canonical order == renderer semantic order == DOM article order（virtualized offscreen omission excepted）。
- ORD-I02: duplicate/replay cannot create duplicate semantic entry。
- ORD-I03: stable ID/clientNonce merges optimistic entry instead of second user message。
- ORD-I04: late baseline/history cannot move newer live entry across logical predecessors/turns。
- ORD-I05: preamble/tool/result/final relative order deterministic。
- ORD-I06: pagination prepend preserves existing relative order/selection/unread anchor。
- ORD-I07: switch/reopen/restart yields same semantic order。

Property cases include append/update/baseline permutations、duplicate、late page、optimistic+ack+authoritative、rapid turns、cross-conversation interleaving、reconnect replay。
