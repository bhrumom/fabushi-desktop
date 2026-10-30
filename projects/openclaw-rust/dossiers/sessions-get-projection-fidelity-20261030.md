# sessions.get projection fidelity — 2026-10-30

`ProductionSessionWorkers` stores a legitimate outbound `send-message` entry
with `kind=send-message`, nested `message.content`, and `timestampMs`. The
Gateway `sessions.get` projection previously read only top-level `text` or
`content` and `timestamp`/`createdAt`, so this valid entry became a fabricated
user message with empty text and timestamp `0`.

The existing Gateway projection owner now recognizes the producer's nested
content and `timestampMs` fields, while preserving the established fallback
for older transcript shapes. No new transcript or message store was added.

Evidence is limited to source inspection, focused source-contract assertions,
and `git diff --check`; local OCRM tests/builds and live OpenClaw
`sessions.get` acceptance remain pending.
