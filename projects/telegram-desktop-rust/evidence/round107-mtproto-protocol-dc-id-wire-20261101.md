# Round 107 — MTProto protocol DC-id wire source-level evidence

Task: 7457eb1c-11a3-4626-8563-a3e495872536 / user round 76  
Stage: A source-level implementation only; validation not run; acceptance false.

## Live authority at start

- Desktop checkpoint: `28f7858cc08bfaa630cec1e2e4393455926abe37`.
- Platform-core checkpoint: `5b7ed34df4c8063fe3db9627552bc673dda9c92f`.
- Fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.

Both checkpoint refs were reread from GitHub before writes. All ref updates used `force=false` with the current expected SHA.

## Fixed-U source conclusion

`SessionPrivate::getProtocolDcId` in `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` is independently isolatable from connection scheduling. Its exact ordered transform is:

1. `BareDcId(_shiftedDcId)` using fixed-U modulus 10000.
2. If the bare id is temporary (>=1000), subtract 1000 for the wire identity only.
3. If test mode, add 10000.
4. If the connection's current DC type is `MediaCluster`, negate the final id.
5. Supply the signed 16-bit value to the existing connection-start wire owner.

The endpoint/directory identity and protocol wire identity are distinct: a temporary directory id such as 1002 remains 1002 for lookup while its wire id is 2; shifted media 160002 looks up directory 2 while writing -2. Fixed-U currentDcType/auth-key selection can force a media-origin session to Regular during broader key lifecycle; this slice therefore models the sign as an explicit protocol wire role and does not claim that broader lifecycle.

## Canonical implementation

- Spec: `9d0f25b21b46ef74f4589dec1d3459117d4bdbd6`, blob `725069760d50242874de30ddc5a2045895c23103`.
- Production: `e9cfc7fbd06002c6510d3bf45df9710e55ff1d57`.
- `native/telegram-protocol/src/dc.rs@a9582e4ece93590c322febbff391ba3ff7635575`.
- `native/telegram-protocol/src/lib.rs@daf27c48a422ed6ed663ca0021ff138436d99c0b`.
- `native/telegram-network/src/lib.rs@3034acab752ca37531033bb8a1d106003b20265d`.
- Authored-only contracts: `ceacfa08800950103c6118b6c579ef582b1e1458`.
- Protocol contract blob: `adc51856cebcdb6e539828e3d4abb9952fc32c29`.
- Network contract blob: `4adbcabf66dc0ca35861f46e40e9a8ca77fb73c5`.

The protocol owner exposes `derive_protocol_dc_identity` plus an explicit Regular/MediaCluster role and returns both `directory_dc_id` and signed `wire_id`. The existing network carrier delegates the rule, keeps endpoint iteration/socket ownership, and passes the signed wire id to the already-closed obfuscated connection-start owner.

## Desktop provider synchronization

Commit `385e5a261da3620733053503027c9a28c71cc221` mirrors canonical:
- `source/mahayana/mahayana-rs/providers/telegram-protocol/src/dc.rs@a9582e4ece93590c322febbff391ba3ff7635575`
- `source/mahayana/mahayana-rs/providers/telegram-protocol/src/lib.rs@daf27c48a422ed6ed663ca0021ff138436d99c0b`
- `source/mahayana/mahayana-rs/providers/telegram-network/src/lib.rs@3034acab752ca37531033bb8a1d106003b20265d`
- `source/mahayana/mahayana-rs/providers/telegram-protocol/tests/protocol_contract.rs@adc51856cebcdb6e539828e3d4abb9952fc32c29`

## Authored-only contracts

Protocol contracts cover regular/test, temporary normalization, shifted media negative ids, shifted temporary-media composition, invalid zero-bare cases, and preservation of a positive regular/CDN role. Network contracts cover separate bare-directory lookup and signed-wire-id carriage for media and temporary identities.

No contract was executed.

## Remaining open scope

Proxy/Web/TLS camouflage; quick-ACK lifecycle/correlation; broader currentDcType and TemporaryMediaCluster auth-key selection/binding/persistence/expiry/renewal; connection scheduling/racing/reconnect/send retry; arbitrary request histories; gzip; complete difference/differenceSlice decoding/application; broader fixed-U source/legal closure; Stage-B executable evidence; independent release ACCEPT.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


## Final source-level compliance readback

Platform-core final compliance commit is `0da9e826633817068f06159d90d46eaf664aa51b`, spec blob `cdb4a4565257b7919f510244ec4a11a4c7fcf32f`. It records R1-R10 and AC-1..AC-5 as source-level/spec-compliance satisfied while explicitly preserving `validation_status=not-run` and `accepted=false`; it references desktop bookkeeping `dc671558c60f4c292e2a8bf6de8faf1000da6c89`.

No test, build, validator, runtime gate, Actions/status/check query, package, deployment or installation evidence was produced in this round.
