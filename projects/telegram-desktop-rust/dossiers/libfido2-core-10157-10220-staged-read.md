# libfido2 core implementation continuation — deterministic orders 10,157–10,220 staged read

This is a staged read dossier only. It does not change PR #42 source accounting while exact-head CI for `b8a52c954b9ea63e9f64b31d68fc537e71e1f2bf` is running.

Exact component authority: `Yubico/libfido2@b974e7cf2ee7392134cc12c08b76a068cf250dd8`, mounted by accepted `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941`.

## 10,157–10,188 — core ceremony, crypto-boundary and device state

The range covers ES384/RS256 regression, mock device I/O, core CMake composition, AES256 PIN-protocol encryption/decryption, assertion and credential CBOR encode/decode, authenticator key exchange, biometric lifecycle, bounded blob/buffer ownership, CBOR helpers, compression, CTAP2 configuration, resident credential management, device capability/open/cancel lifecycle, ECDH, EdDSA/ES256/ES384 verification, stable export surfaces and public FIDO types/APIs.

Applicable product responsibilities map to the existing Coordinator WebAuthn/security owner plus canonical platform-device/Settings/secure-storage/Build/Quality owners. Critical semantics are fail-closed malformed CBOR handling, bounded allocation and cleanup, permission-scoped PIN/UV authorization, deterministic cancellation/timeout, vetted cryptographic verification and explicit key lifetime. No upstream C runtime or custom cryptography is retained as a product owner.

## 10,189–10,220 — transport, PIN, largeBlob, random/reset/touch/TPM

Headers define resident-credential, error, cryptographic key, protocol parameter and device type contracts. HID implementations span FreeBSD/Linux/NetBSD/OpenBSD/macOS/Unix/Windows with report descriptor validation, device metadata, open/read/write/close, polling/signal/cancellation and exact report-length handling.

The remaining core files cover authenticator capability decoding, transport I/O framing and timeout accounting, ISO7816 APDUs, encrypted/compressed largeBlob lifecycle with integrity checks and permission-scoped authorization, logging, Linux netlink/NFC, PCSC, PIN/UV token/retry/reset state, OS randomness, reset, RSA verification, monotonic time, asynchronous touch status and TPM attestation parsing.

Existing-owner-first consequences:
- platform HID/NFC/PCSC differences stay behind one canonical authenticator platform adapter;
- PIN/UV permission tokens and retries stay in the existing WebAuthn ceremony/security state owner;
- secret material and PINs must not enter diagnostics;
- random values use vetted OS cryptographic randomness;
- largeBlob and attestation parsing fail closed on malformed data/integrity failure;
- transport timeouts/cancellation tear down deterministically;
- cryptographic operations remain delegated to vetted libraries.

These rows remain staged reading only and do not reduce `unknown`. The current PR authority remains **10,092/16,125 read; 6,033 unread; 15,845 unknown; 0 omitted** until a coherent shard/accounting promotion is made and fresh exact-head evidence is obtained.
