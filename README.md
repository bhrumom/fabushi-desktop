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
