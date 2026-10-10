# MTProto control/security 5261-5270 complete read

Accepted upstream: `42f8a36d43b8c805bc821905bea4cfeb3af1d41d` (tree `6ac9bbc1b44edcb119b1a724e7b0a321c3b7b8fa`)

This dossier records a direct exact-blob read of recursive orders 5261-5270. It grants source-read and responsibility-decomposition credit only. It does **not** close unknown responsibilities, does not establish MTProto as a Fabushi runtime, and does not grant release/verification credit.

## Responsibility decomposition

- **5261 — `Telegram/SourceFiles/mtproto/mtp_instance.h` @ `eeb56d29cc8316bd6600c26b6f8edf3a912601eb`**: authoritative request/session/config/key control surface: request identity, dependency ordering, cancellation, restart, config refresh, auth-key lifecycle and callback routing.
- **5262 — `Telegram/SourceFiles/mtproto/mtproto_auth_key.cpp` @ `cb4a4e5c3b94ed33d126068792536a08800615f2`**: authentication-key material lifecycle: stable key identity, creation/expiry metadata, bounded serialization and protocol crypto derivation helpers.
- **5263 — `Telegram/SourceFiles/mtproto/mtproto_auth_key.h` @ `31d0e3db369b309b8f2e9ea1d6b83b6a6d51b20c`**: typed authentication-key ownership and expiry contract plus explicit crypto boundary.
- **5264 — `Telegram/SourceFiles/mtproto/mtproto_concurrent_sender.cpp` @ `89ad7d9625f0858574961c6d90c412bf2ce092e8`**: concurrent request adapter: weak-lifetime dispatch, main-thread handoff, parse failure settlement, fail-skip policy, cancel-all and detach semantics.
- **5265 — `Telegram/SourceFiles/mtproto/mtproto_concurrent_sender.h` @ `f96d0fe3ac1f1ca88fea447f62aaf3596e999712`**: typed request-builder/canceller contract with target, delay, dependency and callback ownership.
- **5266 — `Telegram/SourceFiles/mtproto/mtproto_config.cpp` @ `e8fc605ca197d2bc82a9f73da18eeb945967a99f`**: remote configuration state defaults and versioned serialization/deserialization with environment-aware values and bounded persistence.
- **5267 — `Telegram/SourceFiles/mtproto/mtproto_config.h` @ `289230b64480cb8401239782a1ca3203067baeb1`**: typed configuration fields and DcOptions ownership contract.
- **5268 — `Telegram/SourceFiles/mtproto/mtproto_dc_options.cpp` @ `d8499fc1df2ed7fdb88da687a04b824d243ff49a`**: endpoint/CDN-key authority: environment defaults, validated mutation/serialization, change events, proxy/media/CDN filtering and endpoint selection.
- **5269 — `Telegram/SourceFiles/mtproto/mtproto_dc_options.h` @ `4e0b979c4a84661f3365196ab8702dc8cb60fd03`**: typed endpoint, environment and DC-kind lookup contract with concurrent read/write ownership.
- **5270 — `Telegram/SourceFiles/mtproto/mtproto_dh_utils.cpp` @ `db9c0542fa1f2d86c27da38f511d56ac2accc21c`**: Diffie-Hellman safety boundary: prime/generator validation, bounded modular exponentiation and authenticated key material derivation.

## Fabushi disposition

All ten files are **mapped-open-platform-protocol-replacement**. Applicable behavior belongs in the existing Fabushi Coordinator/Host/backend transport, credential/auth, configuration, request-settlement/idempotency and observability owners. Telegram-specific DC selection, MTProto serialization, AES/DH primitives and wire protocol remain source-specific replacement details and must not create a second runtime.

Required closure evidence still includes production-owner audit, lifecycle/cancellation/dependency/idempotency/security tests, exact-head GitHub Actions, packaged temporal acceptance and independent release acceptance. `unknown` remains unchanged and `omitted=0`.
