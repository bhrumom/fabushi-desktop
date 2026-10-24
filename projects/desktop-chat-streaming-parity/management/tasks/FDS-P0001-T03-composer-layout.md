# FDS-P0001-T03 — Stable composer layout

Objective: keep the Bot transcript from consuming 100% height in addition to its sibling composer.

Implementation: BotConversationView is now a flex child with `flex: 1 1 0`, `height:auto`, bounded overflow, and transcript-local scrolling.

Acceptance: composer remains visible and usable during long/streaming transcript.

Status: implemented, CI pending. Branch: `fix/grok-chat-streaming-parity-20260920`.
