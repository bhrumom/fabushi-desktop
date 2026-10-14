# Nimf preedit / Wayland / XIM + range-v3 build provenance — orders 10,797–10,860

## Evidence

- accepted upstream: `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941`
- reviewed exact Fabushi head: `ab7fc412ff80f5858378e781c5b7039d22bb976e`
- Rust desktop runtime: `38134640557`
- source-authority job: `114455966489`
- artifact: `11700566936`
- artifact digest: `sha256:c474a4a2ea18b54068f030e52a3b40ebf765f942bc70cb98d384d627ef17ef45`
- independent raw-byte verification: 64/64 repository/commit/path/blob identities, Git blob SHAs, SHA-256 values and byte lengths matched.

## Responsibility decomposition

### 10,797–10,798 — preedit-window
Start/stop/finalize, preedit text/cursor projection and cursor-location/monitor-bounded placement are product interaction semantics. They remain owned by canonical Composer/input-composition/platform-input; GTK popup implementation is not imported.

### 10,799–10,803 — Wayland
The source carries fail-closed display prepare/read/flush/dispatch, activation/deactivation and context lifecycle, serial ordering, keymap/XKB modifiers, unconsumed-key forwarding, surrounding-text/reset and preedit/commit semantics. Existing Platform Input, PromptRichTextEditor, composition fencing and Electron recovery/security owners remain canonical.

### 10,804–10,824 — XIM/IMdkit
The source carries binary framing/byte-order/length/padding checks, IM/IC connect/open/close/create/destroy lifecycle, attributes, focus/unfocus, event forwarding, sync, commit/reset, preedit/status callbacks and disconnect cleanup. These are source-neutral IPC/input/recovery/security responsibilities, not grounds for a new XIM runtime.

### 10,825–10,830 — package, distro patches, historical Qt5 binary
`nimf.spec` includes a mutable `Source0 .../archive/master.tar.gz`; it is historical provenance and must remain fail-closed. The two patch files change libnimf, Qt clients, engines, indicator/config and input-method behavior and therefore are behavior-delta evidence, not filename-only N/A. The bundled `libqt5im-nimf.so` is a historical HamoniKR Qt5 plugin: 39,848 bytes; Git blob `250bb0e74acfb123a4f2f441b6bcdc1f579765cf`; SHA-256 `e4b22ebc4af772f91f5385ddaf0ef9b00ff6f2898810c94e8f3bcc9202284c1f`. It was verified as raw ELF bytes from the current-head Actions artifact and is never a Fabushi production dependency.

### 10,831–10,835 — localization
LINGUAS/POTFILES/gettext catalogues map to the existing Localization/Settings owner.

### 10,836–10,860 — range-v3 build/CI/config
The accepted component is pinned at `ericniebler/range-v3@a81477931a8aa2ad025c6bda0609f38e09e4d7ec`. Build/CI/package/license/config and compiler/header probes map to Build/Quality/Dependency-Provenance/License. The `doc/gh-pages` submodule has a mutable branch hint in `.gitmodules`; accepted TDesktop CI describes it as documentation-only and recursive authority still binds exact Git objects.

`cmake/GoogleBenchmark.cmake.in` and `cmake/GoogleTest.cmake.in` each use `GIT_TAG master`. `cmake/gbenchmark.cmake` configures/builds the corresponding external project at configure time; order 10,861 `cmake/gtest.cmake` does the same for GoogleTest. These are explicit floating acquisition risks. Fabushi must use immutable commit/digest pins and must not reproduce those mutable acquisitions.

## Canonical-owner decision

No exact-head owner absence was proven. Keep existing:
- PromptRichTextEditor / input-composition-state / Composer
- source/electron-main/platform-input.ts
- canonical IPC, Electron lifecycle/recovery/security
- Settings / Localization
- Build / Quality / Dependency Provenance / Packaging / License

Do not add a Nimf daemon/runtime, Wayland/XIM application runtime, second input store, historical Qt plugin dependency, or range-v3 test dependency.

## Accounting

All 64 rows are `mapped-open`, `unknown_closed=false`, `omitted=false`.
Read-through: 10,860 / 16,125.
Unread: 5,265.
Unknown: 15,845.
Omitted: 0.
First unread: 10,861 `Telegram/ThirdParty/range-v3::cmake/gtest.cmake@a989aec90627c8db83d71012a749e26e64f8e5e0`.
