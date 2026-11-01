# FDS-P0001-T01 — Stream coalescing and final reconciliation

Objective: one assistant body grows smoothly under token deltas and terminal payloads do not duplicate it.

Implementation: legacy `chat.delta` now appends to the live text part without sealing each token; ordinary final `chat.message` is treated as the canonical body when no ordered tool/activity boundary exists; display-equivalent whitespace/variation-selector differences do not create a second reply.

Acceptance: tiny deltas produce one streaming text part; final equivalent text is not duplicated; tool-bearing turns preserve ordering.

Status: implemented, CI pending. Branch: `fix/grok-chat-streaming-parity-20260920`.
