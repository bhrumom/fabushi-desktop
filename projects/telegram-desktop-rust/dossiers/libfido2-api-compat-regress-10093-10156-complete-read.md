# libfido2 API / compatibility / regression complete read — orders 10,093–10,156

Pinned authority: `Yubico/libfido2@b974e7cf2ee7392134cc12c08b76a068cf250dd8`, mounted at `Telegram/ThirdParty/libfido2`.

Exact source identities and blobs were materialized by predecessor exact-head `b8a52c954b9ea63e9f64b31d68fc537e71e1f2bf` Rust runtime run `38120045447`, source-authority job `114412753321`, artifact `11695420570` (digest `sha256:f08d296699a1041550c1fd1421f78b47c04f53f5063eab901369157f37275ab5`). The deterministic 64-blob window is independently cross-checked against the pinned upstream git tree. The next exact source is order 10,157 `regress/es384.c@b55ce015e17a2d58728fef19dd945017f05359ab`.

## Responsibility decomposition

The man/API tranche defines assertion and credential ownership, RP/client-data/authdata/signature/type binding, allow/exclude and resident-key selection, UP/UV, HMAC/largeBlob extensions, credential protection/enterprise attestation, device open/close and I/O, timeout/cancel/touch, PIN and PUAT permission lifecycle, biometric enrollment/info/template management, credential management, error typing and supported public-key conversion boundaries.

The OpenBSD compatibility tranche carries portable libc/clock/endian/console behavior plus security-sensitive lifetime rules: sensitive memory is explicitly cleared on release/reallocation, passphrase input remains bounded and platform-correct, and secret comparison uses timing-safe semantics. These are behavior requirements, not permission to port an OpenBSD compatibility runtime into Fabushi.

The regression tranche exercises malformed and adversarial assertion/credential/device/key behavior, compressed inputs, credential-protection/enterprise-attestation edge cases and EdDSA/ES256 conversion failures. Upstream regression code is evidence authority; Fabushi closure still requires source-neutral production tests and current-head evidence.

## Existing-owner-first result

The shipping owners already exist: Coordinator WebAuthn/security, the Host WebAuthn bridge, canonical Settings/platform-device and Secrets/security boundaries, and Build/Quality/Release. Host already owns the 120-second ceremony timeout/correlated cancel, while Coordinator owns per-request cancellation and pause/dispose settlement, so no second timeout/cancel owner is added.

A concrete applicable gap was found in the existing helper path: the PIN returned by the control plane was cloned into an ordinary `String`, then copied into an ordinary serialized JSON buffer. This descendant moves the PIN out of the control reply and holds both the PIN and serialized helper request bytes in pinned `zeroize 1.9.0` `Zeroizing` buffers until the write completes.

A separate shipping blocker remains open: the Coordinator resolves `sand-webauthn-signer` in packaged resources, but the inspected desktop build/staging composition does not produce and co-stage that helper. Therefore this window remains `mapped-open` / `unknown_closed=false`; source reading and PIN zeroization do not overclaim hardware/security-key release closure.

Accounting after this window: **10,156 / 16,125 read; 5,969 unread; 15,845 unknown; 0 omitted**.
