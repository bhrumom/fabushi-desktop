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

## Product and roadmap

- [Product definition](PRODUCT.md)
- [Roadmap](ROADMAP.md)

## Project governance

Repository work is governed by durable project records rather than chat history:

- [Documentation map](docs/README.md)
- [Architecture](docs/architecture/README.md)
- [Architecture invariants](docs/architecture/invariants.md)
- [Architecture decisions (ADR)](docs/adr/README.md)
- [Development specifications](docs/specs/)
- [Migration records](docs/migrations/README.md)
- [Testing strategy](docs/testing/strategy.md)
- [Release and evidence policy](docs/operations/release-and-evidence.md)
- [Incident response](docs/operations/incident-response.md)
- [Contributing](CONTRIBUTING.md)
- [Security reporting](SECURITY.md)
- [Changelog](CHANGELOG.md)
- [AI agent execution policy](AGENTS.md)

For AI-assisted development, `AGENTS.md` is mandatory and enforces the standard lifecycle:

**Discover → Classify → Spec → Architecture/ADR → Migration → Plan → Implement → Verify → Compliance Review → PR/Merge → Release → Post-release verification**
