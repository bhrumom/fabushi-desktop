# Round 79 — MTProto resPQ exact wire validation

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536
Stage: A — implementation only; executable validation deferred

## Fixed-U source readback

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.
`Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`
defines:
`resPQ#05162463 nonce:int128 server_nonce:int128 pq:string server_public_key_fingerprints:Vector<long> = ResPQ`.

Shipping `DcKeyCreator::pqAnswered` in
`Telegram/SourceFiles/mtproto/details/mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03`
checks the active request nonce, consumes server_nonce/pq/fingerprints, selects a trusted RSA key and factorizes pq before preparing the next DH request.

## Existing-owner disposition

Canonical `native/telegram-protocol/src/handshake.rs@87616f4b96a1775455da42b0bbed77711fb3c9f6`
already implements `parse_res_pq` and `AuthKeyHandshake::receive_res_pq`.
Source readback confirms constructor-first decoding, exact fixed-U field order, strict EOF, active nonce binding,
empty-pq rejection and empty-fingerprint rejection. Canonical
`native/telegram-protocol/src/tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264`
already enforces TL Vector magic, non-negative length and the configured 64-element resPQ vector bound.
No production behavior delta is needed; server_nonce is introduced by the accepted resPQ response and therefore is captured rather than compared to prior state.

## Spec-first and authored-only contracts

Spec-first platform commit:
`03dc0f804a8c80d8d60500e6667cc9b091ccc3cf`.

Spec blob:
`docs/specs/telegram-mtproto-res-pq-wire.md@ab044dcb2b34d7def042e4dd2eb798f3b658c52a`.

Canonical authored-contract commit:
`5e1ba90b32ac7c58b7be0d9630f1630a0a764061`.

Canonical contract blob:
`native/telegram-protocol/tests/protocol_contract.rs@44d2a5d141e7d7854529379aaaad660853c8cbe3`.

Desktop provider contract sync:
`bhrumom/fabushi-desktop@c72c7f85c926d05b21842c41a3b11c1313537793`,
with byte-identical
`source/mahayana/mahayana-rs/providers/telegram-protocol/tests/protocol_contract.rs@44d2a5d141e7d7854529379aaaad660853c8cbe3`.
Desktop facade remains
`source/mahayana/mahayana-rs/providers/telegram-protocol/src/lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf`.

Authored-only cases cover:
- valid exact resPQ with two fingerprints;
- every truncation cut;
- complete four-byte unknown constructor;
- wrong Vector constructor;
- negative Vector length;
- Vector length above the configured resPQ bound;
- declared fingerprint count larger than carried elements;
- trailing bytes;
- active request nonce mismatch;
- empty pq;
- empty fingerprint vector;
- receiver invocation outside AwaitingResPq;
- no state mutation on the receiver-side validation failures above.

These contracts were written and manually reviewed only. They were not executed.

## Scope disposition

Closed by this slice:
- fixed-U exact resPQ constructor and field decoding;
- bounded fingerprint Vector validation;
- active nonce binding;
- empty pq/fingerprint rejection;
- strict truncation/trailing behavior;
- existing AuthKeyHandshake state ownership.

Still fail-closed/open:
- temporary/permanent auth-key binding, expiry, renewal, persistence and teardown;
- connection scheduling/racing, plaintext send/retry/timeout;
- arbitrary multi-request sent/resend/container history;
- gzip_packed bounded decompression;
- complete updates difference/differenceSlice decoding/application;
- broader fixed-U source/legal/release closure;
- all Stage-B executable evidence and independent release ACCEPT.

Ledger state:
`TDRP-R9-MTPROTO-RES-PQ-WIRE-001 = implemented / validation_status=not-run / accepted=false`.

Coverage:
`responsibilities_total=89`, `unknown=16172`, `unread=16145`, `omitted=0`.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator,
unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


## Final source-level compliance readback

Platform-core compliance commit:
`54f1a26231827ac11a1ea0544dc4d1f7c3762c6c`.

Final spec blob:
`docs/specs/telegram-mtproto-res-pq-wire.md@404bbe591fb514aba732e5b4e8f6481cc2904618`.

Remote source readback records R1-R9 and AC-1-AC-5 as source/bookkeeping satisfied while executable Stage-B validation remains
intentionally not run. The responsibility remains
`implemented / validation_status=not-run / accepted=false`.
