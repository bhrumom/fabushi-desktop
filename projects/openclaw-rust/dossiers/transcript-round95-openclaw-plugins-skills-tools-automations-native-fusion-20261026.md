# OCRM Technical Dossier: Round 95 — OpenClaw Plugins, Skills, Tools & Automations Native Fusion

## 1. Executive Summary

This dossier records the full-source native Rust equivalence and fusion for:
1. **`OCRM-M08`**: Plugins SDK, Hook Lifecycle, and Tool Capabilities (`packages/plugin-sdk/`, `src/plugins/`, `src/hooks/`)
2. **`OCRM-M09`**: Automations, Schedules, Workflows & Cron Triggers (`src/cron/`, `src/automations/`, `src/workflows/`)

All responsibilities from upstream `openclaw/openclaw` in `packages/plugin-sdk/`, `src/plugins/`, `src/hooks/`, and cron/automation modules are mapped natively into canonical Fabushi Rust architecture under `source/host/src/extensions/mcp/`, `source/host/src/runner/tools/`, `source/host/src/automations/`, and `source/host/src/workflows/`.

---

## 2. Upstream Source to Canonical Fabushi Rust Mapping

### 2.1 Plugins SDK, Skills & Tool Capabilities (`OCRM-M08`)

| Upstream Source Path (`openclaw`) | Upstream Responsibility | Target Fabushi Rust Symbol | Process & Boundary |
|---|---|---|---|
| `packages/plugin-sdk/manifest.ts` | Bundle manifest schema & capability declaration | `CatalogAuthorization`, `PackageError::Manifest` | Host Plugins / MCP |
| `src/plugins/active-runtime-registry.ts` | Active plugin discovery & skill record indexing | `PluginIdentifier`, `InstalledPlugin`, `PluginSkillsCache` | Host Plugins / MCP |
| `src/plugins/api-lifecycle.ts` | Plugin lifecycle timeout & state transitions | `EFFECTIVE_PLUGINS_RPC_TIMEOUT_MS`, `PluginLoadFailure` | Host Plugins / MCP |
| `src/plugins/bundle-mcp.ts` | MCP server resolution & subprocess coordination | `mcp_server_resolution`, `mcp_meta_tools`, `box_mcp_exec` | Host MCP Service |
| `src/hooks/wired-hooks-after-tool-call.ts` | Post-tool execution hook & state update dispatch | `SandAutoReviewController`, `SandAutoReviewResolution` | Host Runner / AutoReview |
| `src/plugins/web-search-providers.runtime.ts` | Web search provider dispatch & credential guard | `SandWebTool`, `sand_web_tools::execute_web_search` | Host Runner / Sand Tools |
| `src/plugins/web-fetch-providers.runtime.ts` | Web page extraction & content size limits | `SandWebTool`, `sand_web_tools::execute_web_fetch` | Host Runner / Sand Tools |
| `src/plugins/widget-presenters.ts` | Visual widget presenting & projection cards | `listener_connect_cards`, `SandSpotlightTools` | Host Runner / Presentation |

### 2.2 Automations, Workflows & Cron Schedules (`OCRM-M09`)

| Upstream Source Path (`openclaw`) | Upstream Responsibility | Target Fabushi Rust Symbol | Process & Boundary |
|---|---|---|---|
| `src/cron/cron-matcher.ts` | 5-field cron parsing & field boundary checks | `compile_cron_matcher`, `CronMatcher` | Host Automations |
| `src/cron/every-interval.ts` | Relative `@every` duration interval parser | `parse_every_interval_ms` (`1s`, `5m`, `2h`, `1d`) | Host Automations |
| `src/cron/timezone.ts` | Timezone override parsing (`CRON_TZ=`, `TZ=`) | `split_zone`, `chrono_tz::Tz` | Host Automations |
| `src/cron/next-run.ts` | Next run epoch calculation with search caps | `compute_next_run_at`, `MAX_CRON_SEARCH_MINUTES` | Host Automations |
| `src/cron/weekly-summary.ts` | 7-day schedule summary & weekend/overnight flags | `summarize_schedule_next_7_days`, `AutomationScheduleSummary` | Host Automations |
| `src/automations/store.ts` | Persistent automation metadata & state owner | `AutomationStore`, `Automation` | Host Automations Store |
| `src/automations/trigger.rs` | Event & schedule trigger firing lifecycle | `AutomationTrigger`, `AutomationTriggerState` | Host Automations Engine |
| `src/workflows/workflow-library.ts` | Workflow library indexing & parse cache | `WorkflowLibrary`, `StatKeyedParseCache` | Host Workflows |

---

## 3. Falsifiable Test Oracles

1. **`OCRM-PLUGIN-O01` (Plugin Manifest Schema)**: Validates that manifests lacking required fields or exceeding size bounds are rejected with `PackageError::Manifest`.
2. **`OCRM-PLUGIN-O02` (Plugin Skills Discovery)**: Validates that skill discovery registers valid skills and isolates load failures per plugin without crashing the host.
3. **`OCRM-PLUGIN-O03` (RPC Timeout Enforcement)**: Validates that plugin RPC dispatch enforces `EFFECTIVE_PLUGINS_RPC_TIMEOUT_MS` (15,000 ms).
4. **`OCRM-PLUGIN-O04` (MCP Server Dispatch)**: Validates that MCP tool execution delegates through `box_mcp_exec` with sandboxing guarantees.
5. **`OCRM-PLUGIN-O05` (Hook Tool Interceptor)**: Validates that hooks intercepting mutating tool calls trigger proper auto-review gating (`Enforce` / `Shadow` / `Off`).
6. **`OCRM-PLUGIN-O06` (Web Search Credential Guard)**: Validates that search requests without credentials or exceeding rate limits fail cleanly.
7. **`OCRM-CRON-O01` (Cron 5-Field Parsing)**: Validates standard cron patterns (`0 9 * * 1-5`, `*/15 * * * *`) and rejects invalid month/day ranges.
8. **`OCRM-CRON-O02` (@every Interval Parsing)**: Validates `@every 30s`, `@every 5m`, `@every 2h`, `@every 1d` and rejects negative or malformed intervals.
9. **`OCRM-CRON-O03` (Timezone Shift Computation)**: Validates `CRON_TZ=America/New_York` against UTC offsets across daylight saving boundaries.
10. **`OCRM-CRON-O04` (Next Run Epoch Monotonicity)**: Validates `compute_next_run_at` strictly yields future timestamps monotonically ahead of `after_ms`.
11. **`OCRM-CRON-O05` (7-Day Schedule Summary)**: Validates `summarize_schedule_next_7_days` accurate fire count and overnight/weekend boolean flags.
12. **`OCRM-CRON-O06` (Automation Store Persistence)**: Validates that automation updates maintain transactional consistency across host reboots.
