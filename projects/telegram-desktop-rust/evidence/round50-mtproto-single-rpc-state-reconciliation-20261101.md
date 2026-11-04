# Round 50 — MTProto single-RPC state reconciliation evidence

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 19
Stage A: implementation/contracts authored; executable validation not run.

Source: fixed U `f985723b552b30fe1c17d377d57f44e61a15a8c4`, `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`, `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`.
Fixed-U constructors: `msgs_state_req#da69fb52`, `msgs_state_info#04deb57d`, `msgs_all_info#8cc0d131`, `msg_new_detailed_info#809db6df`, `msg_resend_req#7d861a08`.

Canonical spec: `469e46f72d15138aa4fb343a49e06feeaefb1303`.
Canonical implementation: `404623cad89e0d77ab326e140784b893d677f31e`.
Canonical blobs: api `ed695ed575c5950c99818acc7086a2cf8b1c712d`, lib `21ec771260dd356d4a8e653be6e2e250a32ab5ce`, network `c24515c2ee15608f51970a5420769e7713e60327`.
Desktop synchronization: `fe17d4bcdea22d1df8e474961aa12b5f04b291db` with the same three blobs.

The one existing `invoke_raw` outstanding request remains the only request history. After the normal receive window, the same session may send one non-content `msgs_state_req([current_request_message_id])`. A matching one-byte `msgs_state_info`, or `msgs_all_info` that explicitly names the current request, applies the fixed-U low-three-bit rule: `0x04` keeps waiting; other states request one replay through the existing outer retry budget. A repeated state-driven replay signal fails closed with `StateReconciliationLimit`.

`msg_new_detailed_info` does not create another received-message store: the existing `MessageIdGuard` replay window decides whether `answer_msg_id` is already seen. Seen answers are acknowledged; unseen answers are requested through `msg_resend_req` on the same auth key/session/transport.

Authored-only contracts: protocol fixed-U state builder/parser contracts and network state/reconciliation predicate contracts. They were not run.

Open from the same fixed-U source: arbitrary multi-request `haveSent/_resendingIds/_stateAndResendRequests` reconciliation, simultaneous state queries, cancellation of queued resends, container-wide resend bookkeeping, durable sent-request history, higher-level new-session update/difference propagation, broader source/legal closure, Stage-B executable evidence, and independent release ACCEPT.

Status: implemented / validation_status=not-run / accepted=false.
