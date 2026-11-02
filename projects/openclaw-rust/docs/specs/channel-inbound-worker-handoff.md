# Channel inbound worker handoff

Contract: `OCRM-CHANNEL-IDLE-HANDOFF-v1`  
Status: implementation in progress; exact-H verification and full ingress acceptance pending  
Original base: `81d48636b06d20f8bec551cdc334fe40f31e614b`  
Integration parent: `d94aeb372c2fe4483bcf60ebbc5aaae4e6db52ef`  
Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`

## Problem and existing owner

The shipping `wakeForInbound` gateway enqueues an envelope and claims inbound
revival under one `BackgroundWakes` mutex. The worker instead takes an empty
batch and later clears revival under another lock. An ingress between those
operations sees a worker, declines to spawn, and is left pending without one.

Keep the existing queue, mutex, gateway and worker. This is an atomic idle
handoff within one live TranscriptManager lifetime, not a new Agent loop,
scheduler, transport, transcript store or channel registration.

## Requirements and production flow

- IH-01: `BackgroundWakes::take_pending_inbound_or_end_revival(&mut self,
  agent_id)` returns `Some(nonempty_batch)` and retains inbound revival, or
  removes that agent's revival bit and returns `None`. A stored empty vector
  is empty. The operation does not change another agent or queue family.
- IH-02: the actual inbound worker calls this operation under the same mutex
  as enqueue. On `None` it returns immediately, with no later removal of the
  revival bit. A new ingress may already own that bit for a successor worker.
- IH-03: extract the existing queue-drain control into a production-used Host
  function accepting the atomic take-or-end operation, batch processor and
  nonempty-stop cleanup as callbacks for a causal concurrency oracle. The
  processor returns whether draining continues; `None` from take-or-end must
  never invoke stop cleanup. The existing nonempty pause/error exits retain
  their current cleanup. Tests pause after a real idle release before returning
  `None` to this same drain control, admit a successor, then resume the old
  control and assert the successor still owns revival. No
  new background thread, timer, retry policy or state is added.
- IH-04: enqueue-before-idle is consumed by the current worker; idle-before-
  enqueue admits a successor. After idle release, a predecessor's return
  cannot clear successor ownership. Batch order and the existing single-worker
  coalescing decision are preserved.
- IH-05: all Session admission, append, provider selection, Runner dispatch,
  delivery, activity and DM-preemption behavior remains with current owners.

## Verification and limits

Actions tests must execute the real queue owner and production-used drain
control with deterministic channels/barriers: both mutex orderings, successor
admission while the predecessor is returning, exactly one batch claim, empty
vector handling, and cross-agent/queue-family isolation. A fake processing
callback is allowed after a real claim; copying the drain loop into a test or
checking source text alone is not the concurrency oracle. Shipping Host
compilation and existing background-wake/manager contracts remain selected.

The current append error drops the claimed batch; its String error cannot
distinguish known rollback from unknown commit. Deferred/provider/quiesce and
spawn-failure batches have no autonomous inbound rearm. Detached workers also
lack a dispose/recreate generation fence. These remain open production work;
this batch must not add blind replay or claim persistence, transport ACK,
restart, stop/recreate, real-channel or full ingress-to-delivery acceptance.

No data or protocol migration is needed. Rollback restores the prior drain
control and does not remove user state. Existing telemetry remains unchanged.
Source review uses the fixed-U ingress monitor's coalesced wake/final-repump
invariant; exact identities and read limits belong to the existing channel
dossier. The pure debounce contract and its unwired status are not promoted.
