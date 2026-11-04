# Round 119 — Global HTTP-derived network time owner

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 88
Stage: A, deferred executable validation

## Fixed source

- `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`
- `Telegram/SourceFiles/mtproto/special_config_request.cpp@fdb65506b0a886bc2daeccb61090bc44be7e2d88`
- `Telegram/SourceFiles/mtproto/config_loader.cpp@5d07404ba07223016c6728b986dcba10628d7194`
- `Telegram/SourceFiles/mtproto/mtp_instance.cpp@d25fe3b72298fab371854f80186ee8e4777ea087`
- `Telegram/SourceFiles/mtproto/details/mtproto_tls_socket.cpp@e7562e3ca1a1da03f5d368085d5e61d887ab3f16`
- pinned submodule `Telegram/lib_base@95a543e426e35938dd632139d0bb828b268c3b4c`
- `base/unixtime.h@8953d491fe7604df777c18812ba7d8a357c50836`
- `base/unixtime.cpp@b39b788a8a125d29d06f637c57c9b52ac24b138e`

## Source semantics

`SpecialConfigRequest::handleHeaderUnixtime` ignores replies with network error, requires a non-empty case-sensitive raw `Date` header and a valid fixed-U HTTP-date parse, then calls `base::unixtime::http_update(parsed_seconds)`. A time-only request invokes its completion callback only after that mutation; config-bearing successful replies also update the same global HTTP time.

The pinned lib_base owner separates global MTProto/base time from HTTP calibration. `http_update` stores an HTTP shift relative to current base-adjusted time and sets valid=true. `http_now` always returns base-now plus the stored HTTP shift, even when valid=false. `http_invalidate` clears only validity. A non-forced MTProto/global update ignores shift deltas below three seconds without setting its one-shot updated flag; the first material update sets the global base shift, clears HTTP shift and validity, and later non-forced updates do not rewrite that global base.

`Instance::Private::syncHttpUnixtime` starts a time-only SpecialConfigRequest only when HTTP time is not valid and no loader is alive. Fixed-U TLS camouflage consumes `base::unixtime::http_now()` for its ClientHello timestamp.

The temporary-DC application path is separate: ConfigLoader chooses a special endpoint, maps it to a temporary DC id, inserts a TCP-only temporary option, sends `help.getConfig`, and on success replaces only `dc_options`; timeout/termination cancels the request and kills the temporary session. This path remains pending.

## Existing-owner-first decision

Canonical Fabushi `native/telegram-network` already owns both trusted MTProto session-time correction and the real TCP/TLS camouflage shipping path. `telegram-runtime` owns client lifecycle but has no better independent clock owner. `telegram-protocol` already owns the pure HTTP Date parser and must remain side-effect free.

Therefore the global mutation/state boundary is independently implementable without the absent HTTP carrier and belongs in existing `telegram-network`. The carrier will later pass parsed Date seconds to this owner; it does not own clock state.

## Implementation

Platform spec-first sequence:

- spec commit `645c6e924f38bc4e073d9c898bfc64eb5546f583`;
- initial production commit `16e9ea2297b4a8d6d8d41177f74bec927d93f11f`;
- source-review fixed-U global update gate correction `7c822678107d592c34380fe20683a2ecea66acd2`;
- source-compliance/spec commit `1d8c8bd07de0c9a2a970610c39830a77eeff150c`;
- final cross-repository evidence commit `39aa2275f68f168752d9cf24b52dbf56c3716d02`;
- final spec blob `cc27f7ae6eca445aa6902f0af48619da28ab83f8`.

Canonical/mirrored production blobs:

- `network_time.rs@19b77a6960d6059b7b708ceb4ec13643a1ec0944`;
- `telegram-network/src/lib.rs@e0dc33a728916fb7be916956f44cfb0b670486c6`;
- desktop mirror correction commit `53e1a4b22e94a2d688d47e681ef41e6e4bf4247e`.

The new owner uses process-global atomic shifts/validity with checked arithmetic; the real TLS handshake obtains `http_time_now` and feeds the existing explicit-timestamp TLS helper with OS secure randomness. Auth-key establishment and authenticated bad-message 16/17/64 correction feed the trusted global MTProto update boundary while retaining the existing per-session message-id offset.

No Cargo manifest or lock changed. No parallel runtime/time service was added.

## Authored contracts

Eight isolated source contracts cover default uncalibrated time, HTTP shift relative to base, invalidate-preserves-shift, MTProto invalidation/reset, non-forced one-shot behavior, sub-three-second ignore without lockout, repeated HTTP replacement, and checked overflow. They are authored under `#[cfg(test)]` and were not executed.

## Remaining blockers

- direct dedicated NoProxy HTTPS/Web-PKI HTTP carrier;
- 800 ms staggered request/reply lifecycle and cancellation semantics;
- actual live Date-header acquisition/call into `update_http_time`;
- temporary-DC `help.getConfig` timeout/cancel/kill lifecycle and `dc_options` application;
- Stage-B executable evidence and independent release acceptance.

## Ledger disposition

- new `TDRP-R9-MTPROTO-HTTP-DERIVED-NETWORK-TIME-001`: `implemented / rewritten / validation_status=not-run / accepted=false`;
- `TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001`: remains `mapped / pending / validation_status=not-run / accepted=false`;
- coverage: `source_entries_total=16202, responsibilities_total=121, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.
