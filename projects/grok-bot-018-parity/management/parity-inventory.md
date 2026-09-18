# Grok Bot 0.18 parity inventory

Reference: `bhrum/grok-bot-0.18-reconstructed@107877b4e2134fd167d239411386f09e42eadd6d`
Target: `bhrumom/fabushi-desktop@refactor/grok-bot-018-parity-mac`

This inventory is the acceptance source for A8. A row is PASS only when the target has an executable implementation and objective evidence. A visible UI catalog item without an executable backend is a FAIL, not partial parity.

Reference evidence anchors:
- `manifests/reconstruction/renderer-closure.json`: 308 clean renderer modules, 275 reachable, 11 shipped routes, 20 clean feature families, 102 valid/reachable UI anchors, 163 evidenced IPC claims.
- `manifests/reconstruction/electron-main-production-bindings-manifest.json`: secure storage, settings, attachment gateway, main RPC, updater, media protocol, account OAuth, experiments, MCP OAuth, telemetry, notifications, coordinator, IPC, startup, URL policy and failure reporting production bindings.
- `manifests/reconstruction/runner-parity-audit.json`: 70 runner capsules audited, 76 clean runner modules, 340 source import edges, 66 directly behavior-tested modules.
- Reference caveat: 703 JSX-runtime candidates are explicitly unlinked to reviewed first-party evidence by the reference's own renderer-closure report. They are not counted as recoverable product modules unless later evidence classifies them.

| Area | Reference module/evidence | Target implementation | State | Objective evidence / next closure |
|---|---|---|---|---|
| Production shell | `frontend/src/production/ProductionRenderer.tsx`, bootstrap, production CSS | `desktop/src/grok-app.tsx`, `main.tsx` | PARTIAL | Single Grok-style shell exists; not yet component/route parity |
| Agent list/sidebar | sidebar model, sidebar.tsx, preview/status/layout/sections | monolithic Sidebar in `grok-app.tsx` | PARTIAL | basic selection/search only; missing row actions, section state, preview semantics |
| Agent rename/delete | AgentNameEditor, AgentDeleteConfirmation, AgentRowActions | backend methods exist; no production UI flow | FAIL | add reference-aligned row actions/editor/confirmation |
| Command palette/shortcuts | CommandPalette + command providers + global-keyboard-shortcuts | visual `⌘K` hint only | FAIL | no keyboard handler / palette / command providers |
| Conversation workspace | workspace/* incl chat header, transcript, pagination, reply, outline | basic Workspace/Message | PARTIAL | missing reply/thread/pagination/outline/find/rich content |
| Composer | workspace/composer, rich-text-editor, suggestions, references, voice | textarea + inert @/Auto controls | FAIL | implement functional references/suggestions/model/attachments/stop states |
| Attachments/media | attachment gateway; transcript-card attachment; media/pdf/spreadsheet viewers | pick-file inserts local path text | FAIL | no attachment object lifecycle/viewers/gateway |
| Tool transcript | conversation tool-results + transcript-card tool views | generic tool row | PARTIAL | add queued/approval/running/done/error/cancelled lifecycle and structured result |
| Reactions/message actions | transcript-card reactions/actions | none | FAIL | no backend or UI |
| Notices/permission cards | permission-request, notice, auto-review approval | none | FAIL | approval card + IPC + lifecycle required |
| Agent lifecycle | host runner/state + coordinator | `grok-agent-runtime.cjs` thinking/running/idle/error | PARTIAL | no coordinator/host split, waiting approval semantics incomplete |
| Coordinator | `source/node-agent-coordinator`, Electron coordinator binding | none as independent owner | FAIL | introduce coordinator boundary with request/event protocol |
| Host runtime | `source/host`, runner composition/extensions | one local runtime module | FAIL | separate host/turn/tool/extension ownership |
| Exec resource model | agent-exec resource-provider/remote/controlled/stream resources | direct function calls | FAIL | resource registry/controlled execution semantics absent |
| Local files | agent tool execution | list/read | PARTIAL | mutation/search/metadata/error/cancel policy not matched |
| Foreground shell | shell exec/stream | `run_terminal` one-shot execFile | PARTIAL | no stream/cancel/process lifecycle transcript |
| Background shell | `agent-exec/background-shell.ts` | none | FAIL | spawn/write/status/kill required |
| Request context | `agent-exec/request-context.ts` | none | FAIL | execution context propagation absent |
| Subagents | `agent-exec/subagent.ts`, subagent runtime/registry | none | FAIL | create/resume/follow-up/interrupt/background result absent |
| Computer | computer overlay/shell/VNC/teach-recording; computer-use subagent | no Computer execution surface | FAIL | local Mac screenshot/input execution and state required |
| Browser | browser-use subagent + browser auto-review | only open external HTTPS URL | FAIL | no inspect/snapshot/action/state/recheck model |
| Local tool permission | permissions/local-tool store/view | prompt text only | FAIL | always/ask/never + ceiling + approvals/clear required |
| Auto review | computer/browser auto-review + classifier/approval | none | FAIL | mutating action review/approval/recheck/cancel required |
| Cancellation | runner/exec abort semantics | top-level AbortController | PARTIAL | child tool processes not reliably cancelled; approval/tool states missing |
| Error semantics | runner + error boundary/transcript errors | assistant error string | PARTIAL | structured execution/tool/UI failure states missing |
| MCP exec | `source/packages/agent-exec/mcp.ts`, routed MCP bridge | none | FAIL | no server discovery/tools/list/tools/call |
| MCP desktop lifecycle | Electron MCP OAuth adapter/runtime/manager | none | FAIL | no manager, host refresh/sync, desktop IPC |
| OAuth/accounts | account OAuth adapter/session UI | none | FAIL | login/cancel/logout/status/token/account lifecycle absent |
| Plugin browser | plugins overlay browser/view/model | basic static catalog | FAIL | catalog is not reference provider model |
| Plugin server tools | plugins/server-tools loading/toggle/retry | none | FAIL | no server tool list/disable state |
| Plugin auth | plugins github-auth + production-adapter | none | FAIL | no auth controller/account scoped sync |
| Private skills | host/agent skills surfaces | none | FAIL | no real skill provider/execute chain |
| Workflows | reference workflow/agent capabilities | none | FAIL | no executable workflow provider |
| Marketplace metadata | plugin overlay/provider metadata | hard-coded 5 rows | FAIL | no provider-backed metadata; placeholder GitHub/Memory forbidden |
| Automations | automations/routines schedule/run history/trigger schema | none | FAIL | no schedule editor/runtime/run history |
| Settings | settings overlay panels/computer/auto-review/updates | static 3-row overlay | FAIL | no backed settings controls or panels |
| Account/settings notices | account session + settings notice controller | none | FAIL | missing |
| Computer settings | settings computer view | static Local label | FAIL | no runtime state/configuration |
| Updates | update required/status/settings updater | workflow only | FAIL | app-side update state/UI absent |
| Onboarding | signed-in onboarding/computer readiness/suggestions | none | FAIL | missing |
| Access/reconnect/roster | access cover, roster readiness/privacy/reconnect/status | none | FAIL | missing |
| Agent info | avatar/settings/async tasks/channels/shared room/group members | none | FAIL | recoverable reference UI not mapped yet |
| Hidden chats | hidden-chats overlay | none | FAIL | missing |
| Deep links | deep-links overlay/model | none | FAIL | missing |
| Feedback/About | feedback and about overlays | none | FAIL | missing |
| Org chart | org-chart workspace | none | FAIL | missing |
| Window chrome | alerts/notification host/status badge/workspace indicator | Electron window only | FAIL | missing reference states |
| Secure storage | Electron production secureStorage binding | none | FAIL | secrets currently env-only |
| Settings persistence | Electron settings binding | JSON agent state only | PARTIAL | no typed settings service |
| Attachment gateway | Electron attachmentGateway binding | file picker only | FAIL | missing |
| Main RPC | Electron mainRpc binding | ad-hoc IPC handlers | PARTIAL | not reference request/event contract |
| Account OAuth binding | Electron accountOAuth binding | none | FAIL | missing |
| MCP OAuth binding | Electron mcpOAuth binding | none | FAIL | missing |
| Notifications | Electron notifications binding | none | FAIL | missing |
| Telemetry/failure reports | telemetry/reportFailure binding | none | FAIL | missing (may be product-policy configurable but needs explicit parity decision) |
| URL policy | parseAllowedExternalUrl | HTTPS-only browser open | PARTIAL | narrower ad-hoc policy |
| Startup/lifecycle | startup binding + root resilience | single-instance/window | PARTIAL | no service readiness/reconnect state |
| Renderer IPC claims | 163 evidenced claims | <20 ad-hoc methods/events | FAIL | enumerate and close by feature; no broad PASS allowed |
| Fabushi-only contacts/Telegram/payment/MiniApp/Mahayana | absent from Grok production UI | not mounted by `main.tsx` | PASS | source review + prior package evidence |
| Local-computer product difference | reference Box/Computer semantics | local Mac stated | PARTIAL | architecture intent PASS; real Computer execution still open |

## A8 rule

A8 remains NOT COMPLETE while any recoverable required row is FAIL/PARTIAL. Rows may be marked N/A only with a reference-backed reason showing that the reference itself does not classify the item as recoverable product behavior. The 703 unlinked JSX-runtime candidates are the only currently documented reference-level caveat and do not waive any of the evidenced modules/routes/IPC claims above.
