# Completion revival worker handoff

Contract: `OCRM-COMPLETION-IDLE-HANDOFF-v1`  
Exact authoring base: `c7fc0decf452f7c5a34580479bc05e2fc9d070d1`  
Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`  
Status: implementation and exact-H Actions evidence pending

## Existing owner and failure

`CompletionRevivals` is the existing owner of each agent's subagent and shell
completion queues and their respective `reviving` sets. The actual Host cloud,
shell and generated-subagent completion callbacks already enqueue and invoke
its drain methods. No additional queue, worker scheduler or recovery state is
needed.

Both current drains remove the pending vector under one lock. If it is empty,
they break, then acquire the lock again to remove the `reviving` bit. A producer
between those two locks can enqueue, observe the old bit and decline to drain;
the old worker then clears the bit and strands a nonempty queue. A successor
which has already claimed the bit must also be protected from predecessor
cleanup.

## Bounded requirements

- CH-01: under the same existing state mutex used by enqueue, taking an empty
  batch (including an explicitly stored empty vector) also releases that
  family's `reviving` bit for that agent. Taking a nonempty batch retains the
  bit and the existing vector order. Other agents and the other queue family
  are unchanged.
- CH-02: after atomic idle release the old drain returns directly and performs
  no later `reviving` removal. Enqueue before that operation is drained by the
  current worker; enqueue afterward can claim a successor. A predecessor's
  delayed return cannot clear successor ownership or the Host busy signal.
- CH-03: the public production drain delegates to the same private control
  used by the concurrency oracle. A private once-only idle-return seam may
  pause after releasing the state lock; the production wrapper passes a no-op.
  The seam must not hold the mutex while notifying or waiting. Tests exercise
  this actual control, never a copied drain loop or a source-only substitute.
- CH-04: retain existing can-execute/deletion checks, subagent pending dedupe,
  per-batch pending-wake cleanup order, prompts, quiet-origin policy, error and
  quiesce handling, and all report fields. No new thread, timer, retry, provider
  dispatch or event schema is introduced by this change.
- CH-05: deterministic channels/barriers cover both families and both enqueue
  orderings. Hold a successor's real processing callback while releasing the
  predecessor's idle-return seam; assert the successor remains busy and a
  further enqueue remains queued for that worker. Cover cross-agent and
  queue-family isolation, multiple completion order, single batch claim and
  retained delivery/error/quiesce/report assertions. Do not use sleeps as the
  oracle.

## Scope, verification and limits

The scope is `source/host/src/extensions/transcript/completion_revivals.rs`, its
existing `completion_revivals_contract.rs` tests and this narrow contract.
Required evidence is independent review of the full frozen diff and exact-H
GitHub Actions for the affected integration target, new private concurrency
filter and shared shipping Host compile. Authored tests and source inspection
are not passed validation. Existing selectors and valid assertions remain.

This is an in-process idle handoff repair. Clearing pending wakes before
execution, void marker/cleanup result propagation, the shipping background
waiter's hardcoded `quiesced_for_upgrade: false`, unknown outcomes, timeout and
restart reconciliation remain open. It is independent of the frozen checked
upgrade-marker batch and does not change its files or claim durable handoff,
original-turn restoration or full background/Automation acceptance. The
existing Session lifecycle owner and deletion fence remain authoritative.

Source evidence is the exact target owner and actual Host producer/consumer
chain above, under OCRM §17. This target race repair adds no new upstream
full-file reading or global completion claim. Rollback is the bounded patch
and requires no data migration; it restores the known idle race.
