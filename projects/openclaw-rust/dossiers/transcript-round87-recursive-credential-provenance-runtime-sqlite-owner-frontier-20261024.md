

## Section 106 — outbound pending-final completion and restart settlement still do not bridge public Cron identity into queue custody

Frozen source remains `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`). Live target lease was re-read immediately before this append as `9d6dd937fe4924bc18aba1dbfa504d5172b97f84` with tree `04d7fc58ca9b164a14ce7169d0f4f46b0866b422`; the dossier blob was `1596094344f84bbace675a1ad0841413bc75ea6c`. This pass follows Section 105 into the concrete outbound pending-final completion path, outbound restart recovery, and the already-registered session-delivery settlement callback. Only frozen refs with zero exact occurrences in the full dossier before this append are registered.

New frozen identities:
- `src/infra/outbound/delivery-completion.ts@1148d4e41215bef0f0e8ad5d3c65b9477d06d12c`
- `src/channels/message/send.ts@a0a6c025568d9f0418a497e2b6a995c40484415f`
- `src/infra/outbound/delivery-queue-recovery.ts@5ac0792782f8545a4ba11a7b2dfd04f5f347be52`
- `src/infra/outbound/delivery-queue-types.ts@a649f13a9ec832a369d9295b865df1f7ad200005`
- `src/auto-reply/reply/pending-final-delivery.ts@a38efa192d2202dcbde8a2aa0d268a3483c2b8a1`
- `src/config/sessions/session-pending-final-settlement.ts@5d98ee96a9cf67adaefc46a8e93aed3e35dbb826`

### 106.1 Message send promotes only the pending-final owner tuple, not a public Cron run identity

- `pending-final-delivery.ts@a38efa…` reads reply metadata and constructs a `DurableDeliveryCompletion` with `kind: "pending-final"`, the persisted completion fields, and optional session-writer delivery authority. It does not synthesize or look up a Cron receipt/history id.
- `delivery-queue-types.ts@a649f1…` makes the durable shape explicit: pending-final completion contains only command-owner recovery reference, optional agent id, delivery id, intent id, session id, session key, store path, and optional session-writer authority. There is no `requestRunId`, `childSessionKey`, or `child_session_key` field.
- `send.ts@a0a6c0…` resolves that completion from payload metadata when no explicit completion is supplied and passes it to the outbound sender as `deliveryCompletion` with `durability: "required"`. The send path therefore preserves the existing pending-final owner identity; it does not import public Cron receipt/history identity.

### 106.2 Outbound restart recovery settles the durable owner before ACK without adding a public-id cross-index

- `delivery-completion.ts@1148d4…` settles pending-final state through the session actor using `sessionId/intentId/deliveryId`, optional cleanup `sessionKey/recoveryRunId`, and live session-writer/harness authority. Its completion and settlement contracts contain no public Cron request id or task-history child-session field.
- `session-pending-final-settlement.ts@5d98ee…` projects the durable delivery state and, when a verified final receipt exists, records terminal evidence under the pending final's private `sourceRunId`. Cleanup is a separate `pending-final-clear` patch keyed by session/intent plus optional recovery run id. Neither path joins to `cron_run_receipts.request_run_id` or `task_runs.child_session_key`.
- `delivery-queue-recovery.ts@5ac079…` proves restart ordering at the outbound queue boundary. Failed custody first stages the durable failure and idempotently projects `deliveryCompletion` before finalizing the queue failure. Completed-owner recovery first re-projects the owner as queued; already terminal owner state is ACKed without replay. Unknown-after-send reconciliation projects the reconciled result before ACK. A successful replay also calls `settleDurableDelivery` before post-send ACK/finalization. None of those branches reads or writes the public Cron request/history identity.
- The already-registered session queue remains a different durable layer. `session-delivery-queue.records.ts@5d7df3…` carries queue `id`, `sessionKey`, retry/settlement metadata, and an optional subagent owner `runId`; it has no `requestRunId` or `child_session_key` field. `session-delivery-queue-recovery.ts@bd673a…` persists the settlement marker, runs `onSettled`, and only then physically finalizes the row.

### 106.3 The specified restart-dispatch -> session-settlement -> Cron-cleanup chain remains identity-separated

- The already-registered `main-session-restart-dispatch.ts@b762ce…` preserves `restartRecoveryDeliveryRunId` and `restartRecoveryDeliverySourceRunId` as separate private restart identities and invokes its dispatch `onSettled` only after that admission remains current. It does not acquire a public Cron request/history id.
- The already-registered `server-restart-sentinel.ts@9b4faa…` defines `settleQueuedSessionDelivery(entry, outcome, queueContext)` as `settleCorrelatedSubagentDelivery(...)` followed by `removeCronRunContinuationSessionIfIdle(entry.sessionKey, entry.id, queueContext)`. The cleanup correlation is the session-delivery queue id and session key, not a public Cron `requestRunId`.
- The already-registered `server-restart-sentinel.test.ts@35e676…` focused oracle proves correlated-subagent settlement runs before Cron continuation cleanup for a synthetic queue entry. It does not bind that queue entry to a real public receipt/history run id and does not follow cleanup with public `cron.runs { id, runId, limit: 1 }` retrieval. The requested one-public-id integrated fixture therefore remains absent.

### 106.4 Stop and plugin-native focused frontiers remain unchanged

- Code-search intersection for `sessions.processes.stop` with placement generation/owner-epoch leaves only production `session-processes.ts@b1455a…` and `session-processes.test.ts@4d65f9…`. Production re-runs `placement.assertCurrent()` after remote I/O before publication, but the direct stale-response test still dispatches `sessions.processes.list`; its Stop coverage is ordinary authorization with no pending remote Stop, placement replacement, successful stale reply, and rejection assertion. No direct replacement-late-success Stop Gateway oracle was found.
- Frozen plugin-native enumeration still has exactly two production plural callers of `settlePluginNativeAdmissions`: `plugin-generation-preparation.worker.ts@220559…` and `prepared-model-runtime.build.ts@0fec2f…`. `plugin-generation-worker.ts@37e1c1…` calls only singular `settlePluginNativeAdmissionState`; transfer adoption carries retry facts back to the same admission owner. False/throw publication remains retained in that admission state's `publications` map and retried through the same state/key. No distinct failed-publication receiver and no third production plural caller was found.

### 106.5 Dedupe, lease, and remaining acceptance state

Immediately before this append, live lease was `9d6dd937fe4924bc18aba1dbfa504d5172b97f84`, tree `04d7fc58ca9b164a14ce7169d0f4f46b0866b422`, dossier blob `1596094344f84bbace675a1ad0841413bc75ea6c`, 6680 lines, Section 106 count zero, and all six new exact refs above had zero occurrences. Frozen upstream commit/tree were re-read. No Actions were started or rerun; no production code, ledger, RTM, flags, source lock, inventory, or source baseline changed.

- The predecessor public-Cron id -> pending-final/outbound recovery -> restart/lost-reply session-delivery settlement -> exact alias cleanup -> public `cron.runs { id, runId, limit: 1 }` terminal-read fixture remains absent. The direct Stop replacement-late-success Gateway oracle and plugin-native distinct failed-publication receiver/third plural caller also remain absent. Independent ACCEPT, scoped G01/G02, Rust implementation/wiring parity, package/device proof, release acceptance, and final OCRM acceptance remain open.
