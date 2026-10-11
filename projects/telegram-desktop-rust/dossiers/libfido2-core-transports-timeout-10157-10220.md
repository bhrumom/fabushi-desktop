# libfido2 core, transports, timeout and crypto boundary — orders 10,157–10,220

Authority: `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941` -> `Telegram/ThirdParty/libfido2` -> `Yubico/libfido2@b974e7cf2ee7392134cc12c08b76a068cf250dd8`.

Exact identity/materialization evidence: Fabushi exact head `8cd23e6ec07d69b186be674947ac45ce38f3453a`, Rust desktop runtime run `38123961987`, source-authority job `114424363470`, artifact `11695949458`, digest `sha256:fa16db976f759301292bf23dd454bed9f6e70ad386c772333bfbb93c0b329291`. The source-authority job completed successfully, including exact-head binding, recursive inventory validation, range attestation, Revision 9 authority validation, review-window materialization and accepted-mode fail-closed checks.

## Read responsibility

Orders 10,157–10,220 are exact-read by blob identity. The range starts at `regress/es384.c@b55ce015e17a2d58728fef19dd945017f05359ab` and ends at `src/random.c@377843b4215ea6d1d0361a688c3d273eb83328c3`. It covers ES384/RS256 regression, the mock authenticator transport, build/export composition, AES/key agreement/signature primitives, assertion and credential ceremonies, CBOR parsing/serialization, biometric/configuration/credential-management state, device open/close/cancel/timeout, typed errors, public/internal API state, HID on FreeBSD/HIDAPI/Linux/NetBSD/OpenBSD/macOS/Unix/Windows, getInfo capability parsing, framed I/O with one mutable timeout budget, ISO7816, largeBlob integrity/encryption/chunking, diagnostics, Linux netlink discovery, NFC, PCSC, PIN/UV permissions/retries/tokens, and OS CSPRNG behavior.

Existing-owner-first keeps one source-neutral production path: Coordinator WebAuthn/security + Host bridge, canonical Settings/platform-device, Secrets/Security/Crypto dependency, Platform/Resource, Telemetry/Diagnostics and Build/Packaging/Release. No libfido2/Telegram runtime, second identity/credential store, second Settings owner or custom cryptography is introduced.

## Production delta

The source review makes timeout/cancellation an explicit applicable lifecycle responsibility. `src/dev.c` carries `dev->timeout_ms`; `src/io.c` subtracts elapsed time from one mutable budget across write/read/keepalive/continuation frames; assertion, credential and PIN/UV operations use that budget; explicit device cancel is separate. Fabushi already supports remote request cancellation, stop/dispose cancellation and child kill/reap, but `SpawnedWebAuthnSigner::sign_interactive` had no client lifetime timer. A stuck external helper could therefore live indefinitely if the remote peer never sent a cancel.

The existing signer is hardened rather than replaced. It derives the RP timeout hint from the frozen `optionsJson` ceremony representation (also accepting a future flat/nested shape), clamps it to the WebAuthn Level 3 client recommendation of 300,000–600,000 ms, defaults to 300,000 ms, and kills/reaps the helper when the deadline expires. Remote cancellation still interrupts immediately. Focused Rust tests cover default/clamp parsing and a real synthetic stuck child process on Unix. This is a bounded-lifecycle replacement, not proof of full security-key parity.

## Open responsibilities

All 64 rows remain `mapped-open` / `unknown_closed=false`. The external `sand-webauthn-signer` producer, packaged co-staging/provenance, real HID/NFC/PCSC hardware behavior, biometric/resident-credential/largeBlob management policy and three-platform packaged evidence remain open. Crypto operations remain delegated to audited platform/Rust/helper dependencies; no upstream C crypto is copied.

The ledger-wide `target_commit=3bc92400826cc4ca7ac665b467708e22261edc61` is deliberately not rewritten during incomplete acceptance. `scripts/validate-tdrp-revision9-authority.mjs` requires ledger target_commit == exact tested GitHub Actions HEAD only under `--require-accepted`; current `acceptance.accepted=false`, `baseline_ready=false`, unread and unknown are non-zero. Writing an untested descendant SHA now would fabricate evidence.

Accounting after this source read is **10,220 / 16,125 read; 5,905 unread; 15,845 unknown; 0 omitted**. The next deterministic source is order **10,221** `Telegram/ThirdParty/libfido2::src/reset.c@4e09dbbca5cf968ddb9d728dcc13da88cdfd9264`.
