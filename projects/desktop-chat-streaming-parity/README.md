# Desktop chat streaming parity

Objective: make Fabushi desktop agent chat feel like the supplied Grok-style reference: immediate local send feedback, exactly one assistant turn, smooth streaming text, stable composer, and fast Mini App Bot appearance.

Current verified state (2026-09-20): issue reproduced from supplied screenshots/video; implementation is in progress on `fix/grok-chat-streaming-parity-20260920`.

Stage: S1 — streaming/projection repair. Next gate: regression E2E + protected-main integration.

In scope: Mahayana agent transcript projection, stream coalescing/deduplication, composer layout, account/Mini App Bot first-frame hydration, and focused regression coverage.

Non-goals: replacing the Rust Mahayana runtime, changing model quality, or copying Grok proprietary backends.

Source of truth: [SOURCE_OF_TRUTH.md](SOURCE_OF_TRUTH.md). Acceptance: [docs/19-完成定义与验收.md](docs/19-完成定义与验收.md).
