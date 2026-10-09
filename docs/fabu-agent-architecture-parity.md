# Fabu Agent architecture parity

Reference repository: `bhrumom/fabu@main`

This document records the source-level Agent architecture used as the refactor
reference for Fabushi. It is deliberately organized by ownership boundaries,
not UI resemblance.

| Area | Fabu reference | Fabu ownership rule | Fabushi target |
| --- | --- | --- | --- |
| Bot vs Agent identity | `source/host/agents/agent-profile.ts`, `source/host/storage/agent-paths.ts` | Presentation/profile identity is separate from runtime Agent identity | `BotSummary.agentId`, explicit `contact` vs `bot` peers, `desktop/src/fabu-runtime/agent-domain.ts` |
| Agent profile/settings | `source/host/agents/agent-profile.ts` | `profile.json` and `settings.json` are per Agent | `FabuAgentStore.writeProfile/writeSettings` |
| Agent durable state | `source/packages/agent-kv/agent-store.ts` | Immutable content-addressed blobs + mutable latest-root metadata | Platform D1/R2 Agent Store and renderer etag facade |
| Local transcript DB | `source/host/extensions/session/agent-db-schema.ts` | Per-Agent transcript/index database | Mahayana provider-neutral transcript plus per-conversation kernel session |
| Per-turn runner | `source/host/runner/production-turn-agent-owner.ts`, `turn-run-shell.ts` | One invocation creates one Agent context/session pair; caller owns lifecycle | One kernel session per Bot/Agent conversation; Agent state is session-scoped |
| Prompt assembly | `source/host/runner/system-prompt-assembly.ts` | Profile/memory/workflows/channels are independent sections with snapshots | Heavy MCP/memory/workflow assembly bypassed for lightweight turns; normal turns remain capability-aware |
| Context compaction | `source/packages/agent-summarization/*` | Durable transcript is full; inference context is bounded/summarized | `project_model_history` keeps full transcript durable but bounds model input |
| Memory | `source/host/runner/turn-memory.ts` | Memory maintenance never blocks/fails the visible turn | Session-owned memory; persistent FeatureHost memory remains account/Agent scoped |
| Submission journal | `frontend/.../workspace/submission.ts` | Transport pending/queued/sent is separate from Agent generation | `desktop/src/fabu-runtime/submission-queue.ts` |
| Optimistic acknowledgement | `frontend/.../workspace/acknowledgement.ts` | Pending/dispatching/accepted-awaiting-echo/failed are explicit | User message is optimistic until Host acceptance; accepted state no longer stays “发送中” |
| First-token watchdog | `source/host/runner/stream-attempt.ts` | No-output stall has its own deadline, cancellation and bounded retry | Dacheng/DeepSeek first-output deadline + one compact-context retry |
| Tool execution | `source/host/runner/turn-agent-composition.ts`, `turn-run-shell.ts` | Tool executor belongs to the current turn; middleware wraps it | Native Mahayana Agent tool loop + low-latency no-tool conversation lane |
| Progress acknowledgement | `start-of-turn-ack-reminder-middleware.ts`, `send-message-reminder-middleware.ts` | Long work must acknowledge/progress instead of looking frozen | Local-first assistant turn + ordered Agent activity events; follow-up: explicit progress policy |
| Subagents | `source/host/runner/subagent-runtime.ts` | Parent/child lineage, background registry, independent status/usage | Mahayana scheduler exists; follow-up: move scheduler ownership from engine-global to Agent session |
| Approvals | `source/host/runner/auto-review-gate.ts` | Approval mode is Agent policy, side effects do not overlap | Mahayana permission ledger + approval ledger; follow-up: bind approval mode to per-Agent settings |
| Settlement/checkpoints | `source/host/runner/turn-settle.ts` | Step/final checkpoints are durable; memory runs after visible turn | Native session snapshots and platform Agent Store; follow-up: content-address every runtime checkpoint |
| Multi-Agent concurrency | `workspace/submission.ts` active-by-agent map | One active submission per Agent, different Agents may run independently | Submission queue is per Agent; follow-up: replace legacy renderer-global operation pointer with per-Agent operation registry |

## Latency failure that motivated this refactor

The running macOS build showed short “你好” turns taking roughly 56 s, 77 s,
110 s and 266 s while producing only a few words. The old runtime fed the
entire `session.history` to every model request and synchronously assembled MCP,
memory and workflow context before dispatch. Tool-bearing backend streams could
also wait without a dedicated first-token deadline.

The new boundary is the same one used by Fabu: keep all state durable, but make
the active inference context bounded and turn-specific, isolate Agent sessions,
and treat first-token stalls as retryable transport failures instead of normal
“thinking” time.
