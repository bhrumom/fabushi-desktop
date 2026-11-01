# Round 125 — MTProto direct HTTPS/Web-PKI carrier lifecycle

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 / round 94
Fixed upstream: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4
Stage: A / validation not run / accepted=false

## Fixed-U source reread

Carrier behavior was reread from the fixed source tree, not inferred from previous chat:

- special_config_request.cpp@fdb65506b0a886bc2daeccb61090bc44be7e2d88
- special_config_request.h@91901489e778089b03187682a9556e929867b94f
- mtproto_domain_resolver.cpp@eeac8f6a8e2df67967ea7b536fb165e2f8d983e9
- mtproto_domain_resolver.h@cb1979bbbf0454135c8668ed1e5013da85d1dedc
- config_loader.cpp@5d07404ba07223016c6728b986dcba10628d7194
- mtproto_config.cpp@e8fc605ca197d2bc82a9f73da18eeb945967a99f

Confirmed invariants: dedicated QNetworkAccessManager with NoProxy; exact 800 ms stagger; DomainResolver valid result fences its per-key delayed guard while ordinary wrapper erase does not call explicit abort; SpecialConfigRequest acceptance does not internally stop the remaining stagger while the object remains alive; explicit ServiceWebRequest::destroy disconnects/aborts/deleteLater whereas ordinary destruction only deleteLater; finished replies read their body before removal even when error metadata exists.

## Existing-owner-first implementation

The first-party carrier is implemented in the existing platform-core native/telegram-network owner. No TelegramCore/TelegramRuntime/Provider parallel product, Codex HTTP owner, subprocess curl, or system-proxy client was introduced.

Platform sequence:
- spec-first authorization: a3a427f06fd4462a9d533a743a33564acadca0c6
- carrier implementation: dc2541aa769fb8fd3f9699683ba802a6a7afddbe
- dependency provenance: cbe7e60632397e705725db7b30c41ce4bef80eb7
- initial source-compliance spec: 2cc022c6602e69204bcff53bc985c01bfa993983
- target-lock disambiguation correction: 3b9368d2f3856d320557945226d8c8b039e2aed8
- final source-compliance spec: 5dd792399fe3c17d1256c80e0eb9fb2c1c2aa94f

Key blobs:
- direct_https.rs 0c7444d23fdee0aa135f12fd55a50ec3fe42c690
- telegram-network lib.rs bace7702a74f1c3b848ac12add69d8cd74748342
- Cargo.toml 51bd7e2a4ac50b76c3bad11e9aef7ac633f6cfef
- Cargo.lock d0d7968508c43b433f27cf0e8f2fc7aea33a4953
- THIRD_PARTY_NOTICES.md 2e84b5a0ad5d80be35939ee488e4c56b07f9125c
- lifecycle spec 6ad164db526366fae4d7a6084f1c013b39f8cf18

The carrier opens direct TcpStream origin connections to port 443, validates Web PKI with rustls/Web-PKI roots and origin-host SNI, serializes normalized origin-form GET requests, preserves Host override/Accept/User-Agent, bounds response header/body memory, retains readable non-2xx bodies as response data, and has no ambient proxy integration.

DirectHttpsRace preserves the 800 ms spacing and distinct settlement policies. DomainResolver-style acceptance fences future starts and releases outstanding handles without explicit abort. SpecialConfig-style acceptance only finalizes that request, leaving the remaining schedule live while the owner remains alive. Explicit cancel and owner Drop use NetworkAttemptControl cancellation.

The exact rustls 0.23.41 / webpki-roots 0.26.11 dependency closure was copied as source metadata from the repository's already-pinned native/global-dharma/Cargo.lock. Because the target lock already contained getrandom 0.2.17 and 0.3.4, ring 0.17.14's source-lock unqualified getrandom dependency was source-corrected to explicit getrandom 0.2.17 at 3b9368d2f3856d320557945226d8c8b039e2aed8; a pure-text audit then found no remaining ambiguous newly-added dependency references. No dependency resolver or lock generator was executed.

Desktop mirror commit 623936831ac1831022a35b22471a58bd96036104 reproduces direct_https.rs, lib.rs and THIRD_PARTY_NOTICES.md byte-identically.

## Downstream composition boundary

Existing request normalization, response normalization, special-config decode/select, global HTTP-time owner, RuntimeClient SessionOwner, temporary logical 1000+d routing, exact 6000 ms owned request, bare help.getConfig and dc_options-only replacement remain separate canonical owners. This round does not invent a second bootstrap runtime. The aggregate ledger item remains mapped/pending because automatic HTTP acquisition-to-RuntimeClient application is not newly composed in this slice and executable evidence is intentionally absent.

## Authored contracts — not executed

The carrier source contains authored-only contracts for:
- origin-form direct request construction and absence of Proxy-* headers;
- exact 800 ms stagger constant;
- non-2xx readable body preservation;
- bounded chunked-body decoding;
- DomainResolver winner fencing without explicit cancellation;
- SpecialConfig acceptance retaining remaining schedule;
- explicit cancellation being distinct from ordinary finalization.

No contract, unit test, build, typecheck, lint, validator, generator, dependency resolution, lock regeneration, integration/E2E, packaging, deployment, installation, or runtime acceptance was executed.

## Stage-A disposition

Responsibility: TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001
implementation_status: mapped
validation_status: not-run
accepted: false
