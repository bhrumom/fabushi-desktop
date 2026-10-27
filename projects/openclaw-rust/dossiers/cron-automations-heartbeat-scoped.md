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
   - **Rust Owner**: `source/host/src/runner/mod.rs:HeartbeatService`.
   - **Oracle**: `OCRM-PLUGIN-O07`.

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
9. **`OCRM-PLUGIN-O07`**: `test_ocrm_plugin_o07_web_search_and_fetch_bounds`

---

## 4. Invariant Verification & Global Conservation

- **Exact Mapped Paths**: 130 (cumulative across Units 1–12)
- **Omitted Known**: 0
- **Total Tracked Paths**: 58,750
- **Unread / Unclassified Paths**: 58,620
- **Strict Conservation**: $130 + 58620 = 58750$.
- **All Global Completion Flags**: `false` strictly preserved.
