# Fabushi desktop

This repository is the independent Fabushi desktop application.

The Electron desktop source was originally extracted from `bhrumom/fabushi`, and the Rust runtime required by that desktop has now been restored into this repository so the desktop is self-contained again.

Current source boundaries:

- `desktop` — Electron desktop application
- `third_party/mahayana/mahayana-rs` — Mahayana Rust runtime and desktop host
- `third_party/mahayana/codex-rs` — Rust compatibility/runtime crates required by Mahayana
- `native/mahayana-messaging` — native messaging crate used by the Mahayana workspace
- `frontend/...`, `chatgpt-vps-control/lib/...`, and `contracts/...` — the exact shared source dependency closure imported by the desktop TypeScript/Vite code

See `MIGRATION_SOURCE.md` for the original platform extraction, `RUST_RUNTIME_SOURCE.md` for the Rust restoration provenance, and `DESKTOP_SOURCE_CLOSURE.md` for the shared source files required by the desktop build.

## Telegram Desktop Rust migration

The complete file-by-file and module-by-module Rust migration of Telegram Desktop is governed by [TDRP-001: Telegram Desktop Rust equivalence specification](docs/specs/telegram-desktop-rust-equivalence-migration.md).

Start with the [project source of truth and execution policy](projects/telegram-desktop-rust/SOURCE_OF_TRUTH.md), then the [frozen upstream lock](projects/telegram-desktop-rust/upstream.lock.json), [module map](projects/telegram-desktop-rust/module-map.md) and [current status](projects/telegram-desktop-rust/STATUS.md).

This is a specified migration project, not a claim that the current application already implements Telegram Desktop in Rust. The existing Electron/Mahayana/Grok code is not replaced by the specification commit. All builds and tests run only in GitHub Actions or on `htch-runtime`.
