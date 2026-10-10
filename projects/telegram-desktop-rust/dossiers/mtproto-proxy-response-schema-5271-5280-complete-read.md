# MTProto proxy / response / schema 5271-5280 complete read

Accepted upstream: `42f8a36d43b8c805bc821905bea4cfeb3af1d41d` (tree `6ac9bbc1b44edcb119b1a724e7b0a321c3b7b8fa`)

Direct exact-blob read is complete for recursive orders 5271-5280. This grants read-through and responsibility-decomposition credit only. It does not reduce `unknown`, does not permit omission, and does not introduce Telegram/MTProto as a Fabushi runtime.

## Responsibilities

- **5271 — `Telegram/SourceFiles/mtproto/mtproto_dh_utils.h` @ `e857e671500cf4283ea10ae05d8ce6acdb2c24b5`**: typed Diffie-Hellman prime/modexp validation and key-derivation API contract.
- **5272 — `Telegram/SourceFiles/mtproto/mtproto_pch.h` @ `284089e7e19470f7bc2f197b8a09cfed7050b4ad`**: build compilation context for MTProto Qt/network/reactive dependencies; source-specific build input with no standalone shipping product owner.
- **5273 — `Telegram/SourceFiles/mtproto/mtproto_proxy_data.cpp` @ `f9748fcb6358dd8ce7429536ec2b999cb2a989ef`**: proxy configuration validation/canonicalization, credential secret parsing, host/path security binding, resolved-IP expiry and platform proxy conversion.
- **5274 — `Telegram/SourceFiles/mtproto/mtproto_proxy_data.h` @ `d8deebac2359936f72901ba0eec0cbfdfb186e90`**: typed proxy settings/type/status, resolved-address lifetime, capability URL and platform conversion contract.
- **5275 — `Telegram/SourceFiles/mtproto/mtproto_response.cpp` @ `a6456394834ed9e6f605e8acabc2ca73eab8e03a`**: fail-closed protocol error parsing into structured code/type/description with local error construction.
- **5276 — `Telegram/SourceFiles/mtproto/mtproto_response.h` @ `4025f784561e8b6743b7223686bef28cf373d5b7`**: typed response correlation plus temporary/flood/default-handled error classification and user-fallback contract.
- **5277 — `Telegram/SourceFiles/mtproto/proxy_check.cpp` @ `c2c2ddb0cf2286f62ccbf51a60eccaa472c12d79`**: bounded proxy connectivity probe lifecycle with dual-stack selection, latency settlement, cleanup and failure routing.
- **5278 — `Telegram/SourceFiles/mtproto/proxy_check.h` @ `1cbfb15fd2cae6f92f49a4c4d999a30b00073780`**: typed proxy-check connection ownership/reset/drop/start contract.
- **5279 — `Telegram/SourceFiles/mtproto/scheme/api.tl` @ `57e72169e6e8fc8d36af957a96ca20f40b5d8a07`**: Telegram application API schema/generator authority covering typed peer/message/media/account/service methods; protocol schema input requiring source-neutral Fabushi typed API/service responsibility mapping.
- **5280 — `Telegram/SourceFiles/mtproto/scheme/mtproto.tl` @ `76225965bfc794388db241f3f760db8ab79e84f8`**: MTProto core wire schema for auth handshake, ack/replay/state/resend, errors, ping, salts and session lifecycle; source-neutral transport/session responsibilities remain required.

## Canonical disposition

The applicable responsibilities remain mapped-open to existing Fabushi network/settings, Coordinator/Host/backend transport, typed API/service, credential/auth, structured error, retry/idempotency and observability owners. Telegram TL, MTProto crypto/wire framing and source-specific proxy formats are replacement details rather than shipping architecture.

Order 5272 is build compilation context, but under the current completion rule it still receives read-through only; `unknown` is not reduced until all relevant production/platform composition and verification gates are closed. Orders 5279-5280 are protocol schema authorities, not permission to wholesale-mark Telegram product responsibilities non-applicable.

Required closure still includes current-owner audit, production implementation where missing, lifecycle/security/error/retry tests, exact-head GitHub Actions, packaged temporal evidence and independent acceptance. `omitted=0` remains invariant.
