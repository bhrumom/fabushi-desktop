# Strict126 complete read: snap/snapcraft.yaml

Source: `telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf` / tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`
Path/blob: `snap/snapcraft.yaml@a181d828f25d901303f7221824ac30b0ff5e773a` (16,501 bytes)
Read status: complete
Parent Fabushi head/tree: `a4844465a441db4f9f53335a556b3b5b87805111` / `2ad87576dc94abdb6f80bd776c8e22dd760453ab`
Live Fabushi main: `7394f6dc7687f9b986ee5589d40aa680ce23b90a`

## Responsibility decomposition

- package/build identity: `name: telegram-desktop`, `base: core24`, `grade: stable`, `confinement: strict`, command/common-id/desktop/autostart metadata, icon and GNOME extension are upstream Linux package provenance only;
- installed activation/layout: `command: usr/bin/telegram-desktop`, desktop/autostart entries, `/var/tmp` tmpfs layout and install rearrangement express source-neutral install/launch responsibilities;
- confinement/interface declarations: audio-playback, audio-record, bluez, camera, hardware-observe, home, login-session-observe, network, network-bind, network-status, removable-media, u2f-devices and MPRIS are OS package permissions, not Fabushi account/tool authorization;
- release/version provenance: package version derivation and artifact identity remain governed by Fabushi release/evidence owners;
- external acquisitions: patches, Ada, libavif, FFmpeg, zlib/minizip, mozjpeg, OpenAL, OpenSSL, nv-codec-headers, Qt, RNNoise, TDLib, Rust, tlottie, wallet-engine and tg_owt are build inputs already covered by the build-time acquisition inventory;
- Telegram identity, Qt/C++ runtime, TDLib, TON/WalletKit and Telegram WebRTC are not promoted to Fabushi runtime owners.

## Existing-owner-first audit

- `desktop/package.json@3d500622ce9b26e3452d98d0c4a7e7dcc019ac84`: canonical package identity; Linux targets are AppImage/deb, not Snap.
- `source/electron-main/main.ts@b8eeed322ec07e5c8f67fa367cd2f58ac212b4f0`: canonical process and single-instance lifecycle.
- `projects/fabushi-communication-platform/quality/release-entry-exit-criteria.md@c5eb9d0897234bb2fdad4f505549b2350c44aa72`: FQT-REL-001 release/artifact identity gate.
- `projects/fabushi-communication-platform/quality/evidence-contract.md@2d1d709a6ac3204bd05bedf5b3e98f37856e8c70`: FQT-EVID-001 exact evidence rules.
- `projects/telegram-desktop-rust/inventory/build-time-acquisitions.json@cfa989ef17a7bf60164b43417f54cd0b45a2f30e`: existing acquisition authority for exact/mutable build inputs.
- `projects/telegram-desktop-rust/dossiers/snap-packaged-ci-supply-chain-workflow-complete-read.md@d81340074258a8e2f5d06847eeb27da746c54bd8`: strict92 already established that upstream snap.yml consumes this recipe directly and kept .snap release evidence fail-closed.

No evidence justifies a Telegram/Snap runtime, second package manifest, second permission store, second acquisition ledger, second process root, or second release authority.

## Evidence boundary

Current Fabushi has no canonical Snap target/workflow. Existing macOS sign/notarize/packaged-acceptance evidence cannot prove Linux/Snap behavior. AppImage/deb declarations cannot prove Snap confinement, interface grants, autostart/desktop install content, raw .snap identity, signing, or store promotion.

## Traceability

Requirement: `TDRP-R9-UPSTREAM-SNAPCRAFT-PACKAGING-SUPPLY-CHAIN-PROVENANCE-001`

Oracles: `ORA-TDRP-SNAPCRAFT-EXACT-SOURCE-001`, `ORA-TDRP-SNAPCRAFT-PACKAGE-OWNER-001`, `ORA-TDRP-SNAPCRAFT-ACQUISITION-OWNER-001`, `ORA-TDRP-SNAPCRAFT-CONFINEMENT-EVIDENCE-BOUNDARY-001`, `ORA-TDRP-SNAPCRAFT-ARTIFACT-IDENTITY-001`

Invariants: `INV-TDRP-SNAPCRAFT-NO-TELEGRAM-RUNTIME-001`, `INV-TDRP-SNAPCRAFT-NO-SECOND-ACQUISITION-LEDGER-001`, `INV-TDRP-SNAPCRAFT-NO-CROSS-PLATFORM-ENTITLEMENT-PROOF-001`, `INV-TDRP-SNAPCRAFT-INSTALLED-EVIDENCE-REQUIRED-001`, `INV-TDRP-SNAPCRAFT-EXACT-HEAD-ACTIONS-ONLY-001`

## Parent exact-head evidence

- TDRP run `39966961007`: 61/61 success; authority `12507945583` / `sha256:ea43c4eb496ecb4701b496e02bdfb99cf92066a9d0d66d71c0c5faa552227d7c`; closure `12508450146` / `sha256:e5ef3aab8e7ee240d5a51e8db7a67f50f4f20f307f7dcd2a61f34d419c53f266`; reachability `12506918599` / `sha256:508587848be66d2aa705dad2d1f8e94e9763927051f11869960b1f34fe7325d2`.
- Rust desktop runtime `39966961005`: 6/6 success; artifact `12507257039` / `sha256:a178252b198db13577513f832ade38a0cb34f7c52028dd087621d46b8258dab0`.
- Pre-publication PR/ref/tree, live main, and upstream lease were stable.

## Disposition

Strict126 is mapped-open, not accepted. No Fabushi Snap implementation, signed .snap, installed confinement/interface acceptance, source closure, or independent release acceptance is claimed.

Coverage after this entry: `unknown=16,250`, `unread=16,225`, `omitted=0`, `responsibilities=144`.

Next deterministic entry: `tasks/2026/07/24/generate-walletkit-golden-vectors.md@863a1733649dcb8371091d750f6173bb8550a839` (1,296 bytes).
