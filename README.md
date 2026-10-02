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

## Fabushi Bot Communication Platform

The product direction is governed by [FBCP-001](docs/specs/fabushi-bot-communication-platform.md):

**Fabushi Bot is the product. Telegram is a complete communication capability source and network provider, not a separate product or workspace.**

The existing Bot/Agent architecture remains the product foundation. Telegram capabilities are absorbed into unified Fabushi domains such as Identity, Conversations, Messaging, Media, Calls, Search and Notifications, while Agent execution remains owned by Coordinator/Host/Runner, Computer, Plugins/MCP and Automations.

Start with:

- [FBCP source of truth](projects/fabushi-communication-platform/SOURCE_OF_TRUTH.md)
- [FBCP architecture map](projects/fabushi-communication-platform/architecture-map.md)
- [FBCP status](projects/fabushi-communication-platform/STATUS.md)
- [Telegram capability/provider sub-spec](docs/specs/telegram-desktop-rust-equivalence-migration.md)
- [Telegram source research project](projects/telegram-desktop-rust/SOURCE_OF_TRUTH.md)

The Telegram implementation is source-informed, not clean-room. All Telegram/desktop-app C++ production logic must ultimately be replaced by Rust owners. Other boundaries use the best-fit language through explicit architectural decisions.

All builds and tests run only in GitHub Actions or on `htch-runtime`.
