# Next cron run contract — frozen-source discovery

**Recommendation:** repair **manual-run admission/result identity in the existing Automations owner before exposing `cron.run`**. This is a small, real production behavior: an overlapping request must report `already-running`, and an admitted request must report its own persisted run, never whichever history row happens to be newest. A `cron.run` → `cron.trigger` alias would currently misstate scheduling, acceptance, authority, and waiting.

Read-only discovery; **not implemented or verified**. U: `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`. H: `bhrumom/fabushi-desktop@74f11d3d31c6245ac5886dc0181460b2d4349744`, tree `aca91e1cfbedeecb039d442fb3528caac89873a3`, branch `codex/openclaw-rust-fusion-chat-20261101-writer`. Configured model/effort inherited; assignment requested xhigh for wire, authority, execution and recovery. No settings changed.

## 1. What the real upstream caller requires

| Boundary | Fixed-U behavior | H74 difference |
| --- | --- | --- |
| Request | Closed schema accepts exactly `id` or `jobId`, optional `mode: due / force / if-enabled`, `expectedProcessInstanceId`, bounded integer `waitTimeoutMs`; default mode is `force`. | Only `cron.list` and `cron.trigger` are registered. Trigger consumes the ID but ignores mode/process/wait fields. |
| Authority | `cron.run` is an **operator.admin** control-plane mutation. Narrow exceptions require authenticated runtime identity plus a method/run-bound management grant or `operator.cron`, followed by job/account/conversation checks and lock-time rechecks. | Trigger is a mutating, operator-only method gated as `operator.write`. `GatewayCommandContext` carries authentication/tracing, not the upstream signed caller/grant contract. Reusing the trigger classification would widen upstream authority. |
| Admission | `inspectManualRunPreflight` checks stopped/disabled/due/spec/active state; `force` and `if-enabled` preserve scheduled occurrence ownership. `enqueueRun` acknowledges after the existing state-worker `cron.requestRuns` boundary, with a definite `runId`. Rejections include `already-running`, `disabled`, `not-due`, etc. | Native manual execution is synchronous. Its result is discarded. `cron.trigger` unconditionally returns `ok/enqueued=true` after a successful void call. It discovers `runId` by comparing newest history before/after. |
| Waiting/history | Server waiting is optional and bounded, avoids waiting behind the caller's own session, and reads the exact run through `cron.runs` with disclosure-time authority checks. Timeout/abort ends the wait, not the accepted run. CLI `cron run --wait` separately polls `cron.runs {id,runId,limit:1}` and exits according to the terminal result. | No registered `cron.runs` surface; no wait/process validation. Current call can wait through the real Runner terminal. The native UI instead awaits `runAgentAutomationNow` and refreshes `getAgentAutomations`. |

These are actual schema/CLI/handler/operation paths [U1–U8]. Queue transaction internals and complete grant implementation remain unread dependencies for full parity.

## 2. The narrow defect and the owning seam

The current production chain is HTTP registry/scope gate → `UnifiedGatewayApi::call_openclaw_cron_trigger` → CJI's exact Agent/local-ID resolver → `runAgentAutomationNow` → `AutomationRuntime::run_agent_automation_now_with` → `AutomationRunPath` → Session-admitted `FileAutomationStore::begin_fresh_run` → existing routed Runner → the same history settlement owner [H1–H5].

`AutomationRunPath` atomically suppresses a duplicate non-event run with `Outcome(None)` **before executor entry**. It also returns `None` for an admitted `RetainedUnsettled` execution. Its manual wrapper and the native gateway return type erase that distinction and the persisted run ID. Thus a duplicate is reported as enqueued with no receipt; a new request racing the adapter's final history read can also supply the wrong run ID. CJI intentionally left concurrent result correlation outside its identity repair.

Extend `automation_run_path.rs` and `automation_runtime.rs` to return a typed manual admission/receipt from the **actual `begin_fresh_run` return**, distinguishing duplicate/no-target/refusal and admitted outcomes. Preserve legacy background wrappers. In `main.rs`, share one private execution implementation: native `runAgentAutomationNow` maps its result to the existing `Null`/void response; the compatibility handler consumes the typed result. No new public method, registry, queue, scheduler, or history store is needed. Amend CJI's spec/production-port contract first.

**Smallest next acceptance contract:**

1. CJI's Agent/local/canonical resolution remains authoritative. A duplicate returns `{ok:true,ran:false,reason:"already-running"}` with no new run ID or executor entry. Missing/deleted/ambiguous selection still fails before dispatch.
2. Every admitted response uses exactly the ID persisted by that invocation. Settlement/error/interruption/retained state is projected only from that same receipt; a later run cannot replace it. Persistence/dispatch errors remain errors, without retry or invented success. Do not promise zero bookkeeping effects for every admission failure: the existing owner can write notices/last-run metadata before fresh-history refusal.
3. Native UI/wire behavior stays void-plus-refresh [H6]; direct synchronous execution and the current trigger compatibility response are not relabeled as an implemented durable queue.
4. Existing Runner cancellation custody remains: deadline requests cancellation then waits for a genuine terminal; uncertain external effects keep history `running`. Reopening the existing stores preserves that ID/state and does not automatically rerun or fabricate completion. Manual runs currently do not create the background-only cron-session reaper row [H3–H5].

This yields a reviewable user-visible correction and a necessary result contract for later `cron.run`. It does **not** by itself authorize registering that name.

## 3. Dependencies before the real `cron.run` batch

* **Queue and schedule ownership:** reuse `SendTurnDispatcher`/`RunScheduler` and Session/history persistence. Current queue structures are volatile; pending-wake kinds lack manual-cron requests. Establish persisted admission/launch/restart semantics. `earliest_next_run_at` derives cadence from `last_run_at`, which manual `record_run` changes; force/if-enabled preservation therefore needs an existing-store contract, not an adapter-only due check [H4,H7].
* **Authority and history:** bind the principal to existing permission/session owners at commit and disclosure. Implement admin/signed-runtime distinctions, strict input/process validation, exact `cron.runs` visibility, and wait semantics. Caller-supplied `agentId` is not authority. Grant internals need further reading [U2,U3].
* **Execution branch:** no routed provider or Cursor sends `runAgentAutomationNow` to `call_host_lane`. The complete compatibility match lacks this method; subsequent Unified/AppHost dispatch returns `unknown method`. Repair/qualify that branch before claiming all provider lanes [H8].
* **Shared CI first:** H74's `test_ocrm_cron_projection_does_not_invent_schedule_or_terminal_status` (636–667) still searches thin App methods for bodies moved into CJI helpers. This stale source anchor is a likely test flaw. Reconcile exact-H CI and shared fixes before dependent implementation [H9].

## 4. Focused Actions oracle

Extend the actual App CJI fixture plus existing runtime/run-path contracts. Use barriers to hold run A inside the real executor, trigger B, and prove B reports `already-running`, enters no executor, and creates no row. Then release A and start C before A's adapter response is projected: A must still return A's exact persisted ID. Run completed, failed, interrupted, retained-unsettled, and post-admission persistence-error controls; compare the exact selected routine/history and unrelated routine bytes. Close/reopen the real Session/store owners and prove the admitted IDs and unsettled rows remain, without implicit replay. Keep native void-plus-refresh and existing permission regression controls. A later `cron.run` batch additionally needs actual HTTP method/auth negatives, durable queue restart controls, force/if-enabled/due schedule oracles, wait-abort/no-cancel, grant revocation, and CLI-shaped `cron.runs` polling.

All executable verification remains **GitHub Actions only** on the final frozen H. No checks, parsers/generators, CI dispatch, repository edits, pushes, or ledger changes were performed in this discovery.

## Exact references and read limits

[source-reads.md](source-reads.md) binds paths, blobs, semantic read extents, callers, and limits. Full API acquisition is not counted as a complete semantic read. Root `AGENTS.md`, OCRM §17.1–17.5, SOT's current authority/constraints, M04/M19, the current task's relevant automation lineage, and the complete CJI spec governed this investigation. No inventory/read-status/global flag is advanced.
