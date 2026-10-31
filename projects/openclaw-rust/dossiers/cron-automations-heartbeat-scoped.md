# OCRM Technical Dossier — Unit 12: Auto-Reply, Cron & Background Automations (`OCRM-UNIT-AUTOMATION-001`)

**Project ID**: OCRM-001  
**Module**: OCRM-M20/M21 (Cron Scheduling, Proactive Tasks, Heartbeat & Control Commands Subsystems)  
**Upstream Target Authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `ocrm/round20-snapshot-lifecycle-audit-20261023`  
**Reviewer**: `xuyao@ombhrum.com`  

---

## 1. Upstream Source Audit & Line-by-Line Reading

The following 6 core cron scheduling, delivery planning, session reaping, command detection, and heartbeat polling files (totaling 1,415 lines) were audited line-by-line and mapped into canonical shards:

| Shard | Source File | Blob SHA | Lines | Scope | Mapped Responsibility |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `part-0033` | `src/cron/schedule.ts` | `b0e8e7d25211b01204330abd0c5765b355c4582e` | 367 | 1..367 (full-file) | `OCRM-RESP-CRON-SCHEDULE-MATCHER-001` |
| `part-0033` | `src/cron/schedule-identity.ts` | `8c4f3361908adc5b6404ef38427ad5996fea8346` | 150 | 1..150 (full-file) | `OCRM-RESP-CRON-SCHEDULE-IDENTITY-001` |
| `part-0033` | `src/cron/delivery-plan.ts` | `67480d5da551727ca6f7e129a9d75b0e17c42635` | 236 | 1..236 (full-file) | `OCRM-RESP-CRON-DELIVERY-PLAN-001` |
| `part-0033` | `src/cron/session-reaper.ts` | `b1d615cf5b6ab55b3f58775aa4251e36eeb7c53a` | 341 | 1..341 (full-file) | `OCRM-RESP-CRON-SESSION-REAPER-001` |
| `part-0026` | `src/auto-reply/command-detection.ts` | `51733a32d2fb6524e8f71a22a80bcd1dc81f6e87` | 108 | 1..108 (full-file) | `OCRM-RESP-AUTO-REPLY-COMMAND-DETECTOR-001` |
| `part-0026` | `src/auto-reply/heartbeat.ts` | `2000cf4f116f9158a24aba99aead15376e62163e` | 207 | 1..207 (full-file) | `OCRM-RESP-AUTO-REPLY-HEARTBEAT-001` |

---

## 2. Decomposed Runtime Responsibilities

The audited source code was decomposed into 6 discrete Rust runtime responsibilities registered in `parity-ledger.json`:

1. **`OCRM-RESP-CRON-SCHEDULE-MATCHER-001`**:
   - **Behavior**: 5-field cron parsing, relative `@every` interval resolution, timezone support with `CRON_TZ=`/`TZ=` prefixes, and future execution epoch calculation (`compute_next_run_at`).
   - **Rust Owner**: `source/host/src/automations/automation_schedule.rs:compile_cron_matcher`.
   - **Oracles**: `OCRM-CRON-O01`, `OCRM-CRON-O02`, `OCRM-CRON-O03`, `OCRM-CRON-O04`.

2. **`OCRM-RESP-CRON-SCHEDULE-IDENTITY-001`**:
   - **Behavior**: Stable schedule fingerprinting, 7-day schedule summary generation, and weekend/overnight classification for proactive jobs.
   - **Rust Owner**: `source/host/src/automations/automation_schedule.rs:summarize_schedule_next_7_days`.
   - **Oracle**: `OCRM-CRON-O05`.

3. **`OCRM-RESP-CRON-DELIVERY-PLAN-001`**:
   - **Behavior**: Scheduled job delivery mode resolution (`announce` vs `webhook`), destination routing, failure alert notification policies, and target channel prefix inference.
   - **Rust Owner**: `source/host/src/connectors/channel_delivery.rs:HostChannelDelivery`.
   - **Oracle**: `OCRM-PLUGIN-O04`.

4. **`OCRM-RESP-CRON-SESSION-REAPER-001`**:
   - **Behavior**: Expired cron run-session pruning (`sessionRetention`), unreferenced transcript cleanup, and active media task preservation.
   - **Rust Owner**: `source/host/src/extensions/transcript/mod.rs:TranscriptCleanup`.
   - **Oracle**: `OCRM-PLUGIN-O05`.

5. **`OCRM-RESP-AUTO-REPLY-COMMAND-DETECTOR-001`**:
   - **Behavior**: Inbound control command detection (`/abort`, `/stop`, `/status`), session boundary command detection (`/new`, `/reset`), and bang syntax gating.
   - **Rust Owner**: `source/host/src/runner/sand_auto_review.rs:SandAutoReviewMode`.
   - **Oracle**: `OCRM-PLUGIN-O06`.

6. **`OCRM-RESP-AUTO-REPLY-HEARTBEAT-001`**:
   - **Behavior**: Periodic heartbeat poll scheduling, scratchpad effectively-empty detection, silent token reply evaluation (`HEARTBEAT_OK`), and response tool instruction injection.
   - **Declared Rust Owner Candidate**: `source/host/src/runner/mod.rs` (no `HeartbeatService` symbol is present).
   - **Production reachability**: not wired. The repository has production cron/event automation runtime paths, but no heartbeat producer, timer admission, runner consumer, or response-notification consumer for this responsibility.
   - **Oracle status**: no valid heartbeat oracle is currently registered. The historical `OCRM-PLUGIN-O07` test is a web-search/fetch payload-bounds contract and must not be used as heartbeat evidence.

---

## 3. Falsifiable Test Oracles (`source/host/tests/openclaw_plugins_automations_contract.rs`)

1. **`OCRM-CRON-O01`**: `test_ocrm_cron_o01_five_field_parsing`
2. **`OCRM-CRON-O02`**: `test_ocrm_cron_o02_every_interval_parsing`
3. **`OCRM-CRON-O03`**: `test_ocrm_cron_o03_timezone_handling`
4. **`OCRM-CRON-O04`**: `test_ocrm_cron_o04_compute_next_run_monotonicity`
5. **`OCRM-CRON-O05`**: `test_ocrm_cron_o05_seven_day_summary`
6. **`OCRM-PLUGIN-O04`**: `test_ocrm_plugin_o04_plugin_rpc_timeouts`
7. **`OCRM-PLUGIN-O05`**: `test_ocrm_plugin_o05_sand_auto_review_modes_uniform`
8. **`OCRM-PLUGIN-O06`**: `test_ocrm_plugin_o06_tool_schema_validation`
9. **`OCRM-PLUGIN-O07`**: `test_ocrm_plugin_o07_web_search_and_fetch_bounds` (web-search/fetch only; not heartbeat evidence)

---

## 4. Invariant Verification & Global Conservation

- **Exact Mapped Paths**: 130 (cumulative across Units 1–12)
- **Omitted Known**: 0
- **Total Tracked Paths**: 58,750
- **Unread / Unclassified Paths**: 58,620
- **Strict Conservation**: $130 + 58620 = 58750$.
- **All Global Completion Flags**: `false` strictly preserved.

## Current-source correction — 2026-10-31

The historical reaper mapping above is not a current Rust implementation
claim. At the historical OpenClaw discovery subject used for this correction,
`54a2fe137a70a5d2c0c0187ba27294c5ba74d3a0`, `src/cron/session-reaper.ts` was
blob `5474f33412bd9c3a808e78a42be359f976ee35b5` and directly owned retention
resolution, per-agent/store throttling, expired-row discovery, media/subagent
preservation, guarded lifecycle deletion, and transcript artifact cleanup.
The current discovery subject is the discovery-only
`upstream.lock.json::live_candidate`; the fixed implementation inventory row
for this path is separately pinned to blob
`b1d615cf5b6ab55b3f58775aa4251e36eeb7c53a`.

The current Fabushi Rust tree has no `TranscriptCleanup` symbol in
`source/host/src/extensions/transcript/mod.rs`, no per-run cron session row
model in the inspected automation run path, and no cron reaper producer. The
existing automation run records and `OCRM-PLUGIN-O05` auto-review test are not
equivalent evidence. The responsibility is therefore reset to
`implementation=not-started`, `wiring=not-wired`, `verification=not-run`, and
`acceptance=pending` pending the dedicated
`docs/specs/cron-session-reaper.md` decisions CRON-REAPER-D01 through D05.

## Current Rust heartbeat policy slice — 2026-10-31

The existing Host runner now owns a pure Rust heartbeat policy module at
`source/host/src/runner/heartbeat.rs`. It implements conservative scratchpad
emptiness detection, default/configured prompt resolution with both
`heartbeat_respond` and fallback instructions, edge-token handling for
`HEARTBEAT_OK`, exact `NO_REPLY` detection, and bounded acknowledgement
suppression. Focused in-module tests cover markdown/comment noise, prompt
instruction de-duplication, silent acknowledgements, visible message text, and
exact silent-token matching.

This slice does not create a scheduler, timer, heartbeat session, transcript
store, response tool, notification channel, or second automation loop. A
current-source search still finds no production heartbeat producer or consumer
that invokes these policies. Therefore the ledger records
`implementation=implemented` for the policy slice while retaining
`wiring=not-wired`, `verification=not-run`, and `acceptance=pending` for the
full heartbeat responsibility. No heartbeat-specific oracle has been promoted
from the unrelated historical plugin test.

## Current Automation-owner boundary recheck — 2026-11-01

The existing production Automation owner was re-read before considering a
Heartbeat binding. `ProductionAutomationsBackendRuntime` in
`source/host/src/extensions/automations/production.rs` owns one stoppable poll
worker, but its worker inputs and effects are limited to:

- listener relay polling through `BackendRelayRuntime` and `RelayEventSink`;
- automation-fire polling through `AutomationFireBackendRuntime` and
  `FireDispatch`; and
- NotifyBus wake/suspend/reconcile plus joined shutdown.

`ProductionAutomationsLifecycle` composes that backend with cloud schedule
reconciliation and listener connection watching. It does not own a heartbeat
scratch source, heartbeat session admission, `heartbeat_respond` tool result,
heartbeat transcript filtering, or a user notification consumer. Reusing its
timer would therefore lose the upstream heartbeat identity and result
semantics; adding a synthetic fire would create an unowned second contract.

The direct upstream heartbeat responsibility remains split between the
heartbeat policy/filter, the heartbeat runner scheduler/session target, and the
heartbeat response-tool/presentation path. The current Rust policy module is
only the first pure decision slice. The full row remains
`implementation=implemented` for policy only,
`wiring=not-wired`, `verification=not-run`, and `acceptance=pending`.

## Fixed-source response/result closure recheck — 2026-11-01

The fixed OpenClaw source was re-read at
`ad8d12f126d29b0db204187adf5092db80294e70` (tree
`04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`) for the response and result
owners behind `heartbeat_respond`:

- `src/auto-reply/heartbeat-tool-response.ts` owns the typed result contract:
  five outcomes, required `notify`/`summary`, optional notification, reason,
  priority, next-check, and scratch fields, scratch validation, and public-
  payload versus private-scratch separation.
- `src/auto-reply/heartbeat-filter.ts` owns transcript classification and
  removal. It recognizes internal prompts, silent acknowledgements, silent
  response-tool calls, tool-result spans, and preserves a completed visible
  response-tool call rather than filtering it as an acknowledgement.
- `src/infra/heartbeat-runner-scheduler.ts` owns per-agent cadence, flood and
  spacing guards, targeted versus enrolled wake admission, and retry/skip
  outcomes. This is a scheduler/session producer, not a reusable automation
  poll timer.
- `src/infra/heartbeat-session-publication.ts` owns admitted final publication:
  session identity and lifecycle/writer checks, idempotent transcript matching,
  media receipt matching, guarded commit, and completion publication.
- `src/infra/heartbeat-outcome-store.ts` owns durable non-notifying outcome
  persistence and one-user-run claim/context projection. The persisted outcome
  is model-only provenance and is not transcript text.

This recheck rules out promoting a standalone Rust response normalizer as
`heartbeat_respond` production equivalence. The current Rust tree still has no
heartbeat response-tool owner, heartbeat transcript filter, admitted heartbeat
session publication owner, or durable outcome store/consumer. The existing
`ProductionAutomationsBackendRuntime` remains limited to relay/fire polling and
NotifyBus lifecycle work. Reusing that poller would lose the upstream heartbeat
identity and create an unowned contract boundary.

The full responsibility therefore remains:
`implementation=implemented` for the policy slice only,
`wiring=not-wired`, `verification=not-run`, and `acceptance=pending`.

The policy slice now also includes the pure
`normalize_heartbeat_response` contract in
`source/host/src/runner/heartbeat.rs`, governed by
`projects/openclaw-rust/docs/specs/heartbeat-response-policy.md`. It validates
the fixed-source required fields and outcome enum, handles the documented
camelCase/snake_case aliases, enforces the 256 KiB scratch limit, and does not
invent native identities. Its focused tests were added but intentionally not
executed during the development freeze. This changes neither production wiring
nor full-responsibility verification/acceptance.

## Cron session-reaper production reachability recheck — 2026-11-01

The current checkout was re-read against `docs/specs/cron-session-reaper.md`.
The fixed upstream subject is `src/cron/session-reaper.ts`, blob
`b1d615cf5b6ab55b3f58775aa4251e36eeb7c53a`, at commit
`ad8d12f126d29b0db204187adf5092db80294e70` (tree
`04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`). Its production behavior is not
just age-based `runs.json` pruning: it resolves `sessionRetention` with
disabled/zero/invalid fallbacks, throttles sweeps per agent/store, discovers
expired cron session rows through the session-store worker, skips unavailable
agents, active controller work, pending generated media and unsettled
subagents, then performs an expected-row/session-id/updated-at guarded removal
and delegates transcript archival cleanup to the existing lifecycle owner.

`AutomationRun` is an embedded history record (`source/host/src/automations/automation.rs:16`),
and `FileAutomationStore` persists/truncates those records in an automation's
`runs.json` (`source/host/src/automations/automation_store.rs:25-46`). It has no
session identity, lifecycle revision, session updated-at fence, or transcript
artifact reference that can serve as the required CRON-REAPER-D01 row.

`AutomationRunPath::fire_automation_inner` records `begin_run`, executes the
provided wake callback, finishes the run, and emits an observation
(`source/host/src/extensions/transcript/automation_run_path.rs:177-290`). This
is an execution/history owner, not a per-run session producer or retention
consumer. The current source search still finds no `TranscriptCleanup` owner,
cron reaper trigger, expired per-run session discovery, active
media/subagent-preservation consumer, or guarded transcript archive/delete
chain.

A reproducible fixed-commit relative-import census rooted at
`src/cron/session-reaper.ts` reached the same broad exact-tree closure as the
session-compaction roots: 11,895 files, zero missing Git paths, four unresolved
relative edges, and 261 lexical external-module candidates. The sorted
`(path, blob)` digest is
`c10d9f6fd8dcf4b978309026fd622252884b6ed43dc136eb17292b8a24f3ed07` for the
locked commit. This demonstrates that the apparent single-file reaper subject
does not have a safely isolated source closure under the current import graph;
it does not authorize copying the TypeScript cleanup implementation or treating
the global graph as a Rust owner boundary.

The direct conclusion is unchanged but now bound to the current Rust sources:
`OCRM-RESP-CRON-SESSION-REAPER-001` remains
`implementation=not-started`, `wiring=not-wired`, `verification=not-run`, and
`acceptance=pending`. Existing automation history, session deletion, and
subagent cleanup must not be combined into a guessed cron reaper owner.
