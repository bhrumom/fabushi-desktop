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

## Fabushi native communication capability absorption

The product direction is governed by [FBCP-001](docs/specs/fabushi-bot-communication-platform.md):

**The existing Fabushi / PR #20 architecture is the only target architecture. Telegram Desktop is a complete communication-product research source, not Fabushi's network, provider, or second product architecture.**

Telegram capabilities are decomposed and absorbed into current Fabushi owners such as the sidebar, conversation workspace, transcript, composer, Shared Room/member model, attachments/artifacts, permissions, settings, Computer, Plugins/MCP and Automations. A new owner is allowed only when no current owner is suitable and an ADR justifies the smallest possible responsibility.

Fabushi owns its own communication identity, network, synchronization, messaging, media, calls and push infrastructure. MTProto/Telegram networking may be studied for lessons but is not the product runtime.

Start with:

- [FBCP source of truth](projects/fabushi-communication-platform/SOURCE_OF_TRUTH.md)
- [FBCP absorption map](projects/fabushi-communication-platform/architecture-map.md)
- [FBCP status](projects/fabushi-communication-platform/STATUS.md)
- [Telegram research sub-spec](docs/specs/telegram-desktop-rust-equivalence-migration.md)
- [Telegram research project](projects/telegram-desktop-rust/SOURCE_OF_TRUTH.md)

This work is source-informed, not clean-room. Source/provenance and licensing review remain release gates.

All builds and tests run only in GitHub Actions or on `htch-runtime`.
