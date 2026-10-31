# Round 93 — MTProto new-session wire adjacent revalidation

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 62
Stage: A — source-level implementation/revalidation only, zero CI

## Live authorities

- desktop checkpoint before write: `47ab392a0991b73b59d0ddc09a38e2e39957b6ac`
- platform-core checkpoint before this round: `c32cc58ffe44214a102e7c455fe73c00f90e1aed`
- desktop main: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- platform-core main: `26c3cc8c69ab89e8fb38952adb8c438ec0b68e05`
- fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`
- `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`
- `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`

## Adjacent responsibility selected

`new_session_created#9ec20908 first_msg_id:long unique_id:long server_salt:long = NewSession`.

The fixed-U shipping branch reads the same three fields, then separately performs bad-time/request-history checks, server-salt adoption, historical resend iteration, and higher-level difference notification. This round deliberately isolates exact wire decoding from those broader semantics.

## Existing-owner-first disposition

No production delta. Existing canonical `telegram-protocol/src/updates.rs@23a09bb6e4e78ef526823cf01d0a207565fb2a3e` owns typed `NewSessionCreated` and strict `parse_new_session_created`. It consumes exactly 28 bytes (4-byte constructor + three 8-byte signed longs), rejects wrong constructor, truncation and trailing bytes. Current canonical/desktop network `lib.rs@1fc1b1f69e3e37377400d078f916599e0c7b20bc` calls it before salt mutation/current-RPC replay and on the ping/service path before salt adoption.

The unique ledger responsibility remains `TDRP-R9-MTPROTO-NEW-SESSION-WIRE-001`.

## Authored contracts, not run

Platform-core:
- revalidation spec commit `8d86554fcc4afc8d02b7ed38bfd5ca93221fede7`, blob `9e5ee768418dddfbde2f4534d9ff9c26d70ef5ba`;
- external contract commit `0db067351aef9365ada9e168f041599983dc1693`, blob `32a60800c34d53223fed2f1e4aa521e300b510d7`.

Desktop mirrors the contract blob byte-identically. No test was executed.

## Explicitly excluded/open

- arbitrary multi-request `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history;
- broader connection scheduling;
- auth-key binding/persistence;
- `gzip_packed` decompression;
- complete updates.difference / updates.differenceSlice decoding and application;
- Stage-B executable validation and independent release ACCEPT.

Ledger coverage remains `responsibilities_total=97, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, validator, generator, unit/contract/integration/E2E test, package, deploy, install, or runtime acceptance was performed.


### Round 93 final source-level compliance

Platform-core final compliance commit `3e24ea732acd2bcb2fe5bf2e5257f275643384de` / spec blob `298cbba2b593aa17502be752dd406718d745de93` records the fixed-U `NewSession` result-wire revalidation as source-level closed with **no production delta**. Canonical production protocol updates/lib/network blobs remain `23a09bb6e4e78ef526823cf01d0a207565fb2a3e` / `abab39e42f100972484191f0e52e9edd9b88c6cf` / `1fc1b1f69e3e37377400d078f916599e0c7b20bc`. Canonical and desktop authored-only external contract blob is `32a60800c34d53223fed2f1e4aa521e300b510d7` and remains not run.

The existing `TDRP-R9-MTPROTO-NEW-SESSION-WIRE-001` row stays `implemented / validation_status=not-run / accepted=false`. Arbitrary multi-request history, connection scheduling, auth-key binding/persistence, gzip, complete difference decoding/application, broader source/legal closure, Stage-B executable evidence, and independent release ACCEPT remain outside this wire-only closure or open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed.
