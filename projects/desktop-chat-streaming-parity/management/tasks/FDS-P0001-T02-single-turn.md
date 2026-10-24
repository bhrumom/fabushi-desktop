# FDS-P0001-T02 — Single provisional assistant turn

Objective: paint one assistant in-progress surface immediately on submit and adopt the Host operation id without a second normal-path thinking row.

Implementation: submit creates a provisional AssistantTurn using the request id; Host acceptance/runtime events re-key that same turn; the prior standalone thinking message is no longer emitted from the send path; late final bodies for owned operations reconcile into the existing turn.

Status: implemented, CI pending. Branch: `fix/grok-chat-streaming-parity-20260920`.
