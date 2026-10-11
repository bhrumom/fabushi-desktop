# libfido2 API/compat/regression continuation — deterministic orders 10,093–10,156 read

This continuation is intentionally staged off PR #42 so the exact-head GitHub Actions for `b8a52c954b9ea63e9f64b31d68fc537e71e1f2bf` are not invalidated while running.

Identity authority remains accepted recursive inventory `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941` / tree `4cf9e0e1830ff8569e3264d07062d8ff94dbb4f6`, with exact component identity `Yubico/libfido2@b974e7cf2ee7392134cc12c08b76a068cf250dd8`. Each text blob in orders 10,093–10,156 was fetched directly from that exact component commit after its repository/commit/path/blob identity was resolved from the accepted recursive inventory. This dossier does **not** advance persisted source-disposition accounting yet; the registration shard and authority arithmetic must be added only in a coherent descendant before PR advancement.

## 10,093–10,124 — public WebAuthn/FIDO API contracts

The man/API range covers ES384/RS256 public-key conversion ownership; `fido2-assert`, `fido2-cred` and `fido2-token`; assertion allow lists, client/authenticator data, hmac-secret/largeBlob, UP/UV and signature verification; biometric info/enrollment/template lifecycle; full authenticator capability projection; credential exclude lists, creation, attestation/self-attestation verification and resident/credProtect/enterprise-attestation controls; resident-credential management; CTAP 2.1 configuration; PIN/UV Auth token permission scoping; asynchronous touch selection/cancellation; device discovery; largeBlob get/set/remove; device open/close/cancel/capability detection; custom transport/I/O and timeout hooks; PIN/retry/reset lifecycle; error-code mapping; and vetted RSA/EC key conversions.

Existing-owner-first maps these responsibilities to the current Coordinator WebAuthn/security owner plus canonical Settings/security-key management, platform-device, secure-storage and Build/Quality/Release owners. They do not justify a second authenticator runtime, credential store, identity owner or custom cryptography.

## 10,125–10,149 — security-sensitive portability layer

The OpenBSD compatibility layer carries product-neutral but security-relevant invariants: allocation/error propagation for `asprintf`/getline; monotonic clock portability; endian conversion; explicit memory clearing through `explicit_bzero` and `freezero`; overflow-safe `recallocarray` with zeroing of released/shrunk memory; bounded string helpers; POSIX/Windows I/O adaptation; terminal passphrase capture that disables echo and restores terminal/signal state; and `timingsafe_bcmp` constant-time comparison behavior.

Fabushi should preserve these semantics through idiomatic Rust/platform primitives in existing owners: secret/PIN buffers must not be logged or retained, cancellation/errors must restore prompt/terminal state, growth arithmetic must fail closed on overflow, and secret comparison must use vetted constant-time facilities where secrets are compared. The upstream C portability shims themselves are not copied into the product.

## 10,150–10,156 — regression authority

The regression build plus `assert.c`, `compress.c`, `cred.c`, `dev.c`, `eddsa.c` and `es256.c` exercise a large matrix of valid/invalid assertion and credential encodings, U2F/FIDO2 mode differences, extensions, resident/UV behavior, signature and attestation verification, compression round trips, device open/timeout/PIN/reset state, and vetted key conversion. The credential regression alone carries hundreds of assertions and is security-quality authority, not optional sample code.

Applicable closure belongs in canonical WebAuthn/security unit/integration/fuzz tests and packaged-helper acceptance. Invalid or malformed authenticator output must never degrade to success, and cryptographic verification must remain delegated to vetted libraries.

## Staging state

The current accepted PR authority remains **10,092/16,125 read; 6,033 unread; 15,845 unknown; 0 omitted** while its exact-head workflows execute. This dossier records additional completed reading only; it deliberately does not claim persisted read-through 10,156 or reduce `unknown`.

When promoted coherently, the next authority batch is orders **10,093–10,156**, and the next exact unread entry after that batch is **10,157** `Telegram/ThirdParty/libfido2::regress/es384.c@b55ce015e17a2d58728fef19dd945017f05359ab`.
