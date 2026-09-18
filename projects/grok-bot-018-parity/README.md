# Grok Bot 0.18 parity refactor

Project ID: GBR
Owner: Fabushi desktop
Source of truth: this repository project folder + GitHub branch/CI/release evidence.

## Objective

Refactor Fabushi Desktop to reproduce the observable Grok Bot 0.18 reconstructed product surface and agent architecture as closely as the evidence-backed reference repository permits.

Product difference: Grok Bot's remote/cloud computer is replaced by the computer on which Fabushi is installed. The UI must not expose Fabushi-only Messenger/contact/Telegram/payment surfaces during this parity phase.

Reference: bhrum/grok-bot-0.18-reconstructed@107877b4e2134fd167d239411386f09e42eadd6d
Target baseline: bhrumom/fabushi-desktop@12d4aadb4a93a1413ece44aba987162e90b31229

## Delivery order

1. macOS source refactor and parity shell.
2. TypeScript agent/coordinator/local-exec parity; no CLI wrapper as the agent architecture.
3. Grok-style plugins surface and usable local plugin registry.
4. Hide non-reference Fabushi product modules.
5. Build/package in GitHub Actions.
6. Publish a macOS test artifact/release for human testing.

## Acceptance state

In progress. Completion requires merged source plus a macOS package produced by GitHub Actions. Human UX testing remains external by request.
