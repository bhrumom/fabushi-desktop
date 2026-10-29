# OpenClaw Host Infrastructure, Worker Task Pool & Platform Sandbox Parity Dossier

## 1. Overview & Responsibility Mapping
- **Module ID**: `OCRM-RESP-INFRA-UPDATE-STARTUP-001`, `OCRM-RESP-INFRA-PLATFORM-SANDBOX-001`, `OCRM-RESP-INFRA-WORKER-POOL-001`
- **Scope**: 131 upstream files in `src/infra/` within `part-0042` inventory shard.
- **Owners**:
  - `mahayana_host_runtime::extensions::host_upgrade::openclaw_update_startup`
  - `mahayana_host_runtime::extensions::local_exec::openclaw_platform_sandbox`
  - `mahayana_host_runtime::extensions::box_lifecycle::openclaw_worker_pool`

## 2. Audited Files & Upstream Anchors
- **Startup Updates & Triage** (22 files):
  - `src/infra/update-startup.ts`, `src/infra/update-startup.test.ts`, `src/infra/update-startup.test-support.ts`
  - `src/infra/update-startup-campaign-run.test.ts`, `src/infra/update-startup-git.test-support.ts`
  - `src/infra/update-startup-host-owned.test.ts`, `src/infra/update-startup-immutable.test.ts`
  - `src/infra/update-startup-refresh.ts`, `src/infra/update-startup-refresh.test.ts`
  - `src/infra/update-startup-request-adoption.test.ts`, `src/infra/update-startup-suite.test-support.ts`
  - `src/infra/update-startup.campaign-apply-error.test.ts`
  - `src/infra/update-status-schedule.ts`, `src/infra/update-status-state.ts`
  - `src/infra/update-step-identity.ts`, `src/infra/update-step-result.ts`
  - `src/infra/update-system-package-ownership.ts`, `src/infra/update-timeout-provenance.ts`
  - `src/infra/update-triage.ts`, `src/infra/update-triage.test.ts`
  - `src/infra/update-triage-resolution.ts`, `src/infra/update-triage-resolution.test.ts`
- **Platform Sandbox, Windows Integration & Codecs** (46 files):
  - `src/infra/windows-appcontainer-spawn.ts`, `src/infra/windows-appcontainer-spawn.test.ts`
  - `src/infra/windows-powershell-spawn.ts`, `src/infra/windows-shell-command.ts`, `src/infra/windows-shell-command.test.ts`
  - `src/infra/windows-encoding.ts`, `src/infra/windows-encoding.test.ts`
  - `src/infra/windows-diagnostic-env.test.ts`, `src/infra/windows-gateway-firewall-diagnostics.ts`
  - `src/infra/windows-install-roots.ts`, `src/infra/windows-launcher-encoding.ts`
  - `src/infra/windows-port-pids.ts`, `src/infra/windows-private-directory.ts`
  - `src/infra/windows-process-census.ts`, `src/infra/windows-process-start.ts`
  - `src/infra/windows-task-restart.ts`, `src/infra/windows-user-sid.ts`
  - `src/infra/wsl.ts`, `src/infra/wsl.test.ts`
  - `src/infra/widearea-dns.boundary.test.ts`, `src/infra/widearea-dns.ts`
  - `src/infra/voicewake-routing.ts`, `src/infra/voicewake-routing.test.ts`
  - `src/infra/wake-concurrency.test-helpers.ts`, `src/infra/warning-filter.test.ts`
  - `src/infra/watch-node-observation.test.ts`, `src/infra/ws.test.ts`
  - `src/infra/zstd-codec.ts`, `src/infra/utf8-input.ts`
  - And all related vitest configuration and platform fixture files.
- **Worker Task Pool, Sizing & Memory Bounds** (63 files):
  - `src/infra/worker-task-pool.ts`, `src/infra/worker-task-pool.test.ts`, `src/infra/worker-task-pool.types.ts`
  - `src/infra/worker-pool-sizing.ts`, `src/infra/worker-task-capacity.ts`
  - `src/infra/worker-task-host.ts`, `src/infra/worker-task-server.ts`
  - `src/infra/worker-task-transport.ts`, `src/infra/worker-cpu.ts`
  - `src/infra/worker-idle-gc.ts`, `src/infra/worker-memory.ts`
  - `src/infra/worker-native-lifecycle.ts`, `src/infra/worker-transfer-bytes.ts`
  - And all 50 worker lifecycle, memory retention, SQLite idle, and native Section test suites.

## 3. Rust Native Equivalence Architecture
1. **Startup Updates & Supervisor Policies** (`openclaw_update_startup.rs`):
   - Update interval thresholding with deterministic due checks.
   - Update triage error classifier producing tailored recovery actions based on external supervision presence.
2. **Platform Sandboxing & Encoding** (`openclaw_platform_sandbox.rs`):
   - Windows AppContainer isolation configuration with network capability separation.
   - Robust argument escaping for PowerShell and Windows shells.
   - Zstd magic byte header validation (`0x28, 0xB5, 0x2F, 0xFD`).
   - WSL runtime environment detection.
3. **Worker Task Pool & Sizing** (`openclaw_worker_pool.rs`):
   - Configurable min/max worker instances and memory quotas.
   - Dynamic task assignment and completion tracking.
   - Idle worker sweeping with configurable timeouts for automatic memory reclamation.

## 4. Contract Tests & Verification
- Test harness: `source/host/tests/openclaw_infra_contract.rs`
  - `test_ocrm_infra_update_startup_and_triage`
  - `test_ocrm_infra_platform_sandbox_and_zstd`
  - `test_ocrm_infra_worker_pool_lifecycle`
- Evidence: 3/3 passing contract tests with positive and negative cases.
