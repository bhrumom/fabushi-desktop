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

## Telegram Desktop source-informed rearchitecture

TDRP-001 now uses a source-informed rearchitecture model: study the frozen Telegram Desktop source deeply, then redesign the target architecture around capabilities and behavior instead of translating files or mirroring modules.

All Telegram/desktop-app C++ production logic must ultimately be replaced by Rust production owners. Non-C++ areas use the best-fit architecture and language for their boundary, justified by ADRs rather than a blanket language rule.

Start with the [main specification](docs/specs/telegram-desktop-rust-equivalence-migration.md), [project source of truth](projects/telegram-desktop-rust/SOURCE_OF_TRUTH.md), [frozen upstream lock](projects/telegram-desktop-rust/upstream.lock.json), [source research/capability map](projects/telegram-desktop-rust/module-map.md) and [current status](projects/telegram-desktop-rust/STATUS.md).

This is source-informed, not clean-room, and it is not a claim that the current application already implements the target architecture. All builds and tests run only in GitHub Actions or on `htch-runtime`.
