# Source of truth

Authoritative implementation repository: `bhrumom/fabushi-desktop`, protected `main`. Authoritative project path: `projects/desktop-chat-streaming-parity`.

Precedence: latest persisted user requirements -> supplied screenshots/video -> this project's normalized requirements/ADRs -> live GitHub code/PR/CI facts -> external reference implementation -> chat memory.

Implementation claims must be verified from live GitHub/CI/release state. The external `RongleCat/grok-app` repository is a design reference only, not authority for Fabushi behavior.

Conflicts are recorded in management history rather than silently rewriting prior facts.

## 2026-09-22 Grok 0.18 rebuild supersession

For the active Grok Bot 0.18 architecture-equivalent rebuild, `docs/specs/grok-bot-018-runtime-product-parity-recovery.md` is the task-specific authority and supersedes any conflicting requirement in this older project.

In particular, this project's historical requirement to paint a provisional assistant/thinking surface before Host acceptance is **not** applicable to the active rebuild. `CHAT-001` in the active spec requires assistant thinking/running state to begin only after canonical Host/runtime acceptance. The optimistic user bubble may remain local-first.

The older renderer-coalescing work remains useful as historical evidence, but its architecture and completion labels must not be used to preserve `desktop/src`, `desktop/electron`, renderer-owned operation adoption, or another legacy runtime that the active rebuild requires to remove.
