# Strict98 WinGet publication preflight — upstream drift gate

## Exact parent evidence
- Fabushi PR #154 parent exact HEAD: `94de4915c1828a5abc651b4ba282af421d6e23e1`, tree `8c9217e53c6f2c69fce99278d7706558c2ff7185`.
- TDRP run `39898075641` completed success on that exact HEAD. Required artifacts:
  - authority `12477538096`, `sha256:957966daab1d7907d0cabb9e3410b3d1f8f38f0eb38f4b30f8151f82480626eb`
  - closure `12477394597`, `sha256:0a08735564b448a06dd49f6dfcbbd3829ddc28ddb987731362d7c08398bab167`
  - reachability `12477249740`, `sha256:a2864abe1e253b5083325de4d8ac3a58c6c4f54e26a071830e92aac8b6ac3df3`
- Rust desktop runtime run `39898075663` completed 6/6 success on the same exact HEAD; artifact `12477162917`, `sha256:2bd9915e9f3fdf6480f88ef7f5b8df95609c0d9c9897e845616bcba56a15399a`.

## Strict98 source audit completed but not admitted
- Frozen strict98 source remains `.github/workflows/winget.yml@0ff2d50d8b12b67408d3ad1e06dc8e462b5a9390` (700 bytes).
- It runs on release `released` / `prereleased` events and calls mutable `telegramdesktop/winget-releaser@main` with package identifiers `Telegram.TelegramDesktop` and `Telegram.TelegramDesktop.Beta`, installer regex `t(setup|portable).*(exe|zip)$`, and `WINGET_TOKEN`.
- Audit-time provider resolution: `telegramdesktop/winget-releaser@main -> 4ffc7888bffd451b357355dc214d43bb9f23917e`, tree `0b09c320f951cb196fa758241df02baa6f893f72`; `action.yml@a6de3aab26491d01a7a459c3fcaca8b6d5bce393`; `LICENSE.md@74c892aed62da338fe12600bfba912596f044ced` is AGPL-3.0.
- The provider action itself uses mutable `cargo-bins/cargo-binstall@main`; audit-time resolution `8a2045612414e945291083f820cd50b4d8fc2218`, tree `f25f032637b6d602a60199c7893f4441f1bdc210`, `action.yml@bfe4fcd6726e2a272313ea120c3492fa1d979e0e`; repository license metadata is GPL-3.0.
- The action then executes `cargo binstall komac -y` without a Komac version pin. Audit-time Komac main was `e9882dcfd22d9bb2defbb0e25044c331d5e4390d`, tree `35a45b7bef8d48fd5ba0723ba7d26cec385dd832`, GPL-3.0 metadata; this is observation only and not an immutable dependency pin.
- Publication semantics are external-catalog mutation: resolve release assets, generate/update WinGet manifests, submit PRs through the provided token, clean merged branches, and optionally remove old versions. These effects remain under existing FQT release/artifact provenance and credential/security boundaries; upstream publication success cannot establish Fabushi signing, package identity, install/device acceptance, promotion, or independent release ACCEPT.
- Existing-owner preflight found no prior parity-ledger row for this exact source path or proposed strict98 responsibility. Existing stable owners remain `FQT-EVID-001` (`evidence-contract.md@2d1d709a6ac3204bd05bedf5b3e98f37856e8c70`) and `FQT-REL-001` (`release-entry-exit-criteria.md@c5eb9d0897234bb2fdad4f505549b2350c44aa72`). No new WinGet/Telegram release owner is justified.

## Blocking lease drift
Immediately before any strict98 ledger/index/lock write, live `telegramdesktop/tdesktop dev` advanced from the accepted authority `83f9f56347cc3421c1e3ef537c7bb6dd06ce7cd0/bb0159bfedb38dd96e579af4ffe6360b6beb7332` to `a7f453612f4896ac04d0007a36ab0d377e5bcbcf/c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.

The compare is linear (`ahead_by=2`, merge base is the old accepted commit). It modifies `Telegram/CMakeLists.txt`, four existing history list/widget files and the `Telegram/lib_ui` gitlink, and adds:
- `Telegram/SourceFiles/history/view/history_view_touch_scroll.cpp@3a15e4202700bb25e563dd2d79268a4c52c99ed7`
- `Telegram/SourceFiles/history/view/history_view_touch_scroll.h@2ec8ff7f26c40c276679815eaba1c104d9121918`

The strict98 `winget.yml` blob itself remains unchanged at `0ff2d50d8b12b67408d3ad1e06dc8e462b5a9390`, but the upstream baseline lease is not stable. Therefore this preflight does **not** advance unknown/unread counters, does **not** append the strict98 parity row, and does **not** modify inventory/index/lock. The next admissible action is exact upstream rebaseline and fresh exact-head evidence, followed by revalidation of this unchanged strict98 source and provider identities.