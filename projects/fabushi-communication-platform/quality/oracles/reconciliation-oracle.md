# Transcript Reconciliation Oracle

Status: active / normative  
Oracle ID: ORA-RECON-001  
Updated: 2026-10-07

Sources：optimistic local、live append/update、authoritative baseline、pagination/history、ack/clientNonce、reconnect/reload restore。

- REC-I01 idempotent duplicate application。
- REC-I02 supported valid event reorders converge to same semantic transcript。
- REC-I03 baseline cannot erase newer observed live terminal without explicit delete/tombstone。
- REC-I04 nonce/stable-ID resolves optimistic user entry without duplicate。
- REC-I05 empty/partial incoming text cannot overwrite richer committed text absent explicit redaction。
- REC-I06 attachment echo merges without duplicate/lost delivery。
- REC-I07 reload/reconnect convergence preserves final and order。
- REC-I08 scope prevents cross-account/conversation correlation。

Use table + property/state-machine sequences for baseline-before-live/live-before-baseline/update-before-baseline/duplicate/replay/late page/reconnect/random bounded permutations；retain counterexample seed。
