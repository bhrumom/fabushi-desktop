# Round 97 — MTProto TLS ClientHello template adjacent disposition

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 66
Stage: A (source/spec/bookkeeping only; executable validation deferred)

## Live authority

- Desktop checkpoint before write: `13c043939ca6806fdd87e030f67e6e7ed3db2811`.
- Platform-core checkpoint after spec-first disposition: `d2efec8b167386ad70da5607e2fd5833fc6387b0`.
- Desktop main: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`.
- Platform-core main: `26c3cc8c69ab89e8fb38952adb8c438ec0b68e05`.
- Fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- `mtproto.tl`: `76225965bfc794388db241f3f760db8ab79e84f8`.
- `mtproto_tls_socket.cpp`: `e7562e3ca1a1da03f5d368085d5e61d887ab3f16`.

## Disposition

The adjacent `tlsClientHello/TlsBlock` family is a generated local template AST consumed by Telegram Desktop's TLS camouflage generator, not an independent MTProto request/response wire family. `PrepareClientHelloRules()` creates nested scopes/permutations; `Generator::Part::writeBlock` interprets them into TLS ClientHello bytes and owns random/grease/public-key/padding behavior.

The current canonical telegram-protocol API has no TLS-template surface, and the current canonical telegram-network transport has no shipping TLS camouflage generator/socket lifecycle. Adding a standalone parser/serializer would be dead code and would falsely claim production closure. Disposition is therefore **no production delta**.

Platform-core durable spec: `d2efec8b167386ad70da5607e2fd5833fc6387b0` / `docs/specs/telegram-mtproto-tls-client-hello-template.md@bc0363cec846718b2fb0d54bf89943fe292714e4`.

Canonical and desktop telegram-protocol production blobs remain byte-identical: api `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43`, lib `abab39e42f100972484191f0e52e9edd9b88c6cf`.

The unique ledger responsibility is recorded as `mapped / blocked / validation_status=not-run / accepted=false`. It is intentionally not marked implemented until a real existing canonical telegram-network TLS camouflage lifecycle exists. Coverage is `98 responsibilities / 16172 unknown / 16145 unread / 0 omitted / 0 unmapped`.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed.


### Final source-level compliance

Platform-core final compliance is `06ac74bc56fcdb362e4e004980d6c60917311bf6` / spec blob `d40d0a5f6d61f4bf16cbc58e303660350885756a`; it references desktop bookkeeping checkpoint `918837f0456ea3745bd24d8bf11fd72e379b5069`. The responsibility remains `mapped / blocked / validation_status=not-run / accepted=false`, with no production delta and no executable validation.
