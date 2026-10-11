# libfido2 platform/tools/release continuation — deterministic orders 10,221–10,259 staged read

This is a staged reading record only. PR #42 remains pinned to `b8a52c954b9ea63e9f64b31d68fc537e71e1f2bf` while exact-head Actions run; its persisted source authority remains 10,092.

Exact component authority: `Yubico/libfido2@b974e7cf2ee7392134cc12c08b76a068cf250dd8`.

## 10,221–10,226 — U2F / native platform completion

`types.c` owns bounded array cleanup/packing. `u2f.c` carries mature U2F register/authenticate/touch fallback and exact error/state translation. `webauthn.h` is the Windows WebAuthn ABI/contract authority and `winhello.c` adapts native Windows Hello/WebAuthn credentials/assertions, cancellation and capability projection into libfido2 semantics.

For Fabushi these responsibilities stay inside the existing Coordinator WebAuthn/security owner and the canonical Windows platform adapter. Native Windows WebAuthn is a platform backend, not a second identity or credential store. U2F fallback behavior is compatibility/state-machine authority only where product policy still supports legacy authenticators.

## 10,227–10,248 — executable security-key management tools

The command-line tools make the behavior contracts concrete: get/verify assertions, make/verify credentials, biometric enroll/list/rename/remove, CTAP2 config, resident credential enumeration/update/delete, largeBlob get/set/remove, PIN change, authenticator info/reset and capability reporting. Their test script exercises resident/UV/protection and verification paths.

Fabushi must expose any applicable user-visible security-key management through canonical Settings/identity/security surfaces, with permission-gated device operations, explicit destructive confirmation, secret-safe PIN handling and fail-closed verification. These tools are source authority; they are not shipped as a parallel product UI.

## 10,249–10,253 — Linux device access authority

`udev/70-u2f.rules`, the device table, generator and checks encode known authenticator VID/PID coverage plus hidraw/uaccess/security-key tagging. Fabushi packaging/platform ownership must ensure supported Linux authenticators are discoverable through least-privilege device access and must test generated rules/device metadata consistency.

## 10,254–10,259 — Windows dependency/release supply-chain authority

Windows build/release scripts pin and stage dependencies, validate downloaded material with GPG/signature checks, build architecture variants and assemble release outputs. `windows/cygwin.gpg` (`1e87237a6e03f9d216427b292a124153aa050223`) and `windows/libressl.gpg` (`87d5dada4d6a1313727e0f4be89b47ea79d52403`) are binary trust-anchor resources; they were identity-verified rather than mis-decoded as UTF-8. Consuming scripts were read and show they participate in dependency signature verification.

Fabushi's canonical Build/Packaging/Release owner must preserve immutable dependency provenance, signature/checksum verification, architecture-aware co-staging and signing. Upstream trust anchors/scripts are not automatically accepted as Fabushi release policy; final evidence must come from the same accepted Fabushi release candidate.

All responsibilities remain open until implemented/verified in canonical owners. This staged dossier does not reduce `unknown` or advance the current PR accounting.
