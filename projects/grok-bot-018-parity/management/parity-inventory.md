# Grok Bot 0.18 parity inventory

Reference: `bhrum/grok-bot-0.18-reconstructed@107877b4e2134fd167d239411386f09e42eadd6d`  
Target: `bhrumom/fabushi-desktop@refactor/grok-bot-018-parity-mac`

The row-by-row acceptance source is `parity-inventory.generated.json`. It contains every audited reachable reference renderer module, every audited runner capsule, and all 16 Electron production bindings, with target files, state and closure notes. This human file summarizes the current execution state; it must not be used to hide an open generated row.

## State semantics

- **PASS** — executable target behavior exists and has source/test evidence for the mapped reference behavior.
- **PARTIAL** — executable behavior exists, but the reference breadth/state depth/visual interaction is not closed. A8 remains open.
- **FAIL** — recoverable reference behavior has no sufficient target implementation. A8 remains open.
- **PRODUCT_DIFFERENCE** — only for the user-authorized product difference: reference cloud Box/VNC/cursor-agent infrastructure is replaced by the Mac where Fabushi is installed. User-visible semantics still require a local equivalent.
- **EXTERNAL_DEPENDENCY** — reference behavior depends on an external service/catalog/policy not contained in the pinned repository. The target must still expose a real provider seam; this is not permission to fabricate a catalog or a success state.

Current generated counts after the latest inventory refresh:

| Closure | PASS | PARTIAL | FAIL | PRODUCT_DIFFERENCE |
|---|---:|---:|---:|---:|
| renderer modules (275) | 17 | 196 | 61 | 1 |
| runner capsules (70) | 10 | 58 | 0 | 2 |
| Electron bindings (16) | 9 | 7 | 0 | 0 |

A8 is therefore **NOT COMPLETE**.

## Reference evidence anchors

- `manifests/reconstruction/renderer-closure.json`: 308 clean renderer modules, 275 reachable, 102 valid/reachable UI anchors, 163 evidenced IPC claims.
- `manifests/reconstruction/runner-parity-audit.json`: 70 runner capsules audited; 66 directly behavior-tested.
- `manifests/reconstruction/electron-main-production-bindings-manifest.json`: 16 production bindings.
- The reference report itself records 703 JSX-runtime candidates that are not linked to reviewed first-party evidence. Those are not silently promoted to recoverable product modules.

## Current source-level mapping

| Area | Reference | Target | Current state / evidence |
|---|---|---|---|
| production shell | production renderer/bootstrap | `desktop/src/grok-app.tsx`, `main.tsx` | production entry is Grok-style only; legacy contacts/Telegram/payment/MiniApp/Mahayana are not mounted |
| coordinator / host / exec | node coordinator + host runner + agent-exec | `grok-agent-coordinator.cjs` → `grok-agent-runner.cjs` → `grok-host-runtime.cjs` → execution resources/local executor | per-agent run generation/interrupt/quiesce ownership plus independent host/tool execution; not a CLI wrapper |
| local Computer | Computer shell/use + auto review | `local-tool-executor.cjs`, `MacComputerHelper.swift`, Computer renderer shell | screenshot/click/move/drag/scroll/type/key/wait, screenshot stateId recheck, permission/approval and expandable local screen |
| browser | browser tools + review | `grok-local-browser.cjs` | snapshot/stateId/navigate/click/type/key/scroll/screenshot; exact reference subagent/audit breadth remains |
| approvals / cancellation | permission request + runner abort | coordinator/host + renderer tool cards | always/ask/never, allow/deny/cancel, Stop, queued/waiting/running/streaming/done/error/cancelled |
| SendMessage / reactions | send-message schema/tool, reaction tool | `grok-communication-tools.cjs`, coordinator, renderer | text/attachment/widget/secret-request; secure secret path; reply/reaction state; cloud cursor-agent intentionally excluded |
| conversation | workspace/reply/outline/find | app + message interactions + outline | quoted reply context, reactions, outline, in-chat find, cross-workspace message/file/link palette search |
| attachments | attachment gateway + transcript attachments | `grok-attachment-gateway.cjs` + renderer | registered local attachments/previews plus HTTPS handoff; production `sand-media://` Range streaming is wired for media; specialist PDF/spreadsheet viewer parity remains |
| transient stream / observations | stream attempt, retry, turn usage/observation | retry, turn usage, host observation, action audit | bounded retry-before-output, retry-after/backoff, token usage, turn/tool/retry audit; resumable stream/TTFT/await-stall breadth remains |
| memory / state | sand-memory + update_state | memory store + state tool | durable user/agent/project memory, write/forget, routines, workflows, profile/settings mutation; remaining state variants open |
| MCP | MCP exec/manager/OAuth | MCP manager/OAuth + coordinator/host/UI | stdio + Streamable HTTP, tools/list/call, server/tool enable, OAuth discovery/dynamic registration/PKCE/refresh, multi-account secrets |
| Plugins / private skills | provider/browser/server tools/private skills | plugin marketplace/provider seam + MCP + workflow manager | visible installable items require real MCP/private-skill materialization; no fake GitHub/Memory rows |
| account | account OAuth/session | `grok-account-session.cjs` + UI | real PKCE login/cancel/logout/status/name; reference access/subscription policy remains external/open |
| workflows | private SKILL/workflow surfaces | `grok-workflow-manager.cjs` | file-backed SKILL.md, CRUD, enable/disable and prompt injection |
| routines | automation schedule/history | coordinator + schedule parser + UI | cron CRUD/enable/pause/run-now/history; listener/event-trigger breadth remains |
| About / Feedback / deep links | recovered overlays | desktop services + main/preload + app | real version/update track; real configured feedback HTTP provider; `sand://app/v1/info?topic=deep-links` |
| updater | update service/status/settings | desktop services + Electron autoUpdater | real feed/check/status/download/install/track; minimum-version/policy semantics remain |
| onboarding | signed-in onboarding/computer readiness | app + desktop preferences | meet → local Computer demo → jobs → tools → create → hand-off; exact access/readiness semantics remain |
| action audit | action audit/site visit/bot block | `grok-action-audit.cjs` + host | scrubbed tool audit, query-free navigation host tracking and block signatures; exact MCP transport/CDP probe remains |
| secure storage | secureStorage binding | `grok-secret-store.cjs` | Electron safeStorage-backed secrets; secret-request rejects insecure storage |
| notifications | notification binding | main + coordinator | native Electron Notification path exists |
| experiments | experiments production binding/runtime | `grok-experiments.cjs` + account/coordinator IPC | authenticated provider seam, cache, dynamic configs, env gates and dev overrides; exact reference Statsig/bootstrap behavior remains PARTIAL |
| cloud Box/VNC/cursor-agent | cloud-only reference infrastructure | installed-Mac Computer | explicit PRODUCT_DIFFERENCE; no cloud provisioning is reintroduced |

## Known A8 blockers

The generated module inventory remains authoritative. The largest open families are: exact sidebar/workspace/composer visual and state breadth; specialist attachment/media/PDF/spreadsheet viewers; Computer teach/recording and remaining shell states; access/roster/reconnect/shared-room/channel surfaces; exact window chrome/notification states; exact experiments Statsig/bootstrap semantics; event-listener routines; exact auto-review/tool escalation and request-box-help semantics; resumable stream/checkpoint/TTFT/await-stall observation; remaining `update_state` variants; reference-specific MCP management/meta-tool breadth; and exact claim-for-claim renderer IPC closure.

External reference services are not fabricated. In particular, the reference Marketplace production catalog is supplied by an external DashboardService rather than by self-contained catalog data at the pinned commit. The target keeps a real provider seam and refuses to show installable entries that have no executable backend.

## CI evidence

- Historical package mechanism: Run `35323112810` succeeded; artifact `10538065159`, digest `sha256:884a992b9cfa2cf16891ddee05939a5daa52bc2343f8ac92e6fa0c30d9d4adae`. This is not final release evidence.
- Current audited source head `910f65e42b0dd096e9c969eba03666f46ae764c9` passed Run `35360378372`: CommonJS syntax, runtime contract tests, dependency install, and TypeScript/Vite build all succeeded.

## A8 rule

Do not run final macOS packaging, merge PR #1, or publish a prerelease while any recoverable row is PARTIAL/FAIL. Packaging happens only after the generated inventory and source audit make A8 genuinely PASS.
